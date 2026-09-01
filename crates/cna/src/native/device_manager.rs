//! Native graphics-device-manager calls over the reviewed CNA ABI 0.20 slice.

use core::ffi::c_void;

use cna_sys as sys;

use crate::error::{CnaError, ErrorCategory, Result};

use super::Native;

#[derive(Clone, Copy)]
pub(crate) struct NativeGraphicsPreferences {
    pub(crate) graphics_profile: u32,
    pub(crate) is_full_screen: bool,
    pub(crate) prefer_multi_sampling: bool,
    pub(crate) back_buffer_format: u32,
    pub(crate) back_buffer_width: i32,
    pub(crate) back_buffer_height: i32,
    pub(crate) depth_stencil_format: u32,
    pub(crate) synchronize_with_vertical_retrace: bool,
    pub(crate) supported_orientations: u32,
}

impl Native {
    pub(crate) fn create_graphics_device_manager(
        &self,
        game: sys::CNA_Handle,
    ) -> Result<sys::CNA_GraphicsDeviceManagerHandle> {
        let mut manager = sys::CNA_INVALID_HANDLE;
        // SAFETY: game is live and output is writable for the call.
        self.check(unsafe { (self.graphics_device_manager_create)(game, &mut manager) })?;
        if manager == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INVALID_STATE,
                category: ErrorCategory::None,
                message: "CNA returned an invalid GraphicsDeviceManager handle".to_owned(),
            });
        }
        Ok(manager)
    }

    pub(crate) fn graphics_device_manager_preferences(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
    ) -> Result<NativeGraphicsPreferences> {
        let mut graphics_profile = 0;
        let mut is_full_screen = sys::CNA_FALSE;
        let mut prefer_multi_sampling = sys::CNA_FALSE;
        let mut back_buffer_format = 0;
        let mut back_buffer_width = 0;
        let mut back_buffer_height = 0;
        let mut depth_stencil_format = 0;
        let mut synchronize_with_vertical_retrace = sys::CNA_FALSE;
        let mut supported_orientations = 0;
        // SAFETY: the manager is live and every output is writable for its call.
        unsafe {
            self.check((self.graphics_device_manager_get_graphics_profile)(
                manager,
                &mut graphics_profile,
            ))?;
            self.check((self.graphics_device_manager_get_is_full_screen)(
                manager,
                &mut is_full_screen,
            ))?;
            self.check((self.graphics_device_manager_get_prefer_multi_sampling)(
                manager,
                &mut prefer_multi_sampling,
            ))?;
            self.check((self
                .graphics_device_manager_get_preferred_back_buffer_format)(
                manager,
                &mut back_buffer_format,
            ))?;
            self.check((self
                .graphics_device_manager_get_preferred_back_buffer_width)(
                manager,
                &mut back_buffer_width,
            ))?;
            self.check((self
                .graphics_device_manager_get_preferred_back_buffer_height)(
                manager,
                &mut back_buffer_height,
            ))?;
            self.check((self
                .graphics_device_manager_get_preferred_depth_stencil_format)(
                manager,
                &mut depth_stencil_format,
            ))?;
            self.check((self
                .graphics_device_manager_get_synchronize_with_vertical_retrace)(
                manager,
                &mut synchronize_with_vertical_retrace,
            ))?;
            self.check((self.graphics_device_manager_get_supported_orientations)(
                manager,
                &mut supported_orientations,
            ))?;
        }
        Ok(NativeGraphicsPreferences {
            graphics_profile,
            is_full_screen: is_full_screen != sys::CNA_FALSE,
            prefer_multi_sampling: prefer_multi_sampling != sys::CNA_FALSE,
            back_buffer_format,
            back_buffer_width,
            back_buffer_height,
            depth_stencil_format,
            synchronize_with_vertical_retrace: synchronize_with_vertical_retrace != sys::CNA_FALSE,
            supported_orientations,
        })
    }

    pub(crate) fn set_graphics_device_manager_graphics_profile(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
        value: u32,
    ) -> Result<()> {
        // SAFETY: manager is live; CNA validates the enum identity.
        self.check(unsafe { (self.graphics_device_manager_set_graphics_profile)(manager, value) })
    }

    pub(crate) fn set_graphics_device_manager_is_full_screen(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
        value: bool,
    ) -> Result<()> {
        // SAFETY: manager is live and CNA_Bool is canonicalized.
        self.check(unsafe {
            (self.graphics_device_manager_set_is_full_screen)(manager, u8::from(value))
        })
    }

    pub(crate) fn set_graphics_device_manager_prefer_multi_sampling(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
        value: bool,
    ) -> Result<()> {
        // SAFETY: manager is live and CNA_Bool is canonicalized.
        self.check(unsafe {
            (self.graphics_device_manager_set_prefer_multi_sampling)(manager, u8::from(value))
        })
    }

    pub(crate) fn set_graphics_device_manager_back_buffer_format(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
        value: u32,
    ) -> Result<()> {
        // SAFETY: manager is live; CNA validates the enum identity.
        self.check(unsafe {
            (self.graphics_device_manager_set_preferred_back_buffer_format)(manager, value)
        })
    }

    pub(crate) fn set_graphics_device_manager_back_buffer_width(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
        value: i32,
    ) -> Result<()> {
        // SAFETY: manager is live; CNA records arbitrary canonical preference values.
        self.check(unsafe {
            (self.graphics_device_manager_set_preferred_back_buffer_width)(manager, value)
        })
    }

    pub(crate) fn set_graphics_device_manager_back_buffer_height(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
        value: i32,
    ) -> Result<()> {
        // SAFETY: manager is live; CNA records arbitrary canonical preference values.
        self.check(unsafe {
            (self.graphics_device_manager_set_preferred_back_buffer_height)(manager, value)
        })
    }

    pub(crate) fn set_graphics_device_manager_depth_stencil_format(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
        value: u32,
    ) -> Result<()> {
        // SAFETY: manager is live; CNA validates the enum identity.
        self.check(unsafe {
            (self.graphics_device_manager_set_preferred_depth_stencil_format)(manager, value)
        })
    }

    pub(crate) fn set_graphics_device_manager_vertical_retrace(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
        value: bool,
    ) -> Result<()> {
        // SAFETY: manager is live and CNA_Bool is canonicalized.
        self.check(unsafe {
            (self.graphics_device_manager_set_synchronize_with_vertical_retrace)(
                manager,
                u8::from(value),
            )
        })
    }

    pub(crate) fn set_graphics_device_manager_supported_orientations(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
        value: u32,
    ) -> Result<()> {
        // SAFETY: manager is live and orientations are an open bit set.
        self.check(unsafe {
            (self.graphics_device_manager_set_supported_orientations)(manager, value)
        })
    }

    pub(crate) fn apply_graphics_device_manager_changes(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
    ) -> Result<()> {
        // SAFETY: manager is live.
        self.check(unsafe { (self.graphics_device_manager_apply_changes)(manager) })
    }

    pub(crate) fn toggle_graphics_device_manager_full_screen(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
    ) -> Result<()> {
        // SAFETY: manager is live.
        self.check(unsafe { (self.graphics_device_manager_toggle_full_screen)(manager) })
    }

    pub(crate) fn create_graphics_device_manager_device(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
    ) -> Result<()> {
        // SAFETY: manager is live and reuses the game-owned graphics device.
        self.check(unsafe { (self.graphics_device_manager_create_device)(manager) })
    }

    pub(crate) fn begin_graphics_device_manager_draw(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: manager is live and output is writable.
        self.check(unsafe { (self.graphics_device_manager_begin_draw)(manager, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn end_graphics_device_manager_draw(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
    ) -> Result<()> {
        // SAFETY: manager is live.
        self.check(unsafe { (self.graphics_device_manager_end_draw)(manager) })
    }

    pub(crate) fn dispose_graphics_device_manager(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
    ) -> Result<()> {
        // SAFETY: manager is live; CNA disposal is idempotent.
        self.check(unsafe { (self.graphics_device_manager_dispose)(manager) })
    }

    pub(crate) fn subscribe_graphics_device_manager_event(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
        event: sys::CNA_GraphicsDeviceManagerEvent,
        callback: unsafe extern "C" fn(*mut c_void),
        context: *mut c_void,
    ) -> Result<sys::CNA_GameEventRegistrationHandle> {
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: callback and context remain valid until registration release.
        self.check(unsafe {
            (self.graphics_device_manager_subscribe)(
                manager,
                event,
                Some(callback),
                context,
                &mut registration,
            )
        })?;
        Ok(registration)
    }

    pub(crate) fn subscribe_preparing_device_settings(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
        callback: unsafe extern "C" fn(*mut sys::CNA_GraphicsDeviceInformation, *mut c_void),
        context: *mut c_void,
    ) -> Result<sys::CNA_GameEventRegistrationHandle> {
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: callback and context remain valid until registration release.
        self.check(unsafe {
            (self.graphics_device_manager_subscribe_preparing_device_settings_ext)(
                manager,
                Some(callback),
                context,
                &mut registration,
            )
        })?;
        Ok(registration)
    }

    pub(crate) fn destroy_graphics_device_manager(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
    ) -> Result<()> {
        // SAFETY: manager ownership is transferred exactly once on success.
        self.check(unsafe { (self.graphics_device_manager_destroy)(manager) })
    }
}

/// `runtime_graphics_manager.h`'s remaining manager routes.
impl Native {
    pub(crate) fn manager_graphics_device(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
    ) -> Result<Option<sys::CNA_Handle>> {
        let mut value = sys::CNA_INVALID_HANDLE;
        // SAFETY: the manager handle is live and the output is a local.
        self.check(unsafe {
            (self.graphics_device_manager_get_graphics_device)(manager, &mut value)
        })?;
        Ok((value != sys::CNA_INVALID_HANDLE).then_some(value))
    }

    pub(crate) fn manager_presentation_mode(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
    ) -> Result<sys::CNA_PresentationMode> {
        let mut value = sys::CNA_PRESENTATION_MODE_LETTERBOX;
        // SAFETY: the manager handle is live and the output is a local.
        self.check(unsafe {
            (self.graphics_device_manager_get_preferred_presentation_mode_ext)(
                manager, &mut value,
            )
        })?;
        Ok(value)
    }

    pub(crate) fn set_manager_presentation_mode(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
        mode: sys::CNA_PresentationMode,
    ) -> Result<()> {
        // SAFETY: the manager handle is live and the mode is a scalar.
        self.check(unsafe {
            (self.graphics_device_manager_set_preferred_presentation_mode_ext)(manager, mode)
        })
    }

    /// Observes the candidate device settings without being able to change
    /// them.
    ///
    /// The pair to `subscribe_preparing_device_settings`, which is already
    /// bound and takes a `*mut` -- that one is
    /// `..._preparing_device_settings_ext` and exists so a handler can *edit*
    /// the configuration. This one takes a `*const` and cannot, which is what
    /// makes it the right subscription for a caller that only wants to know
    /// what was chosen.
    pub(crate) fn observe_preparing_device_settings(
        &self,
        manager: sys::CNA_GraphicsDeviceManagerHandle,
        callback: sys::CNA_PreparingDeviceSettingsCallback,
        context: *mut c_void,
    ) -> Result<sys::CNA_GameEventRegistrationHandle> {
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: the caller keeps `context` alive until it withdraws this
        // registration, which is the contract the safe layer upholds.
        self.check(unsafe {
            (self.graphics_device_manager_subscribe_preparing_device_settings)(
                manager,
                callback,
                context,
                &mut registration,
            )
        })?;
        Ok(registration)
    }
}
