#![cfg_attr(test, allow(dead_code, unused_variables))]
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
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

/// Neon palette (magenta/purple + blinding yellow)
fn neon() -> (Color, Color, Color, Color) {
    let magenta = Color::Rgb(225, 0, 180);
    let purple = Color::Rgb(130, 0, 200);
    let yellow = Color::Rgb(255, 240, 0);
    let dim = Color::Rgb(190, 190, 200);
    (magenta, purple, yellow, dim)
}

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
    fn to_emit_kind(self) -> EmitKind {
        match self {
            EmitMode::Js => EmitKind::Js,
            EmitMode::Ai => EmitKind::Ai,
        }
    }

    // May be unused in some frontends/tests; keep for API symmetry.
    #[allow(dead_code)]
    fn to_emit_kind_unused(self) -> EmitKind {
        self.to_emit_kind()
    }
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
    // Interactive editing state
    cursor_row: usize,
    cursor_col: usize,
    scroll: usize,
    mode: EditorMode,
    // History (undo/redo)
    history: Vec<String>,
    history_index: usize,
    // Search state
    search_active: bool,
    search_query: String,
    last_match_row: Option<usize>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum EditorMode {
    Append, // original line append model
    Insert, // editing existing lines
}

impl App {
    fn new(filepath: PathBuf, qpoly: QPolyMap) -> Self {
        let buffer = if filepath.exists() {
            fs::read_to_string(&filepath).unwrap_or_default()
        } else {
            String::new()
        };
        Self {
            filepath,
            buffer,
            input: String::new(),
            dirty: false,
            status: String::from(
                "⏎ append • Ctrl+S save • F4 emit=JS/AI • F5 compile • F6 run(JS) • F9 toggle-mouse • Esc/Ctrl+Q quit • F1 key-debug",
            ),
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
            search_active: false,
            search_query: String::new(),
            last_match_row: None,
        }
    }

    fn set_status(&mut self, s: impl Into<String>) {
        self.status = s.into();
        self.last_status_at = Instant::now();
    }

    fn snapshot(&mut self) {
        // Truncate redo tail if any and push current buffer
        if self.history_index < self.history.len() {
            self.history.truncate(self.history_index);
        }
        self.history.push(self.buffer.clone());
        self.history_index = self.history.len();
        // Limit history size
        if self.history.len() > 200 {
            let drop = self.history.len() - 200;
            self.history.drain(0..drop);
            self.history_index = self.history.len();
        }
    }

    fn undo(&mut self) {
        if self.history_index == 0 { return; }
        self.history_index -= 1;
        if let Some(state) = self.history.get(self.history_index) {
            self.buffer = state.clone();
            self.set_status("Undo");
            self.dirty = true; // reflect possible divergence
        }
    }

    fn redo(&mut self) {
        if self.history_index >= self.history.len() { return; }
        self.history_index += 1;
        if self.history_index > 0 { if let Some(state) = self.history.get(self.history_index - 1) { self.buffer = state.clone(); self.set_status("Redo"); self.dirty = true; } }
    }

    fn find_next(&mut self) {
        if self.search_query.is_empty() { return; }
        let q = self.search_query.to_lowercase();
        let start = self.last_match_row.map(|r| r + 1).unwrap_or(0);
        let lines: Vec<&str> = self.buffer.lines().collect();
        let mut found = None;
        for (i, line) in lines.iter().enumerate().skip(start) {
            if line.to_lowercase().contains(&q) { found = Some(i); break; }
        }
        if found.is_none() {
            // wrap
            for (i, line) in lines.iter().enumerate().take(start) {
                if line.to_lowercase().contains(&q) { found = Some(i); break; }
            }
        }
        if let Some(r) = found { self.last_match_row = Some(r); self.cursor_row = r; self.cursor_col = 0; self.set_status(format!("Found at line {}", r+1)); } else { self.set_status("No match"); }
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
    // Move cursor to end
    let line_count = self.buffer.lines().count();
    if line_count > 0 { self.cursor_row = line_count - 1; }
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
                    /* debug_titan */ false,
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
                    /* debug_titan */ false,
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
            /* debug_titan */ false,
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

/// Cheatsheet: mirror a few common defaults
fn cheatsheet_from_map(_map: &QPolyMap) -> Vec<(String, String)> {
    let defaults = vec![
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
    ];
    defaults
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

pub fn run_editor_tui(
    file: Option<PathBuf>,
    config_path: Option<PathBuf>,
    pretty: bool,
    skip_sema: bool,
) -> Result<()> {
    let filepath = file.unwrap_or_else(|| PathBuf::from("untitled.ai"));

    // Load QPoly map: explicit --config > default user path > built-in
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

    // Terminal setup
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        SetTitle("Aeonmi Shard")
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, App::new(filepath, map), pretty, skip_sema);

    // Restore
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        SetTitle("Aeonmi")
    )?;
    terminal.show_cursor()?;
    res
}

// React to Press; allow Repeat for Enter/Backspace/Tab (Windows friendliness)
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
    pretty: bool,
    skip_sema: bool,
) -> Result<()> {
    let tick_rate = Duration::from_millis(100);
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind,
                    ..
                }) => {
                    let accept = match kind {
                        KeyEventKind::Press => true,
                        KeyEventKind::Repeat => {
                            matches!(code, KeyCode::Enter | KeyCode::Backspace | KeyCode::Tab)
                        }
                        _ => false,
                    };
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
                            app.mode = match app.mode { EditorMode::Append => EditorMode::Insert, EditorMode::Insert => EditorMode::Append };
                            app.set_status(match app.mode { EditorMode::Append => "Mode: Append (Enter appends line)", EditorMode::Insert => "Mode: Insert (editing buffer)" });
                        }
                        (KeyCode::F(4), _) => {
                            app.emit_mode = app.emit_mode.toggle();
                            app.set_status(format!("Emit → {}", app.emit_mode.label()));
                        }
                        (KeyCode::F(9), _) => {
                            // Toggle mouse capture on/off to allow normal terminal selection / free roam
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
                        (KeyCode::F(3), _) => { if app.search_active { app.find_next(); } },

                        (KeyCode::Esc, _)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL)
                        | (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                            // Quit: if dirty, warn once then allow quit on next press
                            if app.dirty {
                                app.set_status(
                                    "Unsaved changes — Ctrl+S to save, Esc again to quit.",
                                );
                                app.dirty = false;
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
                            app.set_status("Search: type query, Enter=next, Esc=cancel");
                        }
                        (KeyCode::Char('z'), KeyModifiers::CONTROL) => { app.undo(); }
                        (KeyCode::Char('y'), KeyModifiers::CONTROL) => { app.redo(); }

                        (KeyCode::Backspace, _) => {
                            if app.search_active {
                                app.search_query.pop();
                                app.set_status(format!("Search: {}", app.search_query));
                                continue;
                            }
                            match app.mode {
                                EditorMode::Append => { app.input.pop(); }
                                EditorMode::Insert => {
                                    let mut lines: Vec<String> = app.buffer.lines().map(|s| s.to_string()).collect();
                                    if app.cursor_row < lines.len() {
                                        if app.cursor_col > 0 {
                                            lines[app.cursor_row].remove(app.cursor_col - 1);
                                            app.cursor_col -= 1;
                                            app.dirty = true;
                                        } else if app.cursor_row > 0 {
                                            // merge with previous line
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
                                    let mut lines: Vec<String> = app.buffer.lines().map(|s| s.to_string()).collect();
                                    if lines.is_empty() { lines.push(String::new()); }
                                    if app.cursor_row >= lines.len() { app.cursor_row = lines.len() - 1; }
                                    let (left, right) = if app.cursor_row < lines.len() {
                                        let line = &lines[app.cursor_row];
                                        let (l, r) = line.split_at(app.cursor_col.min(line.len()));
                                        (l.to_string(), r.to_string())
                                    } else { (String::new(), String::new()) };
                                    lines[app.cursor_row] = left;
                                    lines.insert(app.cursor_row + 1, right);
                                    app.cursor_row += 1;
                                    app.cursor_col = 0;
                                    app.buffer = lines.join("\n");
                                    app.dirty = true;
                                }
                            }
                        }
                        (KeyCode::Tab, _) => {
                            match app.mode {
                                EditorMode::Append => app.input.push_str("    "),
                                EditorMode::Insert => {
                                    let mut lines: Vec<String> = app.buffer.lines().map(|s| s.to_string()).collect();
                                    if app.cursor_row < lines.len() {
                                        lines[app.cursor_row].insert_str(app.cursor_col, "    ");
                                        app.cursor_col += 4;
                                        app.buffer = lines.join("\n");
                                        app.dirty = true;
                                    }
                                }
                            }
                        }

                        // Accept characters with NONE/SHIFT/ALT (Windows sometimes sets these)
                        (KeyCode::Char(ch), m)
                            if m.is_empty()
                                || m.contains(KeyModifiers::SHIFT)
                                || m.contains(KeyModifiers::ALT) =>
                        {
                            if app.search_active {
                                app.search_query.push(ch);
                                app.set_status(format!("Search: {}", app.search_query));
                                continue;
                            }
                            app.snapshot();
                            match app.mode {
                                EditorMode::Append => app.input.push(ch),
                                EditorMode::Insert => {
                                    let mut lines: Vec<String> = app.buffer.lines().map(|s| s.to_string()).collect();
                                    if lines.is_empty() { lines.push(String::new()); }
                                    if app.cursor_row >= lines.len() { app.cursor_row = lines.len() - 1; }
                                    if app.cursor_row < lines.len() {
                                        lines[app.cursor_row].insert(app.cursor_col, ch);
                                        app.cursor_col += 1;
                                        app.buffer = lines.join("\n");
                                        app.dirty = true;
                                    }
                                }
                            }
                        }
                        // Navigation keys (Insert mode only for now)
                        (KeyCode::Up, _) => {
                            if app.cursor_row > 0 { app.cursor_row -= 1; }
                            if app.cursor_row < app.scroll { app.scroll = app.cursor_row; }
                            // clamp col
                            let line_len = app.buffer.lines().nth(app.cursor_row).map(|l| l.len()).unwrap_or(0);
                            if app.cursor_col > line_len { app.cursor_col = line_len; }
                        }
                        (KeyCode::Down, _) => {
                            let line_count = app.buffer.lines().count();
                            if app.cursor_row + 1 < line_count { app.cursor_row += 1; }
                            let view_height =  (terminal.size()?.height as usize).saturating_sub(8); // rough header+footer
                            if app.cursor_row >= app.scroll + view_height { app.scroll = app.cursor_row.saturating_sub(view_height).min(app.cursor_row); }
                            let line_len = app.buffer.lines().nth(app.cursor_row).map(|l| l.len()).unwrap_or(0);
                            if app.cursor_col > line_len { app.cursor_col = line_len; }
                        }
                        (KeyCode::Left, _) => {
                            if app.cursor_col > 0 { app.cursor_col -= 1; } else if app.cursor_row > 0 { app.cursor_row -= 1; app.cursor_col = app.buffer.lines().nth(app.cursor_row).map(|l| l.len()).unwrap_or(0); }
                        }
                        (KeyCode::Right, _) => {
                            let line_len = app.buffer.lines().nth(app.cursor_row).map(|l| l.len()).unwrap_or(0);
                            if app.cursor_col < line_len { app.cursor_col += 1; } else {
                                let line_count = app.buffer.lines().count();
                                if app.cursor_row + 1 < line_count { app.cursor_row += 1; app.cursor_col = 0; }
                            }
                        }
                        _ => {}
                    }
                }
                Event::Resize(_, _) => {}
                Event::Mouse(me) => {
                    // Basic mouse handling: if capture disabled, ignore.
                    if app.mouse_capture {
                        // For now just show coordinates in status when key debug on.
                        if app.show_key_debug {
                            app.set_status(format!("mouse: {:?}", me));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

// ratatui::Frame is Frame<'a> (non-generic over backend)
fn ui(f: &mut ratatui::Frame<'_>, app: &App) {
    let (accent, accent_alt, yellow, dim) = neon();

    // vertical: [header, main, input]
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    // Header (centered)
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

    // Main split: [buffer+diagnostics | cheatsheet]
    let main_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)].as_ref())
        .split(rows[1]);

    let left_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(5)].as_ref())
        .split(main_split[0]);

    // Buffer
    let buf_block = Block::default().borders(Borders::ALL).title(Span::styled(
        format!(
            " Buffer — {}{}",
            app.filepath.display(),
            if app.dirty { " *" } else { "" }
        ),
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    ));
    // Visible lines respecting scroll
    let lines: Vec<&str> = app.buffer.lines().collect();
    let height = left_split[0].height.saturating_sub(2) as usize; // minus borders
    let start = app.scroll.min(lines.len());
    let end = (start + height).min(lines.len());
    let mut rendered = String::new();
    for (idx, l) in lines[start..end].iter().enumerate() {
        if idx > 0 { rendered.push('\n'); }
        rendered.push_str(l);
    }
    let highlight = |line: &str, query: Option<&str>| -> Line {
        use ratatui::text::Span;
        let mut spans: Vec<Span> = Vec::new();
        let mut i = 0;
        let lower_q = query.map(|q| q.to_lowercase());
        // Simple tokenization by whitespace
        let keywords = ["function","let","if","else","while","for","return","log","superpose","entangle","measure","qubit"]; 
        while i < line.len() {
            let rest = &line[i..];
            if rest.starts_with('"') { // string literal
                if let Some(end) = rest[1..].find('"') { let token = &line[i..=i+end+1]; spans.push(Span::styled(token.to_string(), Style::default().fg(Color::Rgb(255,240,0)))); i += end+2; continue; } else { spans.push(Span::styled(rest.to_string(), Style::default().fg(Color::Rgb(255,240,0)))); break; }
            }
            if rest.chars().next().unwrap().is_whitespace() { spans.push(Span::raw(rest.chars().next().unwrap().to_string())); i += 1; continue; }
            // token boundary
            let mut end = i;
            for (j,ch) in line[i..].char_indices() { if ch.is_whitespace() { break; } end = i + j; }
            let mut token = &line[i..=end];
            // adjust end (loop sets end to last non-space char). If space follows, fine.
            if token.ends_with(char::is_whitespace) { token = token.trim_end(); }
            let lower = token.to_lowercase();
            if keywords.contains(&lower.as_str()) { spans.push(Span::styled(token.to_string(), Style::default().fg(Color::Rgb(130,0,200)).add_modifier(Modifier::BOLD))); }
            else if token.chars().all(|c| c.is_ascii_digit() || c=='.') && token.chars().any(|c| c.is_ascii_digit()) { spans.push(Span::styled(token.to_string(), Style::default().fg(Color::Rgb(0,255,180)))); }
            else {
                // search highlight
                if let Some(q) = &lower_q { if !q.is_empty() && lower.contains(q) { spans.push(Span::styled(token.to_string(), Style::default().bg(Color::Rgb(255,240,0)).fg(Color::Black))); } else { spans.push(Span::raw(token.to_string())); } }
                else { spans.push(Span::raw(token.to_string())); }
            }
            i = end + 1;
        }
        Line::from(spans)
    };
    let mut lines_styled: Vec<Line> = Vec::new();
    for l in rendered.lines() { lines_styled.push(highlight(l, if app.search_active && !app.search_query.is_empty() { Some(app.search_query.as_str()) } else { None })); }
    let buf_par = Paragraph::new(Text::from(lines_styled)).block(buf_block).wrap(Wrap { trim: false });
    f.render_widget(buf_par, left_split[0]);

    // Cursor (only in Insert mode)
    if matches!(app.mode, EditorMode::Insert) {
        let cursor_screen_row = app.cursor_row.saturating_sub(start);
        if cursor_screen_row < height {
            let cursor_x = (app.cursor_col.min(
                lines.get(app.cursor_row).map(|l| l.len()).unwrap_or(0)
            ) + 1) as u16; // +1 for left border
            let cursor_y = (left_split[0].y + 1 + cursor_screen_row as u16) as u16; // +1 for top border
            f.set_cursor(left_split[0].x + cursor_x, cursor_y);
        }
    }

    // Diagnostics
    let diags_title = Span::styled(
        " Diagnostics ",
        Style::default().fg(accent_alt).add_modifier(Modifier::BOLD),
    );
    let diags_items: Vec<ListItem> = if app.diagnostics.is_empty() {
        vec![ListItem::new(Span::styled(
            "no issues",
            Style::default().fg(dim),
        ))]
    } else {
        app.diagnostics
            .iter()
            .map(|d| ListItem::new(d.as_str()))
            .collect()
    };
    let diags =
        List::new(diags_items).block(Block::default().borders(Borders::ALL).title(diags_title));
    f.render_widget(diags, left_split[1]);

    // Cheatsheet
    let cheats_block = Block::default().borders(Borders::ALL).title(Span::styled(
        " QPoly Cheatsheet ",
        Style::default().fg(yellow).add_modifier(Modifier::BOLD),
    ));
    let cheat_pairs = cheatsheet_from_map(&app.qpoly);
    let mut lines: Vec<Line> = Vec::new();
    for (chord, glyph) in cheat_pairs {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<6}", chord),
                Style::default().fg(yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" → "),
            Span::styled(glyph, Style::default().fg(accent)),
        ]));
    }
    let cheats = Paragraph::new(Text::from(lines)).block(cheats_block);
    f.render_widget(cheats, main_split[1]);

    // Input/status row
    draw_input_and_status(f, rows[2], &app.input, &app.status, accent, yellow);
}

fn draw_input_and_status(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    input: &str,
    status: &str,
    accent: Color,
    yellow: Color,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(2)].as_ref())
        .split(area);

    let mode_label = match app.mode { EditorMode::Append => "APPEND", EditorMode::Insert => "INSERT" };
    let status_line = Paragraph::new(Line::from(vec![
        Span::styled(
            " Aeonmi ",
            Style::default()
                .bg(accent)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(format!("{} | {}", mode_label, status), Style::default().fg(yellow)),
    ]));
    f.render_widget(status_line, rows[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(" Input (Enter to append) ");
    let input_par = Paragraph::new(input.to_string()).block(input_block);
    f.render_widget(input_par, rows[1]);
}
