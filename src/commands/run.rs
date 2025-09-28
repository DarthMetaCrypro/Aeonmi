use colored::Colorize;
use std::path::PathBuf;

use super::compile::compile_pipeline;
use crate::cli::EmitKind;

// Native interpreter pieces
use crate::core::lexer::Lexer;
use crate::core::parser::{Parser as AeParser, ParserError};
use crate::core::lowering::lower_ast_to_ir;
use crate::core::vm::Interpreter;
use crate::core::diagnostics::{print_error, emit_json_error, Span};
use crate::core::lexer::LexerError;

/// Public native interpreter entry (no JS emission)
pub fn run_native(
    input: &PathBuf,
    pretty: bool,
    no_sema: bool,
) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(input)?;
    // Lex
    let mut lexer = Lexer::from_str(&source);
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
                            &input.display().to_string(),
                            &format!("{}", e),
                            &Span::single(line, col),
                        );
                        print_error(
                            &input.display().to_string(),
                            &source,
                            &format!("{}", e),
                            Span::single(line, col),
                        );
                    }
                    _ => eprintln!("{} Lexing error: {}", "error:".bright_red(), e),
                }
            } else {
                eprintln!("{} Lexing error: {}", "error:".bright_red(), e);
            }
            return Ok(()); // mimic compile path exit(1) without aborting process
        }
    };
    // Parse
    let mut parser = AeParser::new(tokens.clone());
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(ParserError { message, line, column }) => {
            if pretty {
                emit_json_error(
                    &input.display().to_string(),
                    &format!("Parsing error: {}", message),
                    &Span::single(line, column),
                );
                print_error(
                    &input.display().to_string(),
                    &source,
                    &format!("Parsing error: {}", message),
                    Span::single(line, column),
                );
            } else {
                eprintln!("{} Parsing error: {}", "error:".bright_red(), message);
            }
            return Ok(());
        }
    };
    if no_sema {
        println!("note: semantic analysis skipped (native)");
    }
    // Lower & interpret
    println!("DEBUG: RUN PATH - native: executing '{}' via Aeonmi VM", input.display());
    match lower_ast_to_ir(&ast, "main") {
        Ok(module) => {
            let mut interp = Interpreter::new();
            if let Err(e) = interp.run_module(&module) {
                eprintln!("{} runtime error: {}", "error:".bright_red(), e.message);
            }
        }
        Err(e) => eprintln!("{} lowering error: {}", "error:".bright_red(), e),
    }
    Ok(())
}

pub fn main_with_opts(
    input: PathBuf,
    out: Option<PathBuf>,
    pretty: bool,
    no_sema: bool,
) -> anyhow::Result<()> {
    // Force native interpreter path if env requests or if node missing
    let force_native = std::env::var("AEONMI_NATIVE").ok().as_deref() == Some("1");
    let node_available = std::process::Command::new("node")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if force_native || !node_available {
        if !node_available && !force_native {
            println!("(node not found — falling back to native interpreter)");
        }
        return run_native(&input, pretty, no_sema);
    }

    let out_path = out.unwrap_or_else(|| PathBuf::from("aeonmi.run.js"));
    compile_pipeline(
        Some(input.clone()),
        EmitKind::Js,
        out_path.clone(),
        false,
        false,
        pretty,
        no_sema,
        false,
    )?;
    match std::process::Command::new("node").arg(&out_path).status() {
        Ok(status) if !status.success() => eprintln!(
            "{} JS runtime exited with status: {}",
            "warn:".yellow().bold(),
            status
        ),
        Err(err) => eprintln!(
            "{} Could not launch Node.js: {} (compiled output is at '{}')",
            "warn:".yellow().bold(),
            err,
            out_path.display()
        ),
        _ => {}
    }
    Ok(())
}
