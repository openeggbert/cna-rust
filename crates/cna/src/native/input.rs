//! Native input polling calls.

use cna_sys as sys;

use crate::error::{CnaError, ErrorCategory, Result};

use super::Native;

impl Native {
    pub(crate) fn keyboard_state(
        &self,
        game: sys::CNA_Handle,
        state: &mut sys::CNA_KeyboardState,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game and output reference are live.
        self.check(unsafe { (self.keyboard_get_state)(game, state) })
    }

    pub(crate) fn keyboard_state_for_player(
        &self,
        game: sys::CNA_Handle,
        player_index: sys::CNA_PlayerIndex,
        state: &mut sys::CNA_KeyboardState,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game and output reference are live.
        self.check(unsafe { (self.keyboard_get_state_for_player)(game, player_index, state) })
    }

    pub(crate) fn mouse_state(
        &self,
        game: sys::CNA_Handle,
        state: &mut sys::CNA_MouseState,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game and versioned output are live.
        self.check(unsafe { (self.mouse_get_state)(game, state) })
    }

    pub(crate) fn mouse_window_handle(
        &self,
        game: sys::CNA_Handle,
        window: &mut u64,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game and opaque-value output are live.
        self.check(unsafe { (self.mouse_get_window_handle)(game, window) })
    }

    pub(crate) fn set_mouse_window_handle(&self, game: sys::CNA_Handle, window: u64) -> Result<()> {
        // SAFETY: CNA treats the value as opaque and never dereferences it.
        self.check(unsafe { (self.mouse_set_window_handle)(game, window) })
    }

    pub(crate) fn set_mouse_position(&self, game: sys::CNA_Handle, x: i32, y: i32) -> Result<()> {
        // SAFETY: the callback-scoped game is live and coordinates are values.
        self.check(unsafe { (self.mouse_set_position)(game, x, y) })
    }

    pub(crate) fn gamepad_state(
        &self,
        game: sys::CNA_Handle,
        player: sys::CNA_PlayerIndex,
        state: &mut sys::CNA_GamePadState,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game and versioned output are live.
        self.check(unsafe { (self.gamepad_get_state)(game, player, state) })
    }

    pub(crate) fn gamepad_state_with_dead_zone(
        &self,
        game: sys::CNA_Handle,
        player: sys::CNA_PlayerIndex,
        dead_zone: sys::CNA_GamePadDeadZone,
        state: &mut sys::CNA_GamePadState,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game and versioned output are live.
        self.check(unsafe {
            (self.gamepad_get_state_with_dead_zone)(game, player, dead_zone, state)
        })
    }

    pub(crate) fn gamepad_capabilities(
        &self,
        game: sys::CNA_Handle,
        player: sys::CNA_PlayerIndex,
        capabilities: &mut sys::CNA_GamePadCapabilities,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game and versioned output are live.
        self.check(unsafe { (self.gamepad_get_capabilities)(game, player, capabilities) })
    }

    pub(crate) fn set_gamepad_vibration(
        &self,
        game: sys::CNA_Handle,
        player: sys::CNA_PlayerIndex,
        left: f32,
        right: f32,
        applied: &mut sys::CNA_Bool,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game and output are live.
        self.check(unsafe { (self.gamepad_set_vibration)(game, player, left, right, applied) })
    }

    pub(crate) fn touch_capabilities(
        &self,
        game: sys::CNA_Handle,
        capabilities: &mut sys::CNA_TouchCapabilities,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game and versioned output are live.
        self.check(unsafe { (self.touch_get_capabilities)(game, capabilities) })
    }

    pub(crate) fn touch_state(
        &self,
        game: sys::CNA_Handle,
        state: &mut sys::CNA_TouchState,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game and versioned output are live.
        self.check(unsafe { (self.touch_get_state)(game, state) })
    }

    pub(crate) fn touch_display_width(&self, game: sys::CNA_Handle) -> Result<i32> {
        let mut value = 0;
        // SAFETY: the callback-scoped game and output pointer are live.
        self.check(unsafe { (self.touch_panel_get_display_width)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_touch_display_width(&self, game: sys::CNA_Handle, value: i32) -> Result<()> {
        // SAFETY: the callback-scoped game is live and the width is passed by value.
        self.check(unsafe { (self.touch_panel_set_display_width)(game, value) })
    }

    pub(crate) fn touch_display_height(&self, game: sys::CNA_Handle) -> Result<i32> {
        let mut value = 0;
        // SAFETY: the callback-scoped game and output pointer are live.
        self.check(unsafe { (self.touch_panel_get_display_height)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_touch_display_height(&self, game: sys::CNA_Handle, value: i32) -> Result<()> {
        // SAFETY: the callback-scoped game is live and the height is passed by value.
        self.check(unsafe { (self.touch_panel_set_display_height)(game, value) })
    }

    pub(crate) fn touch_display_orientation(
        &self,
        game: sys::CNA_Handle,
    ) -> Result<sys::CNA_DisplayOrientation> {
        let mut value = 0;
        // SAFETY: the callback-scoped game and output pointer are live.
        self.check(unsafe { (self.touch_panel_get_display_orientation)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_touch_display_orientation(
        &self,
        game: sys::CNA_Handle,
        value: sys::CNA_DisplayOrientation,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game is live and CNA validates the flag bits.
        self.check(unsafe { (self.touch_panel_set_display_orientation)(game, value) })
    }

    pub(crate) fn enabled_gestures(&self, game: sys::CNA_Handle) -> Result<sys::CNA_GestureType> {
        let mut value = 0;
        // SAFETY: the callback-scoped game and output pointer are live.
        self.check(unsafe { (self.touch_panel_get_enabled_gestures)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_enabled_gestures(
        &self,
        game: sys::CNA_Handle,
        value: sys::CNA_GestureType,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game is live and CNA validates the gesture bits.
        self.check(unsafe { (self.touch_panel_set_enabled_gestures)(game, value) })
    }

    pub(crate) fn is_gesture_available(&self, game: sys::CNA_Handle) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the callback-scoped game and output pointer are live.
        self.check(unsafe { (self.touch_panel_get_is_gesture_available)(game, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn touch_window_handle(&self, game: sys::CNA_Handle) -> Result<u64> {
        let mut value = 0;
        // SAFETY: the callback-scoped game and opaque-value output are live.
        self.check(unsafe { (self.touch_panel_get_window_handle)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_touch_window_handle(&self, game: sys::CNA_Handle, value: u64) -> Result<()> {
        // SAFETY: CNA treats the value as opaque and never dereferences it.
        self.check(unsafe { (self.touch_panel_set_window_handle)(game, value) })
    }

    pub(crate) fn read_gesture(
        &self,
        game: sys::CNA_Handle,
        sample: &mut sys::CNA_GestureSample,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game and versioned output are live.
        self.check(unsafe { (self.touch_panel_read_gesture)(game, sample) })
    }
}

/// `input_keyboard.h`'s layout-aware routes, and the snapshot's own text.
///
/// Everything with `_ext` needs a live game handle because it asks the
/// *current* keyboard layout a question: which key a physical scancode
/// produces right now, and what the platform calls each. None of it is
/// derivable from the `Keys` enum, which is why these are bound rather than
/// answered in Rust the way the snapshot's own set operations are.
impl Native {
    pub(crate) fn keyboard_state_string(
        &self,
        state: &sys::CNA_KeyboardState,
    ) -> Result<String> {
        let mut count = 0;
        // SAFETY: the state is a live borrow and the output is a local.
        self.check(unsafe { (self.keyboard_state_get_string_size)(state, &mut count) })?;
        let length = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("keyboard snapshot text is too large"))?;
        let mut bytes = vec![0_u8; length];
        let mut copied = count;
        // SAFETY: the destination has exactly `count` writable bytes.
        self.check(unsafe {
            (self.keyboard_state_copy_string)(
                state,
                bytes.as_mut_ptr().cast(),
                count,
                &mut copied,
            )
        })?;
        bytes.truncate(usize::try_from(copied).unwrap_or(length));
        String::from_utf8(bytes).map_err(|_| CnaError::Native {
            code: sys::CNA_RESULT_ENCODING,
            category: ErrorCategory::None,
            message: "CNA returned invalid UTF-8 keyboard snapshot text".to_owned(),
        })
    }

    pub(crate) fn keyboard_key_from_scancode(
        &self,
        game: sys::CNA_Handle,
        scancode: sys::CNA_Key,
    ) -> Result<sys::CNA_Key> {
        let mut value = 0;
        // SAFETY: the game handle is live for the call and the output is local.
        self.check(unsafe {
            (self.keyboard_get_key_from_scancode_ext)(game, scancode, &mut value)
        })?;
        Ok(value)
    }

    pub(crate) fn keyboard_modifiers(
        &self,
        game: sys::CNA_Handle,
    ) -> Result<sys::CNA_KeyModifiers> {
        let mut value = sys::CNA_KEY_MODIFIER_NONE;
        // SAFETY: the game handle is live for the call and the output is local.
        self.check(unsafe { (self.keyboard_get_mod_state_ext)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn keyboard_scancode_name(
        &self,
        game: sys::CNA_Handle,
        key: sys::CNA_Key,
    ) -> Result<String> {
        self.keyboard_name(
            game,
            key,
            self.keyboard_get_scancode_name_size_ext,
            self.keyboard_copy_scancode_name_ext,
        )
    }

    pub(crate) fn keyboard_key_name(
        &self,
        game: sys::CNA_Handle,
        key: sys::CNA_Key,
    ) -> Result<String> {
        self.keyboard_name(
            game,
            key,
            self.keyboard_get_key_name_size_ext,
            self.keyboard_copy_key_name_ext,
        )
    }

    fn keyboard_name(
        &self,
        game: sys::CNA_Handle,
        key: sys::CNA_Key,
        size_fn: unsafe extern "C" fn(
            sys::CNA_Handle,
            sys::CNA_Key,
            *mut u64,
        ) -> sys::CNA_Result,
        copy_fn: unsafe extern "C" fn(
            sys::CNA_Handle,
            sys::CNA_Key,
            *mut core::ffi::c_char,
            u64,
            *mut u64,
        ) -> sys::CNA_Result,
    ) -> Result<String> {
        let mut count = 0;
        // SAFETY: the game handle is live and the output is a local.
        self.check(unsafe { size_fn(game, key, &mut count) })?;
        let length = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("a key name is too large"))?;
        let mut bytes = vec![0_u8; length];
        let mut copied = count;
        // SAFETY: the destination has exactly `count` writable bytes.
        self.check(unsafe {
            copy_fn(game, key, bytes.as_mut_ptr().cast(), count, &mut copied)
        })?;
        bytes.truncate(usize::try_from(copied).unwrap_or(length));
        String::from_utf8(bytes).map_err(|_| CnaError::Native {
            code: sys::CNA_RESULT_ENCODING,
            category: ErrorCategory::None,
            message: "CNA returned an invalid UTF-8 key name".to_owned(),
        })
    }

    pub(crate) fn keyboard_scancode_from_name(
        &self,
        game: sys::CNA_Handle,
        name: &str,
    ) -> Result<sys::CNA_Key> {
        let view = sys::CNA_StringView {
            data: name.as_ptr().cast(),
            byte_length: name.len() as u64,
        };
        let mut value = 0;
        // SAFETY: the name outlives the call, which is all the view borrows.
        self.check(unsafe {
            (self.keyboard_get_scancode_from_name_ext)(game, view, &mut value)
        })?;
        Ok(value)
    }

    pub(crate) fn keyboard_key_from_name(
        &self,
        game: sys::CNA_Handle,
        name: &str,
    ) -> Result<sys::CNA_Key> {
        let view = sys::CNA_StringView {
            data: name.as_ptr().cast(),
            byte_length: name.len() as u64,
        };
        let mut value = 0;
        // SAFETY: the name outlives the call, which is all the view borrows.
        self.check(unsafe { (self.keyboard_get_key_from_name_ext)(game, view, &mut value) })?;
        Ok(value)
    }
}

/// `input_mouse.h`'s `_ext` routes, and `input.h`'s dead-zone algorithm.
///
/// The mouse routes reach past XNA. XNA's `Mouse` knows the cursor's position
/// inside the game window and nothing else; these read and move it in *desktop*
/// coordinates, turn relative (pointer-lock) mode on, capture the pointer, and
/// carry a clicked event with the test hooks to drive it.
///
/// The clicked subscription is process-wide rather than per game -- it takes no
/// game handle -- so the registration outlives any one game and is the caller's
/// to withdraw.
impl Native {
    pub(crate) fn mouse_relative_mode(&self, game: sys::CNA_Handle) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the game handle is live for the call; the output is a local.
        self.check(unsafe {
            (self.mouse_get_is_relative_mouse_mode_ext)(game, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn set_mouse_relative_mode(
        &self,
        game: sys::CNA_Handle,
        enabled: bool,
    ) -> Result<()> {
        let value = if enabled { sys::CNA_TRUE } else { sys::CNA_FALSE };
        // SAFETY: the game handle is live for the call.
        self.check(unsafe { (self.mouse_set_is_relative_mouse_mode_ext)(game, value) })
    }

    /// Returns whether the backend accepted the request, which is not the same
    /// as whether the call succeeded.
    pub(crate) fn set_mouse_capture(
        &self,
        game: sys::CNA_Handle,
        enabled: bool,
    ) -> Result<bool> {
        let value = if enabled { sys::CNA_TRUE } else { sys::CNA_FALSE };
        let mut applied = sys::CNA_FALSE;
        // SAFETY: the game handle is live for the call; the output is a local.
        self.check(unsafe { (self.mouse_set_capture_ext)(game, value, &mut applied) })?;
        Ok(applied != sys::CNA_FALSE)
    }

    pub(crate) fn mouse_global_position(&self, game: sys::CNA_Handle) -> Result<(i32, i32)> {
        let (mut x, mut y) = (0, 0);
        // SAFETY: the game handle is live for the call; both outputs are locals.
        self.check(unsafe { (self.mouse_get_global_position_ext)(game, &mut x, &mut y) })?;
        Ok((x, y))
    }

    pub(crate) fn warp_mouse_global(
        &self,
        game: sys::CNA_Handle,
        x: i32,
        y: i32,
    ) -> Result<bool> {
        let mut applied = sys::CNA_FALSE;
        // SAFETY: the game handle is live for the call; the output is a local.
        self.check(unsafe { (self.mouse_warp_global_ext)(game, x, y, &mut applied) })?;
        Ok(applied != sys::CNA_FALSE)
    }

    pub(crate) fn subscribe_mouse_clicked(
        &self,
        callback: sys::CNA_MouseClickedCallback,
        context: *mut core::ffi::c_void,
    ) -> Result<sys::CNA_MouseEventRegistrationHandle> {
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: the caller keeps `context` alive until it withdraws this
        // registration, which is the contract the safe layer upholds.
        self.check(unsafe {
            (self.mouse_subscribe_clicked_ext)(callback, context, &mut registration)
        })?;
        Ok(registration)
    }

    pub(crate) fn unsubscribe_mouse_clicked(
        &self,
        registration: sys::CNA_MouseEventRegistrationHandle,
    ) -> Result<()> {
        // SAFETY: the registration came from the subscribe above and is
        // withdrawn exactly once.
        self.check(unsafe { (self.mouse_unsubscribe_clicked_ext)(registration) })
    }

    pub(crate) fn raise_mouse_clicked(
        &self,
        game: sys::CNA_Handle,
        button: i32,
    ) -> Result<()> {
        // SAFETY: the game handle is live for the call.
        self.check(unsafe { (self.mouse_raise_clicked_ext)(game, button) })
    }

    pub(crate) fn reset_mouse_for_tests(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the game handle is live for the call.
        self.check(unsafe { (self.mouse_reset_for_tests_ext)(game) })
    }

    /// CNA's canonical dead-zone and clamping rules over caller-supplied raw
    /// analog values.
    pub(crate) fn apply_gamepad_dead_zone(
        &self,
        mode: sys::CNA_GamePadDeadZone,
        raw: &sys::CNA_GamePadAnalogState,
    ) -> Result<sys::CNA_GamePadAnalogState> {
        let mut processed = *raw;
        // SAFETY: the input is a live borrow and the output is a live local;
        // neither is retained past the call.
        self.check(unsafe { (self.gamepad_apply_dead_zone)(mode, raw, &mut processed) })?;
        Ok(processed)
    }
}
