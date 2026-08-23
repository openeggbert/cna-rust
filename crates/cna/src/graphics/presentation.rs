#![allow(
    non_snake_case,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use std::any::Any;
use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::extensions::window::WindowHandle;
use crate::game::DisplayOrientation;
use crate::value::Rectangle;

use super::{DepthFormat, PresentInterval, RenderTargetUsage, SurfaceFormat};

/// Immutable XNA display-mode snapshot returned by an adapter or device.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayMode {
    width: i32,
    height: i32,
    format: SurfaceFormat,
}

impl DisplayMode {
    pub(super) fn from_native(value: sys::CNA_DisplayMode) -> Option<Self> {
        Some(Self {
            width: value.width,
            height: value.height,
            format: SurfaceFormat::from_native(value.format)?,
        })
    }

    #[must_use]
    pub const fn Format(&self) -> SurfaceFormat {
        self.format
    }

    #[must_use]
    pub const fn Height(&self) -> i32 {
        self.height
    }

    #[must_use]
    pub const fn Width(&self) -> i32 {
        self.width
    }

    #[must_use]
    pub fn AspectRatio(&self) -> f32 {
        if self.width == 0 || self.height == 0 {
            0.0
        } else {
            self.width as f32 / self.height as f32
        }
    }

    #[must_use]
    pub const fn TitleSafeArea(&self) -> Rectangle {
        Rectangle::new(0, 0, self.width, self.height)
    }

    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "{{Width:{} Height:{} Format:{:?} AspectRatio:{}}}",
            self.width,
            self.height,
            self.format,
            self.AspectRatio()
        )
    }
}

#[derive(Clone, Copy)]
struct PresentationData {
    back_buffer_width: i32,
    back_buffer_height: i32,
    back_buffer_format: SurfaceFormat,
    depth_stencil_format: DepthFormat,
    multi_sample_count: i32,
    display_orientation: DisplayOrientation,
    presentation_interval: PresentInterval,
    render_target_usage: RenderTargetUsage,
    device_window_handle: WindowHandle,
    is_full_screen: bool,
}

/// Stable, independently cloneable XNA presentation-parameter object.
pub struct PresentationParameters {
    data: Mutex<PresentationData>,
}

impl PresentationParameters {
    /// XNA 4.0 Windows defaults are taken from the reference constructor IL.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: Mutex::new(PresentationData {
                back_buffer_width: 0,
                back_buffer_height: 0,
                back_buffer_format: SurfaceFormat::Color,
                depth_stencil_format: DepthFormat::None,
                multi_sample_count: 0,
                display_orientation: DisplayOrientation::Default,
                presentation_interval: PresentInterval::Default,
                render_target_usage: RenderTargetUsage::DiscardContents,
                device_window_handle: WindowHandle::default(),
                is_full_screen: true,
            }),
        }
    }

    pub(super) fn update_from_native(
        &self,
        value: sys::CNA_PresentationParameters,
        window_handle: WindowHandle,
    ) -> bool {
        let Some(back_buffer_format) = SurfaceFormat::from_native(value.back_buffer_format) else {
            return false;
        };
        let Some(depth_stencil_format) = depth_format_from_native(value.depth_stencil_format)
        else {
            return false;
        };
        let Some(presentation_interval) = present_interval_from_native(value.presentation_interval)
        else {
            return false;
        };
        let Some(render_target_usage) = render_target_usage_from_native(value.render_target_usage)
        else {
            return false;
        };
        let Ok(display_orientation) = i32::try_from(value.display_orientation) else {
            return false;
        };
        *self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = PresentationData {
            back_buffer_width: value.back_buffer_width,
            back_buffer_height: value.back_buffer_height,
            back_buffer_format,
            depth_stencil_format,
            multi_sample_count: value.multi_sample_count,
            display_orientation: DisplayOrientation::from_bits(display_orientation),
            presentation_interval,
            render_target_usage,
            device_window_handle: window_handle,
            is_full_screen: value.is_full_screen != sys::CNA_FALSE,
        };
        true
    }

    pub(super) fn to_native(&self, headless_ext: bool) -> sys::CNA_PresentationParameters {
        let value = self.read();
        sys::CNA_PresentationParameters {
            struct_size: core::mem::size_of::<sys::CNA_PresentationParameters>() as u32,
            struct_version: 1,
            back_buffer_format: value.back_buffer_format as u32,
            back_buffer_width: value.back_buffer_width,
            back_buffer_height: value.back_buffer_height,
            depth_stencil_format: value.depth_stencil_format as u32,
            multi_sample_count: value.multi_sample_count,
            presentation_interval: value.presentation_interval as u32,
            display_orientation: value.display_orientation.bits() as u32,
            render_target_usage: value.render_target_usage as u32,
            is_full_screen: u8::from(value.is_full_screen),
            headless_ext: u8::from(headless_ext),
            reserved: [0; 2],
        }
    }

    #[must_use]
    pub fn Clone(&self) -> Self {
        self.clone()
    }

    #[must_use]
    pub fn BackBufferWidth(&self) -> i32 {
        self.read().back_buffer_width
    }
    pub fn SetBackBufferWidth(&self, value: i32) {
        self.write().back_buffer_width = value;
    }
    #[must_use]
    pub fn BackBufferHeight(&self) -> i32 {
        self.read().back_buffer_height
    }
    pub fn SetBackBufferHeight(&self, value: i32) {
        self.write().back_buffer_height = value;
    }
    #[must_use]
    pub fn BackBufferFormat(&self) -> SurfaceFormat {
        self.read().back_buffer_format
    }
    pub fn SetBackBufferFormat(&self, value: SurfaceFormat) {
        self.write().back_buffer_format = value;
    }
    #[must_use]
    pub fn DepthStencilFormat(&self) -> DepthFormat {
        self.read().depth_stencil_format
    }
    pub fn SetDepthStencilFormat(&self, value: DepthFormat) {
        self.write().depth_stencil_format = value;
    }
    #[must_use]
    pub fn MultiSampleCount(&self) -> i32 {
        self.read().multi_sample_count
    }
    pub fn SetMultiSampleCount(&self, value: i32) {
        self.write().multi_sample_count = value;
    }
    #[must_use]
    pub fn DisplayOrientation(&self) -> DisplayOrientation {
        self.read().display_orientation
    }
    pub fn SetDisplayOrientation(&self, value: DisplayOrientation) {
        self.write().display_orientation = value;
    }
    #[must_use]
    pub fn PresentationInterval(&self) -> PresentInterval {
        self.read().presentation_interval
    }
    pub fn SetPresentationInterval(&self, value: PresentInterval) {
        self.write().presentation_interval = value;
    }
    #[must_use]
    pub fn RenderTargetUsage(&self) -> RenderTargetUsage {
        self.read().render_target_usage
    }
    pub fn SetRenderTargetUsage(&self, value: RenderTargetUsage) {
        self.write().render_target_usage = value;
    }
    #[must_use]
    pub fn DeviceWindowHandle(&self) -> WindowHandle {
        self.read().device_window_handle
    }
    pub fn SetDeviceWindowHandle(&self, value: WindowHandle) {
        self.write().device_window_handle = value;
    }
    #[must_use]
    pub fn IsFullScreen(&self) -> bool {
        self.read().is_full_screen
    }
    pub fn SetIsFullScreen(&self, value: bool) {
        self.write().is_full_screen = value;
    }
    #[must_use]
    pub fn Bounds(&self) -> Rectangle {
        let value = self.read();
        Rectangle::new(0, 0, value.back_buffer_width, value.back_buffer_height)
    }

    fn read(&self) -> PresentationData {
        *self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write(&self) -> std::sync::MutexGuard<'_, PresentationData> {
        self.data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Clone for PresentationParameters {
    fn clone(&self) -> Self {
        Self {
            data: Mutex::new(self.read()),
        }
    }
}

impl Default for PresentationParameters {
    fn default() -> Self {
        Self::new()
    }
}

fn depth_format_from_native(value: u32) -> Option<DepthFormat> {
    Some(match value {
        0 => DepthFormat::None,
        1 => DepthFormat::Depth16,
        2 => DepthFormat::Depth24,
        3 => DepthFormat::Depth24Stencil8,
        _ => return None,
    })
}

fn present_interval_from_native(value: u32) -> Option<PresentInterval> {
    Some(match value {
        0 => PresentInterval::Default,
        1 => PresentInterval::One,
        2 => PresentInterval::Two,
        3 => PresentInterval::Immediate,
        _ => return None,
    })
}

fn render_target_usage_from_native(value: u32) -> Option<RenderTargetUsage> {
    Some(match value {
        0 => RenderTargetUsage::DiscardContents,
        1 => RenderTargetUsage::PreserveContents,
        2 => RenderTargetUsage::PlatformContents,
        _ => return None,
    })
}

/// Payload shape for XNA's resource-created event.
#[derive(Clone)]
pub struct ResourceCreatedEventArgs {
    resource: Arc<dyn Any + Send + Sync>,
}

impl ResourceCreatedEventArgs {
    #[must_use]
    pub fn Resource(&self) -> Arc<dyn Any + Send + Sync> {
        Arc::clone(&self.resource)
    }
}

/// Snapshot payload for XNA's resource-destroyed event.
#[derive(Clone)]
pub struct ResourceDestroyedEventArgs {
    name: String,
    tag: Option<Arc<dyn Any + Send + Sync>>,
}

impl ResourceDestroyedEventArgs {
    #[must_use]
    pub fn Name(&self) -> String {
        self.name.clone()
    }
    #[must_use]
    pub fn Tag(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.tag.clone()
    }
}
