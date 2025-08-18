/// Aeonmi/QUBE main — subcommands + back-compat + neon shell by default.

mod core;
mod cli;
mod commands;
mod config; // resolve_config_path, etc.
mod tui;    // tui::editor
mod shell;  // neon Shard shell

use clap::Parser; // trait import enables AeonmiCli::parse()
use std::path::PathBuf;

use crate::cli::{AeonmiCli, Command, EmitKind};
#[cfg(feature = "quantum")]
use crate::cli::BackendKind;
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
        return shell::start(cfg_path, args.pretty_errors, args.no_sema);
    }

    // Back-compat: `aeonmi <file>` or `-i <file>` behaves like compile.
    if args.cmd.is_none() && (args.input_pos.is_some() || args.input_opt.is_some()) {
        use std::process::exit as proc_exit;

        let input = args.input_pos.or(args.input_opt).unwrap();

        let emit_kind = match args.emit_legacy.as_deref() {
            None | Some("js") => EmitKind::Js,
            Some("ai") => EmitKind::Ai,
            Some(other) => { eprintln!("Unsupported --emit kind: {}", other); proc_exit(2); }
        };

        let default_out = if matches!(emit_kind, EmitKind::Ai) { "output.ai" } else { "output.js" };
        let out = args.out_legacy.clone().unwrap_or_else(|| PathBuf::from(default_out));

        return commands::compile::compile_pipeline(
            Some(input),
            emit_kind,
            out,
            args.tokens_legacy,
            args.ast_legacy,
            args.pretty_errors,
            args.no_sema,
            args.debug_titan, // pass debug flag through
        );
    }

    // Subcommands
    match args.cmd {
        // CLI defines `Emit`
        Some(Command::Emit { input, emit, out, tokens, ast, debug_titan }) => {
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

        Some(Command::Run { input, out }) => {
            commands::run::main_with_opts(input, out, args.pretty_errors, args.no_sema)
        }

        Some(Command::Quantum { backend, file, shots }) => {
            #[cfg(feature = "quantum")]
            {
                let backend_str = match backend {
                    BackendKind::Titan => "titan",
                    BackendKind::Aer   => "aer",
                    BackendKind::Ibmq  => "ibmq",
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
            // hook to your formatter when ready
            let _ = (inputs, check);
            println!("(format) placeholder");
            Ok(())
        }

        Some(Command::Lint { inputs, fix }) => {
            // hook to your linter when ready
            let _ = (inputs, fix);
            println!("(lint) placeholder");
            Ok(())
        }

        Some(Command::Repl) => commands::repl::main(),

        Some(Command::Edit { file, tui }) => commands::edit::main(file, cfg_path, tui),

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

        None => {
            use clap::CommandFactory;
            AeonmiCli::command().print_help().ok();
            println!();
            Ok(())
        }
    }
}
