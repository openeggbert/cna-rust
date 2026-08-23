//! Raw declarations for the reviewed CNA C ABI runtime/2D slice.
//!
//! These layouts and function-pointer types are derived from CNA's canonical
//! `modules/c-api/include/CNA/C` headers at ABI 0.7.0. The crate deliberately
//! does not add linker directives: the safe crate resolves a user-selected CNA
//! shared library at runtime and checks its ABI version before loading symbols.

#![no_std]
#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_void};

pub const BINDINGS_AVAILABLE: bool = true;
pub const CNA_ABI_VERSION: u32 = 0x0000_0700;
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
pub const CNA_DISPLAY_ORIENTATION_DEFAULT: CNA_DisplayOrientation = 0;
pub const CNA_DISPLAY_ORIENTATION_LANDSCAPE_LEFT: CNA_DisplayOrientation = 1;
pub const CNA_DISPLAY_ORIENTATION_LANDSCAPE_RIGHT: CNA_DisplayOrientation = 2;
pub const CNA_DISPLAY_ORIENTATION_PORTRAIT: CNA_DisplayOrientation = 4;
pub const CNA_GAME_EVENT_ACTIVATED: CNA_GameEvent = 0;
pub const CNA_GAME_EVENT_DEACTIVATED: CNA_GameEvent = 1;
pub const CNA_GAME_EVENT_DISPOSED: CNA_GameEvent = 2;
pub const CNA_GAME_EVENT_EXITING: CNA_GameEvent = 3;
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

pub type CNA_Result = u32;
pub type CNA_Bool = u8;
pub type CNA_DisplayOrientation = u32;
pub type CNA_GameEvent = u32;
pub type CNA_GameWindowEvent = u32;
pub type CNA_GraphicsDeviceStatus = u32;
pub type CNA_GraphicsProfile = u32;
pub type CNA_PresentInterval = u32;
pub type CNA_DepthFormat = u32;
pub type CNA_RenderTargetUsage = u32;
pub type CNA_CubeMapFace = u32;
pub type CNA_RenderTargetKind = u32;
pub type CNA_ShaderStage = u32;
pub type CNA_NativeHandleValue = u64;
pub type CNA_Handle = u64;
pub type CNA_GameEventRegistrationHandle = CNA_Handle;
pub type CNA_VertexDeclarationHandle = CNA_Handle;
pub type CNA_VertexBufferHandle = CNA_Handle;
pub type CNA_IndexBufferHandle = CNA_Handle;
pub type CNA_EffectHandle = CNA_Handle;
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
pub type CNA_SurfaceFormat = u32;
pub type CNA_TextureDataType = u32;
pub type CNA_TextureImageFormat = u32;
pub type CNA_BufferUsage = u32;
pub type CNA_IndexElementSize = u32;
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
pub type CNA_EffectParameterClass = u32;
pub type CNA_EffectParameterType = u32;
pub type CNA_EffectValueType = u32;
pub type CNA_EffectTextureType = u32;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CNA_StringView {
    pub data: *const c_char,
    pub byte_length: u64,
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

pub type cna_get_abi_version_fn = unsafe extern "C" fn() -> u32;
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
    }
}
