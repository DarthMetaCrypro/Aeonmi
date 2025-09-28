<<<<<<< HEAD
//! Core module tree for Aeonmi compiler/runtime.
//! Only declare modules that exist in the src/core/ directory.

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
=======
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
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))

#[cfg(feature = "quantum")]
pub mod quantum_ir;

pub mod titan;
pub mod vm;