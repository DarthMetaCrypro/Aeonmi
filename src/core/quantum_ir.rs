#![cfg(feature = "quantum")]
#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use std::{fs, path::Path};

#[derive(Clone, Debug)]
pub enum OpKind {
    H,
    X,
    CNOT,
}

#[derive(Clone, Debug)]
pub struct Op {
    pub kind: OpKind,
    pub targets: Vec<usize>,
    pub params: Vec<f64>,
}

#[derive(Clone, Debug)]
pub struct Circuit {
    pub n_qubits: usize,
    pub ops: Vec<Op>,
}

impl Circuit {
    pub fn new(n_qubits: usize) -> Self {
        Self {
            n_qubits,
            ops: Vec::new(),
        }
    }
    pub fn push(&mut self, op: Op) {
        self.ops.push(op);
    }
}

/// Minimal .ai grammar (one instruction per line):
///   qubits N
///   h i
///   x i
///   cnot c t
/// Lines may contain comments starting with `//`.
///
/// Examples:
///   qubits 1
///   h 0
///
///   qubits 2
///   h 0
///   cnot 0 1
pub fn parse_ai_to_ir(path: &Path) -> Result<Circuit> {
    let src =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;

    let mut n_qubits: Option<usize> = None;
    let mut ops: Vec<Op> = Vec::new();
    let mut max_target_seen: isize = -1;

    for (lineno, raw) in src.lines().enumerate() {
        let line = raw.split("//").next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        let toks: Vec<&str> = line.split_whitespace().collect();
        let kw = toks[0].to_ascii_lowercase();

        let bad = |msg: &str| -> anyhow::Error {
            anyhow::anyhow!("{}:{}: {}", path.display(), lineno + 1, msg)
        };

        match kw.as_str() {
            "qubits" => {
                if toks.len() != 2 {
                    bail!(bad("usage: qubits <N>"));
                }
                let n: usize = toks[1].parse().map_err(|_| bad("invalid qubit count"))?;
                if n == 0 {
                    bail!(bad("qubits must be ≥ 1"));
                }
                n_qubits = Some(n);
            }

            "h" | "x" => {
                if toks.len() != 2 {
                    bail!(bad("usage: h <i>   or   x <i>"));
                }
                let i: usize = toks[1].parse().map_err(|_| bad("invalid target index"))?;
                max_target_seen = max_target_seen.max(i as isize);
                let kind = if kw == "h" { OpKind::H } else { OpKind::X };
                ops.push(Op {
                    kind,
                    targets: vec![i],
                    params: vec![],
                });
            }

            "cnot" => {
                if toks.len() != 3 {
                    bail!(bad("usage: cnot <control> <target>"));
                }
                let c: usize = toks[1].parse().map_err(|_| bad("invalid control index"))?;
                let t: usize = toks[2].parse().map_err(|_| bad("invalid target index"))?;
                if c == t {
                    bail!(bad("cnot control and target must differ"));
                }
                max_target_seen = max_target_seen.max(c as isize).max(t as isize);
                ops.push(Op {
                    kind: OpKind::CNOT,
                    targets: vec![c, t],
                    params: vec![],
                });
            }

            other => bail!(bad(&format!("unknown instruction '{other}'"))),
        }
    }

    let inferred = if max_target_seen >= 0 {
        (max_target_seen as usize) + 1
    } else {
        0
    };
    let n_qubits = n_qubits.unwrap_or_else(|| inferred.max(1));

    // Basic bounds check (guards silly files like "qubits 1; cnot 0 1")
    for (idx, op) in ops.iter().enumerate() {
        for &t in &op.targets {
            if t >= n_qubits {
                bail!(
                    "{}: op #{idx} references qubit {} out of range 0..{}",
                    path.display(),
                    t,
                    n_qubits - 1
                );
            }
        }
    }

    Ok(Circuit { n_qubits, ops })
}
