//! Core module tree for Aeonmi compiler/runtime.
//! Only declare modules that exist in the src/core/ directory.

pub mod ai_emitter;
pub mod ai_provider;
pub mod ast;
pub mod code_generator;
pub mod code_actions;
pub mod compiler;
pub mod diagnostics;
pub mod error;
pub mod formatter;
pub mod ir;
pub mod lexer;
pub mod lowering;
pub mod incremental;
pub mod parser;
pub mod qpoly;
pub mod quantum_extract;
pub mod artifact_cache;
pub mod api_keys;
pub mod semantic_analyzer;
pub mod symbols;
pub mod scope_map;
pub mod types;
pub mod titan;
pub mod token;
pub use token::TokenKind; // Re-export only TokenKind; Token not needed externally currently
#[macro_use]
pub mod debug; // gated debug logging (AEONMI_DEBUG=1) provides debug_log! macro
pub mod vm;

#[cfg(feature = "bytecode")]
pub mod bytecode;
#[cfg(feature = "bytecode")]
pub mod vm_bytecode;

#[cfg(feature = "quantum")]
pub mod quantum_ir;
