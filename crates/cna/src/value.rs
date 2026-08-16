/// A two-dimensional vector implemented entirely in Rust.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2 {
    /// Horizontal component.
    pub x: f32,
    /// Vertical component.
    pub y: f32,
}

impl Vector2 {
    /// Creates a vector from its components.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns this vector multiplied by `scale`.
    #[must_use]
    pub fn scale(self, scale: f32) -> Self {
        Self::new(self.x * scale, self.y * scale)
    }

    /// Returns the squared Euclidean length.
    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }
}

impl core::ops::Add for Vector2 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

/// A non-premultiplied color with unsigned-byte RGBA channels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color {
    /// Red channel.
    pub red: u8,
    /// Green channel.
    pub green: u8,
    /// Blue channel.
    pub blue: u8,
    /// Alpha channel.
    pub alpha: u8,
}

impl Color {
    /// The traditional XNA clear color.
    pub const CORNFLOWER_BLUE: Self = Self::new(100, 149, 237, 255);
    /// Opaque white.
    pub const WHITE: Self = Self::new(255, 255, 255, 255);

    /// Creates an RGBA color.
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, Vector2};

    #[test]
    fn vector_arithmetic_is_local() {
        let vector = (Vector2::new(2.0, 3.0) + Vector2::new(4.0, -1.0)).scale(2.0);
        assert_eq!(vector, Vector2::new(12.0, 4.0));
        assert_eq!(Vector2::new(3.0, 4.0).length_squared(), 25.0);
    }

    #[test]
    fn known_colors_match_xna_values() {
        assert_eq!(Color::CORNFLOWER_BLUE, Color::new(100, 149, 237, 255));
    }
}
