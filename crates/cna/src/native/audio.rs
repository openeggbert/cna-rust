//! Audited Audio/XACT calls over the canonical CNA ABI 0.20 table.

use core::ffi::c_void;
use core::mem::size_of;

use cna_sys as sys;

use crate::error::{CnaError, ErrorCategory, Result};

use super::loader::NativeSource;
use super::Native;

#[derive(Debug)]
pub(super) struct AudioApi {
    framework_dispatcher_update: sys::cna_framework_dispatcher_update_fn,
    sound_effect_create_pcm16: sys::cna_sound_effect_create_pcm16_fn,
    sound_effect_create_pcm16_range: sys::cna_sound_effect_create_pcm16_range_ext_fn,
    sound_effect_create_encoded: sys::cna_sound_effect_create_from_encoded_ext_fn,
    sound_effect_duration: sys::cna_sound_effect_get_duration_ticks_fn,
    sound_effect_create_instance: sys::cna_sound_effect_create_instance_fn,
    sound_effect_destroy: sys::cna_sound_effect_destroy_fn,
    sound_effect_name_size: sys::cna_sound_effect_get_name_size_fn,
    sound_effect_copy_name: sys::cna_sound_effect_copy_name_fn,
    sound_effect_set_name: sys::cna_sound_effect_set_name_fn,
    sound_effect_get_master_volume: sys::cna_sound_effect_get_master_volume_fn,
    sound_effect_set_master_volume: sys::cna_sound_effect_set_master_volume_fn,
    sound_effect_get_distance_scale: sys::cna_sound_effect_get_distance_scale_fn,
    sound_effect_set_distance_scale: sys::cna_sound_effect_set_distance_scale_fn,
    sound_effect_get_doppler_scale: sys::cna_sound_effect_get_doppler_scale_fn,
    sound_effect_set_doppler_scale: sys::cna_sound_effect_set_doppler_scale_fn,
    sound_effect_get_speed_of_sound: sys::cna_sound_effect_get_speed_of_sound_fn,
    sound_effect_set_speed_of_sound: sys::cna_sound_effect_set_speed_of_sound_fn,
    sound_effect_play: sys::cna_sound_effect_play_fn,
    sound_effect_play_with_settings: sys::cna_sound_effect_play_with_settings_fn,
    instance_play: sys::cna_sound_effect_instance_play_fn,
    instance_pause: sys::cna_sound_effect_instance_pause_fn,
    instance_resume: sys::cna_sound_effect_instance_resume_fn,
    instance_stop: sys::cna_sound_effect_instance_stop_fn,
    instance_get_info: sys::cna_sound_effect_instance_get_info_fn,
    instance_set_volume: sys::cna_sound_effect_instance_set_volume_fn,
    instance_set_pitch: sys::cna_sound_effect_instance_set_pitch_fn,
    instance_set_pan: sys::cna_sound_effect_instance_set_pan_fn,
    instance_set_is_looped: sys::cna_sound_effect_instance_set_is_looped_fn,
    instance_destroy: sys::cna_sound_effect_instance_destroy_fn,
    instance_apply_3d: sys::cna_sound_effect_instance_apply_3d_fn,
    instance_apply_3d_multi: sys::cna_sound_effect_instance_apply_3d_multi_ext_fn,
    dynamic_create: sys::cna_dynamic_sound_effect_instance_create_fn,
    dynamic_pending_count: sys::cna_dynamic_sound_effect_instance_get_pending_buffer_count_fn,
    dynamic_submit: sys::cna_dynamic_sound_effect_instance_submit_buffer_fn,
    dynamic_duration: sys::cna_dynamic_sound_effect_instance_get_sample_duration_ticks_fn,
    dynamic_size: sys::cna_dynamic_sound_effect_instance_get_sample_size_in_bytes_fn,
    dynamic_subscribe: sys::cna_dynamic_sound_effect_instance_subscribe_buffer_needed_fn,
    engine_subscribe_disposing: sys::cna_audio_engine_subscribe_disposing_ext_fn,
    wave_bank_subscribe_disposing: sys::cna_wave_bank_subscribe_disposing_ext_fn,
    sound_bank_subscribe_disposing: sys::cna_sound_bank_subscribe_disposing_ext_fn,
    cue_subscribe_disposing: sys::cna_cue_subscribe_disposing_ext_fn,
    audio_unsubscribe: sys::cna_audio_unsubscribe_ext_fn,
    microphone_count: sys::cna_microphone_get_count_fn,
    microphone_default: sys::cna_microphone_get_default_index_ext_fn,
    microphone_name_size: sys::cna_microphone_get_name_size_at_fn,
    microphone_copy_name: sys::cna_microphone_copy_name_at_fn,
    microphone_buffer_duration: sys::cna_microphone_get_buffer_duration_ticks_at_fn,
    microphone_set_buffer_duration: sys::cna_microphone_set_buffer_duration_ticks_at_fn,
    microphone_is_headset: sys::cna_microphone_get_is_headset_at_fn,
    microphone_sample_rate: sys::cna_microphone_get_sample_rate_at_fn,
    microphone_state: sys::cna_microphone_get_state_at_fn,
    microphone_start: sys::cna_microphone_start_at_fn,
    microphone_stop: sys::cna_microphone_stop_at_fn,
    microphone_get_data: sys::cna_microphone_get_data_at_fn,
    microphone_duration: sys::cna_microphone_get_sample_duration_ticks_at_fn,
    microphone_size: sys::cna_microphone_get_sample_size_in_bytes_at_fn,
    microphone_subscribe: sys::cna_microphone_subscribe_buffer_ready_at_fn,
    engine_create: sys::cna_audio_engine_create_fn,
    engine_create_with_renderer: sys::cna_audio_engine_create_with_renderer_fn,
    engine_destroy: sys::cna_audio_engine_destroy_fn,
    engine_renderer_count: sys::cna_audio_engine_get_renderer_count_fn,
    renderer_friendly_name_size: sys::cna_audio_engine_get_renderer_friendly_name_size_fn,
    renderer_copy_friendly_name: sys::cna_audio_engine_copy_renderer_friendly_name_fn,
    renderer_id_size: sys::cna_audio_engine_get_renderer_id_size_fn,
    renderer_copy_id: sys::cna_audio_engine_copy_renderer_id_fn,
    engine_get_global: sys::cna_audio_engine_get_global_variable_fn,
    engine_set_global: sys::cna_audio_engine_set_global_variable_fn,
    engine_update: sys::cna_audio_engine_update_fn,
    engine_get_category: sys::cna_audio_engine_get_category_fn,
    category_destroy: sys::cna_audio_category_destroy_fn,
    category_name_size: sys::cna_audio_category_get_name_size_fn,
    category_copy_name: sys::cna_audio_category_copy_name_fn,
    category_pause: sys::cna_audio_category_pause_fn,
    category_resume: sys::cna_audio_category_resume_fn,
    category_set_volume: sys::cna_audio_category_set_volume_fn,
    category_stop: sys::cna_audio_category_stop_fn,
    category_equals: sys::cna_audio_category_equals_fn,
    category_hash: sys::cna_audio_category_get_hash_code_fn,
    wave_bank_create: sys::cna_wave_bank_create_fn,
    wave_bank_create_streaming: sys::cna_wave_bank_create_streaming_fn,
    wave_bank_destroy: sys::cna_wave_bank_destroy_fn,
    wave_bank_is_prepared: sys::cna_wave_bank_get_is_prepared_fn,
    wave_bank_is_in_use: sys::cna_wave_bank_get_is_in_use_fn,
    sound_bank_create: sys::cna_sound_bank_create_fn,
    sound_bank_destroy: sys::cna_sound_bank_destroy_fn,
    sound_bank_is_in_use: sys::cna_sound_bank_get_is_in_use_fn,
    sound_bank_get_cue: sys::cna_sound_bank_get_cue_fn,
    sound_bank_play_cue: sys::cna_sound_bank_play_cue_fn,
    sound_bank_play_cue_3d: sys::cna_sound_bank_play_cue_3d_fn,
    cue_destroy: sys::cna_cue_destroy_fn,
    cue_get_info: sys::cna_cue_get_info_fn,
    cue_name_size: sys::cna_cue_get_name_size_fn,
    cue_copy_name: sys::cna_cue_copy_name_fn,
    cue_apply_3d: sys::cna_cue_apply_3d_fn,
    cue_get_variable: sys::cna_cue_get_variable_fn,
    cue_set_variable: sys::cna_cue_set_variable_fn,
    cue_play: sys::cna_cue_play_fn,
    cue_pause: sys::cna_cue_pause_fn,
    cue_resume: sys::cna_cue_resume_fn,
    cue_stop: sys::cna_cue_stop_fn,
}

impl AudioApi {
    pub(super) fn load(source: &NativeSource) -> Result<Self> {
        macro_rules! symbol {
            ($name:ident, $ty:ty) => {
                super::loader::acquire!(source, $name, $ty)
            };
        }
        Ok(Self {
            framework_dispatcher_update: symbol!(cna_framework_dispatcher_update, sys::cna_framework_dispatcher_update_fn),
            sound_effect_create_pcm16: symbol!(cna_sound_effect_create_pcm16, sys::cna_sound_effect_create_pcm16_fn),
            sound_effect_create_pcm16_range: symbol!(cna_sound_effect_create_pcm16_range_ext, sys::cna_sound_effect_create_pcm16_range_ext_fn),
            sound_effect_create_encoded: symbol!(cna_sound_effect_create_from_encoded_ext, sys::cna_sound_effect_create_from_encoded_ext_fn),
            sound_effect_duration: symbol!(cna_sound_effect_get_duration_ticks, sys::cna_sound_effect_get_duration_ticks_fn),
            sound_effect_create_instance: symbol!(cna_sound_effect_create_instance, sys::cna_sound_effect_create_instance_fn),
            sound_effect_destroy: symbol!(cna_sound_effect_destroy, sys::cna_sound_effect_destroy_fn),
            sound_effect_name_size: symbol!(cna_sound_effect_get_name_size, sys::cna_sound_effect_get_name_size_fn),
            sound_effect_copy_name: symbol!(cna_sound_effect_copy_name, sys::cna_sound_effect_copy_name_fn),
            sound_effect_set_name: symbol!(cna_sound_effect_set_name, sys::cna_sound_effect_set_name_fn),
            sound_effect_get_master_volume: symbol!(cna_sound_effect_get_master_volume, sys::cna_sound_effect_get_master_volume_fn),
            sound_effect_set_master_volume: symbol!(cna_sound_effect_set_master_volume, sys::cna_sound_effect_set_master_volume_fn),
            sound_effect_get_distance_scale: symbol!(cna_sound_effect_get_distance_scale, sys::cna_sound_effect_get_distance_scale_fn),
            sound_effect_set_distance_scale: symbol!(cna_sound_effect_set_distance_scale, sys::cna_sound_effect_set_distance_scale_fn),
            sound_effect_get_doppler_scale: symbol!(cna_sound_effect_get_doppler_scale, sys::cna_sound_effect_get_doppler_scale_fn),
            sound_effect_set_doppler_scale: symbol!(cna_sound_effect_set_doppler_scale, sys::cna_sound_effect_set_doppler_scale_fn),
            sound_effect_get_speed_of_sound: symbol!(cna_sound_effect_get_speed_of_sound, sys::cna_sound_effect_get_speed_of_sound_fn),
            sound_effect_set_speed_of_sound: symbol!(cna_sound_effect_set_speed_of_sound, sys::cna_sound_effect_set_speed_of_sound_fn),
            sound_effect_play: symbol!(cna_sound_effect_play, sys::cna_sound_effect_play_fn),
            sound_effect_play_with_settings: symbol!(cna_sound_effect_play_with_settings, sys::cna_sound_effect_play_with_settings_fn),
            instance_play: symbol!(cna_sound_effect_instance_play, sys::cna_sound_effect_instance_play_fn),
            instance_pause: symbol!(cna_sound_effect_instance_pause, sys::cna_sound_effect_instance_pause_fn),
            instance_resume: symbol!(cna_sound_effect_instance_resume, sys::cna_sound_effect_instance_resume_fn),
            instance_stop: symbol!(cna_sound_effect_instance_stop, sys::cna_sound_effect_instance_stop_fn),
            instance_get_info: symbol!(cna_sound_effect_instance_get_info, sys::cna_sound_effect_instance_get_info_fn),
            instance_set_volume: symbol!(cna_sound_effect_instance_set_volume, sys::cna_sound_effect_instance_set_volume_fn),
            instance_set_pitch: symbol!(cna_sound_effect_instance_set_pitch, sys::cna_sound_effect_instance_set_pitch_fn),
            instance_set_pan: symbol!(cna_sound_effect_instance_set_pan, sys::cna_sound_effect_instance_set_pan_fn),
            instance_set_is_looped: symbol!(cna_sound_effect_instance_set_is_looped, sys::cna_sound_effect_instance_set_is_looped_fn),
            instance_destroy: symbol!(cna_sound_effect_instance_destroy, sys::cna_sound_effect_instance_destroy_fn),
            instance_apply_3d: symbol!(cna_sound_effect_instance_apply_3d, sys::cna_sound_effect_instance_apply_3d_fn),
            instance_apply_3d_multi: symbol!(cna_sound_effect_instance_apply_3d_multi_ext, sys::cna_sound_effect_instance_apply_3d_multi_ext_fn),
            dynamic_create: symbol!(cna_dynamic_sound_effect_instance_create, sys::cna_dynamic_sound_effect_instance_create_fn),
            dynamic_pending_count: symbol!(cna_dynamic_sound_effect_instance_get_pending_buffer_count, sys::cna_dynamic_sound_effect_instance_get_pending_buffer_count_fn),
            dynamic_submit: symbol!(cna_dynamic_sound_effect_instance_submit_buffer, sys::cna_dynamic_sound_effect_instance_submit_buffer_fn),
            dynamic_duration: symbol!(cna_dynamic_sound_effect_instance_get_sample_duration_ticks, sys::cna_dynamic_sound_effect_instance_get_sample_duration_ticks_fn),
            dynamic_size: symbol!(cna_dynamic_sound_effect_instance_get_sample_size_in_bytes, sys::cna_dynamic_sound_effect_instance_get_sample_size_in_bytes_fn),
            dynamic_subscribe: symbol!(cna_dynamic_sound_effect_instance_subscribe_buffer_needed, sys::cna_dynamic_sound_effect_instance_subscribe_buffer_needed_fn),
            engine_subscribe_disposing: symbol!(cna_audio_engine_subscribe_disposing_ext,
                sys::cna_audio_engine_subscribe_disposing_ext_fn),
            wave_bank_subscribe_disposing: symbol!(cna_wave_bank_subscribe_disposing_ext,
                sys::cna_wave_bank_subscribe_disposing_ext_fn),
            sound_bank_subscribe_disposing: symbol!(cna_sound_bank_subscribe_disposing_ext,
                sys::cna_sound_bank_subscribe_disposing_ext_fn),
            cue_subscribe_disposing: symbol!(cna_cue_subscribe_disposing_ext,
                sys::cna_cue_subscribe_disposing_ext_fn),
            audio_unsubscribe: symbol!(cna_audio_unsubscribe_ext, sys::cna_audio_unsubscribe_ext_fn),
            microphone_count: symbol!(cna_microphone_get_count, sys::cna_microphone_get_count_fn),
            microphone_default: symbol!(cna_microphone_get_default_index_ext, sys::cna_microphone_get_default_index_ext_fn),
            microphone_name_size: symbol!(cna_microphone_get_name_size_at, sys::cna_microphone_get_name_size_at_fn),
            microphone_copy_name: symbol!(cna_microphone_copy_name_at, sys::cna_microphone_copy_name_at_fn),
            microphone_buffer_duration: symbol!(cna_microphone_get_buffer_duration_ticks_at, sys::cna_microphone_get_buffer_duration_ticks_at_fn),
            microphone_set_buffer_duration: symbol!(cna_microphone_set_buffer_duration_ticks_at, sys::cna_microphone_set_buffer_duration_ticks_at_fn),
            microphone_is_headset: symbol!(cna_microphone_get_is_headset_at, sys::cna_microphone_get_is_headset_at_fn),
            microphone_sample_rate: symbol!(cna_microphone_get_sample_rate_at, sys::cna_microphone_get_sample_rate_at_fn),
            microphone_state: symbol!(cna_microphone_get_state_at, sys::cna_microphone_get_state_at_fn),
            microphone_start: symbol!(cna_microphone_start_at, sys::cna_microphone_start_at_fn),
            microphone_stop: symbol!(cna_microphone_stop_at, sys::cna_microphone_stop_at_fn),
            microphone_get_data: symbol!(cna_microphone_get_data_at, sys::cna_microphone_get_data_at_fn),
            microphone_duration: symbol!(cna_microphone_get_sample_duration_ticks_at, sys::cna_microphone_get_sample_duration_ticks_at_fn),
            microphone_size: symbol!(cna_microphone_get_sample_size_in_bytes_at, sys::cna_microphone_get_sample_size_in_bytes_at_fn),
            microphone_subscribe: symbol!(cna_microphone_subscribe_buffer_ready_at, sys::cna_microphone_subscribe_buffer_ready_at_fn),
            engine_create: symbol!(cna_audio_engine_create, sys::cna_audio_engine_create_fn),
            engine_create_with_renderer: symbol!(cna_audio_engine_create_with_renderer, sys::cna_audio_engine_create_with_renderer_fn),
            engine_destroy: symbol!(cna_audio_engine_destroy, sys::cna_audio_engine_destroy_fn),
            engine_renderer_count: symbol!(cna_audio_engine_get_renderer_count, sys::cna_audio_engine_get_renderer_count_fn),
            renderer_friendly_name_size: symbol!(cna_audio_engine_get_renderer_friendly_name_size, sys::cna_audio_engine_get_renderer_friendly_name_size_fn),
            renderer_copy_friendly_name: symbol!(cna_audio_engine_copy_renderer_friendly_name, sys::cna_audio_engine_copy_renderer_friendly_name_fn),
            renderer_id_size: symbol!(cna_audio_engine_get_renderer_id_size, sys::cna_audio_engine_get_renderer_id_size_fn),
            renderer_copy_id: symbol!(cna_audio_engine_copy_renderer_id, sys::cna_audio_engine_copy_renderer_id_fn),
            engine_get_global: symbol!(cna_audio_engine_get_global_variable, sys::cna_audio_engine_get_global_variable_fn),
            engine_set_global: symbol!(cna_audio_engine_set_global_variable, sys::cna_audio_engine_set_global_variable_fn),
            engine_update: symbol!(cna_audio_engine_update, sys::cna_audio_engine_update_fn),
            engine_get_category: symbol!(cna_audio_engine_get_category, sys::cna_audio_engine_get_category_fn),
            category_destroy: symbol!(cna_audio_category_destroy, sys::cna_audio_category_destroy_fn),
            category_name_size: symbol!(cna_audio_category_get_name_size, sys::cna_audio_category_get_name_size_fn),
            category_copy_name: symbol!(cna_audio_category_copy_name, sys::cna_audio_category_copy_name_fn),
            category_pause: symbol!(cna_audio_category_pause, sys::cna_audio_category_pause_fn),
            category_resume: symbol!(cna_audio_category_resume, sys::cna_audio_category_resume_fn),
            category_set_volume: symbol!(cna_audio_category_set_volume, sys::cna_audio_category_set_volume_fn),
            category_stop: symbol!(cna_audio_category_stop, sys::cna_audio_category_stop_fn),
            category_equals: symbol!(cna_audio_category_equals, sys::cna_audio_category_equals_fn),
            category_hash: symbol!(cna_audio_category_get_hash_code, sys::cna_audio_category_get_hash_code_fn),
            wave_bank_create: symbol!(cna_wave_bank_create, sys::cna_wave_bank_create_fn),
            wave_bank_create_streaming: symbol!(cna_wave_bank_create_streaming, sys::cna_wave_bank_create_streaming_fn),
            wave_bank_destroy: symbol!(cna_wave_bank_destroy, sys::cna_wave_bank_destroy_fn),
            wave_bank_is_prepared: symbol!(cna_wave_bank_get_is_prepared, sys::cna_wave_bank_get_is_prepared_fn),
            wave_bank_is_in_use: symbol!(cna_wave_bank_get_is_in_use, sys::cna_wave_bank_get_is_in_use_fn),
            sound_bank_create: symbol!(cna_sound_bank_create, sys::cna_sound_bank_create_fn),
            sound_bank_destroy: symbol!(cna_sound_bank_destroy, sys::cna_sound_bank_destroy_fn),
            sound_bank_is_in_use: symbol!(cna_sound_bank_get_is_in_use, sys::cna_sound_bank_get_is_in_use_fn),
            sound_bank_get_cue: symbol!(cna_sound_bank_get_cue, sys::cna_sound_bank_get_cue_fn),
            sound_bank_play_cue: symbol!(cna_sound_bank_play_cue, sys::cna_sound_bank_play_cue_fn),
            sound_bank_play_cue_3d: symbol!(cna_sound_bank_play_cue_3d, sys::cna_sound_bank_play_cue_3d_fn),
            cue_destroy: symbol!(cna_cue_destroy, sys::cna_cue_destroy_fn),
            cue_get_info: symbol!(cna_cue_get_info, sys::cna_cue_get_info_fn),
            cue_name_size: symbol!(cna_cue_get_name_size, sys::cna_cue_get_name_size_fn),
            cue_copy_name: symbol!(cna_cue_copy_name, sys::cna_cue_copy_name_fn),
            cue_apply_3d: symbol!(cna_cue_apply_3d, sys::cna_cue_apply_3d_fn),
            cue_get_variable: symbol!(cna_cue_get_variable, sys::cna_cue_get_variable_fn),
            cue_set_variable: symbol!(cna_cue_set_variable, sys::cna_cue_set_variable_fn),
            cue_play: symbol!(cna_cue_play, sys::cna_cue_play_fn),
            cue_pause: symbol!(cna_cue_pause, sys::cna_cue_pause_fn),
            cue_resume: symbol!(cna_cue_resume, sys::cna_cue_resume_fn),
            cue_stop: symbol!(cna_cue_stop, sys::cna_cue_stop_fn),
        })
    }
}

fn view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView { data: value.as_ptr().cast(), byte_length: value.len() as u64 }
}

fn pcm_info(sample_rate: i32, channels: sys::CNA_AudioChannels) -> Result<sys::CNA_SoundEffectCreateInfo> {
    let sample_rate = u32::try_from(sample_rate).map_err(|_| CnaError::InvalidInput("sample rate must be positive"))?;
    Ok(sys::CNA_SoundEffectCreateInfo {
        struct_size: size_of::<sys::CNA_SoundEffectCreateInfo>() as u32,
        struct_version: 1,
        sample_rate,
        channels,
        reserved: 0,
    })
}

impl Native {
    pub(crate) fn framework_dispatcher_update(&self, game: sys::CNA_Handle) -> Result<()> {
        self.check(unsafe { (self.audio.framework_dispatcher_update)(game) })
    }

    pub(crate) fn create_sound_effect(&self, game: sys::CNA_Handle, bytes: &[u8], sample_rate: i32, channels: sys::CNA_AudioChannels, range: Option<(i32, i32, i32, i32)>) -> Result<sys::CNA_Handle> {
        let info = pcm_info(sample_rate, channels)?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        let result = if let Some((offset, count, loop_start, loop_length)) = range {
            unsafe { (self.audio.sound_effect_create_pcm16_range)(game, &info, bytes.as_ptr(), bytes.len() as u64, offset, count, loop_start, loop_length, &mut handle) }
        } else {
            unsafe { (self.audio.sound_effect_create_pcm16)(game, &info, bytes.as_ptr(), bytes.len() as u64, &mut handle) }
        };
        self.check(result)?;
        Ok(handle)
    }

    pub(crate) fn create_encoded_sound_effect(&self, game: sys::CNA_Handle, bytes: &[u8]) -> Result<sys::CNA_Handle> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        self.check(unsafe { (self.audio.sound_effect_create_encoded)(game, bytes.as_ptr(), bytes.len() as u64, &mut handle) })?;
        Ok(handle)
    }

    pub(crate) fn sound_effect_duration(&self, handle: sys::CNA_Handle) -> Result<i64> { let mut value = 0; self.check(unsafe { (self.audio.sound_effect_duration)(handle, &mut value) })?; Ok(value) }
    pub(crate) fn destroy_sound_effect(&self, handle: sys::CNA_Handle) -> Result<()> { self.check(unsafe { (self.audio.sound_effect_destroy)(handle) }) }
    pub(crate) fn create_sound_effect_instance(&self, handle: sys::CNA_Handle) -> Result<sys::CNA_Handle> { let mut out = sys::CNA_INVALID_HANDLE; self.check(unsafe { (self.audio.sound_effect_create_instance)(handle, &mut out) })?; Ok(out) }
    pub(crate) fn sound_effect_name(&self, handle: sys::CNA_Handle) -> Result<String> { self.copy_string(handle, self.audio.sound_effect_name_size, self.audio.sound_effect_copy_name) }
    pub(crate) fn set_sound_effect_name(&self, handle: sys::CNA_Handle, value: &str) -> Result<()> { self.check(unsafe { (self.audio.sound_effect_set_name)(handle, view(value)) }) }
    pub(crate) fn sound_effect_setting(&self, game: sys::CNA_Handle, setting: u8) -> Result<f32> { let mut value = 0.0; let result = unsafe { match setting { 0 => (self.audio.sound_effect_get_master_volume)(game, &mut value), 1 => (self.audio.sound_effect_get_distance_scale)(game, &mut value), 2 => (self.audio.sound_effect_get_doppler_scale)(game, &mut value), _ => (self.audio.sound_effect_get_speed_of_sound)(game, &mut value) } }; self.check(result)?; Ok(value) }
    pub(crate) fn set_sound_effect_setting(&self, game: sys::CNA_Handle, setting: u8, value: f32) -> Result<()> { let result = unsafe { match setting { 0 => (self.audio.sound_effect_set_master_volume)(game, value), 1 => (self.audio.sound_effect_set_distance_scale)(game, value), 2 => (self.audio.sound_effect_set_doppler_scale)(game, value), _ => (self.audio.sound_effect_set_speed_of_sound)(game, value) } }; self.check(result) }
    pub(crate) fn play_sound_effect(&self, handle: sys::CNA_Handle, settings: Option<(f32, f32, f32)>) -> Result<bool> { let mut played = sys::CNA_FALSE; let result = unsafe { match settings { None => (self.audio.sound_effect_play)(handle, &mut played), Some((volume, pitch, pan)) => (self.audio.sound_effect_play_with_settings)(handle, volume, pitch, pan, &mut played) } }; self.check(result)?; Ok(played != sys::CNA_FALSE) }

    pub(crate) fn instance_info(&self, handle: sys::CNA_Handle) -> Result<sys::CNA_SoundEffectInstanceInfo> { let mut info = sys::CNA_SoundEffectInstanceInfo { struct_size: size_of::<sys::CNA_SoundEffectInstanceInfo>() as u32, struct_version: 1, state: 0, is_looped: 0, reserved0: [0; 3], volume: 0.0, pitch: 0.0, pan: 0.0, reserved1: 0 }; self.check(unsafe { (self.audio.instance_get_info)(handle, &mut info) })?; Ok(info) }
    pub(crate) fn instance_transport(&self, handle: sys::CNA_Handle, action: u8, immediate: bool) -> Result<()> { let result = unsafe { match action { 0 => (self.audio.instance_play)(handle), 1 => (self.audio.instance_pause)(handle), 2 => (self.audio.instance_resume)(handle), _ => (self.audio.instance_stop)(handle, if immediate { sys::CNA_TRUE } else { sys::CNA_FALSE }) } }; self.check(result) }
    pub(crate) fn set_instance_float(&self, handle: sys::CNA_Handle, property: u8, value: f32) -> Result<()> { let result = unsafe { match property { 0 => (self.audio.instance_set_volume)(handle, value), 1 => (self.audio.instance_set_pitch)(handle, value), _ => (self.audio.instance_set_pan)(handle, value) } }; self.check(result) }
    pub(crate) fn set_instance_looped(&self, handle: sys::CNA_Handle, value: bool) -> Result<()> { self.check(unsafe { (self.audio.instance_set_is_looped)(handle, if value { sys::CNA_TRUE } else { sys::CNA_FALSE }) }) }
    pub(crate) fn destroy_sound_effect_instance(&self, handle: sys::CNA_Handle) -> Result<()> { self.check(unsafe { (self.audio.instance_destroy)(handle) }) }
    pub(crate) fn apply_instance_3d(&self, handle: sys::CNA_Handle, listeners: &[sys::CNA_AudioListener], emitter: &sys::CNA_AudioEmitter) -> Result<()> { let result = unsafe { if listeners.len() == 1 { (self.audio.instance_apply_3d)(handle, &listeners[0], emitter) } else { (self.audio.instance_apply_3d_multi)(handle, listeners.as_ptr(), listeners.len() as u64, emitter) } }; self.check(result) }

    pub(crate) fn create_dynamic_instance(&self, game: sys::CNA_Handle, sample_rate: i32, channels: sys::CNA_AudioChannels) -> Result<sys::CNA_Handle> { let mut out = sys::CNA_INVALID_HANDLE; self.check(unsafe { (self.audio.dynamic_create)(game, sample_rate, channels, &mut out) })?; Ok(out) }
    pub(crate) fn dynamic_pending_count(&self, handle: sys::CNA_Handle) -> Result<i32> { let mut count = 0; self.check(unsafe { (self.audio.dynamic_pending_count)(handle, &mut count) })?; Ok(count) }
    pub(crate) fn dynamic_submit(&self, handle: sys::CNA_Handle, bytes: &[u8], offset: i32, count: i32) -> Result<()> { self.check(unsafe { (self.audio.dynamic_submit)(handle, bytes.as_ptr(), bytes.len() as u64, offset, count) }) }
    pub(crate) fn dynamic_duration(&self, handle: sys::CNA_Handle, bytes: i32) -> Result<i64> { let mut ticks = 0; self.check(unsafe { (self.audio.dynamic_duration)(handle, bytes, &mut ticks) })?; Ok(ticks) }
    pub(crate) fn dynamic_size(&self, handle: sys::CNA_Handle, ticks: i64) -> Result<i32> { let mut bytes = 0; self.check(unsafe { (self.audio.dynamic_size)(handle, ticks, &mut bytes) })?; Ok(bytes) }
    pub(crate) fn subscribe_dynamic(&self, handle: sys::CNA_Handle, callback: sys::CNA_AudioEventCallback, context: *mut c_void) -> Result<sys::CNA_Handle> { let mut out = sys::CNA_INVALID_HANDLE; self.check(unsafe { (self.audio.dynamic_subscribe)(handle, callback, context, &mut out) })?; Ok(out) }
    /// Subscribes to CNA's own disposal notification for one XACT object.
    ///
    /// `kind` picks the route: 0 engine, 1 wave bank, 2 sound bank, 3 cue.
    /// They differ only in which handle they accept, so one wrapper keeps the
    /// four call sites from repeating the same six lines.
    pub(crate) fn subscribe_xact_disposing(&self, kind: u8, handle: sys::CNA_Handle, callback: sys::CNA_AudioEventCallback, context: *mut c_void) -> Result<sys::CNA_Handle> {
        let mut out = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is live, the callback is a real `extern "C"` fn,
        // and the context outlives the subscription.
        let result = unsafe {
            match kind {
                0 => (self.audio.engine_subscribe_disposing)(handle, callback, context, &mut out),
                1 => (self.audio.wave_bank_subscribe_disposing)(handle, callback, context, &mut out),
                2 => (self.audio.sound_bank_subscribe_disposing)(handle, callback, context, &mut out),
                _ => (self.audio.cue_subscribe_disposing)(handle, callback, context, &mut out),
            }
        };
        self.check(result)?;
        Ok(out)
    }

    pub(crate) fn unsubscribe_audio(&self, registration: sys::CNA_Handle) -> Result<()> { self.check(unsafe { (self.audio.audio_unsubscribe)(registration) }) }

    pub(crate) fn microphone_count(&self, game: sys::CNA_Handle) -> Result<u64> { let mut count = 0; self.check(unsafe { (self.audio.microphone_count)(game, &mut count) })?; Ok(count) }
    pub(crate) fn microphone_default(&self, game: sys::CNA_Handle) -> Result<Option<u64>> { let mut index = 0; let mut available = sys::CNA_FALSE; self.check(unsafe { (self.audio.microphone_default)(game, &mut index, &mut available) })?; Ok((available != sys::CNA_FALSE).then_some(index)) }
    pub(crate) fn microphone_name(&self, game: sys::CNA_Handle, index: u64) -> Result<String> { self.copy_indexed_string(game, index, self.audio.microphone_name_size, self.audio.microphone_copy_name) }
    pub(crate) fn microphone_buffer_duration(&self, game: sys::CNA_Handle, index: u64) -> Result<i64> { let mut value = 0; self.check(unsafe { (self.audio.microphone_buffer_duration)(game, index, &mut value) })?; Ok(value) }
    pub(crate) fn set_microphone_buffer_duration(&self, game: sys::CNA_Handle, index: u64, ticks: i64) -> Result<()> { self.check(unsafe { (self.audio.microphone_set_buffer_duration)(game, index, ticks) }) }
    pub(crate) fn microphone_is_headset(&self, game: sys::CNA_Handle, index: u64) -> Result<bool> { let mut value = 0; self.check(unsafe { (self.audio.microphone_is_headset)(game, index, &mut value) })?; Ok(value != 0) }
    pub(crate) fn microphone_sample_rate(&self, game: sys::CNA_Handle, index: u64) -> Result<i32> { let mut value = 0; self.check(unsafe { (self.audio.microphone_sample_rate)(game, index, &mut value) })?; Ok(value) }
    pub(crate) fn microphone_state(&self, game: sys::CNA_Handle, index: u64) -> Result<sys::CNA_MicrophoneState> { let mut value = 0; self.check(unsafe { (self.audio.microphone_state)(game, index, &mut value) })?; Ok(value) }
    pub(crate) fn microphone_transport(&self, game: sys::CNA_Handle, index: u64, start: bool) -> Result<()> { let result = unsafe { if start { (self.audio.microphone_start)(game, index) } else { (self.audio.microphone_stop)(game, index) } }; self.check(result) }
    pub(crate) fn microphone_data(&self, game: sys::CNA_Handle, index: u64, destination: &mut [u8]) -> Result<usize> { let mut copied = 0; self.check(unsafe { (self.audio.microphone_get_data)(game, index, destination.as_mut_ptr(), destination.len() as u64, &mut copied) })?; usize::try_from(copied).map_err(|_| CnaError::InvalidInput("microphone byte count is too large")) }
    pub(crate) fn microphone_duration(&self, game: sys::CNA_Handle, index: u64, bytes: i32) -> Result<i64> { let mut ticks = 0; self.check(unsafe { (self.audio.microphone_duration)(game, index, bytes, &mut ticks) })?; Ok(ticks) }
    pub(crate) fn microphone_size(&self, game: sys::CNA_Handle, index: u64, ticks: i64) -> Result<i32> { let mut bytes = 0; self.check(unsafe { (self.audio.microphone_size)(game, index, ticks, &mut bytes) })?; Ok(bytes) }
    pub(crate) fn subscribe_microphone(&self, game: sys::CNA_Handle, index: u64, callback: sys::CNA_AudioEventCallback, context: *mut c_void) -> Result<sys::CNA_Handle> { let mut out = sys::CNA_INVALID_HANDLE; self.check(unsafe { (self.audio.microphone_subscribe)(game, index, callback, context, &mut out) })?; Ok(out) }

    pub(crate) fn create_audio_engine(&self, game: sys::CNA_Handle, file: &str, renderer: Option<(i64, &str)>) -> Result<sys::CNA_Handle> { let mut out = sys::CNA_INVALID_HANDLE; let result = unsafe { match renderer { None => (self.audio.engine_create)(game, view(file), &mut out), Some((ticks, id)) => (self.audio.engine_create_with_renderer)(game, view(file), ticks, view(id), &mut out) } }; self.check(result)?; Ok(out) }
    pub(crate) fn destroy_audio_engine(&self, handle: sys::CNA_Handle) -> Result<()> { self.check(unsafe { (self.audio.engine_destroy)(handle) }) }
    pub(crate) fn audio_renderers(&self, engine: sys::CNA_Handle) -> Result<Vec<(String, String)>> { let mut count = 0; self.check(unsafe { (self.audio.engine_renderer_count)(engine, &mut count) })?; let capacity = usize::try_from(count).map_err(|_| CnaError::InvalidInput("renderer count is too large"))?; let mut result = Vec::with_capacity(capacity); for index in 0..count { let friendly = self.copy_renderer_string(engine, index, self.audio.renderer_friendly_name_size, self.audio.renderer_copy_friendly_name)?; let id = self.copy_renderer_string(engine, index, self.audio.renderer_id_size, self.audio.renderer_copy_id)?; result.push((friendly, id)); } Ok(result) }
    pub(crate) fn audio_engine_global(&self, engine: sys::CNA_Handle, name: &str) -> Result<f32> { let mut value = 0.0; self.check(unsafe { (self.audio.engine_get_global)(engine, view(name), &mut value) })?; Ok(value) }
    pub(crate) fn set_audio_engine_global(&self, engine: sys::CNA_Handle, name: &str, value: f32) -> Result<()> { self.check(unsafe { (self.audio.engine_set_global)(engine, view(name), value) }) }
    pub(crate) fn update_audio_engine(&self, engine: sys::CNA_Handle) -> Result<()> { self.check(unsafe { (self.audio.engine_update)(engine) }) }
    pub(crate) fn audio_category(&self, engine: sys::CNA_Handle, name: &str) -> Result<sys::CNA_Handle> { let mut out = sys::CNA_INVALID_HANDLE; self.check(unsafe { (self.audio.engine_get_category)(engine, view(name), &mut out) })?; Ok(out) }
    pub(crate) fn destroy_audio_category(&self, handle: sys::CNA_Handle) -> Result<()> { self.check(unsafe { (self.audio.category_destroy)(handle) }) }
    pub(crate) fn audio_category_name(&self, handle: sys::CNA_Handle) -> Result<String> { self.copy_string(handle, self.audio.category_name_size, self.audio.category_copy_name) }
    pub(crate) fn audio_category_action(&self, handle: sys::CNA_Handle, action: u8, value: f32, options: sys::CNA_AudioStopOptions) -> Result<()> { let result = unsafe { match action { 0 => (self.audio.category_pause)(handle), 1 => (self.audio.category_resume)(handle), 2 => (self.audio.category_set_volume)(handle, value), _ => (self.audio.category_stop)(handle, options) } }; self.check(result) }
    pub(crate) fn audio_categories_equal(&self, left: sys::CNA_Handle, right: sys::CNA_Handle) -> Result<bool> { let mut value = sys::CNA_FALSE; self.check(unsafe { (self.audio.category_equals)(left, right, &mut value) })?; Ok(value != sys::CNA_FALSE) }
    pub(crate) fn audio_category_hash(&self, handle: sys::CNA_Handle) -> Result<i32> { let mut value = 0; self.check(unsafe { (self.audio.category_hash)(handle, &mut value) })?; Ok(value) }

    pub(crate) fn create_wave_bank(&self, engine: sys::CNA_Handle, file: &str, streaming: Option<(i32, i16)>) -> Result<sys::CNA_Handle> { let mut out = sys::CNA_INVALID_HANDLE; let result = unsafe { match streaming { None => (self.audio.wave_bank_create)(engine, view(file), &mut out), Some((offset, packet)) => (self.audio.wave_bank_create_streaming)(engine, view(file), offset, packet, &mut out) } }; self.check(result)?; Ok(out) }
    pub(crate) fn destroy_wave_bank(&self, handle: sys::CNA_Handle) -> Result<()> { self.check(unsafe { (self.audio.wave_bank_destroy)(handle) }) }
    pub(crate) fn wave_bank_flag(&self, handle: sys::CNA_Handle, prepared: bool) -> Result<bool> { let mut value = 0; let result = unsafe { if prepared { (self.audio.wave_bank_is_prepared)(handle, &mut value) } else { (self.audio.wave_bank_is_in_use)(handle, &mut value) } }; self.check(result)?; Ok(value != 0) }
    pub(crate) fn create_sound_bank(&self, engine: sys::CNA_Handle, file: &str) -> Result<sys::CNA_Handle> { let mut out = sys::CNA_INVALID_HANDLE; self.check(unsafe { (self.audio.sound_bank_create)(engine, view(file), &mut out) })?; Ok(out) }
    pub(crate) fn destroy_sound_bank(&self, handle: sys::CNA_Handle) -> Result<()> { self.check(unsafe { (self.audio.sound_bank_destroy)(handle) }) }
    pub(crate) fn sound_bank_is_in_use(&self, handle: sys::CNA_Handle) -> Result<bool> { let mut value = 0; self.check(unsafe { (self.audio.sound_bank_is_in_use)(handle, &mut value) })?; Ok(value != 0) }
    pub(crate) fn sound_bank_get_cue(&self, bank: sys::CNA_Handle, name: &str) -> Result<sys::CNA_Handle> { let mut out = sys::CNA_INVALID_HANDLE; self.check(unsafe { (self.audio.sound_bank_get_cue)(bank, view(name), &mut out) })?; Ok(out) }
    pub(crate) fn sound_bank_play(&self, bank: sys::CNA_Handle, name: &str, spatial: Option<(&sys::CNA_AudioListener, &sys::CNA_AudioEmitter)>) -> Result<()> { let result = unsafe { spatial.map_or_else(|| (self.audio.sound_bank_play_cue)(bank, view(name)), |(listener, emitter)| (self.audio.sound_bank_play_cue_3d)(bank, view(name), listener, emitter)) }; self.check(result) }
    pub(crate) fn destroy_cue(&self, handle: sys::CNA_Handle) -> Result<()> { self.check(unsafe { (self.audio.cue_destroy)(handle) }) }
    pub(crate) fn cue_info(&self, handle: sys::CNA_Handle) -> Result<sys::CNA_CueInfo> { let mut info = sys::CNA_CueInfo { struct_size: size_of::<sys::CNA_CueInfo>() as u32, struct_version: 1, is_created: 0, is_disposed: 0, is_paused: 0, is_playing: 0, is_prepared: 0, is_preparing: 0, is_stopped: 0, is_stopping: 0 }; self.check(unsafe { (self.audio.cue_get_info)(handle, &mut info) })?; Ok(info) }
    pub(crate) fn cue_name(&self, handle: sys::CNA_Handle) -> Result<String> { self.copy_string(handle, self.audio.cue_name_size, self.audio.cue_copy_name) }
    pub(crate) fn cue_apply_3d(&self, handle: sys::CNA_Handle, listener: &sys::CNA_AudioListener, emitter: &sys::CNA_AudioEmitter) -> Result<()> { self.check(unsafe { (self.audio.cue_apply_3d)(handle, listener, emitter) }) }
    pub(crate) fn cue_variable(&self, handle: sys::CNA_Handle, name: &str) -> Result<f32> { let mut value = 0.0; self.check(unsafe { (self.audio.cue_get_variable)(handle, view(name), &mut value) })?; Ok(value) }
    pub(crate) fn set_cue_variable(&self, handle: sys::CNA_Handle, name: &str, value: f32) -> Result<()> { self.check(unsafe { (self.audio.cue_set_variable)(handle, view(name), value) }) }
    pub(crate) fn cue_transport(&self, handle: sys::CNA_Handle, action: u8, options: sys::CNA_AudioStopOptions) -> Result<()> { let result = unsafe { match action { 0 => (self.audio.cue_play)(handle), 1 => (self.audio.cue_pause)(handle), 2 => (self.audio.cue_resume)(handle), _ => (self.audio.cue_stop)(handle, options) } }; self.check(result) }

    fn copy_string(&self, handle: sys::CNA_Handle, size: unsafe extern "C" fn(sys::CNA_Handle, *mut u64) -> sys::CNA_Result, copy: unsafe extern "C" fn(sys::CNA_Handle, *mut core::ffi::c_char, u64, *mut u64) -> sys::CNA_Result) -> Result<String> { let mut required = 0; self.check(unsafe { size(handle, &mut required) })?; let capacity = usize::try_from(required).map_err(|_| CnaError::InvalidInput("native string is too large"))?; let mut bytes = vec![0_u8; capacity]; let mut copied = 0; self.check(unsafe { copy(handle, bytes.as_mut_ptr().cast(), required, &mut copied) })?; bytes.truncate(usize::try_from(copied).map_err(|_| CnaError::InvalidInput("native string is too large"))?); String::from_utf8(bytes).map_err(|_| CnaError::InvalidInput("native string is not UTF-8")) }
    fn copy_indexed_string(&self, handle: sys::CNA_Handle, index: u64, size: unsafe extern "C" fn(sys::CNA_Handle, u64, *mut u64) -> sys::CNA_Result, copy: unsafe extern "C" fn(sys::CNA_Handle, u64, *mut core::ffi::c_char, u64, *mut u64) -> sys::CNA_Result) -> Result<String> { let mut required = 0; self.check(unsafe { size(handle, index, &mut required) })?; let capacity = usize::try_from(required).map_err(|_| CnaError::InvalidInput("native string is too large"))?; let mut bytes = vec![0_u8; capacity]; let mut copied = 0; self.check(unsafe { copy(handle, index, bytes.as_mut_ptr().cast(), required, &mut copied) })?; bytes.truncate(usize::try_from(copied).map_err(|_| CnaError::InvalidInput("native string is too large"))?); String::from_utf8(bytes).map_err(|_| CnaError::InvalidInput("native string is not UTF-8")) }
    fn copy_renderer_string(&self, handle: sys::CNA_Handle, index: u64, size: unsafe extern "C" fn(sys::CNA_Handle, u64, *mut u64) -> sys::CNA_Result, copy: unsafe extern "C" fn(sys::CNA_Handle, u64, *mut core::ffi::c_char, u64, *mut u64) -> sys::CNA_Result) -> Result<String> { self.copy_indexed_string(handle, index, size, copy) }
}

/// `audio.h` and `xact.h`: the disposal facts, the engine's renderer identity,
/// and the buffer paths a dynamic instance needs.
///
/// Three groups worth telling apart.
///
/// Every XACT object gets a `get_is_disposed` and a `subscribe_disposing_ext`.
/// The safe layer already tracks disposal on the Rust side, so the *reading*
/// route mostly agrees with what Rust knows -- but the *event* does not have a
/// Rust counterpart at all: it fires when CNA disposes the object, including
/// disposals a Rust caller did not initiate, such as an engine taking its
/// children down with it.
///
/// `cna_audio_engine_renderers_equal` and the renderer hash are the identity
/// half of `RendererDetail`. XNA compares renderer descriptors by value; these
/// are CNA's own answers for the same question, and comparing in Rust instead
/// would be restating an equality this ABI already defines.
///
/// The dynamic-instance routes are the submission path: a float buffer rather
/// than the PCM one already bound, the initial queue, a clear, and the pump
/// that moves finished buffers off the queue.
impl Native {
    pub(crate) fn audio_capabilities(&self, game: sys::CNA_Handle) -> Result<bool> {
        let mut info = sys::CNA_AudioCapabilities {
            struct_size: core::mem::size_of::<sys::CNA_AudioCapabilities>() as u32,
            struct_version: 1,
            ..sys::CNA_AudioCapabilities::default()
        };
        // SAFETY: the output is a complete versioned local.
        self.check(unsafe { (self.audio_get_capabilities)(game, &mut info) })?;
        Ok(info.is_playback_available != sys::CNA_FALSE)
    }

    pub(crate) fn audio_engine_is_disposed(&self, handle: sys::CNA_Handle) -> Result<bool> {
        self.read_disposed(handle, self.audio_engine_get_is_disposed)
    }

    pub(crate) fn wave_bank_is_disposed(&self, handle: sys::CNA_Handle) -> Result<bool> {
        self.read_disposed(handle, self.wave_bank_get_is_disposed)
    }

    pub(crate) fn sound_bank_is_disposed(&self, handle: sys::CNA_Handle) -> Result<bool> {
        self.read_disposed(handle, self.sound_bank_get_is_disposed)
    }

    pub(crate) fn sound_effect_is_disposed(&self, handle: sys::CNA_Handle) -> Result<bool> {
        self.read_disposed(handle, self.sound_effect_get_is_disposed)
    }

    pub(crate) fn sound_effect_instance_is_disposed(
        &self,
        handle: sys::CNA_Handle,
    ) -> Result<bool> {
        self.read_disposed(handle, self.sound_effect_instance_get_is_disposed)
    }

    fn read_disposed(
        &self,
        handle: sys::CNA_Handle,
        route: unsafe extern "C" fn(sys::CNA_Handle, *mut sys::CNA_Bool) -> sys::CNA_Result,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { route(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn audio_engine_renderer_text(
        &self,
        handle: sys::CNA_Handle,
        index: u64,
    ) -> Result<String> {
        let mut count = 0_u64;
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe {
            (self.audio_engine_get_renderer_text_size)(handle, index, &mut count)
        })?;
        let capacity = usize::try_from(count).unwrap_or(0);
        let mut bytes = vec![0_u8; capacity];
        let mut written = count;
        // SAFETY: the destination has exactly `count` writable bytes.
        self.check(unsafe {
            (self.audio_engine_copy_renderer_text)(
                handle,
                index,
                bytes.as_mut_ptr().cast(),
                count,
                &mut written,
            )
        })?;
        bytes.truncate(usize::try_from(written).unwrap_or(0).min(capacity));
        String::from_utf8(bytes).map_err(|_| CnaError::Native {
            code: sys::CNA_RESULT_ENCODING,
            category: ErrorCategory::None,
            message: "CNA returned invalid UTF-8 renderer text".to_owned(),
        })
    }

    pub(crate) fn audio_engine_renderer_hash(
        &self,
        handle: sys::CNA_Handle,
        index: u64,
    ) -> Result<i32> {
        let mut value = 0;
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe {
            (self.audio_engine_get_renderer_hash_code)(handle, index, &mut value)
        })?;
        Ok(value)
    }

    pub(crate) fn audio_engine_renderers_equal(
        &self,
        handle: sys::CNA_Handle,
        left: u64,
        right: u64,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe {
            (self.audio_engine_renderers_equal)(handle, left, right, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Submits a range of a float buffer, which CNA copies during the call.
    ///
    /// The route takes the whole slice *and* an offset and count into it, so
    /// the range is validated here before either reaches C: an offset or count
    /// past the end would otherwise be a read past the slice.
    pub(crate) fn submit_dynamic_float_buffer(
        &self,
        handle: sys::CNA_Handle,
        samples: &[f32],
        offset: i32,
        count: i32,
    ) -> Result<()> {
        let start = usize::try_from(offset)
            .map_err(|_| CnaError::InvalidInput("a sample offset cannot be negative"))?;
        let length = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("a sample count cannot be negative"))?;
        // `map_or(true, ..)` rather than `is_none_or`, which is 1.82 and this
        // crate's declared MSRV is 1.74.
        if start
            .checked_add(length)
            .map_or(true, |end| end > samples.len())
        {
            return Err(CnaError::InvalidInput(
                "the submitted range must lie inside the sample buffer",
            ));
        }
        // SAFETY: the slice outlives the call, its length is passed exactly,
        // and the range was just checked to lie inside it.
        self.check(unsafe {
            (self.dynamic_sound_effect_instance_submit_float_buffer_ext)(
                handle,
                samples.as_ptr(),
                samples.len() as u64,
                offset,
                count,
            )
        })
    }

    pub(crate) fn queue_dynamic_initial_buffers(
        &self,
        handle: sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the handle is live.
        self.check(unsafe {
            (self.dynamic_sound_effect_instance_queue_initial_buffers_ext)(handle)
        })
    }

    pub(crate) fn clear_dynamic_buffers(&self, handle: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the handle is live.
        self.check(unsafe { (self.dynamic_sound_effect_instance_clear_buffers_ext)(handle) })
    }

    pub(crate) fn update_dynamic_instance(&self, handle: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the handle is live.
        self.check(unsafe { (self.dynamic_sound_effect_instance_update_ext)(handle) })
    }

    pub(crate) fn check_all_microphone_buffers(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the game handle is live for the call.
        self.check(unsafe { (self.microphone_check_all_buffers_ext)(game) })
    }
}

/// `audio.h`'s file-decoding constructor and the process capability query.
impl Native {
    pub(crate) fn create_sound_effect_from_asset(
        &self,
        game: sys::CNA_Handle,
        asset_name: &str,
    ) -> Result<sys::CNA_Handle> {
        let view = sys::CNA_StringView {
            data: asset_name.as_ptr().cast(),
            byte_length: asset_name.len() as u64,
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the game handle is live, the path outlives the call, and the
        // output is a live local.
        self.check(unsafe {
            (self.sound_effect_create_from_asset_ext)(game, view, &mut handle)
        })?;
        Ok(handle)
    }
}
