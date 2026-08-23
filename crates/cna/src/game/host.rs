#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use core::ffi::c_void;
use core::mem::size_of;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::graphics::GraphicsDevice;
use crate::native::Native;

use super::{Game, GameContext, GameTime};

struct CallbackState<G> {
    game: G,
    native: Arc<Native>,
    callback_error: Option<CnaError>,
    frame_limit: Option<u64>,
    drawn_frames: u64,
    device: Option<GraphicsDevice>,
}

#[derive(Clone, Copy)]
enum Lifecycle {
    Initialize,
    BeginRun,
    EndRun,
    LoadContent,
    Update,
    Draw,
    EndDraw,
    UnloadContent,
    Exiting,
}

/// Runs a game through CNA's native loop until it exits.
pub fn run<G: Game>(game: G) -> Result<()> {
    run_inner(game, None)
}

/// Runs a real CNA game and requests exit after exactly `frames` successful draws.
///
/// This is a binding-level deterministic test utility, not an XNA member.
pub fn run_for_frames<G: Game>(game: G, frames: u64) -> Result<()> {
    if frames == 0 {
        return Err(CnaError::InvalidInput(
            "frame limit must be greater than zero",
        ));
    }
    run_inner(game, Some(frames))
}

fn run_inner<G: Game>(game: G, frame_limit: Option<u64>) -> Result<()> {
    let native = Native::load()?;
    let mut state = Box::new(CallbackState {
        game,
        native: Arc::clone(&native),
        callback_error: None,
        frame_limit,
        drawn_frames: 0,
        device: None,
    });
    let context = core::ptr::addr_of_mut!(*state).cast::<c_void>();

    let callbacks = sys::CNA_GameCallbacks {
        struct_size: size_of::<sys::CNA_GameCallbacks>() as u32,
        struct_version: 1,
        load_content: Some(callback::<G, { Lifecycle::LoadContent as u8 }>),
        update: Some(callback::<G, { Lifecycle::Update as u8 }>),
        draw: Some(callback::<G, { Lifecycle::Draw as u8 }>),
        unload_content: Some(callback::<G, { Lifecycle::UnloadContent as u8 }>),
        exiting: Some(callback::<G, { Lifecycle::Exiting as u8 }>),
        context,
    };
    let title = b"CNA Rust\0";
    let create_info = sys::CNA_GameCreateInfo {
        struct_size: size_of::<sys::CNA_GameCreateInfo>() as u32,
        struct_version: 1,
        is_fixed_time_step: sys::CNA_TRUE,
        reserved: [0; 7],
        target_elapsed_time_ticks: 166_667,
        window_title: sys::CNA_StringView {
            data: title.as_ptr().cast(),
            byte_length: (title.len() - 1) as u64,
        },
        callbacks: &callbacks,
    };
    let mut handle = sys::CNA_INVALID_HANDLE;
    native.create_game(&create_info, &mut handle)?;

    let hooks = sys::CNA_GameFrameHooks {
        struct_size: size_of::<sys::CNA_GameFrameHooks>() as u32,
        struct_version: 1,
        initialize: Some(callback::<G, { Lifecycle::Initialize as u8 }>),
        begin_run: Some(callback::<G, { Lifecycle::BeginRun as u8 }>),
        end_run: Some(callback::<G, { Lifecycle::EndRun as u8 }>),
        begin_draw: Some(begin_draw_callback::<G>),
        end_draw: Some(callback::<G, { Lifecycle::EndDraw as u8 }>),
        context,
    };
    if let Err(error) = native.set_game_frame_hooks(handle, &hooks) {
        let _ = native.destroy_game(handle);
        return Err(error);
    }

    let run_result = native.run_game(handle);

    // ABI 0.7 checks for owned children before native Shutdown sends the
    // user's Exiting/UnloadContent lifecycle callbacks. Release registered
    // native children here without synthesizing a second user callback.
    let cleanup_result = state
        .device
        .as_ref()
        .map_or(Ok(()), GraphicsDevice::dispose_resources);

    let destroy_result = native.destroy_game(handle);
    // The host is ending after this destroy attempt even if CNA reports a
    // failure. Invalidate Rust wrappers unconditionally so no safe operation
    // can reach a possibly-live native parent that Rust can no longer drive.
    if let Some(device) = &state.device {
        device.invalidate();
    }

    // XNA disposes the managed Game object after its exiting/unload lifecycle.
    // This is a separate user-visible notification from native child cleanup.
    state.game.Dispose();

    if let Some(error) = state.callback_error.take() {
        return Err(error);
    }
    run_result?;
    cleanup_result?;
    destroy_result
}

unsafe extern "C" fn callback<G: Game, const LIFECYCLE: u8>(
    game_handle: sys::CNA_Handle,
    time: *const sys::CNA_GameTime,
    context: *mut c_void,
    _error: *mut sys::CNA_CallbackError,
) -> sys::CNA_Result {
    // SAFETY: `run_inner` passes a stable boxed `CallbackState<G>` pointer and
    // CNA invokes callbacks only before the enclosing run/destroy completes.
    let state = unsafe { &mut *context.cast::<CallbackState<G>>() };
    if state.device.is_none() {
        state.device = Some(GraphicsDevice::bind(&state.native, game_handle));
    }
    let device = state.device.as_ref().expect("device initialized above");
    if let Err(error) = device.enter_callback() {
        state.callback_error = Some(error);
        return sys::CNA_RESULT_CALLBACK;
    }
    let mut game_context = GameContext {
        native: &state.native,
        handle: game_handle,
        device,
    };
    let lifecycle = match LIFECYCLE {
        value if value == Lifecycle::Initialize as u8 => Lifecycle::Initialize,
        value if value == Lifecycle::BeginRun as u8 => Lifecycle::BeginRun,
        value if value == Lifecycle::EndRun as u8 => Lifecycle::EndRun,
        value if value == Lifecycle::LoadContent as u8 => Lifecycle::LoadContent,
        value if value == Lifecycle::Update as u8 => Lifecycle::Update,
        value if value == Lifecycle::Draw as u8 => Lifecycle::Draw,
        value if value == Lifecycle::EndDraw as u8 => Lifecycle::EndDraw,
        value if value == Lifecycle::UnloadContent as u8 => Lifecycle::UnloadContent,
        _ => Lifecycle::Exiting,
    };
    let game_time = read_time(time);
    let result = catch_unwind(AssertUnwindSafe(|| match lifecycle {
        Lifecycle::Initialize => state.game.Initialize(&mut game_context),
        Lifecycle::BeginRun => {
            state.game.BeginRun();
            Ok(())
        }
        Lifecycle::EndRun => {
            state.game.EndRun();
            Ok(())
        }
        Lifecycle::LoadContent => state.game.LoadContent(&mut game_context),
        Lifecycle::Update => state.game.Update(&mut game_context, &game_time),
        Lifecycle::Draw => {
            state.game.Draw(&mut game_context, &game_time)?;
            state.drawn_frames += 1;
            if state.frame_limit == Some(state.drawn_frames) {
                game_context.Exit()?;
            }
            Ok(())
        }
        Lifecycle::EndDraw => {
            state.game.EndDraw();
            Ok(())
        }
        Lifecycle::UnloadContent => state.game.UnloadContent(&mut game_context),
        Lifecycle::Exiting => state.game.OnExiting(&mut game_context),
    }));
    device.leave_callback();

    match result {
        Ok(Ok(())) => sys::CNA_RESULT_SUCCESS,
        Ok(Err(error)) => {
            state.callback_error = Some(error);
            sys::CNA_RESULT_CALLBACK
        }
        Err(_) => {
            state.callback_error = Some(CnaError::Callback(
                "Rust panic was contained at the FFI boundary".to_owned(),
            ));
            sys::CNA_RESULT_CALLBACK
        }
    }
}

unsafe extern "C" fn begin_draw_callback<G: Game>(
    game_handle: sys::CNA_Handle,
    _time: *const sys::CNA_GameTime,
    context: *mut c_void,
    should_draw: *mut sys::CNA_Bool,
    _error: *mut sys::CNA_CallbackError,
) -> sys::CNA_Result {
    if should_draw.is_null() {
        return sys::CNA_RESULT_INVALID_ARGUMENT;
    }
    // SAFETY: the callback context and output pointer are owned by CNA for the
    // duration of this synchronous call.
    let state = unsafe { &mut *context.cast::<CallbackState<G>>() };
    if state.device.is_none() {
        state.device = Some(GraphicsDevice::bind(&state.native, game_handle));
    }
    let device = state.device.as_ref().expect("device initialized above");
    if let Err(error) = device.enter_callback() {
        state.callback_error = Some(error);
        return sys::CNA_RESULT_CALLBACK;
    }
    let result = catch_unwind(AssertUnwindSafe(|| state.game.BeginDraw()));
    device.leave_callback();
    let Ok(value) = result else {
        state.callback_error = Some(CnaError::Callback(
            "Rust panic was contained at the FFI boundary".to_owned(),
        ));
        return sys::CNA_RESULT_CALLBACK;
    };
    // SAFETY: non-null was checked above and CNA owns this output.
    unsafe { *should_draw = if value { sys::CNA_TRUE } else { sys::CNA_FALSE } };
    sys::CNA_RESULT_SUCCESS
}

fn read_time(time: *const sys::CNA_GameTime) -> GameTime {
    if time.is_null() {
        GameTime::new()
    } else {
        // SAFETY: CNA documents the pointer as non-null and callback-scoped for
        // update/draw callbacks; it is read and copied synchronously.
        GameTime::from_native(unsafe { &*time })
    }
}
