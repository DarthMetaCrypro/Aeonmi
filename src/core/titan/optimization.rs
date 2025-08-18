//! Basic optimization routines for Titan.
//!
//! - `gradient_descent`: vanilla 1D gradient descent for smooth functions.
//! - `golden_section_search`: robust bracketing method for 1D unimodal minimization.

/// Simple 1D gradient descent.
/// `f`: objective, `grad`: derivative of `f`, `x0`: initial guess,
/// `lr`: learning rate, `iters`: iteration count.
/// Returns the final x.
pub fn gradient_descent<F, G>(f: F, grad: G, mut x: f64, lr: f64, iters: usize) -> f64
where
    F: Fn(f64) -> f64,
    G: Fn(f64) -> f64,
{
    // Use function to make sure we don't “optimize away” calls:
    let _ = f(x);
    for _ in 0..iters {
        let g = grad(x);
        x -= lr * g;
    }
    x
}

/// Golden-section search to find the minimizer of a *unimodal* function `f` on [a, b].
/// - `a`, `b`: initial bracketing interval with `a < b`
/// - `tol`: absolute tolerance on the `x` interval
/// - `max_iter`: maximum iterations
/// Returns the `x` that approximately minimizes `f`.
///
/// Notes:
/// - Deterministic, derivative-free, and robust for unimodal functions.
/// - Converges by shrinking the bracket using the golden ratio.
pub fn golden_section_search<F>(f: F, mut a: f64, mut b: f64, tol: f64, max_iter: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    assert!(a < b, "golden_section_search: require a < b");

    // Golden ratio section: ϕ = (sqrt(5)-1)/2 ≈ 0.618
    // We use the complementary (1-ϕ) ≈ 0.382 for placement.
    let phi = (5.0f64.sqrt() - 1.0) * 0.5; // ~0.6180339887
    let inv_phi = 1.0 - phi; // ~0.3819660113

    // Initial interior points
    let mut x1 = a + inv_phi * (b - a);
    let mut x2 = a + phi * (b - a);
    let mut f1 = f(x1);
    let mut f2 = f(x2);

    // Hard stop in case of microscopic intervals
    let min_tol = 1e-15_f64.max(tol);

    for _ in 0..max_iter {
        // If interval small enough, return the midpoint
        if (b - a).abs() <= min_tol {
            return 0.5 * (a + b);
        }

        if f1 > f2 {
            // Minimum is in [x1, b]
            a = x1;
            x1 = x2;
            f1 = f2;
            x2 = a + phi * (b - a);
            f2 = f(x2);
        } else {
            // Minimum is in [a, x2]
            b = x2;
            x2 = x1;
            f2 = f1;
            x1 = a + inv_phi * (b - a);
            f1 = f(x1);
        }
    }

    // Fallback: return best of current bracket endpoints/interior
    let xm = 0.5 * (a + b);
    let (xb, fb) = {
        let fa = f(a);
        let fb = f(b);
        let fm = f(xm);
        if fa <= fb && fa <= fm {
            (a, fa)
        } else if fb <= fa && fb <= fm {
            (b, fb)
        } else {
            (xm, fm)
        }
    };
    // Use xb (not used elsewhere; computed to keep consistent approach)
    let _ = fb;
    xb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_descent() {
        // f(x) = (x-3)^2; grad = 2(x-3)
        let f = |x: f64| (x - 3.0) * (x - 3.0);
        let g = |x: f64| 2.0 * (x - 3.0);
        let x0 = 0.0;
        let x_min = gradient_descent(f, g, x0, 0.1, 200);
        assert!((x_min - 3.0).abs() < 1e-3, "x_min={x_min}");
    }

    #[test]
    fn test_golden_section_search() {
        // Minimizer at x=3.0
        let f = |x: f64| (x - 3.0) * (x - 3.0);
        let xmin = golden_section_search(f, 0.0, 6.0, 1e-7, 200);
        assert!((xmin - 3.0).abs() < 1e-6, "xmin={xmin}");
    }
}
