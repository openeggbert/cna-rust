#![allow(
    non_snake_case,
    non_upper_case_globals,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::missing_panics_doc
)]

use core::any::Any;
use core::ops::Mul;

use crate::packed::{IPackedVector, IPackedVectorOfT};

use super::{Vector3, Vector4};

/// XNA's packed little-endian RGBA color value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct Color {
    packed_value: u32,
}

impl Color {
    #[must_use]
    pub fn new(vector: Vector3) -> Self {
        Self::from_packed(Self::pack(vector.X, vector.Y, vector.Z, 1.0))
    }

    #[must_use]
    pub fn from_vector(vector: Vector4) -> Self {
        Self::from_packed(Self::pack(vector.X, vector.Y, vector.Z, vector.W))
    }

    #[must_use]
    pub fn from_r_and_g_and_b_as_int32_and_int32_and_int32(r: i32, g: i32, b: i32) -> Self {
        Self::from_rgba_i32(r, g, b, 255)
    }

    #[must_use]
    pub fn from_r_and_g_and_b_as_single_and_single_and_single(r: f32, g: f32, b: f32) -> Self {
        Self::from_packed(Self::pack(r, g, b, 1.0))
    }

    #[must_use]
    pub fn from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
        r: i32,
        g: i32,
        b: i32,
        a: i32,
    ) -> Self {
        Self::from_rgba_i32(r, g, b, a)
    }

    #[must_use]
    pub fn from_r_and_g_and_b_and_a_as_single_and_single_and_single_and_single(
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> Self {
        Self::from_packed(Self::pack(r, g, b, a))
    }

    #[must_use]
    pub const fn R(&self) -> u8 {
        self.packed_value as u8
    }

    pub fn SetR(&mut self, value: u8) {
        self.packed_value = (self.packed_value & 0xffff_ff00) | u32::from(value);
    }

    #[must_use]
    pub const fn G(&self) -> u8 {
        (self.packed_value >> 8) as u8
    }

    pub fn SetG(&mut self, value: u8) {
        self.packed_value = (self.packed_value & 0xffff_00ff) | (u32::from(value) << 8);
    }

    #[must_use]
    pub const fn B(&self) -> u8 {
        (self.packed_value >> 16) as u8
    }

    pub fn SetB(&mut self, value: u8) {
        self.packed_value = (self.packed_value & 0xff00_ffff) | (u32::from(value) << 16);
    }

    #[must_use]
    pub const fn A(&self) -> u8 {
        (self.packed_value >> 24) as u8
    }

    pub fn SetA(&mut self, value: u8) {
        self.packed_value = (self.packed_value & 0x00ff_ffff) | (u32::from(value) << 24);
    }

    #[must_use]
    pub const fn PackedValue(&self) -> u32 {
        self.packed_value
    }

    pub fn SetPackedValue(&mut self, value: u32) {
        self.packed_value = value;
    }

    #[must_use]
    pub fn FromNonPremultiplied(vector: Vector4) -> Self {
        Self::from_packed(Self::pack(
            vector.X * vector.W,
            vector.Y * vector.W,
            vector.Z * vector.W,
            vector.W,
        ))
    }

    #[must_use]
    pub fn FromNonPremultipliedWithRAndGAndBAndA(r: i32, g: i32, b: i32, a: i32) -> Self {
        let r = Self::clamp_i64(i64::from(r) * i64::from(a) / 255);
        let g = Self::clamp_i64(i64::from(g) * i64::from(a) / 255);
        let b = Self::clamp_i64(i64::from(b) * i64::from(a) / 255);
        let a = Self::clamp_i32(a);
        Self::from_packed(r | (g << 8) | (b << 16) | (a << 24))
    }

    #[must_use]
    pub fn ToVector3(&self) -> Vector3 {
        let scale = 1.0 / 255.0;
        Vector3::from_x_and_y_and_z(
            f32::from(self.R()) * scale,
            f32::from(self.G()) * scale,
            f32::from(self.B()) * scale,
        )
    }

    #[must_use]
    pub fn ToVector4(&self) -> Vector4 {
        let scale = 1.0 / 255.0;
        Vector4::from_x_and_y_and_z_and_w(
            f32::from(self.R()) * scale,
            f32::from(self.G()) * scale,
            f32::from(self.B()) * scale,
            f32::from(self.A()) * scale,
        )
    }

    #[must_use]
    pub fn Lerp(value1: Self, value2: Self, amount: f32) -> Self {
        let factor = Self::pack_unorm(65_536.0, amount) as i32;
        let interpolate = |start: u8, end: u8| {
            i32::from(start) + (((i32::from(end) - i32::from(start)) * factor) >> 16)
        };
        let r = interpolate(value1.R(), value2.R());
        let g = interpolate(value1.G(), value2.G());
        let b = interpolate(value1.B(), value2.B());
        let a = interpolate(value1.A(), value2.A());
        Self::from_packed((r | (g << 8) | (b << 16) | (a << 24)) as u32)
    }

    #[must_use]
    pub fn Multiply(value: Self, scale: f32) -> Self {
        let scaled = scale * 65_536.0;
        let factor = if scaled < 0.0 {
            0
        } else if scaled > 16_777_215.0 {
            16_777_215
        } else {
            scaled as u32
        };
        let channel = |component: u8| ((u32::from(component) * factor) >> 16).min(255);
        Self::from_packed(
            channel(value.R())
                | (channel(value.G()) << 8)
                | (channel(value.B()) << 16)
                | (channel(value.A()) << 24),
        )
    }

    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "{{R:{} G:{} B:{} A:{}}}",
            self.R(),
            self.G(),
            self.B(),
            self.A()
        )
    }

    #[must_use]
    pub const fn GetHashCode(&self) -> i32 {
        self.packed_value as i32
    }

    #[must_use]
    pub fn Equals(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>() == Some(self)
    }

    #[must_use]
    pub const fn EqualsWithOther(&self, other: Self) -> bool {
        self.packed_value == other.packed_value
    }

    const fn from_packed(packed_value: u32) -> Self {
        Self { packed_value }
    }

    fn from_rgba_i32(r: i32, g: i32, b: i32, a: i32) -> Self {
        Self::from_packed(
            Self::clamp_i32(r)
                | (Self::clamp_i32(g) << 8)
                | (Self::clamp_i32(b) << 16)
                | (Self::clamp_i32(a) << 24),
        )
    }

    fn clamp_i32(value: i32) -> u32 {
        value.clamp(0, 255) as u32
    }

    fn clamp_i64(value: i64) -> u32 {
        value.clamp(0, 255) as u32
    }

    fn pack(r: f32, g: f32, b: f32, a: f32) -> u32 {
        Self::pack_unorm(255.0, r)
            | (Self::pack_unorm(255.0, g) << 8)
            | (Self::pack_unorm(255.0, b) << 16)
            | (Self::pack_unorm(255.0, a) << 24)
    }

    fn pack_unorm(bitmask: f32, value: f32) -> u32 {
        let scaled = value * bitmask;
        if scaled.is_nan() || scaled <= 0.0 {
            return 0;
        }
        if scaled >= bitmask {
            return bitmask as u32;
        }
        let floor = scaled.floor();
        let fraction = scaled - floor;
        let rounded = if fraction < 0.5 || (fraction == 0.5 && floor as u32 % 2 == 0) {
            floor
        } else {
            floor + 1.0
        };
        rounded as u32
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Multiply(self, rhs)
    }
}

impl IPackedVector for Color {
    fn ToVector4(&self) -> Vector4 {
        Self::ToVector4(self)
    }

    fn PackFromVector4(&mut self, vector: Vector4) {
        self.packed_value = Self::pack(vector.X, vector.Y, vector.Z, vector.W);
    }
}

impl IPackedVectorOfT<u32> for Color {
    fn PackedValue(&self) -> u32 {
        Self::PackedValue(self)
    }

    fn SetPackedValue(&mut self, value: u32) {
        Self::SetPackedValue(self, value);
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::Transparent
    }
}

// Generated from the decompiled code of the pinned Microsoft.Xna.Framework.dll
// SHA-256 38e7093f52d7474bbc6256906519781a1210d7da50a1c667b52716fcf49ca130.
macro_rules! xna_named_colors {
    ($($name:ident = $packed:expr),+ $(,)?) => {
        impl Color {
            $(pub const $name: Self = Self::from_packed($packed);)+
        }

        #[cfg(test)]
        pub(super) const XNA_NAMED_COLORS: &[(&str, Color, u32)] = &[
            $((stringify!($name), Color::$name, $packed)),+
        ];
    };
}

xna_named_colors! {
    Transparent = 0,
    AliceBlue = 4_294_965_488,
    AntiqueWhite = 4_292_340_730,
    Aqua = 4_294_967_040,
    Aquamarine = 4_292_149_119,
    Azure = 4_294_967_280,
    Beige = 4_292_670_965,
    Bisque = 4_291_093_759,
    Black = 4_278_190_080,
    BlanchedAlmond = 4_291_685_375,
    Blue = 4_294_901_760,
    BlueViolet = 4_293_012_362,
    Brown = 4_280_953_509,
    BurlyWood = 4_287_084_766,
    CadetBlue = 4_288_716_383,
    Chartreuse = 4_278_255_487,
    Chocolate = 4_280_183_250,
    Coral = 4_283_465_727,
    CornflowerBlue = 4_293_760_356,
    Cornsilk = 4_292_671_743,
    Crimson = 4_282_127_580,
    Cyan = 4_294_967_040,
    DarkBlue = 4_287_299_584,
    DarkCyan = 4_287_335_168,
    DarkGoldenrod = 4_278_945_464,
    DarkGray = 4_289_309_097,
    DarkGreen = 4_278_215_680,
    DarkKhaki = 4_285_249_469,
    DarkMagenta = 4_287_299_723,
    DarkOliveGreen = 4_281_297_749,
    DarkOrange = 4_278_226_175,
    DarkOrchid = 4_291_572_377,
    DarkRed = 4_278_190_219,
    DarkSalmon = 4_286_224_105,
    DarkSeaGreen = 4_287_347_855,
    DarkSlateBlue = 4_287_315_272,
    DarkSlateGray = 4_283_387_695,
    DarkTurquoise = 4_291_939_840,
    DarkViolet = 4_292_018_324,
    DeepPink = 4_287_829_247,
    DeepSkyBlue = 4_294_950_656,
    DimGray = 4_285_098_345,
    DodgerBlue = 4_294_938_654,
    Firebrick = 4_280_427_186,
    FloralWhite = 4_293_982_975,
    ForestGreen = 4_280_453_922,
    Fuchsia = 4_294_902_015,
    Gainsboro = 4_292_664_540,
    GhostWhite = 4_294_965_496,
    Gold = 4_278_245_375,
    Goldenrod = 4_280_329_690,
    Gray = 4_286_611_584,
    Green = 4_278_222_848,
    GreenYellow = 4_281_335_725,
    Honeydew = 4_293_984_240,
    HotPink = 4_290_013_695,
    IndianRed = 4_284_243_149,
    Indigo = 4_286_709_835,
    Ivory = 4_293_984_255,
    Khaki = 4_287_424_240,
    Lavender = 4_294_633_190,
    LavenderBlush = 4_294_308_095,
    LawnGreen = 4_278_254_716,
    LemonChiffon = 4_291_689_215,
    LightBlue = 4_293_318_829,
    LightCoral = 4_286_611_696,
    LightCyan = 4_294_967_264,
    LightGoldenrodYellow = 4_292_016_890,
    LightGreen = 4_287_688_336,
    LightGray = 4_292_072_403,
    LightPink = 4_290_885_375,
    LightSalmon = 4_286_226_687,
    LightSeaGreen = 4_289_376_800,
    LightSkyBlue = 4_294_626_951,
    LightSlateGray = 4_288_252_023,
    LightSteelBlue = 4_292_789_424,
    LightYellow = 4_292_935_679,
    Lime = 4_278_255_360,
    LimeGreen = 4_281_519_410,
    Linen = 4_293_325_050,
    Magenta = 4_294_902_015,
    Maroon = 4_278_190_208,
    MediumAquamarine = 4_289_383_782,
    MediumBlue = 4_291_624_960,
    MediumOrchid = 4_292_040_122,
    MediumPurple = 4_292_571_283,
    MediumSeaGreen = 4_285_641_532,
    MediumSlateBlue = 4_293_814_395,
    MediumSpringGreen = 4_288_346_624,
    MediumTurquoise = 4_291_613_000,
    MediumVioletRed = 4_286_911_943,
    MidnightBlue = 4_285_536_537,
    MintCream = 4_294_639_605,
    MistyRose = 4_292_994_303,
    Moccasin = 4_290_110_719,
    NavajoWhite = 4_289_584_895,
    Navy = 4_286_578_688,
    OldLace = 4_293_326_333,
    Olive = 4_278_222_976,
    OliveDrab = 4_280_520_299,
    Orange = 4_278_232_575,
    OrangeRed = 4_278_207_999,
    Orchid = 4_292_243_674,
    PaleGoldenrod = 4_289_390_830,
    PaleGreen = 4_288_215_960,
    PaleTurquoise = 4_293_848_751,
    PaleVioletRed = 4_287_852_763,
    PapayaWhip = 4_292_210_687,
    PeachPuff = 4_290_370_303,
    Peru = 4_282_353_101,
    Pink = 4_291_543_295,
    Plum = 4_292_714_717,
    PowderBlue = 4_293_320_880,
    Purple = 4_286_578_816,
    Red = 4_278_190_335,
    RosyBrown = 4_287_598_524,
    RoyalBlue = 4_292_962_625,
    SaddleBrown = 4_279_453_067,
    Salmon = 4_285_694_202,
    SandyBrown = 4_284_523_764,
    SeaGreen = 4_283_927_342,
    SeaShell = 4_293_850_623,
    Sienna = 4_281_160_352,
    Silver = 4_290_822_336,
    SkyBlue = 4_293_643_911,
    SlateBlue = 4_291_648_106,
    SlateGray = 4_287_660_144,
    Snow = 4_294_638_335,
    SpringGreen = 4_286_578_432,
    SteelBlue = 4_290_019_910,
    Tan = 4_287_411_410,
    Teal = 4_286_611_456,
    Thistle = 4_292_394_968,
    Tomato = 4_282_868_735,
    Turquoise = 4_291_878_976,
    Violet = 4_293_821_166,
    Wheat = 4_289_978_101,
    White = u32::MAX,
    WhiteSmoke = 4_294_309_365,
    Yellow = 4_278_255_615,
    YellowGreen = 4_281_519_514,
}
