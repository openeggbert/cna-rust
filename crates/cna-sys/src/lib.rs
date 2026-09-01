//! Raw declarations for the reviewed CNA C ABI runtime/2D slice.
//!
//! These layouts and function-pointer types are derived from CNA's canonical
//! `modules/c-api/include/CNA/C` headers at ABI 0.21.0. The crate deliberately
//! does not add linker directives: the safe crate resolves a user-selected CNA
//! shared library at runtime and checks its ABI version before loading symbols.

#![no_std]
#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_void};

#[cfg(feature = "direct-link")]
pub mod linked;

pub const BINDINGS_AVAILABLE: bool = true;

/// Packs an ABI version the way canonical `CNA_ABI_VERSION_ENCODE` does.
///
/// The representation is fixed by `docs/c-api/ABI_VERSIONING.md`: major in bits
/// 31..16, minor in bits 15..8, patch in bits 7..0.
#[must_use]
pub const fn cna_abi_version_encode(major: u32, minor: u32, patch: u32) -> u32 {
    ((major & 0xFFFF) << 16) | ((minor & 0xFF) << 8) | (patch & 0xFF)
}

/// Extracts the major component of an encoded ABI version.
#[must_use]
pub const fn cna_abi_version_major(version: u32) -> u32 {
    (version >> 16) & 0xFFFF
}

/// Extracts the minor component of an encoded ABI version.
#[must_use]
pub const fn cna_abi_version_minor(version: u32) -> u32 {
    (version >> 8) & 0xFF
}

/// Extracts the patch component of an encoded ABI version.
#[must_use]
pub const fn cna_abi_version_patch(version: u32) -> u32 {
    version & 0xFF
}

/// Major component of the reviewed canonical ABI (`CNA_ABI_VERSION_MAJOR`).
pub const CNA_ABI_VERSION_MAJOR: u32 = 0;
/// Minor component of the reviewed canonical ABI (`CNA_ABI_VERSION_MINOR`).
pub const CNA_ABI_VERSION_MINOR: u32 = 21;
/// Patch component of the reviewed canonical ABI (`CNA_ABI_VERSION_PATCH`).
pub const CNA_ABI_VERSION_PATCH: u32 = 0;

/// The exact canonical ABI version these declarations were reviewed against.
pub const CNA_ABI_VERSION: u32 = cna_abi_version_encode(
    CNA_ABI_VERSION_MAJOR,
    CNA_ABI_VERSION_MINOR,
    CNA_ABI_VERSION_PATCH,
);
pub const CNA_RESULT_SUCCESS: CNA_Result = 0;
pub const CNA_RESULT_INVALID_ARGUMENT: CNA_Result = 1;
pub const CNA_RESULT_INVALID_HANDLE: CNA_Result = 2;
pub const CNA_RESULT_INVALID_STATE: CNA_Result = 3;
pub const CNA_RESULT_OUT_OF_MEMORY: CNA_Result = 4;
pub const CNA_RESULT_IO: CNA_Result = 5;
pub const CNA_RESULT_NOT_SUPPORTED: CNA_Result = 6;
pub const CNA_RESULT_PLATFORM: CNA_Result = 7;
pub const CNA_RESULT_THREAD: CNA_Result = 8;
pub const CNA_RESULT_CALLBACK: CNA_Result = 9;
pub const CNA_RESULT_OVERFLOW: CNA_Result = 10;
pub const CNA_RESULT_ENCODING: CNA_Result = 11;
pub const CNA_RESULT_INTERNAL: CNA_Result = 12;
pub const CNA_RESULT_SHUTTING_DOWN: CNA_Result = 13;
pub const CNA_RESULT_BUFFER_TOO_SMALL: CNA_Result = 14;
pub const CNA_FALSE: CNA_Bool = 0;
pub const CNA_TRUE: CNA_Bool = 1;
pub const CNA_INVALID_HANDLE: CNA_Handle = 0;
pub const CNA_ERROR_INFO_STRUCT_VERSION: u32 = 1;
pub const CNA_ERROR_CATEGORY_NONE: CNA_ErrorCategory = 0;
pub const CNA_ERROR_CATEGORY_ARGUMENT: CNA_ErrorCategory = 1;
pub const CNA_ERROR_CATEGORY_HANDLE: CNA_ErrorCategory = 2;
pub const CNA_ERROR_CATEGORY_STATE: CNA_ErrorCategory = 3;
pub const CNA_ERROR_CATEGORY_MEMORY: CNA_ErrorCategory = 4;
pub const CNA_ERROR_CATEGORY_IO: CNA_ErrorCategory = 5;
pub const CNA_ERROR_CATEGORY_NOT_SUPPORTED: CNA_ErrorCategory = 6;
pub const CNA_ERROR_CATEGORY_PLATFORM: CNA_ErrorCategory = 7;
pub const CNA_ERROR_CATEGORY_THREAD: CNA_ErrorCategory = 8;
pub const CNA_ERROR_CATEGORY_CALLBACK: CNA_ErrorCategory = 9;
pub const CNA_ERROR_CATEGORY_RANGE: CNA_ErrorCategory = 10;
pub const CNA_ERROR_CATEGORY_ENCODING: CNA_ErrorCategory = 11;
pub const CNA_ERROR_CATEGORY_INTERNAL: CNA_ErrorCategory = 12;
pub const CNA_ERROR_CATEGORY_SHUTTING_DOWN: CNA_ErrorCategory = 13;
pub const CNA_MEDIA_STATE_STOPPED: CNA_MediaState = 0;
pub const CNA_MEDIA_STATE_PLAYING: CNA_MediaState = 1;
pub const CNA_MEDIA_STATE_PAUSED: CNA_MediaState = 2;
pub const CNA_MEDIA_SOURCE_TYPE_LOCAL_DEVICE: CNA_MediaSourceType = 0;
pub const CNA_MEDIA_SOURCE_TYPE_WINDOWS_MEDIA_CONNECT: CNA_MediaSourceType = 4;
pub const CNA_VIDEO_SOUNDTRACK_TYPE_MUSIC: CNA_VideoSoundtrackType = 0;
pub const CNA_VIDEO_SOUNDTRACK_TYPE_DIALOG: CNA_VideoSoundtrackType = 1;
pub const CNA_VIDEO_SOUNDTRACK_TYPE_MUSIC_AND_DIALOG: CNA_VideoSoundtrackType = 2;
pub const CNA_VISUALIZATION_DATA_SIZE: u32 = 256;
pub const CNA_VIDEO_FRAME_EXT_STRUCT_VERSION: u32 = 1;
pub const CNA_GRAPHICS_RENDERER_FALLBACK_RECORD_STRUCT_VERSION: u32 = 1;
pub const CNA_LOG_LEVEL_FATAL: CNA_LogLevel = 0;
pub const CNA_LOG_LEVEL_ERROR: CNA_LogLevel = 1;
pub const CNA_LOG_LEVEL_WARN: CNA_LogLevel = 2;
pub const CNA_LOG_LEVEL_INFO: CNA_LogLevel = 3;
pub const CNA_LOG_LEVEL_DEBUG: CNA_LogLevel = 4;
pub const CNA_LOG_LEVEL_TRACE: CNA_LogLevel = 5;
pub const CNA_LOG_LEVEL_EXPERIMENT: CNA_LogLevel = 100;
pub const CNA_LOG_CATEGORY_APPLICATION: CNA_LogCategory = 0;
pub const CNA_LOG_CATEGORY_ERROR: CNA_LogCategory = 1;
pub const CNA_LOG_CATEGORY_SYSTEM: CNA_LogCategory = 2;
pub const CNA_LOG_CATEGORY_AUDIO: CNA_LogCategory = 3;
pub const CNA_LOG_CATEGORY_VIDEO: CNA_LogCategory = 4;
pub const CNA_LOG_CATEGORY_RENDER: CNA_LogCategory = 5;
pub const CNA_LOG_CATEGORY_INPUT: CNA_LogCategory = 6;
pub const CNA_LOG_CATEGORY_TEST: CNA_LogCategory = 7;
pub const CNA_LOG_CATEGORY_GPU: CNA_LogCategory = 8;
pub const CNA_RENDERER_FEATURE_SUPPORT_UNKNOWN: CNA_RendererFeatureSupport = 0;
pub const CNA_RENDERER_FEATURE_SUPPORT_UNSUPPORTED: CNA_RendererFeatureSupport = 1;
pub const CNA_RENDERER_FEATURE_SUPPORT_SUPPORTED: CNA_RendererFeatureSupport = 2;
pub const CNA_RENDERER_FEATURE_SUPPORT_RESTRICTED: CNA_RendererFeatureSupport = 3;
pub const CNA_SHADER_DIALECT_UNKNOWN: CNA_ShaderDialect = 0;
pub const CNA_SHADER_DIALECT_GLSL_DESKTOP: CNA_ShaderDialect = 1;
pub const CNA_SHADER_DIALECT_GLSL_ES: CNA_ShaderDialect = 2;
pub const CNA_SHADER_DIALECT_GLSL_VULKAN: CNA_ShaderDialect = 3;
pub const CNA_SHADER_DIALECT_HLSL: CNA_ShaderDialect = 4;
pub const CNA_SHADER_DIALECT_MSL: CNA_ShaderDialect = 5;
pub const CNA_SHADER_DIALECT_WGSL: CNA_ShaderDialect = 6;
pub const CNA_RENDERER_FORMAT_USAGE_TEXTURE_STORAGE: CNA_RendererFormatUsageFlags = 1 << 0;
pub const CNA_RENDERER_FORMAT_USAGE_SAMPLED: CNA_RendererFormatUsageFlags = 1 << 1;
pub const CNA_RENDERER_FORMAT_USAGE_FILTERABLE: CNA_RendererFormatUsageFlags = 1 << 2;
pub const CNA_RENDERER_FORMAT_USAGE_RENDER_TARGET: CNA_RendererFormatUsageFlags = 1 << 3;
pub const CNA_RENDERER_FORMAT_USAGE_BLENDABLE: CNA_RendererFormatUsageFlags = 1 << 4;
pub const CNA_RENDERER_FORMAT_USAGE_STORAGE_READ: CNA_RendererFormatUsageFlags = 1 << 5;
pub const CNA_RENDERER_FORMAT_USAGE_STORAGE_WRITE: CNA_RendererFormatUsageFlags = 1 << 6;
pub const CNA_RENDERER_FORMAT_USAGE_STORAGE_ATOMIC: CNA_RendererFormatUsageFlags = 1 << 7;
pub const CNA_RENDERER_FORMAT_USAGE_TRANSFER_SOURCE: CNA_RendererFormatUsageFlags = 1 << 8;
pub const CNA_RENDERER_FORMAT_USAGE_TRANSFER_DESTINATION: CNA_RendererFormatUsageFlags = 1 << 9;
pub const CNA_RENDERER_FORMAT_USAGE_MIPMAPPED: CNA_RendererFormatUsageFlags = 1 << 10;
pub const CNA_RENDERER_FORMAT_USAGE_MULTISAMPLE: CNA_RendererFormatUsageFlags = 1 << 11;
pub const CNA_RENDERER_FORMAT_USAGE_COLOR_TRANSFER: CNA_RendererFormatUsageFlags = 1 << 12;
pub const CNA_RENDERER_FORMAT_USAGE_ALL: CNA_RendererFormatUsageFlags = (1 << 13) - 1;
pub const CNA_JOYSTICK_STRUCT_VERSION: u32 = 1;
pub const CNA_JOYSTICK_TYPE_UNKNOWN: CNA_JoystickType = 0;
pub const CNA_JOYSTICK_TYPE_GAMEPAD: CNA_JoystickType = 1;
pub const CNA_JOYSTICK_TYPE_WHEEL: CNA_JoystickType = 2;
pub const CNA_JOYSTICK_TYPE_ARCADE_STICK: CNA_JoystickType = 3;
pub const CNA_JOYSTICK_TYPE_FLIGHT_STICK: CNA_JoystickType = 4;
pub const CNA_JOYSTICK_TYPE_DANCE_PAD: CNA_JoystickType = 5;
pub const CNA_JOYSTICK_TYPE_GUITAR: CNA_JoystickType = 6;
pub const CNA_JOYSTICK_TYPE_DRUM_KIT: CNA_JoystickType = 7;
pub const CNA_JOYSTICK_TYPE_ARCADE_PAD: CNA_JoystickType = 8;
pub const CNA_JOYSTICK_TYPE_THROTTLE: CNA_JoystickType = 9;
pub const CNA_JOYSTICK_HAT_POSITION_CENTERED: CNA_JoystickHatPosition = 0;
pub const CNA_JOYSTICK_HAT_POSITION_UP: CNA_JoystickHatPosition = 1;
pub const CNA_JOYSTICK_HAT_POSITION_RIGHT: CNA_JoystickHatPosition = 2;
pub const CNA_JOYSTICK_HAT_POSITION_DOWN: CNA_JoystickHatPosition = 3;
pub const CNA_JOYSTICK_HAT_POSITION_LEFT: CNA_JoystickHatPosition = 4;
pub const CNA_JOYSTICK_HAT_POSITION_RIGHT_UP: CNA_JoystickHatPosition = 5;
pub const CNA_JOYSTICK_HAT_POSITION_RIGHT_DOWN: CNA_JoystickHatPosition = 6;
pub const CNA_JOYSTICK_HAT_POSITION_LEFT_UP: CNA_JoystickHatPosition = 7;
pub const CNA_JOYSTICK_HAT_POSITION_LEFT_DOWN: CNA_JoystickHatPosition = 8;
pub const CNA_ASCII_QUANTIZE_MODE_BLACK_WHITE: CNA_AsciiQuantizeMode = 0;
pub const CNA_ASCII_QUANTIZE_MODE_COLOR: CNA_AsciiQuantizeMode = 1;
pub const CNA_CRT_MASK_TYPE_NONE: CNA_CRTMaskType = 0;
pub const CNA_CRT_MASK_TYPE_APERTURE_GRILLE: CNA_CRTMaskType = 1;
pub const CNA_CRT_MASK_TYPE_SHADOW_MASK: CNA_CRTMaskType = 2;
pub const CNA_DITHER_MODE_NONE: CNA_DitherMode = 0;
pub const CNA_DITHER_MODE_BAYER_4X4: CNA_DitherMode = 1;
pub const CNA_DITHER_MODE_BAYER_8X8: CNA_DitherMode = 2;
pub const CNA_DEPTH_EFFECT_MODE_COLOR_16_BIT: CNA_DepthEffectMode = 0;
pub const CNA_DEPTH_EFFECT_MODE_COLOR_8_BIT: CNA_DepthEffectMode = 1;
pub const CNA_DEPTH_EFFECT_MODE_GRAYSCALE_4_BIT: CNA_DepthEffectMode = 2;
pub const CNA_DEPTH_EFFECT_MODE_GRAYSCALE_2_BIT: CNA_DepthEffectMode = 3;
pub const CNA_DEPTH_EFFECT_MODE_GRAYSCALE_1_BIT: CNA_DepthEffectMode = 4;
pub const CNA_DEPTH_EFFECT_MODE_PALETTE_256: CNA_DepthEffectMode = 5;
pub const CNA_DEPTH_EFFECT_MODE_PALETTE_16: CNA_DepthEffectMode = 6;
pub const CNA_CNB_ASSET_TYPE_INVALID: u32 = 0x0000_0000;
pub const CNA_CNB_ASSET_TYPE_TEXTURE2D: u32 = 0x0000_0001;
pub const CNA_CNB_ASSET_TYPE_TEXTURE3D: u32 = 0x0000_0002;
pub const CNA_CNB_ASSET_TYPE_TEXTURE_CUBE: u32 = 0x0000_0003;
pub const CNA_CNB_ASSET_TYPE_SPRITE_FONT: u32 = 0x0000_0004;
pub const CNA_CNB_ASSET_TYPE_MODEL: u32 = 0x0000_0005;
pub const CNA_CNB_ASSET_TYPE_ANIMATION_CLIP: u32 = 0x0000_0006;
pub const CNA_CNB_ASSET_TYPE_CURVE: u32 = 0x0000_0007;
pub const CNA_CNB_ASSET_TYPE_SOUND_EFFECT: u32 = 0x0000_0008;
pub const CNA_CNB_ASSET_TYPE_SONG: u32 = 0x0000_0009;
pub const CNA_CNB_ASSET_TYPE_VIDEO: u32 = 0x0000_000A;
pub const CNA_CNB_ASSET_TYPE_EFFECT: u32 = 0x0000_000B;
pub const CNA_CNB_ASSET_TYPE_RESERVED_RANGE_FIRST: u32 = 0x4000_0000;
pub const CNA_CNB_ASSET_TYPE_CUSTOM_RANGE_FIRST: u32 = 0x8000_0000;
pub const CNA_CNB_READ_LIMITS_STRUCT_VERSION: u32 = 1;
pub const CNA_CNB_METADATA_STRUCT_VERSION: u32 = 1;
pub const CNA_CNB_TEXTURE_INFO_STRUCT_VERSION: u32 = 1;
pub const CNA_CNB_MODEL_INFO_STRUCT_VERSION: u32 = 1;
pub const CNA_CNB_SPRITE_FONT_INFO_STRUCT_VERSION: u32 = 1;
pub const CNA_TRANSPARENCY_MODE_NONE: CNA_TransparencyMode = 0;
pub const CNA_TRANSPARENCY_MODE_SORTED: CNA_TransparencyMode = 1;
pub const CNA_TRANSPARENCY_MODE_ORDER_INDEPENDENT: CNA_TransparencyMode = 2;
pub const CNA_ALPHA_MODE_OPAQUE_EXT: CNA_AlphaModeEXT = 0;
pub const CNA_ALPHA_MODE_MASK_EXT: CNA_AlphaModeEXT = 1;
pub const CNA_ALPHA_MODE_BLEND_EXT: CNA_AlphaModeEXT = 2;
pub const CNA_ALPHA_MODE_MAXIMUM_EXT: CNA_AlphaModeEXT = CNA_ALPHA_MODE_BLEND_EXT;
pub const CNA_TONEMAPPING_MODE_NONE: CNA_TonemappingMode = 0;
pub const CNA_TONEMAPPING_MODE_REINHARD: CNA_TonemappingMode = 1;
pub const CNA_TONEMAPPING_MODE_FILMIC: CNA_TonemappingMode = 2;
pub const CNA_TONEMAPPING_MODE_ACES: CNA_TonemappingMode = 3;
pub const CNA_RENDER_QUALITY_LOW: CNA_RenderQuality = 0;
pub const CNA_RENDER_QUALITY_MEDIUM: CNA_RenderQuality = 1;
pub const CNA_RENDER_QUALITY_HIGH: CNA_RenderQuality = 2;
pub const CNA_RENDER_QUALITY_ULTRA: CNA_RenderQuality = 3;
pub const CNA_SHADOW_QUALITY_DISABLED: CNA_ShadowQuality = 0;
pub const CNA_SHADOW_QUALITY_LOW: CNA_ShadowQuality = 1;
pub const CNA_SHADOW_QUALITY_MEDIUM: CNA_ShadowQuality = 2;
pub const CNA_SHADOW_QUALITY_HIGH: CNA_ShadowQuality = 3;
pub const CNA_SHADOW_QUALITY_ULTRA: CNA_ShadowQuality = 4;
pub const CNA_HAPTIC_EFFECT_TYPE_CONSTANT: CNA_HapticEffectType = 0;
pub const CNA_HAPTIC_EFFECT_TYPE_SINE: CNA_HapticEffectType = 1;
pub const CNA_HAPTIC_EFFECT_TYPE_SQUARE: CNA_HapticEffectType = 2;
pub const CNA_HAPTIC_EFFECT_TYPE_TRIANGLE: CNA_HapticEffectType = 3;
pub const CNA_HAPTIC_EFFECT_TYPE_SAWTOOTH_UP: CNA_HapticEffectType = 4;
pub const CNA_HAPTIC_EFFECT_TYPE_SAWTOOTH_DOWN: CNA_HapticEffectType = 5;
pub const CNA_HAPTIC_EFFECT_TYPE_RAMP: CNA_HapticEffectType = 6;
pub const CNA_HAPTIC_EFFECT_TYPE_SPRING: CNA_HapticEffectType = 7;
pub const CNA_HAPTIC_EFFECT_TYPE_DAMPER: CNA_HapticEffectType = 8;
pub const CNA_HAPTIC_EFFECT_TYPE_INERTIA: CNA_HapticEffectType = 9;
pub const CNA_HAPTIC_EFFECT_TYPE_FRICTION: CNA_HapticEffectType = 10;
pub const CNA_HAPTIC_EFFECT_TYPE_LEFT_RIGHT: CNA_HapticEffectType = 11;
pub const CNA_HAPTIC_EFFECT_TYPE_CUSTOM: CNA_HapticEffectType = 12;
pub const CNA_HAPTIC_EFFECT_TYPE_MAXIMUM: CNA_HapticEffectType =
    CNA_HAPTIC_EFFECT_TYPE_CUSTOM;
pub const CNA_HAPTIC_DIRECTION_TYPE_POLAR: CNA_HapticDirectionType = 0;
pub const CNA_HAPTIC_DIRECTION_TYPE_CARTESIAN: CNA_HapticDirectionType = 1;
pub const CNA_HAPTIC_DIRECTION_TYPE_SPHERICAL: CNA_HapticDirectionType = 2;
pub const CNA_HAPTIC_DIRECTION_TYPE_STEERING_AXIS: CNA_HapticDirectionType = 3;
pub const CNA_HAPTIC_DIRECTION_TYPE_MAXIMUM: CNA_HapticDirectionType =
    CNA_HAPTIC_DIRECTION_TYPE_STEERING_AXIS;
pub const CNA_HAPTIC_FEATURE_NONE: CNA_HapticFeature = 0;
pub const CNA_HAPTIC_FEATURE_CONSTANT: CNA_HapticFeature = 0x0000_0001;
pub const CNA_HAPTIC_FEATURE_SINE: CNA_HapticFeature = 0x0000_0002;
pub const CNA_HAPTIC_FEATURE_SQUARE: CNA_HapticFeature = 0x0000_0004;
pub const CNA_HAPTIC_FEATURE_TRIANGLE: CNA_HapticFeature = 0x0000_0008;
pub const CNA_HAPTIC_FEATURE_SAWTOOTH_UP: CNA_HapticFeature = 0x0000_0010;
pub const CNA_HAPTIC_FEATURE_SAWTOOTH_DOWN: CNA_HapticFeature = 0x0000_0020;
pub const CNA_HAPTIC_FEATURE_RAMP: CNA_HapticFeature = 0x0000_0040;
pub const CNA_HAPTIC_FEATURE_SPRING: CNA_HapticFeature = 0x0000_0080;
pub const CNA_HAPTIC_FEATURE_DAMPER: CNA_HapticFeature = 0x0000_0100;
pub const CNA_HAPTIC_FEATURE_INERTIA: CNA_HapticFeature = 0x0000_0200;
pub const CNA_HAPTIC_FEATURE_FRICTION: CNA_HapticFeature = 0x0000_0400;
pub const CNA_HAPTIC_FEATURE_LEFT_RIGHT: CNA_HapticFeature = 0x0000_0800;
pub const CNA_HAPTIC_FEATURE_CUSTOM: CNA_HapticFeature = 0x0000_8000;
pub const CNA_HAPTIC_FEATURE_GAIN: CNA_HapticFeature = 0x0001_0000;
pub const CNA_HAPTIC_FEATURE_AUTOCENTER: CNA_HapticFeature = 0x0002_0000;
pub const CNA_HAPTIC_FEATURE_STATUS: CNA_HapticFeature = 0x0004_0000;
pub const CNA_HAPTIC_FEATURE_PAUSE: CNA_HapticFeature = 0x0008_0000;
pub const CNA_HAPTIC_FEATURE_ALL: CNA_HapticFeature = 0x000F_8FFF;
pub const CNA_SENSOR_TYPE_UNKNOWN: CNA_SensorType = 0;
pub const CNA_SENSOR_TYPE_ACCELEROMETER: CNA_SensorType = 1;
pub const CNA_SENSOR_TYPE_GYROSCOPE: CNA_SensorType = 2;
pub const CNA_SENSOR_TYPE_ACCELEROMETER_LEFT: CNA_SensorType = 3;
pub const CNA_SENSOR_TYPE_GYROSCOPE_LEFT: CNA_SensorType = 4;
pub const CNA_SENSOR_TYPE_ACCELEROMETER_RIGHT: CNA_SensorType = 5;
pub const CNA_SENSOR_TYPE_GYROSCOPE_RIGHT: CNA_SensorType = 6;
pub const CNA_SENSOR_TYPE_MAXIMUM: CNA_SensorType = CNA_SENSOR_TYPE_GYROSCOPE_RIGHT;
pub const CNA_SENSOR_STATE_NOT_SUPPORTED: CNA_SensorState = 0;
pub const CNA_SENSOR_STATE_READY: CNA_SensorState = 1;
pub const CNA_SENSOR_STATE_INITIALIZING: CNA_SensorState = 2;
pub const CNA_SENSOR_STATE_NO_DATA: CNA_SensorState = 3;
pub const CNA_SENSOR_STATE_NO_PERMISSIONS: CNA_SensorState = 4;
pub const CNA_SENSOR_STATE_DISABLED: CNA_SensorState = 5;
pub const CNA_SENSOR_STATE_MAXIMUM: CNA_SensorState = CNA_SENSOR_STATE_DISABLED;
pub const CNA_MOUSE_CURSOR_STOCK_ARROW: CNA_MouseCursorStock = 0;
pub const CNA_MOUSE_CURSOR_STOCK_CROSSHAIR: CNA_MouseCursorStock = 1;
pub const CNA_MOUSE_CURSOR_STOCK_HAND: CNA_MouseCursorStock = 2;
pub const CNA_MOUSE_CURSOR_STOCK_IBEAM: CNA_MouseCursorStock = 3;
pub const CNA_MOUSE_CURSOR_STOCK_NO: CNA_MouseCursorStock = 4;
pub const CNA_MOUSE_CURSOR_STOCK_SIZE_ALL: CNA_MouseCursorStock = 5;
pub const CNA_MOUSE_CURSOR_STOCK_SIZE_NESW: CNA_MouseCursorStock = 6;
pub const CNA_MOUSE_CURSOR_STOCK_SIZE_NS: CNA_MouseCursorStock = 7;
pub const CNA_MOUSE_CURSOR_STOCK_SIZE_NWSE: CNA_MouseCursorStock = 8;
pub const CNA_MOUSE_CURSOR_STOCK_SIZE_WE: CNA_MouseCursorStock = 9;
pub const CNA_MOUSE_CURSOR_STOCK_WAIT: CNA_MouseCursorStock = 10;
pub const CNA_MOUSE_CURSOR_STOCK_WAIT_ARROW: CNA_MouseCursorStock = 11;
pub const CNA_TEXT_INPUT_TYPE_TEXT: CNA_TextInputType = 0;
pub const CNA_TEXT_INPUT_TYPE_TEXT_NAME: CNA_TextInputType = 1;
pub const CNA_TEXT_INPUT_TYPE_TEXT_EMAIL: CNA_TextInputType = 2;
pub const CNA_TEXT_INPUT_TYPE_TEXT_USERNAME: CNA_TextInputType = 3;
pub const CNA_TEXT_INPUT_TYPE_TEXT_PASSWORD_HIDDEN: CNA_TextInputType = 4;
pub const CNA_TEXT_INPUT_TYPE_TEXT_PASSWORD_VISIBLE: CNA_TextInputType = 5;
pub const CNA_TEXT_INPUT_TYPE_NUMBER: CNA_TextInputType = 6;
pub const CNA_TEXT_INPUT_TYPE_NUMBER_PASSWORD_HIDDEN: CNA_TextInputType = 7;
pub const CNA_TEXT_INPUT_TYPE_NUMBER_PASSWORD_VISIBLE: CNA_TextInputType = 8;
pub const CNA_TEXT_INPUT_TYPE_MAXIMUM: CNA_TextInputType =
    CNA_TEXT_INPUT_TYPE_NUMBER_PASSWORD_VISIBLE;
pub const CNA_CNB_SOUND_EFFECT_INFO_STRUCT_VERSION: u32 = 1;
pub const CNA_CNB_AUDIO_FORMAT_UNKNOWN: CNA_CnbAudioFormat = 0;
pub const CNA_CNB_AUDIO_FORMAT_PCM16: CNA_CnbAudioFormat = 1;
pub const CNA_CNB_AUDIO_FORMAT_PCM8: CNA_CnbAudioFormat = 2;
pub const CNA_CNB_AUDIO_FORMAT_PCM_FLOAT32: CNA_CnbAudioFormat = 3;
pub const CNA_CNB_AUDIO_FORMAT_ADPCM: CNA_CnbAudioFormat = 4;
pub const CNA_CNB_MODEL_BONE_STRUCT_VERSION: u32 = 1;
pub const CNA_CNB_MODEL_PART_INFO_STRUCT_VERSION: u32 = 1;
pub const CNA_CNB_MATERIAL_INFO_STRUCT_VERSION: u32 = 1;
pub const CNA_CNB_MESH_INFO_STRUCT_VERSION: u32 = 1;
pub const CNA_CNB_NO_INDEX: u32 = 0xFFFF_FFFF;
pub const CNA_CNB_TEXTURE_SLOT_COUNT: u32 = 7;
pub const CNA_CNB_EFFECT_KIND_BASIC: CNA_CnbEffectKind = 0;
pub const CNA_CNB_EFFECT_KIND_SKINNED: CNA_CnbEffectKind = 1;
pub const CNA_CNB_EFFECT_KIND_DUAL_TEXTURE: CNA_CnbEffectKind = 2;
pub const CNA_CNB_EFFECT_KIND_PBR: CNA_CnbEffectKind = 3;
pub const CNA_CNB_EFFECT_KIND_SKINNED_PBR: CNA_CnbEffectKind = 4;
pub const CNA_CNB_EFFECT_KIND_EXTERNAL: CNA_CnbEffectKind = 5;
pub const CNA_CNB_EFFECT_KIND_MAXIMUM: CNA_CnbEffectKind = CNA_CNB_EFFECT_KIND_EXTERNAL;
pub const CNA_CNB_MATERIAL_TEXTURE_BASE_COLOR: CNA_CnbMaterialTextureSlot = 0;
pub const CNA_CNB_MATERIAL_TEXTURE_SECOND: CNA_CnbMaterialTextureSlot = 1;
pub const CNA_CNB_MATERIAL_TEXTURE_NORMAL: CNA_CnbMaterialTextureSlot = 2;
pub const CNA_CNB_MATERIAL_TEXTURE_METALLIC_ROUGHNESS: CNA_CnbMaterialTextureSlot = 3;
pub const CNA_CNB_MATERIAL_TEXTURE_EMISSIVE: CNA_CnbMaterialTextureSlot = 4;
pub const CNA_CNB_MATERIAL_TEXTURE_OCCLUSION: CNA_CnbMaterialTextureSlot = 5;
pub const CNA_CNB_MATERIAL_TEXTURE_SPECULAR: CNA_CnbMaterialTextureSlot = 6;
pub const CNA_CNB_MATERIAL_TEXTURE_SPECULAR_COLOR: CNA_CnbMaterialTextureSlot = 7;
pub const CNA_CNB_MATERIAL_TEXTURE_MAXIMUM: CNA_CnbMaterialTextureSlot =
    CNA_CNB_MATERIAL_TEXTURE_SPECULAR_COLOR;
pub const CNA_POWER_STATE_ERROR: CNA_PowerState = 0;
pub const CNA_POWER_STATE_UNKNOWN: CNA_PowerState = 1;
pub const CNA_POWER_STATE_ON_BATTERY: CNA_PowerState = 2;
pub const CNA_POWER_STATE_NO_BATTERY: CNA_PowerState = 3;
pub const CNA_POWER_STATE_CHARGING: CNA_PowerState = 4;
pub const CNA_POWER_STATE_CHARGED: CNA_PowerState = 5;
pub const CNA_PLATFORM_DESKTOP: CNA_Platform = 0;
pub const CNA_PLATFORM_ANDROID: CNA_Platform = 1;
pub const CNA_PLATFORM_IOS: CNA_Platform = 2;
pub const CNA_PLATFORM_WEB: CNA_Platform = 3;
pub const CNA_DESKTOP_OS_WINDOWS: CNA_DesktopOS = 0;
pub const CNA_DESKTOP_OS_LINUX: CNA_DesktopOS = 1;
pub const CNA_DESKTOP_OS_MACOSX: CNA_DesktopOS = 2;
pub const CNA_DESKTOP_OS_OTHER: CNA_DesktopOS = 3;
pub const CNA_GRAPHICS_BACKEND_CATEGORY_NATIVE: CNA_GraphicsBackendCategory = 0;
pub const CNA_GRAPHICS_BACKEND_CATEGORY_TRANSLATION_LAYER: CNA_GraphicsBackendCategory = 1;
pub const CNA_GRAPHICS_BACKEND_CATEGORY_SOFTWARE: CNA_GraphicsBackendCategory = 2;
pub const CNA_GRAPHICS_BACKEND_CATEGORY_WEB: CNA_GraphicsBackendCategory = 3;
pub const CNA_GRAPHICS_BACKEND_CATEGORY_DIAGNOSTIC: CNA_GraphicsBackendCategory = 4;
pub const CNA_GRAPHICS_BACKEND_MATURITY_PRODUCTION: CNA_GraphicsBackendMaturity = 0;
pub const CNA_GRAPHICS_BACKEND_MATURITY_SUPPORTED: CNA_GraphicsBackendMaturity = 1;
pub const CNA_GRAPHICS_BACKEND_MATURITY_EXPERIMENTAL: CNA_GraphicsBackendMaturity = 2;
pub const CNA_GRAPHICS_BACKEND_MATURITY_HISTORICAL: CNA_GraphicsBackendMaturity = 3;
pub const CNA_GRAPHICS_BACKEND_MATURITY_DEPRECATED: CNA_GraphicsBackendMaturity = 4;
pub const CNA_GRAPHICS_RENDERER_FALLBACK_NOT_COMPILED_IN: CNA_GraphicsRendererFallbackReason = 0;
pub const CNA_GRAPHICS_RENDERER_FALLBACK_PROBE_UNAVAILABLE: CNA_GraphicsRendererFallbackReason = 1;
pub const CNA_GRAPHICS_RENDERER_FALLBACK_INITIALIZATION_FAILED: CNA_GraphicsRendererFallbackReason =
    2;
pub const CNA_GRAPHICS_RENDERER_FALLBACK_WINDOW_KIND_CONFLICT: CNA_GraphicsRendererFallbackReason =
    3;
pub const CNA_DISPLAY_ORIENTATION_DEFAULT: CNA_DisplayOrientation = 0;
pub const CNA_DISPLAY_ORIENTATION_LANDSCAPE_LEFT: CNA_DisplayOrientation = 1;
pub const CNA_DISPLAY_ORIENTATION_LANDSCAPE_RIGHT: CNA_DisplayOrientation = 2;
pub const CNA_DISPLAY_ORIENTATION_PORTRAIT: CNA_DisplayOrientation = 4;
pub const CNA_GAME_EVENT_ACTIVATED: CNA_GameEvent = 0;
pub const CNA_GAME_EVENT_DEACTIVATED: CNA_GameEvent = 1;
pub const CNA_GAME_EVENT_DISPOSED: CNA_GameEvent = 2;
pub const CNA_GAME_EVENT_EXITING: CNA_GameEvent = 3;
pub const CNA_GRAPHICS_DEVICE_MANAGER_EVENT_DISPOSED: CNA_GraphicsDeviceManagerEvent = 0;
pub const CNA_GRAPHICS_DEVICE_MANAGER_EVENT_DEVICE_CREATED: CNA_GraphicsDeviceManagerEvent = 1;
pub const CNA_GRAPHICS_DEVICE_MANAGER_EVENT_DEVICE_DISPOSING: CNA_GraphicsDeviceManagerEvent = 2;
pub const CNA_GRAPHICS_DEVICE_MANAGER_EVENT_DEVICE_RESET: CNA_GraphicsDeviceManagerEvent = 3;
pub const CNA_GRAPHICS_DEVICE_MANAGER_EVENT_DEVICE_RESETTING: CNA_GraphicsDeviceManagerEvent = 4;
pub const CNA_GRAPHICS_DEVICE_MANAGER_DEFAULT_BACK_BUFFER_WIDTH: i32 = 800;
pub const CNA_GRAPHICS_DEVICE_MANAGER_DEFAULT_BACK_BUFFER_HEIGHT: i32 = 480;
pub const CNA_GAME_WINDOW_EVENT_CLIENT_SIZE_CHANGED: CNA_GameWindowEvent = 0;
pub const CNA_GAME_WINDOW_EVENT_ORIENTATION_CHANGED: CNA_GameWindowEvent = 1;
pub const CNA_GAME_WINDOW_EVENT_SCREEN_DEVICE_NAME_CHANGED: CNA_GameWindowEvent = 2;
pub const CNA_GRAPHICS_DEVICE_STATUS_NORMAL: CNA_GraphicsDeviceStatus = 0;
pub const CNA_GRAPHICS_DEVICE_STATUS_LOST: CNA_GraphicsDeviceStatus = 1;
pub const CNA_GRAPHICS_DEVICE_STATUS_NOT_RESET: CNA_GraphicsDeviceStatus = 2;
pub const CNA_GRAPHICS_PROFILE_REACH: CNA_GraphicsProfile = 0;
pub const CNA_GRAPHICS_PROFILE_HI_DEF: CNA_GraphicsProfile = 1;
pub const CNA_PRESENT_INTERVAL_DEFAULT: CNA_PresentInterval = 0;
pub const CNA_PRESENT_INTERVAL_ONE: CNA_PresentInterval = 1;
pub const CNA_PRESENT_INTERVAL_TWO: CNA_PresentInterval = 2;
pub const CNA_PRESENT_INTERVAL_IMMEDIATE: CNA_PresentInterval = 3;
pub const CNA_DEPTH_FORMAT_NONE: CNA_DepthFormat = 0;
pub const CNA_DEPTH_FORMAT_DEPTH16: CNA_DepthFormat = 1;
pub const CNA_DEPTH_FORMAT_DEPTH24: CNA_DepthFormat = 2;
pub const CNA_DEPTH_FORMAT_DEPTH24_STENCIL8: CNA_DepthFormat = 3;
pub const CNA_RENDER_TARGET_USAGE_DISCARD_CONTENTS: CNA_RenderTargetUsage = 0;
pub const CNA_RENDER_TARGET_USAGE_PRESERVE_CONTENTS: CNA_RenderTargetUsage = 1;
pub const CNA_RENDER_TARGET_USAGE_PLATFORM_CONTENTS: CNA_RenderTargetUsage = 2;
pub const CNA_CUBE_MAP_FACE_POSITIVE_X: CNA_CubeMapFace = 0;
pub const CNA_CUBE_MAP_FACE_NEGATIVE_X: CNA_CubeMapFace = 1;
pub const CNA_CUBE_MAP_FACE_POSITIVE_Y: CNA_CubeMapFace = 2;
pub const CNA_CUBE_MAP_FACE_NEGATIVE_Y: CNA_CubeMapFace = 3;
pub const CNA_CUBE_MAP_FACE_POSITIVE_Z: CNA_CubeMapFace = 4;
pub const CNA_CUBE_MAP_FACE_NEGATIVE_Z: CNA_CubeMapFace = 5;
pub const CNA_RENDER_TARGET_KIND_2D: CNA_RenderTargetKind = 1;
pub const CNA_RENDER_TARGET_KIND_CUBE: CNA_RenderTargetKind = 2;
pub const CNA_SHADER_STAGE_PIXEL: CNA_ShaderStage = 0;
pub const CNA_SHADER_STAGE_VERTEX: CNA_ShaderStage = 1;
pub const CNA_MAX_SAMPLERS: u32 = 16;
pub const CNA_TEXTURE_COLLECTION_MAX_TEXTURES: u32 = 16;
pub const CNA_EFFECT_PARAMETER_CLASS_SCALAR: CNA_EffectParameterClass = 0;
pub const CNA_EFFECT_PARAMETER_CLASS_VECTOR: CNA_EffectParameterClass = 1;
pub const CNA_EFFECT_PARAMETER_CLASS_MATRIX: CNA_EffectParameterClass = 2;
pub const CNA_EFFECT_PARAMETER_CLASS_OBJECT: CNA_EffectParameterClass = 3;
pub const CNA_EFFECT_PARAMETER_CLASS_STRUCT: CNA_EffectParameterClass = 4;
pub const CNA_EFFECT_PARAMETER_TYPE_VOID: CNA_EffectParameterType = 0;
pub const CNA_EFFECT_PARAMETER_TYPE_BOOL: CNA_EffectParameterType = 1;
pub const CNA_EFFECT_PARAMETER_TYPE_INT32: CNA_EffectParameterType = 2;
pub const CNA_EFFECT_PARAMETER_TYPE_SINGLE: CNA_EffectParameterType = 3;
pub const CNA_EFFECT_PARAMETER_TYPE_STRING: CNA_EffectParameterType = 4;
pub const CNA_EFFECT_PARAMETER_TYPE_TEXTURE: CNA_EffectParameterType = 5;
pub const CNA_EFFECT_PARAMETER_TYPE_TEXTURE1D: CNA_EffectParameterType = 6;
pub const CNA_EFFECT_PARAMETER_TYPE_TEXTURE2D: CNA_EffectParameterType = 7;
pub const CNA_EFFECT_PARAMETER_TYPE_TEXTURE3D: CNA_EffectParameterType = 8;
pub const CNA_EFFECT_PARAMETER_TYPE_TEXTURE_CUBE: CNA_EffectParameterType = 9;
pub const CNA_EFFECT_VALUE_BOOLEAN: CNA_EffectValueType = 0;
pub const CNA_EFFECT_VALUE_INT32: CNA_EffectValueType = 1;
pub const CNA_EFFECT_VALUE_SINGLE: CNA_EffectValueType = 2;
pub const CNA_EFFECT_VALUE_MATRIX: CNA_EffectValueType = 3;
pub const CNA_EFFECT_VALUE_MATRIX_TRANSPOSE: CNA_EffectValueType = 4;
pub const CNA_EFFECT_VALUE_QUATERNION: CNA_EffectValueType = 5;
pub const CNA_EFFECT_VALUE_VECTOR2: CNA_EffectValueType = 6;
pub const CNA_EFFECT_VALUE_VECTOR3: CNA_EffectValueType = 7;
pub const CNA_EFFECT_VALUE_VECTOR4: CNA_EffectValueType = 8;
pub const CNA_EFFECT_TEXTURE_BASE: CNA_EffectTextureType = 0;
pub const CNA_EFFECT_TEXTURE_2D: CNA_EffectTextureType = 1;
pub const CNA_EFFECT_TEXTURE_3D: CNA_EffectTextureType = 2;
pub const CNA_EFFECT_TEXTURE_CUBE: CNA_EffectTextureType = 3;
pub const CNA_GRAPHICS_CAPABILITY_THREE_D: CNA_GraphicsCapability = 0;
pub const CNA_GRAPHICS_CAPABILITY_DEPTH_STENCIL_BUFFER: CNA_GraphicsCapability = 1;
pub const CNA_SPRITE_SORT_MODE_DEFERRED: CNA_SpriteSortMode = 0;
pub const CNA_SPRITE_EFFECT_NONE: CNA_SpriteEffects = 0;
pub const CNA_BLEND_STATE_PRESET_DEFAULT: CNA_BlendStatePreset = 0;
pub const CNA_BLEND_STATE_PRESET_ADDITIVE: CNA_BlendStatePreset = 1;
pub const CNA_BLEND_STATE_PRESET_ALPHA_BLEND: CNA_BlendStatePreset = 2;
pub const CNA_BLEND_STATE_PRESET_NON_PREMULTIPLIED: CNA_BlendStatePreset = 3;
pub const CNA_BLEND_STATE_PRESET_OPAQUE: CNA_BlendStatePreset = 4;
pub const CNA_DEPTH_STENCIL_STATE_PRESET_DEFAULT: CNA_DepthStencilStatePreset = 0;
pub const CNA_DEPTH_STENCIL_STATE_PRESET_DEPTH_READ: CNA_DepthStencilStatePreset = 1;
pub const CNA_DEPTH_STENCIL_STATE_PRESET_NONE: CNA_DepthStencilStatePreset = 2;
pub const CNA_RASTERIZER_STATE_PRESET_DEFAULT: CNA_RasterizerStatePreset = 0;
pub const CNA_RASTERIZER_STATE_PRESET_CULL_CLOCKWISE: CNA_RasterizerStatePreset = 1;
pub const CNA_RASTERIZER_STATE_PRESET_CULL_COUNTER_CLOCKWISE: CNA_RasterizerStatePreset = 2;
pub const CNA_RASTERIZER_STATE_PRESET_CULL_NONE: CNA_RasterizerStatePreset = 3;
pub const CNA_SAMPLER_STATE_PRESET_DEFAULT: CNA_SamplerStatePreset = 0;
pub const CNA_SAMPLER_STATE_PRESET_ANISOTROPIC_CLAMP: CNA_SamplerStatePreset = 1;
pub const CNA_SAMPLER_STATE_PRESET_ANISOTROPIC_WRAP: CNA_SamplerStatePreset = 2;
pub const CNA_SAMPLER_STATE_PRESET_LINEAR_CLAMP: CNA_SamplerStatePreset = 3;
pub const CNA_SAMPLER_STATE_PRESET_LINEAR_WRAP: CNA_SamplerStatePreset = 4;
pub const CNA_SAMPLER_STATE_PRESET_POINT_CLAMP: CNA_SamplerStatePreset = 5;
pub const CNA_SAMPLER_STATE_PRESET_POINT_WRAP: CNA_SamplerStatePreset = 6;
pub const CNA_BLEND_ONE: CNA_Blend = 0;
pub const CNA_BLEND_ZERO: CNA_Blend = 1;
pub const CNA_BLEND_SOURCE_COLOR: CNA_Blend = 2;
pub const CNA_BLEND_INVERSE_SOURCE_COLOR: CNA_Blend = 3;
pub const CNA_BLEND_SOURCE_ALPHA: CNA_Blend = 4;
pub const CNA_BLEND_INVERSE_SOURCE_ALPHA: CNA_Blend = 5;
pub const CNA_BLEND_DESTINATION_COLOR: CNA_Blend = 6;
pub const CNA_BLEND_INVERSE_DESTINATION_COLOR: CNA_Blend = 7;
pub const CNA_BLEND_DESTINATION_ALPHA: CNA_Blend = 8;
pub const CNA_BLEND_INVERSE_DESTINATION_ALPHA: CNA_Blend = 9;
pub const CNA_BLEND_FACTOR: CNA_Blend = 10;
pub const CNA_BLEND_INVERSE_FACTOR: CNA_Blend = 11;
pub const CNA_BLEND_SOURCE_ALPHA_SATURATION: CNA_Blend = 12;
pub const CNA_BLEND_FUNCTION_ADD: CNA_BlendFunction = 0;
pub const CNA_BLEND_FUNCTION_SUBTRACT: CNA_BlendFunction = 1;
pub const CNA_BLEND_FUNCTION_REVERSE_SUBTRACT: CNA_BlendFunction = 2;
pub const CNA_BLEND_FUNCTION_MAX: CNA_BlendFunction = 3;
pub const CNA_BLEND_FUNCTION_MIN: CNA_BlendFunction = 4;
pub const CNA_COLOR_WRITE_NONE: CNA_ColorWriteChannels = 0;
pub const CNA_COLOR_WRITE_RED: CNA_ColorWriteChannels = 1;
pub const CNA_COLOR_WRITE_GREEN: CNA_ColorWriteChannels = 2;
pub const CNA_COLOR_WRITE_BLUE: CNA_ColorWriteChannels = 4;
pub const CNA_COLOR_WRITE_ALPHA: CNA_ColorWriteChannels = 8;
pub const CNA_COLOR_WRITE_ALL: CNA_ColorWriteChannels = 15;
pub const CNA_COMPARE_ALWAYS: CNA_CompareFunction = 0;
pub const CNA_COMPARE_NEVER: CNA_CompareFunction = 1;
pub const CNA_COMPARE_LESS: CNA_CompareFunction = 2;
pub const CNA_COMPARE_LESS_EQUAL: CNA_CompareFunction = 3;
pub const CNA_COMPARE_EQUAL: CNA_CompareFunction = 4;
pub const CNA_COMPARE_GREATER_EQUAL: CNA_CompareFunction = 5;
pub const CNA_COMPARE_GREATER: CNA_CompareFunction = 6;
pub const CNA_COMPARE_NOT_EQUAL: CNA_CompareFunction = 7;
pub const CNA_STENCIL_KEEP: CNA_StencilOperation = 0;
pub const CNA_STENCIL_ZERO: CNA_StencilOperation = 1;
pub const CNA_STENCIL_REPLACE: CNA_StencilOperation = 2;
pub const CNA_STENCIL_INCREMENT: CNA_StencilOperation = 3;
pub const CNA_STENCIL_DECREMENT: CNA_StencilOperation = 4;
pub const CNA_STENCIL_INCREMENT_SATURATION: CNA_StencilOperation = 5;
pub const CNA_STENCIL_DECREMENT_SATURATION: CNA_StencilOperation = 6;
pub const CNA_STENCIL_INVERT: CNA_StencilOperation = 7;
pub const CNA_CULL_NONE: CNA_CullMode = 0;
pub const CNA_CULL_CLOCKWISE_FACE: CNA_CullMode = 1;
pub const CNA_CULL_COUNTER_CLOCKWISE_FACE: CNA_CullMode = 2;
pub const CNA_FILL_SOLID: CNA_FillMode = 0;
pub const CNA_FILL_WIREFRAME: CNA_FillMode = 1;
pub const CNA_TEXTURE_ADDRESS_WRAP: CNA_TextureAddressMode = 0;
pub const CNA_TEXTURE_ADDRESS_CLAMP: CNA_TextureAddressMode = 1;
pub const CNA_TEXTURE_ADDRESS_MIRROR: CNA_TextureAddressMode = 2;
pub const CNA_TEXTURE_FILTER_LINEAR: CNA_TextureFilter = 0;
pub const CNA_TEXTURE_FILTER_POINT: CNA_TextureFilter = 1;
pub const CNA_TEXTURE_FILTER_ANISOTROPIC: CNA_TextureFilter = 2;
pub const CNA_AUDIO_CHANNELS_MONO: CNA_AudioChannels = 1;
pub const CNA_AUDIO_CHANNELS_STEREO: CNA_AudioChannels = 2;
pub const CNA_SOUND_STATE_PLAYING: CNA_SoundState = 0;
pub const CNA_SOUND_STATE_PAUSED: CNA_SoundState = 1;
pub const CNA_SOUND_STATE_STOPPED: CNA_SoundState = 2;
pub const CNA_AUDIO_STOP_OPTIONS_AS_AUTHORED: CNA_AudioStopOptions = 0;
pub const CNA_AUDIO_STOP_OPTIONS_IMMEDIATE: CNA_AudioStopOptions = 1;
pub const CNA_MICROPHONE_STATE_STARTED: CNA_MicrophoneState = 0;
pub const CNA_MICROPHONE_STATE_STOPPED: CNA_MicrophoneState = 1;
pub const CNA_AUDIO_ENGINE_CONTENT_VERSION: i32 = 46;
pub const CNA_TEXTURE_FILTER_LINEAR_MIP_POINT: CNA_TextureFilter = 3;
pub const CNA_TEXTURE_FILTER_POINT_MIP_LINEAR: CNA_TextureFilter = 4;
pub const CNA_TEXTURE_FILTER_MIN_LINEAR_MAG_POINT_MIP_LINEAR: CNA_TextureFilter = 5;
pub const CNA_TEXTURE_FILTER_MIN_LINEAR_MAG_POINT_MIP_POINT: CNA_TextureFilter = 6;
pub const CNA_TEXTURE_FILTER_MIN_POINT_MAG_LINEAR_MIP_LINEAR: CNA_TextureFilter = 7;
pub const CNA_TEXTURE_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT: CNA_TextureFilter = 8;
pub const CNA_TEXTURE_DATA_COLOR: CNA_TextureDataType = 0;
pub const CNA_TEXTURE_DATA_BGR565: CNA_TextureDataType = 1;
pub const CNA_TEXTURE_DATA_BGRA5551: CNA_TextureDataType = 2;
pub const CNA_TEXTURE_DATA_BGRA4444: CNA_TextureDataType = 3;
pub const CNA_TEXTURE_DATA_BYTE: CNA_TextureDataType = 4;
pub const CNA_TEXTURE_DATA_NORMALIZED_BYTE2: CNA_TextureDataType = 5;
pub const CNA_TEXTURE_DATA_NORMALIZED_BYTE4: CNA_TextureDataType = 6;
pub const CNA_TEXTURE_DATA_RGBA1010102: CNA_TextureDataType = 7;
pub const CNA_TEXTURE_DATA_RG32: CNA_TextureDataType = 8;
pub const CNA_TEXTURE_DATA_RGBA64: CNA_TextureDataType = 9;
pub const CNA_TEXTURE_DATA_ALPHA8: CNA_TextureDataType = 10;
pub const CNA_TEXTURE_DATA_SINGLE: CNA_TextureDataType = 11;
pub const CNA_TEXTURE_DATA_VECTOR2: CNA_TextureDataType = 12;
pub const CNA_TEXTURE_DATA_VECTOR4: CNA_TextureDataType = 13;
pub const CNA_TEXTURE_DATA_HALF_SINGLE: CNA_TextureDataType = 14;
pub const CNA_TEXTURE_DATA_HALF_VECTOR2: CNA_TextureDataType = 15;
pub const CNA_TEXTURE_DATA_HALF_VECTOR4: CNA_TextureDataType = 16;
pub const CNA_TEXTURE_DATA_USHORT: CNA_TextureDataType = 17;
pub const CNA_TEXTURE_IMAGE_FORMAT_PNG: CNA_TextureImageFormat = 0;
pub const CNA_TEXTURE_IMAGE_FORMAT_JPEG: CNA_TextureImageFormat = 1;
pub const CNA_BUFFER_USAGE_NONE: CNA_BufferUsage = 0;
pub const CNA_BUFFER_USAGE_WRITE_ONLY: CNA_BufferUsage = 1;
pub const CNA_INDEX_ELEMENT_SIZE_SIXTEEN_BITS: CNA_IndexElementSize = 0;
pub const CNA_INDEX_ELEMENT_SIZE_THIRTY_TWO_BITS: CNA_IndexElementSize = 1;
pub const CNA_CLEAR_OPTION_TARGET: CNA_ClearOptions = 1;
pub const CNA_CLEAR_OPTION_DEPTH_BUFFER: CNA_ClearOptions = 2;
pub const CNA_CLEAR_OPTION_STENCIL: CNA_ClearOptions = 4;
pub const CNA_SET_DATA_NONE: CNA_SetDataOptions = 0;
pub const CNA_SET_DATA_DISCARD: CNA_SetDataOptions = 1;
pub const CNA_SET_DATA_NO_OVERWRITE: CNA_SetDataOptions = 2;
pub const CNA_PRIMITIVE_TRIANGLE_LIST: CNA_PrimitiveType = 0;
pub const CNA_PRIMITIVE_TRIANGLE_STRIP: CNA_PrimitiveType = 1;
pub const CNA_PRIMITIVE_LINE_LIST: CNA_PrimitiveType = 2;
pub const CNA_PRIMITIVE_LINE_STRIP: CNA_PrimitiveType = 3;
pub const CNA_USER_VERTEX_SOURCE_RAW_STREAM: CNA_UserVertexSource = 0;
pub const CNA_VERTEX_ELEMENT_FORMAT_SINGLE: CNA_VertexElementFormat = 0;
pub const CNA_VERTEX_ELEMENT_FORMAT_VECTOR2: CNA_VertexElementFormat = 1;
pub const CNA_VERTEX_ELEMENT_FORMAT_VECTOR3: CNA_VertexElementFormat = 2;
pub const CNA_VERTEX_ELEMENT_FORMAT_VECTOR4: CNA_VertexElementFormat = 3;
pub const CNA_VERTEX_ELEMENT_FORMAT_COLOR: CNA_VertexElementFormat = 4;
pub const CNA_VERTEX_ELEMENT_FORMAT_BYTE4: CNA_VertexElementFormat = 5;
pub const CNA_VERTEX_ELEMENT_FORMAT_SHORT2: CNA_VertexElementFormat = 6;
pub const CNA_VERTEX_ELEMENT_FORMAT_SHORT4: CNA_VertexElementFormat = 7;
pub const CNA_VERTEX_ELEMENT_FORMAT_NORMALIZED_SHORT2: CNA_VertexElementFormat = 8;
pub const CNA_VERTEX_ELEMENT_FORMAT_NORMALIZED_SHORT4: CNA_VertexElementFormat = 9;
pub const CNA_VERTEX_ELEMENT_FORMAT_HALF_VECTOR2: CNA_VertexElementFormat = 10;
pub const CNA_VERTEX_ELEMENT_FORMAT_HALF_VECTOR4: CNA_VertexElementFormat = 11;
pub const CNA_VERTEX_ELEMENT_USAGE_POSITION: CNA_VertexElementUsage = 0;
pub const CNA_VERTEX_ELEMENT_USAGE_COLOR: CNA_VertexElementUsage = 1;
pub const CNA_VERTEX_ELEMENT_USAGE_TEXTURE_COORDINATE: CNA_VertexElementUsage = 2;
pub const CNA_VERTEX_ELEMENT_USAGE_NORMAL: CNA_VertexElementUsage = 3;
pub const CNA_VERTEX_ELEMENT_USAGE_BINORMAL: CNA_VertexElementUsage = 4;
pub const CNA_VERTEX_ELEMENT_USAGE_TANGENT: CNA_VertexElementUsage = 5;
pub const CNA_VERTEX_ELEMENT_USAGE_BLEND_INDICES: CNA_VertexElementUsage = 6;
pub const CNA_VERTEX_ELEMENT_USAGE_BLEND_WEIGHT: CNA_VertexElementUsage = 7;
pub const CNA_VERTEX_ELEMENT_USAGE_DEPTH: CNA_VertexElementUsage = 8;
pub const CNA_VERTEX_ELEMENT_USAGE_FOG: CNA_VertexElementUsage = 9;
pub const CNA_VERTEX_ELEMENT_USAGE_POINT_SIZE: CNA_VertexElementUsage = 10;
pub const CNA_VERTEX_ELEMENT_USAGE_SAMPLE: CNA_VertexElementUsage = 11;
pub const CNA_VERTEX_ELEMENT_USAGE_TESSELLATE_FACTOR: CNA_VertexElementUsage = 12;
pub const CNA_VERTEX_TYPE_POSITION_COLOR: CNA_VertexType = 0;
pub const CNA_VERTEX_TYPE_POSITION_COLOR_TEXTURE: CNA_VertexType = 1;
pub const CNA_VERTEX_TYPE_POSITION_NORMAL_TEXTURE: CNA_VertexType = 4;
pub const CNA_VERTEX_TYPE_POSITION_TEXTURE: CNA_VertexType = 6;
pub const CNA_KEY_ESCAPE: CNA_Key = 27;
pub const CNA_MOUSE_BUTTON_LEFT: CNA_MouseButtonFlags = 1 << 0;
pub const CNA_MOUSE_BUTTON_MIDDLE: CNA_MouseButtonFlags = 1 << 1;
pub const CNA_MOUSE_BUTTON_RIGHT: CNA_MouseButtonFlags = 1 << 2;
pub const CNA_MOUSE_BUTTON_X1: CNA_MouseButtonFlags = 1 << 3;
pub const CNA_MOUSE_BUTTON_X2: CNA_MouseButtonFlags = 1 << 4;
pub const CNA_GAMEPAD_TYPE_UNKNOWN: CNA_GamePadType = 0;
pub const CNA_GAMEPAD_TYPE_GAMEPAD: CNA_GamePadType = 1;
pub const CNA_GAMEPAD_TYPE_WHEEL: CNA_GamePadType = 2;
pub const CNA_GAMEPAD_TYPE_ARCADE_STICK: CNA_GamePadType = 3;
pub const CNA_GAMEPAD_TYPE_FLIGHT_STICK: CNA_GamePadType = 4;
pub const CNA_GAMEPAD_TYPE_DANCE_PAD: CNA_GamePadType = 5;
pub const CNA_GAMEPAD_TYPE_GUITAR: CNA_GamePadType = 6;
pub const CNA_GAMEPAD_TYPE_ALTERNATE_GUITAR: CNA_GamePadType = 7;
pub const CNA_GAMEPAD_TYPE_DRUM_KIT: CNA_GamePadType = 8;
pub const CNA_GAMEPAD_TYPE_BIG_BUTTON_PAD: CNA_GamePadType = 9;
pub const CNA_PLAYER_INDEX_ONE: CNA_PlayerIndex = 0;
pub const CNA_PLAYER_INDEX_TWO: CNA_PlayerIndex = 1;
pub const CNA_PLAYER_INDEX_THREE: CNA_PlayerIndex = 2;
pub const CNA_PLAYER_INDEX_FOUR: CNA_PlayerIndex = 3;
pub const CNA_GAMEPAD_DEAD_ZONE_NONE: CNA_GamePadDeadZone = 0;
pub const CNA_GAMEPAD_DEAD_ZONE_INDEPENDENT_AXES: CNA_GamePadDeadZone = 1;
pub const CNA_GAMEPAD_DEAD_ZONE_CIRCULAR: CNA_GamePadDeadZone = 2;
pub const CNA_GAMEPAD_BUTTON_A: CNA_GamePadButtonFlags = 0x0000_1000;
pub const CNA_GAMEPAD_BUTTON_LEFT_TRIGGER: CNA_GamePadButtonFlags = 0x0080_0000;
pub const CNA_GAMEPAD_BUTTON_LEFT_THUMBSTICK_RIGHT: CNA_GamePadButtonFlags = 0x4000_0000;
pub const CNA_GAMEPAD_BUTTON_ALL: CNA_GamePadButtonFlags = 0x7fff_ffff;
pub const CNA_GESTURE_TYPE_NONE: CNA_GestureType = 0;
pub const CNA_GESTURE_TYPE_TAP: CNA_GestureType = 1;
pub const CNA_GESTURE_TYPE_DOUBLE_TAP: CNA_GestureType = 2;
pub const CNA_GESTURE_TYPE_HOLD: CNA_GestureType = 4;
pub const CNA_GESTURE_TYPE_HORIZONTAL_DRAG: CNA_GestureType = 8;
pub const CNA_GESTURE_TYPE_VERTICAL_DRAG: CNA_GestureType = 16;
pub const CNA_GESTURE_TYPE_FREE_DRAG: CNA_GestureType = 32;
pub const CNA_GESTURE_TYPE_PINCH: CNA_GestureType = 64;
pub const CNA_GESTURE_TYPE_FLICK: CNA_GestureType = 128;
pub const CNA_GESTURE_TYPE_DRAG_COMPLETE: CNA_GestureType = 256;
pub const CNA_GESTURE_TYPE_PINCH_COMPLETE: CNA_GestureType = 512;
pub const CNA_GESTURE_TYPE_ALL: CNA_GestureType = 0x0000_03ff;
pub const CNA_FILE_MODE_CREATE_NEW: CNA_FileMode = 1;
pub const CNA_FILE_MODE_CREATE: CNA_FileMode = 2;
pub const CNA_FILE_MODE_OPEN: CNA_FileMode = 3;
pub const CNA_FILE_MODE_OPEN_OR_CREATE: CNA_FileMode = 4;
pub const CNA_FILE_MODE_TRUNCATE: CNA_FileMode = 5;
pub const CNA_FILE_MODE_APPEND: CNA_FileMode = 6;
pub const CNA_FILE_ACCESS_READ: CNA_FileAccess = 1;
pub const CNA_FILE_ACCESS_WRITE: CNA_FileAccess = 2;
pub const CNA_FILE_ACCESS_READ_WRITE: CNA_FileAccess = 3;
pub const CNA_FILE_SHARE_NONE: CNA_FileShare = 0;
pub const CNA_FILE_SHARE_READ: CNA_FileShare = 1;
pub const CNA_FILE_SHARE_WRITE: CNA_FileShare = 2;
pub const CNA_FILE_SHARE_READ_WRITE: CNA_FileShare = 3;
pub const CNA_FILE_SHARE_DELETE: CNA_FileShare = 4;
pub const CNA_FILE_SHARE_INHERITABLE: CNA_FileShare = 16;
pub const CNA_SEEK_ORIGIN_BEGIN: CNA_SeekOrigin = 0;
pub const CNA_SEEK_ORIGIN_CURRENT: CNA_SeekOrigin = 1;
pub const CNA_SEEK_ORIGIN_END: CNA_SeekOrigin = 2;

pub type CNA_Result = u32;
pub type CNA_Bool = u8;
pub type CNA_DisplayOrientation = u32;
pub type CNA_GameEvent = u32;
pub type CNA_GameWindowEvent = u32;
pub type CNA_GraphicsDeviceStatus = u32;
pub type CNA_GraphicsProfile = u32;
pub type CNA_GraphicsDeviceManagerEvent = u32;
pub type CNA_PresentInterval = u32;
pub type CNA_DepthFormat = u32;
pub type CNA_RenderTargetUsage = u32;
pub type CNA_CubeMapFace = u32;
pub type CNA_RenderTargetKind = u32;
pub type CNA_ShaderStage = u32;
pub type CNA_NativeHandleValue = u64;
pub type CNA_Handle = u64;
pub type CNA_GraphicsDeviceManagerHandle = CNA_Handle;
pub type CNA_GameEventRegistrationHandle = CNA_Handle;
pub type CNA_VertexDeclarationHandle = CNA_Handle;
pub type CNA_VertexBufferHandle = CNA_Handle;
pub type CNA_IndexBufferHandle = CNA_Handle;
pub type CNA_OcclusionQueryHandle = CNA_Handle;
pub type CNA_EffectHandle = CNA_Handle;
pub type CNA_DirectionalLightHandle = CNA_Handle;
pub type CNA_EffectAnnotationHandle = CNA_Handle;
pub type CNA_EffectAnnotationCollectionHandle = CNA_Handle;
pub type CNA_EffectParameterHandle = CNA_Handle;
pub type CNA_EffectParameterCollectionHandle = CNA_Handle;
pub type CNA_EffectPassHandle = CNA_Handle;
pub type CNA_EffectPassCollectionHandle = CNA_Handle;
pub type CNA_EffectTechniqueHandle = CNA_Handle;
pub type CNA_EffectTechniqueCollectionHandle = CNA_Handle;
pub type CNA_ErrorCategory = u32;
pub type CNA_GraphicsCapability = u32;
pub type CNA_GraphicsCapabilityFlags = u64;
pub type CNA_GraphicsRendererType = u32;
pub type CNA_GraphicsBackendCategory = u32;
pub type CNA_GraphicsBackendMaturity = u32;
pub type CNA_GraphicsRendererFallbackReason = u32;
pub type CNA_JoystickStateHandle = CNA_Handle;
pub type CNA_JoystickType = u32;
pub type CNA_JoystickHatPosition = u32;
pub type CNA_AsciiPostProcessEffectHandle = CNA_Handle;
pub type CNA_AsciiQuantizeMode = u32;
pub type CNA_CRTMaskType = u32;
pub type CNA_DitherMode = u32;
pub type CNA_DepthEffectMode = u32;
pub type CNA_CnbChunkId = u32;
pub type CNA_CnbByteWriterHandle = CNA_Handle;
pub type CNA_CnbAnimationClipHandle = CNA_Handle;
pub type CNA_CurveHandle = CNA_Handle;
pub type CNA_SystemTrayHandle = CNA_Handle;

pub type CNA_DeviceType = u32;

pub const CNA_DEVICE_TYPE_DEVICE: CNA_DeviceType = 0;
pub const CNA_DEVICE_TYPE_EMULATOR: CNA_DeviceType = 1;

pub type CNA_MessageBoxType = u32;

pub const CNA_MESSAGE_BOX_TYPE_ERROR: CNA_MessageBoxType = 0;
pub const CNA_MESSAGE_BOX_TYPE_WARNING: CNA_MessageBoxType = 1;
pub const CNA_MESSAGE_BOX_TYPE_INFORMATION: CNA_MessageBoxType = 2;

/// The files a dialog returned, borrowed for the duration of the call.
pub type CNA_FileDialogResultCallback = Option<
    unsafe extern "C" fn(files: *const CNA_StringView, count: u64, context: *mut c_void),
>;

/// One tray entry was clicked.
pub type CNA_TrayEntryClickCallback = Option<unsafe extern "C" fn(context: *mut c_void)>;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_FileDialogFilter {
    pub struct_size: u32,
    pub struct_version: u32,
    pub name: CNA_StringView,
    pub pattern: CNA_StringView,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_MessageBoxTestLog {
    pub struct_size: u32,
    pub struct_version: u32,
    pub simple_calls: u32,
    pub choice_calls: u32,
    pub last_type: CNA_MessageBoxType,
    pub last_button_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_VibrationTestLog {
    pub struct_size: u32,
    pub struct_version: u32,
    pub start_calls: u32,
    pub stop_calls: u32,
    pub left_right_calls: u32,
    pub reserved: u32,
    pub last_duration_ticks: i64,
    pub last_intensity: f32,
    pub last_large_motor: f32,
    pub last_small_motor: f32,
    pub reserved_float: f32,
}
pub type CNA_ObjectDictionaryHandle = CNA_Handle;
pub type CNA_ObjectDictionaryValueKind = u32;

pub const CNA_OBJECT_DICTIONARY_VALUE_UNKNOWN: CNA_ObjectDictionaryValueKind = 0;
pub const CNA_OBJECT_DICTIONARY_VALUE_BOOLEAN: CNA_ObjectDictionaryValueKind = 1;
pub const CNA_OBJECT_DICTIONARY_VALUE_INT32: CNA_ObjectDictionaryValueKind = 2;
pub const CNA_OBJECT_DICTIONARY_VALUE_SINGLE: CNA_ObjectDictionaryValueKind = 3;
pub const CNA_OBJECT_DICTIONARY_VALUE_DOUBLE: CNA_ObjectDictionaryValueKind = 4;
pub const CNA_OBJECT_DICTIONARY_VALUE_STRING: CNA_ObjectDictionaryValueKind = 5;
pub const CNA_OBJECT_DICTIONARY_VALUE_VECTOR2: CNA_ObjectDictionaryValueKind = 6;
pub const CNA_OBJECT_DICTIONARY_VALUE_VECTOR3: CNA_ObjectDictionaryValueKind = 7;
pub const CNA_OBJECT_DICTIONARY_VALUE_VECTOR4: CNA_ObjectDictionaryValueKind = 8;
pub const CNA_OBJECT_DICTIONARY_VALUE_MATRIX: CNA_ObjectDictionaryValueKind = 9;
pub const CNA_OBJECT_DICTIONARY_VALUE_QUATERNION: CNA_ObjectDictionaryValueKind = 10;
pub const CNA_OBJECT_DICTIONARY_VALUE_COLOR: CNA_ObjectDictionaryValueKind = 11;
pub const CNA_OBJECT_DICTIONARY_VALUE_BOUNDING_SPHERE: CNA_ObjectDictionaryValueKind = 12;
pub const CNA_OBJECT_DICTIONARY_VALUE_BOUNDING_BOX: CNA_ObjectDictionaryValueKind = 13;
pub const CNA_OBJECT_DICTIONARY_VALUE_FOREIGN_OBJECT: CNA_ObjectDictionaryValueKind = 14;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_ObjectDictionaryEntry {
    pub struct_size: u32,
    pub struct_version: u32,
    pub kind: CNA_ObjectDictionaryValueKind,
    pub is_array: CNA_Bool,
    pub element_count: u64,
}

/// A colour transform, row-major, as `ColorMatrixEffect` stores it.
///
/// Sixteen floats rather than a `CNA_Matrix`: it multiplies RGBA, not
/// positions, and giving it the geometry type would invite it to be passed
/// where a world transform belongs.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_ColorMatrix4x4 {
    pub values: [f32; 16],
}

impl Default for CNA_ColorMatrix4x4 {
    fn default() -> Self {
        Self { values: [0.0; 16] }
    }
}
pub type CNA_CurveKeyCollectionHandle = CNA_Handle;
pub type CNA_CurveLoopType = u32;
pub type CNA_CurveContinuity = u32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CurveKey {
    pub position: f32,
    pub value: f32,
    pub tangent_in: f32,
    pub tangent_out: f32,
    pub continuity: CNA_CurveContinuity,
}

/// A caller's predicate over the representations a texture carries.
///
/// Called synchronously, once per representation in order, and never retained
/// past the call it was passed to. It must not call back into this ABI.
pub type CNA_CnbTextureFormatSupportedFn =
    Option<unsafe extern "C" fn(format: CNA_CnbTextureFormat, context: *mut c_void) -> CNA_Bool>;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbImageImportOptions {
    pub struct_size: u32,
    pub struct_version: u32,
    pub color_key: [u8; 3],
    pub has_color_key: CNA_Bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbVideoInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub duration_milliseconds: u32,
    pub width: u32,
    pub height: u32,
    pub frames_per_second: f32,
    pub soundtrack_type: CNA_VideoSoundtrackType,
    pub reserved: u32,
}
pub type CNA_CnbModelFromCnjHandle = CNA_Handle;
pub type CNA_CnjToCnbResultHandle = CNA_Handle;

pub type CNA_CnbMorphDeltaStream = u32;

pub const CNA_CNB_MORPH_DELTA_POSITION: CNA_CnbMorphDeltaStream = 0;
pub const CNA_CNB_MORPH_DELTA_NORMAL: CNA_CnbMorphDeltaStream = 1;
pub const CNA_CNB_MORPH_DELTA_TANGENT: CNA_CnbMorphDeltaStream = 2;

pub type CNA_CnbMorphKeyStream = u32;

pub const CNA_CNB_MORPH_KEY_WEIGHTS: CNA_CnbMorphKeyStream = 0;
pub const CNA_CNB_MORPH_KEY_IN_TANGENT: CNA_CnbMorphKeyStream = 1;
pub const CNA_CNB_MORPH_KEY_OUT_TANGENT: CNA_CnbMorphKeyStream = 2;

pub type CNA_CnbSkeletonMatrixSet = u32;

pub const CNA_CNB_SKELETON_MATRIX_BIND_POSE: CNA_CnbSkeletonMatrixSet = 0;
pub const CNA_CNB_SKELETON_MATRIX_INVERSE_BIND_POSE: CNA_CnbSkeletonMatrixSet = 1;
pub const CNA_CNB_SKELETON_MATRIX_ROOT_PREFIX: CNA_CnbSkeletonMatrixSet = 2;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbSkeletonInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub joint_count: u64,
    pub has_root_prefix: CNA_Bool,
    pub reserved: [u8; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbMorphInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub vertex_count: u32,
    pub reserved: u32,
    pub target_count: u64,
    pub weight_count: u64,
    pub weight_track_key_count: u64,
    pub recompute_flat_normals: CNA_Bool,
    pub weight_track_step_interpolation: CNA_Bool,
    pub weight_track_cubic_spline: CNA_Bool,
    pub reserved2: [u8; 5],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbMorphWeightKeyInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub time_seconds: f64,
    pub weight_count: u64,
    pub in_tangent_count: u64,
    pub out_tangent_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbModelLight {
    pub direction: [f32; 3],
    pub diffuse_color: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbSamplerState {
    pub filter: u32,
    pub address_u: u32,
    pub address_v: u32,
    pub declared: CNA_Bool,
    pub reserved: [u8; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbTextureTransform {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
}
pub type CNA_CnbReaderHandle = CNA_Handle;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbChunkEntry {
    pub struct_size: u32,
    pub struct_version: u32,
    pub offset: u64,
    pub stored_size: u64,
    pub uncompressed_size: u64,
    pub r#type: CNA_CnbChunkId,
    pub flags: u32,
    pub checksum: u32,
    pub compression: CNA_CnbCompression,
    pub alignment: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbExternalReference {
    pub struct_size: u32,
    pub struct_version: u32,
    pub flags: u32,
    pub expected_asset_type_id: u32,
}
pub type CNA_CnbCompression = u32;
pub type CNA_CnbTextureFormat = u32;
pub type CNA_CnbDocumentHandle = CNA_Handle;
pub type CNA_CnbTextureDataHandle = CNA_Handle;
pub type CNA_CnbModelDataHandle = CNA_Handle;
pub type CNA_CnbLoaderHandle = CNA_Handle;
pub type CNA_CnbWriterHandle = CNA_Handle;
pub type CNA_TransparencyMode = u32;
pub type CNA_AlphaModeEXT = u32;
pub type CNA_TonemappingMode = u32;
pub type CNA_RenderQuality = u32;
pub type CNA_ShadowQuality = u32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_RenderPipelineSettingsEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub hdr_enabled: CNA_Bool,
    pub exposure: f32,
    pub gamma: f32,
    pub tonemapping_mode: CNA_TonemappingMode,
    pub bloom_enabled: CNA_Bool,
    pub bloom_intensity: f32,
    pub bloom_threshold: f32,
    pub bloom_iterations: i32,
    pub ssao_enabled: CNA_Bool,
    pub transparency_mode: CNA_TransparencyMode,
    pub ssao_radius: f32,
    pub ssao_intensity: f32,
    pub ssao_sample_count: i32,
    pub ssr_enabled: CNA_Bool,
    pub ssr_max_distance: f32,
    pub ssr_step_count: i32,
    pub ssr_thickness: f32,
    pub ssr_depth_bias: f32,
    pub ssr_edge_fade: f32,
    pub volumetric_fog_density: f32,
    pub light_shaft_threshold: f32,
    pub light_shaft_intensity: f32,
    pub light_shaft_decay: f32,
    pub height_fog_density: f32,
    pub height_fog_falloff: f32,
    pub height_fog_base_height: f32,
    pub motion_blur_strength: f32,
    pub motion_blur_max_distance: f32,
    pub chromatic_aberration_strength: f32,
    pub film_grain_intensity: f32,
    pub lens_flare_threshold: f32,
    pub lens_flare_intensity: f32,
    pub lens_flare_dispersal: f32,
    pub color_grade_enabled: CNA_Bool,
    pub color_grade_strength: f32,
    pub dof_enabled: CNA_Bool,
    pub dof_focus_distance: f32,
    pub dof_focal_length: f32,
    pub doff_number: f32,
    pub dof_max_radius: f32,
    pub ssr_roughness_blur: f32,
    pub ssr_intensity: f32,
    pub fxaa_enabled: CNA_Bool,
    pub fxaa_edge_threshold_ext: f32,
    pub render_quality: CNA_RenderQuality,
    pub shadow_quality: CNA_ShadowQuality,
    pub shadows_enabled: CNA_Bool,
    pub reserved: [u8; 4],
}

pub type CNA_RenderPipelineHandle = CNA_Handle;
pub type CNA_ShadowMapHandle = CNA_Handle;
pub type CNA_PostProcessPassHandle = CNA_Handle;
pub type CNA_PostProcessChainHandle = CNA_Handle;
pub type CNA_RenderTargetPoolHandle = CNA_Handle;
pub type CNA_GpuTimerHandle = CNA_Handle;
pub type CNA_ParticleSystemHandle = CNA_Handle;
pub type CNA_StorageBufferHandle = CNA_Handle;
pub type CNA_ComputeShaderHandle = CNA_Handle;
pub type CNA_DecalPassHandle = CNA_Handle;
pub type CNA_SkyboxHandle = CNA_Handle;
pub type CNA_AtmosphericSkyHandle = CNA_Handle;
pub type CNA_FullscreenPassHandle = CNA_Handle;
pub type CNA_ScopedRenderTargetHandle = CNA_Handle;
pub type CNA_SpatialUpscalePassHandle = CNA_Handle;
pub type CNA_SpotShadowMapHandle = CNA_Handle;
pub type CNA_CubeShadowMapHandle = CNA_Handle;
pub type CNA_CascadedShadowMapHandle = CNA_Handle;
pub type CNA_DepthNormalPrepassHandle = CNA_Handle;
pub type CNA_TransparentDrawListHandle = CNA_Handle;
pub type CNA_WeightedBlendedTransparencyHandle = CNA_Handle;
pub type CNA_HdrDisplayOutputHandle = CNA_Handle;
pub type CNA_AutoExposureHandle = CNA_Handle;
pub type CNA_CubeLutHandle = CNA_Handle;
pub type CNA_DebugDrawHandle = CNA_Handle;
pub type CNA_FrustumCullerEXTHandle = CNA_Handle;
pub type CNA_LodGroupEXTHandle = CNA_Handle;
pub type CNA_ModelMeshPartHandle = CNA_Handle;
pub type CNA_ModelHandle = CNA_Handle;
pub type CNA_ModelBoneHandle = CNA_Handle;
pub type CNA_ModelBoneCollectionHandle = CNA_Handle;
pub type CNA_ModelMeshHandle = CNA_Handle;
pub type CNA_ModelMeshCollectionHandle = CNA_Handle;
pub type CNA_ModelMeshPartCollectionHandle = CNA_Handle;
pub type CNA_ModelEffectCollectionHandle = CNA_Handle;


pub type CNA_GltfImportDiagnosticSeverityEXT = u32;

pub const CNA_GLTF_IMPORT_SEVERITY_INFORMATION_EXT: CNA_GltfImportDiagnosticSeverityEXT = 0;
pub const CNA_GLTF_IMPORT_SEVERITY_WARNING_EXT: CNA_GltfImportDiagnosticSeverityEXT = 1;

pub type CNA_GltfImportDiagnosticKindEXT = u32;

pub const CNA_GLTF_IMPORT_KIND_INFORMATION_EXT: CNA_GltfImportDiagnosticKindEXT = 0;
pub const CNA_GLTF_IMPORT_KIND_GENERATED_DATA_EXT: CNA_GltfImportDiagnosticKindEXT = 1;
pub const CNA_GLTF_IMPORT_KIND_INVALID_SOURCE_DATA_EXT: CNA_GltfImportDiagnosticKindEXT = 2;
pub const CNA_GLTF_IMPORT_KIND_APPROXIMATION_EXT: CNA_GltfImportDiagnosticKindEXT = 3;
pub const CNA_GLTF_IMPORT_KIND_DROPPED_DATA_EXT: CNA_GltfImportDiagnosticKindEXT = 4;
pub const CNA_GLTF_IMPORT_KIND_UNSUPPORTED_FEATURE_EXT: CNA_GltfImportDiagnosticKindEXT = 5;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GltfImportReportEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub node_count: u64,
    pub mesh_instance_count: u64,
    pub distinct_mesh_count: u64,
    pub shared_mesh_count: u64,
    pub max_node_depth: u64,
    pub camera_node_count: u64,
    pub light_node_count: u64,
    pub imported_light_count: u64,
    pub primitive_count: u64,
    pub skin_count: u64,
    pub animation_count: u64,
    pub clip_count: u64,
    pub diagnostic_count: u64,
    pub warning_count: u64,
    pub dropped_feature_count: u64,
    pub approximation_count: u64,
    pub anything_lost: CNA_Bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GltfImportDiagnosticEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub severity: CNA_GltfImportDiagnosticSeverityEXT,
    pub kind: CNA_GltfImportDiagnosticKindEXT,
    pub count: u64,
    pub worst_magnitude: f64,
    pub detail_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_GltfImportDiagnosticDescriptorEXT {
    pub code: CNA_StringView,
    pub severity: CNA_GltfImportDiagnosticSeverityEXT,
    pub kind: CNA_GltfImportDiagnosticKindEXT,
    pub subject: CNA_StringView,
    pub count: u64,
    pub worst_magnitude: f64,
    pub details: *const CNA_StringView,
    pub detail_count: u64,
    pub message: CNA_StringView,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_ModelCameraEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub scene_node_index: i32,
    pub is_perspective: CNA_Bool,
    pub has_infinite_far_plane: CNA_Bool,
    pub has_authored_aspect_ratio: CNA_Bool,
    pub projection: CNA_Matrix,
    pub world_transform: CNA_Matrix,
    pub aspect_ratio: f32,
    pub field_of_view: f32,
    pub near_plane_distance: f32,
    pub far_plane_distance: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_ModelCameraDescriptorEXT {
    pub name: CNA_StringView,
    pub camera: CNA_ModelCameraEXT,
}
pub type CNA_LodSelectionMode = u32;

pub const CNA_LOD_SELECTION_MODE_DISTANCE: CNA_LodSelectionMode = 0;
pub const CNA_LOD_SELECTION_MODE_SCREEN_SPACE_ERROR: CNA_LodSelectionMode = 1;
pub type CNA_ClusteredLightSetHandle = CNA_Handle;
pub type CNA_ClusteredLightGridHandle = CNA_Handle;
pub type CNA_ClusteredLightAssignmentHandle = CNA_Handle;
pub type CNA_ClusteredLightType = u32;

pub const CNA_CLUSTERED_LIGHT_TYPE_POINT: CNA_ClusteredLightType = 0;
pub const CNA_CLUSTERED_LIGHT_TYPE_SPOT: CNA_ClusteredLightType = 1;

pub const CNA_CLUSTERED_LIGHT_SET_MAX_EXT: i32 = 256;
pub const CNA_CLUSTER_GRID_MAX_TILES_PER_AXIS_EXT: i32 = 128;
pub const CNA_CLUSTER_GRID_MAX_SLICE_COUNT_EXT: i32 = 256;
pub const CNA_CLUSTER_GRID_DEFAULT_TILES_X_EXT: i32 = 16;
pub const CNA_CLUSTER_GRID_DEFAULT_TILES_Y_EXT: i32 = 8;
pub const CNA_CLUSTER_GRID_DEFAULT_SLICE_COUNT_EXT: i32 = 24;
pub const CNA_CLUSTERED_ASSIGNMENT_MAX_LIGHTS_EXT: i32 = 1024;
pub const CNA_CLUSTERED_SHADOW_DEFAULT_BUDGET_EXT: i32 = 4;
pub const CNA_CLUSTERED_SHADOW_DEFAULT_HYSTERESIS_EXT: f32 = 1.25;
pub const CNA_CLUSTERED_COMPUTE_DEFAULT_STRIDE_EXT: i32 = 64;

pub type CNA_ClusteredShadowPolicyHandle = CNA_Handle;
pub type CNA_ClusteredLightBufferHandle = CNA_Handle;
pub type CNA_ClusteredLightComputeHandle = CNA_Handle;
pub type CNA_ClusteredForwardEffectHandle = CNA_Handle;
pub type CNA_LightProbeHandle = CNA_Handle;
pub type CNA_EnvironmentProcessorHandle = CNA_Handle;
pub type CNA_LightProbeVolumeHandle = CNA_Handle;
pub type CNA_LightProbeBakerHandle = CNA_Handle;
pub type CNA_ShaderEffectFactoryHandle = CNA_Handle;
pub type CNA_AreaLightBrdfTableHandle = CNA_Handle;
pub type CNA_GpuInstanceCullerHandle = CNA_Handle;
pub type CNA_InstancedRendererEXTHandle = CNA_Handle;
pub type CNA_SkinnedModelEXTHandle = CNA_Handle;
pub type CNA_SkinningDataHandle = CNA_Handle;
pub type CNA_AnimationPlayerHandle = CNA_Handle;
pub type CNA_MorphTargetDataEXTHandle = CNA_Handle;
pub type CNA_ModelAnimationsEXTHandle = CNA_Handle;
pub type CNA_PbrTextureSlot = u32;
pub type CNA_CameraHandle = CNA_Handle;
pub type CNA_CameraState = u32;
pub type CNA_CameraPosition = u32;

pub const CNA_CAMERA_STATE_NOT_SUPPORTED: CNA_CameraState = 0;
pub const CNA_CAMERA_STATE_CLOSED: CNA_CameraState = 1;
pub const CNA_CAMERA_STATE_OPENING: CNA_CameraState = 2;
pub const CNA_CAMERA_STATE_DENIED: CNA_CameraState = 3;
pub const CNA_CAMERA_STATE_READY: CNA_CameraState = 4;
pub const CNA_CAMERA_POSITION_UNKNOWN: CNA_CameraPosition = 0;
pub const CNA_CAMERA_POSITION_FRONT_FACING: CNA_CameraPosition = 1;
pub const CNA_CAMERA_POSITION_BACK_FACING: CNA_CameraPosition = 2;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_CameraDeviceInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub position: CNA_CameraPosition,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_MorphTargetDeltaEXTDescriptor {
    pub position_deltas: *const CNA_Vector3,
    pub position_delta_count: u64,
    pub normal_deltas: *const CNA_Vector3,
    pub normal_delta_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_MorphWeightKeyframeEXTDescriptor {
    pub time_seconds: f64,
    pub weights: *const f32,
    pub weight_count: u64,
    pub in_tangents: *const f32,
    pub in_tangent_count: u64,
    pub out_tangents: *const f32,
    pub out_tangent_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_MorphWeightTrackEXTDescriptor {
    pub keyframes: *const CNA_MorphWeightKeyframeEXTDescriptor,
    pub keyframe_count: u64,
    pub step_interpolation: CNA_Bool,
    pub cubic_spline: CNA_Bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_MorphTargetDataEXTDescriptor {
    pub base_vertex_bytes: *const u8,
    pub base_vertex_byte_count: u64,
    pub stride: i32,
    pub targets: *const CNA_MorphTargetDeltaEXTDescriptor,
    pub target_count: u64,
    pub weights: *const f32,
    pub weight_count: u64,
    pub weight_track: CNA_MorphWeightTrackEXTDescriptor,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_SkinningDataDescriptor {
    pub bone_count: i32,
    pub reserved: u32,
    pub skeleton_hierarchy: *const i32,
    pub bind_pose: *const CNA_Matrix,
    pub inverse_bind_pose: *const CNA_Matrix,
    pub skeleton_root_prefix: *const CNA_Matrix,
    pub skeleton_root_prefix_count: u64,
    pub clips: *const CNA_NamedAnimationClipEXTDescriptor,
    pub clip_count: u64,
}
pub type CNA_ClipTargetSpaceEXT = u32;

pub const CNA_CLIP_TARGET_SPACE_JOINT_PALETTE_EXT: CNA_ClipTargetSpaceEXT = 0;
pub const CNA_CLIP_TARGET_SPACE_SCENE_NODE_EXT: CNA_ClipTargetSpaceEXT = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_KeyframeEXT {
    pub time_seconds: f64,
    pub translation: CNA_Vector3,
    pub rotation: CNA_Quaternion,
    pub scale: CNA_Vector3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_BoneTrackEXTDescriptor {
    pub bone_index: i32,
    pub reserved: u32,
    pub keyframes: *const CNA_KeyframeEXT,
    pub keyframe_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_AnimationClipEXTDescriptor {
    pub duration_seconds: f64,
    pub tracks: *const CNA_BoneTrackEXTDescriptor,
    pub track_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_NamedAnimationClipEXTDescriptor {
    pub name: CNA_StringView,
    pub clip: CNA_AnimationClipEXTDescriptor,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_SkinnedModelEXTDescriptor {
    pub bone_count: i32,
    pub reserved: u32,
    pub parent_bone_indices: *const i32,
    pub bind_pose_local: *const CNA_Matrix,
    pub inverse_bind_pose_global: *const CNA_Matrix,
    pub clips: *const CNA_NamedAnimationClipEXTDescriptor,
    pub clip_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_IndirectDrawArguments {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub base_instance: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_IndirectDrawIndexedArguments {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub base_instance: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_GpuCullableInstance {
    pub struct_size: u32,
    pub struct_version: u32,
    pub world: CNA_Matrix,
    pub bounds: CNA_BoundingBox,
}
pub type CNA_AreaLightShapeEXT = u32;

pub const CNA_AREA_LIGHT_SHAPE_RECTANGLE_EXT: CNA_AreaLightShapeEXT = 0;
pub const CNA_AREA_LIGHT_SHAPE_DISC_EXT: CNA_AreaLightShapeEXT = 1;
pub const CNA_AREA_LIGHT_SHAPE_TUBE_EXT: CNA_AreaLightShapeEXT = 2;
pub const CNA_AREA_LIGHT_BRDF_TABLE_DEFAULT_SIZE: i32 = 32;
pub const CNA_AREA_LIGHT_BRDF_TABLE_DEFAULT_SAMPLE_COUNT: i32 = 64;
pub const CNA_AREA_LIGHT_QUAD_CORNER_COUNT: i32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_AreaLightEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub shape: CNA_AreaLightShapeEXT,
    pub two_sided: CNA_Bool,
    pub reserved0: [u8; 3],
    pub position: CNA_Vector3,
    pub right_axis: CNA_Vector3,
    pub up_axis: CNA_Vector3,
    pub color: CNA_Vector3,
    pub intensity: f32,
    pub range: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_AreaLightBrdfTerms {
    pub struct_size: u32,
    pub struct_version: u32,
    pub magnitude: f32,
    pub fresnel: f32,
    pub average_tangent: f32,
    pub average_normal: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GltfMaterialSourceEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub base_color_factor: CNA_Vector4,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub emissive_factor: CNA_Vector3,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub ior_ext: f32,
    pub specular_factor_ext: f32,
    pub specular_color_factor_ext: CNA_Vector3,
    pub alpha_mode: CNA_AlphaModeEXT,
    pub alpha_cutoff: f32,
    pub double_sided: CNA_Bool,
    pub reserved: [u8; 3],
    pub texture_coordinate_sets_ext: [i32; 7],
    pub texture_transforms_ext: [CNA_TextureTransformEXT; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_GltfMaterialTexturesEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub slots: [CNA_Handle; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_GltfMaterialExtensionSourceEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub clearcoat_factor_ext: f32,
    pub clearcoat_roughness_factor_ext: f32,
    pub sheen_color_factor_ext: CNA_Vector3,
    pub sheen_roughness_factor_ext: f32,
    pub transmission_factor_ext: f32,
    pub thickness_factor_ext: f32,
    pub attenuation_distance_ext: f32,
    pub attenuation_color_ext: CNA_Vector3,
    pub iridescence_factor_ext: f32,
    pub iridescence_ior_ext: f32,
    pub iridescence_thickness_minimum_ext: f32,
    pub iridescence_thickness_maximum_ext: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_GltfMaterialExtensionTexturesEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub clearcoat: CNA_Handle,
    pub clearcoat_roughness: CNA_Handle,
    pub clearcoat_normal: CNA_Handle,
    pub sheen_color: CNA_Handle,
    pub sheen_roughness: CNA_Handle,
    pub transmission: CNA_Handle,
    pub thickness: CNA_Handle,
    pub iridescence: CNA_Handle,
    pub iridescence_thickness: CNA_Handle,
}
pub type CNA_LightProbeSceneDrawCallback =
    Option<unsafe extern "C" fn(*const CNA_Matrix, *const CNA_Matrix, *mut c_void)>;

pub const CNA_LIGHT_PROBE_BAKER_DEFAULT_FACE_SIZE: i32 = 32;
pub const CNA_LIGHT_PROBE_BAKER_FACE_COUNT: i32 = 6;

pub const CNA_LIGHT_PROBE_COEFFICIENT_COUNT_EXT: i32 = 9;
pub const CNA_LIGHT_PROBE_VISIBILITY_DIRECTIONS_EXT: i32 = 6;
pub const CNA_LIGHT_PROBE_VOLUME_MAX_PROBES_EXT: i32 = 32768;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_ImageBasedLightEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub irradiance: CNA_Handle,
    pub prefiltered_specular: CNA_Handle,
    pub brdf_lut: CNA_Handle,
    pub prefiltered_mip_count: i32,
    pub intensity: f32,
}

pub const CNA_CLUSTERED_FORWARD_MAX_LIGHTS_PER_FRAGMENT_EXT: i32 = 128;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_ClusteredLightEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub r#type: CNA_ClusteredLightType,
    pub casts_shadows: CNA_Bool,
    pub reserved: [u8; 3],
    pub position: CNA_Vector3,
    pub direction: CNA_Vector3,
    pub color: CNA_Vector3,
    pub intensity: f32,
    pub range: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_LodLevelEXT {
    pub part: CNA_ModelMeshPartHandle,
    pub max_distance: f32,
    pub reserved0: u32,
}
pub type CNA_DisplayColorSpace = u32;

pub const CNA_DISPLAY_COLOR_SPACE_SRGB: CNA_DisplayColorSpace = 0;
pub const CNA_DISPLAY_COLOR_SPACE_SCRGB: CNA_DisplayColorSpace = 1;
pub const CNA_DISPLAY_COLOR_SPACE_HDR10: CNA_DisplayColorSpace = 2;
pub type CNA_DepthEncoding = u32;

pub const CNA_DEPTH_ENCODING_AUTOMATIC: CNA_DepthEncoding = 0;
pub const CNA_DEPTH_ENCODING_PACKED: CNA_DepthEncoding = 1;
pub const CNA_DEPTH_ENCODING_HALF_FLOAT: CNA_DepthEncoding = 2;

/// Called once per submitted entry, in the order the list decides.
pub type CNA_TransparentDrawCallback =
    Option<unsafe extern "C" fn(*mut c_void) -> CNA_Result>;
pub type CNA_PunctualLightKindEXT = u32;

pub const CNA_PUNCTUAL_LIGHT_KIND_EXT_NONE: CNA_PunctualLightKindEXT = 0;
pub const CNA_PUNCTUAL_LIGHT_KIND_EXT_POINT: CNA_PunctualLightKindEXT = 1;
pub const CNA_PUNCTUAL_LIGHT_KIND_EXT_SPOT: CNA_PunctualLightKindEXT = 2;
pub const CNA_SHADOW_CASCADE_MAX_EXT: i32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_PointLightEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub position: CNA_Vector3,
    pub color: CNA_Vector3,
    pub intensity: f32,
    pub range: f32,
    pub casts_shadows: CNA_Bool,
    pub reserved: [u8; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_SpotLightEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub position: CNA_Vector3,
    pub direction: CNA_Vector3,
    pub color: CNA_Vector3,
    pub intensity: f32,
    pub range: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
    pub casts_shadows: CNA_Bool,
    pub reserved: [u8; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_PunctualLightEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub kind: CNA_PunctualLightKindEXT,
    pub reserved: u32,
    pub position: CNA_Vector3,
    pub direction: CNA_Vector3,
    pub diffuse_color: CNA_Vector3,
    pub range: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
    pub shadow_depth_bias: f32,
    pub shadow_cube: CNA_Handle,
    pub shadow_map: CNA_Handle,
    pub shadow_view_projection: CNA_Matrix,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CNA_ShadowCascadeStateEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub count: i32,
    pub blend_band: f32,
    pub world_to_atlas: [CNA_Matrix; 4],
    pub split_distance: [f32; 4],
    pub camera_view: CNA_Matrix,
    pub debug_tint: CNA_Bool,
    pub reserved: [u8; 3],
}

impl Default for CNA_ShadowCascadeStateEXT {
    fn default() -> Self {
        Self {
            struct_size: 0,
            struct_version: 0,
            count: 0,
            blend_band: 0.0,
            world_to_atlas: [CNA_Matrix::default(); 4],
            split_distance: [0.0; 4],
            camera_view: CNA_Matrix::default(),
            debug_tint: CNA_FALSE,
            reserved: [0; 3],
        }
    }
}
pub type CNA_LutInterpolation = u32;

pub const CNA_LUT_INTERPOLATION_TRILINEAR: CNA_LutInterpolation = 0;
pub const CNA_LUT_INTERPOLATION_TETRAHEDRAL: CNA_LutInterpolation = 1;
pub type CNA_GraphicsImageAccess = u32;
pub type CNA_GraphicsMemoryBarrier = u32;

pub const CNA_GRAPHICS_IMAGE_ACCESS_READ_ONLY: CNA_GraphicsImageAccess = 0;
pub const CNA_GRAPHICS_IMAGE_ACCESS_WRITE_ONLY: CNA_GraphicsImageAccess = 1;
pub const CNA_GRAPHICS_IMAGE_ACCESS_READ_WRITE: CNA_GraphicsImageAccess = 2;

pub const CNA_GRAPHICS_MEMORY_BARRIER_NONE: CNA_GraphicsMemoryBarrier = 0;
pub const CNA_GRAPHICS_MEMORY_BARRIER_VERTEX_ATTRIB_ARRAY: CNA_GraphicsMemoryBarrier = 1 << 0;
pub const CNA_GRAPHICS_MEMORY_BARRIER_ELEMENT_ARRAY: CNA_GraphicsMemoryBarrier = 1 << 1;
pub const CNA_GRAPHICS_MEMORY_BARRIER_UNIFORM: CNA_GraphicsMemoryBarrier = 1 << 2;
pub const CNA_GRAPHICS_MEMORY_BARRIER_TEXTURE_FETCH: CNA_GraphicsMemoryBarrier = 1 << 3;
pub const CNA_GRAPHICS_MEMORY_BARRIER_SHADER_IMAGE_ACCESS: CNA_GraphicsMemoryBarrier = 1 << 4;
pub const CNA_GRAPHICS_MEMORY_BARRIER_SHADER_STORAGE: CNA_GraphicsMemoryBarrier = 1 << 5;
pub const CNA_GRAPHICS_MEMORY_BARRIER_BUFFER_UPDATE: CNA_GraphicsMemoryBarrier = 1 << 6;
pub const CNA_GRAPHICS_MEMORY_BARRIER_FRAMEBUFFER: CNA_GraphicsMemoryBarrier = 1 << 7;
pub const CNA_GRAPHICS_MEMORY_BARRIER_INDIRECT_COMMAND: CNA_GraphicsMemoryBarrier = 1 << 8;
/// Every bit above, folded together, exactly as the canonical macro defines it.
pub const CNA_GRAPHICS_MEMORY_BARRIER_ALL: CNA_GraphicsMemoryBarrier =
    CNA_GRAPHICS_MEMORY_BARRIER_VERTEX_ATTRIB_ARRAY
        | CNA_GRAPHICS_MEMORY_BARRIER_ELEMENT_ARRAY
        | CNA_GRAPHICS_MEMORY_BARRIER_UNIFORM
        | CNA_GRAPHICS_MEMORY_BARRIER_TEXTURE_FETCH
        | CNA_GRAPHICS_MEMORY_BARRIER_SHADER_IMAGE_ACCESS
        | CNA_GRAPHICS_MEMORY_BARRIER_SHADER_STORAGE
        | CNA_GRAPHICS_MEMORY_BARRIER_BUFFER_UPDATE
        | CNA_GRAPHICS_MEMORY_BARRIER_FRAMEBUFFER
        | CNA_GRAPHICS_MEMORY_BARRIER_INDIRECT_COMMAND;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_Particle {
    pub position: CNA_Vector4,
    pub velocity: CNA_Vector4,
    pub state: CNA_Vector4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_ParticleEmitterSettings {
    pub struct_size: u32,
    pub struct_version: u32,
    pub position: CNA_Vector3,
    pub direction: CNA_Vector3,
    pub gravity: CNA_Vector3,
    pub start_color: CNA_Vector4,
    pub end_color: CNA_Vector4,
    pub cone_angle: f32,
    pub speed: f32,
    pub speed_variance: f32,
    pub lifetime: f32,
    pub lifetime_variance: f32,
    pub drag: f32,
    pub emission_rate: f32,
    pub start_size: f32,
    pub end_size: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_PostProcessContext {
    pub struct_size: u32,
    pub struct_version: u32,
    pub source: CNA_Handle,
    pub source_depth: CNA_Handle,
    pub source_normals: CNA_Handle,
    pub source_velocity: CNA_Handle,
    pub destination: CNA_Handle,
    pub width: i32,
    pub height: i32,
    pub elapsed_seconds: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub has_previous_frame: CNA_Bool,
    pub reserved: [u8; 3],
    pub projection: CNA_Matrix,
    pub inverse_projection: CNA_Matrix,
    pub inverse_view: CNA_Matrix,
    pub previous_view_projection: CNA_Matrix,
    pub settings: *const CNA_RenderPipelineSettingsEXT,
}

impl Default for CNA_PostProcessContext {
    fn default() -> Self {
        Self {
            struct_size: 0,
            struct_version: 0,
            source: CNA_INVALID_HANDLE,
            source_depth: CNA_INVALID_HANDLE,
            source_normals: CNA_INVALID_HANDLE,
            source_velocity: CNA_INVALID_HANDLE,
            destination: CNA_INVALID_HANDLE,
            width: 0,
            height: 0,
            elapsed_seconds: 0.0,
            near_plane: 0.0,
            far_plane: 0.0,
            has_previous_frame: CNA_FALSE,
            reserved: [0; 3],
            projection: CNA_Matrix::default(),
            inverse_projection: CNA_Matrix::default(),
            inverse_view: CNA_Matrix::default(),
            previous_view_projection: CNA_Matrix::default(),
            settings: core::ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_DirectionalLightEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub direction: CNA_Vector3,
    pub color: CNA_Vector3,
    pub intensity: f32,
    pub casts_shadows: CNA_Bool,
    pub reserved: [u8; 3],
}

/// Receives one draw request from inside an open render-pipeline frame.
///
/// Returning anything but `CNA_RESULT_SUCCESS` fails the frame that asked for
/// it: CNA raises the result out of `end` rather than swallowing it.
pub type CNA_RenderPipelineDrawCallback =
    Option<unsafe extern "C" fn(*mut c_void) -> CNA_Result>;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_RenderPipelineFrameStatisticsEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub passes_run: i32,
    pub target_switches: i32,
    pub used_scene_target: CNA_Bool,
    pub drew_skybox: CNA_Bool,
    pub reserved: [u8; 2],
    pub gpu_memory_estimate_bytes: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_PassTimingEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub sample_count: i32,
    pub reserved: [u8; 4],
    pub milliseconds: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_TextureTransformEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub offset: CNA_Vector2,
    pub scale: CNA_Vector2,
    pub rotation: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_PbrMaterialEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub albedo_texture: CNA_Handle,
    pub normal_texture: CNA_Handle,
    pub metallic_roughness_texture: CNA_Handle,
    pub ambient_occlusion_texture: CNA_Handle,
    pub emissive_texture: CNA_Handle,
    pub specular_texture: CNA_Handle,
    pub specular_color_texture: CNA_Handle,
    pub albedo_color: CNA_Color,
    pub emissive_factor: CNA_Vector3,
    pub specular_color_factor: CNA_Vector3,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub ior: f32,
    pub specular_factor: f32,
    pub alpha_cutoff: f32,
    pub alpha_mode: CNA_AlphaModeEXT,
    pub double_sided: CNA_Bool,
    pub base_color_texture_srgb: CNA_Bool,
    pub emissive_texture_srgb: CNA_Bool,
    pub specular_color_texture_srgb: CNA_Bool,
    pub output_encoded_to_srgb: CNA_Bool,
    pub reserved: [u8; 3],
    pub texture_coordinate_sets: [i32; 7],
    pub texture_transforms: [CNA_TextureTransformEXT; 7],
}

impl Default for CNA_PbrMaterialEXT {
    fn default() -> Self {
        // SAFETY-free: every field is a plain scalar, handle or `repr(C)`
        // aggregate of them, so an all-zero value is a valid instance. CNA's
        // own initializer overwrites it before it is used.
        Self {
            struct_size: 0,
            struct_version: 0,
            albedo_texture: 0,
            normal_texture: 0,
            metallic_roughness_texture: 0,
            ambient_occlusion_texture: 0,
            emissive_texture: 0,
            specular_texture: 0,
            specular_color_texture: 0,
            albedo_color: CNA_Color::default(),
            emissive_factor: CNA_Vector3::default(),
            specular_color_factor: CNA_Vector3::default(),
            metallic_factor: 0.0,
            roughness_factor: 0.0,
            normal_scale: 0.0,
            occlusion_strength: 0.0,
            ior: 0.0,
            specular_factor: 0.0,
            alpha_cutoff: 0.0,
            alpha_mode: 0,
            double_sided: 0,
            base_color_texture_srgb: 0,
            emissive_texture_srgb: 0,
            specular_color_texture_srgb: 0,
            output_encoded_to_srgb: 0,
            reserved: [0; 3],
            texture_coordinate_sets: [0; 7],
            texture_transforms: [CNA_TextureTransformEXT {
                struct_size: 0,
                struct_version: 0,
                offset: CNA_Vector2 { x: 0.0, y: 0.0 },
                scale: CNA_Vector2 { x: 0.0, y: 0.0 },
                rotation: 0.0,
            }; 7],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_PbrMaterial {
    pub albedo_texture: CNA_Handle,
    pub normal_texture: CNA_Handle,
    pub metallic_roughness_texture: CNA_Handle,
    pub ambient_occlusion_texture: CNA_Handle,
    pub emissive_texture: CNA_Handle,
    pub albedo_color: CNA_Color,
    pub emissive_color: CNA_Color,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub alpha_cutoff: f32,
    pub alpha_blend_enabled: CNA_Bool,
    pub reserved: [u8; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_RenderPipelineSettings {
    pub exposure: f32,
    pub gamma: f32,
    pub bloom_intensity: f32,
    pub tonemapping_mode: CNA_TonemappingMode,
    pub render_quality: CNA_RenderQuality,
    pub shadow_quality: CNA_ShadowQuality,
    pub hdr_enabled: CNA_Bool,
    pub bloom_enabled: CNA_Bool,
    pub ssao_enabled: CNA_Bool,
    pub shadows_enabled: CNA_Bool,
}

pub type CNA_PbrMaterialExtensionsHandle = CNA_Handle;
pub type CNA_HapticDeviceHandle = CNA_Handle;
pub type CNA_HapticEffectType = u32;
pub type CNA_HapticDirectionType = u32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_HapticDirection {
    pub r#type: CNA_HapticDirectionType,
    pub values: [i32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_HapticEffect {
    pub struct_size: u32,
    pub struct_version: u32,
    pub r#type: CNA_HapticEffectType,
    pub reserved: u32,
    pub direction: CNA_HapticDirection,
    pub length: u32,
    pub delay: u16,
    pub button: u16,
    pub interval: u16,
    pub level: i16,
    pub period: u16,
    pub magnitude: i16,
    pub offset: i16,
    pub phase: u16,
    pub ramp_start: i16,
    pub ramp_end: i16,
    pub right_saturation: [u16; 3],
    pub left_saturation: [u16; 3],
    pub right_coefficient: [i16; 3],
    pub left_coefficient: [i16; 3],
    pub deadband: [u16; 3],
    pub center: [i16; 3],
    pub large_magnitude: u16,
    pub small_magnitude: u16,
    pub custom_period: u16,
    pub custom_channels: u8,
    pub reserved2: u8,
    pub attack_length: u16,
    pub attack_level: u16,
    pub fade_length: u16,
    pub fade_level: u16,
}

pub type CNA_HapticFeature = u32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_HapticCapabilities {
    pub struct_size: u32,
    pub struct_version: u32,
    pub features: CNA_HapticFeature,
    pub axis_count: i32,
    pub max_effects: i32,
    pub max_effects_playing: i32,
    pub is_open: CNA_Bool,
    pub rumble_supported: CNA_Bool,
    pub reserved: [u8; 2],
}

pub type CNA_SensorEventRegistrationHandle = CNA_Handle;
pub type CNA_AccelerometerHandle = CNA_Handle;
pub type CNA_CompassHandle = CNA_Handle;
pub type CNA_GyroscopeHandle = CNA_Handle;
pub type CNA_MotionHandle = CNA_Handle;
pub type CNA_SensorType = u32;
pub type CNA_SensorState = u32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_DateTimeOffset {
    pub ticks: i64,
    pub offset_ticks: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_SensorInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub id: u32,
    pub r#type: CNA_SensorType,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_AccelerometerReading {
    pub struct_size: u32,
    pub struct_version: u32,
    pub timestamp: CNA_DateTimeOffset,
    pub acceleration: CNA_Vector3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CompassReading {
    pub struct_size: u32,
    pub struct_version: u32,
    pub timestamp: CNA_DateTimeOffset,
    pub heading_accuracy: f64,
    pub magnetic_heading: f64,
    pub true_heading: f64,
    pub magnetometer_reading: CNA_Vector3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GyroscopeReading {
    pub struct_size: u32,
    pub struct_version: u32,
    pub timestamp: CNA_DateTimeOffset,
    pub rotation_rate: CNA_Vector3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_AttitudeReading {
    pub struct_size: u32,
    pub struct_version: u32,
    pub timestamp: CNA_DateTimeOffset,
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
    pub quaternion: CNA_Quaternion,
    pub rotation_matrix: CNA_Matrix,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_MotionReading {
    pub struct_size: u32,
    pub struct_version: u32,
    pub timestamp: CNA_DateTimeOffset,
    pub attitude: CNA_AttitudeReading,
    pub device_acceleration: CNA_Vector3,
    pub device_rotation_rate: CNA_Vector3,
    pub gravity: CNA_Vector3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_AccelerometerReadingEventInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub timestamp: CNA_DateTimeOffset,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

// The sensor event callbacks. `Option` so a null callback is expressible: the
// C side accepts one, and a `fn` pointer in Rust cannot be null.
pub type CNA_SensorEventCallback = Option<unsafe extern "C" fn(context: *mut c_void)>;
pub type CNA_AccelerometerReadingCallback =
    Option<unsafe extern "C" fn(reading: *const CNA_AccelerometerReading, context: *mut c_void)>;
pub type CNA_AccelerometerReadingEventCallback = Option<
    unsafe extern "C" fn(info: *const CNA_AccelerometerReadingEventInfo, context: *mut c_void),
>;
pub type CNA_CompassReadingCallback =
    Option<unsafe extern "C" fn(reading: *const CNA_CompassReading, context: *mut c_void)>;
pub type CNA_GyroscopeReadingCallback =
    Option<unsafe extern "C" fn(reading: *const CNA_GyroscopeReading, context: *mut c_void)>;
pub type CNA_MotionReadingCallback =
    Option<unsafe extern "C" fn(reading: *const CNA_MotionReading, context: *mut c_void)>;

pub type CNA_InputDeviceEventRegistrationHandle = CNA_Handle;
pub type CNA_MouseCursorHandle = CNA_Handle;
pub type CNA_MouseCursorStock = u32;

/// Receives one device connection or disconnection.
pub type CNA_InputDeviceHotplugCallback = Option<unsafe extern "C" fn(u32, *mut c_void)>;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_InputDeviceInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub id: u64,
}

pub type CNA_TextInputRegistrationHandle = CNA_Handle;
pub type CNA_TextInputType = u32;

/// Receives one committed UTF-16 code unit.
///
/// A code point above U+FFFF arrives as two calls -- a high surrogate then a
/// low surrogate -- exactly as the canonical event delivers it.
pub type CNA_TextInputCallback = Option<unsafe extern "C" fn(u16, *mut c_void)>;

/// Receives one IME composition update. The info is valid only for the call.
pub type CNA_TextEditingCallback =
    Option<unsafe extern "C" fn(*const CNA_TextEditingEventInfo, *mut c_void)>;

/// Receives the current IME candidate list. Valid only for the call.
pub type CNA_TextEditingCandidatesCallback =
    Option<unsafe extern "C" fn(*const CNA_TextEditingCandidatesEventInfo, *mut c_void)>;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_TextEditingEventInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub text: CNA_StringView,
    pub start: i32,
    pub length: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_TextEditingCandidatesEventInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub candidates: *const CNA_StringView,
    pub candidate_count: i32,
    pub selected: i32,
    pub horizontal: CNA_Bool,
    pub reserved: [u8; 3],
}

pub type CNA_CnbSpriteFontDataHandle = CNA_Handle;
pub type CNA_CnbSoundEffectDataHandle = CNA_Handle;
pub type CNA_CnbAudioFormat = u32;
pub type CNA_CnbEffectKind = u32;
pub type CNA_CnbMaterialTextureSlot = u32;
pub type CNA_PowerState = u32;
pub type CNA_RendererFeature = u32;
pub type CNA_RendererFeatureSupport = u32;
pub type CNA_RendererLimit = u32;
pub type CNA_RendererFormatUsageFlags = u32;
pub type CNA_ShaderDialect = u32;
pub type CNA_LogLevel = u32;
pub type CNA_LogCategory = u32;
pub type CNA_Platform = u32;
pub type CNA_DesktopOS = u32;
pub type CNA_SurfaceFormat = u32;
pub type CNA_TextureDataType = u32;
pub type CNA_TextureImageFormat = u32;
pub type CNA_BufferUsage = u32;
pub type CNA_IndexElementSize = u32;
pub type CNA_ClearOptions = u32;
pub type CNA_SetDataOptions = u32;
pub type CNA_PrimitiveType = u32;
pub type CNA_UserVertexSource = u32;
pub type CNA_VertexElementFormat = u32;
pub type CNA_VertexElementUsage = u32;
pub type CNA_VertexType = u32;
pub type CNA_SpriteSortMode = u32;
pub type CNA_SpriteEffects = u32;
pub type CNA_Char16 = u16;
pub type CNA_Blend = u32;
pub type CNA_BlendFunction = u32;
pub type CNA_ColorWriteChannels = u32;
pub type CNA_CompareFunction = u32;
pub type CNA_StencilOperation = u32;
pub type CNA_CullMode = u32;
pub type CNA_FillMode = u32;
pub type CNA_TextureAddressMode = u32;
pub type CNA_TextureFilter = u32;
pub type CNA_BlendStatePreset = u32;
pub type CNA_DepthStencilStatePreset = u32;
pub type CNA_RasterizerStatePreset = u32;
pub type CNA_SamplerStatePreset = u32;
pub type CNA_Key = u32;
pub type CNA_MouseButtonFlags = u32;
pub type CNA_PlayerIndex = u32;
pub type CNA_GamePadDeadZone = u32;
pub type CNA_GamePadButtonFlags = u32;
pub type CNA_GamePadType = u32;
pub type CNA_TouchLocationState = u32;
pub type CNA_GestureType = u32;
pub type CNA_StorageDeviceHandle = CNA_Handle;
pub type CNA_StorageContainerHandle = CNA_Handle;
pub type CNA_StorageStreamHandle = CNA_Handle;
pub type CNA_FileMode = u32;
pub type CNA_FileAccess = u32;
pub type CNA_FileShare = u32;
pub type CNA_SeekOrigin = u32;
pub type CNA_EffectParameterClass = u32;
pub type CNA_EffectParameterType = u32;
pub type CNA_EffectValueType = u32;
pub type CNA_EffectTextureType = u32;
pub type CNA_AudioChannels = u32;
pub type CNA_SoundState = u32;
pub type CNA_AudioStopOptions = u32;
pub type CNA_MicrophoneState = u32;
pub type CNA_AudioEventRegistrationHandle = CNA_Handle;
pub type CNA_AudioEventCallback = Option<unsafe extern "C" fn(*mut c_void)>;
pub type CNA_MediaState = u32;
pub type CNA_MediaSourceType = u32;
pub type CNA_VideoSoundtrackType = u32;
pub type CNA_SongHandle = CNA_Handle;
pub type CNA_SongCollectionHandle = CNA_Handle;
pub type CNA_MediaLibraryHandle = CNA_Handle;
pub type CNA_AlbumHandle = CNA_Handle;
pub type CNA_AlbumCollectionHandle = CNA_Handle;
pub type CNA_ArtistHandle = CNA_Handle;
pub type CNA_ArtistCollectionHandle = CNA_Handle;
pub type CNA_GenreHandle = CNA_Handle;
pub type CNA_GenreCollectionHandle = CNA_Handle;
pub type CNA_PlaylistHandle = CNA_Handle;
pub type CNA_PlaylistCollectionHandle = CNA_Handle;
pub type CNA_PictureHandle = CNA_Handle;
pub type CNA_PictureCollectionHandle = CNA_Handle;
pub type CNA_PictureAlbumHandle = CNA_Handle;
pub type CNA_PictureAlbumCollectionHandle = CNA_Handle;
pub type CNA_MediaQueueHandle = CNA_Handle;
pub type CNA_MediaPlayerEventRegistrationHandle = CNA_Handle;
pub type CNA_VideoHandle = CNA_Handle;
pub type CNA_VideoPlayerHandle = CNA_Handle;
pub type CNA_MediaPlayerEventCallback = Option<unsafe extern "C" fn(*mut c_void)>;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_StringView {
    pub data: *const c_char,
    pub byte_length: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_VisualizationData {
    pub struct_size: u32,
    pub struct_version: u32,
    pub frequencies: [f32; CNA_VISUALIZATION_DATA_SIZE as usize],
    pub samples: [f32; CNA_VISUALIZATION_DATA_SIZE as usize],
}

impl Default for CNA_VisualizationData {
    fn default() -> Self {
        Self {
            struct_size: core::mem::size_of::<Self>() as u32,
            struct_version: 1,
            frequencies: [0.0; CNA_VISUALIZATION_DATA_SIZE as usize],
            samples: [0.0; CNA_VISUALIZATION_DATA_SIZE as usize],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_Vector2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_BoundingSphere {
    pub center: CNA_Vector3,
    pub radius: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_BoundingFrustum {
    pub matrix: CNA_Matrix,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_VertexPositionColor {
    pub position: CNA_Vector3,
    pub color: CNA_Color,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_BoundingBox {
    pub min: CNA_Vector3,
    pub max: CNA_Vector3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_Matrix {
    pub m11: f32,
    pub m12: f32,
    pub m13: f32,
    pub m14: f32,
    pub m21: f32,
    pub m22: f32,
    pub m23: f32,
    pub m24: f32,
    pub m31: f32,
    pub m32: f32,
    pub m33: f32,
    pub m34: f32,
    pub m41: f32,
    pub m42: f32,
    pub m43: f32,
    pub m44: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_EffectAnnotationInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub row_count: i32,
    pub column_count: i32,
    pub parameter_class: CNA_EffectParameterClass,
    pub parameter_type: CNA_EffectParameterType,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_EffectAnnotationCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub name: CNA_StringView,
    pub semantic: CNA_StringView,
    pub row_count: i32,
    pub column_count: i32,
    pub parameter_class: CNA_EffectParameterClass,
    pub parameter_type: CNA_EffectParameterType,
    pub data: *const f32,
    pub data_count: u64,
    pub cached_string: CNA_StringView,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_EffectParameterInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub row_count: i32,
    pub column_count: i32,
    pub parameter_class: CNA_EffectParameterClass,
    pub parameter_type: CNA_EffectParameterType,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_EffectParameterCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub name: CNA_StringView,
    pub semantic: CNA_StringView,
    pub row_count: i32,
    pub column_count: i32,
    pub parameter_class: CNA_EffectParameterClass,
    pub parameter_type: CNA_EffectParameterType,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_VertexElement {
    pub offset: i32,
    pub format: CNA_VertexElementFormat,
    pub usage: CNA_VertexElementUsage,
    pub usage_index: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_VertexBufferCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub vertex_declaration: CNA_Handle,
    pub vertex_count: i32,
    pub buffer_usage: CNA_BufferUsage,
    pub dynamic: CNA_Bool,
    pub reserved: [u8; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_VertexBufferInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub vertex_count: i32,
    pub buffer_usage: CNA_BufferUsage,
    pub dynamic: CNA_Bool,
    pub is_content_lost: CNA_Bool,
    pub has_renderer: CNA_Bool,
    pub reserved0: u8,
    pub vertex_stride: i32,
    pub vertex_element_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_VertexBufferTransfer {
    pub struct_size: u32,
    pub struct_version: u32,
    pub vertex_type: CNA_VertexType,
    pub options: CNA_SetDataOptions,
    pub start_index: u64,
    pub element_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_VertexBufferBinding {
    pub vertex_buffer: CNA_Handle,
    pub vertex_offset: i32,
    pub instance_frequency: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_IndexBufferCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub index_count: i32,
    pub index_element_size: CNA_IndexElementSize,
    pub buffer_usage: CNA_BufferUsage,
    pub dynamic: CNA_Bool,
    pub reserved: [u8; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_IndexBufferInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub index_count: i32,
    pub index_element_size: CNA_IndexElementSize,
    pub buffer_usage: CNA_BufferUsage,
    pub dynamic: CNA_Bool,
    pub is_content_lost: CNA_Bool,
    pub has_renderer: CNA_Bool,
    pub reserved: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_IndexBufferTransfer {
    pub struct_size: u32,
    pub struct_version: u32,
    pub index_element_size: CNA_IndexElementSize,
    pub options: CNA_SetDataOptions,
    pub start_index: u64,
    pub element_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_UserPrimitives {
    pub struct_size: u32,
    pub struct_version: u32,
    pub primitive_type: CNA_PrimitiveType,
    pub vertex_source: CNA_UserVertexSource,
    pub vertex_data: *const c_void,
    pub vertex_declaration: CNA_VertexDeclarationHandle,
    pub vertex_offset: i32,
    pub num_vertices: i32,
    pub primitive_count: i32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_UserIndices {
    pub struct_size: u32,
    pub struct_version: u32,
    pub index_element_size: CNA_IndexElementSize,
    pub index_offset: i32,
    pub index_data: *const c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_BackBufferReadback {
    pub struct_size: u32,
    pub struct_version: u32,
    pub has_source_rectangle: CNA_Bool,
    pub reserved: [u8; 3],
    pub source_rectangle: CNA_Rectangle,
    pub start_index: u64,
    pub element_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_Texture3DCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mip_map: CNA_Bool,
    pub reserved0: [u8; 3],
    pub format: CNA_SurfaceFormat,
    pub reserved1: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_Texture3DInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub level_count: u32,
    pub format: CNA_SurfaceFormat,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_Texture3DTransfer {
    pub struct_size: u32,
    pub struct_version: u32,
    pub level: i32,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub front: i32,
    pub back: i32,
    pub reserved: u32,
    pub start_index: u64,
    pub element_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_TextureCubeCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub size: u32,
    pub mip_map: CNA_Bool,
    pub reserved0: [u8; 3],
    pub format: CNA_SurfaceFormat,
    pub reserved1: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_TextureCubeInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub size: u32,
    pub level_count: u32,
    pub format: CNA_SurfaceFormat,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_TextureCubeTransfer {
    pub struct_size: u32,
    pub struct_version: u32,
    pub face: CNA_CubeMapFace,
    pub level: i32,
    pub has_rectangle: CNA_Bool,
    pub reserved0: [u8; 3],
    pub rectangle: CNA_Rectangle,
    pub reserved1: u32,
    pub start_index: u64,
    pub element_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_RenderTarget2DCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub width: u32,
    pub height: u32,
    pub mip_map: CNA_Bool,
    pub reserved0: [u8; 3],
    pub format: CNA_SurfaceFormat,
    pub depth_format: CNA_DepthFormat,
    pub multi_sample_count: i32,
    pub usage: CNA_RenderTargetUsage,
    pub reserved1: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_RenderTargetCubeCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub size: u32,
    pub mip_map: CNA_Bool,
    pub reserved: [u8; 3],
    pub format: CNA_SurfaceFormat,
    pub depth_format: CNA_DepthFormat,
    pub multi_sample_count: i32,
    pub usage: CNA_RenderTargetUsage,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_RenderTargetInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub kind: CNA_RenderTargetKind,
    pub width: u32,
    pub height: u32,
    pub level_count: u32,
    pub format: CNA_SurfaceFormat,
    pub depth_format: CNA_DepthFormat,
    pub multi_sample_count: i32,
    pub usage: CNA_RenderTargetUsage,
    pub is_content_lost: CNA_Bool,
    pub renderer_available: CNA_Bool,
    pub reserved: [u8; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_RenderTargetBinding {
    pub struct_size: u32,
    pub struct_version: u32,
    pub render_target: CNA_Handle,
    pub array_slice: i32,
    pub cube_map_face: CNA_CubeMapFace,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_DisplayMode {
    pub struct_size: u32,
    pub struct_version: u32,
    pub width: i32,
    pub height: i32,
    pub aspect_ratio: f32,
    pub format: CNA_SurfaceFormat,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_PresentationParameters {
    pub struct_size: u32,
    pub struct_version: u32,
    pub back_buffer_format: CNA_SurfaceFormat,
    pub back_buffer_width: i32,
    pub back_buffer_height: i32,
    pub depth_stencil_format: CNA_DepthFormat,
    pub multi_sample_count: i32,
    pub presentation_interval: CNA_PresentInterval,
    pub display_orientation: CNA_DisplayOrientation,
    pub render_target_usage: CNA_RenderTargetUsage,
    pub is_full_screen: CNA_Bool,
    pub headless_ext: CNA_Bool,
    pub reserved: [u8; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_GraphicsDeviceInformation {
    pub struct_size: u32,
    pub struct_version: u32,
    pub adapter_index: i32,
    pub graphics_profile: CNA_GraphicsProfile,
    pub presentation_parameters: CNA_PresentationParameters,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_TextureSlotInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub bound: CNA_Bool,
    pub reserved: [u8; 7],
    pub texture: CNA_Handle,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GraphicsAdapterInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub adapter_index: u32,
    pub is_default_adapter: CNA_Bool,
    pub is_wide_screen: CNA_Bool,
    pub use_null_device: CNA_Bool,
    pub use_reference_device: CNA_Bool,
    pub vendor_id: i32,
    pub device_id: i32,
    pub revision: i32,
    pub subsystem_id: i32,
    pub description_byte_length: u64,
    pub device_name_byte_length: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_GraphicsFormatSelection {
    pub struct_size: u32,
    pub struct_version: u32,
    pub exact_match: CNA_Bool,
    pub reserved: [u8; 3],
    pub format: CNA_SurfaceFormat,
    pub depth_format: CNA_DepthFormat,
    pub multi_sample_count: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_ErrorInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub result: CNA_Result,
    pub category: CNA_ErrorCategory,
    pub message_byte_length: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_GameTime {
    pub total_game_time_ticks: i64,
    pub elapsed_game_time_ticks: i64,
    pub is_running_slowly: CNA_Bool,
    pub reserved: [u8; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_CallbackError {
    pub struct_size: u32,
    pub struct_version: u32,
    pub message: CNA_StringView,
}

pub type CNA_GameLifecycleCallback = Option<
    unsafe extern "C" fn(
        CNA_Handle,
        *const CNA_GameTime,
        *mut c_void,
        *mut CNA_CallbackError,
    ) -> CNA_Result,
>;
pub type CNA_GameEventCallback = Option<unsafe extern "C" fn(*mut c_void)>;
pub type CNA_PreparingDeviceSettingsMutatorEXT =
    Option<unsafe extern "C" fn(*mut CNA_GraphicsDeviceInformation, *mut c_void)>;
pub type CNA_StorageCompletionCallback = Option<unsafe extern "C" fn(*mut c_void)>;

pub type CNA_GameBeginDrawCallback = Option<
    unsafe extern "C" fn(
        CNA_Handle,
        *const CNA_GameTime,
        *mut c_void,
        *mut CNA_Bool,
        *mut CNA_CallbackError,
    ) -> CNA_Result,
>;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_GameCallbacks {
    pub struct_size: u32,
    pub struct_version: u32,
    pub load_content: CNA_GameLifecycleCallback,
    pub update: CNA_GameLifecycleCallback,
    pub draw: CNA_GameLifecycleCallback,
    pub unload_content: CNA_GameLifecycleCallback,
    pub exiting: CNA_GameLifecycleCallback,
    pub context: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_GameFrameHooks {
    pub struct_size: u32,
    pub struct_version: u32,
    pub initialize: CNA_GameLifecycleCallback,
    pub begin_run: CNA_GameLifecycleCallback,
    pub end_run: CNA_GameLifecycleCallback,
    pub begin_draw: CNA_GameBeginDrawCallback,
    pub end_draw: CNA_GameLifecycleCallback,
    pub context: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_GameCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub is_fixed_time_step: CNA_Bool,
    pub reserved: [u8; 7],
    pub target_elapsed_time_ticks: i64,
    pub window_title: CNA_StringView,
    pub callbacks: *const CNA_GameCallbacks,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_Viewport {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub min_depth: f32,
    pub max_depth: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_RendererInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub renderer_name_byte_length: u64,
    pub capability_flags: CNA_GraphicsCapabilityFlags,
    pub renderer_type: CNA_GraphicsRendererType,
    pub max_texture_dimension: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_Point {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_JoystickInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub id: u32,
    pub r#type: CNA_JoystickType,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_JoystickCapabilities {
    pub struct_size: u32,
    pub struct_version: u32,
    pub axis_count: i32,
    pub button_count: i32,
    pub hat_count: i32,
    pub ball_count: i32,
    pub r#type: CNA_JoystickType,
    pub power_state: CNA_PowerState,
    pub power_percent: i32,
    pub is_connected: CNA_Bool,
    pub reserved: [u8; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbReadLimits {
    pub struct_size: u32,
    pub struct_version: u32,
    pub max_file_size: u64,
    pub max_chunk_size: u64,
    pub max_total_uncompressed_size: u64,
    pub max_chunk_count: u32,
    pub max_string_bytes: u32,
    pub max_array_element_count: u32,
    pub max_chunk_alignment: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbMetadata {
    pub struct_size: u32,
    pub struct_version: u32,
    pub present: CNA_Bool,
    pub reserved: [u8; 3],
    pub flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbTextureInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub face_count: u32,
    pub mip_count: u32,
    pub representation_count: u32,
}

/// Turns one validated `.cnb` container into a caller-owned object.
///
/// The document and content-manager handles are callback-scoped borrows: both
/// are invalidated before the callback returns and neither has a destroy
/// operation. `out_object` is the caller's own opaque pointer, which this ABI
/// never dereferences, copies or frees.
pub type CNA_CnbLoaderCallback = Option<
    unsafe extern "C" fn(
        *mut c_void,
        CNA_CnbDocumentHandle,
        CNA_Handle,
        CNA_StringView,
        *mut *mut c_void,
    ) -> CNA_Result,
>;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_ContentManagerCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub root_directory: CNA_StringView,
    pub reserved: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbSpriteFontInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub glyph_count: u64,
    pub line_spacing: i32,
    pub spacing: f32,
    pub default_character: CNA_Char16,
    pub has_default_character: CNA_Bool,
    pub reserved: [u8; 5],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbSoundEffectInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub format: CNA_CnbAudioFormat,
    pub sample_rate: u32,
    pub channels: u32,
    pub frame_count: u32,
    pub loop_start: u32,
    pub loop_length: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbModelInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub bone_count: u64,
    pub part_count: u64,
    pub mesh_count: u64,
    pub animation_count: u64,
    pub light_count: u64,
    pub has_skeleton: CNA_Bool,
    pub applies_gltf_lighting_policy: CNA_Bool,
    pub has_bone_hierarchy: CNA_Bool,
    pub reserved: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_CnbModelBone {
    pub struct_size: u32,
    pub struct_version: u32,
    pub parent: i32,
    pub reserved: u32,
    pub transform: [f32; 16],
}

impl Default for CNA_CnbModelBone {
    fn default() -> Self {
        Self {
            struct_size: 0,
            struct_version: 0,
            parent: 0,
            reserved: 0,
            transform: [0.0; 16],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbModelPartInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub vertex_stride: u32,
    pub vertex_count: u32,
    pub index_count: u32,
    pub index_element_size: u32,
    pub primitive_topology: u32,
    pub primitive_count: u32,
    pub effect_kind: CNA_CnbEffectKind,
    pub vertex_color_enabled: CNA_Bool,
    pub unlit: CNA_Bool,
    pub reserved: [u8; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbMaterialInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub base_color_factor: [f32; 4],
    pub emissive_factor: [f32; 3],
    pub specular_color_factor: [f32; 3],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub ior: f32,
    pub specular_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub alpha_cutoff: f32,
    pub alpha_mode: u32,
    pub double_sided: CNA_Bool,
    pub reserved: [u8; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_CnbMeshInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub parent_bone: i32,
    pub reserved: u32,
    pub part_index_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GraphicsRendererFallbackRecord {
    pub struct_size: u32,
    pub struct_version: u32,
    pub r#type: CNA_GraphicsRendererType,
    pub reason: CNA_GraphicsRendererFallbackReason,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_VideoFrameEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub texture: CNA_Handle,
    pub generation: u64,
    pub presentation_time: f64,
    pub available: CNA_Bool,
    pub reserved: [u8; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_Texture2DInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub width: u32,
    pub height: u32,
    pub level_count: u32,
    pub format: CNA_SurfaceFormat,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_Texture2DCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub width: u32,
    pub height: u32,
    pub mip_map: CNA_Bool,
    pub reserved: [u8; 3],
    pub format: CNA_SurfaceFormat,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_Texture2DTransfer {
    pub struct_size: u32,
    pub struct_version: u32,
    pub level: i32,
    pub has_rectangle: CNA_Bool,
    pub reserved: [u8; 3],
    pub rectangle: CNA_Rectangle,
    pub start_index: u64,
    pub element_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_Texture2DDecodeInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub width: u32,
    pub height: u32,
    pub zoom: CNA_Bool,
    pub reserved: [u8; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_BlendState {
    pub struct_size: u32,
    pub struct_version: u32,
    pub alpha_blend_function: CNA_BlendFunction,
    pub alpha_destination_blend: CNA_Blend,
    pub alpha_source_blend: CNA_Blend,
    pub color_blend_function: CNA_BlendFunction,
    pub color_destination_blend: CNA_Blend,
    pub color_source_blend: CNA_Blend,
    pub color_write_channels: CNA_ColorWriteChannels,
    pub color_write_channels1: CNA_ColorWriteChannels,
    pub color_write_channels2: CNA_ColorWriteChannels,
    pub color_write_channels3: CNA_ColorWriteChannels,
    pub blend_factor: CNA_Color,
    pub multi_sample_mask: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_DepthStencilState {
    pub struct_size: u32,
    pub struct_version: u32,
    pub depth_buffer_enable: CNA_Bool,
    pub depth_buffer_write_enable: CNA_Bool,
    pub stencil_enable: CNA_Bool,
    pub two_sided_stencil_mode: CNA_Bool,
    pub depth_buffer_function: CNA_CompareFunction,
    pub stencil_function: CNA_CompareFunction,
    pub stencil_mask: i32,
    pub stencil_write_mask: i32,
    pub reference_stencil: i32,
    pub stencil_fail: CNA_StencilOperation,
    pub stencil_depth_buffer_fail: CNA_StencilOperation,
    pub stencil_pass: CNA_StencilOperation,
    pub counter_clockwise_stencil_function: CNA_CompareFunction,
    pub counter_clockwise_stencil_fail: CNA_StencilOperation,
    pub counter_clockwise_stencil_depth_buffer_fail: CNA_StencilOperation,
    pub counter_clockwise_stencil_pass: CNA_StencilOperation,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_RasterizerState {
    pub struct_size: u32,
    pub struct_version: u32,
    pub cull_mode: CNA_CullMode,
    pub fill_mode: CNA_FillMode,
    pub depth_bias: f32,
    pub slope_scale_depth_bias: f32,
    pub multi_sample_anti_alias: CNA_Bool,
    pub scissor_test_enable: CNA_Bool,
    pub reserved: [u8; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_SamplerState {
    pub struct_size: u32,
    pub struct_version: u32,
    pub address_u: CNA_TextureAddressMode,
    pub address_v: CNA_TextureAddressMode,
    pub address_w: CNA_TextureAddressMode,
    pub filter: CNA_TextureFilter,
    pub max_anisotropy: i32,
    pub max_mip_level: i32,
    pub mip_map_level_of_detail_bias: f32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_SpriteBatchBeginInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub sort_mode: CNA_SpriteSortMode,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_SpriteCommand {
    pub struct_size: u32,
    pub struct_version: u32,
    pub texture: CNA_Handle,
    pub destination: CNA_Rectangle,
    pub source: CNA_Rectangle,
    pub color: CNA_Color,
    pub rotation: f32,
    pub origin: CNA_Vector2,
    pub effects: CNA_SpriteEffects,
    pub layer_depth: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_SpriteFontGlyph {
    pub struct_size: u32,
    pub struct_version: u32,
    pub glyph_bounds: CNA_Rectangle,
    pub cropping: CNA_Rectangle,
    pub character: CNA_Char16,
    pub reserved: u16,
    pub kerning: CNA_Vector3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_SpriteFontCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub texture: CNA_Handle,
    pub glyphs: *const CNA_SpriteFontGlyph,
    pub glyph_count: u64,
    pub line_spacing: i32,
    pub spacing: f32,
    pub default_character: CNA_Char16,
    pub has_default_character: CNA_Bool,
    pub reserved: [u8; 5],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_SpriteFontInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub character_count: u64,
    pub line_spacing: i32,
    pub spacing: f32,
    pub default_character: CNA_Char16,
    pub has_default_character: CNA_Bool,
    pub reserved: [u8; 5],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_SpriteTextCommand {
    pub struct_size: u32,
    pub struct_version: u32,
    pub sprite_font: CNA_Handle,
    pub text: CNA_StringView,
    pub position: CNA_Vector2,
    pub color: CNA_Color,
    pub rotation: f32,
    pub origin: CNA_Vector2,
    pub scale: CNA_Vector2,
    pub effects: CNA_SpriteEffects,
    pub layer_depth: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_KeyboardState {
    pub struct_size: u32,
    pub struct_version: u32,
    pub pressed_key_words: [u64; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_MouseState {
    pub struct_size: u32,
    pub struct_version: u32,
    pub x: i32,
    pub y: i32,
    pub scroll_wheel: i32,
    pub horizontal_scroll_wheel: i32,
    pub pressed_buttons: CNA_MouseButtonFlags,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_GamePadAnalogState {
    pub left_thumb_stick: CNA_Vector2,
    pub right_thumb_stick: CNA_Vector2,
    pub left_trigger: f32,
    pub right_trigger: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_GamePadState {
    pub struct_size: u32,
    pub struct_version: u32,
    pub is_connected: CNA_Bool,
    pub reserved0: [u8; 3],
    pub packet_number: i32,
    pub pressed_buttons: CNA_GamePadButtonFlags,
    pub reserved1: u32,
    pub analog: CNA_GamePadAnalogState,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_GamePadCapabilities {
    pub struct_size: u32,
    pub struct_version: u32,
    pub gamepad_type: CNA_GamePadType,
    pub is_connected: CNA_Bool,
    pub has_a_button: CNA_Bool,
    pub has_b_button: CNA_Bool,
    pub has_x_button: CNA_Bool,
    pub has_y_button: CNA_Bool,
    pub has_back_button: CNA_Bool,
    pub has_start_button: CNA_Bool,
    pub has_big_button: CNA_Bool,
    pub has_dpad_up_button: CNA_Bool,
    pub has_dpad_down_button: CNA_Bool,
    pub has_dpad_left_button: CNA_Bool,
    pub has_dpad_right_button: CNA_Bool,
    pub has_left_shoulder_button: CNA_Bool,
    pub has_right_shoulder_button: CNA_Bool,
    pub has_left_stick_button: CNA_Bool,
    pub has_right_stick_button: CNA_Bool,
    pub has_left_x_thumb_stick: CNA_Bool,
    pub has_left_y_thumb_stick: CNA_Bool,
    pub has_right_x_thumb_stick: CNA_Bool,
    pub has_right_y_thumb_stick: CNA_Bool,
    pub has_left_trigger: CNA_Bool,
    pub has_right_trigger: CNA_Bool,
    pub has_left_vibration_motor: CNA_Bool,
    pub has_right_vibration_motor: CNA_Bool,
    pub has_voice_support: CNA_Bool,
    pub has_light_bar_ext: CNA_Bool,
    pub has_trigger_vibration_motors_ext: CNA_Bool,
    pub has_misc1_ext: CNA_Bool,
    pub has_paddle1_ext: CNA_Bool,
    pub has_paddle2_ext: CNA_Bool,
    pub has_paddle3_ext: CNA_Bool,
    pub has_paddle4_ext: CNA_Bool,
    pub has_touchpad_ext: CNA_Bool,
    pub has_gyro_ext: CNA_Bool,
    pub has_accelerometer_ext: CNA_Bool,
    pub reserved: [u8; 1],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_TouchLocation {
    pub id: i32,
    pub state: CNA_TouchLocationState,
    pub position: CNA_Vector2,
    pub previous_state: CNA_TouchLocationState,
    pub previous_position: CNA_Vector2,
    pub pressure: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_TouchCapabilities {
    pub struct_size: u32,
    pub struct_version: u32,
    pub is_connected: CNA_Bool,
    pub reserved: [u8; 3],
    pub maximum_touch_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_TouchState {
    pub struct_size: u32,
    pub struct_version: u32,
    pub is_connected: CNA_Bool,
    pub reserved: [u8; 3],
    pub touch_count: u32,
    pub touches: [CNA_TouchLocation; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CNA_GestureSample {
    pub struct_size: u32,
    pub struct_version: u32,
    pub gesture_type: CNA_GestureType,
    pub finger_id_ext: i32,
    pub finger_id2_ext: i32,
    pub reserved: u32,
    pub timestamp_ticks: i64,
    pub position: CNA_Vector2,
    pub position2: CNA_Vector2,
    pub delta: CNA_Vector2,
    pub delta2: CNA_Vector2,
}

#[repr(C)]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_SoundEffectCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub sample_rate: u32,
    pub channels: CNA_AudioChannels,
    pub reserved: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_SoundEffectInstanceInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub state: CNA_SoundState,
    pub is_looped: CNA_Bool,
    pub reserved0: [u8; 3],
    pub volume: f32,
    pub pitch: f32,
    pub pan: f32,
    pub reserved1: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_AudioEmitter {
    pub struct_size: u32,
    pub struct_version: u32,
    pub doppler_scale: f32,
    pub forward: CNA_Vector3,
    pub position: CNA_Vector3,
    pub up: CNA_Vector3,
    pub velocity: CNA_Vector3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_AudioListener {
    pub struct_size: u32,
    pub struct_version: u32,
    pub forward: CNA_Vector3,
    pub position: CNA_Vector3,
    pub up: CNA_Vector3,
    pub velocity: CNA_Vector3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_CueInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub is_created: CNA_Bool,
    pub is_disposed: CNA_Bool,
    pub is_paused: CNA_Bool,
    pub is_playing: CNA_Bool,
    pub is_prepared: CNA_Bool,
    pub is_preparing: CNA_Bool,
    pub is_stopped: CNA_Bool,
    pub is_stopping: CNA_Bool,
}

pub type cna_get_abi_version_fn = unsafe extern "C" fn() -> u32;
pub type cna_error_get_last_info_fn = unsafe extern "C" fn(*mut CNA_ErrorInfo) -> CNA_Result;
pub type cna_error_get_last_message_size_fn = unsafe extern "C" fn(*mut u64) -> CNA_Result;
pub type cna_error_copy_last_message_fn =
    unsafe extern "C" fn(*mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_game_create_fn =
    unsafe extern "C" fn(*const CNA_GameCreateInfo, *mut CNA_Handle) -> CNA_Result;
pub type cna_game_set_frame_hooks_ext_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_GameFrameHooks) -> CNA_Result;
pub type cna_game_run_one_frame_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_game_run_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_game_request_exit_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_game_get_is_active_fn = unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_game_get_is_mouse_visible_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_game_set_is_mouse_visible_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Bool) -> CNA_Result;
pub type cna_game_get_is_fixed_time_step_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_game_set_is_fixed_time_step_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Bool) -> CNA_Result;
pub type cna_game_get_target_elapsed_time_ticks_fn =
    unsafe extern "C" fn(CNA_Handle, *mut i64) -> CNA_Result;
pub type cna_game_set_target_elapsed_time_ticks_fn =
    unsafe extern "C" fn(CNA_Handle, i64) -> CNA_Result;
pub type cna_game_get_inactive_sleep_time_ticks_fn =
    unsafe extern "C" fn(CNA_Handle, *mut i64) -> CNA_Result;
pub type cna_game_set_inactive_sleep_time_ticks_fn =
    unsafe extern "C" fn(CNA_Handle, i64) -> CNA_Result;
pub type cna_game_reset_elapsed_time_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_game_suppress_draw_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_game_tick_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_game_set_window_title_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView) -> CNA_Result;
pub type cna_game_subscribe_fn = unsafe extern "C" fn(
    CNA_Handle,
    CNA_GameEvent,
    CNA_GameEventCallback,
    *mut c_void,
    *mut CNA_GameEventRegistrationHandle,
) -> CNA_Result;
pub type cna_game_window_subscribe_fn = unsafe extern "C" fn(
    CNA_Handle,
    CNA_GameWindowEvent,
    CNA_GameEventCallback,
    *mut c_void,
    *mut CNA_GameEventRegistrationHandle,
) -> CNA_Result;
pub type cna_game_unsubscribe_fn =
    unsafe extern "C" fn(CNA_GameEventRegistrationHandle) -> CNA_Result;
pub type cna_game_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_game_window_get_allow_user_resizing_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_game_window_set_allow_user_resizing_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Bool) -> CNA_Result;
pub type cna_game_window_get_client_bounds_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Rectangle) -> CNA_Result;
pub type cna_game_window_get_current_orientation_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_DisplayOrientation) -> CNA_Result;
pub type cna_game_window_get_native_handle_ext_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_game_window_get_screen_device_name_size_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_game_window_copy_screen_device_name_fn =
    unsafe extern "C" fn(CNA_Handle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_game_window_get_title_size_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_game_window_copy_title_fn =
    unsafe extern "C" fn(CNA_Handle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_game_window_begin_screen_device_change_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Bool) -> CNA_Result;
pub type cna_game_window_end_screen_device_change_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView, i32, i32) -> CNA_Result;
pub type cna_game_get_graphics_device_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Handle) -> CNA_Result;
pub type cna_graphics_device_create_fn = unsafe extern "C" fn(
    u32,
    u32,
    *const CNA_PresentationParameters,
    *mut CNA_Handle,
) -> CNA_Result;
pub type cna_graphics_device_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_graphics_device_get_status_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_GraphicsDeviceStatus) -> CNA_Result;
pub type cna_graphics_device_get_graphics_profile_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_GraphicsProfile) -> CNA_Result;
pub type cna_graphics_device_get_presentation_parameters_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_PresentationParameters) -> CNA_Result;
pub type cna_graphics_device_get_display_mode_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_DisplayMode) -> CNA_Result;
pub type cna_graphics_device_get_blend_state_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_BlendState) -> CNA_Result;
pub type cna_graphics_device_get_depth_stencil_state_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_DepthStencilState) -> CNA_Result;
pub type cna_graphics_device_get_rasterizer_state_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_RasterizerState) -> CNA_Result;
pub type cna_graphics_device_get_sampler_state_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_ShaderStage, u32, *mut CNA_SamplerState) -> CNA_Result;
pub type cna_graphics_device_set_sampler_state_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_ShaderStage, u32, *const CNA_SamplerState) -> CNA_Result;
pub type cna_graphics_device_get_texture_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_ShaderStage, u32, *mut CNA_TextureSlotInfo) -> CNA_Result;
pub type cna_graphics_device_set_texture_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_ShaderStage, u32, CNA_Handle) -> CNA_Result;
pub type cna_graphics_device_get_adapter_index_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u32) -> CNA_Result;
pub type cna_graphics_adapter_get_count_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_graphics_adapter_get_info_fn =
    unsafe extern "C" fn(CNA_Handle, u32, *mut CNA_GraphicsAdapterInfo) -> CNA_Result;
pub type cna_graphics_adapter_copy_description_fn =
    unsafe extern "C" fn(CNA_Handle, u32, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_graphics_adapter_copy_device_name_fn =
    unsafe extern "C" fn(CNA_Handle, u32, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_graphics_adapter_get_current_display_mode_fn =
    unsafe extern "C" fn(CNA_Handle, u32, *mut CNA_DisplayMode) -> CNA_Result;
pub type cna_graphics_adapter_get_display_mode_count_fn =
    unsafe extern "C" fn(CNA_Handle, u32, CNA_Bool, CNA_SurfaceFormat, *mut u64) -> CNA_Result;
pub type cna_graphics_adapter_copy_display_modes_fn = unsafe extern "C" fn(
    CNA_Handle,
    u32,
    CNA_Bool,
    CNA_SurfaceFormat,
    *mut CNA_DisplayMode,
    u64,
    *mut u64,
) -> CNA_Result;
pub type cna_graphics_adapter_set_device_preferences_fn =
    unsafe extern "C" fn(CNA_Handle, u32, CNA_Bool, CNA_Bool) -> CNA_Result;
pub type cna_graphics_adapter_is_profile_supported_fn =
    unsafe extern "C" fn(CNA_Handle, u32, CNA_GraphicsProfile, *mut CNA_Bool) -> CNA_Result;
pub type cna_graphics_adapter_query_render_target_format_fn = unsafe extern "C" fn(
    CNA_Handle,
    u32,
    CNA_GraphicsProfile,
    CNA_SurfaceFormat,
    CNA_DepthFormat,
    i32,
    *mut CNA_GraphicsFormatSelection,
) -> CNA_Result;
pub type cna_graphics_adapter_query_backbuffer_format_fn = unsafe extern "C" fn(
    CNA_Handle,
    u32,
    CNA_GraphicsProfile,
    CNA_SurfaceFormat,
    CNA_DepthFormat,
    i32,
    *mut CNA_GraphicsFormatSelection,
) -> CNA_Result;
pub type cna_graphics_adapter_get_native_monitor_handle_fn =
    unsafe extern "C" fn(CNA_Handle, u32, *mut CNA_NativeHandleValue) -> CNA_Result;
pub type cna_graphics_device_get_viewport_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Viewport) -> CNA_Result;
pub type cna_graphics_device_set_viewport_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Viewport) -> CNA_Result;
pub type cna_graphics_device_get_scissor_rectangle_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Rectangle) -> CNA_Result;
pub type cna_graphics_device_set_scissor_rectangle_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Rectangle) -> CNA_Result;
pub type cna_graphics_device_get_blend_factor_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Color) -> CNA_Result;
pub type cna_graphics_device_set_blend_factor_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Color) -> CNA_Result;
pub type cna_graphics_device_get_multi_sample_mask_fn =
    unsafe extern "C" fn(CNA_Handle, *mut i32) -> CNA_Result;
pub type cna_graphics_device_set_multi_sample_mask_fn =
    unsafe extern "C" fn(CNA_Handle, i32) -> CNA_Result;
pub type cna_graphics_device_get_reference_stencil_fn =
    unsafe extern "C" fn(CNA_Handle, *mut i32) -> CNA_Result;
pub type cna_graphics_device_set_reference_stencil_fn =
    unsafe extern "C" fn(CNA_Handle, i32) -> CNA_Result;
pub type cna_graphics_device_set_blend_state_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_BlendState) -> CNA_Result;
pub type cna_graphics_device_set_depth_stencil_state_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_DepthStencilState) -> CNA_Result;
pub type cna_graphics_device_set_rasterizer_state_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_RasterizerState) -> CNA_Result;
pub type cna_graphics_device_present_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_graphics_device_reset_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_graphics_device_reset_with_parameters_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_PresentationParameters, *const u32) -> CNA_Result;
pub type cna_graphics_device_get_backbuffer_data_window_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_BackBufferReadback,
    *mut CNA_Color,
    u64,
) -> CNA_Result;
pub type cna_graphics_device_clear_rgba_fn =
    unsafe extern "C" fn(CNA_Handle, f32, f32, f32, f32) -> CNA_Result;
pub type cna_graphics_device_clear_options_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_ClearOptions, CNA_Color, f32, i32) -> CNA_Result;
pub type cna_graphics_device_set_vertex_buffer_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_VertexBufferHandle) -> CNA_Result;
pub type cna_graphics_device_set_vertex_buffer_offset_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_VertexBufferHandle, i32) -> CNA_Result;
pub type cna_graphics_device_set_vertex_buffers_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_VertexBufferBinding, u64) -> CNA_Result;
pub type cna_graphics_device_get_vertex_buffer_count_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_graphics_device_copy_vertex_buffers_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_VertexBufferBinding, u64, *mut u64) -> CNA_Result;
pub type cna_graphics_device_get_vertex_buffer_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_VertexBufferHandle) -> CNA_Result;
pub type cna_graphics_device_set_index_buffer_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_IndexBufferHandle) -> CNA_Result;
pub type cna_graphics_device_get_index_buffer_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_IndexBufferHandle) -> CNA_Result;
pub type cna_graphics_device_draw_primitives_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_PrimitiveType, i32, i32) -> CNA_Result;
pub type cna_graphics_device_draw_indexed_primitives_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_PrimitiveType, i32, i32, i32, i32, i32) -> CNA_Result;
pub type cna_graphics_device_draw_instanced_primitives_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_PrimitiveType, i32, i32, i32, i32, i32, i32) -> CNA_Result;
pub type cna_graphics_device_draw_user_primitives_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_UserPrimitives) -> CNA_Result;
pub type cna_graphics_device_draw_user_indexed_primitives_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_UserPrimitives,
    *const CNA_UserIndices,
) -> CNA_Result;
pub type cna_occlusion_query_create_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_OcclusionQueryHandle) -> CNA_Result;
pub type cna_occlusion_query_begin_fn =
    unsafe extern "C" fn(CNA_OcclusionQueryHandle) -> CNA_Result;
pub type cna_occlusion_query_end_fn = unsafe extern "C" fn(CNA_OcclusionQueryHandle) -> CNA_Result;
pub type cna_occlusion_query_get_is_complete_fn =
    unsafe extern "C" fn(CNA_OcclusionQueryHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_occlusion_query_get_pixel_count_fn =
    unsafe extern "C" fn(CNA_OcclusionQueryHandle, *mut i32) -> CNA_Result;
pub type cna_occlusion_query_destroy_fn =
    unsafe extern "C" fn(CNA_OcclusionQueryHandle) -> CNA_Result;
pub type cna_graphics_device_set_render_targets_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_RenderTargetBinding, u64) -> CNA_Result;
pub type cna_graphics_device_get_render_target_count_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_graphics_device_copy_render_targets_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_RenderTargetBinding, u64, *mut u64) -> CNA_Result;
pub type cna_graphics_device_supports_capability_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_GraphicsCapability, *mut CNA_Bool) -> CNA_Result;
pub type cna_graphics_device_get_renderer_info_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_RendererInfo) -> CNA_Result;
pub type cna_graphics_device_get_renderer_name_size_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_graphics_device_copy_renderer_name_fn =
    unsafe extern "C" fn(CNA_Handle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_texture2d_create_from_encoded_memory_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const u8,
    u64,
    *const CNA_Texture2DDecodeInfo,
    *mut CNA_Handle,
) -> CNA_Result;
pub type cna_texture2d_create_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_Texture2DCreateInfo, *mut CNA_Handle) -> CNA_Result;
pub type cna_texture2d_get_info_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Texture2DInfo) -> CNA_Result;
pub type cna_texture2d_set_data_fn = unsafe extern "C" fn(
    CNA_Handle,
    CNA_TextureDataType,
    *const CNA_Texture2DTransfer,
    *const c_void,
    u64,
) -> CNA_Result;
pub type cna_texture2d_get_data_fn = unsafe extern "C" fn(
    CNA_Handle,
    CNA_TextureDataType,
    *const CNA_Texture2DTransfer,
    *mut c_void,
    u64,
    *mut u64,
) -> CNA_Result;
pub type cna_texture2d_get_encoded_byte_count_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_TextureImageFormat, u32, u32, *mut u64) -> CNA_Result;
pub type cna_texture2d_copy_encoded_fn = unsafe extern "C" fn(
    CNA_Handle,
    CNA_TextureImageFormat,
    u32,
    u32,
    *mut u8,
    u64,
    *mut u64,
) -> CNA_Result;
pub type cna_texture2d_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_texture3d_create_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_Texture3DCreateInfo, *mut CNA_Handle) -> CNA_Result;
pub type cna_texture3d_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_texture3d_get_info_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Texture3DInfo) -> CNA_Result;
pub type cna_texture3d_set_data_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_Texture3DTransfer,
    *const CNA_Color,
    u64,
) -> CNA_Result;
pub type cna_texture3d_get_data_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_Texture3DTransfer,
    *mut CNA_Color,
    u64,
    *mut u64,
) -> CNA_Result;
pub type cna_texturecube_create_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_TextureCubeCreateInfo,
    *mut CNA_Handle,
) -> CNA_Result;
pub type cna_texturecube_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_texturecube_get_info_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_TextureCubeInfo) -> CNA_Result;
pub type cna_texturecube_set_data_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_TextureCubeTransfer,
    *const CNA_Color,
    u64,
) -> CNA_Result;
pub type cna_texturecube_get_data_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_TextureCubeTransfer,
    *mut CNA_Color,
    u64,
    *mut u64,
) -> CNA_Result;
pub type cna_render_target2d_create_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_RenderTarget2DCreateInfo,
    *mut CNA_Handle,
) -> CNA_Result;
pub type cna_render_target_cube_create_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_RenderTargetCubeCreateInfo,
    *mut CNA_Handle,
) -> CNA_Result;
pub type cna_render_target_get_info_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_RenderTargetInfo) -> CNA_Result;
pub type cna_render_target_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_vertex_declaration_create_fn = unsafe extern "C" fn(
    *const CNA_VertexElement,
    u64,
    *mut CNA_VertexDeclarationHandle,
) -> CNA_Result;
pub type cna_vertex_declaration_create_with_stride_fn = unsafe extern "C" fn(
    i32,
    *const CNA_VertexElement,
    u64,
    *mut CNA_VertexDeclarationHandle,
) -> CNA_Result;
pub type cna_vertex_declaration_destroy_fn =
    unsafe extern "C" fn(CNA_VertexDeclarationHandle) -> CNA_Result;
pub type cna_vertex_buffer_binding_init_fn = unsafe extern "C" fn(
    CNA_VertexBufferHandle,
    i32,
    i32,
    *mut CNA_VertexBufferBinding,
) -> CNA_Result;
pub type cna_vertex_buffer_create_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_VertexBufferCreateInfo,
    *mut CNA_VertexBufferHandle,
) -> CNA_Result;
pub type cna_vertex_buffer_destroy_fn = unsafe extern "C" fn(CNA_VertexBufferHandle) -> CNA_Result;
pub type cna_vertex_buffer_get_info_fn =
    unsafe extern "C" fn(CNA_VertexBufferHandle, *mut CNA_VertexBufferInfo) -> CNA_Result;
pub type cna_vertex_buffer_set_data_fn = unsafe extern "C" fn(
    CNA_VertexBufferHandle,
    *const CNA_VertexBufferTransfer,
    *const c_void,
    u64,
) -> CNA_Result;
pub type cna_vertex_buffer_set_data_raw_fn =
    unsafe extern "C" fn(CNA_VertexBufferHandle, *const c_void, u64, u64, u32) -> CNA_Result;
pub type cna_vertex_buffer_set_data_raw_at_fn =
    unsafe extern "C" fn(CNA_VertexBufferHandle, u64, *const c_void, u64, u64, u32) -> CNA_Result;
pub type cna_vertex_buffer_set_data_raw_with_options_fn = unsafe extern "C" fn(
    CNA_VertexBufferHandle,
    *const c_void,
    u64,
    u64,
    u32,
    CNA_SetDataOptions,
) -> CNA_Result;
pub type cna_vertex_buffer_set_data_raw_at_with_options_fn = unsafe extern "C" fn(
    CNA_VertexBufferHandle,
    u64,
    *const c_void,
    u64,
    u64,
    u32,
    CNA_SetDataOptions,
) -> CNA_Result;
pub type cna_vertex_buffer_get_data_raw_fn =
    unsafe extern "C" fn(CNA_VertexBufferHandle, u64, *mut c_void, u64, u64, u32) -> CNA_Result;
pub type cna_index_buffer_create_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_IndexBufferCreateInfo,
    *mut CNA_IndexBufferHandle,
) -> CNA_Result;
pub type cna_index_buffer_destroy_fn = unsafe extern "C" fn(CNA_IndexBufferHandle) -> CNA_Result;
pub type cna_index_buffer_get_info_fn =
    unsafe extern "C" fn(CNA_IndexBufferHandle, *mut CNA_IndexBufferInfo) -> CNA_Result;
pub type cna_index_buffer_set_data_fn = unsafe extern "C" fn(
    CNA_IndexBufferHandle,
    *const CNA_IndexBufferTransfer,
    *const c_void,
    u64,
) -> CNA_Result;
pub type cna_index_buffer_set_data_at_fn = unsafe extern "C" fn(
    CNA_IndexBufferHandle,
    u64,
    *const CNA_IndexBufferTransfer,
    *const c_void,
    u64,
) -> CNA_Result;
pub type cna_index_buffer_get_data_fn = unsafe extern "C" fn(
    CNA_IndexBufferHandle,
    *const CNA_IndexBufferTransfer,
    *mut c_void,
    u64,
    *mut u64,
) -> CNA_Result;
pub type cna_sprite_batch_create_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Handle) -> CNA_Result;
pub type cna_sprite_batch_begin_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_SpriteBatchBeginInfo) -> CNA_Result;
pub type cna_sprite_batch_begin_with_states_fn = unsafe extern "C" fn(
    CNA_Handle,
    CNA_SpriteSortMode,
    *const CNA_BlendState,
    *const CNA_SamplerState,
    *const CNA_DepthStencilState,
    *const CNA_RasterizerState,
) -> CNA_Result;
pub type cna_sprite_batch_begin_with_effect_fn = unsafe extern "C" fn(
    CNA_Handle,
    CNA_SpriteSortMode,
    *const CNA_BlendState,
    *const CNA_SamplerState,
    *const CNA_DepthStencilState,
    *const CNA_RasterizerState,
    CNA_Handle,
    *const CNA_Matrix,
) -> CNA_Result;
pub type cna_sprite_batch_submit_many_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_SpriteCommand, u64) -> CNA_Result;
pub type cna_sprite_batch_end_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_sprite_batch_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_sprite_batch_draw_string_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_SpriteTextCommand) -> CNA_Result;
pub type cna_sprite_font_create_fn =
    unsafe extern "C" fn(*const CNA_SpriteFontCreateInfo, *mut CNA_Handle) -> CNA_Result;
pub type cna_sprite_font_get_info_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_SpriteFontInfo) -> CNA_Result;
pub type cna_sprite_font_copy_characters_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Char16, u64, *mut u64) -> CNA_Result;
pub type cna_sprite_font_copy_glyphs_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_SpriteFontGlyph, u64, *mut u64) -> CNA_Result;
pub type cna_sprite_font_set_default_character_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Bool, CNA_Char16) -> CNA_Result;
pub type cna_sprite_font_set_line_spacing_fn = unsafe extern "C" fn(CNA_Handle, i32) -> CNA_Result;
pub type cna_sprite_font_set_spacing_fn = unsafe extern "C" fn(CNA_Handle, f32) -> CNA_Result;
pub type cna_sprite_font_measure_utf8_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView, *mut CNA_Vector2) -> CNA_Result;
pub type cna_sprite_font_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_effect_create_empty_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_EffectHandle) -> CNA_Result;
pub type cna_effect_create_compiled_fn =
    unsafe extern "C" fn(CNA_Handle, *const u8, u64, *mut CNA_EffectHandle) -> CNA_Result;
pub type cna_effect_material_create_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_EffectHandle) -> CNA_Result;
pub type cna_effect_destroy_fn = unsafe extern "C" fn(CNA_EffectHandle) -> CNA_Result;
pub type cna_effect_clone_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_EffectHandle) -> CNA_Result;
pub type cna_effect_dispose_fn = unsafe extern "C" fn(CNA_EffectHandle) -> CNA_Result;
pub type cna_effect_apply_fn = unsafe extern "C" fn(CNA_EffectHandle) -> CNA_Result;
pub type cna_effect_get_parameters_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_EffectParameterCollectionHandle) -> CNA_Result;
pub type cna_effect_get_techniques_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_EffectTechniqueCollectionHandle) -> CNA_Result;
pub type cna_effect_get_current_technique_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_EffectTechniqueHandle) -> CNA_Result;
pub type cna_effect_set_current_technique_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_EffectTechniqueHandle) -> CNA_Result;

pub type cna_directional_light_create_fn =
    unsafe extern "C" fn(*mut CNA_DirectionalLightHandle) -> CNA_Result;
pub type cna_directional_light_destroy_fn =
    unsafe extern "C" fn(CNA_DirectionalLightHandle) -> CNA_Result;
pub type cna_directional_light_get_diffuse_color_fn =
    unsafe extern "C" fn(CNA_DirectionalLightHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_directional_light_set_diffuse_color_fn =
    unsafe extern "C" fn(CNA_DirectionalLightHandle, CNA_Vector3) -> CNA_Result;
pub type cna_directional_light_get_direction_fn =
    unsafe extern "C" fn(CNA_DirectionalLightHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_directional_light_set_direction_fn =
    unsafe extern "C" fn(CNA_DirectionalLightHandle, CNA_Vector3) -> CNA_Result;
pub type cna_directional_light_get_specular_color_fn =
    unsafe extern "C" fn(CNA_DirectionalLightHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_directional_light_set_specular_color_fn =
    unsafe extern "C" fn(CNA_DirectionalLightHandle, CNA_Vector3) -> CNA_Result;
pub type cna_directional_light_get_enabled_fn =
    unsafe extern "C" fn(CNA_DirectionalLightHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_directional_light_set_enabled_fn =
    unsafe extern "C" fn(CNA_DirectionalLightHandle, CNA_Bool) -> CNA_Result;
pub type cna_basic_effect_create_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_EffectHandle) -> CNA_Result;
pub type cna_effect_matrices_get_world_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Matrix) -> CNA_Result;
pub type cna_effect_matrices_set_world_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Matrix) -> CNA_Result;
pub type cna_effect_matrices_get_view_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Matrix) -> CNA_Result;
pub type cna_effect_matrices_set_view_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Matrix) -> CNA_Result;
pub type cna_effect_matrices_get_projection_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Matrix) -> CNA_Result;
pub type cna_effect_matrices_set_projection_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Matrix) -> CNA_Result;
pub type cna_effect_fog_get_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_effect_fog_set_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Vector3) -> CNA_Result;
pub type cna_effect_fog_get_enabled_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_effect_fog_set_enabled_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Bool) -> CNA_Result;
pub type cna_effect_fog_get_start_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut f32) -> CNA_Result;
pub type cna_effect_fog_set_start_fn = unsafe extern "C" fn(CNA_EffectHandle, f32) -> CNA_Result;
pub type cna_effect_fog_get_end_fn = unsafe extern "C" fn(CNA_EffectHandle, *mut f32) -> CNA_Result;
pub type cna_effect_fog_set_end_fn = unsafe extern "C" fn(CNA_EffectHandle, f32) -> CNA_Result;
pub type cna_effect_lights_get_ambient_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_effect_lights_set_ambient_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Vector3) -> CNA_Result;
pub type cna_effect_lights_get_directional_light_fn =
    unsafe extern "C" fn(CNA_EffectHandle, u32, *mut CNA_DirectionalLightHandle) -> CNA_Result;
pub type cna_effect_lights_get_enabled_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_effect_lights_set_enabled_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Bool) -> CNA_Result;
pub type cna_effect_lights_enable_default_fn = unsafe extern "C" fn(CNA_EffectHandle) -> CNA_Result;
pub type cna_basic_effect_get_vertex_color_enabled_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_basic_effect_set_vertex_color_enabled_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Bool) -> CNA_Result;
pub type cna_basic_effect_get_prefer_per_pixel_lighting_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_basic_effect_set_prefer_per_pixel_lighting_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Bool) -> CNA_Result;
pub type cna_basic_effect_get_diffuse_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_basic_effect_set_diffuse_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Vector3) -> CNA_Result;
pub type cna_basic_effect_get_emissive_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_basic_effect_set_emissive_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Vector3) -> CNA_Result;
pub type cna_basic_effect_get_specular_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_basic_effect_set_specular_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Vector3) -> CNA_Result;
pub type cna_basic_effect_get_specular_power_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut f32) -> CNA_Result;
pub type cna_basic_effect_set_specular_power_fn =
    unsafe extern "C" fn(CNA_EffectHandle, f32) -> CNA_Result;
pub type cna_basic_effect_get_alpha_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut f32) -> CNA_Result;
pub type cna_basic_effect_set_alpha_fn = unsafe extern "C" fn(CNA_EffectHandle, f32) -> CNA_Result;
pub type cna_basic_effect_get_texture_enabled_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_basic_effect_set_texture_enabled_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Bool) -> CNA_Result;
pub type cna_basic_effect_set_texture_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Handle) -> CNA_Result;
pub type cna_alpha_test_effect_create_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_EffectHandle) -> CNA_Result;
pub type cna_alpha_test_effect_get_diffuse_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_alpha_test_effect_set_diffuse_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Vector3) -> CNA_Result;
pub type cna_alpha_test_effect_get_alpha_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut f32) -> CNA_Result;
pub type cna_alpha_test_effect_set_alpha_fn =
    unsafe extern "C" fn(CNA_EffectHandle, f32) -> CNA_Result;
pub type cna_alpha_test_effect_set_texture_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Handle) -> CNA_Result;
pub type cna_alpha_test_effect_get_vertex_color_enabled_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_alpha_test_effect_set_vertex_color_enabled_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Bool) -> CNA_Result;
pub type cna_alpha_test_effect_get_alpha_function_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_CompareFunction) -> CNA_Result;
pub type cna_alpha_test_effect_set_alpha_function_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_CompareFunction) -> CNA_Result;
pub type cna_alpha_test_effect_get_reference_alpha_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut i32) -> CNA_Result;
pub type cna_alpha_test_effect_set_reference_alpha_fn =
    unsafe extern "C" fn(CNA_EffectHandle, i32) -> CNA_Result;
pub type cna_dual_texture_effect_create_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_EffectHandle) -> CNA_Result;
pub type cna_dual_texture_effect_get_diffuse_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_dual_texture_effect_set_diffuse_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Vector3) -> CNA_Result;
pub type cna_dual_texture_effect_get_alpha_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut f32) -> CNA_Result;
pub type cna_dual_texture_effect_set_alpha_fn =
    unsafe extern "C" fn(CNA_EffectHandle, f32) -> CNA_Result;
pub type cna_dual_texture_effect_set_texture_fn =
    unsafe extern "C" fn(CNA_EffectHandle, u32, CNA_Handle) -> CNA_Result;
pub type cna_dual_texture_effect_get_vertex_color_enabled_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_dual_texture_effect_set_vertex_color_enabled_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Bool) -> CNA_Result;
pub type cna_environment_map_effect_create_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_EffectHandle) -> CNA_Result;
pub type cna_environment_map_effect_get_diffuse_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_environment_map_effect_set_diffuse_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Vector3) -> CNA_Result;
pub type cna_environment_map_effect_get_emissive_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_environment_map_effect_set_emissive_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Vector3) -> CNA_Result;
pub type cna_environment_map_effect_get_alpha_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut f32) -> CNA_Result;
pub type cna_environment_map_effect_set_alpha_fn =
    unsafe extern "C" fn(CNA_EffectHandle, f32) -> CNA_Result;
pub type cna_environment_map_effect_set_texture_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Handle) -> CNA_Result;
pub type cna_environment_map_effect_set_environment_map_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Handle) -> CNA_Result;
pub type cna_environment_map_effect_get_amount_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut f32) -> CNA_Result;
pub type cna_environment_map_effect_set_amount_fn =
    unsafe extern "C" fn(CNA_EffectHandle, f32) -> CNA_Result;
pub type cna_environment_map_effect_get_specular_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_environment_map_effect_set_specular_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Vector3) -> CNA_Result;
pub type cna_environment_map_effect_get_fresnel_factor_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut f32) -> CNA_Result;
pub type cna_environment_map_effect_set_fresnel_factor_fn =
    unsafe extern "C" fn(CNA_EffectHandle, f32) -> CNA_Result;
pub type cna_skinned_effect_create_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_EffectHandle) -> CNA_Result;
pub type cna_skinned_effect_get_diffuse_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_skinned_effect_set_diffuse_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Vector3) -> CNA_Result;
pub type cna_skinned_effect_get_emissive_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_skinned_effect_set_emissive_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Vector3) -> CNA_Result;
pub type cna_skinned_effect_get_specular_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_skinned_effect_set_specular_color_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Vector3) -> CNA_Result;
pub type cna_skinned_effect_get_specular_power_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut f32) -> CNA_Result;
pub type cna_skinned_effect_set_specular_power_fn =
    unsafe extern "C" fn(CNA_EffectHandle, f32) -> CNA_Result;
pub type cna_skinned_effect_get_alpha_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut f32) -> CNA_Result;
pub type cna_skinned_effect_set_alpha_fn =
    unsafe extern "C" fn(CNA_EffectHandle, f32) -> CNA_Result;
pub type cna_skinned_effect_get_prefer_per_pixel_lighting_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_skinned_effect_set_prefer_per_pixel_lighting_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Bool) -> CNA_Result;
pub type cna_skinned_effect_set_texture_fn =
    unsafe extern "C" fn(CNA_EffectHandle, CNA_Handle) -> CNA_Result;
pub type cna_skinned_effect_get_weights_per_vertex_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *mut i32) -> CNA_Result;
pub type cna_skinned_effect_set_weights_per_vertex_fn =
    unsafe extern "C" fn(CNA_EffectHandle, i32) -> CNA_Result;
pub type cna_skinned_effect_set_bone_transforms_fn =
    unsafe extern "C" fn(CNA_EffectHandle, *const CNA_Matrix, u64) -> CNA_Result;
pub type cna_skinned_effect_copy_bone_transforms_fn =
    unsafe extern "C" fn(CNA_EffectHandle, u64, *mut CNA_Matrix, u64, *mut u64) -> CNA_Result;

pub type cna_effect_annotation_destroy_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle) -> CNA_Result;
pub type cna_effect_annotation_create_fn = unsafe extern "C" fn(
    *const CNA_EffectAnnotationCreateInfo,
    *mut CNA_EffectAnnotationHandle,
) -> CNA_Result;
pub type cna_effect_annotation_get_info_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut CNA_EffectAnnotationInfo) -> CNA_Result;
pub type cna_effect_annotation_get_name_byte_count_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut u64) -> CNA_Result;
pub type cna_effect_annotation_copy_name_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_effect_annotation_get_semantic_byte_count_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut u64) -> CNA_Result;
pub type cna_effect_annotation_copy_semantic_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_effect_annotation_get_value_boolean_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_effect_annotation_get_value_int32_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut i32) -> CNA_Result;
pub type cna_effect_annotation_get_value_single_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut f32) -> CNA_Result;
pub type cna_effect_annotation_get_value_string_byte_count_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut u64) -> CNA_Result;
pub type cna_effect_annotation_copy_value_string_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_effect_annotation_get_value_vector2_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut CNA_Vector2) -> CNA_Result;
pub type cna_effect_annotation_get_value_vector3_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut CNA_Vector3) -> CNA_Result;
pub type cna_effect_annotation_get_value_vector4_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut CNA_Vector4) -> CNA_Result;
pub type cna_effect_annotation_get_value_matrix_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationHandle, *mut CNA_Matrix) -> CNA_Result;
pub type cna_effect_annotation_collection_destroy_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationCollectionHandle) -> CNA_Result;
pub type cna_effect_annotation_collection_add_fn = unsafe extern "C" fn(
    CNA_EffectAnnotationCollectionHandle,
    CNA_EffectAnnotationHandle,
) -> CNA_Result;
pub type cna_effect_annotation_collection_get_count_fn =
    unsafe extern "C" fn(CNA_EffectAnnotationCollectionHandle, *mut u64) -> CNA_Result;
pub type cna_effect_annotation_collection_get_at_fn = unsafe extern "C" fn(
    CNA_EffectAnnotationCollectionHandle,
    u64,
    *mut CNA_EffectAnnotationHandle,
) -> CNA_Result;
pub type cna_effect_annotation_collection_find_fn = unsafe extern "C" fn(
    CNA_EffectAnnotationCollectionHandle,
    CNA_StringView,
    *mut CNA_Bool,
    *mut CNA_EffectAnnotationHandle,
) -> CNA_Result;

pub type cna_effect_parameter_destroy_fn =
    unsafe extern "C" fn(CNA_EffectParameterHandle) -> CNA_Result;
pub type cna_effect_parameter_get_info_fn =
    unsafe extern "C" fn(CNA_EffectParameterHandle, *mut CNA_EffectParameterInfo) -> CNA_Result;
pub type cna_effect_parameter_get_name_byte_count_fn =
    unsafe extern "C" fn(CNA_EffectParameterHandle, *mut u64) -> CNA_Result;
pub type cna_effect_parameter_copy_name_fn =
    unsafe extern "C" fn(CNA_EffectParameterHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_effect_parameter_get_semantic_byte_count_fn =
    unsafe extern "C" fn(CNA_EffectParameterHandle, *mut u64) -> CNA_Result;
pub type cna_effect_parameter_copy_semantic_fn =
    unsafe extern "C" fn(CNA_EffectParameterHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_effect_parameter_get_elements_fn = unsafe extern "C" fn(
    CNA_EffectParameterHandle,
    *mut CNA_EffectParameterCollectionHandle,
) -> CNA_Result;
pub type cna_effect_parameter_get_structure_members_fn = unsafe extern "C" fn(
    CNA_EffectParameterHandle,
    *mut CNA_EffectParameterCollectionHandle,
) -> CNA_Result;
pub type cna_effect_parameter_get_annotations_fn = unsafe extern "C" fn(
    CNA_EffectParameterHandle,
    *mut CNA_EffectAnnotationCollectionHandle,
) -> CNA_Result;
pub type cna_effect_parameter_get_value_fn =
    unsafe extern "C" fn(CNA_EffectParameterHandle, CNA_EffectValueType, *mut c_void) -> CNA_Result;
pub type cna_effect_parameter_get_values_fn = unsafe extern "C" fn(
    CNA_EffectParameterHandle,
    CNA_EffectValueType,
    u64,
    *mut c_void,
    u64,
    *mut u64,
) -> CNA_Result;
pub type cna_effect_parameter_set_value_fn = unsafe extern "C" fn(
    CNA_EffectParameterHandle,
    CNA_EffectValueType,
    *const c_void,
) -> CNA_Result;
pub type cna_effect_parameter_set_values_fn = unsafe extern "C" fn(
    CNA_EffectParameterHandle,
    CNA_EffectValueType,
    *const c_void,
    u64,
) -> CNA_Result;
pub type cna_effect_parameter_get_value_string_byte_count_fn =
    unsafe extern "C" fn(CNA_EffectParameterHandle, *mut u64) -> CNA_Result;
pub type cna_effect_parameter_copy_value_string_fn =
    unsafe extern "C" fn(CNA_EffectParameterHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_effect_parameter_set_value_string_fn =
    unsafe extern "C" fn(CNA_EffectParameterHandle, CNA_StringView) -> CNA_Result;
pub type cna_effect_parameter_get_value_texture_fn = unsafe extern "C" fn(
    CNA_EffectParameterHandle,
    CNA_EffectTextureType,
    *mut CNA_Handle,
) -> CNA_Result;
pub type cna_effect_parameter_set_value_texture_fn = unsafe extern "C" fn(
    CNA_EffectParameterHandle,
    CNA_EffectTextureType,
    CNA_Handle,
) -> CNA_Result;
pub type cna_effect_parameter_collection_destroy_fn =
    unsafe extern "C" fn(CNA_EffectParameterCollectionHandle) -> CNA_Result;
pub type cna_effect_parameter_collection_add_create_fn = unsafe extern "C" fn(
    CNA_EffectParameterCollectionHandle,
    *const CNA_EffectParameterCreateInfo,
    *mut CNA_EffectParameterHandle,
) -> CNA_Result;
pub type cna_effect_parameter_collection_get_count_fn =
    unsafe extern "C" fn(CNA_EffectParameterCollectionHandle, *mut u64) -> CNA_Result;
pub type cna_effect_parameter_collection_get_at_fn = unsafe extern "C" fn(
    CNA_EffectParameterCollectionHandle,
    u64,
    *mut CNA_EffectParameterHandle,
) -> CNA_Result;
pub type cna_effect_parameter_collection_find_name_fn = unsafe extern "C" fn(
    CNA_EffectParameterCollectionHandle,
    CNA_StringView,
    *mut CNA_Bool,
    *mut CNA_EffectParameterHandle,
) -> CNA_Result;
pub type cna_effect_parameter_collection_find_semantic_fn = unsafe extern "C" fn(
    CNA_EffectParameterCollectionHandle,
    CNA_StringView,
    *mut CNA_Bool,
    *mut CNA_EffectParameterHandle,
) -> CNA_Result;

pub type cna_effect_pass_destroy_fn = unsafe extern "C" fn(CNA_EffectPassHandle) -> CNA_Result;
pub type cna_effect_pass_get_name_byte_count_fn =
    unsafe extern "C" fn(CNA_EffectPassHandle, *mut u64) -> CNA_Result;
pub type cna_effect_pass_copy_name_fn =
    unsafe extern "C" fn(CNA_EffectPassHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_effect_pass_get_annotations_fn = unsafe extern "C" fn(
    CNA_EffectPassHandle,
    *mut CNA_EffectAnnotationCollectionHandle,
) -> CNA_Result;
pub type cna_effect_pass_apply_fn = unsafe extern "C" fn(CNA_EffectPassHandle) -> CNA_Result;
pub type cna_effect_pass_collection_destroy_fn =
    unsafe extern "C" fn(CNA_EffectPassCollectionHandle) -> CNA_Result;
pub type cna_effect_pass_collection_add_create_fn = unsafe extern "C" fn(
    CNA_EffectPassCollectionHandle,
    CNA_StringView,
    u64,
    *mut CNA_EffectPassHandle,
) -> CNA_Result;
pub type cna_effect_pass_collection_get_count_fn =
    unsafe extern "C" fn(CNA_EffectPassCollectionHandle, *mut u64) -> CNA_Result;
pub type cna_effect_pass_collection_get_at_fn = unsafe extern "C" fn(
    CNA_EffectPassCollectionHandle,
    u64,
    *mut CNA_EffectPassHandle,
) -> CNA_Result;
pub type cna_effect_pass_collection_find_fn = unsafe extern "C" fn(
    CNA_EffectPassCollectionHandle,
    CNA_StringView,
    *mut CNA_Bool,
    *mut CNA_EffectPassHandle,
) -> CNA_Result;

pub type cna_effect_technique_destroy_fn =
    unsafe extern "C" fn(CNA_EffectTechniqueHandle) -> CNA_Result;
pub type cna_effect_technique_get_name_byte_count_fn =
    unsafe extern "C" fn(CNA_EffectTechniqueHandle, *mut u64) -> CNA_Result;
pub type cna_effect_technique_copy_name_fn =
    unsafe extern "C" fn(CNA_EffectTechniqueHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_effect_technique_get_passes_fn = unsafe extern "C" fn(
    CNA_EffectTechniqueHandle,
    *mut CNA_EffectPassCollectionHandle,
) -> CNA_Result;
pub type cna_effect_technique_get_annotations_fn = unsafe extern "C" fn(
    CNA_EffectTechniqueHandle,
    *mut CNA_EffectAnnotationCollectionHandle,
) -> CNA_Result;
pub type cna_effect_technique_collection_destroy_fn =
    unsafe extern "C" fn(CNA_EffectTechniqueCollectionHandle) -> CNA_Result;
pub type cna_effect_technique_collection_add_named_fn = unsafe extern "C" fn(
    CNA_EffectTechniqueCollectionHandle,
    CNA_StringView,
    *mut CNA_EffectTechniqueHandle,
) -> CNA_Result;
pub type cna_effect_technique_collection_get_count_fn =
    unsafe extern "C" fn(CNA_EffectTechniqueCollectionHandle, *mut u64) -> CNA_Result;
pub type cna_effect_technique_collection_get_at_fn = unsafe extern "C" fn(
    CNA_EffectTechniqueCollectionHandle,
    u64,
    *mut CNA_EffectTechniqueHandle,
) -> CNA_Result;
pub type cna_effect_technique_collection_find_fn = unsafe extern "C" fn(
    CNA_EffectTechniqueCollectionHandle,
    CNA_StringView,
    *mut CNA_Bool,
    *mut CNA_EffectTechniqueHandle,
) -> CNA_Result;
pub type cna_keyboard_get_state_for_player_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_PlayerIndex, *mut CNA_KeyboardState) -> CNA_Result;
pub type cna_keyboard_get_state_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_KeyboardState) -> CNA_Result;
pub type cna_keyboard_state_is_key_down_fn =
    unsafe extern "C" fn(*const CNA_KeyboardState, CNA_Key, *mut CNA_Bool) -> CNA_Result;
pub type cna_mouse_get_state_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_MouseState) -> CNA_Result;
pub type cna_mouse_get_window_handle_fn = unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_mouse_set_window_handle_fn = unsafe extern "C" fn(CNA_Handle, u64) -> CNA_Result;
pub type cna_mouse_set_position_fn = unsafe extern "C" fn(CNA_Handle, i32, i32) -> CNA_Result;
pub type cna_gamepad_get_state_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_PlayerIndex, *mut CNA_GamePadState) -> CNA_Result;
pub type cna_gamepad_get_state_with_dead_zone_fn = unsafe extern "C" fn(
    CNA_Handle,
    CNA_PlayerIndex,
    CNA_GamePadDeadZone,
    *mut CNA_GamePadState,
) -> CNA_Result;
pub type cna_gamepad_get_capabilities_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_PlayerIndex, *mut CNA_GamePadCapabilities) -> CNA_Result;
pub type cna_gamepad_set_vibration_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_PlayerIndex, f32, f32, *mut CNA_Bool) -> CNA_Result;
pub type cna_touch_get_capabilities_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_TouchCapabilities) -> CNA_Result;
pub type cna_touch_get_state_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_TouchState) -> CNA_Result;
pub type cna_touch_panel_get_display_width_fn =
    unsafe extern "C" fn(CNA_Handle, *mut i32) -> CNA_Result;
pub type cna_touch_panel_set_display_width_fn = unsafe extern "C" fn(CNA_Handle, i32) -> CNA_Result;
pub type cna_touch_panel_get_display_height_fn =
    unsafe extern "C" fn(CNA_Handle, *mut i32) -> CNA_Result;
pub type cna_touch_panel_set_display_height_fn =
    unsafe extern "C" fn(CNA_Handle, i32) -> CNA_Result;
pub type cna_touch_panel_get_display_orientation_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_DisplayOrientation) -> CNA_Result;
pub type cna_touch_panel_set_display_orientation_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_DisplayOrientation) -> CNA_Result;
pub type cna_touch_panel_get_enabled_gestures_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_GestureType) -> CNA_Result;
pub type cna_touch_panel_set_enabled_gestures_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_GestureType) -> CNA_Result;
pub type cna_touch_panel_get_is_gesture_available_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_touch_panel_get_window_handle_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_touch_panel_set_window_handle_fn = unsafe extern "C" fn(CNA_Handle, u64) -> CNA_Result;
pub type cna_touch_panel_read_gesture_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_GestureSample) -> CNA_Result;
pub type cna_graphics_device_manager_create_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_GraphicsDeviceManagerHandle) -> CNA_Result;
pub type cna_graphics_device_manager_get_graphics_profile_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, *mut CNA_GraphicsProfile) -> CNA_Result;
pub type cna_graphics_device_manager_set_graphics_profile_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, CNA_GraphicsProfile) -> CNA_Result;
pub type cna_graphics_device_manager_get_is_full_screen_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_graphics_device_manager_set_is_full_screen_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, CNA_Bool) -> CNA_Result;
pub type cna_graphics_device_manager_get_prefer_multi_sampling_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_graphics_device_manager_set_prefer_multi_sampling_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, CNA_Bool) -> CNA_Result;
pub type cna_graphics_device_manager_get_preferred_back_buffer_format_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, *mut CNA_SurfaceFormat) -> CNA_Result;
pub type cna_graphics_device_manager_set_preferred_back_buffer_format_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, CNA_SurfaceFormat) -> CNA_Result;
pub type cna_graphics_device_manager_get_preferred_back_buffer_width_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, *mut i32) -> CNA_Result;
pub type cna_graphics_device_manager_set_preferred_back_buffer_width_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, i32) -> CNA_Result;
pub type cna_graphics_device_manager_get_preferred_back_buffer_height_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, *mut i32) -> CNA_Result;
pub type cna_graphics_device_manager_set_preferred_back_buffer_height_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, i32) -> CNA_Result;
pub type cna_graphics_device_manager_get_preferred_depth_stencil_format_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, *mut CNA_DepthFormat) -> CNA_Result;
pub type cna_graphics_device_manager_set_preferred_depth_stencil_format_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, CNA_DepthFormat) -> CNA_Result;
pub type cna_graphics_device_manager_get_synchronize_with_vertical_retrace_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_graphics_device_manager_set_synchronize_with_vertical_retrace_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, CNA_Bool) -> CNA_Result;
pub type cna_graphics_device_manager_get_supported_orientations_fn =
    unsafe extern "C" fn(
        CNA_GraphicsDeviceManagerHandle,
        *mut CNA_DisplayOrientation,
    ) -> CNA_Result;
pub type cna_graphics_device_manager_set_supported_orientations_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, CNA_DisplayOrientation) -> CNA_Result;
pub type cna_graphics_device_manager_apply_changes_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle) -> CNA_Result;
pub type cna_graphics_device_manager_toggle_full_screen_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle) -> CNA_Result;
pub type cna_graphics_device_manager_create_device_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle) -> CNA_Result;
pub type cna_graphics_device_manager_begin_draw_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_graphics_device_manager_end_draw_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle) -> CNA_Result;
pub type cna_graphics_device_manager_dispose_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle) -> CNA_Result;
pub type cna_graphics_device_manager_subscribe_fn = unsafe extern "C" fn(
    CNA_GraphicsDeviceManagerHandle,
    CNA_GraphicsDeviceManagerEvent,
    CNA_GameEventCallback,
    *mut c_void,
    *mut CNA_GameEventRegistrationHandle,
) -> CNA_Result;
pub type cna_graphics_device_manager_subscribe_preparing_device_settings_ext_fn =
    unsafe extern "C" fn(
        CNA_GraphicsDeviceManagerHandle,
        CNA_PreparingDeviceSettingsMutatorEXT,
        *mut c_void,
        *mut CNA_GameEventRegistrationHandle,
    ) -> CNA_Result;
pub type cna_graphics_device_manager_destroy_fn =
    unsafe extern "C" fn(CNA_GraphicsDeviceManagerHandle) -> CNA_Result;
pub type cna_storage_device_show_selector_fn = unsafe extern "C" fn(
    CNA_StorageCompletionCallback,
    *mut c_void,
    *mut CNA_StorageDeviceHandle,
) -> CNA_Result;
pub type cna_storage_device_show_selector_for_player_fn = unsafe extern "C" fn(
    CNA_PlayerIndex,
    CNA_StorageCompletionCallback,
    *mut c_void,
    *mut CNA_StorageDeviceHandle,
) -> CNA_Result;
pub type cna_storage_device_show_selector_with_space_fn = unsafe extern "C" fn(
    i32,
    i32,
    CNA_StorageCompletionCallback,
    *mut c_void,
    *mut CNA_StorageDeviceHandle,
) -> CNA_Result;
pub type cna_storage_device_show_selector_for_player_with_space_fn =
    unsafe extern "C" fn(
        CNA_PlayerIndex,
        i32,
        i32,
        CNA_StorageCompletionCallback,
        *mut c_void,
        *mut CNA_StorageDeviceHandle,
    ) -> CNA_Result;
pub type cna_storage_device_get_free_space_fn =
    unsafe extern "C" fn(CNA_StorageDeviceHandle, *mut i64) -> CNA_Result;
pub type cna_storage_device_get_is_connected_fn =
    unsafe extern "C" fn(CNA_StorageDeviceHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_storage_device_get_total_space_fn =
    unsafe extern "C" fn(CNA_StorageDeviceHandle, *mut i64) -> CNA_Result;
pub type cna_storage_device_delete_container_fn =
    unsafe extern "C" fn(CNA_StorageDeviceHandle, CNA_StringView) -> CNA_Result;
pub type cna_storage_device_subscribe_device_changed_fn =
    unsafe extern "C" fn(CNA_StorageCompletionCallback, *mut c_void, *mut CNA_Handle) -> CNA_Result;
pub type cna_storage_device_unsubscribe_device_changed_fn =
    unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_storage_device_destroy_fn =
    unsafe extern "C" fn(CNA_StorageDeviceHandle) -> CNA_Result;
pub type cna_storage_container_open_fn = unsafe extern "C" fn(
    CNA_StorageDeviceHandle,
    CNA_StringView,
    CNA_StorageCompletionCallback,
    *mut c_void,
    *mut CNA_StorageContainerHandle,
) -> CNA_Result;
pub type cna_storage_container_get_display_name_size_fn =
    unsafe extern "C" fn(CNA_StorageContainerHandle, *mut u64) -> CNA_Result;
pub type cna_storage_container_copy_display_name_fn =
    unsafe extern "C" fn(CNA_StorageContainerHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_storage_container_dispose_fn =
    unsafe extern "C" fn(CNA_StorageContainerHandle) -> CNA_Result;
pub type cna_storage_container_subscribe_disposing_fn = unsafe extern "C" fn(
    CNA_StorageContainerHandle,
    CNA_StorageCompletionCallback,
    *mut c_void,
    *mut CNA_Handle,
) -> CNA_Result;
pub type cna_storage_container_unsubscribe_disposing_fn =
    unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_storage_container_create_directory_fn =
    unsafe extern "C" fn(CNA_StorageContainerHandle, CNA_StringView) -> CNA_Result;
pub type cna_storage_container_directory_exists_fn =
    unsafe extern "C" fn(CNA_StorageContainerHandle, CNA_StringView, *mut CNA_Bool) -> CNA_Result;
pub type cna_storage_container_delete_directory_fn =
    unsafe extern "C" fn(CNA_StorageContainerHandle, CNA_StringView) -> CNA_Result;
pub type cna_storage_container_file_exists_fn =
    unsafe extern "C" fn(CNA_StorageContainerHandle, CNA_StringView, *mut CNA_Bool) -> CNA_Result;
pub type cna_storage_container_delete_file_fn =
    unsafe extern "C" fn(CNA_StorageContainerHandle, CNA_StringView) -> CNA_Result;
pub type cna_storage_container_get_directory_name_count_fn =
    unsafe extern "C" fn(CNA_StorageContainerHandle, CNA_StringView, *mut u64) -> CNA_Result;
pub type cna_storage_container_copy_directory_name_fn = unsafe extern "C" fn(
    CNA_StorageContainerHandle,
    CNA_StringView,
    u64,
    *mut c_char,
    u64,
    *mut u64,
) -> CNA_Result;
pub type cna_storage_container_get_file_name_count_fn =
    unsafe extern "C" fn(CNA_StorageContainerHandle, CNA_StringView, *mut u64) -> CNA_Result;
pub type cna_storage_container_copy_file_name_fn = unsafe extern "C" fn(
    CNA_StorageContainerHandle,
    CNA_StringView,
    u64,
    *mut c_char,
    u64,
    *mut u64,
) -> CNA_Result;
pub type cna_storage_container_create_file_fn = unsafe extern "C" fn(
    CNA_StorageContainerHandle,
    CNA_StringView,
    *mut CNA_StorageStreamHandle,
) -> CNA_Result;
pub type cna_storage_container_open_file_fn = unsafe extern "C" fn(
    CNA_StorageContainerHandle,
    CNA_StringView,
    CNA_FileMode,
    *mut CNA_StorageStreamHandle,
) -> CNA_Result;
pub type cna_storage_container_open_file_access_fn = unsafe extern "C" fn(
    CNA_StorageContainerHandle,
    CNA_StringView,
    CNA_FileMode,
    CNA_FileAccess,
    *mut CNA_StorageStreamHandle,
) -> CNA_Result;
pub type cna_storage_container_open_file_share_fn = unsafe extern "C" fn(
    CNA_StorageContainerHandle,
    CNA_StringView,
    CNA_FileMode,
    CNA_FileAccess,
    CNA_FileShare,
    *mut CNA_StorageStreamHandle,
) -> CNA_Result;
pub type cna_storage_container_destroy_fn =
    unsafe extern "C" fn(CNA_StorageContainerHandle) -> CNA_Result;
pub type cna_storage_stream_read_fn =
    unsafe extern "C" fn(CNA_StorageStreamHandle, *mut u8, u64, *mut u64) -> CNA_Result;
pub type cna_storage_stream_write_fn =
    unsafe extern "C" fn(CNA_StorageStreamHandle, *const u8, u64) -> CNA_Result;
pub type cna_storage_stream_seek_fn =
    unsafe extern "C" fn(CNA_StorageStreamHandle, i64, CNA_SeekOrigin, *mut i64) -> CNA_Result;
pub type cna_storage_stream_get_position_fn =
    unsafe extern "C" fn(CNA_StorageStreamHandle, *mut i64) -> CNA_Result;
pub type cna_storage_stream_get_length_fn =
    unsafe extern "C" fn(CNA_StorageStreamHandle, *mut i64) -> CNA_Result;
pub type cna_storage_stream_set_length_fn =
    unsafe extern "C" fn(CNA_StorageStreamHandle, i64) -> CNA_Result;
pub type cna_storage_stream_get_can_read_fn =
    unsafe extern "C" fn(CNA_StorageStreamHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_storage_stream_get_can_write_fn =
    unsafe extern "C" fn(CNA_StorageStreamHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_storage_stream_get_can_seek_fn =
    unsafe extern "C" fn(CNA_StorageStreamHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_storage_stream_flush_fn = unsafe extern "C" fn(CNA_StorageStreamHandle) -> CNA_Result;
pub type cna_storage_stream_close_fn = unsafe extern "C" fn(CNA_StorageStreamHandle) -> CNA_Result;

pub type cna_framework_dispatcher_update_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_sound_effect_create_pcm16_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_SoundEffectCreateInfo,
    *const u8,
    u64,
    *mut CNA_Handle,
) -> CNA_Result;
pub type cna_sound_effect_create_pcm16_range_ext_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_SoundEffectCreateInfo,
    *const u8,
    u64,
    i32,
    i32,
    i32,
    i32,
    *mut CNA_Handle,
) -> CNA_Result;
pub type cna_sound_effect_create_from_encoded_ext_fn =
    unsafe extern "C" fn(CNA_Handle, *const u8, u64, *mut CNA_Handle) -> CNA_Result;
pub type cna_sound_effect_get_duration_ticks_fn =
    unsafe extern "C" fn(CNA_Handle, *mut i64) -> CNA_Result;
pub type cna_sound_effect_create_instance_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Handle) -> CNA_Result;
pub type cna_sound_effect_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_sound_effect_get_name_size_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_sound_effect_copy_name_fn =
    unsafe extern "C" fn(CNA_Handle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_sound_effect_set_name_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView) -> CNA_Result;
pub type cna_sound_effect_get_master_volume_fn =
    unsafe extern "C" fn(CNA_Handle, *mut f32) -> CNA_Result;
pub type cna_sound_effect_set_master_volume_fn =
    unsafe extern "C" fn(CNA_Handle, f32) -> CNA_Result;
pub type cna_sound_effect_get_distance_scale_fn =
    unsafe extern "C" fn(CNA_Handle, *mut f32) -> CNA_Result;
pub type cna_sound_effect_set_distance_scale_fn =
    unsafe extern "C" fn(CNA_Handle, f32) -> CNA_Result;
pub type cna_sound_effect_get_doppler_scale_fn =
    unsafe extern "C" fn(CNA_Handle, *mut f32) -> CNA_Result;
pub type cna_sound_effect_set_doppler_scale_fn =
    unsafe extern "C" fn(CNA_Handle, f32) -> CNA_Result;
pub type cna_sound_effect_get_speed_of_sound_fn =
    unsafe extern "C" fn(CNA_Handle, *mut f32) -> CNA_Result;
pub type cna_sound_effect_set_speed_of_sound_fn =
    unsafe extern "C" fn(CNA_Handle, f32) -> CNA_Result;
pub type cna_sound_effect_play_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_sound_effect_play_with_settings_fn =
    unsafe extern "C" fn(CNA_Handle, f32, f32, f32, *mut CNA_Bool) -> CNA_Result;
pub type cna_sound_effect_instance_play_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_sound_effect_instance_pause_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_sound_effect_instance_resume_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_sound_effect_instance_stop_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Bool) -> CNA_Result;
pub type cna_sound_effect_instance_get_info_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_SoundEffectInstanceInfo) -> CNA_Result;
pub type cna_sound_effect_instance_set_volume_fn =
    unsafe extern "C" fn(CNA_Handle, f32) -> CNA_Result;
pub type cna_sound_effect_instance_set_pitch_fn =
    unsafe extern "C" fn(CNA_Handle, f32) -> CNA_Result;
pub type cna_sound_effect_instance_set_pan_fn =
    unsafe extern "C" fn(CNA_Handle, f32) -> CNA_Result;
pub type cna_sound_effect_instance_set_is_looped_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Bool) -> CNA_Result;
pub type cna_sound_effect_instance_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_sound_effect_instance_apply_3d_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_AudioListener,
    *const CNA_AudioEmitter,
) -> CNA_Result;
pub type cna_sound_effect_instance_apply_3d_multi_ext_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_AudioListener,
    u64,
    *const CNA_AudioEmitter,
) -> CNA_Result;
pub type cna_dynamic_sound_effect_instance_create_fn =
    unsafe extern "C" fn(CNA_Handle, i32, CNA_AudioChannels, *mut CNA_Handle) -> CNA_Result;
pub type cna_dynamic_sound_effect_instance_get_pending_buffer_count_fn =
    unsafe extern "C" fn(CNA_Handle, *mut i32) -> CNA_Result;
pub type cna_dynamic_sound_effect_instance_submit_buffer_fn =
    unsafe extern "C" fn(CNA_Handle, *const u8, u64, i32, i32) -> CNA_Result;
pub type cna_dynamic_sound_effect_instance_get_sample_duration_ticks_fn =
    unsafe extern "C" fn(CNA_Handle, i32, *mut i64) -> CNA_Result;
pub type cna_dynamic_sound_effect_instance_get_sample_size_in_bytes_fn =
    unsafe extern "C" fn(CNA_Handle, i64, *mut i32) -> CNA_Result;
pub type cna_dynamic_sound_effect_instance_subscribe_buffer_needed_fn = unsafe extern "C" fn(
    CNA_Handle,
    CNA_AudioEventCallback,
    *mut c_void,
    *mut CNA_AudioEventRegistrationHandle,
) -> CNA_Result;
pub type cna_audio_unsubscribe_ext_fn =
    unsafe extern "C" fn(CNA_AudioEventRegistrationHandle) -> CNA_Result;
pub type cna_microphone_get_count_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_microphone_get_default_index_ext_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64, *mut CNA_Bool) -> CNA_Result;
pub type cna_microphone_get_name_size_at_fn =
    unsafe extern "C" fn(CNA_Handle, u64, *mut u64) -> CNA_Result;
pub type cna_microphone_copy_name_at_fn =
    unsafe extern "C" fn(CNA_Handle, u64, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_microphone_get_buffer_duration_ticks_at_fn =
    unsafe extern "C" fn(CNA_Handle, u64, *mut i64) -> CNA_Result;
pub type cna_microphone_set_buffer_duration_ticks_at_fn =
    unsafe extern "C" fn(CNA_Handle, u64, i64) -> CNA_Result;
pub type cna_microphone_get_is_headset_at_fn =
    unsafe extern "C" fn(CNA_Handle, u64, *mut CNA_Bool) -> CNA_Result;
pub type cna_microphone_get_sample_rate_at_fn =
    unsafe extern "C" fn(CNA_Handle, u64, *mut i32) -> CNA_Result;
pub type cna_microphone_get_state_at_fn =
    unsafe extern "C" fn(CNA_Handle, u64, *mut CNA_MicrophoneState) -> CNA_Result;
pub type cna_microphone_start_at_fn = unsafe extern "C" fn(CNA_Handle, u64) -> CNA_Result;
pub type cna_microphone_stop_at_fn = unsafe extern "C" fn(CNA_Handle, u64) -> CNA_Result;
pub type cna_microphone_get_data_at_fn =
    unsafe extern "C" fn(CNA_Handle, u64, *mut u8, u64, *mut u64) -> CNA_Result;
pub type cna_microphone_get_sample_duration_ticks_at_fn =
    unsafe extern "C" fn(CNA_Handle, u64, i32, *mut i64) -> CNA_Result;
pub type cna_microphone_get_sample_size_in_bytes_at_fn =
    unsafe extern "C" fn(CNA_Handle, u64, i64, *mut i32) -> CNA_Result;
pub type cna_microphone_subscribe_buffer_ready_at_fn = unsafe extern "C" fn(
    CNA_Handle,
    u64,
    CNA_AudioEventCallback,
    *mut c_void,
    *mut CNA_AudioEventRegistrationHandle,
) -> CNA_Result;

pub type cna_audio_engine_create_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView, *mut CNA_Handle) -> CNA_Result;
pub type cna_audio_engine_create_with_renderer_fn = unsafe extern "C" fn(
    CNA_Handle,
    CNA_StringView,
    i64,
    CNA_StringView,
    *mut CNA_Handle,
) -> CNA_Result;
pub type cna_audio_engine_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_audio_engine_get_renderer_count_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_audio_engine_get_renderer_friendly_name_size_fn =
    unsafe extern "C" fn(CNA_Handle, u64, *mut u64) -> CNA_Result;
pub type cna_audio_engine_copy_renderer_friendly_name_fn =
    unsafe extern "C" fn(CNA_Handle, u64, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_audio_engine_get_renderer_id_size_fn =
    unsafe extern "C" fn(CNA_Handle, u64, *mut u64) -> CNA_Result;
pub type cna_audio_engine_copy_renderer_id_fn =
    unsafe extern "C" fn(CNA_Handle, u64, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_audio_engine_get_global_variable_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView, *mut f32) -> CNA_Result;
pub type cna_audio_engine_set_global_variable_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView, f32) -> CNA_Result;
pub type cna_audio_engine_update_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_audio_engine_get_category_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView, *mut CNA_Handle) -> CNA_Result;
pub type cna_audio_category_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_audio_category_get_name_size_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_audio_category_copy_name_fn =
    unsafe extern "C" fn(CNA_Handle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_audio_category_pause_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_audio_category_resume_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_audio_category_set_volume_fn =
    unsafe extern "C" fn(CNA_Handle, f32) -> CNA_Result;
pub type cna_audio_category_stop_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_AudioStopOptions) -> CNA_Result;
pub type cna_audio_category_equals_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_audio_category_get_hash_code_fn =
    unsafe extern "C" fn(CNA_Handle, *mut i32) -> CNA_Result;
pub type cna_wave_bank_create_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView, *mut CNA_Handle) -> CNA_Result;
pub type cna_wave_bank_create_streaming_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView, i32, i16, *mut CNA_Handle) -> CNA_Result;
pub type cna_wave_bank_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_wave_bank_get_is_prepared_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_wave_bank_get_is_in_use_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_sound_bank_create_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView, *mut CNA_Handle) -> CNA_Result;
pub type cna_sound_bank_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_sound_bank_get_is_in_use_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_sound_bank_get_cue_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView, *mut CNA_Handle) -> CNA_Result;
pub type cna_sound_bank_play_cue_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView) -> CNA_Result;
pub type cna_sound_bank_play_cue_3d_fn = unsafe extern "C" fn(
    CNA_Handle,
    CNA_StringView,
    *const CNA_AudioListener,
    *const CNA_AudioEmitter,
) -> CNA_Result;
pub type cna_cue_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_cue_get_info_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_CueInfo) -> CNA_Result;
pub type cna_cue_get_name_size_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_cue_copy_name_fn =
    unsafe extern "C" fn(CNA_Handle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_cue_apply_3d_fn = unsafe extern "C" fn(
    CNA_Handle,
    *const CNA_AudioListener,
    *const CNA_AudioEmitter,
) -> CNA_Result;
pub type cna_cue_get_variable_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView, *mut f32) -> CNA_Result;
pub type cna_cue_set_variable_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_StringView, f32) -> CNA_Result;
pub type cna_cue_play_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_cue_pause_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_cue_resume_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_cue_stop_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_AudioStopOptions) -> CNA_Result;

// Media / video -- canonical CNA C ABI 0.7.
pub type cna_media_source_get_available_count_fn =
    unsafe extern "C" fn(CNA_Handle, *mut u32) -> CNA_Result;
pub type cna_media_source_get_type_at_fn =
    unsafe extern "C" fn(CNA_Handle, u32, *mut CNA_MediaSourceType) -> CNA_Result;
pub type cna_media_source_get_name_size_at_fn =
    unsafe extern "C" fn(CNA_Handle, u32, *mut u64) -> CNA_Result;
pub type cna_media_source_copy_name_at_fn =
    unsafe extern "C" fn(CNA_Handle, u32, *mut c_char, u64, *mut u64) -> CNA_Result;

pub type cna_song_create_from_uri_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_StringView, CNA_StringView, *mut CNA_SongHandle,
) -> CNA_Result;
pub type cna_song_get_name_size_fn =
    unsafe extern "C" fn(CNA_SongHandle, *mut u64) -> CNA_Result;
pub type cna_song_copy_name_fn =
    unsafe extern "C" fn(CNA_SongHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_song_get_duration_fn =
    unsafe extern "C" fn(CNA_SongHandle, *mut i64) -> CNA_Result;
pub type cna_song_get_is_protected_fn =
    unsafe extern "C" fn(CNA_SongHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_song_get_is_rated_fn =
    unsafe extern "C" fn(CNA_SongHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_song_get_play_count_fn =
    unsafe extern "C" fn(CNA_SongHandle, *mut i32) -> CNA_Result;
pub type cna_song_get_rating_fn =
    unsafe extern "C" fn(CNA_SongHandle, *mut i32) -> CNA_Result;
pub type cna_song_get_track_number_fn =
    unsafe extern "C" fn(CNA_SongHandle, *mut i32) -> CNA_Result;
pub type cna_song_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_SongHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_song_dispose_fn = unsafe extern "C" fn(CNA_SongHandle) -> CNA_Result;
pub type cna_song_destroy_fn = unsafe extern "C" fn(CNA_SongHandle) -> CNA_Result;
pub type cna_song_equals_fn = unsafe extern "C" fn(
    CNA_SongHandle, CNA_SongHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_song_get_hash_code_fn =
    unsafe extern "C" fn(CNA_SongHandle, *mut i32) -> CNA_Result;
pub type cna_song_get_album_fn = unsafe extern "C" fn(
    CNA_SongHandle, *mut CNA_AlbumHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_song_get_artist_fn = unsafe extern "C" fn(
    CNA_SongHandle, *mut CNA_ArtistHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_song_get_genre_fn = unsafe extern "C" fn(
    CNA_SongHandle, *mut CNA_GenreHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_song_collection_get_at_fn = unsafe extern "C" fn(
    CNA_SongCollectionHandle, i32, *mut CNA_SongHandle,
) -> CNA_Result;
pub type cna_song_collection_get_count_fn =
    unsafe extern "C" fn(CNA_SongCollectionHandle, *mut i32) -> CNA_Result;
pub type cna_song_collection_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_SongCollectionHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_song_collection_dispose_fn =
    unsafe extern "C" fn(CNA_SongCollectionHandle) -> CNA_Result;
pub type cna_song_collection_destroy_fn =
    unsafe extern "C" fn(CNA_SongCollectionHandle) -> CNA_Result;

pub type cna_media_player_get_game_has_control_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_media_player_get_is_muted_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_media_player_set_is_muted_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Bool) -> CNA_Result;
pub type cna_media_player_get_is_repeating_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_media_player_set_is_repeating_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Bool) -> CNA_Result;
pub type cna_media_player_get_is_shuffled_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_media_player_set_is_shuffled_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Bool) -> CNA_Result;
pub type cna_media_player_get_play_position_ticks_fn =
    unsafe extern "C" fn(CNA_Handle, *mut i64) -> CNA_Result;
pub type cna_media_player_get_state_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_MediaState) -> CNA_Result;
pub type cna_media_player_get_volume_fn =
    unsafe extern "C" fn(CNA_Handle, *mut f32) -> CNA_Result;
pub type cna_media_player_set_volume_fn =
    unsafe extern "C" fn(CNA_Handle, f32) -> CNA_Result;
pub type cna_media_player_get_is_visualization_enabled_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Bool) -> CNA_Result;
pub type cna_media_player_set_is_visualization_enabled_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_Bool) -> CNA_Result;
pub type cna_media_player_get_visualization_data_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_VisualizationData) -> CNA_Result;
pub type cna_media_player_get_queue_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_MediaQueueHandle) -> CNA_Result;
pub type cna_media_player_play_song_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_SongHandle) -> CNA_Result;
pub type cna_media_player_play_songs_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_SongCollectionHandle) -> CNA_Result;
pub type cna_media_player_play_songs_from_fn =
    unsafe extern "C" fn(CNA_Handle, CNA_SongCollectionHandle, i32) -> CNA_Result;
pub type cna_media_player_move_next_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_media_player_move_previous_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_media_player_pause_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_media_player_resume_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_media_player_stop_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_media_player_update_ext_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_media_player_program_exit_ext_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_media_player_subscribe_active_song_changed_ext_fn = unsafe extern "C" fn(
    CNA_MediaPlayerEventCallback, *mut c_void, *mut CNA_MediaPlayerEventRegistrationHandle,
) -> CNA_Result;
pub type cna_media_player_subscribe_media_state_changed_ext_fn = unsafe extern "C" fn(
    CNA_MediaPlayerEventCallback, *mut c_void, *mut CNA_MediaPlayerEventRegistrationHandle,
) -> CNA_Result;
pub type cna_media_player_unsubscribe_ext_fn =
    unsafe extern "C" fn(CNA_MediaPlayerEventRegistrationHandle) -> CNA_Result;
pub type cna_media_player_raise_active_song_changed_ext_fn =
    unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_media_player_raise_media_state_changed_ext_fn =
    unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_media_queue_get_count_fn =
    unsafe extern "C" fn(CNA_MediaQueueHandle, *mut i32) -> CNA_Result;
pub type cna_media_queue_get_active_song_index_fn =
    unsafe extern "C" fn(CNA_MediaQueueHandle, *mut i32) -> CNA_Result;
pub type cna_media_queue_set_active_song_index_fn =
    unsafe extern "C" fn(CNA_MediaQueueHandle, i32) -> CNA_Result;
pub type cna_media_queue_get_active_song_fn = unsafe extern "C" fn(
    CNA_MediaQueueHandle, *mut CNA_SongHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_media_queue_get_at_fn = unsafe extern "C" fn(
    CNA_MediaQueueHandle, i32, *mut CNA_SongHandle,
) -> CNA_Result;
pub type cna_media_queue_destroy_fn =
    unsafe extern "C" fn(CNA_MediaQueueHandle) -> CNA_Result;

pub type cna_media_library_create_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_MediaLibraryHandle) -> CNA_Result;
pub type cna_media_library_create_from_source_fn =
    unsafe extern "C" fn(CNA_Handle, u32, *mut CNA_MediaLibraryHandle) -> CNA_Result;
pub type cna_media_library_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_MediaLibraryHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_media_library_dispose_fn =
    unsafe extern "C" fn(CNA_MediaLibraryHandle) -> CNA_Result;
pub type cna_media_library_destroy_fn =
    unsafe extern "C" fn(CNA_MediaLibraryHandle) -> CNA_Result;
pub type cna_media_library_get_media_source_type_fn = unsafe extern "C" fn(
    CNA_MediaLibraryHandle, *mut CNA_MediaSourceType,
) -> CNA_Result;
pub type cna_media_library_get_media_source_name_size_fn =
    unsafe extern "C" fn(CNA_MediaLibraryHandle, *mut u64) -> CNA_Result;
pub type cna_media_library_copy_media_source_name_fn = unsafe extern "C" fn(
    CNA_MediaLibraryHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_media_library_get_songs_fn = unsafe extern "C" fn(
    CNA_MediaLibraryHandle, *mut CNA_SongCollectionHandle,
) -> CNA_Result;
pub type cna_media_library_get_albums_fn = unsafe extern "C" fn(
    CNA_MediaLibraryHandle, *mut CNA_AlbumCollectionHandle,
) -> CNA_Result;
pub type cna_media_library_get_artists_fn = unsafe extern "C" fn(
    CNA_MediaLibraryHandle, *mut CNA_ArtistCollectionHandle,
) -> CNA_Result;
pub type cna_media_library_get_genres_fn = unsafe extern "C" fn(
    CNA_MediaLibraryHandle, *mut CNA_GenreCollectionHandle,
) -> CNA_Result;
pub type cna_media_library_get_playlists_fn = unsafe extern "C" fn(
    CNA_MediaLibraryHandle, *mut CNA_PlaylistCollectionHandle,
) -> CNA_Result;
pub type cna_media_library_get_pictures_fn = unsafe extern "C" fn(
    CNA_MediaLibraryHandle, *mut CNA_PictureCollectionHandle,
) -> CNA_Result;
pub type cna_media_library_get_saved_pictures_fn = unsafe extern "C" fn(
    CNA_MediaLibraryHandle, *mut CNA_PictureCollectionHandle,
) -> CNA_Result;
pub type cna_media_library_get_root_picture_album_fn = unsafe extern "C" fn(
    CNA_MediaLibraryHandle, *mut CNA_PictureAlbumHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_media_library_get_picture_from_token_fn = unsafe extern "C" fn(
    CNA_MediaLibraryHandle, CNA_StringView, *mut CNA_PictureHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_media_library_save_picture_fn = unsafe extern "C" fn(
    CNA_MediaLibraryHandle, CNA_StringView, *const u8, u64, *mut CNA_PictureHandle,
) -> CNA_Result;

pub type cna_album_get_name_size_fn =
    unsafe extern "C" fn(CNA_AlbumHandle, *mut u64) -> CNA_Result;
pub type cna_album_copy_name_fn =
    unsafe extern "C" fn(CNA_AlbumHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_album_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_AlbumHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_album_dispose_fn = unsafe extern "C" fn(CNA_AlbumHandle) -> CNA_Result;
pub type cna_album_destroy_fn = unsafe extern "C" fn(CNA_AlbumHandle) -> CNA_Result;
pub type cna_album_get_hash_code_fn =
    unsafe extern "C" fn(CNA_AlbumHandle, *mut i32) -> CNA_Result;
pub type cna_album_equals_fn = unsafe extern "C" fn(
    CNA_AlbumHandle, CNA_AlbumHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_album_get_songs_fn = unsafe extern "C" fn(
    CNA_AlbumHandle, *mut CNA_SongCollectionHandle,
) -> CNA_Result;
pub type cna_album_get_artist_fn = unsafe extern "C" fn(
    CNA_AlbumHandle, *mut CNA_ArtistHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_album_get_genre_fn = unsafe extern "C" fn(
    CNA_AlbumHandle, *mut CNA_GenreHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_album_get_duration_fn =
    unsafe extern "C" fn(CNA_AlbumHandle, *mut i64) -> CNA_Result;
pub type cna_album_get_has_art_fn =
    unsafe extern "C" fn(CNA_AlbumHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_album_get_art_size_fn =
    unsafe extern "C" fn(CNA_AlbumHandle, *mut u64) -> CNA_Result;
pub type cna_album_copy_art_fn =
    unsafe extern "C" fn(CNA_AlbumHandle, *mut u8, u64, *mut u64) -> CNA_Result;
pub type cna_album_get_thumbnail_size_fn =
    unsafe extern "C" fn(CNA_AlbumHandle, *mut u64) -> CNA_Result;
pub type cna_album_copy_thumbnail_fn =
    unsafe extern "C" fn(CNA_AlbumHandle, *mut u8, u64, *mut u64) -> CNA_Result;
pub type cna_album_collection_get_count_fn =
    unsafe extern "C" fn(CNA_AlbumCollectionHandle, *mut i32) -> CNA_Result;
pub type cna_album_collection_get_at_fn = unsafe extern "C" fn(
    CNA_AlbumCollectionHandle, i32, *mut CNA_AlbumHandle,
) -> CNA_Result;
pub type cna_album_collection_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_AlbumCollectionHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_album_collection_dispose_fn =
    unsafe extern "C" fn(CNA_AlbumCollectionHandle) -> CNA_Result;
pub type cna_album_collection_destroy_fn =
    unsafe extern "C" fn(CNA_AlbumCollectionHandle) -> CNA_Result;

pub type cna_artist_get_name_size_fn =
    unsafe extern "C" fn(CNA_ArtistHandle, *mut u64) -> CNA_Result;
pub type cna_artist_copy_name_fn =
    unsafe extern "C" fn(CNA_ArtistHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_artist_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_ArtistHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_artist_dispose_fn = unsafe extern "C" fn(CNA_ArtistHandle) -> CNA_Result;
pub type cna_artist_destroy_fn = unsafe extern "C" fn(CNA_ArtistHandle) -> CNA_Result;
pub type cna_artist_get_hash_code_fn =
    unsafe extern "C" fn(CNA_ArtistHandle, *mut i32) -> CNA_Result;
pub type cna_artist_equals_fn = unsafe extern "C" fn(
    CNA_ArtistHandle, CNA_ArtistHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_artist_get_songs_fn = unsafe extern "C" fn(
    CNA_ArtistHandle, *mut CNA_SongCollectionHandle,
) -> CNA_Result;
pub type cna_artist_get_albums_fn = unsafe extern "C" fn(
    CNA_ArtistHandle, *mut CNA_AlbumCollectionHandle,
) -> CNA_Result;
pub type cna_artist_collection_get_count_fn =
    unsafe extern "C" fn(CNA_ArtistCollectionHandle, *mut i32) -> CNA_Result;
pub type cna_artist_collection_get_at_fn = unsafe extern "C" fn(
    CNA_ArtistCollectionHandle, i32, *mut CNA_ArtistHandle,
) -> CNA_Result;
pub type cna_artist_collection_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_ArtistCollectionHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_artist_collection_dispose_fn =
    unsafe extern "C" fn(CNA_ArtistCollectionHandle) -> CNA_Result;
pub type cna_artist_collection_destroy_fn =
    unsafe extern "C" fn(CNA_ArtistCollectionHandle) -> CNA_Result;

pub type cna_genre_get_name_size_fn =
    unsafe extern "C" fn(CNA_GenreHandle, *mut u64) -> CNA_Result;
pub type cna_genre_copy_name_fn =
    unsafe extern "C" fn(CNA_GenreHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_genre_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_GenreHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_genre_dispose_fn = unsafe extern "C" fn(CNA_GenreHandle) -> CNA_Result;
pub type cna_genre_destroy_fn = unsafe extern "C" fn(CNA_GenreHandle) -> CNA_Result;
pub type cna_genre_get_hash_code_fn =
    unsafe extern "C" fn(CNA_GenreHandle, *mut i32) -> CNA_Result;
pub type cna_genre_equals_fn = unsafe extern "C" fn(
    CNA_GenreHandle, CNA_GenreHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_genre_get_songs_fn = unsafe extern "C" fn(
    CNA_GenreHandle, *mut CNA_SongCollectionHandle,
) -> CNA_Result;
pub type cna_genre_get_albums_fn = unsafe extern "C" fn(
    CNA_GenreHandle, *mut CNA_AlbumCollectionHandle,
) -> CNA_Result;
pub type cna_genre_collection_get_count_fn =
    unsafe extern "C" fn(CNA_GenreCollectionHandle, *mut i32) -> CNA_Result;
pub type cna_genre_collection_get_at_fn = unsafe extern "C" fn(
    CNA_GenreCollectionHandle, i32, *mut CNA_GenreHandle,
) -> CNA_Result;
pub type cna_genre_collection_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_GenreCollectionHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_genre_collection_dispose_fn =
    unsafe extern "C" fn(CNA_GenreCollectionHandle) -> CNA_Result;
pub type cna_genre_collection_destroy_fn =
    unsafe extern "C" fn(CNA_GenreCollectionHandle) -> CNA_Result;

pub type cna_playlist_get_name_size_fn =
    unsafe extern "C" fn(CNA_PlaylistHandle, *mut u64) -> CNA_Result;
pub type cna_playlist_copy_name_fn =
    unsafe extern "C" fn(CNA_PlaylistHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_playlist_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_PlaylistHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_playlist_dispose_fn = unsafe extern "C" fn(CNA_PlaylistHandle) -> CNA_Result;
pub type cna_playlist_destroy_fn = unsafe extern "C" fn(CNA_PlaylistHandle) -> CNA_Result;
pub type cna_playlist_get_hash_code_fn =
    unsafe extern "C" fn(CNA_PlaylistHandle, *mut i32) -> CNA_Result;
pub type cna_playlist_equals_fn = unsafe extern "C" fn(
    CNA_PlaylistHandle, CNA_PlaylistHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_playlist_get_songs_fn = unsafe extern "C" fn(
    CNA_PlaylistHandle, *mut CNA_SongCollectionHandle,
) -> CNA_Result;
pub type cna_playlist_get_duration_fn =
    unsafe extern "C" fn(CNA_PlaylistHandle, *mut i64) -> CNA_Result;
pub type cna_playlist_collection_get_count_fn =
    unsafe extern "C" fn(CNA_PlaylistCollectionHandle, *mut i32) -> CNA_Result;
pub type cna_playlist_collection_get_at_fn = unsafe extern "C" fn(
    CNA_PlaylistCollectionHandle, i32, *mut CNA_PlaylistHandle,
) -> CNA_Result;
pub type cna_playlist_collection_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_PlaylistCollectionHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_playlist_collection_dispose_fn =
    unsafe extern "C" fn(CNA_PlaylistCollectionHandle) -> CNA_Result;
pub type cna_playlist_collection_destroy_fn =
    unsafe extern "C" fn(CNA_PlaylistCollectionHandle) -> CNA_Result;

pub type cna_picture_get_name_size_fn =
    unsafe extern "C" fn(CNA_PictureHandle, *mut u64) -> CNA_Result;
pub type cna_picture_copy_name_fn =
    unsafe extern "C" fn(CNA_PictureHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_picture_get_album_fn = unsafe extern "C" fn(
    CNA_PictureHandle, *mut CNA_PictureAlbumHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_picture_get_date_unix_ticks_fn =
    unsafe extern "C" fn(CNA_PictureHandle, *mut i64) -> CNA_Result;
pub type cna_picture_get_width_fn =
    unsafe extern "C" fn(CNA_PictureHandle, *mut i32) -> CNA_Result;
pub type cna_picture_get_height_fn =
    unsafe extern "C" fn(CNA_PictureHandle, *mut i32) -> CNA_Result;
pub type cna_picture_get_image_size_fn =
    unsafe extern "C" fn(CNA_PictureHandle, *mut u64) -> CNA_Result;
pub type cna_picture_copy_image_fn =
    unsafe extern "C" fn(CNA_PictureHandle, *mut u8, u64, *mut u64) -> CNA_Result;
pub type cna_picture_get_thumbnail_size_fn =
    unsafe extern "C" fn(CNA_PictureHandle, *mut u64) -> CNA_Result;
pub type cna_picture_copy_thumbnail_fn =
    unsafe extern "C" fn(CNA_PictureHandle, *mut u8, u64, *mut u64) -> CNA_Result;
pub type cna_picture_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_PictureHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_picture_dispose_fn = unsafe extern "C" fn(CNA_PictureHandle) -> CNA_Result;
pub type cna_picture_destroy_fn = unsafe extern "C" fn(CNA_PictureHandle) -> CNA_Result;
pub type cna_picture_equals_fn = unsafe extern "C" fn(
    CNA_PictureHandle, CNA_PictureHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_picture_get_hash_code_fn =
    unsafe extern "C" fn(CNA_PictureHandle, *mut i32) -> CNA_Result;
pub type cna_picture_collection_get_count_fn =
    unsafe extern "C" fn(CNA_PictureCollectionHandle, *mut i32) -> CNA_Result;
pub type cna_picture_collection_get_at_fn = unsafe extern "C" fn(
    CNA_PictureCollectionHandle, i32, *mut CNA_PictureHandle,
) -> CNA_Result;
pub type cna_picture_collection_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_PictureCollectionHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_picture_collection_dispose_fn =
    unsafe extern "C" fn(CNA_PictureCollectionHandle) -> CNA_Result;
pub type cna_picture_collection_destroy_fn =
    unsafe extern "C" fn(CNA_PictureCollectionHandle) -> CNA_Result;

pub type cna_picture_album_get_name_size_fn =
    unsafe extern "C" fn(CNA_PictureAlbumHandle, *mut u64) -> CNA_Result;
pub type cna_picture_album_copy_name_fn =
    unsafe extern "C" fn(CNA_PictureAlbumHandle, *mut c_char, u64, *mut u64) -> CNA_Result;
pub type cna_picture_album_get_parent_fn = unsafe extern "C" fn(
    CNA_PictureAlbumHandle, *mut CNA_PictureAlbumHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_picture_album_get_albums_fn = unsafe extern "C" fn(
    CNA_PictureAlbumHandle, *mut CNA_PictureAlbumCollectionHandle,
) -> CNA_Result;
pub type cna_picture_album_get_pictures_fn = unsafe extern "C" fn(
    CNA_PictureAlbumHandle, *mut CNA_PictureCollectionHandle,
) -> CNA_Result;
pub type cna_picture_album_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_PictureAlbumHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_picture_album_dispose_fn =
    unsafe extern "C" fn(CNA_PictureAlbumHandle) -> CNA_Result;
pub type cna_picture_album_destroy_fn =
    unsafe extern "C" fn(CNA_PictureAlbumHandle) -> CNA_Result;
pub type cna_picture_album_equals_fn = unsafe extern "C" fn(
    CNA_PictureAlbumHandle, CNA_PictureAlbumHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_picture_album_get_hash_code_fn =
    unsafe extern "C" fn(CNA_PictureAlbumHandle, *mut i32) -> CNA_Result;
pub type cna_picture_album_collection_get_count_fn =
    unsafe extern "C" fn(CNA_PictureAlbumCollectionHandle, *mut i32) -> CNA_Result;
pub type cna_picture_album_collection_get_at_fn = unsafe extern "C" fn(
    CNA_PictureAlbumCollectionHandle, i32, *mut CNA_PictureAlbumHandle,
) -> CNA_Result;
pub type cna_picture_album_collection_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_PictureAlbumCollectionHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_picture_album_collection_dispose_fn =
    unsafe extern "C" fn(CNA_PictureAlbumCollectionHandle) -> CNA_Result;
pub type cna_picture_album_collection_destroy_fn =
    unsafe extern "C" fn(CNA_PictureAlbumCollectionHandle) -> CNA_Result;

pub type cna_video_create_with_metadata_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_StringView, i32, i32, i32, f32, CNA_VideoSoundtrackType,
    *mut CNA_VideoHandle,
) -> CNA_Result;
pub type cna_video_get_width_fn =
    unsafe extern "C" fn(CNA_VideoHandle, *mut i32) -> CNA_Result;
pub type cna_video_get_height_fn =
    unsafe extern "C" fn(CNA_VideoHandle, *mut i32) -> CNA_Result;
pub type cna_video_get_frames_per_second_fn =
    unsafe extern "C" fn(CNA_VideoHandle, *mut f32) -> CNA_Result;
pub type cna_video_get_duration_fn =
    unsafe extern "C" fn(CNA_VideoHandle, *mut i64) -> CNA_Result;
pub type cna_video_get_soundtrack_type_fn = unsafe extern "C" fn(
    CNA_VideoHandle, *mut CNA_VideoSoundtrackType,
) -> CNA_Result;
pub type cna_video_destroy_fn = unsafe extern "C" fn(CNA_VideoHandle) -> CNA_Result;
pub type cna_video_player_create_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_VideoPlayerHandle) -> CNA_Result;
pub type cna_video_player_get_is_disposed_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_video_player_get_is_looped_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_video_player_set_is_looped_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle, CNA_Bool) -> CNA_Result;
pub type cna_video_player_get_is_muted_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle, *mut CNA_Bool) -> CNA_Result;
pub type cna_video_player_set_is_muted_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle, CNA_Bool) -> CNA_Result;
pub type cna_video_player_get_play_position_ticks_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle, *mut i64) -> CNA_Result;
pub type cna_video_player_get_state_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle, *mut CNA_MediaState) -> CNA_Result;
pub type cna_video_player_get_volume_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle, *mut f32) -> CNA_Result;
pub type cna_video_player_set_volume_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle, f32) -> CNA_Result;
pub type cna_video_player_get_texture_fn = unsafe extern "C" fn(
    CNA_VideoPlayerHandle, *mut CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_video_player_get_frame_ext_fn = unsafe extern "C" fn(
    CNA_VideoPlayerHandle, *mut CNA_VideoFrameEXT,
) -> CNA_Result;
pub type cna_video_player_play_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle, CNA_VideoHandle) -> CNA_Result;
pub type cna_video_player_stop_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle) -> CNA_Result;
pub type cna_video_player_pause_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle) -> CNA_Result;
pub type cna_video_player_resume_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle) -> CNA_Result;
pub type cna_video_player_dispose_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle) -> CNA_Result;
pub type cna_video_player_destroy_fn =
    unsafe extern "C" fn(CNA_VideoPlayerHandle) -> CNA_Result;

#[cfg(test)]
mod layout_tests {
    use super::*;
    use core::mem::{align_of, size_of};

    #[test]
    #[allow(clippy::too_many_lines)]
    fn reviewed_abi_070_layouts_match_lp64_and_llp64() {
        assert_eq!(
            (size_of::<CNA_StringView>(), align_of::<CNA_StringView>()),
            (16, 8)
        );
        assert_eq!((size_of::<CNA_Color>(), align_of::<CNA_Color>()), (4, 1));
        assert_eq!(
            (size_of::<CNA_GameTime>(), align_of::<CNA_GameTime>()),
            (24, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_CallbackError>(),
                align_of::<CNA_CallbackError>()
            ),
            (24, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_GameCallbacks>(),
                align_of::<CNA_GameCallbacks>()
            ),
            (56, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_GameFrameHooks>(),
                align_of::<CNA_GameFrameHooks>()
            ),
            (56, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_GameCreateInfo>(),
                align_of::<CNA_GameCreateInfo>()
            ),
            (48, 8)
        );
        assert_eq!(
            (size_of::<CNA_Viewport>(), align_of::<CNA_Viewport>()),
            (24, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_RendererInfo>(),
                align_of::<CNA_RendererInfo>()
            ),
            (32, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_Texture2DInfo>(),
                align_of::<CNA_Texture2DInfo>()
            ),
            (24, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_Texture2DCreateInfo>(),
                align_of::<CNA_Texture2DCreateInfo>()
            ),
            (24, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_Texture2DTransfer>(),
                align_of::<CNA_Texture2DTransfer>()
            ),
            (48, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_Texture2DDecodeInfo>(),
                align_of::<CNA_Texture2DDecodeInfo>()
            ),
            (24, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_VertexElement>(),
                align_of::<CNA_VertexElement>()
            ),
            (16, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_VertexBufferCreateInfo>(),
                align_of::<CNA_VertexBufferCreateInfo>()
            ),
            (32, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_VertexBufferInfo>(),
                align_of::<CNA_VertexBufferInfo>()
            ),
            (32, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_VertexBufferTransfer>(),
                align_of::<CNA_VertexBufferTransfer>()
            ),
            (32, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_VertexBufferBinding>(),
                align_of::<CNA_VertexBufferBinding>()
            ),
            (16, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_IndexBufferCreateInfo>(),
                align_of::<CNA_IndexBufferCreateInfo>()
            ),
            (24, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_IndexBufferInfo>(),
                align_of::<CNA_IndexBufferInfo>()
            ),
            (24, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_IndexBufferTransfer>(),
                align_of::<CNA_IndexBufferTransfer>()
            ),
            (32, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_UserPrimitives>(),
                align_of::<CNA_UserPrimitives>()
            ),
            (48, 8)
        );
        assert_eq!(
            (size_of::<CNA_UserIndices>(), align_of::<CNA_UserIndices>()),
            (24, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_BackBufferReadback>(),
                align_of::<CNA_BackBufferReadback>()
            ),
            (48, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_Texture3DCreateInfo>(),
                align_of::<CNA_Texture3DCreateInfo>()
            ),
            (32, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_Texture3DInfo>(),
                align_of::<CNA_Texture3DInfo>()
            ),
            (32, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_Texture3DTransfer>(),
                align_of::<CNA_Texture3DTransfer>()
            ),
            (56, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_TextureCubeCreateInfo>(),
                align_of::<CNA_TextureCubeCreateInfo>()
            ),
            (24, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_TextureCubeInfo>(),
                align_of::<CNA_TextureCubeInfo>()
            ),
            (24, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_TextureCubeTransfer>(),
                align_of::<CNA_TextureCubeTransfer>()
            ),
            (56, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_RenderTarget2DCreateInfo>(),
                align_of::<CNA_RenderTarget2DCreateInfo>()
            ),
            (40, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_RenderTargetCubeCreateInfo>(),
                align_of::<CNA_RenderTargetCubeCreateInfo>()
            ),
            (32, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_RenderTargetInfo>(),
                align_of::<CNA_RenderTargetInfo>()
            ),
            (44, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_RenderTargetBinding>(),
                align_of::<CNA_RenderTargetBinding>()
            ),
            (24, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_SpriteBatchBeginInfo>(),
                align_of::<CNA_SpriteBatchBeginInfo>()
            ),
            (16, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_SpriteCommand>(),
                align_of::<CNA_SpriteCommand>()
            ),
            (72, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_SpriteFontGlyph>(),
                align_of::<CNA_SpriteFontGlyph>()
            ),
            (56, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_SpriteFontCreateInfo>(),
                align_of::<CNA_SpriteFontCreateInfo>()
            ),
            (48, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_SpriteFontInfo>(),
                align_of::<CNA_SpriteFontInfo>()
            ),
            (32, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_SpriteTextCommand>(),
                align_of::<CNA_SpriteTextCommand>()
            ),
            (72, 8)
        );
        assert_eq!(
            (size_of::<CNA_Vector4>(), align_of::<CNA_Vector4>()),
            (16, 4)
        );
        assert_eq!(
            (size_of::<CNA_Quaternion>(), align_of::<CNA_Quaternion>()),
            (16, 4)
        );
        assert_eq!((size_of::<CNA_Matrix>(), align_of::<CNA_Matrix>()), (64, 4));
        assert_eq!(
            (
                size_of::<CNA_EffectAnnotationCreateInfo>(),
                align_of::<CNA_EffectAnnotationCreateInfo>()
            ),
            (88, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_EffectAnnotationInfo>(),
                align_of::<CNA_EffectAnnotationInfo>()
            ),
            (24, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_EffectParameterCreateInfo>(),
                align_of::<CNA_EffectParameterCreateInfo>()
            ),
            (56, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_EffectParameterInfo>(),
                align_of::<CNA_EffectParameterInfo>()
            ),
            (24, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_KeyboardState>(),
                align_of::<CNA_KeyboardState>()
            ),
            (40, 8)
        );
        assert_eq!(
            (size_of::<CNA_MouseState>(), align_of::<CNA_MouseState>()),
            (32, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_GamePadAnalogState>(),
                align_of::<CNA_GamePadAnalogState>()
            ),
            (24, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_GamePadState>(),
                align_of::<CNA_GamePadState>()
            ),
            (48, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_GamePadCapabilities>(),
                align_of::<CNA_GamePadCapabilities>()
            ),
            (48, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_TouchLocation>(),
                align_of::<CNA_TouchLocation>()
            ),
            (32, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_TouchCapabilities>(),
                align_of::<CNA_TouchCapabilities>()
            ),
            (16, 4)
        );
        assert_eq!(
            (size_of::<CNA_TouchState>(), align_of::<CNA_TouchState>()),
            (272, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_GestureSample>(),
                align_of::<CNA_GestureSample>()
            ),
            (64, 8)
        );
        assert_eq!(
            (
                size_of::<CNA_GraphicsDeviceInformation>(),
                align_of::<CNA_GraphicsDeviceInformation>()
            ),
            (60, 4)
        );
        assert_eq!(
            (
                size_of::<CNA_VisualizationData>(),
                align_of::<CNA_VisualizationData>()
            ),
            (2056, 4)
        );
    }
}

/// Receives one formatted log line. The bytes are borrowed for the call.
pub type CNA_LogSinkCallback = Option<
    unsafe extern "C" fn(CNA_LogLevel, CNA_LogCategory, CNA_StringView, *mut c_void),
>;

// --- CNA runtime identity and renderer selection (core_ext.h) ---

pub type cna_platform_get_current_fn = unsafe extern "C" fn(*mut CNA_Platform) -> CNA_Result;
pub type cna_platform_get_is_apple_ext_fn = unsafe extern "C" fn(*mut CNA_Bool) -> CNA_Result;
pub type cna_platform_get_is_mobile_ext_fn = unsafe extern "C" fn(*mut CNA_Bool) -> CNA_Result;
pub type cna_platform_get_current_name_size_ext_fn = unsafe extern "C" fn(*mut u64) -> CNA_Result;
pub type cna_platform_copy_current_name_ext_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_desktop_os_get_current_fn = unsafe extern "C" fn(*mut CNA_DesktopOS) -> CNA_Result;
pub type cna_graphics_backend_get_category_fn = unsafe extern "C" fn(
    CNA_GraphicsRendererType, *mut CNA_GraphicsBackendCategory,
) -> CNA_Result;
pub type cna_graphics_backend_get_current_category_fn = unsafe extern "C" fn(
    *mut CNA_GraphicsBackendCategory,
) -> CNA_Result;
pub type cna_graphics_backend_category_get_name_size_fn = unsafe extern "C" fn(
    CNA_GraphicsBackendCategory, *mut u64,
) -> CNA_Result;
pub type cna_graphics_backend_category_copy_name_fn = unsafe extern "C" fn(
    CNA_GraphicsBackendCategory, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_graphics_backend_get_maturity_fn = unsafe extern "C" fn(
    CNA_GraphicsRendererType, *mut CNA_GraphicsBackendMaturity,
) -> CNA_Result;
pub type cna_graphics_backend_get_current_maturity_fn = unsafe extern "C" fn(
    *mut CNA_GraphicsBackendMaturity,
) -> CNA_Result;
pub type cna_graphics_backend_maturity_get_name_size_fn = unsafe extern "C" fn(
    CNA_GraphicsBackendMaturity, *mut u64,
) -> CNA_Result;
pub type cna_graphics_backend_maturity_copy_name_fn = unsafe extern "C" fn(
    CNA_GraphicsBackendMaturity, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_graphics_renderer_set_preferred_ext_fn = unsafe extern "C" fn(
    CNA_GraphicsRendererType,
) -> CNA_Result;
pub type cna_graphics_renderer_set_preferred_by_name_ext_fn = unsafe extern "C" fn(
    CNA_StringView,
) -> CNA_Result;
pub type cna_graphics_renderer_get_selected_ext_fn = unsafe extern "C" fn(
    *mut CNA_GraphicsRendererType,
) -> CNA_Result;
pub type cna_graphics_renderer_get_active_ext_fn = unsafe extern "C" fn(
    *mut CNA_GraphicsRendererType,
) -> CNA_Result;
pub type cna_graphics_renderer_get_is_latched_ext_fn = unsafe extern "C" fn(
    *mut CNA_Bool,
) -> CNA_Result;
pub type cna_graphics_renderer_get_available_count_ext_fn = unsafe extern "C" fn(
    *mut u64,
) -> CNA_Result;
pub type cna_graphics_renderer_copy_available_ext_fn = unsafe extern "C" fn(
    *mut CNA_GraphicsRendererType, u64, *mut u64,
) -> CNA_Result;
pub type cna_graphics_renderer_get_is_available_ext_fn = unsafe extern "C" fn(
    CNA_GraphicsRendererType, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_graphics_renderer_set_fallback_chain_ext_fn = unsafe extern "C" fn(
    *const CNA_GraphicsRendererType, u64,
) -> CNA_Result;
pub type cna_graphics_renderer_set_automatic_fallback_ext_fn = unsafe extern "C" fn(
    CNA_Bool,
) -> CNA_Result;
pub type cna_graphics_renderer_get_automatic_fallback_ext_fn = unsafe extern "C" fn(
    *mut CNA_Bool,
) -> CNA_Result;
pub type cna_graphics_renderer_get_fallback_count_ext_fn = unsafe extern "C" fn(
    *mut u64,
) -> CNA_Result;
pub type cna_graphics_renderer_get_fallback_at_ext_fn = unsafe extern "C" fn(
    u64, *mut CNA_GraphicsRendererFallbackRecord,
) -> CNA_Result;
pub type cna_graphics_renderer_fallback_get_message_size_ext_fn = unsafe extern "C" fn(
    u64, *mut u64,
) -> CNA_Result;
pub type cna_graphics_renderer_fallback_copy_message_ext_fn = unsafe extern "C" fn(
    u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_graphics_renderer_fallback_reason_get_name_size_ext_fn = unsafe extern "C" fn(
    CNA_GraphicsRendererFallbackReason, *mut u64,
) -> CNA_Result;
pub type cna_graphics_renderer_fallback_reason_copy_name_ext_fn = unsafe extern "C" fn(
    CNA_GraphicsRendererFallbackReason, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_graphics_renderer_try_parse_name_ext_fn = unsafe extern "C" fn(
    CNA_StringView, *mut CNA_GraphicsRendererType, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_graphics_renderer_get_current_type_fn = unsafe extern "C" fn(
    *mut CNA_GraphicsRendererType,
) -> CNA_Result;
pub type cna_graphics_renderer_get_current_name_size_fn = unsafe extern "C" fn(
    *mut u64,
) -> CNA_Result;
pub type cna_graphics_renderer_copy_current_name_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;

// --- CNA logging (core_ext.h) ---

pub type cna_logger_log_fn = unsafe extern "C" fn(
    CNA_LogLevel, CNA_StringView, CNA_LogCategory, CNA_Bool,
) -> CNA_Result;
pub type cna_logger_fatal_fn = unsafe extern "C" fn(CNA_StringView, CNA_LogCategory) -> CNA_Result;
pub type cna_logger_error_fn = unsafe extern "C" fn(CNA_StringView, CNA_LogCategory) -> CNA_Result;
pub type cna_logger_warn_fn = unsafe extern "C" fn(CNA_StringView, CNA_LogCategory) -> CNA_Result;
pub type cna_logger_info_fn = unsafe extern "C" fn(CNA_StringView, CNA_LogCategory) -> CNA_Result;
pub type cna_logger_debug_fn = unsafe extern "C" fn(CNA_StringView, CNA_LogCategory) -> CNA_Result;
pub type cna_logger_trace_fn = unsafe extern "C" fn(CNA_StringView, CNA_LogCategory) -> CNA_Result;
pub type cna_logger_experiment_fn = unsafe extern "C" fn(
    CNA_StringView, CNA_LogCategory,
) -> CNA_Result;
pub type cna_logger_fatal_if_fn = unsafe extern "C" fn(CNA_StringView, CNA_Bool) -> CNA_Result;
pub type cna_logger_error_if_fn = unsafe extern "C" fn(CNA_StringView, CNA_Bool) -> CNA_Result;
pub type cna_logger_warn_if_fn = unsafe extern "C" fn(CNA_StringView, CNA_Bool) -> CNA_Result;
pub type cna_logger_info_if_fn = unsafe extern "C" fn(CNA_StringView, CNA_Bool) -> CNA_Result;
pub type cna_logger_debug_if_fn = unsafe extern "C" fn(CNA_StringView, CNA_Bool) -> CNA_Result;
pub type cna_logger_trace_if_fn = unsafe extern "C" fn(CNA_StringView, CNA_Bool) -> CNA_Result;
pub type cna_logger_experiment_if_fn = unsafe extern "C" fn(CNA_StringView, CNA_Bool) -> CNA_Result;
pub type cna_logger_set_minimum_level_fn = unsafe extern "C" fn(CNA_LogLevel) -> CNA_Result;
pub type cna_logger_get_minimum_level_fn = unsafe extern "C" fn(*mut CNA_LogLevel) -> CNA_Result;
pub type cna_logger_set_sink_ext_fn = unsafe extern "C" fn(
    CNA_LogSinkCallback, *mut c_void,
) -> CNA_Result;
pub type cna_logger_reset_sink_ext_fn = unsafe extern "C" fn() -> CNA_Result;

// --- CNA renderer capability reporting (graphics.h) ---

pub type cna_graphics_device_get_renderer_feature_support_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_RendererFeature, *mut CNA_RendererFeatureSupport,
) -> CNA_Result;
pub type cna_graphics_device_get_renderer_limit_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_RendererLimit, *mut CNA_Bool, *mut u64,
) -> CNA_Result;
pub type cna_graphics_device_get_surface_format_support_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_SurfaceFormat, *mut CNA_RendererFormatUsageFlags, *mut CNA_RendererFormatUsageFlags,
) -> CNA_Result;
pub type cna_graphics_device_get_capability_report_size_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut u64,
) -> CNA_Result;
pub type cna_graphics_device_copy_capability_report_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_graphics_device_get_shader_dialect_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_ShaderDialect,
) -> CNA_Result;

// --- CNA device layer (devices.h, input_devices.h) ---

pub type cna_devices_ext_is_available_fn = unsafe extern "C" fn(*mut CNA_Bool) -> CNA_Result;
pub type cna_power_get_state_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PowerState,
) -> CNA_Result;
pub type cna_power_get_battery_percent_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut i32,
) -> CNA_Result;
pub type cna_power_get_seconds_remaining_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut i32,
) -> CNA_Result;
pub type cna_system_info_get_logical_cpu_core_count_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut i32,
) -> CNA_Result;
pub type cna_system_info_get_system_ram_megabytes_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut i32,
) -> CNA_Result;
pub type cna_locale_get_preferred_count_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut u64,
) -> CNA_Result;
pub type cna_locale_get_language_size_at_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u64, *mut u64,
) -> CNA_Result;
pub type cna_locale_copy_language_at_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_locale_get_country_size_at_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u64, *mut u64,
) -> CNA_Result;
pub type cna_locale_copy_country_at_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_display_info_get_content_scale_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut f32,
) -> CNA_Result;
pub type cna_display_info_get_safe_area_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Rectangle,
) -> CNA_Result;
pub type cna_clipboard_get_text_size_fn = unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_clipboard_copy_text_fn = unsafe extern "C" fn(
    CNA_Handle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_clipboard_set_text_fn = unsafe extern "C" fn(CNA_Handle, CNA_StringView) -> CNA_Result;

// --- CNA .cnb content container (cnb.h) ---

pub type cna_cnb_document_parse_fn = unsafe extern "C" fn(
    *const u8, u64, CNA_StringView, *const CNA_CnbReadLimits, *mut CNA_CnbDocumentHandle,
) -> CNA_Result;
pub type cna_cnb_document_parse_file_fn = unsafe extern "C" fn(
    CNA_StringView, *const CNA_CnbReadLimits, *mut CNA_CnbDocumentHandle,
) -> CNA_Result;
pub type cna_cnb_document_destroy_fn = unsafe extern "C" fn(CNA_CnbDocumentHandle) -> CNA_Result;
pub type cna_cnb_document_get_origin_size_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_document_copy_origin_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_document_get_container_major_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut u16,
) -> CNA_Result;
pub type cna_cnb_document_get_container_minor_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut u16,
) -> CNA_Result;
pub type cna_cnb_document_get_asset_type_id_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut u32,
) -> CNA_Result;
pub type cna_cnb_document_get_asset_schema_version_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut u32,
) -> CNA_Result;
pub type cna_cnb_document_get_chunk_count_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_document_get_metadata_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut CNA_CnbMetadata,
) -> CNA_Result;
pub type cna_cnb_document_get_metadata_asset_type_name_size_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_document_copy_metadata_asset_type_name_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_document_get_metadata_content_name_size_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_document_copy_metadata_content_name_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_get_asset_type_name_size_fn = unsafe extern "C" fn(u32, *mut u64) -> CNA_Result;
pub type cna_cnb_copy_asset_type_name_fn = unsafe extern "C" fn(
    u32, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_is_custom_asset_type_id_fn = unsafe extern "C" fn(
    u32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_cnb_asset_type_id_from_name_fn = unsafe extern "C" fn(
    CNA_StringView, *mut u32,
) -> CNA_Result;
pub type cna_cnb_decode_texture2d_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut CNA_CnbTextureDataHandle,
) -> CNA_Result;
pub type cna_cnb_encode_texture2d_fn = unsafe extern "C" fn(
    CNA_CnbTextureDataHandle, CNA_StringView, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_texture_data_create_rgba8_fn = unsafe extern "C" fn(
    u32, u32, *const u8, u64, *mut CNA_CnbTextureDataHandle,
) -> CNA_Result;
pub type cna_cnb_texture_data_destroy_fn = unsafe extern "C" fn(
    CNA_CnbTextureDataHandle,
) -> CNA_Result;
pub type cna_cnb_texture_data_get_info_fn = unsafe extern "C" fn(
    CNA_CnbTextureDataHandle, *mut CNA_CnbTextureInfo,
) -> CNA_Result;
pub type cna_cnb_texture_data_get_level_dimensions_fn = unsafe extern "C" fn(
    CNA_CnbTextureDataHandle, u32, *mut u32, *mut u32, *mut u32,
) -> CNA_Result;
pub type cna_cnb_texture_data_get_representation_count_fn = unsafe extern "C" fn(
    CNA_CnbTextureDataHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_texture_data_get_representation_format_fn = unsafe extern "C" fn(
    CNA_CnbTextureDataHandle, u64, *mut CNA_CnbTextureFormat,
) -> CNA_Result;
pub type cna_cnb_texture_data_get_level_count_fn = unsafe extern "C" fn(
    CNA_CnbTextureDataHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_texture_data_copy_level_fn = unsafe extern "C" fn(
    CNA_CnbTextureDataHandle, u64, u64, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_render_pipeline_settings_ext_init_fn = unsafe extern "C" fn(
    *mut CNA_RenderPipelineSettingsEXT,
) -> CNA_Result;
pub type cna_render_pipeline_settings_ext_normalize_fn = unsafe extern "C" fn(
    *mut CNA_RenderPipelineSettingsEXT,
) -> CNA_Result;
pub type cna_render_pipeline_settings_ext_apply_render_quality_preset_fn = unsafe extern "C" fn(
    *mut CNA_RenderPipelineSettingsEXT,
) -> CNA_Result;
pub type cna_render_pipeline_settings_ext_apply_from_string_fn = unsafe extern "C" fn(
    *mut CNA_RenderPipelineSettingsEXT, CNA_StringView, *mut i32,
) -> CNA_Result;
pub type cna_pbr_material_ext_init_fn = unsafe extern "C" fn(*mut CNA_PbrMaterialEXT) -> CNA_Result;
pub type cna_pbr_effect_apply_material_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *const CNA_PbrMaterialEXT,
) -> CNA_Result;
pub type cna_pbr_effect_extract_material_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_PbrMaterialEXT,
) -> CNA_Result;
pub type cna_pbr_material_apply_state_fn = unsafe extern "C" fn(
    *const CNA_PbrMaterialEXT, CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_init_fn = unsafe extern "C" fn(*mut CNA_PbrMaterial) -> CNA_Result;
pub type cna_render_pipeline_settings_init_fn = unsafe extern "C" fn(
    *mut CNA_RenderPipelineSettings,
) -> CNA_Result;
pub type cna_engine_layer_get_version_fn = unsafe extern "C" fn(*mut i32) -> CNA_Result;
pub type cna_engine_layer_copy_version_string_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_pbr_effect_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_pbr_effect_get_metallic_factor_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_effect_set_metallic_factor_fn = unsafe extern "C" fn(
    CNA_EffectHandle, f32,
) -> CNA_Result;
pub type cna_pbr_effect_get_roughness_factor_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_effect_set_roughness_factor_fn = unsafe extern "C" fn(
    CNA_EffectHandle, f32,
) -> CNA_Result;
pub type cna_pbr_effect_get_alpha_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_effect_set_alpha_fn = unsafe extern "C" fn(CNA_EffectHandle, f32) -> CNA_Result;
pub type cna_pbr_effect_get_diffuse_color_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_pbr_effect_set_diffuse_color_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_Vector3,
) -> CNA_Result;
pub type cna_pbr_effect_get_emissive_factor_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_pbr_effect_set_emissive_factor_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_Vector3,
) -> CNA_Result;
pub type cna_pbr_effect_get_alpha_cutoff_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_effect_set_alpha_cutoff_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, f32,
) -> CNA_Result;
pub type cna_pbr_effect_get_normal_scale_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_effect_set_normal_scale_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, f32,
) -> CNA_Result;
pub type cna_pbr_effect_get_occlusion_strength_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_effect_set_occlusion_strength_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, f32,
) -> CNA_Result;
pub type cna_pbr_effect_get_ior_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_effect_set_ior_ext_fn = unsafe extern "C" fn(CNA_EffectHandle, f32) -> CNA_Result;
pub type cna_pbr_effect_get_specular_factor_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_effect_set_specular_factor_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, f32,
) -> CNA_Result;
pub type cna_pbr_effect_get_double_sided_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_effect_set_double_sided_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_effect_get_alpha_mode_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_AlphaModeEXT,
) -> CNA_Result;
pub type cna_pbr_effect_set_alpha_mode_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_AlphaModeEXT,
) -> CNA_Result;
pub type cna_pbr_effect_get_vertex_color_enabled_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_effect_set_vertex_color_enabled_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_material_extensions_create_fn = unsafe extern "C" fn(
    *mut CNA_PbrMaterialExtensionsHandle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_destroy_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_copy_from_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, CNA_PbrMaterialExtensionsHandle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_equals_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, CNA_PbrMaterialExtensionsHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_hash_code_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut u64,
) -> CNA_Result;
pub type cna_pbr_material_extensions_is_neutral_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_material_extensions_is_sheen_enabled_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_material_extensions_is_transmission_enabled_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_material_extensions_is_iridescence_enabled_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_material_extensions_is_subsurface_enabled_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_clearcoat_factor_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_clearcoat_factor_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_clearcoat_roughness_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_clearcoat_roughness_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_clearcoat_normal_scale_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_clearcoat_normal_scale_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_sheen_roughness_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_sheen_roughness_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_transmission_factor_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_transmission_factor_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_thickness_factor_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_thickness_factor_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_attenuation_distance_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_attenuation_distance_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_iridescence_factor_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_iridescence_factor_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_iridescence_ior_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_iridescence_ior_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_iridescence_thickness_minimum_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_iridescence_thickness_minimum_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_iridescence_thickness_maximum_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_iridescence_thickness_maximum_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_subsurface_wrap_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_subsurface_wrap_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, f32,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_sheen_color_factor_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_sheen_color_factor_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_attenuation_color_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_attenuation_color_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_subsurface_color_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_subsurface_color_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_haptic_effect_init_fn = unsafe extern "C" fn(*mut CNA_HapticEffect) -> CNA_Result;
pub type cna_haptic_effect_equals_fn = unsafe extern "C" fn(
    *const CNA_HapticEffect, *const u16, u64, *const CNA_HapticEffect, *const u16, u64, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_direction_init_fn = unsafe extern "C" fn(
    *mut CNA_HapticDirection,
) -> CNA_Result;
pub type cna_haptic_direction_equals_fn = unsafe extern "C" fn(
    *const CNA_HapticDirection, *const CNA_HapticDirection, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_create_effect_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, *const CNA_HapticEffect, *const u16, u64, *mut i32,
) -> CNA_Result;
pub type cna_haptic_device_run_effect_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, i32, u32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_stop_effect_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, i32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_update_effect_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, i32, *const CNA_HapticEffect, *const u16, u64, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_destroy_effect_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, i32,
) -> CNA_Result;
pub type cna_haptic_device_get_effect_status_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, i32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_get_is_effect_supported_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, *const CNA_HapticEffect, *const u16, u64, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptics_get_count_fn = unsafe extern "C" fn(CNA_Handle, *mut u32) -> CNA_Result;
pub type cna_haptics_get_id_at_fn = unsafe extern "C" fn(CNA_Handle, u32, *mut u32) -> CNA_Result;
pub type cna_haptics_get_name_size_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut u64,
) -> CNA_Result;
pub type cna_haptics_copy_name_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_haptics_open_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut CNA_HapticDeviceHandle,
) -> CNA_Result;
pub type cna_haptics_open_from_joystick_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut CNA_HapticDeviceHandle,
) -> CNA_Result;
pub type cna_haptics_open_from_mouse_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_HapticDeviceHandle,
) -> CNA_Result;
pub type cna_haptics_get_is_joystick_haptic_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptics_get_is_mouse_haptic_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_destroy_fn = unsafe extern "C" fn(CNA_HapticDeviceHandle) -> CNA_Result;
pub type cna_haptic_device_dispose_fn = unsafe extern "C" fn(CNA_HapticDeviceHandle) -> CNA_Result;
pub type cna_haptic_device_get_is_open_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_get_capabilities_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, *mut CNA_HapticCapabilities,
) -> CNA_Result;
pub type cna_haptic_device_init_rumble_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_play_rumble_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, f32, u32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_stop_rumble_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_set_gain_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, i32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_set_autocenter_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, i32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_pause_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_resume_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_stop_all_effects_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_haptic_device_get_name_size_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, *mut u64,
) -> CNA_Result;
pub type cna_haptic_device_copy_name_fn = unsafe extern "C" fn(
    CNA_HapticDeviceHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_haptic_capabilities_init_fn = unsafe extern "C" fn(
    *mut CNA_HapticCapabilities,
) -> CNA_Result;
pub type cna_haptic_capabilities_equals_fn = unsafe extern "C" fn(
    *const CNA_HapticCapabilities, CNA_StringView, *const CNA_HapticCapabilities, CNA_StringView, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_sensors_get_count_fn = unsafe extern "C" fn(CNA_Handle, *mut u32) -> CNA_Result;
pub type cna_sensors_get_info_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut CNA_SensorInfo,
) -> CNA_Result;
pub type cna_sensors_get_name_size_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut u64,
) -> CNA_Result;
pub type cna_sensors_copy_name_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_accelerometer_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_AccelerometerHandle,
) -> CNA_Result;
pub type cna_accelerometer_destroy_fn = unsafe extern "C" fn(CNA_AccelerometerHandle) -> CNA_Result;
pub type cna_accelerometer_dispose_fn = unsafe extern "C" fn(CNA_AccelerometerHandle) -> CNA_Result;
pub type cna_accelerometer_get_is_supported_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_accelerometer_get_state_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle, *mut CNA_SensorState,
) -> CNA_Result;
pub type cna_accelerometer_get_is_data_valid_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_accelerometer_get_current_value_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle, *mut CNA_AccelerometerReading,
) -> CNA_Result;
pub type cna_accelerometer_get_time_between_updates_ticks_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle, *mut i64,
) -> CNA_Result;
pub type cna_accelerometer_set_time_between_updates_ticks_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle, i64,
) -> CNA_Result;
pub type cna_accelerometer_start_fn = unsafe extern "C" fn(CNA_AccelerometerHandle) -> CNA_Result;
pub type cna_accelerometer_stop_fn = unsafe extern "C" fn(CNA_AccelerometerHandle) -> CNA_Result;
pub type cna_accelerometer_inject_synthetic_update_ext_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle, f32, f32, f32,
) -> CNA_Result;
pub type cna_compass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_CompassHandle,
) -> CNA_Result;
pub type cna_compass_destroy_fn = unsafe extern "C" fn(CNA_CompassHandle) -> CNA_Result;
pub type cna_compass_dispose_fn = unsafe extern "C" fn(CNA_CompassHandle) -> CNA_Result;
pub type cna_compass_get_is_supported_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_compass_get_state_fn = unsafe extern "C" fn(
    CNA_CompassHandle, *mut CNA_SensorState,
) -> CNA_Result;
pub type cna_compass_get_is_data_valid_fn = unsafe extern "C" fn(
    CNA_CompassHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_compass_get_current_value_fn = unsafe extern "C" fn(
    CNA_CompassHandle, *mut CNA_CompassReading,
) -> CNA_Result;
pub type cna_compass_get_time_between_updates_ticks_fn = unsafe extern "C" fn(
    CNA_CompassHandle, *mut i64,
) -> CNA_Result;
pub type cna_compass_set_time_between_updates_ticks_fn = unsafe extern "C" fn(
    CNA_CompassHandle, i64,
) -> CNA_Result;
pub type cna_compass_start_fn = unsafe extern "C" fn(CNA_CompassHandle) -> CNA_Result;
pub type cna_compass_stop_fn = unsafe extern "C" fn(CNA_CompassHandle) -> CNA_Result;
pub type cna_compass_inject_synthetic_update_ext_fn = unsafe extern "C" fn(
    CNA_CompassHandle, *const CNA_CompassReading,
) -> CNA_Result;
pub type cna_gyroscope_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_GyroscopeHandle,
) -> CNA_Result;
pub type cna_gyroscope_destroy_fn = unsafe extern "C" fn(CNA_GyroscopeHandle) -> CNA_Result;
pub type cna_gyroscope_dispose_fn = unsafe extern "C" fn(CNA_GyroscopeHandle) -> CNA_Result;
pub type cna_gyroscope_get_is_supported_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gyroscope_get_state_fn = unsafe extern "C" fn(
    CNA_GyroscopeHandle, *mut CNA_SensorState,
) -> CNA_Result;
pub type cna_gyroscope_get_is_data_valid_fn = unsafe extern "C" fn(
    CNA_GyroscopeHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gyroscope_get_current_value_fn = unsafe extern "C" fn(
    CNA_GyroscopeHandle, *mut CNA_GyroscopeReading,
) -> CNA_Result;
pub type cna_gyroscope_get_time_between_updates_ticks_fn = unsafe extern "C" fn(
    CNA_GyroscopeHandle, *mut i64,
) -> CNA_Result;
pub type cna_gyroscope_set_time_between_updates_ticks_fn = unsafe extern "C" fn(
    CNA_GyroscopeHandle, i64,
) -> CNA_Result;
pub type cna_gyroscope_start_fn = unsafe extern "C" fn(CNA_GyroscopeHandle) -> CNA_Result;
pub type cna_gyroscope_stop_fn = unsafe extern "C" fn(CNA_GyroscopeHandle) -> CNA_Result;
pub type cna_gyroscope_inject_synthetic_update_ext_fn = unsafe extern "C" fn(
    CNA_GyroscopeHandle, f32, f32, f32,
) -> CNA_Result;
pub type cna_input_devices_get_keyboard_count_fn = unsafe extern "C" fn(
    CNA_Handle, *mut u32,
) -> CNA_Result;
pub type cna_input_devices_get_keyboard_info_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut CNA_InputDeviceInfo,
) -> CNA_Result;
pub type cna_input_devices_get_keyboard_name_size_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut u64,
) -> CNA_Result;
pub type cna_input_devices_copy_keyboard_name_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_input_devices_get_mouse_count_fn = unsafe extern "C" fn(
    CNA_Handle, *mut u32,
) -> CNA_Result;
pub type cna_input_devices_get_mouse_info_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut CNA_InputDeviceInfo,
) -> CNA_Result;
pub type cna_input_devices_get_mouse_name_size_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut u64,
) -> CNA_Result;
pub type cna_input_devices_copy_mouse_name_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_input_devices_get_touch_device_count_fn = unsafe extern "C" fn(
    CNA_Handle, *mut u32,
) -> CNA_Result;
pub type cna_input_devices_get_touch_device_info_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut CNA_InputDeviceInfo,
) -> CNA_Result;
pub type cna_input_devices_get_touch_device_name_size_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut u64,
) -> CNA_Result;
pub type cna_input_devices_copy_touch_device_name_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_input_devices_subscribe_keyboard_connected_ext_fn = unsafe extern "C" fn(
    CNA_InputDeviceHotplugCallback, *mut c_void, *mut CNA_InputDeviceEventRegistrationHandle,
) -> CNA_Result;
pub type cna_input_devices_subscribe_keyboard_disconnected_ext_fn = unsafe extern "C" fn(
    CNA_InputDeviceHotplugCallback, *mut c_void, *mut CNA_InputDeviceEventRegistrationHandle,
) -> CNA_Result;
pub type cna_input_devices_subscribe_mouse_connected_ext_fn = unsafe extern "C" fn(
    CNA_InputDeviceHotplugCallback, *mut c_void, *mut CNA_InputDeviceEventRegistrationHandle,
) -> CNA_Result;
pub type cna_input_devices_subscribe_mouse_disconnected_ext_fn = unsafe extern "C" fn(
    CNA_InputDeviceHotplugCallback, *mut c_void, *mut CNA_InputDeviceEventRegistrationHandle,
) -> CNA_Result;
pub type cna_input_devices_unsubscribe_ext_fn = unsafe extern "C" fn(
    CNA_InputDeviceEventRegistrationHandle,
) -> CNA_Result;
pub type cna_input_devices_raise_keyboard_connected_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u32,
) -> CNA_Result;
pub type cna_input_devices_raise_keyboard_disconnected_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u32,
) -> CNA_Result;
pub type cna_input_devices_raise_mouse_connected_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u32,
) -> CNA_Result;
pub type cna_input_devices_raise_mouse_disconnected_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u32,
) -> CNA_Result;
pub type cna_input_device_info_init_fn = unsafe extern "C" fn(
    *mut CNA_InputDeviceInfo,
) -> CNA_Result;
pub type cna_input_device_info_equals_fn = unsafe extern "C" fn(
    *const CNA_InputDeviceInfo, CNA_StringView, *const CNA_InputDeviceInfo, CNA_StringView, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_mouse_cursor_create_ext_fn = unsafe extern "C" fn(
    *mut CNA_MouseCursorHandle,
) -> CNA_Result;
pub type cna_mouse_cursor_create_from_texture2d_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_Handle, i32, i32, *mut CNA_MouseCursorHandle,
) -> CNA_Result;
pub type cna_mouse_cursor_get_stock_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_MouseCursorStock, *mut CNA_MouseCursorHandle,
) -> CNA_Result;
pub type cna_mouse_set_cursor_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_MouseCursorHandle,
) -> CNA_Result;
pub type cna_mouse_cursor_destroy_fn = unsafe extern "C" fn(CNA_MouseCursorHandle) -> CNA_Result;
pub type cna_mouse_cursor_dispose_fn = unsafe extern "C" fn(CNA_MouseCursorHandle) -> CNA_Result;
pub type cna_text_input_subscribe_text_input_ext_fn = unsafe extern "C" fn(
    CNA_TextInputCallback, *mut c_void, *mut CNA_TextInputRegistrationHandle,
) -> CNA_Result;
pub type cna_text_input_subscribe_text_editing_ext_fn = unsafe extern "C" fn(
    CNA_TextEditingCallback, *mut c_void, *mut CNA_TextInputRegistrationHandle,
) -> CNA_Result;
pub type cna_text_input_subscribe_text_editing_candidates_ext_fn = unsafe extern "C" fn(
    CNA_TextEditingCandidatesCallback, *mut c_void, *mut CNA_TextInputRegistrationHandle,
) -> CNA_Result;
pub type cna_text_input_unsubscribe_ext_fn = unsafe extern "C" fn(
    CNA_TextInputRegistrationHandle,
) -> CNA_Result;
pub type cna_text_input_raise_text_input_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u16,
) -> CNA_Result;
pub type cna_text_input_raise_text_editing_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_StringView, i32, i32,
) -> CNA_Result;
pub type cna_text_input_raise_text_editing_candidates_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *const CNA_StringView, i32, i32, CNA_Bool,
) -> CNA_Result;
pub type cna_text_input_get_window_handle_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut u64,
) -> CNA_Result;
pub type cna_text_input_set_window_handle_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u64,
) -> CNA_Result;
pub type cna_text_input_is_active_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_text_input_is_screen_keyboard_shown_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_text_input_is_screen_keyboard_shown_for_window_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u64, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_text_input_start_ext_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_text_input_stop_ext_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_text_input_start_with_type_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_TextInputType,
) -> CNA_Result;
pub type cna_text_input_set_input_rectangle_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_Rectangle,
) -> CNA_Result;
pub type cna_cnb_sprite_font_data_create_fn = unsafe extern "C" fn(
    *mut CNA_CnbSpriteFontDataHandle,
) -> CNA_Result;
pub type cna_cnb_sprite_font_data_destroy_fn = unsafe extern "C" fn(
    CNA_CnbSpriteFontDataHandle,
) -> CNA_Result;
pub type cna_cnb_sprite_font_data_set_info_fn = unsafe extern "C" fn(
    CNA_CnbSpriteFontDataHandle, *const CNA_CnbSpriteFontInfo,
) -> CNA_Result;
pub type cna_cnb_sprite_font_data_get_info_fn = unsafe extern "C" fn(
    CNA_CnbSpriteFontDataHandle, *mut CNA_CnbSpriteFontInfo,
) -> CNA_Result;
pub type cna_cnb_sprite_font_data_add_glyph_fn = unsafe extern "C" fn(
    CNA_CnbSpriteFontDataHandle, *const CNA_SpriteFontGlyph, *mut u64,
) -> CNA_Result;
pub type cna_cnb_sprite_font_data_get_glyph_fn = unsafe extern "C" fn(
    CNA_CnbSpriteFontDataHandle, u64, *mut CNA_SpriteFontGlyph,
) -> CNA_Result;
pub type cna_cnb_sprite_font_data_set_atlas_fn = unsafe extern "C" fn(
    CNA_CnbSpriteFontDataHandle, CNA_CnbTextureDataHandle,
) -> CNA_Result;
pub type cna_cnb_sprite_font_data_copy_atlas_fn = unsafe extern "C" fn(
    CNA_CnbSpriteFontDataHandle, *mut CNA_CnbTextureDataHandle,
) -> CNA_Result;
pub type cna_cnb_encode_sprite_font_fn = unsafe extern "C" fn(
    CNA_CnbSpriteFontDataHandle, CNA_StringView, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_decode_sprite_font_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut CNA_CnbSpriteFontDataHandle,
) -> CNA_Result;
pub type cna_cnb_sound_effect_data_create_fn = unsafe extern "C" fn(
    *const CNA_CnbSoundEffectInfo, *const u8, u64, *mut CNA_CnbSoundEffectDataHandle,
) -> CNA_Result;
pub type cna_cnb_sound_effect_data_destroy_fn = unsafe extern "C" fn(
    CNA_CnbSoundEffectDataHandle,
) -> CNA_Result;
pub type cna_cnb_sound_effect_data_get_info_fn = unsafe extern "C" fn(
    CNA_CnbSoundEffectDataHandle, *mut CNA_CnbSoundEffectInfo,
) -> CNA_Result;
pub type cna_cnb_sound_effect_data_copy_samples_fn = unsafe extern "C" fn(
    CNA_CnbSoundEffectDataHandle, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_encode_sound_effect_fn = unsafe extern "C" fn(
    CNA_CnbSoundEffectDataHandle, CNA_StringView, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_decode_sound_effect_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut CNA_CnbSoundEffectDataHandle,
) -> CNA_Result;
pub type cna_cnb_writer_create_fn = unsafe extern "C" fn(
    u32, u32, *mut CNA_CnbWriterHandle,
) -> CNA_Result;
pub type cna_cnb_writer_destroy_fn = unsafe extern "C" fn(CNA_CnbWriterHandle) -> CNA_Result;
pub type cna_cnb_writer_set_metadata_fn = unsafe extern "C" fn(
    CNA_CnbWriterHandle, CNA_StringView, CNA_StringView,
) -> CNA_Result;
pub type cna_cnb_writer_add_chunk_fn = unsafe extern "C" fn(
    CNA_CnbWriterHandle, CNA_CnbChunkId, *const u8, u64, u32, u32,
) -> CNA_Result;
pub type cna_cnb_writer_build_fn = unsafe extern "C" fn(
    CNA_CnbWriterHandle, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_loader_registry_register_fn = unsafe extern "C" fn(
    u32, CNA_StringView, CNA_CnbLoaderCallback, *mut c_void,
) -> CNA_Result;
pub type cna_cnb_loader_registry_remove_fn = unsafe extern "C" fn(u32, *mut CNA_Bool) -> CNA_Result;
pub type cna_cnb_loader_registry_clear_fn = unsafe extern "C" fn() -> CNA_Result;
pub type cna_cnb_loader_registry_is_registered_fn = unsafe extern "C" fn(
    u32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_cnb_loader_registry_find_fn = unsafe extern "C" fn(
    u32, *mut CNA_Bool, *mut CNA_CnbLoaderHandle,
) -> CNA_Result;
pub type cna_cnb_loader_registry_get_registered_type_name_size_fn = unsafe extern "C" fn(
    u32, *mut u64,
) -> CNA_Result;
pub type cna_cnb_loader_registry_copy_registered_type_name_fn = unsafe extern "C" fn(
    u32, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_loader_registry_resolve_for_document_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut CNA_CnbLoaderHandle,
) -> CNA_Result;
pub type cna_cnb_loader_registry_register_builtins_fn = unsafe extern "C" fn() -> CNA_Result;
pub type cna_cnb_loader_destroy_fn = unsafe extern "C" fn(CNA_CnbLoaderHandle) -> CNA_Result;
pub type cna_cnb_loader_invoke_fn = unsafe extern "C" fn(
    CNA_CnbLoaderHandle, CNA_CnbDocumentHandle, CNA_Handle, CNA_StringView, *mut *mut c_void,
) -> CNA_Result;
pub type cna_content_manager_create_fn = unsafe extern "C" fn(
    CNA_Handle, *const CNA_ContentManagerCreateInfo, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_content_manager_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_cnb_read_limits_init_fn = unsafe extern "C" fn(*mut CNA_CnbReadLimits) -> CNA_Result;
pub type cna_cnb_model_create_fn = unsafe extern "C" fn(*mut CNA_CnbModelDataHandle) -> CNA_Result;
pub type cna_cnb_model_destroy_fn = unsafe extern "C" fn(CNA_CnbModelDataHandle) -> CNA_Result;
pub type cna_cnb_model_set_flags_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, CNA_Bool, CNA_Bool,
) -> CNA_Result;
pub type cna_cnb_model_add_bone_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, CNA_StringView, i32, *const f32, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_add_mesh_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, CNA_StringView, i32, *const u32, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_add_part_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, *const CNA_CnbModelPartInfo, CNA_StringView, CNA_StringView, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_set_part_vertex_bytes_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *const u8, u64,
) -> CNA_Result;
pub type cna_cnb_model_set_part_index_bytes_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *const u8, u64,
) -> CNA_Result;
pub type cna_cnb_model_set_material_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *const CNA_CnbMaterialInfo,
) -> CNA_Result;
pub type cna_cnb_model_set_material_texture_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, CNA_CnbMaterialTextureSlot, CNA_StringView,
) -> CNA_Result;
pub type cna_cnb_encode_model_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, CNA_StringView, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_decode_model_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut CNA_CnbModelDataHandle,
) -> CNA_Result;
pub type cna_cnb_model_get_info_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, *mut CNA_CnbModelInfo,
) -> CNA_Result;
pub type cna_cnb_model_get_bone_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut CNA_CnbModelBone,
) -> CNA_Result;
pub type cna_cnb_model_get_bone_name_size_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_bone_name_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_get_mesh_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut CNA_CnbMeshInfo,
) -> CNA_Result;
pub type cna_cnb_model_get_mesh_name_size_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_mesh_name_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_mesh_part_indices_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut u32, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_get_part_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut CNA_CnbModelPartInfo,
) -> CNA_Result;
pub type cna_cnb_model_get_part_name_size_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_part_name_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_part_vertex_bytes_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_part_index_bytes_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_get_material_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut CNA_CnbMaterialInfo,
) -> CNA_Result;
pub type cna_cnb_model_get_material_texture_size_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, CNA_CnbMaterialTextureSlot, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_material_texture_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, CNA_CnbMaterialTextureSlot, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_texture_format_to_surface_format_fn = unsafe extern "C" fn(
    CNA_CnbTextureFormat, *mut CNA_SurfaceFormat,
) -> CNA_Result;
pub type cna_cnb_get_texture_format_unit_bytes_fn = unsafe extern "C" fn(
    CNA_CnbTextureFormat, *mut u32,
) -> CNA_Result;
pub type cna_cnb_get_texture_format_name_size_fn = unsafe extern "C" fn(
    CNA_CnbTextureFormat, *mut u64,
) -> CNA_Result;
pub type cna_cnb_copy_texture_format_name_fn = unsafe extern "C" fn(
    CNA_CnbTextureFormat, *mut c_char, u64, *mut u64,
) -> CNA_Result;

// --- CNAEXT graphics layer (graphics_ext.h) ---

pub type cna_graphics_ext_is_available_fn = unsafe extern "C" fn(*mut CNA_Bool) -> CNA_Result;
pub type cna_crt_effect_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_crt_effect_get_scanline_intensity_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_crt_effect_set_scanline_intensity_fn = unsafe extern "C" fn(
    CNA_EffectHandle, f32,
) -> CNA_Result;
pub type cna_crt_effect_get_curvature_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_crt_effect_set_curvature_fn = unsafe extern "C" fn(
    CNA_EffectHandle, f32,
) -> CNA_Result;
pub type cna_crt_effect_get_vignette_intensity_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_crt_effect_set_vignette_intensity_fn = unsafe extern "C" fn(
    CNA_EffectHandle, f32,
) -> CNA_Result;
pub type cna_crt_effect_get_mask_intensity_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_crt_effect_set_mask_intensity_fn = unsafe extern "C" fn(
    CNA_EffectHandle, f32,
) -> CNA_Result;
pub type cna_crt_effect_get_mask_type_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_CRTMaskType,
) -> CNA_Result;
pub type cna_crt_effect_set_mask_type_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_CRTMaskType,
) -> CNA_Result;
pub type cna_depth_effect_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_depth_effect_get_mode_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_DepthEffectMode,
) -> CNA_Result;
pub type cna_depth_effect_set_mode_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_DepthEffectMode,
) -> CNA_Result;
pub type cna_depth_effect_get_dither_mode_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_DitherMode,
) -> CNA_Result;
pub type cna_depth_effect_set_dither_mode_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_DitherMode,
) -> CNA_Result;
pub type cna_ascii_post_process_effect_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_AsciiPostProcessEffectHandle,
) -> CNA_Result;
pub type cna_ascii_post_process_effect_get_cell_size_fn = unsafe extern "C" fn(
    CNA_AsciiPostProcessEffectHandle, *mut i32, *mut i32,
) -> CNA_Result;
pub type cna_ascii_post_process_effect_set_cell_size_fn = unsafe extern "C" fn(
    CNA_AsciiPostProcessEffectHandle, i32, i32,
) -> CNA_Result;
pub type cna_ascii_post_process_effect_get_quantize_mode_fn = unsafe extern "C" fn(
    CNA_AsciiPostProcessEffectHandle, *mut CNA_AsciiQuantizeMode,
) -> CNA_Result;
pub type cna_ascii_post_process_effect_set_quantize_mode_fn = unsafe extern "C" fn(
    CNA_AsciiPostProcessEffectHandle, CNA_AsciiQuantizeMode,
) -> CNA_Result;
pub type cna_ascii_post_process_effect_get_last_grid_dimensions_fn = unsafe extern "C" fn(
    CNA_AsciiPostProcessEffectHandle, *mut i32, *mut i32,
) -> CNA_Result;
pub type cna_ascii_post_process_effect_destroy_fn = unsafe extern "C" fn(
    CNA_AsciiPostProcessEffectHandle,
) -> CNA_Result;

// --- CNA raw joystick input (input_joystick.h) ---

pub type cna_joysticks_get_count_fn = unsafe extern "C" fn(CNA_Handle, *mut u32) -> CNA_Result;
pub type cna_joysticks_get_info_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut CNA_JoystickInfo,
) -> CNA_Result;
pub type cna_joysticks_get_name_size_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut u64,
) -> CNA_Result;
pub type cna_joysticks_copy_name_at_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_joysticks_get_capabilities_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut CNA_JoystickCapabilities,
) -> CNA_Result;
pub type cna_joysticks_get_capabilities_name_size_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut u64,
) -> CNA_Result;
pub type cna_joysticks_copy_capabilities_name_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_joysticks_get_capabilities_guid_size_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut u64,
) -> CNA_Result;
pub type cna_joysticks_copy_capabilities_guid_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_joysticks_capture_state_fn = unsafe extern "C" fn(
    CNA_Handle, u32, *mut CNA_JoystickStateHandle,
) -> CNA_Result;
pub type cna_joystick_state_get_axis_count_fn = unsafe extern "C" fn(
    CNA_JoystickStateHandle, *mut u32,
) -> CNA_Result;
pub type cna_joystick_state_copy_axes_fn = unsafe extern "C" fn(
    CNA_JoystickStateHandle, *mut i16, u64, *mut u64,
) -> CNA_Result;
pub type cna_joystick_state_get_button_count_fn = unsafe extern "C" fn(
    CNA_JoystickStateHandle, *mut u32,
) -> CNA_Result;
pub type cna_joystick_state_copy_buttons_fn = unsafe extern "C" fn(
    CNA_JoystickStateHandle, *mut CNA_Bool, u64, *mut u64,
) -> CNA_Result;
pub type cna_joystick_state_get_hat_count_fn = unsafe extern "C" fn(
    CNA_JoystickStateHandle, *mut u32,
) -> CNA_Result;
pub type cna_joystick_state_copy_hats_fn = unsafe extern "C" fn(
    CNA_JoystickStateHandle, *mut CNA_JoystickHatPosition, u64, *mut u64,
) -> CNA_Result;
pub type cna_joystick_state_get_ball_count_fn = unsafe extern "C" fn(
    CNA_JoystickStateHandle, *mut u32,
) -> CNA_Result;
pub type cna_joystick_state_copy_balls_fn = unsafe extern "C" fn(
    CNA_JoystickStateHandle, *mut CNA_Point, u64, *mut u64,
) -> CNA_Result;
pub type cna_joystick_state_equals_fn = unsafe extern "C" fn(
    CNA_JoystickStateHandle, CNA_JoystickStateHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_joystick_state_destroy_fn = unsafe extern "C" fn(
    CNA_JoystickStateHandle,
) -> CNA_Result;

/* ---- GamerServices, Avatar and Net: scalars, handles, layouts and callbacks ---- */

pub type CNA_GamerPresenceMode = u32;
pub type CNA_NotificationPosition = u32;
pub type CNA_GamerZone = u32;
pub type CNA_LeaderboardKey = u32;
pub type CNA_LeaderboardOutcome = u32;
pub type CNA_MessageBoxIcon = u32;
pub type CNA_ControllerSensitivity = u32;
pub type CNA_GameDifficulty = u32;
pub type CNA_GamerPrivilegeSetting = u32;
pub type CNA_RacingCameraAngle = u32;
pub type CNA_AvatarBodyType = u32;
pub type CNA_AvatarRendererState = u32;
pub type CNA_AvatarEyebrow = u32;
pub type CNA_AvatarEye = u32;
pub type CNA_AvatarMouth = u32;
pub type CNA_AvatarAnimationPreset = u32;
pub type CNA_AvatarBone = u32;
pub type CNA_PropertyValueKind = u32;
pub type CNA_NetworkSessionEndReason = u32;
pub type CNA_NetworkSessionJoinError = u32;
pub type CNA_NetworkSessionState = u32;
pub type CNA_NetworkSessionType = u32;
pub type CNA_SendDataOptions = u32;
pub type CNA_NetworkEventType = u32;

pub type CNA_SignedInGamerHandle = CNA_Handle;
pub type CNA_GamerHandle = CNA_Handle;
pub type CNA_GamerProfileHandle = CNA_Handle;
pub type CNA_GamerCollectionHandle = CNA_Handle;
pub type CNA_GamerEnumeratorHandle = CNA_Handle;
pub type CNA_AchievementHandle = CNA_Handle;
pub type CNA_AchievementCollectionHandle = CNA_Handle;
pub type CNA_PropertyDictionaryHandle = CNA_Handle;
pub type CNA_LeaderboardReaderHandle = CNA_Handle;
pub type CNA_LeaderboardEntryHandle = CNA_Handle;
pub type CNA_AvatarDescriptionHandle = CNA_Handle;
pub type CNA_AvatarAnimationHandle = CNA_Handle;
pub type CNA_AvatarRendererHandle = CNA_Handle;
pub type CNA_NetworkSessionPropertiesHandle = CNA_Handle;
pub type CNA_NetworkSessionPropertyEnumeratorHandle = CNA_Handle;
pub type CNA_PacketWriterHandle = CNA_Handle;
pub type CNA_PacketReaderHandle = CNA_Handle;
pub type CNA_NetworkGamerHandle = CNA_Handle;
pub type CNA_NetworkMachineHandle = CNA_Handle;
pub type CNA_AvailableNetworkSessionHandle = CNA_Handle;
pub type CNA_AvailableNetworkSessionCollectionHandle = CNA_Handle;
pub type CNA_NetworkSessionHandle = CNA_Handle;
pub type CNA_NetworkSessionEventRegistrationHandle = CNA_Handle;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_InviteAcceptedEventInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub gamer: CNA_SignedInGamerHandle,
    pub is_current_session: CNA_Bool,
    pub reserved: [u8; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GamerPresence {
    pub struct_size: u32,
    pub struct_version: u32,
    pub presence_mode: CNA_GamerPresenceMode,
    pub presence_value: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GamerPrivileges {
    pub struct_size: u32,
    pub struct_version: u32,
    pub allow_communication: CNA_GamerPrivilegeSetting,
    pub allow_profile_viewing: CNA_GamerPrivilegeSetting,
    pub allow_user_created_content: CNA_GamerPrivilegeSetting,
    pub allow_online_sessions: CNA_Bool,
    pub allow_premium_content: CNA_Bool,
    pub allow_purchase_content: CNA_Bool,
    pub allow_trade_content: CNA_Bool,
    pub reserved: [u8; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GamerProfileInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub gamer_score: i32,
    pub gamer_zone: CNA_GamerZone,
    pub titles_played: i32,
    pub total_achievements: i32,
    pub reputation: f32,
    pub is_disposed: CNA_Bool,
    pub reserved: [u8; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_FriendGamerInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub friend_request_received_from: CNA_Bool,
    pub friend_request_sent_to: CNA_Bool,
    pub has_voice: CNA_Bool,
    pub invite_accepted: CNA_Bool,
    pub invite_received_from: CNA_Bool,
    pub invite_rejected: CNA_Bool,
    pub invite_sent_to: CNA_Bool,
    pub is_away: CNA_Bool,
    pub is_busy: CNA_Bool,
    pub is_joinable: CNA_Bool,
    pub is_online: CNA_Bool,
    pub is_playing: CNA_Bool,
    pub reserved: [u8; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_SignedInGamerEventInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub reserved: u32,
    pub gamer: CNA_SignedInGamerHandle,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_AchievementInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub gamer_score: i32,
    pub display_before_earned: CNA_Bool,
    pub earned_online: CNA_Bool,
    pub is_earned: CNA_Bool,
    pub reserved: u8,
    pub earned_date_time_ticks: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GameDefaults {
    pub struct_size: u32,
    pub struct_version: u32,
    pub game_difficulty: CNA_GameDifficulty,
    pub controller_sensitivity: CNA_ControllerSensitivity,
    pub racing_camera_angle: CNA_RacingCameraAngle,
    pub has_primary_color: CNA_Bool,
    pub has_secondary_color: CNA_Bool,
    pub auto_aim: CNA_Bool,
    pub auto_center: CNA_Bool,
    pub move_with_right_thumb_stick: CNA_Bool,
    pub invert_y_axis: CNA_Bool,
    pub manual_transmission: CNA_Bool,
    pub accelerate_with_buttons: CNA_Bool,
    pub brake_with_buttons: CNA_Bool,
    pub reserved: [u8; 3],
    pub primary_color: CNA_Color,
    pub secondary_color: CNA_Color,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_LeaderboardIdentity {
    pub struct_size: u32,
    pub struct_version: u32,
    pub game_mode: i32,
    pub key: [c_char; 64],
}

impl Default for CNA_LeaderboardIdentity {
    fn default() -> Self {
        Self {
            struct_size: 0,
            struct_version: 0,
            game_mode: 0,
            key: [0; 64],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_LeaderboardReaderInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub page_start: i32,
    pub total_leaderboard_size: i32,
    pub entry_count: i32,
    pub is_disposed: CNA_Bool,
    pub can_page_down: CNA_Bool,
    pub can_page_up: CNA_Bool,
    pub reserved: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_LeaderboardEntryInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub ranking: i32,
    pub has_gamer: CNA_Bool,
    pub reserved: [u8; 3],
    pub rating: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_AvatarExpression {
    pub struct_size: u32,
    pub struct_version: u32,
    pub mouth: CNA_AvatarMouth,
    pub left_eye: CNA_AvatarEye,
    pub right_eye: CNA_AvatarEye,
    pub left_eyebrow: CNA_AvatarEyebrow,
    pub right_eyebrow: CNA_AvatarEyebrow,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_AvatarAppearanceEXT {
    pub struct_size: u32,
    pub struct_version: u32,
    pub skin_color: CNA_Color,
    pub hair_color: CNA_Color,
    pub shirt_color: CNA_Color,
    pub pants_color: CNA_Color,
    pub shoes_color: CNA_Color,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_AvatarDescriptionInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub body_type: CNA_AvatarBodyType,
    pub height: f32,
    pub description_byte_count: u64,
    pub is_valid: CNA_Bool,
    pub reserved: [u8; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_AvatarAnimationInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub bone_transform_count: i32,
    pub is_disposed: CNA_Bool,
    pub reserved: [u8; 3],
    pub current_position_ticks: i64,
    pub length_ticks: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_AvatarRendererInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub state: CNA_AvatarRendererState,
    pub is_disposed: CNA_Bool,
    pub is_real_rendering_enabled: CNA_Bool,
    pub reserved: [u8; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_QualityOfService {
    pub struct_size: u32,
    pub struct_version: u32,
    pub is_available: CNA_Bool,
    pub reserved: [u8; 7],
    pub average_roundtrip_ticks: i64,
    pub minimum_roundtrip_ticks: i64,
    pub bytes_per_second_downstream: i32,
    pub bytes_per_second_upstream: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_OptionalInt32 {
    pub has_value: CNA_Bool,
    pub reserved: [u8; 3],
    pub value: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GameEndedEventInfo {
    pub struct_size: u32,
    pub struct_version: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GameStartedEventInfo {
    pub struct_size: u32,
    pub struct_version: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GamerJoinedEventInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub gamer: CNA_NetworkGamerHandle,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_GamerLeftEventInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub gamer: CNA_NetworkGamerHandle,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_HostChangedEventInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub old_host: CNA_NetworkGamerHandle,
    pub new_host: CNA_NetworkGamerHandle,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_NetworkSessionEndedEventInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub end_reason: CNA_NetworkSessionEndReason,
    pub reserved: [u8; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CNA_WriteLeaderboardsEventInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub gamer: CNA_NetworkGamerHandle,
    pub is_leaving: CNA_Bool,
    pub reserved: [u8; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_AvailableNetworkSessionCreateInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub current_gamer_count: i32,
    pub open_private_gamer_slots: i32,
    pub open_public_gamer_slots: i32,
    pub session_type: CNA_NetworkSessionType,
    pub host_port: u16,
    pub reserved: [u8; 6],
    pub host_gamertag: CNA_StringView,
    pub host_address: CNA_StringView,
    pub session_properties: CNA_NetworkSessionPropertiesHandle,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_NetworkEventInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub r#type: CNA_NetworkEventType,
    pub reliable: CNA_SendDataOptions,
    pub state: CNA_NetworkSessionState,
    pub reason: CNA_NetworkSessionEndReason,
    pub gamer: CNA_NetworkGamerHandle,
    pub sender: CNA_NetworkGamerHandle,
    pub packet: *const u8,
    pub packet_byte_count: u64,
}

pub type CNA_SignedInGamerEventCallback = Option<
    unsafe extern "C" fn(
        *mut c_void,
        *const CNA_SignedInGamerEventInfo,
    ),
>;
pub type CNA_GamerAsyncCallback = Option<unsafe extern "C" fn(*mut c_void)>;
pub type CNA_NetworkSessionAsyncCallback = Option<unsafe extern "C" fn(*mut c_void)>;
pub type CNA_GameStartedCallback = Option<
    unsafe extern "C" fn(
        CNA_NetworkSessionHandle,
        *const CNA_GameStartedEventInfo,
        *mut c_void,
    ),
>;
pub type CNA_GameEndedCallback = Option<
    unsafe extern "C" fn(
        CNA_NetworkSessionHandle,
        *const CNA_GameEndedEventInfo,
        *mut c_void,
    ),
>;
pub type CNA_GamerJoinedCallback = Option<
    unsafe extern "C" fn(
        CNA_NetworkSessionHandle,
        *const CNA_GamerJoinedEventInfo,
        *mut c_void,
    ),
>;
pub type CNA_GamerLeftCallback = Option<
    unsafe extern "C" fn(
        CNA_NetworkSessionHandle,
        *const CNA_GamerLeftEventInfo,
        *mut c_void,
    ),
>;
pub type CNA_HostChangedCallback = Option<
    unsafe extern "C" fn(
        CNA_NetworkSessionHandle,
        *const CNA_HostChangedEventInfo,
        *mut c_void,
    ),
>;
pub type CNA_NetworkSessionEndedCallback = Option<
    unsafe extern "C" fn(
        CNA_NetworkSessionHandle,
        *const CNA_NetworkSessionEndedEventInfo,
        *mut c_void,
    ),
>;
pub type CNA_WriteLeaderboardsCallback = Option<
    unsafe extern "C" fn(
        CNA_NetworkSessionHandle,
        *const CNA_WriteLeaderboardsEventInfo,
        *mut c_void,
    ),
>;
pub type CNA_InviteAcceptedCallback = Option<
    unsafe extern "C" fn(
        *const CNA_InviteAcceptedEventInfo,
        *mut c_void,
    ),
>;

/* ---- GamerServices, Avatar and Net canonical identities ---- */

pub const CNA_GAMER_PRESENCE_MODE_NONE: CNA_GamerPresenceMode = 0;
pub const CNA_GAMER_PRESENCE_MODE_SINGLE_PLAYER: CNA_GamerPresenceMode = 1;
pub const CNA_GAMER_PRESENCE_MODE_MULTIPLAYER: CNA_GamerPresenceMode = 2;
pub const CNA_GAMER_PRESENCE_MODE_LOCAL_CO_OP: CNA_GamerPresenceMode = 3;
pub const CNA_GAMER_PRESENCE_MODE_LOCAL_VERSUS: CNA_GamerPresenceMode = 4;
pub const CNA_GAMER_PRESENCE_MODE_ONLINE_CO_OP: CNA_GamerPresenceMode = 5;
pub const CNA_GAMER_PRESENCE_MODE_ONLINE_VERSUS: CNA_GamerPresenceMode = 6;
pub const CNA_GAMER_PRESENCE_MODE_VERSUS_COMPUTER: CNA_GamerPresenceMode = 7;
pub const CNA_GAMER_PRESENCE_MODE_STAGE: CNA_GamerPresenceMode = 8;
pub const CNA_GAMER_PRESENCE_MODE_LEVEL: CNA_GamerPresenceMode = 9;
pub const CNA_GAMER_PRESENCE_MODE_CO_OP_STAGE: CNA_GamerPresenceMode = 10;
pub const CNA_GAMER_PRESENCE_MODE_CO_OP_LEVEL: CNA_GamerPresenceMode = 11;
pub const CNA_GAMER_PRESENCE_MODE_ARCADE_MODE: CNA_GamerPresenceMode = 12;
pub const CNA_GAMER_PRESENCE_MODE_CAMPAIGN_MODE: CNA_GamerPresenceMode = 13;
pub const CNA_GAMER_PRESENCE_MODE_CHALLENGE_MODE: CNA_GamerPresenceMode = 14;
pub const CNA_GAMER_PRESENCE_MODE_EXPLORATION_MODE: CNA_GamerPresenceMode = 15;
pub const CNA_GAMER_PRESENCE_MODE_PRACTICE_MODE: CNA_GamerPresenceMode = 16;
pub const CNA_GAMER_PRESENCE_MODE_PUZZLE_MODE: CNA_GamerPresenceMode = 17;
pub const CNA_GAMER_PRESENCE_MODE_SCENARIO_MODE: CNA_GamerPresenceMode = 18;
pub const CNA_GAMER_PRESENCE_MODE_STORY_MODE: CNA_GamerPresenceMode = 19;
pub const CNA_GAMER_PRESENCE_MODE_SURVIVAL_MODE: CNA_GamerPresenceMode = 20;
pub const CNA_GAMER_PRESENCE_MODE_TUTORIAL_MODE: CNA_GamerPresenceMode = 21;
pub const CNA_GAMER_PRESENCE_MODE_DIFFICULTY_EASY: CNA_GamerPresenceMode = 22;
pub const CNA_GAMER_PRESENCE_MODE_DIFFICULTY_MEDIUM: CNA_GamerPresenceMode = 23;
pub const CNA_GAMER_PRESENCE_MODE_DIFFICULTY_HARD: CNA_GamerPresenceMode = 24;
pub const CNA_GAMER_PRESENCE_MODE_DIFFICULTY_EXTREME: CNA_GamerPresenceMode = 25;
pub const CNA_GAMER_PRESENCE_MODE_SCORE: CNA_GamerPresenceMode = 26;
pub const CNA_GAMER_PRESENCE_MODE_VERSUS_SCORE: CNA_GamerPresenceMode = 27;
pub const CNA_GAMER_PRESENCE_MODE_WINNING: CNA_GamerPresenceMode = 28;
pub const CNA_GAMER_PRESENCE_MODE_LOSING: CNA_GamerPresenceMode = 29;
pub const CNA_GAMER_PRESENCE_MODE_SCORE_IS_TIED: CNA_GamerPresenceMode = 30;
pub const CNA_GAMER_PRESENCE_MODE_OUTNUMBERED: CNA_GamerPresenceMode = 31;
pub const CNA_GAMER_PRESENCE_MODE_ON_A_ROLL: CNA_GamerPresenceMode = 32;
pub const CNA_GAMER_PRESENCE_MODE_IN_COMBAT: CNA_GamerPresenceMode = 33;
pub const CNA_GAMER_PRESENCE_MODE_BATTLING_BOSS: CNA_GamerPresenceMode = 34;
pub const CNA_GAMER_PRESENCE_MODE_TIME_ATTACK: CNA_GamerPresenceMode = 35;
pub const CNA_GAMER_PRESENCE_MODE_TRYING_FOR_RECORD: CNA_GamerPresenceMode = 36;
pub const CNA_GAMER_PRESENCE_MODE_FREE_PLAY: CNA_GamerPresenceMode = 37;
pub const CNA_GAMER_PRESENCE_MODE_WASTING_TIME: CNA_GamerPresenceMode = 38;
pub const CNA_GAMER_PRESENCE_MODE_STUCK_ON_A_HARD_BIT: CNA_GamerPresenceMode = 39;
pub const CNA_GAMER_PRESENCE_MODE_NEARLY_FINISHED: CNA_GamerPresenceMode = 40;
pub const CNA_GAMER_PRESENCE_MODE_LOOKING_FOR_GAMES: CNA_GamerPresenceMode = 41;
pub const CNA_GAMER_PRESENCE_MODE_WAITING_FOR_PLAYERS: CNA_GamerPresenceMode = 42;
pub const CNA_GAMER_PRESENCE_MODE_WAITING_IN_LOBBY: CNA_GamerPresenceMode = 43;
pub const CNA_GAMER_PRESENCE_MODE_SETTING_UP_MATCH: CNA_GamerPresenceMode = 44;
pub const CNA_GAMER_PRESENCE_MODE_PLAYING_WITH_FRIENDS: CNA_GamerPresenceMode = 45;
pub const CNA_GAMER_PRESENCE_MODE_AT_MENU: CNA_GamerPresenceMode = 46;
pub const CNA_GAMER_PRESENCE_MODE_STARTING_GAME: CNA_GamerPresenceMode = 47;
pub const CNA_GAMER_PRESENCE_MODE_PAUSED: CNA_GamerPresenceMode = 48;
pub const CNA_GAMER_PRESENCE_MODE_GAME_OVER: CNA_GamerPresenceMode = 49;
pub const CNA_GAMER_PRESENCE_MODE_WON_THE_GAME: CNA_GamerPresenceMode = 50;
pub const CNA_GAMER_PRESENCE_MODE_CONFIGURING_SETTINGS: CNA_GamerPresenceMode = 51;
pub const CNA_GAMER_PRESENCE_MODE_CUSTOMIZING_PLAYER: CNA_GamerPresenceMode = 52;
pub const CNA_GAMER_PRESENCE_MODE_EDITING_LEVEL: CNA_GamerPresenceMode = 53;
pub const CNA_GAMER_PRESENCE_MODE_IN_GAME_STORE: CNA_GamerPresenceMode = 54;
pub const CNA_GAMER_PRESENCE_MODE_WATCHING_CUTSCENE: CNA_GamerPresenceMode = 55;
pub const CNA_GAMER_PRESENCE_MODE_WATCHING_CREDITS: CNA_GamerPresenceMode = 56;
pub const CNA_GAMER_PRESENCE_MODE_PLAYING_MINIGAME: CNA_GamerPresenceMode = 57;
pub const CNA_GAMER_PRESENCE_MODE_FOUND_SECRET: CNA_GamerPresenceMode = 58;
pub const CNA_GAMER_PRESENCE_MODE_CORNFLOWER_BLUE: CNA_GamerPresenceMode = 59;
pub const CNA_NOTIFICATION_POSITION_TOP_LEFT: CNA_NotificationPosition = 0;
pub const CNA_NOTIFICATION_POSITION_TOP_CENTER: CNA_NotificationPosition = 1;
pub const CNA_NOTIFICATION_POSITION_TOP_RIGHT: CNA_NotificationPosition = 2;
pub const CNA_NOTIFICATION_POSITION_CENTER_LEFT: CNA_NotificationPosition = 3;
pub const CNA_NOTIFICATION_POSITION_CENTER: CNA_NotificationPosition = 4;
pub const CNA_NOTIFICATION_POSITION_CENTER_RIGHT: CNA_NotificationPosition = 5;
pub const CNA_NOTIFICATION_POSITION_BOTTOM_LEFT: CNA_NotificationPosition = 6;
pub const CNA_NOTIFICATION_POSITION_BOTTOM_CENTER: CNA_NotificationPosition = 7;
pub const CNA_NOTIFICATION_POSITION_BOTTOM_RIGHT: CNA_NotificationPosition = 8;
pub const CNA_GAMER_ZONE_UNKNOWN: CNA_GamerZone = 0;
pub const CNA_GAMER_ZONE_RECREATION: CNA_GamerZone = 1;
pub const CNA_GAMER_ZONE_PRO: CNA_GamerZone = 2;
pub const CNA_GAMER_ZONE_FAMILY: CNA_GamerZone = 3;
pub const CNA_GAMER_ZONE_UNDERGROUND: CNA_GamerZone = 4;
pub const CNA_LEADERBOARD_KEY_BEST_SCORE_LIFE_TIME: CNA_LeaderboardKey = 0;
pub const CNA_LEADERBOARD_KEY_BEST_SCORE_RECENT: CNA_LeaderboardKey = 1;
pub const CNA_LEADERBOARD_KEY_BEST_TIME_LIFE_TIME: CNA_LeaderboardKey = 2;
pub const CNA_LEADERBOARD_KEY_BEST_TIME_RECENT: CNA_LeaderboardKey = 3;
pub const CNA_LEADERBOARD_OUTCOME_NONE: CNA_LeaderboardOutcome = 0;
pub const CNA_LEADERBOARD_OUTCOME_WIN: CNA_LeaderboardOutcome = 1;
pub const CNA_LEADERBOARD_OUTCOME_LOSS: CNA_LeaderboardOutcome = 2;
pub const CNA_LEADERBOARD_OUTCOME_TIE: CNA_LeaderboardOutcome = 3;
pub const CNA_MESSAGE_BOX_ICON_NONE: CNA_MessageBoxIcon = 0;
pub const CNA_MESSAGE_BOX_ICON_ERROR: CNA_MessageBoxIcon = 1;
pub const CNA_MESSAGE_BOX_ICON_WARNING: CNA_MessageBoxIcon = 2;
pub const CNA_MESSAGE_BOX_ICON_ALERT: CNA_MessageBoxIcon = 3;
pub const CNA_CONTROLLER_SENSITIVITY_LOW: CNA_ControllerSensitivity = 0;
pub const CNA_CONTROLLER_SENSITIVITY_MEDIUM: CNA_ControllerSensitivity = 1;
pub const CNA_CONTROLLER_SENSITIVITY_HIGH: CNA_ControllerSensitivity = 2;
pub const CNA_GAME_DIFFICULTY_EASY: CNA_GameDifficulty = 0;
pub const CNA_GAME_DIFFICULTY_NORMAL: CNA_GameDifficulty = 1;
pub const CNA_GAME_DIFFICULTY_HARD: CNA_GameDifficulty = 2;
pub const CNA_GAMER_PRIVILEGE_SETTING_BLOCKED: CNA_GamerPrivilegeSetting = 0;
pub const CNA_GAMER_PRIVILEGE_SETTING_FRIENDS_ONLY: CNA_GamerPrivilegeSetting = 1;
pub const CNA_GAMER_PRIVILEGE_SETTING_EVERYONE: CNA_GamerPrivilegeSetting = 2;
pub const CNA_RACING_CAMERA_ANGLE_BACK: CNA_RacingCameraAngle = 0;
pub const CNA_RACING_CAMERA_ANGLE_FRONT: CNA_RacingCameraAngle = 1;
pub const CNA_RACING_CAMERA_ANGLE_INSIDE: CNA_RacingCameraAngle = 2;
pub const CNA_AVATAR_BODY_TYPE_FEMALE: CNA_AvatarBodyType = 0;
pub const CNA_AVATAR_BODY_TYPE_MALE: CNA_AvatarBodyType = 1;
pub const CNA_AVATAR_RENDERER_STATE_LOADING: CNA_AvatarRendererState = 0;
pub const CNA_AVATAR_RENDERER_STATE_READY: CNA_AvatarRendererState = 1;
pub const CNA_AVATAR_RENDERER_STATE_UNAVAILABLE: CNA_AvatarRendererState = 2;
pub const CNA_AVATAR_EYEBROW_NEUTRAL: CNA_AvatarEyebrow = 0;
pub const CNA_AVATAR_EYEBROW_SAD: CNA_AvatarEyebrow = 1;
pub const CNA_AVATAR_EYEBROW_ANGRY: CNA_AvatarEyebrow = 2;
pub const CNA_AVATAR_EYEBROW_CONFUSED: CNA_AvatarEyebrow = 3;
pub const CNA_AVATAR_EYEBROW_RAISED: CNA_AvatarEyebrow = 4;
pub const CNA_AVATAR_EYE_NEUTRAL: CNA_AvatarEye = 0;
pub const CNA_AVATAR_EYE_SAD: CNA_AvatarEye = 1;
pub const CNA_AVATAR_EYE_ANGRY: CNA_AvatarEye = 2;
pub const CNA_AVATAR_EYE_CONFUSED: CNA_AvatarEye = 3;
pub const CNA_AVATAR_EYE_LAUGHING: CNA_AvatarEye = 4;
pub const CNA_AVATAR_EYE_SHOCKED: CNA_AvatarEye = 5;
pub const CNA_AVATAR_EYE_HAPPY: CNA_AvatarEye = 6;
pub const CNA_AVATAR_EYE_YAWNING: CNA_AvatarEye = 7;
pub const CNA_AVATAR_EYE_SLEEPING: CNA_AvatarEye = 8;
pub const CNA_AVATAR_EYE_LOOK_UP: CNA_AvatarEye = 9;
pub const CNA_AVATAR_EYE_LOOK_DOWN: CNA_AvatarEye = 10;
pub const CNA_AVATAR_EYE_LOOK_LEFT: CNA_AvatarEye = 11;
pub const CNA_AVATAR_EYE_LOOK_RIGHT: CNA_AvatarEye = 12;
pub const CNA_AVATAR_EYE_BLINK: CNA_AvatarEye = 13;
pub const CNA_AVATAR_MOUTH_NEUTRAL: CNA_AvatarMouth = 0;
pub const CNA_AVATAR_MOUTH_SAD: CNA_AvatarMouth = 1;
pub const CNA_AVATAR_MOUTH_ANGRY: CNA_AvatarMouth = 2;
pub const CNA_AVATAR_MOUTH_CONFUSED: CNA_AvatarMouth = 3;
pub const CNA_AVATAR_MOUTH_LAUGHING: CNA_AvatarMouth = 4;
pub const CNA_AVATAR_MOUTH_SHOCKED: CNA_AvatarMouth = 5;
pub const CNA_AVATAR_MOUTH_HAPPY: CNA_AvatarMouth = 6;
pub const CNA_AVATAR_MOUTH_PHONETIC_O: CNA_AvatarMouth = 7;
pub const CNA_AVATAR_MOUTH_PHONETIC_AI: CNA_AvatarMouth = 8;
pub const CNA_AVATAR_MOUTH_PHONETIC_EE: CNA_AvatarMouth = 9;
pub const CNA_AVATAR_MOUTH_PHONETIC_FV: CNA_AvatarMouth = 10;
pub const CNA_AVATAR_MOUTH_PHONETIC_W: CNA_AvatarMouth = 11;
pub const CNA_AVATAR_MOUTH_PHONETIC_L: CNA_AvatarMouth = 12;
pub const CNA_AVATAR_MOUTH_PHONETIC_DTH: CNA_AvatarMouth = 13;
pub const CNA_AVATAR_ANIMATION_PRESET_STAND_0: CNA_AvatarAnimationPreset = 0;
pub const CNA_AVATAR_ANIMATION_PRESET_STAND_1: CNA_AvatarAnimationPreset = 1;
pub const CNA_AVATAR_ANIMATION_PRESET_STAND_2: CNA_AvatarAnimationPreset = 2;
pub const CNA_AVATAR_ANIMATION_PRESET_STAND_3: CNA_AvatarAnimationPreset = 3;
pub const CNA_AVATAR_ANIMATION_PRESET_STAND_4: CNA_AvatarAnimationPreset = 4;
pub const CNA_AVATAR_ANIMATION_PRESET_STAND_5: CNA_AvatarAnimationPreset = 5;
pub const CNA_AVATAR_ANIMATION_PRESET_STAND_6: CNA_AvatarAnimationPreset = 6;
pub const CNA_AVATAR_ANIMATION_PRESET_STAND_7: CNA_AvatarAnimationPreset = 7;
pub const CNA_AVATAR_ANIMATION_PRESET_CLAP: CNA_AvatarAnimationPreset = 8;
pub const CNA_AVATAR_ANIMATION_PRESET_WAVE: CNA_AvatarAnimationPreset = 9;
pub const CNA_AVATAR_ANIMATION_PRESET_CELEBRATE: CNA_AvatarAnimationPreset = 10;
pub const CNA_AVATAR_ANIMATION_PRESET_FEMALE_IDLE_CHECK_NAILS: CNA_AvatarAnimationPreset = 11;
pub const CNA_AVATAR_ANIMATION_PRESET_FEMALE_IDLE_LOOK_AROUND: CNA_AvatarAnimationPreset = 12;
pub const CNA_AVATAR_ANIMATION_PRESET_FEMALE_IDLE_SHIFT_WEIGHT: CNA_AvatarAnimationPreset = 13;
pub const CNA_AVATAR_ANIMATION_PRESET_FEMALE_IDLE_FIX_SHOE: CNA_AvatarAnimationPreset = 14;
pub const CNA_AVATAR_ANIMATION_PRESET_FEMALE_ANGRY: CNA_AvatarAnimationPreset = 15;
pub const CNA_AVATAR_ANIMATION_PRESET_FEMALE_CONFUSED: CNA_AvatarAnimationPreset = 16;
pub const CNA_AVATAR_ANIMATION_PRESET_FEMALE_LAUGH: CNA_AvatarAnimationPreset = 17;
pub const CNA_AVATAR_ANIMATION_PRESET_FEMALE_CRY: CNA_AvatarAnimationPreset = 18;
pub const CNA_AVATAR_ANIMATION_PRESET_FEMALE_SHOCKED: CNA_AvatarAnimationPreset = 19;
pub const CNA_AVATAR_ANIMATION_PRESET_FEMALE_YAWN: CNA_AvatarAnimationPreset = 20;
pub const CNA_AVATAR_ANIMATION_PRESET_MALE_IDLE_LOOK_AROUND: CNA_AvatarAnimationPreset = 21;
pub const CNA_AVATAR_ANIMATION_PRESET_MALE_IDLE_STRETCH: CNA_AvatarAnimationPreset = 22;
pub const CNA_AVATAR_ANIMATION_PRESET_MALE_IDLE_SHIFT_WEIGHT: CNA_AvatarAnimationPreset = 23;
pub const CNA_AVATAR_ANIMATION_PRESET_MALE_IDLE_CHECK_HAND: CNA_AvatarAnimationPreset = 24;
pub const CNA_AVATAR_ANIMATION_PRESET_MALE_ANGRY: CNA_AvatarAnimationPreset = 25;
pub const CNA_AVATAR_ANIMATION_PRESET_MALE_CONFUSED: CNA_AvatarAnimationPreset = 26;
pub const CNA_AVATAR_ANIMATION_PRESET_MALE_LAUGH: CNA_AvatarAnimationPreset = 27;
pub const CNA_AVATAR_ANIMATION_PRESET_MALE_CRY: CNA_AvatarAnimationPreset = 28;
pub const CNA_AVATAR_ANIMATION_PRESET_MALE_SURPRISED: CNA_AvatarAnimationPreset = 29;
pub const CNA_AVATAR_ANIMATION_PRESET_MALE_YAWN: CNA_AvatarAnimationPreset = 30;
pub const CNA_AVATAR_BONE_ROOT: CNA_AvatarBone = 0;
pub const CNA_AVATAR_BONE_BACK_LOWER: CNA_AvatarBone = 1;
pub const CNA_AVATAR_BONE_HIP_LEFT: CNA_AvatarBone = 2;
pub const CNA_AVATAR_BONE_HIP_RIGHT: CNA_AvatarBone = 3;
pub const CNA_AVATAR_BONE_BACK_UPPER: CNA_AvatarBone = 5;
pub const CNA_AVATAR_BONE_KNEE_LEFT: CNA_AvatarBone = 6;
pub const CNA_AVATAR_BONE_KNEE_RIGHT: CNA_AvatarBone = 8;
pub const CNA_AVATAR_BONE_ANKLE_LEFT: CNA_AvatarBone = 11;
pub const CNA_AVATAR_BONE_COLLAR_LEFT: CNA_AvatarBone = 12;
pub const CNA_AVATAR_BONE_NECK: CNA_AvatarBone = 14;
pub const CNA_AVATAR_BONE_ANKLE_RIGHT: CNA_AvatarBone = 15;
pub const CNA_AVATAR_BONE_COLLAR_RIGHT: CNA_AvatarBone = 16;
pub const CNA_AVATAR_BONE_HEAD: CNA_AvatarBone = 19;
pub const CNA_AVATAR_BONE_SHOULDER_LEFT: CNA_AvatarBone = 20;
pub const CNA_AVATAR_BONE_TOE_LEFT: CNA_AvatarBone = 21;
pub const CNA_AVATAR_BONE_SHOULDER_RIGHT: CNA_AvatarBone = 22;
pub const CNA_AVATAR_BONE_TOE_RIGHT: CNA_AvatarBone = 23;
pub const CNA_AVATAR_BONE_ELBOW_LEFT: CNA_AvatarBone = 25;
pub const CNA_AVATAR_BONE_ELBOW_RIGHT: CNA_AvatarBone = 28;
pub const CNA_AVATAR_BONE_WRIST_LEFT: CNA_AvatarBone = 33;
pub const CNA_AVATAR_BONE_WRIST_RIGHT: CNA_AvatarBone = 36;
pub const CNA_AVATAR_BONE_FINGER_INDEX_LEFT: CNA_AvatarBone = 37;
pub const CNA_AVATAR_BONE_FINGER_MIDDLE_LEFT: CNA_AvatarBone = 38;
pub const CNA_AVATAR_BONE_FINGER_RING_LEFT: CNA_AvatarBone = 39;
pub const CNA_AVATAR_BONE_FINGER_SMALL_LEFT: CNA_AvatarBone = 40;
pub const CNA_AVATAR_BONE_PROP_LEFT: CNA_AvatarBone = 41;
pub const CNA_AVATAR_BONE_SPECIAL_LEFT: CNA_AvatarBone = 42;
pub const CNA_AVATAR_BONE_FINGER_THUMB_LEFT: CNA_AvatarBone = 43;
pub const CNA_AVATAR_BONE_FINGER_INDEX_RIGHT: CNA_AvatarBone = 44;
pub const CNA_AVATAR_BONE_FINGER_MIDDLE_RIGHT: CNA_AvatarBone = 45;
pub const CNA_AVATAR_BONE_FINGER_RING_RIGHT: CNA_AvatarBone = 46;
pub const CNA_AVATAR_BONE_FINGER_SMALL_RIGHT: CNA_AvatarBone = 47;
pub const CNA_AVATAR_BONE_PROP_RIGHT: CNA_AvatarBone = 48;
pub const CNA_AVATAR_BONE_SPECIAL_RIGHT: CNA_AvatarBone = 49;
pub const CNA_AVATAR_BONE_FINGER_THUMB_RIGHT: CNA_AvatarBone = 50;
pub const CNA_AVATAR_BONE_FINGER_INDEX_2_LEFT: CNA_AvatarBone = 51;
pub const CNA_AVATAR_BONE_FINGER_MIDDLE_2_LEFT: CNA_AvatarBone = 52;
pub const CNA_AVATAR_BONE_FINGER_RING_2_LEFT: CNA_AvatarBone = 53;
pub const CNA_AVATAR_BONE_FINGER_SMALL_2_LEFT: CNA_AvatarBone = 54;
pub const CNA_AVATAR_BONE_FINGER_THUMB_2_LEFT: CNA_AvatarBone = 55;
pub const CNA_AVATAR_BONE_FINGER_INDEX_2_RIGHT: CNA_AvatarBone = 56;
pub const CNA_AVATAR_BONE_FINGER_MIDDLE_2_RIGHT: CNA_AvatarBone = 57;
pub const CNA_AVATAR_BONE_FINGER_RING_2_RIGHT: CNA_AvatarBone = 58;
pub const CNA_AVATAR_BONE_FINGER_SMALL_2_RIGHT: CNA_AvatarBone = 59;
pub const CNA_AVATAR_BONE_FINGER_THUMB_2_RIGHT: CNA_AvatarBone = 60;
pub const CNA_AVATAR_BONE_FINGER_INDEX_3_LEFT: CNA_AvatarBone = 61;
pub const CNA_AVATAR_BONE_FINGER_MIDDLE_3_LEFT: CNA_AvatarBone = 62;
pub const CNA_AVATAR_BONE_FINGER_RING_3_LEFT: CNA_AvatarBone = 63;
pub const CNA_AVATAR_BONE_FINGER_SMALL_3_LEFT: CNA_AvatarBone = 64;
pub const CNA_AVATAR_BONE_FINGER_THUMB_3_LEFT: CNA_AvatarBone = 65;
pub const CNA_AVATAR_BONE_FINGER_INDEX_3_RIGHT: CNA_AvatarBone = 66;
pub const CNA_AVATAR_BONE_FINGER_MIDDLE_3_RIGHT: CNA_AvatarBone = 67;
pub const CNA_AVATAR_BONE_FINGER_RING_3_RIGHT: CNA_AvatarBone = 68;
pub const CNA_AVATAR_BONE_FINGER_SMALL_3_RIGHT: CNA_AvatarBone = 69;
pub const CNA_AVATAR_BONE_FINGER_THUMB_3_RIGHT: CNA_AvatarBone = 70;
pub const CNA_PROPERTY_VALUE_KIND_UNKNOWN: CNA_PropertyValueKind = 0;
pub const CNA_PROPERTY_VALUE_KIND_DATE_TIME: CNA_PropertyValueKind = 1;
pub const CNA_PROPERTY_VALUE_KIND_DOUBLE: CNA_PropertyValueKind = 2;
pub const CNA_PROPERTY_VALUE_KIND_INT32: CNA_PropertyValueKind = 3;
pub const CNA_PROPERTY_VALUE_KIND_INT64: CNA_PropertyValueKind = 4;
pub const CNA_PROPERTY_VALUE_KIND_OUTCOME: CNA_PropertyValueKind = 5;
pub const CNA_PROPERTY_VALUE_KIND_SINGLE: CNA_PropertyValueKind = 6;
pub const CNA_PROPERTY_VALUE_KIND_STREAM: CNA_PropertyValueKind = 7;
pub const CNA_PROPERTY_VALUE_KIND_STRING: CNA_PropertyValueKind = 8;
pub const CNA_PROPERTY_VALUE_KIND_TIME_SPAN: CNA_PropertyValueKind = 9;
pub const CNA_LEADERBOARD_IDENTITY_KEY_CAPACITY: u32 = 64;
pub const CNA_AVATAR_RENDERER_BONE_COUNT: i32 = 71;
pub const CNA_NETWORK_SESSION_END_REASON_CLIENT_SIGNED_OUT: CNA_NetworkSessionEndReason = 0;
pub const CNA_NETWORK_SESSION_END_REASON_HOST_ENDED_SESSION: CNA_NetworkSessionEndReason = 1;
pub const CNA_NETWORK_SESSION_END_REASON_REMOVED_BY_HOST: CNA_NetworkSessionEndReason = 2;
pub const CNA_NETWORK_SESSION_END_REASON_DISCONNECTED: CNA_NetworkSessionEndReason = 3;
pub const CNA_NETWORK_SESSION_JOIN_ERROR_SESSION_NOT_FOUND: CNA_NetworkSessionJoinError = 0;
pub const CNA_NETWORK_SESSION_JOIN_ERROR_SESSION_NOT_JOINABLE: CNA_NetworkSessionJoinError = 1;
pub const CNA_NETWORK_SESSION_JOIN_ERROR_SESSION_FULL: CNA_NetworkSessionJoinError = 2;
pub const CNA_NETWORK_SESSION_STATE_LOBBY: CNA_NetworkSessionState = 0;
pub const CNA_NETWORK_SESSION_STATE_PLAYING: CNA_NetworkSessionState = 1;
pub const CNA_NETWORK_SESSION_STATE_ENDED: CNA_NetworkSessionState = 2;
pub const CNA_NETWORK_SESSION_TYPE_LOCAL: CNA_NetworkSessionType = 0;
pub const CNA_NETWORK_SESSION_TYPE_SYSTEM_LINK: CNA_NetworkSessionType = 1;
pub const CNA_NETWORK_SESSION_TYPE_PLAYER_MATCH: CNA_NetworkSessionType = 2;
pub const CNA_NETWORK_SESSION_TYPE_RANKED: CNA_NetworkSessionType = 3;
pub const CNA_NETWORK_SESSION_TYPE_LOCAL_WITH_LEADERBOARDS: CNA_NetworkSessionType = 4;
pub const CNA_SEND_DATA_OPTIONS_NONE: CNA_SendDataOptions = 0;
pub const CNA_SEND_DATA_OPTIONS_RELIABLE: CNA_SendDataOptions = 1;
pub const CNA_SEND_DATA_OPTIONS_IN_ORDER: CNA_SendDataOptions = 2;
pub const CNA_SEND_DATA_OPTIONS_RELIABLE_IN_ORDER: CNA_SendDataOptions = 3;
pub const CNA_SEND_DATA_OPTIONS_CHAT: CNA_SendDataOptions = 4;
pub const CNA_NETWORK_EVENT_TYPE_PACKET_SEND: CNA_NetworkEventType = 0;
pub const CNA_NETWORK_EVENT_TYPE_GAMER_JOIN: CNA_NetworkEventType = 1;
pub const CNA_NETWORK_EVENT_TYPE_GAMER_LEAVE: CNA_NetworkEventType = 2;
pub const CNA_NETWORK_EVENT_TYPE_HOST_CHANGE: CNA_NetworkEventType = 3;
pub const CNA_NETWORK_EVENT_TYPE_STATE_CHANGE: CNA_NetworkEventType = 4;
pub const CNA_NETWORK_SESSION_ROSTER_ALL: u32 = 0;
pub const CNA_NETWORK_SESSION_ROSTER_LOCAL: u32 = 1;
pub const CNA_NETWORK_SESSION_ROSTER_REMOTE: u32 = 2;
pub const CNA_NETWORK_SESSION_ROSTER_PREVIOUS: u32 = 3;
pub const CNA_NETWORK_SESSION_MAX_SUPPORTED_GAMERS: i32 = 31;
pub const CNA_NETWORK_SESSION_MAX_PREVIOUS_GAMERS: i32 = 100;

/* ---- GamerServices, Avatar and Net routes ---- */

pub type cna_achievement_collection_add_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle, CNA_AchievementHandle,
) -> CNA_Result;
pub type cna_achievement_collection_clear_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle,
) -> CNA_Result;
pub type cna_achievement_collection_contains_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle, CNA_AchievementHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_achievement_collection_copy_to_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle, *mut CNA_AchievementHandle, u64, i32, *mut u64,
) -> CNA_Result;
pub type cna_achievement_collection_create_ext_fn = unsafe extern "C" fn(
    *const CNA_AchievementHandle, u64, *mut CNA_AchievementCollectionHandle,
) -> CNA_Result;
pub type cna_achievement_collection_destroy_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle,
) -> CNA_Result;
pub type cna_achievement_collection_get_at_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle, i32, *mut CNA_AchievementHandle,
) -> CNA_Result;
pub type cna_achievement_collection_get_by_key_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle, CNA_StringView, *mut CNA_AchievementHandle,
) -> CNA_Result;
pub type cna_achievement_collection_get_count_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle, *mut i32,
) -> CNA_Result;
pub type cna_achievement_collection_get_is_disposed_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_achievement_collection_get_is_read_only_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_achievement_collection_index_of_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle, CNA_AchievementHandle, *mut i32,
) -> CNA_Result;
pub type cna_achievement_collection_insert_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle, i32, CNA_AchievementHandle,
) -> CNA_Result;
pub type cna_achievement_collection_remove_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle, CNA_AchievementHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_achievement_collection_remove_at_fn = unsafe extern "C" fn(
    CNA_AchievementCollectionHandle, i32,
) -> CNA_Result;
pub type cna_achievement_copy_description_fn = unsafe extern "C" fn(
    CNA_AchievementHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_achievement_copy_how_to_earn_fn = unsafe extern "C" fn(
    CNA_AchievementHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_achievement_copy_key_fn = unsafe extern "C" fn(
    CNA_AchievementHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_achievement_copy_name_fn = unsafe extern "C" fn(
    CNA_AchievementHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_achievement_create_ext_fn = unsafe extern "C" fn(
    CNA_StringView, CNA_StringView, CNA_StringView, CNA_Bool, CNA_Bool, i64, *mut CNA_AchievementHandle,
) -> CNA_Result;
pub type cna_achievement_destroy_fn = unsafe extern "C" fn(CNA_AchievementHandle) -> CNA_Result;
pub type cna_achievement_equals_fn = unsafe extern "C" fn(
    CNA_AchievementHandle, CNA_AchievementHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_achievement_get_description_size_fn = unsafe extern "C" fn(
    CNA_AchievementHandle, *mut u64,
) -> CNA_Result;
pub type cna_achievement_get_how_to_earn_size_fn = unsafe extern "C" fn(
    CNA_AchievementHandle, *mut u64,
) -> CNA_Result;
pub type cna_achievement_get_info_fn = unsafe extern "C" fn(
    CNA_AchievementHandle, *mut CNA_AchievementInfo,
) -> CNA_Result;
pub type cna_achievement_get_key_size_fn = unsafe extern "C" fn(
    CNA_AchievementHandle, *mut u64,
) -> CNA_Result;
pub type cna_achievement_get_name_size_fn = unsafe extern "C" fn(
    CNA_AchievementHandle, *mut u64,
) -> CNA_Result;
pub type cna_achievement_get_picture_size_fn = unsafe extern "C" fn(
    CNA_AchievementHandle, *mut u64,
) -> CNA_Result;
pub type cna_available_network_session_collection_copy_session_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionCollectionHandle, i32, *mut CNA_AvailableNetworkSessionHandle,
) -> CNA_Result;
pub type cna_available_network_session_collection_create_ext_fn = unsafe extern "C" fn(
    *const CNA_AvailableNetworkSessionHandle, u64, *mut CNA_AvailableNetworkSessionCollectionHandle,
) -> CNA_Result;
pub type cna_available_network_session_collection_destroy_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionCollectionHandle,
) -> CNA_Result;
pub type cna_available_network_session_collection_dispose_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionCollectionHandle,
) -> CNA_Result;
pub type cna_available_network_session_collection_get_count_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionCollectionHandle, *mut i32,
) -> CNA_Result;
pub type cna_available_network_session_collection_get_is_disposed_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionCollectionHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_available_network_session_copy_connect_address_ext_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_available_network_session_copy_host_gamertag_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_available_network_session_copy_session_properties_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, *mut CNA_NetworkSessionPropertiesHandle,
) -> CNA_Result;
pub type cna_available_network_session_create_ext_fn = unsafe extern "C" fn(
    *const CNA_AvailableNetworkSessionCreateInfo, *const CNA_QualityOfService, *mut CNA_AvailableNetworkSessionHandle,
) -> CNA_Result;
pub type cna_available_network_session_destroy_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle,
) -> CNA_Result;
pub type cna_available_network_session_equals_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, CNA_AvailableNetworkSessionHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_available_network_session_get_connect_address_size_ext_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, *mut u64,
) -> CNA_Result;
pub type cna_available_network_session_get_connect_port_ext_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, *mut u16,
) -> CNA_Result;
pub type cna_available_network_session_get_current_gamer_count_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, *mut i32,
) -> CNA_Result;
pub type cna_available_network_session_get_host_gamertag_size_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, *mut u64,
) -> CNA_Result;
pub type cna_available_network_session_get_open_private_gamer_slots_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, *mut i32,
) -> CNA_Result;
pub type cna_available_network_session_get_open_public_gamer_slots_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, *mut i32,
) -> CNA_Result;
pub type cna_available_network_session_get_quality_of_service_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, *mut CNA_QualityOfService,
) -> CNA_Result;
pub type cna_available_network_session_get_session_type_ext_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, *mut CNA_NetworkSessionType,
) -> CNA_Result;
pub type cna_available_network_session_not_equals_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, CNA_AvailableNetworkSessionHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_avatar_animation_copy_real_clip_name_ext_fn = unsafe extern "C" fn(
    CNA_AvatarAnimationHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_avatar_animation_create_fn = unsafe extern "C" fn(
    CNA_AvatarAnimationPreset, *mut CNA_AvatarAnimationHandle,
) -> CNA_Result;
pub type cna_avatar_animation_destroy_fn = unsafe extern "C" fn(
    CNA_AvatarAnimationHandle,
) -> CNA_Result;
pub type cna_avatar_animation_get_bone_transform_at_fn = unsafe extern "C" fn(
    CNA_AvatarAnimationHandle, i32, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_avatar_animation_get_expression_fn = unsafe extern "C" fn(
    CNA_AvatarAnimationHandle, *mut CNA_AvatarExpression,
) -> CNA_Result;
pub type cna_avatar_animation_get_info_fn = unsafe extern "C" fn(
    CNA_AvatarAnimationHandle, *mut CNA_AvatarAnimationInfo,
) -> CNA_Result;
pub type cna_avatar_animation_get_real_clip_name_size_ext_fn = unsafe extern "C" fn(
    CNA_AvatarAnimationHandle, *mut u64,
) -> CNA_Result;
pub type cna_avatar_animation_preset_copy_clip_name_ext_fn = unsafe extern "C" fn(
    CNA_AvatarAnimationPreset, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_avatar_animation_preset_get_clip_name_size_ext_fn = unsafe extern "C" fn(
    CNA_AvatarAnimationPreset, *mut u64,
) -> CNA_Result;
pub type cna_avatar_animation_set_current_position_fn = unsafe extern "C" fn(
    CNA_AvatarAnimationHandle, i64,
) -> CNA_Result;
pub type cna_avatar_animation_set_real_clip_name_ext_fn = unsafe extern "C" fn(
    CNA_AvatarAnimationHandle, CNA_StringView,
) -> CNA_Result;
pub type cna_avatar_animation_update_fn = unsafe extern "C" fn(
    CNA_AvatarAnimationHandle, i64, CNA_Bool,
) -> CNA_Result;
pub type cna_avatar_appearance_init_ext_fn = unsafe extern "C" fn(
    *mut CNA_AvatarAppearanceEXT,
) -> CNA_Result;
pub type cna_avatar_body_type_copy_content_name_ext_fn = unsafe extern "C" fn(
    CNA_AvatarBodyType, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_avatar_body_type_get_content_name_size_ext_fn = unsafe extern "C" fn(
    CNA_AvatarBodyType, *mut u64,
) -> CNA_Result;
pub type cna_avatar_description_copy_description_fn = unsafe extern "C" fn(
    CNA_AvatarDescriptionHandle, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_avatar_description_create_fn = unsafe extern "C" fn(
    *const u8, u64, *mut CNA_AvatarDescriptionHandle,
) -> CNA_Result;
pub type cna_avatar_description_create_random_fn = unsafe extern "C" fn(
    *mut CNA_AvatarDescriptionHandle,
) -> CNA_Result;
pub type cna_avatar_description_create_random_for_body_type_fn = unsafe extern "C" fn(
    CNA_AvatarBodyType, *mut CNA_AvatarDescriptionHandle,
) -> CNA_Result;
pub type cna_avatar_description_destroy_fn = unsafe extern "C" fn(
    CNA_AvatarDescriptionHandle,
) -> CNA_Result;
pub type cna_avatar_description_get_from_gamer_fn = unsafe extern "C" fn(
    CNA_GamerHandle, CNA_GamerAsyncCallback, *mut c_void, *mut CNA_AvatarDescriptionHandle,
) -> CNA_Result;
pub type cna_avatar_description_get_info_fn = unsafe extern "C" fn(
    CNA_AvatarDescriptionHandle, *mut CNA_AvatarDescriptionInfo,
) -> CNA_Result;
pub type cna_avatar_description_subscribe_changed_ext_fn = unsafe extern "C" fn(
    CNA_GamerAsyncCallback, *mut c_void, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_avatar_expression_init_fn = unsafe extern "C" fn(
    *mut CNA_AvatarExpression,
) -> CNA_Result;
pub type cna_avatar_renderer_create_fn = unsafe extern "C" fn(
    CNA_AvatarDescriptionHandle, CNA_Bool, *mut CNA_AvatarRendererHandle,
) -> CNA_Result;
pub type cna_avatar_renderer_destroy_fn = unsafe extern "C" fn(
    CNA_AvatarRendererHandle,
) -> CNA_Result;
pub type cna_avatar_renderer_draw_animation_fn = unsafe extern "C" fn(
    CNA_AvatarRendererHandle, CNA_AvatarAnimationHandle,
) -> CNA_Result;
pub type cna_avatar_renderer_draw_bones_fn = unsafe extern "C" fn(
    CNA_AvatarRendererHandle, *const CNA_Matrix, u64, *const CNA_AvatarExpression,
) -> CNA_Result;
pub type cna_avatar_renderer_draw_real_ext_fn = unsafe extern "C" fn(
    CNA_AvatarRendererHandle, CNA_StringView, i64, CNA_Bool,
) -> CNA_Result;
pub type cna_avatar_renderer_enable_real_rendering_ext_fn = unsafe extern "C" fn(
    CNA_AvatarRendererHandle, CNA_Handle, CNA_Handle,
) -> CNA_Result;
pub type cna_avatar_renderer_get_bind_pose_at_fn = unsafe extern "C" fn(
    CNA_AvatarRendererHandle, i32, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_avatar_renderer_get_info_fn = unsafe extern "C" fn(
    CNA_AvatarRendererHandle, *mut CNA_AvatarRendererInfo,
) -> CNA_Result;
pub type cna_avatar_renderer_get_lighting_fn = unsafe extern "C" fn(
    CNA_AvatarRendererHandle, *mut CNA_Vector3, *mut CNA_Vector3, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_avatar_renderer_get_parent_bone_at_fn = unsafe extern "C" fn(
    CNA_AvatarRendererHandle, i32, *mut i32,
) -> CNA_Result;
pub type cna_avatar_renderer_get_transforms_fn = unsafe extern "C" fn(
    CNA_AvatarRendererHandle, *mut CNA_Matrix, *mut CNA_Matrix, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_avatar_renderer_set_appearance_ext_fn = unsafe extern "C" fn(
    CNA_AvatarRendererHandle, *const CNA_AvatarAppearanceEXT,
) -> CNA_Result;
pub type cna_avatar_renderer_set_lighting_fn = unsafe extern "C" fn(
    CNA_AvatarRendererHandle, *const CNA_Vector3, *const CNA_Vector3, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_avatar_renderer_set_transforms_fn = unsafe extern "C" fn(
    CNA_AvatarRendererHandle, *const CNA_Matrix, *const CNA_Matrix, *const CNA_Matrix,
) -> CNA_Result;
pub type cna_friend_collection_create_ext_fn = unsafe extern "C" fn(
    *const CNA_GamerHandle, u64, *mut CNA_GamerCollectionHandle,
) -> CNA_Result;
pub type cna_friend_collection_get_is_disposed_fn = unsafe extern "C" fn(
    CNA_GamerCollectionHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_friend_gamer_copy_presence_fn = unsafe extern "C" fn(
    CNA_GamerHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_friend_gamer_create_ext_fn = unsafe extern "C" fn(
    CNA_StringView, CNA_StringView, CNA_Bool, CNA_Bool, CNA_Bool, CNA_Bool, CNA_Bool, CNA_Bool, *mut CNA_GamerHandle,
) -> CNA_Result;
pub type cna_friend_gamer_get_info_fn = unsafe extern "C" fn(
    CNA_GamerHandle, *mut CNA_FriendGamerInfo,
) -> CNA_Result;
pub type cna_friend_gamer_get_presence_size_fn = unsafe extern "C" fn(
    CNA_GamerHandle, *mut u64,
) -> CNA_Result;
pub type cna_game_defaults_init_fn = unsafe extern "C" fn(*mut CNA_GameDefaults) -> CNA_Result;
pub type cna_game_ended_event_info_init_fn = unsafe extern "C" fn(
    *mut CNA_GameEndedEventInfo,
) -> CNA_Result;
pub type cna_gamer_begin_get_from_gamertag_fn = unsafe extern "C" fn(
    CNA_StringView, CNA_GamerAsyncCallback, *mut c_void, *mut CNA_GamerHandle,
) -> CNA_Result;
pub type cna_gamer_begin_get_partner_token_fn = unsafe extern "C" fn(
    CNA_StringView, CNA_GamerAsyncCallback, *mut c_void, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_gamer_begin_get_profile_fn = unsafe extern "C" fn(
    CNA_GamerHandle, CNA_GamerAsyncCallback, *mut c_void, *mut CNA_GamerProfileHandle,
) -> CNA_Result;
pub type cna_gamer_collection_add_fn = unsafe extern "C" fn(
    CNA_GamerCollectionHandle, CNA_GamerHandle,
) -> CNA_Result;
pub type cna_gamer_collection_clear_fn = unsafe extern "C" fn(
    CNA_GamerCollectionHandle,
) -> CNA_Result;
pub type cna_gamer_collection_contains_fn = unsafe extern "C" fn(
    CNA_GamerCollectionHandle, CNA_GamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gamer_collection_copy_to_fn = unsafe extern "C" fn(
    CNA_GamerCollectionHandle, *mut CNA_GamerHandle, u64, i32, *mut u64,
) -> CNA_Result;
pub type cna_gamer_collection_create_enumerator_fn = unsafe extern "C" fn(
    CNA_GamerCollectionHandle, *mut CNA_GamerEnumeratorHandle,
) -> CNA_Result;
pub type cna_gamer_collection_destroy_fn = unsafe extern "C" fn(
    CNA_GamerCollectionHandle,
) -> CNA_Result;
pub type cna_gamer_collection_get_at_fn = unsafe extern "C" fn(
    CNA_GamerCollectionHandle, i32, *mut CNA_GamerHandle,
) -> CNA_Result;
pub type cna_gamer_collection_get_count_fn = unsafe extern "C" fn(
    CNA_GamerCollectionHandle, *mut i32,
) -> CNA_Result;
pub type cna_gamer_collection_index_of_fn = unsafe extern "C" fn(
    CNA_GamerCollectionHandle, CNA_GamerHandle, *mut i32,
) -> CNA_Result;
pub type cna_gamer_collection_remove_fn = unsafe extern "C" fn(
    CNA_GamerCollectionHandle, CNA_GamerHandle,
) -> CNA_Result;
pub type cna_gamer_copy_display_name_fn = unsafe extern "C" fn(
    CNA_GamerHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_gamer_copy_gamertag_fn = unsafe extern "C" fn(
    CNA_GamerHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_gamer_copy_partner_token_fn = unsafe extern "C" fn(
    CNA_StringView, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_gamer_copy_text_fn = unsafe extern "C" fn(
    CNA_GamerHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_gamer_destroy_fn = unsafe extern "C" fn(CNA_GamerHandle) -> CNA_Result;
pub type cna_gamer_enumerator_destroy_fn = unsafe extern "C" fn(
    CNA_GamerEnumeratorHandle,
) -> CNA_Result;
pub type cna_gamer_enumerator_get_current_fn = unsafe extern "C" fn(
    CNA_GamerEnumeratorHandle, *mut CNA_GamerHandle,
) -> CNA_Result;
pub type cna_gamer_enumerator_move_next_fn = unsafe extern "C" fn(
    CNA_GamerEnumeratorHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gamer_enumerator_reset_fn = unsafe extern "C" fn(
    CNA_GamerEnumeratorHandle,
) -> CNA_Result;
pub type cna_gamer_get_display_name_size_fn = unsafe extern "C" fn(
    CNA_GamerHandle, *mut u64,
) -> CNA_Result;
pub type cna_gamer_get_from_gamertag_fn = unsafe extern "C" fn(
    CNA_StringView, *mut CNA_GamerHandle,
) -> CNA_Result;
pub type cna_gamer_get_gamertag_size_fn = unsafe extern "C" fn(
    CNA_GamerHandle, *mut u64,
) -> CNA_Result;
pub type cna_gamer_get_is_disposed_fn = unsafe extern "C" fn(
    CNA_GamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gamer_get_partner_token_size_fn = unsafe extern "C" fn(
    CNA_StringView, *mut u64,
) -> CNA_Result;
pub type cna_gamer_get_profile_fn = unsafe extern "C" fn(
    CNA_GamerHandle, *mut CNA_GamerProfileHandle,
) -> CNA_Result;
pub type cna_gamer_get_signed_in_gamer_at_fn = unsafe extern "C" fn(
    i32, *mut CNA_SignedInGamerHandle,
) -> CNA_Result;
pub type cna_gamer_get_signed_in_gamer_at_player_index_fn = unsafe extern "C" fn(
    CNA_PlayerIndex, *mut CNA_Bool, *mut CNA_SignedInGamerHandle,
) -> CNA_Result;
pub type cna_gamer_get_signed_in_gamer_count_fn = unsafe extern "C" fn(*mut i32) -> CNA_Result;
pub type cna_gamer_get_tag_fn = unsafe extern "C" fn(CNA_GamerHandle, *mut u64) -> CNA_Result;
pub type cna_gamer_get_text_size_fn = unsafe extern "C" fn(CNA_GamerHandle, *mut u64) -> CNA_Result;
pub type cna_gamer_joined_event_info_init_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_GamerJoinedEventInfo,
) -> CNA_Result;
pub type cna_gamer_left_event_info_init_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_GamerLeftEventInfo,
) -> CNA_Result;
pub type cna_gamer_presence_init_fn = unsafe extern "C" fn(*mut CNA_GamerPresence) -> CNA_Result;
pub type cna_gamer_profile_copy_motto_fn = unsafe extern "C" fn(
    CNA_GamerProfileHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_gamer_profile_copy_region_name_fn = unsafe extern "C" fn(
    CNA_GamerProfileHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_gamer_profile_destroy_fn = unsafe extern "C" fn(CNA_GamerProfileHandle) -> CNA_Result;
pub type cna_gamer_profile_get_info_fn = unsafe extern "C" fn(
    CNA_GamerProfileHandle, *mut CNA_GamerProfileInfo,
) -> CNA_Result;
pub type cna_gamer_profile_get_motto_size_fn = unsafe extern "C" fn(
    CNA_GamerProfileHandle, *mut u64,
) -> CNA_Result;
pub type cna_gamer_profile_get_picture_size_fn = unsafe extern "C" fn(
    CNA_GamerProfileHandle, *mut CNA_Bool, *mut u64,
) -> CNA_Result;
pub type cna_gamer_profile_get_region_name_size_fn = unsafe extern "C" fn(
    CNA_GamerProfileHandle, *mut u64,
) -> CNA_Result;
pub type cna_gamer_services_component_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_gamer_services_dispatcher_get_freed_gamer_count_ext_fn = unsafe extern "C" fn(
    *mut u64,
) -> CNA_Result;
pub type cna_gamer_services_dispatcher_get_is_initialized_fn = unsafe extern "C" fn(
    *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gamer_services_dispatcher_get_window_handle_fn = unsafe extern "C" fn(
    *mut u64,
) -> CNA_Result;
pub type cna_gamer_services_dispatcher_initialize_fn = unsafe extern "C" fn(
    CNA_Handle,
) -> CNA_Result;
pub type cna_gamer_services_dispatcher_set_window_handle_fn = unsafe extern "C" fn(
    u64,
) -> CNA_Result;
pub type cna_gamer_services_dispatcher_subscribe_installing_title_update_ext_fn = unsafe extern "C" fn(
    CNA_GamerAsyncCallback, *mut c_void, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_gamer_services_dispatcher_update_fn = unsafe extern "C" fn() -> CNA_Result;
pub type cna_gamer_services_dispatcher_update_async_fn = unsafe extern "C" fn(
    *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gamer_set_display_name_fn = unsafe extern "C" fn(
    CNA_GamerHandle, CNA_StringView,
) -> CNA_Result;
pub type cna_gamer_set_signed_in_gamers_ext_fn = unsafe extern "C" fn(
    *const CNA_SignedInGamerHandle, u64,
) -> CNA_Result;
pub type cna_gamer_set_tag_fn = unsafe extern "C" fn(CNA_GamerHandle, u64) -> CNA_Result;
pub type cna_gamer_signed_in_contains_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gamer_signed_in_index_of_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *mut i32,
) -> CNA_Result;
pub type cna_gamer_unsubscribe_ext_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_game_started_event_info_init_fn = unsafe extern "C" fn(
    *mut CNA_GameStartedEventInfo,
) -> CNA_Result;
pub type cna_guide_begin_show_keyboard_input_fn = unsafe extern "C" fn(
    CNA_PlayerIndex, CNA_StringView, CNA_StringView, CNA_StringView, CNA_Bool, CNA_GamerAsyncCallback, *mut c_void,
) -> CNA_Result;
pub type cna_guide_begin_show_message_box_fn = unsafe extern "C" fn(
    CNA_PlayerIndex, CNA_StringView, CNA_StringView, *const CNA_StringView, u64, i32, CNA_MessageBoxIcon, CNA_GamerAsyncCallback, *mut c_void,
) -> CNA_Result;
pub type cna_guide_copy_pending_keyboard_input_description_ext_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_guide_copy_pending_keyboard_input_display_text_ext_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_guide_copy_pending_keyboard_input_title_ext_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_guide_delay_notifications_fn = unsafe extern "C" fn(i64) -> CNA_Result;
pub type cna_guide_end_show_keyboard_input_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_guide_end_show_keyboard_input_size_fn = unsafe extern "C" fn(*mut u64) -> CNA_Result;
pub type cna_guide_end_show_message_box_fn = unsafe extern "C" fn(
    *mut CNA_Bool, *mut i32,
) -> CNA_Result;
pub type cna_guide_get_has_pending_keyboard_input_ext_fn = unsafe extern "C" fn(
    *mut CNA_Bool,
) -> CNA_Result;
pub type cna_guide_get_has_pending_message_box_ext_fn = unsafe extern "C" fn(
    *mut CNA_Bool,
) -> CNA_Result;
pub type cna_guide_get_is_screen_saver_enabled_fn = unsafe extern "C" fn(
    *mut CNA_Bool,
) -> CNA_Result;
pub type cna_guide_get_is_trial_mode_fn = unsafe extern "C" fn(*mut CNA_Bool) -> CNA_Result;
pub type cna_guide_get_is_visible_fn = unsafe extern "C" fn(*mut CNA_Bool) -> CNA_Result;
pub type cna_guide_get_notification_position_fn = unsafe extern "C" fn(
    *mut CNA_NotificationPosition,
) -> CNA_Result;
pub type cna_guide_get_pending_keyboard_input_description_size_ext_fn = unsafe extern "C" fn(
    *mut u64,
) -> CNA_Result;
pub type cna_guide_get_pending_keyboard_input_display_text_size_ext_fn = unsafe extern "C" fn(
    *mut u64,
) -> CNA_Result;
pub type cna_guide_get_pending_keyboard_input_title_size_ext_fn = unsafe extern "C" fn(
    *mut u64,
) -> CNA_Result;
pub type cna_guide_get_pending_message_box_focus_button_ext_fn = unsafe extern "C" fn(
    *mut i32,
) -> CNA_Result;
pub type cna_guide_get_simulate_trial_mode_fn = unsafe extern "C" fn(*mut CNA_Bool) -> CNA_Result;
pub type cna_guide_render_pending_keyboard_input_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_Handle, CNA_Handle, CNA_Handle,
) -> CNA_Result;
pub type cna_guide_render_pending_message_box_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_Handle, CNA_Handle, CNA_Handle,
) -> CNA_Result;
pub type cna_guide_reset_pending_keyboard_input_ext_fn = unsafe extern "C" fn() -> CNA_Result;
pub type cna_guide_reset_pending_message_box_ext_fn = unsafe extern "C" fn() -> CNA_Result;
pub type cna_guide_set_is_screen_saver_enabled_fn = unsafe extern "C" fn(CNA_Bool) -> CNA_Result;
pub type cna_guide_set_is_trial_mode_fn = unsafe extern "C" fn(CNA_Bool) -> CNA_Result;
pub type cna_guide_set_is_visible_fn = unsafe extern "C" fn(CNA_Bool) -> CNA_Result;
pub type cna_guide_set_notification_position_fn = unsafe extern "C" fn(
    CNA_NotificationPosition,
) -> CNA_Result;
pub type cna_guide_set_simulate_trial_mode_fn = unsafe extern "C" fn(CNA_Bool) -> CNA_Result;
pub type cna_guide_show_achievements_ext_fn = unsafe extern "C" fn(CNA_PlayerIndex) -> CNA_Result;
pub type cna_guide_show_compose_message_fn = unsafe extern "C" fn(
    CNA_PlayerIndex, CNA_StringView, *const CNA_GamerHandle, u64,
) -> CNA_Result;
pub type cna_guide_show_friend_request_fn = unsafe extern "C" fn(
    CNA_PlayerIndex, CNA_GamerHandle,
) -> CNA_Result;
pub type cna_guide_show_friends_fn = unsafe extern "C" fn(CNA_PlayerIndex) -> CNA_Result;
pub type cna_guide_show_game_invite_fn = unsafe extern "C" fn(
    CNA_PlayerIndex, *const CNA_GamerHandle, u64,
) -> CNA_Result;
pub type cna_guide_show_game_invite_for_session_fn = unsafe extern "C" fn(
    CNA_StringView,
) -> CNA_Result;
pub type cna_guide_show_gamer_card_fn = unsafe extern "C" fn(
    CNA_PlayerIndex, CNA_GamerHandle,
) -> CNA_Result;
pub type cna_guide_show_marketplace_fn = unsafe extern "C" fn(CNA_PlayerIndex) -> CNA_Result;
pub type cna_guide_show_messages_fn = unsafe extern "C" fn(CNA_PlayerIndex) -> CNA_Result;
pub type cna_guide_show_party_fn = unsafe extern "C" fn(CNA_PlayerIndex) -> CNA_Result;
pub type cna_guide_show_party_sessions_fn = unsafe extern "C" fn(CNA_PlayerIndex) -> CNA_Result;
pub type cna_guide_show_player_review_fn = unsafe extern "C" fn(
    CNA_PlayerIndex, CNA_GamerHandle,
) -> CNA_Result;
pub type cna_guide_show_players_fn = unsafe extern "C" fn(CNA_PlayerIndex) -> CNA_Result;
pub type cna_guide_show_sign_in_fn = unsafe extern "C" fn(i32, CNA_Bool) -> CNA_Result;
pub type cna_guide_simulate_keyboard_input_cancel_ext_fn = unsafe extern "C" fn() -> CNA_Result;
pub type cna_guide_simulate_message_box_click_ext_fn = unsafe extern "C" fn(i32) -> CNA_Result;
pub type cna_guide_was_keyboard_input_canceled_ext_fn = unsafe extern "C" fn(
    *mut CNA_Bool,
) -> CNA_Result;
pub type cna_host_changed_event_info_init_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, CNA_NetworkGamerHandle, *mut CNA_HostChangedEventInfo,
) -> CNA_Result;
pub type cna_invite_accepted_event_info_init_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, CNA_Bool, *mut CNA_InviteAcceptedEventInfo,
) -> CNA_Result;
pub type cna_leaderboard_entry_create_ext_fn = unsafe extern "C" fn(
    CNA_GamerHandle, i64, i32, *mut CNA_LeaderboardEntryHandle,
) -> CNA_Result;
pub type cna_leaderboard_entry_destroy_fn = unsafe extern "C" fn(
    CNA_LeaderboardEntryHandle,
) -> CNA_Result;
pub type cna_leaderboard_entry_equals_fn = unsafe extern "C" fn(
    CNA_LeaderboardEntryHandle, CNA_LeaderboardEntryHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_leaderboard_entry_get_columns_fn = unsafe extern "C" fn(
    CNA_LeaderboardEntryHandle, *mut CNA_PropertyDictionaryHandle,
) -> CNA_Result;
pub type cna_leaderboard_entry_get_gamer_fn = unsafe extern "C" fn(
    CNA_LeaderboardEntryHandle, *mut CNA_Bool, *mut CNA_GamerHandle,
) -> CNA_Result;
pub type cna_leaderboard_entry_get_info_fn = unsafe extern "C" fn(
    CNA_LeaderboardEntryHandle, *mut CNA_LeaderboardEntryInfo,
) -> CNA_Result;
pub type cna_leaderboard_entry_set_rating_fn = unsafe extern "C" fn(
    CNA_LeaderboardEntryHandle, i64,
) -> CNA_Result;
pub type cna_leaderboard_entry_set_rating_changed_hook_ext_fn = unsafe extern "C" fn(
    CNA_LeaderboardEntryHandle, CNA_GamerAsyncCallback, *mut c_void,
) -> CNA_Result;
pub type cna_leaderboard_identity_init_fn = unsafe extern "C" fn(
    CNA_LeaderboardKey, i32, *mut CNA_LeaderboardIdentity,
) -> CNA_Result;
pub type cna_leaderboard_reader_begin_page_down_fn = unsafe extern "C" fn(
    CNA_LeaderboardReaderHandle, CNA_GamerAsyncCallback, *mut c_void,
) -> CNA_Result;
pub type cna_leaderboard_reader_begin_page_up_fn = unsafe extern "C" fn(
    CNA_LeaderboardReaderHandle, CNA_GamerAsyncCallback, *mut c_void,
) -> CNA_Result;
pub type cna_leaderboard_reader_begin_read_fn = unsafe extern "C" fn(
    *const CNA_LeaderboardIdentity, i32, i32, CNA_GamerAsyncCallback, *mut c_void, *mut CNA_LeaderboardReaderHandle,
) -> CNA_Result;
pub type cna_leaderboard_reader_begin_read_from_gamers_fn = unsafe extern "C" fn(
    *const CNA_LeaderboardIdentity, *const CNA_GamerHandle, u64, CNA_GamerHandle, i32, CNA_GamerAsyncCallback, *mut c_void, *mut CNA_LeaderboardReaderHandle,
) -> CNA_Result;
pub type cna_leaderboard_reader_begin_read_from_pivot_fn = unsafe extern "C" fn(
    *const CNA_LeaderboardIdentity, CNA_GamerHandle, i32, CNA_GamerAsyncCallback, *mut c_void, *mut CNA_LeaderboardReaderHandle,
) -> CNA_Result;
pub type cna_leaderboard_reader_destroy_fn = unsafe extern "C" fn(
    CNA_LeaderboardReaderHandle,
) -> CNA_Result;
pub type cna_leaderboard_reader_get_entry_at_fn = unsafe extern "C" fn(
    CNA_LeaderboardReaderHandle, i32, *mut CNA_LeaderboardEntryHandle,
) -> CNA_Result;
pub type cna_leaderboard_reader_get_identity_fn = unsafe extern "C" fn(
    CNA_LeaderboardReaderHandle, *mut CNA_LeaderboardIdentity,
) -> CNA_Result;
pub type cna_leaderboard_reader_get_info_fn = unsafe extern "C" fn(
    CNA_LeaderboardReaderHandle, *mut CNA_LeaderboardReaderInfo,
) -> CNA_Result;
pub type cna_leaderboard_reader_page_down_fn = unsafe extern "C" fn(
    CNA_LeaderboardReaderHandle,
) -> CNA_Result;
pub type cna_leaderboard_reader_page_up_fn = unsafe extern "C" fn(
    CNA_LeaderboardReaderHandle,
) -> CNA_Result;
pub type cna_leaderboard_reader_read_fn = unsafe extern "C" fn(
    *const CNA_LeaderboardIdentity, i32, i32, *mut CNA_LeaderboardReaderHandle,
) -> CNA_Result;
pub type cna_leaderboard_reader_read_from_gamers_fn = unsafe extern "C" fn(
    *const CNA_LeaderboardIdentity, *const CNA_GamerHandle, u64, CNA_GamerHandle, i32, *mut CNA_LeaderboardReaderHandle,
) -> CNA_Result;
pub type cna_leaderboard_reader_read_from_pivot_fn = unsafe extern "C" fn(
    *const CNA_LeaderboardIdentity, CNA_GamerHandle, i32, *mut CNA_LeaderboardReaderHandle,
) -> CNA_Result;
pub type cna_local_network_gamer_clear_packet_queue_ext_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle,
) -> CNA_Result;
pub type cna_local_network_gamer_create_ext_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, CNA_NetworkSessionHandle, *mut CNA_NetworkGamerHandle,
) -> CNA_Result;
pub type cna_local_network_gamer_enable_send_voice_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, CNA_NetworkGamerHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_local_network_gamer_enqueue_packet_ext_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *const CNA_NetworkEventInfo,
) -> CNA_Result;
pub type cna_local_network_gamer_get_is_data_available_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_local_network_gamer_get_signed_in_gamer_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_SignedInGamerHandle,
) -> CNA_Result;
pub type cna_local_network_gamer_receive_data_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut u8, u64, *mut CNA_NetworkGamerHandle, *mut u64,
) -> CNA_Result;
pub type cna_local_network_gamer_receive_data_at_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut u8, u64, i32, *mut CNA_NetworkGamerHandle, *mut u64,
) -> CNA_Result;
pub type cna_local_network_gamer_receive_data_into_packet_reader_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, CNA_PacketReaderHandle, *mut CNA_NetworkGamerHandle, *mut u64,
) -> CNA_Result;
pub type cna_local_network_gamer_send_data_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *const u8, u64, CNA_SendDataOptions,
) -> CNA_Result;
pub type cna_local_network_gamer_send_data_range_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *const u8, u64, i32, i32, CNA_SendDataOptions,
) -> CNA_Result;
pub type cna_local_network_gamer_send_data_range_to_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *const u8, u64, i32, i32, CNA_SendDataOptions, CNA_NetworkGamerHandle,
) -> CNA_Result;
pub type cna_local_network_gamer_send_data_to_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *const u8, u64, CNA_SendDataOptions, CNA_NetworkGamerHandle,
) -> CNA_Result;
pub type cna_local_network_gamer_send_packet_writer_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, CNA_PacketWriterHandle, CNA_SendDataOptions,
) -> CNA_Result;
pub type cna_local_network_gamer_send_packet_writer_to_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, CNA_PacketWriterHandle, CNA_SendDataOptions, CNA_NetworkGamerHandle,
) -> CNA_Result;
pub type cna_local_network_gamer_send_party_invites_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle,
) -> CNA_Result;
pub type cna_net_get_last_join_error_fn = unsafe extern "C" fn(
    *mut CNA_NetworkSessionJoinError, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_gamer_copy_machine_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_NetworkMachineHandle,
) -> CNA_Result;
pub type cna_network_gamer_create_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_StringView, *mut CNA_NetworkGamerHandle,
) -> CNA_Result;
pub type cna_network_gamer_destroy_fn = unsafe extern "C" fn(CNA_NetworkGamerHandle) -> CNA_Result;
pub type cna_network_gamer_get_has_left_session_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_gamer_get_has_voice_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_gamer_get_id_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut u8,
) -> CNA_Result;
pub type cna_network_gamer_get_is_guest_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_gamer_get_is_host_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_gamer_get_is_local_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_gamer_get_is_muted_by_local_user_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_gamer_get_is_private_slot_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_gamer_get_is_ready_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_gamer_get_is_talking_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_gamer_get_roundtrip_ticks_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut i64,
) -> CNA_Result;
pub type cna_network_gamer_get_session_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_network_gamer_set_has_left_session_ext_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_network_gamer_set_id_ext_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, u8,
) -> CNA_Result;
pub type cna_network_gamer_set_is_host_ext_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_network_gamer_set_is_ready_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_network_gamer_set_machine_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, CNA_NetworkMachineHandle,
) -> CNA_Result;
pub type cna_network_gamer_set_roundtrip_ticks_ext_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, i64,
) -> CNA_Result;
pub type cna_network_machine_create_fn = unsafe extern "C" fn(
    *mut CNA_NetworkMachineHandle,
) -> CNA_Result;
pub type cna_network_machine_destroy_fn = unsafe extern "C" fn(
    CNA_NetworkMachineHandle,
) -> CNA_Result;
pub type cna_network_machine_get_gamer_fn = unsafe extern "C" fn(
    CNA_NetworkMachineHandle, i32, *mut CNA_NetworkGamerHandle,
) -> CNA_Result;
pub type cna_network_machine_get_gamer_count_fn = unsafe extern "C" fn(
    CNA_NetworkMachineHandle, *mut i32,
) -> CNA_Result;
pub type cna_network_machine_remove_from_session_fn = unsafe extern "C" fn(
    CNA_NetworkMachineHandle,
) -> CNA_Result;
pub type cna_network_session_add_local_gamer_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_network_session_add_remote_gamer_ext_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_NetworkGamerHandle,
) -> CNA_Result;
pub type cna_network_session_copy_session_properties_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut CNA_NetworkSessionPropertiesHandle,
) -> CNA_Result;
pub type cna_network_session_copy_type_name_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_network_session_create_fn = unsafe extern "C" fn(
    CNA_NetworkSessionType, i32, i32, *mut CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_create_async_fn = unsafe extern "C" fn(
    CNA_NetworkSessionType, i32, i32, CNA_NetworkSessionAsyncCallback, *mut c_void, *mut CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_create_with_local_gamers_fn = unsafe extern "C" fn(
    CNA_NetworkSessionType, *const CNA_Handle, u64, i32, i32, CNA_NetworkSessionPropertiesHandle, *mut CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_create_with_local_gamers_async_fn = unsafe extern "C" fn(
    CNA_NetworkSessionType, *const CNA_Handle, u64, i32, i32, CNA_NetworkSessionPropertiesHandle, CNA_NetworkSessionAsyncCallback, *mut c_void, *mut CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_create_with_properties_fn = unsafe extern "C" fn(
    CNA_NetworkSessionType, i32, i32, i32, CNA_NetworkSessionPropertiesHandle, *mut CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_create_with_properties_async_fn = unsafe extern "C" fn(
    CNA_NetworkSessionType, i32, i32, i32, CNA_NetworkSessionPropertiesHandle, CNA_NetworkSessionAsyncCallback, *mut c_void, *mut CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_destroy_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_dispose_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_ended_event_info_init_fn = unsafe extern "C" fn(
    CNA_NetworkSessionEndReason, *mut CNA_NetworkSessionEndedEventInfo,
) -> CNA_Result;
pub type cna_network_session_end_game_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_find_fn = unsafe extern "C" fn(
    CNA_NetworkSessionType, i32, CNA_NetworkSessionPropertiesHandle, *mut CNA_AvailableNetworkSessionCollectionHandle,
) -> CNA_Result;
pub type cna_network_session_find_async_fn = unsafe extern "C" fn(
    CNA_NetworkSessionType, i32, CNA_NetworkSessionPropertiesHandle, CNA_NetworkSessionAsyncCallback, *mut c_void, *mut CNA_AvailableNetworkSessionCollectionHandle,
) -> CNA_Result;
pub type cna_network_session_find_gamer_by_id_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, u8, *mut CNA_NetworkGamerHandle,
) -> CNA_Result;
pub type cna_network_session_find_with_local_gamers_fn = unsafe extern "C" fn(
    CNA_NetworkSessionType, *const CNA_Handle, u64, CNA_NetworkSessionPropertiesHandle, *mut CNA_AvailableNetworkSessionCollectionHandle,
) -> CNA_Result;
pub type cna_network_session_find_with_local_gamers_async_fn = unsafe extern "C" fn(
    CNA_NetworkSessionType, *const CNA_Handle, u64, CNA_NetworkSessionPropertiesHandle, CNA_NetworkSessionAsyncCallback, *mut c_void, *mut CNA_AvailableNetworkSessionCollectionHandle,
) -> CNA_Result;
pub type cna_network_session_get_active_action_count_ext_fn = unsafe extern "C" fn(
    *mut i32,
) -> CNA_Result;
pub type cna_network_session_get_allow_host_migration_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_session_get_allow_join_in_progress_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_session_get_bytes_per_second_received_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut i32,
) -> CNA_Result;
pub type cna_network_session_get_bytes_per_second_sent_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut i32,
) -> CNA_Result;
pub type cna_network_session_get_gamer_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, u32, i32, *mut CNA_NetworkGamerHandle,
) -> CNA_Result;
pub type cna_network_session_get_gamer_count_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, u32, *mut i32,
) -> CNA_Result;
pub type cna_network_session_get_host_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut CNA_NetworkGamerHandle,
) -> CNA_Result;
pub type cna_network_session_get_instance_count_ext_fn = unsafe extern "C" fn(
    *mut i32,
) -> CNA_Result;
pub type cna_network_session_get_is_disposed_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_session_get_is_everyone_ready_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_session_get_is_host_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_session_get_max_gamers_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut i32,
) -> CNA_Result;
pub type cna_network_session_get_owned_gamer_count_ext_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut u64,
) -> CNA_Result;
pub type cna_network_session_get_private_gamer_slots_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut i32,
) -> CNA_Result;
pub type cna_network_session_get_session_state_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut CNA_NetworkSessionState,
) -> CNA_Result;
pub type cna_network_session_get_session_type_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut CNA_NetworkSessionType,
) -> CNA_Result;
pub type cna_network_session_get_simulated_latency_ticks_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut i64,
) -> CNA_Result;
pub type cna_network_session_get_simulated_packet_loss_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut f32,
) -> CNA_Result;
pub type cna_network_session_get_type_name_size_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *mut u64,
) -> CNA_Result;
pub type cna_network_session_join_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, *mut CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_join_async_fn = unsafe extern "C" fn(
    CNA_AvailableNetworkSessionHandle, CNA_NetworkSessionAsyncCallback, *mut c_void, *mut CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_join_invited_fn = unsafe extern "C" fn(
    i32, *mut CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_join_invited_async_fn = unsafe extern "C" fn(
    i32, CNA_NetworkSessionAsyncCallback, *mut c_void, *mut CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_join_invited_with_local_gamers_fn = unsafe extern "C" fn(
    *const CNA_Handle, u64, *mut CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_join_invited_with_local_gamers_async_fn = unsafe extern "C" fn(
    *const CNA_Handle, u64, CNA_NetworkSessionAsyncCallback, *mut c_void, *mut CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_properties_add_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle, CNA_OptionalInt32,
) -> CNA_Result;
pub type cna_network_session_properties_clear_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle,
) -> CNA_Result;
pub type cna_network_session_properties_contains_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle, CNA_OptionalInt32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_session_properties_copy_to_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle, *mut CNA_OptionalInt32, u64, i32, *mut u64,
) -> CNA_Result;
pub type cna_network_session_properties_create_fn = unsafe extern "C" fn(
    *mut CNA_NetworkSessionPropertiesHandle,
) -> CNA_Result;
pub type cna_network_session_properties_create_enumerator_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle, *mut CNA_NetworkSessionPropertyEnumeratorHandle,
) -> CNA_Result;
pub type cna_network_session_properties_destroy_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle,
) -> CNA_Result;
pub type cna_network_session_properties_get_count_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle, *mut i32,
) -> CNA_Result;
pub type cna_network_session_properties_get_is_read_only_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_session_properties_get_item_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle, i32, *mut CNA_OptionalInt32,
) -> CNA_Result;
pub type cna_network_session_properties_index_of_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle, CNA_OptionalInt32, *mut i32,
) -> CNA_Result;
pub type cna_network_session_properties_insert_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle, i32, CNA_OptionalInt32,
) -> CNA_Result;
pub type cna_network_session_properties_remove_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle, CNA_OptionalInt32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_session_properties_remove_at_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle, i32,
) -> CNA_Result;
pub type cna_network_session_properties_set_item_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertiesHandle, i32, CNA_OptionalInt32,
) -> CNA_Result;
pub type cna_network_session_property_enumerator_destroy_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertyEnumeratorHandle,
) -> CNA_Result;
pub type cna_network_session_property_enumerator_get_current_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertyEnumeratorHandle, *mut CNA_OptionalInt32,
) -> CNA_Result;
pub type cna_network_session_property_enumerator_move_next_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertyEnumeratorHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_network_session_property_enumerator_reset_fn = unsafe extern "C" fn(
    CNA_NetworkSessionPropertyEnumeratorHandle,
) -> CNA_Result;
pub type cna_network_session_remove_gamer_ext_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_NetworkGamerHandle, CNA_NetworkSessionEndReason,
) -> CNA_Result;
pub type cna_network_session_reset_ready_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_send_network_event_ext_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, *const CNA_NetworkEventInfo,
) -> CNA_Result;
pub type cna_network_session_set_allow_host_migration_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_network_session_set_allow_join_in_progress_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_network_session_set_max_gamers_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, i32,
) -> CNA_Result;
pub type cna_network_session_set_private_gamer_slots_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, i32,
) -> CNA_Result;
pub type cna_network_session_set_simulated_latency_ticks_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, i64,
) -> CNA_Result;
pub type cna_network_session_set_simulated_packet_loss_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, f32,
) -> CNA_Result;
pub type cna_network_session_start_game_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_network_session_subscribe_game_ended_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_GameEndedCallback, *mut c_void, *mut CNA_NetworkSessionEventRegistrationHandle,
) -> CNA_Result;
pub type cna_network_session_subscribe_gamer_joined_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_GamerJoinedCallback, *mut c_void, *mut CNA_NetworkSessionEventRegistrationHandle,
) -> CNA_Result;
pub type cna_network_session_subscribe_gamer_left_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_GamerLeftCallback, *mut c_void, *mut CNA_NetworkSessionEventRegistrationHandle,
) -> CNA_Result;
pub type cna_network_session_subscribe_game_started_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_GameStartedCallback, *mut c_void, *mut CNA_NetworkSessionEventRegistrationHandle,
) -> CNA_Result;
pub type cna_network_session_subscribe_host_changed_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_HostChangedCallback, *mut c_void, *mut CNA_NetworkSessionEventRegistrationHandle,
) -> CNA_Result;
pub type cna_network_session_subscribe_invite_accepted_fn = unsafe extern "C" fn(
    CNA_InviteAcceptedCallback, *mut c_void, *mut CNA_NetworkSessionEventRegistrationHandle,
) -> CNA_Result;
pub type cna_network_session_subscribe_session_ended_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_NetworkSessionEndedCallback, *mut c_void, *mut CNA_NetworkSessionEventRegistrationHandle,
) -> CNA_Result;
pub type cna_network_session_subscribe_write_arbitrated_leaderboard_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_WriteLeaderboardsCallback, *mut c_void, *mut CNA_NetworkSessionEventRegistrationHandle,
) -> CNA_Result;
pub type cna_network_session_subscribe_write_true_skill_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_WriteLeaderboardsCallback, *mut c_void, *mut CNA_NetworkSessionEventRegistrationHandle,
) -> CNA_Result;
pub type cna_network_session_subscribe_write_unarbitrated_leaderboard_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle, CNA_WriteLeaderboardsCallback, *mut c_void, *mut CNA_NetworkSessionEventRegistrationHandle,
) -> CNA_Result;
pub type cna_network_session_unsubscribe_fn = unsafe extern "C" fn(
    CNA_NetworkSessionEventRegistrationHandle,
) -> CNA_Result;
pub type cna_network_session_update_fn = unsafe extern "C" fn(
    CNA_NetworkSessionHandle,
) -> CNA_Result;
pub type cna_packet_reader_create_fn = unsafe extern "C" fn(
    i32, *mut CNA_PacketReaderHandle,
) -> CNA_Result;
pub type cna_packet_reader_destroy_fn = unsafe extern "C" fn(CNA_PacketReaderHandle) -> CNA_Result;
pub type cna_packet_reader_get_length_fn = unsafe extern "C" fn(
    CNA_PacketReaderHandle, *mut i32,
) -> CNA_Result;
pub type cna_packet_reader_get_position_fn = unsafe extern "C" fn(
    CNA_PacketReaderHandle, *mut i32,
) -> CNA_Result;
pub type cna_packet_reader_read_color_fn = unsafe extern "C" fn(
    CNA_PacketReaderHandle, *mut CNA_Color,
) -> CNA_Result;
pub type cna_packet_reader_read_double_fn = unsafe extern "C" fn(
    CNA_PacketReaderHandle, *mut f64,
) -> CNA_Result;
pub type cna_packet_reader_read_matrix_fn = unsafe extern "C" fn(
    CNA_PacketReaderHandle, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_packet_reader_read_quaternion_fn = unsafe extern "C" fn(
    CNA_PacketReaderHandle, *mut CNA_Quaternion,
) -> CNA_Result;
pub type cna_packet_reader_read_single_fn = unsafe extern "C" fn(
    CNA_PacketReaderHandle, *mut f32,
) -> CNA_Result;
pub type cna_packet_reader_read_vector2_fn = unsafe extern "C" fn(
    CNA_PacketReaderHandle, *mut CNA_Vector2,
) -> CNA_Result;
pub type cna_packet_reader_read_vector3_fn = unsafe extern "C" fn(
    CNA_PacketReaderHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_packet_reader_read_vector4_fn = unsafe extern "C" fn(
    CNA_PacketReaderHandle, *mut CNA_Vector4,
) -> CNA_Result;
pub type cna_packet_reader_set_data_ext_fn = unsafe extern "C" fn(
    CNA_PacketReaderHandle, *const u8, u64,
) -> CNA_Result;
pub type cna_packet_reader_set_position_fn = unsafe extern "C" fn(
    CNA_PacketReaderHandle, i32,
) -> CNA_Result;
pub type cna_packet_writer_copy_data_ext_fn = unsafe extern "C" fn(
    CNA_PacketWriterHandle, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_packet_writer_create_fn = unsafe extern "C" fn(
    i32, *mut CNA_PacketWriterHandle,
) -> CNA_Result;
pub type cna_packet_writer_destroy_fn = unsafe extern "C" fn(CNA_PacketWriterHandle) -> CNA_Result;
pub type cna_packet_writer_get_length_fn = unsafe extern "C" fn(
    CNA_PacketWriterHandle, *mut i32,
) -> CNA_Result;
pub type cna_packet_writer_get_position_fn = unsafe extern "C" fn(
    CNA_PacketWriterHandle, *mut i32,
) -> CNA_Result;
pub type cna_packet_writer_set_position_fn = unsafe extern "C" fn(
    CNA_PacketWriterHandle, i32,
) -> CNA_Result;
pub type cna_packet_writer_write_color_fn = unsafe extern "C" fn(
    CNA_PacketWriterHandle, CNA_Color,
) -> CNA_Result;
pub type cna_packet_writer_write_double_fn = unsafe extern "C" fn(
    CNA_PacketWriterHandle, f64,
) -> CNA_Result;
pub type cna_packet_writer_write_matrix_fn = unsafe extern "C" fn(
    CNA_PacketWriterHandle, CNA_Matrix,
) -> CNA_Result;
pub type cna_packet_writer_write_quaternion_fn = unsafe extern "C" fn(
    CNA_PacketWriterHandle, CNA_Quaternion,
) -> CNA_Result;
pub type cna_packet_writer_write_single_fn = unsafe extern "C" fn(
    CNA_PacketWriterHandle, f32,
) -> CNA_Result;
pub type cna_packet_writer_write_vector2_fn = unsafe extern "C" fn(
    CNA_PacketWriterHandle, CNA_Vector2,
) -> CNA_Result;
pub type cna_packet_writer_write_vector3_fn = unsafe extern "C" fn(
    CNA_PacketWriterHandle, CNA_Vector3,
) -> CNA_Result;
pub type cna_packet_writer_write_vector4_fn = unsafe extern "C" fn(
    CNA_PacketWriterHandle, CNA_Vector4,
) -> CNA_Result;
pub type cna_property_dictionary_clear_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle,
) -> CNA_Result;
pub type cna_property_dictionary_contains_key_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_property_dictionary_copy_key_at_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, i32, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_property_dictionary_copy_string_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_property_dictionary_create_ext_fn = unsafe extern "C" fn(
    *mut CNA_PropertyDictionaryHandle,
) -> CNA_Result;
pub type cna_property_dictionary_destroy_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle,
) -> CNA_Result;
pub type cna_property_dictionary_get_count_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, *mut i32,
) -> CNA_Result;
pub type cna_property_dictionary_get_date_time_ticks_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, *mut i64,
) -> CNA_Result;
pub type cna_property_dictionary_get_double_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, *mut f64,
) -> CNA_Result;
pub type cna_property_dictionary_get_int32_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, *mut i32,
) -> CNA_Result;
pub type cna_property_dictionary_get_int64_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, *mut i64,
) -> CNA_Result;
pub type cna_property_dictionary_get_is_read_only_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_property_dictionary_get_key_size_at_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, i32, *mut u64,
) -> CNA_Result;
pub type cna_property_dictionary_get_outcome_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, *mut CNA_LeaderboardOutcome,
) -> CNA_Result;
pub type cna_property_dictionary_get_single_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, *mut f32,
) -> CNA_Result;
pub type cna_property_dictionary_get_stream_size_ext_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, *mut CNA_Bool, *mut u64,
) -> CNA_Result;
pub type cna_property_dictionary_get_string_size_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, *mut u64,
) -> CNA_Result;
pub type cna_property_dictionary_get_time_span_ticks_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, *mut i64,
) -> CNA_Result;
pub type cna_property_dictionary_remove_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_property_dictionary_set_date_time_ticks_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, i64,
) -> CNA_Result;
pub type cna_property_dictionary_set_double_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, f64,
) -> CNA_Result;
pub type cna_property_dictionary_set_int32_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, i32,
) -> CNA_Result;
pub type cna_property_dictionary_set_int64_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, i64,
) -> CNA_Result;
pub type cna_property_dictionary_set_outcome_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, CNA_LeaderboardOutcome,
) -> CNA_Result;
pub type cna_property_dictionary_set_single_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, f32,
) -> CNA_Result;
pub type cna_property_dictionary_set_string_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, CNA_StringView,
) -> CNA_Result;
pub type cna_property_dictionary_set_time_span_ticks_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, i64,
) -> CNA_Result;
pub type cna_property_dictionary_try_get_value_kind_ext_fn = unsafe extern "C" fn(
    CNA_PropertyDictionaryHandle, CNA_StringView, *mut CNA_Bool, *mut CNA_PropertyValueKind,
) -> CNA_Result;
pub type cna_quality_of_service_init_fn = unsafe extern "C" fn(
    *mut CNA_QualityOfService,
) -> CNA_Result;
pub type cna_quality_of_service_init_measured_fn = unsafe extern "C" fn(
    i64, *mut CNA_QualityOfService,
) -> CNA_Result;
pub type cna_signed_in_gamer_award_achievement_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, CNA_StringView,
) -> CNA_Result;
pub type cna_signed_in_gamer_begin_award_achievement_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, CNA_StringView, CNA_GamerAsyncCallback, *mut c_void,
) -> CNA_Result;
pub type cna_signed_in_gamer_begin_get_achievements_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, CNA_GamerAsyncCallback, *mut c_void, *mut CNA_AchievementCollectionHandle,
) -> CNA_Result;
pub type cna_signed_in_gamer_copy_gamertag_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_signed_in_gamer_create_ext_fn = unsafe extern "C" fn(
    CNA_StringView, CNA_Bool, CNA_Bool, CNA_PlayerIndex, *mut CNA_SignedInGamerHandle,
) -> CNA_Result;
pub type cna_signed_in_gamer_destroy_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle,
) -> CNA_Result;
pub type cna_signed_in_gamer_get_achievements_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *mut CNA_AchievementCollectionHandle,
) -> CNA_Result;
pub type cna_signed_in_gamer_get_friends_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *mut CNA_GamerCollectionHandle,
) -> CNA_Result;
pub type cna_signed_in_gamer_get_game_defaults_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *mut CNA_GameDefaults,
) -> CNA_Result;
pub type cna_signed_in_gamer_get_gamertag_size_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *mut u64,
) -> CNA_Result;
pub type cna_signed_in_gamer_get_is_guest_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_signed_in_gamer_get_is_signed_in_to_live_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_signed_in_gamer_get_party_size_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *mut i32,
) -> CNA_Result;
pub type cna_signed_in_gamer_get_player_index_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *mut CNA_PlayerIndex,
) -> CNA_Result;
pub type cna_signed_in_gamer_get_presence_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *mut CNA_GamerPresence,
) -> CNA_Result;
pub type cna_signed_in_gamer_get_privileges_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *mut CNA_GamerPrivileges,
) -> CNA_Result;
pub type cna_signed_in_gamer_is_friend_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, CNA_GamerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_signed_in_gamer_is_headset_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, u64, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_signed_in_gamer_set_party_size_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, i32,
) -> CNA_Result;
pub type cna_signed_in_gamer_set_presence_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, *const CNA_GamerPresence,
) -> CNA_Result;
pub type cna_signed_in_gamer_set_presence_mode_string_ext_fn = unsafe extern "C" fn(
    CNA_SignedInGamerHandle, CNA_StringView,
) -> CNA_Result;
pub type cna_signed_in_gamer_subscribe_signed_in_ext_fn = unsafe extern "C" fn(
    CNA_SignedInGamerEventCallback, *mut c_void, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_signed_in_gamer_subscribe_signed_out_ext_fn = unsafe extern "C" fn(
    CNA_SignedInGamerEventCallback, *mut c_void, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_write_leaderboards_event_info_init_fn = unsafe extern "C" fn(
    CNA_NetworkGamerHandle, CNA_Bool, *mut CNA_WriteLeaderboardsEventInfo,
) -> CNA_Result;

// --- CNA engine layer: the render pipeline (engine_layer.h) ---
pub type cna_render_pipeline_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_RenderPipelineHandle,
) -> CNA_Result;
pub type cna_render_pipeline_destroy_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle,
) -> CNA_Result;
pub type cna_render_pipeline_get_settings_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut CNA_RenderPipelineSettingsEXT,
) -> CNA_Result;
pub type cna_render_pipeline_set_settings_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *const CNA_RenderPipelineSettingsEXT,
) -> CNA_Result;
pub type cna_render_pipeline_resize_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, i32, i32,
) -> CNA_Result;
pub type cna_render_pipeline_begin_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *const CNA_Color,
) -> CNA_Result;
pub type cna_render_pipeline_end_fn = unsafe extern "C" fn(CNA_RenderPipelineHandle) -> CNA_Result;
pub type cna_render_pipeline_set_depth_normal_inputs_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, CNA_Handle, CNA_Handle,
) -> CNA_Result;
pub type cna_render_pipeline_set_velocity_input_ext_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_render_pipeline_set_transparent_scene_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, CNA_RenderPipelineDrawCallback, *mut c_void,
) -> CNA_Result;
pub type cna_render_pipeline_set_camera_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *const CNA_Matrix, *const CNA_Matrix, f32, f32,
) -> CNA_Result;
pub type cna_render_pipeline_set_skybox_camera_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *const CNA_Matrix, *const CNA_Matrix,
) -> CNA_Result;
pub type cna_render_pipeline_copy_transparency_fallback_reason_ext_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_render_pipeline_set_gpu_timing_enabled_ext_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_render_pipeline_is_gpu_timing_enabled_ext_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_render_pipeline_did_skybox_draw_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_render_pipeline_did_shadow_pass_run_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_render_pipeline_get_scene_target_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_render_pipeline_get_scene_target_format_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut CNA_SurfaceFormat,
) -> CNA_Result;
pub type cna_render_pipeline_is_using_scene_target_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_render_pipeline_get_last_frame_pass_count_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut i32,
) -> CNA_Result;
pub type cna_render_pipeline_get_gpu_memory_estimate_bytes_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut u64,
) -> CNA_Result;
pub type cna_render_pipeline_get_statistics_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut CNA_RenderPipelineFrameStatisticsEXT,
) -> CNA_Result;
pub type cna_render_pipeline_release_device_resources_ext_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle,
) -> CNA_Result;
pub type cna_render_pipeline_get_pass_timing_count_ext_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut u64,
) -> CNA_Result;
pub type cna_render_pipeline_get_pass_timing_ext_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, u64, *mut CNA_PassTimingEXT,
) -> CNA_Result;
pub type cna_render_pipeline_copy_pass_timing_name_ext_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;

// --- CNA engine layer: directional-light shadow maps (engine_layer.h) ---
pub type cna_directional_light_ext_init_fn = unsafe extern "C" fn(
    *mut CNA_DirectionalLightEXT,
) -> CNA_Result;
pub type cna_graphics_device_supports_shadow_sampling_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_shadow_map_create_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_ShadowQuality, *mut CNA_ShadowMapHandle,
) -> CNA_Result;
pub type cna_shadow_map_destroy_fn = unsafe extern "C" fn(CNA_ShadowMapHandle) -> CNA_Result;
pub type cna_shadow_map_is_supported_fn = unsafe extern "C" fn(
    CNA_ShadowMapHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_shadow_map_begin_fn = unsafe extern "C" fn(
    CNA_ShadowMapHandle, *const CNA_DirectionalLightEXT, *const CNA_BoundingBox,
) -> CNA_Result;
pub type cna_shadow_map_end_fn = unsafe extern "C" fn(CNA_ShadowMapHandle) -> CNA_Result;
pub type cna_shadow_map_get_caster_effect_fn = unsafe extern "C" fn(
    CNA_ShadowMapHandle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_shadow_map_get_skinned_caster_effect_fn = unsafe extern "C" fn(
    CNA_ShadowMapHandle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_shadow_map_apply_caster_fn = unsafe extern "C" fn(CNA_ShadowMapHandle) -> CNA_Result;
pub type cna_shadow_map_apply_skinned_caster_fn = unsafe extern "C" fn(
    CNA_ShadowMapHandle, *const CNA_Matrix, u64, i32,
) -> CNA_Result;
pub type cna_shadow_map_get_shadow_texture_fn = unsafe extern "C" fn(
    CNA_ShadowMapHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_shadow_map_get_light_view_projection_fn = unsafe extern "C" fn(
    CNA_ShadowMapHandle, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_shadow_map_get_size_fn = unsafe extern "C" fn(
    CNA_ShadowMapHandle, *mut i32,
) -> CNA_Result;
pub type cna_shadow_map_get_quality_fn = unsafe extern "C" fn(
    CNA_ShadowMapHandle, *mut CNA_ShadowQuality,
) -> CNA_Result;
pub type cna_shadow_map_get_depth_bias_fn = unsafe extern "C" fn(
    CNA_ShadowMapHandle, *mut f32,
) -> CNA_Result;
pub type cna_shadow_map_set_depth_bias_fn = unsafe extern "C" fn(
    CNA_ShadowMapHandle, f32,
) -> CNA_Result;
pub type cna_shadow_map_get_filter_radius_fn = unsafe extern "C" fn(
    CNA_ShadowMapHandle, *mut i32,
) -> CNA_Result;
pub type cna_shadow_map_compute_light_view_fn = unsafe extern "C" fn(
    *const CNA_DirectionalLightEXT, *const CNA_BoundingBox, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_shadow_map_compute_light_projection_fn = unsafe extern "C" fn(
    *const CNA_Matrix, *const CNA_BoundingBox, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_shadow_map_size_for_quality_fn = unsafe extern "C" fn(
    CNA_ShadowQuality, *mut i32,
) -> CNA_Result;
pub type cna_shadow_map_filter_radius_for_quality_fn = unsafe extern "C" fn(
    CNA_ShadowQuality, *mut i32,
) -> CNA_Result;
pub type cna_render_pipeline_set_shadow_scene_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, CNA_ShadowMapHandle, *const CNA_DirectionalLightEXT, *const CNA_BoundingBox, CNA_RenderPipelineDrawCallback, *mut c_void,
) -> CNA_Result;
pub type cna_render_pipeline_get_shadow_map_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut CNA_ShadowMapHandle,
) -> CNA_Result;

// --- CNA engine layer: the post-process chain and its passes (engine_layer.h) ---
pub type cna_post_process_context_init_fn = unsafe extern "C" fn(
    *mut CNA_PostProcessContext,
) -> CNA_Result;
pub type cna_blit_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_post_process_effect_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_EffectHandle, CNA_StringView, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_post_process_effect_pass_create_owning_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_EffectHandle, CNA_StringView, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_post_process_effect_pass_get_effect_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_post_process_effect_pass_set_effect_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, CNA_EffectHandle,
) -> CNA_Result;
pub type cna_post_process_pass_apply_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *const CNA_PostProcessContext,
) -> CNA_Result;
pub type cna_post_process_pass_copy_name_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_post_process_pass_is_supported_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_post_process_pass_destroy_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_post_process_chain_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessChainHandle,
) -> CNA_Result;
pub type cna_post_process_chain_destroy_fn = unsafe extern "C" fn(
    CNA_PostProcessChainHandle,
) -> CNA_Result;
pub type cna_post_process_chain_add_pass_fn = unsafe extern "C" fn(
    CNA_PostProcessChainHandle, CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_post_process_chain_add_owned_pass_fn = unsafe extern "C" fn(
    CNA_PostProcessChainHandle, CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_post_process_chain_clear_fn = unsafe extern "C" fn(
    CNA_PostProcessChainHandle,
) -> CNA_Result;
pub type cna_post_process_chain_get_pass_count_fn = unsafe extern "C" fn(
    CNA_PostProcessChainHandle, *mut i32,
) -> CNA_Result;
pub type cna_post_process_chain_apply_fn = unsafe extern "C" fn(
    CNA_PostProcessChainHandle, *const CNA_PostProcessContext,
) -> CNA_Result;
pub type cna_post_process_chain_reset_targets_fn = unsafe extern "C" fn(
    CNA_PostProcessChainHandle,
) -> CNA_Result;
pub type cna_post_process_chain_get_target_pool_fn = unsafe extern "C" fn(
    CNA_PostProcessChainHandle, *mut CNA_RenderTargetPoolHandle,
) -> CNA_Result;
pub type cna_post_process_chain_is_gpu_timing_enabled_fn = unsafe extern "C" fn(
    CNA_PostProcessChainHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_post_process_chain_set_gpu_timing_enabled_fn = unsafe extern "C" fn(
    CNA_PostProcessChainHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_post_process_chain_get_pass_timing_count_fn = unsafe extern "C" fn(
    CNA_PostProcessChainHandle, *mut u64,
) -> CNA_Result;
pub type cna_post_process_chain_get_pass_timing_fn = unsafe extern "C" fn(
    CNA_PostProcessChainHandle, u64, *mut CNA_PassTimingEXT,
) -> CNA_Result;
pub type cna_post_process_chain_copy_pass_timing_name_fn = unsafe extern "C" fn(
    CNA_PostProcessChainHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_render_pipeline_add_user_pass_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_render_pipeline_clear_user_passes_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle,
) -> CNA_Result;
pub type cna_render_target_pool_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_RenderTargetPoolHandle,
) -> CNA_Result;
pub type cna_render_target_pool_destroy_fn = unsafe extern "C" fn(
    CNA_RenderTargetPoolHandle,
) -> CNA_Result;
pub type cna_render_target_pool_acquire_fn = unsafe extern "C" fn(
    CNA_RenderTargetPoolHandle, i32, i32, CNA_SurfaceFormat, CNA_DepthFormat, i32, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_render_target_pool_reset_fn = unsafe extern "C" fn(
    CNA_RenderTargetPoolHandle,
) -> CNA_Result;
pub type cna_render_target_pool_get_target_count_fn = unsafe extern "C" fn(
    CNA_RenderTargetPoolHandle, *mut u64,
) -> CNA_Result;
pub type cna_render_target_pool_get_estimated_bytes_fn = unsafe extern "C" fn(
    CNA_RenderTargetPoolHandle, *mut u64,
) -> CNA_Result;
pub type cna_tonemap_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_tonemap_pass_get_mode_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut CNA_TonemappingMode,
) -> CNA_Result;
pub type cna_tonemap_pass_set_mode_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, CNA_TonemappingMode,
) -> CNA_Result;
pub type cna_tonemap_pass_get_exposure_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_tonemap_pass_set_exposure_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_tonemap_pass_get_gamma_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_tonemap_pass_set_gamma_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_tonemap_pass_is_deband_enabled_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_tonemap_pass_set_deband_enabled_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_tonemap_pass_get_deband_strength_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_tonemap_pass_set_deband_strength_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_tonemap_pass_tonemap_channel_fn = unsafe extern "C" fn(
    CNA_TonemappingMode, f32, f32, f32, *mut f32,
) -> CNA_Result;
pub type cna_fxaa_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_fxaa_pass_get_edge_threshold_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_fxaa_pass_set_edge_threshold_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_fxaa_pass_edge_threshold_for_quality_fn = unsafe extern "C" fn(
    CNA_RenderQuality, *mut f32,
) -> CNA_Result;
pub type cna_fxaa_pass_copy_fragment_glsl_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;

// --- CNA engine layer: GPU timers and particle systems (engine_layer.h) ---
pub type cna_gpu_timer_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_GpuTimerHandle,
) -> CNA_Result;
pub type cna_gpu_timer_destroy_fn = unsafe extern "C" fn(CNA_GpuTimerHandle) -> CNA_Result;
pub type cna_gpu_timer_is_supported_fn = unsafe extern "C" fn(
    CNA_GpuTimerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gpu_timer_copy_unsupported_reason_fn = unsafe extern "C" fn(
    CNA_GpuTimerHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_gpu_timer_begin_fn = unsafe extern "C" fn(CNA_GpuTimerHandle) -> CNA_Result;
pub type cna_gpu_timer_end_fn = unsafe extern "C" fn(CNA_GpuTimerHandle) -> CNA_Result;
pub type cna_gpu_timer_is_result_available_fn = unsafe extern "C" fn(
    CNA_GpuTimerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gpu_timer_poll_fn = unsafe extern "C" fn(
    CNA_GpuTimerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gpu_timer_get_last_milliseconds_fn = unsafe extern "C" fn(
    CNA_GpuTimerHandle, *mut f64,
) -> CNA_Result;
pub type cna_gpu_timer_get_sample_count_fn = unsafe extern "C" fn(
    CNA_GpuTimerHandle, *mut i32,
) -> CNA_Result;
pub type cna_gpu_timer_is_open_fn = unsafe extern "C" fn(
    CNA_GpuTimerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_particle_init_fn = unsafe extern "C" fn(*mut CNA_Particle) -> CNA_Result;
pub type cna_particle_emitter_settings_init_fn = unsafe extern "C" fn(
    *mut CNA_ParticleEmitterSettings,
) -> CNA_Result;
pub type cna_particle_system_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_ParticleSystemHandle,
) -> CNA_Result;
pub type cna_particle_system_create_with_capacity_fn = unsafe extern "C" fn(
    CNA_Handle, i32, *mut CNA_ParticleSystemHandle,
) -> CNA_Result;
pub type cna_particle_system_destroy_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle,
) -> CNA_Result;
pub type cna_particle_system_get_settings_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, *mut CNA_ParticleEmitterSettings,
) -> CNA_Result;
pub type cna_particle_system_set_settings_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, *const CNA_ParticleEmitterSettings,
) -> CNA_Result;
pub type cna_particle_system_get_capacity_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, *mut i32,
) -> CNA_Result;
pub type cna_particle_system_get_active_count_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, *mut i32,
) -> CNA_Result;
pub type cna_particle_system_is_emission_rate_clamped_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_particle_system_update_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, f32,
) -> CNA_Result;
pub type cna_particle_system_reset_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle,
) -> CNA_Result;
pub type cna_particle_system_draw_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, *const CNA_Matrix, *const CNA_Matrix, CNA_Handle,
) -> CNA_Result;
pub type cna_particle_system_copy_particles_ext_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, *mut CNA_Particle, u64, *mut u64,
) -> CNA_Result;
pub type cna_particle_system_step_fn = unsafe extern "C" fn(
    *mut CNA_Particle, i32, *const CNA_ParticleEmitterSettings, f32,
) -> CNA_Result;
pub type cna_particle_system_random_fn = unsafe extern "C" fn(u32, *mut f32) -> CNA_Result;
pub type cna_particle_system_uses_compute_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_particle_system_is_simulation_on_cpu_ext_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_particle_system_set_simulation_on_cpu_ext_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_particle_system_copy_unsupported_reason_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_particle_system_copy_particle_lookup_glsl_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_particle_system_get_softness_ext_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, *mut f32,
) -> CNA_Result;
pub type cna_particle_system_set_softness_ext_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, f32,
) -> CNA_Result;
pub type cna_particle_system_set_depth_input_ext_fn = unsafe extern "C" fn(
    CNA_ParticleSystemHandle, CNA_Handle, f32,
) -> CNA_Result;

// --- CNA engine layer: storage buffers and compute shaders (engine_layer.h) ---
pub type cna_compute_shader_barrier_fn = unsafe extern "C" fn(
    CNA_ComputeShaderHandle, CNA_GraphicsMemoryBarrier,
) -> CNA_Result;
pub type cna_compute_shader_bind_image_fn = unsafe extern "C" fn(
    CNA_ComputeShaderHandle, i32, CNA_Handle, CNA_GraphicsImageAccess,
) -> CNA_Result;
pub type cna_compute_shader_bind_storage_buffer_fn = unsafe extern "C" fn(
    CNA_ComputeShaderHandle, i32, CNA_StorageBufferHandle,
) -> CNA_Result;
pub type cna_compute_shader_bind_texture_fn = unsafe extern "C" fn(
    CNA_ComputeShaderHandle, i32, CNA_StringView, CNA_Handle,
) -> CNA_Result;
pub type cna_compute_shader_copy_compile_error_fn = unsafe extern "C" fn(
    CNA_ComputeShaderHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_compute_shader_create_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_StringView, *mut CNA_ComputeShaderHandle,
) -> CNA_Result;
pub type cna_compute_shader_destroy_fn = unsafe extern "C" fn(
    CNA_ComputeShaderHandle,
) -> CNA_Result;
pub type cna_compute_shader_dispatch_fn = unsafe extern "C" fn(
    CNA_ComputeShaderHandle, i32, i32, i32,
) -> CNA_Result;
pub type cna_compute_shader_is_image_binding_supported_fn = unsafe extern "C" fn(
    CNA_ComputeShaderHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_compute_shader_is_valid_fn = unsafe extern "C" fn(
    CNA_ComputeShaderHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_compute_shader_set_uniform_float_fn = unsafe extern "C" fn(
    CNA_ComputeShaderHandle, CNA_StringView, f32,
) -> CNA_Result;
pub type cna_compute_shader_set_uniform_int_fn = unsafe extern "C" fn(
    CNA_ComputeShaderHandle, CNA_StringView, i32,
) -> CNA_Result;
pub type cna_graphics_memory_barrier_has_fn = unsafe extern "C" fn(
    CNA_GraphicsMemoryBarrier, CNA_GraphicsMemoryBarrier, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_storage_buffer_create_fn = unsafe extern "C" fn(
    CNA_Handle, u64, *mut CNA_StorageBufferHandle,
) -> CNA_Result;
pub type cna_storage_buffer_create_typed_fn = unsafe extern "C" fn(
    CNA_Handle, u64, u64, *mut CNA_StorageBufferHandle,
) -> CNA_Result;
pub type cna_storage_buffer_destroy_fn = unsafe extern "C" fn(
    CNA_StorageBufferHandle,
) -> CNA_Result;
pub type cna_storage_buffer_get_bytes_fn = unsafe extern "C" fn(
    CNA_StorageBufferHandle, *mut c_void, u64,
) -> CNA_Result;
pub type cna_storage_buffer_get_byte_size_fn = unsafe extern "C" fn(
    CNA_StorageBufferHandle, *mut u64,
) -> CNA_Result;
pub type cna_storage_buffer_get_element_byte_size_fn = unsafe extern "C" fn(
    CNA_StorageBufferHandle, *mut u64,
) -> CNA_Result;
pub type cna_storage_buffer_get_element_count_fn = unsafe extern "C" fn(
    CNA_StorageBufferHandle, *mut u64,
) -> CNA_Result;
pub type cna_storage_buffer_get_elements_fn = unsafe extern "C" fn(
    CNA_StorageBufferHandle, *mut c_void, u64, u64,
) -> CNA_Result;
pub type cna_storage_buffer_set_bytes_fn = unsafe extern "C" fn(
    CNA_StorageBufferHandle, *const c_void, u64,
) -> CNA_Result;
pub type cna_storage_buffer_set_elements_fn = unsafe extern "C" fn(
    CNA_StorageBufferHandle, *const c_void, u64, u64,
) -> CNA_Result;

// --- CNA engine layer: decals, skyboxes and atmospheric sky (engine_layer.h) ---
pub type cna_atmospheric_sky_copy_model_glsl_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_atmospheric_sky_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_AtmosphericSkyHandle,
) -> CNA_Result;
pub type cna_atmospheric_sky_destroy_fn = unsafe extern "C" fn(
    CNA_AtmosphericSkyHandle,
) -> CNA_Result;
pub type cna_atmospheric_sky_draw_fn = unsafe extern "C" fn(
    CNA_AtmosphericSkyHandle, *const CNA_Matrix, *const CNA_Matrix, i32, i32,
) -> CNA_Result;
pub type cna_atmospheric_sky_get_intensity_fn = unsafe extern "C" fn(
    CNA_AtmosphericSkyHandle, *mut f32,
) -> CNA_Result;
pub type cna_atmospheric_sky_get_sun_direction_fn = unsafe extern "C" fn(
    CNA_AtmosphericSkyHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_atmospheric_sky_get_turbidity_fn = unsafe extern "C" fn(
    CNA_AtmosphericSkyHandle, *mut f32,
) -> CNA_Result;
pub type cna_atmospheric_sky_is_supported_fn = unsafe extern "C" fn(
    CNA_AtmosphericSkyHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_atmospheric_sky_radiance_fn = unsafe extern "C" fn(
    *const CNA_Vector3, *const CNA_Vector3, f32, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_atmospheric_sky_set_intensity_fn = unsafe extern "C" fn(
    CNA_AtmosphericSkyHandle, f32,
) -> CNA_Result;
pub type cna_atmospheric_sky_set_sun_direction_fn = unsafe extern "C" fn(
    CNA_AtmosphericSkyHandle, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_atmospheric_sky_set_turbidity_fn = unsafe extern "C" fn(
    CNA_AtmosphericSkyHandle, f32,
) -> CNA_Result;
pub type cna_decal_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_DecalPassHandle,
) -> CNA_Result;
pub type cna_decal_pass_destroy_fn = unsafe extern "C" fn(CNA_DecalPassHandle) -> CNA_Result;
pub type cna_decal_pass_draw_fn = unsafe extern "C" fn(
    CNA_DecalPassHandle, CNA_Handle, *const CNA_Matrix, i32, i32,
) -> CNA_Result;
pub type cna_decal_pass_get_max_slope_angle_fn = unsafe extern "C" fn(
    CNA_DecalPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_decal_pass_get_opacity_fn = unsafe extern "C" fn(
    CNA_DecalPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_decal_pass_get_tint_fn = unsafe extern "C" fn(
    CNA_DecalPassHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_decal_pass_is_inside_decal_box_fn = unsafe extern "C" fn(
    *const CNA_Vector3, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_decal_pass_set_camera_fn = unsafe extern "C" fn(
    CNA_DecalPassHandle, *const CNA_Matrix, *const CNA_Matrix, f32,
) -> CNA_Result;
pub type cna_decal_pass_set_max_slope_angle_fn = unsafe extern "C" fn(
    CNA_DecalPassHandle, f32,
) -> CNA_Result;
pub type cna_decal_pass_set_opacity_fn = unsafe extern "C" fn(
    CNA_DecalPassHandle, f32,
) -> CNA_Result;
pub type cna_decal_pass_set_prepass_inputs_fn = unsafe extern "C" fn(
    CNA_DecalPassHandle, CNA_Handle, CNA_Handle,
) -> CNA_Result;
pub type cna_decal_pass_set_tint_fn = unsafe extern "C" fn(
    CNA_DecalPassHandle, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_skybox_compute_view_ray_fn = unsafe extern "C" fn(
    *const CNA_Matrix, *const CNA_Matrix, f32, f32, f32, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_skybox_create_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_Handle, *mut CNA_SkyboxHandle,
) -> CNA_Result;
pub type cna_skybox_destroy_fn = unsafe extern "C" fn(CNA_SkyboxHandle) -> CNA_Result;
pub type cna_skybox_draw_fn = unsafe extern "C" fn(
    CNA_SkyboxHandle, *const CNA_Matrix, *const CNA_Matrix, i32, i32,
) -> CNA_Result;
pub type cna_skybox_get_environment_fn = unsafe extern "C" fn(
    CNA_SkyboxHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_skybox_get_intensity_fn = unsafe extern "C" fn(
    CNA_SkyboxHandle, *mut f32,
) -> CNA_Result;
pub type cna_skybox_get_tint_fn = unsafe extern "C" fn(
    CNA_SkyboxHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_skybox_get_yaw_fn = unsafe extern "C" fn(CNA_SkyboxHandle, *mut f32) -> CNA_Result;
pub type cna_skybox_is_supported_fn = unsafe extern "C" fn(
    CNA_SkyboxHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_skybox_set_environment_fn = unsafe extern "C" fn(
    CNA_SkyboxHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_skybox_set_intensity_fn = unsafe extern "C" fn(CNA_SkyboxHandle, f32) -> CNA_Result;
pub type cna_skybox_set_owned_environment_fn = unsafe extern "C" fn(
    CNA_SkyboxHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_skybox_set_tint_fn = unsafe extern "C" fn(
    CNA_SkyboxHandle, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_skybox_set_yaw_fn = unsafe extern "C" fn(CNA_SkyboxHandle, f32) -> CNA_Result;

// --- CNA engine layer: the screen-space post-process passes (engine_layer.h) ---
pub type cna_aerial_perspective_pass_air_mass_for_distance_fn = unsafe extern "C" fn(
    *const CNA_Vector3, f32, f32, *mut f32,
) -> CNA_Result;
pub type cna_aerial_perspective_pass_copy_fallback_reason_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_aerial_perspective_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_aerial_perspective_pass_get_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_aerial_perspective_pass_get_scale_height_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_aerial_perspective_pass_get_sun_direction_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_aerial_perspective_pass_get_turbidity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_aerial_perspective_pass_set_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_aerial_perspective_pass_set_scale_height_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_aerial_perspective_pass_set_sun_direction_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_aerial_perspective_pass_set_turbidity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_aerial_perspective_pass_transmittance_fn = unsafe extern "C" fn(
    f32, f32, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_ascii_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_ascii_pass_get_effect_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut CNA_AsciiPostProcessEffectHandle,
) -> CNA_Result;
pub type cna_bloom_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_bloom_pass_extract_channel_fn = unsafe extern "C" fn(f32, f32, *mut f32) -> CNA_Result;
pub type cna_bloom_pass_get_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_bloom_pass_get_iterations_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut i32,
) -> CNA_Result;
pub type cna_bloom_pass_get_threshold_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_bloom_pass_iterations_for_quality_fn = unsafe extern "C" fn(
    CNA_RenderQuality, *mut i32,
) -> CNA_Result;
pub type cna_bloom_pass_reset_targets_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_bloom_pass_set_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_bloom_pass_set_iterations_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, i32,
) -> CNA_Result;
pub type cna_bloom_pass_set_threshold_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_chromatic_aberration_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_chromatic_aberration_pass_get_strength_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_chromatic_aberration_pass_set_strength_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_color_grade_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_color_grade_pass_create_identity_lut_fn = unsafe extern "C" fn(
    CNA_Handle, i32, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_color_grade_pass_get_interpolation_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut CNA_LutInterpolation,
) -> CNA_Result;
pub type cna_color_grade_pass_get_lut_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_color_grade_pass_get_strength_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_color_grade_pass_get_volume_lut_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_color_grade_pass_lut_size_for_strip_fn = unsafe extern "C" fn(
    i32, i32, *mut i32,
) -> CNA_Result;
pub type cna_color_grade_pass_set_interpolation_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, CNA_LutInterpolation,
) -> CNA_Result;
pub type cna_color_grade_pass_set_lut_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_color_grade_pass_set_strength_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_color_grade_pass_set_volume_lut_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_contact_shadow_pass_combine_visibility_fn = unsafe extern "C" fn(
    f32, f32, *mut f32,
) -> CNA_Result;
pub type cna_contact_shadow_pass_copy_fallback_reason_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_contact_shadow_pass_copy_occlusion_test_glsl_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_contact_shadow_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_contact_shadow_pass_get_bias_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_contact_shadow_pass_get_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_contact_shadow_pass_get_light_direction_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_contact_shadow_pass_get_max_distance_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_contact_shadow_pass_get_step_count_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut i32,
) -> CNA_Result;
pub type cna_contact_shadow_pass_get_thickness_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_contact_shadow_pass_is_occluded_fn = unsafe extern "C" fn(
    f32, f32, f32, f32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_contact_shadow_pass_set_bias_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_contact_shadow_pass_set_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_contact_shadow_pass_set_light_direction_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_contact_shadow_pass_set_max_distance_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_contact_shadow_pass_set_step_count_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, i32,
) -> CNA_Result;
pub type cna_contact_shadow_pass_set_thickness_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_depth_of_field_pass_circle_of_confusion_millimetres_fn = unsafe extern "C" fn(
    f32, f32, f32, f32, *mut f32,
) -> CNA_Result;
pub type cna_depth_of_field_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_depth_of_field_pass_get_f_number_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_depth_of_field_pass_get_focal_length_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_depth_of_field_pass_get_focus_distance_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_depth_of_field_pass_get_max_radius_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_depth_of_field_pass_set_f_number_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_depth_of_field_pass_set_focal_length_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_depth_of_field_pass_set_focus_distance_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_depth_of_field_pass_set_max_radius_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_film_grain_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_film_grain_pass_get_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_film_grain_pass_set_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_fullscreen_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_FullscreenPassHandle,
) -> CNA_Result;
pub type cna_fullscreen_pass_destroy_fn = unsafe extern "C" fn(
    CNA_FullscreenPassHandle,
) -> CNA_Result;
pub type cna_fullscreen_pass_draw_fn = unsafe extern "C" fn(
    CNA_FullscreenPassHandle, CNA_Handle, CNA_Handle, CNA_EffectHandle, i32, i32, *const CNA_SamplerState,
) -> CNA_Result;
pub type cna_fullscreen_pass_draw_over_current_target_fn = unsafe extern "C" fn(
    CNA_FullscreenPassHandle, CNA_Handle, CNA_EffectHandle, i32, i32, *const CNA_SamplerState,
) -> CNA_Result;
pub type cna_height_fog_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_height_fog_pass_get_base_height_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_height_fog_pass_get_color_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_height_fog_pass_get_density_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_height_fog_pass_get_falloff_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_height_fog_pass_optical_depth_fn = unsafe extern "C" fn(
    f32, f32, f32, f32, f32, f32, *mut f32,
) -> CNA_Result;
pub type cna_height_fog_pass_set_base_height_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_height_fog_pass_set_color_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_height_fog_pass_set_density_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_height_fog_pass_set_falloff_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_lens_flare_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_lens_flare_pass_get_dispersal_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_lens_flare_pass_get_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_lens_flare_pass_get_threshold_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_lens_flare_pass_set_dispersal_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_lens_flare_pass_set_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_lens_flare_pass_set_threshold_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_light_shaft_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_light_shaft_pass_get_decay_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_light_shaft_pass_get_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_light_shaft_pass_get_light_screen_position_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut CNA_Vector2,
) -> CNA_Result;
pub type cna_light_shaft_pass_get_threshold_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_light_shaft_pass_set_decay_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_light_shaft_pass_set_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_light_shaft_pass_set_light_screen_position_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *const CNA_Vector2,
) -> CNA_Result;
pub type cna_light_shaft_pass_set_threshold_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_motion_blur_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_motion_blur_pass_get_max_distance_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_motion_blur_pass_get_strength_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_motion_blur_pass_set_max_distance_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_motion_blur_pass_set_strength_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_scoped_render_target_begin_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_Handle, *mut CNA_ScopedRenderTargetHandle,
) -> CNA_Result;
pub type cna_scoped_render_target_end_fn = unsafe extern "C" fn(
    CNA_ScopedRenderTargetHandle,
) -> CNA_Result;
pub type cna_scoped_render_target_get_has_recorded_previous_fn = unsafe extern "C" fn(
    CNA_ScopedRenderTargetHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_spatial_upscale_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_SpatialUpscalePassHandle,
) -> CNA_Result;
pub type cna_spatial_upscale_pass_destroy_fn = unsafe extern "C" fn(
    CNA_SpatialUpscalePassHandle,
) -> CNA_Result;
pub type cna_spatial_upscale_pass_draw_fn = unsafe extern "C" fn(
    CNA_SpatialUpscalePassHandle, CNA_Handle, i32, i32, i32, i32,
) -> CNA_Result;
pub type cna_spatial_upscale_pass_get_edge_adaptive_fn = unsafe extern "C" fn(
    CNA_SpatialUpscalePassHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_spatial_upscale_pass_get_sharpness_fn = unsafe extern "C" fn(
    CNA_SpatialUpscalePassHandle, *mut f32,
) -> CNA_Result;
pub type cna_spatial_upscale_pass_is_identity_scale_fn = unsafe extern "C" fn(
    i32, i32, i32, i32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_spatial_upscale_pass_set_edge_adaptive_fn = unsafe extern "C" fn(
    CNA_SpatialUpscalePassHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_spatial_upscale_pass_set_sharpness_fn = unsafe extern "C" fn(
    CNA_SpatialUpscalePassHandle, f32,
) -> CNA_Result;
pub type cna_ssao_pass_copy_kernel_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut CNA_Vector3, u64, *mut u64,
) -> CNA_Result;
pub type cna_ssao_pass_copy_occlusion_glsl_fn = unsafe extern "C" fn(
    CNA_Bool, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_ssao_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_ssao_pass_get_half_resolution_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_ssao_pass_get_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_ssao_pass_get_radius_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_ssao_pass_get_sample_count_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut i32,
) -> CNA_Result;
pub type cna_ssao_pass_reset_targets_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_ssao_pass_sample_count_for_quality_fn = unsafe extern "C" fn(
    CNA_RenderQuality, *mut i32,
) -> CNA_Result;
pub type cna_ssao_pass_set_half_resolution_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_ssao_pass_set_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_ssao_pass_set_radius_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_ssao_pass_set_sample_count_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, i32,
) -> CNA_Result;
pub type cna_ssr_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_ssr_pass_get_depth_bias_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_ssr_pass_get_edge_fade_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_ssr_pass_get_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_ssr_pass_get_max_distance_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_ssr_pass_get_roughness_blur_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_ssr_pass_get_step_count_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut i32,
) -> CNA_Result;
pub type cna_ssr_pass_get_thickness_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_ssr_pass_set_depth_bias_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_ssr_pass_set_edge_fade_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_ssr_pass_set_intensity_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_ssr_pass_set_max_distance_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_ssr_pass_set_roughness_blur_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_ssr_pass_set_step_count_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, i32,
) -> CNA_Result;
pub type cna_ssr_pass_set_thickness_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_volumetric_fog_pass_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_PostProcessPassHandle,
) -> CNA_Result;
pub type cna_volumetric_fog_pass_get_anisotropy_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_volumetric_fog_pass_get_density_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_volumetric_fog_pass_get_range_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, *mut f32,
) -> CNA_Result;
pub type cna_volumetric_fog_pass_set_anisotropy_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_volumetric_fog_pass_set_density_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;
pub type cna_volumetric_fog_pass_set_light_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, CNA_ShadowMapHandle, *const CNA_Vector3, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_volumetric_fog_pass_set_range_fn = unsafe extern "C" fn(
    CNA_PostProcessPassHandle, f32,
) -> CNA_Result;

// --- CNA engine layer: spot, cube and cascaded shadow maps (engine_layer.h) ---
pub type cna_cascaded_shadow_map_apply_to_receiver_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, CNA_EffectHandle,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_begin_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, i32,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_compute_bounding_sphere_fn = unsafe extern "C" fn(
    *const CNA_Vector3, *mut CNA_Vector3, *mut f32,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_compute_frustum_corners_fn = unsafe extern "C" fn(
    *const CNA_Matrix, *const CNA_Matrix, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_compute_split_distances_fn = unsafe extern "C" fn(
    f32, f32, i32, f32, *mut f32, u64, *mut u64,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_create_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_ShadowQuality, i32, *mut CNA_CascadedShadowMapHandle,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_destroy_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_end_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_get_blend_band_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, *mut f32,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_get_cascade_count_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, *mut i32,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_get_cascade_matrix_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, i32, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_get_cascade_size_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, *mut i32,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_get_caster_effect_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_get_shadow_texture_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_get_split_distance_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, i32, *mut f32,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_get_split_lambda_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, *mut f32,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_is_debug_tint_enabled_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_is_supported_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_select_cascade_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, f32, *mut i32,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_set_blend_band_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, f32,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_set_debug_tint_enabled_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_set_split_lambda_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, f32,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_snap_to_texel_grid_fn = unsafe extern "C" fn(
    *const CNA_Vector3, f32, i32, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_cascaded_shadow_map_update_fn = unsafe extern "C" fn(
    CNA_CascadedShadowMapHandle, *const CNA_DirectionalLightEXT, *const CNA_Matrix, *const CNA_Matrix,
) -> CNA_Result;
pub type cna_cube_shadow_map_begin_fn = unsafe extern "C" fn(
    CNA_CubeShadowMapHandle, i32,
) -> CNA_Result;
pub type cna_cube_shadow_map_compute_face_projection_fn = unsafe extern "C" fn(
    f32, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_cube_shadow_map_compute_face_view_fn = unsafe extern "C" fn(
    CNA_CubeMapFace, *const CNA_Vector3, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_cube_shadow_map_create_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_ShadowQuality, *mut CNA_CubeShadowMapHandle,
) -> CNA_Result;
pub type cna_cube_shadow_map_destroy_fn = unsafe extern "C" fn(
    CNA_CubeShadowMapHandle,
) -> CNA_Result;
pub type cna_cube_shadow_map_end_fn = unsafe extern "C" fn(CNA_CubeShadowMapHandle) -> CNA_Result;
pub type cna_cube_shadow_map_get_caster_effect_fn = unsafe extern "C" fn(
    CNA_CubeShadowMapHandle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_cube_shadow_map_get_depth_bias_fn = unsafe extern "C" fn(
    CNA_CubeShadowMapHandle, *mut f32,
) -> CNA_Result;
pub type cna_cube_shadow_map_get_light_position_fn = unsafe extern "C" fn(
    CNA_CubeShadowMapHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_cube_shadow_map_get_light_range_fn = unsafe extern "C" fn(
    CNA_CubeShadowMapHandle, *mut f32,
) -> CNA_Result;
pub type cna_cube_shadow_map_get_quality_fn = unsafe extern "C" fn(
    CNA_CubeShadowMapHandle, *mut CNA_ShadowQuality,
) -> CNA_Result;
pub type cna_cube_shadow_map_get_shadow_texture_fn = unsafe extern "C" fn(
    CNA_CubeShadowMapHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_cube_shadow_map_get_size_fn = unsafe extern "C" fn(
    CNA_CubeShadowMapHandle, *mut i32,
) -> CNA_Result;
pub type cna_cube_shadow_map_is_supported_fn = unsafe extern "C" fn(
    CNA_CubeShadowMapHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_cube_shadow_map_set_depth_bias_fn = unsafe extern "C" fn(
    CNA_CubeShadowMapHandle, f32,
) -> CNA_Result;
pub type cna_cube_shadow_map_size_for_quality_fn = unsafe extern "C" fn(
    CNA_ShadowQuality, *mut i32,
) -> CNA_Result;
pub type cna_cube_shadow_map_update_fn = unsafe extern "C" fn(
    CNA_CubeShadowMapHandle, *const CNA_PointLightEXT,
) -> CNA_Result;
pub type cna_shadow_cascade_state_ext_init_fn = unsafe extern "C" fn(
    *mut CNA_ShadowCascadeStateEXT,
) -> CNA_Result;
pub type cna_spot_shadow_map_begin_fn = unsafe extern "C" fn(
    CNA_SpotShadowMapHandle, *const CNA_SpotLightEXT,
) -> CNA_Result;
pub type cna_spot_shadow_map_compute_light_projection_fn = unsafe extern "C" fn(
    *const CNA_SpotLightEXT, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_spot_shadow_map_compute_light_view_fn = unsafe extern "C" fn(
    *const CNA_SpotLightEXT, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_spot_shadow_map_create_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_ShadowQuality, *mut CNA_SpotShadowMapHandle,
) -> CNA_Result;
pub type cna_spot_shadow_map_destroy_fn = unsafe extern "C" fn(
    CNA_SpotShadowMapHandle,
) -> CNA_Result;
pub type cna_spot_shadow_map_end_fn = unsafe extern "C" fn(CNA_SpotShadowMapHandle) -> CNA_Result;
pub type cna_spot_shadow_map_get_caster_effect_fn = unsafe extern "C" fn(
    CNA_SpotShadowMapHandle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_spot_shadow_map_get_depth_bias_fn = unsafe extern "C" fn(
    CNA_SpotShadowMapHandle, *mut f32,
) -> CNA_Result;
pub type cna_spot_shadow_map_get_light_position_fn = unsafe extern "C" fn(
    CNA_SpotShadowMapHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_spot_shadow_map_get_light_range_fn = unsafe extern "C" fn(
    CNA_SpotShadowMapHandle, *mut f32,
) -> CNA_Result;
pub type cna_spot_shadow_map_get_light_view_projection_fn = unsafe extern "C" fn(
    CNA_SpotShadowMapHandle, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_spot_shadow_map_get_quality_fn = unsafe extern "C" fn(
    CNA_SpotShadowMapHandle, *mut CNA_ShadowQuality,
) -> CNA_Result;
pub type cna_spot_shadow_map_get_shadow_texture_fn = unsafe extern "C" fn(
    CNA_SpotShadowMapHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_spot_shadow_map_get_size_fn = unsafe extern "C" fn(
    CNA_SpotShadowMapHandle, *mut i32,
) -> CNA_Result;
pub type cna_spot_shadow_map_is_supported_fn = unsafe extern "C" fn(
    CNA_SpotShadowMapHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_spot_shadow_map_set_depth_bias_fn = unsafe extern "C" fn(
    CNA_SpotShadowMapHandle, f32,
) -> CNA_Result;
pub type cna_point_light_ext_init_fn = unsafe extern "C" fn(*mut CNA_PointLightEXT) -> CNA_Result;
pub type cna_spot_light_ext_init_fn = unsafe extern "C" fn(*mut CNA_SpotLightEXT) -> CNA_Result;
pub type cna_punctual_light_ext_init_fn = unsafe extern "C" fn(
    *mut CNA_PunctualLightEXT,
) -> CNA_Result;

// --- CNA engine layer: the depth/normal prepass and transparency (engine_layer.h) ---
pub type cna_depth_normal_prepass_begin_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, i32, *const CNA_Matrix, *const CNA_Matrix, f32, f32,
) -> CNA_Result;
pub type cna_depth_normal_prepass_copy_depth_decode_glsl_fn = unsafe extern "C" fn(
    CNA_Bool, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_depth_normal_prepass_copy_velocity_decode_glsl_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_depth_normal_prepass_create_fn = unsafe extern "C" fn(
    CNA_Handle, i32, i32, CNA_DepthEncoding, *mut CNA_DepthNormalPrepassHandle,
) -> CNA_Result;
pub type cna_depth_normal_prepass_decode_velocity_ext_fn = unsafe extern "C" fn(
    CNA_Color, *mut CNA_Vector2,
) -> CNA_Result;
pub type cna_depth_normal_prepass_destroy_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle,
) -> CNA_Result;
pub type cna_depth_normal_prepass_end_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle,
) -> CNA_Result;
pub type cna_depth_normal_prepass_get_depth_texture_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_depth_normal_prepass_get_normal_texture_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_depth_normal_prepass_get_pass_count_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, *mut i32,
) -> CNA_Result;
pub type cna_depth_normal_prepass_get_prepass_effect_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_depth_normal_prepass_get_roughness_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, *mut f32,
) -> CNA_Result;
pub type cna_depth_normal_prepass_get_skinned_prepass_effect_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_depth_normal_prepass_get_velocity_texture_ext_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_depth_normal_prepass_has_velocity_ext_fn = unsafe extern "C" fn(
    CNA_Color, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_depth_normal_prepass_is_depth_packed_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_depth_normal_prepass_is_supported_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_depth_normal_prepass_is_using_multiple_render_targets_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_depth_normal_prepass_is_velocity_enabled_ext_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_depth_normal_prepass_pack_depth_fn = unsafe extern "C" fn(
    f32, *mut f32, *mut f32, *mut f32, *mut f32,
) -> CNA_Result;
pub type cna_depth_normal_prepass_resize_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, i32, i32,
) -> CNA_Result;
pub type cna_depth_normal_prepass_set_previous_camera_ext_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, *const CNA_Matrix, *const CNA_Matrix,
) -> CNA_Result;
pub type cna_depth_normal_prepass_set_previous_world_ext_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, *const CNA_Matrix,
) -> CNA_Result;
pub type cna_depth_normal_prepass_set_roughness_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, f32,
) -> CNA_Result;
pub type cna_depth_normal_prepass_set_velocity_enabled_ext_fn = unsafe extern "C" fn(
    CNA_DepthNormalPrepassHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_depth_normal_prepass_unpack_depth_fn = unsafe extern "C" fn(
    f32, f32, f32, f32, *mut f32,
) -> CNA_Result;
pub type cna_depth_normal_prepass_uses_packed_depth_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_transparent_draw_list_camera_position_of_fn = unsafe extern "C" fn(
    *const CNA_Matrix, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_transparent_draw_list_clear_fn = unsafe extern "C" fn(
    CNA_TransparentDrawListHandle,
) -> CNA_Result;
pub type cna_transparent_draw_list_copy_sorted_order_ext_fn = unsafe extern "C" fn(
    CNA_TransparentDrawListHandle, *const CNA_Matrix, *mut i32, u64, *mut u64,
) -> CNA_Result;
pub type cna_transparent_draw_list_create_fn = unsafe extern "C" fn(
    *mut CNA_TransparentDrawListHandle,
) -> CNA_Result;
pub type cna_transparent_draw_list_destroy_fn = unsafe extern "C" fn(
    CNA_TransparentDrawListHandle,
) -> CNA_Result;
pub type cna_transparent_draw_list_draw_sorted_fn = unsafe extern "C" fn(
    CNA_TransparentDrawListHandle, *const CNA_Matrix,
) -> CNA_Result;
pub type cna_transparent_draw_list_get_count_fn = unsafe extern "C" fn(
    CNA_TransparentDrawListHandle, *mut u64,
) -> CNA_Result;
pub type cna_transparent_draw_list_sort_key_fn = unsafe extern "C" fn(
    *const CNA_BoundingBox, *const CNA_Vector3, *mut f32,
) -> CNA_Result;
pub type cna_transparent_draw_list_submit_fn = unsafe extern "C" fn(
    CNA_TransparentDrawListHandle, *const CNA_BoundingBox, CNA_TransparentDrawCallback, *mut c_void,
) -> CNA_Result;
pub type cna_weighted_blended_transparency_begin_fn = unsafe extern "C" fn(
    CNA_WeightedBlendedTransparencyHandle, f32,
) -> CNA_Result;
pub type cna_weighted_blended_transparency_copy_accumulation_glsl_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_weighted_blended_transparency_copy_unsupported_reason_fn = unsafe extern "C" fn(
    CNA_WeightedBlendedTransparencyHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_weighted_blended_transparency_create_fn = unsafe extern "C" fn(
    CNA_Handle, i32, i32, *mut CNA_WeightedBlendedTransparencyHandle,
) -> CNA_Result;
pub type cna_weighted_blended_transparency_destroy_fn = unsafe extern "C" fn(
    CNA_WeightedBlendedTransparencyHandle,
) -> CNA_Result;
pub type cna_weighted_blended_transparency_end_fn = unsafe extern "C" fn(
    CNA_WeightedBlendedTransparencyHandle,
) -> CNA_Result;
pub type cna_weighted_blended_transparency_get_accumulation_texture_ext_fn = unsafe extern "C" fn(
    CNA_WeightedBlendedTransparencyHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_weighted_blended_transparency_get_revealage_texture_ext_fn = unsafe extern "C" fn(
    CNA_WeightedBlendedTransparencyHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_weighted_blended_transparency_is_accumulating_fn = unsafe extern "C" fn(
    CNA_WeightedBlendedTransparencyHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_weighted_blended_transparency_is_supported_fn = unsafe extern "C" fn(
    CNA_WeightedBlendedTransparencyHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_weighted_blended_transparency_resize_fn = unsafe extern "C" fn(
    CNA_WeightedBlendedTransparencyHandle, i32, i32,
) -> CNA_Result;
pub type cna_weighted_blended_transparency_resolve_fn = unsafe extern "C" fn(
    CNA_WeightedBlendedTransparencyHandle, i32, i32,
) -> CNA_Result;
pub type cna_weighted_blended_transparency_weight_fn = unsafe extern "C" fn(
    f32, f32, f32, *mut f32,
) -> CNA_Result;

// --- CNA engine layer: HDR output, auto exposure and cube LUTs (engine_layer.h) ---
pub type cna_auto_exposure_ext_apply_to_fn = unsafe extern "C" fn(
    CNA_AutoExposureHandle, *mut CNA_RenderPipelineSettingsEXT,
) -> CNA_Result;
pub type cna_auto_exposure_ext_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_AutoExposureHandle,
) -> CNA_Result;
pub type cna_auto_exposure_ext_destroy_fn = unsafe extern "C" fn(
    CNA_AutoExposureHandle,
) -> CNA_Result;
pub type cna_auto_exposure_ext_get_brightening_speed_fn = unsafe extern "C" fn(
    CNA_AutoExposureHandle, *mut f32,
) -> CNA_Result;
pub type cna_auto_exposure_ext_get_darkening_speed_fn = unsafe extern "C" fn(
    CNA_AutoExposureHandle, *mut f32,
) -> CNA_Result;
pub type cna_auto_exposure_ext_get_exposure_fn = unsafe extern "C" fn(
    CNA_AutoExposureHandle, *mut f32,
) -> CNA_Result;
pub type cna_auto_exposure_ext_get_key_value_fn = unsafe extern "C" fn(
    CNA_AutoExposureHandle, *mut f32,
) -> CNA_Result;
pub type cna_auto_exposure_ext_measure_average_luminance_fn = unsafe extern "C" fn(
    CNA_AutoExposureHandle, CNA_Handle, *mut f32,
) -> CNA_Result;
pub type cna_auto_exposure_ext_set_adaptation_speeds_fn = unsafe extern "C" fn(
    CNA_AutoExposureHandle, f32, f32,
) -> CNA_Result;
pub type cna_auto_exposure_ext_set_exposure_fn = unsafe extern "C" fn(
    CNA_AutoExposureHandle, f32,
) -> CNA_Result;
pub type cna_auto_exposure_ext_set_exposure_range_fn = unsafe extern "C" fn(
    CNA_AutoExposureHandle, f32, f32,
) -> CNA_Result;
pub type cna_auto_exposure_ext_set_key_value_fn = unsafe extern "C" fn(
    CNA_AutoExposureHandle, f32,
) -> CNA_Result;
pub type cna_auto_exposure_ext_update_fn = unsafe extern "C" fn(
    CNA_AutoExposureHandle, CNA_Handle, f32, *mut f32,
) -> CNA_Result;
pub type cna_cube_lut_copy_title_fn = unsafe extern "C" fn(
    CNA_CubeLutHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cube_lut_create_strip_texture_fn = unsafe extern "C" fn(
    CNA_CubeLutHandle, CNA_Handle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_cube_lut_create_volume_texture_fn = unsafe extern "C" fn(
    CNA_CubeLutHandle, CNA_Handle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_cube_lut_destroy_fn = unsafe extern "C" fn(CNA_CubeLutHandle) -> CNA_Result;
pub type cna_cube_lut_get_domain_max_fn = unsafe extern "C" fn(
    CNA_CubeLutHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_cube_lut_get_domain_min_fn = unsafe extern "C" fn(
    CNA_CubeLutHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_cube_lut_get_entry_fn = unsafe extern "C" fn(
    CNA_CubeLutHandle, i32, i32, i32, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_cube_lut_get_size_fn = unsafe extern "C" fn(CNA_CubeLutHandle, *mut i32) -> CNA_Result;
pub type cna_cube_lut_is_unit_domain_fn = unsafe extern "C" fn(
    CNA_CubeLutHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_cube_lut_load_from_file_fn = unsafe extern "C" fn(
    CNA_StringView, *mut CNA_CubeLutHandle,
) -> CNA_Result;
pub type cna_cube_lut_parse_fn = unsafe extern "C" fn(
    CNA_StringView, *mut CNA_CubeLutHandle,
) -> CNA_Result;
pub type cna_hdr_display_output_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_HdrDisplayOutputHandle,
) -> CNA_Result;
pub type cna_hdr_display_output_decode_pq_fn = unsafe extern "C" fn(f32, *mut f32) -> CNA_Result;
pub type cna_hdr_display_output_destroy_fn = unsafe extern "C" fn(
    CNA_HdrDisplayOutputHandle,
) -> CNA_Result;
pub type cna_hdr_display_output_draw_fn = unsafe extern "C" fn(
    CNA_HdrDisplayOutputHandle, CNA_Handle, CNA_Handle, i32, i32,
) -> CNA_Result;
pub type cna_hdr_display_output_encode_fn = unsafe extern "C" fn(
    CNA_DisplayColorSpace, *const CNA_Vector3, f32, f32, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_hdr_display_output_encode_pq_fn = unsafe extern "C" fn(f32, *mut f32) -> CNA_Result;
pub type cna_hdr_display_output_get_color_space_fn = unsafe extern "C" fn(
    CNA_HdrDisplayOutputHandle, *mut CNA_DisplayColorSpace,
) -> CNA_Result;
pub type cna_hdr_display_output_get_paper_white_nits_fn = unsafe extern "C" fn(
    CNA_HdrDisplayOutputHandle, *mut f32,
) -> CNA_Result;
pub type cna_hdr_display_output_get_peak_nits_fn = unsafe extern "C" fn(
    CNA_HdrDisplayOutputHandle, *mut f32,
) -> CNA_Result;
pub type cna_hdr_display_output_is_supported_fn = unsafe extern "C" fn(
    CNA_HdrDisplayOutputHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_hdr_display_output_rec709_to_rec2020_fn = unsafe extern "C" fn(
    *const CNA_Vector3, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_hdr_display_output_roll_off_fn = unsafe extern "C" fn(
    f32, f32, *mut f32,
) -> CNA_Result;
pub type cna_hdr_display_output_set_color_space_fn = unsafe extern "C" fn(
    CNA_HdrDisplayOutputHandle, CNA_DisplayColorSpace,
) -> CNA_Result;
pub type cna_hdr_display_output_set_paper_white_nits_fn = unsafe extern "C" fn(
    CNA_HdrDisplayOutputHandle, f32,
) -> CNA_Result;
pub type cna_hdr_display_output_set_peak_nits_fn = unsafe extern "C" fn(
    CNA_HdrDisplayOutputHandle, f32,
) -> CNA_Result;

// --- CNA engine layer: debug drawing, frustum culling and LOD groups (engine_layer.h) ---
pub type cna_debug_draw_add_bounding_sphere_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, *const CNA_BoundingSphere, CNA_Color, i32,
) -> CNA_Result;
pub type cna_debug_draw_add_box_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, *const CNA_BoundingBox, CNA_Color,
) -> CNA_Result;
pub type cna_debug_draw_add_cascade_gizmo_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, CNA_CascadedShadowMapHandle, CNA_Color,
) -> CNA_Result;
pub type cna_debug_draw_add_cross_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, *const CNA_Vector3, f32, CNA_Color,
) -> CNA_Result;
pub type cna_debug_draw_add_directional_light_gizmo_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, *const CNA_DirectionalLightEXT, *const CNA_Vector3, f32, CNA_Color,
) -> CNA_Result;
pub type cna_debug_draw_add_frustum_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, CNA_BoundingFrustum, CNA_Color,
) -> CNA_Result;
pub type cna_debug_draw_add_line_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, *const CNA_Vector3, *const CNA_Vector3, CNA_Color,
) -> CNA_Result;
pub type cna_debug_draw_add_point_light_gizmo_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, *const CNA_PointLightEXT, CNA_Color,
) -> CNA_Result;
pub type cna_debug_draw_add_sphere_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, *const CNA_Vector3, f32, CNA_Color, i32,
) -> CNA_Result;
pub type cna_debug_draw_add_spot_light_gizmo_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, *const CNA_SpotLightEXT, CNA_Color, i32,
) -> CNA_Result;
pub type cna_debug_draw_begin_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, *const CNA_Matrix, *const CNA_Matrix,
) -> CNA_Result;
pub type cna_debug_draw_clear_fn = unsafe extern "C" fn(CNA_DebugDrawHandle) -> CNA_Result;
pub type cna_debug_draw_copy_vertices_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, CNA_Bool, *mut CNA_VertexPositionColor, u64, *mut u64,
) -> CNA_Result;
pub type cna_debug_draw_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_DebugDrawHandle,
) -> CNA_Result;
pub type cna_debug_draw_destroy_fn = unsafe extern "C" fn(CNA_DebugDrawHandle) -> CNA_Result;
pub type cna_debug_draw_end_fn = unsafe extern "C" fn(CNA_DebugDrawHandle) -> CNA_Result;
pub type cna_debug_draw_get_line_count_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, *mut i32,
) -> CNA_Result;
pub type cna_debug_draw_is_depth_tested_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_debug_draw_set_depth_tested_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_frustum_culler_ext_create_fn = unsafe extern "C" fn(
    *mut CNA_FrustumCullerEXTHandle,
) -> CNA_Result;
pub type cna_frustum_culler_ext_cull_boxes_fn = unsafe extern "C" fn(
    CNA_FrustumCullerEXTHandle, *const CNA_BoundingBox, u64, *mut u64, u64, *mut u64,
) -> CNA_Result;
pub type cna_frustum_culler_ext_cull_spheres_fn = unsafe extern "C" fn(
    CNA_FrustumCullerEXTHandle, *const CNA_BoundingSphere, u64, *mut u64, u64, *mut u64,
) -> CNA_Result;
pub type cna_frustum_culler_ext_cull_transforms_fn = unsafe extern "C" fn(
    CNA_FrustumCullerEXTHandle, *const CNA_Matrix, u64, *const CNA_BoundingBox, u64, *mut CNA_Matrix, u64, *mut u64,
) -> CNA_Result;
pub type cna_frustum_culler_ext_destroy_fn = unsafe extern "C" fn(
    CNA_FrustumCullerEXTHandle,
) -> CNA_Result;
pub type cna_frustum_culler_ext_get_frustum_fn = unsafe extern "C" fn(
    CNA_FrustumCullerEXTHandle, *mut CNA_BoundingFrustum,
) -> CNA_Result;
pub type cna_frustum_culler_ext_is_box_visible_fn = unsafe extern "C" fn(
    CNA_FrustumCullerEXTHandle, *const CNA_BoundingBox, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_frustum_culler_ext_is_sphere_visible_fn = unsafe extern "C" fn(
    CNA_FrustumCullerEXTHandle, *const CNA_BoundingSphere, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_frustum_culler_ext_set_camera_fn = unsafe extern "C" fn(
    CNA_FrustumCullerEXTHandle, *const CNA_Matrix, *const CNA_Matrix,
) -> CNA_Result;
pub type cna_frustum_culler_ext_set_view_projection_fn = unsafe extern "C" fn(
    CNA_FrustumCullerEXTHandle, *const CNA_Matrix,
) -> CNA_Result;
pub type cna_lod_group_ext_add_level_fn = unsafe extern "C" fn(
    CNA_LodGroupEXTHandle, f32, CNA_ModelMeshPartHandle,
) -> CNA_Result;
pub type cna_lod_group_ext_clear_fn = unsafe extern "C" fn(CNA_LodGroupEXTHandle) -> CNA_Result;
pub type cna_lod_group_ext_copy_levels_fn = unsafe extern "C" fn(
    CNA_LodGroupEXTHandle, *mut CNA_LodLevelEXT, u64, *mut u64,
) -> CNA_Result;
pub type cna_lod_group_ext_create_fn = unsafe extern "C" fn(
    *mut CNA_LodGroupEXTHandle,
) -> CNA_Result;
pub type cna_lod_group_ext_destroy_fn = unsafe extern "C" fn(CNA_LodGroupEXTHandle) -> CNA_Result;
pub type cna_lod_group_ext_get_hysteresis_fn = unsafe extern "C" fn(
    CNA_LodGroupEXTHandle, *mut f32,
) -> CNA_Result;
pub type cna_lod_group_ext_get_selection_mode_fn = unsafe extern "C" fn(
    CNA_LodGroupEXTHandle, *mut CNA_LodSelectionMode,
) -> CNA_Result;
pub type cna_lod_group_ext_projected_radius_pixels_fn = unsafe extern "C" fn(
    CNA_LodGroupEXTHandle, f32, *mut f32,
) -> CNA_Result;
pub type cna_lod_group_ext_reset_hysteresis_fn = unsafe extern "C" fn(
    CNA_LodGroupEXTHandle,
) -> CNA_Result;
pub type cna_lod_group_ext_select_fn = unsafe extern "C" fn(
    CNA_LodGroupEXTHandle, f32, *mut CNA_ModelMeshPartHandle,
) -> CNA_Result;
pub type cna_lod_group_ext_select_index_fn = unsafe extern "C" fn(
    CNA_LodGroupEXTHandle, f32, *mut i32,
) -> CNA_Result;
pub type cna_lod_group_ext_set_hysteresis_fn = unsafe extern "C" fn(
    CNA_LodGroupEXTHandle, f32,
) -> CNA_Result;
pub type cna_lod_group_ext_set_screen_space_parameters_fn = unsafe extern "C" fn(
    CNA_LodGroupEXTHandle, f32, f32, f32,
) -> CNA_Result;
pub type cna_lod_group_ext_set_selection_mode_fn = unsafe extern "C" fn(
    CNA_LodGroupEXTHandle, CNA_LodSelectionMode,
) -> CNA_Result;
pub type cna_clustered_light_assignment_adopt_fn = unsafe extern "C" fn(
    CNA_ClusteredLightAssignmentHandle, i32, *const i32, u64, *const i32, u64,
) -> CNA_Result;
pub type cna_clustered_light_assignment_assign_fn = unsafe extern "C" fn(
    CNA_ClusteredLightAssignmentHandle, CNA_ClusteredLightGridHandle, *const CNA_Matrix, *const CNA_BoundingSphere, u64,
) -> CNA_Result;
pub type cna_clustered_light_assignment_clear_fn = unsafe extern "C" fn(
    CNA_ClusteredLightAssignmentHandle,
) -> CNA_Result;
pub type cna_clustered_light_assignment_copy_indices_fn = unsafe extern "C" fn(
    CNA_ClusteredLightAssignmentHandle, *mut i32, u64, *mut u64,
) -> CNA_Result;
pub type cna_clustered_light_assignment_copy_lights_in_cluster_fn = unsafe extern "C" fn(
    CNA_ClusteredLightAssignmentHandle, i32, *mut i32, u64, *mut u64,
) -> CNA_Result;
pub type cna_clustered_light_assignment_copy_offsets_fn = unsafe extern "C" fn(
    CNA_ClusteredLightAssignmentHandle, *mut i32, u64, *mut u64,
) -> CNA_Result;
pub type cna_clustered_light_assignment_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_ClusteredLightAssignmentHandle,
) -> CNA_Result;
pub type cna_clustered_light_assignment_destroy_fn = unsafe extern "C" fn(
    CNA_ClusteredLightAssignmentHandle,
) -> CNA_Result;
pub type cna_clustered_light_assignment_get_cluster_count_fn = unsafe extern "C" fn(
    CNA_ClusteredLightAssignmentHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_assignment_get_light_count_fn = unsafe extern "C" fn(
    CNA_ClusteredLightAssignmentHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_assignment_get_max_lights_per_cluster_fn = unsafe extern "C" fn(
    CNA_ClusteredLightAssignmentHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_assignment_get_total_reference_count_fn = unsafe extern "C" fn(
    CNA_ClusteredLightAssignmentHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_ext_init_fn = unsafe extern "C" fn(
    *mut CNA_ClusteredLightEXT,
) -> CNA_Result;
pub type cna_clustered_light_grid_cluster_bounds_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle, i32, i32, i32, *mut CNA_BoundingBox,
) -> CNA_Result;
pub type cna_clustered_light_grid_cluster_index_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle, i32, i32, i32, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_grid_create_fn = unsafe extern "C" fn(
    CNA_Handle, i32, i32, i32, *mut CNA_ClusteredLightGridHandle,
) -> CNA_Result;
pub type cna_clustered_light_grid_destroy_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle,
) -> CNA_Result;
pub type cna_clustered_light_grid_get_cluster_count_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_grid_get_far_plane_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle, *mut f32,
) -> CNA_Result;
pub type cna_clustered_light_grid_get_inverse_projection_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_clustered_light_grid_get_near_plane_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle, *mut f32,
) -> CNA_Result;
pub type cna_clustered_light_grid_get_slice_count_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_grid_get_tiles_x_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_grid_get_tiles_y_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_grid_has_projection_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_clustered_light_grid_set_projection_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle, *const CNA_Matrix, f32, f32,
) -> CNA_Result;
pub type cna_clustered_light_grid_slice_distance_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle, i32, *mut f32,
) -> CNA_Result;
pub type cna_clustered_light_grid_slice_for_view_distance_fn = unsafe extern "C" fn(
    CNA_ClusteredLightGridHandle, f32, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_set_add_fn = unsafe extern "C" fn(
    CNA_ClusteredLightSetHandle, *const CNA_ClusteredLightEXT, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_set_add_point_fn = unsafe extern "C" fn(
    CNA_ClusteredLightSetHandle, *const CNA_PointLightEXT, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_set_add_spot_fn = unsafe extern "C" fn(
    CNA_ClusteredLightSetHandle, *const CNA_SpotLightEXT, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_set_clear_fn = unsafe extern "C" fn(
    CNA_ClusteredLightSetHandle,
) -> CNA_Result;
pub type cna_clustered_light_set_copy_bounds_fn = unsafe extern "C" fn(
    CNA_ClusteredLightSetHandle, *mut CNA_BoundingSphere, u64, *mut u64,
) -> CNA_Result;
pub type cna_clustered_light_set_copy_lights_fn = unsafe extern "C" fn(
    CNA_ClusteredLightSetHandle, *mut CNA_ClusteredLightEXT, u64, *mut u64,
) -> CNA_Result;
pub type cna_clustered_light_set_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_ClusteredLightSetHandle,
) -> CNA_Result;
pub type cna_clustered_light_set_destroy_fn = unsafe extern "C" fn(
    CNA_ClusteredLightSetHandle,
) -> CNA_Result;
pub type cna_clustered_light_set_get_at_fn = unsafe extern "C" fn(
    CNA_ClusteredLightSetHandle, i32, *mut CNA_ClusteredLightEXT,
) -> CNA_Result;
pub type cna_clustered_light_set_get_bounds_at_fn = unsafe extern "C" fn(
    CNA_ClusteredLightSetHandle, i32, *mut CNA_BoundingSphere,
) -> CNA_Result;
pub type cna_clustered_light_set_get_count_fn = unsafe extern "C" fn(
    CNA_ClusteredLightSetHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_set_is_empty_fn = unsafe extern "C" fn(
    CNA_ClusteredLightSetHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_clustered_light_set_is_usable_fn = unsafe extern "C" fn(
    *const CNA_ClusteredLightEXT, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_clustered_light_set_remove_at_fn = unsafe extern "C" fn(
    CNA_ClusteredLightSetHandle, i32,
) -> CNA_Result;
pub type cna_clustered_light_set_replace_at_fn = unsafe extern "C" fn(
    CNA_ClusteredLightSetHandle, i32, *const CNA_ClusteredLightEXT,
) -> CNA_Result;
pub type cna_clustered_light_buffer_bind_fn = unsafe extern "C" fn(
    CNA_ClusteredLightBufferHandle, CNA_EffectHandle, i32,
) -> CNA_Result;
pub type cna_clustered_light_buffer_copy_light_lookup_glsl_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_clustered_light_buffer_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_ClusteredLightBufferHandle,
) -> CNA_Result;
pub type cna_clustered_light_buffer_destroy_fn = unsafe extern "C" fn(
    CNA_ClusteredLightBufferHandle,
) -> CNA_Result;
pub type cna_clustered_light_buffer_get_cluster_count_fn = unsafe extern "C" fn(
    CNA_ClusteredLightBufferHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_buffer_get_light_count_fn = unsafe extern "C" fn(
    CNA_ClusteredLightBufferHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_buffer_get_reference_count_fn = unsafe extern "C" fn(
    CNA_ClusteredLightBufferHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_buffer_is_uploaded_fn = unsafe extern "C" fn(
    CNA_ClusteredLightBufferHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_clustered_light_buffer_upload_fn = unsafe extern "C" fn(
    CNA_ClusteredLightBufferHandle, CNA_ClusteredLightSetHandle, CNA_ClusteredLightGridHandle, CNA_ClusteredLightAssignmentHandle,
) -> CNA_Result;
pub type cna_clustered_light_compute_assign_fn = unsafe extern "C" fn(
    CNA_ClusteredLightComputeHandle, CNA_ClusteredLightGridHandle, *const CNA_Matrix, *const CNA_BoundingSphere, u64, CNA_ClusteredLightAssignmentHandle,
) -> CNA_Result;
pub type cna_clustered_light_compute_copy_unsupported_reason_fn = unsafe extern "C" fn(
    CNA_ClusteredLightComputeHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_clustered_light_compute_create_fn = unsafe extern "C" fn(
    CNA_Handle, i32, *mut CNA_ClusteredLightComputeHandle,
) -> CNA_Result;
pub type cna_clustered_light_compute_destroy_fn = unsafe extern "C" fn(
    CNA_ClusteredLightComputeHandle,
) -> CNA_Result;
pub type cna_clustered_light_compute_get_stride_fn = unsafe extern "C" fn(
    CNA_ClusteredLightComputeHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_light_compute_has_overflowed_fn = unsafe extern "C" fn(
    CNA_ClusteredLightComputeHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_clustered_light_compute_is_supported_fn = unsafe extern "C" fn(
    CNA_ClusteredLightComputeHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_clustered_light_compute_used_compute_fn = unsafe extern "C" fn(
    CNA_ClusteredLightComputeHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_clustered_shadow_policy_copy_selected_fn = unsafe extern "C" fn(
    CNA_ClusteredShadowPolicyHandle, *mut i32, u64, *mut u64,
) -> CNA_Result;
pub type cna_clustered_shadow_policy_create_fn = unsafe extern "C" fn(
    CNA_Handle, i32, *mut CNA_ClusteredShadowPolicyHandle,
) -> CNA_Result;
pub type cna_clustered_shadow_policy_destroy_fn = unsafe extern "C" fn(
    CNA_ClusteredShadowPolicyHandle,
) -> CNA_Result;
pub type cna_clustered_shadow_policy_get_budget_fn = unsafe extern "C" fn(
    CNA_ClusteredShadowPolicyHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_shadow_policy_get_hysteresis_fn = unsafe extern "C" fn(
    CNA_ClusteredShadowPolicyHandle, *mut f32,
) -> CNA_Result;
pub type cna_clustered_shadow_policy_get_refused_count_fn = unsafe extern "C" fn(
    CNA_ClusteredShadowPolicyHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_shadow_policy_get_request_count_fn = unsafe extern "C" fn(
    CNA_ClusteredShadowPolicyHandle, *mut i32,
) -> CNA_Result;
pub type cna_clustered_shadow_policy_get_score_fn = unsafe extern "C" fn(
    CNA_ClusteredShadowPolicyHandle, i32, *mut f32,
) -> CNA_Result;
pub type cna_clustered_shadow_policy_is_selected_fn = unsafe extern "C" fn(
    CNA_ClusteredShadowPolicyHandle, i32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_clustered_shadow_policy_reset_fn = unsafe extern "C" fn(
    CNA_ClusteredShadowPolicyHandle,
) -> CNA_Result;
pub type cna_clustered_shadow_policy_select_fn = unsafe extern "C" fn(
    CNA_ClusteredShadowPolicyHandle, CNA_ClusteredLightSetHandle, *const CNA_Matrix, *const CNA_Matrix, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_clustered_shadow_policy_set_budget_fn = unsafe extern "C" fn(
    CNA_ClusteredShadowPolicyHandle, i32,
) -> CNA_Result;
pub type cna_clustered_shadow_policy_set_hysteresis_fn = unsafe extern "C" fn(
    CNA_ClusteredShadowPolicyHandle, f32,
) -> CNA_Result;
pub type cna_clustered_forward_effect_begin_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *const CNA_Matrix, *const CNA_Matrix, *const CNA_Matrix, *const CNA_Vector3, CNA_ClusteredLightBufferHandle,
) -> CNA_Result;
pub type cna_clustered_forward_effect_clear_area_light_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle,
) -> CNA_Result;
pub type cna_clustered_forward_effect_clear_light_probe_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle,
) -> CNA_Result;
pub type cna_clustered_forward_effect_contribution_fn = unsafe extern "C" fn(
    *const CNA_ClusteredLightEXT, *const CNA_Vector3, *const CNA_Vector3, *const CNA_Vector3, *const CNA_Vector3, f32, f32, f32, f32, *const CNA_Vector3, f32, f32, f32, f32, *const CNA_Vector3, f32, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_clustered_forward_effect_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_ClusteredForwardEffectHandle,
) -> CNA_Result;
pub type cna_clustered_forward_effect_destroy_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle,
) -> CNA_Result;
pub type cna_clustered_forward_effect_get_ambient_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_clustered_forward_effect_get_base_color_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_clustered_forward_effect_get_effect_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_clustered_forward_effect_get_ior_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_clustered_forward_effect_get_metallic_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_clustered_forward_effect_get_opaque_frame_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_clustered_forward_effect_get_roughness_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_clustered_forward_effect_has_area_light_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_clustered_forward_effect_has_light_probe_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_clustered_forward_effect_is_supported_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_clustered_forward_effect_set_ambient_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_clustered_forward_effect_set_base_color_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_clustered_forward_effect_set_ior_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, f32,
) -> CNA_Result;
pub type cna_clustered_forward_effect_set_metallic_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, f32,
) -> CNA_Result;
pub type cna_clustered_forward_effect_set_opaque_frame_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_clustered_forward_effect_set_roughness_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, f32,
) -> CNA_Result;
pub type cna_clustered_forward_effect_volume_attenuation_fn = unsafe extern "C" fn(
    *const CNA_Vector3, f32, f32, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_environment_processor_convert_equirectangular_fn = unsafe extern "C" fn(
    CNA_EnvironmentProcessorHandle, CNA_Handle, i32, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_environment_processor_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_EnvironmentProcessorHandle,
) -> CNA_Result;
pub type cna_environment_processor_destroy_fn = unsafe extern "C" fn(
    CNA_EnvironmentProcessorHandle,
) -> CNA_Result;
pub type cna_environment_processor_direction_to_equirectangular_fn = unsafe extern "C" fn(
    *const CNA_Vector3, *mut f32, *mut f32,
) -> CNA_Result;
pub type cna_environment_processor_face_direction_fn = unsafe extern "C" fn(
    i32, f32, f32, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_environment_processor_generate_brdf_lut_fn = unsafe extern "C" fn(
    CNA_EnvironmentProcessorHandle, i32, i32, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_environment_processor_generate_irradiance_fn = unsafe extern "C" fn(
    CNA_EnvironmentProcessorHandle, CNA_Handle, i32, i32, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_environment_processor_generate_prefiltered_specular_fn = unsafe extern "C" fn(
    CNA_EnvironmentProcessorHandle, CNA_Handle, i32, i32, i32, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_environment_processor_generate_probe_fn = unsafe extern "C" fn(
    CNA_EnvironmentProcessorHandle, CNA_Handle, *const CNA_Vector3, *mut CNA_LightProbeHandle,
) -> CNA_Result;
pub type cna_environment_processor_hammersley_fn = unsafe extern "C" fn(
    i32, i32, *mut f32, *mut f32,
) -> CNA_Result;
pub type cna_environment_processor_importance_sample_ggx_fn = unsafe extern "C" fn(
    f32, f32, *const CNA_Vector3, f32, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_environment_processor_mip_for_roughness_fn = unsafe extern "C" fn(
    f32, i32, *mut f32,
) -> CNA_Result;
pub type cna_environment_processor_roughness_for_mip_fn = unsafe extern "C" fn(
    f32, i32, *mut f32,
) -> CNA_Result;
pub type cna_image_based_light_ext_init_fn = unsafe extern "C" fn(
    *mut CNA_ImageBasedLightEXT,
) -> CNA_Result;
pub type cna_image_based_light_ext_is_valid_fn = unsafe extern "C" fn(
    *const CNA_ImageBasedLightEXT, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_light_probe_ext_copy_coefficients_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, *mut CNA_Vector3, u64, *mut u64,
) -> CNA_Result;
pub type cna_light_probe_ext_copy_evaluation_glsl_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_light_probe_ext_copy_from_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, CNA_LightProbeHandle,
) -> CNA_Result;
pub type cna_light_probe_ext_create_fn = unsafe extern "C" fn(
    *mut CNA_LightProbeHandle,
) -> CNA_Result;
pub type cna_light_probe_ext_create_at_fn = unsafe extern "C" fn(
    *const CNA_Vector3, *mut CNA_LightProbeHandle,
) -> CNA_Result;
pub type cna_light_probe_ext_destroy_fn = unsafe extern "C" fn(CNA_LightProbeHandle) -> CNA_Result;
pub type cna_light_probe_ext_equals_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, CNA_LightProbeHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_light_probe_ext_get_coefficient_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, i32, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_light_probe_ext_get_position_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_light_probe_ext_get_visibility_mean_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, i32, *mut f32,
) -> CNA_Result;
pub type cna_light_probe_ext_get_visibility_mean_squared_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, i32, *mut f32,
) -> CNA_Result;
pub type cna_light_probe_ext_has_visibility_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_light_probe_ext_irradiance_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, *const CNA_Vector3, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_light_probe_ext_is_zero_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_light_probe_ext_scale_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, f32,
) -> CNA_Result;
pub type cna_light_probe_ext_set_coefficient_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, i32, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_light_probe_ext_set_position_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, *const CNA_Vector3,
) -> CNA_Result;
pub type cna_light_probe_ext_set_visibility_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, i32, f32, f32,
) -> CNA_Result;
pub type cna_light_probe_ext_visibility_weight_fn = unsafe extern "C" fn(
    CNA_LightProbeHandle, *const CNA_Vector3, f32, *mut f32,
) -> CNA_Result;
pub type cna_light_probe_baker_bake_light_fn = unsafe extern "C" fn(
    CNA_LightProbeBakerHandle, CNA_LightProbeVolumeHandle, CNA_LightProbeSceneDrawCallback, *mut c_void,
) -> CNA_Result;
pub type cna_light_probe_baker_bake_probe_fn = unsafe extern "C" fn(
    CNA_LightProbeBakerHandle, *const CNA_Vector3, CNA_LightProbeSceneDrawCallback, *mut c_void, *mut CNA_LightProbeHandle,
) -> CNA_Result;
pub type cna_light_probe_baker_bake_visibility_fn = unsafe extern "C" fn(
    CNA_LightProbeBakerHandle, CNA_LightProbeVolumeHandle, CNA_LightProbeSceneDrawCallback, *mut c_void,
) -> CNA_Result;
pub type cna_light_probe_baker_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_LightProbeBakerHandle,
) -> CNA_Result;
pub type cna_light_probe_baker_create_with_face_size_fn = unsafe extern "C" fn(
    CNA_Handle, i32, *mut CNA_LightProbeBakerHandle,
) -> CNA_Result;
pub type cna_light_probe_baker_destroy_fn = unsafe extern "C" fn(
    CNA_LightProbeBakerHandle,
) -> CNA_Result;
pub type cna_light_probe_baker_face_count_fn = unsafe extern "C" fn(*mut i32) -> CNA_Result;
pub type cna_light_probe_baker_face_view_fn = unsafe extern "C" fn(
    CNA_LightProbeBakerHandle, i32, *const CNA_Vector3, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_light_probe_baker_get_face_size_fn = unsafe extern "C" fn(
    CNA_LightProbeBakerHandle, *mut i32,
) -> CNA_Result;
pub type cna_light_probe_baker_get_far_plane_fn = unsafe extern "C" fn(
    CNA_LightProbeBakerHandle, *mut f32,
) -> CNA_Result;
pub type cna_light_probe_baker_get_near_plane_fn = unsafe extern "C" fn(
    CNA_LightProbeBakerHandle, *mut f32,
) -> CNA_Result;
pub type cna_light_probe_baker_is_supported_fn = unsafe extern "C" fn(
    CNA_LightProbeBakerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_light_probe_baker_set_planes_fn = unsafe extern "C" fn(
    CNA_LightProbeBakerHandle, f32, f32,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_contains_fn = unsafe extern "C" fn(
    CNA_LightProbeVolumeHandle, *const CNA_Vector3, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_create_fn = unsafe extern "C" fn(
    *const CNA_BoundingBox, i32, i32, i32, *mut CNA_LightProbeVolumeHandle,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_destroy_fn = unsafe extern "C" fn(
    CNA_LightProbeVolumeHandle,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_get_bounds_fn = unsafe extern "C" fn(
    CNA_LightProbeVolumeHandle, *mut CNA_BoundingBox,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_get_count_x_fn = unsafe extern "C" fn(
    CNA_LightProbeVolumeHandle, *mut i32,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_get_count_y_fn = unsafe extern "C" fn(
    CNA_LightProbeVolumeHandle, *mut i32,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_get_count_z_fn = unsafe extern "C" fn(
    CNA_LightProbeVolumeHandle, *mut i32,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_get_probe_fn = unsafe extern "C" fn(
    CNA_LightProbeVolumeHandle, i32, i32, i32, CNA_LightProbeHandle,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_get_probe_count_fn = unsafe extern "C" fn(
    CNA_LightProbeVolumeHandle, *mut i32,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_get_probe_position_fn = unsafe extern "C" fn(
    CNA_LightProbeVolumeHandle, i32, i32, i32, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_irradiance_fn = unsafe extern "C" fn(
    CNA_LightProbeVolumeHandle, *const CNA_Vector3, *const CNA_Vector3, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_is_zero_fn = unsafe extern "C" fn(
    CNA_LightProbeVolumeHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_sample_probe_fn = unsafe extern "C" fn(
    CNA_LightProbeVolumeHandle, *const CNA_Vector3, CNA_LightProbeHandle,
) -> CNA_Result;
pub type cna_light_probe_volume_ext_set_probe_fn = unsafe extern "C" fn(
    CNA_LightProbeVolumeHandle, i32, i32, i32, CNA_LightProbeHandle,
) -> CNA_Result;
pub type cna_pbr_material_ext_copy_to_string_fn = unsafe extern "C" fn(
    *const CNA_PbrMaterialEXT, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_pbr_material_ext_equals_fn = unsafe extern "C" fn(
    *const CNA_PbrMaterialEXT, *const CNA_PbrMaterialEXT, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_material_ext_get_hash_code_fn = unsafe extern "C" fn(
    *const CNA_PbrMaterialEXT, *mut u64,
) -> CNA_Result;
pub type cna_pbr_material_extensions_copy_to_string_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_clearcoat_normal_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_clearcoat_roughness_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_clearcoat_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_iridescence_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_iridescence_thickness_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_sheen_color_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_sheen_roughness_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_thickness_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_get_transmission_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_clearcoat_normal_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_clearcoat_roughness_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_clearcoat_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_iridescence_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_iridescence_thickness_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_sheen_color_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_sheen_roughness_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_thickness_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_material_extensions_set_transmission_texture_fn = unsafe extern "C" fn(
    CNA_PbrMaterialExtensionsHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_skinned_pbr_effect_apply_material_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *const CNA_PbrMaterialEXT,
) -> CNA_Result;
pub type cna_skinned_pbr_effect_copy_bone_transforms_fn = unsafe extern "C" fn(
    CNA_EffectHandle, u64, *mut CNA_Matrix, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinned_pbr_effect_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_skinned_pbr_effect_extract_material_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_PbrMaterialEXT,
) -> CNA_Result;
pub type cna_skinned_pbr_effect_get_weights_per_vertex_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut i32,
) -> CNA_Result;
pub type cna_skinned_pbr_effect_set_bone_transforms_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *const CNA_Matrix, u64,
) -> CNA_Result;
pub type cna_skinned_pbr_effect_set_weights_per_vertex_fn = unsafe extern "C" fn(
    CNA_EffectHandle, i32,
) -> CNA_Result;
pub type cna_thin_film_iridescence_copy_glsl_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_thin_film_iridescence_evaluate_fn = unsafe extern "C" fn(
    f32, f32, f32, f32, *const CNA_Vector3, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_shader_effect_factory_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_ShaderEffectFactoryHandle,
) -> CNA_Result;
pub type cna_shader_effect_factory_acquire_fn = unsafe extern "C" fn(
    CNA_ShaderEffectFactoryHandle, CNA_StringView, CNA_StringView, CNA_StringView, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_shader_effect_factory_contains_fn = unsafe extern "C" fn(
    CNA_ShaderEffectFactoryHandle, CNA_StringView, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_shader_effect_factory_get_compile_count_fn = unsafe extern "C" fn(
    CNA_ShaderEffectFactoryHandle, *mut u64,
) -> CNA_Result;
pub type cna_shader_effect_factory_clear_fn = unsafe extern "C" fn(
    CNA_ShaderEffectFactoryHandle,
) -> CNA_Result;
pub type cna_shader_effect_factory_destroy_fn = unsafe extern "C" fn(
    CNA_ShaderEffectFactoryHandle,
) -> CNA_Result;
pub type cna_clustered_forward_effect_get_material_extensions_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *mut CNA_PbrMaterialExtensionsHandle,
) -> CNA_Result;
pub type cna_clustered_forward_effect_set_material_extensions_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, CNA_PbrMaterialExtensionsHandle,
) -> CNA_Result;
pub type cna_clustered_forward_effect_contribution_with_extensions_fn = unsafe extern "C" fn(
    *const CNA_ClusteredLightEXT, *const CNA_Vector3, *const CNA_Vector3, *const CNA_Vector3, *const CNA_Vector3, f32, f32, CNA_PbrMaterialExtensionsHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_clustered_forward_effect_set_light_probe_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, CNA_LightProbeHandle,
) -> CNA_Result;
pub type cna_clustered_forward_effect_set_light_probe_volume_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, CNA_LightProbeVolumeHandle,
) -> CNA_Result;
pub type cna_gltf_material_source_ext_init_fn = unsafe extern "C" fn(
    *mut CNA_GltfMaterialSourceEXT,
) -> CNA_Result;
pub type cna_gltf_material_extension_source_ext_init_fn = unsafe extern "C" fn(
    *mut CNA_GltfMaterialExtensionSourceEXT,
) -> CNA_Result;
pub type cna_gltf_material_textures_ext_init_fn = unsafe extern "C" fn(
    *mut CNA_GltfMaterialTexturesEXT,
) -> CNA_Result;
pub type cna_gltf_material_extension_textures_ext_init_fn = unsafe extern "C" fn(
    *mut CNA_GltfMaterialExtensionTexturesEXT,
) -> CNA_Result;
pub type cna_gltf_material_bridge_build_material_fn = unsafe extern "C" fn(
    *const CNA_GltfMaterialSourceEXT, *const CNA_GltfMaterialTexturesEXT, *mut CNA_PbrMaterialEXT,
) -> CNA_Result;
pub type cna_gltf_material_bridge_build_extensions_fn = unsafe extern "C" fn(
    *const CNA_GltfMaterialExtensionSourceEXT, *const CNA_GltfMaterialExtensionTexturesEXT, CNA_PbrMaterialExtensionsHandle,
) -> CNA_Result;
pub type cna_render_pipeline_get_skybox_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, *mut CNA_SkyboxHandle,
) -> CNA_Result;
pub type cna_render_pipeline_set_skybox_fn = unsafe extern "C" fn(
    CNA_RenderPipelineHandle, CNA_SkyboxHandle,
) -> CNA_Result;
pub type cna_debug_draw_add_probe_volume_gizmo_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, CNA_LightProbeVolumeHandle, CNA_Color, f32,
) -> CNA_Result;
pub type cna_debug_draw_add_cluster_slice_gizmo_fn = unsafe extern "C" fn(
    CNA_DebugDrawHandle, CNA_ClusteredLightGridHandle, *const CNA_Matrix, CNA_Color,
) -> CNA_Result;
pub type cna_area_light_brdf_table_copy_lookup_glsl_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_area_light_brdf_table_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_AreaLightBrdfTableHandle,
) -> CNA_Result;
pub type cna_area_light_brdf_table_create_with_size_fn = unsafe extern "C" fn(
    CNA_Handle, i32, i32, *mut CNA_AreaLightBrdfTableHandle,
) -> CNA_Result;
pub type cna_area_light_brdf_table_destroy_fn = unsafe extern "C" fn(
    CNA_AreaLightBrdfTableHandle,
) -> CNA_Result;
pub type cna_area_light_brdf_table_evaluate_fn = unsafe extern "C" fn(
    f32, f32, i32, *mut CNA_AreaLightBrdfTerms,
) -> CNA_Result;
pub type cna_area_light_brdf_table_get_generation_milliseconds_fn = unsafe extern "C" fn(
    CNA_AreaLightBrdfTableHandle, *mut f64,
) -> CNA_Result;
pub type cna_area_light_brdf_table_get_sample_count_fn = unsafe extern "C" fn(
    CNA_AreaLightBrdfTableHandle, *mut i32,
) -> CNA_Result;
pub type cna_area_light_brdf_table_get_size_fn = unsafe extern "C" fn(
    CNA_AreaLightBrdfTableHandle, *mut i32,
) -> CNA_Result;
pub type cna_area_light_brdf_table_get_texture_fn = unsafe extern "C" fn(
    CNA_AreaLightBrdfTableHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_area_light_ext_init_fn = unsafe extern "C" fn(*mut CNA_AreaLightEXT) -> CNA_Result;
pub type cna_area_light_ext_is_valid_fn = unsafe extern "C" fn(
    *const CNA_AreaLightEXT, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_area_light_shading_contribution_fn = unsafe extern "C" fn(
    *const CNA_AreaLightEXT, *const CNA_Vector3, *const CNA_Vector3, *const CNA_Vector3, *const CNA_Vector3, f32, f32, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_area_light_shading_copy_shading_glsl_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_area_light_shading_coverage_fn = unsafe extern "C" fn(
    *const CNA_Vector3, *const CNA_Vector3, *const CNA_Vector3, f32, CNA_Bool, *mut f32,
) -> CNA_Result;
pub type cna_area_light_shading_lobe_scale_for_fn = unsafe extern "C" fn(
    f32, *mut f32,
) -> CNA_Result;
pub type cna_area_light_shading_quad_of_fn = unsafe extern "C" fn(
    *const CNA_AreaLightEXT, *const CNA_Vector3, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_clustered_forward_effect_set_area_light_fn = unsafe extern "C" fn(
    CNA_ClusteredForwardEffectHandle, *const CNA_AreaLightEXT, CNA_AreaLightBrdfTableHandle,
) -> CNA_Result;
pub type cna_effect_get_image_based_light_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_ImageBasedLightEXT,
) -> CNA_Result;
pub type cna_effect_get_light_view_projection_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_effect_get_punctual_light_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_PunctualLightEXT,
) -> CNA_Result;
pub type cna_effect_get_shadow_cascades_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_ShadowCascadeStateEXT,
) -> CNA_Result;
pub type cna_effect_get_shadow_depth_bias_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut f32,
) -> CNA_Result;
pub type cna_effect_get_shadow_filter_radius_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut i32,
) -> CNA_Result;
pub type cna_effect_get_shadow_map_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_effect_is_shadows_enabled_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_effect_set_image_based_light_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *const CNA_ImageBasedLightEXT,
) -> CNA_Result;
pub type cna_effect_set_light_view_projection_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *const CNA_Matrix,
) -> CNA_Result;
pub type cna_effect_set_punctual_light_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *const CNA_PunctualLightEXT,
) -> CNA_Result;
pub type cna_effect_set_shadow_cascades_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *const CNA_ShadowCascadeStateEXT,
) -> CNA_Result;
pub type cna_effect_set_shadow_depth_bias_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, f32,
) -> CNA_Result;
pub type cna_effect_set_shadow_filter_radius_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, i32,
) -> CNA_Result;
pub type cna_effect_set_shadow_map_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_effect_set_shadows_enabled_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_copy_instance_elements_fn = unsafe extern "C" fn(
    *mut CNA_VertexElement, u64, *mut u64,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_get_instance_stride_fn = unsafe extern "C" fn(
    *mut i32,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_copy_tint_elements_fn = unsafe extern "C" fn(
    *mut CNA_VertexElement, u64, *mut u64,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_get_tint_stride_fn = unsafe extern "C" fn(
    *mut i32,
) -> CNA_Result;
pub type cna_indirect_draw_arguments_init_fn = unsafe extern "C" fn(
    *mut CNA_IndirectDrawArguments,
) -> CNA_Result;
pub type cna_indirect_draw_indexed_arguments_init_fn = unsafe extern "C" fn(
    *mut CNA_IndirectDrawIndexedArguments,
) -> CNA_Result;
pub type cna_gpu_cullable_instance_init_fn = unsafe extern "C" fn(
    *mut CNA_GpuCullableInstance,
) -> CNA_Result;
pub type cna_gpu_instance_culler_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_GpuInstanceCullerHandle,
) -> CNA_Result;
pub type cna_gpu_instance_culler_destroy_fn = unsafe extern "C" fn(
    CNA_GpuInstanceCullerHandle,
) -> CNA_Result;
pub type cna_gpu_instance_culler_is_supported_fn = unsafe extern "C" fn(
    CNA_GpuInstanceCullerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gpu_instance_culler_copy_unsupported_reason_fn = unsafe extern "C" fn(
    CNA_GpuInstanceCullerHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_gpu_instance_culler_set_instances_fn = unsafe extern "C" fn(
    CNA_GpuInstanceCullerHandle, *const CNA_GpuCullableInstance, u64,
) -> CNA_Result;
pub type cna_gpu_instance_culler_get_instance_count_fn = unsafe extern "C" fn(
    CNA_GpuInstanceCullerHandle, *mut i32,
) -> CNA_Result;
pub type cna_gpu_instance_culler_cull_fn = unsafe extern "C" fn(
    CNA_GpuInstanceCullerHandle, *const CNA_Matrix, *const CNA_Matrix, i32, i32, i32,
) -> CNA_Result;
pub type cna_gpu_instance_culler_draw_fn = unsafe extern "C" fn(
    CNA_GpuInstanceCullerHandle, CNA_PrimitiveType,
) -> CNA_Result;
pub type cna_gpu_instance_culler_read_visible_count_ext_fn = unsafe extern "C" fn(
    CNA_GpuInstanceCullerHandle, *mut i32,
) -> CNA_Result;
pub type cna_gpu_instance_culler_copy_instance_lookup_glsl_fn = unsafe extern "C" fn(
    *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_graphics_device_draw_primitives_indirect_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_PrimitiveType, CNA_StorageBufferHandle, i32,
) -> CNA_Result;
pub type cna_graphics_device_draw_indexed_primitives_indirect_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_PrimitiveType, CNA_StorageBufferHandle, i32,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_create_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_ModelMeshPartHandle, *mut CNA_InstancedRendererEXTHandle,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_destroy_fn = unsafe extern "C" fn(
    CNA_InstancedRendererEXTHandle,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_did_last_draw_instance_fn = unsafe extern "C" fn(
    CNA_InstancedRendererEXTHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_draw_fn = unsafe extern "C" fn(
    CNA_InstancedRendererEXTHandle, CNA_EffectHandle,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_get_instance_capacity_fn = unsafe extern "C" fn(
    CNA_InstancedRendererEXTHandle, *mut i32,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_get_instance_count_fn = unsafe extern "C" fn(
    CNA_InstancedRendererEXTHandle, *mut i32,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_get_last_draw_call_count_fn = unsafe extern "C" fn(
    CNA_InstancedRendererEXTHandle, *mut i32,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_is_fallback_enabled_fn = unsafe extern "C" fn(
    CNA_InstancedRendererEXTHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_is_instancing_supported_fn = unsafe extern "C" fn(
    CNA_InstancedRendererEXTHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_is_tints_enabled_fn = unsafe extern "C" fn(
    CNA_InstancedRendererEXTHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_set_fallback_enabled_fn = unsafe extern "C" fn(
    CNA_InstancedRendererEXTHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_set_instance_tints_fn = unsafe extern "C" fn(
    CNA_InstancedRendererEXTHandle, *const CNA_Color, u64,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_set_instances_fn = unsafe extern "C" fn(
    CNA_InstancedRendererEXTHandle, *const CNA_Matrix, u64,
) -> CNA_Result;
pub type cna_instanced_renderer_ext_set_tints_enabled_fn = unsafe extern "C" fn(
    CNA_InstancedRendererEXTHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_model_mesh_part_create_fn = unsafe extern "C" fn(
    CNA_VertexBufferHandle, CNA_IndexBufferHandle, i32, i32, i32, i32, *mut CNA_ModelMeshPartHandle,
) -> CNA_Result;
pub type cna_model_mesh_part_destroy_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle,
) -> CNA_Result;
pub type cna_model_mesh_part_get_num_vertices_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, *mut i32,
) -> CNA_Result;
pub type cna_model_mesh_part_get_primitive_count_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, *mut i32,
) -> CNA_Result;
pub type cna_model_mesh_part_get_primitive_type_ext_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, *mut CNA_PrimitiveType,
) -> CNA_Result;
pub type cna_model_mesh_part_get_start_index_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, *mut i32,
) -> CNA_Result;
pub type cna_model_mesh_part_get_vertex_offset_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, *mut i32,
) -> CNA_Result;
pub type cna_model_mesh_part_set_primitive_type_ext_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, CNA_PrimitiveType,
) -> CNA_Result;
pub type cna_skinned_model_ext_add_part_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, CNA_StringView, CNA_VertexBufferHandle, CNA_IndexBufferHandle, CNA_ModelMeshPartHandle, CNA_Handle,
) -> CNA_Result;
pub type cna_skinned_model_ext_attach_parts_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, CNA_SkinnedModelEXTHandle,
) -> CNA_Result;
pub type cna_skinned_model_ext_compute_bone_transforms_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, CNA_StringView, f64, CNA_Bool, *mut CNA_Matrix, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_copy_bind_pose_local_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, *mut CNA_Matrix, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_copy_clip_name_at_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_copy_clip_track_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, CNA_StringView, u64, *mut i32, *mut CNA_KeyframeEXT, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_copy_inverse_bind_pose_global_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, *mut CNA_Matrix, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_copy_parent_bone_indices_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, *mut i32, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_copy_part_name_at_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_create_fn = unsafe extern "C" fn(
    *const CNA_SkinnedModelEXTDescriptor, *mut CNA_SkinnedModelEXTHandle,
) -> CNA_Result;
pub type cna_skinned_model_ext_create_default_fn = unsafe extern "C" fn(
    *mut CNA_SkinnedModelEXTHandle,
) -> CNA_Result;
pub type cna_skinned_model_ext_create_move_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, *mut CNA_SkinnedModelEXTHandle,
) -> CNA_Result;
pub type cna_skinned_model_ext_destroy_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle,
) -> CNA_Result;
pub type cna_skinned_model_ext_get_bone_count_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_get_clip_count_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_get_clip_info_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, CNA_StringView, *mut CNA_Bool, *mut f64, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_get_clip_name_byte_count_at_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_get_owned_resource_counts_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, *mut u64, *mut u64, *mut u64, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_get_part_at_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, u64, *mut CNA_ModelMeshPartHandle, *mut CNA_Bool, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_skinned_model_ext_get_part_count_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_get_part_name_byte_count_at_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinned_model_ext_move_assign_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, CNA_SkinnedModelEXTHandle,
) -> CNA_Result;
pub type cna_skinned_model_ext_remove_clip_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, CNA_StringView,
) -> CNA_Result;
pub type cna_skinned_model_ext_remove_part_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, CNA_StringView,
) -> CNA_Result;
pub type cna_skinned_model_ext_set_clip_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, CNA_StringView, *const CNA_AnimationClipEXTDescriptor,
) -> CNA_Result;
pub type cna_skinned_model_ext_set_skeleton_fn = unsafe extern "C" fn(
    CNA_SkinnedModelEXTHandle, i32, *const i32, *const CNA_Matrix, *const CNA_Matrix,
) -> CNA_Result;
pub type cna_animation_player_copy_bone_transforms_fn = unsafe extern "C" fn(
    CNA_AnimationPlayerHandle, *mut CNA_Matrix, u64, *mut u64,
) -> CNA_Result;
pub type cna_animation_player_copy_current_clip_name_fn = unsafe extern "C" fn(
    CNA_AnimationPlayerHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_animation_player_copy_skin_transforms_fn = unsafe extern "C" fn(
    CNA_AnimationPlayerHandle, *mut CNA_Matrix, u64, *mut u64,
) -> CNA_Result;
pub type cna_animation_player_copy_world_transforms_fn = unsafe extern "C" fn(
    CNA_AnimationPlayerHandle, *mut CNA_Matrix, u64, *mut u64,
) -> CNA_Result;
pub type cna_animation_player_create_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, *mut CNA_AnimationPlayerHandle,
) -> CNA_Result;
pub type cna_animation_player_destroy_fn = unsafe extern "C" fn(
    CNA_AnimationPlayerHandle,
) -> CNA_Result;
pub type cna_animation_player_get_current_clip_info_fn = unsafe extern "C" fn(
    CNA_AnimationPlayerHandle, *mut CNA_Bool, *mut f64, *mut u64,
) -> CNA_Result;
pub type cna_animation_player_get_current_clip_name_byte_count_fn = unsafe extern "C" fn(
    CNA_AnimationPlayerHandle, *mut u64,
) -> CNA_Result;
pub type cna_animation_player_get_current_position_fn = unsafe extern "C" fn(
    CNA_AnimationPlayerHandle, *mut f64,
) -> CNA_Result;
pub type cna_animation_player_start_clip_fn = unsafe extern "C" fn(
    CNA_AnimationPlayerHandle, CNA_StringView,
) -> CNA_Result;
pub type cna_animation_player_update_fn = unsafe extern "C" fn(
    CNA_AnimationPlayerHandle, f64, CNA_Bool, CNA_Bool,
) -> CNA_Result;
pub type cna_skinning_data_copy_bind_pose_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, *mut CNA_Matrix, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_copy_clip_name_at_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_copy_clip_track_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, CNA_StringView, u64, *mut i32, *mut CNA_KeyframeEXT, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_copy_inverse_bind_pose_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, *mut CNA_Matrix, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_copy_skeleton_hierarchy_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, *mut i32, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_copy_skeleton_root_name_ext_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_copy_skeleton_root_prefix_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, *mut CNA_Matrix, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_copy_type_name_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_create_fn = unsafe extern "C" fn(
    *const CNA_SkinningDataDescriptor, *mut CNA_SkinningDataHandle,
) -> CNA_Result;
pub type cna_skinning_data_destroy_fn = unsafe extern "C" fn(CNA_SkinningDataHandle) -> CNA_Result;
pub type cna_skinning_data_get_bone_count_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_get_clip_count_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_get_clip_info_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, CNA_StringView, *mut CNA_Bool, *mut f64, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_get_clip_name_byte_count_at_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_get_clip_target_space_ext_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, u64, *mut CNA_ClipTargetSpaceEXT,
) -> CNA_Result;
pub type cna_skinning_data_get_skeleton_root_name_byte_count_ext_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_get_skeleton_root_node_index_ext_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, *mut i32,
) -> CNA_Result;
pub type cna_skinning_data_get_type_name_byte_count_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, *mut u64,
) -> CNA_Result;
pub type cna_skinning_data_set_clip_target_space_ext_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, u64, CNA_ClipTargetSpaceEXT,
) -> CNA_Result;
pub type cna_skinning_data_set_skeleton_root_name_ext_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, CNA_StringView,
) -> CNA_Result;
pub type cna_skinning_data_set_skeleton_root_node_index_ext_fn = unsafe extern "C" fn(
    CNA_SkinningDataHandle, i32,
) -> CNA_Result;
pub type cna_model_mesh_part_get_morph_target_data_ext_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, *mut CNA_Bool, *mut CNA_MorphTargetDataEXTHandle,
) -> CNA_Result;
pub type cna_model_mesh_part_set_morph_target_data_ext_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, CNA_MorphTargetDataEXTHandle,
) -> CNA_Result;
pub type cna_model_mesh_part_set_morph_weights_ext_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, *const f32, u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_blend_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *const f32, u64, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_copy_base_vertex_bytes_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_copy_normal_deltas_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, u64, *mut CNA_Vector3, u64, *mut u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_copy_position_deltas_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, u64, *mut CNA_Vector3, u64, *mut u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_copy_tangent_deltas_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, u64, *mut CNA_Vector3, u64, *mut u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_copy_triangle_indices_ext_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *mut u32, u64, *mut u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_copy_type_name_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_copy_weight_keyframe_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, u64, *mut f64, *mut f32, u64, *mut u64, *mut f32, u64, *mut u64, *mut f32, u64, *mut u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_copy_weights_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *mut f32, u64, *mut u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_create_fn = unsafe extern "C" fn(
    *const CNA_MorphTargetDataEXTDescriptor, *mut CNA_MorphTargetDataEXTHandle,
) -> CNA_Result;
pub type cna_morph_target_data_ext_destroy_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle,
) -> CNA_Result;
pub type cna_morph_target_data_ext_get_base_vertex_byte_count_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *mut u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_get_recompute_flat_normals_ext_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_morph_target_data_ext_get_stride_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *mut i32,
) -> CNA_Result;
pub type cna_morph_target_data_ext_get_target_count_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *mut u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_get_type_name_byte_count_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *mut u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_get_weight_track_info_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *mut u64, *mut CNA_Bool, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_morph_target_data_ext_set_recompute_flat_normals_ext_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_morph_target_data_ext_set_tangent_deltas_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, u64, *const CNA_Vector3, u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_set_triangle_indices_ext_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *const u32, u64,
) -> CNA_Result;
pub type cna_morph_target_data_ext_set_weight_track_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *const CNA_MorphWeightTrackEXTDescriptor,
) -> CNA_Result;
pub type cna_morph_target_data_ext_set_weights_fn = unsafe extern "C" fn(
    CNA_MorphTargetDataEXTHandle, *const f32, u64,
) -> CNA_Result;
pub type cna_morph_weight_track_ext_evaluate_fn = unsafe extern "C" fn(
    *const CNA_MorphWeightTrackEXTDescriptor, f64, *mut f32, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_animations_ext_create_fn = unsafe extern "C" fn(
    *const CNA_NamedAnimationClipEXTDescriptor, u64, *mut CNA_ModelAnimationsEXTHandle,
) -> CNA_Result;
pub type cna_model_animations_ext_destroy_fn = unsafe extern "C" fn(
    CNA_ModelAnimationsEXTHandle,
) -> CNA_Result;
pub type cna_model_animations_ext_get_type_name_byte_count_fn = unsafe extern "C" fn(
    CNA_ModelAnimationsEXTHandle, *mut u64,
) -> CNA_Result;
pub type cna_model_animations_ext_copy_type_name_fn = unsafe extern "C" fn(
    CNA_ModelAnimationsEXTHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_animations_ext_get_clip_count_fn = unsafe extern "C" fn(
    CNA_ModelAnimationsEXTHandle, *mut u64,
) -> CNA_Result;
pub type cna_model_animations_ext_get_clip_name_byte_count_at_fn = unsafe extern "C" fn(
    CNA_ModelAnimationsEXTHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_animations_ext_copy_clip_name_at_fn = unsafe extern "C" fn(
    CNA_ModelAnimationsEXTHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_animations_ext_get_clip_info_at_fn = unsafe extern "C" fn(
    CNA_ModelAnimationsEXTHandle, u64, *mut f64, *mut u64, *mut CNA_ClipTargetSpaceEXT,
) -> CNA_Result;
pub type cna_model_animations_ext_set_clip_target_space_at_fn = unsafe extern "C" fn(
    CNA_ModelAnimationsEXTHandle, u64, CNA_ClipTargetSpaceEXT,
) -> CNA_Result;
pub type cna_model_mesh_part_get_sampler_state_ext_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, CNA_PbrTextureSlot, *mut CNA_SamplerState,
) -> CNA_Result;
pub type cna_model_mesh_part_set_sampler_state_ext_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, CNA_PbrTextureSlot, *const CNA_SamplerState,
) -> CNA_Result;
pub type cna_matrix_create_infinite_perspective_field_of_view_ext_fn = unsafe extern "C" fn(
    f32, f32, f32, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_camera_copy_name_at_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_camera_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_CameraHandle,
) -> CNA_Result;
pub type cna_camera_create_with_test_backend_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_CameraHandle,
) -> CNA_Result;
pub type cna_camera_destroy_fn = unsafe extern "C" fn(CNA_CameraHandle) -> CNA_Result;
pub type cna_camera_device_info_init_fn = unsafe extern "C" fn(
    *mut CNA_CameraDeviceInfo,
) -> CNA_Result;
pub type cna_camera_get_count_ext_fn = unsafe extern "C" fn(CNA_Handle, *mut u64) -> CNA_Result;
pub type cna_camera_get_frame_height_ext_fn = unsafe extern "C" fn(
    CNA_CameraHandle, *mut i32,
) -> CNA_Result;
pub type cna_camera_get_frame_width_ext_fn = unsafe extern "C" fn(
    CNA_CameraHandle, *mut i32,
) -> CNA_Result;
pub type cna_camera_get_info_at_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u64, *mut CNA_CameraDeviceInfo,
) -> CNA_Result;
pub type cna_camera_get_is_supported_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_camera_get_name_size_at_ext_fn = unsafe extern "C" fn(
    CNA_Handle, u64, *mut u64,
) -> CNA_Result;
pub type cna_camera_get_state_ext_fn = unsafe extern "C" fn(
    CNA_CameraHandle, *mut CNA_CameraState,
) -> CNA_Result;
pub type cna_camera_set_test_frame_ext_fn = unsafe extern "C" fn(
    CNA_CameraHandle, i32, i32, *const CNA_Color, u64,
) -> CNA_Result;
pub type cna_camera_set_test_state_ext_fn = unsafe extern "C" fn(
    CNA_CameraHandle, CNA_CameraState,
) -> CNA_Result;
pub type cna_camera_try_acquire_frame_ext_fn = unsafe extern "C" fn(
    CNA_CameraHandle, CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;

// --- RUST-EXT-015g: the Model object graph an XNA game loads and draws ------
//
// Every navigation route below hands back an *owned* handle, including the
// ones the header calls views: `cna_model_get_bones` answers a fresh
// collection handle on each call, and a bone view taken from it keeps
// answering -- name and all -- after `cna_model_destroy`. Measured with
// tools/reproducers/ext015g_model_ownership.c, which is why the safe layer's
// `ModelBone`/`ModelMesh`/`ModelMeshPart` carry no lifetime parameter.
pub type cna_content_manager_load_model_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_StringView, *mut CNA_ModelHandle,
) -> CNA_Result;
pub type cna_model_add_camera_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *const CNA_ModelCameraDescriptorEXT,
) -> CNA_Result;
pub type cna_model_add_gltf_import_diagnostic_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *const CNA_GltfImportDiagnosticDescriptorEXT,
) -> CNA_Result;
pub type cna_model_add_skin_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, CNA_StringView, CNA_SkinningDataHandle, *const u64, u64,
) -> CNA_Result;
pub type cna_model_apply_bind_pose_bone_transforms_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, CNA_SkinningDataHandle, *mut u64,
) -> CNA_Result;
pub type cna_model_apply_clip_to_bones_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, CNA_ModelAnimationsEXTHandle, u64, f64,
) -> CNA_Result;
pub type cna_model_bone_collection_contains_fn = unsafe extern "C" fn(
    CNA_ModelBoneCollectionHandle, CNA_ModelBoneHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_model_bone_collection_destroy_fn = unsafe extern "C" fn(
    CNA_ModelBoneCollectionHandle,
) -> CNA_Result;
pub type cna_model_bone_collection_find_fn = unsafe extern "C" fn(
    CNA_ModelBoneCollectionHandle, CNA_StringView, *mut CNA_Bool, *mut CNA_ModelBoneHandle,
) -> CNA_Result;
pub type cna_model_bone_collection_get_at_fn = unsafe extern "C" fn(
    CNA_ModelBoneCollectionHandle, u64, *mut CNA_ModelBoneHandle,
) -> CNA_Result;
pub type cna_model_bone_collection_get_count_fn = unsafe extern "C" fn(
    CNA_ModelBoneCollectionHandle, *mut u64,
) -> CNA_Result;
pub type cna_model_bone_copy_name_fn = unsafe extern "C" fn(
    CNA_ModelBoneHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_bone_destroy_fn = unsafe extern "C" fn(CNA_ModelBoneHandle) -> CNA_Result;
pub type cna_model_bone_get_children_fn = unsafe extern "C" fn(
    CNA_ModelBoneHandle, *mut CNA_ModelBoneCollectionHandle,
) -> CNA_Result;
pub type cna_model_bone_get_index_fn = unsafe extern "C" fn(
    CNA_ModelBoneHandle, *mut i32,
) -> CNA_Result;
pub type cna_model_bone_get_name_byte_count_fn = unsafe extern "C" fn(
    CNA_ModelBoneHandle, *mut u64,
) -> CNA_Result;
pub type cna_model_bone_get_parent_fn = unsafe extern "C" fn(
    CNA_ModelBoneHandle, *mut CNA_Bool, *mut CNA_ModelBoneHandle,
) -> CNA_Result;
pub type cna_model_bone_get_transform_fn = unsafe extern "C" fn(
    CNA_ModelBoneHandle, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_model_bone_set_transform_fn = unsafe extern "C" fn(
    CNA_ModelBoneHandle, CNA_Matrix,
) -> CNA_Result;
pub type cna_model_clear_cameras_ext_fn = unsafe extern "C" fn(CNA_ModelHandle) -> CNA_Result;
pub type cna_model_clear_skins_ext_fn = unsafe extern "C" fn(CNA_ModelHandle) -> CNA_Result;
pub type cna_model_copy_absolute_bone_transforms_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut CNA_Matrix, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_copy_bone_transforms_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut CNA_Matrix, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_copy_camera_name_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_copy_gltf_import_diagnostic_code_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_copy_gltf_import_diagnostic_detail_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_copy_gltf_import_diagnostic_message_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_copy_gltf_import_diagnostic_subject_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_copy_material_variant_name_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_copy_skin_name_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_create_skin_skeleton_handle_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut CNA_SkinningDataHandle,
) -> CNA_Result;
pub type cna_model_destroy_fn = unsafe extern "C" fn(CNA_ModelHandle) -> CNA_Result;
pub type cna_model_draw_fn = unsafe extern "C" fn(
    CNA_ModelHandle, CNA_Matrix, CNA_Matrix, CNA_Matrix,
) -> CNA_Result;
pub type cna_model_effect_collection_contains_fn = unsafe extern "C" fn(
    CNA_ModelEffectCollectionHandle, CNA_EffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_model_effect_collection_destroy_fn = unsafe extern "C" fn(
    CNA_ModelEffectCollectionHandle,
) -> CNA_Result;
pub type cna_model_effect_collection_get_at_fn = unsafe extern "C" fn(
    CNA_ModelEffectCollectionHandle, u64, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_model_effect_collection_get_count_fn = unsafe extern "C" fn(
    CNA_ModelEffectCollectionHandle, *mut u64,
) -> CNA_Result;
pub type cna_model_get_bones_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut CNA_ModelBoneCollectionHandle,
) -> CNA_Result;
pub type cna_model_get_bone_transform_count_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut u64,
) -> CNA_Result;
pub type cna_model_get_bounding_sphere_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut CNA_Bool, *mut CNA_BoundingSphere,
) -> CNA_Result;
pub type cna_model_get_camera_count_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut u64,
) -> CNA_Result;
pub type cna_model_get_camera_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut CNA_ModelCameraEXT,
) -> CNA_Result;
pub type cna_model_get_camera_name_byte_count_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_get_gltf_import_diagnostic_code_byte_count_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_get_gltf_import_diagnostic_detail_byte_count_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_get_gltf_import_diagnostic_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut CNA_GltfImportDiagnosticEXT,
) -> CNA_Result;
pub type cna_model_get_gltf_import_diagnostic_message_byte_count_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_get_gltf_import_diagnostic_subject_byte_count_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_get_gltf_import_report_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut CNA_GltfImportReportEXT,
) -> CNA_Result;
pub type cna_model_get_material_variant_count_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut u64,
) -> CNA_Result;
pub type cna_model_get_material_variant_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut i32,
) -> CNA_Result;
pub type cna_model_get_material_variant_name_byte_count_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_get_meshes_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut CNA_ModelMeshCollectionHandle,
) -> CNA_Result;
pub type cna_model_get_root_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut CNA_Bool, *mut CNA_ModelBoneHandle,
) -> CNA_Result;
pub type cna_model_get_skin_count_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut u64,
) -> CNA_Result;
pub type cna_model_get_skin_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut CNA_Bool, *mut u64,
) -> CNA_Result;
pub type cna_model_get_skin_mesh_index_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_get_skin_name_byte_count_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_mesh_collection_contains_fn = unsafe extern "C" fn(
    CNA_ModelMeshCollectionHandle, CNA_ModelMeshHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_model_mesh_collection_destroy_fn = unsafe extern "C" fn(
    CNA_ModelMeshCollectionHandle,
) -> CNA_Result;
pub type cna_model_mesh_collection_find_fn = unsafe extern "C" fn(
    CNA_ModelMeshCollectionHandle, CNA_StringView, *mut CNA_Bool, *mut CNA_ModelMeshHandle,
) -> CNA_Result;
pub type cna_model_mesh_collection_get_at_fn = unsafe extern "C" fn(
    CNA_ModelMeshCollectionHandle, u64, *mut CNA_ModelMeshHandle,
) -> CNA_Result;
pub type cna_model_mesh_collection_get_count_fn = unsafe extern "C" fn(
    CNA_ModelMeshCollectionHandle, *mut u64,
) -> CNA_Result;
pub type cna_model_mesh_copy_name_fn = unsafe extern "C" fn(
    CNA_ModelMeshHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_model_mesh_destroy_fn = unsafe extern "C" fn(CNA_ModelMeshHandle) -> CNA_Result;
pub type cna_model_mesh_draw_fn = unsafe extern "C" fn(CNA_ModelMeshHandle) -> CNA_Result;
pub type cna_model_mesh_get_bounding_sphere_fn = unsafe extern "C" fn(
    CNA_ModelMeshHandle, *mut CNA_BoundingSphere,
) -> CNA_Result;
pub type cna_model_mesh_get_effects_fn = unsafe extern "C" fn(
    CNA_ModelMeshHandle, *mut CNA_ModelEffectCollectionHandle,
) -> CNA_Result;
pub type cna_model_mesh_get_mesh_parts_fn = unsafe extern "C" fn(
    CNA_ModelMeshHandle, *mut CNA_ModelMeshPartCollectionHandle,
) -> CNA_Result;
pub type cna_model_mesh_get_name_byte_count_fn = unsafe extern "C" fn(
    CNA_ModelMeshHandle, *mut u64,
) -> CNA_Result;
pub type cna_model_mesh_get_parent_bone_fn = unsafe extern "C" fn(
    CNA_ModelMeshHandle, *mut CNA_Bool, *mut CNA_ModelBoneHandle,
) -> CNA_Result;
pub type cna_model_mesh_part_collection_destroy_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartCollectionHandle,
) -> CNA_Result;
pub type cna_model_mesh_part_collection_get_at_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartCollectionHandle, u64, *mut CNA_ModelMeshPartHandle,
) -> CNA_Result;
pub type cna_model_mesh_part_collection_get_count_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartCollectionHandle, *mut u64,
) -> CNA_Result;
pub type cna_model_mesh_part_get_effect_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, *mut CNA_Bool, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_model_mesh_part_get_index_buffer_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, *mut CNA_Bool, *mut CNA_IndexBufferHandle,
) -> CNA_Result;
pub type cna_model_mesh_part_get_vertex_buffer_fn = unsafe extern "C" fn(
    CNA_ModelMeshPartHandle, *mut CNA_Bool, *mut CNA_VertexBufferHandle,
) -> CNA_Result;
pub type cna_model_set_bone_transforms_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *const CNA_Matrix, u64,
) -> CNA_Result;
pub type cna_model_set_gltf_import_report_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *const CNA_GltfImportReportEXT,
) -> CNA_Result;
pub type cna_model_set_material_variant_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, i32,
) -> CNA_Result;

// --- RUST-EXT-015h: the motion sensor, sensor events, and the test backends -
//
// The accelerometer and gyroscope have no `set_test_backend_ext`; their
// deterministic backend is the `_for_tests_ext` set instead. Those routes are
// bound for the same reason the compass's test backend is: without them there
// is no sensor on any verification machine and no way to reach a single line
// past the unsupported refusal.
pub type cna_accelerometer_copy_last_dispatch_exception_message_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_accelerometer_dispatch_to_instances_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *const CNA_AccelerometerHandle, u64, f32, f32, f32,
) -> CNA_Result;
pub type cna_accelerometer_get_dispatch_exception_count_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut i32,
) -> CNA_Result;
pub type cna_accelerometer_get_last_dispatch_exception_message_size_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut u64,
) -> CNA_Result;
pub type cna_accelerometer_get_subsystem_held_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_accelerometer_is_sensor_connected_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_Handle, i64, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_accelerometer_register_started_instance_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle,
) -> CNA_Result;
pub type cna_accelerometer_set_disposal_cleanup_hook_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle, CNA_SensorEventCallback, *mut c_void,
) -> CNA_Result;
pub type cna_accelerometer_set_event_watch_registration_failure_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_Bool,
) -> CNA_Result;
pub type cna_accelerometer_set_started_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_accelerometer_set_supported_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_accelerometer_subscribe_current_value_changed_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle, CNA_AccelerometerReadingCallback, *mut c_void, *mut CNA_SensorEventRegistrationHandle,
) -> CNA_Result;
pub type cna_accelerometer_subscribe_reading_changed_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle, CNA_AccelerometerReadingEventCallback, *mut c_void, *mut CNA_SensorEventRegistrationHandle,
) -> CNA_Result;
pub type cna_accelerometer_unregister_started_instance_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_AccelerometerHandle,
) -> CNA_Result;
pub type cna_compass_inject_calibration_request_ext_fn = unsafe extern "C" fn(
    CNA_CompassHandle,
) -> CNA_Result;
pub type cna_compass_set_test_backend_ext_fn = unsafe extern "C" fn(
    CNA_CompassHandle, CNA_Bool, CNA_Bool,
) -> CNA_Result;
pub type cna_compass_subscribe_calibrate_fn = unsafe extern "C" fn(
    CNA_CompassHandle, CNA_SensorEventCallback, *mut c_void, *mut CNA_SensorEventRegistrationHandle,
) -> CNA_Result;
pub type cna_compass_subscribe_current_value_changed_fn = unsafe extern "C" fn(
    CNA_CompassHandle, CNA_CompassReadingCallback, *mut c_void, *mut CNA_SensorEventRegistrationHandle,
) -> CNA_Result;
pub type cna_gyroscope_copy_last_dispatch_exception_message_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_gyroscope_dispatch_to_instances_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *const CNA_GyroscopeHandle, u64, f32, f32, f32,
) -> CNA_Result;
pub type cna_gyroscope_get_dispatch_exception_count_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut i32,
) -> CNA_Result;
pub type cna_gyroscope_get_last_dispatch_exception_message_size_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut u64,
) -> CNA_Result;
pub type cna_gyroscope_get_subsystem_held_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_GyroscopeHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gyroscope_is_sensor_connected_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_Handle, i64, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_gyroscope_register_started_instance_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_GyroscopeHandle,
) -> CNA_Result;
pub type cna_gyroscope_set_disposal_cleanup_hook_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_GyroscopeHandle, CNA_SensorEventCallback, *mut c_void,
) -> CNA_Result;
pub type cna_gyroscope_set_event_watch_registration_failure_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_Bool,
) -> CNA_Result;
pub type cna_gyroscope_set_started_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_GyroscopeHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_gyroscope_set_supported_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_GyroscopeHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_gyroscope_subscribe_current_value_changed_fn = unsafe extern "C" fn(
    CNA_GyroscopeHandle, CNA_GyroscopeReadingCallback, *mut c_void, *mut CNA_SensorEventRegistrationHandle,
) -> CNA_Result;
pub type cna_gyroscope_unregister_started_instance_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_GyroscopeHandle,
) -> CNA_Result;
pub type cna_motion_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_MotionHandle,
) -> CNA_Result;
pub type cna_motion_destroy_fn = unsafe extern "C" fn(CNA_MotionHandle) -> CNA_Result;
pub type cna_motion_dispose_fn = unsafe extern "C" fn(CNA_MotionHandle) -> CNA_Result;
pub type cna_motion_get_current_value_fn = unsafe extern "C" fn(
    CNA_MotionHandle, *mut CNA_MotionReading,
) -> CNA_Result;
pub type cna_motion_get_is_attitude_north_referenced_ext_fn = unsafe extern "C" fn(
    CNA_MotionHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_motion_get_is_data_valid_fn = unsafe extern "C" fn(
    CNA_MotionHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_motion_get_is_supported_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_motion_get_state_fn = unsafe extern "C" fn(
    CNA_MotionHandle, *mut CNA_SensorState,
) -> CNA_Result;
pub type cna_motion_get_time_between_updates_ticks_fn = unsafe extern "C" fn(
    CNA_MotionHandle, *mut i64,
) -> CNA_Result;
pub type cna_motion_inject_calibration_request_ext_fn = unsafe extern "C" fn(
    CNA_MotionHandle,
) -> CNA_Result;
pub type cna_motion_inject_synthetic_update_ext_fn = unsafe extern "C" fn(
    CNA_MotionHandle, *const CNA_MotionReading,
) -> CNA_Result;
pub type cna_motion_set_test_backend_ext_fn = unsafe extern "C" fn(
    CNA_MotionHandle, CNA_Bool, CNA_Bool, CNA_Bool,
) -> CNA_Result;
pub type cna_motion_set_time_between_updates_ticks_fn = unsafe extern "C" fn(
    CNA_MotionHandle, i64,
) -> CNA_Result;
pub type cna_motion_start_fn = unsafe extern "C" fn(CNA_MotionHandle) -> CNA_Result;
pub type cna_motion_stop_fn = unsafe extern "C" fn(CNA_MotionHandle) -> CNA_Result;
pub type cna_motion_subscribe_calibrate_fn = unsafe extern "C" fn(
    CNA_MotionHandle, CNA_SensorEventCallback, *mut c_void, *mut CNA_SensorEventRegistrationHandle,
) -> CNA_Result;
pub type cna_motion_subscribe_current_value_changed_fn = unsafe extern "C" fn(
    CNA_MotionHandle, CNA_MotionReadingCallback, *mut c_void, *mut CNA_SensorEventRegistrationHandle,
) -> CNA_Result;
pub type cna_sensors_get_last_error_id_ext_fn = unsafe extern "C" fn(
    *mut i32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_sensor_unsubscribe_ext_fn = unsafe extern "C" fn(
    CNA_SensorEventRegistrationHandle,
) -> CNA_Result;

// --- RUST-EXT-015a: CNB's primitive writer, reader and chunk navigation ----
//
// Not generic byte I/O. These carry CNB's canonical encoding and its checks:
// length-prefixed UTF-8 validated against a read limit, a fixed 48-byte
// keyframe layout, seconds refused unless a `TimeSpan` can hold them, and
// integers decomposed byte by byte so a built document does not depend on
// the host's byte order. Reimplementing that over `Vec<u8>` would be a
// second encoder of the same format, and the two could disagree.
pub type cna_cnb_audio_frame_bytes_fn = unsafe extern "C" fn(
    CNA_CnbAudioFormat, u32, *mut u32,
) -> CNA_Result;
pub type cna_cnb_byte_writer_copy_bytes_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_byte_writer_create_fn = unsafe extern "C" fn(
    *mut CNA_CnbByteWriterHandle,
) -> CNA_Result;
pub type cna_cnb_byte_writer_create_from_bytes_fn = unsafe extern "C" fn(
    *const u8, u64, *mut CNA_CnbByteWriterHandle,
) -> CNA_Result;
pub type cna_cnb_byte_writer_destroy_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle,
) -> CNA_Result;
pub type cna_cnb_byte_writer_get_size_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_byte_writer_take_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_byte_writer_write_bytes_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, *const u8, u64,
) -> CNA_Result;
pub type cna_cnb_byte_writer_write_f32_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, f32,
) -> CNA_Result;
pub type cna_cnb_byte_writer_write_f64_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, f64,
) -> CNA_Result;
pub type cna_cnb_byte_writer_write_i32_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, i32,
) -> CNA_Result;
pub type cna_cnb_byte_writer_write_keyframe_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, *const CNA_KeyframeEXT,
) -> CNA_Result;
pub type cna_cnb_byte_writer_write_string_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, CNA_StringView,
) -> CNA_Result;
pub type cna_cnb_byte_writer_write_u16_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, u16,
) -> CNA_Result;
pub type cna_cnb_byte_writer_write_u32_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, u32,
) -> CNA_Result;
pub type cna_cnb_byte_writer_write_u64_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, u64,
) -> CNA_Result;
pub type cna_cnb_byte_writer_write_u8_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, u8,
) -> CNA_Result;
pub type cna_cnb_byte_writer_write_zeros_fn = unsafe extern "C" fn(
    CNA_CnbByteWriterHandle, u64,
) -> CNA_Result;
pub type cna_cnb_checked_add_fn = unsafe extern "C" fn(u64, u64, *mut u64) -> CNA_Result;
pub type cna_cnb_checked_multiply_fn = unsafe extern "C" fn(u64, u64, *mut u64) -> CNA_Result;
pub type cna_cnb_chunk_entry_is_mandatory_fn = unsafe extern "C" fn(
    *const CNA_CnbChunkEntry, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_cnb_copy_audio_format_name_fn = unsafe extern "C" fn(
    CNA_CnbAudioFormat, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_copy_chunk_id_string_fn = unsafe extern "C" fn(
    CNA_CnbChunkId, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_copy_compressed_fn = unsafe extern "C" fn(
    *const u8, u64, CNA_CnbCompression, i32, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_copy_compression_name_fn = unsafe extern "C" fn(
    CNA_CnbCompression, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_copy_decompressed_fn = unsafe extern "C" fn(
    *const u8, u64, CNA_CnbCompression, u64, u64, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_copy_format_magic_fn = unsafe extern "C" fn(*mut u8, u64, *mut u64) -> CNA_Result;
pub type cna_cnb_copy_logical_name_problem_fn = unsafe extern "C" fn(
    CNA_StringView, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_crc32c_fn = unsafe extern "C" fn(*const u8, u64, *mut u32) -> CNA_Result;
pub type cna_cnb_crc32c_continue_fn = unsafe extern "C" fn(
    u32, *const u8, u64, *mut u32,
) -> CNA_Result;
pub type cna_cnb_crc32c_portable_fn = unsafe extern "C" fn(*const u8, u64, *mut u32) -> CNA_Result;
pub type cna_cnb_crc32c_uses_hardware_fn = unsafe extern "C" fn(*mut CNA_Bool) -> CNA_Result;
pub type cna_cnb_document_copy_chunk_data_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, u64, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_document_copy_external_reference_name_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_document_find_all_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, CNA_CnbChunkId, *mut u64, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_document_find_single_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, CNA_CnbChunkId, *mut CNA_Bool, *mut u64,
) -> CNA_Result;
pub type cna_cnb_document_get_chunk_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, u64, *mut CNA_CnbChunkEntry,
) -> CNA_Result;
pub type cna_cnb_document_get_external_reference_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, u64, CNA_StringView, *mut CNA_CnbExternalReference,
) -> CNA_Result;
pub type cna_cnb_document_get_external_reference_count_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_document_get_external_reference_name_size_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_document_get_limits_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut CNA_CnbReadLimits,
) -> CNA_Result;
pub type cna_cnb_document_open_chunk_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, u64, *mut CNA_CnbReaderHandle,
) -> CNA_Result;
pub type cna_cnb_document_read_embedded_texture2d_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, CNA_StringView, *mut CNA_CnbTextureDataHandle,
) -> CNA_Result;
pub type cna_cnb_document_require_asset_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, u32, u32,
) -> CNA_Result;
pub type cna_cnb_document_require_mandatory_chunks_understood_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *const CNA_CnbChunkId, u64,
) -> CNA_Result;
pub type cna_cnb_document_require_single_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, CNA_CnbChunkId, *mut u64,
) -> CNA_Result;
pub type cna_cnb_get_audio_format_name_size_fn = unsafe extern "C" fn(
    CNA_CnbAudioFormat, *mut u64,
) -> CNA_Result;
pub type cna_cnb_get_chunk_id_string_size_fn = unsafe extern "C" fn(
    CNA_CnbChunkId, *mut u64,
) -> CNA_Result;
pub type cna_cnb_get_compressed_byte_count_fn = unsafe extern "C" fn(
    *const u8, u64, CNA_CnbCompression, i32, *mut u64,
) -> CNA_Result;
pub type cna_cnb_get_compression_name_size_fn = unsafe extern "C" fn(
    CNA_CnbCompression, *mut u64,
) -> CNA_Result;
pub type cna_cnb_get_logical_name_problem_size_fn = unsafe extern "C" fn(
    CNA_StringView, *mut u64,
) -> CNA_Result;
pub type cna_cnb_get_texture_level_byte_size_fn = unsafe extern "C" fn(
    CNA_CnbTextureFormat, u32, u32, u32, *mut u64,
) -> CNA_Result;
pub type cna_cnb_has_magic_fn = unsafe extern "C" fn(*const u8, u64, *mut CNA_Bool) -> CNA_Result;
pub type cna_cnb_is_block_compressed_texture_format_fn = unsafe extern "C" fn(
    CNA_CnbTextureFormat, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_cnb_is_compression_supported_fn = unsafe extern "C" fn(
    CNA_CnbCompression, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_cnb_is_known_texture_format_fn = unsafe extern "C" fn(
    u32, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_cnb_is_well_formed_chunk_id_fn = unsafe extern "C" fn(
    CNA_CnbChunkId, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_cnb_is_well_formed_utf8_fn = unsafe extern "C" fn(
    CNA_StringView, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_cnb_make_chunk_id_fn = unsafe extern "C" fn(
    u8, u8, u8, u8, *mut CNA_CnbChunkId,
) -> CNA_Result;
pub type cna_cnb_reader_copy_context_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_reader_copy_string_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_reader_create_fn = unsafe extern "C" fn(
    *const u8, u64, CNA_StringView, *const CNA_CnbReadLimits, *mut CNA_CnbReaderHandle,
) -> CNA_Result;
pub type cna_cnb_reader_destroy_fn = unsafe extern "C" fn(CNA_CnbReaderHandle) -> CNA_Result;
pub type cna_cnb_reader_fail_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, CNA_StringView,
) -> CNA_Result;
pub type cna_cnb_reader_get_context_size_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_reader_get_position_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_reader_get_remaining_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_reader_get_size_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_reader_read_bytes_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, u64, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_reader_read_count_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, u64, CNA_StringView, *mut u32,
) -> CNA_Result;
pub type cna_cnb_reader_read_f32_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut f32,
) -> CNA_Result;
pub type cna_cnb_reader_read_f64_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut f64,
) -> CNA_Result;
pub type cna_cnb_reader_read_i32_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut i32,
) -> CNA_Result;
pub type cna_cnb_reader_read_keyframe_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut CNA_KeyframeEXT,
) -> CNA_Result;
pub type cna_cnb_reader_read_seconds_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, CNA_StringView, *mut f64,
) -> CNA_Result;
pub type cna_cnb_reader_read_string_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_reader_read_u16_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut u16,
) -> CNA_Result;
pub type cna_cnb_reader_read_u32_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut u32,
) -> CNA_Result;
pub type cna_cnb_reader_read_u64_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_reader_read_u8_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle, *mut u8,
) -> CNA_Result;
pub type cna_cnb_reader_require_exhausted_fn = unsafe extern "C" fn(
    CNA_CnbReaderHandle,
) -> CNA_Result;
pub type cna_cnb_reader_skip_fn = unsafe extern "C" fn(CNA_CnbReaderHandle, u64) -> CNA_Result;
pub type cna_cnb_texture_format_from_surface_format_fn = unsafe extern "C" fn(
    CNA_SurfaceFormat, *mut CNA_CnbTextureFormat,
) -> CNA_Result;
pub type cna_cnb_writer_add_external_reference_fn = unsafe extern "C" fn(
    CNA_CnbWriterHandle, *const CNA_CnbExternalReference, CNA_StringView,
) -> CNA_Result;
pub type cna_cnb_writer_append_embedded_texture2d_fn = unsafe extern "C" fn(
    CNA_CnbWriterHandle, CNA_CnbTextureDataHandle, CNA_StringView,
) -> CNA_Result;
pub type cna_cnb_writer_clear_external_references_fn = unsafe extern "C" fn(
    CNA_CnbWriterHandle,
) -> CNA_Result;
pub type cna_cnb_writer_get_limits_fn = unsafe extern "C" fn(
    CNA_CnbWriterHandle, *mut CNA_CnbReadLimits,
) -> CNA_Result;
pub type cna_cnb_writer_get_schema_chunk_count_fn = unsafe extern "C" fn(
    CNA_CnbWriterHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_writer_set_compression_fn = unsafe extern "C" fn(
    CNA_CnbWriterHandle, CNA_CnbCompression, i32,
) -> CNA_Result;
pub type cna_cnb_writer_set_limits_fn = unsafe extern "C" fn(
    CNA_CnbWriterHandle, *const CNA_CnbReadLimits,
) -> CNA_Result;
pub type cna_cnb_writer_write_to_file_fn = unsafe extern "C" fn(
    CNA_CnbWriterHandle, CNA_StringView,
) -> CNA_Result;

// --- RUST-EXT-015a: the CNB model schema, the asset codecs and the .cnj path
//
// The half of cnb.h a game reaches when it builds or reads content rather
// than merely loading it: a model's animations, lights, morph targets and
// skeleton; the per-asset encoders and decoders; and `.cnj`, which is the
// shape CNA's glTF import writes and which this crate has no reader for.
pub type cna_cnb_animation_clip_copy_keyframes_fn = unsafe extern "C" fn(
    CNA_CnbAnimationClipHandle, u64, *mut CNA_KeyframeEXT, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_animation_clip_destroy_fn = unsafe extern "C" fn(
    CNA_CnbAnimationClipHandle,
) -> CNA_Result;
pub type cna_cnb_animation_clip_get_fn = unsafe extern "C" fn(
    CNA_CnbAnimationClipHandle, *mut f64, *mut u64, *mut CNA_ClipTargetSpaceEXT,
) -> CNA_Result;
pub type cna_cnb_animation_clip_get_track_fn = unsafe extern "C" fn(
    CNA_CnbAnimationClipHandle, u64, *mut i32, *mut u64,
) -> CNA_Result;
pub type cna_cnb_build_model_from_cnj_fn = unsafe extern "C" fn(
    CNA_StringView, CNA_StringView, *mut CNA_CnbModelFromCnjHandle,
) -> CNA_Result;
pub type cna_cnb_cnj_result_copy_absorbed_file_fn = unsafe extern "C" fn(
    CNA_CnjToCnbResultHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_cnj_result_copy_asset_type_name_fn = unsafe extern "C" fn(
    CNA_CnjToCnbResultHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_cnj_result_copy_bytes_fn = unsafe extern "C" fn(
    CNA_CnjToCnbResultHandle, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_cnj_result_copy_external_reference_fn = unsafe extern "C" fn(
    CNA_CnjToCnbResultHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_cnj_result_destroy_fn = unsafe extern "C" fn(
    CNA_CnjToCnbResultHandle,
) -> CNA_Result;
pub type cna_cnb_cnj_result_get_absorbed_file_count_fn = unsafe extern "C" fn(
    CNA_CnjToCnbResultHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_cnj_result_get_absorbed_file_size_fn = unsafe extern "C" fn(
    CNA_CnjToCnbResultHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_cnj_result_get_asset_type_id_fn = unsafe extern "C" fn(
    CNA_CnjToCnbResultHandle, *mut u32,
) -> CNA_Result;
pub type cna_cnb_cnj_result_get_asset_type_name_size_fn = unsafe extern "C" fn(
    CNA_CnjToCnbResultHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_cnj_result_get_external_reference_count_fn = unsafe extern "C" fn(
    CNA_CnjToCnbResultHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_cnj_result_get_external_reference_size_fn = unsafe extern "C" fn(
    CNA_CnjToCnbResultHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_compile_cnj_fn = unsafe extern "C" fn(
    CNA_StringView, CNA_StringView, CNA_StringView, *mut CNA_CnjToCnbResultHandle,
) -> CNA_Result;
pub type cna_cnb_decode_animation_clip_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut CNA_CnbAnimationClipHandle,
) -> CNA_Result;
pub type cna_cnb_decode_curve_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut CNA_CurveHandle,
) -> CNA_Result;
pub type cna_cnb_decode_dds_as_texture_cube_fn = unsafe extern "C" fn(
    *const u8, u64, CNA_StringView, *mut CNA_CnbTextureDataHandle,
) -> CNA_Result;
pub type cna_cnb_decode_song_duration_milliseconds_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut u32,
) -> CNA_Result;
pub type cna_cnb_decode_song_name_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_decode_song_name_size_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_decode_song_stream_reference_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_decode_song_stream_reference_size_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_decode_texture3d_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut CNA_CnbTextureDataHandle,
) -> CNA_Result;
pub type cna_cnb_decode_texture_cube_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut CNA_CnbTextureDataHandle,
) -> CNA_Result;
pub type cna_cnb_decode_video_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut CNA_CnbVideoInfo,
) -> CNA_Result;
pub type cna_cnb_decode_video_stream_reference_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_decode_video_stream_reference_size_fn = unsafe extern "C" fn(
    CNA_CnbDocumentHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_decode_wav_as_sound_effect_fn = unsafe extern "C" fn(
    *const u8, u64, CNA_StringView, *mut CNA_CnbSoundEffectDataHandle,
) -> CNA_Result;
pub type cna_cnb_encode_animation_clip_fn = unsafe extern "C" fn(
    *const CNA_AnimationClipEXTDescriptor, CNA_ClipTargetSpaceEXT, CNA_StringView, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_encode_curve_fn = unsafe extern "C" fn(
    CNA_CurveHandle, CNA_StringView, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_encode_song_fn = unsafe extern "C" fn(
    CNA_StringView, CNA_StringView, u32, CNA_StringView, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_encode_texture3d_fn = unsafe extern "C" fn(
    CNA_CnbTextureDataHandle, CNA_StringView, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_encode_texture_cube_fn = unsafe extern "C" fn(
    CNA_CnbTextureDataHandle, CNA_StringView, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_encode_video_fn = unsafe extern "C" fn(
    CNA_StringView, *const CNA_CnbVideoInfo, CNA_StringView, *mut u8, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_import_dds_as_texture_cube_fn = unsafe extern "C" fn(
    CNA_StringView, *mut CNA_CnbTextureDataHandle,
) -> CNA_Result;
pub type cna_cnb_import_image_as_texture2d_fn = unsafe extern "C" fn(
    CNA_StringView, *const CNA_CnbImageImportOptions, *mut CNA_CnbTextureDataHandle,
) -> CNA_Result;
pub type cna_cnb_import_wav_as_sound_effect_fn = unsafe extern "C" fn(
    CNA_StringView, *mut CNA_CnbSoundEffectDataHandle,
) -> CNA_Result;
pub type cna_cnb_model_add_animation_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, CNA_StringView, *const CNA_AnimationClipEXTDescriptor, CNA_ClipTargetSpaceEXT, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_add_light_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, *const CNA_CnbModelLight, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_add_morph_target_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_add_morph_weight_key_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, f64, *const f32, u64, *const f32, u64, *const f32, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_clear_morph_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64,
) -> CNA_Result;
pub type cna_cnb_model_clear_skeleton_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle,
) -> CNA_Result;
pub type cna_cnb_model_copy_animation_keyframes_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, u64, *mut CNA_KeyframeEXT, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_animation_name_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_morph_target_deltas_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, u64, CNA_CnbMorphDeltaStream, *mut f32, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_morph_weight_key_values_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, u64, CNA_CnbMorphKeyStream, *mut f32, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_morph_weights_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut f32, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_part_external_effect_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_skeleton_hierarchy_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, *mut i32, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_copy_skeleton_matrices_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, CNA_CnbSkeletonMatrixSet, *mut f32, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_from_cnj_copy_absorbed_file_fn = unsafe extern "C" fn(
    CNA_CnbModelFromCnjHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_from_cnj_copy_external_reference_fn = unsafe extern "C" fn(
    CNA_CnbModelFromCnjHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_from_cnj_destroy_fn = unsafe extern "C" fn(
    CNA_CnbModelFromCnjHandle,
) -> CNA_Result;
pub type cna_cnb_model_from_cnj_get_absorbed_file_count_fn = unsafe extern "C" fn(
    CNA_CnbModelFromCnjHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_from_cnj_get_absorbed_file_size_fn = unsafe extern "C" fn(
    CNA_CnbModelFromCnjHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_from_cnj_get_external_reference_count_fn = unsafe extern "C" fn(
    CNA_CnbModelFromCnjHandle, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_from_cnj_get_external_reference_size_fn = unsafe extern "C" fn(
    CNA_CnbModelFromCnjHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_from_cnj_take_model_fn = unsafe extern "C" fn(
    CNA_CnbModelFromCnjHandle, *mut CNA_CnbModelDataHandle,
) -> CNA_Result;
pub type cna_cnb_model_get_animation_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut f64, *mut u64, *mut CNA_ClipTargetSpaceEXT,
) -> CNA_Result;
pub type cna_cnb_model_get_animation_name_size_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_get_animation_track_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, u64, *mut i32, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_get_light_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut CNA_CnbModelLight,
) -> CNA_Result;
pub type cna_cnb_model_get_material_sampler_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, u64, *mut CNA_CnbSamplerState,
) -> CNA_Result;
pub type cna_cnb_model_get_material_texture_coordinate_set_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, u64, *mut u8,
) -> CNA_Result;
pub type cna_cnb_model_get_material_texture_transform_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, u64, *mut CNA_CnbTextureTransform,
) -> CNA_Result;
pub type cna_cnb_model_get_morph_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut CNA_CnbMorphInfo,
) -> CNA_Result;
pub type cna_cnb_model_get_morph_weight_key_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, u64, *mut CNA_CnbMorphWeightKeyInfo,
) -> CNA_Result;
pub type cna_cnb_model_get_part_external_effect_size_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_cnb_model_get_skeleton_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, *mut CNA_CnbSkeletonInfo,
) -> CNA_Result;
pub type cna_cnb_model_has_morph_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_cnb_model_set_material_sampler_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, u64, *const CNA_CnbSamplerState,
) -> CNA_Result;
pub type cna_cnb_model_set_material_texture_coordinate_set_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, u64, u8,
) -> CNA_Result;
pub type cna_cnb_model_set_material_texture_transform_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, u64, *const CNA_CnbTextureTransform,
) -> CNA_Result;
pub type cna_cnb_model_set_morph_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *const CNA_CnbMorphInfo,
) -> CNA_Result;
pub type cna_cnb_model_set_morph_target_deltas_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, u64, CNA_CnbMorphDeltaStream, *const f32, u64,
) -> CNA_Result;
pub type cna_cnb_model_set_morph_weights_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *const f32, u64,
) -> CNA_Result;
pub type cna_cnb_model_set_part_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, u64, *const CNA_CnbModelPartInfo,
) -> CNA_Result;
pub type cna_cnb_model_set_skeleton_fn = unsafe extern "C" fn(
    CNA_CnbModelDataHandle, *const i32, u64, *const f32, *const f32, *const f32,
) -> CNA_Result;
pub type cna_cnb_sprite_font_data_set_glyph_fn = unsafe extern "C" fn(
    CNA_CnbSpriteFontDataHandle, u64, *const CNA_SpriteFontGlyph,
) -> CNA_Result;
pub type cna_cnb_texture_data_add_representation_fn = unsafe extern "C" fn(
    CNA_CnbTextureDataHandle, CNA_CnbTextureFormat, *mut u64,
) -> CNA_Result;
pub type cna_cnb_texture_data_create_fn = unsafe extern "C" fn(
    u32, u32, u32, u32, u32, *mut CNA_CnbTextureDataHandle,
) -> CNA_Result;
pub type cna_cnb_texture_data_select_representation_fn = unsafe extern "C" fn(
    CNA_CnbTextureDataHandle, CNA_CnbTextureFormatSupportedFn, *mut c_void, *mut CNA_Bool, *mut u64,
) -> CNA_Result;
pub type cna_cnb_texture_data_set_level_fn = unsafe extern "C" fn(
    CNA_CnbTextureDataHandle, u64, u64, *const u8, u64,
) -> CNA_Result;

// --- RUST-EXT-015a: the curve marshalling bridge -----------------------------
//
// Just enough of curve.h to hand a Rust `Curve` to CNB's codec and take one
// back. The arithmetic stays Rust's -- `cna::value::curve` evaluates, loops
// and computes tangents itself, and none of those routes are here. These
// eleven exist because `cnb_decode_curve` answers a `CNA_CurveHandle` and
// `cnb_encode_curve` wants one, and a Curve is one of CNB's eight asset
// types: without them a Rust game cannot load a curve asset at all.
pub type cna_curve_create_fn = unsafe extern "C" fn(*mut CNA_CurveHandle) -> CNA_Result;
pub type cna_curve_destroy_fn = unsafe extern "C" fn(CNA_CurveHandle) -> CNA_Result;
pub type cna_curve_get_keys_fn = unsafe extern "C" fn(
    CNA_CurveHandle, *mut CNA_CurveKeyCollectionHandle,
) -> CNA_Result;
pub type cna_curve_get_pre_loop_fn = unsafe extern "C" fn(
    CNA_CurveHandle, *mut CNA_CurveLoopType,
) -> CNA_Result;
pub type cna_curve_get_post_loop_fn = unsafe extern "C" fn(
    CNA_CurveHandle, *mut CNA_CurveLoopType,
) -> CNA_Result;
pub type cna_curve_set_pre_loop_fn = unsafe extern "C" fn(
    CNA_CurveHandle, CNA_CurveLoopType,
) -> CNA_Result;
pub type cna_curve_set_post_loop_fn = unsafe extern "C" fn(
    CNA_CurveHandle, CNA_CurveLoopType,
) -> CNA_Result;
pub type cna_curve_key_collection_add_fn = unsafe extern "C" fn(
    CNA_CurveKeyCollectionHandle, CNA_CurveKey,
) -> CNA_Result;
pub type cna_curve_key_collection_get_fn = unsafe extern "C" fn(
    CNA_CurveKeyCollectionHandle, i32, *mut CNA_CurveKey,
) -> CNA_Result;
pub type cna_curve_key_collection_get_count_fn = unsafe extern "C" fn(
    CNA_CurveKeyCollectionHandle, *mut u64,
) -> CNA_Result;
pub type cna_curve_key_collection_destroy_fn = unsafe extern "C" fn(
    CNA_CurveKeyCollectionHandle,
) -> CNA_Result;

// --- RUST-EXT-015c: the rest of effects.h -----------------------------------
//
// CNA's own `ShaderEffect` -- source in, uniforms set by name -- plus the
// reflection constructors, the PBR texture slots, and the stock effects'
// remaining accessors.
pub type cna_alpha_test_effect_get_texture_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_basic_effect_get_texture_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_color_matrix_effect_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_color_matrix_effect_get_matrix_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_ColorMatrix4x4,
) -> CNA_Result;
pub type cna_color_matrix_effect_get_offset_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Vector4,
) -> CNA_Result;
pub type cna_color_matrix_effect_reset_fn = unsafe extern "C" fn(CNA_EffectHandle) -> CNA_Result;
pub type cna_color_matrix_effect_set_grayscale_fn = unsafe extern "C" fn(
    CNA_EffectHandle,
) -> CNA_Result;
pub type cna_color_matrix_effect_set_matrix_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_ColorMatrix4x4,
) -> CNA_Result;
pub type cna_color_matrix_effect_set_offset_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_Vector4,
) -> CNA_Result;
pub type cna_content_manager_load_effect_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_StringView, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_dual_texture_effect_get_texture_fn = unsafe extern "C" fn(
    CNA_EffectHandle, u32, *mut CNA_Bool, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_effect_copy_fragment_source_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_effect_copy_vertex_source_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_effect_get_fragment_source_byte_count_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut u64,
) -> CNA_Result;
pub type cna_effect_get_graphics_device_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_effect_get_is_compiled_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_effect_get_vertex_source_byte_count_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut u64,
) -> CNA_Result;
pub type cna_effect_has_renderer_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_effect_is_exact_stock_sprite_effect_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_effect_material_get_retained_parameter_texture_count_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut u64,
) -> CNA_Result;
pub type cna_effect_material_retain_parameter_texture_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_EffectTextureType, CNA_Handle,
) -> CNA_Result;
pub type cna_effect_pass_get_index_ext_fn = unsafe extern "C" fn(
    CNA_EffectPassHandle, *mut u32,
) -> CNA_Result;
pub type cna_effect_technique_get_identity_fn = unsafe extern "C" fn(
    CNA_EffectTechniqueHandle, *mut u64,
) -> CNA_Result;
pub type cna_effect_technique_get_index_ext_fn = unsafe extern "C" fn(
    CNA_EffectTechniqueHandle, *mut u32,
) -> CNA_Result;
pub type cna_environment_map_effect_get_environment_map_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_environment_map_effect_get_texture_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_effect_get_encode_output_to_srgb_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_effect_get_specular_color_factor_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Vector3,
) -> CNA_Result;
pub type cna_pbr_effect_get_texture_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_PbrTextureSlot, *mut CNA_Bool, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_effect_get_texture_coordinate_set_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_PbrTextureSlot, *mut i32,
) -> CNA_Result;
pub type cna_pbr_effect_get_texture_is_srgb_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_PbrTextureSlot, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_effect_get_texture_transform_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_PbrTextureSlot, *mut CNA_TextureTransformEXT,
) -> CNA_Result;
pub type cna_pbr_effect_set_encode_output_to_srgb_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_effect_set_specular_color_factor_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_Vector3,
) -> CNA_Result;
pub type cna_pbr_effect_set_texture_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_PbrTextureSlot, CNA_Handle,
) -> CNA_Result;
pub type cna_pbr_effect_set_texture_coordinate_set_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_PbrTextureSlot, i32,
) -> CNA_Result;
pub type cna_pbr_effect_set_texture_is_srgb_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_PbrTextureSlot, CNA_Bool,
) -> CNA_Result;
pub type cna_pbr_effect_set_texture_transform_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_PbrTextureSlot, *const CNA_TextureTransformEXT,
) -> CNA_Result;
pub type cna_shader_effect_copy_compile_error_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_shader_effect_create_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_StringView, CNA_StringView, *mut CNA_EffectHandle,
) -> CNA_Result;
pub type cna_shader_effect_declare_uniform_block_ext_fn = unsafe extern "C" fn(
    CNA_EffectHandle, i32, *const CNA_StringView, *const i32, u64,
) -> CNA_Result;
pub type cna_shader_effect_get_projection_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_shader_effect_get_view_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_shader_effect_get_world_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Matrix,
) -> CNA_Result;
pub type cna_shader_effect_has_renderer_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_shader_effect_is_valid_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_shader_effect_set_projection_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_Matrix,
) -> CNA_Result;
pub type cna_shader_effect_set_texture2d_fn = unsafe extern "C" fn(
    CNA_EffectHandle, i32, CNA_Handle,
) -> CNA_Result;
pub type cna_shader_effect_set_texture3d_fn = unsafe extern "C" fn(
    CNA_EffectHandle, i32, CNA_Handle,
) -> CNA_Result;
pub type cna_shader_effect_set_texture_cube_fn = unsafe extern "C" fn(
    CNA_EffectHandle, i32, CNA_Handle,
) -> CNA_Result;
pub type cna_shader_effect_set_uniform_float_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_StringView, f32,
) -> CNA_Result;
pub type cna_shader_effect_set_uniform_float_array_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_StringView, *const f32, u64,
) -> CNA_Result;
pub type cna_shader_effect_set_uniform_int32_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_StringView, i32,
) -> CNA_Result;
pub type cna_shader_effect_set_uniform_mat4_array_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_StringView, *const f32, i32,
) -> CNA_Result;
pub type cna_shader_effect_set_uniform_matrix_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_StringView, CNA_Matrix,
) -> CNA_Result;
pub type cna_shader_effect_set_uniform_vec3_array_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_StringView, *const f32, i32,
) -> CNA_Result;
pub type cna_shader_effect_set_uniform_vector2_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_StringView, CNA_Vector2,
) -> CNA_Result;
pub type cna_shader_effect_set_uniform_vector2_array_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_StringView, *const CNA_Vector2, u64,
) -> CNA_Result;
pub type cna_shader_effect_set_uniform_vector3_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_StringView, CNA_Vector3,
) -> CNA_Result;
pub type cna_shader_effect_set_uniform_vector4_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_StringView, CNA_Vector4,
) -> CNA_Result;
pub type cna_shader_effect_set_view_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_Matrix,
) -> CNA_Result;
pub type cna_shader_effect_set_world_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_Matrix,
) -> CNA_Result;
pub type cna_skinned_effect_get_texture_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool, *mut CNA_Handle,
) -> CNA_Result;
pub type cna_skinned_effect_get_vertex_color_enabled_fn = unsafe extern "C" fn(
    CNA_EffectHandle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_skinned_effect_set_vertex_color_enabled_fn = unsafe extern "C" fn(
    CNA_EffectHandle, CNA_Bool,
) -> CNA_Result;
pub type cna_sprite_effect_create_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_EffectHandle,
) -> CNA_Result;

// --- RUST-EXT-015a: the content Tag a processor wrote --------------------
//
// A tagged value store, which is what makes it safely projectable: a caller
// asks an entry's kind and CNA answers; the Rust side then picks the
// destination type from *that* answer rather than from a caller's claim, so
// there is no offset or size for a caller to get wrong.
pub type cna_object_dictionary_ext_contains_key_fn = unsafe extern "C" fn(
    CNA_ObjectDictionaryHandle, CNA_StringView, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_object_dictionary_ext_copy_array_fn = unsafe extern "C" fn(
    CNA_ObjectDictionaryHandle, CNA_StringView, CNA_ObjectDictionaryValueKind, *mut c_void, u64, *mut u64,
) -> CNA_Result;
pub type cna_object_dictionary_ext_copy_key_at_fn = unsafe extern "C" fn(
    CNA_ObjectDictionaryHandle, u64, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_object_dictionary_ext_copy_runtime_type_name_fn = unsafe extern "C" fn(
    CNA_ObjectDictionaryHandle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_object_dictionary_ext_copy_string_fn = unsafe extern "C" fn(
    CNA_ObjectDictionaryHandle, CNA_StringView, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_object_dictionary_ext_copy_value_fn = unsafe extern "C" fn(
    CNA_ObjectDictionaryHandle, CNA_StringView, CNA_ObjectDictionaryValueKind, *mut c_void, u64,
) -> CNA_Result;
pub type cna_object_dictionary_ext_destroy_fn = unsafe extern "C" fn(
    CNA_ObjectDictionaryHandle,
) -> CNA_Result;
pub type cna_object_dictionary_ext_get_count_fn = unsafe extern "C" fn(
    CNA_ObjectDictionaryHandle, *mut u64,
) -> CNA_Result;
pub type cna_object_dictionary_ext_get_entry_fn = unsafe extern "C" fn(
    CNA_ObjectDictionaryHandle, CNA_StringView, *mut CNA_ObjectDictionaryEntry,
) -> CNA_Result;
pub type cna_object_dictionary_ext_get_foreign_object_fn = unsafe extern "C" fn(
    CNA_ObjectDictionaryHandle, CNA_StringView, *mut *mut c_void,
) -> CNA_Result;
pub type cna_object_dictionary_ext_get_key_size_at_fn = unsafe extern "C" fn(
    CNA_ObjectDictionaryHandle, u64, *mut u64,
) -> CNA_Result;
pub type cna_object_dictionary_ext_get_runtime_type_name_size_fn = unsafe extern "C" fn(
    CNA_ObjectDictionaryHandle, *mut u64,
) -> CNA_Result;
pub type cna_object_dictionary_ext_get_string_size_fn = unsafe extern "C" fn(
    CNA_ObjectDictionaryHandle, CNA_StringView, *mut u64,
) -> CNA_Result;
pub type cna_content_manager_load_object_dictionary_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_StringView, *mut CNA_ObjectDictionaryHandle,
) -> CNA_Result;
pub type cna_model_get_content_tag_dictionary_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut CNA_Bool, *mut CNA_ObjectDictionaryHandle,
) -> CNA_Result;
pub type cna_model_get_content_tag_foreign_object_ext_fn = unsafe extern "C" fn(
    CNA_ModelHandle, *mut CNA_Bool, *mut *mut c_void,
) -> CNA_Result;

// --- RUST-EXT-015d: the desktop devices, and their test backends ------------
//
// A system tray, message boxes, file dialogs, a vibration motor and a URL
// launcher. Every one of them ships a substitute backend and a test log,
// which is what makes them qualifiable on a machine with no tray, no
// gamepad and no desktop session at all.
pub type cna_devices_clipboard_set_text_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_StringView, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_environment_get_device_type_fn = unsafe extern "C" fn(
    *mut CNA_DeviceType,
) -> CNA_Result;
pub type cna_file_dialog_get_is_supported_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_file_dialog_set_test_backend_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_Bool, *const CNA_StringView, u64,
) -> CNA_Result;
pub type cna_file_dialog_show_open_file_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_FileDialogResultCallback, *mut c_void, *const CNA_FileDialogFilter, u64, CNA_StringView, CNA_Bool,
) -> CNA_Result;
pub type cna_file_dialog_show_open_folder_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_FileDialogResultCallback, *mut c_void, CNA_StringView, CNA_Bool,
) -> CNA_Result;
pub type cna_file_dialog_show_save_file_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_FileDialogResultCallback, *mut c_void, *const CNA_FileDialogFilter, u64, CNA_StringView,
) -> CNA_Result;
pub type cna_message_box_get_is_supported_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_message_box_get_test_log_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_MessageBoxTestLog,
) -> CNA_Result;
pub type cna_message_box_set_test_backend_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_Bool, i32,
) -> CNA_Result;
pub type cna_message_box_show_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_MessageBoxType, CNA_StringView, CNA_StringView, *const CNA_StringView, u64, *mut i32,
) -> CNA_Result;
pub type cna_message_box_show_simple_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_MessageBoxType, CNA_StringView, CNA_StringView,
) -> CNA_Result;
pub type cna_system_tray_add_entry_fn = unsafe extern "C" fn(
    CNA_SystemTrayHandle, CNA_StringView, CNA_Bool, CNA_Bool, CNA_Bool, CNA_TrayEntryClickCallback, *mut c_void, *mut u64,
) -> CNA_Result;
pub type cna_system_tray_click_entry_for_tests_ext_fn = unsafe extern "C" fn(
    CNA_SystemTrayHandle, u64,
) -> CNA_Result;
pub type cna_system_tray_create_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_StringView, *mut CNA_SystemTrayHandle,
) -> CNA_Result;
pub type cna_system_tray_create_with_test_backend_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_StringView, *mut CNA_SystemTrayHandle,
) -> CNA_Result;
pub type cna_system_tray_destroy_fn = unsafe extern "C" fn(CNA_SystemTrayHandle) -> CNA_Result;
pub type cna_system_tray_get_entry_checked_fn = unsafe extern "C" fn(
    CNA_SystemTrayHandle, u64, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_system_tray_get_entry_enabled_fn = unsafe extern "C" fn(
    CNA_SystemTrayHandle, u64, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_system_tray_get_is_supported_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_system_tray_set_entry_checked_fn = unsafe extern "C" fn(
    CNA_SystemTrayHandle, u64, CNA_Bool,
) -> CNA_Result;
pub type cna_system_tray_set_entry_enabled_fn = unsafe extern "C" fn(
    CNA_SystemTrayHandle, u64, CNA_Bool,
) -> CNA_Result;
pub type cna_system_tray_set_entry_label_fn = unsafe extern "C" fn(
    CNA_SystemTrayHandle, u64, CNA_StringView,
) -> CNA_Result;
pub type cna_system_tray_set_tooltip_fn = unsafe extern "C" fn(
    CNA_SystemTrayHandle, CNA_StringView,
) -> CNA_Result;
pub type cna_url_launcher_open_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_StringView, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_vibrate_controller_copy_device_name_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut c_char, u64, *mut u64,
) -> CNA_Result;
pub type cna_vibrate_controller_get_device_name_size_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut u64,
) -> CNA_Result;
pub type cna_vibrate_controller_get_is_supported_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_Bool,
) -> CNA_Result;
pub type cna_vibrate_controller_get_test_log_ext_fn = unsafe extern "C" fn(
    CNA_Handle, *mut CNA_VibrationTestLog,
) -> CNA_Result;
pub type cna_vibrate_controller_set_test_backend_ext_fn = unsafe extern "C" fn(
    CNA_Handle, CNA_Bool, CNA_Bool, CNA_StringView,
) -> CNA_Result;
pub type cna_vibrate_controller_start_fn = unsafe extern "C" fn(CNA_Handle, i64) -> CNA_Result;
pub type cna_vibrate_controller_start_left_right_ext_fn = unsafe extern "C" fn(
    CNA_Handle, f32, f32, i64,
) -> CNA_Result;
pub type cna_vibrate_controller_start_with_intensity_ext_fn = unsafe extern "C" fn(
    CNA_Handle, i64, f32,
) -> CNA_Result;
pub type cna_vibrate_controller_stop_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
