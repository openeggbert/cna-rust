#![allow(non_snake_case, clippy::missing_errors_doc, clippy::too_many_arguments)]

use core::mem::size_of;
use std::any::Any;
use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::{EventArgs, EventHandler};

use super::resource::{EventHandlers, ResourceKind, ResourceState};
use super::{
    CubeMapFace, DepthFormat, GraphicsDevice, GraphicsResource, RenderTargetUsage, SurfaceFormat,
    Texture, Texture2DBase, TextureCubeBase, TextureRuntime,
};

/// Owned XNA two-dimensional render target.
pub struct RenderTarget2D {
    state: Arc<ResourceState>,
    width: i32,
    height: i32,
    level_count: i32,
    format: SurfaceFormat,
    depth_format: DepthFormat,
    multi_sample_count: i32,
    usage: RenderTargetUsage,
    is_content_lost: bool,
    renderer_available: bool,
    content_lost: Arc<EventHandlers<EventArgs>>,
}

#[allow(non_snake_case)]
impl RenderTarget2D {
    pub fn new(graphicsDevice: &GraphicsDevice, width: i32, height: i32) -> Result<Self> {
        Self::create(
            graphicsDevice,
            width,
            height,
            false,
            SurfaceFormat::Color,
            DepthFormat::None,
            0,
            RenderTargetUsage::DiscardContents,
        )
    }

    pub fn from_graphics_device_and_width_and_height_and_mip_map_and_preferred_format_and_preferred_depth_format_and_preferred_multi_sample_count_and_usage(
        graphicsDevice: &GraphicsDevice,
        width: i32,
        height: i32,
        mipMap: bool,
        preferredFormat: SurfaceFormat,
        preferredDepthFormat: DepthFormat,
        preferredMultiSampleCount: i32,
        usage: RenderTargetUsage,
    ) -> Result<Self> {
        Self::create(
            graphicsDevice,
            width,
            height,
            mipMap,
            preferredFormat,
            preferredDepthFormat,
            preferredMultiSampleCount,
            usage,
        )
    }

    fn create(
        graphicsDevice: &GraphicsDevice,
        width: i32,
        height: i32,
        mipMap: bool,
        preferredFormat: SurfaceFormat,
        preferredDepthFormat: DepthFormat,
        preferredMultiSampleCount: i32,
        usage: RenderTargetUsage,
    ) -> Result<Self> {
        if width <= 0 || height <= 0 || preferredMultiSampleCount < 0 {
            return Err(CnaError::InvalidInput(
                "render-target dimensions must be positive and multisampling nonnegative",
            ));
        }
        let info = sys::CNA_RenderTarget2DCreateInfo {
            struct_size: size_of::<sys::CNA_RenderTarget2DCreateInfo>() as u32,
            struct_version: 1,
            width: width as u32,
            height: height as u32,
            mip_map: u8::from(mipMap),
            reserved0: [0; 3],
            format: preferredFormat as u32,
            depth_format: preferredDepthFormat as u32,
            multi_sample_count: preferredMultiSampleCount,
            usage: usage as u32,
            reserved1: 0,
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        graphicsDevice.state.native().create_render_target2d(
            graphicsDevice.handle()?,
            &info,
            &mut handle,
        )?;
        match Self::from_handle(graphicsDevice, handle) {
            Ok(value) => Ok(value),
            Err(error) => {
                let _ = graphicsDevice.state.native().destroy_render_target(handle);
                Err(error)
            }
        }
    }

    pub fn from_graphics_device_and_width_and_height_and_mip_map_and_preferred_format_and_preferred_depth_format(
        graphicsDevice: &GraphicsDevice,
        width: i32,
        height: i32,
        mipMap: bool,
        preferredFormat: SurfaceFormat,
        preferredDepthFormat: DepthFormat,
    ) -> Result<Self> {
        Self::create(
            graphicsDevice,
            width,
            height,
            mipMap,
            preferredFormat,
            preferredDepthFormat,
            0,
            RenderTargetUsage::DiscardContents,
        )
    }

    fn from_handle(device: &GraphicsDevice, handle: sys::CNA_Handle) -> Result<Self> {
        let info = target_info(device, handle, sys::CNA_RENDER_TARGET_KIND_2D)?;
        Ok(Self {
            state: ResourceState::new(device, handle, ResourceKind::RenderTarget2D),
            width: info.width as i32,
            height: info.height as i32,
            level_count: info.level_count as i32,
            format: surface_format(info.format)?,
            depth_format: depth_format(info.depth_format)?,
            multi_sample_count: info.multi_sample_count,
            usage: render_target_usage(info.usage)?,
            is_content_lost: info.is_content_lost != sys::CNA_FALSE,
            renderer_available: info.renderer_available != sys::CNA_FALSE,
            content_lost: Arc::new(EventHandlers::new()),
        })
    }

    fn retained(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            width: self.width,
            height: self.height,
            level_count: self.level_count,
            format: self.format,
            depth_format: self.depth_format,
            multi_sample_count: self.multi_sample_count,
            usage: self.usage,
            is_content_lost: self.is_content_lost,
            renderer_available: self.renderer_available,
            content_lost: Arc::clone(&self.content_lost),
        }
    }

    #[must_use]
    pub const fn DepthStencilFormat(&self) -> DepthFormat {
        self.depth_format
    }
    #[must_use]
    pub const fn MultiSampleCount(&self) -> i32 {
        self.multi_sample_count
    }
    #[must_use]
    pub const fn RenderTargetUsage(&self) -> RenderTargetUsage {
        self.usage
    }
    #[must_use]
    pub const fn IsContentLost(&self) -> bool {
        self.is_content_lost
    }
    pub fn AddContentLostHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.content_lost.add(handler)
    }
    pub fn RemoveContentLostHandler(&self, registration: u64) -> bool {
        self.content_lost.remove(registration)
    }
    pub fn Dispose(&mut self, value: bool) -> Result<()> {
        self.state.dispose_with_event(self, value)
    }

    pub(super) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.state.require_handle()
    }
}

/// Owned XNA cube-map render target.
pub struct RenderTargetCube {
    state: Arc<ResourceState>,
    size: i32,
    level_count: i32,
    format: SurfaceFormat,
    depth_format: DepthFormat,
    multi_sample_count: i32,
    usage: RenderTargetUsage,
    is_content_lost: bool,
    renderer_available: bool,
    content_lost: Arc<EventHandlers<EventArgs>>,
}

#[allow(non_snake_case)]
impl RenderTargetCube {
    pub fn new(
        graphicsDevice: &GraphicsDevice,
        size: i32,
        mipMap: bool,
        preferredFormat: SurfaceFormat,
        preferredDepthFormat: DepthFormat,
    ) -> Result<Self> {
        Self::create(
            graphicsDevice,
            size,
            mipMap,
            preferredFormat,
            preferredDepthFormat,
            0,
            RenderTargetUsage::DiscardContents,
        )
    }

    pub fn from_graphics_device_and_size_and_mip_map_and_preferred_format_and_preferred_depth_format_and_preferred_multi_sample_count_and_usage(
        graphicsDevice: &GraphicsDevice,
        size: i32,
        mipMap: bool,
        preferredFormat: SurfaceFormat,
        preferredDepthFormat: DepthFormat,
        preferredMultiSampleCount: i32,
        usage: RenderTargetUsage,
    ) -> Result<Self> {
        Self::create(
            graphicsDevice,
            size,
            mipMap,
            preferredFormat,
            preferredDepthFormat,
            preferredMultiSampleCount,
            usage,
        )
    }

    fn create(
        graphicsDevice: &GraphicsDevice,
        size: i32,
        mipMap: bool,
        preferredFormat: SurfaceFormat,
        preferredDepthFormat: DepthFormat,
        preferredMultiSampleCount: i32,
        usage: RenderTargetUsage,
    ) -> Result<Self> {
        if size <= 0 || preferredMultiSampleCount < 0 {
            return Err(CnaError::InvalidInput(
                "render-target size must be positive and multisampling nonnegative",
            ));
        }
        let info = sys::CNA_RenderTargetCubeCreateInfo {
            struct_size: size_of::<sys::CNA_RenderTargetCubeCreateInfo>() as u32,
            struct_version: 1,
            size: size as u32,
            mip_map: u8::from(mipMap),
            reserved: [0; 3],
            format: preferredFormat as u32,
            depth_format: preferredDepthFormat as u32,
            multi_sample_count: preferredMultiSampleCount,
            usage: usage as u32,
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        graphicsDevice.state.native().create_render_target_cube(
            graphicsDevice.handle()?,
            &info,
            &mut handle,
        )?;
        match Self::from_handle(graphicsDevice, handle) {
            Ok(value) => Ok(value),
            Err(error) => {
                let _ = graphicsDevice.state.native().destroy_render_target(handle);
                Err(error)
            }
        }
    }

    fn from_handle(device: &GraphicsDevice, handle: sys::CNA_Handle) -> Result<Self> {
        let info = target_info(device, handle, sys::CNA_RENDER_TARGET_KIND_CUBE)?;
        Ok(Self {
            state: ResourceState::new(device, handle, ResourceKind::RenderTargetCube),
            size: info.width as i32,
            level_count: info.level_count as i32,
            format: surface_format(info.format)?,
            depth_format: depth_format(info.depth_format)?,
            multi_sample_count: info.multi_sample_count,
            usage: render_target_usage(info.usage)?,
            is_content_lost: info.is_content_lost != sys::CNA_FALSE,
            renderer_available: info.renderer_available != sys::CNA_FALSE,
            content_lost: Arc::new(EventHandlers::new()),
        })
    }

    fn retained(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            size: self.size,
            level_count: self.level_count,
            format: self.format,
            depth_format: self.depth_format,
            multi_sample_count: self.multi_sample_count,
            usage: self.usage,
            is_content_lost: self.is_content_lost,
            renderer_available: self.renderer_available,
            content_lost: Arc::clone(&self.content_lost),
        }
    }

    #[must_use]
    pub const fn DepthStencilFormat(&self) -> DepthFormat {
        self.depth_format
    }
    #[must_use]
    pub const fn MultiSampleCount(&self) -> i32 {
        self.multi_sample_count
    }
    #[must_use]
    pub const fn RenderTargetUsage(&self) -> RenderTargetUsage {
        self.usage
    }
    #[must_use]
    pub const fn IsContentLost(&self) -> bool {
        self.is_content_lost
    }
    pub fn AddContentLostHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.content_lost.add(handler)
    }
    pub fn RemoveContentLostHandler(&self, registration: u64) -> bool {
        self.content_lost.remove(registration)
    }
    pub fn Dispose(&mut self, value: bool) -> Result<()> {
        self.state.dispose_with_event(self, value)
    }

    pub(super) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.state.require_handle()
    }
}

enum BoundTarget {
    TwoD(RenderTarget2D),
    Cube(RenderTargetCube),
}

/// Safe retained XNA render-target binding value.
pub struct RenderTargetBinding {
    target: BoundTarget,
    cube_map_face: CubeMapFace,
}

#[allow(non_snake_case)]
impl RenderTargetBinding {
    pub fn from_render_target_and_cube_map_face(
        renderTarget: &RenderTargetCube,
        cubeMapFace: CubeMapFace,
    ) -> Result<Self> {
        renderTarget.handle()?;
        Ok(Self {
            target: BoundTarget::Cube(renderTarget.retained()),
            cube_map_face: cubeMapFace,
        })
    }

    pub fn new(renderTarget: &RenderTarget2D) -> Result<Self> {
        renderTarget.handle()?;
        Ok(Self {
            target: BoundTarget::TwoD(renderTarget.retained()),
            cube_map_face: CubeMapFace::PositiveX,
        })
    }

    #[must_use]
    pub const fn CubeMapFace(&self) -> CubeMapFace {
        self.cube_map_face
    }

    #[must_use]
    pub fn RenderTarget(&self) -> &dyn Texture {
        match &self.target {
            BoundTarget::TwoD(value) => value,
            BoundTarget::Cube(value) => value,
        }
    }

    pub(super) fn handle(&self) -> Result<sys::CNA_Handle> {
        match &self.target {
            BoundTarget::TwoD(value) => value.handle(),
            BoundTarget::Cube(value) => value.handle(),
        }
    }

    pub(super) fn device(&self) -> &GraphicsDevice {
        match &self.target {
            BoundTarget::TwoD(value) => value.state.device(),
            BoundTarget::Cube(value) => value.state.device(),
        }
    }

    pub(super) fn dimensions_and_samples(&self) -> (i32, i32, i32) {
        match &self.target {
            BoundTarget::TwoD(value) => (value.width, value.height, value.multi_sample_count),
            BoundTarget::Cube(value) => (value.size, value.size, value.multi_sample_count),
        }
    }

    pub(super) fn to_native(&self) -> Result<sys::CNA_RenderTargetBinding> {
        Ok(sys::CNA_RenderTargetBinding {
            struct_size: size_of::<sys::CNA_RenderTargetBinding>() as u32,
            struct_version: 1,
            render_target: self.handle()?,
            array_slice: 0,
            cube_map_face: self.cube_map_face as u32,
        })
    }
}

impl Clone for RenderTargetBinding {
    fn clone(&self) -> Self {
        Self {
            target: match &self.target {
                BoundTarget::TwoD(value) => BoundTarget::TwoD(value.retained()),
                BoundTarget::Cube(value) => BoundTarget::Cube(value.retained()),
            },
            cube_map_face: self.cube_map_face,
        }
    }
}

impl PartialEq for RenderTargetBinding {
    fn eq(&self, other: &Self) -> bool {
        self.handle().ok() == other.handle().ok() && self.cube_map_face == other.cube_map_face
    }
}

fn target_info(
    device: &GraphicsDevice,
    handle: sys::CNA_Handle,
    expected_kind: sys::CNA_RenderTargetKind,
) -> Result<sys::CNA_RenderTargetInfo> {
    let mut info = sys::CNA_RenderTargetInfo {
        struct_size: size_of::<sys::CNA_RenderTargetInfo>() as u32,
        struct_version: 1,
        ..sys::CNA_RenderTargetInfo::default()
    };
    device
        .state
        .native()
        .render_target_info(handle, &mut info)?;
    if info.kind != expected_kind || info.width == 0 || info.height == 0 {
        return Err(CnaError::InvalidInput(
            "native render-target identity or dimensions are invalid",
        ));
    }
    Ok(info)
}

fn surface_format(value: u32) -> Result<SurfaceFormat> {
    SurfaceFormat::from_native(value).ok_or(CnaError::InvalidInput(
        "native render-target format is unknown",
    ))
}

fn depth_format(value: u32) -> Result<DepthFormat> {
    Ok(match value {
        0 => DepthFormat::None,
        1 => DepthFormat::Depth16,
        2 => DepthFormat::Depth24,
        3 => DepthFormat::Depth24Stencil8,
        _ => return Err(CnaError::InvalidInput("native depth format is unknown")),
    })
}

fn render_target_usage(value: u32) -> Result<RenderTargetUsage> {
    Ok(match value {
        0 => RenderTargetUsage::DiscardContents,
        1 => RenderTargetUsage::PreserveContents,
        2 => RenderTargetUsage::PlatformContents,
        _ => {
            return Err(CnaError::InvalidInput(
                "native render-target usage is unknown",
            ))
        }
    })
}

macro_rules! render_target_traits {
    ($type:ty, $base:ident) => {
        impl Texture for $type {
            fn Format(&self) -> SurfaceFormat {
                self.format
            }
            fn LevelCount(&self) -> i32 {
                self.level_count
            }
        }

        impl $base for $type {}

        impl TextureRuntime for $type {
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
                        "render target belongs to a different graphics device",
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

        impl GraphicsResource for $type {
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

        impl Drop for $type {
            fn drop(&mut self) {}
        }
    };
}

render_target_traits!(RenderTarget2D, Texture2DBase);
render_target_traits!(RenderTargetCube, TextureCubeBase);
