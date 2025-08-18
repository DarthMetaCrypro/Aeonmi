use aeonmi_project::core::titan::probability_statistics::{
    random_sample_normal, random_sample_uniform,
};

#[test]
fn uniform_len_and_range() {
    let xs = random_sample_uniform(0.0, 1.0, 10_000);
    assert_eq!(xs.len(), 10_000);
    assert!(xs.iter().all(|&v| v >= 0.0 && v <= 1.0));
}

#[test]
fn normal_len_and_moments() {
    let xs = random_sample_normal(0.0, 1.0, 20_000);
    assert_eq!(xs.len(), 20_000);
    let mean = xs.iter().copied().sum::<f64>() / xs.len() as f64;
    let var = xs.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / xs.len() as f64;
    assert!(mean.abs() < 0.05, "mean ~= 0, got {mean}");
    assert!((var - 1.0).abs() < 0.1, "var ~= 1, got {var}");
}
