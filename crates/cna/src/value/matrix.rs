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
use super::{Plane, Quaternion, Vector3};

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
        m11: f32,
        m12: f32,
        m13: f32,
        m14: f32,
        m21: f32,
        m22: f32,
        m23: f32,
        m24: f32,
        m31: f32,
        m32: f32,
        m33: f32,
        m34: f32,
        m41: f32,
        m42: f32,
        m43: f32,
        m44: f32,
    ) -> Self {
        Self {
            M11: m11,
            M12: m12,
            M13: m13,
            M14: m14,
            M21: m21,
            M22: m22,
            M23: m23,
            M24: m24,
            M31: m31,
            M32: m32,
            M33: m33,
            M34: m34,
            M41: m41,
            M42: m42,
            M43: m43,
            M44: m44,
        }
    }

    #[must_use]
    pub const fn Up(&self) -> Vector3 {
        Vector3::from_x_and_y_and_z(self.M21, self.M22, self.M23)
    }
    pub fn SetUp(&mut self, value: Vector3) {
        self.M21 = value.X;
        self.M22 = value.Y;
        self.M23 = value.Z;
    }
    #[must_use]
    pub fn Down(&self) -> Vector3 {
        Vector3::from_x_and_y_and_z(-self.M21, -self.M22, -self.M23)
    }
    pub fn SetDown(&mut self, value: Vector3) {
        self.M21 = -value.X;
        self.M22 = -value.Y;
        self.M23 = -value.Z;
    }
    #[must_use]
    pub const fn Right(&self) -> Vector3 {
        Vector3::from_x_and_y_and_z(self.M11, self.M12, self.M13)
    }
    pub fn SetRight(&mut self, value: Vector3) {
        self.M11 = value.X;
        self.M12 = value.Y;
        self.M13 = value.Z;
    }
    #[must_use]
    pub fn Left(&self) -> Vector3 {
        Vector3::from_x_and_y_and_z(-self.M11, -self.M12, -self.M13)
    }
    pub fn SetLeft(&mut self, value: Vector3) {
        self.M11 = -value.X;
        self.M12 = -value.Y;
        self.M13 = -value.Z;
    }
    #[must_use]
    pub fn Forward(&self) -> Vector3 {
        Vector3::from_x_and_y_and_z(-self.M31, -self.M32, -self.M33)
    }
    pub fn SetForward(&mut self, value: Vector3) {
        self.M31 = -value.X;
        self.M32 = -value.Y;
        self.M33 = -value.Z;
    }
    #[must_use]
    pub const fn Backward(&self) -> Vector3 {
        Vector3::from_x_and_y_and_z(self.M31, self.M32, self.M33)
    }
    pub fn SetBackward(&mut self, value: Vector3) {
        self.M31 = value.X;
        self.M32 = value.Y;
        self.M33 = value.Z;
    }
    #[must_use]
    pub const fn Translation(&self) -> Vector3 {
        Vector3::from_x_and_y_and_z(self.M41, self.M42, self.M43)
    }
    pub fn SetTranslation(&mut self, value: Vector3) {
        self.M41 = value.X;
        self.M42 = value.Y;
        self.M43 = value.Z;
    }
    #[must_use]
    pub fn CreateScale(xScale: f32, yScale: f32, zScale: f32) -> Self {
        Self::new(
            xScale, 0.0, 0.0, 0.0, 0.0, yScale, 0.0, 0.0, 0.0, 0.0, zScale, 0.0, 0.0, 0.0, 0.0, 1.0,
        )
    }
    pub fn CreateScaleWithXScaleAndYScaleAndZScaleAndResult(
        xScale: f32,
        yScale: f32,
        zScale: f32,
        result: &mut Self,
    ) {
        *result = Self::CreateScale(xScale, yScale, zScale);
    }
    #[must_use]
    pub fn CreateScaleWithScales(scales: Vector3) -> Self {
        Self::CreateScale(scales.X, scales.Y, scales.Z)
    }
    pub fn CreateScaleWithScalesAndResult(scales: &mut Vector3, result: &mut Self) {
        *result = Self::CreateScaleWithScales(*scales);
    }
    #[must_use]
    pub fn CreateScaleWithScale(scale: f32) -> Self {
        Self::CreateScale(scale, scale, scale)
    }
    pub fn CreateScaleWithScaleAndResult(scale: f32, result: &mut Self) {
        *result = Self::CreateScaleWithScale(scale);
    }
    #[must_use]
    pub fn CreateRotationX(radians: f32) -> Self {
        let s = f64::from(radians).sin() as f32;
        let c = f64::from(radians).cos() as f32;
        Self::new(
            1.0, 0.0, 0.0, 0.0, 0.0, c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0,
        )
    }
    pub fn CreateRotationXWithRadiansAndResult(radians: f32, result: &mut Self) {
        *result = Self::CreateRotationX(radians);
    }
    #[must_use]
    pub fn CreateRotationY(radians: f32) -> Self {
        let s = f64::from(radians).sin() as f32;
        let c = f64::from(radians).cos() as f32;
        Self::new(
            c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
        )
    }
    pub fn CreateRotationYWithRadiansAndResult(radians: f32, result: &mut Self) {
        *result = Self::CreateRotationY(radians);
    }
    #[must_use]
    pub fn CreateRotationZ(radians: f32) -> Self {
        let s = f64::from(radians).sin() as f32;
        let c = f64::from(radians).cos() as f32;
        Self::new(
            c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        )
    }
    pub fn CreateRotationZWithRadiansAndResult(radians: f32, result: &mut Self) {
        *result = Self::CreateRotationZ(radians);
    }
    #[must_use]
    pub fn CreateTranslation(position: Vector3) -> Self {
        Self::CreateTranslationWithXPositionAndYPositionAndZPosition(
            position.X, position.Y, position.Z,
        )
    }
    pub fn CreateTranslationWithPositionAndResult(position: &mut Vector3, result: &mut Self) {
        *result = Self::CreateTranslation(*position);
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
    pub fn CreateTranslationWithXPositionAndYPositionAndZPositionAndResult(
        xPosition: f32,
        yPosition: f32,
        zPosition: f32,
        result: &mut Self,
    ) {
        *result = Self::CreateTranslationWithXPositionAndYPositionAndZPosition(
            xPosition, yPosition, zPosition,
        );
    }
    #[must_use]
    pub fn CreateLookAt(
        cameraPosition: Vector3,
        cameraTarget: Vector3,
        cameraUpVector: Vector3,
    ) -> Self {
        let z = Vector3::NormalizeWithValue(cameraPosition - cameraTarget);
        let x = Vector3::NormalizeWithValue(Vector3::Cross(cameraUpVector, z));
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
            -Vector3::Dot(x, cameraPosition),
            -Vector3::Dot(y, cameraPosition),
            -Vector3::Dot(z, cameraPosition),
            1.0,
        )
    }
    pub fn CreateLookAtWithCameraPositionAndCameraTargetAndCameraUpVectorAndResult(
        cameraPosition: &mut Vector3,
        cameraTarget: &mut Vector3,
        cameraUpVector: &mut Vector3,
        result: &mut Self,
    ) {
        *result = Self::CreateLookAt(*cameraPosition, *cameraTarget, *cameraUpVector);
    }
    #[must_use]
    pub fn CreatePerspectiveFieldOfView(
        fieldOfView: f32,
        aspectRatio: f32,
        nearPlaneDistance: f32,
        farPlaneDistance: f32,
    ) -> Self {
        assert!(
            fieldOfView > 0.0 && fieldOfView < core::f32::consts::PI,
            "field of view must be between zero and Pi"
        );
        assert!(
            nearPlaneDistance > 0.0
                && farPlaneDistance > 0.0
                && nearPlaneDistance < farPlaneDistance,
            "invalid clipping planes"
        );
        let y = 1.0 / (f64::from(fieldOfView * 0.5).tan() as f32);
        let x = y / aspectRatio;
        let denominator = nearPlaneDistance - farPlaneDistance;
        let range = farPlaneDistance / denominator;
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
            range,
            -1.0,
            0.0,
            0.0,
            nearPlaneDistance * farPlaneDistance / denominator,
            0.0,
        )
    }
    pub fn CreatePerspectiveFieldOfViewWithFieldOfViewAndAspectRatioAndNearPlaneDistanceAndFarPlaneDistanceAndResult(
        fieldOfView: f32,
        aspectRatio: f32,
        nearPlaneDistance: f32,
        farPlaneDistance: f32,
        result: &mut Self,
    ) {
        *result = Self::CreatePerspectiveFieldOfView(
            fieldOfView,
            aspectRatio,
            nearPlaneDistance,
            farPlaneDistance,
        );
    }

    #[must_use]
    pub fn CreatePerspective(
        width: f32,
        height: f32,
        nearPlaneDistance: f32,
        farPlaneDistance: f32,
    ) -> Self {
        Self::validate_perspective(nearPlaneDistance, farPlaneDistance);
        let denominator = nearPlaneDistance - farPlaneDistance;
        let range = farPlaneDistance / denominator;
        Self::new(
            2.0 * nearPlaneDistance / width,
            0.0,
            0.0,
            0.0,
            0.0,
            2.0 * nearPlaneDistance / height,
            0.0,
            0.0,
            0.0,
            0.0,
            range,
            -1.0,
            0.0,
            0.0,
            nearPlaneDistance * farPlaneDistance / denominator,
            0.0,
        )
    }
    pub fn CreatePerspectiveWithWidthAndHeightAndNearPlaneDistanceAndFarPlaneDistanceAndResult(
        width: f32,
        height: f32,
        nearPlaneDistance: f32,
        farPlaneDistance: f32,
        result: &mut Self,
    ) {
        *result = Self::CreatePerspective(width, height, nearPlaneDistance, farPlaneDistance);
    }

    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn CreatePerspectiveOffCenter(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        nearPlaneDistance: f32,
        farPlaneDistance: f32,
    ) -> Self {
        Self::validate_perspective(nearPlaneDistance, farPlaneDistance);
        let denominator = nearPlaneDistance - farPlaneDistance;
        let range = farPlaneDistance / denominator;
        Self::new(
            2.0 * nearPlaneDistance / (right - left),
            0.0,
            0.0,
            0.0,
            0.0,
            2.0 * nearPlaneDistance / (top - bottom),
            0.0,
            0.0,
            (left + right) / (right - left),
            (top + bottom) / (top - bottom),
            range,
            -1.0,
            0.0,
            0.0,
            nearPlaneDistance * farPlaneDistance / denominator,
            0.0,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn CreatePerspectiveOffCenterWithLeftAndRightAndBottomAndTopAndNearPlaneDistanceAndFarPlaneDistanceAndResult(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        nearPlaneDistance: f32,
        farPlaneDistance: f32,
        result: &mut Self,
    ) {
        *result = Self::CreatePerspectiveOffCenter(
            left,
            right,
            bottom,
            top,
            nearPlaneDistance,
            farPlaneDistance,
        );
    }

    #[must_use]
    pub fn CreateOrthographic(width: f32, height: f32, zNearPlane: f32, zFarPlane: f32) -> Self {
        Self::new(
            2.0 / width,
            0.0,
            0.0,
            0.0,
            0.0,
            2.0 / height,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0 / (zNearPlane - zFarPlane),
            0.0,
            0.0,
            0.0,
            zNearPlane / (zNearPlane - zFarPlane),
            1.0,
        )
    }
    pub fn CreateOrthographicWithWidthAndHeightAndZNearPlaneAndZFarPlaneAndResult(
        width: f32,
        height: f32,
        zNearPlane: f32,
        zFarPlane: f32,
        result: &mut Self,
    ) {
        *result = Self::CreateOrthographic(width, height, zNearPlane, zFarPlane);
    }

    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn CreateOrthographicOffCenter(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        zNearPlane: f32,
        zFarPlane: f32,
    ) -> Self {
        Self::new(
            2.0 / (right - left),
            0.0,
            0.0,
            0.0,
            0.0,
            2.0 / (top - bottom),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0 / (zNearPlane - zFarPlane),
            0.0,
            (left + right) / (left - right),
            (top + bottom) / (bottom - top),
            zNearPlane / (zNearPlane - zFarPlane),
            1.0,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn CreateOrthographicOffCenterWithLeftAndRightAndBottomAndTopAndZNearPlaneAndZFarPlaneAndResult(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        zNearPlane: f32,
        zFarPlane: f32,
        result: &mut Self,
    ) {
        *result =
            Self::CreateOrthographicOffCenter(left, right, bottom, top, zNearPlane, zFarPlane);
    }

    fn validate_perspective(near: f32, far: f32) {
        assert!(
            near > 0.0 && far > 0.0 && near < far,
            "invalid clipping planes"
        );
    }
    #[must_use]
    pub fn CreateFromAxisAngle(axis: Vector3, angle: f32) -> Self {
        let x = axis.X;
        let y = axis.Y;
        let z = axis.Z;
        let sin = f64::from(angle).sin() as f32;
        let cos = f64::from(angle).cos() as f32;
        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        Self::new(
            xx + cos * (1.0 - xx),
            xy - cos * xy + sin * z,
            xz - cos * xz - sin * y,
            0.0,
            xy - cos * xy - sin * z,
            yy + cos * (1.0 - yy),
            yz - cos * yz + sin * x,
            0.0,
            xz - cos * xz + sin * y,
            yz - cos * yz - sin * x,
            zz + cos * (1.0 - zz),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        )
    }
    pub fn CreateFromAxisAngleWithAxisAndAngleAndResult(
        axis: &mut Vector3,
        angle: f32,
        result: &mut Self,
    ) {
        *result = Self::CreateFromAxisAngle(*axis, angle);
    }

    #[must_use]
    pub fn CreateWorld(position: Vector3, forward: Vector3, up: Vector3) -> Self {
        let backward = Vector3::NormalizeWithValue(-forward);
        let right = Vector3::NormalizeWithValue(Vector3::Cross(up, backward));
        let up = Vector3::Cross(backward, right);
        Self::new(
            right.X, right.Y, right.Z, 0.0, up.X, up.Y, up.Z, 0.0, backward.X, backward.Y,
            backward.Z, 0.0, position.X, position.Y, position.Z, 1.0,
        )
    }
    pub fn CreateWorldWithPositionAndForwardAndUpAndResult(
        position: &mut Vector3,
        forward: &mut Vector3,
        up: &mut Vector3,
        result: &mut Self,
    ) {
        *result = Self::CreateWorld(*position, *forward, *up);
    }
    #[must_use]
    pub fn CreateFromQuaternion(quaternion: Quaternion) -> Self {
        let xx = quaternion.X * quaternion.X;
        let yy = quaternion.Y * quaternion.Y;
        let zz = quaternion.Z * quaternion.Z;
        let xy = quaternion.X * quaternion.Y;
        let zw = quaternion.Z * quaternion.W;
        let zx = quaternion.Z * quaternion.X;
        let yw = quaternion.Y * quaternion.W;
        let yz = quaternion.Y * quaternion.Z;
        let xw = quaternion.X * quaternion.W;
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
    pub fn CreateFromQuaternionWithQuaternionAndResult(
        quaternion: &mut Quaternion,
        result: &mut Self,
    ) {
        *result = Self::CreateFromQuaternion(*quaternion);
    }

    #[must_use]
    pub fn CreateFromYawPitchRoll(yaw: f32, pitch: f32, roll: f32) -> Self {
        Self::CreateFromQuaternion(Quaternion::CreateFromYawPitchRoll(yaw, pitch, roll))
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
    pub fn CreateBillboard(
        objectPosition: Vector3,
        cameraPosition: Vector3,
        cameraUpVector: Vector3,
        cameraForwardVector: Option<Vector3>,
    ) -> Self {
        let mut facing = objectPosition - cameraPosition;
        let squared = facing.LengthSquared();
        if squared < 0.0001 {
            facing = cameraForwardVector.map_or(Vector3::Forward, Neg::neg);
        } else {
            facing *= 1.0 / squared.sqrt();
        }
        let right = Vector3::NormalizeWithValue(Vector3::Cross(cameraUpVector, facing));
        let up = Vector3::Cross(facing, right);
        Self::new(
            right.X,
            right.Y,
            right.Z,
            0.0,
            up.X,
            up.Y,
            up.Z,
            0.0,
            facing.X,
            facing.Y,
            facing.Z,
            0.0,
            objectPosition.X,
            objectPosition.Y,
            objectPosition.Z,
            1.0,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn CreateBillboardWithObjectPositionAndCameraPositionAndCameraUpVectorAndCameraForwardVectorAndResult(
        objectPosition: &mut Vector3,
        cameraPosition: &mut Vector3,
        cameraUpVector: &mut Vector3,
        cameraForwardVector: Option<Vector3>,
        result: &mut Self,
    ) {
        *result = Self::CreateBillboard(
            *objectPosition,
            *cameraPosition,
            *cameraUpVector,
            cameraForwardVector,
        );
    }

    #[must_use]
    pub fn CreateConstrainedBillboard(
        objectPosition: Vector3,
        cameraPosition: Vector3,
        rotateAxis: Vector3,
        cameraForwardVector: Option<Vector3>,
        objectForwardVector: Option<Vector3>,
    ) -> Self {
        let mut facing = objectPosition - cameraPosition;
        let squared = facing.LengthSquared();
        if squared < 0.0001 {
            facing = cameraForwardVector.map_or(Vector3::Forward, Neg::neg);
        } else {
            facing *= 1.0 / squared.sqrt();
        }
        let up = rotateAxis;
        let alignment = Vector3::Dot(rotateAxis, facing);
        let (right, forward) = if alignment.abs() > 0.998_254_66 {
            let mut candidate = objectForwardVector.unwrap_or_else(|| {
                if Vector3::Dot(rotateAxis, Vector3::Forward).abs() > 0.998_254_66 {
                    Vector3::Right
                } else {
                    Vector3::Forward
                }
            });
            if objectForwardVector.is_some()
                && Vector3::Dot(rotateAxis, candidate).abs() > 0.998_254_66
            {
                candidate = if Vector3::Dot(rotateAxis, Vector3::Forward).abs() > 0.998_254_66 {
                    Vector3::Right
                } else {
                    Vector3::Forward
                };
            }
            let right = Vector3::NormalizeWithValue(Vector3::Cross(rotateAxis, candidate));
            let forward = Vector3::NormalizeWithValue(Vector3::Cross(right, rotateAxis));
            (right, forward)
        } else {
            let right = Vector3::NormalizeWithValue(Vector3::Cross(rotateAxis, facing));
            let forward = Vector3::NormalizeWithValue(Vector3::Cross(right, up));
            (right, forward)
        };
        Self::new(
            right.X,
            right.Y,
            right.Z,
            0.0,
            up.X,
            up.Y,
            up.Z,
            0.0,
            forward.X,
            forward.Y,
            forward.Z,
            0.0,
            objectPosition.X,
            objectPosition.Y,
            objectPosition.Z,
            1.0,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn CreateConstrainedBillboardWithObjectPositionAndCameraPositionAndRotateAxisAndCameraForwardVectorAndObjectForwardVectorAndResult(
        objectPosition: &mut Vector3,
        cameraPosition: &mut Vector3,
        rotateAxis: &mut Vector3,
        cameraForwardVector: Option<Vector3>,
        objectForwardVector: Option<Vector3>,
        result: &mut Self,
    ) {
        *result = Self::CreateConstrainedBillboard(
            *objectPosition,
            *cameraPosition,
            *rotateAxis,
            cameraForwardVector,
            objectForwardVector,
        );
    }

    fn normalized_plane(value: Plane) -> Plane {
        let squared = value.Normal.X * value.Normal.X
            + value.Normal.Y * value.Normal.Y
            + value.Normal.Z * value.Normal.Z;
        if (squared - 1.0).abs() < 1.192_092_9e-7 {
            value
        } else {
            let reciprocal = 1.0 / squared.sqrt();
            Plane::from_normal_and_d(value.Normal * reciprocal, value.D * reciprocal)
        }
    }

    #[must_use]
    pub fn CreateShadow(lightDirection: Vector3, plane: Plane) -> Self {
        let plane = Self::normalized_plane(plane);
        let x = plane.Normal.X;
        let y = plane.Normal.Y;
        let z = plane.Normal.Z;
        let dot = x * lightDirection.X + y * lightDirection.Y + z * lightDirection.Z;
        Self::new(
            -x * lightDirection.X + dot,
            -x * lightDirection.Y,
            -x * lightDirection.Z,
            0.0,
            -y * lightDirection.X,
            -y * lightDirection.Y + dot,
            -y * lightDirection.Z,
            0.0,
            -z * lightDirection.X,
            -z * lightDirection.Y,
            -z * lightDirection.Z + dot,
            0.0,
            -plane.D * lightDirection.X,
            -plane.D * lightDirection.Y,
            -plane.D * lightDirection.Z,
            dot,
        )
    }
    pub fn CreateShadowWithLightDirectionAndPlaneAndResult(
        lightDirection: &mut Vector3,
        plane: &mut Plane,
        result: &mut Self,
    ) {
        *result = Self::CreateShadow(*lightDirection, *plane);
    }

    #[must_use]
    pub fn CreateReflection(value: Plane) -> Self {
        let value = Self::normalized_plane(value);
        let x = value.Normal.X;
        let y = value.Normal.Y;
        let z = value.Normal.Z;
        let x2 = -2.0 * x;
        let y2 = -2.0 * y;
        let z2 = -2.0 * z;
        Self::new(
            x2 * x + 1.0,
            y2 * x,
            z2 * x,
            0.0,
            x2 * y,
            y2 * y + 1.0,
            z2 * y,
            0.0,
            x2 * z,
            y2 * z,
            z2 * z + 1.0,
            0.0,
            x2 * value.D,
            y2 * value.D,
            z2 * value.D,
            1.0,
        )
    }
    pub fn CreateReflectionWithValueAndResult(value: &mut Plane, result: &mut Self) {
        *value = Self::normalized_plane(*value);
        *result = Self::CreateReflection(*value);
    }

    pub fn Decompose(
        &self,
        scale: &mut Vector3,
        rotation: &mut Quaternion,
        translation: &mut Vector3,
    ) -> bool {
        *translation = self.Translation();
        let mut rows = [
            Vector3::from_x_and_y_and_z(self.M11, self.M12, self.M13),
            Vector3::from_x_and_y_and_z(self.M21, self.M22, self.M23),
            Vector3::from_x_and_y_and_z(self.M31, self.M32, self.M33),
        ];
        let mut scales = [rows[0].Length(), rows[1].Length(), rows[2].Length()];
        let mut order = [0_usize, 1, 2];
        order.sort_by(|left, right| scales[*right].total_cmp(&scales[*left]));
        let canonical = [Vector3::UnitX, Vector3::UnitY, Vector3::UnitZ];
        let largest = order[0];
        let middle = order[1];
        let smallest = order[2];
        if scales[largest] < 0.0001 {
            rows[largest] = canonical[largest];
        }
        rows[largest].Normalize();
        if scales[middle] < 0.0001 {
            let v = rows[largest];
            let absolute = [v.X.abs(), v.Y.abs(), v.Z.abs()];
            let axis = if absolute[0] < absolute[1] {
                if absolute[0] < absolute[2] {
                    0
                } else {
                    2
                }
            } else if absolute[1] < absolute[2] {
                1
            } else {
                2
            };
            rows[middle] = Vector3::Cross(canonical[axis], rows[largest]);
        }
        rows[middle].Normalize();
        if scales[smallest] < 0.0001 {
            rows[smallest] = Vector3::Cross(rows[largest], rows[middle]);
        }
        rows[smallest].Normalize();
        let mut matrix = Self::new(
            rows[0].X, rows[0].Y, rows[0].Z, 0.0, rows[1].X, rows[1].Y, rows[1].Z, 0.0, rows[2].X,
            rows[2].Y, rows[2].Z, 0.0, 0.0, 0.0, 0.0, 1.0,
        );
        let mut determinant = matrix.Determinant();
        if determinant < 0.0 {
            scales[largest] = -scales[largest];
            rows[largest] = -rows[largest];
            match largest {
                0 => {
                    matrix.M11 = rows[0].X;
                    matrix.M12 = rows[0].Y;
                    matrix.M13 = rows[0].Z;
                }
                1 => {
                    matrix.M21 = rows[1].X;
                    matrix.M22 = rows[1].Y;
                    matrix.M23 = rows[1].Z;
                }
                _ => {
                    matrix.M31 = rows[2].X;
                    matrix.M32 = rows[2].Y;
                    matrix.M33 = rows[2].Z;
                }
            }
            determinant = -determinant;
        }
        *scale = Vector3::from_x_and_y_and_z(scales[0], scales[1], scales[2]);
        let error = determinant - 1.0;
        if error * error > 0.0001 {
            *rotation = Quaternion::Identity;
            false
        } else {
            *rotation = Quaternion::CreateFromRotationMatrix(matrix);
            true
        }
    }

    #[must_use]
    pub fn Transform(value: Self, rotation: Quaternion) -> Self {
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
        let r11 = 1.0 - yy2 - zz2;
        let r12 = xy2 - wz2;
        let r13 = xz2 + wy2;
        let r21 = xy2 + wz2;
        let r22 = 1.0 - xx2 - zz2;
        let r23 = yz2 - wx2;
        let r31 = xz2 - wy2;
        let r32 = yz2 + wx2;
        let r33 = 1.0 - xx2 - yy2;
        Self::new(
            value.M11 * r11 + value.M12 * r12 + value.M13 * r13,
            value.M11 * r21 + value.M12 * r22 + value.M13 * r23,
            value.M11 * r31 + value.M12 * r32 + value.M13 * r33,
            value.M14,
            value.M21 * r11 + value.M22 * r12 + value.M23 * r13,
            value.M21 * r21 + value.M22 * r22 + value.M23 * r23,
            value.M21 * r31 + value.M22 * r32 + value.M23 * r33,
            value.M24,
            value.M31 * r11 + value.M32 * r12 + value.M33 * r13,
            value.M31 * r21 + value.M32 * r22 + value.M33 * r23,
            value.M31 * r31 + value.M32 * r32 + value.M33 * r33,
            value.M34,
            value.M41 * r11 + value.M42 * r12 + value.M43 * r13,
            value.M41 * r21 + value.M42 * r22 + value.M43 * r23,
            value.M41 * r31 + value.M42 * r32 + value.M43 * r33,
            value.M44,
        )
    }
    pub fn TransformWithValueAndRotationAndResult(
        value: &mut Self,
        rotation: &mut Quaternion,
        result: &mut Self,
    ) {
        *result = Self::Transform(*value, *rotation);
    }

    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "{{ {{M11:{} M12:{} M13:{} M14:{}}} {{M21:{} M22:{} M23:{} M24:{}}} {{M31:{} M32:{} M33:{} M34:{}}} {{M41:{} M42:{} M43:{} M44:{}}} }}",
            self.M11,
            self.M12,
            self.M13,
            self.M14,
            self.M21,
            self.M22,
            self.M23,
            self.M24,
            self.M31,
            self.M32,
            self.M33,
            self.M34,
            self.M41,
            self.M42,
            self.M43,
            self.M44,
        )
    }

    #[must_use]
    pub fn Equals(&self, other: Self) -> bool {
        self.M11 == other.M11
            && self.M12 == other.M12
            && self.M13 == other.M13
            && self.M14 == other.M14
            && self.M21 == other.M21
            && self.M22 == other.M22
            && self.M23 == other.M23
            && self.M24 == other.M24
            && self.M31 == other.M31
            && self.M32 == other.M32
            && self.M33 == other.M33
            && self.M34 == other.M34
            && self.M41 == other.M41
            && self.M42 == other.M42
            && self.M43 == other.M43
            && self.M44 == other.M44
    }

    #[must_use]
    pub fn EqualsWithObj(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self.Equals(*other))
    }

    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        self.elements()
            .iter()
            .fold(0_i32, |hash, value| hash.wrapping_add(xna_f32_hash(*value)))
    }

    const fn elements(&self) -> [f32; 16] {
        [
            self.M11, self.M12, self.M13, self.M14, self.M21, self.M22, self.M23, self.M24,
            self.M31, self.M32, self.M33, self.M34, self.M41, self.M42, self.M43, self.M44,
        ]
    }
    #[must_use]
    pub fn Transpose(matrix: Self) -> Self {
        Self::new(
            matrix.M11, matrix.M21, matrix.M31, matrix.M41, matrix.M12, matrix.M22, matrix.M32,
            matrix.M42, matrix.M13, matrix.M23, matrix.M33, matrix.M43, matrix.M14, matrix.M24,
            matrix.M34, matrix.M44,
        )
    }
    pub fn TransposeWithMatrixAndResult(matrix: &mut Self, result: &mut Self) {
        *result = Self::Transpose(*matrix);
    }

    #[must_use]
    pub fn Determinant(&self) -> f32 {
        let n1 = self.M33 * self.M44 - self.M34 * self.M43;
        let n2 = self.M32 * self.M44 - self.M34 * self.M42;
        let n3 = self.M32 * self.M43 - self.M33 * self.M42;
        let n4 = self.M31 * self.M44 - self.M34 * self.M41;
        let n5 = self.M31 * self.M43 - self.M33 * self.M41;
        let n6 = self.M31 * self.M42 - self.M32 * self.M41;
        self.M11 * (self.M22 * n1 - self.M23 * n2 + self.M24 * n3)
            - self.M12 * (self.M21 * n1 - self.M23 * n4 + self.M24 * n5)
            + self.M13 * (self.M21 * n2 - self.M22 * n4 + self.M24 * n6)
            - self.M14 * (self.M21 * n3 - self.M22 * n5 + self.M23 * n6)
    }

    #[must_use]
    pub fn Invert(matrix: Self) -> Self {
        let m = matrix.elements();
        let n1 = m[10] * m[15] - m[11] * m[14];
        let n2 = m[9] * m[15] - m[11] * m[13];
        let n3 = m[9] * m[14] - m[10] * m[13];
        let n4 = m[8] * m[15] - m[11] * m[12];
        let n5 = m[8] * m[14] - m[10] * m[12];
        let n6 = m[8] * m[13] - m[9] * m[12];
        let n7 = m[5] * n1 - m[6] * n2 + m[7] * n3;
        let n8 = -(m[4] * n1 - m[6] * n4 + m[7] * n5);
        let n9 = m[4] * n2 - m[5] * n4 + m[7] * n6;
        let n10 = -(m[4] * n3 - m[5] * n5 + m[6] * n6);
        let reciprocal = 1.0 / (m[0] * n7 + m[1] * n8 + m[2] * n9 + m[3] * n10);
        let m11 = n7 * reciprocal;
        let m21 = n8 * reciprocal;
        let m31 = n9 * reciprocal;
        let m41 = n10 * reciprocal;
        let m12 = -(m[1] * n1 - m[2] * n2 + m[3] * n3) * reciprocal;
        let m22 = (m[0] * n1 - m[2] * n4 + m[3] * n5) * reciprocal;
        let m32 = -(m[0] * n2 - m[1] * n4 + m[3] * n6) * reciprocal;
        let m42 = (m[0] * n3 - m[1] * n5 + m[2] * n6) * reciprocal;
        let n12 = m[6] * m[15] - m[7] * m[14];
        let n13 = m[5] * m[15] - m[7] * m[13];
        let n14 = m[5] * m[14] - m[6] * m[13];
        let n15 = m[4] * m[15] - m[7] * m[12];
        let n16 = m[4] * m[14] - m[6] * m[12];
        let n17 = m[4] * m[13] - m[5] * m[12];
        let m13 = (m[1] * n12 - m[2] * n13 + m[3] * n14) * reciprocal;
        let m23 = -(m[0] * n12 - m[2] * n15 + m[3] * n16) * reciprocal;
        let m33 = (m[0] * n13 - m[1] * n15 + m[3] * n17) * reciprocal;
        let m43 = -(m[0] * n14 - m[1] * n16 + m[2] * n17) * reciprocal;
        let n18 = m[6] * m[11] - m[7] * m[10];
        let n19 = m[5] * m[11] - m[7] * m[9];
        let n20 = m[5] * m[10] - m[6] * m[9];
        let n21 = m[4] * m[11] - m[7] * m[8];
        let n22 = m[4] * m[10] - m[6] * m[8];
        let n23 = m[4] * m[9] - m[5] * m[8];
        let m14 = -(m[1] * n18 - m[2] * n19 + m[3] * n20) * reciprocal;
        let m24 = (m[0] * n18 - m[2] * n21 + m[3] * n22) * reciprocal;
        let m34 = -(m[0] * n19 - m[1] * n21 + m[3] * n23) * reciprocal;
        let m44 = (m[0] * n20 - m[1] * n22 + m[2] * n23) * reciprocal;
        Self::new(
            m11, m12, m13, m14, m21, m22, m23, m24, m31, m32, m33, m34, m41, m42, m43, m44,
        )
    }
    pub fn InvertWithMatrixAndResult(matrix: &mut Self, result: &mut Self) {
        *result = Self::Invert(*matrix);
    }

    #[must_use]
    pub fn Lerp(matrix1: Self, matrix2: Self, amount: f32) -> Self {
        let a = matrix1.elements();
        let b = matrix2.elements();
        let mut values = [0.0; 16];
        for index in 0..16 {
            values[index] = a[index] + (b[index] - a[index]) * amount;
        }
        Self::new(
            values[0], values[1], values[2], values[3], values[4], values[5], values[6], values[7],
            values[8], values[9], values[10], values[11], values[12], values[13], values[14],
            values[15],
        )
    }
    pub fn LerpWithMatrix1AndMatrix2AndAmountAndResult(
        matrix1: &mut Self,
        matrix2: &mut Self,
        amount: f32,
        result: &mut Self,
    ) {
        *result = Self::Lerp(*matrix1, *matrix2, amount);
    }

    #[must_use]
    pub fn Negate(matrix: Self) -> Self {
        Self::MultiplyWithMatrix1AndScaleFactor(matrix, -1.0)
    }
    pub fn NegateWithMatrixAndResult(matrix: &mut Self, result: &mut Self) {
        *result = Self::Negate(*matrix);
    }
    #[must_use]
    pub fn Add(matrix1: Self, matrix2: Self) -> Self {
        Self::new(
            matrix1.M11 + matrix2.M11,
            matrix1.M12 + matrix2.M12,
            matrix1.M13 + matrix2.M13,
            matrix1.M14 + matrix2.M14,
            matrix1.M21 + matrix2.M21,
            matrix1.M22 + matrix2.M22,
            matrix1.M23 + matrix2.M23,
            matrix1.M24 + matrix2.M24,
            matrix1.M31 + matrix2.M31,
            matrix1.M32 + matrix2.M32,
            matrix1.M33 + matrix2.M33,
            matrix1.M34 + matrix2.M34,
            matrix1.M41 + matrix2.M41,
            matrix1.M42 + matrix2.M42,
            matrix1.M43 + matrix2.M43,
            matrix1.M44 + matrix2.M44,
        )
    }
    pub fn AddWithMatrix1AndMatrix2AndResult(
        matrix1: &mut Self,
        matrix2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Add(*matrix1, *matrix2);
    }
    #[must_use]
    pub fn Subtract(matrix1: Self, matrix2: Self) -> Self {
        Self::new(
            matrix1.M11 - matrix2.M11,
            matrix1.M12 - matrix2.M12,
            matrix1.M13 - matrix2.M13,
            matrix1.M14 - matrix2.M14,
            matrix1.M21 - matrix2.M21,
            matrix1.M22 - matrix2.M22,
            matrix1.M23 - matrix2.M23,
            matrix1.M24 - matrix2.M24,
            matrix1.M31 - matrix2.M31,
            matrix1.M32 - matrix2.M32,
            matrix1.M33 - matrix2.M33,
            matrix1.M34 - matrix2.M34,
            matrix1.M41 - matrix2.M41,
            matrix1.M42 - matrix2.M42,
            matrix1.M43 - matrix2.M43,
            matrix1.M44 - matrix2.M44,
        )
    }
    pub fn SubtractWithMatrix1AndMatrix2AndResult(
        matrix1: &mut Self,
        matrix2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Subtract(*matrix1, *matrix2);
    }
    #[must_use]
    pub fn Multiply(matrix1: Self, matrix2: Self) -> Self {
        let a = matrix1;
        let b = matrix2;
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
    pub fn MultiplyWithMatrix1AndMatrix2AndResult(
        matrix1: &mut Self,
        matrix2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Multiply(*matrix1, *matrix2);
    }
    #[must_use]
    pub fn MultiplyWithMatrix1AndScaleFactor(matrix1: Self, scaleFactor: f32) -> Self {
        let m = matrix1;
        let s = scaleFactor;
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
    pub fn MultiplyWithMatrix1AndScaleFactorAndResult(
        matrix1: &mut Self,
        scaleFactor: f32,
        result: &mut Self,
    ) {
        *result = Self::MultiplyWithMatrix1AndScaleFactor(*matrix1, scaleFactor);
    }

    #[must_use]
    pub fn Divide(matrix1: Self, matrix2: Self) -> Self {
        let a = matrix1.elements();
        let b = matrix2.elements();
        Self::new(
            a[0] / b[0],
            a[1] / b[1],
            a[2] / b[2],
            a[3] / b[3],
            a[4] / b[4],
            a[5] / b[5],
            a[6] / b[6],
            a[7] / b[7],
            a[8] / b[8],
            a[9] / b[9],
            a[10] / b[10],
            a[11] / b[11],
            a[12] / b[12],
            a[13] / b[13],
            a[14] / b[14],
            a[15] / b[15],
        )
    }
    pub fn DivideWithMatrix1AndMatrix2AndResult(
        matrix1: &mut Self,
        matrix2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Divide(*matrix1, *matrix2);
    }

    #[must_use]
    pub fn DivideWithMatrix1AndDivider(matrix1: Self, divider: f32) -> Self {
        let reciprocal = 1.0 / divider;
        Self::MultiplyWithMatrix1AndScaleFactor(matrix1, reciprocal)
    }
    pub fn DivideWithMatrix1AndDividerAndResult(
        matrix1: &mut Self,
        divider: f32,
        result: &mut Self,
    ) {
        *result = Self::DivideWithMatrix1AndDivider(*matrix1, divider);
    }
}
impl Default for Matrix {
    fn default() -> Self {
        Self::new(
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        )
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
impl Div for Matrix {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self::Divide(self, rhs)
    }
}
impl Div<f32> for Matrix {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::DivideWithMatrix1AndDivider(self, rhs)
    }
}
impl Neg for Matrix {
    type Output = Self;
    fn neg(self) -> Self {
        Self::Negate(self)
    }
}
