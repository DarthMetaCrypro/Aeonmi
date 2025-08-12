use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::cli::EmitKind;
use crate::core::qpoly::{QPolyMap, default_config_path, ensure_parent_dir};
use super::compile::compile_pipeline;

// NEW
use crate::tui::editor::run_editor_tui;

pub fn main(
    file: Option<PathBuf>,
    config_path: Option<PathBuf>,
    use_tui: bool,
) -> anyhow::Result<()> {
    if use_tui {
        // TUI path: ensure we have a filepath (default if none), load QPolyMap, launch TUI.
        let filepath = file.unwrap_or_else(|| PathBuf::from("untitled.ai"));

        // Load QPoly map: explicit --config > default user path > built-in
        let qpoly = if let Some(p) = config_path.as_ref() {
            if p.exists() {
                match QPolyMap::from_toml_file(p) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("(warn) failed to load config {}: {e}", p.display());
                        QPolyMap::from_user_default_or_builtin()
                    }
                }
            } else {
                eprintln!("(warn) config path not found: {}", p.display());
                QPolyMap::from_user_default_or_builtin()
            }
        } else {
            QPolyMap::from_user_default_or_builtin()
        };

        // io::Result -> anyhow::Result via `?`
        run_editor_tui(filepath, qpoly)?;
        return Ok(());
    }

    // -------- legacy line editor below --------
    let mut filepath = file.unwrap_or_else(|| PathBuf::from("untitled.ai"));
    let mut buf = if filepath.exists() {
        fs::read_to_string(&filepath).unwrap_or_default()
    } else {
        String::new()
    };

    // Load QPoly map: explicit --config > default user path > built-in
    let map = if let Some(p) = config_path {
        if p.exists() {
            match QPolyMap::from_toml_file(&p) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("(warn) failed to load config {}: {e}", p.display());
                    QPolyMap::from_user_default_or_builtin()
                }
            }
        } else {
            eprintln!("(warn) config path not found: {}", p.display());
            QPolyMap::from_user_default_or_builtin()
        }
    } else {
        QPolyMap::from_user_default_or_builtin()
    };

    let dirty = Arc::new(AtomicBool::new(false));
    let running = Arc::new(AtomicBool::new(true));
    {
        let dirty = dirty.clone();
        let running = running.clone();
        ctrlc::set_handler(move || {
            if dirty.load(Ordering::Relaxed) {
                eprintln!("\n(unsaved changes — use :w or :wq)");
            } else {
                running.store(false, Ordering::Relaxed);
            }
        })?;
    }

    println!("Aeonmi edit — file: {}", filepath.display());
    println!("Type lines; commands start with ':'.  (:w, :q, :wq, :p, :compile, :run, :o <file>, :write-config)");

    let stdin = io::stdin();
    while running.load(Ordering::Relaxed) {
        print!("› ");
        io::stdout().flush().ok();

        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            if dirty.load(Ordering::Relaxed) {
                eprintln!("(unsaved changes — use :w or :wq)");
                continue;
            }
            break;
        }
        let line = line.trim_end_matches(&['\r', '\n'][..]).to_string();

        if line.starts_with(':') {
            match line.as_str() {
                ":p" => {
                    println!("--- buffer start ---");
                    print!("{}", buf);
                    if !buf.ends_with('\n') {
                        println!();
                    }
                    println!("--- buffer end ---");
                }
                ":w" => {
                    fs::write(&filepath, &buf)?;
                    dirty.store(false, Ordering::Relaxed);
                    println!("wrote {}", filepath.display());
                }
                ":wq" => {
                    fs::write(&filepath, &buf)?;
                    println!("wrote {}", filepath.display());
                    break;
                }
                ":q" => {
                    if dirty.load(Ordering::Relaxed) {
                        eprintln!("(unsaved changes — use :w or :wq)");
                        continue;
                    }
                    break;
                }
                ":compile" => {
                    let out = PathBuf::from("output.js");
                    compile_pipeline(
                        Some(filepath.clone()),
                        EmitKind::Js,
                        out,
                        false, // print_tokens
                        false, // print_ast
                        true,  // pretty
                        false, // skip_sema
                        false, // debug_titan
                    )?;
                }
                ":run" => {
                    let out = PathBuf::from("aeonmi.run.js");
                    compile_pipeline(
                        Some(filepath.clone()),
                        EmitKind::Js,
                        out.clone(),
                        false, // print_tokens
                        false, // print_ast
                        true,  // pretty
                        false, // skip_sema
                        false, // debug_titan
                    )?;
                    match std::process::Command::new("node").arg(&out).status() {
                        Ok(s) if !s.success() => eprintln!("(warn) node exit: {s}"),
                        Err(e) => eprintln!("(warn) node not available: {e}"),
                        _ => {}
                    }
                }
                other if other.starts_with(":o ") => {
                    let p = other[3..].trim();
                    let newp = PathBuf::from(p);
                    match fs::read_to_string(&newp) {
                        Ok(s) => {
                            filepath = newp;
                            buf = s;
                            dirty.store(false, Ordering::Relaxed);
                            println!("opened {}", filepath.display());
                        }
                        Err(e) => eprintln!("(err) open {}: {e}", newp.display()),
                    }
                }
                ":write-config" => {
                    if let Some(p) = default_config_path() {
                        ensure_parent_dir(&p)?;
                        let sample = r#"# Aeonmi QPoly chords (sample)
[[rules]]; chord="->"; glyph="→"
[[rules]]; chord="<="; glyph="≤"
[[rules]]; chord="!="; glyph="≠"
[[rules]]; chord="|0>"; glyph="∣0⟩"
[[rules]]; chord="|1>"; glyph="∣1⟩"
"#;
                        fs::write(&p, sample)?;
                        println!("wrote default config to {}", p.display());
                    } else {
                        eprintln!("(err) cannot resolve home directory");
                    }
                }
                _ => eprintln!("(cmd) unknown — try :w :q :wq :p :compile :run or :o <file>"),
            }
            continue;
        }

        let transformed = map.apply_line(&line);
        buf.push_str(&transformed);
        buf.push('\n');
        dirty.store(true, Ordering::Relaxed);
    }

    Ok(())
}
