pub fn add_tensors(
    tensor1: &[Vec<Vec<f64>>],
    tensor2: &[Vec<Vec<f64>>],
) -> Result<Vec<Vec<Vec<f64>>>, &'static str> {
    // Adds two tensors element-wise
    if tensor1.len() != tensor2.len()
        || tensor1[0].len() != tensor2[0].len()
        || tensor1[0][0].len() != tensor2[0][0].len()
    {
        return Err("Tensors must have the same dimensions for addition.");
    }

    let result = tensor1
        .iter()
        .zip(tensor2.iter())
        .map(|(mat1, mat2)| {
            mat1.iter()
                .zip(mat2.iter())
                .map(|(row1, row2)| {
                    row1.iter()
                        .zip(row2.iter())
                        .map(|(&val1, &val2)| val1 + val2)
                        .collect()
                })
                .collect()
        })
        .collect();

    Ok(result)
}

pub fn scalar_multiply_tensor(tensor: &[Vec<Vec<f64>>], scalar: f64) -> Vec<Vec<Vec<f64>>> {
    // Multiplies a tensor by a scalar
    tensor
        .iter()
        .map(|matrix| {
            matrix
                .iter()
                .map(|row| row.iter().map(|&val| val * scalar).collect())
                .collect()
        })
        .collect()
}

pub fn dot_product_tensors(
    tensor1: &[Vec<Vec<f64>>],
    tensor2: &[Vec<Vec<f64>>],
) -> Result<f64, &'static str> {
    // Computes the dot product of two tensors
    if tensor1.len() != tensor2.len()
        || tensor1[0].len() != tensor2[0].len()
        || tensor1[0][0].len() != tensor2[0][0].len()
    {
        return Err("Tensors must have the same dimensions for dot product.");
    }

    let result = tensor1
        .iter()
        .zip(tensor2.iter())
        .map(|(mat1, mat2)| {
            mat1.iter()
                .zip(mat2.iter())
                .map(|(row1, row2)| {
                    row1.iter()
                        .zip(row2.iter())
                        .map(|(&val1, &val2)| val1 * val2)
                        .sum::<f64>()
                })
                .sum::<f64>()
        })
        .sum();

    Ok(result)
}

pub fn transpose_matrix(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
    // Transposes a single matrix
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

pub fn tensor_shape(tensor: &[Vec<Vec<f64>>]) -> (usize, usize, usize) {
    // Returns the shape of a tensor (depth, rows, cols)
    let depth = tensor.len();
    let rows = if depth > 0 { tensor[0].len() } else { 0 };
    let cols = if rows > 0 { tensor[0][0].len() } else { 0 };
    (depth, rows, cols)
}
