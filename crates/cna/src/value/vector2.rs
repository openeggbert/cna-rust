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
use super::{MathHelper, Matrix, Quaternion};

/// A two-dimensional XNA value vector.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Vector2 {
    pub X: f32,
    pub Y: f32,
}

impl Vector2 {
    pub const Zero: Self = Self::from_x_and_y(0.0, 0.0);
    pub const One: Self = Self::from_x_and_y(1.0, 1.0);
    pub const UnitX: Self = Self::from_x_and_y(1.0, 0.0);
    pub const UnitY: Self = Self::from_x_and_y(0.0, 1.0);

    #[must_use]
    pub const fn new(value: f32) -> Self {
        Self { X: value, Y: value }
    }

    #[must_use]
    pub const fn from_x_and_y(x: f32, y: f32) -> Self {
        Self { X: x, Y: y }
    }

    #[must_use]
    pub fn ToString(&self) -> String {
        format!("{{X:{} Y:{}}}", self.X, self.Y)
    }

    #[must_use]
    pub fn Equals(&self, other: Self) -> bool {
        self.X == other.X && self.Y == other.Y
    }

    #[must_use]
    pub fn EqualsWithObj(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self.Equals(*other))
    }

    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        xna_f32_hash(self.X).wrapping_add(xna_f32_hash(self.Y))
    }

    #[must_use]
    pub fn Length(&self) -> f32 {
        (self.X * self.X + self.Y * self.Y).sqrt()
    }

    #[must_use]
    pub fn LengthSquared(&self) -> f32 {
        self.X * self.X + self.Y * self.Y
    }

    pub fn Normalize(&mut self) {
        let squared = self.X * self.X + self.Y * self.Y;
        let reciprocal = 1.0 / squared.sqrt();
        self.X *= reciprocal;
        self.Y *= reciprocal;
    }

    #[must_use]
    pub fn NormalizeWithValue(value: Self) -> Self {
        let squared = value.X * value.X + value.Y * value.Y;
        let reciprocal = 1.0 / squared.sqrt();
        Self::from_x_and_y(value.X * reciprocal, value.Y * reciprocal)
    }

    pub fn NormalizeWithValueAndResult(value: &mut Self, result: &mut Self) {
        *result = Self::NormalizeWithValue(*value);
    }

    #[must_use]
    pub fn Add(value1: Self, value2: Self) -> Self {
        Self::from_x_and_y(value1.X + value2.X, value1.Y + value2.Y)
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
        Self::from_x_and_y(value1.X - value2.X, value1.Y - value2.Y)
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
        Self::from_x_and_y(value1.X * value2.X, value1.Y * value2.Y)
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
        Self::from_x_and_y(value1.X * scaleFactor, value1.Y * scaleFactor)
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
        Self::from_x_and_y(value1.X / value2.X, value1.Y / value2.Y)
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
        Self::from_x_and_y(value1.X * reciprocal, value1.Y * reciprocal)
    }

    pub fn DivideWithValue1AndDividerAndResult(value1: &mut Self, divider: f32, result: &mut Self) {
        *result = Self::DivideWithValue1AndDivider(*value1, divider);
    }

    #[must_use]
    pub fn Negate(value: Self) -> Self {
        Self::from_x_and_y(-value.X, -value.Y)
    }

    pub fn NegateWithValueAndResult(value: &mut Self, result: &mut Self) {
        *result = Self::Negate(*value);
    }

    #[must_use]
    pub fn Dot(value1: Self, value2: Self) -> f32 {
        value1.X * value2.X + value1.Y * value2.Y
    }

    pub fn DotWithValue1AndValue2AndResult(value1: &mut Self, value2: &mut Self, result: &mut f32) {
        *result = Self::Dot(*value1, *value2);
    }

    #[must_use]
    pub fn Distance(value1: Self, value2: Self) -> f32 {
        let x = value1.X - value2.X;
        let y = value1.Y - value2.Y;
        (x * x + y * y).sqrt()
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
        x * x + y * y
    }

    pub fn DistanceSquaredWithValue1AndValue2AndResult(
        value1: &mut Self,
        value2: &mut Self,
        result: &mut f32,
    ) {
        *result = Self::DistanceSquared(*value1, *value2);
    }

    #[must_use]
    pub fn Min(value1: Self, value2: Self) -> Self {
        Self::from_x_and_y(
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
        Self::from_x_and_y(
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
    pub fn Lerp(value1: Self, value2: Self, amount: f32) -> Self {
        Self::from_x_and_y(
            value1.X + (value2.X - value1.X) * amount,
            value1.Y + (value2.Y - value1.Y) * amount,
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
    pub fn Clamp(value1: Self, min: Self, max: Self) -> Self {
        Self::from_x_and_y(
            MathHelper::Clamp(value1.X, min.X, max.X),
            MathHelper::Clamp(value1.Y, min.Y, max.Y),
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
    pub fn Reflect(vector: Self, normal: Self) -> Self {
        let dot = vector.X * normal.X + vector.Y * normal.Y;
        Self::from_x_and_y(
            vector.X - 2.0 * dot * normal.X,
            vector.Y - 2.0 * dot * normal.Y,
        )
    }

    pub fn ReflectWithVectorAndNormalAndResult(
        vector: &mut Self,
        normal: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Reflect(*vector, *normal);
    }

    #[must_use]
    pub fn Barycentric(
        value1: Self,
        value2: Self,
        value3: Self,
        amount1: f32,
        amount2: f32,
    ) -> Self {
        Self::from_x_and_y(
            MathHelper::Barycentric(value1.X, value2.X, value3.X, amount1, amount2),
            MathHelper::Barycentric(value1.Y, value2.Y, value3.Y, amount1, amount2),
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
        Self::from_x_and_y(
            MathHelper::CatmullRom(value1.X, value2.X, value3.X, value4.X, amount),
            MathHelper::CatmullRom(value1.Y, value2.Y, value3.Y, value4.Y, amount),
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
        Self::from_x_and_y(
            MathHelper::Hermite(value1.X, tangent1.X, value2.X, tangent2.X, amount),
            MathHelper::Hermite(value1.Y, tangent1.Y, value2.Y, tangent2.Y, amount),
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
    pub fn Transform(position: Self, matrix: Matrix) -> Self {
        Self::from_x_and_y(
            position.X * matrix.M11 + position.Y * matrix.M21 + matrix.M41,
            position.X * matrix.M12 + position.Y * matrix.M22 + matrix.M42,
        )
    }

    pub fn TransformWithPositionAndMatrixAndResult(
        position: &mut Self,
        matrix: &mut Matrix,
        result: &mut Self,
    ) {
        *result = Self::Transform(*position, *matrix);
    }

    #[must_use]
    pub fn TransformWithValueAndRotation(value: Self, rotation: Quaternion) -> Self {
        let x2 = rotation.X + rotation.X;
        let y2 = rotation.Y + rotation.Y;
        let z2 = rotation.Z + rotation.Z;
        let wz2 = rotation.W * z2;
        let xx2 = rotation.X * x2;
        let xy2 = rotation.X * y2;
        let yy2 = rotation.Y * y2;
        let zz2 = rotation.Z * z2;
        Self::from_x_and_y(
            value.X * (1.0 - yy2 - zz2) + value.Y * (xy2 - wz2),
            value.X * (xy2 + wz2) + value.Y * (1.0 - xx2 - zz2),
        )
    }

    pub fn TransformWithValueAndRotationAndResult(
        value: &mut Self,
        rotation: &mut Quaternion,
        result: &mut Self,
    ) {
        *result = Self::TransformWithValueAndRotation(*value, *rotation);
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
            *destination = Self::Transform(*source, *matrix);
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
                Self::Transform(sourceArray[sourceIndex + offset], *matrix);
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
            *destination = Self::TransformWithValueAndRotation(*source, *rotation);
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
                Self::TransformWithValueAndRotation(sourceArray[sourceIndex + offset], *rotation);
        }
    }

    #[must_use]
    pub fn TransformNormal(normal: Self, matrix: Matrix) -> Self {
        Self::from_x_and_y(
            normal.X * matrix.M11 + normal.Y * matrix.M21,
            normal.X * matrix.M12 + normal.Y * matrix.M22,
        )
    }

    pub fn TransformNormalWithNormalAndMatrixAndResult(
        normal: &mut Self,
        matrix: &mut Matrix,
        result: &mut Self,
    ) {
        *result = Self::TransformNormal(*normal, *matrix);
    }

    pub fn TransformNormalWithSourceArrayAndMatrixAndDestinationArray(
        sourceArray: &[Self],
        matrix: &mut Matrix,
        destinationArray: &mut [Self],
    ) {
        assert!(
            destinationArray.len() >= sourceArray.len(),
            "destination array is too small"
        );
        for (source, destination) in sourceArray.iter().zip(destinationArray) {
            *destination = Self::TransformNormal(*source, *matrix);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn TransformNormalWithSourceArrayAndSourceIndexAndMatrixAndDestinationArrayAndDestinationIndexAndLength(
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
                Self::TransformNormal(sourceArray[sourceIndex + offset], *matrix);
        }
    }
}

vector_ops!(
    Vector2,
    MultiplyWithValue1AndScaleFactor,
    DivideWithValue1AndDivider
);
