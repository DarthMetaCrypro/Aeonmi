use aeonmi_project::core::titan::linear_algebra::{
    matrix_multiply, transpose, determinant,
};

fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps
}

#[test]
fn matmul_2x2() {
    let a = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
    let b = vec![vec![5.0, 6.0], vec![7.0, 8.0]];
    let c = matrix_multiply(&a, &b).expect("matmul ok");
    assert_eq!(c, vec![vec![19.0, 22.0], vec![43.0, 50.0]]);
}

#[test]
fn transpose_2x3() {
    let m = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
    let t = transpose(&m);
    assert_eq!(t, vec![vec![1.0, 4.0], vec![2.0, 5.0], vec![3.0, 6.0]]);
}

#[test]
fn determinant_2x2() {
    let m = vec![vec![4.0, 6.0], vec![3.0, 8.0]];
    let d = determinant(&m).expect("det ok");
    assert!(approx_eq(d, 14.0, 1e-9));
}
