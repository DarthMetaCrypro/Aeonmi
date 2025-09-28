// --- 46) Meta-learning gradient update (MAML-style inner step) ---
// θ' = θ - α ∇_θ L(f_θ)
// Generic vector update; plug any grad you computed upstream.
pub fn meta_inner_update(theta: &[f64], grad: &[f64], alpha: f64) -> Vec<f64> {
    assert_eq!(theta.len(), grad.len());
    theta.iter().zip(grad).map(|(t, g)| t - alpha * g).collect()
}

// --- 47) Swarm entropy balancing ---
// S = -Σ p_i log(p_i), returned as a regularizer term you can subtract/add.
pub fn swarm_entropy(weights: &[f64]) -> f64 {
    let sum: f64 = weights.iter().sum();
    if sum <= 0.0 { return 0.0; }
    let mut s = 0.0;
    for &w in weights {
        let p = (w / sum).clamp(1e-12, 1.0);
        s -= p * p.ln();
    }
    s
}

// --- 48) Algorithmic complexity estimator (proxy) ---
// Kolmogorov K(x) is uncomputable; we approximate via Shannon bits:
//  K_hat ≈ n * H_hat  where H_hat = -Σ p(b) log2 p(b) over byte histogram.
pub fn complexity_bits(data: &[u8]) -> f64 {
    if data.is_empty() { return 0.0; }
    let mut hist = [0usize; 256];
    for &b in data { hist[b as usize] += 1; }
    let n = data.len() as f64;
    let mut h = 0.0;
    for &c in &hist {
        if c > 0 {
            let p = c as f64 / n;
            h -= p * p.log2();
        }
    }
    n * h // bits
}

// --- 49) Stochastic resonance helper ---
// y = f(x) + η ; scan noise_std list, pick the one maximizing SNR at a target freq.
// `signal`: samples; `f_nl`: optional nonlinearity; `freq_bin`: index in FFT to monitor.
use num_complex::Complex64;
pub fn stochastic_resonance_scan<F>(
    signal: &[f64],
    f_nl: F,
    noise_stds: &[f64],
    freq_bin: usize,
) -> (f64, Vec<f64>) // (best_std, best_output)
where
    F: Fn(f64) -> f64
{
    assert!(!signal.is_empty());
    let n = signal.len();
    let n_c = n.next_power_of_two();
    let mut best_std = 0.0;
    let mut best_snr = f64::NEG_INFINITY;
    let mut best_out = vec![];

    for &std in noise_stds {
        // build y = f(x) + gaussian noise
        let mut y: Vec<f64> = signal.iter().copied().map(|x| f_nl(x)).collect();
        // simple LCG rng (deterministic, not cryptographic)
        let mut state: u64 = 0x9E3779B97F4A7C15;
        for v in y.iter_mut() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let u = ((state >> 11) as f64) / ((1u64 << 53) as f64); // ~U(0,1)
            // Box–Muller
            state = state.wrapping_mul(2862933555777941757).wrapping_add(3037000493);
            let u2 = ((state >> 11) as f64) / ((1u64 << 53) as f64);
            let r = (-2.0 * u.max(1e-12).ln()).sqrt();
            let phi = 2.0 * std::f64::consts::PI * u2;
            let gauss = r * phi.cos();
            *v += std * gauss;
        }

        // zero-pad & FFT magnitude
        let mut buf: Vec<Complex64> = y.iter().map(|&t| Complex64::new(t, 0.0)).collect();
        buf.resize(n_c, Complex64::new(0.0, 0.0));
        fft_inplace_radix2(&mut buf); // simple Cooley–Tukey below

        let mag: Vec<f64> = buf.iter().map(|c| c.norm()).collect();
        let sig = *mag.get(freq_bin).unwrap_or(&0.0);
        let noise_floor = (mag.iter().sum::<f64>() - sig).max(1e-12) / (mag.len().saturating_sub(1).max(1) as f64);
        let snr = sig / noise_floor;

        if snr > best_snr {
            best_snr = snr;
            best_std = std;
            best_out = y;
        }
    }
    (best_std, best_out)
}

// minimal in-place radix-2 FFT (real use: swap for a crate)
fn fft_inplace_radix2(a: &mut [Complex64]) {
    let n = a.len();
    assert!(n.is_power_of_two());
    // bit-reversal
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 { j ^= bit; bit >>= 1; }
        j ^= bit;
        if i < j { a.swap(i, j); }
    }
    // stages
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * std::f64::consts::PI / (len as f64);
        let wlen = Complex64::from_polar(1.0, ang);
        for i in (0..n).step_by(len) {
            let mut w = Complex64::new(1.0, 0.0);
            for j in 0..(len/2) {
                let u = a[i + j];
                let v = a[i + j + len/2] * w;
                a[i + j] = u + v;
                a[i + j + len/2] = u - v;
                w *= wlen;
            }
        }
        len <<= 1;
    }
}

// --- 50) Quantum tensor contraction (generic N-D) ---
// Contract A (shape_A) over axes_A with B (shape_B) over axes_B.
// Returns (shape_out, data_out). Tensors are row-major in `data`.
pub fn tensor_contract(
    data_a: &[f64], shape_a: &[usize], axes_a: &[usize],
    data_b: &[f64], shape_b: &[usize], axes_b: &[usize],
) -> (Vec<usize>, Vec<f64>) {
    assert_eq!(axes_a.len(), axes_b.len(), "mismatched contraction axes");
    // validate contracted dims equal
    for (&ia, &ib) in axes_a.iter().zip(axes_b.iter()) {
        assert_eq!(shape_a[ia], shape_b[ib], "dimension mismatch on contraction");
    }

    // build output shape
    let mut out_shape = vec![];
    let mut a_outer: Vec<usize> = (0..shape_a.len()).filter(|i| !axes_a.contains(i)).collect();
    let mut b_outer: Vec<usize> = (0..shape_b.len()).filter(|i| !axes_b.contains(i)).collect();
    for &i in &a_outer { out_shape.push(shape_a[i]); }
    for &i in &b_outer { out_shape.push(shape_b[i]); }

    // strides
    let stride = |shape: &[usize]| -> Vec<usize> {
        let mut s = vec![0; shape.len()];
        let mut acc = 1;
        for i in (0..shape.len()).rev() {
            s[i] = acc;
            acc *= shape[i];
        }
        s
    };
    let sa = stride(shape_a);
    let sb = stride(shape_b);

    // index maps for outer loops
    let prod = |dims: &[usize]| dims.iter().product::<usize>().max(1);
    let a_outer_dims: Vec<usize> = a_outer.iter().map(|&i| shape_a[i]).collect();
    let b_outer_dims: Vec<usize> = b_outer.iter().map(|&i| shape_b[i]).collect();
    let a_outer_size = prod(&a_outer_dims);
    let b_outer_size = prod(&b_outer_dims);
    let k_dims: Vec<usize> = axes_a.iter().map(|&i| shape_a[i]).collect();
    let k_size = prod(&k_dims);

    let mut out = vec![0.0; a_outer_size * b_outer_size];

    // helpers to expand a flat index into multi-index and then to flat offset
    let to_multi = |mut idx: usize, dims: &[usize]| -> Vec<usize> {
        let mut m = vec![0; dims.len()];
        for i in (0..dims.len()).rev() {
            let d = dims[i];
            m[i] = idx % d;
            idx /= d;
        }
        m
    };
    let offset = |multi: &[usize], axes: &[usize], strides: &[usize]| -> usize {
        multi.iter().zip(axes.iter()).map(|(v, &ax)| v * strides[ax]).sum()
    };

    for ao in 0..a_outer_size {
        let a_idx = to_multi(ao, &a_outer_dims);
        for bo in 0..b_outer_size {
            let b_idx = to_multi(bo, &b_outer_dims);
            let mut sum = 0.0;
            for k in 0..k_size {
                // expand k into contracted coords
                let k_multi = to_multi(k, &k_dims);
                // assemble full multi-index for A
                let mut a_full = vec![0; shape_a.len()];
                for (pos, &ax) in a_outer.iter().enumerate() { a_full[ax] = a_idx[pos]; }
                for (pos, &ax) in axes_a.iter().enumerate() { a_full[ax] = k_multi[pos]; }
                // assemble full multi-index for B
                let mut b_full = vec![0; shape_b.len()];
                for (pos, &bx) in b_outer.iter().enumerate() { b_full[bx] = b_idx[pos]; }
                for (pos, &bx) in axes_b.iter().enumerate() { b_full[bx] = k_multi[pos]; }

                let oa = a_full.iter().enumerate().map(|(i,&v)| v * sa[i]).sum::<usize>();
                let ob = b_full.iter().enumerate().map(|(i,&v)| v * sb[i]).sum::<usize>();
                sum += data_a[oa] * data_b[ob];
            }
            out[ao * b_outer_size + bo] = sum;
        }
    }
    (out_shape, out)
}
