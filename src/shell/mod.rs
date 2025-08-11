use colored::Colorize;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::cli::EmitKind;
use crate::commands;
use crate::commands::compile::compile_pipeline;

pub fn start(config_path: Option<PathBuf>, pretty: bool, skip_sema: bool) -> anyhow::Result<()> {
    banner();

    let mut cwd = std::env::current_dir()?;
    loop {
        // Prompt
        print!(
            "{} {} {} ",
            "⟦AEONMI⟧".bold().truecolor(225, 0, 180),
            cwd.display().to_string().truecolor(130, 0, 200),
            "›".truecolor(255, 240, 0)
        );
        io::stdout().flush().ok();

        // Read line
        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            println!();
            break;
        }
        let line = line.trim();
        if line.is_empty() { continue; }

        // Parse
        let mut parts = shell_words(line);
        if parts.is_empty() { continue; }
        let cmd = parts.remove(0);

        match cmd.as_str() {
            "help" | "?" => print_help(),
            "exit" | "quit" => break,

            // Navigation
            "pwd" => println!("{}", cwd.display()),
            "cd" => {
                let target = parts.get(0).map(PathBuf::from)
                    .unwrap_or_else(|| dirs_next::home_dir().unwrap_or(cwd.clone()));
                if let Err(e) = std::env::set_current_dir(&target) {
                    eprintln!("{} {}", "err:".red().bold(), e);
                } else {
                    cwd = std::env::current_dir()?;
                }
            }
            "ls" | "dir" => {
                let path = parts.get(0).map(PathBuf::from).unwrap_or_else(|| cwd.clone());
                match fs::read_dir(&path) {
                    Ok(rd) => {
                        for entry in rd.flatten() {
                            let p = entry.path();
                            let name = entry.file_name().to_string_lossy().into_owned();
                            if p.is_dir() {
                                println!("{}", format!("{name}/").truecolor(130, 0, 200));
                            } else {
                                println!("{name}");
                            }
                        }
                    }
                    Err(e) => eprintln!("{} {}: {}", "err:".red().bold(), path.display(), e),
                }
            }

            // FS ops
            "mkdir" => {
                if let Some(p) = parts.get(0) {
                    if let Err(e) = fs::create_dir_all(p) {
                        eprintln!("{} {}", "err:".red().bold(), e);
                    }
                } else { usage("mkdir <path>"); }
            }
            "rm" => {
                if let Some(p) = parts.get(0) {
                    let pb = Path::new(p);
                    let res = if pb.is_dir() { fs::remove_dir_all(pb) } else { fs::remove_file(pb) };
                    if let Err(e) = res { eprintln!("{} {}", "err:".red().bold(), e); }
                } else { usage("rm <path>"); }
            }
            "mv" => {
                if parts.len() < 2 { usage("mv <src> <dst>"); }
                else if let Err(e) = fs::rename(&parts[0], &parts[1]) {
                    eprintln!("{} {}", "err:".red().bold(), e);
                }
            }
            "cp" => {
                if parts.len() < 2 { usage("cp <src> <dst>"); }
                else if let Err(e) = fs::copy(&parts[0], &parts[1]).map(|_|()) {
                    eprintln!("{} {}", "err:".red().bold(), e);
                }
            }
            "cat" => {
                if let Some(p) = parts.get(0) {
                    match fs::read_to_string(p) {
                        Ok(s) => print!("{s}"),
                        Err(e) => eprintln!("{} {}", "err:".red().bold(), e),
                    }
                } else { usage("cat <file>"); }
            }

            // IDE-ish
            "edit" => {
                // edit [--tui] [--config FILE] [FILE]
                let mut tui = false;
                let mut file: Option<PathBuf> = None;
                let mut cfg = config_path.clone();
                let mut i = 0;
                while i < parts.len() {
                    match parts[i].as_str() {
                        "--tui" => { tui = true; i += 1; }
                        "--config" => {
                            if i+1 >= parts.len() { eprintln!("--config needs FILE"); break; }
                            cfg = Some(PathBuf::from(&parts[i+1]));
                            i += 2;
                        }
                        other => { file = Some(PathBuf::from(other)); i += 1; }
                    }
                }
                if let Err(e) = commands::edit::main(file, cfg, tui) {
                    eprintln!("{} {}", "err:".red().bold(), e);
                }
            }

            "compile" => {
                // compile <file.ai> [--emit js|ai] [--out FILE] [--no-sema]
                if parts.is_empty() { usage("compile <file.ai> [--emit js|ai] [--out FILE] [--no-sema]"); continue; }
                let mut input = PathBuf::from(&parts[0]);
                let mut emit = EmitKind::Js;
                let mut out = PathBuf::from("output.js");
                let mut j = 1;
                while j < parts.len() {
                    match parts[j].as_str() {
                        "--emit" if j+1 < parts.len() => {
                            emit = match parts[j+1].as_str() { "ai" => EmitKind::Ai, _ => EmitKind::Js };
                            if matches!(emit, EmitKind::Ai) { out = PathBuf::from("output.ai"); }
                            j += 2;
                        }
                        "--out" if j+1 < parts.len() => { out = PathBuf::from(&parts[j+1]); j += 2; }
                        "--no-sema" => { /* handled via skip_sema */ j += 1; }
                        other => { input = PathBuf::from(other); j += 1; }
                    }
                }
                if let Err(e) = compile_pipeline(Some(input), emit, out, false, false, pretty, skip_sema) {
                    eprintln!("{} {}", "err:".red().bold(), e);
                }
            }

            "run" => {
                // run <file.ai> [--out FILE]
                if parts.is_empty() { usage("run <file.ai> [--out FILE]"); continue; }
                let input = PathBuf::from(&parts[0]);
                let mut out: Option<PathBuf> = None;
                let mut j = 1;
                while j < parts.len() {
                    match parts[j].as_str() {
                        "--out" if j+1 < parts.len() => { out = Some(PathBuf::from(&parts[j+1])); j += 2; }
                        _ => { j += 1; }
                    }
                }
                if let Err(e) = commands::run::main_with_opts(input, out, pretty, skip_sema) {
                    eprintln!("{} {}", "err:".red().bold(), e);
                }
            }

            // Fallback
            other => eprintln!("{} unknown command: {other}", "err:".red().bold()),
        }
    }

    Ok(())
}

fn banner() {
    println!(
        "\n{}  {}\n{}  {}\n",
        "╔══════════════════════════════════════════════════╗".truecolor(225,0,180),
        "",
        "║                A e o n m i   S h a r d          ║".truecolor(255,240,0).bold(),
        "",
    );
    println!(
        "{}  {}",
        "╚══════════════════════════════════════════════════╝".truecolor(225,0,180),
        "type 'help' for commands".truecolor(130,0,200)
    );
}

fn print_help() {
    println!(
        "{}\n\
         {}\n  {}\n  {}\n  {}\n  {}\n  {}\n  {}\n\
         {}\n  {}\n  {}\n  {}\n  {}\n\
         {}\n  {}\n  {}\n\
         {}\n  {}\n",
        "Aeonmi Shard — commands".bold(),
        "Navigation:".truecolor(130,0,200),
        "pwd                 # print working dir",
        "cd [dir]            # change directory",
        "ls [dir]            # list directory",
        "mkdir <path>        # make directory",
        "mv <src> <dst>      # move/rename",
        "cp <src> <dst>      # copy file/dir",

        "Files:".truecolor(130,0,200),
        "cat <file>          # show file",
        "rm <path>           # remove file/dir",
        "edit [--tui] [FILE] # open editor (TUI with --tui)",
        "exit                # quit shell",

        "Build:".truecolor(130,0,200),
        "compile <file.ai> [--emit js|ai] [--out FILE] [--no-sema]",
        "run <file.ai> [--out FILE]     # compile to JS and try Node",

        "Help:".truecolor(130,0,200),
        "help                # show this help",
    );
}

fn usage(s: &str) {
    eprintln!("{} usage: {}", "usage:".yellow().bold(), s);
}

fn shell_words(s: &str) -> Vec<String> {
    // minimal split by whitespace respecting "quoted strings"
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut in_q = false;
    for c in s.chars() {
        match (c, in_q) {
            ('"', false) => in_q = true,
            ('"', true)  => in_q = false,
            (c, _) if c.is_whitespace() && !in_q => {
                if !buf.is_empty() { out.push(std::mem::take(&mut buf)); }
            }
            (c, _) => buf.push(c),
        }
    }
    if !buf.is_empty() { out.push(buf); }
    out
}
