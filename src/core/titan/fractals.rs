pub fn mandelbrot(c_real: f64, c_imag: f64, max_iter: usize) -> usize {
    // Determines the iteration count for the Mandelbrot set at a given complex point
    let mut z_real = 0.0;
    let mut z_imag = 0.0;
    let mut iter = 0;

    while z_real * z_real + z_imag * z_imag <= 4.0 && iter < max_iter {
        let temp = z_real * z_real - z_imag * z_imag + c_real;
        z_imag = 2.0 * z_real * z_imag + c_imag;
        z_real = temp;
        iter += 1;
    }

    iter
}

pub fn julia(z_real: f64, z_imag: f64, c_real: f64, c_imag: f64, max_iter: usize) -> usize {
    // Determines the iteration count for the Julia set at a given complex point
    let mut zr = z_real;
    let mut zi = z_imag;
    let mut iter = 0;

    while zr * zr + zi * zi <= 4.0 && iter < max_iter {
        let temp = zr * zr - zi * zi + c_real;
        zi = 2.0 * zr * zi + c_imag;
        zr = temp;
        iter += 1;
    }

    iter
}

pub fn generate_mandelbrot_set(
    width: usize,
    height: usize,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    max_iter: usize,
) -> Vec<Vec<usize>> {
    // Generates a 2D representation of the Mandelbrot set
    let mut result = vec![vec![0; width]; height];
    let dx = (x_max - x_min) / width as f64;
    let dy = (y_max - y_min) / height as f64;

    for i in 0..height {
        for j in 0..width {
            let c_real = x_min + j as f64 * dx;
            let c_imag = y_min + i as f64 * dy;
            result[i][j] = mandelbrot(c_real, c_imag, max_iter);
        }
    }

    result
}

pub fn generate_julia_set(
    width: usize,
    height: usize,
    c_real: f64,
    c_imag: f64,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    max_iter: usize,
) -> Vec<Vec<usize>> {
    // Generates a 2D representation of the Julia set
    let mut result = vec![vec![0; width]; height];
    let dx = (x_max - x_min) / width as f64;
    let dy = (y_max - y_min) / height as f64;

    for i in 0..height {
        for j in 0..width {
            let z_real = x_min + j as f64 * dx;
            let z_imag = y_min + i as f64 * dy;
            result[i][j] = julia(z_real, z_imag, c_real, c_imag, max_iter);
        }
    }

    result
}
