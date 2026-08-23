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
    pub(super) game_run_one_frame: sys::cna_game_run_one_frame_fn,
    pub(super) game_run: sys::cna_game_run_fn,
    pub(super) game_request_exit: sys::cna_game_request_exit_fn,
    pub(super) game_get_is_active: sys::cna_game_get_is_active_fn,
    pub(super) game_get_is_mouse_visible: sys::cna_game_get_is_mouse_visible_fn,
    pub(super) game_set_is_mouse_visible: sys::cna_game_set_is_mouse_visible_fn,
    pub(super) game_get_is_fixed_time_step: sys::cna_game_get_is_fixed_time_step_fn,
    pub(super) game_set_is_fixed_time_step: sys::cna_game_set_is_fixed_time_step_fn,
    pub(super) game_get_target_elapsed_time_ticks: sys::cna_game_get_target_elapsed_time_ticks_fn,
    pub(super) game_set_target_elapsed_time_ticks: sys::cna_game_set_target_elapsed_time_ticks_fn,
    pub(super) game_get_inactive_sleep_time_ticks: sys::cna_game_get_inactive_sleep_time_ticks_fn,
    pub(super) game_set_inactive_sleep_time_ticks: sys::cna_game_set_inactive_sleep_time_ticks_fn,
    pub(super) game_reset_elapsed_time: sys::cna_game_reset_elapsed_time_fn,
    pub(super) game_suppress_draw: sys::cna_game_suppress_draw_fn,
    pub(super) game_tick: sys::cna_game_tick_fn,
    pub(super) game_set_window_title: sys::cna_game_set_window_title_fn,
    pub(super) game_subscribe: sys::cna_game_subscribe_fn,
    pub(super) game_window_subscribe: sys::cna_game_window_subscribe_fn,
    pub(super) game_unsubscribe: sys::cna_game_unsubscribe_fn,
    pub(super) game_destroy: sys::cna_game_destroy_fn,
    pub(super) game_window_get_allow_user_resizing: sys::cna_game_window_get_allow_user_resizing_fn,
    pub(super) game_window_set_allow_user_resizing: sys::cna_game_window_set_allow_user_resizing_fn,
    pub(super) game_window_get_client_bounds: sys::cna_game_window_get_client_bounds_fn,
    pub(super) game_window_get_current_orientation: sys::cna_game_window_get_current_orientation_fn,
    pub(super) game_window_get_native_handle: sys::cna_game_window_get_native_handle_ext_fn,
    pub(super) game_window_get_screen_device_name_size:
        sys::cna_game_window_get_screen_device_name_size_fn,
    pub(super) game_window_copy_screen_device_name: sys::cna_game_window_copy_screen_device_name_fn,
    pub(super) game_window_get_title_size: sys::cna_game_window_get_title_size_fn,
    pub(super) game_window_copy_title: sys::cna_game_window_copy_title_fn,
    pub(super) game_window_begin_screen_device_change:
        sys::cna_game_window_begin_screen_device_change_fn,
    pub(super) game_window_end_screen_device_change:
        sys::cna_game_window_end_screen_device_change_fn,
    pub(super) game_get_graphics_device: sys::cna_game_get_graphics_device_fn,
    pub(super) graphics_device_get_status: sys::cna_graphics_device_get_status_fn,
    pub(super) graphics_device_get_graphics_profile:
        sys::cna_graphics_device_get_graphics_profile_fn,
    pub(super) graphics_device_get_presentation_parameters:
        sys::cna_graphics_device_get_presentation_parameters_fn,
    pub(super) graphics_device_get_display_mode: sys::cna_graphics_device_get_display_mode_fn,
    pub(super) graphics_device_get_blend_state: sys::cna_graphics_device_get_blend_state_fn,
    pub(super) graphics_device_get_depth_stencil_state:
        sys::cna_graphics_device_get_depth_stencil_state_fn,
    pub(super) graphics_device_get_rasterizer_state:
        sys::cna_graphics_device_get_rasterizer_state_fn,
    pub(super) graphics_device_get_sampler_state: sys::cna_graphics_device_get_sampler_state_fn,
    pub(super) graphics_device_set_sampler_state: sys::cna_graphics_device_set_sampler_state_fn,
    pub(super) graphics_device_get_texture: sys::cna_graphics_device_get_texture_fn,
    pub(super) graphics_device_set_texture: sys::cna_graphics_device_set_texture_fn,
    pub(super) graphics_device_get_adapter_index: sys::cna_graphics_device_get_adapter_index_fn,
    pub(super) graphics_adapter_get_count: sys::cna_graphics_adapter_get_count_fn,
    pub(super) graphics_adapter_get_info: sys::cna_graphics_adapter_get_info_fn,
    pub(super) graphics_adapter_copy_description: sys::cna_graphics_adapter_copy_description_fn,
    pub(super) graphics_adapter_copy_device_name: sys::cna_graphics_adapter_copy_device_name_fn,
    pub(super) graphics_adapter_get_current_display_mode:
        sys::cna_graphics_adapter_get_current_display_mode_fn,
    pub(super) graphics_adapter_get_display_mode_count:
        sys::cna_graphics_adapter_get_display_mode_count_fn,
    pub(super) graphics_adapter_copy_display_modes: sys::cna_graphics_adapter_copy_display_modes_fn,
    pub(super) graphics_adapter_set_device_preferences:
        sys::cna_graphics_adapter_set_device_preferences_fn,
    pub(super) graphics_adapter_is_profile_supported:
        sys::cna_graphics_adapter_is_profile_supported_fn,
    pub(super) graphics_adapter_query_render_target_format:
        sys::cna_graphics_adapter_query_render_target_format_fn,
    pub(super) graphics_adapter_query_backbuffer_format:
        sys::cna_graphics_adapter_query_backbuffer_format_fn,
    pub(super) graphics_adapter_get_native_monitor_handle:
        sys::cna_graphics_adapter_get_native_monitor_handle_fn,
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
    pub(super) graphics_device_reset: sys::cna_graphics_device_reset_fn,
    pub(super) graphics_device_reset_with_parameters:
        sys::cna_graphics_device_reset_with_parameters_fn,
    pub(super) graphics_device_get_backbuffer_data_window:
        sys::cna_graphics_device_get_backbuffer_data_window_fn,
    pub(super) graphics_device_clear_rgba: sys::cna_graphics_device_clear_rgba_fn,
    pub(super) graphics_device_set_vertex_buffer: sys::cna_graphics_device_set_vertex_buffer_fn,
    pub(super) graphics_device_set_vertex_buffer_offset:
        sys::cna_graphics_device_set_vertex_buffer_offset_fn,
    pub(super) graphics_device_set_vertex_buffers: sys::cna_graphics_device_set_vertex_buffers_fn,
    pub(super) graphics_device_get_vertex_buffer_count:
        sys::cna_graphics_device_get_vertex_buffer_count_fn,
    pub(super) graphics_device_copy_vertex_buffers: sys::cna_graphics_device_copy_vertex_buffers_fn,
    pub(super) graphics_device_get_vertex_buffer: sys::cna_graphics_device_get_vertex_buffer_fn,
    pub(super) graphics_device_set_index_buffer: sys::cna_graphics_device_set_index_buffer_fn,
    pub(super) graphics_device_get_index_buffer: sys::cna_graphics_device_get_index_buffer_fn,
    pub(super) graphics_device_draw_primitives: sys::cna_graphics_device_draw_primitives_fn,
    pub(super) graphics_device_draw_indexed_primitives:
        sys::cna_graphics_device_draw_indexed_primitives_fn,
    pub(super) graphics_device_draw_instanced_primitives:
        sys::cna_graphics_device_draw_instanced_primitives_fn,
    pub(super) graphics_device_draw_user_primitives:
        sys::cna_graphics_device_draw_user_primitives_fn,
    pub(super) graphics_device_draw_user_indexed_primitives:
        sys::cna_graphics_device_draw_user_indexed_primitives_fn,
    pub(super) graphics_device_set_render_targets: sys::cna_graphics_device_set_render_targets_fn,
    pub(super) graphics_device_get_render_target_count:
        sys::cna_graphics_device_get_render_target_count_fn,
    pub(super) graphics_device_copy_render_targets: sys::cna_graphics_device_copy_render_targets_fn,
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
    pub(super) texturecube_create: sys::cna_texturecube_create_fn,
    pub(super) texturecube_destroy: sys::cna_texturecube_destroy_fn,
    pub(super) texturecube_get_info: sys::cna_texturecube_get_info_fn,
    pub(super) texturecube_set_data: sys::cna_texturecube_set_data_fn,
    pub(super) texturecube_get_data: sys::cna_texturecube_get_data_fn,
    pub(super) render_target2d_create: sys::cna_render_target2d_create_fn,
    pub(super) render_target_cube_create: sys::cna_render_target_cube_create_fn,
    pub(super) render_target_get_info: sys::cna_render_target_get_info_fn,
    pub(super) render_target_destroy: sys::cna_render_target_destroy_fn,
    pub(super) vertex_declaration_create_with_stride:
        sys::cna_vertex_declaration_create_with_stride_fn,
    pub(super) vertex_declaration_destroy: sys::cna_vertex_declaration_destroy_fn,
    pub(super) vertex_buffer_binding_init: sys::cna_vertex_buffer_binding_init_fn,
    pub(super) vertex_buffer_create: sys::cna_vertex_buffer_create_fn,
    pub(super) vertex_buffer_destroy: sys::cna_vertex_buffer_destroy_fn,
    pub(super) vertex_buffer_get_info: sys::cna_vertex_buffer_get_info_fn,
    pub(super) vertex_buffer_set_data: sys::cna_vertex_buffer_set_data_fn,
    pub(super) vertex_buffer_set_data_raw: sys::cna_vertex_buffer_set_data_raw_fn,
    pub(super) vertex_buffer_set_data_raw_at: sys::cna_vertex_buffer_set_data_raw_at_fn,
    pub(super) vertex_buffer_get_data_raw: sys::cna_vertex_buffer_get_data_raw_fn,
    pub(super) index_buffer_create: sys::cna_index_buffer_create_fn,
    pub(super) index_buffer_destroy: sys::cna_index_buffer_destroy_fn,
    pub(super) index_buffer_get_info: sys::cna_index_buffer_get_info_fn,
    pub(super) index_buffer_set_data: sys::cna_index_buffer_set_data_fn,
    pub(super) index_buffer_set_data_at: sys::cna_index_buffer_set_data_at_fn,
    pub(super) index_buffer_get_data: sys::cna_index_buffer_get_data_fn,
    pub(super) sprite_batch_create: sys::cna_sprite_batch_create_fn,
    pub(super) sprite_batch_begin: sys::cna_sprite_batch_begin_fn,
    pub(super) sprite_batch_begin_with_states: sys::cna_sprite_batch_begin_with_states_fn,
    pub(super) sprite_batch_begin_with_effect: sys::cna_sprite_batch_begin_with_effect_fn,
    pub(super) sprite_batch_submit_many: sys::cna_sprite_batch_submit_many_fn,
    pub(super) sprite_batch_end: sys::cna_sprite_batch_end_fn,
    pub(super) sprite_batch_destroy: sys::cna_sprite_batch_destroy_fn,
    pub(super) sprite_batch_draw_string: sys::cna_sprite_batch_draw_string_fn,
    pub(super) sprite_font_create: sys::cna_sprite_font_create_fn,
    pub(super) sprite_font_get_info: sys::cna_sprite_font_get_info_fn,
    pub(super) sprite_font_copy_characters: sys::cna_sprite_font_copy_characters_fn,
    pub(super) sprite_font_copy_glyphs: sys::cna_sprite_font_copy_glyphs_fn,
    pub(super) sprite_font_set_default_character: sys::cna_sprite_font_set_default_character_fn,
    pub(super) sprite_font_set_line_spacing: sys::cna_sprite_font_set_line_spacing_fn,
    pub(super) sprite_font_set_spacing: sys::cna_sprite_font_set_spacing_fn,
    pub(super) sprite_font_measure_utf8: sys::cna_sprite_font_measure_utf8_fn,
    pub(super) sprite_font_destroy: sys::cna_sprite_font_destroy_fn,
    pub(super) effect_create_empty: sys::cna_effect_create_empty_fn,
    pub(super) effect_create_compiled: sys::cna_effect_create_compiled_fn,
    pub(super) effect_material_create: sys::cna_effect_material_create_fn,
    pub(super) effect_destroy: sys::cna_effect_destroy_fn,
    pub(super) effect_clone: sys::cna_effect_clone_fn,
    pub(super) effect_dispose: sys::cna_effect_dispose_fn,
    pub(super) effect_apply: sys::cna_effect_apply_fn,
    pub(super) effect_get_parameters: sys::cna_effect_get_parameters_fn,
    pub(super) effect_get_techniques: sys::cna_effect_get_techniques_fn,
    pub(super) effect_get_current_technique: sys::cna_effect_get_current_technique_fn,
    pub(super) effect_set_current_technique: sys::cna_effect_set_current_technique_fn,
    pub(super) effect_annotation_create: sys::cna_effect_annotation_create_fn,
    pub(super) effect_annotation_destroy: sys::cna_effect_annotation_destroy_fn,
    pub(super) effect_annotation_get_info: sys::cna_effect_annotation_get_info_fn,
    pub(super) effect_annotation_get_name_byte_count:
        sys::cna_effect_annotation_get_name_byte_count_fn,
    pub(super) effect_annotation_copy_name: sys::cna_effect_annotation_copy_name_fn,
    pub(super) effect_annotation_get_semantic_byte_count:
        sys::cna_effect_annotation_get_semantic_byte_count_fn,
    pub(super) effect_annotation_copy_semantic: sys::cna_effect_annotation_copy_semantic_fn,
    pub(super) effect_annotation_get_value_boolean: sys::cna_effect_annotation_get_value_boolean_fn,
    pub(super) effect_annotation_get_value_int32: sys::cna_effect_annotation_get_value_int32_fn,
    pub(super) effect_annotation_get_value_single: sys::cna_effect_annotation_get_value_single_fn,
    pub(super) effect_annotation_get_value_string_byte_count:
        sys::cna_effect_annotation_get_value_string_byte_count_fn,
    pub(super) effect_annotation_copy_value_string: sys::cna_effect_annotation_copy_value_string_fn,
    pub(super) effect_annotation_get_value_vector2: sys::cna_effect_annotation_get_value_vector2_fn,
    pub(super) effect_annotation_get_value_vector3: sys::cna_effect_annotation_get_value_vector3_fn,
    pub(super) effect_annotation_get_value_vector4: sys::cna_effect_annotation_get_value_vector4_fn,
    pub(super) effect_annotation_get_value_matrix: sys::cna_effect_annotation_get_value_matrix_fn,
    pub(super) effect_annotation_collection_destroy:
        sys::cna_effect_annotation_collection_destroy_fn,
    pub(super) effect_annotation_collection_add: sys::cna_effect_annotation_collection_add_fn,
    pub(super) effect_annotation_collection_get_count:
        sys::cna_effect_annotation_collection_get_count_fn,
    pub(super) effect_annotation_collection_get_at: sys::cna_effect_annotation_collection_get_at_fn,
    pub(super) effect_annotation_collection_find: sys::cna_effect_annotation_collection_find_fn,
    pub(super) effect_parameter_destroy: sys::cna_effect_parameter_destroy_fn,
    pub(super) effect_parameter_get_info: sys::cna_effect_parameter_get_info_fn,
    pub(super) effect_parameter_get_name_byte_count:
        sys::cna_effect_parameter_get_name_byte_count_fn,
    pub(super) effect_parameter_copy_name: sys::cna_effect_parameter_copy_name_fn,
    pub(super) effect_parameter_get_semantic_byte_count:
        sys::cna_effect_parameter_get_semantic_byte_count_fn,
    pub(super) effect_parameter_copy_semantic: sys::cna_effect_parameter_copy_semantic_fn,
    pub(super) effect_parameter_get_elements: sys::cna_effect_parameter_get_elements_fn,
    pub(super) effect_parameter_get_structure_members:
        sys::cna_effect_parameter_get_structure_members_fn,
    pub(super) effect_parameter_get_annotations: sys::cna_effect_parameter_get_annotations_fn,
    pub(super) effect_parameter_get_value: sys::cna_effect_parameter_get_value_fn,
    pub(super) effect_parameter_get_values: sys::cna_effect_parameter_get_values_fn,
    pub(super) effect_parameter_set_value: sys::cna_effect_parameter_set_value_fn,
    pub(super) effect_parameter_set_values: sys::cna_effect_parameter_set_values_fn,
    pub(super) effect_parameter_get_value_string_byte_count:
        sys::cna_effect_parameter_get_value_string_byte_count_fn,
    pub(super) effect_parameter_copy_value_string: sys::cna_effect_parameter_copy_value_string_fn,
    pub(super) effect_parameter_set_value_string: sys::cna_effect_parameter_set_value_string_fn,
    pub(super) effect_parameter_get_value_texture: sys::cna_effect_parameter_get_value_texture_fn,
    pub(super) effect_parameter_set_value_texture: sys::cna_effect_parameter_set_value_texture_fn,
    pub(super) effect_parameter_collection_destroy: sys::cna_effect_parameter_collection_destroy_fn,
    pub(super) effect_parameter_collection_add_create:
        sys::cna_effect_parameter_collection_add_create_fn,
    pub(super) effect_parameter_collection_get_count:
        sys::cna_effect_parameter_collection_get_count_fn,
    pub(super) effect_parameter_collection_get_at: sys::cna_effect_parameter_collection_get_at_fn,
    pub(super) effect_parameter_collection_find_name:
        sys::cna_effect_parameter_collection_find_name_fn,
    pub(super) effect_parameter_collection_find_semantic:
        sys::cna_effect_parameter_collection_find_semantic_fn,
    pub(super) effect_pass_destroy: sys::cna_effect_pass_destroy_fn,
    pub(super) effect_pass_get_name_byte_count: sys::cna_effect_pass_get_name_byte_count_fn,
    pub(super) effect_pass_copy_name: sys::cna_effect_pass_copy_name_fn,
    pub(super) effect_pass_get_annotations: sys::cna_effect_pass_get_annotations_fn,
    pub(super) effect_pass_apply: sys::cna_effect_pass_apply_fn,
    pub(super) effect_pass_collection_destroy: sys::cna_effect_pass_collection_destroy_fn,
    pub(super) effect_pass_collection_add_create: sys::cna_effect_pass_collection_add_create_fn,
    pub(super) effect_pass_collection_get_count: sys::cna_effect_pass_collection_get_count_fn,
    pub(super) effect_pass_collection_get_at: sys::cna_effect_pass_collection_get_at_fn,
    pub(super) effect_pass_collection_find: sys::cna_effect_pass_collection_find_fn,
    pub(super) effect_technique_destroy: sys::cna_effect_technique_destroy_fn,
    pub(super) effect_technique_get_name_byte_count:
        sys::cna_effect_technique_get_name_byte_count_fn,
    pub(super) effect_technique_copy_name: sys::cna_effect_technique_copy_name_fn,
    pub(super) effect_technique_get_passes: sys::cna_effect_technique_get_passes_fn,
    pub(super) effect_technique_get_annotations: sys::cna_effect_technique_get_annotations_fn,
    pub(super) effect_technique_collection_destroy: sys::cna_effect_technique_collection_destroy_fn,
    pub(super) effect_technique_collection_add_named:
        sys::cna_effect_technique_collection_add_named_fn,
    pub(super) effect_technique_collection_get_count:
        sys::cna_effect_technique_collection_get_count_fn,
    pub(super) effect_technique_collection_get_at: sys::cna_effect_technique_collection_get_at_fn,
    pub(super) effect_technique_collection_find: sys::cna_effect_technique_collection_find_fn,
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
            game_run_one_frame: symbol!("cna_game_run_one_frame", sys::cna_game_run_one_frame_fn),
            game_run: symbol!("cna_game_run", sys::cna_game_run_fn),
            game_request_exit: symbol!("cna_game_request_exit", sys::cna_game_request_exit_fn),
            game_get_is_active: symbol!("cna_game_get_is_active", sys::cna_game_get_is_active_fn),
            game_get_is_mouse_visible: symbol!(
                "cna_game_get_is_mouse_visible",
                sys::cna_game_get_is_mouse_visible_fn
            ),
            game_set_is_mouse_visible: symbol!(
                "cna_game_set_is_mouse_visible",
                sys::cna_game_set_is_mouse_visible_fn
            ),
            game_get_is_fixed_time_step: symbol!(
                "cna_game_get_is_fixed_time_step",
                sys::cna_game_get_is_fixed_time_step_fn
            ),
            game_set_is_fixed_time_step: symbol!(
                "cna_game_set_is_fixed_time_step",
                sys::cna_game_set_is_fixed_time_step_fn
            ),
            game_get_target_elapsed_time_ticks: symbol!(
                "cna_game_get_target_elapsed_time_ticks",
                sys::cna_game_get_target_elapsed_time_ticks_fn
            ),
            game_set_target_elapsed_time_ticks: symbol!(
                "cna_game_set_target_elapsed_time_ticks",
                sys::cna_game_set_target_elapsed_time_ticks_fn
            ),
            game_get_inactive_sleep_time_ticks: symbol!(
                "cna_game_get_inactive_sleep_time_ticks",
                sys::cna_game_get_inactive_sleep_time_ticks_fn
            ),
            game_set_inactive_sleep_time_ticks: symbol!(
                "cna_game_set_inactive_sleep_time_ticks",
                sys::cna_game_set_inactive_sleep_time_ticks_fn
            ),
            game_reset_elapsed_time: symbol!(
                "cna_game_reset_elapsed_time",
                sys::cna_game_reset_elapsed_time_fn
            ),
            game_suppress_draw: symbol!("cna_game_suppress_draw", sys::cna_game_suppress_draw_fn),
            game_tick: symbol!("cna_game_tick", sys::cna_game_tick_fn),
            game_set_window_title: symbol!(
                "cna_game_set_window_title",
                sys::cna_game_set_window_title_fn
            ),
            game_subscribe: symbol!("cna_game_subscribe", sys::cna_game_subscribe_fn),
            game_window_subscribe: symbol!(
                "cna_game_window_subscribe",
                sys::cna_game_window_subscribe_fn
            ),
            game_unsubscribe: symbol!("cna_game_unsubscribe", sys::cna_game_unsubscribe_fn),
            game_destroy: symbol!("cna_game_destroy", sys::cna_game_destroy_fn),
            game_window_get_allow_user_resizing: symbol!(
                "cna_game_window_get_allow_user_resizing",
                sys::cna_game_window_get_allow_user_resizing_fn
            ),
            game_window_set_allow_user_resizing: symbol!(
                "cna_game_window_set_allow_user_resizing",
                sys::cna_game_window_set_allow_user_resizing_fn
            ),
            game_window_get_client_bounds: symbol!(
                "cna_game_window_get_client_bounds",
                sys::cna_game_window_get_client_bounds_fn
            ),
            game_window_get_current_orientation: symbol!(
                "cna_game_window_get_current_orientation",
                sys::cna_game_window_get_current_orientation_fn
            ),
            game_window_get_native_handle: symbol!(
                "cna_game_window_get_native_handle_ext",
                sys::cna_game_window_get_native_handle_ext_fn
            ),
            game_window_get_screen_device_name_size: symbol!(
                "cna_game_window_get_screen_device_name_size",
                sys::cna_game_window_get_screen_device_name_size_fn
            ),
            game_window_copy_screen_device_name: symbol!(
                "cna_game_window_copy_screen_device_name",
                sys::cna_game_window_copy_screen_device_name_fn
            ),
            game_window_get_title_size: symbol!(
                "cna_game_window_get_title_size",
                sys::cna_game_window_get_title_size_fn
            ),
            game_window_copy_title: symbol!(
                "cna_game_window_copy_title",
                sys::cna_game_window_copy_title_fn
            ),
            game_window_begin_screen_device_change: symbol!(
                "cna_game_window_begin_screen_device_change",
                sys::cna_game_window_begin_screen_device_change_fn
            ),
            game_window_end_screen_device_change: symbol!(
                "cna_game_window_end_screen_device_change",
                sys::cna_game_window_end_screen_device_change_fn
            ),
            game_get_graphics_device: symbol!(
                "cna_game_get_graphics_device",
                sys::cna_game_get_graphics_device_fn
            ),
            graphics_device_get_status: symbol!(
                "cna_graphics_device_get_status",
                sys::cna_graphics_device_get_status_fn
            ),
            graphics_device_get_graphics_profile: symbol!(
                "cna_graphics_device_get_graphics_profile",
                sys::cna_graphics_device_get_graphics_profile_fn
            ),
            graphics_device_get_presentation_parameters: symbol!(
                "cna_graphics_device_get_presentation_parameters",
                sys::cna_graphics_device_get_presentation_parameters_fn
            ),
            graphics_device_get_display_mode: symbol!(
                "cna_graphics_device_get_display_mode",
                sys::cna_graphics_device_get_display_mode_fn
            ),
            graphics_device_get_blend_state: symbol!(
                "cna_graphics_device_get_blend_state",
                sys::cna_graphics_device_get_blend_state_fn
            ),
            graphics_device_get_depth_stencil_state: symbol!(
                "cna_graphics_device_get_depth_stencil_state",
                sys::cna_graphics_device_get_depth_stencil_state_fn
            ),
            graphics_device_get_rasterizer_state: symbol!(
                "cna_graphics_device_get_rasterizer_state",
                sys::cna_graphics_device_get_rasterizer_state_fn
            ),
            graphics_device_get_sampler_state: symbol!(
                "cna_graphics_device_get_sampler_state",
                sys::cna_graphics_device_get_sampler_state_fn
            ),
            graphics_device_set_sampler_state: symbol!(
                "cna_graphics_device_set_sampler_state",
                sys::cna_graphics_device_set_sampler_state_fn
            ),
            graphics_device_get_texture: symbol!(
                "cna_graphics_device_get_texture",
                sys::cna_graphics_device_get_texture_fn
            ),
            graphics_device_set_texture: symbol!(
                "cna_graphics_device_set_texture",
                sys::cna_graphics_device_set_texture_fn
            ),
            graphics_device_get_adapter_index: symbol!(
                "cna_graphics_device_get_adapter_index",
                sys::cna_graphics_device_get_adapter_index_fn
            ),
            graphics_adapter_get_count: symbol!(
                "cna_graphics_adapter_get_count",
                sys::cna_graphics_adapter_get_count_fn
            ),
            graphics_adapter_get_info: symbol!(
                "cna_graphics_adapter_get_info",
                sys::cna_graphics_adapter_get_info_fn
            ),
            graphics_adapter_copy_description: symbol!(
                "cna_graphics_adapter_copy_description",
                sys::cna_graphics_adapter_copy_description_fn
            ),
            graphics_adapter_copy_device_name: symbol!(
                "cna_graphics_adapter_copy_device_name",
                sys::cna_graphics_adapter_copy_device_name_fn
            ),
            graphics_adapter_get_current_display_mode: symbol!(
                "cna_graphics_adapter_get_current_display_mode",
                sys::cna_graphics_adapter_get_current_display_mode_fn
            ),
            graphics_adapter_get_display_mode_count: symbol!(
                "cna_graphics_adapter_get_display_mode_count",
                sys::cna_graphics_adapter_get_display_mode_count_fn
            ),
            graphics_adapter_copy_display_modes: symbol!(
                "cna_graphics_adapter_copy_display_modes",
                sys::cna_graphics_adapter_copy_display_modes_fn
            ),
            graphics_adapter_set_device_preferences: symbol!(
                "cna_graphics_adapter_set_device_preferences",
                sys::cna_graphics_adapter_set_device_preferences_fn
            ),
            graphics_adapter_is_profile_supported: symbol!(
                "cna_graphics_adapter_is_profile_supported",
                sys::cna_graphics_adapter_is_profile_supported_fn
            ),
            graphics_adapter_query_render_target_format: symbol!(
                "cna_graphics_adapter_query_render_target_format",
                sys::cna_graphics_adapter_query_render_target_format_fn
            ),
            graphics_adapter_query_backbuffer_format: symbol!(
                "cna_graphics_adapter_query_backbuffer_format",
                sys::cna_graphics_adapter_query_backbuffer_format_fn
            ),
            graphics_adapter_get_native_monitor_handle: symbol!(
                "cna_graphics_adapter_get_native_monitor_handle",
                sys::cna_graphics_adapter_get_native_monitor_handle_fn
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
            graphics_device_reset: symbol!(
                "cna_graphics_device_reset",
                sys::cna_graphics_device_reset_fn
            ),
            graphics_device_reset_with_parameters: symbol!(
                "cna_graphics_device_reset_with_parameters",
                sys::cna_graphics_device_reset_with_parameters_fn
            ),
            graphics_device_get_backbuffer_data_window: symbol!(
                "cna_graphics_device_get_backbuffer_data_window",
                sys::cna_graphics_device_get_backbuffer_data_window_fn
            ),
            graphics_device_clear_rgba: symbol!(
                "cna_graphics_device_clear_rgba",
                sys::cna_graphics_device_clear_rgba_fn
            ),
            graphics_device_set_vertex_buffer: symbol!(
                "cna_graphics_device_set_vertex_buffer",
                sys::cna_graphics_device_set_vertex_buffer_fn
            ),
            graphics_device_set_vertex_buffer_offset: symbol!(
                "cna_graphics_device_set_vertex_buffer_offset",
                sys::cna_graphics_device_set_vertex_buffer_offset_fn
            ),
            graphics_device_set_vertex_buffers: symbol!(
                "cna_graphics_device_set_vertex_buffers",
                sys::cna_graphics_device_set_vertex_buffers_fn
            ),
            graphics_device_get_vertex_buffer_count: symbol!(
                "cna_graphics_device_get_vertex_buffer_count",
                sys::cna_graphics_device_get_vertex_buffer_count_fn
            ),
            graphics_device_copy_vertex_buffers: symbol!(
                "cna_graphics_device_copy_vertex_buffers",
                sys::cna_graphics_device_copy_vertex_buffers_fn
            ),
            graphics_device_get_vertex_buffer: symbol!(
                "cna_graphics_device_get_vertex_buffer",
                sys::cna_graphics_device_get_vertex_buffer_fn
            ),
            graphics_device_set_index_buffer: symbol!(
                "cna_graphics_device_set_index_buffer",
                sys::cna_graphics_device_set_index_buffer_fn
            ),
            graphics_device_get_index_buffer: symbol!(
                "cna_graphics_device_get_index_buffer",
                sys::cna_graphics_device_get_index_buffer_fn
            ),
            graphics_device_draw_primitives: symbol!(
                "cna_graphics_device_draw_primitives",
                sys::cna_graphics_device_draw_primitives_fn
            ),
            graphics_device_draw_indexed_primitives: symbol!(
                "cna_graphics_device_draw_indexed_primitives",
                sys::cna_graphics_device_draw_indexed_primitives_fn
            ),
            graphics_device_draw_instanced_primitives: symbol!(
                "cna_graphics_device_draw_instanced_primitives",
                sys::cna_graphics_device_draw_instanced_primitives_fn
            ),
            graphics_device_draw_user_primitives: symbol!(
                "cna_graphics_device_draw_user_primitives",
                sys::cna_graphics_device_draw_user_primitives_fn
            ),
            graphics_device_draw_user_indexed_primitives: symbol!(
                "cna_graphics_device_draw_user_indexed_primitives",
                sys::cna_graphics_device_draw_user_indexed_primitives_fn
            ),
            graphics_device_set_render_targets: symbol!(
                "cna_graphics_device_set_render_targets",
                sys::cna_graphics_device_set_render_targets_fn
            ),
            graphics_device_get_render_target_count: symbol!(
                "cna_graphics_device_get_render_target_count",
                sys::cna_graphics_device_get_render_target_count_fn
            ),
            graphics_device_copy_render_targets: symbol!(
                "cna_graphics_device_copy_render_targets",
                sys::cna_graphics_device_copy_render_targets_fn
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
            texturecube_create: symbol!("cna_texturecube_create", sys::cna_texturecube_create_fn),
            texturecube_destroy: symbol!(
                "cna_texturecube_destroy",
                sys::cna_texturecube_destroy_fn
            ),
            texturecube_get_info: symbol!(
                "cna_texturecube_get_info",
                sys::cna_texturecube_get_info_fn
            ),
            texturecube_set_data: symbol!(
                "cna_texturecube_set_data",
                sys::cna_texturecube_set_data_fn
            ),
            texturecube_get_data: symbol!(
                "cna_texturecube_get_data",
                sys::cna_texturecube_get_data_fn
            ),
            render_target2d_create: symbol!(
                "cna_render_target2d_create",
                sys::cna_render_target2d_create_fn
            ),
            render_target_cube_create: symbol!(
                "cna_render_target_cube_create",
                sys::cna_render_target_cube_create_fn
            ),
            render_target_get_info: symbol!(
                "cna_render_target_get_info",
                sys::cna_render_target_get_info_fn
            ),
            render_target_destroy: symbol!(
                "cna_render_target_destroy",
                sys::cna_render_target_destroy_fn
            ),
            vertex_declaration_create_with_stride: symbol!(
                "cna_vertex_declaration_create_with_stride",
                sys::cna_vertex_declaration_create_with_stride_fn
            ),
            vertex_declaration_destroy: symbol!(
                "cna_vertex_declaration_destroy",
                sys::cna_vertex_declaration_destroy_fn
            ),
            vertex_buffer_binding_init: symbol!(
                "cna_vertex_buffer_binding_init",
                sys::cna_vertex_buffer_binding_init_fn
            ),
            vertex_buffer_create: symbol!(
                "cna_vertex_buffer_create",
                sys::cna_vertex_buffer_create_fn
            ),
            vertex_buffer_destroy: symbol!(
                "cna_vertex_buffer_destroy",
                sys::cna_vertex_buffer_destroy_fn
            ),
            vertex_buffer_get_info: symbol!(
                "cna_vertex_buffer_get_info",
                sys::cna_vertex_buffer_get_info_fn
            ),
            vertex_buffer_set_data: symbol!(
                "cna_vertex_buffer_set_data",
                sys::cna_vertex_buffer_set_data_fn
            ),
            vertex_buffer_set_data_raw: symbol!(
                "cna_vertex_buffer_set_data_raw",
                sys::cna_vertex_buffer_set_data_raw_fn
            ),
            vertex_buffer_set_data_raw_at: symbol!(
                "cna_vertex_buffer_set_data_raw_at",
                sys::cna_vertex_buffer_set_data_raw_at_fn
            ),
            vertex_buffer_get_data_raw: symbol!(
                "cna_vertex_buffer_get_data_raw",
                sys::cna_vertex_buffer_get_data_raw_fn
            ),
            index_buffer_create: symbol!(
                "cna_index_buffer_create",
                sys::cna_index_buffer_create_fn
            ),
            index_buffer_destroy: symbol!(
                "cna_index_buffer_destroy",
                sys::cna_index_buffer_destroy_fn
            ),
            index_buffer_get_info: symbol!(
                "cna_index_buffer_get_info",
                sys::cna_index_buffer_get_info_fn
            ),
            index_buffer_set_data: symbol!(
                "cna_index_buffer_set_data",
                sys::cna_index_buffer_set_data_fn
            ),
            index_buffer_set_data_at: symbol!(
                "cna_index_buffer_set_data_at",
                sys::cna_index_buffer_set_data_at_fn
            ),
            index_buffer_get_data: symbol!(
                "cna_index_buffer_get_data",
                sys::cna_index_buffer_get_data_fn
            ),
            sprite_batch_create: symbol!(
                "cna_sprite_batch_create",
                sys::cna_sprite_batch_create_fn
            ),
            sprite_batch_begin: symbol!("cna_sprite_batch_begin", sys::cna_sprite_batch_begin_fn),
            sprite_batch_begin_with_states: symbol!(
                "cna_sprite_batch_begin_with_states",
                sys::cna_sprite_batch_begin_with_states_fn
            ),
            sprite_batch_begin_with_effect: symbol!(
                "cna_sprite_batch_begin_with_effect",
                sys::cna_sprite_batch_begin_with_effect_fn
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
            sprite_batch_draw_string: symbol!(
                "cna_sprite_batch_draw_string",
                sys::cna_sprite_batch_draw_string_fn
            ),
            sprite_font_create: symbol!("cna_sprite_font_create", sys::cna_sprite_font_create_fn),
            sprite_font_get_info: symbol!(
                "cna_sprite_font_get_info",
                sys::cna_sprite_font_get_info_fn
            ),
            sprite_font_copy_characters: symbol!(
                "cna_sprite_font_copy_characters",
                sys::cna_sprite_font_copy_characters_fn
            ),
            sprite_font_copy_glyphs: symbol!(
                "cna_sprite_font_copy_glyphs",
                sys::cna_sprite_font_copy_glyphs_fn
            ),
            sprite_font_set_default_character: symbol!(
                "cna_sprite_font_set_default_character",
                sys::cna_sprite_font_set_default_character_fn
            ),
            sprite_font_set_line_spacing: symbol!(
                "cna_sprite_font_set_line_spacing",
                sys::cna_sprite_font_set_line_spacing_fn
            ),
            sprite_font_set_spacing: symbol!(
                "cna_sprite_font_set_spacing",
                sys::cna_sprite_font_set_spacing_fn
            ),
            sprite_font_measure_utf8: symbol!(
                "cna_sprite_font_measure_utf8",
                sys::cna_sprite_font_measure_utf8_fn
            ),
            sprite_font_destroy: symbol!(
                "cna_sprite_font_destroy",
                sys::cna_sprite_font_destroy_fn
            ),
            effect_create_empty: symbol!(
                "cna_effect_create_empty",
                sys::cna_effect_create_empty_fn
            ),
            effect_create_compiled: symbol!(
                "cna_effect_create_compiled",
                sys::cna_effect_create_compiled_fn
            ),
            effect_material_create: symbol!(
                "cna_effect_material_create",
                sys::cna_effect_material_create_fn
            ),
            effect_destroy: symbol!("cna_effect_destroy", sys::cna_effect_destroy_fn),
            effect_clone: symbol!("cna_effect_clone", sys::cna_effect_clone_fn),
            effect_dispose: symbol!("cna_effect_dispose", sys::cna_effect_dispose_fn),
            effect_apply: symbol!("cna_effect_apply", sys::cna_effect_apply_fn),
            effect_get_parameters: symbol!(
                "cna_effect_get_parameters",
                sys::cna_effect_get_parameters_fn
            ),
            effect_get_techniques: symbol!(
                "cna_effect_get_techniques",
                sys::cna_effect_get_techniques_fn
            ),
            effect_get_current_technique: symbol!(
                "cna_effect_get_current_technique",
                sys::cna_effect_get_current_technique_fn
            ),
            effect_set_current_technique: symbol!(
                "cna_effect_set_current_technique",
                sys::cna_effect_set_current_technique_fn
            ),
            effect_annotation_create: symbol!(
                "cna_effect_annotation_create",
                sys::cna_effect_annotation_create_fn
            ),
            effect_annotation_destroy: symbol!(
                "cna_effect_annotation_destroy",
                sys::cna_effect_annotation_destroy_fn
            ),
            effect_annotation_get_info: symbol!(
                "cna_effect_annotation_get_info",
                sys::cna_effect_annotation_get_info_fn
            ),
            effect_annotation_get_name_byte_count: symbol!(
                "cna_effect_annotation_get_name_byte_count",
                sys::cna_effect_annotation_get_name_byte_count_fn
            ),
            effect_annotation_copy_name: symbol!(
                "cna_effect_annotation_copy_name",
                sys::cna_effect_annotation_copy_name_fn
            ),
            effect_annotation_get_semantic_byte_count: symbol!(
                "cna_effect_annotation_get_semantic_byte_count",
                sys::cna_effect_annotation_get_semantic_byte_count_fn
            ),
            effect_annotation_copy_semantic: symbol!(
                "cna_effect_annotation_copy_semantic",
                sys::cna_effect_annotation_copy_semantic_fn
            ),
            effect_annotation_get_value_boolean: symbol!(
                "cna_effect_annotation_get_value_boolean",
                sys::cna_effect_annotation_get_value_boolean_fn
            ),
            effect_annotation_get_value_int32: symbol!(
                "cna_effect_annotation_get_value_int32",
                sys::cna_effect_annotation_get_value_int32_fn
            ),
            effect_annotation_get_value_single: symbol!(
                "cna_effect_annotation_get_value_single",
                sys::cna_effect_annotation_get_value_single_fn
            ),
            effect_annotation_get_value_string_byte_count: symbol!(
                "cna_effect_annotation_get_value_string_byte_count",
                sys::cna_effect_annotation_get_value_string_byte_count_fn
            ),
            effect_annotation_copy_value_string: symbol!(
                "cna_effect_annotation_copy_value_string",
                sys::cna_effect_annotation_copy_value_string_fn
            ),
            effect_annotation_get_value_vector2: symbol!(
                "cna_effect_annotation_get_value_vector2",
                sys::cna_effect_annotation_get_value_vector2_fn
            ),
            effect_annotation_get_value_vector3: symbol!(
                "cna_effect_annotation_get_value_vector3",
                sys::cna_effect_annotation_get_value_vector3_fn
            ),
            effect_annotation_get_value_vector4: symbol!(
                "cna_effect_annotation_get_value_vector4",
                sys::cna_effect_annotation_get_value_vector4_fn
            ),
            effect_annotation_get_value_matrix: symbol!(
                "cna_effect_annotation_get_value_matrix",
                sys::cna_effect_annotation_get_value_matrix_fn
            ),
            effect_annotation_collection_destroy: symbol!(
                "cna_effect_annotation_collection_destroy",
                sys::cna_effect_annotation_collection_destroy_fn
            ),
            effect_annotation_collection_add: symbol!(
                "cna_effect_annotation_collection_add",
                sys::cna_effect_annotation_collection_add_fn
            ),
            effect_annotation_collection_get_count: symbol!(
                "cna_effect_annotation_collection_get_count",
                sys::cna_effect_annotation_collection_get_count_fn
            ),
            effect_annotation_collection_get_at: symbol!(
                "cna_effect_annotation_collection_get_at",
                sys::cna_effect_annotation_collection_get_at_fn
            ),
            effect_annotation_collection_find: symbol!(
                "cna_effect_annotation_collection_find",
                sys::cna_effect_annotation_collection_find_fn
            ),
            effect_parameter_destroy: symbol!(
                "cna_effect_parameter_destroy",
                sys::cna_effect_parameter_destroy_fn
            ),
            effect_parameter_get_info: symbol!(
                "cna_effect_parameter_get_info",
                sys::cna_effect_parameter_get_info_fn
            ),
            effect_parameter_get_name_byte_count: symbol!(
                "cna_effect_parameter_get_name_byte_count",
                sys::cna_effect_parameter_get_name_byte_count_fn
            ),
            effect_parameter_copy_name: symbol!(
                "cna_effect_parameter_copy_name",
                sys::cna_effect_parameter_copy_name_fn
            ),
            effect_parameter_get_semantic_byte_count: symbol!(
                "cna_effect_parameter_get_semantic_byte_count",
                sys::cna_effect_parameter_get_semantic_byte_count_fn
            ),
            effect_parameter_copy_semantic: symbol!(
                "cna_effect_parameter_copy_semantic",
                sys::cna_effect_parameter_copy_semantic_fn
            ),
            effect_parameter_get_elements: symbol!(
                "cna_effect_parameter_get_elements",
                sys::cna_effect_parameter_get_elements_fn
            ),
            effect_parameter_get_structure_members: symbol!(
                "cna_effect_parameter_get_structure_members",
                sys::cna_effect_parameter_get_structure_members_fn
            ),
            effect_parameter_get_annotations: symbol!(
                "cna_effect_parameter_get_annotations",
                sys::cna_effect_parameter_get_annotations_fn
            ),
            effect_parameter_get_value: symbol!(
                "cna_effect_parameter_get_value",
                sys::cna_effect_parameter_get_value_fn
            ),
            effect_parameter_get_values: symbol!(
                "cna_effect_parameter_get_values",
                sys::cna_effect_parameter_get_values_fn
            ),
            effect_parameter_set_value: symbol!(
                "cna_effect_parameter_set_value",
                sys::cna_effect_parameter_set_value_fn
            ),
            effect_parameter_set_values: symbol!(
                "cna_effect_parameter_set_values",
                sys::cna_effect_parameter_set_values_fn
            ),
            effect_parameter_get_value_string_byte_count: symbol!(
                "cna_effect_parameter_get_value_string_byte_count",
                sys::cna_effect_parameter_get_value_string_byte_count_fn
            ),
            effect_parameter_copy_value_string: symbol!(
                "cna_effect_parameter_copy_value_string",
                sys::cna_effect_parameter_copy_value_string_fn
            ),
            effect_parameter_set_value_string: symbol!(
                "cna_effect_parameter_set_value_string",
                sys::cna_effect_parameter_set_value_string_fn
            ),
            effect_parameter_get_value_texture: symbol!(
                "cna_effect_parameter_get_value_texture",
                sys::cna_effect_parameter_get_value_texture_fn
            ),
            effect_parameter_set_value_texture: symbol!(
                "cna_effect_parameter_set_value_texture",
                sys::cna_effect_parameter_set_value_texture_fn
            ),
            effect_parameter_collection_destroy: symbol!(
                "cna_effect_parameter_collection_destroy",
                sys::cna_effect_parameter_collection_destroy_fn
            ),
            effect_parameter_collection_add_create: symbol!(
                "cna_effect_parameter_collection_add_create",
                sys::cna_effect_parameter_collection_add_create_fn
            ),
            effect_parameter_collection_get_count: symbol!(
                "cna_effect_parameter_collection_get_count",
                sys::cna_effect_parameter_collection_get_count_fn
            ),
            effect_parameter_collection_get_at: symbol!(
                "cna_effect_parameter_collection_get_at",
                sys::cna_effect_parameter_collection_get_at_fn
            ),
            effect_parameter_collection_find_name: symbol!(
                "cna_effect_parameter_collection_find_name",
                sys::cna_effect_parameter_collection_find_name_fn
            ),
            effect_parameter_collection_find_semantic: symbol!(
                "cna_effect_parameter_collection_find_semantic",
                sys::cna_effect_parameter_collection_find_semantic_fn
            ),
            effect_pass_destroy: symbol!(
                "cna_effect_pass_destroy",
                sys::cna_effect_pass_destroy_fn
            ),
            effect_pass_get_name_byte_count: symbol!(
                "cna_effect_pass_get_name_byte_count",
                sys::cna_effect_pass_get_name_byte_count_fn
            ),
            effect_pass_copy_name: symbol!(
                "cna_effect_pass_copy_name",
                sys::cna_effect_pass_copy_name_fn
            ),
            effect_pass_get_annotations: symbol!(
                "cna_effect_pass_get_annotations",
                sys::cna_effect_pass_get_annotations_fn
            ),
            effect_pass_apply: symbol!("cna_effect_pass_apply", sys::cna_effect_pass_apply_fn),
            effect_pass_collection_destroy: symbol!(
                "cna_effect_pass_collection_destroy",
                sys::cna_effect_pass_collection_destroy_fn
            ),
            effect_pass_collection_add_create: symbol!(
                "cna_effect_pass_collection_add_create",
                sys::cna_effect_pass_collection_add_create_fn
            ),
            effect_pass_collection_get_count: symbol!(
                "cna_effect_pass_collection_get_count",
                sys::cna_effect_pass_collection_get_count_fn
            ),
            effect_pass_collection_get_at: symbol!(
                "cna_effect_pass_collection_get_at",
                sys::cna_effect_pass_collection_get_at_fn
            ),
            effect_pass_collection_find: symbol!(
                "cna_effect_pass_collection_find",
                sys::cna_effect_pass_collection_find_fn
            ),
            effect_technique_destroy: symbol!(
                "cna_effect_technique_destroy",
                sys::cna_effect_technique_destroy_fn
            ),
            effect_technique_get_name_byte_count: symbol!(
                "cna_effect_technique_get_name_byte_count",
                sys::cna_effect_technique_get_name_byte_count_fn
            ),
            effect_technique_copy_name: symbol!(
                "cna_effect_technique_copy_name",
                sys::cna_effect_technique_copy_name_fn
            ),
            effect_technique_get_passes: symbol!(
                "cna_effect_technique_get_passes",
                sys::cna_effect_technique_get_passes_fn
            ),
            effect_technique_get_annotations: symbol!(
                "cna_effect_technique_get_annotations",
                sys::cna_effect_technique_get_annotations_fn
            ),
            effect_technique_collection_destroy: symbol!(
                "cna_effect_technique_collection_destroy",
                sys::cna_effect_technique_collection_destroy_fn
            ),
            effect_technique_collection_add_named: symbol!(
                "cna_effect_technique_collection_add_named",
                sys::cna_effect_technique_collection_add_named_fn
            ),
            effect_technique_collection_get_count: symbol!(
                "cna_effect_technique_collection_get_count",
                sys::cna_effect_technique_collection_get_count_fn
            ),
            effect_technique_collection_get_at: symbol!(
                "cna_effect_technique_collection_get_at",
                sys::cna_effect_technique_collection_get_at_fn
            ),
            effect_technique_collection_find: symbol!(
                "cna_effect_technique_collection_find",
                sys::cna_effect_technique_collection_find_fn
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
