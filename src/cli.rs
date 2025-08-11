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

#[derive(Debug, Parser)]
#[command(
    name = "aeonmi",
    about = "Aeonmi/QUBE/Titan unified tool — compile, run, and edit .ai",
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

    /// Global: path to config (TOML); default: ~/.aeonmi/qpoly.toml
    #[arg(long = "config", value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    // Back-compat positional and -i/--input (legacy)
    #[arg(value_name = "input_pos")]
    pub input_pos: Option<PathBuf>,
    #[arg(short = 'i', long = "input", value_name = "FILE", hide = true)]
    pub input_opt: Option<PathBuf>,

    // Legacy top-level flags to satisfy old tests/usage
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

        // Default out depends on --emit (ai -> output.ai, js -> output.js)
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
    },

    /// Run an .ai file directly (compile-to-js + execute with Node if available)
    Run {
        #[arg(value_name = "INPUT")]
        input: PathBuf,
        #[arg(long = "out", value_name = "FILE")]
        out: Option<PathBuf>,
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
