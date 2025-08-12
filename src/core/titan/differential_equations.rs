pub fn solve_ode<F>(f: F, y0: f64, x0: f64, h: f64, steps: usize) -> Vec<(f64, f64)>
where
    F: Fn(f64, f64) -> f64,
{
    // Solves an ordinary differential equation (ODE) using Euler's method
    let mut results = Vec::new();
    let mut x = x0;
    let mut y = y0;

    results.push((x, y));
    for _ in 0..steps {
        y += h * f(x, y);
        x += h;
        results.push((x, y));
    }

    results
}

pub fn solve_ode_rk4<F>(f: F, y0: f64, x0: f64, h: f64, steps: usize) -> Vec<(f64, f64)>
where
    F: Fn(f64, f64) -> f64,
{
    // Solves an ODE using the Runge-Kutta 4th Order (RK4) method
    let mut results = Vec::new();
    let mut x = x0;
    let mut y = y0;

    results.push((x, y));
    for _ in 0..steps {
        let k1 = h * f(x, y);
        let k2 = h * f(x + h / 2.0, y + k1 / 2.0);
        let k3 = h * f(x + h / 2.0, y + k2 / 2.0);
        let k4 = h * f(x + h, y + k3);

        y += (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
        x += h;
        results.push((x, y));
    }

    results
}

pub fn solve_pde_2d<F>(
    f: F,
    u0: Vec<Vec<f64>>,
    dx: f64,
    dy: f64,
    dt: f64,
    time_steps: usize,
) -> Vec<Vec<Vec<f64>>>
where
    F: Fn(f64, f64, f64, f64, f64, f64) -> f64,
{
    // Solves a 2D Partial Differential Equation (PDE) using finite differences
    let mut u = u0.clone();
    let rows = u.len();
    let cols = u[0].len();
    let mut results = vec![u.clone()];

    for _ in 0..time_steps {
        let mut next_u = u.clone();
        for i in 1..rows - 1 {
            for j in 1..cols - 1 {
                let ux = (u[i + 1][j] - u[i - 1][j]) / (2.0 * dx);
                let uy = (u[i][j + 1] - u[i][j - 1]) / (2.0 * dy);
                let uxx = (u[i + 1][j] - 2.0 * u[i][j] + u[i - 1][j]) / dx.powi(2);
                let uyy = (u[i][j + 1] - 2.0 * u[i][j] + u[i][j - 1]) / dy.powi(2);
                next_u[i][j] = f(u[i][j], ux, uy, uxx, uyy, dt);
            }
        }
        u = next_u.clone();
        results.push(u.clone());
    }

    results
}
