#![allow(clippy::needless_range_loop)]
pub fn dot_product(v1: &[f64], v2: &[f64]) -> Result<f64, &'static str> {
    // Computes the dot product of two vectors
    if v1.len() != v2.len() {
        Err("Vectors must be of the same length.")
    } else {
        Ok(v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum())
    }
}

pub fn matrix_multiply(m1: &[Vec<f64>], m2: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, &'static str> {
    // Multiplies two matrices
    let m1_cols = m1[0].len();
    let m2_rows = m2.len();
    if m1_cols != m2_rows {
        return Err("Matrix dimensions are incompatible for multiplication.");
    }

    let result_rows = m1.len();
    let result_cols = m2[0].len();

    let mut result = vec![vec![0.0; result_cols]; result_rows];

    for i in 0..result_rows {
        for j in 0..result_cols {
            result[i][j] = (0..m1_cols).map(|k| m1[i][k] * m2[k][j]).sum();
        }
    }

    Ok(result)
}

pub fn transpose(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
    // Transposes a matrix
    let rows = matrix.len();
    let cols = matrix[0].len();

    let mut transposed = vec![vec![0.0; rows]; cols];
    for i in 0..rows {
        for j in 0..cols {
            transposed[j][i] = matrix[i][j];
        }
    }

    transposed
}

pub fn determinant(matrix: &[Vec<f64>]) -> Result<f64, &'static str> {
    // Computes the determinant of a square matrix using recursion
    let n = matrix.len();
    if matrix.iter().any(|row| row.len() != n) {
        return Err("Matrix must be square.");
    }

    if n == 1 {
        return Ok(matrix[0][0]);
    }

    if n == 2 {
        return Ok(matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0]);
    }

    let mut det = 0.0;
    for (j, &value) in matrix[0].iter().enumerate() {
        let minor = matrix
            .iter()
            .skip(1)
            .map(|row| {
                row.iter()
                    .enumerate()
                    .filter(|&(col, _)| col != j)
                    .map(|(_, &v)| v)
                    .collect::<Vec<f64>>()
            })
            .collect::<Vec<Vec<f64>>>();

        det += value * determinant(&minor)?.powi(if j % 2 == 0 { 1 } else { -1 });
    }

    Ok(det)
}

pub fn identity_matrix(size: usize) -> Vec<Vec<f64>> {
    // Creates an identity matrix of the given size
    let mut identity = vec![vec![0.0; size]; size];
    for i in 0..size {
        identity[i][i] = 1.0;
    }
    identity
}
