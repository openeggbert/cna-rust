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
        })
    }
}
