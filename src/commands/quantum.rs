#![cfg(feature = "quantum")]
//! CLI entry for quantum runs: Titan local + Aer via Qiskit.
//! Now reads a tiny `.ai` file and executes: `qubits`, `h`, `x`, `cnot`.

use anyhow::{anyhow, bail, Context, Result};
use std::path::PathBuf;

use crate::core::quantum_ir::{parse_ai_to_ir, OpKind};
use crate::core::titan::ops;
use crate::core::titan::{
    gates,
    types::{QOp, QState},
}; // for cnot_n

use nalgebra::DMatrix;
use num_complex::Complex64 as C64;
use rand::Rng;

#[cfg(feature = "qiskit")]
use crate::core::titan::qiskit_bridge;

pub fn main(file: PathBuf, shots: Option<usize>, backend: &str) -> Result<()> {
    quantum_run(file, backend, shots)
}

pub fn quantum_run(file: PathBuf, backend: &str, shots: Option<usize>) -> Result<()> {
    let be = backend.to_ascii_lowercase();

    match be.as_str() {
        "titan" => run_titan(file, shots),
        "aer" | "qiskit" => {
            #[cfg(feature = "qiskit")]
            {
                run_aer(file, shots)
            }
            #[cfg(not(feature = "qiskit"))]
            {
                bail!("Backend 'aer' requires `--features qiskit` and a working Python/Qiskit install.")
            }
        }
        "ibmq" => {
            #[cfg(feature = "qiskit")]
            {
                let ver = qiskit_bridge::qiskit_version().unwrap_or_default();
                bail!("IBMQ path not wired yet (Qiskit v{ver}). Use `aer` for now.");
            }
            #[cfg(not(feature = "qiskit"))]
            {
                bail!("Backend 'ibmq' requires `--features qiskit`.")
            }
        }
        other => bail!("unsupported backend: {other} (try: titan | aer | ibmq)"),
    }
}

fn run_titan(file: PathBuf, shots: Option<usize>) -> Result<()> {
    let circ = parse_ai_to_ir(&file).context("parse .ai failed")?;

    // Start in |0..0>
    let mut psi = QState::zeros(circ.n_qubits);

    // Apply ops sequentially
    for op in circ.ops {
        match op.kind {
            OpKind::H => {
                let u = gates::h();
                let full = expand_1q(&u, circ.n_qubits, op.targets[0]);
                let qop = QOp::try_new_unitary(full).map_err(|e: String| anyhow!(e))?;
                psi = qop.apply(&psi).map_err(|e: String| anyhow!(e))?;
            }
            OpKind::X => {
                let u = gates::x();
                let full = expand_1q(&u, circ.n_qubits, op.targets[0]);
                let qop = QOp::try_new_unitary(full).map_err(|e: String| anyhow!(e))?;
                psi = qop.apply(&psi).map_err(|e: String| anyhow!(e))?;
            }
            OpKind::CNOT => {
                let c = op.targets[0];
                let t = op.targets[1];
                let full = ops::cnot_n(circ.n_qubits, c, t).m;
                let qop = QOp::try_new_unitary(full).map_err(|e: String| anyhow!(e))?;
                psi = qop.apply(&psi).map_err(|e: String| anyhow!(e))?;
            }
        }
    }

    if let Some(nshots) = shots {
        if circ.n_qubits == 1 {
            let (c0, c1) = sample_1q(&psi, nshots);
            println!("Titan shots => 0: {c0}, 1: {c1}");
        } else {
            println!(
                "(Titan) shots currently supported for n=1 only; printing amplitudes instead."
            );
            println!("Titan |ψ_out> amplitudes: {:?}", psi.data.as_slice());
        }
    } else {
        println!("Titan |ψ_out> amplitudes: {:?}", psi.data.as_slice());
    }

    Ok(())
}

#[cfg(feature = "qiskit")]
fn run_aer(file: PathBuf, shots: Option<usize>) -> Result<()> {
    use crate::core::quantum_ir::OpKind;

    let circ = parse_ai_to_ir(&file).context("parse .ai failed")?;

    // Only support a single 1-qubit H/X program for Aer demo, or a 2-qubit H;CNOT Bell.
    // Build a single 1q unitary if possible, otherwise handle Bell special-case.
    let mut is_1q = circ.n_qubits == 1;
    let mut oneq_unitary: Option<DMatrix<C64>> = None;

    if is_1q {
        // Fold a sequence of H/X into one unitary (left-to-right apply).
        let mut u = DMatrix::<C64>::from_diagonal_element(2, 2, C64::new(1.0, 0.0));
        for op in &circ.ops {
            match op.kind {
                OpKind::H => u = gates::h() * u.clone(),
                OpKind::X => u = gates::x() * u.clone(),
                OpKind::CNOT => {
                    is_1q = false;
                    break;
                }
            }
        }
        if is_1q {
            oneq_unitary = Some(u);
        }
    }

    if is_1q {
        let shots = shots.unwrap_or(2000);
        let (c0, c1) = qiskit_bridge::run_1q_unitary_shots(&oneq_unitary.unwrap(), shots)?;
        println!("Aer shots => 0: {c0}, 1: {c1}");
        return Ok(());
    }

    // Bell special-case: qubits 2, ops == [H 0, CNOT 0 1]
    if circ.n_qubits == 2 && circ.ops.len() == 2 {
        if matches!(circ.ops[0].kind, OpKind::H)
            && matches!(circ.ops[1].kind, OpKind::CNOT)
            && circ.ops[0].targets == vec![0]
            && circ.ops[1].targets == vec![0, 1]
        {
            // Equivalent to 1q H expanded then CNOT — just ask Aer to run H on 0
            // and then a CNOT via a tiny Python helper would be ideal, but our current
            // qiskit_bridge exposes only 1q unitary shots. For now: simulate on Titan
            // and sample locally to show balanced counts.
            let mut psi = QState::zeros(2);
            let h = expand_1q(&gates::h(), 2, 0);
            let qop_h = QOp::try_new_unitary(h).map_err(|e: String| anyhow!(e))?;
            psi = qop_h.apply(&psi).map_err(|e: String| anyhow!(e))?;
            let cnot =
                QOp::try_new_unitary(ops::cnot_n(2, 0, 1).m).map_err(|e: String| anyhow!(e))?;
            psi = cnot.apply(&psi).map_err(|e: String| anyhow!(e))?;

            if let Some(nshots) = shots {
                // crude 2-qubit sampler for {00,11} only (Bell path)
                let amps = psi.data.as_slice();
                let p00 = (amps[0].norm_sqr()) as f64;
                let p11 = (amps[3].norm_sqr()) as f64;
                let mut c00 = 0usize;
                let mut c11 = 0usize;
                let mut rng = rand::thread_rng();
                for _ in 0..nshots {
                    let r: f64 = rng.gen();
                    if r < p00 {
                        c00 += 1;
                    } else {
                        c11 += 1;
                    }
                }
                println!("(Aer demo via local sample) Bell shots => 00: {c00}, 11: {c11}");
            } else {
                println!(
                    "(Aer demo via local) Bell |ψ_out> amplitudes: {:?}",
                    psi.data.as_slice()
                );
            }
            return Ok(());
        }
    }

    bail!("Aer path currently exposes 1-qubit unitaries only (or Bell demo).");
}

/// Build an n-qubit unitary that applies `u` (2×2) on `target` and I elsewhere.
fn expand_1q(u: &DMatrix<C64>, n: usize, target: usize) -> DMatrix<C64> {
    debug_assert_eq!(u.nrows(), 2);
    debug_assert_eq!(u.ncols(), 2);

    let id2 = DMatrix::<C64>::from_diagonal_element(2, 2, C64::new(1.0, 0.0));
    let mut out = DMatrix::<C64>::from_diagonal_element(1, 1, C64::new(1.0, 0.0));
    for q in 0..n {
        out = kron(&out, if q == target { u } else { &id2 });
    }
    out
}

/// Kronecker product A ⊗ B.
fn kron(a: &DMatrix<C64>, b: &DMatrix<C64>) -> DMatrix<C64> {
    let (ar, ac) = a.shape();
    let (br, bc) = b.shape();
    let mut m = DMatrix::<C64>::from_element(ar * br, ac * bc, C64::new(0.0, 0.0));
    for i in 0..ar {
        for j in 0..ac {
            let aij = a[(i, j)];
            for k in 0..br {
                for l in 0..bc {
                    m[(i * br + k, j * bc + l)] = aij * b[(k, l)];
                }
            }
        }
    }
    m
}

/// Simple 1-qubit sampler (|ψ⟩ = a|0⟩ + b|1⟩ → counts for 0/1).
fn sample_1q(psi: &QState, shots: usize) -> (usize, usize) {
    let a0 = psi.data[0];
    let a1 = psi.data[1];
    let p1 = a1.norm_sqr();
    let p0 = (1.0 - p1).max(0.0); // numerical safety

    let mut c0 = 0usize;
    let mut c1 = 0usize;
    let mut rng = rand::thread_rng();
    for _ in 0..shots {
        let r: f64 = rng.gen();
        if r < p0 {
            c0 += 1;
        } else {
            c1 += 1;
        }
    }
    (c0, c1)
}
