#[derive(Debug, Clone, Copy)]
pub struct Complex {
    pub real: f64,
    pub imag: f64,
}

impl Complex {
    // Constructor
    pub fn new(real: f64, imag: f64) -> Self {
        Complex { real, imag }
    }

    // Magnitude of the complex number
    pub fn magnitude(&self) -> f64 {
        (self.real.powi(2) + self.imag.powi(2)).sqrt()
    }

    // Phase (angle) of the complex number in radians
    pub fn phase(&self) -> f64 {
        self.imag.atan2(self.real)
    }

    // Addition of two complex numbers
    pub fn add(&self, other: &Complex) -> Complex {
        Complex::new(self.real + other.real, self.imag + other.imag)
    }

    // Subtraction of two complex numbers
    pub fn subtract(&self, other: &Complex) -> Complex {
        Complex::new(self.real - other.real, self.imag - other.imag)
    }

    // Multiplication of two complex numbers
    pub fn multiply(&self, other: &Complex) -> Complex {
        Complex::new(
            self.real * other.real - self.imag * other.imag,
            self.real * other.imag + self.imag * other.real,
        )
    }

    // Division of two complex numbers
    pub fn divide(&self, other: &Complex) -> Result<Complex, &'static str> {
        let denominator = other.real.powi(2) + other.imag.powi(2);
        if denominator == 0.0 {
            return Err("Division by zero is not allowed.");
        }
        Ok(Complex::new(
            (self.real * other.real + self.imag * other.imag) / denominator,
            (self.imag * other.real - self.real * other.imag) / denominator,
        ))
    }

    // Converts to polar form (magnitude, phase)
    pub fn to_polar(&self) -> (f64, f64) {
        (self.magnitude(), self.phase())
    }

    // Creates a complex number from polar coordinates
    pub fn from_polar(magnitude: f64, phase: f64) -> Complex {
        Complex::new(magnitude * phase.cos(), magnitude * phase.sin())
    }
}
