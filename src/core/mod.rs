pub mod ast;
pub mod code_generator;
pub mod compiler;
pub mod diagnostics;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod qpoly;
pub mod semantic_analyzer;
pub mod titan;
pub mod token;
pub mod ir;
pub mod ai_emitter;
pub mod lowering;
pub mod vm;

#[cfg(feature = "quantum")]
pub mod quantum_ir;
