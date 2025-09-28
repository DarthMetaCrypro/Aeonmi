// src/core/titan/math/energy.rs
// Energy set: (1) Einstein field residual, (2) Energy–momentum relation,
// (3) Hamiltonian from L, (4) Helmholtz F, (5) Gibbs G.

/// Small tensor alias for 4D spacetime (μ,ν = 0..3)
pub type T4 = [[f64; 4]; 4];

/// 1) Einstein field equation residual
///   G_{μν} + Λ g_{μν} - (8πG/c^4) T_{μν}  ==  0
/// You provide: metric g, Ricci tensor R_{μν}, Ricci scalar R, Λ, T_{μν}, G, c.
/// We return the 4x4 residual so you can check/solve numerically.
pub fn einstein_field_residual(
    g: &T4,         // metric g_{μν}
    ricci: &T4,     // R_{μν}
    r_scalar: f64,  // R
    lambda: f64,    // Λ
    t: &T4,         // T_{μν}
    big_g: f64,     // Newton G
    c: f64,         // speed of light
) -> T4 {
    // Einstein tensor: G_{μν} = R_{μν} - 1/2 R g_{μν}
    let mut einstein = [[0.0; 4]; 4];
    for mu in 0..4 {
        for nu in 0..4 {
            einstein[mu][nu] = ricci[mu][nu] - 0.5 * r_scalar * g[mu][nu];
        }
    }
    // κ = 8πG / c^4
    let kappa = 8.0 * std::f64::consts::PI * big_g / c.powi(4);

    // residual = G_{μν} + Λ g_{μν} - κ T_{μν}
    let mut resid = [[0.0; 4]; 4];
    for mu in 0..4 {
        for nu in 0..4 {
            resid[mu][nu] = einstein[mu][nu] + lambda * g[mu][nu] - kappa * t[mu][nu];
        }
    }
    resid
}

/// 2) Energy–momentum relation
/// E^2 = (pc)^2 + (mc^2)^2  -> returns E (≥0)
pub fn energy_from_p_m(p: f64, m: f64, c: f64) -> f64 {
    let a = (p * c).powi(2);
    let b = (m * c * c).powi(2);
    (a + b).sqrt()
}

/// 3) Hamiltonian from Lagrangian (first-order form)
/// H = Σ_i p_i \dot{q}_i - L
/// You pass p (generalized momenta), qdot (generalized velocities), and L (scalar).
pub fn hamiltonian(p: &[f64], qdot: &[f64], lagrangian: f64) -> f64 {
    assert_eq!(p.len(), qdot.len(), "p and qdot must have same length");
    let sum: f64 = p.iter().zip(qdot.iter()).map(|(pi, qi)| pi * qi).sum();
    sum - lagrangian
}

/// 4) Helmholtz free energy
/// F = U - T S
pub fn helmholtz_free_energy(internal_energy_u: f64, temperature_t: f64, entropy_s: f64) -> f64 {
    internal_energy_u - temperature_t * entropy_s
}

/// 5) Gibbs free energy
/// G = H - T S
pub fn gibbs_free_energy(enthalpy_h: f64, temperature_t: f64, entropy_s: f64) -> f64 {
    enthalpy_h - temperature_t * entropy_s
}
