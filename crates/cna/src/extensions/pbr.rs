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
use crate::graphics::GraphicsDevice;
use crate::native::Native;
use crate::value::{Color, Vector3};

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
            const fn from_native(value: $native) -> Option<Self> {
                Some(match value {
                    $(sys::$constant => Self::$variant,)+
                    _ => return None,
                })
            }

            const fn to_native(self) -> $native {
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
