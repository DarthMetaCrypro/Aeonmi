pub fn tensor_product(matrix1: &[Vec<f64>], matrix2: &[Vec<f64>]) -> Vec<Vec<f64>> {
    // Computes the Kronecker product of two matrices
    let mut result = Vec::new();
    for row1 in matrix1 {
        for row2 in matrix2 {
            let mut new_row = Vec::new();
            for &val1 in row1 {
                for &val2 in row2 {
                    new_row.push(val1 * val2);
                }
            }
            result.push(new_row);
        }
    }
    result
}

pub fn quantum_superposition(state1: &[f64], state2: &[f64], coeff1: f64, coeff2: f64) -> Vec<f64> {
    // Combines two quantum states into a superposed state
    if state1.len() != state2.len() {
        panic!("States must have the same length for superposition.");
    }
    state1
        .iter()
        .zip(state2.iter())
        .map(|(&s1, &s2)| coeff1 * s1 + coeff2 * s2)
        .collect()
}

pub fn measure_state(probabilities: &[f64]) -> usize {
    // Simulates the measurement of a quantum state based on probabilities
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random_value: f64 = rng.gen();
    let mut cumulative_probability = 0.0;

    for (index, &probability) in probabilities.iter().enumerate() {
        cumulative_probability += probability;
        if random_value <= cumulative_probability {
            return index;
        }
    }
    probabilities.len() - 1 // Default to the last state if rounding issues occur
}

pub fn normalize_state(state: &[f64]) -> Vec<f64> {
    // Normalizes a quantum state vector so that the sum of squares equals 1
    let norm: f64 = state.iter().map(|&val| val * val).sum::<f64>().sqrt();
    if norm == 0.0 {
        panic!("Cannot normalize a zero vector.");
    }
    state.iter().map(|&val| val / norm).collect()
}

pub fn inner_product(state1: &[f64], state2: &[f64]) -> f64 {
    // Computes the inner product of two quantum states
    if state1.len() != state2.len() {
        panic!("States must have the same length for inner product calculation.");
    }
    state1.iter().zip(state2.iter()).map(|(&s1, &s2)| s1 * s2).sum()
}
