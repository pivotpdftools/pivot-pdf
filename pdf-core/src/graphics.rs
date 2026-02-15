/// RGB color for PDF graphics operations.
///
/// Each component is in the range 0.0 (none) to 1.0 (full intensity).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    /// Create a color from RGB components (each 0.0–1.0).
    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Color { r, g, b }
    }

    /// Create a grayscale color (r = g = b = level).
    pub fn gray(level: f64) -> Self {
        Color {
            r: level,
            g: level,
            b: level,
        }
    }
}
