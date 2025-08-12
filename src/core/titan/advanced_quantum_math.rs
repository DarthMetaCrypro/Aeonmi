pub fn density_matrix(state: &[f64]) -> Vec<Vec<f64>> {
    // Constructs the density matrix for a given quantum state
    let n = state.len();
    let mut matrix = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in 0..n {
            matrix[i][j] = state[i] * state[j];
        }
    }

    matrix
}

pub fn quantum_fidelity(rho1: &[Vec<f64>], rho2: &[Vec<f64>]) -> Result<f64, &'static str> {
    // Computes the fidelity between two density matrices
    if rho1.len() != rho2.len() || rho1[0].len() != rho2[0].len() {
        return Err("Density matrices must have the same dimensions.");
    }

    let mut fidelity = 0.0;
    for i in 0..rho1.len() {
        for j in 0..rho1[i].len() {
            fidelity += (rho1[i][j] * rho2[i][j]).sqrt();
        }
    }

    Ok(fidelity.powi(2))
}

pub fn von_neumann_entropy(rho: &[Vec<f64>]) -> Result<f64, &'static str> {
    // Computes the von Neumann entropy of a density matrix
    if rho.len() != rho[0].len() {
        return Err("Density matrix must be square.");
    }

    let eigenvalues = rho.iter().map(|row| row.iter().sum()).collect::<Vec<f64>>(); // Placeholder for eigenvalues
    let entropy: f64 = eigenvalues
        .iter()
        .filter(|&&lambda| lambda > 0.0)
        .map(|&lambda| -lambda * lambda.log2())
        .sum();

    Ok(entropy)
}

pub fn bell_state() -> Vec<f64> {
    // Constructs the Bell state |Φ+⟩ = (|00⟩ + |11⟩) / √2
    let sqrt_2_inv = 1.0 / 2.0_f64.sqrt();
    vec![sqrt_2_inv, 0.0, 0.0, sqrt_2_inv]
}

pub fn apply_quantum_operator(state: &[f64], operator: &[Vec<f64>]) -> Result<Vec<f64>, &'static str> {
    // Applies a quantum operator (matrix) to a quantum state (vector)
    if operator.len() != state.len() || operator[0].len() != state.len() {
        return Err("Operator dimensions must match the state vector length.");
    }

    let mut result = vec![0.0; state.len()];
    for i in 0..operator.len() {
        for j in 0..operator[i].len() {
            result[i] += operator[i][j] * state[j];
        }
    }

    Ok(result)
}
