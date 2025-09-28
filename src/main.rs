mod ai; // AI provider registry & implementations
mod cli;
mod cli_vault;
mod commands;
mod config; // resolve_config_path, etc.
/// Aeonmi/QUBE main — subcommands + back-compat + neon shell by default.
mod core;
mod encryption;
mod io;
mod shell;
mod tui; // tui::editor // neon Shard shell
mod vault;

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
    println!("DEBUG: main() called");
    set_console_title();

    let args = AeonmiCli::parse();

    let cfg_path = resolve_config_path(&args.config);

    // Guarantee a stub metrics file exists for tooling even before GUI loads.
    crate::core::incremental::ensure_metrics_file_exists();
    // Install shutdown flush guard so metrics are persisted on normal process exit.
    let _metrics_guard = crate::core::incremental::install_shutdown_flush_guard();
    // Ctrl-C handler to force immediate metrics persistence on abrupt termination.
    {
        let _ = ctrlc::set_handler(|| {
            crate::core::incremental::force_persist_metrics();
        });
    }

    // Early global metrics dump flag (hidden) for automation: --metrics-dump
    if args.metrics_dump_flag {
        crate::core::incremental::load_metrics();
        use crate::core::incremental::{
            get_deep_propagation, CALL_GRAPH_METRICS, FUNCTION_METRICS, SAVINGS_METRICS, VAR_DEPS,
        };
        let m = CALL_GRAPH_METRICS
            .lock()
            .ok()
            .map(|g| g.clone())
            .unwrap_or_default();
        let v = VAR_DEPS.lock().ok().map(|g| g.clone()).unwrap_or_default();
        let fm = FUNCTION_METRICS
            .lock()
            .ok()
            .map(|g| g.clone())
            .unwrap_or_default();
        let sm = SAVINGS_METRICS
            .lock()
            .ok()
            .map(|g| g.clone())
            .unwrap_or_default();
        let function_metrics: std::collections::HashMap<String, serde_json::Value> = fm.iter().map(|(idx, fm)| (
            idx.to_string(),
            serde_json::json!({ "runs": fm.runs, "total_ns": fm.total_ns, "last_ns": fm.last_ns, "avg_ns": if fm.runs>0 { fm.total_ns / fm.runs as u128 } else { 0 } })
        )).collect();
        let json = serde_json::json!({
            "version": 3,
            "metrics": {"functions": m.functions, "edges": m.edges, "reinfer_events": m.reinfer_events, "variable_edges": m.variable_edges},
            "varReads": v.reads.iter().map(|(k, set)| (k, set.iter().collect::<Vec<_>>())).collect::<std::collections::HashMap<_,_>>(),
            "varWrites": v.writes.iter().map(|(k, set)| (k, set.iter().collect::<Vec<_>>())).collect::<std::collections::HashMap<_,_>>(),
            "functionMetrics": function_metrics,
            "deepPropagation": get_deep_propagation(),
            "savings": {"cumulative_savings_ns": sm.cumulative_savings_ns, "cumulative_partial_ns": sm.cumulative_partial_ns, "cumulative_estimated_full_ns": sm.cumulative_estimated_full_ns}
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(());
    }

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
                use std::thread::sleep;
                use std::time::{Duration, SystemTime};
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

        Some(Command::Run {
            input,
            out,
            watch,
            native,
            emit_ai,
            bytecode,
            opt_stats,
            opt_stats_json,
            disasm,
        }) => {
            if watch {
                use std::thread::sleep;
                use std::time::{Duration, SystemTime};
                let mut last_mtime = std::fs::metadata(&input)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                loop {
                    let _ = {
                        // Optional AI emit only
                        if let Some(ai_path) = &emit_ai {
                            let _ = commands::compile::compile_pipeline(
                                Some(input.clone()),
                                EmitKind::Ai,
                                ai_path.clone(),
                                false,
                                false,
                                args.pretty_errors,
                                args.no_sema,
                                args.debug_titan,
                            );
                        }
                        if native || std::env::var("AEONMI_NATIVE").ok().as_deref() == Some("1") {
                            std::env::set_var("AEONMI_NATIVE", "1");
                            crate::commands::run::run_native(
                                &input,
                                args.pretty_errors,
                                args.no_sema,
                            )
                        } else {
                            commands::run::main_with_opts(
                                input.clone(),
                                out.clone(),
                                args.pretty_errors,
                                args.no_sema,
                            )
                        }
                    };
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
                // Single run
                if let Some(ai_path) = &emit_ai {
                    let _ = commands::compile::compile_pipeline(
                        Some(input.clone()),
                        EmitKind::Ai,
                        ai_path.clone(),
                        false,
                        false,
                        args.pretty_errors,
                        args.no_sema,
                        args.debug_titan,
                    );
                }
                if cfg!(feature = "bytecode")
                    && (bytecode
                        || disasm
                        || opt_stats
                        || opt_stats_json
                        || std::env::var("AEONMI_BYTECODE").ok().as_deref() == Some("1"))
                {
                    #[cfg(feature = "bytecode")]
                    {
                        use crate::core::{
                            bytecode::{disassemble, BytecodeCompiler},
                            lexer::Lexer,
                            parser::Parser,
                            vm_bytecode::VM,
                        };
                        let source = match std::fs::read_to_string(&input) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("read error: {e}");
                                String::new()
                            }
                        };
                        let mut lex = Lexer::from_str(&source);
                        let toks = match lex.tokenize() {
                            Ok(t) => t,
                            Err(e) => {
                                eprintln!("lex error: {e}");
                                return Ok(());
                            }
                        };
                        let mut p = Parser::new(toks);
                        let ast = match p.parse() {
                            Ok(a) => a,
                            Err(e) => {
                                eprintln!("parse error: {e}");
                                return Ok(());
                            }
                        };
                        let chunk = BytecodeCompiler::new().compile(&ast);
                        if disasm {
                            println!("{}", disassemble(&chunk));
                        }
                        let mut vm = VM::new(&chunk);
                        let result = vm.run();
                        if let Some(r) = result {
                            println!("bytecode result: {:?}", r);
                        }
                        if opt_stats_json {
                            // Build JSON object (also attempt to merge into runtime metrics if available)
                            let compile_stats = serde_json::json!({
                                "const_folds": chunk.opt_stats.const_folds,
                                "chain_folds": chunk.opt_stats.chain_folds,
                                "dce_if": chunk.opt_stats.dce_if,
                                "dce_while": chunk.opt_stats.dce_while,
                                "dce_for": chunk.opt_stats.dce_for,
                                "pops_eliminated": chunk.opt_stats.pops_eliminated
                            });
                            // If debug-metrics feature active, stitch into metrics JSON (best-effort, do not fail)
                            #[cfg(feature = "debug-metrics")]
                            {
                                use crate::core::incremental::build_metrics_json;
                                let mut metrics_root = build_metrics_json();
                                metrics_root["compileOptStats"] = compile_stats.clone();
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(&metrics_root)
                                        .unwrap_or_else(|_| "{}".into())
                                );
                            }
                            #[cfg(not(feature = "debug-metrics"))]
                            {
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(&compile_stats)
                                        .unwrap_or_else(|_| "{}".into())
                                );
                            }
                        } else if opt_stats {
                            println!("opt_stats const_folds={} chain_folds={} dce_if={} dce_while={} dce_for={} pops_eliminated={}", chunk.opt_stats.const_folds, chunk.opt_stats.chain_folds, chunk.opt_stats.dce_if, chunk.opt_stats.dce_while, chunk.opt_stats.dce_for, chunk.opt_stats.pops_eliminated);
                        }
                        if vm.stack_overflow {
                            eprintln!("warning: stack overflow detected (frame limit)");
                        }
                    }
                } else if native || std::env::var("AEONMI_NATIVE").ok().as_deref() == Some("1") {
                    std::env::set_var("AEONMI_NATIVE", "1");
                    return commands::run::main_with_opts(
                        input,
                        out,
                        args.pretty_errors,
                        args.no_sema,
                    );
                } else {
                    return commands::run::main_with_opts(
                        input,
                        out,
                        args.pretty_errors,
                        args.no_sema,
                    );
                }
                Ok(())
            }
        }

        Some(Command::Quantum {
            backend: _,
            file: _,
            shots: _,
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

        Some(Command::New {
            file,
            open,
            tui,
            compile,
            run,
        }) => {
            let created_path = file.clone();
            let res = commands::fs::new_file(file);
            if res.is_err() {
                return res;
            }
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
                        let _ = commands::run::main_with_opts(
                            p,
                            Some(out_js),
                            args.pretty_errors,
                            args.no_sema,
                        );
                    }
                }
            }
            Ok(())
        }
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

        Some(Command::Ai { action }) => {
            match action {
                crate::cli::AiAction::Suggest => {
                    println!("ai: suggest (placeholder)");
                    Ok(())
                }
                crate::cli::AiAction::Debug => {
                    println!("ai: debug (placeholder)");
                    Ok(())
                }
                crate::cli::AiAction::Optimize => {
                    println!("ai: optimize (placeholder)");
                    Ok(())
                }
                crate::cli::AiAction::Explain { section } => {
                    println!("ai: explain {:?}", section);
                    Ok(())
                }
                crate::cli::AiAction::Refactor { rule } => {
                    println!("ai: refactor {:?}", rule);
                    Ok(())
                }
                crate::cli::AiAction::Chat {
                    provider,
                    prompt,
                    list,
                    stream,
                } => {
                    use crate::ai::AiRegistry;
                    let reg = AiRegistry::new();
                    if list {
                        let names = reg.list();
                        if names.is_empty() {
                            println!("(no providers enabled) build with --features ai-openai,ai-copilot,...");
                        } else {
                            for n in names {
                                println!("{n}");
                            }
                        }
                        return Ok(());
                    }
                    let prov_name = provider.or_else(|| reg.list().first().map(|s| s.to_string()));
                    if prov_name.is_none() {
                        println!("No AI providers enabled. Build with feature flags (e.g. --features ai-openai)");
                        return Ok(());
                    }
                    let chosen = prov_name.unwrap();
                    let prov = match reg.get(&chosen) {
                        Some(p) => p,
                        None => {
                            println!(
                                "Provider '{}' not found in enabled set: {:?}",
                                chosen,
                                reg.list()
                            );
                            return Ok(());
                        }
                    };
                    let prompt_text = match prompt {
                        Some(t) => t,
                        None => {
                            use std::io::{self, Read};
                            let mut buf = String::new();
                            io::stdin().read_to_string(&mut buf).ok();
                            buf
                        }
                    };
                    if stream {
                        let mut out = String::new();
                        let mut cb = |chunk: &str| {
                            print!("{}", chunk);
                            out.push_str(chunk);
                            std::io::Write::flush(&mut std::io::stdout()).ok();
                        };
                        if let Err(e) = prov.chat_stream(&prompt_text, &mut cb) {
                            eprintln!("chat error: {e}");
                        }
                        println!();
                    } else {
                        match prov.chat(&prompt_text) {
                            Ok(resp) => {
                                println!("{}", resp);
                            }
                            Err(e) => eprintln!("chat error: {e}"),
                        }
                    }
                    Ok(())
                }
            }
        }

        Some(Command::Vault { action, tui }) => commands::vault::dispatch(action, tui),

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

        Some(Command::MetricsDump) => {
            // Load metrics from disk (populate globals) then emit combined JSON identical to persist format
            crate::core::incremental::load_metrics();
            let json = crate::core::incremental::build_metrics_json();
            println!(
                "{}",
                serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())
            );
            Ok(())
        }

        Some(Command::MetricsFlush) => {
            crate::core::incremental::force_persist_metrics();
            let json = crate::core::incremental::build_metrics_json();
            println!(
                "metrics flushed\n{}",
                serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())
            );
            Ok(())
        }

        Some(Command::MetricsPath) => {
            println!(
                "{}",
                crate::core::incremental::metrics_file_location().display()
            );
            Ok(())
        }

        Some(Command::MetricsTop { limit, json }) => {
            crate::core::incremental::load_metrics();
            use crate::core::incremental::{get_deep_propagation, FUNCTION_METRICS};
            let data = FUNCTION_METRICS
                .lock()
                .ok()
                .map(|g| g.clone())
                .unwrap_or_default();
            // rows: idx, last, total, runs, avg, ema
            let mut rows: Vec<(usize, u128, u128, u64, u128, u128)> = data
                .into_iter()
                .map(|(idx, m)| {
                    let avg = if m.runs > 0 {
                        m.total_ns / m.runs as u128
                    } else {
                        0
                    };
                    (idx, m.last_ns, m.total_ns, m.runs, avg, m.ema_ns)
                })
                .collect();
            // Sort by EMA (recent performance) descending fallback to avg
            rows.sort_by(|a, b| b.5.cmp(&a.5).then(b.4.cmp(&a.4)));
            rows.truncate(limit);
            if json {
                let j: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|(i, last, total, runs, avg, ema)| {
                        serde_json::json!({
                            "index": i,
                            "runs": runs,
                            "last_ns": last,
                            "total_ns": total,
                            "avg_ns": avg,
                            "ema_ns": ema
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&j).unwrap_or_else(|_| "[]".to_string())
                );
            } else {
                use crate::core::incremental::{EMA_ALPHA_RUNTIME, WINDOW_CAP_RUNTIME};
                let alpha = EMA_ALPHA_RUNTIME.load(std::sync::atomic::Ordering::Relaxed);
                let win = WINDOW_CAP_RUNTIME.load(std::sync::atomic::Ordering::Relaxed);
                println!(
                    "(deepPropagation={} ema_alpha={} window={})",
                    get_deep_propagation(),
                    alpha,
                    win
                );
                println!("idx  runs  ema_ns    avg_ns    last_ns   total_ns");
                for (i, last, total, runs, avg, ema) in rows {
                    println!("{i:<4} {runs:<5} {ema:<9} {avg:<9} {last:<9} {total}");
                }
            }
            Ok(())
        }

        Some(Command::MetricsConfig {
            set_ema,
            set_window,
            set_history_cap,
            reset,
            json,
        }) => {
            use crate::core::incremental::{
                reset_runtime_metrics_config, set_ema_alpha, set_history_cap as set_hist_cap,
                set_window_capacity, EMA_ALPHA_RUNTIME, SAVINGS_METRICS, WINDOW_CAP_RUNTIME,
            };
            if reset {
                reset_runtime_metrics_config();
            }
            if let Some(a) = set_ema {
                set_ema_alpha(a);
            }
            if let Some(w) = set_window {
                set_window_capacity(w);
            }
            if let Some(h) = set_history_cap {
                set_hist_cap(h);
            }
            let alpha = EMA_ALPHA_RUNTIME.load(std::sync::atomic::Ordering::Relaxed);
            let win = WINDOW_CAP_RUNTIME.load(std::sync::atomic::Ordering::Relaxed);
            let hist_cap = {
                let sm = SAVINGS_METRICS.lock().unwrap();
                sm.history_cap
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({"ema_alpha": alpha, "window": win, "history_cap": hist_cap, "reset": reset})).unwrap());
            } else {
                println!(
                    "ema_alpha={} window={} history_cap={}{}",
                    alpha,
                    win,
                    hist_cap,
                    if reset { " (reset)" } else { "" }
                );
            }
            Ok(())
        }

        Some(Command::MetricsDeep {
            enable,
            disable,
            json,
        }) => {
            use crate::core::incremental::{get_deep_propagation, set_deep_propagation};
            if enable {
                set_deep_propagation(true);
            }
            if disable {
                set_deep_propagation(false);
            }
            let status = get_deep_propagation();
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({"deepPropagation": status}))
                        .unwrap()
                );
            } else {
                println!("deepPropagation={}", status);
            }
            Ok(())
        }

        Some(Command::KeyRotate { json }) => {
            use crate::core::api_keys::rotate_all_keys;
            match rotate_all_keys() {
                Ok(report) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "attempted": report.attempted,
                                "rotated": report.rotated,
                                "errors": report.errors
                            }))
                            .unwrap()
                        );
                    } else {
                        println!(
                            "Key rotation complete: rotated {}/{} providers",
                            report.rotated, report.attempted
                        );
                        if !report.errors.is_empty() {
                            eprintln!("Errors ({}):", report.errors.len());
                            for (prov, err) in report.errors {
                                eprintln!("  {prov}: {err}");
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("key rotation error: {e}");
                    anyhow::bail!(e)
                }
            }
        }
        Some(Command::KeyList { json }) => {
            use crate::core::api_keys::list_providers;
            let list = list_providers();
            if json {
                println!("{}", serde_json::to_string_pretty(&list).unwrap());
            } else {
                for p in list {
                    println!("{p}");
                }
            }
            Ok(())
        }
        Some(Command::KeyGet { provider, json }) => {
            use crate::core::api_keys::get_api_key;
            let val = get_api_key(&provider);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &serde_json::json!({"provider": provider, "key": val})
                    )
                    .unwrap()
                );
            } else {
                match val {
                    Some(k) => println!("{k}"),
                    None => eprintln!("(not found)"),
                }
            }
            Ok(())
        }
        Some(Command::KeySet { provider, key }) => {
            use crate::core::api_keys::set_api_key;
            set_api_key(&provider, &key).map_err(|e| anyhow::anyhow!(e))?;
            println!("stored key for {provider}");
            Ok(())
        }
        Some(Command::KeyDelete { provider }) => {
            use crate::core::api_keys::delete_api_key;
            delete_api_key(&provider).map_err(|e| anyhow::anyhow!(e))?;
            println!("deleted key for {provider}");
            Ok(())
        }

        #[cfg(feature = "debug-metrics")]
        Some(Command::MetricsBench {
            functions,
            samples,
            base_ns,
            step_ns,
            jitter_pct,
            dist,
            sort,
            seed,
            reset,
            json,
            csv,
        }) => {
            use crate::core::incremental::{
                build_metrics_json, record_function_infer, reset_metrics_full,
            };
            if reset {
                reset_metrics_full();
            }
            use rand::rngs::StdRng;
            use rand::{Rng, SeedableRng};
            let fallback_env_seed = std::env::var("AEONMI_SEED")
                .ok()
                .and_then(|s| s.parse::<u64>().ok());
            let eff_seed = seed.or(fallback_env_seed).unwrap_or(0xC0FFEE);
            let mut rng = StdRng::seed_from_u64(eff_seed);
            for f in 0..functions {
                for s in 0..samples {
                    let base_progress = match dist.as_str() {
                        "exp" => ((s + 1) as f64).exp() / ((samples) as f64).exp(),
                        _ => (s as f64) / (samples.max(1) as f64),
                    };
                    let mut dur = base_ns
                        + step_ns * (s as u128)
                        + (f as u128 * step_ns / 2)
                        + (base_progress * step_ns as f64) as u128;
                    if jitter_pct > 0 {
                        let jitter = (dur as f64 * (jitter_pct.min(100) as f64) / 100.0) as u128;
                        let delta: i128 = rng.gen_range(-(jitter as i128)..=(jitter as i128));
                        dur = (dur as i128 + delta).max(1) as u128;
                    }
                    record_function_infer(f, dur);
                }
            }
            let mut summary = build_metrics_json();
            if let Some(csv_path) = csv {
                if let Some(obj) = summary.get("functionMetrics").and_then(|v| v.as_object()) {
                    let mut rows: Vec<(usize, u64, u128, u128, u128, u128)> = Vec::new();
                    for (k, v) in obj {
                        if let Ok(idx) = k.parse::<usize>() {
                            let runs = v.get("runs").and_then(|x| x.as_u64()).unwrap_or(0);
                            let ema = v.get("ema_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128;
                            let avg = v.get("avg_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128;
                            let last =
                                v.get("last_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128;
                            let total =
                                v.get("total_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128;
                            rows.push((idx, runs, ema, avg, last, total));
                        }
                    }
                    match sort.as_str() {
                        "avg" => rows.sort_by(|a, b| b.3.cmp(&a.3)),
                        "last" => rows.sort_by(|a, b| b.4.cmp(&a.4)),
                        _ => rows.sort_by(|a, b| b.2.cmp(&a.2)),
                    };
                    let mut csv_data = String::from("index,runs,ema_ns,avg_ns,last_ns,total_ns\n");
                    for (i, r, e, a, l, t) in rows {
                        csv_data.push_str(&format!("{i},{r},{e},{a},{l},{t}\n"));
                    }
                    let _ = std::fs::write(csv_path, csv_data);
                }
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&summary).unwrap());
            } else {
                println!("bench recorded functions={} samples={} windowCapacity={} dist={} jitter_pct={} seed={}", functions, samples, summary.get("windowCapacity").and_then(|v| v.as_u64()).unwrap_or(0), dist, jitter_pct, eff_seed);
            }
            Ok(())
        }
        #[cfg(feature = "debug-metrics")]
        Some(Command::MetricsDebug { pretty }) => {
            use crate::core::incremental::debug_snapshot;
            let snap = debug_snapshot();
            if pretty {
                println!("{}", serde_json::to_string_pretty(&snap).unwrap());
            } else {
                println!("{}", snap);
            }
            Ok(())
        }
        Some(Command::MetricsExport { file }) => {
            use crate::core::incremental::build_metrics_json;
            let json = build_metrics_json();
            if let Some(obj) = json.get("functionMetrics").and_then(|v| v.as_object()) {
                let mut rows: Vec<(usize, u64, u128, u128, u128, u128)> = Vec::new();
                for (k, v) in obj {
                    if let Ok(idx) = k.parse::<usize>() {
                        let runs = v.get("runs").and_then(|x| x.as_u64()).unwrap_or(0);
                        let ema = v.get("ema_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128;
                        let avg = v.get("avg_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128;
                        let last = v.get("last_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128;
                        let total = v.get("total_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128;
                        rows.push((idx, runs, ema, avg, last, total));
                    }
                }
                rows.sort_by(|a, b| b.2.cmp(&a.2));
                let mut csv = String::from("index,runs,ema_ns,avg_ns,last_ns,total_ns\n");
                for (i, r, e, a, l, t) in rows {
                    csv.push_str(&format!("{i},{r},{e},{a},{l},{t}\n"));
                }
                if let Err(e) = std::fs::write(&file, csv) {
                    eprintln!("export error: {e}");
                } else {
                    println!("exported {}", file.display());
                }
            } else {
                eprintln!("no function metrics");
            }
            Ok(())
        }
        #[cfg(feature = "debug-metrics")]
        Some(Command::MetricsInjectSavings { partial, full }) => {
            use crate::core::incremental::record_savings;
            record_savings(partial, full);
            println!("injected savings partial={partial} full={full}");
            Ok(())
        }
        #[cfg(feature = "debug-metrics")]
        Some(Command::MetricsInjectFunc { index, dur }) => {
            use crate::core::incremental::record_function_infer;
            record_function_infer(index, dur);
            println!("injected func index={index} dur={dur}");
            Ok(())
        }

        Some(Command::Exec {
            file,
            args: passthrough,
            watch,
            keep_temp,
            no_run,
        }) => {
            use std::thread::sleep;
            use std::time::{Duration, SystemTime};
            // Because the Exec command uses a trailing var arg to allow arbitrary program args,
            // flags like --keep-temp / --no-run placed after the file name are captured inside
            // passthrough. Tests pass them that way, so we detect and elevate them here.
            let mut keep_temp_flag = keep_temp;
            let mut no_run_flag = no_run;
            let mut passthrough_filtered: Vec<String> = Vec::new();
            for a in &passthrough {
                match a.as_str() {
                    "--keep-temp" => keep_temp_flag = true,
                    "--no-run" => no_run_flag = true,
                    _ => passthrough_filtered.push(a.clone()),
                }
            }
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
                        let force_native =
                            std::env::var("AEONMI_NATIVE").ok().as_deref() == Some("1");
                        let node_available = std::process::Command::new("node")
                            .arg("--version")
                            .output()
                            .map(|o| o.status.success())
                            .unwrap_or(false);
                        if force_native || !node_available {
                            if no_run {
                                // Even in native/ no node environment, honor --no-run by producing JS artifact for tests.
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
                                if !keep_temp {
                                    let _ = std::fs::remove_file(&out_js);
                                }
                                Ok(())
                            } else {
                                // Native interpretation path
                                println!(
                                    "DEBUG: EXEC PATH - native: executing '{}' via Aeonmi VM",
                                    file.display()
                                );
                                use crate::core::diagnostics::{
                                    emit_json_error, print_error, Span,
                                };
                                use crate::core::lexer::Lexer;
                                use crate::core::lexer::LexerError;
                                use crate::core::lowering::lower_ast_to_ir;
                                use crate::core::parser::{Parser as AeParser, ParserError};
                                use crate::core::vm::Interpreter;
                                let src = match std::fs::read_to_string(file) {
                                    Ok(s) => s,
                                    Err(e) => anyhow::bail!("read error: {e}"),
                                };
                                let mut lexer = Lexer::from_str(&src);
                                let tokens = match lexer.tokenize() {
                                    Ok(t) => t,
                                    Err(e) => {
                                        if pretty {
                                            match e {
                                                LexerError::UnexpectedCharacter(_, line, col)
                                                | LexerError::UnterminatedString(line, col)
                                                | LexerError::InvalidNumber(_, line, col)
                                                | LexerError::InvalidQubitLiteral(_, line, col)
                                                | LexerError::UnterminatedComment(line, col) => {
                                                    emit_json_error(
                                                        &file.display().to_string(),
                                                        &format!("{}", e),
                                                        &Span::single(line, col),
                                                    );
                                                    print_error(
                                                        &file.display().to_string(),
                                                        &src,
                                                        &format!("{}", e),
                                                        Span::single(line, col),
                                                    );
                                                }
                                                _ => eprintln!("lex error: {e}"),
                                            }
                                        } else {
                                            eprintln!("lex error: {e}");
                                        }
                                        return Ok(());
                                    }
                                };
                                let mut parser = AeParser::new(tokens.clone());
                                let ast = match parser.parse() {
                                    Ok(a) => a,
                                    Err(ParserError {
                                        message,
                                        line,
                                        column,
                                    }) => {
                                        if pretty {
                                            emit_json_error(
                                                &file.display().to_string(),
                                                &format!("Parsing error: {message}"),
                                                &Span::single(line, column),
                                            );
                                            print_error(
                                                &file.display().to_string(),
                                                &src,
                                                &format!("Parsing error: {message}"),
                                                Span::single(line, column),
                                            );
                                        } else {
                                            eprintln!("parse error: {message}");
                                        }
                                        return Ok(());
                                    }
                                };
                                if skip_sema {
                                    println!("note: semantic analysis skipped (native)");
                                }
                                match lower_ast_to_ir(&ast, "main") {
                                    Ok(module) => {
                                        println!("DEBUG: About to call run_module in main.rs");
                                        let mut interp = Interpreter::new();
                                        if !no_run {
                                            if let Err(e) = interp.run_module(&module) {
                                                eprintln!("TEST ERROR: {}", e.message);
                                            }
                                        }
                                    }
                                    Err(e) => eprintln!("lowering error: {e}"),
                                }
                                Ok(())
                            }
                        } else {
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
                            // If user only wants compilation (--no-run) and didn't request keep-temp, remove temp now
                            if no_run {
                                if !keep_temp {
                                    let _ = std::fs::remove_file(&out_js);
                                }
                            } else {
                                let status = std::process::Command::new("node")
                                    .arg(&out_js)
                                    .args(passthrough)
                                    .status();
                                match status {
                                    Ok(s) if s.success() => {}
                                    Ok(s) => anyhow::bail!("node exited with status {}", s),
                                    Err(e) => anyhow::bail!("failed to execute node: {e}"),
                                }
                                if !keep_temp {
                                    let _ = std::fs::remove_file(&out_js);
                                }
                            }
                            Ok(())
                        }
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
                                        Ok(s) => {
                                            anyhow::bail!("rust exec exited with status {}", s)
                                        }
                                        Err(e) => anyhow::bail!("failed to run rust exe: {e}"),
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
                    let _ = run_once(
                        &file,
                        &passthrough_filtered,
                        args.pretty_errors,
                        args.no_sema,
                        args.debug_titan,
                        keep_temp_flag,
                        no_run_flag,
                    );
                    if std::env::var("AEONMI_WATCH_ONCE").ok().as_deref() == Some("1") {
                        break;
                    }
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
                run_once(
                    &file,
                    &passthrough_filtered,
                    args.pretty_errors,
                    args.no_sema,
                    args.debug_titan,
                    keep_temp_flag,
                    no_run_flag,
                )
            }
        }

        Some(Command::Native {
            input,
            emit_ai,
            watch,
        }) => {
            use std::thread::sleep;
            use std::time::{Duration, SystemTime};
            fn run_native_file(
                p: &PathBuf,
                emit_ai: &Option<PathBuf>,
                pretty: bool,
                skip_sema: bool,
            ) -> anyhow::Result<()> {
                if let Some(ai_out) = emit_ai {
                    let _ = commands::compile::compile_pipeline(
                        Some(p.clone()),
                        EmitKind::Ai,
                        ai_out.clone(),
                        false,
                        false,
                        pretty,
                        skip_sema,
                        false,
                    );
                }
                std::env::set_var("AEONMI_NATIVE", "1");
                commands::run::main_with_opts(p.clone(), None, pretty, skip_sema)
            }
            if watch {
                let mut last_mtime = std::fs::metadata(&input)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                loop {
                    let _ = run_native_file(&input, &emit_ai, args.pretty_errors, args.no_sema);
                    sleep(Duration::from_millis(500));
                    if let Ok(meta) = std::fs::metadata(&input) {
                        if let Ok(m) = meta.modified() {
                            if m > last_mtime {
                                last_mtime = m;
                                println!("[watch] detected change, re-running (native)...");
                                continue;
                            }
                        }
                    }
                }
            } else {
                run_native_file(&input, &emit_ai, args.pretty_errors, args.no_sema)
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
