// 1D Time-Dependent Schrödinger_Equation via Crank–Nicolson (ħ = m = 1)
// i ∂ψ/∂t = [ -1/2 ∂²/∂x² + V(x) ] ψ
// No external BLAS needed; custom complex tridiagonal (Thomas) solver.

use num_complex::Complex64;
use std::f64::consts::PI;

fn main() {
    // ---- grid & time ----
    let x_min = -10.0;
    let x_max = 10.0;
    let n: usize = 800;                // grid points
    let steps: usize = 1500;           // time steps to simulate
    let output_every: usize = 300;

    let dx = (x_max - x_min) / (n as f64 - 1.0);
    let dt = 5e-4;                     // keep small enough for accuracy

    // ---- potential: harmonic well (0.5 * ω^2 * x^2). Change as you like. ----
    let omega = 1.0;
    let x: Vec<f64> = (0..n).map(|i| x_min + i as f64 * dx).collect();
    let v: Vec<f64> = x.iter().map(|&xi| 0.5 * omega * omega * xi * xi).collect();

    // ---- initial state: Gaussian wave packet ----
    let x0 = -4.0;                     // center
    let sigma = 0.5;                   // width
    let k0 = 6.0;                      // momentum (sets group velocity)
    let mut psi: Vec<Complex64> = x.iter()
        .map(|&xi| {
            let gauss = (-((xi - x0).powi(2)) / (2.0 * sigma * sigma)).exp();
            let phase = Complex64::new(0.0, k0 * xi).exp();
            Complex64::new(gauss, 0.0) * phase
        })
        .collect();
    normalize(&mut psi, dx);

    // ---- precompute CN matrices: A ψ^{n+1} = B ψ^{n} ----
    // H = -0.5 D2 + V
    // A = I + i dt/2 H,  B = I - i dt/2 H
    let iunit = Complex64::new(0.0, 1.0);
    let alpha = iunit * (dt * 0.5);

    // Discrete Laplacian pieces:
    let inv_dx2 = 1.0 / (dx * dx);
    // For H: diag contribution from Laplacian is + (1/dx^2), off-diagonals are -1/(2 dx^2)
    let h_off = -0.5 * inv_dx2; // off-diagonal (real)
    let h_diag_base = inv_dx2;  // base (from kinetic) plus V_i later

    // A tridiagonal coefficients
    let a_off = alpha * h_off; // same for sub/super
    let mut a_diag: Vec<Complex64> = v
        .iter()
        .map(|&vi| Complex64::new(1.0, 0.0) + alpha * Complex64::new(h_diag_base + vi, 0.0))
        .collect();

    // B tridiagonal coefficients (we won't solve with B; we use it to build RHS)
    let b_off = -a_off; // because B = I - i dt/2 H
    let b_diag: Vec<Complex64> = v
        .iter()
        .map(|&vi| Complex64::new(1.0, 0.0) - alpha * Complex64::new(h_diag_base + vi, 0.0))
        .collect();

    // Enforce Dirichlet boundaries (ψ=0 at ends): set A[0,0]=A[n-1,n-1]=1 and kill couplings
    a_diag[0]  = Complex64::new(1.0, 0.0);
    a_diag[n-1]= Complex64::new(1.0, 0.0);

    // Pre-allocate RHS and work buffers
    let mut rhs = vec![Complex64::new(0.0, 0.0); n];
    let mut psi_next = psi.clone();

    for t in 0..steps {
        // Build RHS = B * psi
        rhs[0] = Complex64::new(0.0, 0.0);
        for i in 1..n-1 {
            rhs[i] = b_diag[i] * psi[i] + b_off * (psi[i - 1] + psi[i + 1]);
        }
        rhs[n - 1] = Complex64::new(0.0, 0.0);

        // Solve A * psi_next = RHS with complex Thomas algorithm
        complex_thomas_solve(
            n,
            a_diag.as_slice(),
            a_off,               // sub-diagonal (constant)
            a_off,               // super-diagonal (constant)
            rhs.as_slice(),
            &mut psi_next,
        );

        // Hard-apply boundary conditions (safety)
        psi_next[0] = Complex64::new(0.0, 0.0);
        psi_next[n - 1] = Complex64::new(0.0, 0.0);

        // Renormalize (CN is unitary in exact arithmetic; floating error creeps)
        normalize(&mut psi_next, dx);

        // swap
        psi.copy_from_slice(&psi_next);

        if t % output_every == 0 {
            // quick diagnostics: probability conservation and <x>, <p> rough readouts
            let prob = total_probability(&psi, dx);
            let exp_x = expectation_x(&psi, &x, dx);
            println!("step={:5}  P≈{:.6}  <x>≈{:+.4}", t, prob, exp_x);
        }
    }

    // Final dump of |ψ|^2 for plotting elsewhere (CSV: x,prob)
    println!("# x, |psi|^2");
    for i in 0..n {
        println!("{:.8}, {:.8}", x[i], psi[i].norm_sqr());
    }
}

fn normalize(psi: &mut [Complex64], dx: f64) {
    let norm: f64 = psi.iter().map(|c| c.norm_sqr()).sum::<f64>() * dx;
    let scale = 1.0 / norm.sqrt();
    for c in psi.iter_mut() {
        *c *= scale;
    }
}

fn total_probability(psi: &[Complex64], dx: f64) -> f64 {
    psi.iter().map(|c| c.norm_sqr()).sum::<f64>() * dx
}

fn expectation_x(psi: &[Complex64], x: &[f64], dx: f64) -> f64 {
    psi.iter().zip(x.iter())
        .map(|(c, &xi)| c.norm_sqr() * xi)
        .sum::<f64>() * dx
}

/// Solve tridiagonal system with constant off-diagonals:
///   a_diag[i] * x[i] + a_sub * x[i-1] + a_sup * x[i+1] = rhs[i]
/// Dirichlet rows (i=0, n-1) assumed already embedded in a_diag/rhs by caller.
/// Output written to `x_out`.
fn complex_thomas_solve(
    n: usize,
    a_diag: &[Complex64],
    a_sub: Complex64,
    a_sup: Complex64,
    rhs: &[Complex64],
    x_out: &mut [Complex64],
) {
    // forward sweep
    let mut cprime = vec![Complex64::new(0.0, 0.0); n];
    let mut dprime = vec![Complex64::new(0.0, 0.0); n];

    let mut denom = a_diag[0];
    cprime[0] = a_sup / denom;
    dprime[0] = rhs[0] / denom;

    for i in 1..n {
        denom = a_diag[i] - a_sub * cprime[i - 1];
        // For the last row, cprime[n-1] not used, but compute safely
        cprime[i] = a_sup / denom;
        dprime[i] = (rhs[i] - a_sub * dprime[i - 1]) / denom;
    }

    // back substitution
    x_out[n - 1] = dprime[n - 1];
    for i in (0..n - 1).rev() {
        x_out[i] = dprime[i] - cprime[i] * x_out[i + 1];
    }
}

// Optional: plane-wave phase, if needed elsewhere
#[allow(dead_code)]
fn phase(k: f64, x: f64) -> Complex64 {
    Complex64::new(0.0, k * x).exp()
}
