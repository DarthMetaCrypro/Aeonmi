//! Standard quantum gates and builders (feature: `quantum`).
use nalgebra::DMatrix;
use num_complex::Complex64 as C64;

#[inline]
fn c(r: f64, i: f64) -> C64 {
    C64::new(r, i)
}

pub fn i2() -> DMatrix<C64> {
    DMatrix::identity(2, 2)
}
pub fn x() -> DMatrix<C64> {
    DMatrix::from_row_slice(2, 2, &[c(0.0, 0.0), c(1.0, 0.0), c(1.0, 0.0), c(0.0, 0.0)])
}
pub fn y() -> DMatrix<C64> {
    DMatrix::from_row_slice(2, 2, &[c(0.0, 0.0), c(0.0, -1.0), c(0.0, 1.0), c(0.0, 0.0)])
}
pub fn z() -> DMatrix<C64> {
    DMatrix::from_row_slice(2, 2, &[c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(-1.0, 0.0)])
}
pub fn h() -> DMatrix<C64> {
    let s = 1.0_f64 / 2.0_f64.sqrt();
    DMatrix::from_row_slice(2, 2, &[c(s, 0.0), c(s, 0.0), c(s, 0.0), c(-s, 0.0)])
}
pub fn s() -> DMatrix<C64> {
    DMatrix::from_row_slice(2, 2, &[c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(0.0, 1.0)])
}
pub fn t() -> DMatrix<C64> {
    let phi = std::f64::consts::FRAC_PI_4;
    DMatrix::from_row_slice(
        2,
        2,
        &[
            c(1.0, 0.0),
            c(0.0, 0.0),
            c(0.0, 0.0),
            c(phi.cos(), phi.sin()),
        ],
    )
}
pub fn rx(theta: f64) -> DMatrix<C64> {
    let (c0, s0) = ((theta / 2.0).cos(), (theta / 2.0).sin());
    DMatrix::from_row_slice(2, 2, &[c(c0, 0.0), c(0.0, -s0), c(0.0, -s0), c(c0, 0.0)])
}
pub fn ry(theta: f64) -> DMatrix<C64> {
    let (c0, s0) = ((theta / 2.0).cos(), (theta / 2.0).sin());
    DMatrix::from_row_slice(2, 2, &[c(c0, 0.0), c(-s0, 0.0), c(s0, 0.0), c(c0, 0.0)])
}
pub fn rz(theta: f64) -> DMatrix<C64> {
    let e_m = C64::from_polar(1.0, -theta / 2.0);
    let e_p = C64::from_polar(1.0, theta / 2.0);
    DMatrix::from_row_slice(2, 2, &[e_m, c(0.0, 0.0), c(0.0, 0.0), e_p])
}
