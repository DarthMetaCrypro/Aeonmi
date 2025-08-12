use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum EmitKind {
    #[clap(alias = "js")] Js,
    #[clap(alias = "ai")] Ai,
}
impl Default for EmitKind {
    fn default() -> Self { EmitKind::Js }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum BackendKind {
    #[clap(alias = "titan")] Titan,
    #[clap(alias = "aer")]   Aer,
    #[clap(alias = "ibmq")]  Ibmq,
}

#[derive(Debug, Parser)]
#[command(
    name = "aeonmi",
    about = "Aeonmi/QUBE/Titan — compile, run, quantum, and edit .ai",
    version,
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct AeonmiCli {
    /// Global: pretty diagnostics
    #[arg(long = "pretty-errors", action = ArgAction::SetTrue, global = true)]
    pub pretty_errors: bool,

    /// Global: disable semantic analysis
    #[arg(long = "no-sema", action = ArgAction::SetTrue, global = true)]
    pub no_sema: bool,

    /// Global: enable Titan library debug output
    #[arg(long = "debug-titan", action = ArgAction::SetTrue, global = true)]
    pub debug_titan: bool,

    /// Global: path to config (TOML); default: ~/.aeonmi/qpoly.toml
    #[arg(long = "config", value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    // Back-compat positional and -i/--input (legacy)
    #[arg(value_name = "input_pos")]
    pub input_pos: Option<PathBuf>,
    #[arg(short = 'i', long = "input", value_name = "FILE", hide = true)]
    pub input_opt: Option<PathBuf>,

    // Legacy top-level flags
    #[arg(long = "emit", value_name = "KIND", hide = true)]
    pub emit_legacy: Option<String>,
    #[arg(long = "out", value_name = "FILE", hide = true)]
    pub out_legacy: Option<PathBuf>,
    #[arg(long = "tokens", action = ArgAction::SetTrue, hide = true)]
    pub tokens_legacy: bool,
    #[arg(long = "ast", action = ArgAction::SetTrue, hide = true)]
    pub ast_legacy: bool,

    #[command(subcommand)]
    pub cmd: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Compile .ai to JS or validated .ai
    Compile {
        #[arg(value_name = "INPUT")]
        input: Option<PathBuf>,

        #[arg(long = "emit", value_enum, default_value_t = EmitKind::Js)]
        emit: EmitKind,

        #[arg(
            long = "out",
            value_name = "FILE",
            default_value = "output.js",
            default_value_if("emit", "ai", "output.ai")
        )]
        out: PathBuf,

        #[arg(long = "tokens", action = ArgAction::SetTrue)]
        tokens: bool,

        #[arg(long = "ast", action = ArgAction::SetTrue)]
        ast: bool,

        /// Enable Titan debug mode during compilation
        #[arg(long = "debug-titan", action = ArgAction::SetTrue)]
        debug_titan: bool,
    },

    /// Run an .ai file directly (compile-to-js + execute with Node if available)
    Run {
        #[arg(value_name = "INPUT")]
        input: PathBuf,
        #[arg(long = "out", value_name = "FILE")]
        out: Option<PathBuf>,
    },

    /// Quantum execution (Titan local or Qiskit backends)
    ///
    /// Usage:
    ///   aeonmi quantum titan FILE.ai
    ///   aeonmi quantum aer   FILE.ai --shots 2000
    Quantum {
        /// Backend: titan (local), aer (Qiskit Aer), ibmq (Qiskit cloud)
        #[arg(value_enum, value_name = "BACKEND")]
        backend: BackendKind,

        /// Input .ai file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Shots for sampling backends (ignored for Titan state simulation)
        #[arg(long = "shots", value_name = "N")]
        shots: Option<usize>,
    },

    /// Format .ai files
    Format {
        #[arg(value_name = "INPUTS")]
        inputs: Vec<PathBuf>,
        #[arg(long = "check", action = ArgAction::SetTrue)]
        check: bool,
    },

    /// Lint .ai files
    Lint {
        #[arg(value_name = "INPUTS")]
        inputs: Vec<PathBuf>,
        #[arg(long = "fix", action = ArgAction::SetTrue)]
        fix: bool,
    },

    /// Interactive REPL (placeholder for now)
    Repl,

    /// Editor: line-mode by default; pass --tui for TUI editor
    Edit {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        #[arg(long = "tui", action = ArgAction::SetTrue)]
        tui: bool,
    },

    /// Debug helpers
    Tokens { #[arg(value_name = "INPUT")] input: PathBuf },
    Ast    { #[arg(value_name = "INPUT")] input: PathBuf },
}
