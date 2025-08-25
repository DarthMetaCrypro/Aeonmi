// src/core/titan/algorithmic/numerics.rs

/// 31) Runge–Kutta 4th order (single step)
/// y_{n+1} = y_n + 1/6 (k1 + 2k2 + 2k3 + k4)
/// dy/dt = f(t, y)
pub fn rk4_step<F>(f: F, t: f64, y: f64, h: f64) -> f64
where F: Fn(f64, f64) -> f64 {
    let k1 = h * f(t, y);
    let k2 = h * f(t + h/2.0, y + k1/2.0);
    let k3 = h * f(t + h/2.0, y + k2/2.0);
    let k4 = h * f(t + h, y + k3);
    y + (k1 + 2.0*k2 + 2.0*k3 + k4) / 6.0
}

/// 32) Finite difference derivative
/// f'(x) ≈ (f(x+h) - f(x)) / h
pub fn finite_diff<F>(f: F, x: f64, h: f64) -> f64
where F: Fn(f64) -> f64 {
    (f(x + h) - f(x)) / h
}

/// 33) FFT butterfly operation (Cooley–Tukey radix-2 step)
/// X[k] = E[k] + W_N^k O[k]; X[k+N/2] = E[k] - W_N^k O[k]
use num_complex::Complex64;
use std::f64::consts::PI;

pub fn fft_butterfly(e: &[Complex64], o: &[Complex64]) -> Vec<Complex64> {
    let n2 = e.len();
    assert_eq!(o.len(), n2);
    let n = 2*n2;
    let mut out = vec![Complex64::new(0.0,0.0); n];
    for k in 0..n2 {
        let wn = Complex64::from_polar(1.0, -2.0*PI*(k as f64)/ (n as f64));
        out[k]       = e[k] + wn * o[k];
        out[k+n2]    = e[k] - wn * o[k];
    }
    out
}

/// 34) LU Decomposition (Doolittle, no pivoting)
/// A = L U ; returns (L,U) with L unit lower-triangular
pub fn lu_decompose(mut a: Vec<Vec<f64>>) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let n = a.len();
    assert!(n>0 && a.iter().all(|r| r.len()==n));
    let mut l = vec![vec![0.0; n]; n];
    let mut u = vec![vec![0.0; n]; n];

    for i in 0..n { l[i][i] = 1.0; }

    for j in 0..n {
        for i in 0..=j {
            let mut sum = 0.0;
            for k in 0..i { sum += l[i][k] * u[k][j]; }
            u[i][j] = a[i][j] - sum;
        }
        for i in j+1..n {
            let mut sum = 0.0;
            for k in 0..j { sum += l[i][k] * u[k][j]; }
            l[i][j] = (a[i][j] - sum) / u[j][j];
        }
    }
    (l,u)
}

/// 35) QR Decomposition (Gram–Schmidt)
/// A = Q R ; returns (Q,R)
pub fn qr_decompose(a: Vec<Vec<f64>>) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let n = a.len();
    let m = a[0].len();
    assert!(a.iter().all(|r| r.len()==m));

    let mut q = vec![vec![0.0; m]; n];
    let mut r = vec![vec![0.0; m]; m];
    let mut a_copy = a.clone();

    for j in 0..m {
        let mut v: Vec<f64> = a_copy.iter().map(|row| row[j]).collect();
        for i in 0..j {
            let dot = a_copy.iter().zip(q.iter()).map(|(row, qrow)| row[j]*qrow[i]).sum::<f64>();
            r[i][j] = dot;
            for k in 0..n { v[k] -= dot * q[k][i]; }
        }
        let norm = (v.iter().map(|x| x*x).sum::<f64>()).sqrt();
        r[j][j] = norm;
        for k in 0..n { q[k][j] = v[k] / norm; }
    }
    (q,r)
}

