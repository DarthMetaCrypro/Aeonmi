#![cfg_attr(test, allow(dead_code, unused_variables))]

use std::fs;
use std::io;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};

use crate::cli::EmitKind;
use crate::commands::compile::compile_pipeline;
use crate::core::qpoly::QPolyMap;

// ---------- Palette / Theme ----------
fn neon() -> (Color, Color, Color, Color) {
    (
        Color::Rgb(225, 0, 180),
        Color::Rgb(130, 0, 200),
        Color::Rgb(255, 240, 0),
        Color::Rgb(190, 190, 200),
    )
}

// ---------- Basic Types ----------
#[derive(Copy, Clone)]
enum EmitMode {
    Js,
    Ai,
}
impl EmitMode {
    fn toggle(self) -> Self {
        match self {
            EmitMode::Js => EmitMode::Ai,
            EmitMode::Ai => EmitMode::Js,
        }
    }
    fn label(self) -> &'static str {
        match self {
            EmitMode::Js => "JS",
            EmitMode::Ai => "AI",
        }
    }
    #[allow(dead_code)]
    fn to_emit_kind(self) -> EmitKind {
        match self {
            EmitMode::Js => EmitKind::Js,
            EmitMode::Ai => EmitKind::Ai,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum EditorMode {
    Append,
    Insert,
}

#[derive(Copy, Clone, Debug)]
enum ButtonAction {
    Save,
    Compile,
    Run,
    ToggleEmit,
    ToggleMode,
    Search,
    ToggleMouse,
    Quit,
}

struct ButtonSpec {
    area: Rect,
    label: String,
    action: ButtonAction,
}

struct App {
    filepath: PathBuf,
    buffer: String,
    input: String,
    dirty: bool,
    status: String,
    last_status_at: Instant,
    diagnostics: Vec<String>,
    qpoly: QPolyMap,
    emit_mode: EmitMode,
    show_key_debug: bool,
    last_key_debug: String,
    mouse_capture: bool,
    cursor_row: usize,
    cursor_col: usize,
    scroll: usize,
    mode: EditorMode,
    history: Vec<String>,
    history_index: usize,
    last_snapshot_at: Instant,
    chars_since_snapshot: usize,
    paste_active: bool,
    last_key_time: Instant,
    search_active: bool,
    search_query: String,
    last_match_row: Option<usize>,
    search_matches: Vec<usize>,
    search_index: usize,
}

// ---------- App Impl ----------
impl App {
    fn new(filepath: PathBuf, qpoly: QPolyMap) -> Self {
        let buffer = if filepath.exists() {
            fs::read_to_string(&filepath).unwrap_or_default()
        } else {
            String::new()
        };
        let persisted_search = fs::read_to_string(".aeonmi_last_search").ok().unwrap_or_default();
        Self {
            filepath,
            buffer,
            input: String::new(),
            dirty: false,
            status: "⏎ append • Ctrl+S save • F4 emit=JS/AI • F5 compile • F6 run(JS) • F9 toggle-mouse • Esc/Ctrl+Q quit • F1 key-debug".into(),
            last_status_at: Instant::now(),
            diagnostics: vec![],
            qpoly,
            emit_mode: EmitMode::Js,
            show_key_debug: false,
            last_key_debug: String::new(),
            mouse_capture: true,
            cursor_row: 0,
            cursor_col: 0,
            scroll: 0,
            mode: EditorMode::Append,
            history: Vec::new(),
            history_index: 0,
            last_snapshot_at: Instant::now(),
            chars_since_snapshot: 0,
            paste_active: false,
            last_key_time: Instant::now(),
            search_active: false,
            search_query: persisted_search.trim().to_string(),
            last_match_row: None,
            search_matches: Vec::new(),
            search_index: 0,
        }
    }

    fn set_status(&mut self, s: impl Into<String>) {
        self.status = s.into();
        self.last_status_at = Instant::now();
    }

    fn snapshot(&mut self) {
        if self.history_index < self.history.len() {
            self.history.truncate(self.history_index);
        }
        self.history.push(self.buffer.clone());
        self.history_index = self.history.len();
        if self.history.len() > 200 {
            let drop = self.history.len() - 200;
            self.history.drain(0..drop);
            self.history_index = self.history.len();
        }
        self.last_snapshot_at = Instant::now();
        self.chars_since_snapshot = 0;
    }

    fn undo(&mut self) {
        if self.history_index == 0 {
            return;
        }
        self.history_index -= 1;
        if let Some(s) = self.history.get(self.history_index) {
            self.buffer = s.clone();
            self.set_status("Undo");
            self.dirty = true;
        }
    }

    fn redo(&mut self) {
        if self.history_index >= self.history.len() {
            return;
        }
        self.history_index += 1;
        if self.history_index > 0 {
            if let Some(s) = self.history.get(self.history_index - 1) {
                self.buffer = s.clone();
                self.set_status("Redo");
                self.dirty = true;
            }
        }
    }

    fn find_next(&mut self) {
        if self.search_query.is_empty() { return; }
        if self.search_matches.is_empty() { self.rebuild_search_matches(); }
        if self.search_matches.is_empty() { self.set_status("No match"); return; }
        self.search_index = (self.search_index + 1) % self.search_matches.len();
        let r = self.search_matches[self.search_index];
        self.last_match_row = Some(r);
        self.cursor_row = r;
        self.cursor_col = 0;
        self.set_status(format!("Search: {}/{}", self.search_index + 1, self.search_matches.len()));
    }

    fn find_prev(&mut self) {
        if self.search_query.is_empty() { return; }
        if self.search_matches.is_empty() { self.rebuild_search_matches(); }
        if self.search_matches.is_empty() { self.set_status("No match"); return; }
        if self.search_index == 0 { self.search_index = self.search_matches.len()-1; } else { self.search_index -= 1; }
        let r = self.search_matches[self.search_index];
        self.last_match_row = Some(r);
        self.cursor_row = r;
        self.cursor_col = 0;
        self.set_status(format!("Search: {}/{}", self.search_index + 1, self.search_matches.len()));
    }

    fn rebuild_search_matches(&mut self) {
        self.search_matches.clear();
        self.search_index = 0;
        if self.search_query.is_empty() { return; }
        let q = self.search_query.to_lowercase();
        for (i, l) in self.buffer.lines().enumerate() {
            if l.to_lowercase().contains(&q) { self.search_matches.push(i); }
        }
        if !self.search_matches.is_empty() {
            self.last_match_row = Some(self.search_matches[0]);
            self.cursor_row = self.search_matches[0];
            self.cursor_col = 0;
        }
    }

    fn add_line(&mut self) {
        let transformed = self.qpoly.apply_line(&self.input);
        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }
        self.buffer.push_str(&transformed);
        self.input.clear();
        self.dirty = true;
        self.set_status("Line added.");
        let lc = self.buffer.lines().count();
        if lc > 0 {
            self.cursor_row = lc - 1;
        }
        self.cursor_col = self.buffer.lines().last().map(|l| l.len()).unwrap_or(0);
    }

    fn save(&mut self) -> Result<()> {
        fs::write(&self.filepath, &self.buffer)?;
        self.dirty = false;
        self.set_status(format!("Saved {}", self.filepath.display()));
        Ok(())
    }

    fn compile(&mut self, pretty: bool, skip_sema: bool) {
        if self.dirty {
            if let Err(e) = self.save() {
                self.set_status(format!("Save failed: {e}"));
                return;
            }
        }
        match self.emit_mode {
            EmitMode::Ai => {
                let out = PathBuf::from("output.ai");
                match compile_pipeline(
                    Some(self.filepath.clone()),
                    EmitKind::Ai,
                    out.clone(),
                    false,
                    false,
                    pretty,
                    skip_sema,
                    false,
                ) {
                    Ok(()) => self.set_status(format!("Wrote → {}", out.display())),
                    Err(e) => self.set_status(format!("Emit .ai error: {e}")),
                }
            }
            EmitMode::Js => {
                let out = PathBuf::from("output.js");
                match compile_pipeline(
                    Some(self.filepath.clone()),
                    EmitKind::Js,
                    out.clone(),
                    false,
                    false,
                    pretty,
                    skip_sema,
                    false,
                ) {
                    Ok(()) => self.set_status(format!("Compiled → {}", out.display())),
                    Err(e) => self.set_status(format!("Compile error: {e}")),
                }
            }
        }
    }

    fn run(&mut self, pretty: bool, skip_sema: bool) {
        if !matches!(self.emit_mode, EmitMode::Js) {
            self.set_status("Switch to JS (F4) to run here, or use CLI `run`.");
            return;
        }
        if self.dirty {
            if let Err(e) = self.save() {
                self.set_status(format!("Save failed: {e}"));
                return;
            }
        }
        let out = PathBuf::from("aeonmi.run.js");
        match compile_pipeline(
            Some(self.filepath.clone()),
            EmitKind::Js,
            out.clone(),
            false,
            false,
            pretty,
            skip_sema,
            false,
        ) {
            Ok(()) => match std::process::Command::new("node").arg(&out).status() {
                Ok(s) if !s.success() => self.set_status(format!("node exited with {s}")),
                Err(e) => self.set_status(format!("Node not available: {e}")),
                _ => self.set_status("Run OK."),
            },
            Err(e) => self.set_status(format!("Compile error: {e}")),
        }
    }
}

// ---------- Cheatsheet ----------
fn cheatsheet_from_map(_m: &QPolyMap) -> Vec<(String, String)> {
    [
        ("->", "→"),
        ("<-", "←"),
        ("<=", "≤"),
        (">=", "≥"),
        ("!=", "≠"),
        ("==", "＝"),
        ("<=>", "⇔"),
        ("::", "∷"),
        (":=", "≔"),
        ("|0>", "∣0⟩"),
        ("|1>", "∣1⟩"),
    ]
    .into_iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect()
}

// ---------- Entry Point ----------
pub fn run_editor_tui(
    file: Option<PathBuf>,
    config_path: Option<PathBuf>,
    pretty: bool,
    skip_sema: bool,
) -> Result<()> {
    let filepath = file.unwrap_or_else(|| PathBuf::from("untitled.ai"));
    let map = if let Some(p) = config_path {
        if p.exists() {
            QPolyMap::from_toml_file(&p).unwrap_or_else(|e| {
                eprintln!("(warn) failed to load {}: {e}", p.display());
                QPolyMap::from_user_default_or_builtin()
            })
        } else {
            eprintln!("(warn) config path not found: {}", p.display());
            QPolyMap::from_user_default_or_builtin()
        }
    } else {
        QPolyMap::from_user_default_or_builtin()
    };

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, SetTitle("Aeonmi Shard"))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(filepath, map);
    let res = panic::catch_unwind(AssertUnwindSafe(|| run_app(&mut terminal, app, pretty, skip_sema)));

    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        SetTitle("Aeonmi")
    )?;
    terminal.show_cursor()?;

    match res {
        Ok(inner) => inner,
        Err(panic_payload) => {
            let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "<non-string panic>".into()
            };
            let _ = std::fs::write(
                "tui_crash.log",
                format!(
                    "Editor panic captured at {:?}:\n{}\n",
                    std::time::SystemTime::now(),
                    msg
                ),
            );
            anyhow::bail!(
                "Editor crashed (panic captured). See tui_crash.log for details."
            )
        }
    }
}

// ---------- Event Loop ----------
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
    pretty: bool,
    skip_sema: bool,
) -> Result<()> {
    let tick_rate = Duration::from_millis(100);

    'outer: loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind,
                    ..
                }) => {
                    let accept = matches!(kind, KeyEventKind::Press)
                        || (matches!(kind, KeyEventKind::Repeat)
                            && matches!(code, KeyCode::Enter | KeyCode::Backspace | KeyCode::Tab));
                    if !accept {
                        continue;
                    }

                    if app.show_key_debug {
                        app.last_key_debug =
                            format!("key: {:?} mods: {:?} kind: {:?}", code, modifiers, kind);
                        app.set_status(app.last_key_debug.clone());
                    }

                    match (code, modifiers) {
                        (KeyCode::F(1), _) => {
                            app.show_key_debug = !app.show_key_debug;
                            app.set_status(if app.show_key_debug {
                                "Key debug ON"
                            } else {
                                "Key debug OFF"
                            });
                        }
                        (KeyCode::F(2), _) => {
                            app.mode = match app.mode {
                                EditorMode::Append => EditorMode::Insert,
                                EditorMode::Insert => EditorMode::Append,
                            };
                            app.set_status(match app.mode {
                                EditorMode::Append => "Mode: Append (Enter appends line)",
                                EditorMode::Insert => "Mode: Insert (editing buffer)",
                            });
                        }
                        (KeyCode::F(4), _) => {
                            app.emit_mode = app.emit_mode.toggle();
                            app.set_status(format!("Emit → {}", app.emit_mode.label()));
                        }
                        (KeyCode::F(9), _) => {
                            app.mouse_capture = !app.mouse_capture;
                            if app.mouse_capture {
                                let _ = execute!(std::io::stdout(), EnableMouseCapture);
                                app.set_status("Mouse capture ON (F9 to release)");
                            } else {
                                let _ = execute!(std::io::stdout(), DisableMouseCapture);
                                app.set_status("Mouse capture OFF (F9 to recapture)");
                            }
                        }
                        (KeyCode::F(5), _) => app.compile(pretty, skip_sema),
                        (KeyCode::F(6), _) => app.run(pretty, skip_sema),
                        (KeyCode::F(3), _) => {
                            if app.search_active {
                                app.find_next();
                            }
                        }
                        (KeyCode::Char('n'), KeyModifiers::NONE) => {
                            if app.search_active { app.find_next(); }
                        }
                        (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
                            if app.search_active { app.find_prev(); }
                        }
                        (KeyCode::Esc, _) => {
                            if app.search_active {
                                app.search_active = false;
                                app.search_matches.clear();
                                app.set_status("Search canceled");
                                continue;
                            }
                            if app.dirty {
                                app.set_status(
                                    "Unsaved changes — Ctrl+S to save, Esc again to quit.",
                                );
                                app.dirty = false; // one-shot warning
                                continue;
                            }
                            break;
                        }
                        (KeyCode::Char('c'), KeyModifiers::CONTROL)
                        | (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                            if app.search_active {
                                app.search_active = false;
                                app.set_status("Search canceled");
                                continue;
                            }
                            break;
                        }
                        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                            if let Err(e) = app.save() {
                                app.set_status(format!("Save failed: {e}"));
                            }
                        }
                        (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                            app.search_active = true;
                            app.search_query.clear();
                            app.search_matches.clear();
                            app.set_status("Search: (type) Enter/ n next, Shift+N prev, Esc cancel");
                        }
                        (KeyCode::Char('z'), KeyModifiers::CONTROL) => app.undo(),
                        (KeyCode::Char('y'), KeyModifiers::CONTROL) => app.redo(),
                        (KeyCode::Backspace, _) => {
                            if app.search_active {
                                app.search_query.pop();
                                app.search_matches.clear();
                                if app.search_query.is_empty() { app.set_status("Search: (empty)"); } else { app.set_status(format!("Search: {}", app.search_query)); }
                                continue;
                            }
                            match app.mode {
                                EditorMode::Append => {
                                    app.input.pop();
                                }
                                EditorMode::Insert => {
                                    let mut lines: Vec<String> =
                                        app.buffer.lines().map(|s| s.to_string()).collect();
                                    if app.cursor_row < lines.len() {
                                        if app.cursor_col > 0 {
                                            lines[app.cursor_row].remove(app.cursor_col - 1);
                                            app.cursor_col -= 1;
                                            app.dirty = true;
                                        } else if app.cursor_row > 0 {
                                            let prev_len = lines[app.cursor_row - 1].len();
                                            let current = lines.remove(app.cursor_row);
                                            lines[app.cursor_row - 1].push_str(&current);
                                            app.cursor_row -= 1;
                                            app.cursor_col = prev_len;
                                            app.dirty = true;
                                        }
                                        app.buffer = lines.join("\n");
                                    }
                                }
                            }
                        }
                        (KeyCode::Enter, _) => {
                            if app.search_active {
                                app.find_next();
                                continue;
                            }
                            match app.mode {
                                EditorMode::Append => app.add_line(),
                                EditorMode::Insert => {
                                    let mut lines: Vec<String> =
                                        app.buffer.lines().map(|s| s.to_string()).collect();
                                    if lines.is_empty() {
                                        lines.push(String::new());
                                    }
                                    if app.cursor_row >= lines.len() {
                                        app.cursor_row = lines.len() - 1;
                                    }
                                    let (left, right) = if app.cursor_row < lines.len() {
                                        let line = &lines[app.cursor_row];
                                        let (l, r) = line
                                            .split_at(app.cursor_col.min(line.len()));
                                        (l.to_string(), r.to_string())
                                    } else {
                                        (String::new(), String::new())
                                    };
                                    lines[app.cursor_row] = left;
                                    lines.insert(app.cursor_row + 1, right);
                                    app.cursor_row += 1;
                                    app.cursor_col = 0;
                                    app.buffer = lines.join("\n");
                                    app.dirty = true;
                                }
                            }
                        }
                        (KeyCode::Tab, _) => match app.mode {
                            EditorMode::Append => app.input.push_str("    "),
                            EditorMode::Insert => {
                                let mut lines: Vec<String> =
                                    app.buffer.lines().map(|s| s.to_string()).collect();
                                if app.cursor_row < lines.len() {
                                    lines[app.cursor_row].insert_str(app.cursor_col, "    ");
                                    app.cursor_col += 4;
                                    app.buffer = lines.join("\n");
                                    app.dirty = true;
                                }
                            }
                        },
                        (KeyCode::Char(ch), m)
                            if m.is_empty()
                                || m.contains(KeyModifiers::SHIFT)
                                || m.contains(KeyModifiers::ALT) =>
                        {
                            if app.search_active {
                                app.search_query.push(ch);
                                app.search_matches.clear();
                                app.set_status(format!("Search: {}", app.search_query));
                                continue;
                            }
                            let now = Instant::now();
                            let since_last = now.saturating_duration_since(app.last_key_time);
                            app.last_key_time = now;
                            if since_last < Duration::from_millis(5) {
                                app.paste_active = true;
                            }
                            if since_last > Duration::from_millis(40) {
                                app.paste_active = false;
                            }
                            app.chars_since_snapshot += 1;
                            let need_snapshot = !app.paste_active
                                && (now.saturating_duration_since(app.last_snapshot_at)
                                    > Duration::from_millis(300)
                                    || app.chars_since_snapshot > 80);
                            if need_snapshot {
                                app.snapshot();
                            }
                            if app.paste_active && app.chars_since_snapshot == 1 {
                                app.snapshot();
                            }
                            match app.mode {
                                EditorMode::Append => app.input.push(ch),
                                EditorMode::Insert => {
                                    let mut lines: Vec<String> = app
                                        .buffer
                                        .lines()
                                        .map(|s| s.to_string())
                                        .collect();
                                    if lines.is_empty() {
                                        lines.push(String::new());
                                    }
                                    if app.cursor_row >= lines.len() {
                                        app.cursor_row = lines.len() - 1;
                                    }
                                    if app.cursor_row < lines.len() {
                                        lines[app.cursor_row].insert(app.cursor_col, ch);
                                        app.cursor_col += 1;
                                        app.buffer = lines.join("\n");
                                        app.dirty = true;
                                    }
                                }
                            }
                            if app.paste_active && app.chars_since_snapshot > 2000 {
                                app.snapshot();
                                app.paste_active = false;
                            }
                        }
                        (KeyCode::Up, _) => {
                            if app.cursor_row > 0 {
                                app.cursor_row -= 1;
                            }
                            if app.cursor_row < app.scroll {
                                app.scroll = app.cursor_row;
                            }
                            let line_len = app
                                .buffer
                                .lines()
                                .nth(app.cursor_row)
                                .map(|l| l.len())
                                .unwrap_or(0);
                            if app.cursor_col > line_len {
                                app.cursor_col = line_len;
                            }
                        }
                        (KeyCode::Down, _) => {
                            let line_count = app.buffer.lines().count();
                            if app.cursor_row + 1 < line_count {
                                app.cursor_row += 1;
                            }
                            let view_height =
                                (terminal.size()?.height as usize).saturating_sub(8);
                            if app.cursor_row >= app.scroll + view_height {
                                app.scroll = app
                                    .cursor_row
                                    .saturating_sub(view_height)
                                    .min(app.cursor_row);
                            }
                            let line_len = app
                                .buffer
                                .lines()
                                .nth(app.cursor_row)
                                .map(|l| l.len())
                                .unwrap_or(0);
                            if app.cursor_col > line_len {
                                app.cursor_col = line_len;
                            }
                        }
                        (KeyCode::Left, _) => {
                            if app.cursor_col > 0 {
                                app.cursor_col -= 1;
                            } else if app.cursor_row > 0 {
                                app.cursor_row -= 1;
                                app.cursor_col = app
                                    .buffer
                                    .lines()
                                    .nth(app.cursor_row)
                                    .map(|l| l.len())
                                    .unwrap_or(0);
                            }
                        }
                        (KeyCode::Right, _) => {
                            let line_len = app
                                .buffer
                                .lines()
                                .nth(app.cursor_row)
                                .map(|l| l.len())
                                .unwrap_or(0);
                            if app.cursor_col < line_len {
                                app.cursor_col += 1;
                            } else {
                                let line_count = app.buffer.lines().count();
                                if app.cursor_row + 1 < line_count {
                                    app.cursor_row += 1;
                                    app.cursor_col = 0;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::Mouse(me) => {
                    if app.mouse_capture {
                        if app.show_key_debug {
                            app.set_status(format!("mouse: {:?}", me));
                        }
                        if let crossterm::event::MouseEvent {
                            kind: MouseEventKind::Down(MouseButton::Left),
                            column,
                            row,
                            ..
                        } = me
                        {
                            if row == 1 {
                                let size = terminal.size()?;
                                let area = Rect {
                                    x: 0,
                                    y: 1,
                                    width: size.width,
                                    height: 1,
                                };
                                let specs = compute_button_specs(area, &app);
                                for spec in specs {
                                    if column >= spec.area.x
                                        && column < spec.area.x + spec.area.width
                                    {
                                        match spec.action {
                                            ButtonAction::Save => {
                                                if let Err(e) = app.save() {
                                                    app.set_status(format!(
                                                        "Save failed: {e}"
                                                    ));
                                                }
                                            }
                                            ButtonAction::Compile =>
                                                app.compile(pretty, skip_sema),
                                            ButtonAction::Run =>
                                                app.run(pretty, skip_sema),
                                            ButtonAction::ToggleEmit => {
                                                app.emit_mode = app.emit_mode.toggle();
                                                app.set_status(format!(
                                                    "Emit → {}",
                                                    app.emit_mode.label()
                                                ));
                                            }
                                            ButtonAction::ToggleMode => {
                                                app.mode = match app.mode {
                                                    EditorMode::Append =>
                                                        EditorMode::Insert,
                                                    EditorMode::Insert =>
                                                        EditorMode::Append,
                                                };
                                                app.set_status(match app.mode {
                                                    EditorMode::Append =>
                                                        "Mode: Append",
                                                    EditorMode::Insert =>
                                                        "Mode: Insert",
                                                });
                                            }
                                            ButtonAction::Search => {
                                                app.search_active = true;
                                                app.search_query.clear();
                                                app.set_status(
                                                    "Search: type query, Enter=next, Esc=cancel",
                                                );
                                            }
                                            ButtonAction::ToggleMouse => {
                                                app.mouse_capture = !app.mouse_capture;
                                                if app.mouse_capture {
                                                    let _ = execute!(
                                                        std::io::stdout(),
                                                        EnableMouseCapture
                                                    );
                                                    app.set_status(
                                                        "Mouse capture ON",
                                                    );
                                                } else {
                                                    let _ = execute!(
                                                        std::io::stdout(),
                                                        DisableMouseCapture
                                                    );
                                                    app.set_status(
                                                        "Mouse capture OFF",
                                                    );
                                                }
                                            }
                                            ButtonAction::Quit => {
                                                if app.dirty {
                                                    app.set_status(
                                                        "Unsaved changes — Ctrl+S to save, click Quit again",
                                                    );
                                                    app.dirty = false;
                                                } else {
                                                    break 'outer;
                                                }
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }
    Ok(())
}

// ---------- Rendering ----------
fn ui(f: &mut ratatui::Frame<'_>, app: &App) {
    let (accent, accent_alt, yellow, dim) = neon();
    let _unused = (accent_alt, dim); // silence unused for now

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1), // header
                Constraint::Length(1), // buttons
                Constraint::Min(5),    // main
                Constraint::Length(1), // status
                Constraint::Length(3), // input
            ]
            .as_ref(),
        )
        .split(f.size());

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " A E O N M I   S H A R D ",
            Style::default()
                .fg(Color::Black)
                .bg(accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("emit: {}", app.emit_mode.label()),
            Style::default().fg(yellow),
        ),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(header, rows[0]);

    // Buttons row
    draw_buttons_row(f, rows[1], app);

    // Main split
    let main_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)].as_ref())
        .split(rows[2]);
    let left_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(5)].as_ref())
        .split(main_split[0]);

    // Buffer block
    let buf_block = Block::default().borders(Borders::ALL).title(Span::styled(
        format!(
            " Buffer — {}{}",
            app.filepath.display(),
            if app.dirty { " *" } else { "" }
        ),
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    ));

    let lines: Vec<&str> = app.buffer.lines().collect();
    let height = left_split[0].height.saturating_sub(2) as usize; // minus borders
    let start = app.scroll.min(lines.len());
    let end = (start + height).min(lines.len());
    let mut rendered = String::new();
    for (i, l) in lines[start..end].iter().enumerate() {
        if i > 0 {
            rendered.push('\n');
        }
        rendered.push_str(l);
    }

    let highlight = |line: &str, query: Option<&str>| -> Line {
        use ratatui::text::Span;
        let mut spans: Vec<Span> = Vec::new();
        let mut i = 0;
        let lower_q = query.map(|q| q.to_lowercase());
        let keywords = [
            "function", "let", "if", "else", "while", "for", "return", "log", "superpose",
            "entangle", "measure", "qubit",
        ];
        while i < line.len() {
            let rest = &line[i..];
            if rest.starts_with('"') {
                if let Some(end) = rest[1..].find('"') {
                    let token = &line[i..=i + end + 1];
                    spans.push(Span::styled(
                        token.to_string(),
                        Style::default().fg(Color::Rgb(255, 240, 0)),
                    ));
                    i += end + 2;
                    continue;
                } else {
                    spans.push(Span::styled(
                        rest.to_string(),
                        Style::default().fg(Color::Rgb(255, 240, 0)),
                    ));
                    break;
                }
            }
            if rest.chars().next().unwrap().is_whitespace() {
                spans.push(Span::raw(rest.chars().next().unwrap().to_string()));
                i += 1;
                continue;
            }
            let mut end = i;
            for (j, ch) in line[i..].char_indices() {
                if ch.is_whitespace() {
                    break;
                }
                end = i + j;
            }
            let mut token = &line[i..=end];
            if token.ends_with(char::is_whitespace) {
                token = token.trim_end();
            }
            let lower = token.to_lowercase();
            if keywords.contains(&lower.as_str()) {
                spans.push(Span::styled(
                    token.to_string(),
                    Style::default()
                        .fg(Color::Rgb(130, 0, 200))
                        .add_modifier(Modifier::BOLD),
                ));
            } else if token.chars().all(|c| c.is_ascii_digit() || c == '.')
                && token.chars().any(|c| c.is_ascii_digit())
            {
                spans.push(Span::styled(
                    token.to_string(),
                    Style::default().fg(Color::Rgb(0, 255, 180)),
                ));
            } else if let Some(q) = &lower_q {
                if !q.is_empty() && lower.contains(q) {
                    spans.push(Span::styled(
                        token.to_string(),
                        Style::default()
                            .bg(Color::Rgb(255, 240, 0))
                            .fg(Color::Black),
                    ));
                } else {
                    spans.push(Span::raw(token.to_string()));
                }
            } else {
                spans.push(Span::raw(token.to_string()));
            }
            i = end + 1;
        }
        Line::from(spans)
    };

    let mut lines_styled: Vec<Line> = Vec::new();
    for l in rendered.lines() {
        lines_styled.push(highlight(
            l,
            if app.search_active && !app.search_query.is_empty() {
                Some(app.search_query.as_str())
            } else {
                None
            },
        ));
    }
    if matches!(app.mode, EditorMode::Append)
        && !app.input.is_empty()
        && end == lines.len()
    {
        lines_styled.push(Line::from(vec![Span::styled(
            format!("> {}", app.input),
            Style::default()
                .fg(Color::Rgb(90, 90, 90))
                .add_modifier(Modifier::ITALIC),
        )]));
    }
    let buf_par = Paragraph::new(Text::from(lines_styled))
        .block(buf_block)
        .wrap(Wrap { trim: false });
    f.render_widget(buf_par, left_split[0]);

    if matches!(app.mode, EditorMode::Insert) {
        let cursor_screen_row = app.cursor_row.saturating_sub(start);
        if cursor_screen_row < height {
            let cursor_x =
                (app.cursor_col.min(lines.get(app.cursor_row).map(|l| l.len()).unwrap_or(0)) + 1)
                    as u16; // +1 for left border
            let cursor_y = (left_split[0].y + 1 + cursor_screen_row as u16) as u16; // +1 for top
            f.set_cursor(left_split[0].x + cursor_x, cursor_y);
        }
    }

    // Diagnostics
    let diags_title = Span::styled(
        " Diagnostics ",
        Style::default()
            .fg(Color::Rgb(130, 0, 200))
            .add_modifier(Modifier::BOLD),
    );
    let diags_items: Vec<ListItem> = if app.diagnostics.is_empty() {
        vec![ListItem::new(Span::styled(
            "no issues",
            Style::default().fg(Color::Rgb(190, 190, 200)),
        ))]
    } else {
        app.diagnostics
            .iter()
            .map(|d| ListItem::new(d.as_str()))
            .collect()
    };
    let diags = List::new(diags_items)
        .block(Block::default().borders(Borders::ALL).title(diags_title));
    f.render_widget(diags, left_split[1]);

    // Cheatsheet
    let cheats_block = Block::default().borders(Borders::ALL).title(Span::styled(
        " QPoly Cheatsheet ",
        Style::default()
            .fg(Color::Rgb(255, 240, 0))
            .add_modifier(Modifier::BOLD),
    ));
    let cheat_pairs = cheatsheet_from_map(&app.qpoly);
    let mut lines_vec: Vec<Line> = Vec::new();
    for (chord, glyph) in cheat_pairs {
        lines_vec.push(Line::from(vec![
            Span::styled(
                format!("{:<6}", chord),
                Style::default()
                    .fg(Color::Rgb(255, 240, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" → "),
            Span::styled(glyph, Style::default().fg(Color::Rgb(225, 0, 180))),
        ]));
    }
    let cheats = Paragraph::new(Text::from(lines_vec)).block(cheats_block);
    f.render_widget(cheats, main_split[1]);

    draw_status_and_input(f, rows[3], rows[4], app, accent, yellow);
}

fn draw_status_and_input(
    f: &mut ratatui::Frame<'_>,
    status_area: Rect,
    input_area: Rect,
    app: &App,
    accent: Color,
    yellow: Color,
) {
    let mode_label = match app.mode {
        EditorMode::Append => "APPEND",
        EditorMode::Insert => "INSERT",
    };
    let status_line = Paragraph::new(Line::from(vec![
        Span::styled(
            " Aeonmi ",
            Style::default()
                .bg(accent)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            if app.search_active {
                let count = app.search_matches.len();
                let idx = if count>0 { app.search_index+1 } else { 0 };
                format!("{} | /{} [{} / {}] {}", mode_label, app.search_query, idx, count, app.status)
            } else {
                format!("{} | {}", mode_label, app.status)
            },
            Style::default().fg(yellow),
        ),
    ]));
    f.render_widget(status_line, status_area);

    let title = match app.mode {
        EditorMode::Append => " Input (Enter appends line) ",
        EditorMode::Insert => " Input (Insert mode shows buffer edits) ",
    };
    let input_block = Block::default().borders(Borders::ALL).title(title);
    let input_par = Paragraph::new(app.input.clone())
        .block(input_block)
        .wrap(Wrap { trim: false });
    f.render_widget(input_par, input_area);
}

fn draw_buttons_row(f: &mut ratatui::Frame<'_>, area: Rect, app: &App) {
    for b in compute_button_specs(area, app) {
        let (fg, bg) = match b.action {
            ButtonAction::Save => (Color::Black, Color::Rgb(0, 255, 180)),
            ButtonAction::Compile => (Color::Black, Color::Rgb(255, 240, 0)),
            ButtonAction::Run => (Color::Black, Color::Rgb(0, 200, 255)),
            ButtonAction::ToggleEmit => (Color::Black, Color::Rgb(225, 0, 180)),
            ButtonAction::ToggleMode => (Color::Black, Color::Rgb(130, 0, 200)),
            ButtonAction::Search => (Color::Black, Color::Rgb(255, 170, 0)),
            ButtonAction::ToggleMouse => (Color::Black, Color::Rgb(90, 90, 90)),
            ButtonAction::Quit => (Color::White, Color::Red),
        };
        let para = Paragraph::new(Line::from(vec![Span::styled(
            format!(" {} ", b.label),
            Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
        )]));
        f.render_widget(para, b.area);
    }
}

fn compute_button_specs(area: Rect, app: &App) -> Vec<ButtonSpec> {
    let labels = vec![
        (ButtonAction::Save, "Save".to_string()),
        (ButtonAction::Compile, "Compile".to_string()),
        (ButtonAction::Run, "Run".to_string()),
        (ButtonAction::ToggleEmit, format!("Emit:{}", app.emit_mode.label())),
        (
            ButtonAction::ToggleMode,
            match app.mode {
                EditorMode::Append => "Mode:Append".to_string(),
                EditorMode::Insert => "Mode:Insert".to_string(),
            },
        ),
        (ButtonAction::Search, "Search".to_string()),
        (
            ButtonAction::ToggleMouse,
            format!("Mouse:{}", if app.mouse_capture { "On" } else { "Off" }),
        ),
        (ButtonAction::Quit, "Quit".to_string()),
    ];
    let mut specs = Vec::new();
    let mut x = area.x;
    for (action, label) in labels {
        let w = (label.len() as u16) + 2; // padding
        if x + w > area.x + area.width {
            break;
        }
        let rect = Rect {
            x,
            y: area.y,
            width: w,
            height: 1,
        };
        specs.push(ButtonSpec {
            area: rect,
            label,
            action,
        });
        x = x.saturating_add(w);
        if x + 1 < area.x + area.width {
            x += 1;
        }
    }
    specs
}
