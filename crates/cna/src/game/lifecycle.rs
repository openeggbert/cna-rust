#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use crate::error::Result;

use super::{GameContext, GameTime};

/// User lifecycle contract composed with CNA's host-owned XNA game state.
#[allow(non_snake_case)]
pub trait Game {
    fn Dispose(&mut self) {}

    fn BeginRun(&mut self) {}

    fn EndRun(&mut self) {}

    fn BeginDraw(&mut self) -> bool {
        true
    }

    fn EndDraw(&mut self) {}

    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let _ = game;
        Ok(())
    }

    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let _ = game;
        Ok(())
    }

    fn Update(&mut self, game: &mut GameContext<'_>, time: &GameTime) -> Result<()> {
        let _ = (game, time);
        Ok(())
    }

    fn Draw(&mut self, game: &mut GameContext<'_>, time: &GameTime) -> Result<()> {
        let _ = (game, time);
        Ok(())
    }

    fn UnloadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let _ = game;
        Ok(())
    }

    fn OnExiting(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let _ = game;
        Ok(())
    }
}
