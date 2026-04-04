/// An angle value that can be specified in either degrees or radians.
///
/// Used by [`PdfDocument::arc`] to accept angles in either unit without
/// requiring separate methods.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle(f64); // stored in radians

impl Angle {
    /// Create an angle from degrees.
    pub fn degrees(deg: f64) -> Self {
        Angle(deg.to_radians())
    }

    /// Create an angle from radians.
    pub fn radians(rad: f64) -> Self {
        Angle(rad)
    }

    /// Return the angle in radians.
    pub fn to_radians(self) -> f64 {
        self.0
    }
}

/// RGB color for PDF graphics operations.
///
/// Each component is in the range 0.0 (none) to 1.0 (full intensity).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component (0.0–1.0).
    pub r: f64,
    /// Green component (0.0–1.0).
    pub g: f64,
    /// Blue component (0.0–1.0).
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
