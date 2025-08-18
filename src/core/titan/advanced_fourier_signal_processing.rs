use std::f64::consts::PI;

pub fn fast_fourier_transform(data: &[f64]) -> Vec<(f64, f64)> {
    // Computes the Fast Fourier Transform (FFT) using a recursive algorithm
    let n = data.len();
    if n == 0 || n & (n - 1) != 0 {
        panic!("Input size must be a power of 2.");
    }

    if n == 1 {
        return vec![(data[0], 0.0)];
    }

    let even: Vec<f64> = data.iter().step_by(2).cloned().collect();
    let odd: Vec<f64> = data.iter().skip(1).step_by(2).cloned().collect();

    let fft_even = fast_fourier_transform(&even);
    let fft_odd = fast_fourier_transform(&odd);

    let mut result = vec![(0.0, 0.0); n];
    for k in 0..n / 2 {
        let angle = -2.0 * PI * k as f64 / n as f64;
        let twiddle = (
            angle.cos() * fft_odd[k].0 - angle.sin() * fft_odd[k].1,
            angle.sin() * fft_odd[k].0 + angle.cos() * fft_odd[k].1,
        );

        result[k] = (fft_even[k].0 + twiddle.0, fft_even[k].1 + twiddle.1);
        result[k + n / 2] = (fft_even[k].0 - twiddle.0, fft_even[k].1 - twiddle.1);
    }

    result
}

pub fn inverse_fast_fourier_transform(data: &[(f64, f64)]) -> Vec<f64> {
    // Computes the Inverse Fast Fourier Transform (IFFT)
    let n = data.len();
    let conjugates: Vec<(f64, f64)> = data.iter().map(|&(re, im)| (re, -im)).collect(); // Removed mut

    let fft_result =
        fast_fourier_transform(&conjugates.iter().map(|&(re, _)| re).collect::<Vec<f64>>());
    fft_result
        .iter()
        .map(|&(re, _)| re / n as f64) // Changed im to _ to silence warning
        .collect()
}

pub fn spectrogram(signal: &[f64], window_size: usize, step_size: usize) -> Vec<Vec<(f64, f64)>> {
    // Computes the spectrogram of a signal
    if window_size > signal.len() {
        panic!("Window size cannot be larger than the signal length.");
    }

    let mut result = Vec::new();
    let mut start = 0;

    while start + window_size <= signal.len() {
        let window = &signal[start..start + window_size];
        let fft_result = fast_fourier_transform(window);
        result.push(fft_result);
        start += step_size;
    }

    result
}
