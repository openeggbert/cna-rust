//! The graphics device's own surface: the back buffer, the packed-colour data
//! paths, render targets, presentation parameters, and vertex declarations
//! read back rather than built.
//!
//! What ties these together is that each answers a question about a device or a
//! resource *as it currently is*, rather than creating something. A tool that
//! did not build a buffer can still describe it; a test can read the back
//! buffer it just drew; a caller can reset a device without going through
//! `GraphicsDeviceManager`.
//!
//! The RGBA8 paths are the packed-colour fast lane beside the general typed
//! `GetData`/`SetData` that already exist: the renderer moves them without a
//! conversion. The back buffer has no other reader at all.

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::graphics_resource::HasResourceState;
use crate::native::Native;
use crate::value::{Color, Rectangle};
use crate::Microsoft::Xna::Framework::Graphics::{
    CubeMapFace, GraphicsDevice, PresentationParameters, RenderTarget2D, RenderTargetCube,
    RenderTargetUsage, SurfaceFormat, Texture2D, VertexBuffer, VertexElement,
    VertexElementFormat, VertexElementUsage,
};

/// What the back buffer currently is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackBufferInfo {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// The surface format.
    pub format: SurfaceFormat,
}

/// The device surface routes with no XNA counterpart.
pub trait DeviceSurface {
    /// What the back buffer currently is.
    ///
    /// Distinct from the `PresentationParameters` the device was created with:
    /// those are what was *asked for*, and this is what the renderer settled
    /// on, which a driver is free to differ from.
    fn back_buffer_info(&self) -> Result<BackBufferInfo>;

    /// Reads the back buffer as packed colours.
    ///
    /// The destination must hold at least `width * height` pixels; the number
    /// actually written is returned. This is what a screenshot, a golden-image
    /// test or a video capture reads, and there is no other route to it.
    fn read_back_buffer(&self, pixels: &mut [Color]) -> Result<usize>;

    /// Binds a render target, or unbinds with `None`.
    fn set_render_target(&self, target: Option<&RenderTarget2D>) -> Result<()>;

    /// Binds one face of a cube render target, or unbinds with `None`.
    fn set_render_target_face(
        &self,
        target: Option<&RenderTargetCube>,
        face: CubeMapFace,
    ) -> Result<()>;

    /// Applies presentation parameters to a live device.
    ///
    /// The reset path XNA drives through `GraphicsDeviceManager`. Applying them
    /// directly is what a caller without a manager -- a tool, a test, an
    /// embedder -- needs.
    fn apply_presentation_parameters(&self, parameters: &PresentationParameters) -> Result<()>;

    /// Re-enumerates the adapters the process can see.
    ///
    /// Adapters are enumerated once and cached; a monitor plugged in after that
    /// is invisible until this runs.
    fn refresh_adapters(&self) -> Result<()>;
}

impl DeviceSurface for GraphicsDevice {
    fn back_buffer_info(&self) -> Result<BackBufferInfo> {
        let (width, height, format) = self.state_native().backbuffer_info(self.handle()?)?;
        Ok(BackBufferInfo {
            width,
            height,
            format: SurfaceFormat::from_native(format)
                .ok_or(CnaError::InvalidInput("native surface format is unknown"))?,
        })
    }

    fn read_back_buffer(&self, pixels: &mut [Color]) -> Result<usize> {
        let mut native = vec![sys::CNA_Color::default(); pixels.len()];
        let written = self
            .state_native()
            .backbuffer_data(self.handle()?, &mut native)?;
        for (destination, source) in pixels.iter_mut().zip(native.iter().take(written)) {
            *destination = from_native_color(*source);
        }
        Ok(written)
    }

    fn set_render_target(&self, target: Option<&RenderTarget2D>) -> Result<()> {
        let handle = match target {
            Some(target) => target.resource_state().require_handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        self.state_native()
            .set_render_target2d(self.handle()?, handle)
    }

    fn set_render_target_face(
        &self,
        target: Option<&RenderTargetCube>,
        face: CubeMapFace,
    ) -> Result<()> {
        let handle = match target {
            Some(target) => target.resource_state().require_handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        self.state_native()
            .set_render_target_cube(self.handle()?, handle, face as u32)
    }

    fn apply_presentation_parameters(&self, parameters: &PresentationParameters) -> Result<()> {
        let native = parameters.to_native(true);
        self.state_native()
            .set_device_presentation_parameters(self.handle()?, &native)
    }

    fn refresh_adapters(&self) -> Result<()> {
        self.state_native().refresh_graphics_adapters(self.handle()?)
    }
}

/// Whether a `RenderTargetUsage` keeps what was drawn between frames.
///
/// XNA's rule, asked of CNA rather than restated: the enum has three members
/// and which of them preserve is the ABI's answer, not this side's.
pub fn usage_preserves_contents(usage: RenderTargetUsage) -> Result<bool> {
    Native::process()?.usage_preserves_contents(usage as u32)
}

/// Clones presentation parameters through CNA's own copy.
///
/// Answers the native structure rather than a Rust `PresentationParameters`,
/// and deliberately: the point of asking CNA to clone is that it copies the
/// *whole versioned structure*, including a field a future ABI adds that this
/// build has no Rust member for. Converting the result back into the Rust type
/// would drop exactly what the route was called for. A caller comparing two
/// configurations, or handing one straight to
/// [`DeviceSurface::apply_presentation_parameters`], wants the native value.
pub fn clone_presentation_parameters(
    parameters: &PresentationParameters,
) -> Result<sys::CNA_PresentationParameters> {
    Native::process()?.clone_presentation_parameters(&parameters.to_native(true))
}

/// The client bounds these presentation parameters describe.
pub fn presentation_bounds(parameters: &PresentationParameters) -> Result<Rectangle> {
    let bounds = Native::process()?.presentation_parameter_bounds(&parameters.to_native(true))?;
    Ok(Rectangle::new(bounds.x, bounds.y, bounds.width, bounds.height))
}

/// The packed-colour data paths beside the general typed ones.
pub trait Rgba8Data {
    /// Uploads packed colours, one per texel.
    fn set_rgba8(&self, pixels: &[Color]) -> Result<()>;

    /// Reads packed colours back; answers how many were written.
    fn read_rgba8(&self, pixels: &mut [Color]) -> Result<usize>;
}

impl Rgba8Data for Texture2D {
    fn set_rgba8(&self, pixels: &[Color]) -> Result<()> {
        let native: Vec<sys::CNA_Color> = pixels.iter().copied().map(to_native_color).collect();
        let state = self.resource_state();
        state
            .device()
            .state_native()
            .set_texture2d_rgba8(state.require_handle()?, &native)
    }

    fn read_rgba8(&self, pixels: &mut [Color]) -> Result<usize> {
        let state = self.resource_state();
        let mut native = vec![sys::CNA_Color::default(); pixels.len()];
        let written = state
            .device()
            .state_native()
            .texture2d_rgba8(state.require_handle()?, &mut native)?;
        for (destination, source) in pixels.iter_mut().zip(native.iter().take(written)) {
            *destination = from_native_color(*source);
        }
        Ok(written)
    }
}

/// A vertex layout read back from a buffer that already has one.
///
/// `VertexDeclaration::new` builds a layout from elements the caller states;
/// this reports the layout a *buffer* already carries, which is what a tool
/// describing a buffer it did not create needs.
///
/// There is no counterpart for [`VertexDeclaration`] itself, and deliberately:
/// the Rust `VertexDeclaration` holds its elements and stride in Rust and has
/// no native handle at all, so `cna_vertex_declaration_get_stride` and its
/// neighbours have nothing on this side to be called with.
pub trait ReadsVertexLayout {
    /// The byte stride of one vertex.
    fn native_stride(&self) -> Result<i32>;

    /// The elements, in the order CNA reports them.
    fn native_elements(&self) -> Result<Vec<VertexElement>>;
}

impl ReadsVertexLayout for VertexBuffer {
    fn native_stride(&self) -> Result<i32> {
        // A buffer's stride is its declaration's, and the declaration is the
        // one the buffer was created with rather than one the caller still
        // holds -- which is the whole reason to read it from the buffer.
        let elements = self.native_elements()?;
        let state = self.resource_state();
        let _ = state.require_handle()?;
        Ok(elements
            .iter()
            .map(|element| element.Offset() + element.VertexElementFormat().byte_size())
            .max()
            .unwrap_or(0))
    }

    fn native_elements(&self) -> Result<Vec<VertexElement>> {
        let state = self.resource_state();
        let native = state
            .device()
            .state_native()
            .vertex_buffer_declaration_elements(state.require_handle()?)?;
        native.iter().map(convert_element).collect()
    }
}

fn convert_element(native: &sys::CNA_VertexElement) -> Result<VertexElement> {
    Ok(VertexElement::new(
        native.offset,
        VertexElementFormat::from_native_value(native.format)
            .ok_or(CnaError::InvalidInput("native vertex element format is unknown"))?,
        VertexElementUsage::from_native_value(native.usage)
            .ok_or(CnaError::InvalidInput("native vertex element usage is unknown"))?,
        native.usage_index,
    ))
}

const fn to_native_color(color: Color) -> sys::CNA_Color {
    sys::CNA_Color {
        r: color.R(),
        g: color.G(),
        b: color.B(),
        a: color.A(),
    }
}

fn from_native_color(value: sys::CNA_Color) -> Color {
    Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
        i32::from(value.r),
        i32::from(value.g),
        i32::from(value.b),
        i32::from(value.a),
    )
}

/// The platform window handle a device presents to.
///
/// Distinct from [`crate::extensions::window::NativeWindow::native_handles`],
/// which answers the *game window's* handles: a device created outside a game
/// has no game window, and this is the only route to whatever surface it does
/// present to. The value is the platform's and is not the caller's to free.
pub fn device_native_window(device: &GraphicsDevice) -> Result<sys::CNA_NativeHandleValue> {
    device.state_native().device_native_window(device.handle()?)
}

/// Clones a device configuration through CNA's own copy.
///
/// Answers the native structure for the same reason
/// [`clone_presentation_parameters`] does: the point of asking CNA is that it
/// copies the whole versioned structure, and converting back would drop
/// whatever this build has no member for.
pub fn clone_device_information(
    information: &sys::CNA_GraphicsDeviceInformation,
) -> Result<sys::CNA_GraphicsDeviceInformation> {
    Native::process()?.clone_device_information(information)
}

/// One sprite in a batched submission.
///
/// The batched form exists because a per-sprite call crosses the FFI boundary
/// once per sprite; this crosses it once for the whole array. A caller drawing
/// a tilemap or a particle system is the reason.
#[derive(Clone, Copy, Debug)]
pub struct ScaledSprite {
    /// Where to draw, in screen space.
    pub position: crate::value::Vector2,
    /// The source rectangle within the texture.
    pub source: Rectangle,
    /// Per-channel tint multiplied with the sampled texture.
    pub color: Color,
    /// Rotation in radians.
    pub rotation: f32,
    /// The origin the rotation and scale are about.
    pub origin: crate::value::Vector2,
    /// Per-axis scale.
    pub scale: crate::value::Vector2,
    /// Flip flags.
    pub effects: crate::Microsoft::Xna::Framework::Graphics::SpriteEffects,
    /// Sort depth, zero to one.
    pub layer_depth: f32,
}

/// Batched sprite submission, and the arbitrary-mesh escape hatch.
pub trait BatchedSprites {
    /// Submits many scaled sprites in one crossing.
    fn submit_scaled(&self, sprites: &[ScaledSprite]) -> Result<()>;

    /// Draws an arbitrary triangle mesh through the batch.
    ///
    /// The three vertex arrays must be the same length, and every index must
    /// be inside them; both are checked here, because CNA is handed raw
    /// pointers and counts and would read past the end otherwise.
    fn draw_mesh(
        &self,
        positions: &[crate::value::Vector2],
        colors: &[Color],
        texture_coordinates: &[crate::value::Vector2],
        indices: &[u16],
    ) -> Result<()>;
}

impl BatchedSprites for crate::Microsoft::Xna::Framework::Graphics::SpriteBatch {
    fn submit_scaled(&self, sprites: &[ScaledSprite]) -> Result<()> {
        let commands: Vec<sys::CNA_SpriteScaledCommand> = sprites
            .iter()
            .map(|sprite| sys::CNA_SpriteScaledCommand {
                position: sys::CNA_Vector2 {
                    x: sprite.position.X,
                    y: sprite.position.Y,
                },
                source: sys::CNA_Rectangle {
                    x: sprite.source.X,
                    y: sprite.source.Y,
                    width: sprite.source.Width,
                    height: sprite.source.Height,
                },
                color: to_native_color(sprite.color),
                rotation: sprite.rotation,
                origin: sys::CNA_Vector2 {
                    x: sprite.origin.X,
                    y: sprite.origin.Y,
                },
                scale: sys::CNA_Vector2 {
                    x: sprite.scale.X,
                    y: sprite.scale.Y,
                },
                effects: sprite.effects.bits(),
                layer_depth: sprite.layer_depth,
            })
            .collect();
        let state = self.resource_state();
        state
            .device()
            .state_native()
            .submit_scaled_sprites(state.require_handle()?, &commands)
    }

    fn draw_mesh(
        &self,
        positions: &[crate::value::Vector2],
        colors: &[Color],
        texture_coordinates: &[crate::value::Vector2],
        indices: &[u16],
    ) -> Result<()> {
        if positions.len() != colors.len() || positions.len() != texture_coordinates.len() {
            return Err(CnaError::InvalidInput(
                "a sprite mesh needs one colour and one texture coordinate per position",
            ));
        }
        let vertex_count = positions.len();
        if indices.iter().any(|index| usize::from(*index) >= vertex_count) {
            return Err(CnaError::InvalidInput(
                "every sprite-mesh index must name a vertex that exists",
            ));
        }
        let native_positions: Vec<sys::CNA_Vector2> = positions
            .iter()
            .map(|value| sys::CNA_Vector2 {
                x: value.X,
                y: value.Y,
            })
            .collect();
        let native_colors: Vec<sys::CNA_Color> =
            colors.iter().copied().map(to_native_color).collect();
        let native_coordinates: Vec<sys::CNA_Vector2> = texture_coordinates
            .iter()
            .map(|value| sys::CNA_Vector2 {
                x: value.X,
                y: value.Y,
            })
            .collect();
        let state = self.resource_state();
        let mesh = sys::CNA_SpriteMeshEXT {
            struct_size: core::mem::size_of::<sys::CNA_SpriteMeshEXT>() as u32,
            struct_version: 1,
            effect: sys::CNA_INVALID_HANDLE,
            positions: native_positions.as_ptr(),
            colors: native_colors.as_ptr(),
            texture_coordinates: native_coordinates.as_ptr(),
            indices: indices.as_ptr(),
            vertex_count: vertex_count as u64,
            index_count: indices.len() as u64,
        };
        state
            .device()
            .state_native()
            .draw_sprite_mesh(state.require_handle()?, &mesh)
    }
}

/// Reads a vertex buffer's bytes back.
///
/// The typed `GetData` already exists; this is the untyped one, for a caller
/// that wants the raw record bytes -- a tool dumping a buffer, or a test
/// checking what an upload actually stored.
pub fn read_vertex_buffer_bytes(
    buffer: &VertexBuffer,
    start_index: u64,
    element_count: u64,
    destination: &mut [u8],
) -> Result<usize> {
    let transfer = sys::CNA_VertexBufferTransfer {
        struct_size: core::mem::size_of::<sys::CNA_VertexBufferTransfer>() as u32,
        struct_version: 1,
        vertex_type: 0,
        options: sys::CNA_SET_DATA_NONE,
        start_index,
        element_count,
    };
    let state = buffer.resource_state();
    state
        .device()
        .state_native()
        .vertex_buffer_data(state.require_handle()?, &transfer, destination)
}
