//! Native game lifecycle calls.

use core::ffi::c_void;

use cna_sys as sys;

use crate::error::Result;

use super::Native;

impl Native {
    pub(crate) fn create_game(
        &self,
        info: &sys::CNA_GameCreateInfo,
        handle: &mut sys::CNA_Handle,
    ) -> Result<()> {
        #[cfg(feature = "native-fault-injection")]
        super::fault::check("game-create")?;
        // SAFETY: references provide initialized, live input/output objects for
        // the synchronous call; nested pointers are owned by the caller.
        self.check(unsafe { (self.game_create)(info, handle) })
    }

    pub(crate) fn set_game_frame_hooks(
        &self,
        game: sys::CNA_Handle,
        hooks: &sys::CNA_GameFrameHooks,
    ) -> Result<()> {
        // SAFETY: the internal caller supplies its live owned game handle and
        // CNA copies this fully initialized versioned structure.
        self.check(unsafe { (self.game_set_frame_hooks)(game, hooks) })
    }

    pub(crate) fn run_game(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: only the runner owns and uses this handle on its native thread.
        self.check(unsafe { (self.game_run)(game) })
    }

    pub(crate) fn run_game_one_frame(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the host owns the live handle and invokes the frame on its
        // creation thread, outside a lifecycle callback.
        self.check(unsafe { (self.game_run_one_frame)(game) })
    }

    pub(crate) fn request_game_exit(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: GameContext guarantees a live callback-scoped game handle.
        self.check(unsafe { (self.game_request_exit)(game) })
    }

    pub(crate) fn game_is_active(&self, game: sys::CNA_Handle) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the output is initialized writable storage and the state
        // retains a live parent handle while this synchronous call runs.
        self.check(unsafe { (self.game_get_is_active)(game, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn game_is_mouse_visible(&self, game: sys::CNA_Handle) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: see `game_is_active`.
        self.check(unsafe { (self.game_get_is_mouse_visible)(game, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn set_game_is_mouse_visible(
        &self,
        game: sys::CNA_Handle,
        value: bool,
    ) -> Result<()> {
        // SAFETY: the handle is retained by the active game state.
        self.check(unsafe {
            (self.game_set_is_mouse_visible)(
                game,
                if value { sys::CNA_TRUE } else { sys::CNA_FALSE },
            )
        })
    }

    pub(crate) fn game_is_fixed_time_step(&self, game: sys::CNA_Handle) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: see `game_is_active`.
        self.check(unsafe { (self.game_get_is_fixed_time_step)(game, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn set_game_is_fixed_time_step(
        &self,
        game: sys::CNA_Handle,
        value: bool,
    ) -> Result<()> {
        // SAFETY: the handle is retained by the active game state.
        self.check(unsafe {
            (self.game_set_is_fixed_time_step)(
                game,
                if value { sys::CNA_TRUE } else { sys::CNA_FALSE },
            )
        })
    }

    pub(crate) fn game_target_elapsed_time_ticks(&self, game: sys::CNA_Handle) -> Result<i64> {
        let mut value = 0;
        // SAFETY: see `game_is_active`.
        self.check(unsafe { (self.game_get_target_elapsed_time_ticks)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_game_target_elapsed_time_ticks(
        &self,
        game: sys::CNA_Handle,
        value: i64,
    ) -> Result<()> {
        // SAFETY: the handle is retained by the active game state.
        self.check(unsafe { (self.game_set_target_elapsed_time_ticks)(game, value) })
    }

    pub(crate) fn game_inactive_sleep_time_ticks(&self, game: sys::CNA_Handle) -> Result<i64> {
        let mut value = 0;
        // SAFETY: see `game_is_active`.
        self.check(unsafe { (self.game_get_inactive_sleep_time_ticks)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_game_inactive_sleep_time_ticks(
        &self,
        game: sys::CNA_Handle,
        value: i64,
    ) -> Result<()> {
        // SAFETY: the handle is retained by the active game state.
        self.check(unsafe { (self.game_set_inactive_sleep_time_ticks)(game, value) })
    }

    pub(crate) fn reset_game_elapsed_time(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the handle is retained by the active game state.
        self.check(unsafe { (self.game_reset_elapsed_time)(game) })
    }

    pub(crate) fn suppress_game_draw(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the handle is retained by the active game state.
        self.check(unsafe { (self.game_suppress_draw)(game) })
    }

    pub(crate) fn tick_game(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the handle is retained by the active game state. CNA rejects
        // callback re-entry, and that error is propagated unchanged.
        self.check(unsafe { (self.game_tick)(game) })
    }

    pub(crate) fn set_game_window_title(&self, game: sys::CNA_Handle, title: &str) -> Result<()> {
        let title = sys::CNA_StringView {
            data: title.as_ptr().cast(),
            byte_length: title.len() as u64,
        };
        // SAFETY: the string view borrows UTF-8 bytes for this synchronous call.
        self.check(unsafe { (self.game_set_window_title)(game, title) })
    }

    pub(crate) fn subscribe_game_event(
        &self,
        game: sys::CNA_Handle,
        event: sys::CNA_GameEvent,
        callback: unsafe extern "C" fn(*mut c_void),
        context: *mut c_void,
    ) -> Result<sys::CNA_GameEventRegistrationHandle> {
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: the host keeps `context` live until it synchronously
        // unsubscribes the returned registration before destroying the game.
        self.check(unsafe {
            (self.game_subscribe)(game, event, Some(callback), context, &mut registration)
        })?;
        Ok(registration)
    }

    pub(crate) fn subscribe_game_window_event(
        &self,
        game: sys::CNA_Handle,
        event: sys::CNA_GameWindowEvent,
        callback: unsafe extern "C" fn(*mut c_void),
        context: *mut c_void,
    ) -> Result<sys::CNA_GameEventRegistrationHandle> {
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: identical registration lifetime to `subscribe_game_event`.
        self.check(unsafe {
            (self.game_window_subscribe)(game, event, Some(callback), context, &mut registration)
        })?;
        Ok(registration)
    }

    pub(crate) fn unsubscribe_game_event(
        &self,
        registration: sys::CNA_GameEventRegistrationHandle,
    ) -> Result<()> {
        // SAFETY: the host releases each owned registration exactly once.
        self.check(unsafe { (self.game_unsubscribe)(registration) })
    }

    pub(crate) fn destroy_game(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: only the runner calls this for its exactly-once owned handle.
        self.check(unsafe { (self.game_destroy)(game) })?;
        #[cfg(feature = "native-fault-injection")]
        super::fault::check("game-destroy")?;
        Ok(())
    }
}
