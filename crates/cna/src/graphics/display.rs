#![allow(
    non_snake_case,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use std::sync::{Arc, OnceLock, Weak};
use std::vec::IntoIter;

use cna_sys as sys;

use crate::error::{CnaError, ErrorCategory, Result};
use crate::extensions::window::WindowHandle;

use super::device::DeviceState;
use super::{DepthFormat, DisplayMode, GraphicsDevice, GraphicsProfile, SurfaceFormat};

/// Immutable display-mode snapshot collection.
pub struct DisplayModeCollection {
    values: Vec<DisplayMode>,
}

impl DisplayModeCollection {
    pub(super) fn new(values: Vec<DisplayMode>) -> Self {
        Self { values }
    }

    #[must_use]
    pub fn Item(&self, format: SurfaceFormat) -> Vec<DisplayMode> {
        self.values
            .iter()
            .filter(|value| value.Format() == format)
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn GetEnumerator(&self) -> IntoIter<DisplayMode> {
        self.values.clone().into_iter()
    }
}

impl IntoIterator for DisplayModeCollection {
    type Item = DisplayMode;
    type IntoIter = IntoIter<DisplayMode>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

/// Stable adapter identity associated with one durable graphics device.
pub struct GraphicsAdapter {
    device: Weak<DeviceState>,
    index: u32,
    placeholder: bool,
    current_display_mode: OnceLock<DisplayMode>,
    supported_display_modes: OnceLock<DisplayModeCollection>,
}

impl Clone for GraphicsAdapter {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            index: self.index,
            placeholder: self.placeholder,
            current_display_mode: OnceLock::new(),
            supported_display_modes: OnceLock::new(),
        }
    }
}

impl GraphicsAdapter {
    pub(crate) fn default_placeholder() -> Self {
        Self::proposal_placeholder(0)
    }

    pub(crate) fn proposal_placeholder(index: u32) -> Self {
        Self {
            device: Weak::new(),
            index,
            placeholder: true,
            current_display_mode: OnceLock::new(),
            supported_display_modes: OnceLock::new(),
        }
    }

    /// The canonical adapter index `cna_graphics_device_create` takes.
    ///
    /// Both adapter shapes carry one: a placeholder names the index it stands
    /// for, and an enumerated adapter names the index it was enumerated at.
    /// CNA validates the index itself and refuses an out-of-range one, so this
    /// does not re-enumerate -- an independent device is created before there
    /// is any device to enumerate through.
    pub(crate) const fn independent_construction_index(&self) -> Result<u32> {
        Ok(self.index)
    }

    pub(crate) fn same_identity(&self, other: &Self) -> bool {
        (self.placeholder && other.placeholder && self.index == other.index)
            || (!self.placeholder
                && !other.placeholder
                && self.index == other.index
                && self.device.as_ptr() == other.device.as_ptr())
    }

    pub(crate) fn identity_hash(&self) -> i32 {
        if self.placeholder {
            return self.index as i32;
        }
        let pointer = self.device.as_ptr() as usize;
        (pointer as u64 ^ (pointer as u64 >> 32) ^ u64::from(self.index)) as i32
    }

    pub(super) fn all(device: &GraphicsDevice) -> Result<&[Self]> {
        if device.state.adapters.get().is_none() {
            let count = device
                .state
                .native()
                .graphics_adapter_count(device.handle()?)?;
            let count = u32::try_from(count)
                .map_err(|_| CnaError::InvalidInput("graphics-adapter count is too large"))?;
            let values = (0..count)
                .map(|index| Self {
                    device: Arc::downgrade(&device.state),
                    index,
                    placeholder: false,
                    current_display_mode: OnceLock::new(),
                    supported_display_modes: OnceLock::new(),
                })
                .collect();
            let _ = device.state.adapters.set(values);
        }
        Ok(device
            .state
            .adapters
            .get()
            .expect("graphics adapters set above"))
    }

    pub fn Adapters(graphicsDevice: &GraphicsDevice) -> Result<&[GraphicsAdapter]> {
        Self::all(graphicsDevice)
    }

    pub fn DefaultAdapter(graphicsDevice: &GraphicsDevice) -> Result<&Self> {
        for adapter in Self::all(graphicsDevice)? {
            if adapter.IsDefaultAdapter()? {
                return Ok(adapter);
            }
        }
        Err(CnaError::Native {
            code: sys::CNA_RESULT_INVALID_STATE,
            category: ErrorCategory::None,
            message: "CNA enumerated no default graphics adapter".to_owned(),
        })
    }

    pub fn UseReferenceDevice(graphicsDevice: &GraphicsDevice) -> Result<bool> {
        Ok(Self::DefaultAdapter(graphicsDevice)?
            .info()?
            .use_reference_device
            != sys::CNA_FALSE)
    }

    pub fn SetUseReferenceDevice(graphicsDevice: &GraphicsDevice, value: bool) -> Result<()> {
        let adapter = Self::DefaultAdapter(graphicsDevice)?;
        let info = adapter.info()?;
        adapter.device()?.native().set_graphics_adapter_preferences(
            graphicsDevice.handle()?,
            adapter.index,
            info.use_null_device != sys::CNA_FALSE,
            value,
        )
    }

    pub fn UseNullDevice(graphicsDevice: &GraphicsDevice) -> Result<bool> {
        Ok(Self::DefaultAdapter(graphicsDevice)?
            .info()?
            .use_null_device
            != sys::CNA_FALSE)
    }

    pub fn SetUseNullDevice(graphicsDevice: &GraphicsDevice, value: bool) -> Result<()> {
        let adapter = Self::DefaultAdapter(graphicsDevice)?;
        let info = adapter.info()?;
        adapter.device()?.native().set_graphics_adapter_preferences(
            graphicsDevice.handle()?,
            adapter.index,
            value,
            info.use_reference_device != sys::CNA_FALSE,
        )
    }

    pub fn MonitorHandle(&self) -> Result<WindowHandle> {
        let device = self.device()?;
        Ok(WindowHandle(
            device
                .native()
                .graphics_adapter_monitor_handle(device.handle()?, self.index)?,
        ))
    }

    pub fn SupportedDisplayModes(&self) -> Result<&DisplayModeCollection> {
        if self.supported_display_modes.get().is_none() {
            let device = self.device()?;
            let values = device
                .native()
                .graphics_adapter_display_modes(device.handle()?, self.index)?
                .into_iter()
                .map(|value| {
                    DisplayMode::from_native(value).ok_or_else(|| CnaError::Native {
                        code: sys::CNA_RESULT_INTERNAL,
                        category: ErrorCategory::None,
                        message: "CNA returned an unknown display-mode format".to_owned(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let _ = self
                .supported_display_modes
                .set(DisplayModeCollection::new(values));
        }
        self.supported_display_modes.get().ok_or(CnaError::Native {
            code: sys::CNA_RESULT_INTERNAL,
            category: ErrorCategory::None,
            message: "supported display-mode identity could not be initialized".to_owned(),
        })
    }

    pub fn CurrentDisplayMode(&self) -> Result<&DisplayMode> {
        if self.current_display_mode.get().is_none() {
            let device = self.device()?;
            let native = device
                .native()
                .graphics_adapter_current_display_mode(device.handle()?, self.index)?;
            let value = DisplayMode::from_native(native).ok_or_else(|| CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                category: ErrorCategory::None,
                message: "CNA returned an unknown current display-mode format".to_owned(),
            })?;
            let _ = self.current_display_mode.set(value);
        }
        self.current_display_mode.get().ok_or(CnaError::Native {
            code: sys::CNA_RESULT_INTERNAL,
            category: ErrorCategory::None,
            message: "current display-mode identity could not be initialized".to_owned(),
        })
    }

    pub fn IsWideScreen(&self) -> Result<bool> {
        Ok(self.info()?.is_wide_screen != sys::CNA_FALSE)
    }

    pub fn IsDefaultAdapter(&self) -> Result<bool> {
        Ok(self.info()?.is_default_adapter != sys::CNA_FALSE)
    }

    pub fn Revision(&self) -> Result<i32> {
        Ok(self.info()?.revision)
    }
    pub fn SubSystemId(&self) -> Result<i32> {
        Ok(self.info()?.subsystem_id)
    }
    pub fn DeviceId(&self) -> Result<i32> {
        Ok(self.info()?.device_id)
    }
    pub fn VendorId(&self) -> Result<i32> {
        Ok(self.info()?.vendor_id)
    }

    pub fn DeviceName(&self) -> Result<String> {
        let device = self.device()?;
        let info = self.info()?;
        device.native().graphics_adapter_device_name(
            device.handle()?,
            self.index,
            info.device_name_byte_length,
        )
    }

    pub fn Description(&self) -> Result<String> {
        let device = self.device()?;
        let info = self.info()?;
        device.native().graphics_adapter_description(
            device.handle()?,
            self.index,
            info.description_byte_length,
        )
    }

    pub fn IsProfileSupported(&self, graphicsProfile: GraphicsProfile) -> Result<bool> {
        let device = self.device()?;
        device.native().graphics_adapter_profile_supported(
            device.handle()?,
            self.index,
            graphicsProfile as u32,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn QueryBackBufferFormat(
        &self,
        graphicsProfile: GraphicsProfile,
        format: SurfaceFormat,
        depthFormat: DepthFormat,
        multiSampleCount: i32,
        selectedFormat: &mut SurfaceFormat,
        selectedDepthFormat: &mut DepthFormat,
        selectedMultiSampleCount: &mut i32,
    ) -> Result<bool> {
        self.query_format(
            false,
            graphicsProfile,
            format,
            depthFormat,
            multiSampleCount,
            selectedFormat,
            selectedDepthFormat,
            selectedMultiSampleCount,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn QueryRenderTargetFormat(
        &self,
        graphicsProfile: GraphicsProfile,
        format: SurfaceFormat,
        depthFormat: DepthFormat,
        multiSampleCount: i32,
        selectedFormat: &mut SurfaceFormat,
        selectedDepthFormat: &mut DepthFormat,
        selectedMultiSampleCount: &mut i32,
    ) -> Result<bool> {
        self.query_format(
            true,
            graphicsProfile,
            format,
            depthFormat,
            multiSampleCount,
            selectedFormat,
            selectedDepthFormat,
            selectedMultiSampleCount,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn query_format(
        &self,
        render_target: bool,
        graphics_profile: GraphicsProfile,
        format: SurfaceFormat,
        depth_format: DepthFormat,
        multi_sample_count: i32,
        selected_format: &mut SurfaceFormat,
        selected_depth_format: &mut DepthFormat,
        selected_multi_sample_count: &mut i32,
    ) -> Result<bool> {
        let device = self.device()?;
        let value = device.native().graphics_adapter_format_selection(
            render_target,
            device.handle()?,
            self.index,
            graphics_profile as u32,
            format as u32,
            depth_format as u32,
            multi_sample_count,
        )?;
        *selected_format =
            SurfaceFormat::from_native(value.format).ok_or_else(|| CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                category: ErrorCategory::None,
                message: "CNA selected an unknown surface format".to_owned(),
            })?;
        *selected_depth_format = match value.depth_format {
            0 => DepthFormat::None,
            1 => DepthFormat::Depth16,
            2 => DepthFormat::Depth24,
            3 => DepthFormat::Depth24Stencil8,
            _ => {
                return Err(CnaError::Native {
                    code: sys::CNA_RESULT_INTERNAL,
                    category: ErrorCategory::None,
                    message: "CNA selected an unknown depth format".to_owned(),
                })
            }
        };
        *selected_multi_sample_count = value.multi_sample_count;
        Ok(value.exact_match != sys::CNA_FALSE)
    }

    fn info(&self) -> Result<sys::CNA_GraphicsAdapterInfo> {
        let device = self.device()?;
        device
            .native()
            .graphics_adapter_info(device.handle()?, self.index)
    }

    fn device(&self) -> Result<Arc<DeviceState>> {
        let device = self
            .device
            .upgrade()
            .ok_or(CnaError::InvalidInput("graphics device is disposed"))?;
        device.ensure_alive()?;
        Ok(device)
    }

    pub(super) fn index_for(&self, expected: &Arc<DeviceState>) -> Result<u32> {
        let device = self.device()?;
        if !Arc::ptr_eq(&device, expected) {
            return Err(CnaError::InvalidInput(
                "graphics adapter belongs to another graphics device",
            ));
        }
        Ok(self.index)
    }

    pub(super) fn proposal_index_for(&self, expected: &Arc<DeviceState>) -> Result<i32> {
        if self.placeholder {
            i32::try_from(self.index)
                .map_err(|_| CnaError::InvalidInput("graphics adapter index exceeds i32"))
        } else {
            i32::try_from(self.index_for(expected)?)
                .map_err(|_| CnaError::InvalidInput("graphics adapter index exceeds i32"))
        }
    }
}
