use aeonmi_project::core::titan::quantum_math::{inner_product, normalize_state};

#[test]
fn normalize_and_inner_product() {
    let v = vec![3.0, 4.0]; // ||v|| = 5
    let n = normalize_state(&v); // should be [0.6, 0.8]
    let dot = inner_product(&n, &n); // ≈ 1.0
    assert!((dot - 1.0).abs() < 1e-9);
}
