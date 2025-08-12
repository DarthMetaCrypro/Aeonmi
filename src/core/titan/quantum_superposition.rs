pub fn create_superposition(state: &[f64]) -> Vec<f64> {
    let norm: f64 = state.iter().map(|&x| x * x).sum::<f64>().sqrt();
    if norm == 0.0 {
        panic!("Cannot normalize a zero vector.");
    }
    state.iter().map(|&x| x / norm).collect()
}

pub fn apply_gate(state: &[f64], gate: &[Vec<f64>]) -> Vec<f64> {
    if state.len() != gate.len() {
        panic!("State vector length and gate dimension must match.");
    }
    gate.iter()
        .map(|row| {
            row.iter()
                .zip(state.iter())
                .map(|(&g, &s)| g * s)
                .sum::<f64>()
        })
        .collect()
}
