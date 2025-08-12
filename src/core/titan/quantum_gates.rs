pub fn pauli_x() -> Vec<Vec<f64>> {
    vec![vec![0.0, 1.0], vec![1.0, 0.0]]
}

pub fn pauli_y() -> Vec<Vec<f64>> {
    vec![vec![0.0, -1.0], vec![1.0, 0.0]]
}

pub fn pauli_z() -> Vec<Vec<f64>> {
    vec![vec![1.0, 0.0], vec![0.0, -1.0]]
}

pub fn hadamard() -> Vec<Vec<f64>> {
    let scale = 1.0 / 2.0_f64.sqrt();
    vec![vec![scale, scale], vec![scale, -scale]]
}

pub fn cnot() -> Vec<Vec<f64>> {
    vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![0.0, 0.0, 0.0, 1.0],
        vec![0.0, 0.0, 1.0, 0.0],
    ]
}
