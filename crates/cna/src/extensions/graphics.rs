//! CNA graphics facts and construction routes XNA 4.0 does not declare.

#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use std::sync::Arc;

use crate::error::Result;
use crate::graphics::{
    Effect, EffectAnnotation, EffectAnnotationCollection, EffectParameter,
    EffectParameterCollection, EffectParameterDescriptor, EffectPass, EffectPassCollection,
    EffectTechnique, EffectTechniqueCollection, EffectTechniqueDescriptor, GraphicsDevice,
    ModelBone, ModelBoneCollection, ModelEffectCollection, ModelMesh, ModelMeshCollection,
    ModelMeshPart, ModelMeshPartCollection, SurfaceFormat,
};
use crate::native::Native;

/// Renderer facts queried from CNA rather than inferred from a name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RendererInfo {
    pub name: String,
    pub supports_3d: bool,
    pub supports_depth_stencil: bool,
    pub max_texture_dimension: u32,
}

/// CNA renderer diagnostics for a strict XNA `GraphicsDevice`.
pub trait RendererInfoExt {
    /// Queries CNA's active native renderer.
    ///
    /// # Errors
    ///
    /// Returns the exact error reported by CNA.
    fn renderer_info(&self) -> Result<RendererInfo>;
}

impl RendererInfoExt for GraphicsDevice {
    fn renderer_info(&self) -> Result<RendererInfo> {
        let (name, supports_3d, supports_depth_stencil, max_texture_dimension) =
            self.renderer_info()?;
        Ok(RendererInfo {
            name,
            supports_3d,
            supports_depth_stencil,
            max_texture_dimension,
        })
    }
}

/// CNA's unquantized color clear.
///
/// XNA has no equivalent. `GraphicsDevice.Clear(ClearOptions, Vector4, ...)`
/// looks like a floating-point clear but its first statement is
/// `new Color(color)`, so every strict overload reaches the device with eight
/// bits per channel. CNA also publishes a clear that keeps the four `f32`
/// channels, and this is that route.
pub trait FloatClearExt {
    /// Clears the color target from four finite linear channels.
    ///
    /// This clears the color target only; it carries no depth or stencil
    /// value, which is why it is not a spelling of an XNA overload.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, including the argument error for a
    /// non-finite channel and the backend's refusal when it cannot clear.
    fn clear_color_channels(&self, rgba: [f32; 4]) -> Result<()>;
}

impl FloatClearExt for GraphicsDevice {
    fn clear_color_channels(&self, rgba: [f32; 4]) -> Result<()> {
        GraphicsDevice::clear_color_channels(self, rgba)
    }
}

/// Inherited read-only collection operations for XNA model graph views.
#[allow(non_snake_case)]
pub trait ModelCollectionExt<T: ?Sized> {
    fn Count(&self) -> Result<i32>;
    fn ItemAt(&self, index: i32) -> Result<Arc<T>>;
}

macro_rules! model_collection_ext {
    ($collection:ty, $item:ty) => {
        impl ModelCollectionExt<$item> for $collection {
            fn Count(&self) -> Result<i32> {
                i32::try_from(self.count()).map_err(|_| {
                    crate::CnaError::InvalidInput("model collection count exceeds i32")
                })
            }

            fn ItemAt(&self, index: i32) -> Result<Arc<$item>> {
                let index = usize::try_from(index).map_err(|_| {
                    crate::CnaError::InvalidInput(
                        "model collection index must not be negative",
                    )
                })?;
                self.item_at(index)
            }
        }
    };
}

model_collection_ext!(ModelBoneCollection, ModelBone);
model_collection_ext!(ModelMeshCollection, ModelMesh);
model_collection_ext!(ModelMeshPartCollection, ModelMeshPart);

impl ModelCollectionExt<dyn crate::graphics::EffectBase> for ModelEffectCollection {
    fn Count(&self) -> Result<i32> {
        i32::try_from(self.count()?)
            .map_err(|_| crate::CnaError::InvalidInput("model effect count exceeds i32"))
    }

    fn ItemAt(&self, index: i32) -> Result<Arc<dyn crate::graphics::EffectBase>> {
        let index = usize::try_from(index).map_err(|_| {
            crate::CnaError::InvalidInput("model collection index must not be negative")
        })?;
        self.item_at(index)
    }
}

/// CNA construction support for a reflection-capable empty Effect.
///
/// This is intentionally outside XNA's namespace: XNA's public Effect
/// constructor accepts compiled bytecode, while CNA's empty graph is a
/// useful native integration and custom tooling primitive.
pub trait EffectFactoryExt {
    fn create_empty_effect(&self) -> Result<Effect>;
    fn create_reflection_effect(
        &self,
        parameters: &[EffectParameterDescriptor],
        techniques: &[EffectTechniqueDescriptor],
    ) -> Result<Effect>;
}

impl EffectFactoryExt for GraphicsDevice {
    fn create_empty_effect(&self) -> Result<Effect> {
        Effect::create_empty(self)
    }

    fn create_reflection_effect(
        &self,
        parameters: &[EffectParameterDescriptor],
        techniques: &[EffectTechniqueDescriptor],
    ) -> Result<Effect> {
        Effect::create_reflection(self, parameters, techniques)
    }
}

/// Restores the CLR integer indexer without inventing an additional
/// strict XNA member name in Rust's non-overloadable method surface.
pub trait EffectAnnotationCollectionExt {
    fn item_at(&self, index: i32) -> Result<Arc<EffectAnnotation>>;
}
impl EffectAnnotationCollectionExt for EffectAnnotationCollection {
    fn item_at(&self, index: i32) -> Result<Arc<EffectAnnotation>> {
        self.item_at(index)
    }
}

pub trait EffectParameterCollectionExt {
    fn item_at(&self, index: i32) -> Result<Arc<EffectParameter>>;
}
impl EffectParameterCollectionExt for EffectParameterCollection {
    fn item_at(&self, index: i32) -> Result<Arc<EffectParameter>> {
        self.item_at(index)
    }
}

pub trait EffectPassCollectionExt {
    fn item_at(&self, index: i32) -> Result<Arc<EffectPass>>;
}
impl EffectPassCollectionExt for EffectPassCollection {
    fn item_at(&self, index: i32) -> Result<Arc<EffectPass>> {
        self.item_at(index)
    }
}

pub trait EffectTechniqueCollectionExt {
    fn item_at(&self, index: i32) -> Result<Arc<EffectTechnique>>;
}
impl EffectTechniqueCollectionExt for EffectTechniqueCollection {
    fn item_at(&self, index: i32) -> Result<Arc<EffectTechnique>> {
        self.item_at(index)
    }
}

/// One capability CNA can be asked about by name.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RendererFeature(u32);

impl RendererFeature {
    pub const THREE_DIMENSIONAL_PIPELINE: Self = Self(0);
    pub const DEPTH_STENCIL_BUFFER: Self = Self(1);
    pub const MULTI_SAMPLE_ANTI_ALIASING: Self = Self(2);
    pub const MULTIPLE_RENDER_TARGETS: Self = Self(3);
    pub const ANISOTROPIC_FILTERING: Self = Self(4);
    pub const WIRE_FRAME_RASTERIZATION: Self = Self(5);
    pub const OCCLUSION_QUERIES: Self = Self(6);
    pub const SHADER_EFFECTS: Self = Self(7);
    pub const SHADER_EFFECT_SOURCE_EXECUTION: Self = Self(8);
    pub const TEXTURE_3D_STORAGE: Self = Self(9);
    pub const MULTI_STREAM_VERTEX_INPUT: Self = Self(10);
    pub const INSTANCED_DRAWING: Self = Self(11);
    pub const STENCIL_BUFFER: Self = Self(12);
    pub const ADDITIVE_BLENDING: Self = Self(13);
    pub const COMPILED_XNA_EFFECTS: Self = Self(14);
    pub const FLOAT32_RENDER_TARGETS: Self = Self(15);
    pub const FLOAT16_RENDER_TARGETS: Self = Self(16);
    pub const FLOAT16_TEXTURE_LINEAR_FILTERING: Self = Self(17);
    pub const COMPUTE_SHADERS: Self = Self(18);
    pub const COMPUTE_IMAGE_BINDING: Self = Self(19);
    pub const INDIRECT_DRAWING: Self = Self(20);
    pub const SHADOW_SAMPLING: Self = Self(21);
    pub const IMAGE_BASED_LIGHTING: Self = Self(22);
    pub const GPU_TIMERS: Self = Self(23);
    pub const SHADER_DIALECT_GLSL_DESKTOP: Self = Self(24);
    pub const SHADER_DIALECT_GLSL_ES: Self = Self(25);
    pub const SHADER_DIALECT_GLSL_VULKAN: Self = Self(26);
    pub const SHADER_DIALECT_HLSL: Self = Self(27);
    pub const SHADER_DIALECT_MSL: Self = Self(28);
    pub const SHADER_DIALECT_WGSL: Self = Self(29);

    /// Every feature this build of the binding knows how to name.
    pub const ALL: [Self; 30] = [
        Self(0), Self(1), Self(2), Self(3), Self(4), Self(5), Self(6), Self(7), Self(8), Self(9),
        Self(10), Self(11), Self(12), Self(13), Self(14), Self(15), Self(16), Self(17), Self(18),
        Self(19), Self(20), Self(21), Self(22), Self(23), Self(24), Self(25), Self(26), Self(27),
        Self(28), Self(29),
    ];

    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// How well a renderer supports one feature.
///
/// `Unknown` is a real answer, not a failure: a renderer that has not been
/// asked, or cannot answer, says so rather than claiming support either way.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum FeatureSupport {
    Unknown,
    Unsupported,
    Supported,
    /// Usable, but under restrictions the capability report describes.
    Restricted,
    /// A level a newer CNA introduced.
    Unrecognized(u32),
}

/// One numeric renderer limit CNA can be asked about.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RendererLimit(u32);

impl RendererLimit {
    pub const MAX_TEXTURE_DIMENSION: Self = Self(0);
    pub const MAX_VERTEX_STREAMS: Self = Self(1);
    pub const MAX_COMPUTE_WORK_GROUP_COUNT_X: Self = Self(2);
    pub const MAX_COMPUTE_WORK_GROUP_COUNT_Y: Self = Self(3);
    pub const MAX_COMPUTE_WORK_GROUP_COUNT_Z: Self = Self(4);
    pub const MAX_COMPUTE_WORK_GROUP_SIZE_X: Self = Self(5);
    pub const MAX_COMPUTE_WORK_GROUP_SIZE_Y: Self = Self(6);
    pub const MAX_COMPUTE_WORK_GROUP_SIZE_Z: Self = Self(7);
    pub const MAX_COMPUTE_WORK_GROUP_INVOCATIONS: Self = Self(8);
    pub const MAX_VERTEX_SHADER_STORAGE_BLOCKS: Self = Self(9);

    /// Every limit this build of the binding knows how to name.
    pub const ALL: [Self; 10] = [
        Self(0), Self(1), Self(2), Self(3), Self(4), Self(5), Self(6), Self(7), Self(8), Self(9),
    ];

    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// The shading language a renderer accepts.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ShaderDialect {
    Unknown,
    GlslDesktop,
    GlslEs,
    GlslVulkan,
    Hlsl,
    Msl,
    Wgsl,
    /// A dialect a newer CNA introduced.
    Unrecognized(u32),
}

/// What a renderer can do with one surface format.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct FormatUsage(u32);

impl FormatUsage {
    pub const TEXTURE_STORAGE: Self = Self(1 << 0);
    pub const SAMPLED: Self = Self(1 << 1);
    pub const FILTERABLE: Self = Self(1 << 2);
    pub const RENDER_TARGET: Self = Self(1 << 3);
    pub const BLENDABLE: Self = Self(1 << 4);
    pub const STORAGE_READ: Self = Self(1 << 5);
    pub const STORAGE_WRITE: Self = Self(1 << 6);
    pub const STORAGE_ATOMIC: Self = Self(1 << 7);
    pub const TRANSFER_SOURCE: Self = Self(1 << 8);
    pub const TRANSFER_DESTINATION: Self = Self(1 << 9);
    pub const MIPMAPPED: Self = Self(1 << 10);
    pub const MULTISAMPLE: Self = Self(1 << 11);
    pub const COLOR_TRANSFER: Self = Self(1 << 12);
    pub const ALL: Self = Self((1 << 13) - 1);

    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl core::ops::BitOr for FormatUsage {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitAnd for FormatUsage {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

/// What a renderer answered about one surface format.
///
/// The two masks are deliberately separate. `known` is what the renderer has
/// an answer for at all; `supported` is what it can actually do. A usage
/// outside `known` is "not asked", which is different from "refused", and
/// flattening the two would turn an unanswered question into a denial.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FormatSupport {
    pub known: FormatUsage,
    pub supported: FormatUsage,
}

impl FormatSupport {
    /// Whether the renderer positively supports every usage in `usage`.
    #[must_use]
    pub const fn supports(self, usage: FormatUsage) -> bool {
        self.supported.contains(usage)
    }

    /// Whether the renderer has an answer for every usage in `usage`.
    #[must_use]
    pub const fn knows(self, usage: FormatUsage) -> bool {
        self.known.contains(usage)
    }
}

/// CNA's renderer capability reporting for a strict XNA `GraphicsDevice`.
///
/// XNA answered capability questions through `GraphicsProfile` alone. CNA
/// supports many more backends than XNA's two profiles could describe, so it
/// publishes per-feature, per-limit and per-format answers instead.
pub trait RendererCapabilityExt {
    /// How well the renderer supports one feature.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn feature_support(&self, feature: RendererFeature) -> Result<FeatureSupport>;

    /// One numeric limit, or `None` when the renderer does not publish it.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn limit(&self, limit: RendererLimit) -> Result<Option<u64>>;

    /// What the renderer can do with one surface format.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn format_support(&self, format: SurfaceFormat) -> Result<FormatSupport>;

    /// The shading language the renderer accepts.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn shader_dialect(&self) -> Result<ShaderDialect>;

    /// CNA's own human-readable capability report for this device.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn capability_report(&self) -> Result<String>;
}

impl RendererCapabilityExt for GraphicsDevice {
    fn feature_support(&self, feature: RendererFeature) -> Result<FeatureSupport> {
        Ok(match self.renderer_feature_support(feature.value())? {
            0 => FeatureSupport::Unknown,
            1 => FeatureSupport::Unsupported,
            2 => FeatureSupport::Supported,
            3 => FeatureSupport::Restricted,
            other => FeatureSupport::Unrecognized(other),
        })
    }

    fn limit(&self, limit: RendererLimit) -> Result<Option<u64>> {
        GraphicsDevice::renderer_limit(self, limit.value())
    }

    fn format_support(&self, format: SurfaceFormat) -> Result<FormatSupport> {
        let (known, supported) = self.surface_format_support(format as u32)?;
        Ok(FormatSupport {
            known: FormatUsage(known),
            supported: FormatUsage(supported),
        })
    }

    fn shader_dialect(&self) -> Result<ShaderDialect> {
        Ok(match GraphicsDevice::shader_dialect(self)? {
            0 => ShaderDialect::Unknown,
            1 => ShaderDialect::GlslDesktop,
            2 => ShaderDialect::GlslEs,
            3 => ShaderDialect::GlslVulkan,
            4 => ShaderDialect::Hlsl,
            5 => ShaderDialect::Msl,
            6 => ShaderDialect::Wgsl,
            other => ShaderDialect::Unrecognized(other),
        })
    }

    fn capability_report(&self) -> Result<String> {
        GraphicsDevice::capability_report(self)
    }
}

/// The mask a CRT effect simulates.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CrtMaskType {
    None,
    ApertureGrille,
    ShadowMask,
    /// A mask a newer CNA introduced.
    Unrecognized(u32),
}

/// How a depth effect quantizes what it writes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DepthEffectMode {
    Color16Bit,
    Color8Bit,
    Grayscale4Bit,
    Grayscale2Bit,
    Grayscale1Bit,
    Palette256,
    Palette16,
    /// A mode a newer CNA introduced.
    Unrecognized(u32),
}

/// The dither pattern a depth effect applies.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DitherMode {
    None,
    Bayer4x4,
    Bayer8x8,
    /// A mode a newer CNA introduced.
    Unrecognized(u32),
}

/// How an ASCII post-process effect reduces colour.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AsciiQuantizeMode {
    BlackWhite,
    Color,
    /// A mode a newer CNA introduced.
    Unrecognized(u32),
}

macro_rules! identity {
    ($name:ident, $native:ty, $($variant:ident => $constant:path,)+) => {
        impl $name {
            const fn from_native(value: $native) -> Self {
                match value {
                    $($constant => Self::$variant,)+
                    other => Self::Unrecognized(other),
                }
            }

            const fn as_native(self) -> $native {
                match self {
                    $(Self::$variant => $constant,)+
                    Self::Unrecognized(value) => value,
                }
            }
        }
    };
}

identity!(
    CrtMaskType,
    cna_sys::CNA_CRTMaskType,
    None => cna_sys::CNA_CRT_MASK_TYPE_NONE,
    ApertureGrille => cna_sys::CNA_CRT_MASK_TYPE_APERTURE_GRILLE,
    ShadowMask => cna_sys::CNA_CRT_MASK_TYPE_SHADOW_MASK,
);
identity!(
    DepthEffectMode,
    cna_sys::CNA_DepthEffectMode,
    Color16Bit => cna_sys::CNA_DEPTH_EFFECT_MODE_COLOR_16_BIT,
    Color8Bit => cna_sys::CNA_DEPTH_EFFECT_MODE_COLOR_8_BIT,
    Grayscale4Bit => cna_sys::CNA_DEPTH_EFFECT_MODE_GRAYSCALE_4_BIT,
    Grayscale2Bit => cna_sys::CNA_DEPTH_EFFECT_MODE_GRAYSCALE_2_BIT,
    Grayscale1Bit => cna_sys::CNA_DEPTH_EFFECT_MODE_GRAYSCALE_1_BIT,
    Palette256 => cna_sys::CNA_DEPTH_EFFECT_MODE_PALETTE_256,
    Palette16 => cna_sys::CNA_DEPTH_EFFECT_MODE_PALETTE_16,
);
identity!(
    DitherMode,
    cna_sys::CNA_DitherMode,
    None => cna_sys::CNA_DITHER_MODE_NONE,
    Bayer4x4 => cna_sys::CNA_DITHER_MODE_BAYER_4X4,
    Bayer8x8 => cna_sys::CNA_DITHER_MODE_BAYER_8X8,
);
identity!(
    AsciiQuantizeMode,
    cna_sys::CNA_AsciiQuantizeMode,
    BlackWhite => cna_sys::CNA_ASCII_QUANTIZE_MODE_BLACK_WHITE,
    Color => cna_sys::CNA_ASCII_QUANTIZE_MODE_COLOR,
);

/// Whether this build contains CNA's extended graphics layer.
///
/// The layer is a build option. Every route below is exported in both states
/// and refuses with `NOT_SUPPORTED` when it is compiled out, so ask this
/// rather than reading a refusal as a renderer limitation.
pub fn is_available() -> Result<bool> {
    let native = crate::native::Native::process()?;
    let mut value = cna_sys::CNA_FALSE;
    // SAFETY: the output is a live local of the declared type.
    native.check(unsafe { (native.runtime.graphics_ext_is_available)(&mut value) })?;
    Ok(value != cna_sys::CNA_FALSE)
}

/// CNA's extended effects, which are XNA `Effect`s with extra knobs.
///
/// A CRT or depth effect is the same handle kind as a strict XNA `Effect`
/// upstream, so it is projected as one: the knobs live here rather than in a
/// parallel type that would not be usable where an `Effect` is expected.
pub trait ExtendedEffectExt {
    /// Creates CNA's CRT post-processing effect.
    fn create_crt_effect(&self) -> Result<Effect>;

    /// Creates CNA's depth-visualization effect.
    fn create_depth_effect(&self) -> Result<Effect>;
}

impl ExtendedEffectExt for GraphicsDevice {
    fn create_crt_effect(&self) -> Result<Effect> {
        let handle = self.create_extended_effect(true)?;
        Ok(Effect::adopt_extended(self, handle))
    }

    fn create_depth_effect(&self) -> Result<Effect> {
        let handle = self.create_extended_effect(false)?;
        Ok(Effect::adopt_extended(self, handle))
    }
}

/// The CRT effect's own settings.
///
/// The trait is implemented for every `Effect`, because CNA gives a CRT effect
/// the ordinary `Effect` handle kind; calling one of these on an effect that
/// is not a CRT effect is refused by CNA rather than silently accepted.
pub trait CrtEffectExt {
    fn scanline_intensity(&self) -> Result<f32>;
    fn set_scanline_intensity(&self, value: f32) -> Result<()>;
    fn curvature(&self) -> Result<f32>;
    fn set_curvature(&self, value: f32) -> Result<()>;
    fn vignette_intensity(&self) -> Result<f32>;
    fn set_vignette_intensity(&self, value: f32) -> Result<()>;
    fn mask_intensity(&self) -> Result<f32>;
    fn set_mask_intensity(&self, value: f32) -> Result<()>;
    fn mask_type(&self) -> Result<CrtMaskType>;
    fn set_mask_type(&self, value: CrtMaskType) -> Result<()>;
}

/// The depth effect's own settings. See [`CrtEffectExt`] for why it is a trait
/// on every `Effect`.
pub trait DepthEffectExt {
    fn depth_mode(&self) -> Result<DepthEffectMode>;
    fn set_depth_mode(&self, value: DepthEffectMode) -> Result<()>;
    fn dither_mode(&self) -> Result<DitherMode>;
    fn set_dither_mode(&self, value: DitherMode) -> Result<()>;
}

macro_rules! effect_float {
    ($getter:ident, $setter:ident, $get_slot:ident, $set_slot:ident) => {
        fn $getter(&self) -> Result<f32> {
            let (native, handle) = self.extended_effect_target()?;
            let mut value = 0.0;
            // SAFETY: the handle is live and the output is a live local.
            native.check(unsafe { (native.runtime.$get_slot)(handle, &mut value) })?;
            Ok(value)
        }

        fn $setter(&self, value: f32) -> Result<()> {
            let (native, handle) = self.extended_effect_target()?;
            // SAFETY: the handle is live and the value is a plain float.
            native.check(unsafe { (native.runtime.$set_slot)(handle, value) })
        }
    };
}

impl CrtEffectExt for Effect {
    effect_float!(
        scanline_intensity,
        set_scanline_intensity,
        crt_get_scanline_intensity,
        crt_set_scanline_intensity
    );
    effect_float!(curvature, set_curvature, crt_get_curvature, crt_set_curvature);
    effect_float!(
        vignette_intensity,
        set_vignette_intensity,
        crt_get_vignette_intensity,
        crt_set_vignette_intensity
    );
    effect_float!(
        mask_intensity,
        set_mask_intensity,
        crt_get_mask_intensity,
        crt_set_mask_intensity
    );

    fn mask_type(&self) -> Result<CrtMaskType> {
        let (native, handle) = self.extended_effect_target()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is a live local.
        native.check(unsafe { (native.runtime.crt_get_mask_type)(handle, &mut value) })?;
        Ok(CrtMaskType::from_native(value))
    }

    fn set_mask_type(&self, value: CrtMaskType) -> Result<()> {
        let (native, handle) = self.extended_effect_target()?;
        // SAFETY: the handle is live and the identity is a fixed-width value.
        native.check(unsafe { (native.runtime.crt_set_mask_type)(handle, value.as_native()) })
    }
}

impl DepthEffectExt for Effect {
    fn depth_mode(&self) -> Result<DepthEffectMode> {
        let (native, handle) = self.extended_effect_target()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is a live local.
        native.check(unsafe { (native.runtime.depth_get_mode)(handle, &mut value) })?;
        Ok(DepthEffectMode::from_native(value))
    }

    fn set_depth_mode(&self, value: DepthEffectMode) -> Result<()> {
        let (native, handle) = self.extended_effect_target()?;
        // SAFETY: the handle is live and the identity is a fixed-width value.
        native.check(unsafe { (native.runtime.depth_set_mode)(handle, value.as_native()) })
    }

    fn dither_mode(&self) -> Result<DitherMode> {
        let (native, handle) = self.extended_effect_target()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is a live local.
        native.check(unsafe { (native.runtime.depth_get_dither_mode)(handle, &mut value) })?;
        Ok(DitherMode::from_native(value))
    }

    fn set_dither_mode(&self, value: DitherMode) -> Result<()> {
        let (native, handle) = self.extended_effect_target()?;
        // SAFETY: the handle is live and the identity is a fixed-width value.
        native.check(unsafe { (native.runtime.depth_set_dither_mode)(handle, value.as_native()) })
    }
}

/// CNA's ASCII post-processing effect.
///
/// Unlike the CRT and depth effects this has its own handle kind rather than
/// being an XNA `Effect`, so it is its own owned type and is released by
/// `Drop`.
pub struct AsciiPostProcessEffect {
    device: GraphicsDevice,
    handle: cna_sys::CNA_AsciiPostProcessEffectHandle,
}

impl AsciiPostProcessEffect {
    /// Creates the effect for one graphics device.
    ///
    /// This is a CNA concept, so it takes ordinary Rust naming rather than the
    /// XNA parameter spelling the strict hierarchy preserves.
    pub fn new(graphics_device: &GraphicsDevice) -> Result<Self> {
        let handle = graphics_device.create_ascii_post_process_effect()?;
        Ok(Self {
            device: graphics_device.clone(),
            handle,
        })
    }

    fn native(&self) -> Result<Arc<Native>> {
        self.device.extended_effect_native()
    }

    /// The character cell the effect quantizes into.
    pub fn cell_size(&self) -> Result<(i32, i32)> {
        let native = self.native()?;
        let mut width = 0;
        let mut height = 0;
        // SAFETY: the handle is owned and both outputs are live locals.
        native.check(unsafe {
            (native.runtime.ascii_get_cell_size)(self.handle, &mut width, &mut height)
        })?;
        Ok((width, height))
    }

    /// Sets the character cell.
    pub fn set_cell_size(&self, width: i32, height: i32) -> Result<()> {
        let native = self.native()?;
        // SAFETY: the handle is owned and both values are plain integers.
        native.check(unsafe { (native.runtime.ascii_set_cell_size)(self.handle, width, height) })
    }

    /// How the effect reduces colour.
    pub fn quantize_mode(&self) -> Result<AsciiQuantizeMode> {
        let native = self.native()?;
        let mut value = 0;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe {
            (native.runtime.ascii_get_quantize_mode)(self.handle, &mut value)
        })?;
        Ok(AsciiQuantizeMode::from_native(value))
    }

    /// Sets how the effect reduces colour.
    pub fn set_quantize_mode(&self, value: AsciiQuantizeMode) -> Result<()> {
        let native = self.native()?;
        // SAFETY: the handle is owned and the identity is a fixed-width value.
        native.check(unsafe {
            (native.runtime.ascii_set_quantize_mode)(self.handle, value.as_native())
        })
    }

    /// The grid the last draw produced, in columns and rows.
    pub fn last_grid_dimensions(&self) -> Result<(i32, i32)> {
        let native = self.native()?;
        let mut columns = 0;
        let mut rows = 0;
        // SAFETY: the handle is owned and both outputs are live locals.
        native.check(unsafe {
            (native.runtime.ascii_get_last_grid_dimensions)(self.handle, &mut columns, &mut rows)
        })?;
        Ok((columns, rows))
    }
}

/// The `graphics_ext.h` route that actually draws the effect.
impl AsciiPostProcessEffect {
    /// Draws the effect over a source texture.
    ///
    /// `destination` of `None` covers the whole current render target, which is
    /// what a full-screen post-process wants; a rectangle restricts it, which
    /// is what a split-screen or a picture-in-picture wants.
    ///
    /// Everything else on this type configures the effect. This is the only
    /// route that puts pixels anywhere, so without it the whole family is
    /// settings with nothing to apply them to.
    pub fn Draw(
        &self,
        source: &crate::Microsoft::Xna::Framework::Graphics::Texture2D,
        destination: Option<crate::value::Rectangle>,
    ) -> crate::error::Result<()> {
        use crate::extensions::graphics_resource::HasResourceState;
        let native = destination.map(|value| cna_sys::CNA_Rectangle {
            x: value.X,
            y: value.Y,
            width: value.Width,
            height: value.Height,
        });
        self.device.state_native().draw_ascii_post_process(
            self.handle,
            source.resource_state().require_handle()?,
            native.as_ref(),
        )
    }
}

impl Drop for AsciiPostProcessEffect {
    fn drop(&mut self) {
        if let Ok(native) = self.device.extended_effect_native() {
            // SAFETY: the handle is owned by this value and released once.
            let _ = unsafe { (native.runtime.ascii_effect_destroy)(self.handle) };
        }
    }
}
