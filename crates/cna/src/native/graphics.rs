//! Native graphics-device and owned-resource calls.

#![allow(clippy::similar_names)]

use cna_sys as sys;

use crate::error::{CnaError, ErrorCategory, Result};

use super::Native;

#[derive(Clone, Copy)]
pub(crate) enum StockMatrixProperty {
    World,
    View,
    Projection,
}

#[derive(Clone, Copy)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum BasicVector3Property {
    FogColor,
    AmbientLightColor,
    DiffuseColor,
    EmissiveColor,
    SpecularColor,
}

#[derive(Clone, Copy)]
pub(crate) enum BasicBoolProperty {
    FogEnabled,
    LightingEnabled,
    VertexColorEnabled,
    PreferPerPixelLighting,
    TextureEnabled,
}

#[derive(Clone, Copy)]
pub(crate) enum BasicFloatProperty {
    FogStart,
    FogEnd,
    SpecularPower,
    Alpha,
}

#[derive(Clone, Copy)]
pub(crate) enum StockEffectKind {
    AlphaTest,
    DualTexture,
    EnvironmentMap,
    Skinned,
}

impl Native {
    pub(crate) fn create_stock_effect(
        &self,
        device: sys::CNA_Handle,
        kind: StockEffectKind,
        handle: &mut sys::CNA_EffectHandle,
    ) -> Result<()> {
        let function = match kind {
            StockEffectKind::AlphaTest => self.alpha_test_effect_create,
            StockEffectKind::DualTexture => self.dual_texture_effect_create,
            StockEffectKind::EnvironmentMap => self.environment_map_effect_create,
            StockEffectKind::Skinned => self.skinned_effect_create,
        };
        // SAFETY: device is live and output receives one owned stock-effect handle.
        self.check(unsafe { function(device, handle) })
    }

    pub(crate) fn stock_diffuse_color(
        &self,
        handle: sys::CNA_EffectHandle,
        kind: StockEffectKind,
    ) -> Result<sys::CNA_Vector3> {
        let function = match kind {
            StockEffectKind::AlphaTest => self.alpha_test_effect_get_diffuse_color,
            StockEffectKind::DualTexture => self.dual_texture_effect_get_diffuse_color,
            StockEffectKind::EnvironmentMap => self.environment_map_effect_get_diffuse_color,
            StockEffectKind::Skinned => self.skinned_effect_get_diffuse_color,
        };
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: effect is live and output is writable.
        self.check(unsafe { function(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_stock_diffuse_color(
        &self,
        handle: sys::CNA_EffectHandle,
        kind: StockEffectKind,
        value: sys::CNA_Vector3,
    ) -> Result<()> {
        let function = match kind {
            StockEffectKind::AlphaTest => self.alpha_test_effect_set_diffuse_color,
            StockEffectKind::DualTexture => self.dual_texture_effect_set_diffuse_color,
            StockEffectKind::EnvironmentMap => self.environment_map_effect_set_diffuse_color,
            StockEffectKind::Skinned => self.skinned_effect_set_diffuse_color,
        };
        // SAFETY: effect is live and vector is passed by value.
        self.check(unsafe { function(handle, value) })
    }

    pub(crate) fn stock_alpha(
        &self,
        handle: sys::CNA_EffectHandle,
        kind: StockEffectKind,
    ) -> Result<f32> {
        let function = match kind {
            StockEffectKind::AlphaTest => self.alpha_test_effect_get_alpha,
            StockEffectKind::DualTexture => self.dual_texture_effect_get_alpha,
            StockEffectKind::EnvironmentMap => self.environment_map_effect_get_alpha,
            StockEffectKind::Skinned => self.skinned_effect_get_alpha,
        };
        let mut value = 0.0;
        // SAFETY: effect is live and output is writable.
        self.check(unsafe { function(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_stock_alpha(
        &self,
        handle: sys::CNA_EffectHandle,
        kind: StockEffectKind,
        value: f32,
    ) -> Result<()> {
        let function = match kind {
            StockEffectKind::AlphaTest => self.alpha_test_effect_set_alpha,
            StockEffectKind::DualTexture => self.dual_texture_effect_set_alpha,
            StockEffectKind::EnvironmentMap => self.environment_map_effect_set_alpha,
            StockEffectKind::Skinned => self.skinned_effect_set_alpha,
        };
        // SAFETY: effect is live and scalar is passed by value.
        self.check(unsafe { function(handle, value) })
    }

    pub(crate) fn stock_vertex_color(
        &self,
        handle: sys::CNA_EffectHandle,
        kind: StockEffectKind,
    ) -> Result<bool> {
        let function = match kind {
            StockEffectKind::AlphaTest => self.alpha_test_effect_get_vertex_color_enabled,
            StockEffectKind::DualTexture => self.dual_texture_effect_get_vertex_color_enabled,
            _ => {
                return Err(CnaError::InvalidInput(
                    "stock effect has no vertex-color property",
                ))
            }
        };
        let mut value = sys::CNA_FALSE;
        // SAFETY: effect is live and output is writable.
        self.check(unsafe { function(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn set_stock_vertex_color(
        &self,
        handle: sys::CNA_EffectHandle,
        kind: StockEffectKind,
        value: bool,
    ) -> Result<()> {
        let function = match kind {
            StockEffectKind::AlphaTest => self.alpha_test_effect_set_vertex_color_enabled,
            StockEffectKind::DualTexture => self.dual_texture_effect_set_vertex_color_enabled,
            _ => {
                return Err(CnaError::InvalidInput(
                    "stock effect has no vertex-color property",
                ))
            }
        };
        // SAFETY: effect is live and CNA_Bool is passed by value.
        self.check(unsafe { function(handle, u8::from(value)) })
    }

    pub(crate) fn stock_set_texture(
        &self,
        handle: sys::CNA_EffectHandle,
        kind: StockEffectKind,
        index: u32,
        texture: sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: effect/texture handles are same-device or texture is invalid.
        let result = unsafe {
            match kind {
                StockEffectKind::AlphaTest => (self.alpha_test_effect_set_texture)(handle, texture),
                StockEffectKind::DualTexture => {
                    (self.dual_texture_effect_set_texture)(handle, index, texture)
                }
                StockEffectKind::EnvironmentMap => {
                    (self.environment_map_effect_set_texture)(handle, texture)
                }
                StockEffectKind::Skinned => (self.skinned_effect_set_texture)(handle, texture),
            }
        };
        self.check(result)
    }

    pub(crate) fn alpha_function(&self, handle: sys::CNA_EffectHandle) -> Result<u32> {
        let mut value = 0;
        // SAFETY: effect is live and output is writable.
        self.check(unsafe { (self.alpha_test_effect_get_alpha_function)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_alpha_function(
        &self,
        handle: sys::CNA_EffectHandle,
        value: u32,
    ) -> Result<()> {
        // SAFETY: effect is live and enum representation is audited.
        self.check(unsafe { (self.alpha_test_effect_set_alpha_function)(handle, value) })
    }

    pub(crate) fn reference_alpha(&self, handle: sys::CNA_EffectHandle) -> Result<i32> {
        let mut value = 0;
        // SAFETY: effect is live and output is writable.
        self.check(unsafe { (self.alpha_test_effect_get_reference_alpha)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_reference_alpha(
        &self,
        handle: sys::CNA_EffectHandle,
        value: i32,
    ) -> Result<()> {
        // SAFETY: effect is live and scalar is passed by value.
        self.check(unsafe { (self.alpha_test_effect_set_reference_alpha)(handle, value) })
    }

    pub(crate) fn environment_emissive(
        &self,
        handle: sys::CNA_EffectHandle,
    ) -> Result<sys::CNA_Vector3> {
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: effect is live and output is writable.
        self.check(unsafe {
            (self.environment_map_effect_get_emissive_color)(handle, &mut value)
        })?;
        Ok(value)
    }

    pub(crate) fn set_environment_emissive(
        &self,
        handle: sys::CNA_EffectHandle,
        value: sys::CNA_Vector3,
    ) -> Result<()> {
        // SAFETY: effect is live and vector is passed by value.
        self.check(unsafe { (self.environment_map_effect_set_emissive_color)(handle, value) })
    }

    pub(crate) fn environment_float(
        &self,
        handle: sys::CNA_EffectHandle,
        property: u8,
    ) -> Result<f32> {
        let function = match property {
            0 => self.environment_map_effect_get_amount,
            _ => self.environment_map_effect_get_fresnel_factor,
        };
        let mut value = 0.0;
        // SAFETY: effect is live and output is writable.
        self.check(unsafe { function(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_environment_float(
        &self,
        handle: sys::CNA_EffectHandle,
        property: u8,
        value: f32,
    ) -> Result<()> {
        let function = match property {
            0 => self.environment_map_effect_set_amount,
            _ => self.environment_map_effect_set_fresnel_factor,
        };
        // SAFETY: effect is live and scalar is passed by value.
        self.check(unsafe { function(handle, value) })
    }

    pub(crate) fn environment_specular(
        &self,
        handle: sys::CNA_EffectHandle,
    ) -> Result<sys::CNA_Vector3> {
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: effect is live and output is writable.
        self.check(unsafe { (self.environment_map_effect_get_specular)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_environment_specular(
        &self,
        handle: sys::CNA_EffectHandle,
        value: sys::CNA_Vector3,
    ) -> Result<()> {
        // SAFETY: effect is live and vector is passed by value.
        self.check(unsafe { (self.environment_map_effect_set_specular)(handle, value) })
    }

    pub(crate) fn environment_set_map(
        &self,
        handle: sys::CNA_EffectHandle,
        texture: sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: handles are live same-device resources or map is invalid.
        self.check(unsafe { (self.environment_map_effect_set_environment_map)(handle, texture) })
    }

    pub(crate) fn skinned_vector3(
        &self,
        handle: sys::CNA_EffectHandle,
        property: u8,
    ) -> Result<sys::CNA_Vector3> {
        let function = match property {
            0 => self.skinned_effect_get_emissive_color,
            _ => self.skinned_effect_get_specular_color,
        };
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: effect is live and output is writable.
        self.check(unsafe { function(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_skinned_vector3(
        &self,
        handle: sys::CNA_EffectHandle,
        property: u8,
        value: sys::CNA_Vector3,
    ) -> Result<()> {
        let function = match property {
            0 => self.skinned_effect_set_emissive_color,
            _ => self.skinned_effect_set_specular_color,
        };
        // SAFETY: effect is live and vector is passed by value.
        self.check(unsafe { function(handle, value) })
    }

    pub(crate) fn skinned_specular_power(&self, handle: sys::CNA_EffectHandle) -> Result<f32> {
        let mut value = 0.0;
        // SAFETY: effect is live and output is writable.
        self.check(unsafe { (self.skinned_effect_get_specular_power)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_skinned_specular_power(
        &self,
        handle: sys::CNA_EffectHandle,
        value: f32,
    ) -> Result<()> {
        // SAFETY: effect is live and scalar is passed by value.
        self.check(unsafe { (self.skinned_effect_set_specular_power)(handle, value) })
    }

    pub(crate) fn skinned_prefer_pixel(&self, handle: sys::CNA_EffectHandle) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: effect is live and output is writable.
        self.check(unsafe {
            (self.skinned_effect_get_prefer_per_pixel_lighting)(handle, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn set_skinned_prefer_pixel(
        &self,
        handle: sys::CNA_EffectHandle,
        value: bool,
    ) -> Result<()> {
        // SAFETY: effect is live and CNA_Bool is passed by value.
        self.check(unsafe {
            (self.skinned_effect_set_prefer_per_pixel_lighting)(handle, u8::from(value))
        })
    }

    pub(crate) fn skinned_weights(&self, handle: sys::CNA_EffectHandle) -> Result<i32> {
        let mut value = 0;
        // SAFETY: effect is live and output is writable.
        self.check(unsafe { (self.skinned_effect_get_weights_per_vertex)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_skinned_weights(
        &self,
        handle: sys::CNA_EffectHandle,
        value: i32,
    ) -> Result<()> {
        // SAFETY: effect is live and scalar is passed by value.
        self.check(unsafe { (self.skinned_effect_set_weights_per_vertex)(handle, value) })
    }

    pub(crate) fn set_skinned_bones(
        &self,
        handle: sys::CNA_EffectHandle,
        transforms: &[sys::CNA_Matrix],
    ) -> Result<()> {
        let count = u64::try_from(transforms.len())
            .map_err(|_| CnaError::InvalidInput("bone transform array is too large"))?;
        // SAFETY: CNA copies the live matrix slice synchronously.
        self.check(unsafe {
            (self.skinned_effect_set_bone_transforms)(handle, transforms.as_ptr(), count)
        })
    }

    pub(crate) fn copy_skinned_bones(
        &self,
        handle: sys::CNA_EffectHandle,
        transforms: &mut [sys::CNA_Matrix],
    ) -> Result<usize> {
        let count = u64::try_from(transforms.len())
            .map_err(|_| CnaError::InvalidInput("bone transform array is too large"))?;
        let mut written = 0;
        // SAFETY: CNA writes at most capacity matrices and reports the count.
        self.check(unsafe {
            (self.skinned_effect_copy_bone_transforms)(
                handle,
                count,
                transforms.as_mut_ptr(),
                count,
                &mut written,
            )
        })?;
        usize::try_from(written)
            .map_err(|_| CnaError::InvalidInput("native bone count exceeds usize"))
    }

    pub(crate) fn create_basic_effect(
        &self,
        device: sys::CNA_Handle,
        handle: &mut sys::CNA_EffectHandle,
    ) -> Result<()> {
        // SAFETY: the live borrowed device and owned output handle satisfy CNA's contract.
        self.check(unsafe { (self.basic_effect_create)(device, handle) })
    }

    pub(crate) fn stock_matrix(
        &self,
        handle: sys::CNA_EffectHandle,
        property: StockMatrixProperty,
    ) -> Result<sys::CNA_Matrix> {
        let function = match property {
            StockMatrixProperty::World => self.effect_matrices_get_world,
            StockMatrixProperty::View => self.effect_matrices_get_view,
            StockMatrixProperty::Projection => self.effect_matrices_get_projection,
        };
        let mut value = sys::CNA_Matrix::default();
        // SAFETY: the effect is live and the result storage is writable.
        self.check(unsafe { function(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_stock_matrix(
        &self,
        handle: sys::CNA_EffectHandle,
        property: StockMatrixProperty,
        value: sys::CNA_Matrix,
    ) -> Result<()> {
        let function = match property {
            StockMatrixProperty::World => self.effect_matrices_set_world,
            StockMatrixProperty::View => self.effect_matrices_set_view,
            StockMatrixProperty::Projection => self.effect_matrices_set_projection,
        };
        // SAFETY: the effect is live and the matrix is passed by value.
        self.check(unsafe { function(handle, value) })
    }

    pub(crate) fn basic_vector3(
        &self,
        handle: sys::CNA_EffectHandle,
        property: BasicVector3Property,
    ) -> Result<sys::CNA_Vector3> {
        let function = match property {
            BasicVector3Property::FogColor => self.effect_fog_get_color,
            BasicVector3Property::AmbientLightColor => self.effect_lights_get_ambient_color,
            BasicVector3Property::DiffuseColor => self.basic_effect_get_diffuse_color,
            BasicVector3Property::EmissiveColor => self.basic_effect_get_emissive_color,
            BasicVector3Property::SpecularColor => self.basic_effect_get_specular_color,
        };
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the effect is live and the result storage is writable.
        self.check(unsafe { function(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_basic_vector3(
        &self,
        handle: sys::CNA_EffectHandle,
        property: BasicVector3Property,
        value: sys::CNA_Vector3,
    ) -> Result<()> {
        let function = match property {
            BasicVector3Property::FogColor => self.effect_fog_set_color,
            BasicVector3Property::AmbientLightColor => self.effect_lights_set_ambient_color,
            BasicVector3Property::DiffuseColor => self.basic_effect_set_diffuse_color,
            BasicVector3Property::EmissiveColor => self.basic_effect_set_emissive_color,
            BasicVector3Property::SpecularColor => self.basic_effect_set_specular_color,
        };
        // SAFETY: the effect is live and the vector is passed by value.
        self.check(unsafe { function(handle, value) })
    }

    pub(crate) fn basic_bool(
        &self,
        handle: sys::CNA_EffectHandle,
        property: BasicBoolProperty,
    ) -> Result<bool> {
        let function = match property {
            BasicBoolProperty::FogEnabled => self.effect_fog_get_enabled,
            BasicBoolProperty::LightingEnabled => self.effect_lights_get_enabled,
            BasicBoolProperty::VertexColorEnabled => self.basic_effect_get_vertex_color_enabled,
            BasicBoolProperty::PreferPerPixelLighting => {
                self.basic_effect_get_prefer_per_pixel_lighting
            }
            BasicBoolProperty::TextureEnabled => self.basic_effect_get_texture_enabled,
        };
        let mut value = sys::CNA_FALSE;
        // SAFETY: the effect is live and the result storage is writable.
        self.check(unsafe { function(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn set_basic_bool(
        &self,
        handle: sys::CNA_EffectHandle,
        property: BasicBoolProperty,
        value: bool,
    ) -> Result<()> {
        let function = match property {
            BasicBoolProperty::FogEnabled => self.effect_fog_set_enabled,
            BasicBoolProperty::LightingEnabled => self.effect_lights_set_enabled,
            BasicBoolProperty::VertexColorEnabled => self.basic_effect_set_vertex_color_enabled,
            BasicBoolProperty::PreferPerPixelLighting => {
                self.basic_effect_set_prefer_per_pixel_lighting
            }
            BasicBoolProperty::TextureEnabled => self.basic_effect_set_texture_enabled,
        };
        // SAFETY: the effect is live and CNA_Bool is passed by value.
        self.check(unsafe { function(handle, u8::from(value)) })
    }

    pub(crate) fn basic_float(
        &self,
        handle: sys::CNA_EffectHandle,
        property: BasicFloatProperty,
    ) -> Result<f32> {
        let function = match property {
            BasicFloatProperty::FogStart => self.effect_fog_get_start,
            BasicFloatProperty::FogEnd => self.effect_fog_get_end,
            BasicFloatProperty::SpecularPower => self.basic_effect_get_specular_power,
            BasicFloatProperty::Alpha => self.basic_effect_get_alpha,
        };
        let mut value = 0.0;
        // SAFETY: the effect is live and the result storage is writable.
        self.check(unsafe { function(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_basic_float(
        &self,
        handle: sys::CNA_EffectHandle,
        property: BasicFloatProperty,
        value: f32,
    ) -> Result<()> {
        let function = match property {
            BasicFloatProperty::FogStart => self.effect_fog_set_start,
            BasicFloatProperty::FogEnd => self.effect_fog_set_end,
            BasicFloatProperty::SpecularPower => self.basic_effect_set_specular_power,
            BasicFloatProperty::Alpha => self.basic_effect_set_alpha,
        };
        // SAFETY: the effect is live and the scalar is passed by value.
        self.check(unsafe { function(handle, value) })
    }

    pub(crate) fn basic_set_texture(
        &self,
        handle: sys::CNA_EffectHandle,
        texture: sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: handles are live same-device resources or texture is invalid.
        self.check(unsafe { (self.basic_effect_set_texture)(handle, texture) })
    }

    pub(crate) fn enable_default_lighting(&self, handle: sys::CNA_EffectHandle) -> Result<()> {
        // SAFETY: handle is a live effect implementing IEffectLights.
        self.check(unsafe { (self.effect_lights_enable_default)(handle) })
    }

    pub(crate) fn effect_directional_light(
        &self,
        handle: sys::CNA_EffectHandle,
        index: u32,
        light: &mut sys::CNA_DirectionalLightHandle,
    ) -> Result<()> {
        // SAFETY: index and output are validated synchronously by CNA.
        self.check(unsafe { (self.effect_lights_get_directional_light)(handle, index, light) })
    }

    pub(crate) fn create_directional_light(
        &self,
        light: &mut sys::CNA_DirectionalLightHandle,
    ) -> Result<()> {
        // SAFETY: output receives one owned handle.
        self.check(unsafe { (self.directional_light_create)(light) })
    }

    pub(crate) fn destroy_directional_light(
        &self,
        light: sys::CNA_DirectionalLightHandle,
    ) -> Result<()> {
        // SAFETY: caller owns the light and destroys it once.
        self.check(unsafe { (self.directional_light_destroy)(light) })
    }

    pub(crate) fn directional_light_vector3(
        &self,
        light: sys::CNA_DirectionalLightHandle,
        property: u8,
    ) -> Result<sys::CNA_Vector3> {
        let function = match property {
            0 => self.directional_light_get_direction,
            1 => self.directional_light_get_diffuse_color,
            _ => self.directional_light_get_specular_color,
        };
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: light is live and output is writable.
        self.check(unsafe { function(light, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_directional_light_vector3(
        &self,
        light: sys::CNA_DirectionalLightHandle,
        property: u8,
        value: sys::CNA_Vector3,
    ) -> Result<()> {
        let function = match property {
            0 => self.directional_light_set_direction,
            1 => self.directional_light_set_diffuse_color,
            _ => self.directional_light_set_specular_color,
        };
        // SAFETY: light is live and the vector is passed by value.
        self.check(unsafe { function(light, value) })
    }

    pub(crate) fn directional_light_enabled(
        &self,
        light: sys::CNA_DirectionalLightHandle,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: light is live and output is writable.
        self.check(unsafe { (self.directional_light_get_enabled)(light, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn set_directional_light_enabled(
        &self,
        light: sys::CNA_DirectionalLightHandle,
        value: bool,
    ) -> Result<()> {
        // SAFETY: light is live and CNA_Bool is passed by value.
        self.check(unsafe { (self.directional_light_set_enabled)(light, u8::from(value)) })
    }

    pub(crate) fn create_occlusion_query(
        &self,
        device: sys::CNA_Handle,
        handle: &mut sys::CNA_OcclusionQueryHandle,
    ) -> Result<()> {
        // SAFETY: device is live and CNA writes one owned query handle synchronously.
        self.check(unsafe { (self.occlusion_query_create)(device, handle) })
    }

    pub(crate) fn begin_occlusion_query(&self, handle: sys::CNA_Handle) -> Result<()> {
        // SAFETY: caller owns and validates the live handle.
        self.check(unsafe { (self.occlusion_query_begin)(handle) })
    }

    pub(crate) fn end_occlusion_query(&self, handle: sys::CNA_Handle) -> Result<()> {
        // SAFETY: caller owns and validates the begun query handle.
        self.check(unsafe { (self.occlusion_query_end)(handle) })
    }

    pub(crate) fn occlusion_query_is_complete(&self, handle: sys::CNA_Handle) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: caller owns the live handle and output is writable.
        self.check(unsafe { (self.occlusion_query_get_is_complete)(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn occlusion_query_pixel_count(&self, handle: sys::CNA_Handle) -> Result<i32> {
        let mut value = 0;
        // SAFETY: caller owns the live handle and output is writable.
        self.check(unsafe { (self.occlusion_query_get_pixel_count)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn destroy_occlusion_query(&self, handle: sys::CNA_Handle) -> Result<()> {
        // SAFETY: ResourceState guarantees single destruction of the owned handle.
        self.check(unsafe { (self.occlusion_query_destroy)(handle) })
    }

    pub(crate) fn borrow_graphics_device(
        &self,
        game: sys::CNA_Handle,
        device: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the caller is callback-scoped and supplies a valid output.
        self.check(unsafe { (self.game_get_graphics_device)(game, device) })
    }

    pub(crate) fn clear_graphics_device(
        &self,
        device: sys::CNA_Handle,
        rgba: [f32; 4],
    ) -> Result<()> {
        // SAFETY: GraphicsDevice guarantees its callback-scoped handle.
        self.check(unsafe {
            (self.graphics_device_clear_rgba)(device, rgba[0], rgba[1], rgba[2], rgba[3])
        })
    }

    pub(crate) fn clear_graphics_device_options(
        &self,
        device: sys::CNA_Handle,
        options: sys::CNA_ClearOptions,
        color: sys::CNA_Color,
        depth: f32,
        stencil: i32,
    ) -> Result<()> {
        // SAFETY: GraphicsDevice guarantees its callback-scoped handle.
        self.check(unsafe {
            (self.graphics_device_clear_options)(device, options, color, depth, stencil)
        })
    }

    /// Creates a graphics device this caller owns, outside any game.
    pub(crate) fn create_graphics_device(
        &self,
        adapter_index: u32,
        profile: sys::CNA_GraphicsProfile,
        parameters: &sys::CNA_PresentationParameters,
    ) -> Result<sys::CNA_Handle> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the parameter structure is a live initialized local and the
        // output is a live local; CNA copies the parameters during the call.
        self.check(unsafe {
            (self.graphics_device_create)(adapter_index, profile, parameters, &mut handle)
        })?;
        Ok(handle)
    }

    /// Destroys a device created by `create_graphics_device`.
    ///
    /// CNA refuses a game's borrowed device here, so only an independently
    /// created handle ever reaches this route.
    pub(crate) fn destroy_graphics_device(&self, device: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the handle came from cna_graphics_device_create and has not
        // been destroyed, which the caller's alive flag establishes.
        self.check(unsafe { (self.graphics_device_destroy)(device) })
    }

    pub(crate) fn graphics_device_status(
        &self,
        device: sys::CNA_Handle,
        status: &mut sys::CNA_GraphicsDeviceStatus,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_status)(device, status) })
    }

    pub(crate) fn graphics_profile(
        &self,
        device: sys::CNA_Handle,
        profile: &mut sys::CNA_GraphicsProfile,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_graphics_profile)(device, profile) })
    }

    pub(crate) fn presentation_parameters(
        &self,
        device: sys::CNA_Handle,
        parameters: &mut sys::CNA_PresentationParameters,
    ) -> Result<()> {
        // SAFETY: the caller supplies a complete versioned writable output.
        self.check(unsafe {
            (self.graphics_device_get_presentation_parameters)(device, parameters)
        })
    }

    pub(crate) fn display_mode(
        &self,
        device: sys::CNA_Handle,
        mode: &mut sys::CNA_DisplayMode,
    ) -> Result<()> {
        // SAFETY: the caller supplies a complete versioned writable output.
        self.check(unsafe { (self.graphics_device_get_display_mode)(device, mode) })
    }

    pub(crate) fn blend_state(
        &self,
        device: sys::CNA_Handle,
        state: &mut sys::CNA_BlendState,
    ) -> Result<()> {
        // SAFETY: the caller supplies a complete versioned writable output.
        self.check(unsafe { (self.graphics_device_get_blend_state)(device, state) })
    }

    pub(crate) fn depth_stencil_state(
        &self,
        device: sys::CNA_Handle,
        state: &mut sys::CNA_DepthStencilState,
    ) -> Result<()> {
        // SAFETY: the caller supplies a complete versioned writable output.
        self.check(unsafe { (self.graphics_device_get_depth_stencil_state)(device, state) })
    }

    pub(crate) fn rasterizer_state(
        &self,
        device: sys::CNA_Handle,
        state: &mut sys::CNA_RasterizerState,
    ) -> Result<()> {
        // SAFETY: the caller supplies a complete versioned writable output.
        self.check(unsafe { (self.graphics_device_get_rasterizer_state)(device, state) })
    }

    pub(crate) fn sampler_state(
        &self,
        device: sys::CNA_Handle,
        stage: sys::CNA_ShaderStage,
        slot: u32,
        state: &mut sys::CNA_SamplerState,
    ) -> Result<()> {
        // SAFETY: stage/slot are validated by the collection and output is live.
        self.check(unsafe { (self.graphics_device_get_sampler_state)(device, stage, slot, state) })
    }

    pub(crate) fn set_sampler_state(
        &self,
        device: sys::CNA_Handle,
        stage: sys::CNA_ShaderStage,
        slot: u32,
        state: &sys::CNA_SamplerState,
    ) -> Result<()> {
        // SAFETY: the complete descriptor is copied synchronously.
        self.check(unsafe { (self.graphics_device_set_sampler_state)(device, stage, slot, state) })
    }

    pub(crate) fn texture_slot(
        &self,
        device: sys::CNA_Handle,
        stage: sys::CNA_ShaderStage,
        slot: u32,
        info: &mut sys::CNA_TextureSlotInfo,
    ) -> Result<()> {
        // SAFETY: stage/slot are validated by the collection and output is live.
        self.check(unsafe { (self.graphics_device_get_texture)(device, stage, slot, info) })
    }

    pub(crate) fn set_texture_slot(
        &self,
        device: sys::CNA_Handle,
        stage: sys::CNA_ShaderStage,
        slot: u32,
        texture: sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the collection and concrete texture wrapper validate both handles.
        self.check(unsafe { (self.graphics_device_set_texture)(device, stage, slot, texture) })
    }

    pub(crate) fn graphics_viewport(
        &self,
        device: sys::CNA_Handle,
        viewport: &mut sys::CNA_Viewport,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_viewport)(device, viewport) })
    }

    pub(crate) fn set_graphics_viewport(
        &self,
        device: sys::CNA_Handle,
        viewport: sys::CNA_Viewport,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and the POD value is copied.
        self.check(unsafe { (self.graphics_device_set_viewport)(device, viewport) })
    }

    pub(crate) fn graphics_scissor_rectangle(
        &self,
        device: sys::CNA_Handle,
        rectangle: &mut sys::CNA_Rectangle,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_scissor_rectangle)(device, rectangle) })
    }

    pub(crate) fn set_graphics_scissor_rectangle(
        &self,
        device: sys::CNA_Handle,
        rectangle: sys::CNA_Rectangle,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and the POD value is copied.
        self.check(unsafe { (self.graphics_device_set_scissor_rectangle)(device, rectangle) })
    }

    pub(crate) fn graphics_blend_factor(
        &self,
        device: sys::CNA_Handle,
        color: &mut sys::CNA_Color,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_blend_factor)(device, color) })
    }

    pub(crate) fn set_graphics_blend_factor(
        &self,
        device: sys::CNA_Handle,
        color: sys::CNA_Color,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and the POD value is copied.
        self.check(unsafe { (self.graphics_device_set_blend_factor)(device, color) })
    }

    pub(crate) fn graphics_multi_sample_mask(
        &self,
        device: sys::CNA_Handle,
        value: &mut i32,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is valid.
        self.check(unsafe { (self.graphics_device_get_multi_sample_mask)(device, value) })
    }

    pub(crate) fn set_graphics_multi_sample_mask(
        &self,
        device: sys::CNA_Handle,
        value: i32,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and the scalar is copied.
        self.check(unsafe { (self.graphics_device_set_multi_sample_mask)(device, value) })
    }

    pub(crate) fn graphics_reference_stencil(
        &self,
        device: sys::CNA_Handle,
        value: &mut i32,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is valid.
        self.check(unsafe { (self.graphics_device_get_reference_stencil)(device, value) })
    }

    pub(crate) fn set_graphics_reference_stencil(
        &self,
        device: sys::CNA_Handle,
        value: i32,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and the scalar is copied.
        self.check(unsafe { (self.graphics_device_set_reference_stencil)(device, value) })
    }

    pub(crate) fn set_graphics_blend_state(
        &self,
        device: sys::CNA_Handle,
        state: &sys::CNA_BlendState,
    ) -> Result<()> {
        // SAFETY: CNA copies the complete versioned descriptor synchronously.
        self.check(unsafe { (self.graphics_device_set_blend_state)(device, state) })
    }

    pub(crate) fn set_graphics_depth_stencil_state(
        &self,
        device: sys::CNA_Handle,
        state: &sys::CNA_DepthStencilState,
    ) -> Result<()> {
        // SAFETY: CNA copies the complete versioned descriptor synchronously.
        self.check(unsafe { (self.graphics_device_set_depth_stencil_state)(device, state) })
    }

    pub(crate) fn set_graphics_rasterizer_state(
        &self,
        device: sys::CNA_Handle,
        state: &sys::CNA_RasterizerState,
    ) -> Result<()> {
        // SAFETY: CNA copies the complete versioned descriptor synchronously.
        self.check(unsafe { (self.graphics_device_set_rasterizer_state)(device, state) })
    }

    pub(crate) fn present_graphics_device(&self, device: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the device is callback-scoped and CNA performs presentation synchronously.
        self.check(unsafe { (self.graphics_device_present)(device) })
    }

    pub(crate) fn reset_graphics_device(&self, device: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the device is callback-scoped and CNA completes reset synchronously.
        self.check(unsafe { (self.graphics_device_reset)(device) })
    }

    pub(crate) fn reset_graphics_device_with_parameters(
        &self,
        device: sys::CNA_Handle,
        parameters: &sys::CNA_PresentationParameters,
        adapter_index: Option<u32>,
    ) -> Result<()> {
        let adapter = adapter_index
            .as_ref()
            .map_or(core::ptr::null(), |value| value as *const u32);
        // SAFETY: CNA copies the complete descriptor and optional scalar synchronously.
        self.check(unsafe {
            (self.graphics_device_reset_with_parameters)(device, parameters, adapter)
        })
    }

    pub(crate) fn get_backbuffer_data_window(
        &self,
        device: sys::CNA_Handle,
        readback: &sys::CNA_BackBufferReadback,
        destination: &mut [sys::CNA_Color],
    ) -> Result<()> {
        let capacity = u64::try_from(destination.len())
            .map_err(|_| CnaError::InvalidInput("back-buffer destination is too large"))?;
        let pointer = if destination.is_empty() {
            core::ptr::null_mut()
        } else {
            destination.as_mut_ptr()
        };
        // SAFETY: descriptor and writable POD destination remain live for this synchronous call.
        self.check(unsafe {
            (self.graphics_device_get_backbuffer_data_window)(device, readback, pointer, capacity)
        })
    }

    pub(crate) fn set_graphics_vertex_buffer(
        &self,
        device: sys::CNA_Handle,
        buffer: sys::CNA_Handle,
        offset: i32,
    ) -> Result<()> {
        // SAFETY: both handles and the nonnegative offset are validated by the wrappers.
        self.check(unsafe {
            if offset == 0 {
                (self.graphics_device_set_vertex_buffer)(device, buffer)
            } else {
                (self.graphics_device_set_vertex_buffer_offset)(device, buffer, offset)
            }
        })
    }

    pub(crate) fn set_graphics_vertex_buffers(
        &self,
        device: sys::CNA_Handle,
        bindings: &[sys::CNA_VertexBufferBinding],
    ) -> Result<()> {
        let count = u64::try_from(bindings.len())
            .map_err(|_| CnaError::InvalidInput("vertex binding array is too large"))?;
        let pointer = if bindings.is_empty() {
            core::ptr::null()
        } else {
            bindings.as_ptr()
        };
        // SAFETY: the complete POD slice is copied synchronously.
        self.check(unsafe { (self.graphics_device_set_vertex_buffers)(device, pointer, count) })
    }

    pub(crate) fn graphics_vertex_buffer_count(
        &self,
        device: sys::CNA_Handle,
        count: &mut u64,
    ) -> Result<()> {
        // SAFETY: the callback-scoped handle and scalar output remain live.
        self.check(unsafe { (self.graphics_device_get_vertex_buffer_count)(device, count) })
    }

    pub(crate) fn copy_graphics_vertex_buffers(
        &self,
        device: sys::CNA_Handle,
        destination: &mut [sys::CNA_VertexBufferBinding],
        count: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(destination.len())
            .map_err(|_| CnaError::InvalidInput("vertex binding array is too large"))?;
        let pointer = if destination.is_empty() {
            core::ptr::null_mut()
        } else {
            destination.as_mut_ptr()
        };
        // SAFETY: destination describes capacity writable POD elements; CNA writes atomically.
        self.check(unsafe {
            (self.graphics_device_copy_vertex_buffers)(device, pointer, capacity, count)
        })
    }

    pub(crate) fn graphics_vertex_buffer(
        &self,
        device: sys::CNA_Handle,
        buffer: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the callback-scoped handle and scalar output remain live.
        self.check(unsafe { (self.graphics_device_get_vertex_buffer)(device, buffer) })
    }

    pub(crate) fn set_graphics_index_buffer(
        &self,
        device: sys::CNA_Handle,
        buffer: sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: wrappers validate the borrowed device and owned buffer identities.
        self.check(unsafe { (self.graphics_device_set_index_buffer)(device, buffer) })
    }

    pub(crate) fn graphics_index_buffer(
        &self,
        device: sys::CNA_Handle,
        buffer: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the callback-scoped handle and scalar output remain live.
        self.check(unsafe { (self.graphics_device_get_index_buffer)(device, buffer) })
    }

    pub(crate) fn draw_primitives(
        &self,
        device: sys::CNA_Handle,
        primitive_type: sys::CNA_PrimitiveType,
        start_vertex: i32,
        primitive_count: i32,
    ) -> Result<()> {
        // SAFETY: Rust validates topology/count/range and the device is callback-scoped.
        self.check(unsafe {
            (self.graphics_device_draw_primitives)(
                device,
                primitive_type,
                start_vertex,
                primitive_count,
            )
        })
    }

    pub(crate) fn draw_indexed_primitives(
        &self,
        device: sys::CNA_Handle,
        primitive_type: sys::CNA_PrimitiveType,
        base_vertex: i32,
        min_vertex_index: i32,
        num_vertices: i32,
        start_index: i32,
        primitive_count: i32,
    ) -> Result<()> {
        // SAFETY: Rust validates bound resources and every scalar range first.
        self.check(unsafe {
            (self.graphics_device_draw_indexed_primitives)(
                device,
                primitive_type,
                base_vertex,
                min_vertex_index,
                num_vertices,
                start_index,
                primitive_count,
            )
        })
    }

    pub(crate) fn draw_instanced_primitives(
        &self,
        device: sys::CNA_Handle,
        primitive_type: sys::CNA_PrimitiveType,
        base_vertex: i32,
        min_vertex_index: i32,
        num_vertices: i32,
        start_index: i32,
        primitive_count: i32,
        instance_count: i32,
    ) -> Result<()> {
        // SAFETY: Rust validates bound resources and every scalar range first.
        self.check(unsafe {
            (self.graphics_device_draw_instanced_primitives)(
                device,
                primitive_type,
                base_vertex,
                min_vertex_index,
                num_vertices,
                start_index,
                primitive_count,
                instance_count,
            )
        })
    }

    pub(crate) fn draw_user_primitives(
        &self,
        device: sys::CNA_Handle,
        primitives: &sys::CNA_UserPrimitives,
    ) -> Result<()> {
        // SAFETY: Rust owns the descriptor, declaration and source bytes for the whole call.
        self.check(unsafe { (self.graphics_device_draw_user_primitives)(device, primitives) })
    }

    pub(crate) fn draw_user_indexed_primitives(
        &self,
        device: sys::CNA_Handle,
        primitives: &sys::CNA_UserPrimitives,
        indices: &sys::CNA_UserIndices,
    ) -> Result<()> {
        // SAFETY: Rust owns both descriptors and source arrays for the synchronous call.
        self.check(unsafe {
            (self.graphics_device_draw_user_indexed_primitives)(device, primitives, indices)
        })
    }

    pub(crate) fn set_render_targets(
        &self,
        device: sys::CNA_Handle,
        bindings: &[sys::CNA_RenderTargetBinding],
    ) -> Result<()> {
        let count = u64::try_from(bindings.len())
            .map_err(|_| CnaError::InvalidInput("render-target binding array is too large"))?;
        let pointer = if bindings.is_empty() {
            core::ptr::null()
        } else {
            bindings.as_ptr()
        };
        // SAFETY: CNA synchronously validates and copies the complete POD binding array.
        self.check(unsafe { (self.graphics_device_set_render_targets)(device, pointer, count) })
    }

    pub(crate) fn render_target_count(
        &self,
        device: sys::CNA_Handle,
        count: &mut u64,
    ) -> Result<()> {
        // SAFETY: the callback-scoped handle and scalar output remain live.
        self.check(unsafe { (self.graphics_device_get_render_target_count)(device, count) })
    }

    pub(crate) fn copy_render_targets(
        &self,
        device: sys::CNA_Handle,
        destination: &mut [sys::CNA_RenderTargetBinding],
        count: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(destination.len())
            .map_err(|_| CnaError::InvalidInput("render-target binding array is too large"))?;
        let pointer = if destination.is_empty() {
            core::ptr::null_mut()
        } else {
            destination.as_mut_ptr()
        };
        // SAFETY: destination describes capacity writable POD elements; CNA writes atomically.
        self.check(unsafe {
            (self.graphics_device_copy_render_targets)(device, pointer, capacity, count)
        })
    }

    pub(crate) fn renderer_info(
        &self,
        device: sys::CNA_Handle,
        info: &mut sys::CNA_RendererInfo,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_renderer_info)(device, info) })
    }

    pub(crate) fn renderer_feature_support(
        &self,
        device: sys::CNA_Handle,
        feature: sys::CNA_RendererFeature,
    ) -> Result<sys::CNA_RendererFeatureSupport> {
        let mut support = 0;
        // SAFETY: the device is callback-scoped and the output is a live local.
        self.check(unsafe { (self.graphics_device_feature_support)(device, feature, &mut support) })?;
        Ok(support)
    }

    pub(crate) fn renderer_limit(
        &self,
        device: sys::CNA_Handle,
        limit: sys::CNA_RendererLimit,
    ) -> Result<Option<u64>> {
        let mut known = sys::CNA_FALSE;
        let mut value = 0_u64;
        // SAFETY: the device is callback-scoped and both outputs are live locals.
        self.check(unsafe { (self.graphics_device_limit)(device, limit, &mut known, &mut value) })?;
        Ok((known != sys::CNA_FALSE).then_some(value))
    }

    pub(crate) fn surface_format_support(
        &self,
        device: sys::CNA_Handle,
        format: sys::CNA_SurfaceFormat,
    ) -> Result<(sys::CNA_RendererFormatUsageFlags, sys::CNA_RendererFormatUsageFlags)> {
        let mut known = 0;
        let mut supported = 0;
        // SAFETY: the device is callback-scoped and both outputs are live locals.
        self.check(unsafe {
            (self.graphics_device_format_support)(device, format, &mut known, &mut supported)
        })?;
        Ok((known, supported))
    }

    pub(crate) fn shader_dialect(
        &self,
        device: sys::CNA_Handle,
    ) -> Result<sys::CNA_ShaderDialect> {
        let mut dialect = 0;
        // SAFETY: the device is callback-scoped and the output is a live local.
        self.check(unsafe { (self.graphics_device_shader_dialect)(device, &mut dialect) })?;
        Ok(dialect)
    }

    pub(crate) fn capability_report(&self, device: sys::CNA_Handle) -> Result<String> {
        let mut bytes = 0_u64;
        // SAFETY: the device is callback-scoped and the output is a live local.
        self.check(unsafe { (self.graphics_device_capability_report_size)(device, &mut bytes) })?;
        let capacity = usize::try_from(bytes)
            .map_err(|_| CnaError::InvalidInput("capability report is too large"))?;
        if capacity == 0 {
            return Ok(String::new());
        }
        let mut buffer = vec![0_u8; capacity];
        let mut copied = 0_u64;
        // SAFETY: `buffer` holds exactly `bytes` writable bytes for the call.
        self.check(unsafe {
            (self.graphics_device_copy_capability_report)(
                device,
                buffer.as_mut_ptr().cast::<core::ffi::c_char>(),
                bytes,
                &mut copied,
            )
        })?;
        let copied = usize::try_from(copied)
            .map_err(|_| CnaError::InvalidInput("capability report is too large"))?;
        buffer.truncate(copied.min(capacity));
        Ok(String::from_utf8_lossy(&buffer).into_owned())
    }

    pub(crate) fn renderer_name_size(&self, device: sys::CNA_Handle, size: &mut u64) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is valid.
        self.check(unsafe { (self.graphics_device_get_renderer_name_size)(device, size) })
    }

    pub(crate) fn copy_renderer_name(
        &self,
        device: sys::CNA_Handle,
        destination: &mut [u8],
        copied: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(destination.len())
            .map_err(|_| CnaError::InvalidInput("renderer-name buffer is too large"))?;
        // SAFETY: the slice describes exactly `capacity` writable bytes and all
        // references remain live through the synchronous call.
        self.check(unsafe {
            (self.graphics_device_copy_renderer_name)(
                device,
                destination.as_mut_ptr().cast(),
                capacity,
                copied,
            )
        })
    }

    pub(crate) fn create_texture_from_encoded(
        &self,
        device: sys::CNA_Handle,
        bytes: &[u8],
        decode: Option<&sys::CNA_Texture2DDecodeInfo>,
        texture: &mut sys::CNA_Handle,
    ) -> Result<()> {
        let count = u64::try_from(bytes.len())
            .map_err(|_| CnaError::InvalidInput("encoded texture is too large"))?;
        let decode = decode.map_or(core::ptr::null(), |value| value as *const _);
        // SAFETY: the encoded slice and optional decode structure remain live;
        // the output reference is valid for the synchronous call.
        self.check(unsafe {
            (self.texture2d_create_from_encoded_memory)(
                device,
                bytes.as_ptr(),
                count,
                decode,
                texture,
            )
        })
    }

    pub(crate) fn create_texture(
        &self,
        device: sys::CNA_Handle,
        info: &sys::CNA_Texture2DCreateInfo,
        texture: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the callback-scoped device and versioned input/output remain live.
        self.check(unsafe { (self.texture2d_create)(device, info, texture) })
    }

    pub(crate) fn create_texture_cube(
        &self,
        device: sys::CNA_Handle,
        info: &sys::CNA_TextureCubeCreateInfo,
        handle: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: CNA copies the descriptor and writes one owned handle synchronously.
        self.check(unsafe { (self.texturecube_create)(device, info, handle) })
    }

    pub(crate) fn create_texture3d(
        &self,
        device: sys::CNA_Handle,
        info: &sys::CNA_Texture3DCreateInfo,
        handle: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: CNA copies the descriptor and writes one owned handle synchronously.
        self.check(unsafe { (self.texture3d_create)(device, info, handle) })
    }

    pub(crate) fn destroy_texture3d(&self, handle: sys::CNA_Handle) -> Result<()> {
        // SAFETY: ResourceState guarantees single destruction of the owned handle.
        self.check(unsafe { (self.texture3d_destroy)(handle) })
    }

    pub(crate) fn texture3d_info(
        &self,
        handle: sys::CNA_Handle,
        info: &mut sys::CNA_Texture3DInfo,
    ) -> Result<()> {
        // SAFETY: caller initializes the complete versioned writable descriptor.
        self.check(unsafe { (self.texture3d_get_info)(handle, info) })
    }

    pub(crate) fn set_texture3d_data(
        &self,
        handle: sys::CNA_Handle,
        transfer: &sys::CNA_Texture3DTransfer,
        data: &[sys::CNA_Color],
    ) -> Result<()> {
        let capacity = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("volume texture source is too large"))?;
        let pointer = if data.is_empty() {
            core::ptr::null()
        } else {
            data.as_ptr()
        };
        // SAFETY: CNA reads the complete Color slice synchronously and retains no pointer.
        self.check(unsafe { (self.texture3d_set_data)(handle, transfer, pointer, capacity) })
    }

    pub(crate) fn get_texture3d_data(
        &self,
        handle: sys::CNA_Handle,
        transfer: &sys::CNA_Texture3DTransfer,
        data: &mut [sys::CNA_Color],
        required: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("volume texture destination is too large"))?;
        let pointer = if data.is_empty() {
            core::ptr::null_mut()
        } else {
            data.as_mut_ptr()
        };
        // SAFETY: CNA writes the caller-owned POD slice atomically and reports the exact count.
        self.check(unsafe {
            (self.texture3d_get_data)(handle, transfer, pointer, capacity, required)
        })
    }

    pub(crate) fn destroy_texture_cube(&self, handle: sys::CNA_Handle) -> Result<()> {
        // SAFETY: ResourceState guarantees single destruction of an owned cube texture handle.
        self.check(unsafe { (self.texturecube_destroy)(handle) })
    }

    pub(crate) fn texture_cube_info(
        &self,
        handle: sys::CNA_Handle,
        info: &mut sys::CNA_TextureCubeInfo,
    ) -> Result<()> {
        // SAFETY: caller initializes the complete versioned writable descriptor.
        self.check(unsafe { (self.texturecube_get_info)(handle, info) })
    }

    pub(crate) fn set_texture_cube_data(
        &self,
        handle: sys::CNA_Handle,
        transfer: &sys::CNA_TextureCubeTransfer,
        data: &[sys::CNA_Color],
    ) -> Result<()> {
        let capacity = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("cube texture source is too large"))?;
        let pointer = if data.is_empty() {
            core::ptr::null()
        } else {
            data.as_ptr()
        };
        // SAFETY: CNA reads the complete Color slice synchronously and retains no pointer.
        self.check(unsafe { (self.texturecube_set_data)(handle, transfer, pointer, capacity) })
    }

    pub(crate) fn get_texture_cube_data(
        &self,
        handle: sys::CNA_Handle,
        transfer: &sys::CNA_TextureCubeTransfer,
        data: &mut [sys::CNA_Color],
        required: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("cube texture destination is too large"))?;
        let pointer = if data.is_empty() {
            core::ptr::null_mut()
        } else {
            data.as_mut_ptr()
        };
        // SAFETY: CNA writes the caller-owned POD slice atomically and reports the exact count.
        self.check(unsafe {
            (self.texturecube_get_data)(handle, transfer, pointer, capacity, required)
        })
    }

    pub(crate) fn create_render_target2d(
        &self,
        device: sys::CNA_Handle,
        info: &sys::CNA_RenderTarget2DCreateInfo,
        handle: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: CNA copies the descriptor and writes one owned handle synchronously.
        self.check(unsafe { (self.render_target2d_create)(device, info, handle) })
    }

    pub(crate) fn create_render_target_cube(
        &self,
        device: sys::CNA_Handle,
        info: &sys::CNA_RenderTargetCubeCreateInfo,
        handle: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: CNA copies the descriptor and writes one owned handle synchronously.
        self.check(unsafe { (self.render_target_cube_create)(device, info, handle) })
    }

    pub(crate) fn render_target_info(
        &self,
        handle: sys::CNA_Handle,
        info: &mut sys::CNA_RenderTargetInfo,
    ) -> Result<()> {
        // SAFETY: caller initializes the complete versioned writable descriptor.
        self.check(unsafe { (self.render_target_get_info)(handle, info) })
    }

    pub(crate) fn destroy_render_target(&self, handle: sys::CNA_Handle) -> Result<()> {
        // SAFETY: ResourceState guarantees single destruction of an unbound owned target handle.
        self.check(unsafe { (self.render_target_destroy)(handle) })
    }

    pub(crate) fn texture_info(
        &self,
        texture: sys::CNA_Handle,
        info: &mut sys::CNA_Texture2DInfo,
    ) -> Result<()> {
        #[cfg(feature = "native-fault-injection")]
        super::fault::check("texture-info")?;
        // SAFETY: the owned texture handle and initialized output are live.
        self.check(unsafe { (self.texture2d_get_info)(texture, info) })
    }

    pub(crate) fn set_texture_data<T: Copy>(
        &self,
        texture: sys::CNA_Handle,
        data_type: sys::CNA_TextureDataType,
        transfer: &sys::CNA_Texture2DTransfer,
        data: &[T],
    ) -> Result<()> {
        let capacity = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("texture data array is too large"))?;
        // SAFETY: the reviewed data-type identity matches T's representation;
        // both the versioned transfer and source slice remain live for the call.
        self.check(unsafe {
            (self.texture2d_set_data)(texture, data_type, transfer, data.as_ptr().cast(), capacity)
        })
    }

    pub(crate) fn get_texture_data<T: Copy>(
        &self,
        texture: sys::CNA_Handle,
        data_type: sys::CNA_TextureDataType,
        transfer: &sys::CNA_Texture2DTransfer,
        data: &mut [T],
        required: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("texture data array is too large"))?;
        // SAFETY: the reviewed data-type identity matches T's representation;
        // the destination is valid for capacity elements and native writes atomically.
        self.check(unsafe {
            (self.texture2d_get_data)(
                texture,
                data_type,
                transfer,
                data.as_mut_ptr().cast(),
                capacity,
                required,
            )
        })
    }

    pub(crate) fn encoded_texture_size(
        &self,
        texture: sys::CNA_Handle,
        format: sys::CNA_TextureImageFormat,
        width: u32,
        height: u32,
        size: &mut u64,
    ) -> Result<()> {
        // SAFETY: the owned texture and output remain live.
        self.check(unsafe {
            (self.texture2d_get_encoded_byte_count)(texture, format, width, height, size)
        })
    }

    pub(crate) fn copy_encoded_texture(
        &self,
        texture: sys::CNA_Handle,
        format: sys::CNA_TextureImageFormat,
        width: u32,
        height: u32,
        destination: &mut [u8],
        copied: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(destination.len())
            .map_err(|_| CnaError::InvalidInput("encoded texture is too large"))?;
        // SAFETY: the destination describes capacity writable bytes and all
        // references remain live through the synchronous call.
        self.check(unsafe {
            (self.texture2d_copy_encoded)(
                texture,
                format,
                width,
                height,
                destination.as_mut_ptr(),
                capacity,
                copied,
            )
        })
    }

    pub(crate) fn destroy_texture(&self, texture: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the caller transfers exactly-once ownership of a live handle.
        self.check(unsafe { (self.texture2d_destroy)(texture) })
    }

    pub(crate) fn create_vertex_declaration(
        &self,
        stride: i32,
        elements: &[sys::CNA_VertexElement],
        declaration: &mut sys::CNA_Handle,
    ) -> Result<()> {
        let count = u64::try_from(elements.len())
            .map_err(|_| CnaError::InvalidInput("vertex declaration is too large"))?;
        let pointer = if elements.is_empty() {
            core::ptr::null()
        } else {
            elements.as_ptr()
        };
        // SAFETY: the validated POD slice is copied synchronously and the output is live.
        self.check(unsafe {
            (self.vertex_declaration_create_with_stride)(stride, pointer, count, declaration)
        })
    }

    pub(crate) fn destroy_vertex_declaration(&self, declaration: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the caller transfers exactly-once ownership of a live handle.
        self.check(unsafe { (self.vertex_declaration_destroy)(declaration) })
    }

    pub(crate) fn initialize_vertex_buffer_binding(
        &self,
        buffer: sys::CNA_Handle,
        vertex_offset: i32,
        instance_frequency: i32,
        binding: &mut sys::CNA_VertexBufferBinding,
    ) -> Result<()> {
        // SAFETY: scalar inputs were validated by CNA and the output is live.
        self.check(unsafe {
            (self.vertex_buffer_binding_init)(buffer, vertex_offset, instance_frequency, binding)
        })
    }

    pub(crate) fn create_vertex_buffer(
        &self,
        device: sys::CNA_Handle,
        info: &sys::CNA_VertexBufferCreateInfo,
        buffer: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the callback-scoped device and versioned input/output remain live.
        self.check(unsafe { (self.vertex_buffer_create)(device, info, buffer) })
    }

    pub(crate) fn vertex_buffer_info(
        &self,
        buffer: sys::CNA_Handle,
        info: &mut sys::CNA_VertexBufferInfo,
    ) -> Result<()> {
        // SAFETY: the owned handle and initialized versioned output remain live.
        self.check(unsafe { (self.vertex_buffer_get_info)(buffer, info) })
    }

    /// The caller must prove that `T` has the exact CNA layout selected by `transfer.vertex_type`.
    pub(crate) unsafe fn set_typed_vertex_data<T: Copy>(
        &self,
        buffer: sys::CNA_Handle,
        transfer: &sys::CNA_VertexBufferTransfer,
        data: &[T],
    ) -> Result<()> {
        let capacity = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("vertex data array is too large"))?;
        let pointer = if data.is_empty() {
            core::ptr::null()
        } else {
            data.as_ptr().cast()
        };
        // SAFETY: the caller establishes the documented T/vertex_type layout identity.
        self.check(unsafe { (self.vertex_buffer_set_data)(buffer, transfer, pointer, capacity) })
    }

    pub(crate) fn set_raw_vertex_data(
        &self,
        buffer: sys::CNA_Handle,
        offset_in_bytes: Option<u64>,
        data: &[u8],
        vertex_count: u64,
        vertex_stride: u32,
    ) -> Result<()> {
        let byte_count = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("vertex data array is too large"))?;
        let pointer = if data.is_empty() {
            core::ptr::null()
        } else {
            data.as_ptr().cast()
        };
        // SAFETY: the byte slice describes exactly byte_count readable bytes and is copied.
        let result = unsafe {
            match offset_in_bytes {
                Some(offset) => (self.vertex_buffer_set_data_raw_at)(
                    buffer,
                    offset,
                    pointer,
                    byte_count,
                    vertex_count,
                    vertex_stride,
                ),
                None => (self.vertex_buffer_set_data_raw)(
                    buffer,
                    pointer,
                    byte_count,
                    vertex_count,
                    vertex_stride,
                ),
            }
        };
        self.check(result)
    }

    /// Uploads raw vertex bytes with an explicit streaming hint.
    ///
    /// Split from `set_raw_vertex_data` rather than folded into it: the
    /// option-carrying canonical routes are a different pair of symbols, and a
    /// caller that passes `SetDataOptions::None` must keep reaching the
    /// original pair so the qualified static-buffer path is unchanged.
    pub(crate) fn set_raw_vertex_data_with_options(
        &self,
        buffer: sys::CNA_Handle,
        offset_in_bytes: Option<u64>,
        data: &[u8],
        vertex_count: u64,
        vertex_stride: u32,
        options: sys::CNA_SetDataOptions,
    ) -> Result<()> {
        let byte_count = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("vertex data array is too large"))?;
        let pointer = if data.is_empty() {
            core::ptr::null()
        } else {
            data.as_ptr().cast()
        };
        // SAFETY: the byte slice describes exactly byte_count readable bytes and is copied.
        let result = unsafe {
            match offset_in_bytes {
                Some(offset) => (self.vertex_buffer_set_data_raw_at_with_options)(
                    buffer,
                    offset,
                    pointer,
                    byte_count,
                    vertex_count,
                    vertex_stride,
                    options,
                ),
                None => (self.vertex_buffer_set_data_raw_with_options)(
                    buffer,
                    pointer,
                    byte_count,
                    vertex_count,
                    vertex_stride,
                    options,
                ),
            }
        };
        self.check(result)
    }

    pub(crate) fn get_raw_vertex_data(
        &self,
        buffer: sys::CNA_Handle,
        offset_in_bytes: u64,
        destination: &mut [u8],
        vertex_count: u64,
        vertex_stride: u32,
    ) -> Result<()> {
        let byte_count = u64::try_from(destination.len())
            .map_err(|_| CnaError::InvalidInput("vertex data array is too large"))?;
        let pointer = if destination.is_empty() {
            core::ptr::null_mut()
        } else {
            destination.as_mut_ptr().cast()
        };
        // SAFETY: the slice describes exactly byte_count writable bytes; CNA writes atomically.
        self.check(unsafe {
            (self.vertex_buffer_get_data_raw)(
                buffer,
                offset_in_bytes,
                pointer,
                byte_count,
                vertex_count,
                vertex_stride,
            )
        })
    }

    pub(crate) fn destroy_vertex_buffer(&self, buffer: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the caller transfers exactly-once ownership of a live handle.
        self.check(unsafe { (self.vertex_buffer_destroy)(buffer) })
    }

    pub(crate) fn create_index_buffer(
        &self,
        device: sys::CNA_Handle,
        info: &sys::CNA_IndexBufferCreateInfo,
        buffer: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the callback-scoped device and versioned input/output remain live.
        self.check(unsafe { (self.index_buffer_create)(device, info, buffer) })
    }

    pub(crate) fn index_buffer_info(
        &self,
        buffer: sys::CNA_Handle,
        info: &mut sys::CNA_IndexBufferInfo,
    ) -> Result<()> {
        // SAFETY: the owned handle and initialized versioned output remain live.
        self.check(unsafe { (self.index_buffer_get_info)(buffer, info) })
    }

    pub(crate) fn set_index_data<T: Copy>(
        &self,
        buffer: sys::CNA_Handle,
        offset_in_bytes: Option<u64>,
        transfer: &sys::CNA_IndexBufferTransfer,
        data: &[T],
    ) -> Result<()> {
        let capacity = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("index data array is too large"))?;
        let pointer = if data.is_empty() {
            core::ptr::null()
        } else {
            data.as_ptr().cast()
        };
        // SAFETY: callers pass only u16 or u32 slices matching index_element_size.
        let result = unsafe {
            match offset_in_bytes {
                Some(offset) => {
                    (self.index_buffer_set_data_at)(buffer, offset, transfer, pointer, capacity)
                }
                None => (self.index_buffer_set_data)(buffer, transfer, pointer, capacity),
            }
        };
        self.check(result)
    }

    pub(crate) fn get_index_data<T: Copy>(
        &self,
        buffer: sys::CNA_Handle,
        transfer: &sys::CNA_IndexBufferTransfer,
        destination: &mut [T],
        required: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(destination.len())
            .map_err(|_| CnaError::InvalidInput("index data array is too large"))?;
        let pointer = if destination.is_empty() {
            core::ptr::null_mut()
        } else {
            destination.as_mut_ptr().cast()
        };
        // SAFETY: callers pass only u16 or u32 slices matching index_element_size.
        self.check(unsafe {
            (self.index_buffer_get_data)(buffer, transfer, pointer, capacity, required)
        })
    }

    pub(crate) fn destroy_index_buffer(&self, buffer: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the caller transfers exactly-once ownership of a live handle.
        self.check(unsafe { (self.index_buffer_destroy)(buffer) })
    }

    pub(crate) fn create_sprite_batch(
        &self,
        device: sys::CNA_Handle,
        batch: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is valid.
        self.check(unsafe { (self.sprite_batch_create)(device, batch) })
    }

    pub(crate) fn begin_sprite_batch(
        &self,
        batch: sys::CNA_Handle,
        info: &sys::CNA_SpriteBatchBeginInfo,
    ) -> Result<()> {
        // SAFETY: the owned handle and versioned input are live.
        self.check(unsafe { (self.sprite_batch_begin)(batch, info) })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn begin_sprite_batch_with_states(
        &self,
        batch: sys::CNA_Handle,
        sort_mode: sys::CNA_SpriteSortMode,
        blend: &sys::CNA_BlendState,
        sampler: &sys::CNA_SamplerState,
        depth_stencil: &sys::CNA_DepthStencilState,
        rasterizer: &sys::CNA_RasterizerState,
    ) -> Result<()> {
        // SAFETY: all descriptors are complete version-one POD values copied
        // synchronously by CNA and the batch handle remains owned by Rust.
        self.check(unsafe {
            (self.sprite_batch_begin_with_states)(
                batch,
                sort_mode,
                blend,
                sampler,
                depth_stencil,
                rasterizer,
            )
        })
    }

    pub(crate) fn submit_sprite(
        &self,
        batch: sys::CNA_Handle,
        command: &sys::CNA_SpriteCommand,
    ) -> Result<()> {
        // SAFETY: both the owned handle and POD command are live; count is one.
        self.check(unsafe { (self.sprite_batch_submit_many)(batch, command, 1) })
    }

    pub(crate) fn draw_sprite_string(
        &self,
        batch: sys::CNA_Handle,
        command: &sys::CNA_SpriteTextCommand,
    ) -> Result<()> {
        // SAFETY: the command is complete version-one POD and its UTF-8 view
        // remains live for the synchronous native call.
        self.check(unsafe { (self.sprite_batch_draw_string)(batch, command) })
    }

    pub(crate) fn end_sprite_batch(&self, batch: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the wrapper enforces an active begin/end interval.
        self.check(unsafe { (self.sprite_batch_end)(batch) })
    }

    pub(crate) fn destroy_sprite_batch(&self, batch: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the caller transfers exactly-once ownership of a live handle.
        self.check(unsafe { (self.sprite_batch_destroy)(batch) })
    }

    pub(crate) fn create_sprite_font(
        &self,
        info: &sys::CNA_SpriteFontCreateInfo,
        font: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the versioned descriptor and complete glyph slice referenced
        // by it remain live for this synchronous copying call.
        self.check(unsafe { (self.sprite_font_create)(info, font) })
    }

    pub(crate) fn sprite_font_info(
        &self,
        font: sys::CNA_Handle,
        info: &mut sys::CNA_SpriteFontInfo,
    ) -> Result<()> {
        // SAFETY: the owned handle is live and the output is initialized.
        self.check(unsafe { (self.sprite_font_get_info)(font, info) })
    }

    pub(crate) fn copy_sprite_font_characters(
        &self,
        font: sys::CNA_Handle,
        destination: &mut [sys::CNA_Char16],
        count: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(destination.len())
            .map_err(|_| CnaError::InvalidInput("SpriteFont character table is too large"))?;
        let pointer = if destination.is_empty() {
            core::ptr::null_mut()
        } else {
            destination.as_mut_ptr()
        };
        // SAFETY: the output points to exactly `capacity` writable elements.
        self.check(unsafe { (self.sprite_font_copy_characters)(font, pointer, capacity, count) })
    }

    pub(crate) fn copy_sprite_font_glyphs(
        &self,
        font: sys::CNA_Handle,
        destination: &mut [sys::CNA_SpriteFontGlyph],
        count: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(destination.len())
            .map_err(|_| CnaError::InvalidInput("SpriteFont glyph table is too large"))?;
        let pointer = if destination.is_empty() {
            core::ptr::null_mut()
        } else {
            destination.as_mut_ptr()
        };
        // SAFETY: the output points to exactly `capacity` writable elements.
        self.check(unsafe { (self.sprite_font_copy_glyphs)(font, pointer, capacity, count) })
    }

    pub(crate) fn set_sprite_font_default_character(
        &self,
        font: sys::CNA_Handle,
        value: Option<sys::CNA_Char16>,
    ) -> Result<()> {
        let (has_value, value) = value.map_or((sys::CNA_FALSE, 0), |value| (sys::CNA_TRUE, value));
        // SAFETY: the owned handle is live and the scalar values are valid.
        self.check(unsafe { (self.sprite_font_set_default_character)(font, has_value, value) })
    }

    pub(crate) fn set_sprite_font_line_spacing(
        &self,
        font: sys::CNA_Handle,
        value: i32,
    ) -> Result<()> {
        // SAFETY: the owned handle is live and the scalar is copied.
        self.check(unsafe { (self.sprite_font_set_line_spacing)(font, value) })
    }

    pub(crate) fn set_sprite_font_spacing(&self, font: sys::CNA_Handle, value: f32) -> Result<()> {
        // SAFETY: the owned handle is live and Rust validates finiteness.
        self.check(unsafe { (self.sprite_font_set_spacing)(font, value) })
    }

    pub(crate) fn measure_sprite_font(
        &self,
        font: sys::CNA_Handle,
        text: sys::CNA_StringView,
        size: &mut sys::CNA_Vector2,
    ) -> Result<()> {
        // SAFETY: the UTF-8 view remains live for the synchronous call and the
        // output points to one writable vector.
        self.check(unsafe { (self.sprite_font_measure_utf8)(font, text, size) })
    }

    pub(crate) fn destroy_sprite_font(&self, font: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the caller transfers exactly-once ownership of a live handle.
        self.check(unsafe { (self.sprite_font_destroy)(font) })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn begin_sprite_batch_with_effect(
        &self,
        batch: sys::CNA_Handle,
        sort_mode: sys::CNA_SpriteSortMode,
        blend: &sys::CNA_BlendState,
        sampler: &sys::CNA_SamplerState,
        depth_stencil: &sys::CNA_DepthStencilState,
        rasterizer: &sys::CNA_RasterizerState,
        effect: sys::CNA_Handle,
        transform: Option<&sys::CNA_Matrix>,
    ) -> Result<()> {
        // SAFETY: descriptors and optional matrix remain live for this
        // synchronous call; Rust validates all resource/device identities.
        self.check(unsafe {
            (self.sprite_batch_begin_with_effect)(
                batch,
                sort_mode,
                blend,
                sampler,
                depth_stencil,
                rasterizer,
                effect,
                transform.map_or(core::ptr::null(), |value| value as *const sys::CNA_Matrix),
            )
        })
    }

    pub(crate) fn create_empty_effect(
        &self,
        device: sys::CNA_Handle,
        effect: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the device is live during the callback and output is valid.
        self.check(unsafe { (self.effect_create_empty)(device, effect) })
    }

    pub(crate) fn create_compiled_effect(
        &self,
        device: sys::CNA_Handle,
        code: &[u8],
        effect: &mut sys::CNA_Handle,
    ) -> Result<()> {
        let count = u64::try_from(code.len())
            .map_err(|_| CnaError::InvalidInput("effect bytecode is too large"))?;
        // SAFETY: the nonempty byte slice and output remain live for the call.
        self.check(unsafe { (self.effect_create_compiled)(device, code.as_ptr(), count, effect) })
    }

    pub(crate) fn create_effect_material(
        &self,
        source: sys::CNA_Handle,
        effect: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: source is a live owned effect and output is valid.
        self.check(unsafe { (self.effect_material_create)(source, effect) })
    }

    pub(crate) fn clone_effect(
        &self,
        source: sys::CNA_Handle,
        effect: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: source is live and output receives independent ownership.
        self.check(unsafe { (self.effect_clone)(source, effect) })
    }

    pub(crate) fn dispose_effect_contents(&self, effect: sys::CNA_Handle) -> Result<()> {
        // SAFETY: handle ownership is retained; this only releases contents.
        self.check(unsafe { (self.effect_dispose)(effect) })
    }

    pub(crate) fn destroy_effect(&self, effect: sys::CNA_Handle) -> Result<()> {
        // SAFETY: caller transfers exactly-once effect-handle ownership.
        self.check(unsafe { (self.effect_destroy)(effect) })
    }

    pub(crate) fn apply_effect(&self, effect: sys::CNA_Handle) -> Result<()> {
        // SAFETY: handle and owning device are live.
        self.check(unsafe { (self.effect_apply)(effect) })
    }

    pub(crate) fn effect_parameters(
        &self,
        effect: sys::CNA_Handle,
        collection: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: handle is live and output receives an owned collection view.
        self.check(unsafe { (self.effect_get_parameters)(effect, collection) })
    }

    pub(crate) fn effect_techniques(
        &self,
        effect: sys::CNA_Handle,
        collection: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: handle is live and output receives an owned collection view.
        self.check(unsafe { (self.effect_get_techniques)(effect, collection) })
    }

    pub(crate) fn current_effect_technique(
        &self,
        effect: sys::CNA_Handle,
        technique: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: handle is live and output receives an owned technique view.
        self.check(unsafe { (self.effect_get_current_technique)(effect, technique) })
    }

    pub(crate) fn set_current_effect_technique(
        &self,
        effect: sys::CNA_Handle,
        technique: sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: both handles were validated as live and parent-compatible.
        self.check(unsafe { (self.effect_set_current_technique)(effect, technique) })
    }

    fn copy_effect_text(
        &self,
        handle: sys::CNA_Handle,
        count_fn: unsafe extern "C" fn(sys::CNA_Handle, *mut u64) -> sys::CNA_Result,
        copy_fn: unsafe extern "C" fn(
            sys::CNA_Handle,
            *mut core::ffi::c_char,
            u64,
            *mut u64,
        ) -> sys::CNA_Result,
    ) -> Result<String> {
        let mut count = 0;
        // SAFETY: output count points to one writable scalar.
        self.check(unsafe { count_fn(handle, &mut count) })?;
        let length = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("native effect text is too large"))?;
        let mut bytes = vec![0_u8; length];
        let mut copied = count;
        // SAFETY: destination has exactly count writable bytes.
        self.check(unsafe { copy_fn(handle, bytes.as_mut_ptr().cast(), count, &mut copied) })?;
        if copied != count {
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                category: ErrorCategory::None,
                message: "CNA changed an effect text value between count and copy".to_owned(),
            });
        }
        String::from_utf8(bytes).map_err(|_| CnaError::Native {
            code: sys::CNA_RESULT_ENCODING,
            category: ErrorCategory::None,
            message: "CNA returned invalid UTF-8 effect text".to_owned(),
        })
    }

    pub(crate) fn effect_annotation_name(&self, handle: sys::CNA_Handle) -> Result<String> {
        self.copy_effect_text(
            handle,
            self.effect_annotation_get_name_byte_count,
            self.effect_annotation_copy_name,
        )
    }

    pub(crate) fn effect_annotation_semantic(&self, handle: sys::CNA_Handle) -> Result<String> {
        self.copy_effect_text(
            handle,
            self.effect_annotation_get_semantic_byte_count,
            self.effect_annotation_copy_semantic,
        )
    }

    pub(crate) fn effect_annotation_string(&self, handle: sys::CNA_Handle) -> Result<String> {
        self.copy_effect_text(
            handle,
            self.effect_annotation_get_value_string_byte_count,
            self.effect_annotation_copy_value_string,
        )
    }

    pub(crate) fn effect_parameter_name(&self, handle: sys::CNA_Handle) -> Result<String> {
        self.copy_effect_text(
            handle,
            self.effect_parameter_get_name_byte_count,
            self.effect_parameter_copy_name,
        )
    }

    pub(crate) fn effect_parameter_semantic(&self, handle: sys::CNA_Handle) -> Result<String> {
        self.copy_effect_text(
            handle,
            self.effect_parameter_get_semantic_byte_count,
            self.effect_parameter_copy_semantic,
        )
    }

    pub(crate) fn effect_parameter_string(&self, handle: sys::CNA_Handle) -> Result<String> {
        self.copy_effect_text(
            handle,
            self.effect_parameter_get_value_string_byte_count,
            self.effect_parameter_copy_value_string,
        )
    }

    pub(crate) fn effect_pass_name(&self, handle: sys::CNA_Handle) -> Result<String> {
        self.copy_effect_text(
            handle,
            self.effect_pass_get_name_byte_count,
            self.effect_pass_copy_name,
        )
    }

    pub(crate) fn effect_technique_name(&self, handle: sys::CNA_Handle) -> Result<String> {
        self.copy_effect_text(
            handle,
            self.effect_technique_get_name_byte_count,
            self.effect_technique_copy_name,
        )
    }

    pub(crate) fn effect_annotation_info(
        &self,
        handle: sys::CNA_Handle,
        info: &mut sys::CNA_EffectAnnotationInfo,
    ) -> Result<()> {
        // SAFETY: output is a writable versioned descriptor.
        self.check(unsafe { (self.effect_annotation_get_info)(handle, info) })
    }

    pub(crate) fn effect_parameter_info(
        &self,
        handle: sys::CNA_Handle,
        info: &mut sys::CNA_EffectParameterInfo,
    ) -> Result<()> {
        // SAFETY: output is a writable versioned descriptor.
        self.check(unsafe { (self.effect_parameter_get_info)(handle, info) })
    }

    pub(crate) fn effect_annotation_value<T: Copy + Default>(
        &self,
        handle: sys::CNA_Handle,
        function: unsafe extern "C" fn(sys::CNA_Handle, *mut T) -> sys::CNA_Result,
    ) -> Result<T> {
        let mut value = T::default();
        // SAFETY: the selected C function and T are paired by the private caller.
        self.check(unsafe { function(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn annotation_boolean(&self, handle: sys::CNA_Handle) -> Result<sys::CNA_Bool> {
        self.effect_annotation_value(handle, self.effect_annotation_get_value_boolean)
    }
    pub(crate) fn annotation_int32(&self, handle: sys::CNA_Handle) -> Result<i32> {
        self.effect_annotation_value(handle, self.effect_annotation_get_value_int32)
    }
    pub(crate) fn annotation_single(&self, handle: sys::CNA_Handle) -> Result<f32> {
        self.effect_annotation_value(handle, self.effect_annotation_get_value_single)
    }
    pub(crate) fn annotation_vector2(&self, handle: sys::CNA_Handle) -> Result<sys::CNA_Vector2> {
        self.effect_annotation_value(handle, self.effect_annotation_get_value_vector2)
    }
    pub(crate) fn annotation_vector3(&self, handle: sys::CNA_Handle) -> Result<sys::CNA_Vector3> {
        self.effect_annotation_value(handle, self.effect_annotation_get_value_vector3)
    }
    pub(crate) fn annotation_vector4(&self, handle: sys::CNA_Handle) -> Result<sys::CNA_Vector4> {
        self.effect_annotation_value(handle, self.effect_annotation_get_value_vector4)
    }
    pub(crate) fn annotation_matrix(&self, handle: sys::CNA_Handle) -> Result<sys::CNA_Matrix> {
        self.effect_annotation_value(handle, self.effect_annotation_get_value_matrix)
    }

    pub(crate) fn effect_parameter_value<T: Copy + Default>(
        &self,
        handle: sys::CNA_Handle,
        value_type: sys::CNA_EffectValueType,
    ) -> Result<T> {
        let mut value = T::default();
        // SAFETY: private callers pair the tagged identity with the exact POD T.
        self.check(unsafe {
            (self.effect_parameter_get_value)(
                handle,
                value_type,
                core::ptr::addr_of_mut!(value).cast(),
            )
        })?;
        Ok(value)
    }

    pub(crate) fn effect_parameter_values<T: Copy + Default>(
        &self,
        handle: sys::CNA_Handle,
        value_type: sys::CNA_EffectValueType,
        count: usize,
    ) -> Result<Vec<T>> {
        let native_count = u64::try_from(count)
            .map_err(|_| CnaError::InvalidInput("effect value count is too large"))?;
        let mut values = vec![T::default(); count];
        let mut actual = 0;
        let pointer = if values.is_empty() {
            core::ptr::null_mut()
        } else {
            values.as_mut_ptr().cast()
        };
        // SAFETY: private callers pair T with the tag and destination capacity.
        self.check(unsafe {
            (self.effect_parameter_get_values)(
                handle,
                value_type,
                native_count,
                pointer,
                native_count,
                &mut actual,
            )
        })?;
        values.truncate(usize::try_from(actual).map_err(|_| CnaError::Native {
            code: sys::CNA_RESULT_OVERFLOW,
            category: ErrorCategory::None,
            message: "CNA returned an oversized effect array count".to_owned(),
        })?);
        Ok(values)
    }

    pub(crate) fn set_effect_parameter_value<T: Copy>(
        &self,
        handle: sys::CNA_Handle,
        value_type: sys::CNA_EffectValueType,
        value: &T,
    ) -> Result<()> {
        // SAFETY: private callers pair T with the tagged native overload.
        self.check(unsafe {
            (self.effect_parameter_set_value)(handle, value_type, (value as *const T).cast())
        })
    }

    pub(crate) fn set_effect_parameter_values<T: Copy>(
        &self,
        handle: sys::CNA_Handle,
        value_type: sys::CNA_EffectValueType,
        values: &[T],
    ) -> Result<()> {
        let count = u64::try_from(values.len())
            .map_err(|_| CnaError::InvalidInput("effect value count is too large"))?;
        let pointer = if values.is_empty() {
            core::ptr::null()
        } else {
            values.as_ptr().cast()
        };
        // SAFETY: private callers pair T with the tag and slice capacity.
        self.check(unsafe {
            (self.effect_parameter_set_values)(handle, value_type, pointer, count)
        })
    }

    pub(crate) fn set_effect_parameter_string(
        &self,
        handle: sys::CNA_Handle,
        value: sys::CNA_StringView,
    ) -> Result<()> {
        // SAFETY: string view remains live for the synchronous copying call.
        self.check(unsafe { (self.effect_parameter_set_value_string)(handle, value) })
    }

    pub(crate) fn effect_parameter_texture(
        &self,
        handle: sys::CNA_Handle,
        texture_type: sys::CNA_EffectTextureType,
    ) -> Result<sys::CNA_Handle> {
        let mut texture = sys::CNA_INVALID_HANDLE;
        // SAFETY: output points to one writable handle.
        self.check(unsafe {
            (self.effect_parameter_get_value_texture)(handle, texture_type, &mut texture)
        })?;
        Ok(texture)
    }

    pub(crate) fn set_effect_parameter_texture(
        &self,
        handle: sys::CNA_Handle,
        texture_type: sys::CNA_EffectTextureType,
        texture: sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the wrapper validates the texture handle and device identity.
        self.check(unsafe {
            (self.effect_parameter_set_value_texture)(handle, texture_type, texture)
        })
    }

    pub(crate) fn effect_parameter_child_collection(
        &self,
        handle: sys::CNA_Handle,
        structure_members: bool,
        collection: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: output receives one owned child collection view.
        let result = unsafe {
            if structure_members {
                (self.effect_parameter_get_structure_members)(handle, collection)
            } else {
                (self.effect_parameter_get_elements)(handle, collection)
            }
        };
        self.check(result)
    }

    pub(crate) fn effect_parameter_annotations(
        &self,
        handle: sys::CNA_Handle,
        collection: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: output receives one owned annotation collection view.
        self.check(unsafe { (self.effect_parameter_get_annotations)(handle, collection) })
    }

    pub(crate) fn effect_pass_annotations(
        &self,
        handle: sys::CNA_Handle,
        collection: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: output receives one owned annotation collection view.
        self.check(unsafe { (self.effect_pass_get_annotations)(handle, collection) })
    }

    pub(crate) fn effect_technique_passes(
        &self,
        handle: sys::CNA_Handle,
        collection: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: output receives one owned pass collection view.
        self.check(unsafe { (self.effect_technique_get_passes)(handle, collection) })
    }

    pub(crate) fn effect_technique_annotations(
        &self,
        handle: sys::CNA_Handle,
        collection: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: output receives one owned annotation collection view.
        self.check(unsafe { (self.effect_technique_get_annotations)(handle, collection) })
    }

    pub(crate) fn create_effect_annotation(
        &self,
        info: &sys::CNA_EffectAnnotationCreateInfo,
        annotation: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the versioned descriptor and referenced slices remain live
        // for this synchronous copying call.
        self.check(unsafe { (self.effect_annotation_create)(info, annotation) })
    }

    pub(crate) fn add_effect_annotation(
        &self,
        collection: sys::CNA_Handle,
        annotation: sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: both owned handles are live; CNA copies the annotation value.
        self.check(unsafe { (self.effect_annotation_collection_add)(collection, annotation) })
    }

    pub(crate) fn add_effect_parameter(
        &self,
        collection: sys::CNA_Handle,
        info: &sys::CNA_EffectParameterCreateInfo,
        parameter: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the versioned descriptor remains live and output receives an
        // independently owned stable parameter view.
        self.check(unsafe {
            (self.effect_parameter_collection_add_create)(collection, info, parameter)
        })
    }

    pub(crate) fn add_effect_technique(
        &self,
        collection: sys::CNA_Handle,
        name: sys::CNA_StringView,
        technique: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the text view is synchronously copied and output is valid.
        self.check(unsafe {
            (self.effect_technique_collection_add_named)(collection, name, technique)
        })
    }

    pub(crate) fn add_effect_pass(
        &self,
        collection: sys::CNA_Handle,
        name: sys::CNA_StringView,
        technique_identity: u64,
        pass: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the text view is synchronously copied and output is valid.
        self.check(unsafe {
            (self.effect_pass_collection_add_create)(collection, name, technique_identity, pass)
        })
    }

    pub(crate) fn effect_collection_count(&self, handle: sys::CNA_Handle, kind: u8) -> Result<u64> {
        let mut count = 0;
        // SAFETY: every branch has the identical count signature and kind is private.
        let result = unsafe {
            match kind {
                0 => (self.effect_annotation_collection_get_count)(handle, &mut count),
                1 => (self.effect_parameter_collection_get_count)(handle, &mut count),
                2 => (self.effect_pass_collection_get_count)(handle, &mut count),
                3 => (self.effect_technique_collection_get_count)(handle, &mut count),
                _ => return Err(CnaError::InvalidInput("unknown effect collection kind")),
            }
        };
        self.check(result)?;
        Ok(count)
    }

    pub(crate) fn effect_collection_get_at(
        &self,
        handle: sys::CNA_Handle,
        kind: u8,
        index: u64,
    ) -> Result<sys::CNA_Handle> {
        let mut child = sys::CNA_INVALID_HANDLE;
        // SAFETY: every branch has the identical indexed-view signature.
        let result = unsafe {
            match kind {
                0 => (self.effect_annotation_collection_get_at)(handle, index, &mut child),
                1 => (self.effect_parameter_collection_get_at)(handle, index, &mut child),
                2 => (self.effect_pass_collection_get_at)(handle, index, &mut child),
                3 => (self.effect_technique_collection_get_at)(handle, index, &mut child),
                _ => return Err(CnaError::InvalidInput("unknown effect collection kind")),
            }
        };
        self.check(result)?;
        Ok(child)
    }

    pub(crate) fn effect_collection_find(
        &self,
        handle: sys::CNA_Handle,
        kind: u8,
        value: sys::CNA_StringView,
        semantic: bool,
    ) -> Result<Option<sys::CNA_Handle>> {
        let mut found = sys::CNA_FALSE;
        let mut child = sys::CNA_INVALID_HANDLE;
        // SAFETY: every branch has the identical find-view signature.
        let result = unsafe {
            match (kind, semantic) {
                (0, false) => {
                    (self.effect_annotation_collection_find)(handle, value, &mut found, &mut child)
                }
                (1, false) => (self.effect_parameter_collection_find_name)(
                    handle, value, &mut found, &mut child,
                ),
                (1, true) => (self.effect_parameter_collection_find_semantic)(
                    handle, value, &mut found, &mut child,
                ),
                (2, false) => {
                    (self.effect_pass_collection_find)(handle, value, &mut found, &mut child)
                }
                (3, false) => {
                    (self.effect_technique_collection_find)(handle, value, &mut found, &mut child)
                }
                _ => return Err(CnaError::InvalidInput("unknown effect collection lookup")),
            }
        };
        self.check(result)?;
        Ok((found == sys::CNA_TRUE).then_some(child))
    }

    pub(crate) fn destroy_effect_view(&self, handle: sys::CNA_Handle, kind: u8) -> Result<()> {
        // SAFETY: private view wrappers transfer exactly-once ownership.
        let result = unsafe {
            match kind {
                0 => (self.effect_annotation_destroy)(handle),
                1 => (self.effect_parameter_destroy)(handle),
                2 => (self.effect_pass_destroy)(handle),
                3 => (self.effect_technique_destroy)(handle),
                4 => (self.effect_annotation_collection_destroy)(handle),
                5 => (self.effect_parameter_collection_destroy)(handle),
                6 => (self.effect_pass_collection_destroy)(handle),
                7 => (self.effect_technique_collection_destroy)(handle),
                _ => return Err(CnaError::InvalidInput("unknown effect view kind")),
            }
        };
        self.check(result)
    }

    pub(crate) fn apply_effect_pass(&self, pass: sys::CNA_Handle) -> Result<()> {
        // SAFETY: pass view and its retained owning effect are live.
        self.check(unsafe { (self.effect_pass_apply)(pass) })
    }
}
