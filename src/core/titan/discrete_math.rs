pub fn factorial(n: u64) -> u64 {
    // Computes the factorial of a number (n!)
    (1..=n).product()
}

pub fn combinations(n: u64, k: u64) -> Result<u64, &'static str> {
    // Computes combinations (n choose k)
    if k > n {
        return Err("k cannot be greater than n.");
    }
    Ok(factorial(n) / (factorial(k) * factorial(n - k)))
}

pub fn gcd(a: u64, b: u64) -> u64 {
    // Computes the greatest common divisor (GCD) using the Euclidean algorithm
    let mut x = a;
    let mut y = b;
    while y != 0 {
        let temp = y;
        y = x % y;
        x = temp;
    }
    x
}

pub fn lcm(a: u64, b: u64) -> u64 {
    // Computes the least common multiple (LCM) using GCD
    if a == 0 || b == 0 {
        0
    } else {
        (a * b) / gcd(a, b)
    }
}

pub fn is_prime(n: u64) -> bool {
    // Checks if a number is prime
    if n <= 1 {
        return false;
    }
    for i in 2..=((n as f64).sqrt() as u64) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

pub fn sieve_of_eratosthenes(limit: u64) -> Vec<u64> {
    // Generates a list of prime numbers up to a given limit using the Sieve of Eratosthenes
    let mut is_prime = vec![true; (limit + 1) as usize];
    is_prime[0] = false;
    if limit > 0 {
        is_prime[1] = false;
    }

    for i in 2..=((limit as f64).sqrt() as usize) {
        if is_prime[i] {
            for j in (i * i..=limit as usize).step_by(i) {
                is_prime[j] = false;
            }
        }
    }

    is_prime
        .iter()
        .enumerate()
        .filter(|&(_, &prime)| prime)
        .map(|(i, _)| i as u64)
        .collect()
}

pub fn graph_is_connected(adj_matrix: &[Vec<bool>]) -> bool {
    // Determines if a graph is connected using breadth-first search (BFS)
    let n = adj_matrix.len();
    if n == 0 {
        return true;
    }

    let mut visited = vec![false; n];
    let mut queue = vec![0];
    visited[0] = true;

    while let Some(node) = queue.pop() {
        for (neighbor, &is_connected) in adj_matrix[node].iter().enumerate() {
            if is_connected && !visited[neighbor] {
                visited[neighbor] = true;
                queue.push(neighbor);
            }
        }
    }

    visited.iter().all(|&v| v)
}
