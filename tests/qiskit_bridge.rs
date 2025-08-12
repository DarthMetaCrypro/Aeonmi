#[cfg(feature = "qiskit")]
mod q {
    use aeonmi_project::core::titan::{gates, qiskit_bridge};

    #[test]
    fn qiskit_version_ok() {
        let v = qiskit_bridge::qiskit_version().unwrap();
        assert!(!v.is_empty());
        println!("Qiskit version: {}", v);
    }

    #[test]
    fn run_hadamard_shots() {
        let h = gates::h();
        let (c0, c1) = qiskit_bridge::run_1q_unitary_shots(&h, 2000).unwrap();
        // Roughly balanced
        let diff = (c0 as i64 - c1 as i64).abs();
        println!("H shots => 0: {}, 1: {}", c0, c1);
        assert!(diff < 400, "imbalanced: {c0} vs {c1}");
    }
}
