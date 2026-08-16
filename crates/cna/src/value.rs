/// A two-dimensional vector implemented entirely in Rust.
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2 {
    /// Horizontal component.
    pub X: f32,
    /// Vertical component.
    pub Y: f32,
}

impl Vector2 {
    /// Creates a vector from its components.
    #[must_use]
    pub const fn new(X: f32, Y: f32) -> Self {
        Self { X, Y }
    }

    pub fn Zero() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl core::ops::Add for Vector2 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.X + other.X, self.Y + other.Y)
    }
}

/// A three-dimensional vector.
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3 {
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
}

impl Vector3 {
    pub const fn new(X: f32, Y: f32, Z: f32) -> Self {
        Self { X, Y, Z }
    }
    pub fn Zero() -> Self { Self::new(0.0, 0.0, 0.0) }
    pub fn Up() -> Self { Self::new(0.0, 1.0, 0.0) }
}

/// A 4x4 matrix.
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix {
    pub M: [[f32; 4]; 4],
}

impl Default for Matrix {
    fn default() -> Self {
        Self::CreateIdentity()
    }
}

impl Matrix {
    pub fn CreateIdentity() -> Self {
        let mut M = [[0.0; 4]; 4];
        for i in 0..4 { M[i][i] = 1.0; }
        Self { M }
    }

    pub fn CreateScale(scale: f32) -> Self {
        let mut m = Self::CreateIdentity();
        m.M[0][0] = scale;
        m.M[1][1] = scale;
        m.M[2][2] = scale;
        m
    }

    pub fn CreateRotationX(_radians: f32) -> Self { Self::CreateIdentity() }
    pub fn CreateRotationY(_radians: f32) -> Self { Self::CreateIdentity() }
    pub fn CreateTranslation(_x: f32, _y: f32, _z: f32) -> Self { Self::CreateIdentity() }
    pub fn CreateLookAt(_pos: Vector3, _target: Vector3, _up: Vector3) -> Self { Self::CreateIdentity() }
    pub fn CreatePerspectiveFieldOfView(_fov: f32, _aspect: f32, _near: f32, _far: f32) -> Self { Self::CreateIdentity() }
}

impl core::ops::Mul for Matrix {
    type Output = Self;
    fn mul(self, _rhs: Self) -> Self::Output { Self::CreateIdentity() }
}

/// A non-premultiplied color with unsigned-byte RGBA channels.
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color {
    pub R: u8,
    pub G: u8,
    pub B: u8,
    pub A: u8,
}

impl Color {
    pub const CORNFLOWER_BLUE: Self = Self::new(100, 149, 237, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);

    #[must_use]
    pub const fn new(R: u8, G: u8, B: u8, A: u8) -> Self {
        Self { R, G, B, A }
    }

    pub fn from_rgba(R: u8, G: u8, B: u8, A: u8) -> Self {
        Self::new(R, G, B, A)
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, Vector2};

    #[test]
    fn vector_arithmetic_is_local() {
        let vector = (Vector2::new(2.0, 3.0) + Vector2::new(4.0, -1.0));
        assert_eq!(vector, Vector2::new(6.0, 2.0));
    }

    #[test]
    fn known_colors_match_xna_values() {
        assert_eq!(Color::CORNFLOWER_BLUE, Color::new(100, 149, 237, 255));
    }
}
