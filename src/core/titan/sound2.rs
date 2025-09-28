// --- 36) Doppler shift (acoustics) -------------------------------------------
// Sign convention (standard acoustics):
//   v_source > 0  → source moving TOWARD observer
//   v_observer > 0 → observer moving TOWARD source
// f' = f * (c + v_observer) / (c - v_source)
pub fn doppler_shift(f: f64, c: f64, v_source: f64, v_observer: f64) -> f64 {
    let denom = (c - v_source).max(1e-12);
    f * (c + v_observer) / denom
}

// Convenience: stationary source or observer
pub fn doppler_observer_moving(f: f64, c: f64, v_observer: f64) -> f64 {
    f * (c + v_observer) / c
}
pub fn doppler_source_moving(f: f64, c: f64, v_source: f64) -> f64 {
    let denom = (c - v_source).max(1e-12);
    f * c / denom
}

// --- 37) Standing Wave Ratio (SWR) -------------------------------------------
// SWR = (1 + |Γ|) / (1 - |Γ|), with |Γ| clamped to [0, 1 - ε]
pub fn swr_from_gamma_mag(gamma_mag: f64) -> f64 {
    let g = gamma_mag.clamp(0.0, 1.0 - 1e-12);
    (1.0 + g) / (1.0 - g)
}

// --- 38) Acoustic impedance ---------------------------------------------------
// Characteristic (plane-wave) impedance: Z0 = ρ c
pub fn acoustic_impedance_char(rho: f64, c: f64) -> f64 {
    rho * c
}
// Local (point) acoustic impedance: Z = p / u  (pressure over particle velocity)
pub fn acoustic_impedance_local(pressure: f64, particle_velocity: f64) -> f64 {
    pressure / particle_velocity.max(1e-12)
}

// --- 39) Vibrational partition function (quantum harmonic oscillator) --------
// Z_v = 1 / (1 - exp(-ħ ω / k_B T))
pub fn vibrational_partition_qho(omega: f64, temperature: f64, hbar: f64, k_b: f64) -> f64 {
    let beta = 1.0 / (k_b * temperature.max(1e-12));
    let x = (-hbar * omega * beta).exp();
    1.0 / (1.0 - x.max(1e-300))
}
// Optional thermodynamic helpers: <E> and C_v for a single mode
pub fn vibrational_mean_energy(omega: f64, temperature: f64, hbar: f64, k_b: f64) -> f64 {
    let beta = 1.0 / (k_b * temperature.max(1e-12));
    let x = (hbar * omega * beta).exp();
    // <n> = 1/(x-1);  <E> = ħω(<n> + 1/2)
    hbar * omega * (1.0 / (x - 1.0).max(1e-300) + 0.5)
}
pub fn vibrational_heat_capacity_cv(omega: f64, temperature: f64, hbar: f64, k_b: f64) -> f64 {
    let theta = hbar * omega / k_b;
    let y = (theta / temperature.max(1e-12)).exp();
    let denom = (y - 1.0).powi(2).max(1e-300);
    k_b * (theta / temperature).powi(2) * y / denom
}

// --- 40) Phonon dispersion (1D) ----------------------------------------------
// (a) Cosine form used in some texts: ω(k) = ω0 * sqrt(1 - cos(k a))
pub fn phonon_dispersion_cos(k: f64, omega0: f64, a: f64) -> f64 {
    let val = (1.0 - (k * a).cos()).max(0.0);
    omega0 * val.sqrt()
}
// (b) Monatomic chain, nearest-neighbor: ω(k) = 2 sqrt(κ/m) |sin(k a / 2)|
pub fn phonon_dispersion_chain(k: f64, kappa: f64, mass: f64, a: f64) -> f64 {
    2.0 * (kappa / mass).sqrt() * ((k * a) * 0.5).sin().abs()
}
