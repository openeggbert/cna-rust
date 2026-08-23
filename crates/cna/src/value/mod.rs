//! Pure XNA value types, organized internally without changing the public XNA namespace.

mod color;
mod curve;
mod geometry;
mod math_helper;
mod matrix;
mod quaternion;
mod vector2;
mod vector3;
mod vector4;
pub(crate) mod vector_support;

pub use color::Color;
pub use curve::{
    Curve, CurveContinuity, CurveKey, CurveKeyCollection, CurveLoopType, CurveTangent,
};
pub use geometry::{
    BoundingBox, BoundingFrustum, BoundingSphere, ContainmentType, Plane, PlaneIntersectionType,
    Point, Ray, Rectangle,
};
pub use math_helper::MathHelper;
pub use matrix::Matrix;
pub use quaternion::Quaternion;
pub use vector2::Vector2;
pub use vector3::Vector3;
pub use vector4::Vector4;

#[cfg(test)]
mod tests;
