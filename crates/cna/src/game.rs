use std::time::Duration;

use crate::error::{CnaError, Result};

/// Timing information supplied to one update or draw callback.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameTime {
    /// Total time since the game loop started.
    pub total: Duration,
    /// Time since the previous update.
    pub elapsed: Duration,
    /// Whether the fixed-step loop is falling behind.
    pub running_slowly: bool,
}

/// Receives lifecycle callbacks from CNA's native game loop.
#[allow(non_snake_case)]
pub trait Game {
    /// Initializes game-owned state.
    fn Initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Loads game content.
    fn LoadContent(&mut self) -> Result<()> {
        Ok(())
    }

    /// Advances game state.
    fn Update(&mut self, _time: &GameTime) -> Result<()> {
        Ok(())
    }

    /// Draws one frame.
    fn Draw(&mut self, _time: &GameTime) -> Result<()> {
        Ok(())
    }

    /// Releases game-owned content during shutdown.
    fn UnloadContent(&mut self) -> Result<()> {
        Ok(())
    }

    fn Exit(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Hands a game to CNA's native loop.
///
/// # Errors
///
/// Returns [`CnaError::NativeUnavailable`] while the canonical ABI is absent,
/// and will later propagate failures reported by the native game loop.
pub fn run<G: Game>(_game: G) -> Result<()> {
    Err(CnaError::NativeUnavailable)
}
