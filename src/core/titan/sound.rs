// src/core/titan/math/sound.rs

use num_complex::Complex64 as C;
use std::f64::consts::PI;

/// 31) 1D wave eq (fixed ends) — leapfrog
/// u^{n+1}_i = 2u^n_i - u^{n-1}_i + r^2 (u^n_{i+1} - 2u^n_i + u^n_{i-1}),
/// r = c Δt / Δx. Requires r ≤ 1 for stability.
/// Returns u_next; boundaries pinned to 0.
pub fn wave1d_leapfrog_step(
    u_curr: &[f64],
    u_prev: &[f64],
    c: f64,
    dx: f64,
    dt: f64,
) -> Vec<f64> {
    let n = u_curr.len();
    assert_eq!(u_prev.len(), n);
    assert!(n >= 3, "need at least 3 points");
    let r2 = (c * dt / dx).powi(2);
    let mut u_next = vec![0.0; n];
    for i in 1..n - 1 {
        let lap = u_curr[i - 1] - 2.0 * u_curr[i] + u_curr[i + 1];
        u_next[i] = 2.0 * u_curr[i] - u_prev[i] + r2 * lap;
    }
    // fixed ends (Dirichlet)
    u_next[0] = 0.0;
    u_next[n - 1] = 0.0;
    u_next
}

/// 32) Discrete Fourier Transform (DFT) — simple, O(N^2)
/// Returns complex spectrum X[k], k=0..N-1. (Use FFT for large N.)
pub fn dft(signal: &[f64]) -> Vec<C> {
    let n = signal.len();
    let mut out = vec![C::new(0.0, 0.0); n];
    for k in 0..n {
        let mut acc = C::new(0.0, 0.0);
        for (n_idx, &x) in signal.iter().enumerate() {
            let ang = -2.0 * PI * (k as f64) * (n_idx as f64) / (n as f64);
            acc += C::from_polar(x, ang);
        }
        out[k] = acc;
    }
    out
}

/// 33) 1D Helmholtz solver (Dirichlet) on uniform grid
/// Discretize: (ψ_{i-1} - 2ψ_i + ψ_{i+1})/dx^2 + k^2 ψ_i = s_i
/// Solve tridiagonal A ψ = s for interior i=1..N-2; ψ_0=left, ψ_{N-1}=right.
pub fn helmholtz_1d_dirichlet(
    k: f64,
    dx: f64,
    source: &[f64],   // length N
    left_bc: f64,
    right_bc: f64,
) -> Vec<f64> {
    let n = source.len();
    assert!(n >= 3);
    let m = n - 2; // interior unknowns
    let a = 1.0 / (dx * dx);
    let b = -2.0 / (dx * dx) + k * k;

    // build tridiagonal
    let sub = vec![a; m - 1];
    let mut diag = vec![b; m];
    let sup = vec![a; m - 1];
    let mut rhs = vec![0.0; m];
    // RHS includes boundary terms
    for i in 0..m {
        rhs[i] = source[i + 1];
    }
    rhs[0] -= a * left_bc;
    rhs[m - 1] -= a * right_bc;

    // Thomas algorithm
    let mut cprime = vec![0.0; m - 1];
    let mut dprime = vec![0.0; m];
    cprime[0] = sup[0] / diag[0];
    dprime[0] = rhs[0] / diag[0];
    for i in 1..m - 1 {
        let denom = diag[i] - sub[i - 1] * cprime[i - 1];
        cprime[i] = sup[i] / denom;
        dprime[i] = (rhs[i] - sub[i - 1] * dprime[i - 1]) / denom;
    }
    let denom = diag[m - 1] - sub[m - 2] * cprime[m - 2];
    dprime[m - 1] = (rhs[m - 1] - sub[m - 2] * dprime[m - 2]) / denom;

    // back-substitution
    let mut psi_int = vec![0.0; m];
    psi_int[m - 1] = dprime[m - 1];
    for i in (0..m - 1).rev() {
        psi_int[i] = dprime[i] - cprime[i] * psi_int[i + 1];
    }

    // stitch with boundaries
    let mut psi = vec![0.0; n];
    psi[0] = left_bc;
    for i in 0..m { psi[i + 1] = psi_int[i]; }
    psi[n - 1] = right_bc;
    psi
}

/// 34) String/pipe resonance modes (open-open string)
/// f_n = n v / (2L), n = 1..modes
pub fn resonance_modes(v: f64, length_l: f64, modes: usize) -> Vec<f64> {
    (1..=modes).map(|n| (n as f64) * v / (2.0 * length_l)).collect()
}

/// 35) Harmonic oscillator analytic solution
/// m x'' + k x = 0 → ω = √(k/m)
/// x(t) = A cos(ω t) + B sin(ω t), with A = x0, B = v0/ω
pub fn harmonic_oscillator_solution(x0: f64, v0: f64, m: f64, k: f64, t: f64) -> (f64, f64) {
    let omega = (k / m).sqrt();
    let x = x0 * (omega * t).cos() + (v0 / omega) * (omega * t).sin();
    let v = -x0 * omega * (omega * t).sin() + v0 * (omega * t).cos();
    (x, v)
}
