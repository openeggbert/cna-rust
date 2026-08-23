#![allow(
    non_snake_case,
    non_upper_case_globals,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::missing_panics_doc
)]

use core::any::Any;
use core::ops::{Add, Div, Mul, Neg, Sub};

use super::vector_support::xna_f32_hash;
use super::{Matrix, Vector3};

/// XNA quaternion value.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Quaternion {
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
    pub W: f32,
}

impl Quaternion {
    pub const Identity: Self = Self::from_x_and_y_and_z_and_w(0.0, 0.0, 0.0, 1.0);

    #[must_use]
    pub const fn new(vectorPart: Vector3, scalarPart: f32) -> Self {
        Self {
            X: vectorPart.X,
            Y: vectorPart.Y,
            Z: vectorPart.Z,
            W: scalarPart,
        }
    }

    #[must_use]
    pub const fn from_x_and_y_and_z_and_w(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            X: x,
            Y: y,
            Z: z,
            W: w,
        }
    }

    #[must_use]
    pub fn ToString(&self) -> String {
        format!("{{X:{} Y:{} Z:{} W:{}}}", self.X, self.Y, self.Z, self.W)
    }

    #[must_use]
    pub fn Equals(&self, other: Self) -> bool {
        self.X == other.X && self.Y == other.Y && self.Z == other.Z && self.W == other.W
    }

    #[must_use]
    pub fn EqualsWithObj(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self.Equals(*other))
    }

    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        xna_f32_hash(self.X)
            .wrapping_add(xna_f32_hash(self.Y))
            .wrapping_add(xna_f32_hash(self.Z))
            .wrapping_add(xna_f32_hash(self.W))
    }

    #[must_use]
    pub fn LengthSquared(&self) -> f32 {
        self.X * self.X + self.Y * self.Y + self.Z * self.Z + self.W * self.W
    }

    #[must_use]
    pub fn Length(&self) -> f32 {
        (self.X * self.X + self.Y * self.Y + self.Z * self.Z + self.W * self.W).sqrt()
    }

    pub fn Normalize(&mut self) {
        let reciprocal =
            1.0 / (self.X * self.X + self.Y * self.Y + self.Z * self.Z + self.W * self.W).sqrt();
        self.X *= reciprocal;
        self.Y *= reciprocal;
        self.Z *= reciprocal;
        self.W *= reciprocal;
    }

    #[must_use]
    pub fn NormalizeWithQuaternion(quaternion: Self) -> Self {
        let reciprocal = 1.0
            / (quaternion.X * quaternion.X
                + quaternion.Y * quaternion.Y
                + quaternion.Z * quaternion.Z
                + quaternion.W * quaternion.W)
                .sqrt();
        Self::from_x_and_y_and_z_and_w(
            quaternion.X * reciprocal,
            quaternion.Y * reciprocal,
            quaternion.Z * reciprocal,
            quaternion.W * reciprocal,
        )
    }

    pub fn NormalizeWithQuaternionAndResult(quaternion: &mut Self, result: &mut Self) {
        *result = Self::NormalizeWithQuaternion(*quaternion);
    }

    pub fn Conjugate(&mut self) {
        self.X = -self.X;
        self.Y = -self.Y;
        self.Z = -self.Z;
    }

    #[must_use]
    pub fn ConjugateWithValue(value: Self) -> Self {
        Self::from_x_and_y_and_z_and_w(-value.X, -value.Y, -value.Z, value.W)
    }

    pub fn ConjugateWithValueAndResult(value: &mut Self, result: &mut Self) {
        *result = Self::ConjugateWithValue(*value);
    }

    #[must_use]
    pub fn Inverse(quaternion: Self) -> Self {
        let squared = quaternion.X * quaternion.X
            + quaternion.Y * quaternion.Y
            + quaternion.Z * quaternion.Z
            + quaternion.W * quaternion.W;
        let reciprocal = 1.0 / squared;
        Self::from_x_and_y_and_z_and_w(
            -quaternion.X * reciprocal,
            -quaternion.Y * reciprocal,
            -quaternion.Z * reciprocal,
            quaternion.W * reciprocal,
        )
    }

    pub fn InverseWithQuaternionAndResult(quaternion: &mut Self, result: &mut Self) {
        *result = Self::Inverse(*quaternion);
    }

    #[must_use]
    pub fn CreateFromAxisAngle(axis: Vector3, angle: f32) -> Self {
        let half = angle * 0.5;
        let sin = f64::from(half).sin() as f32;
        let cos = f64::from(half).cos() as f32;
        Self::from_x_and_y_and_z_and_w(axis.X * sin, axis.Y * sin, axis.Z * sin, cos)
    }

    pub fn CreateFromAxisAngleWithAxisAndAngleAndResult(
        axis: &mut Vector3,
        angle: f32,
        result: &mut Self,
    ) {
        *result = Self::CreateFromAxisAngle(*axis, angle);
    }

    #[must_use]
    pub fn CreateFromYawPitchRoll(yaw: f32, pitch: f32, roll: f32) -> Self {
        let half_roll = roll * 0.5;
        let sin_roll = f64::from(half_roll).sin() as f32;
        let cos_roll = f64::from(half_roll).cos() as f32;
        let half_pitch = pitch * 0.5;
        let sin_pitch = f64::from(half_pitch).sin() as f32;
        let cos_pitch = f64::from(half_pitch).cos() as f32;
        let half_yaw = yaw * 0.5;
        let sin_yaw = f64::from(half_yaw).sin() as f32;
        let cos_yaw = f64::from(half_yaw).cos() as f32;
        Self::from_x_and_y_and_z_and_w(
            cos_yaw * sin_pitch * cos_roll + sin_yaw * cos_pitch * sin_roll,
            sin_yaw * cos_pitch * cos_roll - cos_yaw * sin_pitch * sin_roll,
            cos_yaw * cos_pitch * sin_roll - sin_yaw * sin_pitch * cos_roll,
            cos_yaw * cos_pitch * cos_roll + sin_yaw * sin_pitch * sin_roll,
        )
    }

    pub fn CreateFromYawPitchRollWithYawAndPitchAndRollAndResult(
        yaw: f32,
        pitch: f32,
        roll: f32,
        result: &mut Self,
    ) {
        *result = Self::CreateFromYawPitchRoll(yaw, pitch, roll);
    }

    #[must_use]
    pub fn CreateFromRotationMatrix(matrix: Matrix) -> Self {
        let trace = matrix.M11 + matrix.M22 + matrix.M33;
        if trace > 0.0 {
            let root = (trace + 1.0).sqrt();
            let reciprocal = 0.5 / root;
            Self::from_x_and_y_and_z_and_w(
                (matrix.M23 - matrix.M32) * reciprocal,
                (matrix.M31 - matrix.M13) * reciprocal,
                (matrix.M12 - matrix.M21) * reciprocal,
                root * 0.5,
            )
        } else if matrix.M11 >= matrix.M22 && matrix.M11 >= matrix.M33 {
            let root = (1.0 + matrix.M11 - matrix.M22 - matrix.M33).sqrt();
            let reciprocal = 0.5 / root;
            Self::from_x_and_y_and_z_and_w(
                0.5 * root,
                (matrix.M12 + matrix.M21) * reciprocal,
                (matrix.M13 + matrix.M31) * reciprocal,
                (matrix.M23 - matrix.M32) * reciprocal,
            )
        } else if matrix.M22 > matrix.M33 {
            let root = (1.0 + matrix.M22 - matrix.M11 - matrix.M33).sqrt();
            let reciprocal = 0.5 / root;
            Self::from_x_and_y_and_z_and_w(
                (matrix.M21 + matrix.M12) * reciprocal,
                0.5 * root,
                (matrix.M32 + matrix.M23) * reciprocal,
                (matrix.M31 - matrix.M13) * reciprocal,
            )
        } else {
            let root = (1.0 + matrix.M33 - matrix.M11 - matrix.M22).sqrt();
            let reciprocal = 0.5 / root;
            Self::from_x_and_y_and_z_and_w(
                (matrix.M31 + matrix.M13) * reciprocal,
                (matrix.M32 + matrix.M23) * reciprocal,
                0.5 * root,
                (matrix.M12 - matrix.M21) * reciprocal,
            )
        }
    }

    pub fn CreateFromRotationMatrixWithMatrixAndResult(matrix: &mut Matrix, result: &mut Self) {
        *result = Self::CreateFromRotationMatrix(*matrix);
    }

    #[must_use]
    pub fn Dot(quaternion1: Self, quaternion2: Self) -> f32 {
        quaternion1.X * quaternion2.X
            + quaternion1.Y * quaternion2.Y
            + quaternion1.Z * quaternion2.Z
            + quaternion1.W * quaternion2.W
    }

    pub fn DotWithQuaternion1AndQuaternion2AndResult(
        quaternion1: &mut Self,
        quaternion2: &mut Self,
        result: &mut f32,
    ) {
        *result = Self::Dot(*quaternion1, *quaternion2);
    }

    #[must_use]
    pub fn Slerp(quaternion1: Self, quaternion2: Self, amount: f32) -> Self {
        let mut dot = Self::Dot(quaternion1, quaternion2);
        let mut negate_second = false;
        if dot < 0.0 {
            negate_second = true;
            dot = -dot;
        }
        let (weight1, weight2) = if dot > 0.999_999 {
            (1.0 - amount, if negate_second { -amount } else { amount })
        } else {
            let angle = f64::from(dot).acos() as f32;
            let reciprocal_sin = (1.0 / f64::from(angle).sin()) as f32;
            let weight1 = f64::from((1.0 - amount) * angle).sin() as f32 * reciprocal_sin;
            let weight2 = f64::from(amount * angle).sin() as f32 * reciprocal_sin;
            (weight1, if negate_second { -weight2 } else { weight2 })
        };
        Self::from_x_and_y_and_z_and_w(
            weight1 * quaternion1.X + weight2 * quaternion2.X,
            weight1 * quaternion1.Y + weight2 * quaternion2.Y,
            weight1 * quaternion1.Z + weight2 * quaternion2.Z,
            weight1 * quaternion1.W + weight2 * quaternion2.W,
        )
    }

    pub fn SlerpWithQuaternion1AndQuaternion2AndAmountAndResult(
        quaternion1: &mut Self,
        quaternion2: &mut Self,
        amount: f32,
        result: &mut Self,
    ) {
        *result = Self::Slerp(*quaternion1, *quaternion2, amount);
    }

    #[must_use]
    pub fn Lerp(quaternion1: Self, quaternion2: Self, amount: f32) -> Self {
        let inverse = 1.0 - amount;
        let mut result = if Self::Dot(quaternion1, quaternion2) >= 0.0 {
            Self::from_x_and_y_and_z_and_w(
                inverse * quaternion1.X + amount * quaternion2.X,
                inverse * quaternion1.Y + amount * quaternion2.Y,
                inverse * quaternion1.Z + amount * quaternion2.Z,
                inverse * quaternion1.W + amount * quaternion2.W,
            )
        } else {
            Self::from_x_and_y_and_z_and_w(
                inverse * quaternion1.X - amount * quaternion2.X,
                inverse * quaternion1.Y - amount * quaternion2.Y,
                inverse * quaternion1.Z - amount * quaternion2.Z,
                inverse * quaternion1.W - amount * quaternion2.W,
            )
        };
        result.Normalize();
        result
    }

    pub fn LerpWithQuaternion1AndQuaternion2AndAmountAndResult(
        quaternion1: &mut Self,
        quaternion2: &mut Self,
        amount: f32,
        result: &mut Self,
    ) {
        *result = Self::Lerp(*quaternion1, *quaternion2, amount);
    }

    #[must_use]
    pub fn Concatenate(value1: Self, value2: Self) -> Self {
        Self::Multiply(value2, value1)
    }

    pub fn ConcatenateWithValue1AndValue2AndResult(
        value1: &mut Self,
        value2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Concatenate(*value1, *value2);
    }

    #[must_use]
    pub fn Negate(quaternion: Self) -> Self {
        Self::from_x_and_y_and_z_and_w(-quaternion.X, -quaternion.Y, -quaternion.Z, -quaternion.W)
    }

    pub fn NegateWithQuaternionAndResult(quaternion: &mut Self, result: &mut Self) {
        *result = Self::Negate(*quaternion);
    }

    #[must_use]
    pub fn Add(quaternion1: Self, quaternion2: Self) -> Self {
        Self::from_x_and_y_and_z_and_w(
            quaternion1.X + quaternion2.X,
            quaternion1.Y + quaternion2.Y,
            quaternion1.Z + quaternion2.Z,
            quaternion1.W + quaternion2.W,
        )
    }

    pub fn AddWithQuaternion1AndQuaternion2AndResult(
        quaternion1: &mut Self,
        quaternion2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Add(*quaternion1, *quaternion2);
    }

    #[must_use]
    pub fn Subtract(quaternion1: Self, quaternion2: Self) -> Self {
        Self::from_x_and_y_and_z_and_w(
            quaternion1.X - quaternion2.X,
            quaternion1.Y - quaternion2.Y,
            quaternion1.Z - quaternion2.Z,
            quaternion1.W - quaternion2.W,
        )
    }

    pub fn SubtractWithQuaternion1AndQuaternion2AndResult(
        quaternion1: &mut Self,
        quaternion2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Subtract(*quaternion1, *quaternion2);
    }

    #[must_use]
    pub fn Multiply(quaternion1: Self, quaternion2: Self) -> Self {
        let cross_x = quaternion1.Y * quaternion2.Z - quaternion1.Z * quaternion2.Y;
        let cross_y = quaternion1.Z * quaternion2.X - quaternion1.X * quaternion2.Z;
        let cross_z = quaternion1.X * quaternion2.Y - quaternion1.Y * quaternion2.X;
        let dot = quaternion1.X * quaternion2.X
            + quaternion1.Y * quaternion2.Y
            + quaternion1.Z * quaternion2.Z;
        Self::from_x_and_y_and_z_and_w(
            quaternion1.X * quaternion2.W + quaternion2.X * quaternion1.W + cross_x,
            quaternion1.Y * quaternion2.W + quaternion2.Y * quaternion1.W + cross_y,
            quaternion1.Z * quaternion2.W + quaternion2.Z * quaternion1.W + cross_z,
            quaternion1.W * quaternion2.W - dot,
        )
    }

    pub fn MultiplyWithQuaternion1AndQuaternion2AndResult(
        quaternion1: &mut Self,
        quaternion2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Multiply(*quaternion1, *quaternion2);
    }

    #[must_use]
    pub fn MultiplyWithQuaternion1AndScaleFactor(quaternion1: Self, scaleFactor: f32) -> Self {
        Self::from_x_and_y_and_z_and_w(
            quaternion1.X * scaleFactor,
            quaternion1.Y * scaleFactor,
            quaternion1.Z * scaleFactor,
            quaternion1.W * scaleFactor,
        )
    }

    pub fn MultiplyWithQuaternion1AndScaleFactorAndResult(
        quaternion1: &mut Self,
        scaleFactor: f32,
        result: &mut Self,
    ) {
        *result = Self::MultiplyWithQuaternion1AndScaleFactor(*quaternion1, scaleFactor);
    }

    #[must_use]
    pub fn Divide(quaternion1: Self, quaternion2: Self) -> Self {
        let squared = quaternion2.X * quaternion2.X
            + quaternion2.Y * quaternion2.Y
            + quaternion2.Z * quaternion2.Z
            + quaternion2.W * quaternion2.W;
        let reciprocal = 1.0 / squared;
        let x2 = -quaternion2.X * reciprocal;
        let y2 = -quaternion2.Y * reciprocal;
        let z2 = -quaternion2.Z * reciprocal;
        let w2 = quaternion2.W * reciprocal;
        let cross_x = quaternion1.Y * z2 - quaternion1.Z * y2;
        let cross_y = quaternion1.Z * x2 - quaternion1.X * z2;
        let cross_z = quaternion1.X * y2 - quaternion1.Y * x2;
        let dot = quaternion1.X * x2 + quaternion1.Y * y2 + quaternion1.Z * z2;
        Self::from_x_and_y_and_z_and_w(
            quaternion1.X * w2 + x2 * quaternion1.W + cross_x,
            quaternion1.Y * w2 + y2 * quaternion1.W + cross_y,
            quaternion1.Z * w2 + z2 * quaternion1.W + cross_z,
            quaternion1.W * w2 - dot,
        )
    }

    pub fn DivideWithQuaternion1AndQuaternion2AndResult(
        quaternion1: &mut Self,
        quaternion2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Divide(*quaternion1, *quaternion2);
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
impl Div for Quaternion {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self::Divide(self, rhs)
    }
}
impl Neg for Quaternion {
    type Output = Self;
    fn neg(self) -> Self {
        Self::Negate(self)
    }
}
