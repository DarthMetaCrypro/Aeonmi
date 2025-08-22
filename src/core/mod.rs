// core module tree - only declare modules that actually exist in the directory

pub mod token;
pub mod lexer;
pub mod ast;
pub mod parser;

pub mod semantic_analyzer;
pub mod ir;
pub mod lowering;
pub mod code_generator;
pub mod compiler;
pub mod diagnostics;
pub mod error;

pub mod ai_emitter;
pub mod formatter;
pub mod qpoly;

#[cfg(feature = "quantum")]
pub mod quantum_ir;

pub mod titan;
pub mod vm;
