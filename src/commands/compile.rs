use std::fs;
use std::path::PathBuf;
use std::process::exit;

use colored::Colorize;

use crate::cli::EmitKind;
use crate::core::compiler::Compiler;
use crate::core::diagnostics::{print_error, Span};
use crate::core::lexer::{Lexer, LexerError};
use crate::core::parser::{Parser as AeParser, ParserError};

#[allow(dead_code)]
pub fn main_with_opts(
    input: PathBuf,
    emit: EmitKind,
    out: PathBuf,
    print_tokens: bool,
    print_ast: bool,
    pretty: bool,
    skip_sema: bool,
    debug_titan: bool,
) -> anyhow::Result<()> {
    compile_pipeline(
        Some(input),
        emit,
        out,
        print_tokens,
        print_ast,
        pretty,
        skip_sema,
        debug_titan,
    )
}

/// exposed so `run` (and others) can reuse it
pub fn compile_pipeline(
    input: Option<PathBuf>,
    emit: EmitKind,
    out: PathBuf,
    print_tokens: bool,
    print_ast: bool,
    pretty: bool,
    skip_sema: bool,
    debug_titan: bool,
) -> anyhow::Result<()> {
    let input_path = input
        .as_ref()
        .map(|p| p.as_path())
        .unwrap_or_else(|| std::path::Path::new("examples/hello.ai"));

    // Load source (with fallback)
    let source = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!(
                "{} Could not read '{}', using default inline code.",
                "warn:".yellow().bold(),
                input_path.display()
            );
            "let x = 42; log(x);".to_string()
        }
    };

    if print_tokens || print_ast {
        println!("=== Source Code ===\n{}\n", source);
    }

    // Lex
    let mut lexer = Lexer::new(&source);
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
                        print_error(
                            &input_path.display().to_string(),
                            &source,
                            &format!("{}", e),
                            Span::single(line, col),
                        );
                    }
                }
            } else {
                eprintln!("{} Lexing error: {}", "error:".bright_red(), e);
            }
            exit(1);
        }
    };

    if print_tokens {
        println!("=== Tokens ===");
        for token in &tokens {
            println!("{}", token);
        }
        println!();
    }

    // Parse
    let mut parser = AeParser::new(tokens.clone());
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e @ ParserError { line, col, .. }) => {
            if pretty {
                print_error(
                    &input_path.display().to_string(),
                    &source,
                    &format!("Parsing error: {}", e.message),
                    Span::single(line, col),
                );
            } else {
                eprintln!("{} Parsing error: {}", "error:".bright_red(), e);
            }
            exit(1);
        }
    };

    if print_ast {
        println!("=== AST ===\n{:#?}\n", ast);
    }

    // Emit
    match emit {
        EmitKind::Ai => {
            if let Err(e) = fs::write(&out, &source) {
                if pretty {
                    eprintln!("{} {}", "error:".bright_red().bold(), e);
                } else {
                    eprintln!("Failed to write output: {}", e);
                }
                exit(1);
            }
            println!(
                "{} {}",
                "ok:".green().bold(),
                format!("Wrote Aeonmi source to '{}'.", out.display())
            );
            Ok(())
        }
        EmitKind::Js => {
            let compiler = Compiler::new();
            let run_semantic = !skip_sema;
            let res = compiler.compile_with(
                &source,
                &out.display().to_string(),
                run_semantic,
                debug_titan,
            );
            match res {
                Ok(_) => {
                    println!(
                        "{} {}",
                        "ok:".green().bold(),
                        format!(
                            "Compilation successful. Output written to '{}'.",
                            out.display()
                        )
                    );
                    Ok(())
                }
                Err(e) => {
                    if pretty {
                        eprintln!("{} {}", "error:".bright_red().bold(), e);
                    } else {
                        eprintln!("Compilation failed: {}", e);
                    }
                    exit(1);
                }
            }
        }
    }
}
