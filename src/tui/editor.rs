use std::fs;
use std::io;
use std::io::stdout;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

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
    _diagnostics: Vec<String>,
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
            status: String::from(
                "⏎ append • Ctrl+S save • F4 emit=JS/AI • F5 compile • F6 run(JS) • Esc/Ctrl+Q quit • F1 debug",
            ),
            last_status_at: Instant::now(),
            _diagnostics: vec![],
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

    fn save(&mut self) -> io::Result<()> {
        fs::write(&self.filepath, &self.buffer)?;
        self.dirty = false;
        self.set_status(format!("Saved {}", self.filepath.display()));
        Ok(())
    }

    fn compile(&mut self, pretty: bool, skip_sema: bool, debug_titan: bool) {
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
                    debug_titan,
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
                    debug_titan,
                ) {
                    Ok(()) => self.set_status(format!("Compiled → {}", out.display())),
                    Err(e) => self.set_status(format!("Compile error: {e}")),
                }
            }
        }
    }

    fn run(&mut self, pretty: bool, skip_sema: bool, debug_titan: bool) {
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
            debug_titan,
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

fn draw_help() -> Paragraph<'static> {
    let (_magenta, purple, yellow, dim) = neon();
    let lines = vec![
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("F4", Style::default().fg(yellow).add_modifier(Modifier::BOLD)),
            Span::raw("  Toggle emit mode (JS/AI)"),
        ]),
        Line::from(vec![
            Span::styled("F5", Style::default().fg(yellow).add_modifier(Modifier::BOLD)),
            Span::raw("  Compile (Titan debug ON)"),
        ]),
        Line::from(vec![
            Span::styled("F6", Style::default().fg(yellow).add_modifier(Modifier::BOLD)),
            Span::raw("  Run (compile→aeonmi.run.js)"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+S", Style::default().fg(yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Save buffer"),
        ]),
        Line::from(vec![
            Span::styled("Esc/Ctrl+Q", Style::default().fg(yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Exit editor"),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "Cheatsheet:",
            Style::default().fg(purple).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("  • .ai scripts compile via pipeline")),
        Line::from(Span::raw("  • QUBE ops map to Titan/Qiskit soon")),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "Status clears after a few seconds",
            Style::default().fg(dim),
        )),
    ];

    Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled("Help", Style::default().fg(purple))),
        )
        .wrap(Wrap { trim: false })
}

fn ui(f: &mut Frame, app: &App) {
    let (magenta, _purple, yellow, _dim) = neon();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(f.size());

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(layout[0]);

    // Editor panel: buffer + current input line
    let title = format!(
        "{}  [EMIT:{}]",
        app.filepath.to_string_lossy(),
        app.emit_mode.label()
    );

    let mut text = String::new();
    if !app.buffer.is_empty() {
        text.push_str(&app.buffer);
        text.push('\n');
    }
    text.push_str("> ");
    text.push_str(&app.input);

    let editor = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(title, Style::default().fg(magenta))),
    )
    .wrap(Wrap { trim: false });

    // Help panel
    let help = draw_help();

    // Status line (auto-fade)
    let status_txt = if app.last_status_at.elapsed() > Duration::from_secs(5) {
        if app.show_key_debug {
            format!("Key: {}", app.last_key_debug)
        } else {
            String::new()
        }
    } else {
        app.status.clone()
    };
    let status = Paragraph::new(Span::styled(status_txt, Style::default().fg(yellow)));

    f.render_widget(editor, top[0]);
    f.render_widget(help, top[1]);
    f.render_widget(status, layout[1]);
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
    pretty: bool,
    skip_sema: bool,
) -> io::Result<()> {
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
                        app.last_key_debug = format!("{code:?} + {modifiers:?} ({kind:?})");
                    }

                    match (code, modifiers) {
                        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                            if let Err(e) = app.save() {
                                app.set_status(format!("Save failed: {e}"));
                            }
                        }
                        (KeyCode::Char('q'), KeyModifiers::CONTROL) | (KeyCode::Esc, _) => {
                            return Ok(());
                        }
                        (KeyCode::F(1), _) => {
                            app.show_key_debug = !app.show_key_debug;
                            app.set_status(if app.show_key_debug {
                                "Key debug ON"
                            } else {
                                "Key debug OFF"
                            });
                        }
                        (KeyCode::F(4), _) => {
                            app.emit_mode = app.emit_mode.toggle();
                            app.set_status(format!("Emit → {}", app.emit_mode.label()));
                        }
                        (KeyCode::F(5), _) => app.compile(pretty, skip_sema, true), // Titan debug ON
                        (KeyCode::F(6), _) => app.run(pretty, skip_sema, true),     // Titan debug ON

                        (KeyCode::Enter, _) => {
                            app.add_line();
                        }
                        (KeyCode::Backspace, _) => {
                            app.input.pop();
                        }
                        (KeyCode::Tab, _) => {
                            app.input.push('\t');
                            app.dirty = true;
                        }
                        (KeyCode::Char(c), KeyModifiers::NONE) => {
                            app.input.push(c);
                            app.dirty = true;
                        }
                        (KeyCode::Char(c), m) if m.contains(KeyModifiers::SHIFT) => {
                            app.input.push(c);
                            app.dirty = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

/// Public entry for the TUI editor so `src/commands/edit.rs` can use it.
pub fn run_editor_tui(filepath: PathBuf, qpoly: QPolyMap) -> io::Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;
    let app = App::new(filepath, qpoly);

    let res = run_app(&mut terminal, app, true, false);

    // Always try to restore terminal state
    disable_raw_mode().ok();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    res
}
