//! Minimal quantum IR stub used by the CLI and future compiler passes.
//! We keep this file warning-free until the .ai → IR pipeline starts emitting ops.
#![allow(dead_code)]

use anyhow::{Result, anyhow};
use std::path::Path;

/// Gate kinds we expect to support first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpKind {
    H,
    X,
    CNOT { control: usize, target: usize },
    RX,
    RY,
    RZ,
    Measure { qubit: usize, cbit: usize },
}

/// A single operation in the circuit.
#[derive(Debug, Clone)]
pub struct Op {
    pub kind: OpKind,
    pub targets: Vec<usize>,
    pub params: Vec<f64>,
}

/// A very small circuit container.
#[derive(Debug, Clone)]
pub struct Circuit {
    pub n_qubits: usize,
    pub ops: Vec<Op>,
}

impl Circuit {
    pub fn new(n_qubits: usize) -> Self {
        Self { n_qubits, ops: Vec::new() }
    }
    pub fn push(&mut self, op: Op) { self.ops.push(op); }
}

/// Temporary parser stub: accept a path, return a fixed H on |0⟩ circuit.
/// This will be replaced by the real .ai → IR pass.
pub fn parse_ai_to_ir(_path: &Path) -> Result<Circuit> {
    let mut c = Circuit::new(1);
    c.push(Op {
        kind: OpKind::H,
        targets: vec![0],
        params: vec![],
    });
    Ok(c)
}

/// Tiny validator for future use.
pub fn validate(circ: &Circuit) -> Result<()> {
    if circ.n_qubits == 0 {
        return Err(anyhow!("circuit has zero qubits"));
    }
    Ok(())
}
