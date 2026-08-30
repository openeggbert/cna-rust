//! CNA graphics facts and construction routes XNA 4.0 does not declare.

#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use std::sync::Arc;

use crate::error::Result;
use crate::graphics::{
    Effect, EffectAnnotation, EffectAnnotationCollection, EffectParameter,
    EffectParameterCollection, EffectParameterDescriptor, EffectPass, EffectPassCollection,
    EffectTechnique, EffectTechniqueCollection, EffectTechniqueDescriptor, GraphicsDevice,
    ModelBone, ModelBoneCollection, ModelEffectCollection, ModelMesh, ModelMeshCollection,
    ModelMeshPart, ModelMeshPartCollection,
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
