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

struct CallbackState<'game, G> {
    game: &'game mut G,
    native: Arc<Native>,
    callback_error: Option<CnaError>,
    frame_limit: Option<u64>,
    drawn_frames: u64,
    device: Option<GraphicsDevice>,
    event_registrations: Vec<sys::CNA_GameEventRegistrationHandle>,
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

#[derive(Clone, Copy)]
enum RunMode {
    Continuous,
    OneFrame,
    Frames(u64),
}

/// Runs a game through CNA's native loop until it exits.
pub fn run<G: Game>(mut game: G) -> Result<()> {
    run_inner(&mut game, RunMode::Continuous)
}

/// Runs a real CNA game and requests exit after exactly `frames` successful draws.
///
/// This is a binding-level deterministic test utility, not an XNA member.
pub fn run_for_frames<G: Game>(mut game: G, frames: u64) -> Result<()> {
    if frames == 0 {
        return Err(CnaError::InvalidInput(
            "frame limit must be greater than zero",
        ));
    }
    run_inner(&mut game, RunMode::Frames(frames))
}

pub(super) fn run_borrowed<G: Game>(game: &mut G) -> Result<()> {
    run_inner(game, RunMode::Continuous)
}

pub(super) fn run_one_frame_borrowed<G: Game>(game: &mut G) -> Result<()> {
    run_inner(game, RunMode::OneFrame)
}

#[allow(clippy::too_many_lines)]
fn run_inner<G: Game>(game: &mut G, mode: RunMode) -> Result<()> {
    let native = Native::load()?;
    let mut state = Box::new(CallbackState {
        game,
        native: Arc::clone(&native),
        callback_error: None,
        frame_limit: match mode {
            RunMode::Frames(frames) => Some(frames),
            RunMode::Continuous | RunMode::OneFrame => None,
        },
        drawn_frames: 0,
        device: None,
        event_registrations: Vec::new(),
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
    let (is_fixed_time_step, target_elapsed_time_ticks, title) =
        state.game.game_state().create_configuration();
    let create_info = sys::CNA_GameCreateInfo {
        struct_size: size_of::<sys::CNA_GameCreateInfo>() as u32,
        struct_version: 1,
        is_fixed_time_step: if is_fixed_time_step {
            sys::CNA_TRUE
        } else {
            sys::CNA_FALSE
        },
        reserved: [0; 7],
        target_elapsed_time_ticks,
        window_title: sys::CNA_StringView {
            data: title.as_ptr().cast(),
            byte_length: title.len() as u64,
        },
        callbacks: &callbacks,
    };
    let mut handle = sys::CNA_INVALID_HANDLE;
    native.create_game(&create_info, &mut handle)?;
    let device = GraphicsDevice::bind(&state.native, handle);
    state.device = Some(device.clone());
    if let Err(error) = state
        .game
        .game_state()
        .attach(&state.native, handle, &device)
    {
        device.invalidate();
        let _ = native.destroy_game(handle);
        return Err(error);
    }
    if let Err(error) = subscribe_events(&mut state, handle, context) {
        let _ = unsubscribe_events(&mut state);
        let _ = state.game.game_state().dispose_graphics_device_manager();
        state.game.game_state().detach();
        device.invalidate();
        let _ = native.destroy_game(handle);
        return Err(error);
    }

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
        let _ = state.game.game_state().dispose_graphics_device_manager();
        state.game.game_state().detach();
        device.invalidate();
        let _ = native.destroy_game(handle);
        return Err(error);
    }

    let run_result = match mode {
        RunMode::OneFrame => native.run_game_one_frame(handle),
        RunMode::Continuous | RunMode::Frames(_) => native.run_game(handle),
    };

    // The registrations borrow `CallbackState`; detach them before native
    // destruction can invalidate the owned handles or the boxed context.
    let unsubscribe_result = unsubscribe_events(&mut state);
    let manager_cleanup_result = state.game.game_state().dispose_graphics_device_manager();

    // ABI 0.7 checks for owned children before native Shutdown sends the
    // user's Exiting/UnloadContent lifecycle callbacks. Release registered
    // native children here without synthesizing a second user callback.
    let content_cleanup_result = state.game.game_state().cleanup_content();
    let cleanup_result = state
        .device
        .as_ref()
        .map_or(Ok(()), GraphicsDevice::dispose_resources);

    // Keep the durable device identity alive while native destruction delivers
    // CNA's one user-visible UnloadContent callback. Native children have
    // already been released, so the ABI 0.7 ownership precondition is met.
    let destroy_result = native.destroy_game(handle);
    // No native operation is possible after destroy returns (including its
    // failure path). Invalidate exactly once and emit the managed Disposing
    // notification only after the final native lifecycle callback has ended.
    if let Some(device) = &state.device {
        device.invalidate();
    }
    state.game.game_state().detach();

    // XNA disposes the managed Game object after its exiting/unload lifecycle.
    // This is a separate user-visible notification from native child cleanup.
    state.game.Dispose();

    if let Some(error) = state.callback_error.take() {
        return Err(error);
    }
    run_result?;
    unsubscribe_result?;
    manager_cleanup_result?;
    content_cleanup_result?;
    cleanup_result?;
    destroy_result
}

fn subscribe_events<G: Game>(
    state: &mut CallbackState<'_, G>,
    game: sys::CNA_Handle,
    context: *mut c_void,
) -> Result<()> {
    let registration = state.native.subscribe_game_event(
        game,
        sys::CNA_GAME_EVENT_ACTIVATED,
        native_event_callback::<G, 0>,
        context,
    )?;
    state.event_registrations.push(registration);
    let registration = state.native.subscribe_game_event(
        game,
        sys::CNA_GAME_EVENT_DEACTIVATED,
        native_event_callback::<G, 1>,
        context,
    )?;
    state.event_registrations.push(registration);
    let registration = state.native.subscribe_game_window_event(
        game,
        sys::CNA_GAME_WINDOW_EVENT_CLIENT_SIZE_CHANGED,
        native_event_callback::<G, 2>,
        context,
    )?;
    state.event_registrations.push(registration);
    let registration = state.native.subscribe_game_window_event(
        game,
        sys::CNA_GAME_WINDOW_EVENT_ORIENTATION_CHANGED,
        native_event_callback::<G, 3>,
        context,
    )?;
    state.event_registrations.push(registration);
    let registration = state.native.subscribe_game_window_event(
        game,
        sys::CNA_GAME_WINDOW_EVENT_SCREEN_DEVICE_NAME_CHANGED,
        native_event_callback::<G, 4>,
        context,
    )?;
    state.event_registrations.push(registration);
    Ok(())
}

fn unsubscribe_events<G: Game>(state: &mut CallbackState<'_, G>) -> Result<()> {
    let mut first_error = None;
    for registration in state.event_registrations.drain(..).rev() {
        if let Err(error) = state.native.unsubscribe_game_event(registration) {
            if first_error.is_none() {
                first_error = Some(error);
            }
        }
    }
    first_error.map_or(Ok(()), Err)
}

unsafe extern "C" fn native_event_callback<G: Game, const EVENT: u8>(context: *mut c_void) {
    // SAFETY: every subscription receives the same live boxed context as the
    // lifecycle callbacks and is removed before that box can be released.
    let state = unsafe { &mut *context.cast::<CallbackState<'_, G>>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<()> {
        match EVENT {
            0 => {
                state.game.game_state().refresh_native_properties()?;
                state.game.OnActivated(
                    state.game.game_state().as_ref(),
                    crate::extensions::events::EventArgs,
                );
            }
            1 => {
                state.game.game_state().refresh_native_properties()?;
                state.game.OnDeactivated(
                    state.game.game_state().as_ref(),
                    crate::extensions::events::EventArgs,
                );
            }
            2 => state.game.Window().native_client_size_changed()?,
            3 => state.game.Window().native_orientation_changed()?,
            _ => state.game.Window().native_screen_device_name_changed()?,
        }
        Ok(())
    }));
    let error = match result {
        Ok(Ok(())) => return,
        Ok(Err(error)) => error,
        Err(_) => {
            CnaError::Callback("Rust panic was contained in a native event callback".to_owned())
        }
    };
    if state.callback_error.is_none() {
        state.callback_error = Some(error);
    }
    if let Ok(active) = state.game.game_state().active() {
        let _ = active.native.request_game_exit(active.handle);
    }
}

unsafe extern "C" fn callback<G: Game, const LIFECYCLE: u8>(
    game_handle: sys::CNA_Handle,
    time: *const sys::CNA_GameTime,
    context: *mut c_void,
    _error: *mut sys::CNA_CallbackError,
) -> sys::CNA_Result {
    // SAFETY: `run_inner` passes a stable boxed `CallbackState<G>` pointer and
    // CNA invokes callbacks only before the enclosing run/destroy completes.
    let state = unsafe { &mut *context.cast::<CallbackState<'_, G>>() };
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
        Lifecycle::Initialize => {
            state.game.Initialize(&mut game_context)?;
            state.game.game_state().refresh_native_properties()?;
            state.game.game_state().initialize_components();
            Ok(())
        }
        Lifecycle::BeginRun => {
            state.game.BeginRun();
            Ok(())
        }
        Lifecycle::EndRun => {
            state.game.EndRun();
            // CNA retains raw renderer buffer bindings. Clear them while the
            // callback-scoped device handle is still valid so registered
            // buffers can be destroyed safely during host teardown.
            device.unbind_all_render_targets()?;
            device.unbind_all_buffers()?;
            Ok(())
        }
        Lifecycle::LoadContent => state.game.LoadContent(&mut game_context),
        Lifecycle::Update => {
            state.game.Update(&mut game_context, &game_time)?;
            state.game.game_state().update_components(&game_time);
            Ok(())
        }
        Lifecycle::Draw => {
            state.game.Draw(&mut game_context, &game_time)?;
            state.game.game_state().draw_components(&game_time);
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
    _game_handle: sys::CNA_Handle,
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
    let state = unsafe { &mut *context.cast::<CallbackState<'_, G>>() };
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
