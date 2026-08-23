#![allow(non_snake_case, non_upper_case_globals, clippy::missing_panics_doc)]

use core::any::Any;

use super::gjk::Gjk;
use crate::value::{
    BoundingBox, BoundingSphere, ContainmentType, Matrix, Plane, PlaneIntersectionType, Ray,
    Vector3,
};

/// XNA view frustum derived from a combined view/projection matrix.
pub struct BoundingFrustum {
    matrix: Matrix,
    planes: [Plane; 6],
    corners: [Vector3; 8],
}

impl BoundingFrustum {
    pub const CornerCount: i32 = 8;

    #[must_use]
    pub fn new(value: Matrix) -> Self {
        let mut result = Self {
            matrix: Matrix::default(),
            planes: [Plane::default(); 6],
            corners: [Vector3::Zero; 8],
        };
        result.set_matrix(value);
        result
    }

    fn set_matrix(&mut self, value: Matrix) {
        self.matrix = value;
        self.planes[2] = Plane::from_a_and_b_and_c_and_d(
            0.0 - value.M14 - value.M11,
            0.0 - value.M24 - value.M21,
            0.0 - value.M34 - value.M31,
            0.0 - value.M44 - value.M41,
        );
        self.planes[3] = Plane::from_a_and_b_and_c_and_d(
            0.0 - value.M14 + value.M11,
            0.0 - value.M24 + value.M21,
            0.0 - value.M34 + value.M31,
            0.0 - value.M44 + value.M41,
        );
        self.planes[4] = Plane::from_a_and_b_and_c_and_d(
            0.0 - value.M14 + value.M12,
            0.0 - value.M24 + value.M22,
            0.0 - value.M34 + value.M32,
            0.0 - value.M44 + value.M42,
        );
        self.planes[5] = Plane::from_a_and_b_and_c_and_d(
            0.0 - value.M14 - value.M12,
            0.0 - value.M24 - value.M22,
            0.0 - value.M34 - value.M32,
            0.0 - value.M44 - value.M42,
        );
        self.planes[0] =
            Plane::from_a_and_b_and_c_and_d(-value.M13, -value.M23, -value.M33, -value.M43);
        self.planes[1] = Plane::from_a_and_b_and_c_and_d(
            0.0 - value.M14 + value.M13,
            0.0 - value.M24 + value.M23,
            0.0 - value.M34 + value.M33,
            0.0 - value.M44 + value.M43,
        );
        for plane in &mut self.planes {
            let length = plane.Normal.Length();
            plane.Normal /= length;
            plane.D /= length;
        }

        let mut ray = Self::intersection_line(self.planes[0], self.planes[2]);
        self.corners[0] = Self::intersection(self.planes[4], ray);
        self.corners[3] = Self::intersection(self.planes[5], ray);
        ray = Self::intersection_line(self.planes[3], self.planes[0]);
        self.corners[1] = Self::intersection(self.planes[4], ray);
        self.corners[2] = Self::intersection(self.planes[5], ray);
        ray = Self::intersection_line(self.planes[2], self.planes[1]);
        self.corners[4] = Self::intersection(self.planes[4], ray);
        self.corners[7] = Self::intersection(self.planes[5], ray);
        ray = Self::intersection_line(self.planes[1], self.planes[3]);
        self.corners[5] = Self::intersection(self.planes[4], ray);
        self.corners[6] = Self::intersection(self.planes[5], ray);
    }

    fn intersection_line(first: Plane, second: Plane) -> Ray {
        let direction = Vector3::Cross(first.Normal, second.Normal);
        let length_squared = direction.LengthSquared();
        let position = Vector3::Cross(
            second.Normal * (0.0 - first.D) + first.Normal * second.D,
            direction,
        ) / length_squared;
        Ray::new(position, direction)
    }

    fn intersection(plane: Plane, ray: Ray) -> Vector3 {
        let distance = (0.0 - plane.D - Vector3::Dot(plane.Normal, ray.Position))
            / Vector3::Dot(plane.Normal, ray.Direction);
        ray.Position + ray.Direction * distance
    }

    #[must_use]
    pub const fn Near(&self) -> Plane {
        self.planes[0]
    }
    #[must_use]
    pub const fn Far(&self) -> Plane {
        self.planes[1]
    }
    #[must_use]
    pub const fn Left(&self) -> Plane {
        self.planes[2]
    }
    #[must_use]
    pub const fn Right(&self) -> Plane {
        self.planes[3]
    }
    #[must_use]
    pub const fn Top(&self) -> Plane {
        self.planes[4]
    }
    #[must_use]
    pub const fn Bottom(&self) -> Plane {
        self.planes[5]
    }
    #[must_use]
    pub const fn Matrix(&self) -> Matrix {
        self.matrix
    }
    pub fn SetMatrix(&mut self, value: Matrix) {
        self.set_matrix(value);
    }

    #[must_use]
    pub fn GetCorners(&self) -> Vec<Vector3> {
        self.corners.to_vec()
    }
    pub fn GetCornersWithCorners(&self, corners: &mut [Vector3]) {
        assert!(corners.len() >= 8, "at least eight corners are required");
        corners[..8].copy_from_slice(&self.corners);
    }

    #[must_use]
    pub fn Equals(&self, other: &Self) -> bool {
        self.matrix.Equals(other.matrix)
    }
    #[must_use]
    pub fn EqualsWithObj(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self.Equals(other))
    }
    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        self.matrix.GetHashCode()
    }
    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "{{Near:{} Far:{} Left:{} Right:{} Top:{} Bottom:{}}}",
            self.Near().ToString(),
            self.Far().ToString(),
            self.Left().ToString(),
            self.Right().ToString(),
            self.Top().ToString(),
            self.Bottom().ToString()
        )
    }

    fn support_mapping(&self, direction: Vector3) -> Vector3 {
        let mut index = 0;
        let mut maximum = Vector3::Dot(self.corners[0], direction);
        for position in 1..8 {
            let value = Vector3::Dot(self.corners[position], direction);
            if value > maximum {
                index = position;
                maximum = value;
            }
        }
        self.corners[index]
    }

    fn gjk_intersects(
        &self,
        mut direction: Vector3,
        other_support: impl Fn(Vector3) -> Vector3,
    ) -> bool {
        let mut gjk = Gjk::default();
        gjk.reset();
        let mut previous_distance_squared = f32::MAX;
        loop {
            let this_support = self.support_mapping(-direction);
            let support = this_support - other_support(direction);
            if Vector3::Dot(direction, support) > 0.0 {
                return false;
            }
            gjk.add_support_point(support);
            direction = gjk.closest_point();
            let previous = previous_distance_squared;
            previous_distance_squared = direction.LengthSquared();
            if previous - previous_distance_squared <= 1e-5 * previous {
                return false;
            }
            let threshold = 4e-5 * gjk.max_length_squared();
            if gjk.full_simplex() || previous_distance_squared < threshold {
                return true;
            }
        }
    }

    #[must_use]
    pub fn Intersects(&self, r#box: BoundingBox) -> bool {
        let mut direction = self.corners[0] - r#box.Min;
        if direction.LengthSquared() < 1e-5 {
            direction = self.corners[0] - r#box.Max;
        }
        self.gjk_intersects(direction, |value| r#box.support_mapping(value))
    }
    pub fn IntersectsWithBoxAndResult(&self, r#box: &mut BoundingBox, result: &mut bool) {
        *result = self.Intersects(*r#box);
    }

    #[must_use]
    pub fn IntersectsWithFrustum(&self, frustum: &Self) -> bool {
        let mut direction = self.corners[0] - frustum.corners[0];
        if direction.LengthSquared() < 1e-5 {
            direction = self.corners[0] - frustum.corners[1];
        }
        self.gjk_intersects(direction, |value| frustum.support_mapping(value))
    }

    #[must_use]
    pub fn IntersectsWithPlane(&self, plane: Plane) -> PlaneIntersectionType {
        let mut sides = 0;
        for corner in self.corners {
            sides |= if Vector3::Dot(corner, plane.Normal) + plane.D > 0.0 {
                1
            } else {
                2
            };
            if sides == 3 {
                return PlaneIntersectionType::Intersecting;
            }
        }
        if sides == 1 {
            PlaneIntersectionType::Front
        } else {
            PlaneIntersectionType::Back
        }
    }
    pub fn IntersectsWithPlaneAndResult(
        &self,
        plane: &mut Plane,
        result: &mut PlaneIntersectionType,
    ) {
        *result = self.IntersectsWithPlane(*plane);
    }

    #[must_use]
    pub fn IntersectsWithRay(&self, ray: Ray) -> Option<f32> {
        if self.ContainsWithPoint(ray.Position) == ContainmentType::Contains {
            return Some(0.0);
        }
        let mut minimum = f32::MIN;
        let mut maximum = f32::MAX;
        for plane in self.planes {
            let direction_dot = Vector3::Dot(ray.Direction, plane.Normal);
            let position = Vector3::Dot(ray.Position, plane.Normal) + plane.D;
            if direction_dot.abs() < 1e-5 {
                if position > 0.0 {
                    return None;
                }
                continue;
            }
            let distance = (0.0 - position) / direction_dot;
            if direction_dot < 0.0 {
                if distance > maximum {
                    return None;
                }
                if distance > minimum {
                    minimum = distance;
                }
            } else {
                if distance < minimum {
                    return None;
                }
                if distance < maximum {
                    maximum = distance;
                }
            }
        }
        let distance = if minimum >= 0.0 { minimum } else { maximum };
        if distance >= 0.0 {
            Some(distance)
        } else {
            None
        }
    }
    pub fn IntersectsWithRayAndResult(&self, ray: &mut Ray, result: &mut Option<f32>) {
        *result = self.IntersectsWithRay(*ray);
    }

    #[must_use]
    pub fn IntersectsWithSphere(&self, sphere: BoundingSphere) -> bool {
        let mut direction = self.corners[0] - sphere.Center;
        if direction.LengthSquared() < 1e-5 {
            direction = Vector3::UnitX;
        }
        self.gjk_intersects(direction, |value| sphere.support_mapping(value))
    }
    pub fn IntersectsWithSphereAndResult(&self, sphere: &mut BoundingSphere, result: &mut bool) {
        *result = self.IntersectsWithSphere(*sphere);
    }

    #[must_use]
    pub fn Contains(&self, r#box: BoundingBox) -> ContainmentType {
        let mut intersects = false;
        for plane in self.planes {
            match r#box.IntersectsWithPlane(plane) {
                PlaneIntersectionType::Front => return ContainmentType::Disjoint,
                PlaneIntersectionType::Intersecting => intersects = true,
                PlaneIntersectionType::Back => {}
            }
        }
        if intersects {
            ContainmentType::Intersects
        } else {
            ContainmentType::Contains
        }
    }
    pub fn ContainsWithBoxAndResult(&self, r#box: &mut BoundingBox, result: &mut ContainmentType) {
        *result = self.Contains(*r#box);
    }

    #[must_use]
    pub fn ContainsWithFrustum(&self, frustum: &Self) -> ContainmentType {
        if !self.IntersectsWithFrustum(frustum) {
            return ContainmentType::Disjoint;
        }
        for corner in frustum.corners {
            if self.ContainsWithPoint(corner) == ContainmentType::Disjoint {
                return ContainmentType::Intersects;
            }
        }
        ContainmentType::Contains
    }

    #[must_use]
    pub fn ContainsWithPoint(&self, point: Vector3) -> ContainmentType {
        for plane in self.planes {
            if plane.Normal.X * point.X
                + plane.Normal.Y * point.Y
                + plane.Normal.Z * point.Z
                + plane.D
                > 1e-5
            {
                return ContainmentType::Disjoint;
            }
        }
        ContainmentType::Contains
    }
    pub fn ContainsWithPointAndResult(&self, point: &mut Vector3, result: &mut ContainmentType) {
        *result = self.ContainsWithPoint(*point);
    }

    #[must_use]
    pub fn ContainsWithSphere(&self, sphere: BoundingSphere) -> ContainmentType {
        let mut behind = 0;
        for plane in self.planes {
            let distance = Vector3::Dot(plane.Normal, sphere.Center) + plane.D;
            if distance > sphere.Radius {
                return ContainmentType::Disjoint;
            }
            if distance < 0.0 - sphere.Radius {
                behind += 1;
            }
        }
        if behind == 6 {
            ContainmentType::Contains
        } else {
            ContainmentType::Intersects
        }
    }
    pub fn ContainsWithSphereAndResult(
        &self,
        sphere: &mut BoundingSphere,
        result: &mut ContainmentType,
    ) {
        *result = self.ContainsWithSphere(*sphere);
    }

    pub(super) const fn corners(&self) -> &[Vector3; 8] {
        &self.corners
    }
}

impl PartialEq for BoundingFrustum {
    fn eq(&self, other: &Self) -> bool {
        self.Equals(other)
    }
}
