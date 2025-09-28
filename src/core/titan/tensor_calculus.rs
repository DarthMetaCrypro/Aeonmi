#![allow(clippy::needless_range_loop)]
pub fn tensor_addition(
    tensor1: &[Vec<Vec<f64>>],
    tensor2: &[Vec<Vec<f64>>],
) -> Result<Vec<Vec<Vec<f64>>>, &'static str> {
    // Performs element-wise addition of two tensors
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

pub fn tensor_contraction(
    tensor: &[Vec<Vec<f64>>],
    axis1: usize,
    axis2: usize,
) -> Result<Vec<Vec<f64>>, &'static str> {
    // Contracts a tensor along two specified axes
    if axis1 >= tensor.len() || axis2 >= tensor.len() {
        return Err("Invalid axes for contraction.");
    }

    let mut contracted = vec![vec![0.0; tensor[0][0].len()]; tensor[0].len()];

    for i in 0..tensor.len() {
        for j in 0..tensor[i].len() {
            for k in 0..tensor[i][j].len() {
                contracted[j][k] += tensor[i][j][k];
            }
        }
    }

    Ok(contracted)
}

pub fn tensor_outer_product(vec1: &[f64], vec2: &[f64]) -> Vec<Vec<f64>> {
    // Computes the outer product of two vectors
    vec1.iter()
        .map(|&val1| vec2.iter().map(|&val2| val1 * val2).collect())
        .collect()
}

pub fn tensor_trace(tensor: &[Vec<Vec<f64>>]) -> Result<f64, &'static str> {
    // Computes the trace of a 3D tensor (sum of diagonal elements)
    if tensor.len() != tensor[0].len() || tensor.len() != tensor[0][0].len() {
        return Err("Tensor must be cubic for trace calculation.");
    }

    Ok(tensor.iter().enumerate().map(|(i, mat)| mat[i][i]).sum())
}

pub fn tensor_scalar_multiply(tensor: &[Vec<Vec<f64>>], scalar: f64) -> Vec<Vec<Vec<f64>>> {
    // Multiplies every element of a tensor by a scalar
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
