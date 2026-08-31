#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use core::mem::size_of;
use std::any::Any;
use std::io::{Read, Write};
use std::sync::Arc;

use cna_sys as sys;

use crate::content::{ContentDisposable, ContentLoadable};
use crate::error::{CnaError, ErrorCategory, Result};
use crate::extensions::events::EventHandler;
use crate::value::Rectangle;

use super::resource::{BorrowedHandle, ResourceKind, ResourceState};
use super::{GraphicsDevice, GraphicsResource, SurfaceFormat, Texture, TextureRuntime};

/// Composition marker for XNA types inheriting `Texture2D`.
pub trait Texture2DBase: Texture {}

/// Owned native XNA `Texture2D` resource.
pub struct Texture2D {
    state: Arc<ResourceState>,
    width: i32,
    height: i32,
    level_count: i32,
    format: SurfaceFormat,
}

#[allow(non_snake_case)]
impl Texture2D {
    pub fn new(graphicsDevice: &GraphicsDevice, width: i32, height: i32) -> Result<Self> {
        Self::from_graphics_device_and_width_and_height_and_mip_map_and_format(
            graphicsDevice,
            width,
            height,
            false,
            SurfaceFormat::Color,
        )
    }

    pub fn from_graphics_device_and_width_and_height_and_mip_map_and_format(
        graphicsDevice: &GraphicsDevice,
        width: i32,
        height: i32,
        mipMap: bool,
        format: SurfaceFormat,
    ) -> Result<Self> {
        if width <= 0 || height <= 0 {
            return Err(CnaError::InvalidInput(
                "texture dimensions must be greater than zero",
            ));
        }
        let native_width =
            u32::try_from(width).map_err(|_| CnaError::InvalidInput("texture width is invalid"))?;
        let native_height = u32::try_from(height)
            .map_err(|_| CnaError::InvalidInput("texture height is invalid"))?;
        let info = sys::CNA_Texture2DCreateInfo {
            struct_size: size_of::<sys::CNA_Texture2DCreateInfo>() as u32,
            struct_version: 1,
            width: native_width,
            height: native_height,
            mip_map: if mipMap {
                sys::CNA_TRUE
            } else {
                sys::CNA_FALSE
            },
            reserved: [0; 3],
            format: u32::try_from(format as i32)
                .expect("all selected-profile SurfaceFormat values are nonnegative"),
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        graphicsDevice.state.native().create_texture(
            graphicsDevice.handle()?,
            &info,
            &mut handle,
        )?;
        match Self::from_handle(graphicsDevice, handle) {
            Ok(texture) => Ok(texture),
            Err(error) => {
                let _ = graphicsDevice.state.native().destroy_texture(handle);
                Err(error)
            }
        }
    }

    pub fn FromStream<R: Read>(
        graphicsDevice: &GraphicsDevice,
        stream: &mut R,
        width: i32,
        height: i32,
        zoom: bool,
    ) -> Result<Self> {
        if width <= 0 || height <= 0 {
            return Err(CnaError::InvalidInput(
                "decoded texture dimensions must be greater than zero",
            ));
        }
        let decode = sys::CNA_Texture2DDecodeInfo {
            struct_size: size_of::<sys::CNA_Texture2DDecodeInfo>() as u32,
            struct_version: 1,
            width: u32::try_from(width)
                .map_err(|_| CnaError::InvalidInput("decoded texture width is invalid"))?,
            height: u32::try_from(height)
                .map_err(|_| CnaError::InvalidInput("decoded texture height is invalid"))?,
            zoom: if zoom { sys::CNA_TRUE } else { sys::CNA_FALSE },
            reserved: [0; 7],
        };
        Self::from_stream(graphicsDevice, stream, Some(&decode))
    }

    /// XNA's two-argument `FromStream` overload.
    pub fn FromStreamWithGraphicsDeviceAndStream<R: Read>(
        graphicsDevice: &GraphicsDevice,
        stream: &mut R,
    ) -> Result<Self> {
        Self::from_stream(graphicsDevice, stream, None)
    }

    fn from_stream<R: Read>(
        graphicsDevice: &GraphicsDevice,
        stream: &mut R,
        decode: Option<&sys::CNA_Texture2DDecodeInfo>,
    ) -> Result<Self> {
        let mut bytes = Vec::new();
        stream
            .read_to_end(&mut bytes)
            .map_err(|error| CnaError::Native {
                code: sys::CNA_RESULT_IO,
                category: ErrorCategory::None,
                message: error.to_string(),
            })?;
        if bytes.is_empty() {
            return Err(CnaError::InvalidInput(
                "encoded texture data must not be empty",
            ));
        }
        let mut handle = sys::CNA_INVALID_HANDLE;
        graphicsDevice.state.native().create_texture_from_encoded(
            graphicsDevice.handle()?,
            &bytes,
            decode,
            &mut handle,
        )?;
        match Self::from_handle(graphicsDevice, handle) {
            Ok(texture) => Ok(texture),
            Err(error) => {
                let _ = graphicsDevice.state.native().destroy_texture(handle);
                Err(error)
            }
        }
    }

    fn from_handle(graphics_device: &GraphicsDevice, handle: sys::CNA_Handle) -> Result<Self> {
        Self::adopt(graphics_device, handle, None)
    }

    /// Adopts a texture CNA created and handed over outright.
    ///
    /// The engine layer has factory routes that publish an owned texture --
    /// `cna_color_grade_pass_create_identity_lut` is the first -- and the
    /// caller destroys it. This is that adoption, and it destroys the handle
    /// on failure so a refused wrap never strands one.
    pub(crate) fn from_owned_handle(
        graphics_device: &GraphicsDevice,
        handle: sys::CNA_Handle,
    ) -> Result<Self> {
        match Self::from_handle(graphics_device, handle) {
            Ok(texture) => Ok(texture),
            Err(error) => {
                let _ = graphics_device.state.native().destroy_texture(handle);
                Err(error)
            }
        }
    }

    /// Wraps a texture another native object owns for the duration of a borrow.
    ///
    /// `VideoPlayer::GetTexture` is the one caller: CNA hands back the frame
    /// texture on a borrow that ends when the frame advances, so the resulting
    /// `Texture2D` never destroys the handle and re-validates the borrow on
    /// every native use.
    pub(crate) fn from_borrowed_handle(
        graphics_device: &GraphicsDevice,
        handle: sys::CNA_Handle,
        owner: Arc<dyn BorrowedHandle>,
    ) -> Result<Self> {
        Self::adopt(graphics_device, handle, Some(owner))
    }

    fn adopt(
        graphics_device: &GraphicsDevice,
        handle: sys::CNA_Handle,
        owner: Option<Arc<dyn BorrowedHandle>>,
    ) -> Result<Self> {
        let mut info = sys::CNA_Texture2DInfo {
            struct_size: size_of::<sys::CNA_Texture2DInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_Texture2DInfo::default()
        };
        graphics_device
            .state
            .native()
            .texture_info(handle, &mut info)?;
        Ok(Self {
            state: match owner {
                None => ResourceState::new(graphics_device, handle, ResourceKind::Texture2D),
                Some(owner) => {
                    ResourceState::borrowed(graphics_device, handle, ResourceKind::Texture2D, owner)
                }
            },
            width: i32::try_from(info.width)
                .map_err(|_| CnaError::InvalidInput("texture width exceeds i32"))?,
            height: i32::try_from(info.height)
                .map_err(|_| CnaError::InvalidInput("texture height exceeds i32"))?,
            level_count: i32::try_from(info.level_count)
                .map_err(|_| CnaError::InvalidInput("texture level count exceeds i32"))?,
            format: SurfaceFormat::from_native(info.format)
                .ok_or(CnaError::InvalidInput("native texture format is unknown"))?,
        })
    }

    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.state.require_handle()
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
    pub const fn Bounds(&self) -> Rectangle {
        Rectangle::new(0, 0, self.width, self.height)
    }

    pub fn SetData<T: Copy>(&self, data: &[T]) -> Result<()> {
        let count = i32::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("texture data array is too large"))?;
        self.SetDataWithLevelAndRectAndDataAndStartIndexAndElementCount(0, None, data, 0, count)
    }

    pub fn SetDataWithDataAndStartIndexAndElementCount<T: Copy>(
        &self,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        self.SetDataWithLevelAndRectAndDataAndStartIndexAndElementCount(
            0,
            None,
            data,
            startIndex,
            elementCount,
        )
    }

    pub fn SetDataWithLevelAndRectAndDataAndStartIndexAndElementCount<T: Copy>(
        &self,
        level: i32,
        rect: Option<Rectangle>,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        let data_type = texture_data_type::<T>()?;
        let transfer =
            self.transfer(level, rect, data_type, data.len(), startIndex, elementCount)?;
        self.state.device().state.native().set_texture_data(
            self.state.require_handle()?,
            data_type,
            &transfer,
            data,
        )
    }

    pub fn GetData<T: Copy>(&self, data: &mut [T]) -> Result<()> {
        let count = i32::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("texture data array is too large"))?;
        self.GetDataWithLevelAndRectAndDataAndStartIndexAndElementCount(0, None, data, 0, count)
    }

    pub fn GetDataWithDataAndStartIndexAndElementCount<T: Copy>(
        &self,
        data: &mut [T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        self.GetDataWithLevelAndRectAndDataAndStartIndexAndElementCount(
            0,
            None,
            data,
            startIndex,
            elementCount,
        )
    }

    pub fn GetDataWithLevelAndRectAndDataAndStartIndexAndElementCount<T: Copy>(
        &self,
        level: i32,
        rect: Option<Rectangle>,
        data: &mut [T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        let data_type = texture_data_type::<T>()?;
        let transfer =
            self.transfer(level, rect, data_type, data.len(), startIndex, elementCount)?;
        let mut required = 0;
        self.state.device().state.native().get_texture_data(
            self.state.require_handle()?,
            data_type,
            &transfer,
            data,
            &mut required,
        )
    }

    pub fn SaveAsPng<W: Write>(&self, stream: &mut W, width: i32, height: i32) -> Result<()> {
        self.save(stream, width, height, sys::CNA_TEXTURE_IMAGE_FORMAT_PNG)
    }

    pub fn SaveAsJpeg<W: Write>(&self, stream: &mut W, width: i32, height: i32) -> Result<()> {
        self.save(stream, width, height, sys::CNA_TEXTURE_IMAGE_FORMAT_JPEG)
    }

    fn transfer(
        &self,
        level: i32,
        rect: Option<Rectangle>,
        data_type: sys::CNA_TextureDataType,
        capacity: usize,
        start_index: i32,
        element_count: i32,
    ) -> Result<sys::CNA_Texture2DTransfer> {
        if level < 0 || level >= self.level_count {
            return Err(CnaError::InvalidInput(
                "texture mip level is outside the allocated chain",
            ));
        }
        if start_index < 0 || element_count < 0 {
            return Err(CnaError::InvalidInput(
                "texture array window must not be negative",
            ));
        }
        let end = usize::try_from(start_index)
            .ok()
            .and_then(|start| {
                usize::try_from(element_count)
                    .ok()
                    .and_then(|count| start.checked_add(count))
            })
            .ok_or(CnaError::InvalidInput("texture array window overflows"))?;
        if end > capacity {
            return Err(CnaError::InvalidInput(
                "texture array window exceeds the supplied data",
            ));
        }

        let level_shift = u32::try_from(level)
            .map_err(|_| CnaError::InvalidInput("texture mip level is invalid"))?;
        let level_width = self.width.checked_shr(level_shift).unwrap_or(0).max(1);
        let level_height = self.height.checked_shr(level_shift).unwrap_or(0).max(1);
        let region = rect.unwrap_or(Rectangle::new(0, 0, level_width, level_height));
        if region.X < 0
            || region.Y < 0
            || region.Width <= 0
            || region.Height <= 0
            || region.X > level_width - region.Width
            || region.Y > level_height - region.Height
        {
            return Err(CnaError::InvalidInput(
                "texture rectangle is outside the selected mip level",
            ));
        }
        let required = if data_type == sys::CNA_TEXTURE_DATA_BYTE
            && matches!(
                self.format,
                SurfaceFormat::Dxt1 | SurfaceFormat::Dxt3 | SurfaceFormat::Dxt5
            ) {
            let block_bytes = if self.format == SurfaceFormat::Dxt1 {
                8
            } else {
                16
            };
            i64::from((region.Width + 3) / 4) * i64::from((region.Height + 3) / 4) * block_bytes
        } else {
            i64::from(region.Width) * i64::from(region.Height)
        };
        if i64::from(element_count) < required {
            return Err(CnaError::InvalidInput(
                "texture element count is smaller than the selected region",
            ));
        }

        Ok(sys::CNA_Texture2DTransfer {
            struct_size: size_of::<sys::CNA_Texture2DTransfer>() as u32,
            struct_version: 1,
            level,
            has_rectangle: if rect.is_some() {
                sys::CNA_TRUE
            } else {
                sys::CNA_FALSE
            },
            reserved: [0; 3],
            rectangle: sys::CNA_Rectangle {
                x: region.X,
                y: region.Y,
                width: region.Width,
                height: region.Height,
            },
            start_index: u64::try_from(start_index)
                .map_err(|_| CnaError::InvalidInput("texture start index is invalid"))?,
            element_count: u64::try_from(element_count)
                .map_err(|_| CnaError::InvalidInput("texture element count is invalid"))?,
        })
    }

    fn save<W: Write>(
        &self,
        stream: &mut W,
        width: i32,
        height: i32,
        format: sys::CNA_TextureImageFormat,
    ) -> Result<()> {
        if width <= 0 || height <= 0 {
            return Err(CnaError::InvalidInput(
                "encoded texture dimensions must be greater than zero",
            ));
        }
        let native_width = u32::try_from(width)
            .map_err(|_| CnaError::InvalidInput("encoded texture width is invalid"))?;
        let native_height = u32::try_from(height)
            .map_err(|_| CnaError::InvalidInput("encoded texture height is invalid"))?;
        let handle = self.state.require_handle()?;
        let native = self.state.device().state.native();
        let mut size = 0;
        native.encoded_texture_size(handle, format, native_width, native_height, &mut size)?;
        let capacity = usize::try_from(size)
            .map_err(|_| CnaError::InvalidInput("encoded texture is too large"))?;
        let mut bytes = vec![0; capacity];
        let mut copied = 0;
        native.copy_encoded_texture(
            handle,
            format,
            native_width,
            native_height,
            &mut bytes,
            &mut copied,
        )?;
        let copied = usize::try_from(copied)
            .map_err(|_| CnaError::InvalidInput("encoded texture is too large"))?;
        if copied > bytes.len() {
            return Err(CnaError::InvalidInput(
                "native encoded texture size exceeded its reported capacity",
            ));
        }
        stream
            .write_all(&bytes[..copied])
            .map_err(|error| CnaError::Native {
                code: sys::CNA_RESULT_IO,
                category: ErrorCategory::None,
                message: error.to_string(),
            })
    }

    pub fn Dispose(&mut self, value: bool) -> Result<()> {
        self.state.dispose_with_event(self, value)
    }
}

fn texture_data_type<T: Copy>() -> Result<sys::CNA_TextureDataType> {
    let name = std::any::type_name::<T>();
    let (data_type, expected_size) = match name {
        "cna::value::color::Color" if cfg!(target_endian = "little") => {
            (sys::CNA_TEXTURE_DATA_COLOR, 4)
        }
        "cna::packed::Bgr565" => (sys::CNA_TEXTURE_DATA_BGR565, 2),
        "cna::packed::Bgra5551" => (sys::CNA_TEXTURE_DATA_BGRA5551, 2),
        "cna::packed::Bgra4444" => (sys::CNA_TEXTURE_DATA_BGRA4444, 2),
        "u8" => (sys::CNA_TEXTURE_DATA_BYTE, 1),
        "cna::packed::NormalizedByte2" => (sys::CNA_TEXTURE_DATA_NORMALIZED_BYTE2, 2),
        "cna::packed::NormalizedByte4" => (sys::CNA_TEXTURE_DATA_NORMALIZED_BYTE4, 4),
        "cna::packed::Rgba1010102" => (sys::CNA_TEXTURE_DATA_RGBA1010102, 4),
        "cna::packed::Rg32" => (sys::CNA_TEXTURE_DATA_RG32, 4),
        "cna::packed::Rgba64" => (sys::CNA_TEXTURE_DATA_RGBA64, 8),
        "cna::packed::Alpha8" => (sys::CNA_TEXTURE_DATA_ALPHA8, 1),
        "f32" => (sys::CNA_TEXTURE_DATA_SINGLE, 4),
        "cna::value::vector2::Vector2" => (sys::CNA_TEXTURE_DATA_VECTOR2, 8),
        "cna::value::vector4::Vector4" => (sys::CNA_TEXTURE_DATA_VECTOR4, 16),
        "cna::packed::HalfSingle" => (sys::CNA_TEXTURE_DATA_HALF_SINGLE, 2),
        "cna::packed::HalfVector2" => (sys::CNA_TEXTURE_DATA_HALF_VECTOR2, 4),
        "cna::packed::HalfVector4" => (sys::CNA_TEXTURE_DATA_HALF_VECTOR4, 8),
        "u16" => (sys::CNA_TEXTURE_DATA_USHORT, 2),
        _ => {
            return Err(CnaError::InvalidInput(
                "texture data element type has no exact CNA transfer representation",
            ))
        }
    };
    if size_of::<T>() != expected_size {
        return Err(CnaError::InvalidInput(
            "texture data element layout does not match the CNA transfer representation",
        ));
    }
    Ok(data_type)
}

impl Texture for Texture2D {
    fn Format(&self) -> crate::Microsoft::Xna::Framework::Graphics::SurfaceFormat {
        self.format
    }

    fn LevelCount(&self) -> i32 {
        self.level_count
    }
}

impl Texture2DBase for Texture2D {}

impl TextureRuntime for Texture2D {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn bind_texture_slot(
        &self,
        device: &GraphicsDevice,
        vertex_stage: bool,
        index: u32,
    ) -> Result<()> {
        if !self.state.device().is_same_device(device) {
            return Err(CnaError::InvalidInput(
                "texture belongs to a different graphics device",
            ));
        }
        let texture = self.state.require_handle()?;
        device.state.native().set_texture_slot(
            device.handle()?,
            if vertex_stage {
                sys::CNA_SHADER_STAGE_VERTEX
            } else {
                sys::CNA_SHADER_STAGE_PIXEL
            },
            index,
            texture,
        )
    }
}

impl ContentDisposable for Texture2D {
    fn DisposeContent(&self) -> Result<()> {
        self.state.dispose_native()
    }
}

impl ContentLoadable for Texture2D {
    fn ContentDisposable(value: &Arc<Self>) -> Option<Arc<dyn ContentDisposable>> {
        Some(Arc::clone(value) as Arc<dyn ContentDisposable>)
    }
}

impl GraphicsResource for Texture2D {
    fn GraphicsDevice(&self) -> Option<&GraphicsDevice> {
        Some(self.state.device())
    }

    fn IsDisposed(&self) -> bool {
        self.state.handle().is_none()
    }

    fn Name(&self) -> String {
        self.state.name()
    }

    fn SetName(&mut self, value: &str) {
        self.state.set_name(value);
    }

    fn Tag(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.state.tag()
    }

    fn SetTag(&mut self, value: Option<Arc<dyn Any + Send + Sync>>) {
        self.state.set_tag(value);
    }

    fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.add_disposing_handler(handler)
    }

    fn RemoveDisposingHandler(&self, registration: u64) -> bool {
        self.state.remove_disposing_handler(registration)
    }

    fn Dispose(&mut self, value: bool) -> Result<()> {
        Self::Dispose(self, value)
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        let _ = self.state.dispose_native();
    }
}
