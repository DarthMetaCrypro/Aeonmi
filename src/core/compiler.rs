//! Aeonmi/QUBE/Titan Compiler Pipeline
//! Runs validation, lexing, parsing, semantic analysis, and code generation.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::{
    code_generator::CodeGenerator,
    error::CoreError,
    lexer::Lexer,
    parser::{Parser, ParserError},
    semantic_analyzer::SemanticAnalyzer,
    token::TokenKind,
};

// Expose IR types so the CLI can request an IR build (stubbed for now).
// NOTE: Avoid importing `Import` to keep warnings clean for now.
use crate::core::ir::{Block, Decl, Expr, FnDecl, Lit, Module, Stmt};

use crate::core::titan; // Titan math/quantum library entry point

/// Represents the Aeonmi/QUBE/Titan compiler
pub struct Compiler;

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Compiler
    }

    /// Back-compat: run semantic analysis; no Titan debug.
    #[allow(dead_code)]
    pub fn compile(&self, code: &str, output_file: &str) -> Result<(), CoreError> {
        self.compile_with(code, output_file, true, false)
    }

    /// Main compilation pipeline with option to skip semantic analysis
    /// and optionally run Titan debug integration.
    pub fn compile_with(
        &self,
        code: &str,
        output_file: &str,
        run_semantic: bool,
        debug_titan: bool,
    ) -> Result<(), CoreError> {
        // 1) Validate input
        let summary = self.validate_and_summarize(code)?;
        println!("{summary}");

        // 2) Lexing
        let mut lexer = Lexer::from_str(code);
        let tokens = lexer
            .tokenize()
            .map_err(|e| CoreError::general_error(&format!("Lexing error: {}", e)))?;

        if tokens.iter().any(|t| matches!(t.kind, TokenKind::EOF)) {
            println!("Lexer: Found EOF token.");
        }
        println!("Lexer: {} tokens generated.", tokens.len());

        // 3) Parsing
        let mut parser = Parser::new(tokens.clone());
        let ast = parser
            .parse()
            .map_err(|e: ParserError| CoreError::general_error(&format!("Parsing error: {}", e)))?;
        println!("Parser: AST generated successfully.");

        // 4) Semantic analysis (optional)
        if run_semantic {
            let mut analyzer = SemanticAnalyzer::new();
            match analyzer.analyze(&ast) {
                Ok(()) => println!("Semantic Analyzer: No semantic errors found."),
                Err(e) => {
                    println!("Semantic Analyzer errors: {}", e);
                    return Err(CoreError::general_error(&format!(
                        "Semantic analysis error: {}", e
                    )));
                }
            }
        } else {
            println!("Semantic Analyzer: skipped by flag.");
        }

        // 4.5) Titan library debug (optional)
        if debug_titan {
            println!("--- Titan Debug Mode Active ---");
            let v1 = vec![1.0, 2.0, 3.0];
            let v2 = vec![4.0, 5.0, 6.0];

            match titan::linear_algebra::dot_product(&v1, &v2) {
                Ok(result) => println!(
                    "Titan Test - Dot Product of {:?} and {:?} = {}",
                    v1, v2, result
                ),
                Err(e) => println!("Titan Test - Dot Product Error: {}", e),
            }

            let identity = titan::linear_algebra::identity_matrix(3);
            println!("Titan Test - 3x3 Identity Matrix:");
            for row in &identity {
                println!("{:?}", row);
            }
            println!("--- End Titan Debug ---");
        }

        // 5) Code generation (to JS)
        let mut generator = CodeGenerator::new();
        let output_code = generator
            .generate(&ast)
            .map_err(|e| CoreError::general_error(&format!("Code generation error: {}", e)))?;
        println!("Code Generator: Output code generated successfully.");

        // 6) Write file
        let mut file = File::create(output_file)
            .map_err(|e| CoreError::io_error(&format!("Failed to create output file: {}", e)))?;
        file.write_all(output_code.as_bytes())
            .map_err(|e| CoreError::io_error(&format!("Failed to write to output file: {}", e)))?;

        println!(
            "Compilation successful. Output written to '{}'.",
            output_file
        );
        Ok(())
    }

    /// Validates code before compilation
    pub fn validate_and_summarize(&self, code: &str) -> Result<String, CoreError> {
        if code.trim().is_empty() {
            return Err(CoreError::invalid_operation("No source code provided"));
        }
        let lines = code.lines().count();
        let chars = code.chars().count();
        Ok(format!(
            "Validation complete: {} lines, {} characters.",
            lines, chars
        ))
    }
}

/* ---------------- IR build (stub) for `emit --format ai` ---------------- */

/// Build a minimal IR `Module` from a `.qube` file so `emit --format ai` works.
/// This does NOT parse yet; it just creates an IR that prints a stub line.
/// Replace with real AST→IR lowering when ready.
#[allow(dead_code)]
pub fn compile_file_to_ir(path: &Path) -> Result<Module, String> {
    // Ensure the file exists/readable (so CLI errors are still real)
    let _src =
        fs::read_to_string(path).map_err(|e| format!("read {} failed: {e}", path.display()))?;

    let name = module_name_from_path(path);

    // fn main() { print("⧉ aeonmi: IR lowering stub — replace me"); }
    let main_fn = Decl::Fn(FnDecl {
        name: "main".to_string(),
        params: vec![],
        body: Block {
            stmts: vec![Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Ident("print".into())),
                args: vec![Expr::Lit(Lit::String(
                    "⧉ aeonmi: IR lowering stub — replace me".into(),
                ))],
            })],
        },
    });

    Ok(Module {
        name,
        imports: vec![], // add imports when needed
        decls: vec![main_fn],
    })
}

#[allow(dead_code)]
fn module_name_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("aeonmi")
        .to_string()
}

/* ---------------- Helpers used by CLI `main.rs` ---------------- */

/// Compile the given `.qube` to JS and run it with Node.
/// Matches existing console messages used by tests. Node errors are non-fatal.
#[allow(dead_code)]
pub fn compile_and_run_js(input: &PathBuf) -> Result<(), String> {
    let src = fs::read_to_string(input)
        .map_err(|e| format!("failed to read {}: {e}", input.display()))?;

    // Keep the legacy output name (tests may expect this exact filename)
    let out = PathBuf::from("aeonmi.run.js");

    let compiler = Compiler::new();
    compiler
        .compile_with(&src, out.to_str().unwrap(), true, false)
        .map_err(|e| format!("{e}"))?;

    // Try to run with node; warn but don't fail if it's missing or exits non-zero.
    match Command::new("node").arg(&out).status() {
        Ok(status) => {
            if !status.success() {
                eprintln!("warn: JS runtime exited with status: {status}");
            }
        }
        Err(e) => {
            eprintln!("warn: unable to launch Node.js: {e}");
        }
    }

    Ok(())
}

/// Compile the given `.qube` to JS and write it to `out` (or default name).
#[allow(dead_code)]
pub fn compile_and_write_js(input: &PathBuf, out: Option<&PathBuf>) -> Result<(), String> {
    let src = fs::read_to_string(input)
        .map_err(|e| format!("failed to read {}: {e}", input.display()))?;

    let out_path = out
        .cloned()
        .unwrap_or_else(|| PathBuf::from("aeonmi.run.js"));

    let compiler = Compiler::new();
    compiler
        .compile_with(&src, out_path.to_str().unwrap(), true, false)
        .map_err(|e| format!("{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_code_fails() {
        let compiler = Compiler::new();
        let result = compiler.validate_and_summarize("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_passes() {
        let compiler = Compiler::new();
        let result = compiler.validate_and_summarize("let x = 42;");
        assert_eq!(
            result.unwrap(),
            "Validation complete: 1 lines, 11 characters."
        );
    }

    #[test]
    fn test_full_pipeline_runs() {
        let compiler = Compiler::new();
        let result = compiler.compile("let x = 42;", "test_output.js");
        assert!(result.is_ok());
    }
}
