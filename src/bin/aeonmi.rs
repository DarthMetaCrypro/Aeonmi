// src/bin/Aeonmi.rs
//! Aeonmi Shard — standalone neon shell (Windows-friendly).
//! Commands: help, ls, cd, mkdir, mv, cat, edit [--tui] <file>, compile <file> [--emit js|ai] [--out <path>], run <file>, clear, exit

use std::{env, fs};
use std::io::{self, Write, Read};
use std::path::{Path, PathBuf};
use std::process::Command as Pcmd;

use colored::Colorize;
use crossterm::{
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
    cursor,
    style::{Print},
};
use clap::{Arg, Command}; // only for parsing one-off args like --emit in compile cmd

// Pull in your project modules
use aeonmi_project::cli::EmitKind;
use aeonmi_project::commands;

// Small helpers for colors
fn neon_magenta() -> colored::Color { colored::Color::TrueColor { r: 225, g: 0, b: 180 } }
fn neon_purple()  -> colored::Color { colored::Color::TrueColor { r: 130, g: 0, b: 200 } }
fn neon_yellow()  -> colored::Color { colored::Color::TrueColor { r: 255, g: 240, b: 0 } }
fn neon_dim()     -> colored::Color { colored::Color::TrueColor { r: 190, g: 190, b: 200 } }

fn print_centered_header() -> crossterm::Result<()> {
    // use terminal size to center the title
    let (cols, _) = terminal::size()?;
    let title = " A E O N M I   S H A R D ";
    let pad = if cols as usize > title.len() { (cols as usize - title.len()) / 2 } else { 0 };
    let line = format!("{}{}", " ".repeat(pad), title);
    execute!(
        io::stdout(),
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        Print(line.bold().truecolor(0,0,0).on_color(neon_magenta())),
        cursor::MoveTo(0, 2),
        Print("Welcome. Type ".to_string().color(neon_dim())),
        Print("help".to_string().color(neon_yellow()).bold()),
        Print(" for commands.\n".to_string().color(neon_dim())),
    )?;
    Ok(())
}

fn pwd() -> PathBuf { env::current_dir().unwrap_or_else(|_| PathBuf::from(".")) }

fn prompt() {
    let dir = pwd();
    let prompt = format!("⟂ {} >", dir.display()).color(neon_purple()).bold();
    print!("{} ", prompt);
    let _ = io::stdout().flush();
}

fn list_dir(path: &Path) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(path)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name().to_ascii_lowercase());
    for e in entries {
        let meta = e.metadata();
        let name = e.file_name().to_string_lossy().to_string();
        match meta {
            Ok(m) if m.is_dir() => println!("{}", name.color(neon_yellow()).bold()),
            Ok(_) => println!("{}", name),
            Err(_) => println!("{}", name),
        }
    }
    Ok(())
}

fn read_line() -> io::Result<String> {
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim_end_matches(&['\r','\n'][..]).to_string())
}

fn parse_words(line: &str) -> Vec<String> {
    // very simple splitter; supports quotes
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_q = false;
    for ch in line.chars() {
        match ch {
            '"' => { in_q = !in_q; }
            ' ' | '\t' if !in_q => {
                if !cur.is_empty() { out.push(std::mem::take(&mut cur)); }
            }
            _ => cur.push(ch),
        }
    }
    if !cur.is_empty() { out.push(cur); }
    out
}

fn compile_cmd(args: &[String]) -> anyhow::Result<()> {
    // use a tiny Clap parser just for flags after 'compile'
    let cmd = Command::new("compile")
        .arg(Arg::new("input").required(true))
        .arg(Arg::new("emit").long("emit").value_parser(["js","ai"]).default_value("js"))
        .arg(Arg::new("out").long("out").value_name("PATH"));
    let matches = cmd.try_get_matches_from(args.iter())?;

    let input = PathBuf::from(matches.get_one::<String>("input").unwrap());
    let emit_s = matches.get_one::<String>("emit").unwrap().as_str();
    let emit = if emit_s == "ai" { EmitKind::Ai } else { EmitKind::Js };
    let out = matches.get_one::<String>("out")
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| if matches!(emit, EmitKind::Ai) { PathBuf::from("output.ai") } else { PathBuf::from("output.js") });

    commands::compile::compile_pipeline(
        Some(input),
        emit,
        out,
        /*tokens*/ false,
        /*ast*/ false,
        /*pretty*/ true,
        /*skip_sema*/ false,
    )?;
    Ok(())
}

fn run_cmd(args: &[String]) -> anyhow::Result<()> {
    let cmd = Command::new("run")
        .arg(Arg::new("input").required(true))
        .arg(Arg::new("out").long("out").value_name("PATH"));
    let matches = cmd.try_get_matches_from(args.iter())?;
    let input = PathBuf::from(matches.get_one::<String>("input").unwrap());
    let out = matches.get_one::<String>("out").map(|s| PathBuf::from(s));
    commands::run::main_with_opts(input, out, /*pretty*/true, /*no_sema*/false)?;
    Ok(())
}

fn cat_file(path: &Path) -> io::Result<()> {
    let mut f = fs::File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    println!("{}", s);
    Ok(())
}

fn print_help() {
    let h = r#"
Commands:
  help                         Show this help
  ls [path]                    List directory
  cd <path>                    Change directory
  mkdir <name>                 Create directory
  mv <src> <dst>               Move/rename
  cat <file>                   Print file
  clear                        Clear screen

  edit <file> [--tui]          Open Aeonmi editor (line mode or TUI)
  compile <file> [--emit js|ai] [--out PATH]
                               Compile/emit using your pipeline
  run <file> [--out PATH]      Compile to JS and try to run with Node

  exit | quit                  Leave Aeonmi Shard
"#;
    print!("{}", h.color(neon_dim()));
}

fn main() -> anyhow::Result<()> {
    // Make sure stdout is a console; show header
    print_centered_header().ok();

    // Start in current working directory (double-clicked exe inherits start-in)
    loop {
        prompt();
        let line = match read_line() {
            Ok(s) => s,
            Err(_) => break,
        };
        if line.is_empty() { continue; }

        let words = parse_words(&line);
        if words.is_empty() { continue; }
        let cmd = words[0].to_ascii_lowercase();
        let args = &words[1..];

        match cmd.as_str() {
            "help" | "h" | "?" => print_help(),

            "ls" => {
                let p = if args.is_empty() { pwd() } else { PathBuf::from(&args[0]) };
                if let Err(e) = list_dir(&p) { eprintln!("{}", format!("ls: {e}").red()); }
            }

            "cd" => {
                if let Some(p) = args.get(0) {
                    if let Err(e) = env::set_current_dir(p) {
                        eprintln!("{}", format!("cd: {e}").red());
                    }
                } else {
                    eprintln!("{}", "cd: missing path".red());
                }
            }

            "mkdir" => {
                if let Some(name) = args.get(0) {
                    if let Err(e) = fs::create_dir_all(name) {
                        eprintln!("{}", format!("mkdir: {e}").red());
                    }
                } else {
                    eprintln!("{}", "mkdir: missing name".red());
                }
            }

            "mv" => {
                if args.len() < 2 {
                    eprintln!("{}", "mv: usage mv <src> <dst>".red());
                } else if let Err(e) = fs::rename(&args[0], &args[1]) {
                    eprintln!("{}", format!("mv: {e}").red());
                }
            }

            "cat" => {
                if let Some(f) = args.get(0) {
                    if let Err(e) = cat_file(Path::new(f)) {
                        eprintln!("{}", format!("cat: {e}").red());
                    }
                } else {
                    eprintln!("{}", "cat: missing file".red());
                }
            }

            "clear" | "cls" => {
                let _ = execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0,0));
                let _ = print_centered_header();
            }

            "edit" => {
                if args.is_empty() {
                    eprintln!("{}", "edit: missing file".red());
                    continue;
                }
                let file = PathBuf::from(&args[0]);
                let use_tui = args.iter().any(|a| a == "--tui");
                if let Err(e) = commands::edit::main(Some(file), /*config*/ None, use_tui) {
                    eprintln!("{}", format!("edit: {e}").red());
                } else {
                    // redraw header after editor exits
                    let _ = print_centered_header();
                }
            }

            "compile" => {
                let joined: Vec<String> = std::iter::once("compile".to_string())
                    .chain(args.iter().cloned()).collect();
                if let Err(e) = compile_cmd(&joined) {
                    eprintln!("{}", format!("compile: {e}").red());
                }
            }

            "run" => {
                let joined: Vec<String> = std::iter::once("run".to_string())
                    .chain(args.iter().cloned()).collect();
                if let Err(e) = run_cmd(&joined) {
                    eprintln!("{}", format!("run: {e}").red());
                }
            }

            "exit" | "quit" => break,

            _ => eprintln!("{}", format!("unknown cmd: {cmd} (try 'help')").red()),
        }
    }

    Ok(())
}
