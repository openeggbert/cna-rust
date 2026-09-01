//! `texture.h`: CNA's format arithmetic, and textures with no graphics device.
//!
//! # Why the format routes are called rather than restated
//!
//! How many bytes a `SurfaceFormat` costs, how many texels a compression block
//! covers, what pixel-store alignment it wants -- all of it is arithmetic a
//! binding could write out in a `match`. Doing so is how two sides of an ABI
//! drift apart, and this codebase has a measured example of the consequence:
//! `RUST-UPSTREAM-024`, where a restated stride list in CNA's own C layer went
//! stale against CNA's own canonical table and now refuses every physically
//! based morph target. So these are calls, not tables.
//!
//! # Textures with no device
//!
//! [`StandaloneTexture`] is deliberately **not** a
//! [`Texture2D`](crate::Microsoft::Xna::Framework::Graphics::Texture2D). An XNA
//! `Texture2D` is a `GraphicsResource`, and a `GraphicsResource` has a
//! `GraphicsDevice`; these have none. Giving them the `Texture2D` type would
//! mean either inventing a device they do not have or making every existing
//! `Texture2D` carry an absent one. They are their own type, which is what
//! they are: pixels held without a renderer, for a tool, a test, or a headless
//! pipeline.

use std::sync::Mutex;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::graphics_resource::HasResourceState;
use crate::native::Native;
use crate::value::Color;
use crate::Microsoft::Xna::Framework::Graphics::{GraphicsDevice, SurfaceFormat, Texture2D};

/// The image encodings `save_to_file` writes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ImageFormat {
    /// Lossless, with alpha.
    Png,
    /// Lossy, without alpha.
    Jpeg,
}

impl ImageFormat {
    const fn to_native(self) -> sys::CNA_TextureImageFormat {
        match self {
            Self::Png => sys::CNA_TEXTURE_IMAGE_FORMAT_PNG,
            Self::Jpeg => sys::CNA_TEXTURE_IMAGE_FORMAT_JPEG,
        }
    }
}

/// What CNA says a `SurfaceFormat` costs and requires.
///
/// Every answer here is read from CNA rather than computed, so a format whose
/// packing changes upstream changes here with it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormatFacts {
    /// Bytes per compression block, or per texel when uncompressed.
    pub bytes_per_unit: i32,
    /// Texels one compression block covers, squared: 1 for uncompressed.
    pub block_size_squared: i32,
    /// The pixel-store alignment the format wants, from one through eight.
    pub pixel_store_alignment: i32,
}

impl FormatFacts {
    /// Reads every fact CNA has about a format.
    pub fn of(format: SurfaceFormat) -> Result<Self> {
        let native = Native::process()?;
        let value = format as sys::CNA_SurfaceFormat;
        Ok(Self {
            bytes_per_unit: native.texture_format_size(value)?,
            block_size_squared: native.texture_block_size_squared(value)?,
            pixel_store_alignment: native.texture_pixel_store_alignment(value)?,
        })
    }

    /// Whether the format is compressed, as its block size reports it.
    #[must_use]
    pub const fn is_compressed(self) -> bool {
        self.block_size_squared > 1
    }
}

/// Whether an element size may be used to read this format back.
///
/// `Ok(())` when compatible. The refusal is passed through as CNA gave it
/// rather than reduced to a bool, because "this format does not divide evenly"
/// and "this is not a format" are different answers.
pub fn validate_get_data_element_size(
    format: SurfaceFormat,
    element_size_in_bytes: i32,
) -> Result<()> {
    Native::process()?
        .validate_texture_get_data_format(format as sys::CNA_SurfaceFormat, element_size_in_bytes)
}

/// Whether the renderer-independent base `Texture` contract accepts a format.
///
/// Narrow by design: CNA answers success for `Color` alone and `NOT_SUPPORTED`
/// for every other valid format. That is the base contract, not the renderer's
/// -- a renderer accepts far more -- and the two are worth not confusing.
pub fn validate_base_texture_format(format: SurfaceFormat) -> Result<()> {
    Native::process()?.validate_texture_format(format as sys::CNA_SurfaceFormat)
}

/// The format and mip facts every texture kind shares.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextureFacts {
    /// Number of mip levels.
    pub level_count: u32,
    /// The surface format.
    pub format: SurfaceFormat,
}

/// Where a texture's pixels actually live.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageInfo {
    /// A renderer resource is retained.
    pub has_renderer: bool,
    /// An authoritative CPU pixel shadow is retained.
    pub has_cpu_shadow: bool,
}

/// A `Texture2D` held without any graphics device.
///
/// Owns its handle and destroys it on drop. See the module documentation for
/// why this is not a `Texture2D`.
#[derive(Debug)]
pub struct StandaloneTexture {
    handle: Mutex<sys::CNA_Handle>,
    native: std::sync::Arc<Native>,
}

impl StandaloneTexture {
    /// A default standalone texture: no dimensions, no renderer, no pixels.
    pub fn empty() -> Result<Self> {
        let native = Native::process()?;
        let handle = native.create_standalone_texture2d()?;
        Ok(Self {
            handle: Mutex::new(handle),
            native,
        })
    }

    /// Decodes an image file into a CPU-backed standalone texture.
    pub fn from_file(path: &str) -> Result<Self> {
        let native = Native::process()?;
        let handle = native.create_texture2d_from_file(path)?;
        Ok(Self {
            handle: Mutex::new(handle),
            native,
        })
    }

    /// Builds a CPU-only texture from exactly `width * height` pixels.
    pub fn from_pixels(
        width: u32,
        height: u32,
        format: SurfaceFormat,
        pixels: &[Color],
    ) -> Result<Self> {
        let expected = usize::try_from(width)
            .ok()
            .and_then(|w| usize::try_from(height).ok().and_then(|h| w.checked_mul(h)))
            .ok_or(CnaError::InvalidInput("texture dimensions overflow"))?;
        if pixels.len() != expected {
            return Err(CnaError::InvalidInput(
                "a CPU-only texture needs exactly width * height pixels",
            ));
        }
        let native = Native::process()?;
        let colors: Vec<sys::CNA_Color> = pixels.iter().copied().map(to_native_color).collect();
        let handle = native.create_cpu_only_texture2d(
            width,
            height,
            format as sys::CNA_SurfaceFormat,
            &colors,
        )?;
        Ok(Self {
            handle: Mutex::new(handle),
            native,
        })
    }

    /// The format and mip facts CNA reports for this texture.
    pub fn facts(&self) -> Result<TextureFacts> {
        let info = self.native.texture_common_info(self.get()?)?;
        Ok(TextureFacts {
            level_count: info.level_count,
            format: SurfaceFormat::from_native(info.format)
                .ok_or(CnaError::InvalidInput("native texture format is unknown"))?,
        })
    }

    /// Where this texture's pixels live.
    pub fn storage(&self) -> Result<StorageInfo> {
        let (has_renderer, has_cpu_shadow) = self.native.texture2d_storage(self.get()?)?;
        Ok(StorageInfo {
            has_renderer,
            has_cpu_shadow,
        })
    }

    /// Uploads tightly packed RGBA8 bytes.
    ///
    /// The slice must be exactly four bytes per pixel of the texture's own
    /// dimensions; anything else is refused rather than truncated.
    pub fn set_rgba8_bytes(&self, bytes: &[u8]) -> Result<()> {
        self.native.set_texture2d_rgba8_bytes(self.get()?, bytes)
    }

    /// Encodes the texture straight to a file.
    pub fn save_to_file(&self, format: ImageFormat, path: &str) -> Result<()> {
        self.native
            .save_texture2d_file(self.get()?, format.to_native(), path)
    }

    /// Releases the handle early. Idempotent.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        self.native.destroy_texture(handle)
    }

    fn get(&self) -> Result<sys::CNA_Handle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput(
                "the standalone texture has been released",
            ));
        }
        Ok(handle)
    }
}

impl Drop for StandaloneTexture {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// The `texture.h` routes that take a device-owned `Texture2D`.
pub trait Texture2DFile {
    /// Decodes an image file onto this texture's device.
    fn from_file(device: &GraphicsDevice, path: &str) -> Result<Texture2D>;

    /// Creates and uploads a `Color` texture in one call.
    fn from_pixels(
        device: &GraphicsDevice,
        width: u32,
        height: u32,
        pixels: &[Color],
    ) -> Result<Texture2D>;

    /// The format and mip facts CNA reports.
    fn facts(&self) -> Result<TextureFacts>;

    /// Where this texture's pixels live.
    fn storage(&self) -> Result<StorageInfo>;

    /// Uploads tightly packed RGBA8 bytes.
    fn set_rgba8_bytes(&self, bytes: &[u8]) -> Result<()>;

    /// Encodes the texture straight to a file, without a Rust stream.
    fn save_to_file(&self, format: ImageFormat, path: &str) -> Result<()>;
}

impl Texture2DFile for Texture2D {
    fn from_file(device: &GraphicsDevice, path: &str) -> Result<Texture2D> {
        let handle = device
            .state_native()
            .create_texture2d_from_file_with_device(device.handle()?, path)?;
        Texture2D::from_owned_handle(device, handle)
    }

    fn from_pixels(
        device: &GraphicsDevice,
        width: u32,
        height: u32,
        pixels: &[Color],
    ) -> Result<Texture2D> {
        let colors: Vec<sys::CNA_Color> = pixels.iter().copied().map(to_native_color).collect();
        let handle = device.state_native().create_texture2d_from_rgba8(
            device.handle()?,
            width,
            height,
            &colors,
        )?;
        Texture2D::from_owned_handle(device, handle)
    }

    fn facts(&self) -> Result<TextureFacts> {
        let info = self
            .resource_state()
            .device()
            .state_native()
            .texture_common_info(self.resource_state().require_handle()?)?;
        Ok(TextureFacts {
            level_count: info.level_count,
            format: SurfaceFormat::from_native(info.format)
                .ok_or(CnaError::InvalidInput("native texture format is unknown"))?,
        })
    }

    fn storage(&self) -> Result<StorageInfo> {
        let (has_renderer, has_cpu_shadow) = self
            .resource_state()
            .device()
            .state_native()
            .texture2d_storage(self.resource_state().require_handle()?)?;
        Ok(StorageInfo {
            has_renderer,
            has_cpu_shadow,
        })
    }

    fn set_rgba8_bytes(&self, bytes: &[u8]) -> Result<()> {
        self.resource_state().device().state_native()
            .set_texture2d_rgba8_bytes(self.resource_state().require_handle()?, bytes)
    }

    fn save_to_file(&self, format: ImageFormat, path: &str) -> Result<()> {
        self.resource_state().device().state_native().save_texture2d_file(
            self.resource_state().require_handle()?,
            format.to_native(),
            path,
        )
    }
}

fn to_native_color(color: Color) -> sys::CNA_Color {
    sys::CNA_Color {
        r: color.R(),
        g: color.G(),
        b: color.B(),
        a: color.A(),
    }
}
