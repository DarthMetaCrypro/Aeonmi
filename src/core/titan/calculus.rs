pub fn differentiate<F>(function: F, x: f64, h: f64) -> f64
where
    F: Fn(f64) -> f64,
{
    // Numerical differentiation using the symmetric difference quotient
    (function(x + h) - function(x - h)) / (2.0 * h)
}

pub fn integrate<F>(function: F, a: f64, b: f64, n: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    // Numerical integration using the trapezoidal rule
    if n == 0 {
        panic!("Number of intervals 'n' must be greater than zero.");
    }
    let h = (b - a) / n as f64;
    let mut total = 0.5 * (function(a) + function(b));
    for i in 1..n {
        total += function(a + i as f64 * h);
    }
    total * h
}

pub fn find_extrema<F>(function: F, x_start: f64, x_end: f64, step: f64) -> (f64, f64)
where
    F: Fn(f64) -> f64,
{
    // Finds the maximum and minimum of a function over a range [x_start, x_end]
    let mut max_value = function(x_start);
    let mut min_value = max_value;
    let mut x = x_start;

    while x <= x_end {
        let fx = function(x);
        if fx > max_value {
            max_value = fx;
        }
        if fx < min_value {
            min_value = fx;
        }
        x += step;
    }

    (min_value, max_value)
}

pub fn arc_length<F>(function: F, a: f64, b: f64, n: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    // Calculates the arc length of a curve defined by a function over [a, b]
    if n == 0 {
        panic!("Number of intervals 'n' must be greater than zero.");
    }
    let h = (b - a) / n as f64;
    let mut length = 0.0;

    for i in 0..n {
        let x1 = a + i as f64 * h;
        let x2 = a + (i as f64 + 1.0) * h;
        let y1 = function(x1);
        let y2 = function(x2);
        length += ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    }

    length
}
