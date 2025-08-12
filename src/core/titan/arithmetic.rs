pub fn add(a: f64, b: f64) -> f64 {
    a + b
}

pub fn subtract(a: f64, b: f64) -> f64 {
    a - b
}

pub fn multiply(a: f64, b: f64) -> f64 {
    a * b
}

pub fn divide(a: f64, b: f64) -> Result<f64, &'static str> {
    if b == 0.0 {
        Err("Division by zero is not allowed.")
    } else {
        Ok(a / b)
    }
}

pub fn power(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

pub fn factorial(n: u64) -> u64 {
    if n == 0 {
        1
    } else {
        (1..=n).product()
    }
}
