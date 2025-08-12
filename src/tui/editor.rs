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
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use crossterm::terminal::{self, SetTitle};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};

use crate::cli::EmitKind;
use crate::core::qpoly::QPolyMap;
use crate::commands::compile::compile_pipeline;

/// Neon palette (magenta/purple + blinding yellow)
fn neon() -> (Color, Color, Color, Color) {
    let magenta = Color::Rgb(225, 0, 180);
    let purple  = Color::Rgb(130, 0, 200);
    let yellow  = Color::Rgb(255, 240, 0);
    let dim     = Color::Rgb(190, 190, 200);
    (magenta, purple, yellow, dim)
}

#[derive(Copy, Clone)]
enum EmitMode { Js, Ai }

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
            status: String::from("⏎ append • Ctrl+S save • F4 emit=JS/AI • F5 compile • F6 run(JS) • Esc/Ctrl+Q quit • F1 debug"),
            last_status_at: Instant::now(),
            diagnostics: vec![],
            qpoly,
            emit_mode: EmitMode::Js,
            show_key_debug: false,
            last_key_debug: String::new(),
        }
    }

    fn set_status(&mut self, s: impl Into<String>) {
        self.status = s.into();
        self.last_status_at = Instant::now();
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
        ("->", "→"), ("<-", "←"), ("<=", "≤"), (">=", "≥"),
        ("!=", "≠"), ("==", "＝"), ("<=>", "⇔"),
        ("::", "∷"), (":=", "≔"), ("|0>", "∣0⟩"), ("|1>", "∣1⟩"),
    ];
    defaults.into_iter().map(|(a,b)| (a.to_string(), b.to_string())).collect()
}

pub fn run_editor_tui(
    file: Option<PathBuf>,
    _config_path: Option<PathBuf>,
    pretty: bool,
    skip_sema: bool,
) -> Result<()> {
    let filepath = file.unwrap_or_else(|| PathBuf::from("untitled.ai"));
    let map = QPolyMap::from_user_default_or_builtin();

    // Terminal setup
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, SetTitle("Aeonmi Shard"))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, App::new(filepath, map), pretty, skip_sema);

    // Restore
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture, SetTitle("Aeonmi"))?;
    terminal.show_cursor()?;
    res
}

// Only react to Press; allow Repeat for Enter/Backspace/Tab to be safe on Windows
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
                Event::Key(KeyEvent { code, modifiers, kind, .. }) => {
                    let accept = match kind {
                        KeyEventKind::Press => true,
                        KeyEventKind::Repeat => matches!(code, KeyCode::Enter | KeyCode::Backspace | KeyCode::Tab),
                        _ => false,
                    };
                    if !accept { continue; }

                    // Optional live key debug (F1 toggles)
                    if app.show_key_debug {
                        app.last_key_debug = format!("key: {:?} mods: {:?} kind: {:?}", code, modifiers, kind);
                        app.set_status(app.last_key_debug.clone());
                    }

                    match (code, modifiers) {
                        (KeyCode::F(1), _) => {
                            app.show_key_debug = !app.show_key_debug;
                            app.set_status(if app.show_key_debug {"Key debug ON"} else {"Key debug OFF"});
                        }
                        (KeyCode::F(4), _) => {
                            app.emit_mode = app.emit_mode.toggle();
                            app.set_status(format!("Emit → {}", app.emit_mode.label()));
                        }
                        (KeyCode::F(5), _) => app.compile(pretty, skip_sema),
                        (KeyCode::F(6), _) => app.run(pretty, skip_sema),

                        (KeyCode::Esc, _) |
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) |
                        (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                            // Quit: if dirty, warn once then allow quit on next press
                            if app.dirty {
                                app.set_status("Unsaved changes — Ctrl+S to save, Esc again to quit.");
                                app.dirty = false;
                                continue;
                            }
                            break;
                        }
                        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                            if let Err(e) = app.save() { app.set_status(format!("Save failed: {e}")); }
                        }

                        (KeyCode::Backspace, _) => { app.input.pop(); }
                        (KeyCode::Enter, _)     => { app.add_line(); }
                        (KeyCode::Tab, _)       => { app.input.push_str("    "); }

                        // Accept characters with NONE/SHIFT/ALT to be friendlier on Windows
                        (KeyCode::Char(ch), m)
                            if m.is_empty() || m.contains(KeyModifiers::SHIFT) || m.contains(KeyModifiers::ALT) =>
                        {
                            app.input.push(ch);
                        }
                        _ => {}
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }
    Ok(())
}

fn ui(f: &mut ratatui::Frame<'_>, app: &App) {
    let (accent, accent_alt, yellow, dim) = neon();

    // vertical: [header, main, input]
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ].as_ref())
        .split(f.size());

    // Header (centered)
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" A E O N M I   S H A R D ", Style::default().fg(Color::Black).bg(accent).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(format!("emit: {}", app.emit_mode.label()), Style::default().fg(yellow)),
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
    let buf_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(" Buffer — {}{}", app.filepath.display(), if app.dirty { " *" } else { "" }),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ));
    let buf_text = Text::from(app.buffer.as_str());
    let buf_par = Paragraph::new(buf_text).block(buf_block).wrap(Wrap { trim: false });
    f.render_widget(buf_par, left_split[0]);

    // Diagnostics
    let diags_title = Span::styled(" Diagnostics ", Style::default().fg(accent_alt).add_modifier(Modifier::BOLD));
    let diags_items: Vec<ListItem> = if app.diagnostics.is_empty() {
        vec![ListItem::new(Span::styled("no issues", Style::default().fg(dim)))]
    } else {
        app.diagnostics.iter().map(|d| ListItem::new(d.as_str())).collect()
    };
    let diags = List::new(diags_items).block(Block::default().borders(Borders::ALL).title(diags_title));
    f.render_widget(diags, left_split[1]);

    // Cheatsheet
    let cheats_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" QPoly Cheatsheet ", Style::default().fg(yellow).add_modifier(Modifier::BOLD)));
    let cheat_pairs = cheatsheet_from_map(&app.qpoly);
    let mut lines: Vec<Line> = Vec::new();
    for (chord, glyph) in cheat_pairs {
        lines.push(Line::from(vec![
            Span::styled(format!("{:<6}", chord), Style::default().fg(yellow).add_modifier(Modifier::BOLD)),
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

    let status_line = Paragraph::new(Line::from(vec![
        Span::styled(" Aeonmi ", Style::default().bg(accent).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(status, Style::default().fg(yellow)),
    ]));
    f.render_widget(status_line, rows[0]);

    let input_block = Block::default().borders(Borders::ALL).title(" Input (Enter to append) ");
    let input_par = Paragraph::new(input.to_string()).block(input_block);
    f.render_widget(input_par, rows[1]);
}
