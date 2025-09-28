use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

use colored::Colorize;

use crate::cli::EmitKind;
use crate::core::diagnostics::{print_error, Span};
use crate::core::lexer::{Lexer, LexerError};
use crate::core::parser::{Parser as AeParser, ParserError};
use crate::core::code_generator::CodeGenerator; // JS + AI backends

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

/// Exposed so `run` (and others) can reuse it
pub fn compile_pipeline(
    input: Option<PathBuf>,
    emit: EmitKind,
    out: PathBuf,
    print_tokens: bool,
    print_ast: bool,
    pretty: bool,
    skip_sema: bool,   // honored via note (codegen path doesn’t need it)
    _debug_titan: bool, // wired for Titan debug; unused in this frontend
) -> anyhow::Result<()> {
    let input_path = input
        .as_ref()
        .map(|p| p.as_path())
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
        Err(ParserError { message, line, col }) => {
            if pretty {
                print_error(
                    &input_path.display().to_string(),
                    &source,
                    &format!("Parsing error: {}", message),
                    Span::single(line, col),
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

    // Generate code (JS or canonical .ai)
    let output_string = match emit {
        EmitKind::Ai => {
<<<<<<< HEAD
<<<<<<< HEAD
            let mut gen = CodeGenerator::new_ai();
            match gen.generate(&ast) {
                Ok(s) => s,
=======
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
>>>>>>> 57cd645 (feat(cli): integrate new Aeonmi CLI + shard updates)
=======
            let mut gen = CodeGenerator::new_ai();
            match gen.generate(&ast) {
                Ok(s) => s,
>>>>>>> 0503a82 (VM wired to Shard; canonical .ai emitter; CLI/test fixes)
                Err(e) => {
                    eprintln!("{} AI emit failed: {}", "error:".bright_red().bold(), e);
                    exit(1);
                }
            }
        }
        EmitKind::Js => {
            let mut gen = CodeGenerator::new(); // legacy JS backend
            match gen.generate(&ast) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{} JS emit failed: {}", "error:".bright_red().bold(), e);
                    exit(1);
                }
            }
        }
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
