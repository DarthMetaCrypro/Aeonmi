// src/core/lib.rs
//! Aeonmi/QUBE/Titan Core Library
//! Links all core compiler/interpreter modules together.
//! Updated: 2025-08-10

pub mod token;              // Token definitions and helpers
pub mod lexer;              // Lexical analysis
pub mod ast;                // Abstract Syntax Tree
pub mod parser;             // Parsing into AST

// Compilation chain
pub mod semantic_analyzer;  // Type & semantic checks
pub mod ir_generator;       // Intermediate representation
pub mod code_generator;     // Code generation / backend
pub mod runtime;            // Execution runtime

// Tools & utilities
pub mod symbol_table;       // Scope and symbol management
pub mod syntax_validator;   // Syntax validation helpers
pub mod error;              // Error types and handling
pub mod utils;              // General utilities

// Optional / extended modules (Aeonmi AI, QUBE quantum math, etc.)
pub mod quantum_math;       // Quantum-specific math ops
pub mod neural;             // Neural computation utilities
pub mod neural_network;     // Neural network layer
pub mod probability_statistics;
pub mod stochastic_processes;
pub mod debugger;           // Debugger tooling
pub mod compiler;           // Full compile pipeline

// ...existing mod declarations...
pub mod formatter; // canonical .ai pretty-printer
