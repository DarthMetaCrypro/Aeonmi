#[cfg(feature = "quantum")]
mod q {
    use nalgebra::DVector;
    use num_complex::Complex64 as C64;
    use aeonmi_project::core::titan::{
        types::{QState, QOp},
        gates, ops
    };

    #[test]
    fn hadamard_on_zero() {
        let h = gates::h();
        let op = QOp::try_new_unitary(h).unwrap();
        let psi0 = QState::try_new(
            DVector::from_vec(vec![C64::new(1.0,0.0), C64::new(0.0,0.0)]),
            false
        ).unwrap();
        let psi1 = op.apply(&psi0).unwrap();
        // |+> amplitudes ~ [1/√2, 1/√2]
        let a0 = psi1.data[0].re;
        let a1 = psi1.data[1].re;
        assert!((a0 - 0.70710678).abs() < 1e-6);
        assert!((a1 - 0.70710678).abs() < 1e-6);
    }

    #[test]
    fn cnot_build() {
        let cnot = ops::cnot_n(2, 0, 1).m;
        assert_eq!(cnot.nrows(), 4);
        assert_eq!(cnot.ncols(), 4);
    }
}
