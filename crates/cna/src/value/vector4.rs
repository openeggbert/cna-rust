#![allow(
    non_snake_case,
    non_upper_case_globals,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::missing_panics_doc
)]

use core::any::Any;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use super::vector_support::{checked_transform_range, vector_ops, xna_f32_hash};
use super::{MathHelper, Matrix, Quaternion, Vector2, Vector3};

/// A four-dimensional XNA value vector.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Vector4 {
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
    pub W: f32,
}

impl Vector4 {
    pub const Zero: Self = Self::from_x_and_y_and_z_and_w(0.0, 0.0, 0.0, 0.0);
    pub const One: Self = Self::from_x_and_y_and_z_and_w(1.0, 1.0, 1.0, 1.0);
    pub const UnitX: Self = Self::from_x_and_y_and_z_and_w(1.0, 0.0, 0.0, 0.0);
    pub const UnitY: Self = Self::from_x_and_y_and_z_and_w(0.0, 1.0, 0.0, 0.0);
    pub const UnitZ: Self = Self::from_x_and_y_and_z_and_w(0.0, 0.0, 1.0, 0.0);
    pub const UnitW: Self = Self::from_x_and_y_and_z_and_w(0.0, 0.0, 0.0, 1.0);

    #[must_use]
    pub const fn new(value: f32) -> Self {
        Self {
            X: value,
            Y: value,
            Z: value,
            W: value,
        }
    }

    #[must_use]
    pub const fn from_value_and_w(value: Vector3, w: f32) -> Self {
        Self {
            X: value.X,
            Y: value.Y,
            Z: value.Z,
            W: w,
        }
    }

    #[must_use]
    pub const fn from_value_and_z_and_w(value: Vector2, z: f32, w: f32) -> Self {
        Self {
            X: value.X,
            Y: value.Y,
            Z: z,
            W: w,
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
    pub fn Length(&self) -> f32 {
        (self.X * self.X + self.Y * self.Y + self.Z * self.Z + self.W * self.W).sqrt()
    }

    #[must_use]
    pub fn LengthSquared(&self) -> f32 {
        self.X * self.X + self.Y * self.Y + self.Z * self.Z + self.W * self.W
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
    pub fn NormalizeWithVector(vector: Self) -> Self {
        let reciprocal = 1.0
            / (vector.X * vector.X
                + vector.Y * vector.Y
                + vector.Z * vector.Z
                + vector.W * vector.W)
                .sqrt();
        Self::from_x_and_y_and_z_and_w(
            vector.X * reciprocal,
            vector.Y * reciprocal,
            vector.Z * reciprocal,
            vector.W * reciprocal,
        )
    }

    pub fn NormalizeWithVectorAndResult(vector: &mut Self, result: &mut Self) {
        *result = Self::NormalizeWithVector(*vector);
    }

    #[must_use]
    pub fn Distance(value1: Self, value2: Self) -> f32 {
        let x = value1.X - value2.X;
        let y = value1.Y - value2.Y;
        let z = value1.Z - value2.Z;
        let w = value1.W - value2.W;
        (x * x + y * y + z * z + w * w).sqrt()
    }

    pub fn DistanceWithValue1AndValue2AndResult(
        value1: &mut Self,
        value2: &mut Self,
        result: &mut f32,
    ) {
        *result = Self::Distance(*value1, *value2);
    }

    #[must_use]
    pub fn DistanceSquared(value1: Self, value2: Self) -> f32 {
        let x = value1.X - value2.X;
        let y = value1.Y - value2.Y;
        let z = value1.Z - value2.Z;
        let w = value1.W - value2.W;
        x * x + y * y + z * z + w * w
    }

    pub fn DistanceSquaredWithValue1AndValue2AndResult(
        value1: &mut Self,
        value2: &mut Self,
        result: &mut f32,
    ) {
        *result = Self::DistanceSquared(*value1, *value2);
    }

    #[must_use]
    pub fn Dot(vector1: Self, vector2: Self) -> f32 {
        vector1.X * vector2.X
            + vector1.Y * vector2.Y
            + vector1.Z * vector2.Z
            + vector1.W * vector2.W
    }

    pub fn DotWithVector1AndVector2AndResult(
        vector1: &mut Self,
        vector2: &mut Self,
        result: &mut f32,
    ) {
        *result = Self::Dot(*vector1, *vector2);
    }

    #[must_use]
    pub fn Min(value1: Self, value2: Self) -> Self {
        Self::from_x_and_y_and_z_and_w(
            if value1.X < value2.X {
                value1.X
            } else {
                value2.X
            },
            if value1.Y < value2.Y {
                value1.Y
            } else {
                value2.Y
            },
            if value1.Z < value2.Z {
                value1.Z
            } else {
                value2.Z
            },
            if value1.W < value2.W {
                value1.W
            } else {
                value2.W
            },
        )
    }

    pub fn MinWithValue1AndValue2AndResult(
        value1: &mut Self,
        value2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Min(*value1, *value2);
    }

    #[must_use]
    pub fn Max(value1: Self, value2: Self) -> Self {
        Self::from_x_and_y_and_z_and_w(
            if value1.X > value2.X {
                value1.X
            } else {
                value2.X
            },
            if value1.Y > value2.Y {
                value1.Y
            } else {
                value2.Y
            },
            if value1.Z > value2.Z {
                value1.Z
            } else {
                value2.Z
            },
            if value1.W > value2.W {
                value1.W
            } else {
                value2.W
            },
        )
    }

    pub fn MaxWithValue1AndValue2AndResult(
        value1: &mut Self,
        value2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Max(*value1, *value2);
    }

    #[must_use]
    pub fn Clamp(value1: Self, min: Self, max: Self) -> Self {
        Self::from_x_and_y_and_z_and_w(
            MathHelper::Clamp(value1.X, min.X, max.X),
            MathHelper::Clamp(value1.Y, min.Y, max.Y),
            MathHelper::Clamp(value1.Z, min.Z, max.Z),
            MathHelper::Clamp(value1.W, min.W, max.W),
        )
    }

    pub fn ClampWithValue1AndMinAndMaxAndResult(
        value1: &mut Self,
        min: &mut Self,
        max: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Clamp(*value1, *min, *max);
    }

    #[must_use]
    pub fn Lerp(value1: Self, value2: Self, amount: f32) -> Self {
        Self::from_x_and_y_and_z_and_w(
            value1.X + (value2.X - value1.X) * amount,
            value1.Y + (value2.Y - value1.Y) * amount,
            value1.Z + (value2.Z - value1.Z) * amount,
            value1.W + (value2.W - value1.W) * amount,
        )
    }

    pub fn LerpWithValue1AndValue2AndAmountAndResult(
        value1: &mut Self,
        value2: &mut Self,
        amount: f32,
        result: &mut Self,
    ) {
        *result = Self::Lerp(*value1, *value2, amount);
    }

    #[must_use]
    pub fn Barycentric(
        value1: Self,
        value2: Self,
        value3: Self,
        amount1: f32,
        amount2: f32,
    ) -> Self {
        Self::from_x_and_y_and_z_and_w(
            MathHelper::Barycentric(value1.X, value2.X, value3.X, amount1, amount2),
            MathHelper::Barycentric(value1.Y, value2.Y, value3.Y, amount1, amount2),
            MathHelper::Barycentric(value1.Z, value2.Z, value3.Z, amount1, amount2),
            MathHelper::Barycentric(value1.W, value2.W, value3.W, amount1, amount2),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn BarycentricWithValue1AndValue2AndValue3AndAmount1AndAmount2AndResult(
        value1: &mut Self,
        value2: &mut Self,
        value3: &mut Self,
        amount1: f32,
        amount2: f32,
        result: &mut Self,
    ) {
        *result = Self::Barycentric(*value1, *value2, *value3, amount1, amount2);
    }

    #[must_use]
    pub fn SmoothStep(value1: Self, value2: Self, amount: f32) -> Self {
        let amount = MathHelper::Clamp(amount, 0.0, 1.0);
        Self::Lerp(value1, value2, amount * amount * (3.0 - 2.0 * amount))
    }

    pub fn SmoothStepWithValue1AndValue2AndAmountAndResult(
        value1: &mut Self,
        value2: &mut Self,
        amount: f32,
        result: &mut Self,
    ) {
        *result = Self::SmoothStep(*value1, *value2, amount);
    }

    #[must_use]
    pub fn CatmullRom(value1: Self, value2: Self, value3: Self, value4: Self, amount: f32) -> Self {
        Self::from_x_and_y_and_z_and_w(
            MathHelper::CatmullRom(value1.X, value2.X, value3.X, value4.X, amount),
            MathHelper::CatmullRom(value1.Y, value2.Y, value3.Y, value4.Y, amount),
            MathHelper::CatmullRom(value1.Z, value2.Z, value3.Z, value4.Z, amount),
            MathHelper::CatmullRom(value1.W, value2.W, value3.W, value4.W, amount),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn CatmullRomWithValue1AndValue2AndValue3AndValue4AndAmountAndResult(
        value1: &mut Self,
        value2: &mut Self,
        value3: &mut Self,
        value4: &mut Self,
        amount: f32,
        result: &mut Self,
    ) {
        *result = Self::CatmullRom(*value1, *value2, *value3, *value4, amount);
    }

    #[must_use]
    pub fn Hermite(
        value1: Self,
        tangent1: Self,
        value2: Self,
        tangent2: Self,
        amount: f32,
    ) -> Self {
        Self::from_x_and_y_and_z_and_w(
            MathHelper::Hermite(value1.X, tangent1.X, value2.X, tangent2.X, amount),
            MathHelper::Hermite(value1.Y, tangent1.Y, value2.Y, tangent2.Y, amount),
            MathHelper::Hermite(value1.Z, tangent1.Z, value2.Z, tangent2.Z, amount),
            MathHelper::Hermite(value1.W, tangent1.W, value2.W, tangent2.W, amount),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn HermiteWithValue1AndTangent1AndValue2AndTangent2AndAmountAndResult(
        value1: &mut Self,
        tangent1: &mut Self,
        value2: &mut Self,
        tangent2: &mut Self,
        amount: f32,
        result: &mut Self,
    ) {
        *result = Self::Hermite(*value1, *tangent1, *value2, *tangent2, amount);
    }

    #[must_use]
    pub fn Transform(position: Vector2, matrix: Matrix) -> Self {
        Self::from_x_and_y_and_z_and_w(
            position.X * matrix.M11 + position.Y * matrix.M21 + matrix.M41,
            position.X * matrix.M12 + position.Y * matrix.M22 + matrix.M42,
            position.X * matrix.M13 + position.Y * matrix.M23 + matrix.M43,
            position.X * matrix.M14 + position.Y * matrix.M24 + matrix.M44,
        )
    }

    pub fn TransformWithPositionAndMatrixAndResultAsVector2ByRefAndMatrixByRefAndVector4ByRef(
        position: &mut Vector2,
        matrix: &mut Matrix,
        result: &mut Self,
    ) {
        *result = Self::Transform(*position, *matrix);
    }

    #[must_use]
    pub fn TransformWithPositionAndMatrix(position: Vector3, matrix: Matrix) -> Self {
        Self::from_x_and_y_and_z_and_w(
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
            position.X * matrix.M14
                + position.Y * matrix.M24
                + position.Z * matrix.M34
                + matrix.M44,
        )
    }

    pub fn TransformWithPositionAndMatrixAndResultAsVector3ByRefAndMatrixByRefAndVector4ByRef(
        position: &mut Vector3,
        matrix: &mut Matrix,
        result: &mut Self,
    ) {
        *result = Self::TransformWithPositionAndMatrix(*position, *matrix);
    }

    #[must_use]
    pub fn TransformWithVectorAndMatrix(vector: Self, matrix: Matrix) -> Self {
        Self::from_x_and_y_and_z_and_w(
            vector.X * matrix.M11
                + vector.Y * matrix.M21
                + vector.Z * matrix.M31
                + vector.W * matrix.M41,
            vector.X * matrix.M12
                + vector.Y * matrix.M22
                + vector.Z * matrix.M32
                + vector.W * matrix.M42,
            vector.X * matrix.M13
                + vector.Y * matrix.M23
                + vector.Z * matrix.M33
                + vector.W * matrix.M43,
            vector.X * matrix.M14
                + vector.Y * matrix.M24
                + vector.Z * matrix.M34
                + vector.W * matrix.M44,
        )
    }

    pub fn TransformWithVectorAndMatrixAndResult(
        vector: &mut Self,
        matrix: &mut Matrix,
        result: &mut Self,
    ) {
        *result = Self::TransformWithVectorAndMatrix(*vector, *matrix);
    }

    fn rotate(x: f32, y: f32, z: f32, w: f32, rotation: Quaternion) -> Self {
        let x2 = rotation.X + rotation.X;
        let y2 = rotation.Y + rotation.Y;
        let z2 = rotation.Z + rotation.Z;
        let wx2 = rotation.W * x2;
        let wy2 = rotation.W * y2;
        let wz2 = rotation.W * z2;
        let xx2 = rotation.X * x2;
        let xy2 = rotation.X * y2;
        let xz2 = rotation.X * z2;
        let yy2 = rotation.Y * y2;
        let yz2 = rotation.Y * z2;
        let zz2 = rotation.Z * z2;
        Self::from_x_and_y_and_z_and_w(
            x * (1.0 - yy2 - zz2) + y * (xy2 - wz2) + z * (xz2 + wy2),
            x * (xy2 + wz2) + y * (1.0 - xx2 - zz2) + z * (yz2 - wx2),
            x * (xz2 - wy2) + y * (yz2 + wx2) + z * (1.0 - xx2 - yy2),
            w,
        )
    }

    #[must_use]
    pub fn TransformWithValueAndRotationAsVector2AndQuaternion(
        value: Vector2,
        rotation: Quaternion,
    ) -> Self {
        Self::rotate(value.X, value.Y, 0.0, 1.0, rotation)
    }

    pub fn TransformWithValueAndRotationAndResultAsVector2ByRefAndQuaternionByRefAndVector4ByRef(
        value: &mut Vector2,
        rotation: &mut Quaternion,
        result: &mut Self,
    ) {
        *result = Self::TransformWithValueAndRotationAsVector2AndQuaternion(*value, *rotation);
    }

    #[must_use]
    pub fn TransformWithValueAndRotationAsVector3AndQuaternion(
        value: Vector3,
        rotation: Quaternion,
    ) -> Self {
        Self::rotate(value.X, value.Y, value.Z, 1.0, rotation)
    }

    pub fn TransformWithValueAndRotationAndResultAsVector3ByRefAndQuaternionByRefAndVector4ByRef(
        value: &mut Vector3,
        rotation: &mut Quaternion,
        result: &mut Self,
    ) {
        *result = Self::TransformWithValueAndRotationAsVector3AndQuaternion(*value, *rotation);
    }

    #[must_use]
    pub fn TransformWithValueAndRotationAsVector4AndQuaternion(
        value: Self,
        rotation: Quaternion,
    ) -> Self {
        Self::rotate(value.X, value.Y, value.Z, value.W, rotation)
    }

    pub fn TransformWithValueAndRotationAndResultAsVector4ByRefAndQuaternionByRefAndVector4ByRef(
        value: &mut Self,
        rotation: &mut Quaternion,
        result: &mut Self,
    ) {
        *result = Self::TransformWithValueAndRotationAsVector4AndQuaternion(*value, *rotation);
    }

    pub fn TransformWithSourceArrayAndMatrixAndDestinationArray(
        sourceArray: &[Self],
        matrix: &mut Matrix,
        destinationArray: &mut [Self],
    ) {
        assert!(
            destinationArray.len() >= sourceArray.len(),
            "destination array is too small"
        );
        for (source, destination) in sourceArray.iter().zip(destinationArray) {
            *destination = Self::TransformWithVectorAndMatrix(*source, *matrix);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn TransformWithSourceArrayAndSourceIndexAndMatrixAndDestinationArrayAndDestinationIndexAndLength(
        sourceArray: &[Self],
        sourceIndex: i32,
        matrix: &mut Matrix,
        destinationArray: &mut [Self],
        destinationIndex: i32,
        length: i32,
    ) {
        let (sourceIndex, destinationIndex, length) = checked_transform_range(
            sourceArray.len(),
            sourceIndex,
            destinationArray.len(),
            destinationIndex,
            length,
        );
        for offset in 0..length {
            destinationArray[destinationIndex + offset] =
                Self::TransformWithVectorAndMatrix(sourceArray[sourceIndex + offset], *matrix);
        }
    }

    pub fn TransformWithSourceArrayAndRotationAndDestinationArray(
        sourceArray: &[Self],
        rotation: &mut Quaternion,
        destinationArray: &mut [Self],
    ) {
        assert!(
            destinationArray.len() >= sourceArray.len(),
            "destination array is too small"
        );
        for (source, destination) in sourceArray.iter().zip(destinationArray) {
            *destination =
                Self::TransformWithValueAndRotationAsVector4AndQuaternion(*source, *rotation);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn TransformWithSourceArrayAndSourceIndexAndRotationAndDestinationArrayAndDestinationIndexAndLength(
        sourceArray: &[Self],
        sourceIndex: i32,
        rotation: &mut Quaternion,
        destinationArray: &mut [Self],
        destinationIndex: i32,
        length: i32,
    ) {
        let (sourceIndex, destinationIndex, length) = checked_transform_range(
            sourceArray.len(),
            sourceIndex,
            destinationArray.len(),
            destinationIndex,
            length,
        );
        for offset in 0..length {
            destinationArray[destinationIndex + offset] =
                Self::TransformWithValueAndRotationAsVector4AndQuaternion(
                    sourceArray[sourceIndex + offset],
                    *rotation,
                );
        }
    }

    #[must_use]
    pub fn Negate(value: Self) -> Self {
        Self::from_x_and_y_and_z_and_w(-value.X, -value.Y, -value.Z, -value.W)
    }

    pub fn NegateWithValueAndResult(value: &mut Self, result: &mut Self) {
        *result = Self::Negate(*value);
    }

    #[must_use]
    pub fn Add(value1: Self, value2: Self) -> Self {
        Self::from_x_and_y_and_z_and_w(
            value1.X + value2.X,
            value1.Y + value2.Y,
            value1.Z + value2.Z,
            value1.W + value2.W,
        )
    }

    pub fn AddWithValue1AndValue2AndResult(
        value1: &mut Self,
        value2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Add(*value1, *value2);
    }

    #[must_use]
    pub fn Subtract(value1: Self, value2: Self) -> Self {
        Self::from_x_and_y_and_z_and_w(
            value1.X - value2.X,
            value1.Y - value2.Y,
            value1.Z - value2.Z,
            value1.W - value2.W,
        )
    }

    pub fn SubtractWithValue1AndValue2AndResult(
        value1: &mut Self,
        value2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Subtract(*value1, *value2);
    }

    #[must_use]
    pub fn Multiply(value1: Self, value2: Self) -> Self {
        Self::from_x_and_y_and_z_and_w(
            value1.X * value2.X,
            value1.Y * value2.Y,
            value1.Z * value2.Z,
            value1.W * value2.W,
        )
    }

    pub fn MultiplyWithValue1AndValue2AndResult(
        value1: &mut Self,
        value2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Multiply(*value1, *value2);
    }

    #[must_use]
    pub fn MultiplyWithValue1AndScaleFactor(value1: Self, scaleFactor: f32) -> Self {
        Self::from_x_and_y_and_z_and_w(
            value1.X * scaleFactor,
            value1.Y * scaleFactor,
            value1.Z * scaleFactor,
            value1.W * scaleFactor,
        )
    }

    pub fn MultiplyWithValue1AndScaleFactorAndResult(
        value1: &mut Self,
        scaleFactor: f32,
        result: &mut Self,
    ) {
        *result = Self::MultiplyWithValue1AndScaleFactor(*value1, scaleFactor);
    }

    #[must_use]
    pub fn Divide(value1: Self, value2: Self) -> Self {
        Self::from_x_and_y_and_z_and_w(
            value1.X / value2.X,
            value1.Y / value2.Y,
            value1.Z / value2.Z,
            value1.W / value2.W,
        )
    }

    pub fn DivideWithValue1AndValue2AndResult(
        value1: &mut Self,
        value2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Divide(*value1, *value2);
    }

    #[must_use]
    pub fn DivideWithValue1AndDivider(value1: Self, divider: f32) -> Self {
        let reciprocal = 1.0 / divider;
        Self::from_x_and_y_and_z_and_w(
            value1.X * reciprocal,
            value1.Y * reciprocal,
            value1.Z * reciprocal,
            value1.W * reciprocal,
        )
    }

    pub fn DivideWithValue1AndDividerAndResult(value1: &mut Self, divider: f32, result: &mut Self) {
        *result = Self::DivideWithValue1AndDivider(*value1, divider);
    }
}

vector_ops!(
    Vector4,
    MultiplyWithValue1AndScaleFactor,
    DivideWithValue1AndDivider
);
