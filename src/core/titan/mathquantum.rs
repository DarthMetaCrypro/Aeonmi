// src/core/titan/math/quantum.rs
// 11) Schrödinger 1D (Crank–Nicolson step)
// 12) Dirac 1D (2-spinor, RK4 step)
// 13) Klein–Gordon 1D (leapfrog step)
// 14) Uncertainty Δx, Δp from ψ(x)
// 15) Free-particle path-integral propagator K_free

use num_complex::Complex64 as C;
use std::f64::consts::PI;

#[inline] fn i() -> C { C::new(0.0, 1.0) }

/// ----- 11) Schrödinger: one CN step on a uniform grid (Dirichlet ends) -----
/// iħ ∂ψ/∂t = [-(ħ²/2m) ∂²/∂x² + V(x)] ψ
/// psi: in/out wavefunction length N; v: potential; dx, dt, ħ, m
pub fn schrodinger_cn_step_1d(psi: &mut [C], v: &[f64], dx: f64, dt: f64, hbar: f64, m: f64) {
    let n = psi.len();
    assert_eq!(v.len(), n);
    assert!(n >= 3, "need at least 3 points");
    let inv_dx2 = 1.0 / (dx * dx);
    let kin = - (hbar*hbar) / (2.0*m);      // coefficient of Laplacian in H
    // Build A, B: (I + i dt/2ħ H) ψ^{n+1} = (I - i dt/2ħ H) ψ^n
    let s = i() * (dt / (2.0 * hbar));
    let h_off = kin * (-1.0 * inv_dx2);     // off-diagonal element of H contribution
    let h_diag_base = kin * (2.0 * inv_dx2);// diagonal kinetic part; + V_i later

    let a_off = s * h_off;
    let b_off = -a_off;
    let mut a_diag = vec![C::new(1.0, 0.0); n];
    let mut b_diag = vec![C::new(1.0, 0.0); n];

    for i in 0..n {
        let h_ii = h_diag_base + v[i];
        a_diag[i] += s * C::new(h_ii, 0.0);
        b_diag[i] -= s * C::new(h_ii, 0.0);
    }
    // Dirichlet at boundaries
    a_diag[0] = C::new(1.0, 0.0);
    a_diag[n-1] = C::new(1.0, 0.0);

    // RHS = B ψ
    let mut rhs = vec![C::new(0.0, 0.0); n];
    rhs[0] = C::new(0.0, 0.0);
    for i in 1..n-1 {
        rhs[i] = b_diag[i]*psi[i] + b_off*(psi[i-1] + psi[i+1]);
    }
    rhs[n-1] = C::new(0.0, 0.0);

    // Solve A ψ^{n+1} = RHS (tridiagonal with constant off-diagonals)
    let mut next = vec![C::new(0.0, 0.0); n];
    thomas_constant_offdiag(&a_diag, a_off, a_off, &rhs, &mut next);
    // Enforce boundaries and overwrite
    next[0] = C::new(0.0, 0.0);
    next[n-1] = C::new(0.0, 0.0);
    psi.copy_from_slice(&next);
}

/// Tridiagonal Thomas solver with constant off-diagonals.
fn thomas_constant_offdiag(
    a_diag: &[C], a_sub: C, a_sup: C, rhs: &[C], out: &mut [C]
){
    let n = a_diag.len();
    let mut cprime = vec![C::new(0.0,0.0); n];
    let mut dprime = vec![C::new(0.0,0.0); n];

    let mut denom = a_diag[0];
    cprime[0] = a_sup / denom;
    dprime[0] = rhs[0] / denom;

    for i in 1..n {
        denom = a_diag[i] - a_sub * cprime[i-1];
        cprime[i] = a_sup / denom;
        dprime[i] = (rhs[i] - a_sub * dprime[i-1]) / denom;
    }
    out[n-1] = dprime[n-1];
    for i in (0..n-1).rev() {
        out[i] = dprime[i] - cprime[i]*out[i+1];
    }
}

/// ----- 12) Dirac 1D (ħ=c=1 units by default): RK4 time step -----
/// ∂ψ/∂t = -i H ψ,  H = -i c α ∂/∂x + β m c^2 + V(x)
/// Represent α = σ_x, β = σ_z; ψ = (ψ_up, ψ_dn).
/// Arrays must have equal length N; central differences; Dirichlet ends.
pub fn dirac_1d_rk4_step(
    psi_up: &mut [C],
    psi_dn: &mut [C],
    v: &[f64],
    m: f64,
    c: f64,
    hbar: f64,
    dx: f64,
    dt: f64,
) {
    let n = psi_up.len();
    assert_eq!(psi_dn.len(), n);
    assert_eq!(v.len(), n);
    assert!(n >= 3);

    // right-hand side: dψ/dt = -(i/ħ) H ψ
    let rhs = |up:&[C], dn:&[C], out_up:&mut [C], out_dn:&mut [C]| {
        // spatial derivative ∂/∂x (central)
        let fac = c / (2.0*dx); // appears with α term
        for i in 0..n {
            out_up[i] = C::new(0.0,0.0);
            out_dn[i] = C::new(0.0,0.0);
        }
        for i in 1..n-1 {
            // α σ_x mixes components with derivative
            let d_up = (up[i+1] - up[i-1]) * C::new(1.0,0.0) * (0.5/dx);
            let d_dn = (dn[i+1] - dn[i-1]) * C::new(1.0,0.0) * (0.5/dx);
            // H ψ components:
            // H_up = -i c * (∂/∂x of dn) + m c^2 * up + V up
            // H_dn = -i c * (∂/∂x of up) - m c^2 * dn + V dn
            let h_up = -i()*C::new(c,0.0)*d_dn + C::new(m*c*c,0.0)*up[i] + C::new(v[i],0.0)*up[i];
            let h_dn = -i()*C::new(c,0.0)*d_up - C::new(m*c*c,0.0)*dn[i] + C::new(v[i],0.0)*dn[i];
            // dψ/dt = -(i/ħ) H ψ
            let pref = -i()/C::new(hbar,0.0);
            out_up[i] = pref * h_up;
            out_dn[i] = pref * h_dn;
        }
        // hard Dirichlet
        out_up[0]=C::new(0.0,0.0); out_up[n-1]=C::new(0.0,0.0);
        out_dn[0]=C::new(0.0,0.0); out_dn[n-1]=C::new(0.0,0.0);
    };

    // RK4
    let mut k1u=vec![C::new(0.0,0.0);n]; let mut k1d=k1u.clone();
    let mut k2u=k1u.clone(); let mut k2d=k1u.clone();
    let mut k3u=k1u.clone(); let mut k3d=k1u.clone();
    let mut k4u=k1u.clone(); let mut k4d=k1u.clone();

    rhs(psi_up, psi_dn, &mut k1u, &mut k1d);

    let mut tmpu=psi_up.to_vec(); let mut tmpd=psi_dn.to_vec();
    for i in 0..n { tmpu[i]=psi_up[i]+k1u[i]*(dt/2.0); tmpd[i]=psi_dn[i]+k1d[i]*(dt/2.0); }
    rhs(&tmpu,&tmpd,&mut k2u,&mut k2d);

    for i in 0..n { tmpu[i]=psi_up[i]+k2u[i]*(dt/2.0); tmpd[i]=psi_dn[i]+k2d[i]*(dt/2.0); }
    rhs(&tmpu,&tmpd,&mut k3u,&mut k3d);

    for i in 0..n { tmpu[i]=psi_up[i]+k3u[i]*dt; tmpd[i]=psi_dn[i]+k3d[i]*dt; }
    rhs(&tmpu,&tmpd,&mut k4u,&mut k4d);

    for i in 0..n {
        psi_up[i] += (k1u[i] + 2.0*k2u[i] + 2.0*k3u[i] + k4u[i]) * (dt/6.0);
        psi_dn[i] += (k1d[i] + 2.0*k2d[i] + 2.0*k3d[i] + k4d[i]) * (dt/6.0);
    }
    // boundaries
    psi_up[0]=C::new(0.0,0.0); psi_up[n-1]=C::new(0.0,0.0);
    psi_dn[0]=C::new(0.0,0.0); psi_dn[n-1]=C::new(0.0,0.0);
}

/// ----- 13) Klein–Gordon 1D: leapfrog (φ, π) -----
/// KG: ∂²_t φ - c² ∂²_x φ + (m c²/ħ)² φ = 0
/// Introduce π = ∂φ/∂t. One step: π(t+dt/2) then φ(t+dt) then π(t+dt).
pub fn klein_gordon_leapfrog_step(
    phi: &mut [f64],   // field φ(x)
    pi_t: &mut [f64],  // conjugate momentum π(x) at time t
    dx: f64,
    dt: f64,
    m: f64,
    c: f64,
    hbar: f64,
){
    let n = phi.len();
    assert_eq!(pi_t.len(), n);
    assert!(n>=3);
    let mu2 = (m*c*c/hbar).powi(2);
    // compute laplacian φ_xx
    let mut lap = vec![0.0; n];
    for i in 1..n-1 {
        lap[i] = (phi[i-1] - 2.0*phi[i] + phi[i+1]) / (dx*dx);
    }
    // half step for π
    for i in 1..n-1 {
        pi_t[i] += dt*0.5 * ( c*c*lap[i] - mu2*phi[i] );
    }
    // full step for φ
    for i in 0..n {
        phi[i] += dt * pi_t[i];
    }
    // recompute laplacian at new φ
    for i in 1..n-1 {
        lap[i] = (phi[i-1] - 2.0*phi[i] + phi[i+1]) / (dx*dx);
    }
    // second half-step for π
    for i in 1..n-1 {
        pi_t[i] += dt*0.5 * ( c*c*lap[i] - mu2*phi[i] );
    }
    // Dirichlet boundaries
    phi[0]=0.0; phi[n-1]=0.0; pi_t[0]=0.0; pi_t[n-1]=0.0;
}

/// ----- 14) Uncertainty: Δx and Δp from ψ(x) -----
/// Δx = sqrt(<x^2>-<x>^2),  Δp = sqrt(<p^2>-<p>^2), with p̂ = -iħ ∂/∂x
/// Uses central differences for ∂ψ/∂x to avoid FFT.
pub fn uncertainty_dx_dp(psi: &[C], x: &[f64], dx: f64, hbar: f64) -> (f64, f64) {
    let n = psi.len();
    assert_eq!(x.len(), n);
    assert!(n>=3);
    // normalize (in case)
    let norm: f64 = psi.iter().map(|c| c.norm_sqr()).sum::<f64>() * dx;
    let inv = 1.0 / norm.max(1e-300);
    let prob: Vec<f64> = psi.iter().map(|c| c.norm_sqr()*inv).collect();

    let ex = sum_prod(&prob, x, dx);
    let ex2 = sum_prod(&prob, &x.iter().map(|xi| xi*xi).collect::<Vec<_>>(), dx);
    let dx_std = (ex2 - ex*ex).max(0.0).sqrt();

    // p expectation via derivative
    let mut dpsi = vec![C::new(0.0,0.0); n];
    for i in 1..n-1 {
        dpsi[i] = (psi[i+1] - psi[i-1]) * C::new(1.0,0.0) * (0.5/dx);
    }
    let p_oppsi: Vec<C> = dpsi.iter().map(|d| -i()*C::new(hbar,0.0)*(*d)).collect();

    // <p> = ∫ ψ* (p̂ ψ) dx
    let mut ep = 0.0;
    for i in 0..n { ep += (psi[i].conj()*p_oppsi[i]).re * dx; }
    // <p^2> = ∫ |p̂ ψ|^2 dx
    let mut ep2 = 0.0;
    for i in 0..n { ep2 += p_oppsi[i].norm_sqr() * dx; }
    let dp_std = (ep2 - ep*ep).max(0.0).sqrt();
    (dx_std, dp_std)
}

fn sum_prod(a: &[f64], b: &[f64], dx: f64) -> f64 {
    a.iter().zip(b).map(|(ai,bi)| ai*bi).sum::<f64>() * dx
}

/// ----- 15) Free-particle propagator (path-integral kernel) -----
/// K(x_f, x_i; t) = sqrt( m / (2π i ħ t) ) * exp( i m (Δx)^2 / (2 ħ t) )
pub fn k_free(xf: f64, xi: f64, t: f64, m: f64, hbar: f64) -> C {
    assert!(t != 0.0, "t must be non-zero");
    let dx = xf - xi;
    let pref = (m / (2.0*PI*hbar*(t.abs()))).sqrt(); // magnitude; handle sign via phase
    let phase = C::new(0.0, m*dx*dx.signum()*dx.abs() / (2.0*hbar*t));
    // include 1/sqrt(i) factor for sign(t); sqrt(1/i) = e^{-iπ/4}
    let root_i = C::from_polar(1.0, -std::f64::consts::FRAC_PI_4 * t.signum());
    root_i * C::new(pref, 0.0) * phase.exp()
}
