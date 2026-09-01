//! Audited CNA function table and ABI-version-checked loading.

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::{CnaError, Result};

#[cfg(all(not(feature = "direct-link"), any(unix, windows)))]
use super::loader::{library_candidates, Library};
use super::loader::NativeSource;
use super::audio::AudioApi;
use super::engine::EngineApi;
use super::gamer_services::GamerServicesApi;
use super::net::NetApi;
use super::media::MediaApi;
use super::models::ModelsApi;
use super::runtime::RuntimeApi;

#[derive(Debug)]
pub(crate) struct Native {
    pub(super) _source: NativeSource,
    pub(super) audio: AudioApi,
    pub(crate) media: MediaApi,
    pub(crate) runtime: RuntimeApi,
    pub(crate) engine: EngineApi,
    pub(crate) models: ModelsApi,
    pub(crate) gamer_services: GamerServicesApi,
    /// Resolved with the library so a CNA build missing any Net route fails at
    /// load. The safe `Microsoft.Xna.Framework.Net` projection is the next
    /// slice of the wider profile and is what will read it.
    #[allow(dead_code)]
    pub(crate) net: NetApi,
    pub(super) error_get_last_info: sys::cna_error_get_last_info_fn,
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
    pub(super) graphics_device_create: sys::cna_graphics_device_create_fn,
    pub(super) graphics_device_destroy: sys::cna_graphics_device_destroy_fn,
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
    pub(super) graphics_device_clear_options: sys::cna_graphics_device_clear_options_fn,
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
    pub(super) occlusion_query_create: sys::cna_occlusion_query_create_fn,
    pub(super) occlusion_query_begin: sys::cna_occlusion_query_begin_fn,
    pub(super) occlusion_query_end: sys::cna_occlusion_query_end_fn,
    pub(super) occlusion_query_get_is_complete: sys::cna_occlusion_query_get_is_complete_fn,
    pub(super) occlusion_query_get_pixel_count: sys::cna_occlusion_query_get_pixel_count_fn,
    pub(super) occlusion_query_destroy: sys::cna_occlusion_query_destroy_fn,
    pub(super) graphics_device_set_render_targets: sys::cna_graphics_device_set_render_targets_fn,
    pub(super) graphics_device_get_render_target_count:
        sys::cna_graphics_device_get_render_target_count_fn,
    pub(super) graphics_device_copy_render_targets: sys::cna_graphics_device_copy_render_targets_fn,
    pub(super) graphics_device_get_renderer_info: sys::cna_graphics_device_get_renderer_info_fn,
    pub(super) graphics_device_feature_support:
        sys::cna_graphics_device_get_renderer_feature_support_ext_fn,
    pub(super) graphics_device_limit: sys::cna_graphics_device_get_renderer_limit_ext_fn,
    pub(super) graphics_device_format_support:
        sys::cna_graphics_device_get_surface_format_support_ext_fn,
    pub(super) graphics_device_capability_report_size:
        sys::cna_graphics_device_get_capability_report_size_ext_fn,
    pub(super) graphics_device_copy_capability_report:
        sys::cna_graphics_device_copy_capability_report_ext_fn,
    pub(super) graphics_device_shader_dialect: sys::cna_graphics_device_get_shader_dialect_ext_fn,
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
    pub(super) texture3d_create: sys::cna_texture3d_create_fn,
    pub(super) texture3d_destroy: sys::cna_texture3d_destroy_fn,
    pub(super) texture3d_get_info: sys::cna_texture3d_get_info_fn,
    pub(super) texture3d_set_data: sys::cna_texture3d_set_data_fn,
    pub(super) texture3d_get_data: sys::cna_texture3d_get_data_fn,
    pub(super) texturecube_create: sys::cna_texturecube_create_fn,
    pub(super) texturecube_destroy: sys::cna_texturecube_destroy_fn,
    pub(super) texturecube_get_info: sys::cna_texturecube_get_info_fn,
    pub(super) texturecube_set_data: sys::cna_texturecube_set_data_fn,
    pub(super) texturecube_get_data: sys::cna_texturecube_get_data_fn,
    pub(super) render_target2d_create: sys::cna_render_target2d_create_fn,
    pub(super) render_target_cube_create: sys::cna_render_target_cube_create_fn,
    pub(super) render_target_get_info: sys::cna_render_target_get_info_fn,
    pub(crate) render_target_destroy: sys::cna_render_target_destroy_fn,
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
    pub(super) vertex_buffer_set_data_raw_with_options:
        sys::cna_vertex_buffer_set_data_raw_with_options_fn,
    pub(super) vertex_buffer_set_data_raw_at_with_options:
        sys::cna_vertex_buffer_set_data_raw_at_with_options_fn,
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
    pub(crate) effect_destroy: sys::cna_effect_destroy_fn,
    pub(super) effect_clone: sys::cna_effect_clone_fn,
    pub(super) effect_dispose: sys::cna_effect_dispose_fn,
    pub(super) effect_apply: sys::cna_effect_apply_fn,
    pub(super) effect_get_parameters: sys::cna_effect_get_parameters_fn,
    pub(super) effect_get_techniques: sys::cna_effect_get_techniques_fn,
    pub(super) effect_get_current_technique: sys::cna_effect_get_current_technique_fn,
    pub(super) effect_set_current_technique: sys::cna_effect_set_current_technique_fn,
    pub(super) directional_light_create: sys::cna_directional_light_create_fn,
    pub(super) directional_light_destroy: sys::cna_directional_light_destroy_fn,
    pub(super) directional_light_get_diffuse_color: sys::cna_directional_light_get_diffuse_color_fn,
    pub(super) directional_light_set_diffuse_color: sys::cna_directional_light_set_diffuse_color_fn,
    pub(super) directional_light_get_direction: sys::cna_directional_light_get_direction_fn,
    pub(super) directional_light_set_direction: sys::cna_directional_light_set_direction_fn,
    pub(super) directional_light_get_specular_color:
        sys::cna_directional_light_get_specular_color_fn,
    pub(super) directional_light_set_specular_color:
        sys::cna_directional_light_set_specular_color_fn,
    pub(super) directional_light_get_enabled: sys::cna_directional_light_get_enabled_fn,
    pub(super) directional_light_set_enabled: sys::cna_directional_light_set_enabled_fn,
    pub(super) basic_effect_create: sys::cna_basic_effect_create_fn,
    pub(crate) graphics_device_clear_color_depth: sys::cna_graphics_device_clear_color_depth_fn,
    pub(crate) graphics_device_dispose: sys::cna_graphics_device_dispose_fn,
    pub(crate) graphics_device_executes_shader_effect_source_ext: sys::cna_graphics_device_executes_shader_effect_source_ext_fn,
    pub(crate) graphics_device_get_display_color_space_ext: sys::cna_graphics_device_get_display_color_space_ext_fn,
    pub(crate) graphics_device_get_is_disposed: sys::cna_graphics_device_get_is_disposed_fn,
    pub(crate) graphics_device_get_max_compute_work_group_count_ext: sys::cna_graphics_device_get_max_compute_work_group_count_ext_fn,
    pub(crate) graphics_device_get_max_compute_work_group_invocations_ext: sys::cna_graphics_device_get_max_compute_work_group_invocations_ext_fn,
    pub(crate) graphics_device_get_max_compute_work_group_size_ext: sys::cna_graphics_device_get_max_compute_work_group_size_ext_fn,
    pub(crate) graphics_device_get_tracked_resource_count: sys::cna_graphics_device_get_tracked_resource_count_fn,
    pub(crate) graphics_device_get_unsupported_3d_call_behavior: sys::cna_graphics_device_get_unsupported_3d_call_behavior_fn,
    pub(crate) graphics_device_notify_content_lost_resources_ext: sys::cna_graphics_device_notify_content_lost_resources_ext_fn,
    pub(crate) graphics_device_recreate_renderer_for_multi_sample_count_ext: sys::cna_graphics_device_recreate_renderer_for_multi_sample_count_ext_fn,
    pub(crate) graphics_device_set_blend_enabled: sys::cna_graphics_device_set_blend_enabled_fn,
    pub(crate) graphics_device_set_context_recovery_enabled: sys::cna_graphics_device_set_context_recovery_enabled_fn,
    pub(crate) graphics_device_set_current_effect: sys::cna_graphics_device_set_current_effect_fn,
    pub(crate) graphics_device_set_depth_test_enabled: sys::cna_graphics_device_set_depth_test_enabled_fn,
    pub(crate) graphics_device_set_depth_write_enabled: sys::cna_graphics_device_set_depth_write_enabled_fn,
    pub(crate) graphics_device_set_display_color_space_ext: sys::cna_graphics_device_set_display_color_space_ext_fn,
    pub(crate) graphics_device_set_graphics_profile_ext: sys::cna_graphics_device_set_graphics_profile_ext_fn,
    pub(crate) graphics_device_set_string_marker_ext: sys::cna_graphics_device_set_string_marker_ext_fn,
    pub(crate) graphics_device_set_unsupported_3d_call_behavior: sys::cna_graphics_device_set_unsupported_3d_call_behavior_fn,
    pub(crate) graphics_device_subscribe_event: sys::cna_graphics_device_subscribe_event_fn,
    pub(crate) graphics_device_subscribe_resource_created: sys::cna_graphics_device_subscribe_resource_created_fn,
    pub(crate) graphics_device_subscribe_resource_destroyed: sys::cna_graphics_device_subscribe_resource_destroyed_fn,
    pub(crate) graphics_device_supports_display_color_space_ext: sys::cna_graphics_device_supports_display_color_space_ext_fn,
    pub(crate) graphics_device_supports_image_based_lighting_ext: sys::cna_graphics_device_supports_image_based_lighting_ext_fn,
    pub(crate) graphics_device_supports_surface_format_as_render_target_ext: sys::cna_graphics_device_supports_surface_format_as_render_target_ext_fn,
    pub(crate) graphics_device_unbind_texture: sys::cna_graphics_device_unbind_texture_fn,
    pub(crate) graphics_device_unsubscribe: sys::cna_graphics_device_unsubscribe_fn,
    pub(crate) occlusion_query_get_is_pixel_count_precise_ext: sys::cna_occlusion_query_get_is_pixel_count_precise_ext_fn,
    pub(crate) occlusion_query_has_renderer: sys::cna_occlusion_query_has_renderer_fn,
    pub(crate) primitive_type_get_vertex_count: sys::cna_primitive_type_get_vertex_count_fn,
    pub(crate) alpha_test_effect_get_texture: sys::cna_alpha_test_effect_get_texture_fn,
    pub(crate) basic_effect_get_texture: sys::cna_basic_effect_get_texture_fn,
    pub(crate) color_matrix_effect_create: sys::cna_color_matrix_effect_create_fn,
    pub(crate) color_matrix_effect_get_matrix: sys::cna_color_matrix_effect_get_matrix_fn,
    pub(crate) color_matrix_effect_get_offset: sys::cna_color_matrix_effect_get_offset_fn,
    pub(crate) color_matrix_effect_reset: sys::cna_color_matrix_effect_reset_fn,
    pub(crate) color_matrix_effect_set_grayscale: sys::cna_color_matrix_effect_set_grayscale_fn,
    pub(crate) color_matrix_effect_set_matrix: sys::cna_color_matrix_effect_set_matrix_fn,
    pub(crate) color_matrix_effect_set_offset: sys::cna_color_matrix_effect_set_offset_fn,
    pub(crate) content_manager_load_effect: sys::cna_content_manager_load_effect_fn,
    pub(crate) dual_texture_effect_get_texture: sys::cna_dual_texture_effect_get_texture_fn,
    pub(crate) effect_copy_fragment_source: sys::cna_effect_copy_fragment_source_fn,
    pub(crate) effect_copy_vertex_source: sys::cna_effect_copy_vertex_source_fn,
    pub(crate) effect_get_fragment_source_byte_count: sys::cna_effect_get_fragment_source_byte_count_fn,
    pub(crate) effect_get_graphics_device: sys::cna_effect_get_graphics_device_fn,
    pub(crate) effect_get_is_compiled_ext: sys::cna_effect_get_is_compiled_ext_fn,
    pub(crate) effect_get_vertex_source_byte_count: sys::cna_effect_get_vertex_source_byte_count_fn,
    pub(crate) effect_has_renderer: sys::cna_effect_has_renderer_fn,
    pub(crate) effect_is_exact_stock_sprite_effect: sys::cna_effect_is_exact_stock_sprite_effect_fn,
    pub(crate) effect_material_get_retained_parameter_texture_count_ext: sys::cna_effect_material_get_retained_parameter_texture_count_ext_fn,
    pub(crate) effect_material_retain_parameter_texture_ext: sys::cna_effect_material_retain_parameter_texture_ext_fn,
    pub(crate) effect_pass_get_index_ext: sys::cna_effect_pass_get_index_ext_fn,
    pub(crate) effect_technique_get_identity: sys::cna_effect_technique_get_identity_fn,
    pub(crate) effect_technique_get_index_ext: sys::cna_effect_technique_get_index_ext_fn,
    pub(crate) environment_map_effect_get_environment_map: sys::cna_environment_map_effect_get_environment_map_fn,
    pub(crate) environment_map_effect_get_texture: sys::cna_environment_map_effect_get_texture_fn,
    pub(crate) pbr_effect_get_encode_output_to_srgb_ext: sys::cna_pbr_effect_get_encode_output_to_srgb_ext_fn,
    pub(crate) pbr_effect_get_specular_color_factor_ext: sys::cna_pbr_effect_get_specular_color_factor_ext_fn,
    pub(crate) pbr_effect_get_texture: sys::cna_pbr_effect_get_texture_fn,
    pub(crate) pbr_effect_get_texture_coordinate_set_ext: sys::cna_pbr_effect_get_texture_coordinate_set_ext_fn,
    pub(crate) pbr_effect_get_texture_is_srgb_ext: sys::cna_pbr_effect_get_texture_is_srgb_ext_fn,
    pub(crate) pbr_effect_get_texture_transform_ext: sys::cna_pbr_effect_get_texture_transform_ext_fn,
    pub(crate) pbr_effect_set_encode_output_to_srgb_ext: sys::cna_pbr_effect_set_encode_output_to_srgb_ext_fn,
    pub(crate) pbr_effect_set_specular_color_factor_ext: sys::cna_pbr_effect_set_specular_color_factor_ext_fn,
    pub(crate) pbr_effect_set_texture: sys::cna_pbr_effect_set_texture_fn,
    pub(crate) pbr_effect_set_texture_coordinate_set_ext: sys::cna_pbr_effect_set_texture_coordinate_set_ext_fn,
    pub(crate) pbr_effect_set_texture_is_srgb_ext: sys::cna_pbr_effect_set_texture_is_srgb_ext_fn,
    pub(crate) pbr_effect_set_texture_transform_ext: sys::cna_pbr_effect_set_texture_transform_ext_fn,
    pub(crate) shader_effect_copy_compile_error_ext: sys::cna_shader_effect_copy_compile_error_ext_fn,
    pub(crate) shader_effect_create: sys::cna_shader_effect_create_fn,
    pub(crate) shader_effect_declare_uniform_block_ext: sys::cna_shader_effect_declare_uniform_block_ext_fn,
    pub(crate) shader_effect_get_projection: sys::cna_shader_effect_get_projection_fn,
    pub(crate) shader_effect_get_view: sys::cna_shader_effect_get_view_fn,
    pub(crate) shader_effect_get_world: sys::cna_shader_effect_get_world_fn,
    pub(crate) shader_effect_has_renderer: sys::cna_shader_effect_has_renderer_fn,
    pub(crate) shader_effect_is_valid: sys::cna_shader_effect_is_valid_fn,
    pub(crate) shader_effect_set_projection: sys::cna_shader_effect_set_projection_fn,
    pub(crate) shader_effect_set_texture2d: sys::cna_shader_effect_set_texture2d_fn,
    pub(crate) shader_effect_set_texture3d: sys::cna_shader_effect_set_texture3d_fn,
    pub(crate) shader_effect_set_texture_cube: sys::cna_shader_effect_set_texture_cube_fn,
    pub(crate) shader_effect_set_uniform_float: sys::cna_shader_effect_set_uniform_float_fn,
    pub(crate) shader_effect_set_uniform_float_array: sys::cna_shader_effect_set_uniform_float_array_fn,
    pub(crate) shader_effect_set_uniform_int32: sys::cna_shader_effect_set_uniform_int32_fn,
    pub(crate) shader_effect_set_uniform_mat4_array: sys::cna_shader_effect_set_uniform_mat4_array_fn,
    pub(crate) shader_effect_set_uniform_matrix: sys::cna_shader_effect_set_uniform_matrix_fn,
    pub(crate) shader_effect_set_uniform_vec3_array: sys::cna_shader_effect_set_uniform_vec3_array_fn,
    pub(crate) shader_effect_set_uniform_vector2: sys::cna_shader_effect_set_uniform_vector2_fn,
    pub(crate) shader_effect_set_uniform_vector2_array: sys::cna_shader_effect_set_uniform_vector2_array_fn,
    pub(crate) shader_effect_set_uniform_vector3: sys::cna_shader_effect_set_uniform_vector3_fn,
    pub(crate) shader_effect_set_uniform_vector4: sys::cna_shader_effect_set_uniform_vector4_fn,
    pub(crate) shader_effect_set_view: sys::cna_shader_effect_set_view_fn,
    pub(crate) shader_effect_set_world: sys::cna_shader_effect_set_world_fn,
    pub(crate) skinned_effect_get_texture: sys::cna_skinned_effect_get_texture_fn,
    pub(crate) skinned_effect_get_vertex_color_enabled: sys::cna_skinned_effect_get_vertex_color_enabled_fn,
    pub(crate) skinned_effect_set_vertex_color_enabled: sys::cna_skinned_effect_set_vertex_color_enabled_fn,
    pub(crate) sprite_effect_create: sys::cna_sprite_effect_create_fn,
    pub(super) effect_matrices_get_world: sys::cna_effect_matrices_get_world_fn,
    pub(super) effect_matrices_set_world: sys::cna_effect_matrices_set_world_fn,
    pub(super) effect_matrices_get_view: sys::cna_effect_matrices_get_view_fn,
    pub(super) effect_matrices_set_view: sys::cna_effect_matrices_set_view_fn,
    pub(super) effect_matrices_get_projection: sys::cna_effect_matrices_get_projection_fn,
    pub(super) effect_matrices_set_projection: sys::cna_effect_matrices_set_projection_fn,
    pub(super) effect_fog_get_color: sys::cna_effect_fog_get_color_fn,
    pub(super) effect_fog_set_color: sys::cna_effect_fog_set_color_fn,
    pub(super) effect_fog_get_enabled: sys::cna_effect_fog_get_enabled_fn,
    pub(super) effect_fog_set_enabled: sys::cna_effect_fog_set_enabled_fn,
    pub(super) effect_fog_get_start: sys::cna_effect_fog_get_start_fn,
    pub(super) effect_fog_set_start: sys::cna_effect_fog_set_start_fn,
    pub(super) effect_fog_get_end: sys::cna_effect_fog_get_end_fn,
    pub(super) effect_fog_set_end: sys::cna_effect_fog_set_end_fn,
    pub(super) effect_lights_get_ambient_color: sys::cna_effect_lights_get_ambient_color_fn,
    pub(super) effect_lights_set_ambient_color: sys::cna_effect_lights_set_ambient_color_fn,
    pub(super) effect_lights_get_directional_light: sys::cna_effect_lights_get_directional_light_fn,
    pub(super) effect_lights_get_enabled: sys::cna_effect_lights_get_enabled_fn,
    pub(super) effect_lights_set_enabled: sys::cna_effect_lights_set_enabled_fn,
    pub(super) effect_lights_enable_default: sys::cna_effect_lights_enable_default_fn,
    pub(super) basic_effect_get_vertex_color_enabled:
        sys::cna_basic_effect_get_vertex_color_enabled_fn,
    pub(super) basic_effect_set_vertex_color_enabled:
        sys::cna_basic_effect_set_vertex_color_enabled_fn,
    pub(super) basic_effect_get_prefer_per_pixel_lighting:
        sys::cna_basic_effect_get_prefer_per_pixel_lighting_fn,
    pub(super) basic_effect_set_prefer_per_pixel_lighting:
        sys::cna_basic_effect_set_prefer_per_pixel_lighting_fn,
    pub(super) basic_effect_get_diffuse_color: sys::cna_basic_effect_get_diffuse_color_fn,
    pub(super) basic_effect_set_diffuse_color: sys::cna_basic_effect_set_diffuse_color_fn,
    pub(super) basic_effect_get_emissive_color: sys::cna_basic_effect_get_emissive_color_fn,
    pub(super) basic_effect_set_emissive_color: sys::cna_basic_effect_set_emissive_color_fn,
    pub(super) basic_effect_get_specular_color: sys::cna_basic_effect_get_specular_color_fn,
    pub(super) basic_effect_set_specular_color: sys::cna_basic_effect_set_specular_color_fn,
    pub(super) basic_effect_get_specular_power: sys::cna_basic_effect_get_specular_power_fn,
    pub(super) basic_effect_set_specular_power: sys::cna_basic_effect_set_specular_power_fn,
    pub(super) basic_effect_get_alpha: sys::cna_basic_effect_get_alpha_fn,
    pub(super) basic_effect_set_alpha: sys::cna_basic_effect_set_alpha_fn,
    pub(super) basic_effect_get_texture_enabled: sys::cna_basic_effect_get_texture_enabled_fn,
    pub(super) basic_effect_set_texture_enabled: sys::cna_basic_effect_set_texture_enabled_fn,
    pub(super) basic_effect_set_texture: sys::cna_basic_effect_set_texture_fn,
    pub(super) alpha_test_effect_create: sys::cna_alpha_test_effect_create_fn,
    pub(super) alpha_test_effect_get_diffuse_color: sys::cna_alpha_test_effect_get_diffuse_color_fn,
    pub(super) alpha_test_effect_set_diffuse_color: sys::cna_alpha_test_effect_set_diffuse_color_fn,
    pub(super) alpha_test_effect_get_alpha: sys::cna_alpha_test_effect_get_alpha_fn,
    pub(super) alpha_test_effect_set_alpha: sys::cna_alpha_test_effect_set_alpha_fn,
    pub(super) alpha_test_effect_set_texture: sys::cna_alpha_test_effect_set_texture_fn,
    pub(super) alpha_test_effect_get_vertex_color_enabled:
        sys::cna_alpha_test_effect_get_vertex_color_enabled_fn,
    pub(super) alpha_test_effect_set_vertex_color_enabled:
        sys::cna_alpha_test_effect_set_vertex_color_enabled_fn,
    pub(super) alpha_test_effect_get_alpha_function:
        sys::cna_alpha_test_effect_get_alpha_function_fn,
    pub(super) alpha_test_effect_set_alpha_function:
        sys::cna_alpha_test_effect_set_alpha_function_fn,
    pub(super) alpha_test_effect_get_reference_alpha:
        sys::cna_alpha_test_effect_get_reference_alpha_fn,
    pub(super) alpha_test_effect_set_reference_alpha:
        sys::cna_alpha_test_effect_set_reference_alpha_fn,
    pub(super) dual_texture_effect_create: sys::cna_dual_texture_effect_create_fn,
    pub(super) dual_texture_effect_get_diffuse_color:
        sys::cna_dual_texture_effect_get_diffuse_color_fn,
    pub(super) dual_texture_effect_set_diffuse_color:
        sys::cna_dual_texture_effect_set_diffuse_color_fn,
    pub(super) dual_texture_effect_get_alpha: sys::cna_dual_texture_effect_get_alpha_fn,
    pub(super) dual_texture_effect_set_alpha: sys::cna_dual_texture_effect_set_alpha_fn,
    pub(super) dual_texture_effect_set_texture: sys::cna_dual_texture_effect_set_texture_fn,
    pub(super) dual_texture_effect_get_vertex_color_enabled:
        sys::cna_dual_texture_effect_get_vertex_color_enabled_fn,
    pub(super) dual_texture_effect_set_vertex_color_enabled:
        sys::cna_dual_texture_effect_set_vertex_color_enabled_fn,
    pub(super) environment_map_effect_create: sys::cna_environment_map_effect_create_fn,
    pub(super) environment_map_effect_get_diffuse_color:
        sys::cna_environment_map_effect_get_diffuse_color_fn,
    pub(super) environment_map_effect_set_diffuse_color:
        sys::cna_environment_map_effect_set_diffuse_color_fn,
    pub(super) environment_map_effect_get_emissive_color:
        sys::cna_environment_map_effect_get_emissive_color_fn,
    pub(super) environment_map_effect_set_emissive_color:
        sys::cna_environment_map_effect_set_emissive_color_fn,
    pub(super) environment_map_effect_get_alpha: sys::cna_environment_map_effect_get_alpha_fn,
    pub(super) environment_map_effect_set_alpha: sys::cna_environment_map_effect_set_alpha_fn,
    pub(super) environment_map_effect_set_texture: sys::cna_environment_map_effect_set_texture_fn,
    pub(super) environment_map_effect_set_environment_map:
        sys::cna_environment_map_effect_set_environment_map_fn,
    pub(super) environment_map_effect_get_amount: sys::cna_environment_map_effect_get_amount_fn,
    pub(super) environment_map_effect_set_amount: sys::cna_environment_map_effect_set_amount_fn,
    pub(super) environment_map_effect_get_specular: sys::cna_environment_map_effect_get_specular_fn,
    pub(super) environment_map_effect_set_specular: sys::cna_environment_map_effect_set_specular_fn,
    pub(super) environment_map_effect_get_fresnel_factor:
        sys::cna_environment_map_effect_get_fresnel_factor_fn,
    pub(super) environment_map_effect_set_fresnel_factor:
        sys::cna_environment_map_effect_set_fresnel_factor_fn,
    pub(super) skinned_effect_create: sys::cna_skinned_effect_create_fn,
    pub(super) skinned_effect_get_diffuse_color: sys::cna_skinned_effect_get_diffuse_color_fn,
    pub(super) skinned_effect_set_diffuse_color: sys::cna_skinned_effect_set_diffuse_color_fn,
    pub(super) skinned_effect_get_emissive_color: sys::cna_skinned_effect_get_emissive_color_fn,
    pub(super) skinned_effect_set_emissive_color: sys::cna_skinned_effect_set_emissive_color_fn,
    pub(super) skinned_effect_get_specular_color: sys::cna_skinned_effect_get_specular_color_fn,
    pub(super) skinned_effect_set_specular_color: sys::cna_skinned_effect_set_specular_color_fn,
    pub(super) skinned_effect_get_specular_power: sys::cna_skinned_effect_get_specular_power_fn,
    pub(super) skinned_effect_set_specular_power: sys::cna_skinned_effect_set_specular_power_fn,
    pub(super) skinned_effect_get_alpha: sys::cna_skinned_effect_get_alpha_fn,
    pub(super) skinned_effect_set_alpha: sys::cna_skinned_effect_set_alpha_fn,
    pub(super) skinned_effect_get_prefer_per_pixel_lighting:
        sys::cna_skinned_effect_get_prefer_per_pixel_lighting_fn,
    pub(super) skinned_effect_set_prefer_per_pixel_lighting:
        sys::cna_skinned_effect_set_prefer_per_pixel_lighting_fn,
    pub(super) skinned_effect_set_texture: sys::cna_skinned_effect_set_texture_fn,
    pub(super) skinned_effect_get_weights_per_vertex:
        sys::cna_skinned_effect_get_weights_per_vertex_fn,
    pub(super) skinned_effect_set_weights_per_vertex:
        sys::cna_skinned_effect_set_weights_per_vertex_fn,
    pub(super) skinned_effect_set_bone_transforms: sys::cna_skinned_effect_set_bone_transforms_fn,
    pub(super) skinned_effect_copy_bone_transforms: sys::cna_skinned_effect_copy_bone_transforms_fn,
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
    pub(super) keyboard_get_state_for_player: sys::cna_keyboard_get_state_for_player_fn,
    pub(super) mouse_get_state: sys::cna_mouse_get_state_fn,
    pub(super) mouse_get_window_handle: sys::cna_mouse_get_window_handle_fn,
    pub(super) mouse_set_window_handle: sys::cna_mouse_set_window_handle_fn,
    pub(super) mouse_set_position: sys::cna_mouse_set_position_fn,
    pub(super) gamepad_get_state: sys::cna_gamepad_get_state_fn,
    pub(super) gamepad_get_state_with_dead_zone: sys::cna_gamepad_get_state_with_dead_zone_fn,
    pub(super) gamepad_get_capabilities: sys::cna_gamepad_get_capabilities_fn,
    pub(super) gamepad_set_vibration: sys::cna_gamepad_set_vibration_fn,
    pub(super) touch_get_capabilities: sys::cna_touch_get_capabilities_fn,
    pub(super) touch_get_state: sys::cna_touch_get_state_fn,
    pub(super) touch_panel_get_display_width: sys::cna_touch_panel_get_display_width_fn,
    pub(super) touch_panel_set_display_width: sys::cna_touch_panel_set_display_width_fn,
    pub(super) touch_panel_get_display_height: sys::cna_touch_panel_get_display_height_fn,
    pub(super) touch_panel_set_display_height: sys::cna_touch_panel_set_display_height_fn,
    pub(super) touch_panel_get_display_orientation: sys::cna_touch_panel_get_display_orientation_fn,
    pub(super) touch_panel_set_display_orientation: sys::cna_touch_panel_set_display_orientation_fn,
    pub(super) touch_panel_get_enabled_gestures: sys::cna_touch_panel_get_enabled_gestures_fn,
    pub(super) touch_panel_set_enabled_gestures: sys::cna_touch_panel_set_enabled_gestures_fn,
    pub(super) touch_panel_get_is_gesture_available:
        sys::cna_touch_panel_get_is_gesture_available_fn,
    pub(super) touch_panel_get_window_handle: sys::cna_touch_panel_get_window_handle_fn,
    pub(super) touch_panel_set_window_handle: sys::cna_touch_panel_set_window_handle_fn,
    pub(super) touch_panel_read_gesture: sys::cna_touch_panel_read_gesture_fn,
    pub(super) graphics_device_manager_create: sys::cna_graphics_device_manager_create_fn,
    pub(super) graphics_device_manager_get_graphics_profile:
        sys::cna_graphics_device_manager_get_graphics_profile_fn,
    pub(super) graphics_device_manager_set_graphics_profile:
        sys::cna_graphics_device_manager_set_graphics_profile_fn,
    pub(super) graphics_device_manager_get_is_full_screen:
        sys::cna_graphics_device_manager_get_is_full_screen_fn,
    pub(super) graphics_device_manager_set_is_full_screen:
        sys::cna_graphics_device_manager_set_is_full_screen_fn,
    pub(super) graphics_device_manager_get_prefer_multi_sampling:
        sys::cna_graphics_device_manager_get_prefer_multi_sampling_fn,
    pub(super) graphics_device_manager_set_prefer_multi_sampling:
        sys::cna_graphics_device_manager_set_prefer_multi_sampling_fn,
    pub(super) graphics_device_manager_get_preferred_back_buffer_format:
        sys::cna_graphics_device_manager_get_preferred_back_buffer_format_fn,
    pub(super) graphics_device_manager_set_preferred_back_buffer_format:
        sys::cna_graphics_device_manager_set_preferred_back_buffer_format_fn,
    pub(super) graphics_device_manager_get_preferred_back_buffer_width:
        sys::cna_graphics_device_manager_get_preferred_back_buffer_width_fn,
    pub(super) graphics_device_manager_set_preferred_back_buffer_width:
        sys::cna_graphics_device_manager_set_preferred_back_buffer_width_fn,
    pub(super) graphics_device_manager_get_preferred_back_buffer_height:
        sys::cna_graphics_device_manager_get_preferred_back_buffer_height_fn,
    pub(super) graphics_device_manager_set_preferred_back_buffer_height:
        sys::cna_graphics_device_manager_set_preferred_back_buffer_height_fn,
    pub(super) graphics_device_manager_get_preferred_depth_stencil_format:
        sys::cna_graphics_device_manager_get_preferred_depth_stencil_format_fn,
    pub(super) graphics_device_manager_set_preferred_depth_stencil_format:
        sys::cna_graphics_device_manager_set_preferred_depth_stencil_format_fn,
    pub(super) graphics_device_manager_get_synchronize_with_vertical_retrace:
        sys::cna_graphics_device_manager_get_synchronize_with_vertical_retrace_fn,
    pub(super) graphics_device_manager_set_synchronize_with_vertical_retrace:
        sys::cna_graphics_device_manager_set_synchronize_with_vertical_retrace_fn,
    pub(super) graphics_device_manager_get_supported_orientations:
        sys::cna_graphics_device_manager_get_supported_orientations_fn,
    pub(super) graphics_device_manager_set_supported_orientations:
        sys::cna_graphics_device_manager_set_supported_orientations_fn,
    pub(super) graphics_device_manager_apply_changes:
        sys::cna_graphics_device_manager_apply_changes_fn,
    pub(super) graphics_device_manager_toggle_full_screen:
        sys::cna_graphics_device_manager_toggle_full_screen_fn,
    pub(super) graphics_device_manager_create_device:
        sys::cna_graphics_device_manager_create_device_fn,
    pub(super) graphics_device_manager_begin_draw: sys::cna_graphics_device_manager_begin_draw_fn,
    pub(super) graphics_device_manager_end_draw: sys::cna_graphics_device_manager_end_draw_fn,
    pub(super) graphics_device_manager_dispose: sys::cna_graphics_device_manager_dispose_fn,
    pub(super) graphics_device_manager_subscribe: sys::cna_graphics_device_manager_subscribe_fn,
    pub(super) graphics_device_manager_subscribe_preparing_device_settings_ext:
        sys::cna_graphics_device_manager_subscribe_preparing_device_settings_ext_fn,
    pub(super) graphics_device_manager_destroy: sys::cna_graphics_device_manager_destroy_fn,
    pub(super) storage_device_show_selector: sys::cna_storage_device_show_selector_fn,
    pub(super) storage_device_show_selector_for_player:
        sys::cna_storage_device_show_selector_for_player_fn,
    pub(super) storage_device_show_selector_with_space:
        sys::cna_storage_device_show_selector_with_space_fn,
    pub(super) storage_device_show_selector_for_player_with_space:
        sys::cna_storage_device_show_selector_for_player_with_space_fn,
    pub(super) storage_device_get_free_space: sys::cna_storage_device_get_free_space_fn,
    pub(super) storage_device_get_is_connected: sys::cna_storage_device_get_is_connected_fn,
    pub(super) storage_device_get_total_space: sys::cna_storage_device_get_total_space_fn,
    pub(super) storage_device_delete_container: sys::cna_storage_device_delete_container_fn,
    pub(super) storage_device_subscribe_device_changed:
        sys::cna_storage_device_subscribe_device_changed_fn,
    pub(super) storage_device_unsubscribe_device_changed:
        sys::cna_storage_device_unsubscribe_device_changed_fn,
    pub(super) storage_device_destroy: sys::cna_storage_device_destroy_fn,
    pub(super) storage_container_open: sys::cna_storage_container_open_fn,
    pub(super) storage_container_get_display_name_size:
        sys::cna_storage_container_get_display_name_size_fn,
    pub(super) storage_container_copy_display_name: sys::cna_storage_container_copy_display_name_fn,
    pub(super) storage_container_dispose: sys::cna_storage_container_dispose_fn,
    pub(super) storage_container_subscribe_disposing:
        sys::cna_storage_container_subscribe_disposing_fn,
    pub(super) storage_container_unsubscribe_disposing:
        sys::cna_storage_container_unsubscribe_disposing_fn,
    pub(super) storage_container_create_directory: sys::cna_storage_container_create_directory_fn,
    pub(super) storage_container_directory_exists: sys::cna_storage_container_directory_exists_fn,
    pub(super) storage_container_delete_directory: sys::cna_storage_container_delete_directory_fn,
    pub(super) storage_container_file_exists: sys::cna_storage_container_file_exists_fn,
    pub(super) storage_container_delete_file: sys::cna_storage_container_delete_file_fn,
    pub(super) storage_container_get_directory_name_count:
        sys::cna_storage_container_get_directory_name_count_fn,
    pub(super) storage_container_copy_directory_name:
        sys::cna_storage_container_copy_directory_name_fn,
    pub(super) storage_container_get_file_name_count:
        sys::cna_storage_container_get_file_name_count_fn,
    pub(super) storage_container_copy_file_name: sys::cna_storage_container_copy_file_name_fn,
    pub(super) storage_container_create_file: sys::cna_storage_container_create_file_fn,
    pub(super) storage_container_open_file: sys::cna_storage_container_open_file_fn,
    pub(super) storage_container_open_file_access: sys::cna_storage_container_open_file_access_fn,
    pub(super) storage_container_open_file_share: sys::cna_storage_container_open_file_share_fn,
    pub(super) storage_container_destroy: sys::cna_storage_container_destroy_fn,
    pub(super) storage_stream_read: sys::cna_storage_stream_read_fn,
    pub(super) storage_stream_write: sys::cna_storage_stream_write_fn,
    pub(super) storage_stream_seek: sys::cna_storage_stream_seek_fn,
    pub(super) storage_stream_get_position: sys::cna_storage_stream_get_position_fn,
    pub(super) storage_stream_get_length: sys::cna_storage_stream_get_length_fn,
    pub(super) storage_stream_set_length: sys::cna_storage_stream_set_length_fn,
    pub(super) storage_stream_get_can_read: sys::cna_storage_stream_get_can_read_fn,
    pub(super) storage_stream_get_can_write: sys::cna_storage_stream_get_can_write_fn,
    pub(super) storage_stream_get_can_seek: sys::cna_storage_stream_get_can_seek_fn,
    pub(super) storage_stream_flush: sys::cna_storage_stream_flush_fn,
    pub(super) storage_stream_close: sys::cna_storage_stream_close_fn,
}

impl Native {
    /// Returns the process-wide table, loading the library on first use.
    ///
    /// CNA's runtime-identity and renderer-selection routes are process-global
    /// and must be reachable before a `Game` exists, so they cannot go through
    /// a game-owned table. Only a successful load is cached: a failure stays
    /// retryable, because the caller may not have pointed at a library yet.
    pub(crate) fn process() -> Result<Arc<Self>> {
        static PROCESS: Mutex<Option<Arc<Native>>> = Mutex::new(None);
        let mut cached = PROCESS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(native) = cached.as_ref() {
            return Ok(Arc::clone(native));
        }
        let native = Self::load()?;
        *cached = Some(Arc::clone(&native));
        Ok(native)
    }

    /// Fills the tables from whichever source this build was configured with.
    ///
    /// The two modes differ only here. Everything above sees the same tables,
    /// the same ABI check and the same errors.
    pub(crate) fn load() -> Result<Arc<Self>> {
        #[cfg(feature = "direct-link")]
        {
            // CNA's symbols are part of this executable. There is nothing to
            // search for and nothing that could fail to open, so the only way
            // to fail is the ABI check `from_source` performs.
            Self::from_source(NativeSource::Linked).map(Arc::new)
        }
        #[cfg(all(not(feature = "direct-link"), any(unix, windows)))]
        {
            Self::load_platform().map(Arc::new)
        }
        #[cfg(all(not(feature = "direct-link"), not(any(unix, windows))))]
        {
            Err(CnaError::UnsupportedPlatform)
        }
    }

    /// Opens the first candidate library the platform loader accepts.
    #[cfg(all(not(feature = "direct-link"), any(unix, windows)))]
    fn load_platform() -> Result<Self> {
        let candidates = library_candidates();
        let mut diagnostics = Vec::new();
        for candidate in &candidates {
            match Library::open(candidate) {
                Ok(library) => return Self::from_source(NativeSource::Dynamic(library)),
                Err(error) => diagnostics.push(format!("{}: {error}", candidate.display())),
            }
        }
        Err(CnaError::NativeUnavailable {
            searched: candidates,
            details: diagnostics.join("; "),
        })
    }

    fn from_source(source: NativeSource) -> Result<Self> {
        macro_rules! symbol {
            ($name:ident, $ty:ty) => {
                super::loader::acquire!(source, $name, $ty)
            };
        }

        let get_abi_version = symbol!(cna_get_abi_version, sys::cna_get_abi_version_fn);
        // SAFETY: the symbol has the audited zero-argument ABI declaration.
        let actual = unsafe { get_abi_version() };
        if let Err(rejection) = super::abi::admit(actual) {
            return Err(CnaError::AbiVersionMismatch {
                expected: sys::CNA_ABI_VERSION,
                actual,
                reason: rejection.reason(),
            });
        }

        Ok(Self {
            runtime: RuntimeApi::load(&source)?,
            engine: EngineApi::load(&source)?,
            models: ModelsApi::load(&source)?,
            gamer_services: GamerServicesApi::load(&source)?,
            net: NetApi::load(&source)?,
            error_get_last_info: symbol!(cna_error_get_last_info,
                sys::cna_error_get_last_info_fn
            ),
            error_get_last_message_size: symbol!(cna_error_get_last_message_size,
                sys::cna_error_get_last_message_size_fn
            ),
            error_copy_last_message: symbol!(cna_error_copy_last_message,
                sys::cna_error_copy_last_message_fn
            ),
            game_create: symbol!(cna_game_create, sys::cna_game_create_fn),
            game_set_frame_hooks: symbol!(cna_game_set_frame_hooks_ext,
                sys::cna_game_set_frame_hooks_ext_fn
            ),
            game_run_one_frame: symbol!(cna_game_run_one_frame, sys::cna_game_run_one_frame_fn),
            game_run: symbol!(cna_game_run, sys::cna_game_run_fn),
            game_request_exit: symbol!(cna_game_request_exit, sys::cna_game_request_exit_fn),
            game_get_is_active: symbol!(cna_game_get_is_active, sys::cna_game_get_is_active_fn),
            game_get_is_mouse_visible: symbol!(cna_game_get_is_mouse_visible,
                sys::cna_game_get_is_mouse_visible_fn
            ),
            game_set_is_mouse_visible: symbol!(cna_game_set_is_mouse_visible,
                sys::cna_game_set_is_mouse_visible_fn
            ),
            game_get_is_fixed_time_step: symbol!(cna_game_get_is_fixed_time_step,
                sys::cna_game_get_is_fixed_time_step_fn
            ),
            game_set_is_fixed_time_step: symbol!(cna_game_set_is_fixed_time_step,
                sys::cna_game_set_is_fixed_time_step_fn
            ),
            game_get_target_elapsed_time_ticks: symbol!(cna_game_get_target_elapsed_time_ticks,
                sys::cna_game_get_target_elapsed_time_ticks_fn
            ),
            game_set_target_elapsed_time_ticks: symbol!(cna_game_set_target_elapsed_time_ticks,
                sys::cna_game_set_target_elapsed_time_ticks_fn
            ),
            game_get_inactive_sleep_time_ticks: symbol!(cna_game_get_inactive_sleep_time_ticks,
                sys::cna_game_get_inactive_sleep_time_ticks_fn
            ),
            game_set_inactive_sleep_time_ticks: symbol!(cna_game_set_inactive_sleep_time_ticks,
                sys::cna_game_set_inactive_sleep_time_ticks_fn
            ),
            game_reset_elapsed_time: symbol!(cna_game_reset_elapsed_time,
                sys::cna_game_reset_elapsed_time_fn
            ),
            game_suppress_draw: symbol!(cna_game_suppress_draw, sys::cna_game_suppress_draw_fn),
            game_tick: symbol!(cna_game_tick, sys::cna_game_tick_fn),
            game_set_window_title: symbol!(cna_game_set_window_title,
                sys::cna_game_set_window_title_fn
            ),
            game_subscribe: symbol!(cna_game_subscribe, sys::cna_game_subscribe_fn),
            game_window_subscribe: symbol!(cna_game_window_subscribe,
                sys::cna_game_window_subscribe_fn
            ),
            game_unsubscribe: symbol!(cna_game_unsubscribe, sys::cna_game_unsubscribe_fn),
            game_destroy: symbol!(cna_game_destroy, sys::cna_game_destroy_fn),
            game_window_get_allow_user_resizing: symbol!(cna_game_window_get_allow_user_resizing,
                sys::cna_game_window_get_allow_user_resizing_fn
            ),
            game_window_set_allow_user_resizing: symbol!(cna_game_window_set_allow_user_resizing,
                sys::cna_game_window_set_allow_user_resizing_fn
            ),
            game_window_get_client_bounds: symbol!(cna_game_window_get_client_bounds,
                sys::cna_game_window_get_client_bounds_fn
            ),
            game_window_get_current_orientation: symbol!(cna_game_window_get_current_orientation,
                sys::cna_game_window_get_current_orientation_fn
            ),
            game_window_get_native_handle: symbol!(cna_game_window_get_native_handle_ext,
                sys::cna_game_window_get_native_handle_ext_fn
            ),
            game_window_get_screen_device_name_size: symbol!(cna_game_window_get_screen_device_name_size,
                sys::cna_game_window_get_screen_device_name_size_fn
            ),
            game_window_copy_screen_device_name: symbol!(cna_game_window_copy_screen_device_name,
                sys::cna_game_window_copy_screen_device_name_fn
            ),
            game_window_get_title_size: symbol!(cna_game_window_get_title_size,
                sys::cna_game_window_get_title_size_fn
            ),
            game_window_copy_title: symbol!(cna_game_window_copy_title,
                sys::cna_game_window_copy_title_fn
            ),
            game_window_begin_screen_device_change: symbol!(cna_game_window_begin_screen_device_change,
                sys::cna_game_window_begin_screen_device_change_fn
            ),
            game_window_end_screen_device_change: symbol!(cna_game_window_end_screen_device_change,
                sys::cna_game_window_end_screen_device_change_fn
            ),
            game_get_graphics_device: symbol!(cna_game_get_graphics_device,
                sys::cna_game_get_graphics_device_fn
            ),
            graphics_device_create: symbol!(cna_graphics_device_create,
                sys::cna_graphics_device_create_fn
            ),
            graphics_device_destroy: symbol!(cna_graphics_device_destroy,
                sys::cna_graphics_device_destroy_fn
            ),
            graphics_device_get_status: symbol!(cna_graphics_device_get_status,
                sys::cna_graphics_device_get_status_fn
            ),
            graphics_device_get_graphics_profile: symbol!(cna_graphics_device_get_graphics_profile,
                sys::cna_graphics_device_get_graphics_profile_fn
            ),
            graphics_device_get_presentation_parameters: symbol!(cna_graphics_device_get_presentation_parameters,
                sys::cna_graphics_device_get_presentation_parameters_fn
            ),
            graphics_device_get_display_mode: symbol!(cna_graphics_device_get_display_mode,
                sys::cna_graphics_device_get_display_mode_fn
            ),
            graphics_device_get_blend_state: symbol!(cna_graphics_device_get_blend_state,
                sys::cna_graphics_device_get_blend_state_fn
            ),
            graphics_device_get_depth_stencil_state: symbol!(cna_graphics_device_get_depth_stencil_state,
                sys::cna_graphics_device_get_depth_stencil_state_fn
            ),
            graphics_device_get_rasterizer_state: symbol!(cna_graphics_device_get_rasterizer_state,
                sys::cna_graphics_device_get_rasterizer_state_fn
            ),
            graphics_device_get_sampler_state: symbol!(cna_graphics_device_get_sampler_state,
                sys::cna_graphics_device_get_sampler_state_fn
            ),
            graphics_device_set_sampler_state: symbol!(cna_graphics_device_set_sampler_state,
                sys::cna_graphics_device_set_sampler_state_fn
            ),
            graphics_device_get_texture: symbol!(cna_graphics_device_get_texture,
                sys::cna_graphics_device_get_texture_fn
            ),
            graphics_device_set_texture: symbol!(cna_graphics_device_set_texture,
                sys::cna_graphics_device_set_texture_fn
            ),
            graphics_device_get_adapter_index: symbol!(cna_graphics_device_get_adapter_index,
                sys::cna_graphics_device_get_adapter_index_fn
            ),
            graphics_adapter_get_count: symbol!(cna_graphics_adapter_get_count,
                sys::cna_graphics_adapter_get_count_fn
            ),
            graphics_adapter_get_info: symbol!(cna_graphics_adapter_get_info,
                sys::cna_graphics_adapter_get_info_fn
            ),
            graphics_adapter_copy_description: symbol!(cna_graphics_adapter_copy_description,
                sys::cna_graphics_adapter_copy_description_fn
            ),
            graphics_adapter_copy_device_name: symbol!(cna_graphics_adapter_copy_device_name,
                sys::cna_graphics_adapter_copy_device_name_fn
            ),
            graphics_adapter_get_current_display_mode: symbol!(cna_graphics_adapter_get_current_display_mode,
                sys::cna_graphics_adapter_get_current_display_mode_fn
            ),
            graphics_adapter_get_display_mode_count: symbol!(cna_graphics_adapter_get_display_mode_count,
                sys::cna_graphics_adapter_get_display_mode_count_fn
            ),
            graphics_adapter_copy_display_modes: symbol!(cna_graphics_adapter_copy_display_modes,
                sys::cna_graphics_adapter_copy_display_modes_fn
            ),
            graphics_adapter_set_device_preferences: symbol!(cna_graphics_adapter_set_device_preferences,
                sys::cna_graphics_adapter_set_device_preferences_fn
            ),
            graphics_adapter_is_profile_supported: symbol!(cna_graphics_adapter_is_profile_supported,
                sys::cna_graphics_adapter_is_profile_supported_fn
            ),
            graphics_adapter_query_render_target_format: symbol!(cna_graphics_adapter_query_render_target_format,
                sys::cna_graphics_adapter_query_render_target_format_fn
            ),
            graphics_adapter_query_backbuffer_format: symbol!(cna_graphics_adapter_query_backbuffer_format,
                sys::cna_graphics_adapter_query_backbuffer_format_fn
            ),
            graphics_adapter_get_native_monitor_handle: symbol!(cna_graphics_adapter_get_native_monitor_handle,
                sys::cna_graphics_adapter_get_native_monitor_handle_fn
            ),
            graphics_device_get_viewport: symbol!(cna_graphics_device_get_viewport,
                sys::cna_graphics_device_get_viewport_fn
            ),
            graphics_device_set_viewport: symbol!(cna_graphics_device_set_viewport,
                sys::cna_graphics_device_set_viewport_fn
            ),
            graphics_device_get_scissor_rectangle: symbol!(cna_graphics_device_get_scissor_rectangle,
                sys::cna_graphics_device_get_scissor_rectangle_fn
            ),
            graphics_device_set_scissor_rectangle: symbol!(cna_graphics_device_set_scissor_rectangle,
                sys::cna_graphics_device_set_scissor_rectangle_fn
            ),
            graphics_device_get_blend_factor: symbol!(cna_graphics_device_get_blend_factor,
                sys::cna_graphics_device_get_blend_factor_fn
            ),
            graphics_device_set_blend_factor: symbol!(cna_graphics_device_set_blend_factor,
                sys::cna_graphics_device_set_blend_factor_fn
            ),
            graphics_device_get_multi_sample_mask: symbol!(cna_graphics_device_get_multi_sample_mask,
                sys::cna_graphics_device_get_multi_sample_mask_fn
            ),
            graphics_device_set_multi_sample_mask: symbol!(cna_graphics_device_set_multi_sample_mask,
                sys::cna_graphics_device_set_multi_sample_mask_fn
            ),
            graphics_device_get_reference_stencil: symbol!(cna_graphics_device_get_reference_stencil,
                sys::cna_graphics_device_get_reference_stencil_fn
            ),
            graphics_device_set_reference_stencil: symbol!(cna_graphics_device_set_reference_stencil,
                sys::cna_graphics_device_set_reference_stencil_fn
            ),
            graphics_device_set_blend_state: symbol!(cna_graphics_device_set_blend_state,
                sys::cna_graphics_device_set_blend_state_fn
            ),
            graphics_device_set_depth_stencil_state: symbol!(cna_graphics_device_set_depth_stencil_state,
                sys::cna_graphics_device_set_depth_stencil_state_fn
            ),
            graphics_device_set_rasterizer_state: symbol!(cna_graphics_device_set_rasterizer_state,
                sys::cna_graphics_device_set_rasterizer_state_fn
            ),
            graphics_device_present: symbol!(cna_graphics_device_present,
                sys::cna_graphics_device_present_fn
            ),
            graphics_device_reset: symbol!(cna_graphics_device_reset,
                sys::cna_graphics_device_reset_fn
            ),
            graphics_device_reset_with_parameters: symbol!(cna_graphics_device_reset_with_parameters,
                sys::cna_graphics_device_reset_with_parameters_fn
            ),
            graphics_device_get_backbuffer_data_window: symbol!(cna_graphics_device_get_backbuffer_data_window,
                sys::cna_graphics_device_get_backbuffer_data_window_fn
            ),
            graphics_device_clear_rgba: symbol!(cna_graphics_device_clear_rgba,
                sys::cna_graphics_device_clear_rgba_fn
            ),
            graphics_device_clear_options: symbol!(cna_graphics_device_clear_options,
                sys::cna_graphics_device_clear_options_fn
            ),
            graphics_device_set_vertex_buffer: symbol!(cna_graphics_device_set_vertex_buffer,
                sys::cna_graphics_device_set_vertex_buffer_fn
            ),
            graphics_device_set_vertex_buffer_offset: symbol!(cna_graphics_device_set_vertex_buffer_offset,
                sys::cna_graphics_device_set_vertex_buffer_offset_fn
            ),
            graphics_device_set_vertex_buffers: symbol!(cna_graphics_device_set_vertex_buffers,
                sys::cna_graphics_device_set_vertex_buffers_fn
            ),
            graphics_device_get_vertex_buffer_count: symbol!(cna_graphics_device_get_vertex_buffer_count,
                sys::cna_graphics_device_get_vertex_buffer_count_fn
            ),
            graphics_device_copy_vertex_buffers: symbol!(cna_graphics_device_copy_vertex_buffers,
                sys::cna_graphics_device_copy_vertex_buffers_fn
            ),
            graphics_device_get_vertex_buffer: symbol!(cna_graphics_device_get_vertex_buffer,
                sys::cna_graphics_device_get_vertex_buffer_fn
            ),
            graphics_device_set_index_buffer: symbol!(cna_graphics_device_set_index_buffer,
                sys::cna_graphics_device_set_index_buffer_fn
            ),
            graphics_device_get_index_buffer: symbol!(cna_graphics_device_get_index_buffer,
                sys::cna_graphics_device_get_index_buffer_fn
            ),
            graphics_device_draw_primitives: symbol!(cna_graphics_device_draw_primitives,
                sys::cna_graphics_device_draw_primitives_fn
            ),
            graphics_device_draw_indexed_primitives: symbol!(cna_graphics_device_draw_indexed_primitives,
                sys::cna_graphics_device_draw_indexed_primitives_fn
            ),
            graphics_device_draw_instanced_primitives: symbol!(cna_graphics_device_draw_instanced_primitives,
                sys::cna_graphics_device_draw_instanced_primitives_fn
            ),
            graphics_device_draw_user_primitives: symbol!(cna_graphics_device_draw_user_primitives,
                sys::cna_graphics_device_draw_user_primitives_fn
            ),
            graphics_device_draw_user_indexed_primitives: symbol!(cna_graphics_device_draw_user_indexed_primitives,
                sys::cna_graphics_device_draw_user_indexed_primitives_fn
            ),
            occlusion_query_create: symbol!(cna_occlusion_query_create,
                sys::cna_occlusion_query_create_fn
            ),
            occlusion_query_begin: symbol!(cna_occlusion_query_begin,
                sys::cna_occlusion_query_begin_fn
            ),
            occlusion_query_end: symbol!(cna_occlusion_query_end,
                sys::cna_occlusion_query_end_fn
            ),
            occlusion_query_get_is_complete: symbol!(cna_occlusion_query_get_is_complete,
                sys::cna_occlusion_query_get_is_complete_fn
            ),
            occlusion_query_get_pixel_count: symbol!(cna_occlusion_query_get_pixel_count,
                sys::cna_occlusion_query_get_pixel_count_fn
            ),
            occlusion_query_destroy: symbol!(cna_occlusion_query_destroy,
                sys::cna_occlusion_query_destroy_fn
            ),
            graphics_device_set_render_targets: symbol!(cna_graphics_device_set_render_targets,
                sys::cna_graphics_device_set_render_targets_fn
            ),
            graphics_device_get_render_target_count: symbol!(cna_graphics_device_get_render_target_count,
                sys::cna_graphics_device_get_render_target_count_fn
            ),
            graphics_device_copy_render_targets: symbol!(cna_graphics_device_copy_render_targets,
                sys::cna_graphics_device_copy_render_targets_fn
            ),
            graphics_device_get_renderer_info: symbol!(cna_graphics_device_get_renderer_info,
                sys::cna_graphics_device_get_renderer_info_fn
            ),
            graphics_device_feature_support: symbol!(cna_graphics_device_get_renderer_feature_support_ext,
                sys::cna_graphics_device_get_renderer_feature_support_ext_fn
            ),
            graphics_device_limit: symbol!(cna_graphics_device_get_renderer_limit_ext,
                sys::cna_graphics_device_get_renderer_limit_ext_fn
            ),
            graphics_device_format_support: symbol!(cna_graphics_device_get_surface_format_support_ext,
                sys::cna_graphics_device_get_surface_format_support_ext_fn
            ),
            graphics_device_capability_report_size: symbol!(cna_graphics_device_get_capability_report_size_ext,
                sys::cna_graphics_device_get_capability_report_size_ext_fn
            ),
            graphics_device_copy_capability_report: symbol!(cna_graphics_device_copy_capability_report_ext,
                sys::cna_graphics_device_copy_capability_report_ext_fn
            ),
            graphics_device_shader_dialect: symbol!(cna_graphics_device_get_shader_dialect_ext,
                sys::cna_graphics_device_get_shader_dialect_ext_fn
            ),
            graphics_device_get_renderer_name_size: symbol!(cna_graphics_device_get_renderer_name_size,
                sys::cna_graphics_device_get_renderer_name_size_fn
            ),
            graphics_device_copy_renderer_name: symbol!(cna_graphics_device_copy_renderer_name,
                sys::cna_graphics_device_copy_renderer_name_fn
            ),
            texture2d_create_from_encoded_memory: symbol!(cna_texture2d_create_from_encoded_memory,
                sys::cna_texture2d_create_from_encoded_memory_fn
            ),
            texture2d_create: symbol!(cna_texture2d_create, sys::cna_texture2d_create_fn),
            texture2d_get_info: symbol!(cna_texture2d_get_info, sys::cna_texture2d_get_info_fn),
            texture2d_set_data: symbol!(cna_texture2d_set_data, sys::cna_texture2d_set_data_fn),
            texture2d_get_data: symbol!(cna_texture2d_get_data, sys::cna_texture2d_get_data_fn),
            texture2d_get_encoded_byte_count: symbol!(cna_texture2d_get_encoded_byte_count,
                sys::cna_texture2d_get_encoded_byte_count_fn
            ),
            texture2d_copy_encoded: symbol!(cna_texture2d_copy_encoded,
                sys::cna_texture2d_copy_encoded_fn
            ),
            texture2d_destroy: symbol!(cna_texture2d_destroy, sys::cna_texture2d_destroy_fn),
            texture3d_create: symbol!(cna_texture3d_create, sys::cna_texture3d_create_fn),
            texture3d_destroy: symbol!(cna_texture3d_destroy, sys::cna_texture3d_destroy_fn),
            texture3d_get_info: symbol!(cna_texture3d_get_info, sys::cna_texture3d_get_info_fn),
            texture3d_set_data: symbol!(cna_texture3d_set_data, sys::cna_texture3d_set_data_fn),
            texture3d_get_data: symbol!(cna_texture3d_get_data, sys::cna_texture3d_get_data_fn),
            texturecube_create: symbol!(cna_texturecube_create, sys::cna_texturecube_create_fn),
            texturecube_destroy: symbol!(cna_texturecube_destroy,
                sys::cna_texturecube_destroy_fn
            ),
            texturecube_get_info: symbol!(cna_texturecube_get_info,
                sys::cna_texturecube_get_info_fn
            ),
            texturecube_set_data: symbol!(cna_texturecube_set_data,
                sys::cna_texturecube_set_data_fn
            ),
            texturecube_get_data: symbol!(cna_texturecube_get_data,
                sys::cna_texturecube_get_data_fn
            ),
            render_target2d_create: symbol!(cna_render_target2d_create,
                sys::cna_render_target2d_create_fn
            ),
            render_target_cube_create: symbol!(cna_render_target_cube_create,
                sys::cna_render_target_cube_create_fn
            ),
            render_target_get_info: symbol!(cna_render_target_get_info,
                sys::cna_render_target_get_info_fn
            ),
            render_target_destroy: symbol!(cna_render_target_destroy,
                sys::cna_render_target_destroy_fn
            ),
            vertex_declaration_create_with_stride: symbol!(cna_vertex_declaration_create_with_stride,
                sys::cna_vertex_declaration_create_with_stride_fn
            ),
            vertex_declaration_destroy: symbol!(cna_vertex_declaration_destroy,
                sys::cna_vertex_declaration_destroy_fn
            ),
            vertex_buffer_binding_init: symbol!(cna_vertex_buffer_binding_init,
                sys::cna_vertex_buffer_binding_init_fn
            ),
            vertex_buffer_create: symbol!(cna_vertex_buffer_create,
                sys::cna_vertex_buffer_create_fn
            ),
            vertex_buffer_destroy: symbol!(cna_vertex_buffer_destroy,
                sys::cna_vertex_buffer_destroy_fn
            ),
            vertex_buffer_get_info: symbol!(cna_vertex_buffer_get_info,
                sys::cna_vertex_buffer_get_info_fn
            ),
            vertex_buffer_set_data: symbol!(cna_vertex_buffer_set_data,
                sys::cna_vertex_buffer_set_data_fn
            ),
            vertex_buffer_set_data_raw: symbol!(cna_vertex_buffer_set_data_raw,
                sys::cna_vertex_buffer_set_data_raw_fn
            ),
            vertex_buffer_set_data_raw_at: symbol!(cna_vertex_buffer_set_data_raw_at,
                sys::cna_vertex_buffer_set_data_raw_at_fn
            ),
            vertex_buffer_set_data_raw_with_options: symbol!(cna_vertex_buffer_set_data_raw_with_options,
                sys::cna_vertex_buffer_set_data_raw_with_options_fn
            ),
            vertex_buffer_set_data_raw_at_with_options: symbol!(cna_vertex_buffer_set_data_raw_at_with_options,
                sys::cna_vertex_buffer_set_data_raw_at_with_options_fn
            ),
            vertex_buffer_get_data_raw: symbol!(cna_vertex_buffer_get_data_raw,
                sys::cna_vertex_buffer_get_data_raw_fn
            ),
            index_buffer_create: symbol!(cna_index_buffer_create,
                sys::cna_index_buffer_create_fn
            ),
            index_buffer_destroy: symbol!(cna_index_buffer_destroy,
                sys::cna_index_buffer_destroy_fn
            ),
            index_buffer_get_info: symbol!(cna_index_buffer_get_info,
                sys::cna_index_buffer_get_info_fn
            ),
            index_buffer_set_data: symbol!(cna_index_buffer_set_data,
                sys::cna_index_buffer_set_data_fn
            ),
            index_buffer_set_data_at: symbol!(cna_index_buffer_set_data_at,
                sys::cna_index_buffer_set_data_at_fn
            ),
            index_buffer_get_data: symbol!(cna_index_buffer_get_data,
                sys::cna_index_buffer_get_data_fn
            ),
            sprite_batch_create: symbol!(cna_sprite_batch_create,
                sys::cna_sprite_batch_create_fn
            ),
            sprite_batch_begin: symbol!(cna_sprite_batch_begin, sys::cna_sprite_batch_begin_fn),
            sprite_batch_begin_with_states: symbol!(cna_sprite_batch_begin_with_states,
                sys::cna_sprite_batch_begin_with_states_fn
            ),
            sprite_batch_begin_with_effect: symbol!(cna_sprite_batch_begin_with_effect,
                sys::cna_sprite_batch_begin_with_effect_fn
            ),
            sprite_batch_submit_many: symbol!(cna_sprite_batch_submit_many,
                sys::cna_sprite_batch_submit_many_fn
            ),
            sprite_batch_end: symbol!(cna_sprite_batch_end, sys::cna_sprite_batch_end_fn),
            sprite_batch_destroy: symbol!(cna_sprite_batch_destroy,
                sys::cna_sprite_batch_destroy_fn
            ),
            sprite_batch_draw_string: symbol!(cna_sprite_batch_draw_string,
                sys::cna_sprite_batch_draw_string_fn
            ),
            sprite_font_create: symbol!(cna_sprite_font_create, sys::cna_sprite_font_create_fn),
            sprite_font_get_info: symbol!(cna_sprite_font_get_info,
                sys::cna_sprite_font_get_info_fn
            ),
            sprite_font_copy_characters: symbol!(cna_sprite_font_copy_characters,
                sys::cna_sprite_font_copy_characters_fn
            ),
            sprite_font_copy_glyphs: symbol!(cna_sprite_font_copy_glyphs,
                sys::cna_sprite_font_copy_glyphs_fn
            ),
            sprite_font_set_default_character: symbol!(cna_sprite_font_set_default_character,
                sys::cna_sprite_font_set_default_character_fn
            ),
            sprite_font_set_line_spacing: symbol!(cna_sprite_font_set_line_spacing,
                sys::cna_sprite_font_set_line_spacing_fn
            ),
            sprite_font_set_spacing: symbol!(cna_sprite_font_set_spacing,
                sys::cna_sprite_font_set_spacing_fn
            ),
            sprite_font_measure_utf8: symbol!(cna_sprite_font_measure_utf8,
                sys::cna_sprite_font_measure_utf8_fn
            ),
            sprite_font_destroy: symbol!(cna_sprite_font_destroy,
                sys::cna_sprite_font_destroy_fn
            ),
            effect_create_empty: symbol!(cna_effect_create_empty,
                sys::cna_effect_create_empty_fn
            ),
            effect_create_compiled: symbol!(cna_effect_create_compiled,
                sys::cna_effect_create_compiled_fn
            ),
            effect_material_create: symbol!(cna_effect_material_create,
                sys::cna_effect_material_create_fn
            ),
            effect_destroy: symbol!(cna_effect_destroy, sys::cna_effect_destroy_fn),
            effect_clone: symbol!(cna_effect_clone, sys::cna_effect_clone_fn),
            effect_dispose: symbol!(cna_effect_dispose, sys::cna_effect_dispose_fn),
            effect_apply: symbol!(cna_effect_apply, sys::cna_effect_apply_fn),
            effect_get_parameters: symbol!(cna_effect_get_parameters,
                sys::cna_effect_get_parameters_fn
            ),
            effect_get_techniques: symbol!(cna_effect_get_techniques,
                sys::cna_effect_get_techniques_fn
            ),
            effect_get_current_technique: symbol!(cna_effect_get_current_technique,
                sys::cna_effect_get_current_technique_fn
            ),
            effect_set_current_technique: symbol!(cna_effect_set_current_technique,
                sys::cna_effect_set_current_technique_fn
            ),
            directional_light_create: symbol!(cna_directional_light_create,
                sys::cna_directional_light_create_fn
            ),
            directional_light_destroy: symbol!(cna_directional_light_destroy,
                sys::cna_directional_light_destroy_fn
            ),
            directional_light_get_diffuse_color: symbol!(cna_directional_light_get_diffuse_color,
                sys::cna_directional_light_get_diffuse_color_fn
            ),
            directional_light_set_diffuse_color: symbol!(cna_directional_light_set_diffuse_color,
                sys::cna_directional_light_set_diffuse_color_fn
            ),
            directional_light_get_direction: symbol!(cna_directional_light_get_direction,
                sys::cna_directional_light_get_direction_fn
            ),
            directional_light_set_direction: symbol!(cna_directional_light_set_direction,
                sys::cna_directional_light_set_direction_fn
            ),
            directional_light_get_specular_color: symbol!(cna_directional_light_get_specular_color,
                sys::cna_directional_light_get_specular_color_fn
            ),
            directional_light_set_specular_color: symbol!(cna_directional_light_set_specular_color,
                sys::cna_directional_light_set_specular_color_fn
            ),
            directional_light_get_enabled: symbol!(cna_directional_light_get_enabled,
                sys::cna_directional_light_get_enabled_fn
            ),
            directional_light_set_enabled: symbol!(cna_directional_light_set_enabled,
                sys::cna_directional_light_set_enabled_fn
            ),
            basic_effect_create: symbol!(cna_basic_effect_create,
                sys::cna_basic_effect_create_fn
            ),
            graphics_device_clear_color_depth: symbol!(cna_graphics_device_clear_color_depth, sys::cna_graphics_device_clear_color_depth_fn),
            graphics_device_dispose: symbol!(cna_graphics_device_dispose, sys::cna_graphics_device_dispose_fn),
            graphics_device_executes_shader_effect_source_ext: symbol!(cna_graphics_device_executes_shader_effect_source_ext, sys::cna_graphics_device_executes_shader_effect_source_ext_fn),
            graphics_device_get_display_color_space_ext: symbol!(cna_graphics_device_get_display_color_space_ext, sys::cna_graphics_device_get_display_color_space_ext_fn),
            graphics_device_get_is_disposed: symbol!(cna_graphics_device_get_is_disposed, sys::cna_graphics_device_get_is_disposed_fn),
            graphics_device_get_max_compute_work_group_count_ext: symbol!(cna_graphics_device_get_max_compute_work_group_count_ext, sys::cna_graphics_device_get_max_compute_work_group_count_ext_fn),
            graphics_device_get_max_compute_work_group_invocations_ext: symbol!(cna_graphics_device_get_max_compute_work_group_invocations_ext, sys::cna_graphics_device_get_max_compute_work_group_invocations_ext_fn),
            graphics_device_get_max_compute_work_group_size_ext: symbol!(cna_graphics_device_get_max_compute_work_group_size_ext, sys::cna_graphics_device_get_max_compute_work_group_size_ext_fn),
            graphics_device_get_tracked_resource_count: symbol!(cna_graphics_device_get_tracked_resource_count, sys::cna_graphics_device_get_tracked_resource_count_fn),
            graphics_device_get_unsupported_3d_call_behavior: symbol!(cna_graphics_device_get_unsupported_3d_call_behavior, sys::cna_graphics_device_get_unsupported_3d_call_behavior_fn),
            graphics_device_notify_content_lost_resources_ext: symbol!(cna_graphics_device_notify_content_lost_resources_ext, sys::cna_graphics_device_notify_content_lost_resources_ext_fn),
            graphics_device_recreate_renderer_for_multi_sample_count_ext: symbol!(cna_graphics_device_recreate_renderer_for_multi_sample_count_ext, sys::cna_graphics_device_recreate_renderer_for_multi_sample_count_ext_fn),
            graphics_device_set_blend_enabled: symbol!(cna_graphics_device_set_blend_enabled, sys::cna_graphics_device_set_blend_enabled_fn),
            graphics_device_set_context_recovery_enabled: symbol!(cna_graphics_device_set_context_recovery_enabled, sys::cna_graphics_device_set_context_recovery_enabled_fn),
            graphics_device_set_current_effect: symbol!(cna_graphics_device_set_current_effect, sys::cna_graphics_device_set_current_effect_fn),
            graphics_device_set_depth_test_enabled: symbol!(cna_graphics_device_set_depth_test_enabled, sys::cna_graphics_device_set_depth_test_enabled_fn),
            graphics_device_set_depth_write_enabled: symbol!(cna_graphics_device_set_depth_write_enabled, sys::cna_graphics_device_set_depth_write_enabled_fn),
            graphics_device_set_display_color_space_ext: symbol!(cna_graphics_device_set_display_color_space_ext, sys::cna_graphics_device_set_display_color_space_ext_fn),
            graphics_device_set_graphics_profile_ext: symbol!(cna_graphics_device_set_graphics_profile_ext, sys::cna_graphics_device_set_graphics_profile_ext_fn),
            graphics_device_set_string_marker_ext: symbol!(cna_graphics_device_set_string_marker_ext, sys::cna_graphics_device_set_string_marker_ext_fn),
            graphics_device_set_unsupported_3d_call_behavior: symbol!(cna_graphics_device_set_unsupported_3d_call_behavior, sys::cna_graphics_device_set_unsupported_3d_call_behavior_fn),
            graphics_device_subscribe_event: symbol!(cna_graphics_device_subscribe_event, sys::cna_graphics_device_subscribe_event_fn),
            graphics_device_subscribe_resource_created: symbol!(cna_graphics_device_subscribe_resource_created, sys::cna_graphics_device_subscribe_resource_created_fn),
            graphics_device_subscribe_resource_destroyed: symbol!(cna_graphics_device_subscribe_resource_destroyed, sys::cna_graphics_device_subscribe_resource_destroyed_fn),
            graphics_device_supports_display_color_space_ext: symbol!(cna_graphics_device_supports_display_color_space_ext, sys::cna_graphics_device_supports_display_color_space_ext_fn),
            graphics_device_supports_image_based_lighting_ext: symbol!(cna_graphics_device_supports_image_based_lighting_ext, sys::cna_graphics_device_supports_image_based_lighting_ext_fn),
            graphics_device_supports_surface_format_as_render_target_ext: symbol!(cna_graphics_device_supports_surface_format_as_render_target_ext, sys::cna_graphics_device_supports_surface_format_as_render_target_ext_fn),
            graphics_device_unbind_texture: symbol!(cna_graphics_device_unbind_texture, sys::cna_graphics_device_unbind_texture_fn),
            graphics_device_unsubscribe: symbol!(cna_graphics_device_unsubscribe, sys::cna_graphics_device_unsubscribe_fn),
            occlusion_query_get_is_pixel_count_precise_ext: symbol!(cna_occlusion_query_get_is_pixel_count_precise_ext, sys::cna_occlusion_query_get_is_pixel_count_precise_ext_fn),
            occlusion_query_has_renderer: symbol!(cna_occlusion_query_has_renderer, sys::cna_occlusion_query_has_renderer_fn),
            primitive_type_get_vertex_count: symbol!(cna_primitive_type_get_vertex_count, sys::cna_primitive_type_get_vertex_count_fn),
            alpha_test_effect_get_texture: symbol!(cna_alpha_test_effect_get_texture, sys::cna_alpha_test_effect_get_texture_fn),
            basic_effect_get_texture: symbol!(cna_basic_effect_get_texture, sys::cna_basic_effect_get_texture_fn),
            color_matrix_effect_create: symbol!(cna_color_matrix_effect_create, sys::cna_color_matrix_effect_create_fn),
            color_matrix_effect_get_matrix: symbol!(cna_color_matrix_effect_get_matrix, sys::cna_color_matrix_effect_get_matrix_fn),
            color_matrix_effect_get_offset: symbol!(cna_color_matrix_effect_get_offset, sys::cna_color_matrix_effect_get_offset_fn),
            color_matrix_effect_reset: symbol!(cna_color_matrix_effect_reset, sys::cna_color_matrix_effect_reset_fn),
            color_matrix_effect_set_grayscale: symbol!(cna_color_matrix_effect_set_grayscale, sys::cna_color_matrix_effect_set_grayscale_fn),
            color_matrix_effect_set_matrix: symbol!(cna_color_matrix_effect_set_matrix, sys::cna_color_matrix_effect_set_matrix_fn),
            color_matrix_effect_set_offset: symbol!(cna_color_matrix_effect_set_offset, sys::cna_color_matrix_effect_set_offset_fn),
            content_manager_load_effect: symbol!(cna_content_manager_load_effect, sys::cna_content_manager_load_effect_fn),
            dual_texture_effect_get_texture: symbol!(cna_dual_texture_effect_get_texture, sys::cna_dual_texture_effect_get_texture_fn),
            effect_copy_fragment_source: symbol!(cna_effect_copy_fragment_source, sys::cna_effect_copy_fragment_source_fn),
            effect_copy_vertex_source: symbol!(cna_effect_copy_vertex_source, sys::cna_effect_copy_vertex_source_fn),
            effect_get_fragment_source_byte_count: symbol!(cna_effect_get_fragment_source_byte_count, sys::cna_effect_get_fragment_source_byte_count_fn),
            effect_get_graphics_device: symbol!(cna_effect_get_graphics_device, sys::cna_effect_get_graphics_device_fn),
            effect_get_is_compiled_ext: symbol!(cna_effect_get_is_compiled_ext, sys::cna_effect_get_is_compiled_ext_fn),
            effect_get_vertex_source_byte_count: symbol!(cna_effect_get_vertex_source_byte_count, sys::cna_effect_get_vertex_source_byte_count_fn),
            effect_has_renderer: symbol!(cna_effect_has_renderer, sys::cna_effect_has_renderer_fn),
            effect_is_exact_stock_sprite_effect: symbol!(cna_effect_is_exact_stock_sprite_effect, sys::cna_effect_is_exact_stock_sprite_effect_fn),
            effect_material_get_retained_parameter_texture_count_ext: symbol!(cna_effect_material_get_retained_parameter_texture_count_ext, sys::cna_effect_material_get_retained_parameter_texture_count_ext_fn),
            effect_material_retain_parameter_texture_ext: symbol!(cna_effect_material_retain_parameter_texture_ext, sys::cna_effect_material_retain_parameter_texture_ext_fn),
            effect_pass_get_index_ext: symbol!(cna_effect_pass_get_index_ext, sys::cna_effect_pass_get_index_ext_fn),
            effect_technique_get_identity: symbol!(cna_effect_technique_get_identity, sys::cna_effect_technique_get_identity_fn),
            effect_technique_get_index_ext: symbol!(cna_effect_technique_get_index_ext, sys::cna_effect_technique_get_index_ext_fn),
            environment_map_effect_get_environment_map: symbol!(cna_environment_map_effect_get_environment_map, sys::cna_environment_map_effect_get_environment_map_fn),
            environment_map_effect_get_texture: symbol!(cna_environment_map_effect_get_texture, sys::cna_environment_map_effect_get_texture_fn),
            pbr_effect_get_encode_output_to_srgb_ext: symbol!(cna_pbr_effect_get_encode_output_to_srgb_ext, sys::cna_pbr_effect_get_encode_output_to_srgb_ext_fn),
            pbr_effect_get_specular_color_factor_ext: symbol!(cna_pbr_effect_get_specular_color_factor_ext, sys::cna_pbr_effect_get_specular_color_factor_ext_fn),
            pbr_effect_get_texture: symbol!(cna_pbr_effect_get_texture, sys::cna_pbr_effect_get_texture_fn),
            pbr_effect_get_texture_coordinate_set_ext: symbol!(cna_pbr_effect_get_texture_coordinate_set_ext, sys::cna_pbr_effect_get_texture_coordinate_set_ext_fn),
            pbr_effect_get_texture_is_srgb_ext: symbol!(cna_pbr_effect_get_texture_is_srgb_ext, sys::cna_pbr_effect_get_texture_is_srgb_ext_fn),
            pbr_effect_get_texture_transform_ext: symbol!(cna_pbr_effect_get_texture_transform_ext, sys::cna_pbr_effect_get_texture_transform_ext_fn),
            pbr_effect_set_encode_output_to_srgb_ext: symbol!(cna_pbr_effect_set_encode_output_to_srgb_ext, sys::cna_pbr_effect_set_encode_output_to_srgb_ext_fn),
            pbr_effect_set_specular_color_factor_ext: symbol!(cna_pbr_effect_set_specular_color_factor_ext, sys::cna_pbr_effect_set_specular_color_factor_ext_fn),
            pbr_effect_set_texture: symbol!(cna_pbr_effect_set_texture, sys::cna_pbr_effect_set_texture_fn),
            pbr_effect_set_texture_coordinate_set_ext: symbol!(cna_pbr_effect_set_texture_coordinate_set_ext, sys::cna_pbr_effect_set_texture_coordinate_set_ext_fn),
            pbr_effect_set_texture_is_srgb_ext: symbol!(cna_pbr_effect_set_texture_is_srgb_ext, sys::cna_pbr_effect_set_texture_is_srgb_ext_fn),
            pbr_effect_set_texture_transform_ext: symbol!(cna_pbr_effect_set_texture_transform_ext, sys::cna_pbr_effect_set_texture_transform_ext_fn),
            shader_effect_copy_compile_error_ext: symbol!(cna_shader_effect_copy_compile_error_ext, sys::cna_shader_effect_copy_compile_error_ext_fn),
            shader_effect_create: symbol!(cna_shader_effect_create, sys::cna_shader_effect_create_fn),
            shader_effect_declare_uniform_block_ext: symbol!(cna_shader_effect_declare_uniform_block_ext, sys::cna_shader_effect_declare_uniform_block_ext_fn),
            shader_effect_get_projection: symbol!(cna_shader_effect_get_projection, sys::cna_shader_effect_get_projection_fn),
            shader_effect_get_view: symbol!(cna_shader_effect_get_view, sys::cna_shader_effect_get_view_fn),
            shader_effect_get_world: symbol!(cna_shader_effect_get_world, sys::cna_shader_effect_get_world_fn),
            shader_effect_has_renderer: symbol!(cna_shader_effect_has_renderer, sys::cna_shader_effect_has_renderer_fn),
            shader_effect_is_valid: symbol!(cna_shader_effect_is_valid, sys::cna_shader_effect_is_valid_fn),
            shader_effect_set_projection: symbol!(cna_shader_effect_set_projection, sys::cna_shader_effect_set_projection_fn),
            shader_effect_set_texture2d: symbol!(cna_shader_effect_set_texture2d, sys::cna_shader_effect_set_texture2d_fn),
            shader_effect_set_texture3d: symbol!(cna_shader_effect_set_texture3d, sys::cna_shader_effect_set_texture3d_fn),
            shader_effect_set_texture_cube: symbol!(cna_shader_effect_set_texture_cube, sys::cna_shader_effect_set_texture_cube_fn),
            shader_effect_set_uniform_float: symbol!(cna_shader_effect_set_uniform_float, sys::cna_shader_effect_set_uniform_float_fn),
            shader_effect_set_uniform_float_array: symbol!(cna_shader_effect_set_uniform_float_array, sys::cna_shader_effect_set_uniform_float_array_fn),
            shader_effect_set_uniform_int32: symbol!(cna_shader_effect_set_uniform_int32, sys::cna_shader_effect_set_uniform_int32_fn),
            shader_effect_set_uniform_mat4_array: symbol!(cna_shader_effect_set_uniform_mat4_array, sys::cna_shader_effect_set_uniform_mat4_array_fn),
            shader_effect_set_uniform_matrix: symbol!(cna_shader_effect_set_uniform_matrix, sys::cna_shader_effect_set_uniform_matrix_fn),
            shader_effect_set_uniform_vec3_array: symbol!(cna_shader_effect_set_uniform_vec3_array, sys::cna_shader_effect_set_uniform_vec3_array_fn),
            shader_effect_set_uniform_vector2: symbol!(cna_shader_effect_set_uniform_vector2, sys::cna_shader_effect_set_uniform_vector2_fn),
            shader_effect_set_uniform_vector2_array: symbol!(cna_shader_effect_set_uniform_vector2_array, sys::cna_shader_effect_set_uniform_vector2_array_fn),
            shader_effect_set_uniform_vector3: symbol!(cna_shader_effect_set_uniform_vector3, sys::cna_shader_effect_set_uniform_vector3_fn),
            shader_effect_set_uniform_vector4: symbol!(cna_shader_effect_set_uniform_vector4, sys::cna_shader_effect_set_uniform_vector4_fn),
            shader_effect_set_view: symbol!(cna_shader_effect_set_view, sys::cna_shader_effect_set_view_fn),
            shader_effect_set_world: symbol!(cna_shader_effect_set_world, sys::cna_shader_effect_set_world_fn),
            skinned_effect_get_texture: symbol!(cna_skinned_effect_get_texture, sys::cna_skinned_effect_get_texture_fn),
            skinned_effect_get_vertex_color_enabled: symbol!(cna_skinned_effect_get_vertex_color_enabled, sys::cna_skinned_effect_get_vertex_color_enabled_fn),
            skinned_effect_set_vertex_color_enabled: symbol!(cna_skinned_effect_set_vertex_color_enabled, sys::cna_skinned_effect_set_vertex_color_enabled_fn),
            sprite_effect_create: symbol!(cna_sprite_effect_create, sys::cna_sprite_effect_create_fn),
            effect_matrices_get_world: symbol!(cna_effect_matrices_get_world,
                sys::cna_effect_matrices_get_world_fn
            ),
            effect_matrices_set_world: symbol!(cna_effect_matrices_set_world,
                sys::cna_effect_matrices_set_world_fn
            ),
            effect_matrices_get_view: symbol!(cna_effect_matrices_get_view,
                sys::cna_effect_matrices_get_view_fn
            ),
            effect_matrices_set_view: symbol!(cna_effect_matrices_set_view,
                sys::cna_effect_matrices_set_view_fn
            ),
            effect_matrices_get_projection: symbol!(cna_effect_matrices_get_projection,
                sys::cna_effect_matrices_get_projection_fn
            ),
            effect_matrices_set_projection: symbol!(cna_effect_matrices_set_projection,
                sys::cna_effect_matrices_set_projection_fn
            ),
            effect_fog_get_color: symbol!(cna_effect_fog_get_color,
                sys::cna_effect_fog_get_color_fn
            ),
            effect_fog_set_color: symbol!(cna_effect_fog_set_color,
                sys::cna_effect_fog_set_color_fn
            ),
            effect_fog_get_enabled: symbol!(cna_effect_fog_get_enabled,
                sys::cna_effect_fog_get_enabled_fn
            ),
            effect_fog_set_enabled: symbol!(cna_effect_fog_set_enabled,
                sys::cna_effect_fog_set_enabled_fn
            ),
            effect_fog_get_start: symbol!(cna_effect_fog_get_start,
                sys::cna_effect_fog_get_start_fn
            ),
            effect_fog_set_start: symbol!(cna_effect_fog_set_start,
                sys::cna_effect_fog_set_start_fn
            ),
            effect_fog_get_end: symbol!(cna_effect_fog_get_end, sys::cna_effect_fog_get_end_fn),
            effect_fog_set_end: symbol!(cna_effect_fog_set_end, sys::cna_effect_fog_set_end_fn),
            effect_lights_get_ambient_color: symbol!(cna_effect_lights_get_ambient_color,
                sys::cna_effect_lights_get_ambient_color_fn
            ),
            effect_lights_set_ambient_color: symbol!(cna_effect_lights_set_ambient_color,
                sys::cna_effect_lights_set_ambient_color_fn
            ),
            effect_lights_get_directional_light: symbol!(cna_effect_lights_get_directional_light,
                sys::cna_effect_lights_get_directional_light_fn
            ),
            effect_lights_get_enabled: symbol!(cna_effect_lights_get_enabled,
                sys::cna_effect_lights_get_enabled_fn
            ),
            effect_lights_set_enabled: symbol!(cna_effect_lights_set_enabled,
                sys::cna_effect_lights_set_enabled_fn
            ),
            effect_lights_enable_default: symbol!(cna_effect_lights_enable_default,
                sys::cna_effect_lights_enable_default_fn
            ),
            basic_effect_get_vertex_color_enabled: symbol!(cna_basic_effect_get_vertex_color_enabled,
                sys::cna_basic_effect_get_vertex_color_enabled_fn
            ),
            basic_effect_set_vertex_color_enabled: symbol!(cna_basic_effect_set_vertex_color_enabled,
                sys::cna_basic_effect_set_vertex_color_enabled_fn
            ),
            basic_effect_get_prefer_per_pixel_lighting: symbol!(cna_basic_effect_get_prefer_per_pixel_lighting,
                sys::cna_basic_effect_get_prefer_per_pixel_lighting_fn
            ),
            basic_effect_set_prefer_per_pixel_lighting: symbol!(cna_basic_effect_set_prefer_per_pixel_lighting,
                sys::cna_basic_effect_set_prefer_per_pixel_lighting_fn
            ),
            basic_effect_get_diffuse_color: symbol!(cna_basic_effect_get_diffuse_color,
                sys::cna_basic_effect_get_diffuse_color_fn
            ),
            basic_effect_set_diffuse_color: symbol!(cna_basic_effect_set_diffuse_color,
                sys::cna_basic_effect_set_diffuse_color_fn
            ),
            basic_effect_get_emissive_color: symbol!(cna_basic_effect_get_emissive_color,
                sys::cna_basic_effect_get_emissive_color_fn
            ),
            basic_effect_set_emissive_color: symbol!(cna_basic_effect_set_emissive_color,
                sys::cna_basic_effect_set_emissive_color_fn
            ),
            basic_effect_get_specular_color: symbol!(cna_basic_effect_get_specular_color,
                sys::cna_basic_effect_get_specular_color_fn
            ),
            basic_effect_set_specular_color: symbol!(cna_basic_effect_set_specular_color,
                sys::cna_basic_effect_set_specular_color_fn
            ),
            basic_effect_get_specular_power: symbol!(cna_basic_effect_get_specular_power,
                sys::cna_basic_effect_get_specular_power_fn
            ),
            basic_effect_set_specular_power: symbol!(cna_basic_effect_set_specular_power,
                sys::cna_basic_effect_set_specular_power_fn
            ),
            basic_effect_get_alpha: symbol!(cna_basic_effect_get_alpha,
                sys::cna_basic_effect_get_alpha_fn
            ),
            basic_effect_set_alpha: symbol!(cna_basic_effect_set_alpha,
                sys::cna_basic_effect_set_alpha_fn
            ),
            basic_effect_get_texture_enabled: symbol!(cna_basic_effect_get_texture_enabled,
                sys::cna_basic_effect_get_texture_enabled_fn
            ),
            basic_effect_set_texture_enabled: symbol!(cna_basic_effect_set_texture_enabled,
                sys::cna_basic_effect_set_texture_enabled_fn
            ),
            basic_effect_set_texture: symbol!(cna_basic_effect_set_texture,
                sys::cna_basic_effect_set_texture_fn
            ),
            alpha_test_effect_create: symbol!(cna_alpha_test_effect_create,
                sys::cna_alpha_test_effect_create_fn
            ),
            alpha_test_effect_get_diffuse_color: symbol!(cna_alpha_test_effect_get_diffuse_color,
                sys::cna_alpha_test_effect_get_diffuse_color_fn
            ),
            alpha_test_effect_set_diffuse_color: symbol!(cna_alpha_test_effect_set_diffuse_color,
                sys::cna_alpha_test_effect_set_diffuse_color_fn
            ),
            alpha_test_effect_get_alpha: symbol!(cna_alpha_test_effect_get_alpha,
                sys::cna_alpha_test_effect_get_alpha_fn
            ),
            alpha_test_effect_set_alpha: symbol!(cna_alpha_test_effect_set_alpha,
                sys::cna_alpha_test_effect_set_alpha_fn
            ),
            alpha_test_effect_set_texture: symbol!(cna_alpha_test_effect_set_texture,
                sys::cna_alpha_test_effect_set_texture_fn
            ),
            alpha_test_effect_get_vertex_color_enabled: symbol!(cna_alpha_test_effect_get_vertex_color_enabled,
                sys::cna_alpha_test_effect_get_vertex_color_enabled_fn
            ),
            alpha_test_effect_set_vertex_color_enabled: symbol!(cna_alpha_test_effect_set_vertex_color_enabled,
                sys::cna_alpha_test_effect_set_vertex_color_enabled_fn
            ),
            alpha_test_effect_get_alpha_function: symbol!(cna_alpha_test_effect_get_alpha_function,
                sys::cna_alpha_test_effect_get_alpha_function_fn
            ),
            alpha_test_effect_set_alpha_function: symbol!(cna_alpha_test_effect_set_alpha_function,
                sys::cna_alpha_test_effect_set_alpha_function_fn
            ),
            alpha_test_effect_get_reference_alpha: symbol!(cna_alpha_test_effect_get_reference_alpha,
                sys::cna_alpha_test_effect_get_reference_alpha_fn
            ),
            alpha_test_effect_set_reference_alpha: symbol!(cna_alpha_test_effect_set_reference_alpha,
                sys::cna_alpha_test_effect_set_reference_alpha_fn
            ),
            dual_texture_effect_create: symbol!(cna_dual_texture_effect_create,
                sys::cna_dual_texture_effect_create_fn
            ),
            dual_texture_effect_get_diffuse_color: symbol!(cna_dual_texture_effect_get_diffuse_color,
                sys::cna_dual_texture_effect_get_diffuse_color_fn
            ),
            dual_texture_effect_set_diffuse_color: symbol!(cna_dual_texture_effect_set_diffuse_color,
                sys::cna_dual_texture_effect_set_diffuse_color_fn
            ),
            dual_texture_effect_get_alpha: symbol!(cna_dual_texture_effect_get_alpha,
                sys::cna_dual_texture_effect_get_alpha_fn
            ),
            dual_texture_effect_set_alpha: symbol!(cna_dual_texture_effect_set_alpha,
                sys::cna_dual_texture_effect_set_alpha_fn
            ),
            dual_texture_effect_set_texture: symbol!(cna_dual_texture_effect_set_texture,
                sys::cna_dual_texture_effect_set_texture_fn
            ),
            dual_texture_effect_get_vertex_color_enabled: symbol!(cna_dual_texture_effect_get_vertex_color_enabled,
                sys::cna_dual_texture_effect_get_vertex_color_enabled_fn
            ),
            dual_texture_effect_set_vertex_color_enabled: symbol!(cna_dual_texture_effect_set_vertex_color_enabled,
                sys::cna_dual_texture_effect_set_vertex_color_enabled_fn
            ),
            environment_map_effect_create: symbol!(cna_environment_map_effect_create,
                sys::cna_environment_map_effect_create_fn
            ),
            environment_map_effect_get_diffuse_color: symbol!(cna_environment_map_effect_get_diffuse_color,
                sys::cna_environment_map_effect_get_diffuse_color_fn
            ),
            environment_map_effect_set_diffuse_color: symbol!(cna_environment_map_effect_set_diffuse_color,
                sys::cna_environment_map_effect_set_diffuse_color_fn
            ),
            environment_map_effect_get_emissive_color: symbol!(cna_environment_map_effect_get_emissive_color,
                sys::cna_environment_map_effect_get_emissive_color_fn
            ),
            environment_map_effect_set_emissive_color: symbol!(cna_environment_map_effect_set_emissive_color,
                sys::cna_environment_map_effect_set_emissive_color_fn
            ),
            environment_map_effect_get_alpha: symbol!(cna_environment_map_effect_get_alpha,
                sys::cna_environment_map_effect_get_alpha_fn
            ),
            environment_map_effect_set_alpha: symbol!(cna_environment_map_effect_set_alpha,
                sys::cna_environment_map_effect_set_alpha_fn
            ),
            environment_map_effect_set_texture: symbol!(cna_environment_map_effect_set_texture,
                sys::cna_environment_map_effect_set_texture_fn
            ),
            environment_map_effect_set_environment_map: symbol!(cna_environment_map_effect_set_environment_map,
                sys::cna_environment_map_effect_set_environment_map_fn
            ),
            environment_map_effect_get_amount: symbol!(cna_environment_map_effect_get_amount,
                sys::cna_environment_map_effect_get_amount_fn
            ),
            environment_map_effect_set_amount: symbol!(cna_environment_map_effect_set_amount,
                sys::cna_environment_map_effect_set_amount_fn
            ),
            environment_map_effect_get_specular: symbol!(cna_environment_map_effect_get_specular,
                sys::cna_environment_map_effect_get_specular_fn
            ),
            environment_map_effect_set_specular: symbol!(cna_environment_map_effect_set_specular,
                sys::cna_environment_map_effect_set_specular_fn
            ),
            environment_map_effect_get_fresnel_factor: symbol!(cna_environment_map_effect_get_fresnel_factor,
                sys::cna_environment_map_effect_get_fresnel_factor_fn
            ),
            environment_map_effect_set_fresnel_factor: symbol!(cna_environment_map_effect_set_fresnel_factor,
                sys::cna_environment_map_effect_set_fresnel_factor_fn
            ),
            skinned_effect_create: symbol!(cna_skinned_effect_create,
                sys::cna_skinned_effect_create_fn
            ),
            skinned_effect_get_diffuse_color: symbol!(cna_skinned_effect_get_diffuse_color,
                sys::cna_skinned_effect_get_diffuse_color_fn
            ),
            skinned_effect_set_diffuse_color: symbol!(cna_skinned_effect_set_diffuse_color,
                sys::cna_skinned_effect_set_diffuse_color_fn
            ),
            skinned_effect_get_emissive_color: symbol!(cna_skinned_effect_get_emissive_color,
                sys::cna_skinned_effect_get_emissive_color_fn
            ),
            skinned_effect_set_emissive_color: symbol!(cna_skinned_effect_set_emissive_color,
                sys::cna_skinned_effect_set_emissive_color_fn
            ),
            skinned_effect_get_specular_color: symbol!(cna_skinned_effect_get_specular_color,
                sys::cna_skinned_effect_get_specular_color_fn
            ),
            skinned_effect_set_specular_color: symbol!(cna_skinned_effect_set_specular_color,
                sys::cna_skinned_effect_set_specular_color_fn
            ),
            skinned_effect_get_specular_power: symbol!(cna_skinned_effect_get_specular_power,
                sys::cna_skinned_effect_get_specular_power_fn
            ),
            skinned_effect_set_specular_power: symbol!(cna_skinned_effect_set_specular_power,
                sys::cna_skinned_effect_set_specular_power_fn
            ),
            skinned_effect_get_alpha: symbol!(cna_skinned_effect_get_alpha,
                sys::cna_skinned_effect_get_alpha_fn
            ),
            skinned_effect_set_alpha: symbol!(cna_skinned_effect_set_alpha,
                sys::cna_skinned_effect_set_alpha_fn
            ),
            skinned_effect_get_prefer_per_pixel_lighting: symbol!(cna_skinned_effect_get_prefer_per_pixel_lighting,
                sys::cna_skinned_effect_get_prefer_per_pixel_lighting_fn
            ),
            skinned_effect_set_prefer_per_pixel_lighting: symbol!(cna_skinned_effect_set_prefer_per_pixel_lighting,
                sys::cna_skinned_effect_set_prefer_per_pixel_lighting_fn
            ),
            skinned_effect_set_texture: symbol!(cna_skinned_effect_set_texture,
                sys::cna_skinned_effect_set_texture_fn
            ),
            skinned_effect_get_weights_per_vertex: symbol!(cna_skinned_effect_get_weights_per_vertex,
                sys::cna_skinned_effect_get_weights_per_vertex_fn
            ),
            skinned_effect_set_weights_per_vertex: symbol!(cna_skinned_effect_set_weights_per_vertex,
                sys::cna_skinned_effect_set_weights_per_vertex_fn
            ),
            skinned_effect_set_bone_transforms: symbol!(cna_skinned_effect_set_bone_transforms,
                sys::cna_skinned_effect_set_bone_transforms_fn
            ),
            skinned_effect_copy_bone_transforms: symbol!(cna_skinned_effect_copy_bone_transforms,
                sys::cna_skinned_effect_copy_bone_transforms_fn
            ),
            effect_annotation_create: symbol!(cna_effect_annotation_create,
                sys::cna_effect_annotation_create_fn
            ),
            effect_annotation_destroy: symbol!(cna_effect_annotation_destroy,
                sys::cna_effect_annotation_destroy_fn
            ),
            effect_annotation_get_info: symbol!(cna_effect_annotation_get_info,
                sys::cna_effect_annotation_get_info_fn
            ),
            effect_annotation_get_name_byte_count: symbol!(cna_effect_annotation_get_name_byte_count,
                sys::cna_effect_annotation_get_name_byte_count_fn
            ),
            effect_annotation_copy_name: symbol!(cna_effect_annotation_copy_name,
                sys::cna_effect_annotation_copy_name_fn
            ),
            effect_annotation_get_semantic_byte_count: symbol!(cna_effect_annotation_get_semantic_byte_count,
                sys::cna_effect_annotation_get_semantic_byte_count_fn
            ),
            effect_annotation_copy_semantic: symbol!(cna_effect_annotation_copy_semantic,
                sys::cna_effect_annotation_copy_semantic_fn
            ),
            effect_annotation_get_value_boolean: symbol!(cna_effect_annotation_get_value_boolean,
                sys::cna_effect_annotation_get_value_boolean_fn
            ),
            effect_annotation_get_value_int32: symbol!(cna_effect_annotation_get_value_int32,
                sys::cna_effect_annotation_get_value_int32_fn
            ),
            effect_annotation_get_value_single: symbol!(cna_effect_annotation_get_value_single,
                sys::cna_effect_annotation_get_value_single_fn
            ),
            effect_annotation_get_value_string_byte_count: symbol!(cna_effect_annotation_get_value_string_byte_count,
                sys::cna_effect_annotation_get_value_string_byte_count_fn
            ),
            effect_annotation_copy_value_string: symbol!(cna_effect_annotation_copy_value_string,
                sys::cna_effect_annotation_copy_value_string_fn
            ),
            effect_annotation_get_value_vector2: symbol!(cna_effect_annotation_get_value_vector2,
                sys::cna_effect_annotation_get_value_vector2_fn
            ),
            effect_annotation_get_value_vector3: symbol!(cna_effect_annotation_get_value_vector3,
                sys::cna_effect_annotation_get_value_vector3_fn
            ),
            effect_annotation_get_value_vector4: symbol!(cna_effect_annotation_get_value_vector4,
                sys::cna_effect_annotation_get_value_vector4_fn
            ),
            effect_annotation_get_value_matrix: symbol!(cna_effect_annotation_get_value_matrix,
                sys::cna_effect_annotation_get_value_matrix_fn
            ),
            effect_annotation_collection_destroy: symbol!(cna_effect_annotation_collection_destroy,
                sys::cna_effect_annotation_collection_destroy_fn
            ),
            effect_annotation_collection_add: symbol!(cna_effect_annotation_collection_add,
                sys::cna_effect_annotation_collection_add_fn
            ),
            effect_annotation_collection_get_count: symbol!(cna_effect_annotation_collection_get_count,
                sys::cna_effect_annotation_collection_get_count_fn
            ),
            effect_annotation_collection_get_at: symbol!(cna_effect_annotation_collection_get_at,
                sys::cna_effect_annotation_collection_get_at_fn
            ),
            effect_annotation_collection_find: symbol!(cna_effect_annotation_collection_find,
                sys::cna_effect_annotation_collection_find_fn
            ),
            effect_parameter_destroy: symbol!(cna_effect_parameter_destroy,
                sys::cna_effect_parameter_destroy_fn
            ),
            effect_parameter_get_info: symbol!(cna_effect_parameter_get_info,
                sys::cna_effect_parameter_get_info_fn
            ),
            effect_parameter_get_name_byte_count: symbol!(cna_effect_parameter_get_name_byte_count,
                sys::cna_effect_parameter_get_name_byte_count_fn
            ),
            effect_parameter_copy_name: symbol!(cna_effect_parameter_copy_name,
                sys::cna_effect_parameter_copy_name_fn
            ),
            effect_parameter_get_semantic_byte_count: symbol!(cna_effect_parameter_get_semantic_byte_count,
                sys::cna_effect_parameter_get_semantic_byte_count_fn
            ),
            effect_parameter_copy_semantic: symbol!(cna_effect_parameter_copy_semantic,
                sys::cna_effect_parameter_copy_semantic_fn
            ),
            effect_parameter_get_elements: symbol!(cna_effect_parameter_get_elements,
                sys::cna_effect_parameter_get_elements_fn
            ),
            effect_parameter_get_structure_members: symbol!(cna_effect_parameter_get_structure_members,
                sys::cna_effect_parameter_get_structure_members_fn
            ),
            effect_parameter_get_annotations: symbol!(cna_effect_parameter_get_annotations,
                sys::cna_effect_parameter_get_annotations_fn
            ),
            effect_parameter_get_value: symbol!(cna_effect_parameter_get_value,
                sys::cna_effect_parameter_get_value_fn
            ),
            effect_parameter_get_values: symbol!(cna_effect_parameter_get_values,
                sys::cna_effect_parameter_get_values_fn
            ),
            effect_parameter_set_value: symbol!(cna_effect_parameter_set_value,
                sys::cna_effect_parameter_set_value_fn
            ),
            effect_parameter_set_values: symbol!(cna_effect_parameter_set_values,
                sys::cna_effect_parameter_set_values_fn
            ),
            effect_parameter_get_value_string_byte_count: symbol!(cna_effect_parameter_get_value_string_byte_count,
                sys::cna_effect_parameter_get_value_string_byte_count_fn
            ),
            effect_parameter_copy_value_string: symbol!(cna_effect_parameter_copy_value_string,
                sys::cna_effect_parameter_copy_value_string_fn
            ),
            effect_parameter_set_value_string: symbol!(cna_effect_parameter_set_value_string,
                sys::cna_effect_parameter_set_value_string_fn
            ),
            effect_parameter_get_value_texture: symbol!(cna_effect_parameter_get_value_texture,
                sys::cna_effect_parameter_get_value_texture_fn
            ),
            effect_parameter_set_value_texture: symbol!(cna_effect_parameter_set_value_texture,
                sys::cna_effect_parameter_set_value_texture_fn
            ),
            effect_parameter_collection_destroy: symbol!(cna_effect_parameter_collection_destroy,
                sys::cna_effect_parameter_collection_destroy_fn
            ),
            effect_parameter_collection_add_create: symbol!(cna_effect_parameter_collection_add_create,
                sys::cna_effect_parameter_collection_add_create_fn
            ),
            effect_parameter_collection_get_count: symbol!(cna_effect_parameter_collection_get_count,
                sys::cna_effect_parameter_collection_get_count_fn
            ),
            effect_parameter_collection_get_at: symbol!(cna_effect_parameter_collection_get_at,
                sys::cna_effect_parameter_collection_get_at_fn
            ),
            effect_parameter_collection_find_name: symbol!(cna_effect_parameter_collection_find_name,
                sys::cna_effect_parameter_collection_find_name_fn
            ),
            effect_parameter_collection_find_semantic: symbol!(cna_effect_parameter_collection_find_semantic,
                sys::cna_effect_parameter_collection_find_semantic_fn
            ),
            effect_pass_destroy: symbol!(cna_effect_pass_destroy,
                sys::cna_effect_pass_destroy_fn
            ),
            effect_pass_get_name_byte_count: symbol!(cna_effect_pass_get_name_byte_count,
                sys::cna_effect_pass_get_name_byte_count_fn
            ),
            effect_pass_copy_name: symbol!(cna_effect_pass_copy_name,
                sys::cna_effect_pass_copy_name_fn
            ),
            effect_pass_get_annotations: symbol!(cna_effect_pass_get_annotations,
                sys::cna_effect_pass_get_annotations_fn
            ),
            effect_pass_apply: symbol!(cna_effect_pass_apply, sys::cna_effect_pass_apply_fn),
            effect_pass_collection_destroy: symbol!(cna_effect_pass_collection_destroy,
                sys::cna_effect_pass_collection_destroy_fn
            ),
            effect_pass_collection_add_create: symbol!(cna_effect_pass_collection_add_create,
                sys::cna_effect_pass_collection_add_create_fn
            ),
            effect_pass_collection_get_count: symbol!(cna_effect_pass_collection_get_count,
                sys::cna_effect_pass_collection_get_count_fn
            ),
            effect_pass_collection_get_at: symbol!(cna_effect_pass_collection_get_at,
                sys::cna_effect_pass_collection_get_at_fn
            ),
            effect_pass_collection_find: symbol!(cna_effect_pass_collection_find,
                sys::cna_effect_pass_collection_find_fn
            ),
            effect_technique_destroy: symbol!(cna_effect_technique_destroy,
                sys::cna_effect_technique_destroy_fn
            ),
            effect_technique_get_name_byte_count: symbol!(cna_effect_technique_get_name_byte_count,
                sys::cna_effect_technique_get_name_byte_count_fn
            ),
            effect_technique_copy_name: symbol!(cna_effect_technique_copy_name,
                sys::cna_effect_technique_copy_name_fn
            ),
            effect_technique_get_passes: symbol!(cna_effect_technique_get_passes,
                sys::cna_effect_technique_get_passes_fn
            ),
            effect_technique_get_annotations: symbol!(cna_effect_technique_get_annotations,
                sys::cna_effect_technique_get_annotations_fn
            ),
            effect_technique_collection_destroy: symbol!(cna_effect_technique_collection_destroy,
                sys::cna_effect_technique_collection_destroy_fn
            ),
            effect_technique_collection_add_named: symbol!(cna_effect_technique_collection_add_named,
                sys::cna_effect_technique_collection_add_named_fn
            ),
            effect_technique_collection_get_count: symbol!(cna_effect_technique_collection_get_count,
                sys::cna_effect_technique_collection_get_count_fn
            ),
            effect_technique_collection_get_at: symbol!(cna_effect_technique_collection_get_at,
                sys::cna_effect_technique_collection_get_at_fn
            ),
            effect_technique_collection_find: symbol!(cna_effect_technique_collection_find,
                sys::cna_effect_technique_collection_find_fn
            ),
            keyboard_get_state: symbol!(cna_keyboard_get_state, sys::cna_keyboard_get_state_fn),
            keyboard_get_state_for_player: symbol!(cna_keyboard_get_state_for_player,
                sys::cna_keyboard_get_state_for_player_fn
            ),
            mouse_get_state: symbol!(cna_mouse_get_state, sys::cna_mouse_get_state_fn),
            mouse_get_window_handle: symbol!(cna_mouse_get_window_handle,
                sys::cna_mouse_get_window_handle_fn
            ),
            mouse_set_window_handle: symbol!(cna_mouse_set_window_handle,
                sys::cna_mouse_set_window_handle_fn
            ),
            mouse_set_position: symbol!(cna_mouse_set_position, sys::cna_mouse_set_position_fn),
            gamepad_get_state: symbol!(cna_gamepad_get_state, sys::cna_gamepad_get_state_fn),
            gamepad_get_state_with_dead_zone: symbol!(cna_gamepad_get_state_with_dead_zone,
                sys::cna_gamepad_get_state_with_dead_zone_fn
            ),
            gamepad_get_capabilities: symbol!(cna_gamepad_get_capabilities,
                sys::cna_gamepad_get_capabilities_fn
            ),
            gamepad_set_vibration: symbol!(cna_gamepad_set_vibration,
                sys::cna_gamepad_set_vibration_fn
            ),
            touch_get_capabilities: symbol!(cna_touch_get_capabilities,
                sys::cna_touch_get_capabilities_fn
            ),
            touch_get_state: symbol!(cna_touch_get_state, sys::cna_touch_get_state_fn),
            touch_panel_get_display_width: symbol!(cna_touch_panel_get_display_width,
                sys::cna_touch_panel_get_display_width_fn
            ),
            touch_panel_set_display_width: symbol!(cna_touch_panel_set_display_width,
                sys::cna_touch_panel_set_display_width_fn
            ),
            touch_panel_get_display_height: symbol!(cna_touch_panel_get_display_height,
                sys::cna_touch_panel_get_display_height_fn
            ),
            touch_panel_set_display_height: symbol!(cna_touch_panel_set_display_height,
                sys::cna_touch_panel_set_display_height_fn
            ),
            touch_panel_get_display_orientation: symbol!(cna_touch_panel_get_display_orientation,
                sys::cna_touch_panel_get_display_orientation_fn
            ),
            touch_panel_set_display_orientation: symbol!(cna_touch_panel_set_display_orientation,
                sys::cna_touch_panel_set_display_orientation_fn
            ),
            touch_panel_get_enabled_gestures: symbol!(cna_touch_panel_get_enabled_gestures,
                sys::cna_touch_panel_get_enabled_gestures_fn
            ),
            touch_panel_set_enabled_gestures: symbol!(cna_touch_panel_set_enabled_gestures,
                sys::cna_touch_panel_set_enabled_gestures_fn
            ),
            touch_panel_get_is_gesture_available: symbol!(cna_touch_panel_get_is_gesture_available,
                sys::cna_touch_panel_get_is_gesture_available_fn
            ),
            touch_panel_get_window_handle: symbol!(cna_touch_panel_get_window_handle,
                sys::cna_touch_panel_get_window_handle_fn
            ),
            touch_panel_set_window_handle: symbol!(cna_touch_panel_set_window_handle,
                sys::cna_touch_panel_set_window_handle_fn
            ),
            touch_panel_read_gesture: symbol!(cna_touch_panel_read_gesture,
                sys::cna_touch_panel_read_gesture_fn
            ),
            graphics_device_manager_create: symbol!(cna_graphics_device_manager_create,
                sys::cna_graphics_device_manager_create_fn
            ),
            graphics_device_manager_get_graphics_profile: symbol!(cna_graphics_device_manager_get_graphics_profile,
                sys::cna_graphics_device_manager_get_graphics_profile_fn
            ),
            graphics_device_manager_set_graphics_profile: symbol!(cna_graphics_device_manager_set_graphics_profile,
                sys::cna_graphics_device_manager_set_graphics_profile_fn
            ),
            graphics_device_manager_get_is_full_screen: symbol!(cna_graphics_device_manager_get_is_full_screen,
                sys::cna_graphics_device_manager_get_is_full_screen_fn
            ),
            graphics_device_manager_set_is_full_screen: symbol!(cna_graphics_device_manager_set_is_full_screen,
                sys::cna_graphics_device_manager_set_is_full_screen_fn
            ),
            graphics_device_manager_get_prefer_multi_sampling: symbol!(cna_graphics_device_manager_get_prefer_multi_sampling,
                sys::cna_graphics_device_manager_get_prefer_multi_sampling_fn
            ),
            graphics_device_manager_set_prefer_multi_sampling: symbol!(cna_graphics_device_manager_set_prefer_multi_sampling,
                sys::cna_graphics_device_manager_set_prefer_multi_sampling_fn
            ),
            graphics_device_manager_get_preferred_back_buffer_format: symbol!(cna_graphics_device_manager_get_preferred_back_buffer_format,
                sys::cna_graphics_device_manager_get_preferred_back_buffer_format_fn
            ),
            graphics_device_manager_set_preferred_back_buffer_format: symbol!(cna_graphics_device_manager_set_preferred_back_buffer_format,
                sys::cna_graphics_device_manager_set_preferred_back_buffer_format_fn
            ),
            graphics_device_manager_get_preferred_back_buffer_width: symbol!(cna_graphics_device_manager_get_preferred_back_buffer_width,
                sys::cna_graphics_device_manager_get_preferred_back_buffer_width_fn
            ),
            graphics_device_manager_set_preferred_back_buffer_width: symbol!(cna_graphics_device_manager_set_preferred_back_buffer_width,
                sys::cna_graphics_device_manager_set_preferred_back_buffer_width_fn
            ),
            graphics_device_manager_get_preferred_back_buffer_height: symbol!(cna_graphics_device_manager_get_preferred_back_buffer_height,
                sys::cna_graphics_device_manager_get_preferred_back_buffer_height_fn
            ),
            graphics_device_manager_set_preferred_back_buffer_height: symbol!(cna_graphics_device_manager_set_preferred_back_buffer_height,
                sys::cna_graphics_device_manager_set_preferred_back_buffer_height_fn
            ),
            graphics_device_manager_get_preferred_depth_stencil_format: symbol!(cna_graphics_device_manager_get_preferred_depth_stencil_format,
                sys::cna_graphics_device_manager_get_preferred_depth_stencil_format_fn
            ),
            graphics_device_manager_set_preferred_depth_stencil_format: symbol!(cna_graphics_device_manager_set_preferred_depth_stencil_format,
                sys::cna_graphics_device_manager_set_preferred_depth_stencil_format_fn
            ),
            graphics_device_manager_get_synchronize_with_vertical_retrace: symbol!(cna_graphics_device_manager_get_synchronize_with_vertical_retrace,
                sys::cna_graphics_device_manager_get_synchronize_with_vertical_retrace_fn
            ),
            graphics_device_manager_set_synchronize_with_vertical_retrace: symbol!(cna_graphics_device_manager_set_synchronize_with_vertical_retrace,
                sys::cna_graphics_device_manager_set_synchronize_with_vertical_retrace_fn
            ),
            graphics_device_manager_get_supported_orientations: symbol!(cna_graphics_device_manager_get_supported_orientations,
                sys::cna_graphics_device_manager_get_supported_orientations_fn
            ),
            graphics_device_manager_set_supported_orientations: symbol!(cna_graphics_device_manager_set_supported_orientations,
                sys::cna_graphics_device_manager_set_supported_orientations_fn
            ),
            graphics_device_manager_apply_changes: symbol!(cna_graphics_device_manager_apply_changes,
                sys::cna_graphics_device_manager_apply_changes_fn
            ),
            graphics_device_manager_toggle_full_screen: symbol!(cna_graphics_device_manager_toggle_full_screen,
                sys::cna_graphics_device_manager_toggle_full_screen_fn
            ),
            graphics_device_manager_create_device: symbol!(cna_graphics_device_manager_create_device,
                sys::cna_graphics_device_manager_create_device_fn
            ),
            graphics_device_manager_begin_draw: symbol!(cna_graphics_device_manager_begin_draw,
                sys::cna_graphics_device_manager_begin_draw_fn
            ),
            graphics_device_manager_end_draw: symbol!(cna_graphics_device_manager_end_draw,
                sys::cna_graphics_device_manager_end_draw_fn
            ),
            graphics_device_manager_dispose: symbol!(cna_graphics_device_manager_dispose,
                sys::cna_graphics_device_manager_dispose_fn
            ),
            graphics_device_manager_subscribe: symbol!(cna_graphics_device_manager_subscribe,
                sys::cna_graphics_device_manager_subscribe_fn
            ),
            graphics_device_manager_subscribe_preparing_device_settings_ext: symbol!(cna_graphics_device_manager_subscribe_preparing_device_settings_ext,
                sys::cna_graphics_device_manager_subscribe_preparing_device_settings_ext_fn
            ),
            graphics_device_manager_destroy: symbol!(cna_graphics_device_manager_destroy,
                sys::cna_graphics_device_manager_destroy_fn
            ),
            storage_device_show_selector: symbol!(cna_storage_device_show_selector,
                sys::cna_storage_device_show_selector_fn
            ),
            storage_device_show_selector_for_player: symbol!(cna_storage_device_show_selector_for_player,
                sys::cna_storage_device_show_selector_for_player_fn
            ),
            storage_device_show_selector_with_space: symbol!(cna_storage_device_show_selector_with_space,
                sys::cna_storage_device_show_selector_with_space_fn
            ),
            storage_device_show_selector_for_player_with_space: symbol!(cna_storage_device_show_selector_for_player_with_space,
                sys::cna_storage_device_show_selector_for_player_with_space_fn
            ),
            storage_device_get_free_space: symbol!(cna_storage_device_get_free_space,
                sys::cna_storage_device_get_free_space_fn
            ),
            storage_device_get_is_connected: symbol!(cna_storage_device_get_is_connected,
                sys::cna_storage_device_get_is_connected_fn
            ),
            storage_device_get_total_space: symbol!(cna_storage_device_get_total_space,
                sys::cna_storage_device_get_total_space_fn
            ),
            storage_device_delete_container: symbol!(cna_storage_device_delete_container,
                sys::cna_storage_device_delete_container_fn
            ),
            storage_device_subscribe_device_changed: symbol!(cna_storage_device_subscribe_device_changed,
                sys::cna_storage_device_subscribe_device_changed_fn
            ),
            storage_device_unsubscribe_device_changed: symbol!(cna_storage_device_unsubscribe_device_changed,
                sys::cna_storage_device_unsubscribe_device_changed_fn
            ),
            storage_device_destroy: symbol!(cna_storage_device_destroy,
                sys::cna_storage_device_destroy_fn
            ),
            storage_container_open: symbol!(cna_storage_container_open,
                sys::cna_storage_container_open_fn
            ),
            storage_container_get_display_name_size: symbol!(cna_storage_container_get_display_name_size,
                sys::cna_storage_container_get_display_name_size_fn
            ),
            storage_container_copy_display_name: symbol!(cna_storage_container_copy_display_name,
                sys::cna_storage_container_copy_display_name_fn
            ),
            storage_container_dispose: symbol!(cna_storage_container_dispose,
                sys::cna_storage_container_dispose_fn
            ),
            storage_container_subscribe_disposing: symbol!(cna_storage_container_subscribe_disposing,
                sys::cna_storage_container_subscribe_disposing_fn
            ),
            storage_container_unsubscribe_disposing: symbol!(cna_storage_container_unsubscribe_disposing,
                sys::cna_storage_container_unsubscribe_disposing_fn
            ),
            storage_container_create_directory: symbol!(cna_storage_container_create_directory,
                sys::cna_storage_container_create_directory_fn
            ),
            storage_container_directory_exists: symbol!(cna_storage_container_directory_exists,
                sys::cna_storage_container_directory_exists_fn
            ),
            storage_container_delete_directory: symbol!(cna_storage_container_delete_directory,
                sys::cna_storage_container_delete_directory_fn
            ),
            storage_container_file_exists: symbol!(cna_storage_container_file_exists,
                sys::cna_storage_container_file_exists_fn
            ),
            storage_container_delete_file: symbol!(cna_storage_container_delete_file,
                sys::cna_storage_container_delete_file_fn
            ),
            storage_container_get_directory_name_count: symbol!(cna_storage_container_get_directory_name_count,
                sys::cna_storage_container_get_directory_name_count_fn
            ),
            storage_container_copy_directory_name: symbol!(cna_storage_container_copy_directory_name,
                sys::cna_storage_container_copy_directory_name_fn
            ),
            storage_container_get_file_name_count: symbol!(cna_storage_container_get_file_name_count,
                sys::cna_storage_container_get_file_name_count_fn
            ),
            storage_container_copy_file_name: symbol!(cna_storage_container_copy_file_name,
                sys::cna_storage_container_copy_file_name_fn
            ),
            storage_container_create_file: symbol!(cna_storage_container_create_file,
                sys::cna_storage_container_create_file_fn
            ),
            storage_container_open_file: symbol!(cna_storage_container_open_file,
                sys::cna_storage_container_open_file_fn
            ),
            storage_container_open_file_access: symbol!(cna_storage_container_open_file_access,
                sys::cna_storage_container_open_file_access_fn
            ),
            storage_container_open_file_share: symbol!(cna_storage_container_open_file_share,
                sys::cna_storage_container_open_file_share_fn
            ),
            storage_container_destroy: symbol!(cna_storage_container_destroy,
                sys::cna_storage_container_destroy_fn
            ),
            storage_stream_read: symbol!(cna_storage_stream_read,
                sys::cna_storage_stream_read_fn
            ),
            storage_stream_write: symbol!(cna_storage_stream_write,
                sys::cna_storage_stream_write_fn
            ),
            storage_stream_seek: symbol!(cna_storage_stream_seek,
                sys::cna_storage_stream_seek_fn
            ),
            storage_stream_get_position: symbol!(cna_storage_stream_get_position,
                sys::cna_storage_stream_get_position_fn
            ),
            storage_stream_get_length: symbol!(cna_storage_stream_get_length,
                sys::cna_storage_stream_get_length_fn
            ),
            storage_stream_set_length: symbol!(cna_storage_stream_set_length,
                sys::cna_storage_stream_set_length_fn
            ),
            storage_stream_get_can_read: symbol!(cna_storage_stream_get_can_read,
                sys::cna_storage_stream_get_can_read_fn
            ),
            storage_stream_get_can_write: symbol!(cna_storage_stream_get_can_write,
                sys::cna_storage_stream_get_can_write_fn
            ),
            storage_stream_get_can_seek: symbol!(cna_storage_stream_get_can_seek,
                sys::cna_storage_stream_get_can_seek_fn
            ),
            storage_stream_flush: symbol!(cna_storage_stream_flush,
                sys::cna_storage_stream_flush_fn
            ),
            storage_stream_close: symbol!(cna_storage_stream_close,
                sys::cna_storage_stream_close_fn
            ),
            audio: AudioApi::load(&source)?,
            media: MediaApi::load(&source)?,
            _source: source,
        })
    }
}
