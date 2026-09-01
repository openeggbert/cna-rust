//! CNA's own `ShaderEffect`: shader source in, uniforms set by name.
//!
//! None of this is XNA. XNA compiled `.fx` files offline and a game reached
//! parameters through `EffectParameterCollection`; CNA takes GLSL-family source
//! at runtime and sets uniforms by name, which is a different model and lives
//! here rather than in the strict projection.
//!
//! # Construction succeeds; compilation is a separate question
//!
//! [`ShaderEffect::new`] returning `Ok` means the effect *object* exists. It
//! does not mean the source compiled, and upstream is explicit that it will not
//! normalise this: "Measured across two of CNA's renderers, the same nonsense
//! text is accepted by both, and afterwards one reports it valid while the
//! other reports it invalid." So ask [`ShaderEffect::is_valid`] afterwards, and
//! read [`ShaderEffect::compile_error`] when it says no.
//!
//! The one case settled everywhere is both sources empty, which is refused
//! identically on every renderer.
//!
//! This is deliberately *not* the contract of
//! [`crate::extensions::content::NativeContentManager`]'s effect load, which
//! owns the whole load including the compile and so can report a compile
//! failure as a failure. Two neighbouring routes, opposite promises.

#![allow(clippy::missing_errors_doc)]

use std::sync::Arc;

use cna_sys as sys;

use crate::error::Result;
use crate::graphics::{Effect, GraphicsDevice, Texture2D, Texture3D, TextureCube};
use crate::native::Native;
use crate::value::{Matrix, Vector2, Vector3, Vector4};

/// A runtime-compiled shader effect.
///
/// Wraps an [`Effect`], so everything the strict projection can do with one --
/// techniques, passes, disposal -- works here too. What this type adds is the
/// uniform surface, which has no XNA counterpart.
pub struct ShaderEffect {
    native: Arc<Native>,
    effect: Effect,
}

impl ShaderEffect {
    /// Creates an effect from vertex and fragment source.
    ///
    /// Read the module documentation before treating success as a compile.
    /// Both sources empty is the one refusal every renderer agrees on.
    pub fn new(
        device: &GraphicsDevice,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is borrowed for the call, both sources are
        // borrowed and copied by CNA, and the output is a live local.
        native.check(unsafe {
            (native.shader_effect_create)(
                device.handle()?,
                string_view(vertex_source),
                string_view(fragment_source),
                &mut handle,
            )
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

    /// Whether the renderer considers this effect usable.
    ///
    /// The answer construction does not give. What "valid" means is the
    /// renderer's own judgement, and two of CNA's disagree about the same
    /// nonsense source -- so this reports rather than interprets.
    pub fn is_valid(&self) -> Result<bool> {
        let handle = self.effect.handle()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.shader_effect_is_valid)(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Whether a renderer is attached at all.
    ///
    /// `false` means the uniform setters have nowhere to send anything, which
    /// is a different fact from an effect whose source did not compile.
    pub fn has_renderer(&self) -> Result<bool> {
        let handle = self.effect.handle()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.shader_effect_has_renderer)(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// What the renderer said when it refused the source.
    ///
    /// Empty when there is nothing to report, which includes a renderer that
    /// does not compile at construction at all.
    pub fn compile_error(&self) -> Result<String> {
        let handle = self.effect.handle()?;
        let native = Arc::clone(&self.native);
        let mut required = 0_u64;
        // The size probe is the copy route asked with zero capacity; there is
        // no separate size route for this one.
        // SAFETY: a null destination with zero capacity asks for the size.
        crate::extensions::content::accept_size_probe(&native, unsafe {
            (native.shader_effect_copy_compile_error_ext)(
                handle,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        })?;
        crate::native::runtime::read_string_of_size(
            required,
            |value| native.check(value),
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable bytes.
            |destination, capacity, written| unsafe {
                (native.shader_effect_copy_compile_error_ext)(
                    handle,
                    destination,
                    capacity,
                    written,
                )
            },
        )
    }
}

/// Declares one uniform setter over its native route.
macro_rules! set_uniform {
    ($(#[$meta:meta])* $method:ident, $route:ident, $value:ty, $convert:expr) => {
        $(#[$meta])*
        pub fn $method(&self, name: &str, value: $value) -> Result<()> {
            let handle = self.effect.handle()?;
            #[allow(clippy::redundant_closure_call)]
            let native_value = $convert(value);
            // SAFETY: the handle is owned, the name is borrowed for the call,
            // and the value is passed by value.
            self.native.check(unsafe {
                (self.native.$route)(handle, string_view(name), native_value)
            })
        }
    };
}

/// Setting uniforms by name.
///
/// A name the shader does not declare is the renderer's business, not this
/// crate's: CNA passes it through and the renderer decides whether to ignore it
/// or refuse. Nothing here pretends to know the shader's interface.
impl ShaderEffect {
    set_uniform!(/// Sets a `float` uniform.
        set_float, shader_effect_set_uniform_float, f32, |value| value);
    set_uniform!(/// Sets an `int` uniform.
        set_int32, shader_effect_set_uniform_int32, i32, |value| value);
    set_uniform!(/// Sets a `vec2` uniform.
        set_vector2, shader_effect_set_uniform_vector2, Vector2,
        |value: Vector2| sys::CNA_Vector2 { x: value.X, y: value.Y });
    set_uniform!(/// Sets a `vec3` uniform.
        set_vector3, shader_effect_set_uniform_vector3, Vector3,
        |value: Vector3| sys::CNA_Vector3 { x: value.X, y: value.Y, z: value.Z });
    set_uniform!(/// Sets a `vec4` uniform.
        set_vector4, shader_effect_set_uniform_vector4, Vector4,
        |value: Vector4| sys::CNA_Vector4 { x: value.X, y: value.Y, z: value.Z, w: value.W });
    set_uniform!(/// Sets a `mat4` uniform.
        set_matrix, shader_effect_set_uniform_matrix, Matrix, native_matrix);

    /// Sets a `float[]` uniform.
    pub fn set_float_array(&self, name: &str, values: &[f32]) -> Result<()> {
        let handle = self.effect.handle()?;
        // SAFETY: the handle is owned, and the name and the array are borrowed
        // for the call with the array's own length.
        self.native.check(unsafe {
            (self.native.shader_effect_set_uniform_float_array)(
                handle,
                string_view(name),
                pointer(values),
                values.len() as u64,
            )
        })
    }

    /// Sets a `vec2[]` uniform.
    pub fn set_vector2_array(&self, name: &str, values: &[Vector2]) -> Result<()> {
        let handle = self.effect.handle()?;
        let native: Vec<sys::CNA_Vector2> = values
            .iter()
            .map(|value| sys::CNA_Vector2 {
                x: value.X,
                y: value.Y,
            })
            .collect();
        // SAFETY: the handle is owned and both the name and the converted array
        // outlive the call.
        self.native.check(unsafe {
            (self.native.shader_effect_set_uniform_vector2_array)(
                handle,
                string_view(name),
                if native.is_empty() {
                    core::ptr::null()
                } else {
                    native.as_ptr()
                },
                native.len() as u64,
            )
        })
    }

    /// Sets a `vec3[]` uniform, as three floats per element.
    ///
    /// Takes floats rather than `Vector3` values because the route does: the
    /// count it wants is the *element* count, and packing here would hide which
    /// of the two a caller is passing.
    pub fn set_vector3_array(&self, name: &str, values: &[f32], element_count: i32) -> Result<()> {
        let handle = self.effect.handle()?;
        // SAFETY: the handle is owned, the name and array are borrowed for the
        // call, and the element count is the caller's own claim about the
        // array's shape -- upstream validates it against the array size.
        self.native.check(unsafe {
            (self.native.shader_effect_set_uniform_vec3_array)(
                handle,
                string_view(name),
                pointer(values),
                element_count,
            )
        })
    }

    /// Sets a `mat4[]` uniform, as sixteen floats per element.
    ///
    /// The shape a skinning palette arrives in.
    pub fn set_matrix_array(&self, name: &str, values: &[f32], element_count: i32) -> Result<()> {
        let handle = self.effect.handle()?;
        // SAFETY: as above.
        self.native.check(unsafe {
            (self.native.shader_effect_set_uniform_mat4_array)(
                handle,
                string_view(name),
                pointer(values),
                element_count,
            )
        })
    }

    /// Binds a 2D texture to a sampler slot.
    pub fn set_texture2d(&self, slot: i32, texture: Option<&Texture2D>) -> Result<()> {
        let handle = self.effect.handle()?;
        let texture_handle = match texture {
            Some(texture) => texture.handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        // SAFETY: both handles belong to live values and the slot is by value.
        self.native.check(unsafe {
            (self.native.shader_effect_set_texture2d)(handle, slot, texture_handle)
        })
    }

    /// Binds a volume texture to a sampler slot.
    pub fn set_texture3d(&self, slot: i32, texture: Option<&Texture3D>) -> Result<()> {
        let handle = self.effect.handle()?;
        let texture_handle = match texture {
            Some(texture) => texture.native_handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        // SAFETY: as above.
        self.native.check(unsafe {
            (self.native.shader_effect_set_texture3d)(handle, slot, texture_handle)
        })
    }

    /// Binds a cube map to a sampler slot.
    pub fn set_texture_cube(&self, slot: i32, texture: Option<&TextureCube>) -> Result<()> {
        let handle = self.effect.handle()?;
        let texture_handle = match texture {
            Some(texture) => texture.handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        // SAFETY: as above.
        self.native.check(unsafe {
            (self.native.shader_effect_set_texture_cube)(handle, slot, texture_handle)
        })
    }

    /// Declares a uniform block: a binding point and the names in it.
    ///
    /// Upstream takes the names and their offsets as parallel arrays, so this
    /// takes pairs and splits them -- two arrays that could disagree in length
    /// is exactly the shape a Rust API should not hand a caller.
    pub fn declare_uniform_block(&self, binding: i32, members: &[(&str, i32)]) -> Result<()> {
        let handle = self.effect.handle()?;
        let names: Vec<sys::CNA_StringView> =
            members.iter().map(|(name, _)| string_view(name)).collect();
        let offsets: Vec<i32> = members.iter().map(|(_, offset)| *offset).collect();
        // SAFETY: the handle is owned, and both arrays outlive the call with
        // the same length by construction.
        self.native.check(unsafe {
            (self.native.shader_effect_declare_uniform_block_ext)(
                handle,
                binding,
                if names.is_empty() {
                    core::ptr::null()
                } else {
                    names.as_ptr()
                },
                if offsets.is_empty() {
                    core::ptr::null()
                } else {
                    offsets.as_ptr()
                },
                names.len() as u64,
            )
        })
    }
}

/// The three transforms every CNA shader effect carries by name.
impl ShaderEffect {
    /// The world transform.
    pub fn world(&self) -> Result<Matrix> {
        self.matrix(|native, handle, out| {
            // SAFETY: owned handle, live output.
            unsafe { (native.shader_effect_get_world)(handle, out) }
        })
    }

    /// Replaces the world transform.
    pub fn set_world(&self, value: Matrix) -> Result<()> {
        let handle = self.effect.handle()?;
        // SAFETY: the handle is owned and the matrix is by value.
        self.native
            .check(unsafe { (self.native.shader_effect_set_world)(handle, native_matrix(value)) })
    }

    /// The view transform.
    pub fn view(&self) -> Result<Matrix> {
        self.matrix(|native, handle, out| {
            // SAFETY: owned handle, live output.
            unsafe { (native.shader_effect_get_view)(handle, out) }
        })
    }

    /// Replaces the view transform.
    pub fn set_view(&self, value: Matrix) -> Result<()> {
        let handle = self.effect.handle()?;
        // SAFETY: the handle is owned and the matrix is by value.
        self.native
            .check(unsafe { (self.native.shader_effect_set_view)(handle, native_matrix(value)) })
    }

    /// The projection transform.
    pub fn projection(&self) -> Result<Matrix> {
        self.matrix(|native, handle, out| {
            // SAFETY: owned handle, live output.
            unsafe { (native.shader_effect_get_projection)(handle, out) }
        })
    }

    /// Replaces the projection transform.
    pub fn set_projection(&self, value: Matrix) -> Result<()> {
        let handle = self.effect.handle()?;
        // SAFETY: the handle is owned and the matrix is by value.
        self.native.check(unsafe {
            (self.native.shader_effect_set_projection)(handle, native_matrix(value))
        })
    }

    fn matrix(
        &self,
        route: impl FnOnce(&Native, sys::CNA_Handle, *mut sys::CNA_Matrix) -> sys::CNA_Result,
    ) -> Result<Matrix> {
        let handle = self.effect.handle()?;
        let mut value = sys::CNA_Matrix::default();
        self.native
            .check(route(&self.native, handle, &mut value))?;
        Ok(value_matrix(value))
    }
}

fn pointer(values: &[f32]) -> *const f32 {
    if values.is_empty() {
        core::ptr::null()
    } else {
        values.as_ptr()
    }
}

fn string_view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast::<core::ffi::c_char>(),
        byte_length: value.len() as u64,
    }
}

fn native_matrix(value: Matrix) -> sys::CNA_Matrix {
    sys::CNA_Matrix {
        m11: value.M11, m12: value.M12, m13: value.M13, m14: value.M14,
        m21: value.M21, m22: value.M22, m23: value.M23, m24: value.M24,
        m31: value.M31, m32: value.M32, m33: value.M33, m34: value.M34,
        m41: value.M41, m42: value.M42, m43: value.M43, m44: value.M44,
    }
}

fn value_matrix(value: sys::CNA_Matrix) -> Matrix {
    Matrix {
        M11: value.m11, M12: value.m12, M13: value.m13, M14: value.m14,
        M21: value.m21, M22: value.m22, M23: value.m23, M24: value.m24,
        M31: value.m31, M32: value.m32, M33: value.m33, M34: value.m34,
        M41: value.m41, M42: value.m42, M43: value.m43, M44: value.m44,
    }
}
