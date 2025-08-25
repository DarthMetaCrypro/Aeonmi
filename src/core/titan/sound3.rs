use num_complex::Complex64 as C;
use std::f64::consts::PI;

/* --------------------- 41) STFT (Hann window, hop) ---------------------- */
// Returns matrix X [frames][fft_len] (complex spectrum per frame).
// signal padded at ends; no zero-mean detrend here.
pub fn stft_hann(
    signal: &[f64],
    fft_len: usize,
    hop: usize,
) -> Vec<Vec<C>> {
    assert!(fft_len.is_power_of_two(), "fft_len must be power of two");
    assert!(hop > 0 && hop <= fft_len);

    let n = signal.len();
    let win: Vec<f64> = (0..fft_len)
        .map(|n| 0.5 * (1.0 - (2.0 * PI * n as f64 / (fft_len as f64)).cos()))
        .collect();

    let mut frames = vec![];
    let mut start = 0usize;
    while start + fft_len <= n {
        let mut buf: Vec<C> = (0..fft_len)
            .map(|i| C::new(signal[start + i] * win[i], 0.0))
            .collect();
        fft_inplace_radix2(&mut buf);
        frames.push(buf);
        start += hop;
    }
    frames
}

/* ---------------------- 42) iSTFT (OLA, Hann/COLA) ---------------------- */
// Overlap-add with Hann window using COLA hop = fft_len/2 by default.
// If you pass a different hop, you’re responsible for COLA consistency.
pub fn istft_hann(
    spectrogram: &[Vec<C>],
    fft_len: usize,
    hop: usize,
) -> Vec<f64> {
    assert!(!spectrogram.is_empty());
    let frames = spectrogram.len();
    let out_len = (frames - 1) * hop + fft_len;

    let win: Vec<f64> = (0..fft_len)
        .map(|n| 0.5 * (1.0 - (2.0 * PI * n as f64 / (fft_len as f64)).cos()))
        .collect();

    // inverse FFT per frame
    let mut out = vec![0.0f64; out_len];
    let mut ola_norm = vec![0.0f64; out_len];
    for (fidx, spec) in spectrogram.iter().enumerate() {
        let mut time = spec.clone();
        ifft_inplace_radix2(&mut time);
        let base = fidx * hop;
        for i in 0..fft_len {
            let w = win[i];
            out[base + i] += time[i].re * w;
            ola_norm[base + i] += w * w;
        }
    }
    for i in 0..out_len {
        let nrm = ola_norm[i];
        if nrm > 1e-12 { out[i] /= nrm; }
    }
    out
}

/* -------------------- 43) Morlet CWT coefficient ------------------------ */
// One scale at center time index t0.
// ψ(t) = π^{-1/4} exp(i ω0 τ) exp(-τ^2/2), τ = (t - t0)/scale
pub fn cwt_morlet_coeff(
    signal: &[f64],
    t0: usize,
    scale: f64,
    omega0: f64,
    dt: f64,
) -> C {
    assert!(t0 < signal.len());
    assert!(scale > 0.0 && dt > 0.0);
    let n = signal.len();
    let norm = PI.powf(-0.25);

    let mut acc = C::new(0.0, 0.0);
    for (t, &x) in signal.iter().enumerate() {
        let tau = ((t as f64 - t0 as f64) * dt) / scale;
        let gauss = (-0.5 * tau * tau).exp() * norm;
        let phase = C::from_polar(1.0, omega0 * tau);
        acc += C::new(x, 0.0) * phase * gauss;
    }
    (acc / scale.sqrt())
}

/* ---------------- 44) Rectangular membrane modal freqs ------------------ */
// Ideal membrane with fixed edges (Lx, Ly), wave speed c:
// f_{m,n} = (c/2) * sqrt( (m/Lx)^2 + (n/Ly)^2 ), m,n = 1,2,...
pub fn membrane_modes_rect(c: f64, lx: f64, ly: f64, m_max: usize, n_max: usize) -> Vec<f64> {
    let mut freqs = Vec::with_capacity(m_max * n_max);
    for m in 1..=m_max {
        for n in 1..=n_max {
            let val = ( (m as f64 / lx).powi(2) + (n as f64 / ly).powi(2) ).sqrt();
            freqs.push(0.5 * c * val);
        }
    }
    freqs
}

/* ----------------- 45) Single-bin Goertzel (DFT bin) -------------------- */
// Efficient DFT magnitude at k-bin for real signal.
pub fn goertzel_mag(signal: &[f64], k: usize, n: usize) -> f64 {
    assert!(n > 0 && k < n);
    let w = 2.0 * PI * (k as f64) / (n as f64);
    let coeff = 2.0 * w.cos();
    let mut s0 = 0.0;
    let mut s1 = 0.0;
    let mut s2 = 0.0;
    for (idx, &x) in signal.iter().take(n).enumerate() {
        s0 = x + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    let real = s1 - s2 * w.cos();
    let imag = s2 * w.sin();
    (real * real + imag * imag).sqrt()
}

/* ------------------- tiny FFT/iFFT (radix-2, in-place) ------------------ */
fn fft_inplace_radix2(a: &mut [C]) {
    let n = a.len();
    assert!(n.is_power_of_two());
    // bit-reverse
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 { j ^= bit; bit >>= 1; }
        j ^= bit;
        if i < j { a.swap(i, j); }
    }
    let mut len = 2;
    while len <= n {
        let theta = -2.0 * PI / (len as f64);
        let wlen = C::from_polar(1.0, theta);
        for i in (0..n).step_by(len) {
            let mut w = C::new(1.0, 0.0);
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
fn ifft_inplace_radix2(a: &mut [C]) {
    // conjugate, FFT, conjugate, scale
    let n = a.len() as f64;
    for v in a.iter_mut() { *v = v.conj(); }
    fft_inplace_radix2(a);
    for v in a.iter_mut() { *v = v.conj() / C::new(n, 0.0); }
}
