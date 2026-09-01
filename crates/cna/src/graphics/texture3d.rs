#![allow(non_snake_case, clippy::missing_errors_doc)]

use core::mem::size_of;
use std::any::Any;
use std::sync::Arc;

use cna_sys as sys;

use crate::content::{ContentDisposable, ContentLoadable};
use crate::error::{CnaError, Result};
use crate::extensions::events::EventHandler;
use crate::value::Color;

use super::resource::{ResourceKind, ResourceState};
use super::texture_cube::{color_to_native, native_to_color};
use super::{GraphicsDevice, GraphicsResource, SurfaceFormat, Texture, TextureRuntime};

mod volume_texture_data_sealed {
    pub trait Sealed {}
}

/// Safe element contract for the exact ABI-0.20 `Texture3D` transfer route.
pub trait Texture3DData: volume_texture_data_sealed::Sealed + Copy + Send + Sync + 'static {
    #[doc(hidden)]
    fn to_color(self) -> Color;
    #[doc(hidden)]
    fn from_color(value: Color) -> Self;
}

impl volume_texture_data_sealed::Sealed for Color {}

impl Texture3DData for Color {
    fn to_color(self) -> Color {
        self
    }

    fn from_color(value: Color) -> Self {
        value
    }
}

/// Owned native XNA volume texture.
pub struct Texture3D {
    state: Arc<ResourceState>,
    width: i32,
    height: i32,
    depth: i32,
    level_count: i32,
    format: SurfaceFormat,
}

impl Texture3D {
    pub fn new(
        graphicsDevice: &GraphicsDevice,
        width: i32,
        height: i32,
        depth: i32,
        mipMap: bool,
        format: SurfaceFormat,
    ) -> Result<Self> {
        if width <= 0 || height <= 0 || depth <= 0 {
            return Err(CnaError::InvalidInput(
                "volume texture dimensions must be greater than zero",
            ));
        }
        let info = sys::CNA_Texture3DCreateInfo {
            struct_size: u32::try_from(size_of::<sys::CNA_Texture3DCreateInfo>())
                .map_err(|_| CnaError::InvalidInput("Texture3D create-info size exceeds u32"))?,
            struct_version: 1,
            width: u32::try_from(width)
                .map_err(|_| CnaError::InvalidInput("volume texture width is invalid"))?,
            height: u32::try_from(height)
                .map_err(|_| CnaError::InvalidInput("volume texture height is invalid"))?,
            depth: u32::try_from(depth)
                .map_err(|_| CnaError::InvalidInput("volume texture depth is invalid"))?,
            mip_map: u8::from(mipMap),
            reserved0: [0; 3],
            format: format as u32,
            reserved1: 0,
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        graphicsDevice.state.native().create_texture3d(
            graphicsDevice.handle()?,
            &info,
            &mut handle,
        )?;
        match Self::from_handle(graphicsDevice, handle) {
            Ok(value) => Ok(value),
            Err(error) => {
                let _ = graphicsDevice.state.native().destroy_texture3d(handle);
                Err(error)
            }
        }
    }

    /// Adopts a volume texture CNA created and handed over outright.
    ///
    /// `cna_cube_lut_create_volume_texture` is the caller: the engine allocates
    /// the texture and the caller destroys it. Destroys the handle on failure
    /// so a refused wrap never strands one.
    pub(crate) fn from_owned_handle(
        graphics_device: &GraphicsDevice,
        handle: sys::CNA_Handle,
    ) -> Result<Self> {
        match Self::from_handle(graphics_device, handle) {
            Ok(texture) => Ok(texture),
            Err(error) => {
                let _ = graphics_device.state.native().destroy_texture3d(handle);
                Err(error)
            }
        }
    }

    fn from_handle(graphics_device: &GraphicsDevice, handle: sys::CNA_Handle) -> Result<Self> {
        let mut info = sys::CNA_Texture3DInfo {
            struct_size: u32::try_from(size_of::<sys::CNA_Texture3DInfo>())
                .map_err(|_| CnaError::InvalidInput("Texture3D info size exceeds u32"))?,
            struct_version: 1,
            ..sys::CNA_Texture3DInfo::default()
        };
        graphics_device
            .state
            .native()
            .texture3d_info(handle, &mut info)?;
        Ok(Self {
            state: ResourceState::new(graphics_device, handle, ResourceKind::Texture3D),
            width: i32::try_from(info.width)
                .map_err(|_| CnaError::InvalidInput("volume texture width exceeds i32"))?,
            height: i32::try_from(info.height)
                .map_err(|_| CnaError::InvalidInput("volume texture height exceeds i32"))?,
            depth: i32::try_from(info.depth)
                .map_err(|_| CnaError::InvalidInput("volume texture depth exceeds i32"))?,
            level_count: i32::try_from(info.level_count)
                .map_err(|_| CnaError::InvalidInput("volume texture level count exceeds i32"))?,
            format: SurfaceFormat::from_native(info.format).ok_or(CnaError::InvalidInput(
                "native volume texture format is unknown",
            ))?,
        })
    }

    /// The native handle, for an engine route that borrows this texture.
    pub(crate) fn native_handle(&self) -> Result<sys::CNA_Handle> {
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
    pub const fn Depth(&self) -> i32 {
        self.depth
    }

    pub fn SetData<T: Texture3DData>(&self, data: &[T]) -> Result<()> {
        let count = i32::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("volume texture source is too large"))?;
        self.SetDataWithDataAndStartIndexAndElementCount(data, 0, count)
    }

    pub fn SetDataWithDataAndStartIndexAndElementCount<T: Texture3DData>(
        &self,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        self.SetDataWithLevelAndLeftAndTopAndRightAndBottomAndFrontAndBackAndDataAndStartIndexAndElementCount(
            0, 0, 0, self.width, self.height, 0, self.depth, data, startIndex, elementCount,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn SetDataWithLevelAndLeftAndTopAndRightAndBottomAndFrontAndBackAndDataAndStartIndexAndElementCount<
        T: Texture3DData,
    >(
        &self,
        level: i32,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        front: i32,
        back: i32,
        data: &[T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        let transfer = self.transfer(
            level,
            left,
            top,
            right,
            bottom,
            front,
            back,
            data.len(),
            startIndex,
            elementCount,
        )?;
        let colors = data
            .iter()
            .map(|value| color_to_native(value.to_color()))
            .collect::<Vec<_>>();
        self.state.device().state.native().set_texture3d_data(
            self.state.require_handle()?,
            &transfer,
            &colors,
        )
    }

    pub fn GetData<T: Texture3DData>(&self, data: &mut [T]) -> Result<()> {
        let count = i32::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("volume texture destination is too large"))?;
        self.GetDataWithDataAndStartIndexAndElementCount(data, 0, count)
    }

    pub fn GetDataWithDataAndStartIndexAndElementCount<T: Texture3DData>(
        &self,
        data: &mut [T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        self.GetDataWithLevelAndLeftAndTopAndRightAndBottomAndFrontAndBackAndDataAndStartIndexAndElementCount(
            0, 0, 0, self.width, self.height, 0, self.depth, data, startIndex, elementCount,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn GetDataWithLevelAndLeftAndTopAndRightAndBottomAndFrontAndBackAndDataAndStartIndexAndElementCount<
        T: Texture3DData,
    >(
        &self,
        level: i32,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        front: i32,
        back: i32,
        data: &mut [T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        let transfer = self.transfer(
            level,
            left,
            top,
            right,
            bottom,
            front,
            back,
            data.len(),
            startIndex,
            elementCount,
        )?;
        let mut colors = data
            .iter()
            .map(|value| color_to_native(value.to_color()))
            .collect::<Vec<_>>();
        let mut required = 0;
        self.state.device().state.native().get_texture3d_data(
            self.state.require_handle()?,
            &transfer,
            &mut colors,
            &mut required,
        )?;
        let start = usize::try_from(startIndex)
            .map_err(|_| CnaError::InvalidInput("start index must not be negative"))?;
        let end = start
            .checked_add(
                usize::try_from(elementCount)
                    .map_err(|_| CnaError::InvalidInput("element count must not be negative"))?,
            )
            .ok_or(CnaError::InvalidInput(
                "volume texture array window overflows",
            ))?;
        for (destination, value) in data[start..end].iter_mut().zip(&colors[start..end]) {
            *destination = T::from_color(native_to_color(*value));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn transfer(
        &self,
        level: i32,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        front: i32,
        back: i32,
        capacity: usize,
        start_index: i32,
        element_count: i32,
    ) -> Result<sys::CNA_Texture3DTransfer> {
        self.state.require_handle()?;
        if level < 0 || level >= self.level_count {
            return Err(CnaError::InvalidInput(
                "volume texture mip level is outside the allocated chain",
            ));
        }
        let start = usize::try_from(start_index)
            .map_err(|_| CnaError::InvalidInput("start index must not be negative"))?;
        let count = usize::try_from(element_count)
            .map_err(|_| CnaError::InvalidInput("element count must not be negative"))?;
        if start.checked_add(count).ok_or(CnaError::InvalidInput(
            "volume texture array window overflows",
        ))? > capacity
        {
            return Err(CnaError::InvalidInput(
                "volume texture array window exceeds the supplied data",
            ));
        }
        let shift = u32::try_from(level)
            .map_err(|_| CnaError::InvalidInput("volume texture mip level is negative"))?;
        let width = self.width.checked_shr(shift).unwrap_or(0).max(1);
        let height = self.height.checked_shr(shift).unwrap_or(0).max(1);
        let depth = self.depth.checked_shr(shift).unwrap_or(0).max(1);
        if left < 0
            || top < 0
            || front < 0
            || right <= left
            || bottom <= top
            || back <= front
            || right > width
            || bottom > height
            || back > depth
        {
            return Err(CnaError::InvalidInput(
                "volume texture box is outside the selected mip level",
            ));
        }
        let required = i64::from(right - left)
            .checked_mul(i64::from(bottom - top))
            .and_then(|value| value.checked_mul(i64::from(back - front)))
            .ok_or(CnaError::InvalidInput("volume texture box size overflows"))?;
        if i64::from(element_count) < required {
            return Err(CnaError::InvalidInput(
                "volume texture element count is smaller than the selected box",
            ));
        }
        Ok(sys::CNA_Texture3DTransfer {
            struct_size: u32::try_from(size_of::<sys::CNA_Texture3DTransfer>())
                .map_err(|_| CnaError::InvalidInput("Texture3D transfer size exceeds u32"))?,
            struct_version: 1,
            level,
            left,
            top,
            right,
            bottom,
            front,
            back,
            reserved: 0,
            start_index: u64::try_from(start)
                .map_err(|_| CnaError::InvalidInput("start index exceeds u64"))?,
            element_count: u64::try_from(count)
                .map_err(|_| CnaError::InvalidInput("element count exceeds u64"))?,
        })
    }

    pub fn Dispose(&mut self, value: bool) -> Result<()> {
        self.state.dispose_with_event(self, value)
    }
}

impl Texture for Texture3D {
    fn Format(&self) -> SurfaceFormat {
        self.format
    }

    fn LevelCount(&self) -> i32 {
        self.level_count
    }
}

impl TextureRuntime for Texture3D {
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

impl GraphicsResource for Texture3D {
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

impl Drop for Texture3D {
    fn drop(&mut self) {}
}

impl ContentDisposable for Texture3D {
    fn DisposeContent(&self) -> Result<()> {
        self.state.dispose_native()
    }
}

impl ContentLoadable for Texture3D {
    fn ContentDisposable(value: &Arc<Self>) -> Option<Arc<dyn ContentDisposable>> {
        Some(Arc::clone(value) as Arc<dyn ContentDisposable>)
    }
}


impl crate::extensions::graphics_resource::HasResourceState for Texture3D {
    fn resource_state(&self) -> &super::resource::ResourceState {
        &self.state
    }
}
