//! Native display, adapter and presentation queries.

#![allow(clippy::cast_possible_truncation)]

use core::mem::size_of;

use cna_sys as sys;

use crate::error::{CnaError, Result};

use super::Native;

impl Native {
    pub(crate) fn graphics_adapter_index(&self, device: sys::CNA_Handle) -> Result<u32> {
        let mut value = 0;
        // SAFETY: the callback-scoped device and output remain live.
        self.check(unsafe { (self.graphics_device_get_adapter_index)(device, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn graphics_adapter_count(&self, device: sys::CNA_Handle) -> Result<u64> {
        let mut value = 0;
        // SAFETY: the callback-scoped device and output remain live.
        self.check(unsafe { (self.graphics_adapter_get_count)(device, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn graphics_adapter_info(
        &self,
        device: sys::CNA_Handle,
        index: u32,
    ) -> Result<sys::CNA_GraphicsAdapterInfo> {
        let mut value = sys::CNA_GraphicsAdapterInfo {
            struct_size: size_of::<sys::CNA_GraphicsAdapterInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_GraphicsAdapterInfo::default()
        };
        // SAFETY: index is native-enumerated and output is fully initialized.
        self.check(unsafe { (self.graphics_adapter_get_info)(device, index, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn graphics_adapter_description(
        &self,
        device: sys::CNA_Handle,
        index: u32,
        byte_count: u64,
    ) -> Result<String> {
        self.adapter_string(
            device,
            index,
            byte_count,
            self.graphics_adapter_copy_description,
        )
    }

    pub(crate) fn graphics_adapter_device_name(
        &self,
        device: sys::CNA_Handle,
        index: u32,
        byte_count: u64,
    ) -> Result<String> {
        self.adapter_string(
            device,
            index,
            byte_count,
            self.graphics_adapter_copy_device_name,
        )
    }

    fn adapter_string(
        &self,
        device: sys::CNA_Handle,
        index: u32,
        byte_count: u64,
        copy: sys::cna_graphics_adapter_copy_description_fn,
    ) -> Result<String> {
        let capacity = usize::try_from(byte_count)
            .map_err(|_| CnaError::InvalidInput("adapter string is too large"))?;
        let mut bytes = vec![0_u8; capacity];
        let mut copied = 0;
        // SAFETY: the destination supplies exactly byte_count writable bytes.
        self.check(unsafe {
            copy(
                device,
                index,
                bytes.as_mut_ptr().cast(),
                byte_count,
                &mut copied,
            )
        })?;
        debug_assert_eq!(copied, byte_count);
        String::from_utf8(bytes).map_err(|_| CnaError::Native {
            code: sys::CNA_RESULT_ENCODING,
            message: "CNA returned a non-UTF-8 adapter string".to_owned(),
        })
    }

    pub(crate) fn graphics_adapter_current_display_mode(
        &self,
        device: sys::CNA_Handle,
        index: u32,
    ) -> Result<sys::CNA_DisplayMode> {
        let mut value = display_mode_output();
        // SAFETY: index is native-enumerated and output is fully initialized.
        self.check(unsafe {
            (self.graphics_adapter_get_current_display_mode)(device, index, &mut value)
        })?;
        Ok(value)
    }

    pub(crate) fn graphics_adapter_display_modes(
        &self,
        device: sys::CNA_Handle,
        index: u32,
    ) -> Result<Vec<sys::CNA_DisplayMode>> {
        let mut count = 0;
        // SAFETY: output count is live; false disables the format filter.
        self.check(unsafe {
            (self.graphics_adapter_get_display_mode_count)(
                device,
                index,
                sys::CNA_FALSE,
                0,
                &mut count,
            )
        })?;
        let capacity = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("display-mode collection is too large"))?;
        let mut values = vec![display_mode_output(); capacity];
        let mut copied = 0;
        // SAFETY: values supplies exactly count initialized writable elements.
        self.check(unsafe {
            (self.graphics_adapter_copy_display_modes)(
                device,
                index,
                sys::CNA_FALSE,
                0,
                values.as_mut_ptr(),
                count,
                &mut copied,
            )
        })?;
        debug_assert_eq!(copied, count);
        Ok(values)
    }

    pub(crate) fn set_graphics_adapter_preferences(
        &self,
        device: sys::CNA_Handle,
        index: u32,
        use_null: bool,
        use_reference: bool,
    ) -> Result<()> {
        // SAFETY: adapter identity is native-enumerated and bools are normalized.
        self.check(unsafe {
            (self.graphics_adapter_set_device_preferences)(
                device,
                index,
                u8::from(use_null),
                u8::from(use_reference),
            )
        })
    }

    pub(crate) fn graphics_adapter_profile_supported(
        &self,
        device: sys::CNA_Handle,
        index: u32,
        profile: sys::CNA_GraphicsProfile,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: all values are reviewed fixed-width identities and output is live.
        self.check(unsafe {
            (self.graphics_adapter_is_profile_supported)(device, index, profile, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn graphics_adapter_format_selection(
        &self,
        render_target: bool,
        device: sys::CNA_Handle,
        index: u32,
        profile: sys::CNA_GraphicsProfile,
        format: sys::CNA_SurfaceFormat,
        depth_format: sys::CNA_DepthFormat,
        multi_sample_count: i32,
    ) -> Result<sys::CNA_GraphicsFormatSelection> {
        let mut value = sys::CNA_GraphicsFormatSelection {
            struct_size: size_of::<sys::CNA_GraphicsFormatSelection>() as u32,
            struct_version: 1,
            ..sys::CNA_GraphicsFormatSelection::default()
        };
        let result = if render_target {
            // SAFETY: all identities are mapped enums and output is versioned.
            unsafe {
                (self.graphics_adapter_query_render_target_format)(
                    device,
                    index,
                    profile,
                    format,
                    depth_format,
                    multi_sample_count,
                    &mut value,
                )
            }
        } else {
            // SAFETY: same prototype and guarantees as the render-target route.
            unsafe {
                (self.graphics_adapter_query_backbuffer_format)(
                    device,
                    index,
                    profile,
                    format,
                    depth_format,
                    multi_sample_count,
                    &mut value,
                )
            }
        };
        self.check(result)?;
        Ok(value)
    }

    pub(crate) fn graphics_adapter_monitor_handle(
        &self,
        device: sys::CNA_Handle,
        index: u32,
    ) -> Result<u64> {
        let mut value = 0;
        // SAFETY: the route deliberately returns NOT_SUPPORTED after validation.
        self.check(unsafe {
            (self.graphics_adapter_get_native_monitor_handle)(device, index, &mut value)
        })?;
        Ok(value)
    }
}

fn display_mode_output() -> sys::CNA_DisplayMode {
    sys::CNA_DisplayMode {
        struct_size: size_of::<sys::CNA_DisplayMode>() as u32,
        struct_version: 1,
        ..sys::CNA_DisplayMode::default()
    }
}
