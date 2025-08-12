//! Core quantum types for Titan (feature: `quantum`).
use nalgebra::{DMatrix, DVector};
use num_complex::Complex64 as C64;

pub const EPS: f64 = 1e-12;

#[derive(Clone, Debug)]
pub struct QState {
    pub data: DVector<C64>,
}

impl QState {
    /// Create from raw vector; rejects non-normalized unless `auto_normalize = true`.
    pub fn try_new(vec: DVector<C64>, auto_normalize: bool) -> Result<Self, String> {
        let mut v = vec;
        let norm = v.iter().map(|z| z.norm_sqr()).sum::<f64>().sqrt();
        if (norm - 1.0).abs() < EPS {
            Ok(Self { data: v })
        } else if auto_normalize {
            if norm < EPS {
                return Err("QState norm ~ 0".into());
            }
            v /= C64::from(norm);
            Ok(Self { data: v })
        } else {
            Err(format!("QState not normalized (||ψ|| = {norm})"))
        }
    }

    pub fn zeros(n: usize) -> Self {
        Self { data: DVector::from_element(n, C64::new(0.0, 0.0)) }
    }

    pub fn len(&self) -> usize { self.data.len() }
}

#[derive(Clone, Debug)]
pub struct QOp {
    pub m: DMatrix<C64>,
}

impl QOp {
    pub fn try_new_unitary(m: DMatrix<C64>) -> Result<Self, String> {
        if m.nrows() != m.ncols() {
            return Err("QOp must be square".into());
        }
        // Unitarity: U^† U = I
        let u_dag_u = m.adjoint() * &m;
        let i = DMatrix::<C64>::identity(m.nrows(), m.ncols());
        let max_diff = (u_dag_u - i)
            .iter()
            .map(|z| z.norm())
            .fold(0.0_f64, f64::max);
        if max_diff > 1e-8 {
            return Err(format!("QOp not unitary (‖UᴴU−I‖∞={max_diff:e})"));
        }
        Ok(Self { m })
    }

    /// Apply to a full state vector (dimensions must match).
    pub fn apply(&self, psi: &QState) -> Result<QState, String> {
        if self.m.ncols() != psi.len() {
            return Err("dimension mismatch in QOp::apply".into());
        }
        Ok(QState { data: &self.m * &psi.data })
    }
}
