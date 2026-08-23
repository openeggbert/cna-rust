#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use std::any::Any;
use std::error::Error;

use crate::error::Result;
use crate::extensions::events::{EventArgs, EventHandler};
use crate::graphics::GraphicsDevice;

use super::{
    GameComponentCollection, GameContext, GameServiceContainer, GameStateAccess, GameTime,
    GameWindow, LaunchParameters, TimeSpan,
};

/// User lifecycle contract composed with CNA's host-owned XNA game state.
#[allow(non_snake_case)]
pub trait Game: GameStateAccess {
    #[must_use]
    fn new() -> Self
    where
        Self: Sized + Default,
    {
        Self::default()
    }

    fn LaunchParameters(&self) -> &LaunchParameters {
        self.game_state().LaunchParameters()
    }

    fn Components(&self) -> &GameComponentCollection {
        self.game_state().Components()
    }

    fn Services(&self) -> &GameServiceContainer {
        self.game_state().Services()
    }

    fn InactiveSleepTime(&self) -> TimeSpan {
        self.game_state().InactiveSleepTime()
    }

    fn SetInactiveSleepTime(&mut self, value: TimeSpan) -> Result<()> {
        self.game_state().SetInactiveSleepTime(value)
    }

    fn IsMouseVisible(&self) -> bool {
        self.game_state().IsMouseVisible()
    }

    fn SetIsMouseVisible(&mut self, value: bool) -> Result<()> {
        self.game_state().SetIsMouseVisible(value)
    }

    fn TargetElapsedTime(&self) -> TimeSpan {
        self.game_state().TargetElapsedTime()
    }

    fn SetTargetElapsedTime(&mut self, value: TimeSpan) -> Result<()> {
        self.game_state().SetTargetElapsedTime(value)
    }

    fn IsFixedTimeStep(&self) -> bool {
        self.game_state().IsFixedTimeStep()
    }

    fn SetIsFixedTimeStep(&mut self, value: bool) -> Result<()> {
        self.game_state().SetIsFixedTimeStep(value)
    }

    fn Window(&self) -> &GameWindow {
        self.game_state().Window()
    }

    fn IsActive(&self) -> bool {
        self.game_state().IsActive()
    }

    fn GraphicsDevice(&self) -> Result<&GraphicsDevice> {
        self.game_state().GraphicsDevice()
    }

    fn AddActivatedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.game_state().add_activated(handler)
    }

    fn RemoveActivatedHandler(&self, registration: u64) -> bool {
        self.game_state().remove_activated(registration)
    }

    fn AddDeactivatedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.game_state().add_deactivated(handler)
    }

    fn RemoveDeactivatedHandler(&self, registration: u64) -> bool {
        self.game_state().remove_deactivated(registration)
    }

    fn AddExitingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.game_state().add_exiting(handler)
    }

    fn RemoveExitingHandler(&self, registration: u64) -> bool {
        self.game_state().remove_exiting(registration)
    }

    fn AddDisposedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.game_state().add_disposed(handler)
    }

    fn RemoveDisposedHandler(&self, registration: u64) -> bool {
        self.game_state().remove_disposed(registration)
    }

    fn Run(&mut self) -> Result<()>
    where
        Self: Sized,
    {
        super::host::run_borrowed(self)
    }

    fn RunOneFrame(&mut self) -> Result<()>
    where
        Self: Sized,
    {
        super::host::run_one_frame_borrowed(self)
    }

    fn Tick(&self) -> Result<()> {
        self.game_state().tick()
    }

    fn SuppressDraw(&self) -> Result<()> {
        self.game_state().suppress_draw()
    }

    fn Exit(&self) -> Result<()> {
        self.game_state().exit()
    }

    fn ResetElapsedTime(&self) -> Result<()> {
        self.game_state().reset_elapsed_time()
    }

    fn Dispose(&mut self) {
        self.DisposeWithDisposing(true);
    }

    fn DisposeWithDisposing(&mut self, disposing: bool) {
        if disposing {
            let _ = self.game_state().emit_disposed();
        }
    }

    fn Finalize(&self) {}

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
        if self.game_state().emit_exiting() {
            Err(crate::error::CnaError::Callback(
                "Rust event-handler panic was contained before the native boundary".to_owned(),
            ))
        } else {
            Ok(())
        }
    }

    fn OnActivated(&self, sender: &dyn Any, args: EventArgs) {
        let _ = (sender, args);
        let _ = self.game_state().emit_activated();
    }

    fn OnDeactivated(&self, sender: &dyn Any, args: EventArgs) {
        let _ = (sender, args);
        let _ = self.game_state().emit_deactivated();
    }

    fn ShowMissingRequirementMessage(&self, exception: &dyn Error) -> bool {
        let _ = exception;
        false
    }
}
