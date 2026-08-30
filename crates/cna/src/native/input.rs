//! Native input polling calls.

use cna_sys as sys;

use crate::error::Result;

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
