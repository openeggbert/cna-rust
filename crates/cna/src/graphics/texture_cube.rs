#![allow(non_snake_case, clippy::missing_errors_doc)]

use core::mem::size_of;
use std::any::Any;
use std::sync::Arc;

use cna_sys as sys;

use crate::content::{ContentDisposable, ContentLoadable};
use crate::error::{CnaError, Result};
use crate::extensions::events::EventHandler;
use crate::value::{Color, Rectangle};

use super::resource::{ResourceKind, ResourceState};
use super::{
    CubeMapFace, GraphicsDevice, GraphicsResource, SurfaceFormat, Texture, TextureRuntime,
};

mod cube_texture_data_sealed {
    pub trait Sealed {}
}

/// Safe element contract for ABI-0.20 cube-texture transfers.
///
/// CNA currently exposes only exact `Color` texels for this resource family.
pub trait CubeTextureData: cube_texture_data_sealed::Sealed + Copy + Send + Sync + 'static {
    #[doc(hidden)]
    fn to_color(self) -> Color;
    #[doc(hidden)]
    fn from_color(value: Color) -> Self;
}

impl cube_texture_data_sealed::Sealed for Color {}

impl CubeTextureData for Color {
    fn to_color(self) -> Color {
        self
    }

    fn from_color(value: Color) -> Self {
        value
    }
}

/// Composition marker for XNA types inheriting `TextureCube`.
pub trait TextureCubeBase: Texture {}

/// Owned native XNA cube texture.
pub struct TextureCube {
    state: Arc<ResourceState>,
    size: i32,
    level_count: i32,
    format: SurfaceFormat,
}

#[allow(non_snake_case)]
impl TextureCube {
    pub fn new(
        graphicsDevice: &GraphicsDevice,
        size: i32,
        mipMap: bool,
        format: SurfaceFormat,
    ) -> Result<Self> {
        if size <= 0 {
            return Err(CnaError::InvalidInput(
                "cube texture size must be greater than zero",
            ));
        }
        let info = sys::CNA_TextureCubeCreateInfo {
            struct_size: size_of::<sys::CNA_TextureCubeCreateInfo>() as u32,
            struct_version: 1,
            size: u32::try_from(size)
                .map_err(|_| CnaError::InvalidInput("cube texture size is invalid"))?,
            mip_map: u8::from(mipMap),
            reserved0: [0; 3],
            format: format as u32,
            reserved1: 0,
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        graphicsDevice.state.native().create_texture_cube(
            graphicsDevice.handle()?,
            &info,
            &mut handle,
        )?;
        match Self::from_handle(graphicsDevice, handle, ResourceKind::TextureCube) {
            Ok(value) => Ok(value),
            Err(error) => {
                let _ = graphicsDevice.state.native().destroy_texture_cube(handle);
                Err(error)
            }
        }
    }

    pub(super) fn from_handle(
        graphics_device: &GraphicsDevice,
        handle: sys::CNA_Handle,
        kind: ResourceKind,
    ) -> Result<Self> {
        let mut info = sys::CNA_TextureCubeInfo {
            struct_size: size_of::<sys::CNA_TextureCubeInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_TextureCubeInfo::default()
        };
        graphics_device
            .state
            .native()
            .texture_cube_info(handle, &mut info)?;
        Ok(Self {
            state: ResourceState::new(graphics_device, handle, kind),
            size: i32::try_from(info.size)
                .map_err(|_| CnaError::InvalidInput("cube texture size exceeds i32"))?,
            level_count: i32::try_from(info.level_count)
                .map_err(|_| CnaError::InvalidInput("cube texture level count exceeds i32"))?,
            format: SurfaceFormat::from_native(info.format).ok_or(CnaError::InvalidInput(
                "native cube texture format is unknown",
            ))?,
        })
    }

    #[must_use]
    pub const fn Size(&self) -> i32 {
        self.size
    }

    pub fn SetData<T: CubeTextureData>(&self, cubeMapFace: CubeMapFace, data: &[T]) -> Result<()> {
        let count = i32::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("cube texture source is too large"))?;
        self.SetDataWithCubeMapFaceAndDataAndStartIndexAndElementCount(cubeMapFace, data, 0, count)
    }

    pub fn SetDataWithCubeMapFaceAndDataAndStartIndexAndElementCount<T: CubeTextureData>(
        &self,
        cubeMapFace: CubeMapFace,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        self.SetDataWithCubeMapFaceAndLevelAndRectAndDataAndStartIndexAndElementCount(
            cubeMapFace,
            0,
            None,
            data,
            startIndex,
            elementCount,
        )
    }

    pub fn SetDataWithCubeMapFaceAndLevelAndRectAndDataAndStartIndexAndElementCount<
        T: CubeTextureData,
    >(
        &self,
        cubeMapFace: CubeMapFace,
        level: i32,
        rect: Option<Rectangle>,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        let transfer = self.transfer(
            cubeMapFace,
            level,
            rect,
            data.len(),
            startIndex,
            elementCount,
        )?;
        let colors = data
            .iter()
            .map(|value| color_to_native(value.to_color()))
            .collect::<Vec<_>>();
        self.state.device().state.native().set_texture_cube_data(
            self.state.require_handle()?,
            &transfer,
            &colors,
        )
    }

    pub fn GetData<T: CubeTextureData>(
        &self,
        cubeMapFace: CubeMapFace,
        data: &mut [T],
    ) -> Result<()> {
        let count = i32::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("cube texture destination is too large"))?;
        self.GetDataWithCubeMapFaceAndDataAndStartIndexAndElementCount(cubeMapFace, data, 0, count)
    }

    pub fn GetDataWithCubeMapFaceAndDataAndStartIndexAndElementCount<T: CubeTextureData>(
        &self,
        cubeMapFace: CubeMapFace,
        data: &mut [T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        self.GetDataWithCubeMapFaceAndLevelAndRectAndDataAndStartIndexAndElementCount(
            cubeMapFace,
            0,
            None,
            data,
            startIndex,
            elementCount,
        )
    }

    pub fn GetDataWithCubeMapFaceAndLevelAndRectAndDataAndStartIndexAndElementCount<
        T: CubeTextureData,
    >(
        &self,
        cubeMapFace: CubeMapFace,
        level: i32,
        rect: Option<Rectangle>,
        data: &mut [T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        let transfer = self.transfer(
            cubeMapFace,
            level,
            rect,
            data.len(),
            startIndex,
            elementCount,
        )?;
        let mut colors = data
            .iter()
            .map(|value| color_to_native(value.to_color()))
            .collect::<Vec<_>>();
        let mut required = 0;
        self.state.device().state.native().get_texture_cube_data(
            self.state.require_handle()?,
            &transfer,
            &mut colors,
            &mut required,
        )?;
        let start = startIndex as usize;
        let end = start + elementCount as usize;
        for (destination, value) in data[start..end].iter_mut().zip(&colors[start..end]) {
            *destination = T::from_color(native_to_color(*value));
        }
        Ok(())
    }

    fn transfer(
        &self,
        face: CubeMapFace,
        level: i32,
        rect: Option<Rectangle>,
        capacity: usize,
        start_index: i32,
        element_count: i32,
    ) -> Result<sys::CNA_TextureCubeTransfer> {
        if level < 0 || level >= self.level_count {
            return Err(CnaError::InvalidInput(
                "cube texture mip level is outside the allocated chain",
            ));
        }
        let start = usize::try_from(start_index)
            .map_err(|_| CnaError::InvalidInput("start index must not be negative"))?;
        let count = usize::try_from(element_count)
            .map_err(|_| CnaError::InvalidInput("element count must not be negative"))?;
        if start.checked_add(count).ok_or(CnaError::InvalidInput(
            "cube texture array window overflows",
        ))? > capacity
        {
            return Err(CnaError::InvalidInput(
                "cube texture array window exceeds the supplied data",
            ));
        }
        let level_size = self.size.checked_shr(level as u32).unwrap_or(0).max(1);
        let region = rect.unwrap_or(Rectangle::new(0, 0, level_size, level_size));
        if region.X < 0
            || region.Y < 0
            || region.Width <= 0
            || region.Height <= 0
            || region.X > level_size - region.Width
            || region.Y > level_size - region.Height
        {
            return Err(CnaError::InvalidInput(
                "cube texture rectangle is outside the selected mip level",
            ));
        }
        let required = i64::from(region.Width) * i64::from(region.Height);
        if i64::from(element_count) < required {
            return Err(CnaError::InvalidInput(
                "cube texture element count is smaller than the selected region",
            ));
        }
        Ok(sys::CNA_TextureCubeTransfer {
            struct_size: size_of::<sys::CNA_TextureCubeTransfer>() as u32,
            struct_version: 1,
            face: face as u32,
            level,
            has_rectangle: u8::from(rect.is_some()),
            reserved0: [0; 3],
            rectangle: sys::CNA_Rectangle {
                x: region.X,
                y: region.Y,
                width: region.Width,
                height: region.Height,
            },
            reserved1: 0,
            start_index: start as u64,
            element_count: count as u64,
        })
    }

    pub fn Dispose(&mut self, value: bool) -> Result<()> {
        self.state.dispose_with_event(self, value)
    }
}

impl TextureCube {
    /// The native handle, for a route that is about to take ownership of it.
    ///
    /// Paired with [`TextureCube::relinquish`]: read the handle, call the
    /// consuming route, and only relinquish when it succeeded.
    pub(crate) fn native_handle(&self) -> Result<sys::CNA_Handle> {
        self.state.require_handle()
    }

    /// Forgets the handle after a consuming route has taken it.
    pub(crate) fn relinquish(&self) {
        self.state.relinquish();
    }

    /// Adopts a cube map CNA created and handed over outright.
    ///
    /// The engine layer's environment processor publishes owned cubes -- the
    /// equirectangular conversion, the irradiance convolution and the
    /// prefiltered specular chain -- and the caller destroys each. This is that
    /// adoption, and it destroys the handle on failure so a refused wrap never
    /// strands one.
    pub(crate) fn from_owned_handle(
        graphics_device: &GraphicsDevice,
        handle: sys::CNA_Handle,
    ) -> Result<Self> {
        match Self::from_handle(graphics_device, handle, ResourceKind::TextureCube) {
            Ok(value) => Ok(value),
            Err(error) => {
                let _ = graphics_device.state.native().destroy_texture_cube(handle);
                Err(error)
            }
        }
    }
}

pub(super) fn color_to_native(value: Color) -> sys::CNA_Color {
    sys::CNA_Color {
        r: value.R(),
        g: value.G(),
        b: value.B(),
        a: value.A(),
    }
}

pub(super) fn native_to_color(value: sys::CNA_Color) -> Color {
    Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
        i32::from(value.r),
        i32::from(value.g),
        i32::from(value.b),
        i32::from(value.a),
    )
}

impl Texture for TextureCube {
    fn Format(&self) -> SurfaceFormat {
        self.format
    }

    fn LevelCount(&self) -> i32 {
        self.level_count
    }
}

impl TextureCubeBase for TextureCube {}

impl TextureRuntime for TextureCube {
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
        device.state.native().set_texture_slot(
            device.handle()?,
            if vertex_stage {
                sys::CNA_SHADER_STAGE_VERTEX
            } else {
                sys::CNA_SHADER_STAGE_PIXEL
            },
            index,
            self.state.require_handle()?,
        )
    }
}

impl TextureCube {
    pub(super) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.state.require_handle()
    }
}

impl GraphicsResource for TextureCube {
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

impl Drop for TextureCube {
    fn drop(&mut self) {}
}

impl ContentDisposable for TextureCube {
    fn DisposeContent(&self) -> Result<()> {
        self.state.dispose_native()
    }
}

impl ContentLoadable for TextureCube {
    fn ContentDisposable(value: &Arc<Self>) -> Option<Arc<dyn ContentDisposable>> {
        Some(Arc::clone(value) as Arc<dyn ContentDisposable>)
    }
}
