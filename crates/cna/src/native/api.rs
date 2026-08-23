//! Audited CNA function table and ABI-version-checked loading.

use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};

use super::loader::{library_candidates, Library};

#[derive(Debug)]
pub(crate) struct Native {
    #[cfg(unix)]
    pub(super) _library: Library,
    pub(super) error_get_last_message_size: sys::cna_error_get_last_message_size_fn,
    pub(super) error_copy_last_message: sys::cna_error_copy_last_message_fn,
    pub(super) game_create: sys::cna_game_create_fn,
    pub(super) game_set_frame_hooks: sys::cna_game_set_frame_hooks_ext_fn,
    pub(super) game_run: sys::cna_game_run_fn,
    pub(super) game_request_exit: sys::cna_game_request_exit_fn,
    pub(super) game_destroy: sys::cna_game_destroy_fn,
    pub(super) game_get_graphics_device: sys::cna_game_get_graphics_device_fn,
    pub(super) graphics_device_get_viewport: sys::cna_graphics_device_get_viewport_fn,
    pub(super) graphics_device_set_viewport: sys::cna_graphics_device_set_viewport_fn,
    pub(super) graphics_device_get_scissor_rectangle:
        sys::cna_graphics_device_get_scissor_rectangle_fn,
    pub(super) graphics_device_set_scissor_rectangle:
        sys::cna_graphics_device_set_scissor_rectangle_fn,
    pub(super) graphics_device_get_blend_factor: sys::cna_graphics_device_get_blend_factor_fn,
    pub(super) graphics_device_set_blend_factor: sys::cna_graphics_device_set_blend_factor_fn,
    pub(super) graphics_device_get_multi_sample_mask:
        sys::cna_graphics_device_get_multi_sample_mask_fn,
    pub(super) graphics_device_set_multi_sample_mask:
        sys::cna_graphics_device_set_multi_sample_mask_fn,
    pub(super) graphics_device_get_reference_stencil:
        sys::cna_graphics_device_get_reference_stencil_fn,
    pub(super) graphics_device_set_reference_stencil:
        sys::cna_graphics_device_set_reference_stencil_fn,
    pub(super) graphics_device_set_blend_state: sys::cna_graphics_device_set_blend_state_fn,
    pub(super) graphics_device_set_depth_stencil_state:
        sys::cna_graphics_device_set_depth_stencil_state_fn,
    pub(super) graphics_device_set_rasterizer_state:
        sys::cna_graphics_device_set_rasterizer_state_fn,
    pub(super) graphics_device_present: sys::cna_graphics_device_present_fn,
    pub(super) graphics_device_clear_rgba: sys::cna_graphics_device_clear_rgba_fn,
    pub(super) graphics_device_get_renderer_info: sys::cna_graphics_device_get_renderer_info_fn,
    pub(super) graphics_device_get_renderer_name_size:
        sys::cna_graphics_device_get_renderer_name_size_fn,
    pub(super) graphics_device_copy_renderer_name: sys::cna_graphics_device_copy_renderer_name_fn,
    pub(super) texture2d_create_from_encoded_memory:
        sys::cna_texture2d_create_from_encoded_memory_fn,
    pub(super) texture2d_create: sys::cna_texture2d_create_fn,
    pub(super) texture2d_get_info: sys::cna_texture2d_get_info_fn,
    pub(super) texture2d_set_data: sys::cna_texture2d_set_data_fn,
    pub(super) texture2d_get_data: sys::cna_texture2d_get_data_fn,
    pub(super) texture2d_get_encoded_byte_count: sys::cna_texture2d_get_encoded_byte_count_fn,
    pub(super) texture2d_copy_encoded: sys::cna_texture2d_copy_encoded_fn,
    pub(super) texture2d_destroy: sys::cna_texture2d_destroy_fn,
    pub(super) sprite_batch_create: sys::cna_sprite_batch_create_fn,
    pub(super) sprite_batch_begin: sys::cna_sprite_batch_begin_fn,
    pub(super) sprite_batch_begin_with_states: sys::cna_sprite_batch_begin_with_states_fn,
    pub(super) sprite_batch_submit_many: sys::cna_sprite_batch_submit_many_fn,
    pub(super) sprite_batch_end: sys::cna_sprite_batch_end_fn,
    pub(super) sprite_batch_destroy: sys::cna_sprite_batch_destroy_fn,
    pub(super) keyboard_get_state: sys::cna_keyboard_get_state_fn,
    pub(super) mouse_get_state: sys::cna_mouse_get_state_fn,
    pub(super) mouse_get_window_handle: sys::cna_mouse_get_window_handle_fn,
    pub(super) mouse_set_window_handle: sys::cna_mouse_set_window_handle_fn,
    pub(super) mouse_set_position: sys::cna_mouse_set_position_fn,
    pub(super) gamepad_get_state: sys::cna_gamepad_get_state_fn,
    pub(super) gamepad_get_state_with_dead_zone: sys::cna_gamepad_get_state_with_dead_zone_fn,
    pub(super) gamepad_get_capabilities: sys::cna_gamepad_get_capabilities_fn,
    pub(super) gamepad_set_vibration: sys::cna_gamepad_set_vibration_fn,
}

impl Native {
    pub(crate) fn load() -> Result<Arc<Self>> {
        #[cfg(unix)]
        {
            Self::load_unix().map(Arc::new)
        }
        #[cfg(not(unix))]
        {
            Err(CnaError::UnsupportedPlatform)
        }
    }

    #[cfg(unix)]
    fn load_unix() -> Result<Self> {
        let candidates = library_candidates();
        let mut diagnostics = Vec::new();
        for candidate in &candidates {
            match Library::open(candidate) {
                Ok(library) => return Self::from_library(library),
                Err(error) => diagnostics.push(format!("{}: {error}", candidate.display())),
            }
        }
        Err(CnaError::NativeUnavailable {
            searched: candidates,
            details: diagnostics.join("; "),
        })
    }

    #[cfg(unix)]
    fn from_library(library: Library) -> Result<Self> {
        macro_rules! symbol {
            ($name:literal, $ty:ty) => {{
                // SAFETY: every requested type is copied directly from the
                // canonical CNA header declaration named by `$name`.
                unsafe { library.symbol::<$ty>($name)? }
            }};
        }

        let get_abi_version = symbol!("cna_get_abi_version", sys::cna_get_abi_version_fn);
        // SAFETY: the symbol has the audited zero-argument ABI declaration.
        let actual = unsafe { get_abi_version() };
        if actual != sys::CNA_ABI_VERSION {
            return Err(CnaError::AbiVersionMismatch {
                expected: sys::CNA_ABI_VERSION,
                actual,
            });
        }

        Ok(Self {
            error_get_last_message_size: symbol!(
                "cna_error_get_last_message_size",
                sys::cna_error_get_last_message_size_fn
            ),
            error_copy_last_message: symbol!(
                "cna_error_copy_last_message",
                sys::cna_error_copy_last_message_fn
            ),
            game_create: symbol!("cna_game_create", sys::cna_game_create_fn),
            game_set_frame_hooks: symbol!(
                "cna_game_set_frame_hooks_ext",
                sys::cna_game_set_frame_hooks_ext_fn
            ),
            game_run: symbol!("cna_game_run", sys::cna_game_run_fn),
            game_request_exit: symbol!("cna_game_request_exit", sys::cna_game_request_exit_fn),
            game_destroy: symbol!("cna_game_destroy", sys::cna_game_destroy_fn),
            game_get_graphics_device: symbol!(
                "cna_game_get_graphics_device",
                sys::cna_game_get_graphics_device_fn
            ),
            graphics_device_get_viewport: symbol!(
                "cna_graphics_device_get_viewport",
                sys::cna_graphics_device_get_viewport_fn
            ),
            graphics_device_set_viewport: symbol!(
                "cna_graphics_device_set_viewport",
                sys::cna_graphics_device_set_viewport_fn
            ),
            graphics_device_get_scissor_rectangle: symbol!(
                "cna_graphics_device_get_scissor_rectangle",
                sys::cna_graphics_device_get_scissor_rectangle_fn
            ),
            graphics_device_set_scissor_rectangle: symbol!(
                "cna_graphics_device_set_scissor_rectangle",
                sys::cna_graphics_device_set_scissor_rectangle_fn
            ),
            graphics_device_get_blend_factor: symbol!(
                "cna_graphics_device_get_blend_factor",
                sys::cna_graphics_device_get_blend_factor_fn
            ),
            graphics_device_set_blend_factor: symbol!(
                "cna_graphics_device_set_blend_factor",
                sys::cna_graphics_device_set_blend_factor_fn
            ),
            graphics_device_get_multi_sample_mask: symbol!(
                "cna_graphics_device_get_multi_sample_mask",
                sys::cna_graphics_device_get_multi_sample_mask_fn
            ),
            graphics_device_set_multi_sample_mask: symbol!(
                "cna_graphics_device_set_multi_sample_mask",
                sys::cna_graphics_device_set_multi_sample_mask_fn
            ),
            graphics_device_get_reference_stencil: symbol!(
                "cna_graphics_device_get_reference_stencil",
                sys::cna_graphics_device_get_reference_stencil_fn
            ),
            graphics_device_set_reference_stencil: symbol!(
                "cna_graphics_device_set_reference_stencil",
                sys::cna_graphics_device_set_reference_stencil_fn
            ),
            graphics_device_set_blend_state: symbol!(
                "cna_graphics_device_set_blend_state",
                sys::cna_graphics_device_set_blend_state_fn
            ),
            graphics_device_set_depth_stencil_state: symbol!(
                "cna_graphics_device_set_depth_stencil_state",
                sys::cna_graphics_device_set_depth_stencil_state_fn
            ),
            graphics_device_set_rasterizer_state: symbol!(
                "cna_graphics_device_set_rasterizer_state",
                sys::cna_graphics_device_set_rasterizer_state_fn
            ),
            graphics_device_present: symbol!(
                "cna_graphics_device_present",
                sys::cna_graphics_device_present_fn
            ),
            graphics_device_clear_rgba: symbol!(
                "cna_graphics_device_clear_rgba",
                sys::cna_graphics_device_clear_rgba_fn
            ),
            graphics_device_get_renderer_info: symbol!(
                "cna_graphics_device_get_renderer_info",
                sys::cna_graphics_device_get_renderer_info_fn
            ),
            graphics_device_get_renderer_name_size: symbol!(
                "cna_graphics_device_get_renderer_name_size",
                sys::cna_graphics_device_get_renderer_name_size_fn
            ),
            graphics_device_copy_renderer_name: symbol!(
                "cna_graphics_device_copy_renderer_name",
                sys::cna_graphics_device_copy_renderer_name_fn
            ),
            texture2d_create_from_encoded_memory: symbol!(
                "cna_texture2d_create_from_encoded_memory",
                sys::cna_texture2d_create_from_encoded_memory_fn
            ),
            texture2d_create: symbol!("cna_texture2d_create", sys::cna_texture2d_create_fn),
            texture2d_get_info: symbol!("cna_texture2d_get_info", sys::cna_texture2d_get_info_fn),
            texture2d_set_data: symbol!("cna_texture2d_set_data", sys::cna_texture2d_set_data_fn),
            texture2d_get_data: symbol!("cna_texture2d_get_data", sys::cna_texture2d_get_data_fn),
            texture2d_get_encoded_byte_count: symbol!(
                "cna_texture2d_get_encoded_byte_count",
                sys::cna_texture2d_get_encoded_byte_count_fn
            ),
            texture2d_copy_encoded: symbol!(
                "cna_texture2d_copy_encoded",
                sys::cna_texture2d_copy_encoded_fn
            ),
            texture2d_destroy: symbol!("cna_texture2d_destroy", sys::cna_texture2d_destroy_fn),
            sprite_batch_create: symbol!(
                "cna_sprite_batch_create",
                sys::cna_sprite_batch_create_fn
            ),
            sprite_batch_begin: symbol!("cna_sprite_batch_begin", sys::cna_sprite_batch_begin_fn),
            sprite_batch_begin_with_states: symbol!(
                "cna_sprite_batch_begin_with_states",
                sys::cna_sprite_batch_begin_with_states_fn
            ),
            sprite_batch_submit_many: symbol!(
                "cna_sprite_batch_submit_many",
                sys::cna_sprite_batch_submit_many_fn
            ),
            sprite_batch_end: symbol!("cna_sprite_batch_end", sys::cna_sprite_batch_end_fn),
            sprite_batch_destroy: symbol!(
                "cna_sprite_batch_destroy",
                sys::cna_sprite_batch_destroy_fn
            ),
            keyboard_get_state: symbol!("cna_keyboard_get_state", sys::cna_keyboard_get_state_fn),
            mouse_get_state: symbol!("cna_mouse_get_state", sys::cna_mouse_get_state_fn),
            mouse_get_window_handle: symbol!(
                "cna_mouse_get_window_handle",
                sys::cna_mouse_get_window_handle_fn
            ),
            mouse_set_window_handle: symbol!(
                "cna_mouse_set_window_handle",
                sys::cna_mouse_set_window_handle_fn
            ),
            mouse_set_position: symbol!("cna_mouse_set_position", sys::cna_mouse_set_position_fn),
            gamepad_get_state: symbol!("cna_gamepad_get_state", sys::cna_gamepad_get_state_fn),
            gamepad_get_state_with_dead_zone: symbol!(
                "cna_gamepad_get_state_with_dead_zone",
                sys::cna_gamepad_get_state_with_dead_zone_fn
            ),
            gamepad_get_capabilities: symbol!(
                "cna_gamepad_get_capabilities",
                sys::cna_gamepad_get_capabilities_fn
            ),
            gamepad_set_vibration: symbol!(
                "cna_gamepad_set_vibration",
                sys::cna_gamepad_set_vibration_fn
            ),
            _library: library,
        })
    }
}
