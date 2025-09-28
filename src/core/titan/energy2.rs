// --- 6) Lagrangian density (scalar field) ---------------------------
// 𝓛 = 1/2 (∂_μ φ) (∂^μ φ) − V(φ)
// Here we take: dphi_cov = (∂_μ φ) with lowered index μ,
// and g_contra = g^{μν} (inverse metric) to raise it.
pub fn lagrangian_density_scalar(
    dphi_cov: &[f64; 4], // ∂_μ φ
    g_contra: &T4,       // g^{μν}
    potential_v: f64,    // V(φ)
) -> f64 {
    let mut kinetic = 0.0;
    for mu in 0..4 {
        for nu in 0..4 {
            kinetic += dphi_cov[mu] * g_contra[mu][nu] * dphi_cov[nu];
        }
    }
    0.5 * kinetic - potential_v
}

// --- 7) Navier–Stokes energy transport (RHS) -----------------------
// ρ De/Dt = -p ∇·v + Φ + ∇·(k ∇T)
// Returns De/Dt given fields; you can step with e_next = e + dt * rhs.
pub fn energy_transport_rhs(
    rho: f64,           // mass density ρ
    pressure: f64,      // p
    div_v: f64,         // ∇·v
    viscous_phi: f64,   // Φ (viscous dissipation)
    div_k_grad_t: f64,  // ∇·(k ∇T)  (heat conduction term)
) -> f64 {
    (-pressure * div_v + viscous_phi + div_k_grad_t) / rho
}
pub fn energy_update_euler(e: f64, rhs: f64, dt: f64) -> f64 { e + dt * rhs }

// --- 8) Virial theorem helpers -------------------------------------
// For homogeneous V of degree n: 2⟨T⟩ = n⟨V⟩  ⇒ residual = 2T - nV (→0 when satisfied).
pub fn virial_residual_degree_n(t_avg: f64, v_avg: f64, n: f64) -> f64 {
    2.0 * t_avg - n * v_avg
}
// Instantaneous scalar virial: G = Σ_i r_i · F_i  (handy for time-averaging)
pub fn instantaneous_scalar_virial(r: &[Vec<f64>], f: &[Vec<f64>]) -> f64 {
    assert_eq!(r.len(), f.len());
    let mut sum = 0.0;
    for (ri, fi) in r.iter().zip(f.iter()) {
        assert_eq!(ri.len(), fi.len());
        sum += ri.iter().zip(fi.iter()).map(|(a, b)| a * b).sum::<f64>();
    }
    sum
}

// --- 9) Planck’s radiation law -------------------------------------
// Spectral radiance per frequency:  I(ν,T) = (2 h ν^3 / c^2) * 1/(exp(hν/kT) - 1)
pub fn planck_intensity_nu(nu: f64, t: f64, h: f64, c: f64, k_b: f64) -> f64 {
    let x = (h * nu) / (k_b * t).max(1e-300);
    let denom = (x.exp() - 1.0).max(1e-300);
    (2.0 * h * nu.powi(3) / (c * c)) / denom
}
// Spectral radiance per wavelength:  I(λ,T) = (2 h c^2 / λ^5) * 1/(exp(hc/(λkT)) - 1)
pub fn planck_intensity_lambda(lambda: f64, t: f64, h: f64, c: f64, k_b: f64) -> f64 {
    let x = (h * c) / ((lambda * k_b * t).max(1e-300));
    let denom = (x.exp() - 1.0).max(1e-300);
    (2.0 * h * c * c) / (lambda.powi(5)) / denom
}

// --- 10) Joule–Thomson coefficient μ_JT ----------------------------
// μ_JT = (∂T/∂P)_H  ≈ (1/C_p) [ T (∂V/∂T)_P − V ]
pub fn joule_thomson_mu_from_dvdt(t: f64, v: f64, dv_dT_at_p: f64, c_p: f64) -> f64 {
    (t * dv_dT_at_p - v) / c_p
}
// Equivalent using thermal expansion α = (1/V)(∂V/∂T)_P:  μ_JT = (V/C_p) [T α − 1]
pub fn joule_thomson_mu_from_alpha(t: f64, v: f64, alpha: f64, c_p: f64) -> f64 {
    v * (t * alpha - 1.0) / c_p
}
