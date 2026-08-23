//! Native game lifecycle calls.

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

    pub(crate) fn request_game_exit(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: GameContext guarantees a live callback-scoped game handle.
        self.check(unsafe { (self.game_request_exit)(game) })
    }

    pub(crate) fn destroy_game(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: only the runner calls this for its exactly-once owned handle.
        self.check(unsafe { (self.game_destroy)(game) })?;
        #[cfg(feature = "native-fault-injection")]
        super::fault::check("game-destroy")?;
        Ok(())
    }
}
