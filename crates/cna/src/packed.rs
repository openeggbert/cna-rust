#![allow(non_snake_case)]

use std::any::Any;

use crate::value::{Vector2, Vector3, Vector4};

/// Common XNA packed-vector conversion contract.
pub trait IPackedVector {
    fn ToVector4(&self) -> Vector4;
    fn PackFromVector4(&mut self, vector: Vector4);
}

/// Typed XNA packed-value property contract.
pub trait IPackedVectorOfT<TPacked>: IPackedVector {
    fn PackedValue(&self) -> TPacked;
    fn SetPackedValue(&mut self, value: TPacked);
}

fn round_ties_even(value: f32) -> f32 {
    let lower = value.floor();
    let fraction = value - lower;
    if fraction < 0.5 {
        lower
    } else if fraction > 0.5 {
        lower + 1.0
    } else if (lower as i64) & 1 == 0 {
        lower
    } else {
        lower + 1.0
    }
}

fn clamp_and_round(value: f32, min: f32, max: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else if value == f32::NEG_INFINITY || value < min {
        min
    } else if value == f32::INFINITY || value > max {
        max
    } else {
        round_ties_even(value)
    }
}

fn pack_unsigned(mask: u32, value: f32) -> u32 {
    clamp_and_round(value, 0.0, mask as f32) as u32
}

fn pack_signed(mask: u32, value: f32) -> u32 {
    let max = (mask >> 1) as f32;
    let min = -max - 1.0;
    (clamp_and_round(value, min, max) as i32 as u32) & mask
}

fn pack_unorm(mask: u32, value: f32) -> u32 {
    clamp_and_round(value * mask as f32, 0.0, mask as f32) as u32
}

fn unpack_unorm(mask: u32, value: u32) -> f32 {
    (value & mask) as f32 / mask as f32
}

fn pack_snorm(mask: u32, value: f32) -> u32 {
    let max = (mask >> 1) as f32;
    (clamp_and_round(value * max, -max, max) as i32 as u32) & mask
}

fn unpack_snorm(mask: u32, mut value: u32) -> f32 {
    let sign_bit = (mask + 1) >> 1;
    if value & sign_bit != 0 {
        if value & mask == sign_bit {
            return -1.0;
        }
        value |= !mask;
    } else {
        value &= mask;
    }
    value as i32 as f32 / (mask >> 1) as f32
}

fn pack_half(value: f32) -> u16 {
    const MAX_NORMAL: u32 = 1_207_955_455;
    const MIN_NORMAL: u32 = 947_912_704;

    let bits = value.to_bits();
    let sign = (bits & 0x8000_0000) >> 16;
    let mut magnitude = bits & 0x7fff_ffff;
    if magnitude > MAX_NORMAL {
        return (sign | 0x7fff) as u16;
    }
    if magnitude < MIN_NORMAL {
        let mantissa = (magnitude & 0x007f_ffff) | 0x0080_0000;
        let shift = 113 - (magnitude >> 23) as i32;
        magnitude = if shift <= 31 { mantissa >> shift } else { 0 };
        return (sign | ((magnitude + 4095 + ((magnitude >> 13) & 1)) >> 13)) as u16;
    }
    (sign | ((magnitude - 939_524_096 + 4095 + ((magnitude >> 13) & 1)) >> 13)) as u16
}

fn unpack_half(value: u16) -> f32 {
    let bits = if value & 0x7c00 == 0 {
        let mut mantissa = u32::from(value) & 0x03ff;
        if mantissa == 0 {
            u32::from(value & 0x8000) << 16
        } else {
            let mut exponent = -14;
            while mantissa & 0x0400 == 0 {
                exponent -= 1;
                mantissa <<= 1;
            }
            mantissa &= !0x0400;
            (u32::from(value & 0x8000) << 16) | (((exponent + 127) as u32) << 23) | (mantissa << 13)
        }
    } else {
        (u32::from(value & 0x8000) << 16)
            | (((i32::from((value >> 10) & 0x1f) - 15 + 127) as u32) << 23)
            | (u32::from(value & 0x03ff) << 13)
    };
    f32::from_bits(bits)
}

fn hash_u64(value: u64) -> i32 {
    ((value as u32) ^ (value >> 32) as u32) as i32
}

macro_rules! packed_value_type {
    ($name:ident, $storage:ty, $digits:literal, $hash:expr) => {
        #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct $name($storage);

        impl $name {
            pub fn PackedValue(&self) -> $storage {
                self.0
            }

            pub fn SetPackedValue(&mut self, value: $storage) {
                self.0 = value;
            }

            pub fn Equals(&self, obj: &dyn Any) -> bool {
                obj.downcast_ref::<Self>() == Some(self)
            }

            pub fn EqualsWithOther(&self, other: Self) -> bool {
                *self == other
            }

            pub fn GetHashCode(&self) -> i32 {
                ($hash)(self.0)
            }

            pub fn ToString(&self) -> String {
                format!(concat!("{:0", $digits, "X}"), self.0)
            }
        }

        impl IPackedVectorOfT<$storage> for $name {
            fn PackedValue(&self) -> $storage {
                self.0
            }

            fn SetPackedValue(&mut self, value: $storage) {
                self.0 = value;
            }
        }
    };
}

packed_value_type!(Alpha8, u8, 2, |value: u8| i32::from(value));

impl Alpha8 {
    pub fn new(alpha: f32) -> Self {
        Self(pack_unorm(255, alpha) as u8)
    }

    pub fn ToAlpha(&self) -> f32 {
        unpack_unorm(255, u32::from(self.0))
    }
}

impl IPackedVector for Alpha8 {
    fn ToVector4(&self) -> Vector4 {
        Vector4::from_x_and_y_and_z_and_w(0.0, 0.0, 0.0, self.ToAlpha())
    }

    fn PackFromVector4(&mut self, vector: Vector4) {
        self.0 = pack_unorm(255, vector.W) as u8;
    }
}

packed_value_type!(Bgr565, u16, 4, |value: u16| i32::from(value));

impl Bgr565 {
    pub fn new(vector: Vector3) -> Self {
        Self::from_x_and_y_and_z(vector.X, vector.Y, vector.Z)
    }

    pub fn from_x_and_y_and_z(x: f32, y: f32, z: f32) -> Self {
        Self(((pack_unorm(31, x) << 11) | (pack_unorm(63, y) << 5) | pack_unorm(31, z)) as u16)
    }

    pub fn ToVector3(&self) -> Vector3 {
        Vector3::from_x_and_y_and_z(
            unpack_unorm(31, u32::from(self.0) >> 11),
            unpack_unorm(63, u32::from(self.0) >> 5),
            unpack_unorm(31, u32::from(self.0)),
        )
    }
}

impl IPackedVector for Bgr565 {
    fn ToVector4(&self) -> Vector4 {
        let value = self.ToVector3();
        Vector4::from_x_and_y_and_z_and_w(value.X, value.Y, value.Z, 1.0)
    }

    fn PackFromVector4(&mut self, vector: Vector4) {
        *self = Self::from_x_and_y_and_z(vector.X, vector.Y, vector.Z);
    }
}

macro_rules! unorm4_16_type {
    ($name:ident, $r_bits:expr, $g_bits:expr, $b_bits:expr, $a_bits:expr,
     $r_shift:expr, $g_shift:expr, $b_shift:expr, $a_shift:expr) => {
        packed_value_type!($name, u16, 4, |value: u16| i32::from(value));

        impl $name {
            pub fn new(vector: Vector4) -> Self {
                Self::from_x_and_y_and_z_and_w(vector.X, vector.Y, vector.Z, vector.W)
            }

            pub fn from_x_and_y_and_z_and_w(x: f32, y: f32, z: f32, w: f32) -> Self {
                Self(
                    ((pack_unorm($r_bits, x) << $r_shift)
                        | (pack_unorm($g_bits, y) << $g_shift)
                        | (pack_unorm($b_bits, z) << $b_shift)
                        | (pack_unorm($a_bits, w) << $a_shift)) as u16,
                )
            }

            pub fn ToVector4(&self) -> Vector4 {
                Vector4::from_x_and_y_and_z_and_w(
                    unpack_unorm($r_bits, u32::from(self.0) >> $r_shift),
                    unpack_unorm($g_bits, u32::from(self.0) >> $g_shift),
                    unpack_unorm($b_bits, u32::from(self.0) >> $b_shift),
                    unpack_unorm($a_bits, u32::from(self.0) >> $a_shift),
                )
            }
        }

        impl IPackedVector for $name {
            fn ToVector4(&self) -> Vector4 {
                <$name>::ToVector4(self)
            }

            fn PackFromVector4(&mut self, vector: Vector4) {
                *self = Self::new(vector);
            }
        }
    };
}

unorm4_16_type!(Bgra4444, 15, 15, 15, 15, 8, 4, 0, 12);
unorm4_16_type!(Bgra5551, 31, 31, 31, 1, 10, 5, 0, 15);

packed_value_type!(Byte4, u32, 8, |value: u32| value as i32);

impl Byte4 {
    pub fn new(vector: Vector4) -> Self {
        Self::from_x_and_y_and_z_and_w(vector.X, vector.Y, vector.Z, vector.W)
    }

    pub fn from_x_and_y_and_z_and_w(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self(
            pack_unsigned(255, x)
                | (pack_unsigned(255, y) << 8)
                | (pack_unsigned(255, z) << 16)
                | (pack_unsigned(255, w) << 24),
        )
    }

    pub fn ToVector4(&self) -> Vector4 {
        Vector4::from_x_and_y_and_z_and_w(
            (self.0 & 0xff) as f32,
            ((self.0 >> 8) & 0xff) as f32,
            ((self.0 >> 16) & 0xff) as f32,
            ((self.0 >> 24) & 0xff) as f32,
        )
    }
}

impl IPackedVector for Byte4 {
    fn ToVector4(&self) -> Vector4 {
        Byte4::ToVector4(self)
    }

    fn PackFromVector4(&mut self, vector: Vector4) {
        *self = Self::new(vector);
    }
}

packed_value_type!(HalfSingle, u16, 4, |value: u16| i32::from(value));

impl HalfSingle {
    pub fn new(value: f32) -> Self {
        Self(pack_half(value))
    }

    pub fn ToSingle(&self) -> f32 {
        unpack_half(self.0)
    }
}

impl IPackedVector for HalfSingle {
    fn ToVector4(&self) -> Vector4 {
        Vector4::from_x_and_y_and_z_and_w(self.ToSingle(), 0.0, 0.0, 1.0)
    }

    fn PackFromVector4(&mut self, vector: Vector4) {
        self.0 = pack_half(vector.X);
    }
}

packed_value_type!(HalfVector2, u32, 8, |value: u32| value as i32);

impl HalfVector2 {
    pub fn new(vector: Vector2) -> Self {
        Self::from_x_and_y(vector.X, vector.Y)
    }

    pub fn from_x_and_y(x: f32, y: f32) -> Self {
        Self(u32::from(pack_half(x)) | (u32::from(pack_half(y)) << 16))
    }

    pub fn ToVector2(&self) -> Vector2 {
        Vector2::from_x_and_y(
            unpack_half(self.0 as u16),
            unpack_half((self.0 >> 16) as u16),
        )
    }
}

impl IPackedVector for HalfVector2 {
    fn ToVector4(&self) -> Vector4 {
        let value = self.ToVector2();
        Vector4::from_x_and_y_and_z_and_w(value.X, value.Y, 0.0, 1.0)
    }

    fn PackFromVector4(&mut self, vector: Vector4) {
        *self = Self::from_x_and_y(vector.X, vector.Y);
    }
}

packed_value_type!(HalfVector4, u64, 16, |value: u64| hash_u64(value));

impl HalfVector4 {
    pub fn new(vector: Vector4) -> Self {
        Self::from_x_and_y_and_z_and_w(vector.X, vector.Y, vector.Z, vector.W)
    }

    pub fn from_x_and_y_and_z_and_w(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self(
            u64::from(pack_half(x))
                | (u64::from(pack_half(y)) << 16)
                | (u64::from(pack_half(z)) << 32)
                | (u64::from(pack_half(w)) << 48),
        )
    }

    pub fn ToVector4(&self) -> Vector4 {
        Vector4::from_x_and_y_and_z_and_w(
            unpack_half(self.0 as u16),
            unpack_half((self.0 >> 16) as u16),
            unpack_half((self.0 >> 32) as u16),
            unpack_half((self.0 >> 48) as u16),
        )
    }
}

impl IPackedVector for HalfVector4 {
    fn ToVector4(&self) -> Vector4 {
        HalfVector4::ToVector4(self)
    }

    fn PackFromVector4(&mut self, vector: Vector4) {
        *self = Self::new(vector);
    }
}

macro_rules! packed2_type {
    ($name:ident, $storage:ty, $digits:literal, $pack:ident, $unpack:ident, $mask:expr) => {
        packed_value_type!($name, $storage, $digits, |value: $storage| value as i32);

        impl $name {
            pub fn new(vector: Vector2) -> Self {
                Self::from_x_and_y(vector.X, vector.Y)
            }

            pub fn from_x_and_y(x: f32, y: f32) -> Self {
                Self(($pack($mask, x) | ($pack($mask, y) << 16)) as $storage)
            }

            pub fn ToVector2(&self) -> Vector2 {
                Vector2::from_x_and_y(
                    $unpack($mask, self.0 as u32),
                    $unpack($mask, (self.0 as u32) >> 16),
                )
            }
        }

        impl IPackedVector for $name {
            fn ToVector4(&self) -> Vector4 {
                let value = self.ToVector2();
                Vector4::from_x_and_y_and_z_and_w(value.X, value.Y, 0.0, 1.0)
            }

            fn PackFromVector4(&mut self, vector: Vector4) {
                *self = Self::from_x_and_y(vector.X, vector.Y);
            }
        }
    };
}

packed2_type!(NormalizedShort2, u32, 8, pack_snorm, unpack_snorm, 65_535);
packed2_type!(Rg32, u32, 8, pack_unorm, unpack_unorm, 65_535);

packed_value_type!(NormalizedByte2, u16, 4, |value: u16| i32::from(value));

impl NormalizedByte2 {
    pub fn new(vector: Vector2) -> Self {
        Self::from_x_and_y(vector.X, vector.Y)
    }

    pub fn from_x_and_y(x: f32, y: f32) -> Self {
        Self((pack_snorm(255, x) | (pack_snorm(255, y) << 8)) as u16)
    }

    pub fn ToVector2(&self) -> Vector2 {
        Vector2::from_x_and_y(
            unpack_snorm(255, u32::from(self.0)),
            unpack_snorm(255, u32::from(self.0) >> 8),
        )
    }
}

impl IPackedVector for NormalizedByte2 {
    fn ToVector4(&self) -> Vector4 {
        let value = self.ToVector2();
        Vector4::from_x_and_y_and_z_and_w(value.X, value.Y, 0.0, 1.0)
    }

    fn PackFromVector4(&mut self, vector: Vector4) {
        *self = Self::from_x_and_y(vector.X, vector.Y);
    }
}

macro_rules! packed4_type {
    ($name:ident, $storage:ty, $digits:literal, $pack:ident, $unpack:ident, $mask:expr,
     $lane_bits:expr, $hash:expr) => {
        packed_value_type!($name, $storage, $digits, $hash);

        impl $name {
            pub fn new(vector: Vector4) -> Self {
                Self::from_x_and_y_and_z_and_w(vector.X, vector.Y, vector.Z, vector.W)
            }

            pub fn from_x_and_y_and_z_and_w(x: f32, y: f32, z: f32, w: f32) -> Self {
                Self(
                    (($pack($mask, x) as u64)
                        | (($pack($mask, y) as u64) << $lane_bits)
                        | (($pack($mask, z) as u64) << ($lane_bits * 2))
                        | (($pack($mask, w) as u64) << ($lane_bits * 3))) as $storage,
                )
            }

            pub fn ToVector4(&self) -> Vector4 {
                let value = self.0 as u64;
                Vector4::from_x_and_y_and_z_and_w(
                    $unpack($mask, value as u32),
                    $unpack($mask, (value >> $lane_bits) as u32),
                    $unpack($mask, (value >> ($lane_bits * 2)) as u32),
                    $unpack($mask, (value >> ($lane_bits * 3)) as u32),
                )
            }
        }

        impl IPackedVector for $name {
            fn ToVector4(&self) -> Vector4 {
                <$name>::ToVector4(self)
            }

            fn PackFromVector4(&mut self, vector: Vector4) {
                *self = Self::new(vector);
            }
        }
    };
}

packed4_type!(
    NormalizedByte4,
    u32,
    8,
    pack_snorm,
    unpack_snorm,
    255,
    8,
    |value: u32| value as i32
);
packed4_type!(
    NormalizedShort4,
    u64,
    16,
    pack_snorm,
    unpack_snorm,
    65_535,
    16,
    |value: u64| hash_u64(value)
);
packed4_type!(
    Rgba64,
    u64,
    16,
    pack_unorm,
    unpack_unorm,
    65_535,
    16,
    |value: u64| hash_u64(value)
);

packed_value_type!(Rgba1010102, u32, 8, |value: u32| value as i32);

impl Rgba1010102 {
    pub fn new(vector: Vector4) -> Self {
        Self::from_x_and_y_and_z_and_w(vector.X, vector.Y, vector.Z, vector.W)
    }

    pub fn from_x_and_y_and_z_and_w(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self(
            pack_unorm(1023, x)
                | (pack_unorm(1023, y) << 10)
                | (pack_unorm(1023, z) << 20)
                | (pack_unorm(3, w) << 30),
        )
    }

    pub fn ToVector4(&self) -> Vector4 {
        Vector4::from_x_and_y_and_z_and_w(
            unpack_unorm(1023, self.0),
            unpack_unorm(1023, self.0 >> 10),
            unpack_unorm(1023, self.0 >> 20),
            unpack_unorm(3, self.0 >> 30),
        )
    }
}

impl IPackedVector for Rgba1010102 {
    fn ToVector4(&self) -> Vector4 {
        Rgba1010102::ToVector4(self)
    }

    fn PackFromVector4(&mut self, vector: Vector4) {
        *self = Self::new(vector);
    }
}

packed_value_type!(Short2, u32, 8, |value: u32| value as i32);

impl Short2 {
    pub fn new(vector: Vector2) -> Self {
        Self::from_x_and_y(vector.X, vector.Y)
    }

    pub fn from_x_and_y(x: f32, y: f32) -> Self {
        Self(pack_signed(65_535, x) | (pack_signed(65_535, y) << 16))
    }

    pub fn ToVector2(&self) -> Vector2 {
        Vector2::from_x_and_y(
            self.0 as u16 as i16 as f32,
            (self.0 >> 16) as u16 as i16 as f32,
        )
    }
}

impl IPackedVector for Short2 {
    fn ToVector4(&self) -> Vector4 {
        let value = self.ToVector2();
        Vector4::from_x_and_y_and_z_and_w(value.X, value.Y, 0.0, 1.0)
    }

    fn PackFromVector4(&mut self, vector: Vector4) {
        *self = Self::from_x_and_y(vector.X, vector.Y);
    }
}

packed_value_type!(Short4, u64, 16, |value: u64| hash_u64(value));

impl Short4 {
    pub fn new(vector: Vector4) -> Self {
        Self::from_x_and_y_and_z_and_w(vector.X, vector.Y, vector.Z, vector.W)
    }

    pub fn from_x_and_y_and_z_and_w(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self(
            u64::from(pack_signed(65_535, x))
                | (u64::from(pack_signed(65_535, y)) << 16)
                | (u64::from(pack_signed(65_535, z)) << 32)
                | (u64::from(pack_signed(65_535, w)) << 48),
        )
    }

    pub fn ToVector4(&self) -> Vector4 {
        Vector4::from_x_and_y_and_z_and_w(
            self.0 as u16 as i16 as f32,
            (self.0 >> 16) as u16 as i16 as f32,
            (self.0 >> 32) as u16 as i16 as f32,
            (self.0 >> 48) as u16 as i16 as f32,
        )
    }
}

impl IPackedVector for Short4 {
    fn ToVector4(&self) -> Vector4 {
        Short4::ToVector4(self)
    }

    fn PackFromVector4(&mut self, vector: Vector4) {
        *self = Self::new(vector);
    }
}
