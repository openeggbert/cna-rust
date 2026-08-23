#![allow(
    non_snake_case,
    non_upper_case_globals,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::missing_panics_doc
)]

pub(crate) fn xna_f32_hash(value: f32) -> i32 {
    if value == 0.0 {
        0
    } else {
        value.to_bits() as i32
    }
}

pub(super) fn checked_transform_range(
    source_len: usize,
    source_index: i32,
    destination_len: usize,
    destination_index: i32,
    length: i32,
) -> (usize, usize, usize) {
    // XNA's range overloads intentionally perform no work when `length` is
    // negative; their `while (length > 0)` loop is the observable authority.
    if length <= 0 {
        return (0, 0, 0);
    }
    let source_index = usize::try_from(source_index).expect("source index must be nonnegative");
    let destination_index =
        usize::try_from(destination_index).expect("destination index must be nonnegative");
    let length = length as usize;
    assert!(
        source_index
            .checked_add(length)
            .is_some_and(|end| end <= source_len),
        "source array is too small"
    );
    assert!(
        destination_index
            .checked_add(length)
            .is_some_and(|end| end <= destination_len),
        "destination array is too small"
    );
    (source_index, destination_index, length)
}

macro_rules! vector_ops {
    ($type:ty, $multiply_scalar:ident, $divide_scalar:ident) => {
        impl Add for $type {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self::Add(self, rhs)
            }
        }
        impl Sub for $type {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self::Subtract(self, rhs)
            }
        }
        impl Mul for $type {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self {
                Self::Multiply(self, rhs)
            }
        }
        impl Mul<f32> for $type {
            type Output = Self;
            fn mul(self, rhs: f32) -> Self {
                Self::$multiply_scalar(self, rhs)
            }
        }
        impl Div for $type {
            type Output = Self;
            fn div(self, rhs: Self) -> Self {
                Self::Divide(self, rhs)
            }
        }
        impl Div<f32> for $type {
            type Output = Self;
            fn div(self, rhs: f32) -> Self {
                Self::$divide_scalar(self, rhs)
            }
        }
        impl Neg for $type {
            type Output = Self;
            fn neg(self) -> Self {
                Self::Negate(self)
            }
        }
        impl AddAssign for $type {
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }
        impl SubAssign for $type {
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }
        impl MulAssign<f32> for $type {
            fn mul_assign(&mut self, rhs: f32) {
                *self = *self * rhs;
            }
        }
        impl DivAssign<f32> for $type {
            fn div_assign(&mut self, rhs: f32) {
                *self = *self / rhs;
            }
        }
    };
}

pub(super) use vector_ops;
