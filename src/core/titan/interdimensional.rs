// src/core/titan/math/interdimensional.rs
// (append near your existing morris_thorne_metric)

pub type T4 = [[f64; 4]; 4];

/// Morris–Thorne wormhole metric with Φ(r)=0:
/// ds^2 = -c^2 dt^2 + dr^2/(1 - b(r)/r) + r^2(dθ^2 + sin^2θ dφ^2)
pub fn morris_thorne_phi0<Fb>(r: f64, theta: f64, b_fn: Fb, c: f64) -> T4
where
    Fb: Fn(f64) -> f64,
{
    let b = b_fn(r);
    let denom = 1.0 - b / r;
    assert!(denom != 0.0, "metric singular at b(r)=r");
    let mut g = [[0.0; 4]; 4];
    g[0][0] = -(c * c);
    g[1][1] = 1.0 / denom;
    g[2][2] = r * r;
    g[3][3] = r * r * theta.sin().powi(2);
    g
}

/// Quick throat/flare-out check at r0:
/// throat: b(r0) ≈ r0 ; flare-out: b'(r0) < 1
pub fn flare_out_ok<Fb>(r0: f64, b_fn: Fb, h: f64) -> (bool, f64, f64)
where
    Fb: Fn(f64) -> f64,
{
    let b0 = b_fn(r0);
    let db = (b_fn(r0 + h) - b_fn(r0 - h)) / (2.0 * h); // central diff
    let throat_ok = (b0 - r0).abs() <= 1e-9 * (1.0 + r0.abs());
    let flare_ok = db < 1.0;
    (throat_ok && flare_ok, b0, db)
}

/// Proper radial distance ℓ(r1→r2) = ∫ dr / sqrt(1 - b(r)/r)
pub fn proper_radial_distance<Fb>(r1: f64, r2: f64, steps: usize, b_fn: Fb) -> Option<f64>
where
    Fb: Fn(f64) -> f64,
{
    if steps < 2 { return None; }
    let (a, b) = if r1 <= r2 { (r1, r2) } else { (r2, r1) };
    let h = (b - a) / (steps as f64 - 1.0);
    let mut acc = 0.0;
    for i in 0..steps {
        let r = a + h * i as f64;
        let denom = 1.0 - b_fn(r) / r;
        if denom <= 0.0 { return None; }
        let f = 1.0 / denom.sqrt();
        acc += if i == 0 || i == steps - 1 { 0.5 * f } else { f };
    }
    Some(acc * h)
}

/// Equatorial embedding profile z(r) (t=const, θ=π/2):
/// dz/dr = ± sqrt( b(r) / (r - b(r)) )
pub fn embedding_z_profile<Fb>(r_samples: &[f64], b_fn: Fb) -> Option<Vec<f64>>
where
    Fb: Fn(f64) -> f64,
{
    if r_samples.len() < 2 { return None; }
    let mut z = vec![0.0; r_samples.len()];
    for i in 1..r_samples.len() {
        let r0 = r_samples[i - 1];
        let r1 = r_samples[i];
        let h = r1 - r0;
        let deriv = |r: f64| -> Option<f64> {
            let b = b_fn(r);
            let denom = r - b;
            if denom <= 0.0 || b < 0.0 { return None; }
            Some((b / denom).sqrt())
        };
        let d0 = deriv(r0)?;
        let d1 = deriv(r1)?;
        z[i] = z[i - 1] + 0.5 * (d0 + d1) * h; // trapezoid
    }
    Some(z)
}

/// Canonical shape function often used in examples: b(r) = r0^2 / r
pub fn shape_canonical(r: f64, r0: f64) -> f64 {
    (r0 * r0) / r
}
