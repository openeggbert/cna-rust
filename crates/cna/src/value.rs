#![allow(
    non_snake_case,
    non_upper_case_globals,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::missing_panics_doc
)]

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// XNA single-precision math helpers.
pub struct MathHelper;

impl MathHelper {
    pub const E: f32 = core::f32::consts::E;
    pub const Log10E: f32 = core::f32::consts::LOG10_E;
    pub const Log2E: f32 = core::f32::consts::LOG2_E;
    pub const Pi: f32 = core::f32::consts::PI;
    pub const PiOver2: f32 = core::f32::consts::FRAC_PI_2;
    pub const PiOver4: f32 = core::f32::consts::FRAC_PI_4;
    pub const TwoPi: f32 = core::f32::consts::TAU;

    #[must_use]
    pub fn Clamp(value: f32, min: f32, max: f32) -> f32 {
        if value > max {
            max
        } else if value < min {
            min
        } else {
            value
        }
    }

    #[must_use]
    pub fn Lerp(value1: f32, value2: f32, amount: f32) -> f32 {
        value1 + (value2 - value1) * amount
    }

    #[must_use]
    pub fn SmoothStep(value1: f32, value2: f32, amount: f32) -> f32 {
        Self::Lerp(
            value1,
            value2,
            Self::Clamp(amount, 0.0, 1.0).powi(2) * (3.0 - 2.0 * Self::Clamp(amount, 0.0, 1.0)),
        )
    }

    #[must_use]
    pub fn ToDegrees(radians: f32) -> f32 {
        radians * 57.295_78
    }

    #[must_use]
    pub fn ToRadians(degrees: f32) -> f32 {
        degrees * 0.017_453_292
    }

    #[must_use]
    pub fn WrapAngle(mut angle: f32) -> f32 {
        if angle > -Self::Pi && angle <= Self::Pi {
            return angle;
        }
        angle %= Self::TwoPi;
        if angle <= -Self::Pi {
            angle += Self::TwoPi;
        } else if angle > Self::Pi {
            angle -= Self::TwoPi;
        }
        angle
    }
}

/// A two-dimensional XNA value vector.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2 {
    pub X: f32,
    pub Y: f32,
}

impl Vector2 {
    pub const Zero: Self = Self::new(0.0, 0.0);
    pub const One: Self = Self::new(1.0, 1.0);
    pub const UnitX: Self = Self::new(1.0, 0.0);
    pub const UnitY: Self = Self::new(0.0, 1.0);

    #[must_use]
    pub const fn new(X: f32, Y: f32) -> Self {
        Self { X, Y }
    }
    #[must_use]
    pub fn Length(&self) -> f32 {
        self.LengthSquared().sqrt()
    }
    #[must_use]
    pub fn LengthSquared(&self) -> f32 {
        self.X * self.X + self.Y * self.Y
    }
    pub fn Normalize(&mut self) {
        *self = Self::NormalizeWithValue(*self);
    }
    #[must_use]
    pub fn NormalizeWithValue(value: Self) -> Self {
        value / value.Length()
    }
    #[must_use]
    pub fn Add(value1: Self, value2: Self) -> Self {
        Self::new(value1.X + value2.X, value1.Y + value2.Y)
    }
    #[must_use]
    pub fn Subtract(value1: Self, value2: Self) -> Self {
        Self::new(value1.X - value2.X, value1.Y - value2.Y)
    }
    #[must_use]
    pub fn Multiply(value1: Self, value2: Self) -> Self {
        Self::new(value1.X * value2.X, value1.Y * value2.Y)
    }
    #[must_use]
    pub fn MultiplyWithValue1AndScaleFactor(value: Self, scale: f32) -> Self {
        Self::new(value.X * scale, value.Y * scale)
    }
    #[must_use]
    pub fn Divide(value1: Self, value2: Self) -> Self {
        Self::new(value1.X / value2.X, value1.Y / value2.Y)
    }
    #[must_use]
    pub fn DivideWithValue1AndDivider(value: Self, divider: f32) -> Self {
        Self::new(value.X / divider, value.Y / divider)
    }
    #[must_use]
    pub fn Negate(value: Self) -> Self {
        Self::new(-value.X, -value.Y)
    }
    #[must_use]
    pub fn Dot(value1: Self, value2: Self) -> f32 {
        value1.X * value2.X + value1.Y * value2.Y
    }
    #[must_use]
    pub fn Distance(value1: Self, value2: Self) -> f32 {
        Self::DistanceSquared(value1, value2).sqrt()
    }
    #[must_use]
    pub fn DistanceSquared(value1: Self, value2: Self) -> f32 {
        let x = value1.X - value2.X;
        let y = value1.Y - value2.Y;
        x * x + y * y
    }
    #[must_use]
    pub fn Lerp(value1: Self, value2: Self, amount: f32) -> Self {
        Self::new(
            MathHelper::Lerp(value1.X, value2.X, amount),
            MathHelper::Lerp(value1.Y, value2.Y, amount),
        )
    }
    #[must_use]
    pub fn Clamp(value: Self, min: Self, max: Self) -> Self {
        Self::new(
            MathHelper::Clamp(value.X, min.X, max.X),
            MathHelper::Clamp(value.Y, min.Y, max.Y),
        )
    }
    #[must_use]
    pub fn Reflect(vector: Self, normal: Self) -> Self {
        vector - normal * (2.0 * Self::Dot(vector, normal))
    }
    #[must_use]
    pub fn Transform(position: Self, matrix: Matrix) -> Self {
        Self::new(
            position.X * matrix.M11 + position.Y * matrix.M21 + matrix.M41,
            position.X * matrix.M12 + position.Y * matrix.M22 + matrix.M42,
        )
    }
}

macro_rules! vector_ops {
    ($type:ty, $multiply_scalar:ident, $divide_scalar:ident) => {
        impl Add for $type {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self::Add(self, rhs)
            }
        }
        impl Sub for $type {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self::Subtract(self, rhs)
            }
        }
        impl Mul for $type {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self {
                Self::Multiply(self, rhs)
            }
        }
        impl Mul<f32> for $type {
            type Output = Self;
            fn mul(self, rhs: f32) -> Self {
                Self::$multiply_scalar(self, rhs)
            }
        }
        impl Div for $type {
            type Output = Self;
            fn div(self, rhs: Self) -> Self {
                Self::Divide(self, rhs)
            }
        }
        impl Div<f32> for $type {
            type Output = Self;
            fn div(self, rhs: f32) -> Self {
                Self::$divide_scalar(self, rhs)
            }
        }
        impl Neg for $type {
            type Output = Self;
            fn neg(self) -> Self {
                Self::Negate(self)
            }
        }
        impl AddAssign for $type {
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }
        impl SubAssign for $type {
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }
        impl MulAssign<f32> for $type {
            fn mul_assign(&mut self, rhs: f32) {
                *self = *self * rhs;
            }
        }
        impl DivAssign<f32> for $type {
            fn div_assign(&mut self, rhs: f32) {
                *self = *self / rhs;
            }
        }
    };
}
vector_ops!(
    Vector2,
    MultiplyWithValue1AndScaleFactor,
    DivideWithValue1AndDivider
);

/// A three-dimensional XNA value vector.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3 {
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
}

impl Vector3 {
    pub const Zero: Self = Self::new(0.0, 0.0, 0.0);
    pub const One: Self = Self::new(1.0, 1.0, 1.0);
    pub const UnitX: Self = Self::new(1.0, 0.0, 0.0);
    pub const UnitY: Self = Self::new(0.0, 1.0, 0.0);
    pub const UnitZ: Self = Self::new(0.0, 0.0, 1.0);
    pub const Up: Self = Self::new(0.0, 1.0, 0.0);
    pub const Down: Self = Self::new(0.0, -1.0, 0.0);
    pub const Right: Self = Self::new(1.0, 0.0, 0.0);
    pub const Left: Self = Self::new(-1.0, 0.0, 0.0);
    pub const Forward: Self = Self::new(0.0, 0.0, -1.0);
    pub const Backward: Self = Self::new(0.0, 0.0, 1.0);

    #[must_use]
    pub const fn new(X: f32, Y: f32, Z: f32) -> Self {
        Self { X, Y, Z }
    }
    #[must_use]
    pub fn Length(&self) -> f32 {
        self.LengthSquared().sqrt()
    }
    #[must_use]
    pub fn LengthSquared(&self) -> f32 {
        self.X * self.X + self.Y * self.Y + self.Z * self.Z
    }
    pub fn Normalize(&mut self) {
        *self = Self::NormalizeWithValue(*self);
    }
    #[must_use]
    pub fn NormalizeWithValue(value: Self) -> Self {
        value / value.Length()
    }
    #[must_use]
    pub fn Add(a: Self, b: Self) -> Self {
        Self::new(a.X + b.X, a.Y + b.Y, a.Z + b.Z)
    }
    #[must_use]
    pub fn Subtract(a: Self, b: Self) -> Self {
        Self::new(a.X - b.X, a.Y - b.Y, a.Z - b.Z)
    }
    #[must_use]
    pub fn Multiply(a: Self, b: Self) -> Self {
        Self::new(a.X * b.X, a.Y * b.Y, a.Z * b.Z)
    }
    #[must_use]
    pub fn MultiplyWithValue1AndScaleFactor(v: Self, s: f32) -> Self {
        Self::new(v.X * s, v.Y * s, v.Z * s)
    }
    #[must_use]
    pub fn Divide(a: Self, b: Self) -> Self {
        Self::new(a.X / b.X, a.Y / b.Y, a.Z / b.Z)
    }
    #[must_use]
    pub fn DivideWithValue1AndValue2(v: Self, s: f32) -> Self {
        Self::new(v.X / s, v.Y / s, v.Z / s)
    }
    #[must_use]
    pub fn Negate(v: Self) -> Self {
        Self::new(-v.X, -v.Y, -v.Z)
    }
    #[must_use]
    pub fn Dot(a: Self, b: Self) -> f32 {
        a.X * b.X + a.Y * b.Y + a.Z * b.Z
    }
    #[must_use]
    pub fn Cross(a: Self, b: Self) -> Self {
        Self::new(
            a.Y * b.Z - a.Z * b.Y,
            a.Z * b.X - a.X * b.Z,
            a.X * b.Y - a.Y * b.X,
        )
    }
    #[must_use]
    pub fn Distance(a: Self, b: Self) -> f32 {
        Self::DistanceSquared(a, b).sqrt()
    }
    #[must_use]
    pub fn DistanceSquared(a: Self, b: Self) -> f32 {
        let d = Self::Subtract(a, b);
        d.LengthSquared()
    }
    #[must_use]
    pub fn Lerp(a: Self, b: Self, amount: f32) -> Self {
        a + (b - a) * amount
    }
    #[must_use]
    pub fn Transform(position: Self, matrix: Matrix) -> Self {
        Self::new(
            position.X * matrix.M11
                + position.Y * matrix.M21
                + position.Z * matrix.M31
                + matrix.M41,
            position.X * matrix.M12
                + position.Y * matrix.M22
                + position.Z * matrix.M32
                + matrix.M42,
            position.X * matrix.M13
                + position.Y * matrix.M23
                + position.Z * matrix.M33
                + matrix.M43,
        )
    }
}
vector_ops!(
    Vector3,
    MultiplyWithValue1AndScaleFactor,
    DivideWithValue1AndValue2
);

/// A four-dimensional XNA value vector.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector4 {
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
    pub W: f32,
}

impl Vector4 {
    pub const Zero: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const One: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const UnitX: Self = Self::new(1.0, 0.0, 0.0, 0.0);
    pub const UnitY: Self = Self::new(0.0, 1.0, 0.0, 0.0);
    pub const UnitZ: Self = Self::new(0.0, 0.0, 1.0, 0.0);
    pub const UnitW: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    #[must_use]
    pub const fn new(X: f32, Y: f32, Z: f32, W: f32) -> Self {
        Self { X, Y, Z, W }
    }
    #[must_use]
    pub fn Length(&self) -> f32 {
        self.LengthSquared().sqrt()
    }
    #[must_use]
    pub fn LengthSquared(&self) -> f32 {
        self.X * self.X + self.Y * self.Y + self.Z * self.Z + self.W * self.W
    }
    pub fn Normalize(&mut self) {
        *self = Self::NormalizeWithVector(*self);
    }
    #[must_use]
    pub fn NormalizeWithVector(value: Self) -> Self {
        value / value.Length()
    }
    #[must_use]
    pub fn Add(a: Self, b: Self) -> Self {
        Self::new(a.X + b.X, a.Y + b.Y, a.Z + b.Z, a.W + b.W)
    }
    #[must_use]
    pub fn Subtract(a: Self, b: Self) -> Self {
        Self::new(a.X - b.X, a.Y - b.Y, a.Z - b.Z, a.W - b.W)
    }
    #[must_use]
    pub fn Multiply(a: Self, b: Self) -> Self {
        Self::new(a.X * b.X, a.Y * b.Y, a.Z * b.Z, a.W * b.W)
    }
    #[must_use]
    pub fn MultiplyWithValue1AndScaleFactor(v: Self, s: f32) -> Self {
        Self::new(v.X * s, v.Y * s, v.Z * s, v.W * s)
    }
    #[must_use]
    pub fn Divide(a: Self, b: Self) -> Self {
        Self::new(a.X / b.X, a.Y / b.Y, a.Z / b.Z, a.W / b.W)
    }
    #[must_use]
    pub fn DivideWithValue1AndDivider(v: Self, s: f32) -> Self {
        Self::new(v.X / s, v.Y / s, v.Z / s, v.W / s)
    }
    #[must_use]
    pub fn Negate(v: Self) -> Self {
        Self::new(-v.X, -v.Y, -v.Z, -v.W)
    }
    #[must_use]
    pub fn Dot(a: Self, b: Self) -> f32 {
        a.X * b.X + a.Y * b.Y + a.Z * b.Z + a.W * b.W
    }
}
vector_ops!(
    Vector4,
    MultiplyWithValue1AndScaleFactor,
    DivideWithValue1AndDivider
);

/// XNA quaternion value.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Quaternion {
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
    pub W: f32,
}

impl Quaternion {
    pub const Identity: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    #[must_use]
    pub const fn new(X: f32, Y: f32, Z: f32, W: f32) -> Self {
        Self { X, Y, Z, W }
    }
    #[must_use]
    pub fn Length(&self) -> f32 {
        self.LengthSquared().sqrt()
    }
    #[must_use]
    pub fn LengthSquared(&self) -> f32 {
        self.X * self.X + self.Y * self.Y + self.Z * self.Z + self.W * self.W
    }
    pub fn Normalize(&mut self) {
        let n = 1.0 / self.Length();
        self.X *= n;
        self.Y *= n;
        self.Z *= n;
        self.W *= n;
    }
    #[must_use]
    pub fn NormalizeWithQuaternion(mut value: Self) -> Self {
        value.Normalize();
        value
    }
    #[must_use]
    pub fn Conjugate(value: Self) -> Self {
        Self::new(-value.X, -value.Y, -value.Z, value.W)
    }
    #[must_use]
    pub fn Inverse(value: Self) -> Self {
        Self::Conjugate(value) / value.LengthSquared()
    }
    #[must_use]
    pub fn CreateFromAxisAngle(axis: Vector3, angle: f32) -> Self {
        let half = angle * 0.5;
        let sin = half.sin();
        Self::new(axis.X * sin, axis.Y * sin, axis.Z * sin, half.cos())
    }
    #[must_use]
    pub fn CreateFromYawPitchRoll(yaw: f32, pitch: f32, roll: f32) -> Self {
        let (sr, cr) = (roll * 0.5).sin_cos();
        let (sp, cp) = (pitch * 0.5).sin_cos();
        let (sy, cy) = (yaw * 0.5).sin_cos();
        Self::new(
            cy * sp * cr + sy * cp * sr,
            sy * cp * cr - cy * sp * sr,
            cy * cp * sr - sy * sp * cr,
            cy * cp * cr + sy * sp * sr,
        )
    }
    #[must_use]
    pub fn Dot(a: Self, b: Self) -> f32 {
        a.X * b.X + a.Y * b.Y + a.Z * b.Z + a.W * b.W
    }
    #[must_use]
    pub fn Add(a: Self, b: Self) -> Self {
        Self::new(a.X + b.X, a.Y + b.Y, a.Z + b.Z, a.W + b.W)
    }
    #[must_use]
    pub fn Subtract(a: Self, b: Self) -> Self {
        Self::new(a.X - b.X, a.Y - b.Y, a.Z - b.Z, a.W - b.W)
    }
    #[must_use]
    pub fn Negate(v: Self) -> Self {
        Self::new(-v.X, -v.Y, -v.Z, -v.W)
    }
    #[must_use]
    pub fn Multiply(a: Self, b: Self) -> Self {
        Self::new(
            a.W * b.X + a.X * b.W + a.Y * b.Z - a.Z * b.Y,
            a.W * b.Y - a.X * b.Z + a.Y * b.W + a.Z * b.X,
            a.W * b.Z + a.X * b.Y - a.Y * b.X + a.Z * b.W,
            a.W * b.W - a.X * b.X - a.Y * b.Y - a.Z * b.Z,
        )
    }
    #[must_use]
    pub fn MultiplyWithQuaternion1AndScaleFactor(v: Self, s: f32) -> Self {
        Self::new(v.X * s, v.Y * s, v.Z * s, v.W * s)
    }
}
impl Add for Quaternion {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::Add(self, rhs)
    }
}
impl Sub for Quaternion {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::Subtract(self, rhs)
    }
}
impl Mul for Quaternion {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::Multiply(self, rhs)
    }
}
impl Mul<f32> for Quaternion {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::MultiplyWithQuaternion1AndScaleFactor(self, rhs)
    }
}
impl Div<f32> for Quaternion {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::MultiplyWithQuaternion1AndScaleFactor(self, 1.0 / rhs)
    }
}
impl Neg for Quaternion {
    type Output = Self;
    fn neg(self) -> Self {
        Self::Negate(self)
    }
}

/// Row-major XNA 4x4 matrix value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix {
    pub M11: f32,
    pub M12: f32,
    pub M13: f32,
    pub M14: f32,
    pub M21: f32,
    pub M22: f32,
    pub M23: f32,
    pub M24: f32,
    pub M31: f32,
    pub M32: f32,
    pub M33: f32,
    pub M34: f32,
    pub M41: f32,
    pub M42: f32,
    pub M43: f32,
    pub M44: f32,
}

impl Matrix {
    pub const Identity: Self = Self::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        M11: f32,
        M12: f32,
        M13: f32,
        M14: f32,
        M21: f32,
        M22: f32,
        M23: f32,
        M24: f32,
        M31: f32,
        M32: f32,
        M33: f32,
        M34: f32,
        M41: f32,
        M42: f32,
        M43: f32,
        M44: f32,
    ) -> Self {
        Self {
            M11,
            M12,
            M13,
            M14,
            M21,
            M22,
            M23,
            M24,
            M31,
            M32,
            M33,
            M34,
            M41,
            M42,
            M43,
            M44,
        }
    }
    #[must_use]
    pub fn CreateScale(xScale: f32, yScale: f32, zScale: f32) -> Self {
        Self::new(
            xScale, 0.0, 0.0, 0.0, 0.0, yScale, 0.0, 0.0, 0.0, 0.0, zScale, 0.0, 0.0, 0.0, 0.0, 1.0,
        )
    }
    #[must_use]
    pub fn CreateScaleWithScale(scale: f32) -> Self {
        Self::CreateScale(scale, scale, scale)
    }
    #[must_use]
    pub fn CreateRotationX(r: f32) -> Self {
        let (s, c) = r.sin_cos();
        Self::new(
            1.0, 0.0, 0.0, 0.0, 0.0, c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0,
        )
    }
    #[must_use]
    pub fn CreateRotationY(r: f32) -> Self {
        let (s, c) = r.sin_cos();
        Self::new(
            c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
        )
    }
    #[must_use]
    pub fn CreateRotationZ(r: f32) -> Self {
        let (s, c) = r.sin_cos();
        Self::new(
            c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        )
    }
    #[must_use]
    pub fn CreateTranslation(position: Vector3) -> Self {
        Self::CreateTranslationWithXPositionAndYPositionAndZPosition(
            position.X, position.Y, position.Z,
        )
    }
    #[must_use]
    pub fn CreateTranslationWithXPositionAndYPositionAndZPosition(
        xPosition: f32,
        yPosition: f32,
        zPosition: f32,
    ) -> Self {
        Self::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, xPosition, yPosition,
            zPosition, 1.0,
        )
    }
    #[must_use]
    pub fn CreateLookAt(position: Vector3, target: Vector3, up: Vector3) -> Self {
        let z = Vector3::NormalizeWithValue(position - target);
        let x = Vector3::NormalizeWithValue(Vector3::Cross(up, z));
        let y = Vector3::Cross(z, x);
        Self::new(
            x.X,
            y.X,
            z.X,
            0.0,
            x.Y,
            y.Y,
            z.Y,
            0.0,
            x.Z,
            y.Z,
            z.Z,
            0.0,
            -Vector3::Dot(x, position),
            -Vector3::Dot(y, position),
            -Vector3::Dot(z, position),
            1.0,
        )
    }
    #[must_use]
    pub fn CreatePerspectiveFieldOfView(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        assert!(
            fov > 0.0 && fov < core::f32::consts::PI,
            "field of view must be between zero and Pi"
        );
        assert!(
            near > 0.0 && far > 0.0 && near < far,
            "invalid clipping planes"
        );
        let y = 1.0 / (fov * 0.5).tan();
        let x = y / aspect;
        Self::new(
            x,
            0.0,
            0.0,
            0.0,
            0.0,
            y,
            0.0,
            0.0,
            0.0,
            0.0,
            far / (near - far),
            -1.0,
            0.0,
            0.0,
            near * far / (near - far),
            0.0,
        )
    }
    #[must_use]
    pub fn CreateFromQuaternion(q: Quaternion) -> Self {
        let xx = q.X * q.X;
        let yy = q.Y * q.Y;
        let zz = q.Z * q.Z;
        let xy = q.X * q.Y;
        let zw = q.Z * q.W;
        let zx = q.Z * q.X;
        let yw = q.Y * q.W;
        let yz = q.Y * q.Z;
        let xw = q.X * q.W;
        Self::new(
            1.0 - 2.0 * (yy + zz),
            2.0 * (xy + zw),
            2.0 * (zx - yw),
            0.0,
            2.0 * (xy - zw),
            1.0 - 2.0 * (zz + xx),
            2.0 * (yz + xw),
            0.0,
            2.0 * (zx + yw),
            2.0 * (yz - xw),
            1.0 - 2.0 * (yy + xx),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        )
    }
    #[must_use]
    pub fn Transpose(m: Self) -> Self {
        Self::new(
            m.M11, m.M21, m.M31, m.M41, m.M12, m.M22, m.M32, m.M42, m.M13, m.M23, m.M33, m.M43,
            m.M14, m.M24, m.M34, m.M44,
        )
    }
    #[must_use]
    pub fn Add(a: Self, b: Self) -> Self {
        Self::new(
            a.M11 + b.M11,
            a.M12 + b.M12,
            a.M13 + b.M13,
            a.M14 + b.M14,
            a.M21 + b.M21,
            a.M22 + b.M22,
            a.M23 + b.M23,
            a.M24 + b.M24,
            a.M31 + b.M31,
            a.M32 + b.M32,
            a.M33 + b.M33,
            a.M34 + b.M34,
            a.M41 + b.M41,
            a.M42 + b.M42,
            a.M43 + b.M43,
            a.M44 + b.M44,
        )
    }
    #[must_use]
    pub fn Subtract(a: Self, b: Self) -> Self {
        Self::new(
            a.M11 - b.M11,
            a.M12 - b.M12,
            a.M13 - b.M13,
            a.M14 - b.M14,
            a.M21 - b.M21,
            a.M22 - b.M22,
            a.M23 - b.M23,
            a.M24 - b.M24,
            a.M31 - b.M31,
            a.M32 - b.M32,
            a.M33 - b.M33,
            a.M34 - b.M34,
            a.M41 - b.M41,
            a.M42 - b.M42,
            a.M43 - b.M43,
            a.M44 - b.M44,
        )
    }
    #[must_use]
    pub fn Multiply(a: Self, b: Self) -> Self {
        Self::new(
            a.M11 * b.M11 + a.M12 * b.M21 + a.M13 * b.M31 + a.M14 * b.M41,
            a.M11 * b.M12 + a.M12 * b.M22 + a.M13 * b.M32 + a.M14 * b.M42,
            a.M11 * b.M13 + a.M12 * b.M23 + a.M13 * b.M33 + a.M14 * b.M43,
            a.M11 * b.M14 + a.M12 * b.M24 + a.M13 * b.M34 + a.M14 * b.M44,
            a.M21 * b.M11 + a.M22 * b.M21 + a.M23 * b.M31 + a.M24 * b.M41,
            a.M21 * b.M12 + a.M22 * b.M22 + a.M23 * b.M32 + a.M24 * b.M42,
            a.M21 * b.M13 + a.M22 * b.M23 + a.M23 * b.M33 + a.M24 * b.M43,
            a.M21 * b.M14 + a.M22 * b.M24 + a.M23 * b.M34 + a.M24 * b.M44,
            a.M31 * b.M11 + a.M32 * b.M21 + a.M33 * b.M31 + a.M34 * b.M41,
            a.M31 * b.M12 + a.M32 * b.M22 + a.M33 * b.M32 + a.M34 * b.M42,
            a.M31 * b.M13 + a.M32 * b.M23 + a.M33 * b.M33 + a.M34 * b.M43,
            a.M31 * b.M14 + a.M32 * b.M24 + a.M33 * b.M34 + a.M34 * b.M44,
            a.M41 * b.M11 + a.M42 * b.M21 + a.M43 * b.M31 + a.M44 * b.M41,
            a.M41 * b.M12 + a.M42 * b.M22 + a.M43 * b.M32 + a.M44 * b.M42,
            a.M41 * b.M13 + a.M42 * b.M23 + a.M43 * b.M33 + a.M44 * b.M43,
            a.M41 * b.M14 + a.M42 * b.M24 + a.M43 * b.M34 + a.M44 * b.M44,
        )
    }
    #[must_use]
    pub fn MultiplyWithMatrix1AndScaleFactor(m: Self, s: f32) -> Self {
        Self::new(
            m.M11 * s,
            m.M12 * s,
            m.M13 * s,
            m.M14 * s,
            m.M21 * s,
            m.M22 * s,
            m.M23 * s,
            m.M24 * s,
            m.M31 * s,
            m.M32 * s,
            m.M33 * s,
            m.M34 * s,
            m.M41 * s,
            m.M42 * s,
            m.M43 * s,
            m.M44 * s,
        )
    }
}
impl Default for Matrix {
    fn default() -> Self {
        Self::Identity
    }
}
impl Add for Matrix {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::Add(self, rhs)
    }
}
impl Sub for Matrix {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::Subtract(self, rhs)
    }
}
impl Mul for Matrix {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::Multiply(self, rhs)
    }
}
impl Mul<f32> for Matrix {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::MultiplyWithMatrix1AndScaleFactor(self, rhs)
    }
}

/// An unpacked XNA RGBA color.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}
impl Color {
    pub const Transparent: Self = Self::new(0, 0, 0, 0);
    pub const Black: Self = Self::new(0, 0, 0, 255);
    pub const White: Self = Self::new(255, 255, 255, 255);
    pub const CornflowerBlue: Self = Self::new(100, 149, 237, 255);
    #[must_use]
    pub const fn new(R: u8, G: u8, B: u8, A: u8) -> Self {
        Self {
            r: R,
            g: G,
            b: B,
            a: A,
        }
    }
    #[must_use]
    pub fn from_r_and_g_and_b_as_int32_and_int32_and_int32(R: i32, G: i32, B: i32) -> Self {
        Self::new(
            R.clamp(0, 255) as u8,
            G.clamp(0, 255) as u8,
            B.clamp(0, 255) as u8,
            255,
        )
    }
    #[must_use]
    pub const fn R(&self) -> u8 {
        self.r
    }
    pub fn SetR(&mut self, value: u8) {
        self.r = value;
    }
    #[must_use]
    pub const fn G(&self) -> u8 {
        self.g
    }
    pub fn SetG(&mut self, value: u8) {
        self.g = value;
    }
    #[must_use]
    pub const fn B(&self) -> u8 {
        self.b
    }
    pub fn SetB(&mut self, value: u8) {
        self.b = value;
    }
    #[must_use]
    pub const fn A(&self) -> u8 {
        self.a
    }
    pub fn SetA(&mut self, value: u8) {
        self.a = value;
    }
    #[must_use]
    pub fn PackedValue(&self) -> u32 {
        u32::from_le_bytes([self.r, self.g, self.b, self.a])
    }
    pub fn SetPackedValue(&mut self, value: u32) {
        let [r, g, b, a] = value.to_le_bytes();
        self.r = r;
        self.g = g;
        self.b = b;
        self.a = a;
    }
    #[must_use]
    pub fn Lerp(value1: Self, value2: Self, amount: f32) -> Self {
        let amount = MathHelper::Clamp(amount, 0.0, 1.0);
        Self::new(
            (f32::from(value1.r) + (f32::from(value2.r) - f32::from(value1.r)) * amount) as u8,
            (f32::from(value1.g) + (f32::from(value2.g) - f32::from(value1.g)) * amount) as u8,
            (f32::from(value1.b) + (f32::from(value2.b) - f32::from(value1.b)) * amount) as u8,
            (f32::from(value1.a) + (f32::from(value2.a) - f32::from(value1.a)) * amount) as u8,
        )
    }
}
impl Default for Color {
    fn default() -> Self {
        Self::Transparent
    }
}

/// XNA integer point.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Point {
    pub X: i32,
    pub Y: i32,
}
impl Point {
    pub const Zero: Self = Self::new(0, 0);
    #[must_use]
    pub const fn new(X: i32, Y: i32) -> Self {
        Self { X, Y }
    }
}

/// XNA integer rectangle.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Rectangle {
    pub X: i32,
    pub Y: i32,
    pub Width: i32,
    pub Height: i32,
}
impl Rectangle {
    pub const Empty: Self = Self::new(0, 0, 0, 0);
    #[must_use]
    pub const fn new(X: i32, Y: i32, Width: i32, Height: i32) -> Self {
        Self {
            X,
            Y,
            Width,
            Height,
        }
    }
    #[must_use]
    pub fn Left(&self) -> i32 {
        self.X
    }
    #[must_use]
    pub fn Right(&self) -> i32 {
        self.X + self.Width
    }
    #[must_use]
    pub fn Top(&self) -> i32 {
        self.Y
    }
    #[must_use]
    pub fn Bottom(&self) -> i32 {
        self.Y + self.Height
    }
    #[must_use]
    pub fn Center(&self) -> Point {
        Point::new(self.X + self.Width / 2, self.Y + self.Height / 2)
    }
    #[must_use]
    pub fn IsEmpty(&self) -> bool {
        self.Width == 0 && self.Height == 0 && self.X == 0 && self.Y == 0
    }
    #[must_use]
    pub fn Contains(&self, x: i32, y: i32) -> bool {
        x >= self.Left() && x < self.Right() && y >= self.Top() && y < self.Bottom()
    }
    #[must_use]
    pub fn Intersects(&self, other: Self) -> bool {
        other.Left() < self.Right()
            && self.Left() < other.Right()
            && other.Top() < self.Bottom()
            && self.Top() < other.Bottom()
    }
}

/// XNA plane equation `Normal · p + D = 0`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Plane {
    pub Normal: Vector3,
    pub D: f32,
}
impl Plane {
    #[must_use]
    pub const fn new(normal: Vector3, d: f32) -> Self {
        Self {
            Normal: normal,
            D: d,
        }
    }
    #[must_use]
    pub fn DotCoordinate(&self, value: Vector3) -> f32 {
        Vector3::Dot(self.Normal, value) + self.D
    }
    pub fn Normalize(&mut self) {
        let factor = 1.0 / self.Normal.Length();
        self.Normal *= factor;
        self.D *= factor;
    }
}

/// XNA ray.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ray {
    pub Position: Vector3,
    pub Direction: Vector3,
}
impl Ray {
    #[must_use]
    pub const fn new(position: Vector3, direction: Vector3) -> Self {
        Self {
            Position: position,
            Direction: direction,
        }
    }
}

/// Axis-aligned XNA bounding box.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundingBox {
    pub Min: Vector3,
    pub Max: Vector3,
}
impl BoundingBox {
    #[must_use]
    pub const fn new(min: Vector3, max: Vector3) -> Self {
        Self { Min: min, Max: max }
    }
    #[must_use]
    pub fn Contains(&self, point: Vector3) -> bool {
        point.X >= self.Min.X
            && point.X <= self.Max.X
            && point.Y >= self.Min.Y
            && point.Y <= self.Max.Y
            && point.Z >= self.Min.Z
            && point.Z <= self.Max.Z
    }
}

/// XNA bounding sphere.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundingSphere {
    pub Center: Vector3,
    pub Radius: f32,
}
impl BoundingSphere {
    #[must_use]
    pub const fn new(center: Vector3, radius: f32) -> Self {
        Self {
            Center: center,
            Radius: radius,
        }
    }
    #[must_use]
    pub fn Contains(&self, point: Vector3) -> bool {
        Vector3::DistanceSquared(point, self.Center) <= self.Radius * self.Radius
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, MathHelper, Matrix, Rectangle, Vector2, Vector3};

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 1.0e-5, "{actual} != {expected}");
    }

    #[test]
    fn named_values_match_xna() {
        assert_eq!(Vector2::Zero, Vector2::new(0.0, 0.0));
        assert_eq!(Color::CornflowerBlue, Color::new(100, 149, 237, 255));
    }
    #[test]
    fn matrix_multiplication_is_real() {
        let m = Matrix::CreateScaleWithScale(2.0)
            * Matrix::CreateTranslation(Vector3::new(3.0, 4.0, 5.0));
        let p = Vector3::Transform(Vector3::new(1.0, 2.0, 3.0), m);
        assert_eq!(p, Vector3::new(5.0, 8.0, 11.0));
    }
    #[test]
    fn rotations_follow_xna_row_vector_convention() {
        let p = Vector3::Transform(
            Vector3::Up,
            Matrix::CreateRotationX(core::f32::consts::FRAC_PI_2),
        );
        close(p.Y, 0.0);
        close(p.Z, 1.0);
    }
    #[test]
    fn perspective_has_expected_xna_terms() {
        let p = Matrix::CreatePerspectiveFieldOfView(MathHelper::PiOver2, 2.0, 1.0, 10.0);
        close(p.M11, 0.5);
        close(p.M22, 1.0);
        close(p.M34, -1.0);
        close(p.M43, -10.0 / 9.0);
    }
    #[test]
    fn rectangle_edges_are_half_open() {
        let r = Rectangle::new(2, 3, 4, 5);
        assert!(r.Contains(2, 3));
        assert!(!r.Contains(6, 8));
    }
}
