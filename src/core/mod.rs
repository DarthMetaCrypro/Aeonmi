//! Core module tree for Aeonmi compiler/runtime.
//! Only declare modules that exist in the src/core/ directory.

pub mod ai_emitter;
pub mod ast;
pub mod code_generator;
pub mod compiler;
pub mod diagnostics;
pub mod error;
pub mod formatter;
pub mod ir;
pub mod lexer;
pub mod lowering;
pub mod parser;
pub mod qpoly;
pub mod semantic_analyzer;
pub mod titan;
pub mod token;
pub use token::TokenKind; // Re-export only TokenKind; Token not needed externally currently
pub mod vm;

#[cfg(feature = "quantum")]
pub mod quantum_ir;
