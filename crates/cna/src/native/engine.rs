//! Audited engine-layer calls over the canonical CNA ABI table.
//!
//! CNA's engine layer is a build-time choice, so every route here exists in the
//! exported ABI whether or not the layer was compiled in: a library without it
//! answers `NOT_SUPPORTED` rather than failing to resolve. Resolving the whole
//! table at load is still what this module does, because a *missing* symbol is
//! a different fact from a present one that refuses, and only the first should
//! stop the library from loading at all.

use cna_sys as sys;

use crate::error::Result;

use super::loader::NativeSource;

/// Every reviewed engine-layer route, resolved once when the tables are filled.
#[derive(Debug)]
pub(crate) struct EngineApi {
    pub(crate) render_pipeline_create: sys::cna_render_pipeline_create_fn,
    pub(crate) render_pipeline_destroy: sys::cna_render_pipeline_destroy_fn,
    pub(crate) render_pipeline_get_settings: sys::cna_render_pipeline_get_settings_fn,
    pub(crate) render_pipeline_set_settings: sys::cna_render_pipeline_set_settings_fn,
    pub(crate) render_pipeline_resize: sys::cna_render_pipeline_resize_fn,
    pub(crate) render_pipeline_begin: sys::cna_render_pipeline_begin_fn,
    pub(crate) render_pipeline_end: sys::cna_render_pipeline_end_fn,
    pub(crate) render_pipeline_set_depth_normal_inputs:
        sys::cna_render_pipeline_set_depth_normal_inputs_fn,
    pub(crate) render_pipeline_set_velocity_input_ext:
        sys::cna_render_pipeline_set_velocity_input_ext_fn,
    pub(crate) render_pipeline_set_transparent_scene:
        sys::cna_render_pipeline_set_transparent_scene_fn,
    pub(crate) render_pipeline_set_camera: sys::cna_render_pipeline_set_camera_fn,
    pub(crate) render_pipeline_set_skybox_camera: sys::cna_render_pipeline_set_skybox_camera_fn,
    pub(crate) render_pipeline_copy_transparency_fallback_reason_ext:
        sys::cna_render_pipeline_copy_transparency_fallback_reason_ext_fn,
    pub(crate) render_pipeline_set_gpu_timing_enabled_ext:
        sys::cna_render_pipeline_set_gpu_timing_enabled_ext_fn,
    pub(crate) render_pipeline_is_gpu_timing_enabled_ext:
        sys::cna_render_pipeline_is_gpu_timing_enabled_ext_fn,
    pub(crate) render_pipeline_did_skybox_draw: sys::cna_render_pipeline_did_skybox_draw_fn,
    pub(crate) render_pipeline_did_shadow_pass_run:
        sys::cna_render_pipeline_did_shadow_pass_run_fn,
    pub(crate) render_pipeline_get_scene_target: sys::cna_render_pipeline_get_scene_target_fn,
    pub(crate) render_pipeline_get_scene_target_format:
        sys::cna_render_pipeline_get_scene_target_format_fn,
    pub(crate) render_pipeline_is_using_scene_target:
        sys::cna_render_pipeline_is_using_scene_target_fn,
    pub(crate) render_pipeline_get_last_frame_pass_count:
        sys::cna_render_pipeline_get_last_frame_pass_count_fn,
    pub(crate) render_pipeline_get_gpu_memory_estimate_bytes:
        sys::cna_render_pipeline_get_gpu_memory_estimate_bytes_fn,
    pub(crate) render_pipeline_get_statistics: sys::cna_render_pipeline_get_statistics_fn,
    pub(crate) render_pipeline_release_device_resources_ext:
        sys::cna_render_pipeline_release_device_resources_ext_fn,
    pub(crate) render_pipeline_get_pass_timing_count_ext:
        sys::cna_render_pipeline_get_pass_timing_count_ext_fn,
    pub(crate) render_pipeline_get_pass_timing_ext:
        sys::cna_render_pipeline_get_pass_timing_ext_fn,
    pub(crate) render_pipeline_copy_pass_timing_name_ext:
        sys::cna_render_pipeline_copy_pass_timing_name_ext_fn,
    pub(crate) render_pipeline_set_shadow_scene: sys::cna_render_pipeline_set_shadow_scene_fn,
    pub(crate) render_pipeline_get_shadow_map: sys::cna_render_pipeline_get_shadow_map_fn,
    pub(crate) directional_light_ext_init: sys::cna_directional_light_ext_init_fn,
    pub(crate) graphics_device_supports_shadow_sampling_ext:
        sys::cna_graphics_device_supports_shadow_sampling_ext_fn,
    pub(crate) shadow_map_create: sys::cna_shadow_map_create_fn,
    pub(crate) shadow_map_destroy: sys::cna_shadow_map_destroy_fn,
    pub(crate) shadow_map_is_supported: sys::cna_shadow_map_is_supported_fn,
    pub(crate) shadow_map_begin: sys::cna_shadow_map_begin_fn,
    pub(crate) shadow_map_end: sys::cna_shadow_map_end_fn,
    pub(crate) shadow_map_get_caster_effect: sys::cna_shadow_map_get_caster_effect_fn,
    pub(crate) shadow_map_get_skinned_caster_effect:
        sys::cna_shadow_map_get_skinned_caster_effect_fn,
    pub(crate) shadow_map_apply_caster: sys::cna_shadow_map_apply_caster_fn,
    pub(crate) shadow_map_apply_skinned_caster: sys::cna_shadow_map_apply_skinned_caster_fn,
    pub(crate) shadow_map_get_shadow_texture: sys::cna_shadow_map_get_shadow_texture_fn,
    pub(crate) shadow_map_get_light_view_projection:
        sys::cna_shadow_map_get_light_view_projection_fn,
    pub(crate) shadow_map_get_size: sys::cna_shadow_map_get_size_fn,
    pub(crate) shadow_map_get_quality: sys::cna_shadow_map_get_quality_fn,
    pub(crate) shadow_map_get_depth_bias: sys::cna_shadow_map_get_depth_bias_fn,
    pub(crate) shadow_map_set_depth_bias: sys::cna_shadow_map_set_depth_bias_fn,
    pub(crate) shadow_map_get_filter_radius: sys::cna_shadow_map_get_filter_radius_fn,
    pub(crate) shadow_map_compute_light_view: sys::cna_shadow_map_compute_light_view_fn,
    pub(crate) shadow_map_compute_light_projection:
        sys::cna_shadow_map_compute_light_projection_fn,
    pub(crate) shadow_map_size_for_quality: sys::cna_shadow_map_size_for_quality_fn,
    pub(crate) shadow_map_filter_radius_for_quality:
        sys::cna_shadow_map_filter_radius_for_quality_fn,
    pub(crate) post_process_context_init: sys::cna_post_process_context_init_fn,
    pub(crate) blit_pass_create: sys::cna_blit_pass_create_fn,
    pub(crate) post_process_effect_pass_create: sys::cna_post_process_effect_pass_create_fn,
    pub(crate) post_process_effect_pass_create_owning: sys::cna_post_process_effect_pass_create_owning_fn,
    pub(crate) post_process_effect_pass_get_effect: sys::cna_post_process_effect_pass_get_effect_fn,
    pub(crate) post_process_effect_pass_set_effect: sys::cna_post_process_effect_pass_set_effect_fn,
    pub(crate) post_process_pass_apply: sys::cna_post_process_pass_apply_fn,
    pub(crate) post_process_pass_copy_name: sys::cna_post_process_pass_copy_name_fn,
    pub(crate) post_process_pass_is_supported: sys::cna_post_process_pass_is_supported_fn,
    pub(crate) post_process_pass_destroy: sys::cna_post_process_pass_destroy_fn,
    pub(crate) post_process_chain_create: sys::cna_post_process_chain_create_fn,
    pub(crate) post_process_chain_destroy: sys::cna_post_process_chain_destroy_fn,
    pub(crate) post_process_chain_add_pass: sys::cna_post_process_chain_add_pass_fn,
    pub(crate) post_process_chain_add_owned_pass: sys::cna_post_process_chain_add_owned_pass_fn,
    pub(crate) post_process_chain_clear: sys::cna_post_process_chain_clear_fn,
    pub(crate) post_process_chain_get_pass_count: sys::cna_post_process_chain_get_pass_count_fn,
    pub(crate) post_process_chain_apply: sys::cna_post_process_chain_apply_fn,
    pub(crate) post_process_chain_reset_targets: sys::cna_post_process_chain_reset_targets_fn,
    pub(crate) post_process_chain_get_target_pool: sys::cna_post_process_chain_get_target_pool_fn,
    pub(crate) post_process_chain_is_gpu_timing_enabled: sys::cna_post_process_chain_is_gpu_timing_enabled_fn,
    pub(crate) post_process_chain_set_gpu_timing_enabled: sys::cna_post_process_chain_set_gpu_timing_enabled_fn,
    pub(crate) post_process_chain_get_pass_timing_count: sys::cna_post_process_chain_get_pass_timing_count_fn,
    pub(crate) post_process_chain_get_pass_timing: sys::cna_post_process_chain_get_pass_timing_fn,
    pub(crate) post_process_chain_copy_pass_timing_name: sys::cna_post_process_chain_copy_pass_timing_name_fn,
    pub(crate) render_pipeline_add_user_pass: sys::cna_render_pipeline_add_user_pass_fn,
    pub(crate) render_pipeline_clear_user_passes: sys::cna_render_pipeline_clear_user_passes_fn,
    pub(crate) render_target_pool_create: sys::cna_render_target_pool_create_fn,
    pub(crate) render_target_pool_destroy: sys::cna_render_target_pool_destroy_fn,
    pub(crate) render_target_pool_acquire: sys::cna_render_target_pool_acquire_fn,
    pub(crate) render_target_pool_reset: sys::cna_render_target_pool_reset_fn,
    pub(crate) render_target_pool_get_target_count: sys::cna_render_target_pool_get_target_count_fn,
    pub(crate) render_target_pool_get_estimated_bytes: sys::cna_render_target_pool_get_estimated_bytes_fn,
    pub(crate) tonemap_pass_create: sys::cna_tonemap_pass_create_fn,
    pub(crate) tonemap_pass_get_mode: sys::cna_tonemap_pass_get_mode_fn,
    pub(crate) tonemap_pass_set_mode: sys::cna_tonemap_pass_set_mode_fn,
    pub(crate) tonemap_pass_get_exposure: sys::cna_tonemap_pass_get_exposure_fn,
    pub(crate) tonemap_pass_set_exposure: sys::cna_tonemap_pass_set_exposure_fn,
    pub(crate) tonemap_pass_get_gamma: sys::cna_tonemap_pass_get_gamma_fn,
    pub(crate) tonemap_pass_set_gamma: sys::cna_tonemap_pass_set_gamma_fn,
    pub(crate) tonemap_pass_is_deband_enabled: sys::cna_tonemap_pass_is_deband_enabled_fn,
    pub(crate) tonemap_pass_set_deband_enabled: sys::cna_tonemap_pass_set_deband_enabled_fn,
    pub(crate) tonemap_pass_get_deband_strength: sys::cna_tonemap_pass_get_deband_strength_fn,
    pub(crate) tonemap_pass_set_deband_strength: sys::cna_tonemap_pass_set_deband_strength_fn,
    pub(crate) tonemap_pass_tonemap_channel: sys::cna_tonemap_pass_tonemap_channel_fn,
    pub(crate) fxaa_pass_create: sys::cna_fxaa_pass_create_fn,
    pub(crate) fxaa_pass_get_edge_threshold: sys::cna_fxaa_pass_get_edge_threshold_fn,
    pub(crate) fxaa_pass_set_edge_threshold: sys::cna_fxaa_pass_set_edge_threshold_fn,
    pub(crate) fxaa_pass_edge_threshold_for_quality: sys::cna_fxaa_pass_edge_threshold_for_quality_fn,
    pub(crate) fxaa_pass_copy_fragment_glsl: sys::cna_fxaa_pass_copy_fragment_glsl_fn,
}

impl EngineApi {
    pub(super) fn load(source: &NativeSource) -> Result<Self> {
        macro_rules! symbol {
            ($name:ident, $ty:ty) => {
                super::loader::acquire!(source, $name, $ty)
            };
        }
        Ok(Self {
            render_pipeline_create: symbol!(cna_render_pipeline_create, _),
            render_pipeline_destroy: symbol!(cna_render_pipeline_destroy, _),
            render_pipeline_get_settings: symbol!(cna_render_pipeline_get_settings, _),
            render_pipeline_set_settings: symbol!(cna_render_pipeline_set_settings, _),
            render_pipeline_resize: symbol!(cna_render_pipeline_resize, _),
            render_pipeline_begin: symbol!(cna_render_pipeline_begin, _),
            render_pipeline_end: symbol!(cna_render_pipeline_end, _),
            render_pipeline_set_depth_normal_inputs: symbol!(
                cna_render_pipeline_set_depth_normal_inputs, _
            ),
            render_pipeline_set_velocity_input_ext: symbol!(
                cna_render_pipeline_set_velocity_input_ext, _
            ),
            render_pipeline_set_transparent_scene: symbol!(
                cna_render_pipeline_set_transparent_scene, _
            ),
            render_pipeline_set_camera: symbol!(cna_render_pipeline_set_camera, _),
            render_pipeline_set_skybox_camera: symbol!(cna_render_pipeline_set_skybox_camera, _),
            render_pipeline_copy_transparency_fallback_reason_ext: symbol!(
                cna_render_pipeline_copy_transparency_fallback_reason_ext, _
            ),
            render_pipeline_set_gpu_timing_enabled_ext: symbol!(
                cna_render_pipeline_set_gpu_timing_enabled_ext, _
            ),
            render_pipeline_is_gpu_timing_enabled_ext: symbol!(
                cna_render_pipeline_is_gpu_timing_enabled_ext, _
            ),
            render_pipeline_did_skybox_draw: symbol!(cna_render_pipeline_did_skybox_draw, _),
            render_pipeline_did_shadow_pass_run: symbol!(
                cna_render_pipeline_did_shadow_pass_run, _
            ),
            render_pipeline_get_scene_target: symbol!(cna_render_pipeline_get_scene_target, _),
            render_pipeline_get_scene_target_format: symbol!(
                cna_render_pipeline_get_scene_target_format, _
            ),
            render_pipeline_is_using_scene_target: symbol!(
                cna_render_pipeline_is_using_scene_target, _
            ),
            render_pipeline_get_last_frame_pass_count: symbol!(
                cna_render_pipeline_get_last_frame_pass_count, _
            ),
            render_pipeline_get_gpu_memory_estimate_bytes: symbol!(
                cna_render_pipeline_get_gpu_memory_estimate_bytes, _
            ),
            render_pipeline_get_statistics: symbol!(cna_render_pipeline_get_statistics, _),
            render_pipeline_release_device_resources_ext: symbol!(
                cna_render_pipeline_release_device_resources_ext, _
            ),
            render_pipeline_get_pass_timing_count_ext: symbol!(
                cna_render_pipeline_get_pass_timing_count_ext, _
            ),
            render_pipeline_get_pass_timing_ext: symbol!(
                cna_render_pipeline_get_pass_timing_ext, _
            ),
            render_pipeline_copy_pass_timing_name_ext: symbol!(
                cna_render_pipeline_copy_pass_timing_name_ext, _
            ),
            render_pipeline_set_shadow_scene: symbol!(cna_render_pipeline_set_shadow_scene, _),
            render_pipeline_get_shadow_map: symbol!(cna_render_pipeline_get_shadow_map, _),
            directional_light_ext_init: symbol!(cna_directional_light_ext_init, _),
            graphics_device_supports_shadow_sampling_ext: symbol!(
                cna_graphics_device_supports_shadow_sampling_ext, _
            ),
            shadow_map_create: symbol!(cna_shadow_map_create, _),
            shadow_map_destroy: symbol!(cna_shadow_map_destroy, _),
            shadow_map_is_supported: symbol!(cna_shadow_map_is_supported, _),
            shadow_map_begin: symbol!(cna_shadow_map_begin, _),
            shadow_map_end: symbol!(cna_shadow_map_end, _),
            shadow_map_get_caster_effect: symbol!(cna_shadow_map_get_caster_effect, _),
            shadow_map_get_skinned_caster_effect: symbol!(
                cna_shadow_map_get_skinned_caster_effect, _
            ),
            shadow_map_apply_caster: symbol!(cna_shadow_map_apply_caster, _),
            shadow_map_apply_skinned_caster: symbol!(cna_shadow_map_apply_skinned_caster, _),
            shadow_map_get_shadow_texture: symbol!(cna_shadow_map_get_shadow_texture, _),
            shadow_map_get_light_view_projection: symbol!(
                cna_shadow_map_get_light_view_projection, _
            ),
            shadow_map_get_size: symbol!(cna_shadow_map_get_size, _),
            shadow_map_get_quality: symbol!(cna_shadow_map_get_quality, _),
            shadow_map_get_depth_bias: symbol!(cna_shadow_map_get_depth_bias, _),
            shadow_map_set_depth_bias: symbol!(cna_shadow_map_set_depth_bias, _),
            shadow_map_get_filter_radius: symbol!(cna_shadow_map_get_filter_radius, _),
            shadow_map_compute_light_view: symbol!(cna_shadow_map_compute_light_view, _),
            shadow_map_compute_light_projection: symbol!(
                cna_shadow_map_compute_light_projection, _
            ),
            shadow_map_size_for_quality: symbol!(cna_shadow_map_size_for_quality, _),
            shadow_map_filter_radius_for_quality: symbol!(
                cna_shadow_map_filter_radius_for_quality, _
            ),
            post_process_context_init: symbol!(cna_post_process_context_init, _),
            blit_pass_create: symbol!(cna_blit_pass_create, _),
            post_process_effect_pass_create: symbol!(cna_post_process_effect_pass_create, _),
            post_process_effect_pass_create_owning: symbol!(cna_post_process_effect_pass_create_owning, _),
            post_process_effect_pass_get_effect: symbol!(cna_post_process_effect_pass_get_effect, _),
            post_process_effect_pass_set_effect: symbol!(cna_post_process_effect_pass_set_effect, _),
            post_process_pass_apply: symbol!(cna_post_process_pass_apply, _),
            post_process_pass_copy_name: symbol!(cna_post_process_pass_copy_name, _),
            post_process_pass_is_supported: symbol!(cna_post_process_pass_is_supported, _),
            post_process_pass_destroy: symbol!(cna_post_process_pass_destroy, _),
            post_process_chain_create: symbol!(cna_post_process_chain_create, _),
            post_process_chain_destroy: symbol!(cna_post_process_chain_destroy, _),
            post_process_chain_add_pass: symbol!(cna_post_process_chain_add_pass, _),
            post_process_chain_add_owned_pass: symbol!(cna_post_process_chain_add_owned_pass, _),
            post_process_chain_clear: symbol!(cna_post_process_chain_clear, _),
            post_process_chain_get_pass_count: symbol!(cna_post_process_chain_get_pass_count, _),
            post_process_chain_apply: symbol!(cna_post_process_chain_apply, _),
            post_process_chain_reset_targets: symbol!(cna_post_process_chain_reset_targets, _),
            post_process_chain_get_target_pool: symbol!(cna_post_process_chain_get_target_pool, _),
            post_process_chain_is_gpu_timing_enabled: symbol!(cna_post_process_chain_is_gpu_timing_enabled, _),
            post_process_chain_set_gpu_timing_enabled: symbol!(cna_post_process_chain_set_gpu_timing_enabled, _),
            post_process_chain_get_pass_timing_count: symbol!(cna_post_process_chain_get_pass_timing_count, _),
            post_process_chain_get_pass_timing: symbol!(cna_post_process_chain_get_pass_timing, _),
            post_process_chain_copy_pass_timing_name: symbol!(cna_post_process_chain_copy_pass_timing_name, _),
            render_pipeline_add_user_pass: symbol!(cna_render_pipeline_add_user_pass, _),
            render_pipeline_clear_user_passes: symbol!(cna_render_pipeline_clear_user_passes, _),
            render_target_pool_create: symbol!(cna_render_target_pool_create, _),
            render_target_pool_destroy: symbol!(cna_render_target_pool_destroy, _),
            render_target_pool_acquire: symbol!(cna_render_target_pool_acquire, _),
            render_target_pool_reset: symbol!(cna_render_target_pool_reset, _),
            render_target_pool_get_target_count: symbol!(cna_render_target_pool_get_target_count, _),
            render_target_pool_get_estimated_bytes: symbol!(cna_render_target_pool_get_estimated_bytes, _),
            tonemap_pass_create: symbol!(cna_tonemap_pass_create, _),
            tonemap_pass_get_mode: symbol!(cna_tonemap_pass_get_mode, _),
            tonemap_pass_set_mode: symbol!(cna_tonemap_pass_set_mode, _),
            tonemap_pass_get_exposure: symbol!(cna_tonemap_pass_get_exposure, _),
            tonemap_pass_set_exposure: symbol!(cna_tonemap_pass_set_exposure, _),
            tonemap_pass_get_gamma: symbol!(cna_tonemap_pass_get_gamma, _),
            tonemap_pass_set_gamma: symbol!(cna_tonemap_pass_set_gamma, _),
            tonemap_pass_is_deband_enabled: symbol!(cna_tonemap_pass_is_deband_enabled, _),
            tonemap_pass_set_deband_enabled: symbol!(cna_tonemap_pass_set_deband_enabled, _),
            tonemap_pass_get_deband_strength: symbol!(cna_tonemap_pass_get_deband_strength, _),
            tonemap_pass_set_deband_strength: symbol!(cna_tonemap_pass_set_deband_strength, _),
            tonemap_pass_tonemap_channel: symbol!(cna_tonemap_pass_tonemap_channel, _),
            fxaa_pass_create: symbol!(cna_fxaa_pass_create, _),
            fxaa_pass_get_edge_threshold: symbol!(cna_fxaa_pass_get_edge_threshold, _),
            fxaa_pass_set_edge_threshold: symbol!(cna_fxaa_pass_set_edge_threshold, _),
            fxaa_pass_edge_threshold_for_quality: symbol!(cna_fxaa_pass_edge_threshold_for_quality, _),
            fxaa_pass_copy_fragment_glsl: symbol!(cna_fxaa_pass_copy_fragment_glsl, _),
        })
    }
}
