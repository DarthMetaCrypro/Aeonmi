use std::f64::consts::PI;

pub fn area_circle(radius: f64) -> f64 {
    // Calculates the area of a circle given its radius
    if radius < 0.0 {
        panic!("Radius cannot be negative.");
    }
    PI * radius.powi(2)
}

pub fn perimeter_circle(radius: f64) -> f64 {
    // Calculates the perimeter (circumference) of a circle given its radius
    if radius < 0.0 {
        panic!("Radius cannot be negative.");
    }
    2.0 * PI * radius
}

pub fn area_rectangle(length: f64, width: f64) -> f64 {
    // Calculates the area of a rectangle
    if length < 0.0 || width < 0.0 {
        panic!("Length and width cannot be negative.");
    }
    length * width
}

pub fn perimeter_rectangle(length: f64, width: f64) -> f64 {
    // Calculates the perimeter of a rectangle
    if length < 0.0 || width < 0.0 {
        panic!("Length and width cannot be negative.");
    }
    2.0 * (length + width)
}

pub fn area_triangle(base: f64, height: f64) -> f64 {
    // Calculates the area of a triangle
    if base < 0.0 || height < 0.0 {
        panic!("Base and height cannot be negative.");
    }
    0.5 * base * height
}

pub fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    // Calculates the distance between two points (x1, y1) and (x2, y2)
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}

pub fn midpoint(x1: f64, y1: f64, x2: f64, y2: f64) -> (f64, f64) {
    // Calculates the midpoint of two points (x1, y1) and (x2, y2)
    ((x1 + x2) / 2.0, (y1 + y2) / 2.0)
}
