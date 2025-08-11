// src/commands/run.rs
use std::path::PathBuf;
use colored::Colorize;
use crate::cli::EmitKind;
use super::compile::compile_pipeline;

pub fn main_with_opts(
    input: PathBuf,
    out: Option<PathBuf>,
    pretty: bool,
    no_sema: bool,
) -> anyhow::Result<()> {
    let out_path = out.unwrap_or_else(|| PathBuf::from("aeonmi.run.js"));
    compile_pipeline(Some(input.clone()), EmitKind::Js, out_path.clone(), false, false, pretty, no_sema)?;
    match std::process::Command::new("node").arg(&out_path).status() {
        Ok(status) if !status.success() => eprintln!("{} JS runtime exited with status: {}", "warn:".yellow().bold(), status),
        Err(err) => eprintln!("{} Could not launch Node.js: {} (compiled output is at '{}')", "warn:".yellow().bold(), err, out_path.display()),
        _ => {}
    }
    Ok(())
}
