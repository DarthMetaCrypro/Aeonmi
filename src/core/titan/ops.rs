//! Kronecker products and controlled lifts (feature: `quantum`).
use crate::core::titan::types::QOp;
use nalgebra::DMatrix;
use num_complex::Complex64 as C64;

#[inline]
fn c(r: f64, i: f64) -> C64 {
    C64::new(r, i)
}

/// Kronecker product A ⊗ B
pub fn kron(a: &DMatrix<C64>, b: &DMatrix<C64>) -> DMatrix<C64> {
    let (ar, ac) = (a.nrows(), a.ncols());
    let (br, bc) = (b.nrows(), b.ncols());
    let mut out = DMatrix::<C64>::from_element(ar * br, ac * bc, c(0.0, 0.0));
    for i in 0..ar {
        for j in 0..ac {
            let aij = a[(i, j)];
            for k in 0..br {
                for l in 0..bc {
                    out[(i * br + k, j * bc + l)] = aij * b[(k, l)];
                }
            }
        }
    }
    out
}

/// Promote a 1-qubit gate `u` onto `n_qubits`, targeting `target` index (0 = least significant).
pub fn lift_1q(u: &DMatrix<C64>, n_qubits: usize, target: usize) -> QOp {
    assert_eq!(u.nrows(), 2);
    assert_eq!(u.ncols(), 2);
    assert!(target < n_qubits);

    // Build by kron-ing I or U in the right slot with little-endian qubit order.
    let i2 = DMatrix::<C64>::identity(2, 2);
    let mut acc = DMatrix::<C64>::from_element(1, 1, c(1.0, 0.0));
    for q in 0..n_qubits {
        let m = if q == target { u.clone() } else { i2.clone() };
        acc = kron(&acc, &m);
    }
    QOp { m: acc }
}

/// Build an n-qubit CNOT as a full 2^n unitary (control -> target).
pub fn cnot_n(n_qubits: usize, control: usize, target: usize) -> QOp {
    assert!(control < n_qubits && target < n_qubits && control != target);
    let dim = 1usize << n_qubits;
    let mut m = DMatrix::<C64>::from_element(dim, dim, c(0.0, 0.0));
    for basis in 0..dim {
        let bit_c = (basis >> control) & 1;
        let mut out = basis;
        if bit_c == 1 {
            // flip target bit
            out ^= 1usize << target;
        }
        m[(out, basis)] = c(1.0, 0.0);
    }
    QOp { m }
}
