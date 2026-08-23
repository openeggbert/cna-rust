#![allow(non_snake_case, non_upper_case_globals, clippy::missing_panics_doc)]

use core::any::Any;

use crate::value::vector_support::xna_f32_hash;
use crate::value::{
    BoundingFrustum, ContainmentType, MathHelper, Matrix, Plane, PlaneIntersectionType, Ray,
    Vector3,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundingBox {
    pub Min: Vector3,
    pub Max: Vector3,
}

impl BoundingBox {
    pub const CornerCount: i32 = 8;

    #[must_use]
    pub const fn new(min: Vector3, max: Vector3) -> Self {
        Self { Min: min, Max: max }
    }

    #[must_use]
    pub fn GetCorners(&self) -> Vec<Vector3> {
        vec![
            Vector3::from_x_and_y_and_z(self.Min.X, self.Max.Y, self.Max.Z),
            Vector3::from_x_and_y_and_z(self.Max.X, self.Max.Y, self.Max.Z),
            Vector3::from_x_and_y_and_z(self.Max.X, self.Min.Y, self.Max.Z),
            Vector3::from_x_and_y_and_z(self.Min.X, self.Min.Y, self.Max.Z),
            Vector3::from_x_and_y_and_z(self.Min.X, self.Max.Y, self.Min.Z),
            Vector3::from_x_and_y_and_z(self.Max.X, self.Max.Y, self.Min.Z),
            Vector3::from_x_and_y_and_z(self.Max.X, self.Min.Y, self.Min.Z),
            Vector3::from_x_and_y_and_z(self.Min.X, self.Min.Y, self.Min.Z),
        ]
    }

    pub fn GetCornersWithCorners(&mut self, corners: &mut [Vector3]) {
        assert!(corners.len() >= 8, "at least eight corners are required");
        corners[..8].copy_from_slice(&self.GetCorners());
    }

    #[must_use]
    pub fn Equals(&self, other: Self) -> bool {
        self.Min.Equals(other.Min) && self.Max.Equals(other.Max)
    }
    #[must_use]
    pub fn EqualsWithObj(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self.Equals(*other))
    }
    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        self.Min.GetHashCode().wrapping_add(self.Max.GetHashCode())
    }
    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "{{Min:{} Max:{}}}",
            self.Min.ToString(),
            self.Max.ToString()
        )
    }

    #[must_use]
    pub fn CreateMerged(original: Self, additional: Self) -> Self {
        Self::new(
            Vector3::Min(original.Min, additional.Min),
            Vector3::Max(original.Max, additional.Max),
        )
    }
    pub fn CreateMergedWithOriginalAndAdditionalAndResult(
        original: &mut Self,
        additional: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::CreateMerged(*original, *additional);
    }

    #[must_use]
    pub fn CreateFromSphere(sphere: BoundingSphere) -> Self {
        let radius = Vector3::new(sphere.Radius);
        Self::new(sphere.Center - radius, sphere.Center + radius)
    }
    pub fn CreateFromSphereWithSphereAndResult(sphere: &mut BoundingSphere, result: &mut Self) {
        *result = Self::CreateFromSphere(*sphere);
    }

    #[must_use]
    pub fn CreateFromPoints(points: &[Vector3]) -> Self {
        assert!(!points.is_empty(), "at least one point is required");
        let mut minimum = Vector3::new(f32::MAX);
        let mut maximum = Vector3::new(f32::MIN);
        for point in points {
            minimum = Vector3::Min(minimum, *point);
            maximum = Vector3::Max(maximum, *point);
        }
        Self::new(minimum, maximum)
    }

    #[must_use]
    pub fn Intersects(&self, r#box: Self) -> bool {
        !(self.Max.X < r#box.Min.X
            || self.Min.X > r#box.Max.X
            || self.Max.Y < r#box.Min.Y
            || self.Min.Y > r#box.Max.Y
            || self.Max.Z < r#box.Min.Z
            || self.Min.Z > r#box.Max.Z)
    }
    pub fn IntersectsWithBoxAndResult(&mut self, r#box: &mut Self, result: &mut bool) {
        *result = self.Intersects(*r#box);
    }
    #[must_use]
    pub fn IntersectsWithFrustum(&self, frustum: &BoundingFrustum) -> bool {
        frustum.Intersects(*self)
    }
    #[must_use]
    pub fn IntersectsWithPlane(&self, plane: Plane) -> PlaneIntersectionType {
        plane.Intersects(*self)
    }
    pub fn IntersectsWithPlaneAndResult(
        &mut self,
        plane: &mut Plane,
        result: &mut PlaneIntersectionType,
    ) {
        *result = self.IntersectsWithPlane(*plane);
    }

    #[must_use]
    pub fn IntersectsWithRay(&self, ray: Ray) -> Option<f32> {
        let mut minimum = 0.0;
        let mut maximum = f32::MAX;
        for (position, direction, low, high) in [
            (ray.Position.X, ray.Direction.X, self.Min.X, self.Max.X),
            (ray.Position.Y, ray.Direction.Y, self.Min.Y, self.Max.Y),
            (ray.Position.Z, ray.Direction.Z, self.Min.Z, self.Max.Z),
        ] {
            if direction.abs() < 1e-6 {
                if position < low || position > high {
                    return None;
                }
            } else {
                let reciprocal = 1.0 / direction;
                let mut near = (low - position) * reciprocal;
                let mut far = (high - position) * reciprocal;
                if near > far {
                    core::mem::swap(&mut near, &mut far);
                }
                minimum = MathHelper::Max(near, minimum);
                maximum = MathHelper::Min(far, maximum);
                if minimum > maximum {
                    return None;
                }
            }
        }
        Some(minimum)
    }
    pub fn IntersectsWithRayAndResult(&mut self, ray: &mut Ray, result: &mut Option<f32>) {
        *result = self.IntersectsWithRay(*ray);
    }

    #[must_use]
    pub fn IntersectsWithSphere(&self, sphere: BoundingSphere) -> bool {
        let closest = Vector3::Clamp(sphere.Center, self.Min, self.Max);
        !(Vector3::DistanceSquared(sphere.Center, closest) > sphere.Radius * sphere.Radius)
    }
    pub fn IntersectsWithSphereAndResult(
        &mut self,
        sphere: &mut BoundingSphere,
        result: &mut bool,
    ) {
        *result = self.IntersectsWithSphere(*sphere);
    }

    #[must_use]
    pub fn Contains(&self, r#box: Self) -> ContainmentType {
        if !self.Intersects(r#box) {
            return ContainmentType::Disjoint;
        }
        if self.Min.X <= r#box.Min.X
            && r#box.Max.X <= self.Max.X
            && self.Min.Y <= r#box.Min.Y
            && r#box.Max.Y <= self.Max.Y
            && self.Min.Z <= r#box.Min.Z
            && r#box.Max.Z <= self.Max.Z
        {
            ContainmentType::Contains
        } else {
            ContainmentType::Intersects
        }
    }
    pub fn ContainsWithBoxAndResult(&mut self, r#box: &mut Self, result: &mut ContainmentType) {
        *result = self.Contains(*r#box);
    }
    #[must_use]
    pub fn ContainsWithFrustum(&self, frustum: &BoundingFrustum) -> ContainmentType {
        if !frustum.Intersects(*self) {
            return ContainmentType::Disjoint;
        }
        if frustum
            .corners()
            .iter()
            .any(|corner| self.ContainsWithPoint(*corner) == ContainmentType::Disjoint)
        {
            ContainmentType::Intersects
        } else {
            ContainmentType::Contains
        }
    }
    #[must_use]
    pub fn ContainsWithPoint(&self, point: Vector3) -> ContainmentType {
        if self.Min.X <= point.X
            && point.X <= self.Max.X
            && self.Min.Y <= point.Y
            && point.Y <= self.Max.Y
            && self.Min.Z <= point.Z
            && point.Z <= self.Max.Z
        {
            ContainmentType::Contains
        } else {
            ContainmentType::Disjoint
        }
    }
    pub fn ContainsWithPointAndResult(
        &mut self,
        point: &mut Vector3,
        result: &mut ContainmentType,
    ) {
        *result = self.ContainsWithPoint(*point);
    }
    #[must_use]
    pub fn ContainsWithSphere(&self, sphere: BoundingSphere) -> ContainmentType {
        let closest = Vector3::Clamp(sphere.Center, self.Min, self.Max);
        let radius = sphere.Radius;
        if Vector3::DistanceSquared(sphere.Center, closest) > radius * radius {
            return ContainmentType::Disjoint;
        }
        if self.Min.X + radius <= sphere.Center.X
            && sphere.Center.X <= self.Max.X - radius
            && self.Max.X - self.Min.X > radius
            && self.Min.Y + radius <= sphere.Center.Y
            && sphere.Center.Y <= self.Max.Y - radius
            && self.Max.Y - self.Min.Y > radius
            && self.Min.Z + radius <= sphere.Center.Z
            && sphere.Center.Z <= self.Max.Z - radius
            && self.Max.X - self.Min.X > radius
        {
            ContainmentType::Contains
        } else {
            ContainmentType::Intersects
        }
    }
    pub fn ContainsWithSphereAndResult(
        &mut self,
        sphere: &mut BoundingSphere,
        result: &mut ContainmentType,
    ) {
        *result = self.ContainsWithSphere(*sphere);
    }

    pub(super) fn support_mapping(&self, direction: Vector3) -> Vector3 {
        Vector3::from_x_and_y_and_z(
            if direction.X >= 0.0 {
                self.Max.X
            } else {
                self.Min.X
            },
            if direction.Y >= 0.0 {
                self.Max.Y
            } else {
                self.Min.Y
            },
            if direction.Z >= 0.0 {
                self.Max.Z
            } else {
                self.Min.Z
            },
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundingSphere {
    pub Center: Vector3,
    pub Radius: f32,
}

impl BoundingSphere {
    #[must_use]
    pub fn new(center: Vector3, radius: f32) -> Self {
        assert!(!(radius < 0.0), "radius must be nonnegative");
        Self {
            Center: center,
            Radius: radius,
        }
    }

    #[must_use]
    pub fn Transform(&self, matrix: Matrix) -> Self {
        let center = Vector3::Transform(self.Center, matrix);
        let x = matrix.M11 * matrix.M11 + matrix.M12 * matrix.M12 + matrix.M13 * matrix.M13;
        let y = matrix.M21 * matrix.M21 + matrix.M22 * matrix.M22 + matrix.M23 * matrix.M23;
        let z = matrix.M31 * matrix.M31 + matrix.M32 * matrix.M32 + matrix.M33 * matrix.M33;
        let maximum = if x.is_nan() || y.is_nan() || z.is_nan() {
            f32::NAN
        } else if x > y {
            if x > z {
                x
            } else {
                z
            }
        } else if y > z {
            y
        } else {
            z
        };
        Self {
            Center: center,
            Radius: self.Radius * maximum.sqrt(),
        }
    }
    pub fn TransformWithMatrixAndResult(&mut self, matrix: &mut Matrix, result: &mut Self) {
        *result = self.Transform(*matrix);
    }

    #[must_use]
    pub fn Equals(&self, other: Self) -> bool {
        self.Center.Equals(other.Center) && self.Radius == other.Radius
    }
    #[must_use]
    pub fn EqualsWithObj(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self.Equals(*other))
    }
    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        self.Center
            .GetHashCode()
            .wrapping_add(xna_f32_hash(self.Radius))
    }
    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "{{Center:{} Radius:{}}}",
            self.Center.ToString(),
            self.Radius
        )
    }

    #[must_use]
    pub fn CreateFromBoundingBox(r#box: BoundingBox) -> Self {
        let center = Vector3::Lerp(r#box.Min, r#box.Max, 0.5);
        Self {
            Center: center,
            Radius: Vector3::Distance(r#box.Min, r#box.Max) * 0.5,
        }
    }
    pub fn CreateFromBoundingBoxWithBoxAndResult(r#box: &mut BoundingBox, result: &mut Self) {
        *result = Self::CreateFromBoundingBox(*r#box);
    }

    #[must_use]
    pub fn CreateFromPoints(points: &[Vector3]) -> Self {
        assert!(!points.is_empty(), "at least one point is required");
        let mut minimum_x = points[0];
        let mut maximum_x = points[0];
        let mut minimum_y = points[0];
        let mut maximum_y = points[0];
        let mut minimum_z = points[0];
        let mut maximum_z = points[0];
        for point in points {
            if point.X < minimum_x.X {
                minimum_x = *point;
            }
            if point.X > maximum_x.X {
                maximum_x = *point;
            }
            if point.Y < minimum_y.Y {
                minimum_y = *point;
            }
            if point.Y > maximum_y.Y {
                maximum_y = *point;
            }
            if point.Z < minimum_z.Z {
                minimum_z = *point;
            }
            if point.Z > maximum_z.Z {
                maximum_z = *point;
            }
        }
        let x_distance = Vector3::Distance(maximum_x, minimum_x);
        let y_distance = Vector3::Distance(maximum_y, minimum_y);
        let z_distance = Vector3::Distance(maximum_z, minimum_z);
        let (mut center, mut radius) = if x_distance > y_distance {
            if x_distance > z_distance {
                (Vector3::Lerp(maximum_x, minimum_x, 0.5), x_distance * 0.5)
            } else {
                (Vector3::Lerp(maximum_z, minimum_z, 0.5), z_distance * 0.5)
            }
        } else if y_distance > z_distance {
            (Vector3::Lerp(maximum_y, minimum_y, 0.5), y_distance * 0.5)
        } else {
            (Vector3::Lerp(maximum_z, minimum_z, 0.5), z_distance * 0.5)
        };
        for point in points {
            let difference = *point - center;
            let distance = difference.Length();
            if distance > radius {
                radius = (radius + distance) * 0.5;
                center += difference * (1.0 - radius / distance);
            }
        }
        Self {
            Center: center,
            Radius: radius,
        }
    }

    #[must_use]
    pub fn CreateFromFrustum(frustum: &BoundingFrustum) -> Self {
        Self::CreateFromPoints(frustum.corners())
    }

    #[must_use]
    pub fn CreateMerged(original: Self, additional: Self) -> Self {
        let difference = additional.Center - original.Center;
        let distance = difference.Length();
        if original.Radius + additional.Radius >= distance {
            if original.Radius - additional.Radius >= distance {
                return original;
            }
            if additional.Radius - original.Radius >= distance {
                return additional;
            }
        }
        let direction = difference / distance;
        let minimum = MathHelper::Min(0.0 - original.Radius, distance - additional.Radius);
        let maximum = MathHelper::Max(original.Radius, distance + additional.Radius);
        let radius = (maximum - minimum) * 0.5;
        Self {
            Center: original.Center + direction * (radius + minimum),
            Radius: radius,
        }
    }
    pub fn CreateMergedWithOriginalAndAdditionalAndResult(
        original: &mut Self,
        additional: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::CreateMerged(*original, *additional);
    }

    #[must_use]
    pub fn Intersects(&self, r#box: BoundingBox) -> bool {
        r#box.IntersectsWithSphere(*self)
    }
    pub fn IntersectsWithBoxAndResult(&mut self, r#box: &mut BoundingBox, result: &mut bool) {
        *result = self.Intersects(*r#box);
    }
    #[must_use]
    pub fn IntersectsWithFrustum(&self, frustum: &BoundingFrustum) -> bool {
        frustum.IntersectsWithSphere(*self)
    }
    #[must_use]
    pub fn IntersectsWithSphere(&self, sphere: Self) -> bool {
        let distance_squared = Vector3::DistanceSquared(self.Center, sphere.Center);
        let radius = self.Radius;
        let other_radius = sphere.Radius;
        radius * radius + 2.0 * radius * other_radius + other_radius * other_radius
            > distance_squared
    }
    pub fn IntersectsWithSphereAndResult(&mut self, sphere: &mut Self, result: &mut bool) {
        *result = self.IntersectsWithSphere(*sphere);
    }
    #[must_use]
    pub fn IntersectsWithPlane(&self, plane: Plane) -> PlaneIntersectionType {
        plane.IntersectsWithSphere(*self)
    }
    pub fn IntersectsWithPlaneAndResult(
        &mut self,
        plane: &mut Plane,
        result: &mut PlaneIntersectionType,
    ) {
        *result = self.IntersectsWithPlane(*plane);
    }
    #[must_use]
    pub fn IntersectsWithRay(&self, ray: Ray) -> Option<f32> {
        ray.IntersectsWithSphere(*self)
    }
    pub fn IntersectsWithRayAndResult(&mut self, ray: &mut Ray, result: &mut Option<f32>) {
        *result = self.IntersectsWithRay(*ray);
    }

    #[must_use]
    pub fn Contains(&self, r#box: BoundingBox) -> ContainmentType {
        if !r#box.IntersectsWithSphere(*self) {
            ContainmentType::Disjoint
        } else {
            let radius_squared = self.Radius * self.Radius;
            if r#box
                .GetCorners()
                .iter()
                .all(|corner| (*corner - self.Center).LengthSquared() <= radius_squared)
            {
                ContainmentType::Contains
            } else {
                ContainmentType::Intersects
            }
        }
    }
    pub fn ContainsWithBoxAndResult(
        &mut self,
        r#box: &mut BoundingBox,
        result: &mut ContainmentType,
    ) {
        *result = self.Contains(*r#box);
    }
    #[must_use]
    pub fn ContainsWithFrustum(&self, frustum: &BoundingFrustum) -> ContainmentType {
        if !frustum.IntersectsWithSphere(*self) {
            return ContainmentType::Disjoint;
        }
        let radius_squared = self.Radius * self.Radius;
        if frustum
            .corners()
            .iter()
            .any(|corner| (*corner - self.Center).LengthSquared() > radius_squared)
        {
            ContainmentType::Intersects
        } else {
            ContainmentType::Contains
        }
    }
    #[must_use]
    pub fn ContainsWithPoint(&self, point: Vector3) -> ContainmentType {
        if Vector3::DistanceSquared(point, self.Center) < self.Radius * self.Radius {
            ContainmentType::Contains
        } else {
            ContainmentType::Disjoint
        }
    }
    pub fn ContainsWithPointAndResult(
        &mut self,
        point: &mut Vector3,
        result: &mut ContainmentType,
    ) {
        *result = self.ContainsWithPoint(*point);
    }
    #[must_use]
    pub fn ContainsWithSphere(&self, sphere: Self) -> ContainmentType {
        let distance = Vector3::Distance(self.Center, sphere.Center);
        if self.Radius + sphere.Radius < distance {
            ContainmentType::Disjoint
        } else if self.Radius - sphere.Radius < distance {
            ContainmentType::Intersects
        } else {
            ContainmentType::Contains
        }
    }
    pub fn ContainsWithSphereAndResult(&mut self, sphere: &mut Self, result: &mut ContainmentType) {
        *result = self.ContainsWithSphere(*sphere);
    }

    pub(super) fn support_mapping(&self, direction: Vector3) -> Vector3 {
        let length = direction.Length();
        self.Center + direction * (self.Radius / length)
    }
}
