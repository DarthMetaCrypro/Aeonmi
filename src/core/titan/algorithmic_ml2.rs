// src/core/titan/algorithmic_ml2.rs
// (append to the file you already created)

// ---------- 16) EM (GMM) — E-step (diagonal covariances) ----------
// Inputs:
//   data: [n x d], weights: [k], means: [k x d], cov_diag: [k x d] (variances per dim)
// Output:
//   resp: [n x k] responsibilities where resp[i][j] = P(z=j|x_i)
pub fn gmm_e_step_diag(
    data: &[Vec<f64>],
    weights: &[f64],
    means: &[Vec<f64>],
    cov_diag: &[Vec<f64>],
) -> Vec<Vec<f64>> {
    let n = data.len();
    let k = weights.len();
    assert!(k == means.len() && k == cov_diag.len(), "inconsistent K");
    let d = if n == 0 { means[0].len() } else { data[0].len() };
    assert!(means.iter().all(|m| m.len() == d));
    assert!(cov_diag.iter().all(|v| v.len() == d));

    // precompute log-normalization for each component
    let mut log_norm = vec![0.0; k];
    for j in 0..k {
        let det = cov_diag[j].iter().product::<f64>();
        let det = det.max(1e-300);
        log_norm[j] = -0.5 * (d as f64) * (2.0 * std::f64::consts::PI).ln()
                    - 0.5 * det.ln();
    }

    let mut resp = vec![vec![0.0; k]; n];
    for i in 0..n {
        // log-prob per component
        let mut logp = vec![0.0; k];
        for j in 0..k {
            let mut quad = 0.0;
            for t in 0..d {
                let diff = data[i][t] - means[j][t];
                let var  = cov_diag[j][t].max(1e-12);
                quad += diff * diff / var;
            }
            logp[j] = (weights[j].max(1e-300)).ln() + log_norm[j] - 0.5 * quad;
        }
        // log-sum-exp for stability
        let m = logp.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mut sum = 0.0;
        for j in 0..k { sum += (logp[j] - m).exp(); }
        let lse = m + sum.ln();
        for j in 0..k {
            resp[i][j] = (logp[j] - lse).exp();
        }
    }
    resp
}

// ---------- 17) Gaussian Naive Bayes (log-prob & predict) ----------
// priors: [c], means: [c x d], vars: [c x d] (variances per dim)
pub fn gnb_log_proba(x: &[f64], priors: &[f64], means: &[Vec<f64>], vars: &[Vec<f64>]) -> Vec<f64> {
    let c = priors.len();
    let d = x.len();
    assert!(means.len() == c && vars.len() == c);
    assert!(means.iter().all(|m| m.len() == d) && vars.iter().all(|v| v.len() == d));

    let two_pi = 2.0 * std::f64::consts::PI;
    let mut logp = vec![0.0; c];
    for k in 0..c {
        let mut s = (priors[k].max(1e-300)).ln();
        for j in 0..d {
            let var = vars[k][j].max(1e-12);
            let diff = x[j] - means[k][j];
            s += -0.5 * ((two_pi * var).ln() + diff * diff / var);
        }
        logp[k] = s;
    }
    logp
}

pub fn gnb_predict(x: &[f64], priors: &[f64], means: &[Vec<f64>], vars: &[Vec<f64>]) -> usize {
    let lp = gnb_log_proba(x, priors, means, vars);
    lp.iter()
        .enumerate()
        .max_by(|a,b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i,_)| i)
        .unwrap_or(0)
}

// ---------- 18) PCA (top principal component via power iteration) ----------
pub fn pca_first_component(data: &[Vec<f64>], iters: usize, tol: f64) -> (f64, Vec<f64>) {
    let n = data.len();
    assert!(n > 0, "empty data");
    let d = data[0].len();
    assert!(data.iter().all(|x| x.len() == d));

    // mean-center
    let mut mean = vec![0.0; d];
    for x in data { for j in 0..d { mean[j] += x[j]; } }
    for j in 0..d { mean[j] /= n as f64; }
    let mut xc = vec![vec![0.0; d]; n];
    for i in 0..n { for j in 0..d { xc[i][j] = data[i][j] - mean[j]; } }

    // power method on covariance implicitly: v_{t+1} ∝ (X^T X) v_t
    let mut v = vec![0.0; d];
    v[0] = 1.0; // init
    let mut lambda = 0.0;

    for _ in 0..iters {
        // y = X^T (X v)
        let mut xv = vec![0.0; n];
        for i in 0..n {
            xv[i] = dot(&xc[i], &v);
        }
        let mut y = vec![0.0; d];
        for j in 0..d {
            let mut s = 0.0;
            for i in 0..n { s += xc[i][j] * xv[i]; }
            y[j] = s;
        }
        let norm = l2(&y).max(1e-18);
        for j in 0..d { v[j] = y[j] / norm; }
        let new_lambda = dot(&v, &y) / (n as f64 - 1.0); // Rayleigh quotient / (n-1)
        if (new_lambda - lambda).abs() < tol { lambda = new_lambda; break; }
        lambda = new_lambda;
    }
    (lambda, v)
}

fn dot(a: &[f64], b: &[f64]) -> f64 { a.iter().zip(b).map(|(x,y)| x*y).sum() }
fn l2(a: &[f64]) -> f64 { a.iter().map(|x| x*x).sum::<f64>().sqrt() }

// ---------- 19) Gradient Boosting — additive update ----------
// F_m(x) = F_{m-1}(x) + ν * h_m(x)
// Inputs: prev predictions yhat_prev[n], weak learner preds h[n], learning rate nu
pub fn gbm_add_step(yhat_prev: &[f64], h_pred: &[f64], nu: f64) -> Vec<f64> {
    assert_eq!(yhat_prev.len(), h_pred.len());
    yhat_prev.iter().zip(h_pred).map(|(p, h)| p + nu * h).collect()
}

// ---------- 20) Transformer Attention (single-head) ----------
// Shapes:
//   Q: [n_q x d_k], K: [n_k x d_k], V: [n_k x d_v]
// Returns:
//   O: [n_q x d_v]
pub fn attention_single_head(q: &[Vec<f64>], k: &[Vec<f64>], v: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n_q = q.len(); let d_k = if n_q==0 {0} else { q[0].len() };
    let n_k = k.len(); assert!(n_k > 0, "empty K");
    assert!(k.iter().all(|row| row.len() == d_k));
    assert!(v.len() == n_k);
    let d_v = v[0].len();
    assert!(v.iter().all(|row| row.len() == d_v));

    let scale = 1.0 / (d_k as f64).sqrt().max(1.0);
    // scores = Q K^T  -> [n_q x n_k]
    let mut scores = vec![vec![0.0; n_k]; n_q];
    for i in 0..n_q {
        for j in 0..n_k {
            scores[i][j] = scale * dot(&q[i], &k[j]);
        }
    }
    // row-wise softmax
    for i in 0..n_q {
        let m = scores[i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mut s = 0.0;
        for j in 0..n_k { scores[i][j] = (scores[i][j] - m).exp(); s += scores[i][j]; }
        let inv = if s == 0.0 { 1.0 / n_k as f64 } else { 1.0 / s };
        for j in 0..n_k { scores[i][j] *= inv; }
    }
    // O = softmax(scores) V
    let mut out = vec![vec![0.0; d_v]; n_q];
    for i in 0..n_q {
        for j in 0..n_k {
            let w = scores[i][j];
            for t in 0..d_v { out[i][t] += w * v[j][t]; }
        }
    }
    out
}
