//! `input_mouse.h`'s `_ext` routes: the desktop, capture, and pointer lock.
//!
//! XNA's `Mouse` knows one thing -- where the cursor is inside the game window.
//! These reach past that:
//!
//! * the cursor in *desktop* coordinates, read and moved, which is what a
//!   multi-monitor setup or a windowed game repositioning itself needs;
//! * relative (pointer-lock) mode, which is how a first-person camera reads
//!   motion once the cursor has nowhere left to travel;
//! * capture, which keeps the pointer's events coming while a drag continues
//!   outside the window;
//! * a clicked event, with the hooks to raise one and to reset the state, so a
//!   click path can be qualified on a machine with no hands on it.
//!
//! Two shapes here are worth reading before use.
//!
//! `set_capture` and `warp_global` return **whether the backend accepted the
//! request**, which is a different question from whether the call succeeded. A
//! platform that has no notion of capture answers `Ok(false)`, not an error,
//! and a caller that treats the `Ok` as "it happened" will be wrong on it.
//!
//! The clicked subscription is **process-wide**. `cna_mouse_subscribe_clicked_ext`
//! takes no game handle, so the registration is not scoped to a game and does
//! not end with one; [`ClickSubscription`] withdraws it on drop.

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::Result;
use crate::input::{GamePadDeadZone, GamePadThumbSticks, GamePadTriggers};
use crate::native::Native;
use crate::value::Vector2;
use crate::Microsoft::Xna::Framework::GameContext;

/// The cursor and pointer behaviour XNA's `Mouse` does not reach.
#[derive(Clone, Copy, Debug)]
pub struct MouseDesktop;

impl MouseDesktop {
    /// Whether relative (pointer-lock) mode is active.
    pub fn is_relative_mode(game: &GameContext<'_>) -> Result<bool> {
        game.native.mouse_relative_mode(game.handle)
    }

    /// Turns relative (pointer-lock) mode on or off.
    pub fn set_relative_mode(game: &GameContext<'_>, enabled: bool) -> Result<()> {
        game.native.set_mouse_relative_mode(game.handle, enabled)
    }

    /// Turns pointer capture on or off.
    ///
    /// Answers whether the backend **accepted** the request. `Ok(false)` means
    /// the call worked and the platform declined; it is not a failure.
    #[must_use = "the backend may decline capture, and the answer says whether it did"]
    pub fn set_capture(game: &GameContext<'_>, enabled: bool) -> Result<bool> {
        game.native.set_mouse_capture(game.handle, enabled)
    }

    /// The cursor position in desktop coordinates.
    pub fn global_position(game: &GameContext<'_>) -> Result<(i32, i32)> {
        game.native.mouse_global_position(game.handle)
    }

    /// Moves the cursor in desktop coordinates.
    ///
    /// Answers whether the backend accepted the request, as [`Self::set_capture`]
    /// does.
    #[must_use = "the backend may decline the warp, and the answer says whether it did"]
    pub fn warp_global(game: &GameContext<'_>, x: i32, y: i32) -> Result<bool> {
        game.native.warp_mouse_global(game.handle, x, y)
    }

    /// Raises the clicked event, for driving a click path in a test.
    pub fn raise_clicked(game: &GameContext<'_>, button: i32) -> Result<()> {
        game.native.raise_mouse_clicked(game.handle, button)
    }

    /// Resets the process-wide mouse state.
    ///
    /// Process-wide, not game-wide: it clears state a previous game left
    /// behind, which is what makes a click test repeatable.
    pub fn reset_for_tests(game: &GameContext<'_>) -> Result<()> {
        game.native.reset_mouse_for_tests(game.handle)
    }

    /// Subscribes to the process-wide clicked event.
    ///
    /// The returned value withdraws the registration when dropped, so it must
    /// be held for as long as the callback should run.
    pub fn on_clicked(
        callback: impl FnMut(i32) + Send + 'static,
    ) -> Result<ClickSubscription> {
        unsafe extern "C" fn trampoline(button: i32, context: *mut core::ffi::c_void) {
            if context.is_null() {
                return;
            }
            // SAFETY: the context is the box the subscription owns and is freed
            // only after the registration naming it has been withdrawn.
            let closure = unsafe { &mut *context.cast::<ClickClosure>() };
            // A panic must not cross back into C, and a click has nowhere to
            // report one.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| closure(button)));
        }

        let native = Native::process()?;
        let boxed: ClickClosure = Box::new(callback);
        let context = Box::into_raw(Box::new(boxed)).cast::<core::ffi::c_void>();
        match native.subscribe_mouse_clicked(Some(trampoline), context) {
            Ok(registration) => Ok(ClickSubscription {
                native,
                registration: Mutex::new(registration),
                callback: Mutex::new(context),
            }),
            Err(error) => {
                // CNA never took the pointer, so this is the only owner left.
                // SAFETY: the box was created immediately above.
                drop(unsafe { Box::from_raw(context.cast::<ClickClosure>()) });
                Err(error)
            }
        }
    }
}

type ClickClosure = Box<dyn FnMut(i32) + Send + 'static>;

/// A live registration on the process-wide mouse clicked event.
///
/// Withdraws itself on drop, in the one order that is safe: the registration is
/// cancelled *before* the boxed closure behind it is freed.
#[must_use = "dropping a ClickSubscription immediately unsubscribes it"]
pub struct ClickSubscription {
    native: Arc<Native>,
    registration: Mutex<sys::CNA_MouseEventRegistrationHandle>,
    callback: Mutex<*mut core::ffi::c_void>,
}

// SAFETY: the pointer is an owned box this value alone frees, and the closure
// behind it is required to be `Send`.
unsafe impl Send for ClickSubscription {}

impl ClickSubscription {
    /// Withdraws the subscription early. Idempotent.
    pub fn unsubscribe(&self) -> Result<()> {
        let mut guard = self
            .registration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let registration = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if registration == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        let result = self.native.unsubscribe_mouse_clicked(registration);
        let mut callback = self
            .callback
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pointer = core::mem::replace(&mut *callback, core::ptr::null_mut());
        if !pointer.is_null() {
            // SAFETY: the pointer came from `Box::into_raw` in `on_clicked`,
            // and the registration naming it is already withdrawn.
            drop(unsafe { Box::from_raw(pointer.cast::<ClickClosure>()) });
        }
        result
    }
}

impl Drop for ClickSubscription {
    fn drop(&mut self) {
        let _ = self.unsubscribe();
    }
}

impl core::fmt::Debug for ClickSubscription {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ClickSubscription")
            .field(
                "registration",
                &*self
                    .registration
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            )
            .finish_non_exhaustive()
    }
}

const fn to_native(value: Vector2) -> sys::CNA_Vector2 {
    sys::CNA_Vector2 {
        x: value.X,
        y: value.Y,
    }
}

const fn from_native(value: sys::CNA_Vector2) -> Vector2 {
    Vector2::from_x_and_y(value.x, value.y)
}

/// Raw analog values, before CNA's dead-zone rules are applied.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RawAnalogState {
    /// Left thumbstick, unclamped.
    pub left_thumb_stick: Vector2,
    /// Right thumbstick, unclamped.
    pub right_thumb_stick: Vector2,
    /// Left trigger, unclamped.
    pub left_trigger: f32,
    /// Right trigger, unclamped.
    pub right_trigger: f32,
}

/// The result of applying a dead-zone mode, in the XNA types.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProcessedAnalogState {
    /// Both sticks, after the mode's rules and XNA's clamping.
    pub thumb_sticks: GamePadThumbSticks,
    /// Both triggers, after the mode's rules and XNA's clamping.
    pub triggers: GamePadTriggers,
}

/// Applies CNA's canonical dead-zone and clamping rules to raw analog values.
///
/// Distinct from reading a pad with a mode, which
/// [`GamePad::GetStateWithPlayerIndexAndDeadZoneMode`] already does: this takes
/// values the caller has in hand -- from a joystick, a replay, a network packet
/// -- and puts them through the same algorithm. The algorithm is CNA's, and
/// this calls it rather than restating it, so the two can never disagree.
///
/// [`GamePad::GetStateWithPlayerIndexAndDeadZoneMode`]: crate::Microsoft::Xna::Framework::Input::GamePad::GetStateWithPlayerIndexAndDeadZoneMode
pub fn apply_dead_zone(
    mode: GamePadDeadZone,
    raw: RawAnalogState,
) -> Result<ProcessedAnalogState> {
    let native = Native::process()?;
    let input = sys::CNA_GamePadAnalogState {
        left_thumb_stick: to_native(raw.left_thumb_stick),
        right_thumb_stick: to_native(raw.right_thumb_stick),
        left_trigger: raw.left_trigger,
        right_trigger: raw.right_trigger,
    };
    let processed = native.apply_gamepad_dead_zone(mode as sys::CNA_GamePadDeadZone, &input)?;
    Ok(ProcessedAnalogState {
        thumb_sticks: GamePadThumbSticks::new(
            from_native(processed.left_thumb_stick),
            from_native(processed.right_thumb_stick),
        ),
        triggers: GamePadTriggers::new(processed.left_trigger, processed.right_trigger),
    })
}
