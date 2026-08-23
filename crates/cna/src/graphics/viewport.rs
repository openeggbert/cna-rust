#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use cna_sys as sys;

use crate::value::{Matrix, Rectangle, Vector3};

/// XNA viewport value with property-shaped accessors.
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Viewport {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    min_depth: f32,
    max_depth: f32,
}

#[allow(non_snake_case)]
impl Viewport {
    #[must_use]
    pub const fn new(bounds: Rectangle) -> Self {
        Self {
            x: bounds.X,
            y: bounds.Y,
            width: bounds.Width,
            height: bounds.Height,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }

    #[must_use]
    pub const fn from_x_and_y_and_width_and_height(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Self {
        Self::new(Rectangle::new(x, y, width, height))
    }

    pub(super) fn from_native(value: sys::CNA_Viewport) -> Self {
        Self {
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
            min_depth: value.min_depth,
            max_depth: value.max_depth,
        }
    }

    pub(super) const fn to_native(self) -> sys::CNA_Viewport {
        sys::CNA_Viewport {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            min_depth: self.min_depth,
            max_depth: self.max_depth,
        }
    }

    #[must_use]
    pub const fn X(&self) -> i32 {
        self.x
    }
    pub fn SetX(&mut self, value: i32) {
        self.x = value;
    }
    #[must_use]
    pub const fn Y(&self) -> i32 {
        self.y
    }
    pub fn SetY(&mut self, value: i32) {
        self.y = value;
    }
    #[must_use]
    pub const fn Width(&self) -> i32 {
        self.width
    }
    pub fn SetWidth(&mut self, value: i32) {
        self.width = value;
    }
    #[must_use]
    pub const fn Height(&self) -> i32 {
        self.height
    }
    pub fn SetHeight(&mut self, value: i32) {
        self.height = value;
    }
    #[must_use]
    pub const fn MinDepth(&self) -> f32 {
        self.min_depth
    }
    pub fn SetMinDepth(&mut self, value: f32) {
        self.min_depth = value;
    }
    #[must_use]
    pub const fn MaxDepth(&self) -> f32 {
        self.max_depth
    }
    pub fn SetMaxDepth(&mut self, value: f32) {
        self.max_depth = value;
    }
    #[must_use]
    pub const fn Bounds(&self) -> Rectangle {
        Rectangle::new(self.x, self.y, self.width, self.height)
    }
    pub fn SetBounds(&mut self, value: Rectangle) {
        self.x = value.X;
        self.y = value.Y;
        self.width = value.Width;
        self.height = value.Height;
    }
    #[must_use]
    pub fn AspectRatio(&self) -> f32 {
        if self.width == 0 || self.height == 0 {
            0.0
        } else {
            self.width as f32 / self.height as f32
        }
    }

    #[must_use]
    pub const fn TitleSafeArea(&self) -> Rectangle {
        self.Bounds()
    }

    fn within_epsilon(a: f32, b: f32) -> bool {
        let difference = a - b;
        -f32::from_bits(1) <= difference && difference <= f32::from_bits(1)
    }

    #[must_use]
    pub fn Project(
        &self,
        source: Vector3,
        projection: Matrix,
        view: Matrix,
        world: Matrix,
    ) -> Vector3 {
        let matrix = Matrix::Multiply(Matrix::Multiply(world, view), projection);
        let mut result = Vector3::Transform(source, matrix);
        let divisor =
            source.X * matrix.M14 + source.Y * matrix.M24 + source.Z * matrix.M34 + matrix.M44;
        if !Self::within_epsilon(divisor, 1.0) {
            result /= divisor;
        }
        result.X = (result.X + 1.0) * 0.5 * self.width as f32 + self.x as f32;
        result.Y = (0.0 - result.Y + 1.0) * 0.5 * self.height as f32 + self.y as f32;
        result.Z = result.Z * (self.max_depth - self.min_depth) + self.min_depth;
        result
    }

    #[must_use]
    pub fn Unproject(
        &self,
        mut source: Vector3,
        projection: Matrix,
        view: Matrix,
        world: Matrix,
    ) -> Vector3 {
        let matrix = Matrix::Invert(Matrix::Multiply(Matrix::Multiply(world, view), projection));
        source.X = (source.X - self.x as f32) / self.width as f32 * 2.0 - 1.0;
        source.Y = 0.0 - ((source.Y - self.y as f32) / self.height as f32 * 2.0 - 1.0);
        source.Z = (source.Z - self.min_depth) / (self.max_depth - self.min_depth);
        let mut result = Vector3::Transform(source, matrix);
        let divisor =
            source.X * matrix.M14 + source.Y * matrix.M24 + source.Z * matrix.M34 + matrix.M44;
        if !Self::within_epsilon(divisor, 1.0) {
            result /= divisor;
        }
        result
    }

    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "{{X:{} Y:{} Width:{} Height:{} MinDepth:{} MaxDepth:{}}}",
            self.X(),
            self.Y(),
            self.Width(),
            self.Height(),
            self.MinDepth(),
            self.MaxDepth()
        )
    }
}
