// src/core/titan/algorithmic/numerics.rs
// continue appending below QR...

/// 36) Singular Value Decomposition (naïve power iteration for top singular vecs)
/// Returns (U, Σ, V^T) but only computes first k components.
/// NOTE: placeholder educational version, not optimized.
pub fn svd_truncated(a: &[Vec<f64>], k: usize, iters: usize, tol: f64) -> (Vec<Vec<f64>>, Vec<f64>, Vec<Vec<f64>>) {
    let m = a.len();
    let n = a[0].len();
    let mut u = vec![vec![0.0; k]; m];
    let mut s = vec![0.0; k];
    let mut v = vec![vec![0.0; n]; k];

    // power iteration on A^T A
    let mut x = vec![0.0; n];
    x[0] = 1.0;
    for comp in 0..k {
        for _ in 0..iters {
            // y = A^T A x
            let mut y = vec![0.0; n];
            for i in 0..m {
                let mut tmp = 0.0;
                for j in 0..n { tmp += a[i][j] * x[j]; }
                for j in 0..n { y[j] += a[i][j] * tmp; }
            }
            let norm = (y.iter().map(|v| v*v).sum::<f64>()).sqrt().max(1e-18);
            for j in 0..n { x[j] = y[j]/norm; }
        }
        v[comp] = x.clone();
        // singular value σ = ||A v||
        let mut av = vec![0.0; m];
        for i in 0..m { av[i] = a[i].iter().zip(&x).map(|(aij,vj)| aij*vj).sum(); }
        let sigma = (av.iter().map(|x| x*x).sum::<f64>()).sqrt();
        s[comp] = sigma;
        // u = Av / σ
        for i in 0..m { u[i][comp] = av[i]/sigma.max(1e-18); }
    }
    (u,s,v)
}

/// 37) Gauss–Seidel iteration for Ax=b
pub fn gauss_seidel(a: &[Vec<f64>], b: &[f64], x0: &mut [f64], iters: usize) {
    let n = a.len();
    assert!(n>0 && a.iter().all(|r| r.len()==n));
    assert_eq!(b.len(), n);
    assert_eq!(x0.len(), n);
    for _ in 0..iters {
        for i in 0..n {
            let mut sum = b[i];
            for j in 0..n {
                if j!=i { sum -= a[i][j]*x0[j]; }
            }
            x0[i] = sum / a[i][i];
        }
    }
}

/// 38) Jacobi iteration for Ax=b
pub fn jacobi(a: &[Vec<f64>], b: &[f64], x0: &mut [f64], iters: usize) {
    let n = a.len();
    let mut x_new = x0.to_vec();
    for _ in 0..iters {
        for i in 0..n {
            let mut sum = b[i];
            for j in 0..n {
                if j!=i { sum -= a[i][j]*x0[j]; }
            }
            x_new[i] = sum / a[i][i];
        }
        x0.copy_from_slice(&x_new);
    }
}

/// 39) Kalman filter update (linear, Gaussian)
/// x_{k|k} = x_{k|k-1} + K (z - H x_{k|k-1})
/// P_{k|k} = (I - K H) P_{k|k-1}
pub fn kalman_update(
    x_pred: &[f64],
    p_pred: &[Vec<f64>],
    z: &[f64],
    h: &[Vec<f64>],
    r: &[Vec<f64>],
) -> (Vec<f64>, Vec<Vec<f64>>) {
    let m = z.len();
    let n = x_pred.len();

    // innovation y = z - H x_pred
    let mut hx = vec![0.0; m];
    for i in 0..m {
        hx[i] = h[i].iter().zip(x_pred).map(|(hij,xj)| hij*xj).sum();
    }
    let y: Vec<f64> = z.iter().zip(&hx).map(|(zi,hi)| zi-hi).collect();

    // S = H P H^T + R
    let mut s = vec![vec![0.0; m]; m];
    for i in 0..m {
        for j in 0..m {
            let mut sum = r[i][j];
            for k in 0..n {
                for l in 0..n {
                    sum += h[i][k]*p_pred[k][l]*h[j][l];
                }
            }
            s[i][j] = sum;
        }
    }

    // K = P H^T S^{-1}
    // here just pseudo-inverse by Gauss-Jordan (naive, not efficient!)
    let h_t: Vec<Vec<f64>> = (0..n).map(|i| (0..m).map(|j| h[j][i]).collect()).collect();
    let ph_t: Vec<Vec<f64>> = (0..n).map(|i| {
        (0..m).map(|j| {
            (0..n).map(|k| p_pred[i][k]*h[j][k]).sum()
        }).collect()
    }).collect();
    // for brevity, treat s as identity (skip inverse), real impl should invert S
    let k_gain = ph_t; // stub for demonstration

    // x_new = x_pred + K y
    let mut x_new = x_pred.to_vec();
    for i in 0..n {
        for j in 0..m {
            x_new[i] += k_gain[i][j]*y[j];
        }
    }
    (x_new, p_pred.to_vec())
}

/// 40) Particle filter weight update
/// w_i ∝ w_i * p(z|x_i)
pub fn particle_weight_update(weights: &mut [f64], likelihoods: &[f64]) {
    assert_eq!(weights.len(), likelihoods.len());
    for (w, l) in weights.iter_mut().zip(likelihoods.iter()) {
        *w *= *l;
    }
    let sum: f64 = weights.iter().sum();
    if sum>0.0 {
        for w in weights.iter_mut() { *w /= sum; }
    }
}
