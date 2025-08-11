// src/core/compiler.rs
//! Aeonmi/QUBE/Titan Compiler Pipeline
//! Runs validation, lexing, parsing, semantic analysis, and code generation.

use std::fs::File;
use std::io::Write;

use crate::core::{
    error::CoreError,
    lexer::Lexer,
    parser::{Parser, ParserError},
    semantic_analyzer::SemanticAnalyzer,
    code_generator::CodeGenerator,
    token::TokenKind,
};

/// Represents the Aeonmi/QUBE/Titan compiler
pub struct Compiler;

impl Compiler {
    pub fn new() -> Self { Compiler }

    /// Back-compat: default to running semantic analysis.
    pub fn compile(&self, code: &str, output_file: &str) -> Result<(), CoreError> {
        self.compile_with(code, output_file, true)
    }

    /// Main compilation pipeline with option to skip semantic analysis
    pub fn compile_with(&self, code: &str, output_file: &str, run_semantic: bool) -> Result<(), CoreError> {
        // 1) Validate input
        let summary = self.validate_and_summarize(code)?;
        println!("{summary}");

        // 2) Lexing
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize()
            .map_err(|e| CoreError::general_error(&format!("Lexing error: {}", e)))?;

        if tokens.iter().any(|t| matches!(t.kind, TokenKind::EOF)) {
            println!("Lexer: Found EOF token.");
        }
        println!("Lexer: {} tokens generated.", tokens.len());

        // 3) Parsing
        let mut parser = Parser::new(tokens.clone());
        let ast = parser.parse()
            .map_err(|e: ParserError| CoreError::general_error(&format!("Parsing error: {}", e)))?;

        println!("Parser: AST generated successfully.");

        // 4) Semantic analysis (optional)
        if run_semantic {
            let mut analyzer = SemanticAnalyzer::new();
            analyzer.analyze(&ast)
                .map_err(|e| CoreError::general_error(&format!("Semantic analysis error: {}", e)))?;
            println!("Semantic Analyzer: No semantic errors found.");
        } else {
            println!("Semantic Analyzer: skipped by flag.");
        }

        // 5) Code generation
        let mut generator = CodeGenerator::new();
        let output_code = generator.generate(&ast)
            .map_err(|e| CoreError::general_error(&format!("Code generation error: {}", e)))?;
        println!("Code Generator: Output code generated successfully.");

        // 6) Write file
        let mut file = File::create(output_file)
            .map_err(|e| CoreError::general_error(&format!("Failed to create output file: {}", e)))?;
        file.write_all(output_code.as_bytes())
            .map_err(|e| CoreError::general_error(&format!("Failed to write to output file: {}", e)))?;

        println!("Compilation successful. Output written to '{}'.", output_file);
        Ok(())
    }

    /// Validates code before compilation
    pub fn validate_and_summarize(&self, code: &str) -> Result<String, CoreError> {
        if code.trim().is_empty() {
            return Err(CoreError::general_error("Code is empty. Nothing to compile."));
        }
        let lines = code.lines().count();
        let chars = code.chars().count();
        Ok(format!("Validation complete: {} lines, {} characters.", lines, chars))
    }
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
        assert_eq!(result.unwrap(), "Validation complete: 1 lines, 11 characters.");
    }
    #[test]
    fn test_full_pipeline_runs() {
        let compiler = Compiler::new();
        let result = compiler.compile("let x = 42;", "test_output.js");
        assert!(result.is_ok());
    }
}
