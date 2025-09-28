// Binary entropy H2(p)
fn h2(p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 { 0.0 } else { -p*p.log2() - (1.0-p)*(1.0-p).log2() }
}

/// Sifted key fraction for BB84 (random bases) ~ 1/2
pub fn bb84_sift_fraction() -> f64 { 0.5 }

/// Approx. secure key rate per detected bit (asymptotic):
/// R_secure ≈ sift * [1 - 2 H2(QBER)]
pub fn bb84_secure_rate(qber: f64) -> f64 {
    let sift = bb84_sift_fraction();
    let term = 1.0 - 2.0 * h2(qber.clamp(0.0, 0.5));
    (sift * term).max(0.0)
}

/// Very rough detection probability model:
/// p_det ≈ 1 - exp(-η_sys * μ) + p_dark
pub fn detection_prob(eta_sys: f64, mean_photons: f64, dark: f64) -> f64 {
    (1.0 - (-eta_sys * mean_photons).exp() + dark).clamp(0.0, 1.0)
}

/// End-to-end **throughput** estimate:
/// throughput ≈ p_det * bb84_secure_rate(qber) * clock_hz
pub fn bb84_throughput_hz(clock_hz: f64, eta_sys: f64, mu: f64, dark: f64, qber: f64) -> f64 {
    let p = detection_prob(eta_sys, mu, dark);
    p * bb84_secure_rate(qber) * clock_hz
}
