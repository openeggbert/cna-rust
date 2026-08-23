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
}
