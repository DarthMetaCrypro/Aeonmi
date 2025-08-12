//! Qiskit bridge via pyo3 (feature: `qiskit`).
//! Minimal demo: send a unitary to Python/Qiskit, build a circuit, and run shots.

use anyhow::{bail, Result};
use nalgebra::DMatrix;
use num_complex::Complex64 as C64;
use numpy::PyArray2;
use pyo3::prelude::*;
use pyo3::types::PyModule;

/// Convert a nalgebra complex matrix to a NumPy (complex128) 2D array.
fn to_pyarray<'py>(py: Python<'py>, m: &DMatrix<C64>) -> &'py PyArray2<num_complex::Complex64> {
    let (r, c) = (m.nrows(), m.ncols());
    let mut rows: Vec<Vec<num_complex::Complex64>> = Vec::with_capacity(r);
    for i in 0..r {
        let mut row: Vec<num_complex::Complex64> = Vec::with_capacity(c);
        for j in 0..c {
            let z = m[(i, j)];
            row.push(num_complex::Complex64 { re: z.re, im: z.im });
        }
        rows.push(row);
    }

    // In numpy = 0.21, `from_vec2` is marked deprecated in favor of a future `from_vec2_bound`.
    // Use a localized allow to keep the build warning-free without changing behavior.
    #[allow(deprecated)]
    {
        PyArray2::from_vec2(py, &rows).expect("PyArray2::from_vec2 failed")
    }
}

/// Return Qiskit version as a quick smoke test.
pub fn qiskit_version() -> Result<String> {
    Python::with_gil(|py| {
        // Use the new bound API to silence deprecation warnings in PyO3 0.21+.
        let qiskit = PyModule::import_bound(py, "qiskit")?;
        let ver: String = qiskit.getattr("__version__")?.extract()?;
        Ok(ver)
    })
}

/// Send a 1-qubit unitary and run it on |0>, measuring Z-basis shots.
/// Returns (counts of 0, counts of 1).
pub fn run_1q_unitary_shots(u: &DMatrix<C64>, shots: usize) -> Result<(u64, u64)> {
    if u.nrows() != 2 || u.ncols() != 2 {
        bail!("expected 2x2 unitary");
    }

    Python::with_gil(|py| -> Result<(u64, u64)> {
        let np_u = to_pyarray(py, u);
        let code = r#"
import numpy as np
from qiskit import QuantumCircuit, transpile
from qiskit_aer import Aer
from qiskit.quantum_info import Operator

def run_u(u, shots):
    qc = QuantumCircuit(1, 1)
    qc.append(Operator(u), [0])
    qc.measure(0, 0)
    sim = Aer.get_backend('aer_simulator')
    tqc = transpile(qc, sim)
    result = sim.run(tqc, shots=shots).result()
    counts = result.get_counts()
    c0 = counts.get('0', 0)
    c1 = counts.get('1', 0)
    return int(c0), int(c1)
"#;

        // Use new bound constructor to silence deprecation warnings.
        let m = PyModule::from_code_bound(py, code, "aeonmi_qiskit.py", "aeonmi_qiskit")?;
        let func = m.getattr("run_u")?;

        // Let PyO3 build the argument tuple from a Rust tuple.
        let (c0, c1): (u64, u64) = func.call1((np_u, shots))?.extract()?;
        Ok((c0, c1))
    })
}
