#![allow(non_snake_case, non_upper_case_globals)]

use core::any::Any;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Point {
    pub X: i32,
    pub Y: i32,
}

impl Point {
    pub const Zero: Self = Self::new(0, 0);
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { X: x, Y: y }
    }
    #[must_use]
    pub const fn Equals(&self, other: Self) -> bool {
        self.X == other.X && self.Y == other.Y
    }
    #[must_use]
    pub fn EqualsWithObj(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self.Equals(*other))
    }
    #[must_use]
    pub const fn GetHashCode(&self) -> i32 {
        self.X.wrapping_add(self.Y)
    }
    #[must_use]
    pub fn ToString(&self) -> String {
        format!("{{X:{} Y:{}}}", self.X, self.Y)
    }
}

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
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            X: x,
            Y: y,
            Width: width,
            Height: height,
        }
    }
    #[must_use]
    pub const fn Left(&self) -> i32 {
        self.X
    }
    #[must_use]
    pub const fn Right(&self) -> i32 {
        self.X.wrapping_add(self.Width)
    }
    #[must_use]
    pub const fn Top(&self) -> i32 {
        self.Y
    }
    #[must_use]
    pub const fn Bottom(&self) -> i32 {
        self.Y.wrapping_add(self.Height)
    }
    #[must_use]
    pub const fn Location(&self) -> Point {
        Point::new(self.X, self.Y)
    }
    pub fn SetLocation(&mut self, value: Point) {
        self.X = value.X;
        self.Y = value.Y;
    }
    #[must_use]
    pub const fn Center(&self) -> Point {
        Point::new(
            self.X.wrapping_add(self.Width / 2),
            self.Y.wrapping_add(self.Height / 2),
        )
    }
    #[must_use]
    pub const fn IsEmpty(&self) -> bool {
        self.Width == 0 && self.Height == 0 && self.X == 0 && self.Y == 0
    }
    pub fn Offset(&mut self, amount: Point) {
        self.X = self.X.wrapping_add(amount.X);
        self.Y = self.Y.wrapping_add(amount.Y);
    }
    pub fn OffsetWithOffsetXAndOffsetY(&mut self, offsetX: i32, offsetY: i32) {
        self.X = self.X.wrapping_add(offsetX);
        self.Y = self.Y.wrapping_add(offsetY);
    }
    pub fn Inflate(&mut self, horizontalAmount: i32, verticalAmount: i32) {
        self.X = self.X.wrapping_sub(horizontalAmount);
        self.Y = self.Y.wrapping_sub(verticalAmount);
        self.Width = self.Width.wrapping_add(horizontalAmount.wrapping_mul(2));
        self.Height = self.Height.wrapping_add(verticalAmount.wrapping_mul(2));
    }
    #[must_use]
    pub const fn Contains(&self, x: i32, y: i32) -> bool {
        self.X <= x && x < self.Right() && self.Y <= y && y < self.Bottom()
    }
    #[must_use]
    pub const fn ContainsWithValueAsPoint(&self, value: Point) -> bool {
        self.Contains(value.X, value.Y)
    }
    pub fn ContainsWithValueAndResultAsPointByRefAndBooleanByRef(
        &mut self,
        value: &mut Point,
        result: &mut bool,
    ) {
        *result = self.ContainsWithValueAsPoint(*value);
    }
    #[must_use]
    pub const fn ContainsWithValueAsRectangle(&self, value: Self) -> bool {
        self.X <= value.X
            && value.Right() <= self.Right()
            && self.Y <= value.Y
            && value.Bottom() <= self.Bottom()
    }
    pub fn ContainsWithValueAndResultAsRectangleByRefAndBooleanByRef(
        &mut self,
        value: &mut Self,
        result: &mut bool,
    ) {
        *result = self.ContainsWithValueAsRectangle(*value);
    }
    #[must_use]
    pub const fn Intersects(&self, value: Self) -> bool {
        value.X < self.Right()
            && self.X < value.Right()
            && value.Y < self.Bottom()
            && self.Y < value.Bottom()
    }
    pub fn IntersectsWithValueAndResult(&mut self, value: &mut Self, result: &mut bool) {
        *result = self.Intersects(*value);
    }
    #[must_use]
    pub const fn Intersect(value1: Self, value2: Self) -> Self {
        let right = if value1.Right() < value2.Right() {
            value1.Right()
        } else {
            value2.Right()
        };
        let bottom = if value1.Bottom() < value2.Bottom() {
            value1.Bottom()
        } else {
            value2.Bottom()
        };
        let left = if value1.X > value2.X {
            value1.X
        } else {
            value2.X
        };
        let top = if value1.Y > value2.Y {
            value1.Y
        } else {
            value2.Y
        };
        if right > left && bottom > top {
            Self::new(
                left,
                top,
                right.wrapping_sub(left),
                bottom.wrapping_sub(top),
            )
        } else {
            Self::Empty
        }
    }
    pub fn IntersectWithValue1AndValue2AndResult(
        value1: &mut Self,
        value2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Intersect(*value1, *value2);
    }
    #[must_use]
    pub const fn Union(value1: Self, value2: Self) -> Self {
        let right = if value1.Right() > value2.Right() {
            value1.Right()
        } else {
            value2.Right()
        };
        let bottom = if value1.Bottom() > value2.Bottom() {
            value1.Bottom()
        } else {
            value2.Bottom()
        };
        let left = if value1.X < value2.X {
            value1.X
        } else {
            value2.X
        };
        let top = if value1.Y < value2.Y {
            value1.Y
        } else {
            value2.Y
        };
        Self::new(
            left,
            top,
            right.wrapping_sub(left),
            bottom.wrapping_sub(top),
        )
    }
    pub fn UnionWithValue1AndValue2AndResult(
        value1: &mut Self,
        value2: &mut Self,
        result: &mut Self,
    ) {
        *result = Self::Union(*value1, *value2);
    }
    #[must_use]
    pub const fn Equals(&self, other: Self) -> bool {
        self.X == other.X
            && self.Y == other.Y
            && self.Width == other.Width
            && self.Height == other.Height
    }
    #[must_use]
    pub fn EqualsWithObj(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self.Equals(*other))
    }
    #[must_use]
    pub const fn GetHashCode(&self) -> i32 {
        self.X
            .wrapping_add(self.Y)
            .wrapping_add(self.Width)
            .wrapping_add(self.Height)
    }
    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "{{X:{} Y:{} Width:{} Height:{}}}",
            self.X, self.Y, self.Width, self.Height
        )
    }
}
