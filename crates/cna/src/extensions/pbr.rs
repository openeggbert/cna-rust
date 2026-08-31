//! CNA's physically based rendering: materials, effects and pipeline settings.
//!
//! None of this is XNA. `BasicEffect` had a diffuse colour and a specular
//! power; there is no metallic factor, no roughness, no index of refraction,
//! no tonemapping operator and no HDR anywhere in
//! `Microsoft.Xna.Framework.Graphics`. Putting any of it there would mean
//! declaring members Microsoft never did, so it lives here.
//!
//! Availability is queried, never assumed. These routes need CNA's engine
//! layer, which is a build-time choice: a library without it answers
//! `NOT_SUPPORTED` rather than pretending, and [`engine_layer_version`] is how
//! a caller finds out before trying. Nothing here falls back to a software
//! imitation.

#![allow(clippy::missing_errors_doc)]

use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::engine::BorrowedRenderTarget;
use crate::graphics::{GraphicsDevice, GraphicsResource, Texture2D};
use crate::native::Native;
use crate::value::{Color, Matrix, Vector3, Vector4};

/// The engine-layer revision the linked library was built with.
///
/// Zero means the library has no engine layer, and every routine in this
/// module will refuse. Upstream is explicit that this is the query to make:
/// a symbol exists either way, because the exported ABI is one shape
/// regardless of which parts were built.
pub fn engine_layer_version() -> Result<i32> {
    let native = Native::process()?;
    let mut value = 0_i32;
    // SAFETY: the output is a live local of the declared type.
    native.check(unsafe { (native.runtime.engine_layer_get_version)(&mut value) })?;
    Ok(value)
}

/// The engine-layer revision as text, for logs and about-boxes.
///
/// This is one route rather than CNA's usual size-then-copy pair, so the size
/// probe answers `BUFFER_TOO_SMALL` rather than success. That is the size
/// being reported, not a failure, and treating it as one is the difference
/// between reading the string and refusing to.
pub fn engine_layer_version_string() -> Result<String> {
    let native = Native::process()?;
    let api = &native.runtime;
    let mut required = 0_u64;
    // SAFETY: a null destination with zero capacity asks for the size.
    let probe = unsafe {
        (api.engine_layer_copy_version_string)(core::ptr::null_mut(), 0, &mut required)
    };
    if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
        native.check(probe)?;
    }
    let capacity = usize::try_from(required)
        .map_err(|_| CnaError::InvalidInput("CNA text is too large"))?;
    if capacity == 0 {
        return Ok(String::new());
    }
    let mut buffer = vec![0_u8; capacity];
    let mut written = 0_u64;
    // SAFETY: the buffer holds exactly `required` writable bytes.
    native.check(unsafe {
        (api.engine_layer_copy_version_string)(
            buffer.as_mut_ptr().cast::<core::ffi::c_char>(),
            required,
            &mut written,
        )
    })?;
    let written = usize::try_from(written)
        .map_err(|_| CnaError::InvalidInput("CNA text is too large"))?;
    buffer.truncate(written.min(capacity));
    while buffer.last() == Some(&0) {
        buffer.pop();
    }
    String::from_utf8(buffer).map_err(|_| CnaError::InvalidInput("CNA text is not valid UTF-8"))
}

/// How a material's alpha is interpreted.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum AlphaMode {
    #[default]
    Opaque,
    /// Alpha-tested against a cutoff.
    Mask,
    Blend,
}

impl AlphaMode {
    const fn from_native(value: sys::CNA_AlphaModeEXT) -> Option<Self> {
        Some(match value {
            sys::CNA_ALPHA_MODE_OPAQUE_EXT => Self::Opaque,
            sys::CNA_ALPHA_MODE_MASK_EXT => Self::Mask,
            sys::CNA_ALPHA_MODE_BLEND_EXT => Self::Blend,
            _ => return None,
        })
    }

    const fn to_native(self) -> sys::CNA_AlphaModeEXT {
        match self {
            Self::Opaque => sys::CNA_ALPHA_MODE_OPAQUE_EXT,
            Self::Mask => sys::CNA_ALPHA_MODE_MASK_EXT,
            Self::Blend => sys::CNA_ALPHA_MODE_BLEND_EXT,
        }
    }
}

/// Which tonemapping operator maps HDR to the display.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum TonemappingMode {
    None,
    Reinhard,
    Filmic,
    Aces,
}

/// The overall render-quality preset.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum RenderQuality {
    Low,
    Medium,
    High,
    Ultra,
}

/// The shadow-map quality preset.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ShadowQuality {
    Disabled,
    Low,
    Medium,
    High,
    Ultra,
}

macro_rules! identity {
    ($name:ident, $native:ty, $($variant:ident => $constant:ident),+ $(,)?) => {
        impl $name {
            pub(crate) const fn from_native(value: $native) -> Option<Self> {
                Some(match value {
                    $(sys::$constant => Self::$variant,)+
                    _ => return None,
                })
            }

            pub(crate) const fn to_native(self) -> $native {
                match self {
                    $(Self::$variant => sys::$constant,)+
                }
            }
        }
    };
}

identity!(
    TonemappingMode, sys::CNA_TonemappingMode,
    None => CNA_TONEMAPPING_MODE_NONE,
    Reinhard => CNA_TONEMAPPING_MODE_REINHARD,
    Filmic => CNA_TONEMAPPING_MODE_FILMIC,
    Aces => CNA_TONEMAPPING_MODE_ACES,
);

identity!(
    RenderQuality, sys::CNA_RenderQuality,
    Low => CNA_RENDER_QUALITY_LOW,
    Medium => CNA_RENDER_QUALITY_MEDIUM,
    High => CNA_RENDER_QUALITY_HIGH,
    Ultra => CNA_RENDER_QUALITY_ULTRA,
);

identity!(
    ShadowQuality, sys::CNA_ShadowQuality,
    Disabled => CNA_SHADOW_QUALITY_DISABLED,
    Low => CNA_SHADOW_QUALITY_LOW,
    Medium => CNA_SHADOW_QUALITY_MEDIUM,
    High => CNA_SHADOW_QUALITY_HIGH,
    Ultra => CNA_SHADOW_QUALITY_ULTRA,
);

/// Converts CNA's four-byte colour to XNA's packed one.
fn color_from_native(value: sys::CNA_Color) -> Color {
    Color::FromNonPremultipliedWithRAndGAndBAndA(
        i32::from(value.r),
        i32::from(value.g),
        i32::from(value.b),
        i32::from(value.a),
    )
}

/// A physically based material, as a value.
///
/// Textures are deliberately absent from this type. The canonical structure
/// carries non-owning handle slots, and a safe Rust value that held raw handles
/// would be a raw-handle leak; textures are set on the effect, where the
/// lifetime relationship is real.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PbrMaterial {
    pub albedo_color: Color,
    pub emissive_color: Color,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub alpha_cutoff: f32,
    pub alpha_blend_enabled: bool,
}

impl PbrMaterial {
    /// The canonical defaults, taken from CNA rather than restated here.
    ///
    /// Asking the library means a default that changes upstream changes here
    /// too, instead of this crate quietly disagreeing with the renderer.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_PbrMaterial::default();
        // SAFETY: the structure is a caller-owned output CNA fills entirely.
        native.check(unsafe { (native.runtime.pbr_material_init)(&mut value) })?;
        Ok(Self {
            albedo_color: color_from_native(value.albedo_color),
            emissive_color: color_from_native(value.emissive_color),
            metallic_factor: value.metallic_factor,
            roughness_factor: value.roughness_factor,
            normal_scale: value.normal_scale,
            occlusion_strength: value.occlusion_strength,
            alpha_cutoff: value.alpha_cutoff,
            alpha_blend_enabled: value.alpha_blend_enabled != sys::CNA_FALSE,
        })
    }
}

/// The renderer's post-processing and quality settings, as a value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPipelineSettings {
    pub exposure: f32,
    pub gamma: f32,
    pub bloom_intensity: f32,
    pub tonemapping_mode: TonemappingMode,
    pub render_quality: RenderQuality,
    pub shadow_quality: ShadowQuality,
    pub hdr_enabled: bool,
    pub bloom_enabled: bool,
    pub ssao_enabled: bool,
    pub shadows_enabled: bool,
}

impl RenderPipelineSettings {
    /// The canonical defaults, taken from CNA.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_RenderPipelineSettings::default();
        // SAFETY: the structure is a caller-owned output CNA fills entirely.
        native.check(unsafe { (native.runtime.render_pipeline_settings_init)(&mut value) })?;
        Ok(Self {
            exposure: value.exposure,
            gamma: value.gamma,
            bloom_intensity: value.bloom_intensity,
            tonemapping_mode: TonemappingMode::from_native(value.tonemapping_mode).ok_or(
                CnaError::UnsupportedRuntime("CNA named a tonemapping mode this build lacks"),
            )?,
            render_quality: RenderQuality::from_native(value.render_quality).ok_or(
                CnaError::UnsupportedRuntime("CNA named a render quality this build lacks"),
            )?,
            shadow_quality: ShadowQuality::from_native(value.shadow_quality).ok_or(
                CnaError::UnsupportedRuntime("CNA named a shadow quality this build lacks"),
            )?,
            hdr_enabled: value.hdr_enabled != sys::CNA_FALSE,
            bloom_enabled: value.bloom_enabled != sys::CNA_FALSE,
            ssao_enabled: value.ssao_enabled != sys::CNA_FALSE,
            shadows_enabled: value.shadows_enabled != sys::CNA_FALSE,
        })
    }

    /// Round-trips through CNA's own identity space.
    ///
    /// Useful as a check that a value this crate built is one CNA can name:
    /// every enum here is a closed Rust type, so the only way it can be wrong
    /// is if the mapping is.
    #[must_use]
    pub const fn native_identities(&self) -> (u32, u32, u32) {
        (
            self.tonemapping_mode.to_native(),
            self.render_quality.to_native(),
            self.shadow_quality.to_native(),
        )
    }
}

/// A physically based effect owned by this value.
///
/// Needs the engine layer. Construction is where that is discovered: a library
/// without it refuses here rather than at the first property set.
pub struct PbrEffect {
    native: Arc<Native>,
    handle: sys::CNA_EffectHandle,
    device: GraphicsDevice,
}

macro_rules! scalar_property {
    ($get:ident, $set:ident, $native_get:ident, $native_set:ident, $type:ty, $doc:literal) => {
        #[doc = $doc]
        pub fn $get(&self) -> Result<$type> {
            let mut value = <$type>::default();
            // SAFETY: the handle is owned and the output is a live local.
            self.native
                .check(unsafe { (self.native.runtime.$native_get)(self.handle, &mut value) })?;
            Ok(value)
        }

        #[doc = $doc]
        pub fn $set(&self, value: $type) -> Result<()> {
            // SAFETY: the handle is owned and the value is by value.
            self.native
                .check(unsafe { (self.native.runtime.$native_set)(self.handle, value) })
        }
    };
}

impl PbrEffect {
    /// Creates an effect on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.runtime.pbr_effect_create)(device.handle()?, &mut handle)
        })?;
        Ok(Self {
            native: Arc::clone(native),
            handle,
            device: device.clone(),
        })
    }

    /// The device this effect belongs to.
    #[must_use]
    pub const fn graphics_device(&self) -> &GraphicsDevice {
        &self.device
    }

    scalar_property!(
        metallic_factor, set_metallic_factor,
        pbr_effect_get_metallic_factor, pbr_effect_set_metallic_factor, f32,
        "How metallic the surface is, from 0 through 1."
    );
    scalar_property!(
        roughness_factor, set_roughness_factor,
        pbr_effect_get_roughness_factor, pbr_effect_set_roughness_factor, f32,
        "How rough the surface is, from 0 through 1."
    );
    scalar_property!(
        alpha, set_alpha,
        pbr_effect_get_alpha, pbr_effect_set_alpha, f32,
        "Material opacity."
    );
    scalar_property!(
        alpha_cutoff, set_alpha_cutoff,
        pbr_effect_get_alpha_cutoff, pbr_effect_set_alpha_cutoff, f32,
        "The threshold `AlphaMode::Mask` tests against."
    );
    scalar_property!(
        normal_scale, set_normal_scale,
        pbr_effect_get_normal_scale, pbr_effect_set_normal_scale, f32,
        "Normal-map intensity, where 1 is full strength."
    );
    scalar_property!(
        occlusion_strength, set_occlusion_strength,
        pbr_effect_get_occlusion_strength, pbr_effect_set_occlusion_strength, f32,
        "Ambient-occlusion strength, from 0 through 1."
    );
    scalar_property!(
        ior, set_ior,
        pbr_effect_get_ior, pbr_effect_set_ior, f32,
        "Index of refraction."
    );
    scalar_property!(
        specular_factor, set_specular_factor,
        pbr_effect_get_specular_factor, pbr_effect_set_specular_factor, f32,
        "Specular strength."
    );

    /// The albedo (base colour) factor.
    pub fn diffuse_color(&self) -> Result<Vector3> {
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.pbr_effect_get_diffuse_color)(self.handle, &mut value)
        })?;
        Ok(Vector3 {
            X: value.x,
            Y: value.y,
            Z: value.z,
        })
    }

    /// Sets the albedo (base colour) factor.
    pub fn set_diffuse_color(&self, value: Vector3) -> Result<()> {
        let native_value = sys::CNA_Vector3 {
            x: value.X,
            y: value.Y,
            z: value.Z,
        };
        // SAFETY: the vector is passed by value.
        self.native.check(unsafe {
            (self.native.runtime.pbr_effect_set_diffuse_color)(self.handle, native_value)
        })
    }

    /// The emissive factor.
    pub fn emissive_factor(&self) -> Result<Vector3> {
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.pbr_effect_get_emissive_factor)(self.handle, &mut value)
        })?;
        Ok(Vector3 {
            X: value.x,
            Y: value.y,
            Z: value.z,
        })
    }

    /// Sets the emissive factor.
    pub fn set_emissive_factor(&self, value: Vector3) -> Result<()> {
        let native_value = sys::CNA_Vector3 {
            x: value.X,
            y: value.Y,
            z: value.Z,
        };
        // SAFETY: the vector is passed by value.
        self.native.check(unsafe {
            (self.native.runtime.pbr_effect_set_emissive_factor)(self.handle, native_value)
        })
    }

    /// How the material's alpha is interpreted.
    pub fn alpha_mode(&self) -> Result<AlphaMode> {
        let mut value = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.pbr_effect_get_alpha_mode)(self.handle, &mut value)
        })?;
        AlphaMode::from_native(value).ok_or(CnaError::UnsupportedRuntime(
            "CNA named an alpha mode this build does not know",
        ))
    }

    /// Sets how the material's alpha is interpreted.
    pub fn set_alpha_mode(&self, value: AlphaMode) -> Result<()> {
        // SAFETY: the identity is checked and passed by value.
        self.native.check(unsafe {
            (self.native.runtime.pbr_effect_set_alpha_mode)(self.handle, value.to_native())
        })
    }

    /// Whether the surface is rendered from both sides.
    pub fn double_sided(&self) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.pbr_effect_get_double_sided)(self.handle, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Sets whether the surface is rendered from both sides.
    pub fn set_double_sided(&self, value: bool) -> Result<()> {
        // SAFETY: the flag is passed by value.
        self.native.check(unsafe {
            (self.native.runtime.pbr_effect_set_double_sided)(self.handle, u8::from(value))
        })
    }

    /// Whether the effect samples per-vertex colour.
    pub fn vertex_color_enabled(&self) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.pbr_effect_get_vertex_color_enabled)(self.handle, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Sets whether the effect samples per-vertex colour.
    pub fn set_vertex_color_enabled(&self, value: bool) -> Result<()> {
        // SAFETY: the flag is passed by value.
        self.native.check(unsafe {
            (self.native.runtime.pbr_effect_set_vertex_color_enabled)(self.handle, u8::from(value))
        })
    }

    /// Applies every scalar of a material value to this effect.
    ///
    /// The material's textures are not applied here, because the value type
    /// deliberately carries none: a texture is an owned resource with a
    /// lifetime, and a plain value cannot express that safely.
    pub fn apply(&self, material: PbrMaterial) -> Result<()> {
        self.set_metallic_factor(material.metallic_factor)?;
        self.set_roughness_factor(material.roughness_factor)?;
        self.set_normal_scale(material.normal_scale)?;
        self.set_occlusion_strength(material.occlusion_strength)?;
        self.set_alpha_cutoff(material.alpha_cutoff)?;
        self.set_alpha_mode(if material.alpha_blend_enabled {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        })
    }
}

impl Drop for PbrEffect {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        let _ = unsafe { (self.native.effect_destroy)(self.handle) };
    }
}

/// How transparent geometry is resolved.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum TransparencyMode {
    None,
    Sorted,
    OrderIndependent,
}

identity!(
    TransparencyMode, sys::CNA_TransparencyMode,
    None => CNA_TRANSPARENCY_MODE_NONE,
    Sorted => CNA_TRANSPARENCY_MODE_SORTED,
    OrderIndependent => CNA_TRANSPARENCY_MODE_ORDER_INDEPENDENT,
);

/// The engine layer's complete render-pipeline settings.
///
/// Fifty fields, which is why this is an owned value with accessors rather
/// than a public structure: a `#[repr(C)]` field set is the ABI's shape, not an
/// API, and exposing it would make every later CNA field addition a breaking
/// change here.
///
/// The value is meaningful on its own. `normalize` runs every field through
/// the engine's own setter and reads it back, so a caller can see what a value
/// will actually become before handing it to a pipeline -- upstream documents
/// thirty-one such corrections, ten clamping to a two-sided range and
/// twenty-one flooring.
#[derive(Clone, Copy, Debug)]
pub struct EngineRenderSettings {
    inner: sys::CNA_RenderPipelineSettingsEXT,
}

macro_rules! settings_scalar {
    ($get:ident, $set:ident, $field:ident, $type:ty, $doc:literal) => {
        #[doc = $doc]
        #[must_use]
        pub const fn $get(&self) -> $type {
            self.inner.$field
        }

        #[doc = $doc]
        pub fn $set(&mut self, value: $type) -> &mut Self {
            self.inner.$field = value;
            self
        }
    };
}

macro_rules! settings_flag {
    ($get:ident, $set:ident, $field:ident, $doc:literal) => {
        #[doc = $doc]
        #[must_use]
        pub const fn $get(&self) -> bool {
            self.inner.$field != sys::CNA_FALSE
        }

        #[doc = $doc]
        pub fn $set(&mut self, value: bool) -> &mut Self {
            self.inner.$field = if value { sys::CNA_TRUE } else { sys::CNA_FALSE };
            self
        }
    };
}

impl EngineRenderSettings {
    /// The settings exactly as the ABI carries them.
    pub(crate) const fn as_native(&self) -> &sys::CNA_RenderPipelineSettingsEXT {
        &self.inner
    }

    /// The settings, for a route that updates them in place.
    pub(crate) fn as_native_mut(&mut self) -> &mut sys::CNA_RenderPipelineSettingsEXT {
        &mut self.inner
    }

    /// Adopts a structure CNA filled, keeping the versioning fields it set.
    pub(crate) const fn from_native(inner: sys::CNA_RenderPipelineSettingsEXT) -> Self {
        Self { inner }
    }

    /// The engine's own defaults.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut inner = sys::CNA_RenderPipelineSettingsEXT {
            struct_size: core::mem::size_of::<sys::CNA_RenderPipelineSettingsEXT>() as u32,
            struct_version: 1,
            ..sys::CNA_RenderPipelineSettingsEXT::default()
        };
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.runtime.render_pipeline_settings_ext_init)(&mut inner) })?;
        Ok(Self { inner })
    }

    /// Rewrites the settings as the engine would store them.
    ///
    /// This is the difference between what a caller asked for and what the
    /// engine will use. Calling it is how a settings screen can show the value
    /// that will actually take effect rather than the one that was typed.
    pub fn normalize(&mut self) -> Result<&mut Self> {
        let native = Native::process()?;
        // SAFETY: the structure is this value's own and is updated in place.
        native.check(unsafe {
            (native.runtime.render_pipeline_settings_normalize)(&mut self.inner)
        })?;
        Ok(self)
    }

    /// Applies the preset the settings' own render quality names.
    ///
    /// Upstream derives only the fields a quality dial has been decided for --
    /// today bloom's pyramid level count and the FXAA edge threshold -- and
    /// deliberately leaves the rest alone rather than guessing. This does not
    /// paper over that.
    pub fn apply_quality_preset(&mut self) -> Result<&mut Self> {
        let native = Native::process()?;
        // SAFETY: the structure is this value's own and is updated in place.
        native.check(unsafe {
            (native.runtime.render_pipeline_settings_apply_render_quality_preset)(&mut self.inner)
        })?;
        Ok(self)
    }

    /// Applies serialized settings text, answering how many fields it recognised.
    ///
    /// Unrecognised fields are skipped rather than refused, which is what makes
    /// the count meaningful: a caller compares it with what it meant to set and
    /// can tell a typo from a stale key.
    pub fn apply_from_text(&mut self, text: &str) -> Result<usize> {
        let native = Native::process()?;
        let view = sys::CNA_StringView {
            data: text.as_ptr().cast::<core::ffi::c_char>(),
            byte_length: text.len() as u64,
        };
        let mut applied = 0_i32;
        // SAFETY: `text` is borrowed for the duration of the call and the
        // structure is this value's own.
        native.check(unsafe {
            (native.runtime.render_pipeline_settings_apply_from_string)(
                &mut self.inner,
                view,
                &mut applied,
            )
        })?;
        usize::try_from(applied)
            .map_err(|_| CnaError::InvalidInput("CNA reported a negative applied-field count"))
    }

    settings_flag!(hdr_enabled, set_hdr_enabled, hdr_enabled, "Whether HDR rendering is on.");
    settings_flag!(bloom_enabled, set_bloom_enabled, bloom_enabled, "Whether the bloom pass runs.");
    settings_flag!(ssao_enabled, set_ssao_enabled, ssao_enabled, "Whether the SSAO pass runs.");
    settings_flag!(ssr_enabled, set_ssr_enabled, ssr_enabled, "Whether screen-space reflections run.");
    settings_flag!(fxaa_enabled, set_fxaa_enabled, fxaa_enabled, "Whether FXAA runs.");
    settings_flag!(dof_enabled, set_dof_enabled, dof_enabled, "Whether depth of field runs.");
    settings_flag!(shadows_enabled, set_shadows_enabled, shadows_enabled, "Whether shadows render.");
    settings_scalar!(exposure, set_exposure, exposure, f32, "Scene exposure multiplier.");
    settings_scalar!(gamma, set_gamma, gamma, f32, "Display gamma.");
    settings_scalar!(bloom_intensity, set_bloom_intensity, bloom_intensity, f32, "Bloom intensity.");
    settings_scalar!(bloom_threshold, set_bloom_threshold, bloom_threshold, f32, "Bloom luminance threshold.");
    settings_scalar!(bloom_iterations, set_bloom_iterations, bloom_iterations, i32, "Bloom pyramid level count.");
    settings_scalar!(ssao_radius, set_ssao_radius, ssao_radius, f32, "SSAO sampling radius.");
    settings_scalar!(ssao_intensity, set_ssao_intensity, ssao_intensity, f32, "SSAO intensity.");
    settings_scalar!(ssao_sample_count, set_ssao_sample_count, ssao_sample_count, i32, "SSAO sample count.");
    settings_scalar!(ssr_max_distance, set_ssr_max_distance, ssr_max_distance, f32, "How far SSR marches.");
    settings_scalar!(ssr_step_count, set_ssr_step_count, ssr_step_count, i32, "How many SSR steps are taken.");
    settings_scalar!(fxaa_edge_threshold, set_fxaa_edge_threshold, fxaa_edge_threshold_ext, f32, "FXAA edge threshold.");
    settings_scalar!(motion_blur_strength, set_motion_blur_strength, motion_blur_strength, f32, "Motion-blur strength.");
    settings_scalar!(film_grain_intensity, set_film_grain_intensity, film_grain_intensity, f32, "Film-grain intensity.");

    /// The tonemapping operator.
    pub fn tonemapping_mode(&self) -> Result<TonemappingMode> {
        TonemappingMode::from_native(self.inner.tonemapping_mode).ok_or(
            CnaError::UnsupportedRuntime("CNA named a tonemapping mode this build lacks"),
        )
    }

    /// Sets the tonemapping operator.
    pub fn set_tonemapping_mode(&mut self, value: TonemappingMode) -> &mut Self {
        self.inner.tonemapping_mode = value.to_native();
        self
    }

    /// The overall render-quality preset.
    pub fn render_quality(&self) -> Result<RenderQuality> {
        RenderQuality::from_native(self.inner.render_quality).ok_or(
            CnaError::UnsupportedRuntime("CNA named a render quality this build lacks"),
        )
    }

    /// Sets the overall render-quality preset.
    pub fn set_render_quality(&mut self, value: RenderQuality) -> &mut Self {
        self.inner.render_quality = value.to_native();
        self
    }

    /// The shadow-map quality preset.
    pub fn shadow_quality(&self) -> Result<ShadowQuality> {
        ShadowQuality::from_native(self.inner.shadow_quality).ok_or(
            CnaError::UnsupportedRuntime("CNA named a shadow quality this build lacks"),
        )
    }

    /// Sets the shadow-map quality preset.
    pub fn set_shadow_quality(&mut self, value: ShadowQuality) -> &mut Self {
        self.inner.shadow_quality = value.to_native();
        self
    }

    /// How transparent geometry is resolved.
    pub fn transparency_mode(&self) -> Result<TransparencyMode> {
        TransparencyMode::from_native(self.inner.transparency_mode).ok_or(
            CnaError::UnsupportedRuntime("CNA named a transparency mode this build lacks"),
        )
    }

    /// Sets how transparent geometry is resolved.
    pub fn set_transparency_mode(&mut self, value: TransparencyMode) -> &mut Self {
        self.inner.transparency_mode = value.to_native();
        self
    }
}

/// How one texture's coordinates are transformed before sampling.
///
/// glTF's `KHR_texture_transform`, per slot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextureTransform {
    /// Translation, applied after scaling and rotation.
    pub offset: (f32, f32),
    /// Per-axis scale.
    pub scale: (f32, f32),
    /// Counter-clockwise rotation, in radians.
    pub rotation: f32,
}

impl TextureTransform {
    const fn from_native(value: sys::CNA_TextureTransformEXT) -> Self {
        Self {
            offset: (value.offset.x, value.offset.y),
            scale: (value.scale.x, value.scale.y),
            rotation: value.rotation,
        }
    }

    const fn to_native(self) -> sys::CNA_TextureTransformEXT {
        sys::CNA_TextureTransformEXT {
            struct_size: core::mem::size_of::<sys::CNA_TextureTransformEXT>() as u32,
            struct_version: 1,
            offset: sys::CNA_Vector2 {
                x: self.offset.0,
                y: self.offset.1,
            },
            scale: sys::CNA_Vector2 {
                x: self.scale.0,
                y: self.scale.1,
            },
            rotation: self.rotation,
        }
    }
}

/// The number of per-slot state entries a material carries.
///
/// Seven, in the importer's own order -- base colour, normal,
/// metallic-roughness, occlusion, emissive, specular, specular colour. This is
/// deliberately **not** the same as the eight texture *names*
/// [`CnbMaterialTexture`](crate::extensions::content::CnbMaterialTexture)
/// addresses, which include `DualTextureEffect`'s second layer; upstream warns
/// that confusing the two index spaces is a real trap, so they are separate
/// types here and neither can be passed where the other belongs.
pub const TEXTURE_SLOT_COUNT: usize = 7;

/// A material's per-slot state entry.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum TextureSlot {
    BaseColor,
    Normal,
    MetallicRoughness,
    Occlusion,
    Emissive,
    Specular,
    SpecularColor,
}

impl TextureSlot {
    /// Every slot, in the importer's own order.
    pub const ALL: [Self; TEXTURE_SLOT_COUNT] = [
        Self::BaseColor,
        Self::Normal,
        Self::MetallicRoughness,
        Self::Occlusion,
        Self::Emissive,
        Self::Specular,
        Self::SpecularColor,
    ];

    const fn index(self) -> usize {
        match self {
            Self::BaseColor => 0,
            Self::Normal => 1,
            Self::MetallicRoughness => 2,
            Self::Occlusion => 3,
            Self::Emissive => 4,
            Self::Specular => 5,
            Self::SpecularColor => 6,
        }
    }
}

/// The complete PBR material, including its per-slot state.
///
/// This is the shape `PbrEffect::apply_full` and `extract_full` exchange, and
/// it is an owned value with accessors rather than a public structure for the
/// same reason [`EngineRenderSettings`] is: the `repr(C)` field set is the
/// ABI's shape, and a later CNA field would otherwise be a breaking change.
///
/// Textures are absent here too. The canonical structure has non-owning handle
/// slots; a safe Rust value holding them would be a raw-handle leak, and the
/// lifetime relationship belongs on the effect.
#[derive(Clone, Copy, Debug)]
pub struct PbrMaterialFull {
    inner: sys::CNA_PbrMaterialEXT,
}

impl PbrMaterialFull {
    /// The canonical defaults, taken from CNA.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut inner = sys::CNA_PbrMaterialEXT {
            struct_size: core::mem::size_of::<sys::CNA_PbrMaterialEXT>() as u32,
            struct_version: 1,
            ..sys::CNA_PbrMaterialEXT::default()
        };
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.runtime.pbr_material_init_ext)(&mut inner) })?;
        Ok(Self { inner })
    }

    settings_scalar!(metallic_factor, set_metallic_factor, metallic_factor, f32, "Metallic factor.");
    settings_scalar!(roughness_factor, set_roughness_factor, roughness_factor, f32, "Roughness factor.");
    settings_scalar!(normal_scale, set_normal_scale, normal_scale, f32, "Normal-map intensity.");
    settings_scalar!(occlusion_strength, set_occlusion_strength, occlusion_strength, f32, "Occlusion strength.");
    settings_scalar!(ior, set_ior, ior, f32, "Index of refraction.");
    settings_scalar!(specular_factor, set_specular_factor, specular_factor, f32, "Specular strength.");
    settings_scalar!(alpha_cutoff, set_alpha_cutoff, alpha_cutoff, f32, "Alpha-mask threshold.");
    settings_flag!(double_sided, set_double_sided, double_sided, "Whether the surface renders from both sides.");
    settings_flag!(output_encoded_to_srgb, set_output_encoded_to_srgb, output_encoded_to_srgb, "Whether output is sRGB-encoded.");

    /// How the material's alpha is interpreted.
    pub fn alpha_mode(&self) -> Result<AlphaMode> {
        AlphaMode::from_native(self.inner.alpha_mode).ok_or(CnaError::UnsupportedRuntime(
            "CNA named an alpha mode this build does not know",
        ))
    }

    /// Sets how the material's alpha is interpreted.
    pub fn set_alpha_mode(&mut self, value: AlphaMode) -> &mut Self {
        self.inner.alpha_mode = value.to_native();
        self
    }

    /// The emissive factor.
    #[must_use]
    pub const fn emissive_factor(&self) -> Vector3 {
        Vector3 {
            X: self.inner.emissive_factor.x,
            Y: self.inner.emissive_factor.y,
            Z: self.inner.emissive_factor.z,
        }
    }

    /// Sets the emissive factor.
    pub fn set_emissive_factor(&mut self, value: Vector3) -> &mut Self {
        self.inner.emissive_factor = sys::CNA_Vector3 {
            x: value.X,
            y: value.Y,
            z: value.Z,
        };
        self
    }

    /// Which UV set a slot samples.
    #[must_use]
    pub const fn texture_coordinate_set(&self, slot: TextureSlot) -> i32 {
        self.inner.texture_coordinate_sets[slot.index()]
    }

    /// Sets which UV set a slot samples.
    pub fn set_texture_coordinate_set(&mut self, slot: TextureSlot, value: i32) -> &mut Self {
        self.inner.texture_coordinate_sets[slot.index()] = value;
        self
    }

    /// One slot's coordinate transform.
    #[must_use]
    pub const fn texture_transform(&self, slot: TextureSlot) -> TextureTransform {
        TextureTransform::from_native(self.inner.texture_transforms[slot.index()])
    }

    /// Sets one slot's coordinate transform.
    pub fn set_texture_transform(
        &mut self,
        slot: TextureSlot,
        value: TextureTransform,
    ) -> &mut Self {
        self.inner.texture_transforms[slot.index()] = value.to_native();
        self
    }

    /// Applies the device state this material implies -- blending, depth write
    /// and culling.
    ///
    /// Separate from applying the material to an effect, because it changes the
    /// *device*, not the effect, and doing both silently would be two
    /// unrelated side effects under one name.
    pub fn apply_state(&self, device: &GraphicsDevice) -> Result<()> {
        let native = device.state_native();
        // SAFETY: the material is a live local CNA reads during the call and
        // the device handle is live.
        native.check(unsafe {
            (native.runtime.pbr_material_apply_state)(&self.inner, device.handle()?)
        })
    }
}

impl PbrEffect {
    pub(crate) const fn native_handle(&self) -> sys::CNA_EffectHandle {
        self.handle
    }

    /// Applies a complete material, every field of which crosses.
    pub fn apply_full(&self, material: &PbrMaterialFull) -> Result<()> {
        // SAFETY: the material is a live local CNA reads during the call.
        self.native.check(unsafe {
            (self.native.runtime.pbr_effect_apply_material)(self.handle, &material.inner)
        })
    }

    /// Reads the complete material this effect currently carries.
    pub fn extract_full(&self) -> Result<PbrMaterialFull> {
        let mut inner = sys::CNA_PbrMaterialEXT {
            struct_size: core::mem::size_of::<sys::CNA_PbrMaterialEXT>() as u32,
            struct_version: 1,
            ..sys::CNA_PbrMaterialEXT::default()
        };
        // SAFETY: the structure is a caller-owned versioned output.
        self.native.check(unsafe {
            (self.native.runtime.pbr_effect_extract_material)(self.handle, &mut inner)
        })?;
        Ok(PbrMaterialFull { inner })
    }
}

/// glTF's optional material extensions, as one owned object.
///
/// Clearcoat, sheen, transmission, volume attenuation, iridescence and
/// subsurface scattering: the `KHR_materials_*` state a physically based
/// material may carry beyond the base metallic-roughness model. XNA has no
/// counterpart for any of it.
///
/// An owned handle rather than a value, because CNA models it as one -- it is
/// the largest single engine-layer family and upstream keeps it behind a
/// handle so it can grow without moving anything.
///
/// The nine texture slots are `RETAINED_DEPENDENCY`. CNA keeps a raw
/// `Texture2D*` in each and retains nothing, so the setters *take* the texture
/// and this value holds it for exactly as long as CNA points at it. The
/// getters hand back a lifetime-bound [`BorrowedRenderTarget`], because the
/// handle CNA publishes there is a fresh one that has to be released.
pub struct PbrMaterialExtensions {
    native: Arc<Native>,
    handle: sys::CNA_PbrMaterialExtensionsHandle,
    textures: ExtensionTextures,
}

/// The nine texture slots, held as Rust resources.
///
/// CNA stores a raw `Texture2D*` in each slot and retains nothing, so these are
/// what keep the textures alive for exactly as long as CNA points at them.
#[derive(Default)]
struct ExtensionTextures {
    clearcoat: Option<Texture2D>,
    clearcoat_roughness: Option<Texture2D>,
    clearcoat_normal: Option<Texture2D>,
    sheen_color: Option<Texture2D>,
    sheen_roughness: Option<Texture2D>,
    transmission: Option<Texture2D>,
    thickness: Option<Texture2D>,
    iridescence: Option<Texture2D>,
    iridescence_thickness: Option<Texture2D>,
}

macro_rules! extension_scalar {
    ($get:ident, $set:ident, $native_get:ident, $native_set:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $get(&self) -> Result<f32> {
            let mut value = 0.0_f32;
            // SAFETY: the handle is owned and the output is a live local.
            self.native
                .check(unsafe { (self.native.runtime.$native_get)(self.handle, &mut value) })?;
            Ok(value)
        }

        #[doc = $doc]
        pub fn $set(&self, value: f32) -> Result<()> {
            // SAFETY: the handle is owned and the value is by value.
            self.native
                .check(unsafe { (self.native.runtime.$native_set)(self.handle, value) })
        }
    };
}

macro_rules! extension_color {
    ($get:ident, $set:ident, $native_get:ident, $native_set:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $get(&self) -> Result<Vector3> {
            let mut value = sys::CNA_Vector3::default();
            // SAFETY: the handle is owned and the output is a live local.
            self.native
                .check(unsafe { (self.native.runtime.$native_get)(self.handle, &mut value) })?;
            Ok(Vector3 {
                X: value.x,
                Y: value.y,
                Z: value.z,
            })
        }

        #[doc = $doc]
        pub fn $set(&self, value: Vector3) -> Result<()> {
            let native_value = sys::CNA_Vector3 {
                x: value.X,
                y: value.Y,
                z: value.Z,
            };
            // SAFETY: the vector is a live local CNA reads during the call.
            self.native
                .check(unsafe { (self.native.runtime.$native_set)(self.handle, &native_value) })
        }
    };
}

macro_rules! extension_flag {
    ($name:ident, $native:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Result<bool> {
            let mut value = sys::CNA_FALSE;
            // SAFETY: the handle is owned and the output is a live local.
            self.native
                .check(unsafe { (self.native.runtime.$native)(self.handle, &mut value) })?;
            Ok(value != sys::CNA_FALSE)
        }
    };
}

impl PbrMaterialExtensions {
    /// A new extension set, at CNA's neutral defaults.
    pub fn new() -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local receiving a newly owned handle.
        native.check(unsafe { (native.runtime.pbr_ext_create)(&mut handle) })?;
        Ok(Self {
            native,
            handle,
            textures: ExtensionTextures::default(),
        })
    }

    /// Copies every value from `source` into this set.
    pub fn copy_from(&self, source: &Self) -> Result<()> {
        // SAFETY: both handles are owned and live for the call.
        self.native
            .check(unsafe { (self.native.runtime.pbr_ext_copy_from)(self.handle, source.handle) })
    }

    /// Whether CNA considers these the same extensions as `other`.
    pub fn same_extensions(&self, other: &Self) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: both handles are owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.pbr_ext_equals)(self.handle, other.handle, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// CNA's own hash of this extension set.
    pub fn hash_code(&self) -> Result<u64> {
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.runtime.pbr_ext_get_hash_code)(self.handle, &mut value) })?;
        Ok(value)
    }

    extension_flag!(
        is_neutral, pbr_ext_is_neutral,
        "Whether every extension is at its neutral value, so the material behaves as though it carried none."
    );
    extension_flag!(is_sheen_enabled, pbr_ext_is_sheen_enabled, "Whether sheen contributes.");
    extension_flag!(is_transmission_enabled, pbr_ext_is_transmission_enabled, "Whether transmission contributes.");
    extension_flag!(is_iridescence_enabled, pbr_ext_is_iridescence_enabled, "Whether iridescence contributes.");
    extension_flag!(is_subsurface_enabled, pbr_ext_is_subsurface_enabled, "Whether subsurface scattering contributes.");

    extension_scalar!(clearcoat_factor, set_clearcoat_factor, pbr_ext_get_clearcoat_factor, pbr_ext_set_clearcoat_factor, "`KHR_materials_clearcoat.clearcoatFactor`.");
    extension_scalar!(clearcoat_roughness, set_clearcoat_roughness, pbr_ext_get_clearcoat_roughness, pbr_ext_set_clearcoat_roughness, "`KHR_materials_clearcoat.clearcoatRoughnessFactor`.");
    extension_scalar!(clearcoat_normal_scale, set_clearcoat_normal_scale, pbr_ext_get_clearcoat_normal_scale, pbr_ext_set_clearcoat_normal_scale, "The clearcoat normal map's intensity.");
    extension_scalar!(sheen_roughness, set_sheen_roughness, pbr_ext_get_sheen_roughness, pbr_ext_set_sheen_roughness, "`KHR_materials_sheen.sheenRoughnessFactor`.");
    extension_scalar!(transmission_factor, set_transmission_factor, pbr_ext_get_transmission_factor, pbr_ext_set_transmission_factor, "`KHR_materials_transmission.transmissionFactor`.");
    extension_scalar!(thickness_factor, set_thickness_factor, pbr_ext_get_thickness_factor, pbr_ext_set_thickness_factor, "`KHR_materials_volume.thicknessFactor`.");
    extension_scalar!(attenuation_distance, set_attenuation_distance, pbr_ext_get_attenuation_distance, pbr_ext_set_attenuation_distance, "`KHR_materials_volume.attenuationDistance`.");
    extension_scalar!(iridescence_factor, set_iridescence_factor, pbr_ext_get_iridescence_factor, pbr_ext_set_iridescence_factor, "`KHR_materials_iridescence.iridescenceFactor`.");
    extension_scalar!(iridescence_ior, set_iridescence_ior, pbr_ext_get_iridescence_ior, pbr_ext_set_iridescence_ior, "`KHR_materials_iridescence.iridescenceIor`.");
    extension_scalar!(iridescence_thickness_minimum, set_iridescence_thickness_minimum, pbr_ext_get_iridescence_thickness_minimum, pbr_ext_set_iridescence_thickness_minimum, "The thin-film thickness range's lower bound.");
    extension_scalar!(iridescence_thickness_maximum, set_iridescence_thickness_maximum, pbr_ext_get_iridescence_thickness_maximum, pbr_ext_set_iridescence_thickness_maximum, "The thin-film thickness range's upper bound.");
    extension_scalar!(subsurface_wrap, set_subsurface_wrap, pbr_ext_get_subsurface_wrap, pbr_ext_set_subsurface_wrap, "How far light wraps around a subsurface-scattering surface.");

    extension_color!(sheen_color_factor, set_sheen_color_factor, pbr_ext_get_sheen_color_factor, pbr_ext_set_sheen_color_factor, "`KHR_materials_sheen.sheenColorFactor`.");
    extension_color!(attenuation_color, set_attenuation_color, pbr_ext_get_attenuation_color, pbr_ext_set_attenuation_color, "`KHR_materials_volume.attenuationColor`.");
    extension_color!(subsurface_color, set_subsurface_color, pbr_ext_get_subsurface_color, pbr_ext_set_subsurface_color, "The colour subsurface scattering tints light with.");
}

impl Drop for PbrMaterialExtensions {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        let _ = unsafe { (self.native.runtime.pbr_ext_destroy)(self.handle) };
    }
}

impl core::fmt::Debug for PbrMaterialExtensions {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PbrMaterialExtensions")
            .field("handle", &self.handle)
            .finish_non_exhaustive()
    }
}

/// CNA's size-then-copy text protocol.
fn copy_native_text(
    native: &Arc<Native>,
    mut route: impl FnMut(*mut core::ffi::c_char, u64, *mut u64) -> sys::CNA_Result,
) -> Result<String> {
    let mut required = 0_u64;
    let probe = route(core::ptr::null_mut(), 0, &mut required);
    if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
        native.check(probe)?;
    }
    let capacity =
        usize::try_from(required).map_err(|_| CnaError::InvalidInput("CNA text is too large"))?;
    if capacity == 0 {
        return Ok(String::new());
    }
    let mut buffer = vec![0_u8; capacity];
    let mut written = 0_u64;
    native.check(route(
        buffer.as_mut_ptr().cast::<core::ffi::c_char>(),
        required,
        &mut written,
    ))?;
    let written =
        usize::try_from(written).map_err(|_| CnaError::InvalidInput("CNA text is too large"))?;
    buffer.truncate(written.min(capacity));
    while buffer.last() == Some(&0) {
        buffer.pop();
    }
    String::from_utf8(buffer).map_err(|_| CnaError::InvalidInput("CNA text is not valid UTF-8"))
}

macro_rules! extension_texture {
    ($field:ident, $get:ident, $set:ident, $has:ident, $native_get:ident, $native_set:ident, $doc:literal) => {
        #[doc = $doc]
        ///
        /// The view borrows this value: CNA publishes a *fresh* handle for the
        /// slot, which has to be released, so it is a lifetime-bound view
        /// rather than a plain texture.
        pub fn $get(&self) -> Result<Option<BorrowedRenderTarget<'_>>> {
            let mut value = sys::CNA_INVALID_HANDLE;
            // SAFETY: the handle is owned and the output is a live local.
            self.native
                .check(unsafe { (self.native.engine.$native_get)(self.handle, &mut value) })?;
            if value == sys::CNA_INVALID_HANDLE {
                return Ok(None);
            }
            let Some(device) = self.textures.$field.as_ref().and_then(Texture2D::GraphicsDevice)
            else {
                // SAFETY: the handle is the view CNA just published; releasing
                // it here rather than leaking it is the only correct choice
                // when there is no device to wrap it with.
                let _ = unsafe { (self.native.render_target_destroy)(value) };
                return Err(CnaError::InvalidInput(
                    "the texture slot names a texture this value does not hold",
                ));
            };
            BorrowedRenderTarget::new(&self.native, device, value).map(Some)
        }

        #[doc = $doc]
        ///
        /// **Takes** the texture: CNA keeps a raw pointer and retains nothing,
        /// so this value holds it for exactly as long as CNA points at it, and
        /// `None` clears the slot and releases the previous one.
        pub fn $set(&mut self, texture: Option<Texture2D>) -> Result<()> {
            let handle = match texture.as_ref() {
                Some(texture) => texture.handle()?,
                None => sys::CNA_INVALID_HANDLE,
            };
            // SAFETY: the extensions handle is owned and the texture handle is
            // live for the call, kept alive afterwards by the value this
            // stores.
            self.native
                .check(unsafe { (self.native.engine.$native_set)(self.handle, handle) })?;
            self.textures.$field = texture;
            Ok(())
        }

        #[doc = $doc]
        ///
        /// Whether the slot is filled, without publishing a view to ask.
        #[must_use]
        pub const fn $has(&self) -> bool {
            self.textures.$field.is_some()
        }
    };
}

impl PbrMaterialExtensions {
    extension_texture!(
        clearcoat,
        clearcoat_texture,
        set_clearcoat_texture,
        has_clearcoat_texture,
        pbr_material_extensions_get_clearcoat_texture,
        pbr_material_extensions_set_clearcoat_texture,
        "The clearcoat strength texture."
    );
    extension_texture!(
        clearcoat_roughness,
        clearcoat_roughness_texture,
        set_clearcoat_roughness_texture,
        has_clearcoat_roughness_texture,
        pbr_material_extensions_get_clearcoat_roughness_texture,
        pbr_material_extensions_set_clearcoat_roughness_texture,
        "The clearcoat roughness texture."
    );
    extension_texture!(
        clearcoat_normal,
        clearcoat_normal_texture,
        set_clearcoat_normal_texture,
        has_clearcoat_normal_texture,
        pbr_material_extensions_get_clearcoat_normal_texture,
        pbr_material_extensions_set_clearcoat_normal_texture,
        "The clearcoat normal map, which is separate from the base normal map."
    );
    extension_texture!(
        sheen_color,
        sheen_color_texture,
        set_sheen_color_texture,
        has_sheen_color_texture,
        pbr_material_extensions_get_sheen_color_texture,
        pbr_material_extensions_set_sheen_color_texture,
        "The sheen colour texture."
    );
    extension_texture!(
        sheen_roughness,
        sheen_roughness_texture,
        set_sheen_roughness_texture,
        has_sheen_roughness_texture,
        pbr_material_extensions_get_sheen_roughness_texture,
        pbr_material_extensions_set_sheen_roughness_texture,
        "The sheen roughness texture."
    );
    extension_texture!(
        transmission,
        transmission_texture,
        set_transmission_texture,
        has_transmission_texture,
        pbr_material_extensions_get_transmission_texture,
        pbr_material_extensions_set_transmission_texture,
        "The transmission factor texture."
    );
    extension_texture!(
        thickness,
        thickness_texture,
        set_thickness_texture,
        has_thickness_texture,
        pbr_material_extensions_get_thickness_texture,
        pbr_material_extensions_set_thickness_texture,
        "The volume thickness texture."
    );
    extension_texture!(
        iridescence,
        iridescence_texture,
        set_iridescence_texture,
        has_iridescence_texture,
        pbr_material_extensions_get_iridescence_texture,
        pbr_material_extensions_set_iridescence_texture,
        "The iridescence strength texture."
    );
    extension_texture!(
        iridescence_thickness,
        iridescence_thickness_texture,
        set_iridescence_thickness_texture,
        has_iridescence_thickness_texture,
        pbr_material_extensions_get_iridescence_thickness_texture,
        pbr_material_extensions_set_iridescence_thickness_texture,
        "The iridescence film thickness texture."
    );

    /// CNA's own rendering of this extension set as text.
    pub fn to_native_string(&self) -> Result<String> {
        let native = Arc::clone(&self.native);
        let handle = self.handle;
        copy_native_text(&native, |destination, capacity, out_bytes| {
            // SAFETY: the handle is owned and this is CNA's size-then-copy
            // protocol, driven by `copy_native_text`.
            unsafe {
                (native.engine.pbr_material_extensions_copy_to_string)(
                    handle,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
    }
}

impl PbrMaterialFull {
    /// Whether CNA considers this the same material as `other`.
    ///
    /// Its own comparison, not a field-by-field Rust one: the structure carries
    /// texture handle slots and reserved padding, and only upstream knows which
    /// of them take part.
    pub fn same_material(&self, other: &Self) -> Result<bool> {
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: both structures are live locals CNA reads during the call.
        native.check(unsafe {
            (native.engine.pbr_material_ext_equals)(&self.inner, &other.inner, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// CNA's own hash of this material.
    pub fn hash_code(&self) -> Result<u64> {
        let native = Native::process()?;
        let mut value = 0_u64;
        // SAFETY: the structure is a live local CNA reads during the call.
        native.check(unsafe {
            (native.engine.pbr_material_ext_get_hash_code)(&self.inner, &mut value)
        })?;
        Ok(value)
    }

    /// CNA's own rendering of this material as text.
    pub fn to_native_string(&self) -> Result<String> {
        let native = Native::process()?;
        let inner = self.inner;
        copy_native_text(&native, |destination, capacity, out_bytes| {
            // SAFETY: the structure is a live local and this is CNA's
            // size-then-copy protocol, driven by `copy_native_text`.
            unsafe {
                (native.engine.pbr_material_ext_copy_to_string)(
                    &inner,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
    }
}

/// The thin-film interference a `KHR_materials_iridescence` surface shows.
///
/// Two pure functions and no state: the value a shader computes per pixel, and
/// the GLSL that computes it, so a hand-written shader and this crate cannot
/// drift apart about what iridescence means.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct ThinFilmIridescence;

impl ThinFilmIridescence {
    /// The interference colour for one viewing angle and film thickness.
    ///
    /// `cos_theta` is the cosine of the angle between the view and the normal,
    /// `thickness_nm` the film thickness in nanometres, and `base_f0` the
    /// surface's normal-incidence reflectance underneath the film.
    pub fn evaluate(
        outside_ior: f32,
        film_ior: f32,
        cos_theta: f32,
        thickness_nm: f32,
        base_f0: Vector3,
    ) -> Result<Vector3> {
        let native = Native::process()?;
        let base = sys::CNA_Vector3 {
            x: base_f0.X,
            y: base_f0.Y,
            z: base_f0.Z,
        };
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the reflectance is borrowed for the call and the output is a
        // live local.
        native.check(unsafe {
            (native.engine.thin_film_iridescence_evaluate)(
                outside_ior,
                film_ior,
                cos_theta,
                thickness_nm,
                &base,
                &mut value,
            )
        })?;
        Ok(Vector3 {
            X: value.x,
            Y: value.y,
            Z: value.z,
        })
    }

    /// The GLSL that evaluates the same interference.
    pub fn glsl() -> Result<String> {
        let native = Native::process()?;
        let api = Arc::clone(&native);
        copy_native_text(&native, |destination, capacity, out_bytes| {
            // SAFETY: CNA's size-then-copy protocol, driven by
            // `copy_native_text`.
            unsafe {
                (api.engine.thin_film_iridescence_copy_glsl)(destination, capacity, out_bytes)
            }
        })
    }
}

/// A physically based effect that also skins its vertices.
///
/// `OWNED`. The same material model as [`PbrEffect`], plus the bone palette an
/// animated mesh needs -- and CNA refuses the material routes when the handle
/// is the wrong kind of effect, which is why the two are separate types here
/// rather than one with a flag.
pub struct SkinnedPbrEffect {
    native: Arc<Native>,
    handle: sys::CNA_EffectHandle,
    device: GraphicsDevice,
}

impl SkinnedPbrEffect {
    /// Creates the effect on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live for the call and the output is a
        // live local.
        native.check(unsafe {
            (native.runtime.skinned_pbr_effect_create)(device.handle()?, &mut handle)
        })?;
        Ok(Self {
            native: Arc::clone(native),
            handle,
            device: device.clone(),
        })
    }

    /// The device this effect belongs to.
    #[must_use]
    pub const fn graphics_device(&self) -> &GraphicsDevice {
        &self.device
    }

    pub(crate) const fn native_handle(&self) -> sys::CNA_EffectHandle {
        self.handle
    }

    /// How many bone weights each vertex carries.
    pub fn weights_per_vertex(&self) -> Result<i32> {
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.skinned_pbr_effect_get_weights_per_vertex)(self.handle, &mut value)
        })?;
        Ok(value)
    }

    /// Sets it.
    pub fn set_weights_per_vertex(&self, value: i32) -> Result<()> {
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self.native.runtime.skinned_pbr_effect_set_weights_per_vertex)(self.handle, value)
        })
    }

    /// Uploads the bone palette.
    pub fn set_bone_transforms(&self, transforms: &[Matrix]) -> Result<()> {
        let native_transforms: Vec<sys::CNA_Matrix> = transforms
            .iter()
            .copied()
            .map(crate::extensions::engine::matrix_to_native)
            .collect();
        // SAFETY: the handle is owned and the array is borrowed for the call
        // with its own length.
        self.native.check(unsafe {
            (self.native.runtime.skinned_pbr_effect_set_bone_transforms)(
                self.handle,
                native_transforms.as_ptr(),
                native_transforms.len() as u64,
            )
        })
    }

    /// Reads the bone palette back.
    pub fn bone_transforms(&self, requested: usize) -> Result<Vec<Matrix>> {
        let mut buffer = vec![sys::CNA_Matrix::default(); requested];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `requested`
        // writable matrices, which is the capacity passed alongside it.
        self.native.check(unsafe {
            (self.native.runtime.skinned_pbr_effect_copy_bone_transforms)(
                self.handle,
                requested as u64,
                buffer.as_mut_ptr(),
                buffer.len() as u64,
                &mut count,
            )
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported more bones than fit in memory"))?;
        Ok(buffer
            .into_iter()
            .take(count.min(requested))
            .map(crate::extensions::engine::matrix_from_native)
            .collect())
    }

    /// Applies a complete material, every field of which crosses.
    pub fn apply_full(&self, material: &PbrMaterialFull) -> Result<()> {
        // SAFETY: the handle is owned and the material is a live local CNA
        // reads during the call.
        self.native.check(unsafe {
            (self.native.engine.skinned_pbr_effect_apply_material)(self.handle, &material.inner)
        })
    }

    /// Reads the complete material this effect currently carries.
    pub fn extract_full(&self) -> Result<PbrMaterialFull> {
        let mut inner = sys::CNA_PbrMaterialEXT {
            struct_size: core::mem::size_of::<sys::CNA_PbrMaterialEXT>() as u32,
            struct_version: 1,
            ..sys::CNA_PbrMaterialEXT::default()
        };
        // SAFETY: the handle is owned and the structure is a caller-owned
        // versioned output.
        self.native.check(unsafe {
            (self.native.engine.skinned_pbr_effect_extract_material)(self.handle, &mut inner)
        })?;
        Ok(PbrMaterialFull { inner })
    }
}

impl Drop for SkinnedPbrEffect {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        // CNA counts it against the parent game's owned children and refuses to
        // destroy a game while one is outstanding, so leaving it to the process
        // would abort at shutdown rather than leak quietly.
        let _ = unsafe { (self.native.effect_destroy)(self.handle) };
    }
}

/// A glTF material's base metallic-roughness values, as the importer read them.
///
/// A staging value: [`GltfMaterialBridge::build_material`] turns one of these
/// plus its textures into a [`PbrMaterialFull`], so an importer describes what
/// the file said and CNA decides what that means for the renderer.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct GltfMaterialSource {
    /// `pbrMetallicRoughness.baseColorFactor`.
    pub base_color_factor: Vector4,
    /// `pbrMetallicRoughness.metallicFactor`.
    pub metallic_factor: f32,
    /// `pbrMetallicRoughness.roughnessFactor`.
    pub roughness_factor: f32,
    /// `emissiveFactor`.
    pub emissive_factor: Vector3,
    /// `normalTexture.scale`.
    pub normal_scale: f32,
    /// `occlusionTexture.strength`.
    pub occlusion_strength: f32,
    /// `KHR_materials_ior.ior`.
    pub ior: f32,
    /// `KHR_materials_specular.specularFactor`.
    pub specular_factor: f32,
    /// `KHR_materials_specular.specularColorFactor`.
    pub specular_color_factor: Vector3,
    /// `alphaMode`.
    pub alpha_mode: AlphaMode,
    /// `alphaCutoff`.
    pub alpha_cutoff: f32,
    /// `doubleSided`.
    pub double_sided: bool,
    /// `KHR_texture_transform` texture-coordinate set per slot.
    pub texture_coordinate_sets: [i32; TEXTURE_SLOT_COUNT],
    /// `KHR_texture_transform` transform per slot.
    pub texture_transforms: [TextureTransform; TEXTURE_SLOT_COUNT],
}

impl GltfMaterialSource {
    /// CNA's own defaults, asked of the library rather than restated here.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_GltfMaterialSourceEXT::default();
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.engine.gltf_material_source_ext_init)(&mut value) })?;
        Self::from_native(&value)
    }

    fn from_native(value: &sys::CNA_GltfMaterialSourceEXT) -> Result<Self> {
        let mut transforms =
            [TextureTransform::from_native(sys::CNA_TextureTransformEXT::default());
                TEXTURE_SLOT_COUNT];
        for (slot, transform) in transforms.iter_mut().enumerate() {
            *transform = TextureTransform::from_native(value.texture_transforms_ext[slot]);
        }
        Ok(Self {
            base_color_factor: Vector4 {
                X: value.base_color_factor.x,
                Y: value.base_color_factor.y,
                Z: value.base_color_factor.z,
                W: value.base_color_factor.w,
            },
            metallic_factor: value.metallic_factor,
            roughness_factor: value.roughness_factor,
            emissive_factor: Vector3 {
                X: value.emissive_factor.x,
                Y: value.emissive_factor.y,
                Z: value.emissive_factor.z,
            },
            normal_scale: value.normal_scale,
            occlusion_strength: value.occlusion_strength,
            ior: value.ior_ext,
            specular_factor: value.specular_factor_ext,
            specular_color_factor: Vector3 {
                X: value.specular_color_factor_ext.x,
                Y: value.specular_color_factor_ext.y,
                Z: value.specular_color_factor_ext.z,
            },
            alpha_mode: AlphaMode::from_native(value.alpha_mode)
                .ok_or(CnaError::InvalidInput("native alpha mode is unknown"))?,
            alpha_cutoff: value.alpha_cutoff,
            double_sided: value.double_sided != sys::CNA_FALSE,
            texture_coordinate_sets: value.texture_coordinate_sets_ext,
            texture_transforms: transforms,
        })
    }

    fn to_native(self) -> sys::CNA_GltfMaterialSourceEXT {
        let mut transforms = [sys::CNA_TextureTransformEXT::default(); TEXTURE_SLOT_COUNT];
        for (slot, transform) in transforms.iter_mut().enumerate() {
            *transform = self.texture_transforms[slot].to_native();
        }
        sys::CNA_GltfMaterialSourceEXT {
            struct_size: core::mem::size_of::<sys::CNA_GltfMaterialSourceEXT>() as u32,
            struct_version: 1,
            base_color_factor: sys::CNA_Vector4 {
                x: self.base_color_factor.X,
                y: self.base_color_factor.Y,
                z: self.base_color_factor.Z,
                w: self.base_color_factor.W,
            },
            metallic_factor: self.metallic_factor,
            roughness_factor: self.roughness_factor,
            emissive_factor: sys::CNA_Vector3 {
                x: self.emissive_factor.X,
                y: self.emissive_factor.Y,
                z: self.emissive_factor.Z,
            },
            normal_scale: self.normal_scale,
            occlusion_strength: self.occlusion_strength,
            ior_ext: self.ior,
            specular_factor_ext: self.specular_factor,
            specular_color_factor_ext: sys::CNA_Vector3 {
                x: self.specular_color_factor.X,
                y: self.specular_color_factor.Y,
                z: self.specular_color_factor.Z,
            },
            alpha_mode: self.alpha_mode.to_native(),
            alpha_cutoff: self.alpha_cutoff,
            double_sided: u8::from(self.double_sided),
            reserved: [0; 3],
            texture_coordinate_sets_ext: self.texture_coordinate_sets,
            texture_transforms_ext: transforms,
        }
    }
}

/// The `KHR_materials_*` factors a glTF file carried, as the importer read them.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct GltfMaterialExtensionSource {
    /// `KHR_materials_clearcoat.clearcoatFactor`.
    pub clearcoat_factor: f32,
    /// `KHR_materials_clearcoat.clearcoatRoughnessFactor`.
    pub clearcoat_roughness_factor: f32,
    /// `KHR_materials_sheen.sheenColorFactor`.
    pub sheen_color_factor: Vector3,
    /// `KHR_materials_sheen.sheenRoughnessFactor`.
    pub sheen_roughness_factor: f32,
    /// `KHR_materials_transmission.transmissionFactor`.
    pub transmission_factor: f32,
    /// `KHR_materials_volume.thicknessFactor`.
    pub thickness_factor: f32,
    /// `KHR_materials_volume.attenuationDistance`.
    pub attenuation_distance: f32,
    /// `KHR_materials_volume.attenuationColor`.
    pub attenuation_color: Vector3,
    /// `KHR_materials_iridescence.iridescenceFactor`.
    pub iridescence_factor: f32,
    /// `KHR_materials_iridescence.iridescenceIor`.
    pub iridescence_ior: f32,
    /// `KHR_materials_iridescence.iridescenceThicknessMinimum`.
    pub iridescence_thickness_minimum: f32,
    /// `KHR_materials_iridescence.iridescenceThicknessMaximum`.
    pub iridescence_thickness_maximum: f32,
}

impl GltfMaterialExtensionSource {
    /// CNA's own defaults, asked of the library rather than restated here.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_GltfMaterialExtensionSourceEXT::default();
        // SAFETY: the structure is a caller-owned versioned output.
        native
            .check(unsafe { (native.engine.gltf_material_extension_source_ext_init)(&mut value) })?;
        Ok(Self::from_native(&value))
    }

    fn from_native(value: &sys::CNA_GltfMaterialExtensionSourceEXT) -> Self {
        let vector = |v: sys::CNA_Vector3| Vector3 {
            X: v.x,
            Y: v.y,
            Z: v.z,
        };
        Self {
            clearcoat_factor: value.clearcoat_factor_ext,
            clearcoat_roughness_factor: value.clearcoat_roughness_factor_ext,
            sheen_color_factor: vector(value.sheen_color_factor_ext),
            sheen_roughness_factor: value.sheen_roughness_factor_ext,
            transmission_factor: value.transmission_factor_ext,
            thickness_factor: value.thickness_factor_ext,
            attenuation_distance: value.attenuation_distance_ext,
            attenuation_color: vector(value.attenuation_color_ext),
            iridescence_factor: value.iridescence_factor_ext,
            iridescence_ior: value.iridescence_ior_ext,
            iridescence_thickness_minimum: value.iridescence_thickness_minimum_ext,
            iridescence_thickness_maximum: value.iridescence_thickness_maximum_ext,
        }
    }

    fn to_native(self) -> sys::CNA_GltfMaterialExtensionSourceEXT {
        let vector = |v: Vector3| sys::CNA_Vector3 {
            x: v.X,
            y: v.Y,
            z: v.Z,
        };
        sys::CNA_GltfMaterialExtensionSourceEXT {
            struct_size: core::mem::size_of::<sys::CNA_GltfMaterialExtensionSourceEXT>() as u32,
            struct_version: 1,
            clearcoat_factor_ext: self.clearcoat_factor,
            clearcoat_roughness_factor_ext: self.clearcoat_roughness_factor,
            sheen_color_factor_ext: vector(self.sheen_color_factor),
            sheen_roughness_factor_ext: self.sheen_roughness_factor,
            transmission_factor_ext: self.transmission_factor,
            thickness_factor_ext: self.thickness_factor,
            attenuation_distance_ext: self.attenuation_distance,
            attenuation_color_ext: vector(self.attenuation_color),
            iridescence_factor_ext: self.iridescence_factor,
            iridescence_ior_ext: self.iridescence_ior,
            iridescence_thickness_minimum_ext: self.iridescence_thickness_minimum,
            iridescence_thickness_maximum_ext: self.iridescence_thickness_maximum,
        }
    }
}

/// Turns what a glTF importer read into what a CNA renderer shades with.
///
/// Two pure functions and no state. The textures are `BORROWED` for the call
/// only: the bridge records the handles in the material it builds, and the
/// caller keeps the textures alive afterwards.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct GltfMaterialBridge;

impl GltfMaterialBridge {
    /// Builds a complete material from a glTF source and its seven textures.
    ///
    /// `textures` is in [`TextureSlot::ALL`] order, and a slot may be `None`.
    pub fn build_material(
        source: GltfMaterialSource,
        textures: &[Option<&Texture2D>; TEXTURE_SLOT_COUNT],
    ) -> Result<PbrMaterialFull> {
        let native = Native::process()?;
        let source = source.to_native();
        let mut slots = [sys::CNA_INVALID_HANDLE; TEXTURE_SLOT_COUNT];
        for (slot, texture) in textures.iter().enumerate() {
            if let Some(texture) = texture {
                slots[slot] = texture.handle()?;
            }
        }
        let mut native_textures = sys::CNA_GltfMaterialTexturesEXT {
            struct_size: core::mem::size_of::<sys::CNA_GltfMaterialTexturesEXT>() as u32,
            struct_version: 1,
            slots,
        };
        // The initializer fills the versioning; the slots are the caller's, so
        // they are written after it rather than before.
        let mut probe = sys::CNA_GltfMaterialTexturesEXT::default();
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.engine.gltf_material_textures_ext_init)(&mut probe) })?;
        native_textures.struct_size = probe.struct_size;
        native_textures.struct_version = probe.struct_version;
        let mut inner = sys::CNA_PbrMaterialEXT {
            struct_size: core::mem::size_of::<sys::CNA_PbrMaterialEXT>() as u32,
            struct_version: 1,
            ..sys::CNA_PbrMaterialEXT::default()
        };
        // SAFETY: both inputs are live locals CNA reads during the call and the
        // output is a caller-owned versioned structure.
        native.check(unsafe {
            (native.engine.gltf_material_bridge_build_material)(
                &source,
                &native_textures,
                &mut inner,
            )
        })?;
        Ok(PbrMaterialFull { inner })
    }

    /// Fills `destination` from a glTF extension source and its nine textures.
    ///
    /// Writes into an extension set the caller already owns, which is CNA's own
    /// shape for the route: the extensions are a handle, and a handle cannot be
    /// returned by value.
    pub fn build_extensions(
        source: GltfMaterialExtensionSource,
        textures: &GltfMaterialExtensionTextures<'_>,
        destination: &PbrMaterialExtensions,
    ) -> Result<()> {
        let native = Native::process()?;
        let source = source.to_native();
        let native_textures = textures.to_native(&native)?;
        // SAFETY: both inputs are live locals CNA reads during the call and the
        // destination handle is owned.
        native.check(unsafe {
            (native.engine.gltf_material_bridge_build_extensions)(
                &source,
                &native_textures,
                destination.handle,
            )
        })
    }
}

/// The nine `KHR_materials_*` textures a glTF file referenced.
///
/// Borrowed for the duration of the build call only.
#[derive(Clone, Copy, Default)]
#[non_exhaustive]
pub struct GltfMaterialExtensionTextures<'a> {
    /// `KHR_materials_clearcoat.clearcoatTexture`.
    pub clearcoat: Option<&'a Texture2D>,
    /// `KHR_materials_clearcoat.clearcoatRoughnessTexture`.
    pub clearcoat_roughness: Option<&'a Texture2D>,
    /// `KHR_materials_clearcoat.clearcoatNormalTexture`.
    pub clearcoat_normal: Option<&'a Texture2D>,
    /// `KHR_materials_sheen.sheenColorTexture`.
    pub sheen_color: Option<&'a Texture2D>,
    /// `KHR_materials_sheen.sheenRoughnessTexture`.
    pub sheen_roughness: Option<&'a Texture2D>,
    /// `KHR_materials_transmission.transmissionTexture`.
    pub transmission: Option<&'a Texture2D>,
    /// `KHR_materials_volume.thicknessTexture`.
    pub thickness: Option<&'a Texture2D>,
    /// `KHR_materials_iridescence.iridescenceTexture`.
    pub iridescence: Option<&'a Texture2D>,
    /// `KHR_materials_iridescence.iridescenceThicknessTexture`.
    pub iridescence_thickness: Option<&'a Texture2D>,
}

impl GltfMaterialExtensionTextures<'_> {
    fn to_native(&self, native: &Arc<Native>) -> Result<sys::CNA_GltfMaterialExtensionTexturesEXT> {
        let mut value = sys::CNA_GltfMaterialExtensionTexturesEXT::default();
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe {
            (native.engine.gltf_material_extension_textures_ext_init)(&mut value)
        })?;
        let handle = |texture: Option<&Texture2D>| -> Result<sys::CNA_Handle> {
            match texture {
                Some(texture) => texture.handle(),
                None => Ok(sys::CNA_INVALID_HANDLE),
            }
        };
        value.clearcoat = handle(self.clearcoat)?;
        value.clearcoat_roughness = handle(self.clearcoat_roughness)?;
        value.clearcoat_normal = handle(self.clearcoat_normal)?;
        value.sheen_color = handle(self.sheen_color)?;
        value.sheen_roughness = handle(self.sheen_roughness)?;
        value.transmission = handle(self.transmission)?;
        value.thickness = handle(self.thickness)?;
        value.iridescence = handle(self.iridescence)?;
        value.iridescence_thickness = handle(self.iridescence_thickness)?;
        Ok(value)
    }
}

impl PbrMaterialExtensions {
    /// Wraps a handle another object owns, for a bounded borrow.
    ///
    /// The view holds its owner alive and releases only itself. Its texture
    /// slots answer "the texture slot names a texture this value does not
    /// hold", because the Rust resources keeping those textures alive belong to
    /// whoever set them, not to the view.
    pub(crate) fn from_borrowed_handle(
        native: &Arc<Native>,
        handle: sys::CNA_PbrMaterialExtensionsHandle,
    ) -> Self {
        Self {
            native: Arc::clone(native),
            handle,
            textures: ExtensionTextures::default(),
        }
    }

    pub(crate) const fn native_handle(&self) -> sys::CNA_PbrMaterialExtensionsHandle {
        self.handle
    }
}

impl core::fmt::Debug for GltfMaterialExtensionTextures<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("GltfMaterialExtensionTextures")
            .field("clearcoat", &self.clearcoat.is_some())
            .field("clearcoat_roughness", &self.clearcoat_roughness.is_some())
            .field("clearcoat_normal", &self.clearcoat_normal.is_some())
            .field("sheen_color", &self.sheen_color.is_some())
            .field("sheen_roughness", &self.sheen_roughness.is_some())
            .field("transmission", &self.transmission.is_some())
            .field("thickness", &self.thickness.is_some())
            .field("iridescence", &self.iridescence.is_some())
            .field(
                "iridescence_thickness",
                &self.iridescence_thickness.is_some(),
            )
            .finish()
    }
}
