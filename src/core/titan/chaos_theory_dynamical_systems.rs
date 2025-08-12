pub fn lyapunov_exponent(
    initial_conditions: &[f64],
    system: impl Fn(&[f64]) -> Vec<f64>,
    perturbation: f64,
    iterations: usize,
) -> Result<f64, &'static str> {
    // Computes the Lyapunov exponent for a dynamical system
    if initial_conditions.is_empty() {
        return Err("Initial conditions cannot be empty.");
    }

    let mut state = initial_conditions.to_vec();
    let mut perturbed_state: Vec<f64> = state.iter().map(|&x| x + perturbation).collect();
    let mut divergence = perturbation.abs();

    for _ in 0..iterations {
        state = system(&state);
        perturbed_state = system(&perturbed_state);

        divergence = perturbed_state
            .iter()
            .zip(&state)
            .map(|(p, s)| (p - s).abs())
            .sum::<f64>()
            / state.len() as f64;

        if divergence <= 1e-10 {
            return Err("Divergence collapsed to zero; Lyapunov exponent is undefined.");
        }
    }

    Ok((divergence / perturbation).ln() / iterations as f64)
}

pub fn bifurcation_map(
    parameter_range: &[f64],
    system: impl Fn(f64, f64) -> f64,
    initial_state: f64,
    iterations: usize,
    discard: usize,
) -> Vec<(f64, Vec<f64>)> {
    // Generates a bifurcation map for a parameterized system
    let mut map = Vec::new();

    for &param in parameter_range {
        let mut state = initial_state;
        let mut values = Vec::new();

        for iteration in 0..(iterations + discard) {
            state = system(state, param);
            if iteration >= discard {
                values.push(state);
            }
        }

        map.push((param, values));
    }

    map
}

pub fn attractor_reconstruction(
    time_series: &[f64],
    delay: usize,
    dimensions: usize,
) -> Result<Vec<Vec<f64>>, &'static str> {
    // Reconstructs a phase-space attractor from a time series
    if time_series.len() < (dimensions - 1) * delay + 1 {
        return Err("Time series is too short for the specified dimensions and delay.");
    }

    let mut attractor = Vec::new();
    for i in 0..(time_series.len() - (dimensions - 1) * delay) {
        let point: Vec<f64> = (0..dimensions)
            .map(|d| time_series[i + d * delay])
            .collect();
        attractor.push(point);
    }

    Ok(attractor)
}
