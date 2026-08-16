use std::time::Duration;

use crate::{CnaError, Result};

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

/// Borrowed access to CNA services during a lifecycle callback.
#[derive(Debug, Default)]
pub struct GameContext {
    _private: (),
}

/// Receives lifecycle callbacks from CNA's native game loop.
pub trait Game {
    /// Initializes game-owned state.
    ///
    /// # Errors
    ///
    /// Returns an error when initialization cannot complete.
    fn initialize(&mut self, _context: &mut GameContext) -> Result<()> {
        Ok(())
    }

    /// Loads game content.
    ///
    /// # Errors
    ///
    /// Returns an error when content cannot be loaded.
    fn load_content(&mut self, _context: &mut GameContext) -> Result<()> {
        Ok(())
    }

    /// Advances game state.
    ///
    /// # Errors
    ///
    /// Returns an error when the update cannot complete.
    fn update(&mut self, _context: &mut GameContext, _time: &GameTime) -> Result<()> {
        Ok(())
    }

    /// Draws one frame.
    ///
    /// # Errors
    ///
    /// Returns an error when the frame cannot be drawn.
    fn draw(&mut self, _context: &mut GameContext, _time: &GameTime) -> Result<()> {
        Ok(())
    }

    /// Releases game-owned content during shutdown.
    ///
    /// # Errors
    ///
    /// Returns an error when cleanup cannot complete.
    fn unload_content(&mut self, _context: &mut GameContext) -> Result<()> {
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
