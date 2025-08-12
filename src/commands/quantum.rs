#![cfg(feature = "quantum")]
//! CLI entry for quantum runs.
//!
//! Backends:
//! - `titan` : local Titan path (matrix apply)
//! - `aer`   : Qiskit Aer simulator (requires `--features qiskit` + Python/Qiskit)
//! - `ibmq`  : placeholder (will route via Qiskit when enabled)
//!
//! Examples:
//!   cargo run --features quantum -- quantum --backend titan foo.ai
//!   cargo run --features "quantum,qiskit" -- quantum --backend aer foo.ai --shots 2000

#![allow(dead_code)]

use std::path::PathBuf;
use anyhow::{anyhow, bail, Context, Result};

use crate::core::titan::{
    gates,
    types::{QOp, QState},
};

#[cfg(feature = "qiskit")]
use crate::core::titan::qiskit_bridge;

/// Run a quantum job from a (future) .ai file.
/// For now we ignore the file contents and execute small smoke paths.
pub fn quantum_run(file: PathBuf, backend: &str, _shots: Option<usize>) -> Result<()> {
    // Normalize backend selection (case-insensitive).
    let be = backend.to_ascii_lowercase();
    // Keep the path around for future IR; don’t warn for now.
    let _ = &file;

    match be.as_str() {
        "titan" => {
            use nalgebra::DVector;
            use num_complex::Complex64 as C64;

            // Apply H on |0⟩ → |+⟩
            let h = gates::h();
            let op = QOp::try_new_unitary(h)
                .map_err(|e: String| anyhow!(e))
                .context("failed to construct unitary operator for H")?;

            // Build |0⟩ explicitly as a length-2 vector: [1, 0]^T
            // (For n qubits, state length must be 2^n.)
            let psi0 = QState::try_new(
                DVector::from_vec(vec![C64::new(1.0, 0.0), C64::new(0.0, 0.0)]),
                /*auto_normalize*/ false,
            )
            .map_err(|e: String| anyhow!(e))
            .context("failed to construct |0⟩ state")?;

            let psi1 = op
                .apply(&psi0)
                .map_err(|e: String| anyhow!(e))
                .context("failed to apply operator to state")?;

            println!("Titan |ψ_out> amplitudes: {:?}", psi1.data.as_slice());
            Ok(())
        }

        // Accept "aer" (and alias "qiskit") for convenience.
        "aer" | "qiskit" => {
            #[cfg(feature = "qiskit")]
            {
                let h = gates::h();
                let nshots = _shots.unwrap_or(2000);
                let (c0, c1) = qiskit_bridge::run_1q_unitary_shots(&h, nshots)
                    .context("Qiskit Aer run failed")?;
                println!("Aer shots => 0: {c0}, 1: {c1}");
                Ok(())
            }
            #[cfg(not(feature = "qiskit"))]
            {
                bail!("Backend 'aer' requires `--features qiskit` and a working Python/Qiskit install.");
            }
        }

        "ibmq" => {
            #[cfg(feature = "qiskit")]
            {
                // Placeholder: we’ll wire account/backends soon.
                let ver = qiskit_bridge::qiskit_version().unwrap_or_default();
                bail!("IBMQ path not wired yet (Qiskit v{ver}). Use `aer` for now.");
            }
            #[cfg(not(feature = "qiskit"))]
            {
                bail!("Backend 'ibmq' requires `--features qiskit` and a working Python/Qiskit install.");
            }
        }

        other => bail!("unsupported backend: {other} (try: titan | aer | ibmq)"),
    }
}
