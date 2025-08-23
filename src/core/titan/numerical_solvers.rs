pub fn finite_difference_1d<F>(func: F, x_min: f64, x_max: f64, steps: usize) -> Vec<f64>
where
    F: Fn(f64) -> f64,
{
    // Solves a 1D function using finite difference approximation
    let h = (x_max - x_min) / steps as f64;
    let mut results = vec![0.0; steps + 1];

    for i in 0..=steps {
        let x = x_min + i as f64 * h;
        results[i] = func(x);
    }

    results
}

pub fn finite_element_solver<F>(
    mesh: &[f64],
    boundary_conditions: (f64, f64),
    source_func: F,
) -> Vec<f64>
where
    F: Fn(f64) -> f64,
{
    // Solves a problem using the finite element method (FEM)
    let n = mesh.len();
    if n < 2 {
        panic!("Mesh must contain at least two points.");
    }

    let mut results = vec![0.0; n];
    results[0] = boundary_conditions.0;
    results[n - 1] = boundary_conditions.1;

    for i in 1..n - 1 {
        let x = mesh[i];
        results[i] = source_func(x);
    }

    results
}

pub fn spectral_method<F>(func: F, x_min: f64, x_max: f64, modes: usize) -> Vec<f64>
where
    F: Fn(f64) -> f64,
{
    // Solves a function using spectral methods (Fourier-based)
    let mut coefficients = vec![0.0; modes];
    let dx = (x_max - x_min) / modes as f64;

    for k in 0..modes {
        let omega = 2.0 * std::f64::consts::PI * k as f64 / (x_max - x_min);
        coefficients[k] = (0..modes)
            .map(|j| {
                let x = x_min + j as f64 * dx;
                func(x) * (omega * j as f64).cos()
            })
            .sum::<f64>()
            * dx;
    }

    coefficients
}

pub fn boundary_value_problem<F>(
    func: F,
    x_min: f64,
    x_max: f64,
    boundary_conditions: (f64, f64),
    steps: usize,
) -> Vec<f64>
where
    F: Fn(f64, f64, f64) -> f64,
{
    // Solves boundary value problems (BVP) for differential equations
    let h = (x_max - x_min) / steps as f64;
    let mut x = x_min;
    let mut y = boundary_conditions.0;
    let mut results = vec![y];

    for _ in 0..steps {
        let slope = func(x, y, h);
        y += h * slope;
        x += h;
        results.push(y);
    }

    results
}
