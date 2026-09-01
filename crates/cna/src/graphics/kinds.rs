#![allow(non_upper_case_globals)]

use core::ops::{BitAnd, BitOr, BitOrAssign};

macro_rules! xna_enum {
    ($name:ident { $($variant:ident = $value:expr),+ $(,)? }) => {
        #[repr(i32)]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum $name {
            $($variant = $value),+
        }

        impl $name {
            /// The member a native value names, or `None` for one this enum has
            /// no member for.
            ///
            /// `pub(crate)` and `const`, so the crate's own decoding stays a
            /// compile-time conversion. The public spelling is the trait
            /// implementation below: this is a CNA concept, and a strict XNA
            /// enum may not carry one inherently.
            #[must_use]
            pub(crate) const fn from_native_value(value: u32) -> Option<Self> {
                $(if value == $value as u32 { return Some(Self::$variant); })+
                None
            }
        }

        impl crate::extensions::graphics::NativeEnumValue for $name {
            fn from_native_value(value: u32) -> Option<Self> {
                // The inherent `const fn` above; an inherent associated
                // function is resolved before a trait one.
                Self::from_native_value(value)
            }
        }
    };
}

xna_enum!(Blend {
    One = 0,
    Zero = 1,
    SourceColor = 2,
    InverseSourceColor = 3,
    SourceAlpha = 4,
    InverseSourceAlpha = 5,
    DestinationColor = 6,
    InverseDestinationColor = 7,
    DestinationAlpha = 8,
    InverseDestinationAlpha = 9,
    BlendFactor = 10,
    InverseBlendFactor = 11,
    SourceAlphaSaturation = 12,
});

xna_enum!(BlendFunction {
    Add = 0,
    Subtract = 1,
    ReverseSubtract = 2,
    Min = 3,
    Max = 4,
});

xna_enum!(CompareFunction {
    Always = 0,
    Never = 1,
    Less = 2,
    LessEqual = 3,
    Equal = 4,
    GreaterEqual = 5,
    Greater = 6,
    NotEqual = 7,
});

xna_enum!(StencilOperation {
    Keep = 0,
    Zero = 1,
    Replace = 2,
    Increment = 3,
    Decrement = 4,
    IncrementSaturation = 5,
    DecrementSaturation = 6,
    Invert = 7,
});

xna_enum!(CullMode {
    None = 0,
    CullClockwiseFace = 1,
    CullCounterClockwiseFace = 2,
});

xna_enum!(FillMode {
    Solid = 0,
    WireFrame = 1,
});

xna_enum!(DepthFormat {
    None = 0,
    Depth16 = 1,
    Depth24 = 2,
    Depth24Stencil8 = 3,
});

xna_enum!(GraphicsDeviceStatus {
    Normal = 0,
    Lost = 1,
    NotReset = 2,
});

xna_enum!(GraphicsProfile {
    Reach = 0,
    HiDef = 1,
});

xna_enum!(PresentInterval {
    Default = 0,
    One = 1,
    Two = 2,
    Immediate = 3,
});

xna_enum!(RenderTargetUsage {
    DiscardContents = 0,
    PreserveContents = 1,
    PlatformContents = 2,
});

xna_enum!(CubeMapFace {
    PositiveX = 0,
    NegativeX = 1,
    PositiveY = 2,
    NegativeY = 3,
    PositiveZ = 4,
    NegativeZ = 5,
});

xna_enum!(IndexElementSize {
    SixteenBits = 0,
    ThirtyTwoBits = 1,
});

xna_enum!(PrimitiveType {
    TriangleList = 0,
    TriangleStrip = 1,
    LineList = 2,
    LineStrip = 3,
});

xna_enum!(VertexElementFormat {
    Single = 0,
    Vector2 = 1,
    Vector3 = 2,
    Vector4 = 3,
    Color = 4,
    Byte4 = 5,
    Short2 = 6,
    Short4 = 7,
    NormalizedShort2 = 8,
    NormalizedShort4 = 9,
    HalfVector2 = 10,
    HalfVector4 = 11,
});

xna_enum!(VertexElementUsage {
    Position = 0,
    Color = 1,
    TextureCoordinate = 2,
    Normal = 3,
    Binormal = 4,
    Tangent = 5,
    BlendIndices = 6,
    BlendWeight = 7,
    Depth = 8,
    Fog = 9,
    PointSize = 10,
    Sample = 11,
    TessellateFactor = 12,
});

xna_enum!(TextureAddressMode {
    Wrap = 0,
    Clamp = 1,
    Mirror = 2,
});

xna_enum!(TextureFilter {
    Linear = 0,
    Point = 1,
    Anisotropic = 2,
    LinearMipPoint = 3,
    PointMipLinear = 4,
    MinLinearMagPointMipLinear = 5,
    MinLinearMagPointMipPoint = 6,
    MinPointMagLinearMipLinear = 7,
    MinPointMagLinearMipPoint = 8,
});

/// Open flags representation of XNA's `ColorWriteChannels`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ColorWriteChannels(i32);

impl ColorWriteChannels {
    pub const None: Self = Self(0);
    pub const Red: Self = Self(1);
    pub const Green: Self = Self(2);
    pub const Blue: Self = Self(4);
    pub const Alpha: Self = Self(8);
    pub const All: Self = Self(15);

    pub(crate) const fn bits(self) -> u32 {
        u32::from_ne_bytes(self.0.to_ne_bytes())
    }

    pub(super) const fn from_bits(value: u32) -> Self {
        Self(i32::from_ne_bytes(value.to_ne_bytes()))
    }
}

impl BitOr for ColorWriteChannels {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ColorWriteChannels {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for ColorWriteChannels {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

/// Open flags representation of XNA's `ClearOptions`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ClearOptions(i32);

impl ClearOptions {
    pub const Target: Self = Self(1);
    pub const DepthBuffer: Self = Self(2);
    pub const Stencil: Self = Self(4);

    pub(crate) const fn bits(self) -> u32 {
        u32::from_ne_bytes(self.0.to_ne_bytes())
    }
}

macro_rules! open_graphics_flags {
    ($name:ident { $($constant:ident = $value:expr),+ $(,)? }) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
        pub struct $name(i32);

        impl $name {
            $(pub const $constant: Self = Self($value);)+

            pub(crate) const fn bits(self) -> u32 {
                u32::from_ne_bytes(self.0.to_ne_bytes())
            }
        }

        impl BitOr for $name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl BitAnd for $name {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }
    };
}

open_graphics_flags!(BufferUsage {
    None = 0,
    WriteOnly = 1,
});

open_graphics_flags!(SetDataOptions {
    None = 0,
    Discard = 1,
    NoOverwrite = 2,
});

impl BitOr for ClearOptions {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ClearOptions {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for ClearOptions {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

/// Exact XNA 4.0 `SpriteBatch` ordering modes.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SpriteSortMode {
    #[default]
    Deferred = 0,
    Immediate = 1,
    Texture = 2,
    BackToFront = 3,
    FrontToBack = 4,
}

/// Exact XNA 4.0 sprite-flip flags.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SpriteEffects(i32);

impl SpriteEffects {
    pub const None: Self = Self(0);
    pub const FlipHorizontally: Self = Self(1);
    pub const FlipVertically: Self = Self(2);

    pub(crate) const fn bits(self) -> u32 {
        u32::from_ne_bytes(self.0.to_ne_bytes())
    }
}

impl BitOr for SpriteEffects {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for SpriteEffects {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for SpriteEffects {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

/// Exact XNA 4.0 surface-format identities.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SurfaceFormat {
    #[default]
    Color = 0,
    Bgr565 = 1,
    Bgra5551 = 2,
    Bgra4444 = 3,
    Dxt1 = 4,
    Dxt3 = 5,
    Dxt5 = 6,
    NormalizedByte2 = 7,
    NormalizedByte4 = 8,
    Rgba1010102 = 9,
    Rg32 = 10,
    Rgba64 = 11,
    Alpha8 = 12,
    Single = 13,
    Vector2 = 14,
    Vector4 = 15,
    HalfSingle = 16,
    HalfVector2 = 17,
    HalfVector4 = 18,
    HdrBlendable = 19,
}

impl SurfaceFormat {
    pub(crate) fn from_native(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::Color,
            1 => Self::Bgr565,
            2 => Self::Bgra5551,
            3 => Self::Bgra4444,
            4 => Self::Dxt1,
            5 => Self::Dxt3,
            6 => Self::Dxt5,
            7 => Self::NormalizedByte2,
            8 => Self::NormalizedByte4,
            9 => Self::Rgba1010102,
            10 => Self::Rg32,
            11 => Self::Rgba64,
            12 => Self::Alpha8,
            13 => Self::Single,
            14 => Self::Vector2,
            15 => Self::Vector4,
            16 => Self::HalfSingle,
            17 => Self::HalfVector2,
            18 => Self::HalfVector4,
            19 => Self::HdrBlendable,
            _ => return None,
        })
    }
}
