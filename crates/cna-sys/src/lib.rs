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
pub const CNA_GRAPHICS_CAPABILITY_THREE_D: CNA_GraphicsCapability = 0;
pub const CNA_GRAPHICS_CAPABILITY_DEPTH_STENCIL_BUFFER: CNA_GraphicsCapability = 1;
pub const CNA_SPRITE_SORT_MODE_DEFERRED: CNA_SpriteSortMode = 0;
pub const CNA_SPRITE_EFFECT_NONE: CNA_SpriteEffects = 0;
pub const CNA_KEY_ESCAPE: CNA_Key = 27;

pub type CNA_Result = u32;
pub type CNA_Bool = u8;
pub type CNA_Handle = u64;
pub type CNA_ErrorCategory = u32;
pub type CNA_GraphicsCapability = u32;
pub type CNA_GraphicsCapabilityFlags = u64;
pub type CNA_GraphicsRendererType = u32;
pub type CNA_SurfaceFormat = u32;
pub type CNA_SpriteSortMode = u32;
pub type CNA_SpriteEffects = u32;
pub type CNA_Key = u32;

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
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
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
pub struct CNA_Texture2DDecodeInfo {
    pub struct_size: u32,
    pub struct_version: u32,
    pub width: u32,
    pub height: u32,
    pub zoom: CNA_Bool,
    pub reserved: [u8; 7],
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
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CNA_KeyboardState {
    pub struct_size: u32,
    pub struct_version: u32,
    pub pressed_key_words: [u64; 4],
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
pub type cna_game_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_game_get_graphics_device_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Handle) -> CNA_Result;
pub type cna_graphics_device_get_viewport_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Viewport) -> CNA_Result;
pub type cna_graphics_device_clear_rgba_fn =
    unsafe extern "C" fn(CNA_Handle, f32, f32, f32, f32) -> CNA_Result;
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
pub type cna_texture2d_get_info_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Texture2DInfo) -> CNA_Result;
pub type cna_texture2d_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_sprite_batch_create_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_Handle) -> CNA_Result;
pub type cna_sprite_batch_begin_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_SpriteBatchBeginInfo) -> CNA_Result;
pub type cna_sprite_batch_submit_many_fn =
    unsafe extern "C" fn(CNA_Handle, *const CNA_SpriteCommand, u64) -> CNA_Result;
pub type cna_sprite_batch_end_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_sprite_batch_destroy_fn = unsafe extern "C" fn(CNA_Handle) -> CNA_Result;
pub type cna_keyboard_get_state_fn =
    unsafe extern "C" fn(CNA_Handle, *mut CNA_KeyboardState) -> CNA_Result;
pub type cna_keyboard_state_is_key_down_fn =
    unsafe extern "C" fn(*const CNA_KeyboardState, CNA_Key, *mut CNA_Bool) -> CNA_Result;

#[cfg(test)]
mod layout_tests {
    use super::*;
    use core::mem::{align_of, size_of};

    #[test]
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
                size_of::<CNA_Texture2DDecodeInfo>(),
                align_of::<CNA_Texture2DDecodeInfo>()
            ),
            (24, 4)
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
                size_of::<CNA_KeyboardState>(),
                align_of::<CNA_KeyboardState>()
            ),
            (40, 8)
        );
    }
}
