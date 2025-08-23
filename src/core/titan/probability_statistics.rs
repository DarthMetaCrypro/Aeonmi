use rand::distributions::{Distribution, Uniform};
use rand_distr::Normal;

pub fn multivariate_gaussian(
    mean: &[f64],
    covariance: &[Vec<f64>],
    _samples: usize,
) -> Result<Vec<Vec<f64>>, &'static str> {
    // Generates samples from a multivariate Gaussian distribution
    let dim = mean.len();
    if covariance.len() != dim || covariance.iter().any(|row| row.len() != dim) {
        return Err(
            "Covariance matrix must be square and match the dimensionality of the mean vector.",
        );
    }

    // Decompose covariance matrix (placeholder for Cholesky decomposition)
    Err("Cholesky decomposition not yet implemented.")
}

pub fn random_sample_uniform(min: f64, max: f64, n: usize) -> Vec<f64> {
    // Samples n random values from a uniform distribution
    let mut rng = rand::thread_rng();
    let uniform = Uniform::new(min, max);
    (0..n).map(|_| uniform.sample(&mut rng)).collect()
}

pub fn random_sample_normal(mean: f64, std_dev: f64, n: usize) -> Vec<f64> {
    // Samples n random values from a normal distribution
    let mut rng = rand::thread_rng();
    let normal = Normal::new(mean, std_dev).unwrap();
    (0..n).map(|_| normal.sample(&mut rng)).collect()
}

pub fn bayesian_inference(prior: f64, likelihood: f64, evidence: f64) -> f64 {
    // Computes posterior probability using Bayes' theorem: P(H|E) = (P(E|H) * P(H)) / P(E)
    if evidence == 0.0 {
        panic!("Evidence cannot be zero in Bayesian inference.");
    }
    (likelihood * prior) / evidence
}

pub fn compute_entropy(probabilities: &[f64]) -> f64 {
    // Computes the Shannon entropy of a probability distribution
    probabilities
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.log2())
        .sum()
}
