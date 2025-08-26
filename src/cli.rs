use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, ValueEnum)]
#[derive(Default)]
pub enum EmitKind {
    #[clap(alias = "js")]
    #[default]
    Js,
    #[clap(alias = "ai")]
    Ai,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum BackendKind {
    #[clap(alias = "titan")]
    Titan,
    #[clap(alias = "aer")]
    Aer,
    #[clap(alias = "ibmq")]
    Ibmq,
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
    /// Emit compiled output
    ///
    /// Examples:
    ///   aeonmi emit --format ai demo.qube -o out.ai
    ///   aeonmi emit demo.qube --format js -o out.js
    Emit {
        /// Input file (.qube / .ai)
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output format (alias: --format)
        #[arg(long = "emit", value_enum, default_value_t = EmitKind::Js, visible_alias = "format")]
        emit: EmitKind,

        /// Output file path (short: -o). Defaults by format.
        #[arg(
            short = 'o',
            long = "out",
            value_name = "FILE",
            default_value = "output.js",
            default_value_if("emit", "ai", "output.ai")
        )]
        out: PathBuf,

        /// Dump tokens (debug)
        #[arg(long = "tokens", action = ArgAction::SetTrue)]
        tokens: bool,

        /// Dump AST (debug)
        #[arg(long = "ast", action = ArgAction::SetTrue)]
        ast: bool,

        /// Enable Titan debug mode during compilation (overrides global if set)
        #[arg(long = "debug-titan", action = ArgAction::SetTrue)]
        debug_titan: bool,
    /// Watch input for changes and re-run the emit when modified
    #[arg(long = "watch", action = ArgAction::SetTrue)]
    watch: bool,
    },

    /// Run an .ai file directly (compile-to-js + execute with Node if available)
    Run {
        #[arg(value_name = "INPUT")]
        input: PathBuf,
        #[arg(long = "out", value_name = "FILE")]
        out: Option<PathBuf>,
    /// Watch input and re-run when changed
    #[arg(long = "watch", action = ArgAction::SetTrue)]
    watch: bool,
    /// Force native VM interpreter (no JS emit / Node). Env AEONMI_NATIVE=1 also works.
    #[arg(long = "native", action = ArgAction::SetTrue)]
    native: bool,
    /// Additionally emit canonical AI form to FILE (no JS) before executing (works with or without --native)
    #[arg(long = "emit-ai", value_name = "FILE")]
    emit_ai: Option<PathBuf>,
    },

    /// Quantum execution (Titan local or Qiskit backends)
    Quantum {
        #[arg(value_enum, value_name = "BACKEND")]
        backend: BackendKind,
        #[arg(value_name = "FILE")]
        file: PathBuf,
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

    /// File operations (new/open/save/import/export)
    New {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        /// Open editor after creating (line mode unless --tui)
        #[arg(long = "open", action = ArgAction::SetTrue)]
        open: bool,
        /// Open TUI editor after creating
        #[arg(long = "tui", action = ArgAction::SetTrue)]
        tui: bool,
        /// Compile immediately after creating (saves to output.js or output.ai depending on --emit js default)
        #[arg(long = "compile", action = ArgAction::SetTrue)]
        compile: bool,
        /// Run (implies JS target) after creation (compiles then runs with node)
        #[arg(long = "run", action = ArgAction::SetTrue)]
        run: bool,
    },
    Open {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    Save {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },
    SaveAs {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    Close {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },
    Import {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    Export {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(value_name = "FORMAT")]
        format: Option<String>,
    },
    Upload {
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
    Download {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Debug helpers
    Tokens {
        #[arg(value_name = "INPUT")]
        input: PathBuf,
    },
    Ast {
        #[arg(value_name = "INPUT")]
        input: PathBuf,
    },
    /// VM / Runtime control for the Aeonmi VM
    Vm {
        #[command(subcommand)]
        action: VmAction,
    },

    /// AI helpers (placeholders)
    Ai {
        #[command(subcommand)]
        action: AiAction,
    },

    /// Run a Cargo command (passthrough to system `cargo`). Example: aeonmi cargo build --release
    Cargo {
        #[arg(value_name = "ARGS", trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Run a Python command / script (passthrough to `python`). Example: aeonmi python script.py
    Python {
        #[arg(value_name = "ARGS", trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Run a Node.js command / script (passthrough to `node`). Example: aeonmi node file.js
    Node {
        #[arg(value_name = "ARGS", trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Auto-detect and run a file by extension (.ai, .js, .py, .rs)
    ///
    /// Examples:
    ///   aeonmi exec script.py arg1 arg2
    ///   aeonmi exec tool.js --flag
    ///   aeonmi exec module.rs            (temporary rustc build & run)
    ///   aeonmi exec program.ai           (compile to JS then node)
    Exec {
        /// File to execute (.ai | .js | .py | .rs)
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Additional arguments passed to the underlying runtime
        #[arg(value_name = "ARGS", trailing_var_arg = true)]
        args: Vec<String>,
    /// Watch the file and re-run on change
    #[arg(long = "watch", action = ArgAction::SetTrue)]
    watch: bool,
    /// Keep temporary compiled artifacts (e.g., __exec_tmp.js)
    #[arg(long = "keep-temp", action = ArgAction::SetTrue)]
    keep_temp: bool,
    /// (AI/JS only) Compile but skip executing node (useful for tests without node installed)
    #[arg(long = "no-run", action = ArgAction::SetTrue, hide = true)]
    no_run: bool,
    },

    /// Run an .ai file with the native VM (no JS / Node).
    Native {
        #[arg(value_name = "INPUT")]
        input: PathBuf,
        /// Additionally emit canonical AI form to FILE prior to execution.
        #[arg(long = "emit-ai", value_name = "FILE")]
        emit_ai: Option<PathBuf>,
        /// Watch file for changes and re-run.
        #[arg(long = "watch", action = ArgAction::SetTrue)]
        watch: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum VmAction {
    Start,
    Stop,
    Status,
    Reset,
    Snapshot { #[arg(value_name = "NAME")] name: String },
    Restore { #[arg(value_name = "NAME")] name: String },
    Mount { #[arg(value_name = "DIR")] dir: PathBuf },
}

#[derive(Debug, Subcommand)]
pub enum AiAction {
    Suggest,
    Debug,
    Optimize,
    Explain { #[arg(value_name = "SECTION")] section: Option<String> },
    Refactor { #[arg(value_name = "RULE")] rule: Option<String> },
    Chat {
        /// Provider name (openai, copilot, perplexity, deepseek)
        #[arg(long = "provider", value_name = "NAME")]
        provider: Option<String>,
        /// Prompt text (if omitted will read from stdin)
        #[arg(value_name = "PROMPT")] 
        prompt: Option<String>,
    /// List enabled providers instead of sending a prompt
    #[arg(long = "list", default_value_t = false)]
    list: bool,
    /// Stream tokens (currently OpenAI only)
    #[arg(long = "stream", default_value_t = false)]
    stream: bool,
    },
}
