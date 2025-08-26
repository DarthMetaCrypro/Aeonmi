use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

use colored::Colorize;

use crate::cli::EmitKind;
use crate::core::code_generator::CodeGenerator;
use crate::core::diagnostics::{print_error, emit_json_error, Span};
use crate::core::lexer::{Lexer, LexerError};
use crate::core::parser::{Parser as AeParser, ParserError}; // JS + AI backends
use crate::core::artifact_cache::{get_artifact, put_artifact};
use sha1::{Sha1, Digest};

#[allow(dead_code, clippy::too_many_arguments)]
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

/// Exposed so `run` (and others) can reuse it
#[allow(clippy::too_many_arguments)]
pub fn compile_pipeline(
    input: Option<PathBuf>,
    emit: EmitKind,
    out: PathBuf,
    print_tokens: bool,
    print_ast: bool,
    pretty: bool,
    skip_sema: bool,    // honored via note (codegen path doesn’t need it)
    _debug_titan: bool, // wired for Titan debug; unused in this frontend
) -> anyhow::Result<()> {
    let input_path = input.as_deref()
        .unwrap_or_else(|| Path::new("examples/hello.ai"));

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
                            &input_path.display().to_string(),
                            &format!("{}", e),
                            &Span::single(line, col),
                        );
                        print_error(
                            &input_path.display().to_string(),
                            &source,
                            &format!("{}", e),
                            Span::single(line, col),
                        );
                    }
                    _ => {
                        // Fallback: show full diagnostic for other lexer errors
                        eprintln!("{} Lexing error: {}", "error:".bright_red(), e);
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
        Err(ParserError {
            message,
            line,
            column,
        }) => {
            if pretty {
                emit_json_error(
                    &input_path.display().to_string(),
                    &format!("Parsing error: {}", message),
                    &Span::single(line, column),
                );
                print_error(
                    &input_path.display().to_string(),
                    &source,
                    &format!("Parsing error: {}", message),
                    Span::single(line, column),
                );
            } else {
                eprintln!("{} Parsing error: {}", "error:".bright_red(), message);
            }
            exit(1);
        }
    };

    if print_ast {
        println!("=== AST ===\n{:#?}\n", ast);
    }

    // Honor --no-sema with a clear note (expected by tests)
    if skip_sema {
        println!("note: semantic analysis skipped");
    }

    // Artifact cache key: hash(source)+emit kind
    let mut hasher = Sha1::new(); hasher.update(source.as_bytes()); hasher.update(match emit { EmitKind::Ai=>b"AI", EmitKind::Js=>b"JS" });
    let key = format!("{:x}", hasher.finalize());
    let output_string = if let Some(entry) = get_artifact(&key) { String::from_utf8(entry.data).unwrap_or_default() } else {
        let generated = match emit {
            EmitKind::Ai => {
                let mut gen = CodeGenerator::new_ai();
                match gen.generate(&ast) { Ok(s)=>s, Err(e)=>{ eprintln!("{} AI emit failed: {}", "error:".bright_red().bold(), e); exit(1);} }
            }
            EmitKind::Js => {
                let mut gen = CodeGenerator::new();
                match gen.generate(&ast) { Ok(s)=>s, Err(e)=>{ eprintln!("{} JS emit failed: {}", "error:".bright_red().bold(), e); exit(1);} }
            }
        };
        put_artifact(key.clone(), generated.as_bytes().to_vec());
        generated
    };

    // Ensure output directory exists
    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!(
                    "{} could not create output dir '{}': {}",
                    "error:".bright_red().bold(),
                    parent.display(),
                    e
                );
                exit(1);
            }
        }
    }

    // Write file
    if let Err(e) = fs::write(&out, output_string) {
        if pretty {
            eprintln!("{} {}", "error:".bright_red().bold(), e);
        } else {
            eprintln!("Failed to write output: {}", e);
        }
        exit(1);
    }

    // Match legacy success phrasing exactly (tests depend on it)
    match emit {
        EmitKind::Js => println!("ok: wrote js to '{}'.", out.display()),
        EmitKind::Ai => println!("ok: wrote ai to '{}'.", out.display()),
    }

    Ok(())
}
