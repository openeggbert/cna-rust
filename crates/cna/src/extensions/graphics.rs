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
