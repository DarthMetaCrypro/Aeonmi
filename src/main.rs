mod cli;
mod commands;
mod config; // resolve_config_path, etc.
/// Aeonmi/QUBE main — subcommands + back-compat + neon shell by default.
mod core;
mod io;
mod shell;
mod tui; // tui::editor // neon Shard shell
mod ai; // AI provider registry & implementations

use clap::Parser; // trait import enables AeonmiCli::parse()
use std::path::PathBuf;

#[cfg(feature = "quantum")]
use crate::cli::BackendKind;
use crate::cli::{AeonmiCli, Command, EmitKind};

use crate::config::resolve_config_path;

fn set_console_title() {
    use crossterm::{execute, terminal::SetTitle};
    let _ = execute!(std::io::stdout(), SetTitle("Aeonmi Shard"));
}

fn main() -> anyhow::Result<()> {
    set_console_title();

    let args = AeonmiCli::parse();

    let cfg_path = resolve_config_path(&args.config);

    // If *no* subcommand and *no* legacy args: open the Aeonmi Shard shell.
    let no_legacy = args.input_pos.is_none()
        && args.input_opt.is_none()
        && args.emit_legacy.is_none()
        && args.out_legacy.is_none()
        && !args.tokens_legacy
        && !args.ast_legacy;

    if args.cmd.is_none() && no_legacy {
        // Start the neon shard shell as default interactive mode
        return shell::start(cfg_path, args.pretty_errors, args.no_sema);
    }

    // Backward compatibility: `aeonmi <file>` or `-i <file>` behaves like compile.
    if args.cmd.is_none() && (args.input_pos.is_some() || args.input_opt.is_some()) {
        use std::process::exit as proc_exit;

        let input = args.input_pos.or(args.input_opt).unwrap();

        let emit_kind = match args.emit_legacy.as_deref() {
            None | Some("js") => EmitKind::Js,
            Some("ai") => EmitKind::Ai,
            Some(other) => {
                eprintln!("Unsupported --emit kind: {}", other);
                proc_exit(2);
            }
        };

        let default_out = if matches!(emit_kind, EmitKind::Ai) {
            "output.ai"
        } else {
            "output.js"
        };

        let out = args
            .out_legacy
            .clone()
            .unwrap_or_else(|| PathBuf::from(default_out));

        return commands::compile::compile_pipeline(
            Some(input),
            emit_kind,
            out,
            args.tokens_legacy,
            args.ast_legacy,
            args.pretty_errors,
            args.no_sema,
            args.debug_titan,
        );
    }

    // Match and dispatch explicitly supported subcommands
    match args.cmd {
        Some(Command::Emit {
            input,
            emit,
            out,
            tokens,
            ast,
            debug_titan,
            watch,
        }) => {
            if watch {
                use std::time::{Duration, SystemTime};
                use std::thread::sleep;
                let mut last_mtime = std::fs::metadata(&input)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                loop {
                    let _ = commands::compile::compile_pipeline(
                        Some(input.clone()),
                        emit,
                        out.clone(),
                        tokens,
                        ast,
                        args.pretty_errors,
                        args.no_sema,
                        debug_titan,
                    );
                    sleep(Duration::from_millis(500));
                    if let Ok(meta) = std::fs::metadata(&input) {
                        if let Ok(m) = meta.modified() {
                            if m > last_mtime {
                                last_mtime = m;
                                println!("[watch] detected change, rebuilding...");
                                continue;
                            }
                        }
                    }
                }
            } else {
                commands::compile::compile_pipeline(
                    Some(input),
                    emit,
                    out,
                    tokens,
                    ast,
                    args.pretty_errors,
                    args.no_sema,
                    debug_titan,
                )
            }
        }

        Some(Command::Run { input, out, watch }) => {
            if watch {
                use std::time::{Duration, SystemTime};
                use std::thread::sleep;
                let mut last_mtime = std::fs::metadata(&input)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                loop {
                    let _ = commands::run::main_with_opts(input.clone(), out.clone(), args.pretty_errors, args.no_sema);
                    sleep(Duration::from_millis(500));
                    if let Ok(meta) = std::fs::metadata(&input) {
                        if let Ok(m) = meta.modified() {
                            if m > last_mtime {
                                last_mtime = m;
                                println!("[watch] detected change, rerunning...");
                                continue;
                            }
                        }
                    }
                }
            } else {
                commands::run::main_with_opts(input, out, args.pretty_errors, args.no_sema)
            }
        }

        Some(Command::Quantum {
            backend: backend @ _,
            file: file @ _,
            shots: shots @ _,
        }) => {
            #[cfg(feature = "quantum")]
            {
                let backend_str = match backend {
                    BackendKind::Titan => "titan",
                    BackendKind::Aer => "aer",
                    BackendKind::Ibmq => "ibmq",
                };
                return commands::quantum::quantum_run(file, backend_str, shots);
            }
            #[cfg(not(feature = "quantum"))]
            {
                eprintln!("The 'quantum' subcommand requires building with the `--features quantum` flag.");
                std::process::exit(2);
            }
        }

        Some(Command::Format { inputs, check }) => {
            // Call the batch formatter. It returns 0 when no files changed,
            // 1 when files were reformatted.
            match crate::commands::format::main(inputs, check) {
                Ok(code) => {
                    if code != 0 {
                        std::process::exit(code);
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }

        Some(Command::Lint { inputs, fix }) => {
            // TODO: hook to linter when ready
            let _ = (inputs, fix);
            println!("(lint) placeholder");
            Ok(())
        }

        Some(Command::Repl) => commands::repl::main(),

        Some(Command::Edit { file, tui }) => commands::edit::main(file, cfg_path, tui),

    Some(Command::New { file, open, tui, compile, run }) => {
        let created_path = file.clone();
        let res = commands::fs::new_file(file);
        if res.is_err() { return res; }
        if open {
            let _ = commands::edit::main(created_path.clone(), cfg_path.clone(), tui);
        }
        if compile || run {
            if let Some(p) = created_path.clone() {
                // Default to AI emit now (user request)
                let out_ai = PathBuf::from("output.ai");
                let _ = commands::compile::compile_pipeline(
                    Some(p.clone()),
                    EmitKind::Ai,
                    out_ai.clone(),
                    false,
                    false,
                    args.pretty_errors,
                    args.no_sema,
                    args.debug_titan,
                );
                if run {
                    // For run we still need JS path: compile JS then execute
                    let out_js = PathBuf::from("output.js");
                    let _ = commands::compile::compile_pipeline(
                        Some(p.clone()),
                        EmitKind::Js,
                        out_js.clone(),
                        false,
                        false,
                        args.pretty_errors,
                        args.no_sema,
                        args.debug_titan,
                    );
                    let _ = commands::run::main_with_opts(p, Some(out_js), args.pretty_errors, args.no_sema);
                }
            }
        }
        Ok(())
    },
    Some(Command::Open { file }) => commands::fs::open(file),
    Some(Command::Save { file }) => commands::fs::save(file),
    Some(Command::SaveAs { file }) => commands::fs::save_as(file),
    Some(Command::Close { file }) => commands::fs::close(file),
    Some(Command::Import { file }) => commands::fs::import(file),
    Some(Command::Export { file, format }) => commands::fs::export(file, format),
    Some(Command::Upload { path }) => commands::fs::upload(path),
    Some(Command::Download { file }) => commands::fs::download(file),

    Some(Command::Tokens { input }) => commands::compile::compile_pipeline(
            Some(input),
            EmitKind::Js,
            PathBuf::from("output.js"),
            /*tokens*/ true,
            /*ast*/ false,
            args.pretty_errors,
            args.no_sema,
            args.debug_titan,
        ),

        Some(Command::Ast { input }) => commands::compile::compile_pipeline(
            Some(input),
            EmitKind::Js,
            PathBuf::from("output.js"),
            /*tokens*/ false,
            /*ast*/ true,
            args.pretty_errors,
            args.no_sema,
            args.debug_titan,
        ),

        Some(Command::Vm { action }) => match action {
            crate::cli::VmAction::Start => commands::vm::start(),
            crate::cli::VmAction::Stop => commands::vm::stop(),
            crate::cli::VmAction::Status => commands::vm::status(),
            crate::cli::VmAction::Reset => commands::vm::reset(),
            crate::cli::VmAction::Snapshot { name } => commands::vm::snapshot(name),
            crate::cli::VmAction::Restore { name } => commands::vm::restore(name),
            crate::cli::VmAction::Mount { dir } => commands::vm::mount(dir),
        },

        Some(Command::Ai { action }) => match action {
            crate::cli::AiAction::Suggest => { println!("ai: suggest (placeholder)"); Ok(()) }
            crate::cli::AiAction::Debug => { println!("ai: debug (placeholder)"); Ok(()) }
            crate::cli::AiAction::Optimize => { println!("ai: optimize (placeholder)"); Ok(()) }
            crate::cli::AiAction::Explain { section } => { println!("ai: explain {:?}", section); Ok(()) }
            crate::cli::AiAction::Refactor { rule } => { println!("ai: refactor {:?}", rule); Ok(()) }
            crate::cli::AiAction::Chat { provider, prompt, list, stream } => {
                use crate::ai::AiRegistry;
                let reg = AiRegistry::new();
                if list {
                    let names = reg.list();
                    if names.is_empty() { println!("(no providers enabled) build with --features ai-openai,ai-copilot,..."); } else { for n in names { println!("{n}"); } }
                    return Ok(());
                }
                let prov_name = provider.or_else(|| reg.list().first().map(|s| s.to_string()));
                if prov_name.is_none() { println!("No AI providers enabled. Build with feature flags (e.g. --features ai-openai)"); return Ok(()); }
                let chosen = prov_name.unwrap();
                let prov = match reg.get(&chosen) { Some(p) => p, None => { println!("Provider '{}' not found in enabled set: {:?}", chosen, reg.list()); return Ok(()); } };
                let prompt_text = match prompt { Some(t) => t, None => {
                    use std::io::{self, Read}; let mut buf = String::new(); io::stdin().read_to_string(&mut buf).ok(); buf
                }};
                if stream {
                    let mut out = String::new();
                    let mut cb = |chunk: &str| { print!("{}", chunk); out.push_str(chunk); std::io::Write::flush(&mut std::io::stdout()).ok(); };
                    if let Err(e) = prov.chat_stream(&prompt_text, &mut cb) { eprintln!("chat error: {e}"); }
                    println!();
                } else {
                    match prov.chat(&prompt_text) { Ok(resp) => { println!("{}", resp); }, Err(e) => eprintln!("chat error: {e}"), }
                }
                Ok(())
            }
        },

        Some(Command::Cargo { args }) => {
            // Pass through to system cargo
            let status = std::process::Command::new("cargo").args(&args).status();
            match status {
                Ok(s) if s.success() => Ok(()),
                Ok(s) => {
                    anyhow::bail!("cargo exited with status {}", s);
                }
                Err(e) => anyhow::bail!("failed to execute cargo: {e}"),
            }
        }

        Some(Command::Python { args }) => {
            let exe = if cfg!(windows) { "python" } else { "python3" };
            let status = std::process::Command::new(exe).args(&args).status();
            match status {
                Ok(s) if s.success() => Ok(()),
                Ok(s) => anyhow::bail!("python exited with status {}", s),
                Err(e) => anyhow::bail!("failed to execute python: {e}"),
            }
        }

        Some(Command::Node { args }) => {
            let status = std::process::Command::new("node").args(&args).status();
            match status {
                Ok(s) if s.success() => Ok(()),
                Ok(s) => anyhow::bail!("node exited with status {}", s),
                Err(e) => anyhow::bail!("failed to execute node: {e}"),
            }
        }

    Some(Command::Exec { file, args: passthrough, watch, keep_temp, no_run }) => {
            use std::time::{Duration, SystemTime};
            use std::thread::sleep;
            fn run_once(
                file: &PathBuf,
                passthrough: &[String],
                pretty: bool,
                skip_sema: bool,
                debug_titan: bool,
                keep_temp: bool,
                no_run: bool,
            ) -> anyhow::Result<()> {
                let ext = file
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                match ext.as_str() {
                    "ai" => {
                        let out_js = PathBuf::from("__exec_tmp.js");
                        commands::compile::compile_pipeline(
                            Some(file.clone()),
                            EmitKind::Js,
                            out_js.clone(),
                            false,
                            false,
                            pretty,
                            skip_sema,
                            debug_titan,
                        )?;
                        if !no_run {
                            let status = std::process::Command::new("node")
                                .arg(&out_js)
                                .args(passthrough)
                                .status();
                            match status {
                                Ok(s) if s.success() => {}
                                Ok(s) => anyhow::bail!("node exited with status {}", s),
                                Err(e) => anyhow::bail!("failed to execute node: {e}"),
                            }
                        }
                        if !keep_temp {
                            let _ = std::fs::remove_file(&out_js);
                        }
                        Ok(())
                    }
                    "js" => {
                        if no_run {
                            return Ok(());
                        }
                        let status = std::process::Command::new("node")
                            .arg(&file)
                            .args(passthrough)
                            .status();
                        match status {
                            Ok(s) if s.success() => Ok(()),
                            Ok(s) => anyhow::bail!("node exited with status {}", s),
                            Err(e) => anyhow::bail!("failed to execute node: {e}"),
                        }
                    }
                    "py" => {
                        let exe = if cfg!(windows) { "python" } else { "python3" };
                        if no_run {
                            return Ok(());
                        }
                        let status = std::process::Command::new(exe)
                            .arg(&file)
                            .args(passthrough)
                            .status();
                        match status {
                            Ok(s) if s.success() => Ok(()),
                            Ok(s) => anyhow::bail!("python exited with status {}", s),
                            Err(e) => anyhow::bail!("failed to execute python: {e}"),
                        }
                    }
                    "rs" => {
                        let out_exe = if cfg!(windows) {
                            "__exec_tmp_rs.exe"
                        } else {
                            "__exec_tmp_rs"
                        };
                        let status_compile = std::process::Command::new("rustc")
                            .arg(&file)
                            .arg("-O")
                            .arg("-o")
                            .arg(out_exe)
                            .status();
                        match status_compile {
                            Ok(s) if s.success() => {
                                if !no_run {
                                    let status_run = std::process::Command::new(out_exe)
                                        .args(passthrough)
                                        .status();
                                    match status_run {
                                        Ok(s) if s.success() => {}
                                        Ok(s) =>
                                            anyhow::bail!("rust exec exited with status {}", s),
                                        Err(e) =>
                                            anyhow::bail!("failed to run rust exe: {e}"),
                                    }
                                }
                                if !keep_temp {
                                    let _ = std::fs::remove_file(if cfg!(windows) {
                                        "__exec_tmp_rs.exe"
                                    } else {
                                        "__exec_tmp_rs"
                                    });
                                }
                                Ok(())
                            }
                            Ok(s) => anyhow::bail!("rustc exited with status {}", s),
                            Err(e) => anyhow::bail!("failed to execute rustc: {e}"),
                        }
                    }
                    other => {
                        anyhow::bail!(
                            "Unsupported extension '{other}'. Supported: .ai .js .py .rs"
                        );
                    }
                }
            }
            if watch {
                let mut last_mtime = std::fs::metadata(&file)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                loop {
                    let _ = run_once(&file, &passthrough, args.pretty_errors, args.no_sema, args.debug_titan, keep_temp, no_run);
                    if std::env::var("AEONMI_WATCH_ONCE").ok().as_deref() == Some("1") { break; }
                    sleep(Duration::from_millis(500));
                    if let Ok(meta) = std::fs::metadata(&file) {
                        if let Ok(m) = meta.modified() {
                            if m > last_mtime {
                                last_mtime = m;
                                println!("[watch] change detected, re-running...");
                                continue;
                            }
                        }
                    }
                }
                Ok(())
            } else {
                run_once(&file, &passthrough, args.pretty_errors, args.no_sema, args.debug_titan, keep_temp, no_run)
            }
        }

        None => {
            use clap::CommandFactory;
            AeonmiCli::command().print_help().ok();
            println!();
            Ok(())
        }
    }
}
