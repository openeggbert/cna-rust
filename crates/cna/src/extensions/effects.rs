//! The rest of `effects.h`: what an effect *is*, and the effects CNA adds.
//!
//! [`crate::graphics::effect`] projects XNA's `Effect` -- parameters,
//! techniques, passes, annotations -- and this module completes the header
//! around it: the source an effect was built from, whether the renderer
//! compiled it, CNA's `ColorMatrixEffect`, and the stock `SpriteEffect`.
//!
//! # Two ways to make an effect, two different promises
//!
//! [`crate::extensions::shader_effect::ShaderEffect::new`] hands source to a
//! renderer and succeeds whether or not it compiles; ask `is_valid`
//! afterwards. [`load_effect`] owns the whole load *including* the compile, so
//! a shader the renderer cannot compile is a failed load. Upstream warns
//! against carrying either contract over to the other, and this module keeps
//! them apart.

#![allow(non_snake_case, clippy::missing_errors_doc)]

use std::sync::Arc;

use cna_sys as sys;

use crate::error::Result;
use crate::extensions::content::NativeContentManager;
use crate::graphics::{
    Effect, EffectBase, EffectMaterial, EffectPass, EffectTechnique, GraphicsDevice,
};
use crate::native::Native;
use crate::value::Vector4;

/// What an effect was built from, and whether it works.
impl Effect {
    /// The vertex-shader source, empty when the effect was not built from any.
    pub fn vertex_source(&self) -> Result<String> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let api = Arc::clone(&native);
        crate::native::runtime::read_string(
            |value| native.check(value),
            // SAFETY: owned handle, live outputs; CNA's size-then-copy pair.
            |bytes| unsafe { (api.effect_get_vertex_source_byte_count)(handle, bytes) },
            |destination, capacity, written| unsafe {
                (api.effect_copy_vertex_source)(handle, destination, capacity, written)
            },
        )
    }

    /// The fragment-shader source, empty when the effect was not built from any.
    pub fn fragment_source(&self) -> Result<String> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let api = Arc::clone(&native);
        crate::native::runtime::read_string(
            |value| native.check(value),
            // SAFETY: owned handle, live outputs; CNA's size-then-copy pair.
            |bytes| unsafe { (api.effect_get_fragment_source_byte_count)(handle, bytes) },
            |destination, capacity, written| unsafe {
                (api.effect_copy_fragment_source)(handle, destination, capacity, written)
            },
        )
    }

    /// Whether the renderer compiled this effect.
    ///
    /// Distinct from having a renderer at all: an effect with no renderer is
    /// not compiled and never will be, and the two answers together say which
    /// of those a caller is looking at.
    pub fn is_compiled(&self) -> Result<bool> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe { (native.effect_get_is_compiled_ext)(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Whether a renderer is attached at all.
    pub fn has_renderer(&self) -> Result<bool> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe { (native.effect_has_renderer)(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Whether this is exactly the stock sprite effect.
    ///
    /// `SpriteBatch` substitutes its own effect when a caller passes none, and
    /// this is how a caller tells the substitute from an effect of their own
    /// that happens to draw sprites.
    pub fn is_exact_stock_sprite_effect(&self) -> Result<bool> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe { (native.effect_is_exact_stock_sprite_effect)(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// The identity of the device this effect belongs to.
    ///
    /// A handle value rather than a [`GraphicsDevice`]: the device is not this
    /// effect's to hand out, and the identity is what answers the question a
    /// caller actually has -- whether two effects share a device.
    ///
    /// # Needs a running `Game`
    ///
    /// Upstream resolves this through the effect's *parent game* rather than
    /// through the device it was made with, so an effect built on an
    /// independently constructed [`GraphicsDevice`] -- the shape this crate's
    /// own tests use, and the one that needs no `Game` at all -- is refused
    /// with an invalid-game-handle failure. The header does not say so; it was
    /// measured. Inside a `Game` callback it answers normally.
    pub fn graphics_device_identity(&self) -> Result<u64> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe { (native.effect_get_graphics_device)(handle, &mut value) })?;
        Ok(value)
    }
}

impl EffectPass {
    /// This pass's index within its technique.
    ///
    /// CNA's own addition: XNA's `EffectPass` has a name but no index, and a
    /// renderer that addresses passes positionally needs one.
    pub fn index(&self) -> Result<u32> {
        let handle = self.state.require_handle()?;
        let native = Native::process()?;
        let mut value = 0_u32;
        // SAFETY: the handle is a live view and the output is a live local.
        native.check(unsafe { (native.effect_pass_get_index_ext)(handle, &mut value) })?;
        Ok(value)
    }
}

impl EffectTechnique {
    /// This technique's index within its effect.
    pub fn index(&self) -> Result<u32> {
        let handle = self.state.require_handle()?;
        let native = Native::process()?;
        let mut value = 0_u32;
        // SAFETY: the handle is a live view and the output is a live local.
        native.check(unsafe { (native.effect_technique_get_index_ext)(handle, &mut value) })?;
        Ok(value)
    }

    /// This technique's stable identity.
    ///
    /// What a pass is tagged with, so a pass can say which technique it belongs
    /// to without holding a reference to it.
    pub fn identity(&self) -> Result<u64> {
        let handle = self.state.require_handle()?;
        let native = Native::process()?;
        let mut value = 0_u64;
        // SAFETY: the handle is a live view and the output is a live local.
        native.check(unsafe { (native.effect_technique_get_identity)(handle, &mut value) })?;
        Ok(value)
    }
}

impl EffectMaterial {
    /// How many parameter textures this material is keeping alive.
    ///
    /// A material retains the textures its parameters point at, because the
    /// parameter slot itself is non-owning. This is the count of those.
    pub fn retained_parameter_texture_count(&self) -> Result<u64> {
        let handle = self.effect.handle()?;
        let native = Native::process()?;
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe {
            (native.effect_material_get_retained_parameter_texture_count_ext)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Asks the material to keep one parameter texture alive.
    ///
    /// The texture handle is borrowed for the call and retained by CNA
    /// afterwards, which is what stops a parameter's texture being freed while
    /// the material still points at it.
    pub fn retain_parameter_texture(
        &self,
        texture_type: u32,
        texture: &crate::graphics::Texture2D,
    ) -> Result<()> {
        let handle = self.effect.handle()?;
        let native = Native::process()?;
        // SAFETY: both handles belong to live values.
        native.check(unsafe {
            (native.effect_material_retain_parameter_texture_ext)(
                handle,
                texture_type,
                texture.handle()?,
            )
        })
    }
}

/// XNA's stock sprite effect, as an ordinary [`Effect`].
///
/// `SpriteBatch` makes one of these for a caller who passes no effect. Creating
/// one directly is how a caller draws sprites with the stock shader while still
/// controlling the batch's other state.
pub fn create_sprite_effect(device: &GraphicsDevice) -> Result<Effect> {
    let native = Native::process()?;
    let mut handle = sys::CNA_INVALID_HANDLE;
    // SAFETY: the device handle is borrowed for the call and the output is a
    // live local receiving a new owned handle.
    native.check(unsafe { (native.sprite_effect_create)(device.handle()?, &mut handle) })?;
    Ok(Effect::from_handle(device, handle))
}

/// Loads a compiled effect from content.
///
/// Unlike [`crate::extensions::shader_effect::ShaderEffect::new`], this owns the
/// compile: a shader the renderer cannot compile is a *failed load*, not an
/// effect that answers `is_valid() == false`.
pub fn load_effect(
    content_manager: &NativeContentManager,
    device: &GraphicsDevice,
    asset_name: &str,
) -> Result<Effect> {
    let native = Native::process()?;
    let mut handle = sys::CNA_INVALID_HANDLE;
    // SAFETY: the manager handle is borrowed for the call, the name is borrowed
    // and copied, and the output is a live local.
    native.check(unsafe {
        (native.content_manager_load_effect)(
            content_manager.handle(),
            sys::CNA_StringView {
                data: asset_name.as_ptr().cast::<core::ffi::c_char>(),
                byte_length: asset_name.len() as u64,
            },
            &mut handle,
        )
    })?;
    Ok(Effect::from_handle(device, handle))
}

/// CNA's colour-transform effect: a 4x4 matrix over RGBA, plus an offset.
///
/// Not XNA. It is the effect a post-process colour grade is written with, and
/// the matrix multiplies colour rather than position -- which is why it takes
/// sixteen floats rather than a [`crate::value::Matrix`], so it cannot be
/// passed where a world transform belongs.
pub struct ColorMatrixEffect {
    native: Arc<Native>,
    effect: Effect,
}

impl ColorMatrixEffect {
    /// Creates the effect, set to the identity transform.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is borrowed for the call and the output is
        // a live local receiving a new owned handle.
        native.check(unsafe {
            (native.color_matrix_effect_create)(device.handle()?, &mut handle)
        })?;
        Ok(Self {
            native,
            effect: Effect::from_handle(device, handle),
        })
    }

    /// The effect itself, for everything the strict projection already does.
    #[must_use]
    pub const fn effect(&self) -> &Effect {
        &self.effect
    }

    /// The colour transform, row-major.
    pub fn matrix(&self) -> Result<[f32; 16]> {
        let handle = self.effect.handle()?;
        let mut value = sys::CNA_ColorMatrix4x4::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.color_matrix_effect_get_matrix)(handle, &mut value) })?;
        Ok(value.values)
    }

    /// Replaces the colour transform.
    pub fn set_matrix(&self, values: [f32; 16]) -> Result<()> {
        let handle = self.effect.handle()?;
        // SAFETY: the handle is owned and the matrix is passed by value.
        self.native.check(unsafe {
            (self.native.color_matrix_effect_set_matrix)(
                handle,
                sys::CNA_ColorMatrix4x4 { values },
            )
        })
    }

    /// The constant added after the transform.
    pub fn offset(&self) -> Result<Vector4> {
        let handle = self.effect.handle()?;
        let mut value = sys::CNA_Vector4::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.color_matrix_effect_get_offset)(handle, &mut value) })?;
        Ok(Vector4 {
            X: value.x,
            Y: value.y,
            Z: value.z,
            W: value.w,
        })
    }

    /// Replaces the constant added after the transform.
    pub fn set_offset(&self, value: Vector4) -> Result<()> {
        let handle = self.effect.handle()?;
        // SAFETY: the handle is owned and the vector is by value.
        self.native.check(unsafe {
            (self.native.color_matrix_effect_set_offset)(
                handle,
                sys::CNA_Vector4 {
                    x: value.X,
                    y: value.Y,
                    z: value.Z,
                    w: value.W,
                },
            )
        })
    }

    /// Restores the identity transform and a zero offset.
    pub fn reset(&self) -> Result<()> {
        let handle = self.effect.handle()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.color_matrix_effect_reset)(handle) })
    }

    /// Sets the standard luminance transform.
    ///
    /// CNA's own coefficients rather than a caller's, so a grayscale pass looks
    /// the same everywhere it is used.
    pub fn set_grayscale(&self) -> Result<()> {
        let handle = self.effect.handle()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.color_matrix_effect_set_grayscale)(handle) })
    }
}

/// What a stock effect's textures actually are, as CNA holds them.
///
/// The strict projection's `Texture()` answers the `Arc<Texture2D>` the caller
/// last set, which is what keeps the texture alive on the Rust side. That is
/// the right answer for an effect a Rust caller built, and no answer at all for
/// one that came from [`load_effect`] -- its textures are the asset's, and this
/// side never saw them.
///
/// These report the identity CNA holds. A handle value rather than a texture,
/// for the same reason as everywhere else here: the texture is the effect's,
/// and an owning Rust value would promise a lifetime this side cannot keep.
/// `None` means the slot is empty.
pub trait StockEffectTextures {
    /// The identity of the texture CNA has bound, if any.
    fn native_texture_identity(&self) -> Result<Option<u64>>;
}

fn texture_identity(
    route: unsafe extern "C" fn(sys::CNA_EffectHandle, *mut sys::CNA_Bool, *mut sys::CNA_Handle)
        -> sys::CNA_Result,
    effect: &Effect,
) -> Result<Option<u64>> {
    let handle = effect.handle()?;
    let native = Native::process()?;
    let mut present = sys::CNA_FALSE;
    let mut texture = sys::CNA_INVALID_HANDLE;
    // SAFETY: the handle is owned and both outputs are live locals.
    native.check(unsafe { route(handle, &mut present, &mut texture) })?;
    Ok((present != sys::CNA_FALSE).then_some(texture))
}

macro_rules! stock_texture {
    ($type:ty, $route:ident) => {
        impl StockEffectTextures for $type {
            fn native_texture_identity(&self) -> Result<Option<u64>> {
                let native = Native::process()?;
                texture_identity(native.$route, self.AsEffect())
            }
        }
    };
}

stock_texture!(crate::graphics::AlphaTestEffect, alpha_test_effect_get_texture);
stock_texture!(crate::graphics::BasicEffect, basic_effect_get_texture);

impl crate::graphics::DualTextureEffect {
    /// The identity of the texture CNA has bound to one layer, if any.
    ///
    /// Two layers rather than one, which is why this takes an index where the
    /// other stock effects take nothing: `DualTextureEffect` is the effect that
    /// blends two, and a shared accessor would have had to pick one.
    pub fn native_texture_identity(&self, layer: u32) -> Result<Option<u64>> {
        let handle = self.AsEffect().handle()?;
        let native = Native::process()?;
        let mut present = sys::CNA_FALSE;
        let mut texture = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and both outputs are live locals.
        native.check(unsafe {
            (native.dual_texture_effect_get_texture)(handle, layer, &mut present, &mut texture)
        })?;
        Ok((present != sys::CNA_FALSE).then_some(texture))
    }
}
stock_texture!(crate::graphics::EnvironmentMapEffect, environment_map_effect_get_texture);
stock_texture!(crate::graphics::SkinnedEffect, skinned_effect_get_texture);

impl crate::graphics::EnvironmentMapEffect {
    /// The identity of the cube map CNA has bound, if any.
    ///
    /// The environment map is a second slot beside the base texture, so it
    /// needs its own accessor rather than the shared trait's.
    pub fn native_environment_map_identity(&self) -> Result<Option<u64>> {
        let native = Native::process()?;
        texture_identity(native.environment_map_effect_get_environment_map, self.AsEffect())
    }
}

impl crate::graphics::SkinnedEffect {
    /// Whether the effect reads a per-vertex colour channel.
    ///
    /// XNA's `SkinnedEffect` has this and the strict projection did not,
    /// which is why it is here rather than beside the other stock properties.
    pub fn VertexColorEnabled(&self) -> Result<bool> {
        let handle = self.AsEffect().handle()?;
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe {
            (native.skinned_effect_get_vertex_color_enabled)(handle, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Sets whether the effect reads a per-vertex colour channel.
    pub fn SetVertexColorEnabled(&self, value: bool) -> Result<()> {
        let handle = self.AsEffect().handle()?;
        let native = Native::process()?;
        // SAFETY: the handle is owned and the flag is by value.
        native.check(unsafe {
            (native.skinned_effect_set_vertex_color_enabled)(handle, u8::from(value))
        })
    }
}
