pub fn solve_linear(a: f64, b: f64) -> Result<f64, &'static str> {
    // Solves ax + b = 0
    if a == 0.0 {
        Err("No solution exists if 'a' is zero.")
    } else {
        Ok(-b / a)
    }
}

pub fn quadratic_roots(a: f64, b: f64, c: f64) -> Result<(f64, f64), &'static str> {
    // Solves ax^2 + bx + c = 0
    if a == 0.0 {
        Err("Coefficient 'a' cannot be zero for a quadratic equation.")
    } else {
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            Err("No real roots exist for the given equation.")
        } else {
            let root1 = (-b + discriminant.sqrt()) / (2.0 * a);
            let root2 = (-b - discriminant.sqrt()) / (2.0 * a);
            Ok((root1, root2))
        }
    }
}

pub fn evaluate_polynomial(coefficients: &[f64], x: f64) -> f64 {
    // Evaluates a polynomial at a given x value
    // For example, coefficients = [2, -4, 3] represents 2x^2 - 4x + 3
    coefficients
        .iter()
        .enumerate()
        .map(|(i, &coeff)| coeff * x.powi(i as i32))
        .sum()
}

pub fn find_gcd(a: u64, b: u64) -> u64 {
    // Finds the greatest common divisor (GCD) using the Euclidean algorithm
    let mut x = a;
    let mut y = b;
    while y != 0 {
        let temp = y;
        y = x % y;
        x = temp;
    }
    x
}

pub fn find_lcm(a: u64, b: u64) -> u64 {
    // Finds the least common multiple (LCM) using the relationship: LCM(a, b) = (a * b) / GCD(a, b)
    if a == 0 || b == 0 {
        0
    } else {
        (a * b) / find_gcd(a, b)
    }
}
