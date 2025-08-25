// src/core/titan/algorithmic_ml.rs

/// --- 11) Logistic regression sigmoid & prob ---
/// h_theta(x) = 1 / (1 + exp(-θ·x))
pub fn logistic_prob(theta: &[f64], x: &[f64]) -> f64 {
    assert_eq!(theta.len(), x.len(), "theta/x length mismatch");
    let z: f64 = theta.iter().zip(x).map(|(t, xi)| t * xi).sum();
    1.0 / (1.0 + (-z).exp())
}

/// Binary cross-entropy loss (optional helper)
pub fn bce_loss(y_true: f64, y_prob: f64) -> f64 {
    // clamp for numerical stability
    let p = y_prob.clamp(1e-12, 1.0 - 1e-12);
    -(y_true * p.ln() + (1.0 - y_true) * (1.0 - p).ln())
}

/// Gradient of BCE wrt θ for a single (x, y)
/// ∇_θ = (h(x) - y) * x
pub fn logistic_grad(theta: &[f64], x: &[f64], y: f64) -> Vec<f64> {
    let p = logistic_prob(theta, x);
    x.iter().map(|xi| (p - y) * xi).collect()
}

/// --- 12) Softmax ---
/// softmax(z)_i = exp(z_i - max(z)) / Σ_j exp(z_j - max(z))
pub fn softmax(z: &[f64]) -> Vec<f64> {
    let zmax = z.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = z.iter().map(|&zi| (zi - zmax).exp()).collect();
    let sum: f64 = exps.iter().sum();
    if sum == 0.0 {
        // degenerate fallback: uniform
        vec![1.0 / z.len() as f64; z.len()]
    } else {
        exps.iter().map(|e| e / sum).collect()
    }
}

/// --- 13) Backprop weight update (vanilla SGD) ---
/// ΔW = -η * dE/dW ;  W_next = W + ΔW
/// Here: apply gradient matrix `dW` to weight matrix `W` in-place.
pub fn sgd_update(weights: &mut [Vec<f64>], dweights: &[Vec<f64>], lr: f64) {
    assert_eq!(weights.len(), dweights.len(), "row mismatch");
    for (w_row, g_row) in weights.iter_mut().zip(dweights.iter()) {
        assert_eq!(w_row.len(), g_row.len(), "col mismatch");
        for (w, g) in w_row.iter_mut().zip(g_row.iter()) {
            *w -= lr * g;
        }
    }
}

/// --- 14) SVM hinge-loss gradient (linear, L2 reg) ---
/// For one sample (x, y∈{-1,+1}): 
/// if y*(θ·x) < 1:  ∇ = λθ - y x
/// else:           ∇ = λθ
pub fn svm_hinge_grad(theta: &[f64], x: &[f64], y: f64, lambda: f64) -> Vec<f64> {
    assert_eq!(theta.len(), x.len(), "theta/x length mismatch");
    let margin = y * theta.iter().zip(x).map(|(t, xi)| t * xi).sum::<f64>();
    if margin < 1.0 {
        theta.iter().zip(x).map(|(t, xi)| lambda * t - y * xi).collect()
    } else {
        theta.iter().map(|t| lambda * t).collect()
    }
}

/// --- 15) K-means centroid update ---
/// μ_k = (1/|C_k|) Σ_{x_i ∈ C_k} x_i
/// `assignments[i]` = cluster index for sample i.
/// `k` = number of clusters.
pub fn kmeans_update_centroids(
    data: &[Vec<f64>],
    assignments: &[usize],
    k: usize,
) -> Vec<Vec<f64>> {
    assert_eq!(data.len(), assignments.len(), "data/assignments length mismatch");
    let d = if data.is_empty() { 0 } else { data[0].len() };
    let mut sums = vec![vec![0.0; d]; k];
    let mut counts = vec![0usize; k];

    for (x, &c) in data.iter().zip(assignments.iter()) {
        assert!(c < k, "assignment out of range");
        for j in 0..d { sums[c][j] += x[j]; }
        counts[c] += 1;
    }

    for c in 0..k {
        let denom = counts[c].max(1) as f64; // avoid div-by-zero; keeps empty cluster centroid unchanged at zeros
        for j in 0..d { sums[c][j] /= denom; }
    }
    sums
}
