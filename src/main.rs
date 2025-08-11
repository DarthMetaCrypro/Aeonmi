//! Aeonmi/QUBE Main Entry Point
//! CLI: Lexer → Parser → Compiler, with debug flags.

mod core;

use clap::{Arg, Command};
use colored::Colorize;

use crate::core::diagnostics::{print_error, Span};
use crate::core::lexer::{Lexer, LexerError};
use crate::core::parser::{Parser, ParserError};
use crate::core::compiler::Compiler;

use std::fs;
use std::process::exit;

fn main() {
    let matches = Command::new("aeonmi")
        .about("Aeonmi/QUBE/Titan unified compiler")
        .arg(Arg::new("input")
             .help("Input source file (.ai)")
             .required(false))
        .arg(Arg::new("emit")
             .long("emit").value_name("KIND")
             .default_value("js")
             .help("Emit kind (currently: js)"))
        .arg(Arg::new("out")
             .long("out").value_name("FILE")
             .help("Output file path (e.g., output.js)")
             .default_value("output.js"))
        .arg(Arg::new("tokens")
             .long("tokens")
             .help("Print tokens").num_args(0))
        .arg(Arg::new("ast")
             .long("ast")
             .help("Print AST").num_args(0))
        .arg(Arg::new("no-sema")
             .long("no-sema")
             .help("Skip semantic analysis").num_args(0))
        .arg(Arg::new("pretty-errors")
             .long("pretty-errors")
             .help("Use pretty, colored diagnostics").num_args(0))
        .get_matches();

    let input_path = matches.get_one::<String>("input")
        .map(|s| s.as_str())
        .unwrap_or("examples/hello.ai");
    let emit_kind = matches.get_one::<String>("emit").map(String::as_str).unwrap_or("js");
    let out_path  = matches.get_one::<String>("out").map(String::as_str).unwrap_or("output.js");
    let print_tokens = matches.contains_id("tokens");
    let print_ast    = matches.contains_id("ast");
    let skip_sema    = matches.contains_id("no-sema");
    let pretty       = matches.contains_id("pretty-errors");

    if emit_kind != "js" {
        eprintln!("{}", format!("Unsupported --emit kind: {}", emit_kind).bright_red());
        exit(2);
    }

    let source = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("{} Could not read '{}', using default inline code.",
                      "warn:".yellow().bold(), input_path);
            "let x = 42; log(x);".to_string()
        }
    };

    if print_tokens || print_ast {
        println!("=== Source Code ===\n{}\n", source);
    }

    // Stage 1: Lex
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
                        print_error(input_path, &source, &format!("{}", e), Span::single(line, col));
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
        for token in &tokens { println!("{}", token); }
        println!();
    }

    // Stage 2: Parse
    let mut parser = Parser::new(tokens.clone());
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e @ ParserError { line, col, .. }) => {
            if pretty {
                print_error(input_path, &source, &format!("Parsing error: {}", e.message), Span::single(line, col));
            } else {
                eprintln!("{} Parsing error: {}", "error:".bright_red(), e);
            }
            exit(1);
        }
    };

    if print_ast {
        println!("=== AST ===\n{:#?}\n", ast);
    }

    // Stage 3: Compile
    let compiler = Compiler::new();
    let run_semantic = !skip_sema;
    let res = compiler.compile_with(&source, out_path, run_semantic);
    match res {
        Ok(_) => {
            println!("{} {}", "ok:".green().bold(), format!("Compilation successful. Output written to '{}'.", out_path));
        }
        Err(e) => {
            // CoreError (general string) — no span; show plainly
            if pretty {
                eprintln!("{} {}", "error:".bright_red().bold(), e);
            } else {
                eprintln!("Compilation failed: {}", e);
            }
            exit(1);
        }
    }
}
