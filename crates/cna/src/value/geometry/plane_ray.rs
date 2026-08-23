#![allow(non_snake_case)]

use core::any::Any;

use crate::value::vector_support::xna_f32_hash;
use crate::value::{
    BoundingBox, BoundingFrustum, BoundingSphere, Matrix, PlaneIntersectionType, Quaternion,
    Vector3, Vector4,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Plane {
    pub Normal: Vector3,
    pub D: f32,
}

impl Plane {
    #[must_use]
    pub const fn new(value: Vector4) -> Self {
        Self {
            Normal: Vector3::from_x_and_y_and_z(value.X, value.Y, value.Z),
            D: value.W,
        }
    }

    #[must_use]
    pub const fn from_normal_and_d(normal: Vector3, d: f32) -> Self {
        Self {
            Normal: normal,
            D: d,
        }
    }

    #[must_use]
    pub fn from_point1_and_point2_and_point3(
        point1: Vector3,
        point2: Vector3,
        point3: Vector3,
    ) -> Self {
        let x1 = point2.X - point1.X;
        let y1 = point2.Y - point1.Y;
        let z1 = point2.Z - point1.Z;
        let x2 = point3.X - point1.X;
        let y2 = point3.Y - point1.Y;
        let z2 = point3.Z - point1.Z;
        let x = y1 * z2 - z1 * y2;
        let y = z1 * x2 - x1 * z2;
        let z = x1 * y2 - y1 * x2;
        let reciprocal = 1.0 / (x * x + y * y + z * z).sqrt();
        let normal = Vector3::from_x_and_y_and_z(x * reciprocal, y * reciprocal, z * reciprocal);
        Self::from_normal_and_d(
            normal,
            -(normal.X * point1.X + normal.Y * point1.Y + normal.Z * point1.Z),
        )
    }

    #[must_use]
    pub const fn from_a_and_b_and_c_and_d(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self::from_normal_and_d(Vector3::from_x_and_y_and_z(a, b, c), d)
    }

    #[must_use]
    pub fn Equals(&self, other: Self) -> bool {
        self.Normal.X == other.Normal.X
            && self.Normal.Y == other.Normal.Y
            && self.Normal.Z == other.Normal.Z
            && self.D == other.D
    }

    #[must_use]
    pub fn EqualsWithObj(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self.Equals(*other))
    }

    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        self.Normal.GetHashCode().wrapping_add(xna_f32_hash(self.D))
    }

    #[must_use]
    pub fn ToString(&self) -> String {
        format!("{{Normal:{} D:{}}}", self.Normal.ToString(), self.D)
    }

    pub fn Normalize(&mut self) {
        let squared = self.Normal.X * self.Normal.X
            + self.Normal.Y * self.Normal.Y
            + self.Normal.Z * self.Normal.Z;
        if !((squared - 1.0).abs() < 1.192_092_9e-7) {
            let reciprocal = 1.0 / squared.sqrt();
            self.Normal.X *= reciprocal;
            self.Normal.Y *= reciprocal;
            self.Normal.Z *= reciprocal;
            self.D *= reciprocal;
        }
    }

    #[must_use]
    pub fn NormalizeWithValue(value: Self) -> Self {
        let mut result = value;
        result.Normalize();
        result
    }

    pub fn NormalizeWithValueAndResult(value: &mut Self, result: &mut Self) {
        *result = Self::NormalizeWithValue(*value);
    }

    #[must_use]
    pub fn Transform(plane: Self, matrix: Matrix) -> Self {
        let inverse = Matrix::Invert(matrix);
        let x = plane.Normal.X;
        let y = plane.Normal.Y;
        let z = plane.Normal.Z;
        let d = plane.D;
        Self::from_a_and_b_and_c_and_d(
            x * inverse.M11 + y * inverse.M12 + z * inverse.M13 + d * inverse.M14,
            x * inverse.M21 + y * inverse.M22 + z * inverse.M23 + d * inverse.M24,
            x * inverse.M31 + y * inverse.M32 + z * inverse.M33 + d * inverse.M34,
            x * inverse.M41 + y * inverse.M42 + z * inverse.M43 + d * inverse.M44,
        )
    }

    pub fn TransformWithPlaneAndMatrixAndResult(
        plane: &mut Self,
        matrix: &mut Matrix,
        result: &mut Self,
    ) {
        *result = Self::Transform(*plane, *matrix);
    }

    #[must_use]
    pub fn TransformWithPlaneAndRotation(plane: Self, rotation: Quaternion) -> Self {
        Self::from_normal_and_d(
            Vector3::TransformWithValueAndRotation(plane.Normal, rotation),
            plane.D,
        )
    }

    pub fn TransformWithPlaneAndRotationAndResult(
        plane: &mut Self,
        rotation: &mut Quaternion,
        result: &mut Self,
    ) {
        *result = Self::TransformWithPlaneAndRotation(*plane, *rotation);
    }

    #[must_use]
    pub fn Dot(&self, value: Vector4) -> f32 {
        self.Normal.X * value.X
            + self.Normal.Y * value.Y
            + self.Normal.Z * value.Z
            + self.D * value.W
    }
    pub fn DotWithValueAndResult(&mut self, value: &mut Vector4, result: &mut f32) {
        *result = self.Dot(*value);
    }
    #[must_use]
    pub fn DotCoordinate(&self, value: Vector3) -> f32 {
        self.Normal.X * value.X + self.Normal.Y * value.Y + self.Normal.Z * value.Z + self.D
    }
    pub fn DotCoordinateWithValueAndResult(&mut self, value: &mut Vector3, result: &mut f32) {
        *result = self.DotCoordinate(*value);
    }
    #[must_use]
    pub fn DotNormal(&self, value: Vector3) -> f32 {
        self.Normal.X * value.X + self.Normal.Y * value.Y + self.Normal.Z * value.Z
    }
    pub fn DotNormalWithValueAndResult(&mut self, value: &mut Vector3, result: &mut f32) {
        *result = self.DotNormal(*value);
    }

    #[must_use]
    pub fn Intersects(&self, r#box: BoundingBox) -> PlaneIntersectionType {
        let near = Vector3::from_x_and_y_and_z(
            if self.Normal.X >= 0.0 {
                r#box.Min.X
            } else {
                r#box.Max.X
            },
            if self.Normal.Y >= 0.0 {
                r#box.Min.Y
            } else {
                r#box.Max.Y
            },
            if self.Normal.Z >= 0.0 {
                r#box.Min.Z
            } else {
                r#box.Max.Z
            },
        );
        if self.DotCoordinate(near) > 0.0 {
            return PlaneIntersectionType::Front;
        }
        let far = Vector3::from_x_and_y_and_z(
            if self.Normal.X >= 0.0 {
                r#box.Max.X
            } else {
                r#box.Min.X
            },
            if self.Normal.Y >= 0.0 {
                r#box.Max.Y
            } else {
                r#box.Min.Y
            },
            if self.Normal.Z >= 0.0 {
                r#box.Max.Z
            } else {
                r#box.Min.Z
            },
        );
        if self.DotCoordinate(far) < 0.0 {
            PlaneIntersectionType::Back
        } else {
            PlaneIntersectionType::Intersecting
        }
    }
    pub fn IntersectsWithBoxAndResult(
        &mut self,
        r#box: &mut BoundingBox,
        result: &mut PlaneIntersectionType,
    ) {
        *result = self.Intersects(*r#box);
    }
    #[must_use]
    pub fn IntersectsWithFrustum(&self, frustum: &BoundingFrustum) -> PlaneIntersectionType {
        frustum.IntersectsWithPlane(*self)
    }
    #[must_use]
    pub fn IntersectsWithSphere(&self, sphere: BoundingSphere) -> PlaneIntersectionType {
        let distance = self.DotCoordinate(sphere.Center);
        if distance > sphere.Radius {
            PlaneIntersectionType::Front
        } else if distance < 0.0 - sphere.Radius {
            PlaneIntersectionType::Back
        } else {
            PlaneIntersectionType::Intersecting
        }
    }
    pub fn IntersectsWithSphereAndResult(
        &mut self,
        sphere: &mut BoundingSphere,
        result: &mut PlaneIntersectionType,
    ) {
        *result = self.IntersectsWithSphere(*sphere);
    }
}

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
    #[must_use]
    pub fn Equals(&self, other: Self) -> bool {
        self.Position.Equals(other.Position) && self.Direction.Equals(other.Direction)
    }
    #[must_use]
    pub fn EqualsWithObj(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self.Equals(*other))
    }
    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        self.Position
            .GetHashCode()
            .wrapping_add(self.Direction.GetHashCode())
    }
    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "{{Position:{} Direction:{}}}",
            self.Position.ToString(),
            self.Direction.ToString()
        )
    }
    #[must_use]
    pub fn Intersects(&self, r#box: BoundingBox) -> Option<f32> {
        r#box.IntersectsWithRay(*self)
    }
    pub fn IntersectsWithBoxAndResult(
        &mut self,
        r#box: &mut BoundingBox,
        result: &mut Option<f32>,
    ) {
        *result = self.Intersects(*r#box);
    }
    #[must_use]
    pub fn IntersectsWithFrustum(&self, frustum: &BoundingFrustum) -> Option<f32> {
        frustum.IntersectsWithRay(*self)
    }
    #[must_use]
    pub fn IntersectsWithPlane(&self, plane: Plane) -> Option<f32> {
        let denominator = plane.Normal.X * self.Direction.X
            + plane.Normal.Y * self.Direction.Y
            + plane.Normal.Z * self.Direction.Z;
        if denominator.abs() < 1e-5 {
            return None;
        }
        let position_dot = plane.Normal.X * self.Position.X
            + plane.Normal.Y * self.Position.Y
            + plane.Normal.Z * self.Position.Z;
        let mut distance = (0.0 - plane.D - position_dot) / denominator;
        if distance < 0.0 {
            if distance < -1e-5 {
                return None;
            }
            distance = 0.0;
        }
        Some(distance)
    }
    pub fn IntersectsWithPlaneAndResult(&mut self, plane: &mut Plane, result: &mut Option<f32>) {
        *result = self.IntersectsWithPlane(*plane);
    }
    #[must_use]
    pub fn IntersectsWithSphere(&self, sphere: BoundingSphere) -> Option<f32> {
        let x = sphere.Center.X - self.Position.X;
        let y = sphere.Center.Y - self.Position.Y;
        let z = sphere.Center.Z - self.Position.Z;
        let distance_squared = x * x + y * y + z * z;
        let radius_squared = sphere.Radius * sphere.Radius;
        if distance_squared <= radius_squared {
            return Some(0.0);
        }
        let projection = x * self.Direction.X + y * self.Direction.Y + z * self.Direction.Z;
        if projection < 0.0 {
            return None;
        }
        let perpendicular_squared = distance_squared - projection * projection;
        if perpendicular_squared > radius_squared {
            return None;
        }
        Some(projection - (radius_squared - perpendicular_squared).sqrt())
    }
    pub fn IntersectsWithSphereAndResult(
        &mut self,
        sphere: &mut BoundingSphere,
        result: &mut Option<f32>,
    ) {
        *result = self.IntersectsWithSphere(*sphere);
    }
}
