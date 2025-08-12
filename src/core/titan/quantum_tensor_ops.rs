pub fn kronecker_product(matrix1: &[Vec<f64>], matrix2: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let rows1 = matrix1.len();
    let cols1 = matrix1[0].len();
    let rows2 = matrix2.len();
    let cols2 = matrix2[0].len();

    let mut result = vec![vec![0.0; cols1 * cols2]; rows1 * rows2];

    for i in 0..rows1 {
        for j in 0..cols1 {
            for k in 0..rows2 {
                for l in 0..cols2 {
                    result[i * rows2 + k][j * cols2 + l] = matrix1[i][j] * matrix2[k][l];
                }
            }
        }
    }

    result
}

pub fn trace(matrix: &[Vec<f64>]) -> f64 {
    if matrix.len() != matrix[0].len() {
        panic!("Matrix must be square to compute trace.");
    }
    matrix.iter().enumerate().map(|(i, row)| row[i]).sum()
}
