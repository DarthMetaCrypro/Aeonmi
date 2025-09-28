use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::cli_vault::VaultCommand as VaultSubcommand;

#[derive(Copy, Clone, Debug, ValueEnum, Default)]
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

    /// Hidden: dump metrics JSON (fallback if subcommand shadowed)
    #[arg(long = "metrics-dump", action = ArgAction::SetTrue, hide = true, global = true)]
    pub metrics_dump_flag: bool,

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
        /// Use bytecode VM (feature: bytecode). Env AEONMI_BYTECODE=1 also works.
        #[arg(long = "bytecode", action = ArgAction::SetTrue)]
        bytecode: bool,
        /// Additionally emit canonical AI form to FILE (no JS) before executing (works with or without --native)
        #[arg(long = "emit-ai", value_name = "FILE")]
        emit_ai: Option<PathBuf>,
        /// Print optimization stats (bytecode mode only)
        #[arg(long = "opt-stats", action = ArgAction::SetTrue)]
        opt_stats: bool,
        /// Emit optimization stats JSON (implies --bytecode)
        #[arg(long = "opt-stats-json", action = ArgAction::SetTrue)]
        opt_stats_json: bool,
        /// Disassemble compiled bytecode (implies --bytecode)
        #[arg(long = "disasm", action = ArgAction::SetTrue)]
        disasm: bool,
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

    /// Domain Quantum Vault operations
    Vault {
        #[command(subcommand)]
        action: VaultSubcommand,
        /// Render Ratatui dashboard for supported subcommands
        #[arg(long = "tui", action = ArgAction::SetTrue)]
        tui: bool,
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

    /// Dump persisted metrics (call graph, variable deps, function timings, savings)
    #[command(name = "metrics-dump")]
    MetricsDump,

    /// Force immediate metrics persistence (bypass debounce) and then exit
    #[command(name = "metrics-flush")]
    MetricsFlush,

    /// Print the absolute path to the metrics JSON file
    #[command(name = "metrics-path")]
    MetricsPath,

    /// Show top N slowest functions by average inference time
    #[command(name = "metrics-top")]
    MetricsTop {
        /// Limit number of entries (default 10)
        #[arg(long = "limit", value_name = "N", default_value_t = 10)]
        limit: usize,
        /// Output JSON instead of table
        #[arg(long = "json", action = ArgAction::SetTrue)]
        json: bool,
    },

    /// Configure runtime metrics parameters (EMA alpha, window capacity)
    #[command(name = "metrics-config")]
    MetricsConfig {
        /// Set EMA alpha percent (1-100)
        #[arg(long = "set-ema", value_name = "PCT")]
        set_ema: Option<u64>,
        /// Set rolling window capacity (4-256)
        #[arg(long = "set-window", value_name = "N")]
        set_window: Option<usize>,
        /// Set savings history capacity (8-256)
        #[arg(long = "set-history-cap", value_name = "N")]
        set_history_cap: Option<usize>,
        /// Reset to defaults (ema=20, window=16)
        #[arg(long = "reset", action = ArgAction::SetTrue)]
        reset: bool,
        /// Show current config (implied if no setters)
        #[arg(long = "json", action = ArgAction::SetTrue)]
        json: bool,
    },

    /// Control deep propagation behavior for incremental analysis
    #[command(name = "metrics-deep")]
    MetricsDeep {
        /// Enable deep propagation
        #[arg(long = "enable", action = ArgAction::SetTrue)]
        enable: bool,
        /// Disable deep propagation
        #[arg(long = "disable", action = ArgAction::SetTrue)]
        disable: bool,
        /// Output JSON status
        #[arg(long = "json", action = ArgAction::SetTrue)]
        json: bool,
    },

    /// Re-encrypt all stored API keys with the current KDF/derivation (rotation/migration)
    #[command(name = "key-rotate")]
    KeyRotate {
        /// Output JSON report
        #[arg(long = "json", action = ArgAction::SetTrue)]
        json: bool,
    },

    /// List stored API key providers
    #[command(name = "key-list")]
    KeyList {
        /// JSON output
        #[arg(long = "json", action = ArgAction::SetTrue)]
        json: bool,
    },
    /// Get (decrypt) an API key for a provider
    #[command(name = "key-get")]
    KeyGet {
        #[arg(value_name = "PROVIDER")]
        provider: String,
        /// JSON output
        #[arg(long = "json", action = ArgAction::SetTrue)]
        json: bool,
    },
    /// Set (create/update) an API key for a provider
    #[command(name = "key-set")]
    KeySet {
        #[arg(value_name = "PROVIDER")]
        provider: String,
        #[arg(value_name = "KEY")]
        key: String,
    },
    /// Delete an API key for a provider
    #[command(name = "key-delete")]
    KeyDelete {
        #[arg(value_name = "PROVIDER")]
        provider: String,
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

    /// Benchmark / synthesize function inference metrics (requires feature: debug-metrics)
    #[cfg(feature = "debug-metrics")]
    #[command(name = "metrics-bench")]
    MetricsBench {
        /// Number of synthetic functions
        #[arg(long = "functions", value_name = "N", default_value_t = 5)]
        functions: usize,
        /// Samples per function
        #[arg(long = "samples", value_name = "N", default_value_t = 10)]
        samples: usize,
        /// Base duration ns for first sample
        #[arg(long = "base-ns", value_name = "NS", default_value_t = 1000)]
        base_ns: u128,
        /// Increment per sample
        #[arg(long = "step-ns", value_name = "NS", default_value_t = 100)]
        step_ns: u128,
        /// Add random jitter percent (0-100)
        #[arg(long = "jitter-pct", value_name = "PCT", default_value_t = 0)]
        jitter_pct: u64,
        /// Distribution: linear|exp
        #[arg(long = "dist", value_name = "KIND", default_value = "linear")]
        dist: String,
        /// Sort output by: ema|avg|last (no effect on generation order)
        #[arg(long = "sort", value_name = "FIELD", default_value = "ema")]
        sort: String,
        /// RNG seed (u64) for reproducibility (default fixed)
        #[arg(long = "seed", value_name = "SEED")]
        seed: Option<u64>,
        /// Reset metrics before benchmarking
        #[arg(long = "reset", action = ArgAction::SetTrue)]
        reset: bool,
        /// JSON output summary
        #[arg(long = "json", action = ArgAction::SetTrue)]
        json: bool,
        /// Also emit CSV to file (columns: index,runs,ema_ns,avg_ns,last_ns,total_ns)
        #[arg(long = "csv", value_name = "FILE")]
        csv: Option<std::path::PathBuf>,
    },

    /// Dump internal metrics state (windows, EMA, savings history) for debugging (feature: debug-metrics)
    #[cfg(feature = "debug-metrics")]
    #[command(name = "metrics-debug")]
    MetricsDebug {
        /// JSON output (always JSON currently)
        #[arg(long = "pretty", action = ArgAction::SetTrue)]
        pretty: bool,
    },

    /// Export function metrics to CSV (read-only; always available)
    #[command(name = "metrics-export")]
    MetricsExport {
        /// Output CSV file path
        #[arg(value_name = "FILE")]
        file: std::path::PathBuf,
    },

    /// Inject synthetic savings sample (test hook, feature debug-metrics)
    #[cfg(feature = "debug-metrics")]
    #[command(name = "metrics-inject-savings")]
    MetricsInjectSavings {
        #[arg(long = "partial", value_name = "NS")]
        partial: u128,
        #[arg(long = "full", value_name = "NS")]
        full: u128,
    },
    /// Inject synthetic function timing (test hook, feature debug-metrics)
    #[cfg(feature = "debug-metrics")]
    #[command(name = "metrics-inject-func")]
    MetricsInjectFunc {
        #[arg(long = "index", value_name = "I")]
        index: usize,
        #[arg(long = "dur", value_name = "NS")]
        dur: u128,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum VmAction {
    Start,
    Stop,
    Status,
    Reset,
    Snapshot { name: String },
    Restore { name: String },
    Mount { dir: std::path::PathBuf },
}

#[derive(Subcommand, Debug, Clone)]
pub enum AiAction {
    Suggest,
    Debug,
    Optimize,
    Explain {
        section: Option<String>,
    },
    Refactor {
        rule: Option<String>,
    },
    Chat {
        provider: Option<String>,
        prompt: Option<String>,
        list: bool,
        stream: bool,
    },
}
