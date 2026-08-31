//! Raw declarations for the reviewed CNA C ABI runtime/2D slice.
//!
//! These layouts and function-pointer types are derived from CNA's canonical
//! `modules/c-api/include/CNA/C` headers at ABI 0.20.0. The crate deliberately
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

pub type CNA_HapticDeviceHandle = CNA_Handle;
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
