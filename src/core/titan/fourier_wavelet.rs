use std::f64::consts::PI;

pub fn fourier_transform(data: &[f64]) -> Vec<(f64, f64)> {
    // Computes the discrete Fourier transform (DFT) of a real-valued signal
    let n = data.len();
    let mut result = Vec::with_capacity(n);

    for k in 0..n {
        let mut re = 0.0;
        let mut im = 0.0;

        for (i, &x) in data.iter().enumerate() {
            let angle = 2.0 * PI * k as f64 * i as f64 / n as f64;
            re += x * angle.cos();
            im -= x * angle.sin();
        }

        result.push((re, im));
    }

    result
}

pub fn inverse_fourier_transform(data: &[(f64, f64)]) -> Vec<f64> {
    // Computes the inverse discrete Fourier transform (IDFT)
    let n = data.len();
    let mut result = Vec::with_capacity(n);

    for i in 0..n {
        let mut value = 0.0;

        for (k, &(re, im)) in data.iter().enumerate() {
            let angle = 2.0 * PI * k as f64 * i as f64 / n as f64;
            value += re * angle.cos() - im * angle.sin();
        }

        result.push(value / n as f64);
    }

    result
}

pub fn wavelet_transform(data: &[f64], wavelet: &[f64]) -> Vec<f64> {
    // Computes the wavelet transform of a signal
    let data_len = data.len();
    let wavelet_len = wavelet.len();
    let mut result = vec![0.0; data_len];

    for i in 0..(data_len - wavelet_len + 1) {
        result[i] = data[i..(i + wavelet_len)]
            .iter()
            .zip(wavelet.iter())
            .map(|(&x, &w)| x * w)
            .sum();
    }

    result
}

pub fn inverse_wavelet_transform(data: &[f64], wavelet: &[f64]) -> Vec<f64> {
    // Computes the inverse wavelet transform of a signal
    let data_len = data.len();
    let wavelet_len = wavelet.len();
    let mut result = vec![0.0; data_len + wavelet_len - 1];

    for i in 0..data_len {
        for j in 0..wavelet_len {
            result[i + j] += data[i] * wavelet[j];
        }
    }

    result
}
