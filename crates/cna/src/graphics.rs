#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use core::mem::size_of;
use std::io::Read;
use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::native::Native;
use crate::value::{Color, Rectangle, Vector2};

/// Public behavior shared by XNA graphics resources.
#[allow(non_snake_case)]
pub trait GraphicsResource {
    fn IsDisposed(&self) -> bool;
    fn Dispose(&mut self) -> Result<()>;
}

/// XNA base relationship projected as a Rust trait.
#[allow(non_snake_case)]
pub trait Texture: GraphicsResource {
    fn LevelCount(&self) -> i32;
}

/// Callback-scoped borrowed graphics device.
pub struct GraphicsDevice<'callback> {
    native: &'callback Arc<Native>,
    handle: sys::CNA_Handle,
}

#[allow(non_snake_case)]
impl<'callback> GraphicsDevice<'callback> {
    pub(crate) fn borrow(native: &'callback Arc<Native>, game: sys::CNA_Handle) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        native.borrow_graphics_device(game, &mut handle)?;
        Ok(Self { native, handle })
    }

    pub fn Clear(&self, color: Color) -> Result<()> {
        let scale = 1.0 / 255.0;
        self.native.clear_graphics_device(
            self.handle,
            [
                f32::from(color.R()) * scale,
                f32::from(color.G()) * scale,
                f32::from(color.B()) * scale,
                f32::from(color.A()) * scale,
            ],
        )
    }

    pub fn Viewport(&self) -> Result<Viewport> {
        let mut viewport = sys::CNA_Viewport::default();
        self.native.graphics_viewport(self.handle, &mut viewport)?;
        Ok(Viewport::from_native(viewport))
    }

    pub(crate) fn renderer_info(&self) -> Result<(String, bool, bool, u32)> {
        let mut info = sys::CNA_RendererInfo {
            struct_size: size_of::<sys::CNA_RendererInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_RendererInfo::default()
        };
        self.native.renderer_info(self.handle, &mut info)?;
        let mut name_size = 0_u64;
        self.native
            .renderer_name_size(self.handle, &mut name_size)?;
        let capacity = usize::try_from(name_size)
            .map_err(|_| CnaError::InvalidInput("renderer name is too large"))?;
        let mut bytes = vec![0_u8; capacity];
        let mut copied = 0_u64;
        self.native
            .copy_renderer_name(self.handle, &mut bytes, &mut copied)?;
        let name = String::from_utf8_lossy(&bytes).into_owned();
        let supports_3d =
            info.capability_flags & (1_u64 << sys::CNA_GRAPHICS_CAPABILITY_THREE_D) != 0;
        let supports_depth = info.capability_flags
            & (1_u64 << sys::CNA_GRAPHICS_CAPABILITY_DEPTH_STENCIL_BUFFER)
            != 0;
        Ok((
            name,
            supports_3d,
            supports_depth,
            info.max_texture_dimension,
        ))
    }
}

/// XNA viewport value with property-shaped accessors.
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Viewport {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    min_depth: f32,
    max_depth: f32,
}

#[allow(non_snake_case)]
impl Viewport {
    #[must_use]
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }

    fn from_native(value: sys::CNA_Viewport) -> Self {
        Self {
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
            min_depth: value.min_depth,
            max_depth: value.max_depth,
        }
    }

    #[must_use]
    pub const fn X(&self) -> i32 {
        self.x
    }
    #[must_use]
    pub const fn Y(&self) -> i32 {
        self.y
    }
    #[must_use]
    pub const fn Width(&self) -> i32 {
        self.width
    }
    #[must_use]
    pub const fn Height(&self) -> i32 {
        self.height
    }
    #[must_use]
    pub const fn MinDepth(&self) -> f32 {
        self.min_depth
    }
    #[must_use]
    pub const fn MaxDepth(&self) -> f32 {
        self.max_depth
    }
    #[must_use]
    pub const fn Bounds(&self) -> Rectangle {
        Rectangle::new(self.x, self.y, self.width, self.height)
    }
    #[must_use]
    pub fn AspectRatio(&self) -> f32 {
        if self.width == 0 || self.height == 0 {
            0.0
        } else {
            self.width as f32 / self.height as f32
        }
    }
}

/// Owned native XNA `Texture2D` resource.
pub struct Texture2D {
    native: Arc<Native>,
    handle: sys::CNA_Handle,
    width: i32,
    height: i32,
    level_count: i32,
}

#[allow(non_snake_case)]
impl Texture2D {
    /// XNA `FromStream` projected over Rust's `Read` trait.
    pub fn FromStream<R: Read>(device: &GraphicsDevice<'_>, stream: &mut R) -> Result<Self> {
        let mut bytes = Vec::new();
        stream
            .read_to_end(&mut bytes)
            .map_err(|error| CnaError::Native {
                code: sys::CNA_RESULT_IO,
                message: error.to_string(),
            })?;
        if bytes.is_empty() {
            return Err(CnaError::InvalidInput(
                "encoded texture data must not be empty",
            ));
        }
        let mut handle = sys::CNA_INVALID_HANDLE;
        device
            .native
            .create_texture_from_encoded(device.handle, &bytes, &mut handle)?;
        match Self::from_handle(Arc::clone(device.native), handle) {
            Ok(texture) => Ok(texture),
            Err(error) => {
                let _ = device.native.destroy_texture(handle);
                Err(error)
            }
        }
    }

    fn from_handle(native: Arc<Native>, handle: sys::CNA_Handle) -> Result<Self> {
        let mut info = sys::CNA_Texture2DInfo {
            struct_size: size_of::<sys::CNA_Texture2DInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_Texture2DInfo::default()
        };
        native.texture_info(handle, &mut info)?;
        Ok(Self {
            native,
            handle,
            width: i32::try_from(info.width)
                .map_err(|_| CnaError::InvalidInput("texture width exceeds i32"))?,
            height: i32::try_from(info.height)
                .map_err(|_| CnaError::InvalidInput("texture height exceeds i32"))?,
            level_count: i32::try_from(info.level_count)
                .map_err(|_| CnaError::InvalidInput("texture level count exceeds i32"))?,
        })
    }

    #[must_use]
    pub const fn Width(&self) -> i32 {
        self.width
    }

    #[must_use]
    pub const fn Height(&self) -> i32 {
        self.height
    }
}

impl Texture for Texture2D {
    fn LevelCount(&self) -> i32 {
        self.level_count
    }
}

impl GraphicsResource for Texture2D {
    fn IsDisposed(&self) -> bool {
        self.handle == sys::CNA_INVALID_HANDLE
    }

    fn Dispose(&mut self) -> Result<()> {
        if self.IsDisposed() {
            return Ok(());
        }
        let handle = self.handle;
        self.native.destroy_texture(handle)?;
        self.handle = sys::CNA_INVALID_HANDLE;
        Ok(())
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        let _ = self.Dispose();
    }
}

/// Owned `SpriteBatch`. A mutable borrow enforces one begin/draw/end sequence.
pub struct SpriteBatch {
    native: Arc<Native>,
    handle: sys::CNA_Handle,
    begun: bool,
}

#[allow(non_snake_case)]
impl SpriteBatch {
    pub fn new(device: &GraphicsDevice<'_>) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        device
            .native
            .create_sprite_batch(device.handle, &mut handle)?;
        Ok(Self {
            native: Arc::clone(device.native),
            handle,
            begun: false,
        })
    }

    pub fn Begin(&mut self) -> Result<()> {
        if self.begun {
            return Err(CnaError::InvalidInput("SpriteBatch.Begin was called twice"));
        }
        let info = sys::CNA_SpriteBatchBeginInfo {
            struct_size: size_of::<sys::CNA_SpriteBatchBeginInfo>() as u32,
            struct_version: 1,
            sort_mode: sys::CNA_SPRITE_SORT_MODE_DEFERRED,
            reserved: 0,
        };
        self.native.begin_sprite_batch(self.handle, &info)?;
        self.begun = true;
        Ok(())
    }

    pub fn Draw(&mut self, texture: &Texture2D, position: Vector2, color: Color) -> Result<()> {
        self.DrawWithTextureAndDestinationRectangleAndColor(
            texture,
            Rectangle::new(
                position.X as i32,
                position.Y as i32,
                texture.Width(),
                texture.Height(),
            ),
            color,
        )
    }

    /// Deterministic projection of the destination-rectangle Draw overload.
    pub fn DrawWithTextureAndDestinationRectangleAndColor(
        &mut self,
        texture: &Texture2D,
        destination: Rectangle,
        color: Color,
    ) -> Result<()> {
        if !self.begun {
            return Err(CnaError::InvalidInput(
                "SpriteBatch.Draw requires an active Begin/End interval",
            ));
        }
        if texture.IsDisposed() {
            return Err(CnaError::InvalidInput("cannot draw a disposed Texture2D"));
        }
        let command = sys::CNA_SpriteCommand {
            struct_size: size_of::<sys::CNA_SpriteCommand>() as u32,
            struct_version: 1,
            texture: texture.handle,
            destination: sys::CNA_Rectangle {
                x: destination.X,
                y: destination.Y,
                width: destination.Width,
                height: destination.Height,
            },
            source: sys::CNA_Rectangle::default(),
            color: sys::CNA_Color {
                r: color.R(),
                g: color.G(),
                b: color.B(),
                a: color.A(),
            },
            rotation: 0.0,
            origin: sys::CNA_Vector2::default(),
            effects: sys::CNA_SPRITE_EFFECT_NONE,
            layer_depth: 0.0,
        };
        self.native.submit_sprite(self.handle, &command)
    }

    pub fn End(&mut self) -> Result<()> {
        if !self.begun {
            return Err(CnaError::InvalidInput(
                "SpriteBatch.End requires an active interval",
            ));
        }
        let result = self.native.end_sprite_batch(self.handle);
        self.begun = false;
        result
    }
}

impl GraphicsResource for SpriteBatch {
    fn IsDisposed(&self) -> bool {
        self.handle == sys::CNA_INVALID_HANDLE
    }

    fn Dispose(&mut self) -> Result<()> {
        if self.IsDisposed() {
            return Ok(());
        }
        let handle = self.handle;
        self.native.destroy_sprite_batch(handle)?;
        self.handle = sys::CNA_INVALID_HANDLE;
        self.begun = false;
        Ok(())
    }
}

impl Drop for SpriteBatch {
    fn drop(&mut self) {
        let _ = self.Dispose();
    }
}
