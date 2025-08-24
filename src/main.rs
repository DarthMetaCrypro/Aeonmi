mod cli;
mod commands;
mod config; // resolve_config_path, etc.
/// Aeonmi/QUBE main — subcommands + back-compat + neon shell by default.
mod core;
mod io;
mod shell;
mod tui; // tui::editor // neon Shard shell

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
            backend,
            file,
            shots,
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
                Err(e) => Err(e.into()),
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

    Some(Command::New { file }) => commands::fs::new_file(file),
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
        },

        None => {
            use clap::CommandFactory;
            AeonmiCli::command().print_help().ok();
            println!();
            Ok(())
        }
    }
}
