use num_complex::Complex64 as C;
use std::f64::consts::PI;

#[inline] fn i() -> C { C::new(0.0, 1.0) }

/* ===================== 16) QHO energies & eigenfunctions ===================== */

/// E_n = ħ ω (n + 1/2)
pub fn qho_energy(n: usize, omega: f64, hbar: f64) -> f64 {
    hbar * omega * (n as f64 + 0.5)
}

/// Physicist’s Hermite H_n(ξ) via recurrence:
/// H_0=1, H_1=2ξ, H_{n+1}=2ξ H_n - 2n H_{n-1}
fn hermite_phys(n: usize, xi: f64) -> f64 {
    match n {
        0 => 1.0,
        1 => 2.0 * xi,
        _ => {
            let mut h0 = 1.0;
            let mut h1 = 2.0 * xi;
            for k in 1..n {
                let h2 = 2.0 * xi * h1 - 2.0 * (k as f64) * h0;
                h0 = h1;
                h1 = h2;
            }
            h1
        }
    }
}

/// log(n!) using Stirling for stability
fn ln_factorial(n: usize) -> f64 {
    if n < 2 { 0.0 } else {
        let n = n as f64;
        n * n.ln() - n + 0.5 * (2.0 * PI * n).ln()
    }
}

/// ψ_n(x) = (mω/πħ)^{1/4} * 1/√(2^n n!) * H_n(ξ) * e^{-ξ^2/2},  ξ = √(mω/ħ) x
pub fn qho_eigenfunction(n: usize, x: f64, m: f64, omega: f64, hbar: f64) -> f64 {
    let xi = (m * omega / hbar).sqrt() * x;
    let h = hermite_phys(n, xi);
    let pref = (m * omega / (PI * hbar)).powf(0.25);
    let ln_norm = -0.5 * ( (2u64.pow(n as u32) as f64).ln() + ln_factorial(n) );
    let norm = ln_norm.exp();
    pref * norm * h * (-0.5 * xi * xi).exp()
}

/* ================== 17) Density matrix evolution (von Neumann) ================= */

/// ρ̇ = -(i/ħ)[H, ρ]; one RK4 step (unitary, small dt recommended).
pub fn rho_unitary_step_rk4(
    rho: &mut [Vec<C>],
    h: &[Vec<C>],
    dt: f64,
    hbar: f64,
) {
    let n = rho.len();
    assert!(n > 0 && rho.iter().all(|r| r.len() == n));
    assert!(h.len() == n && h.iter().all(|r| r.len() == n));

    let k1 = rho_deriv(rho, h, hbar);
    let r2 = mat_add(rho, &mat_scale(&k1, dt / 2.0));
    let k2 = rho_deriv(&r2, h, hbar);
    let r3 = mat_add(rho, &mat_scale(&k2, dt / 2.0));
    let k3 = rho_deriv(&r3, h, hbar);
    let r4 = mat_add(rho, &mat_scale(&k3, dt));
    let k4 = rho_deriv(&r4, h, hbar);

    let incr = mat_add_many(&[
        &k1,
        &mat_scale(&k2, 2.0),
        &mat_scale(&k3, 2.0),
        &k4
    ]);
    let incr = mat_scale(&incr, dt / 6.0);

    for i in 0..n { for j in 0..n { rho[i][j] += incr[i][j]; } }
    // re-hermitize lightly to counter fp drift
    for i in 0..n { for j in 0..n {
        let avg = 0.5*(rho[i][j] + rho[j][i].conj());
        rho[i][j] = avg; rho[j][i] = avg.conj();
    }}
    // renormalize trace to 1
    let mut tr = C::new(0.0,0.0); for k in 0..n { tr += rho[k][k]; }
    if tr.re != 0.0 || tr.im != 0.0 {
        let s = 1.0 / tr.re.max(1e-18);
        for i in 0..n { rho[i][i] *= s; }
    }
}

fn rho_deriv(rho: &[Vec<C>], h: &[Vec<C>], hbar: f64) -> Vec<Vec<C>> {
    let comm = commutator(h, rho);
    mat_scale(&comm, -1.0) * (i() / C::new(hbar, 0.0))
}

fn commutator(a: &[Vec<C>], b: &[Vec<C>]) -> Vec<Vec<C>> {
    let ab = mat_mul(a, b);
    let ba = mat_mul(b, a);
    let n = ab.len(); let m = ab[0].len();
    let mut out = vec![vec![C::new(0.0,0.0); m]; n];
    for i in 0..n { for j in 0..m { out[i][j] = ab[i][j] - ba[i][j]; } }
    out
}

fn mat_mul(a: &[Vec<C>], b: &[Vec<C>]) -> Vec<Vec<C>> {
    let n = a.len(); let m = b[0].len(); let kdim = a[0].len();
    assert!(b.len()==kdim);
    let mut out = vec![vec![C::new(0.0,0.0); m]; n];
    for i in 0..n {
        for k in 0..kdim {
            let aik = a[i][k];
            for j in 0..m { out[i][j] += aik * b[k][j]; }
        }
    }
    out
}
fn mat_add(a: &[Vec<C>], b: &[Vec<C>]) -> Vec<Vec<C>> {
    let n = a.len(); let m = a[0].len();
    let mut out = vec![vec![C::new(0.0,0.0); m]; n];
    for i in 0..n { for j in 0..m { out[i][j] = a[i][j] + b[i][j]; } }
    out
}
fn mat_add_many(ms: &[&[Vec<C>]]) -> Vec<Vec<C>> {
    let n = ms[0].len(); let m = ms[0][0].len();
    let mut out = vec![vec![C::new(0.0,0.0); m]; n];
    for mat in ms {
        for i in 0..n { for j in 0..m { out[i][j] += mat[i][j]; } }
    }
    out
}
fn mat_scale(a: &[Vec<C>], s: f64) -> Vec<Vec<C>> {
    let n = a.len(); let m = a[0].len();
    let mut out = vec![vec![C::new(0.0,0.0); m]; n];
    for i in 0..n { for j in 0..m { out[i][j] = a[i][j] * C::new(s,0.0); } }
    out
}

/* ========================== 18) Bloch helpers (1D) =========================== */

/// Tight-binding (1D) dispersion: E(k) = ε - 2 t cos(k a)
pub fn bloch_dispersion_1d(k: f64, epsilon: f64, t: f64, a: f64) -> f64 {
    epsilon - 2.0 * t * (k * a).cos()
}

/// Build Bloch wave ψ_k(x_i) = e^{i k x_i} u(x_i) for a single cell-sampled u(x)
pub fn bloch_wave_1d(k: f64, x: &[f64], u: &[C]) -> Vec<C> {
    assert_eq!(x.len(), u.len());
    x.iter().zip(u.iter())
        .map(|(xi, ui)| (i()*C::new(k*xi,0.0)).exp() * *ui)
        .collect()
}

/* ======================= 19) WKB tunneling probability ======================= */

/// T ≈ exp(-2 ∫_a^b κ(x) dx), κ(x)=√(2m(V(x)-E))/ħ over classically forbidden region V>E.
/// `x` and `v` are aligned; simple trapezoid rule. Returns 1.0 if no barrier.
pub fn wkb_tunneling_prob(x: &[f64], v: &[f64], e: f64, m: f64, hbar: f64) -> f64 {
    assert_eq!(x.len(), v.len());
    if x.len() < 2 { return 1.0; }
    let mut integral = 0.0;
    for i in 0..x.len()-1 {
        let (x0, x1) = (x[i], x[i+1]);
        let (v0, v1) = (v[i], v[i+1]);
        let kappa = |vv: f64| -> f64 {
            if vv > e { ((2.0*m*(vv - e)).max(0.0)).sqrt() / hbar } else { 0.0 }
        };
        let f0 = kappa(v0);
        let f1 = kappa(v1);
        integral += 0.5 * (f0 + f1) * (x1 - x0);
    }
    (-2.0 * integral).exp()
}

/* ============================ 20) CHSH (Bell) ================================ */

/// CHSH S = E(a,b) + E(a,b') + E(a',b) - E(a',b')
pub fn chsh_s(e_ab: f64, e_abp: f64, e_apb: f64, e_apbp: f64) -> f64 {
    e_ab + e_abp + e_apb - e_apbp
}

/// Estimate correlation E = ⟨A B⟩ from ±1 outcomes
pub fn correlation_pm1(a: &[i8], b: &[i8]) -> f64 {
    assert_eq!(a.len(), b.len());
    let n = a.len().max(1) as f64;
    let mut s = 0.0;
    for (&ai, &bi) in a.iter().zip(b.iter()) {
        s += (ai as f64) * (bi as f64);
    }
    s / n
}
