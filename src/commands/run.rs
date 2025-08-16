use std::path::PathBuf;
use colored::Colorize;

use super::compile::compile_pipeline;
use crate::cli::EmitKind;

pub fn main_with_opts(
    input: PathBuf,
    out: Option<PathBuf>,
    pretty: bool,
    no_sema: bool,
) -> anyhow::Result<()> {
    let out_path = out.unwrap_or_else(|| PathBuf::from("aeonmi.run.js"));

    // Compile to JS, then try to run with Node
    compile_pipeline(
        Some(input.clone()),
        EmitKind::Js,
        out_path.clone(),
<<<<<<< HEAD
        /*print_tokens*/ false,
        /*print_ast*/ false,
        pretty,
        no_sema,
        /*debug_titan*/ false,
=======
        false, // print_tokens
        false, // print_ast
        pretty,
        no_sema,
        false, // debug_titan (default off here)
>>>>>>> 57cd645 (feat(cli): integrate new Aeonmi CLI + shard updates)
    )?;

    match std::process::Command::new("node").arg(&out_path).status() {
        Ok(status) if !status.success() => {
            eprintln!(
                "{} JS runtime exited with status: {}",
                "warn:".yellow().bold(),
                status
            )
        }
        Err(err) => {
            eprintln!(
                "{} Could not launch Node.js: {} (compiled output is at '{}')",
                "warn:".yellow().bold(),
                err,
                out_path.display()
            )
        }
        _ => {}
    }

    Ok(())
}
