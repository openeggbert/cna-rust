#![allow(non_snake_case, clippy::missing_errors_doc)]

use std::fs::File;
use std::io::Read;

use crate::error::{CnaError, Result};
use super::GameContext;

/// Pumps framework services that have a managed Rust dispatcher.
pub struct FrameworkDispatcher;

impl FrameworkDispatcher {
    pub fn Update(game: &GameContext<'_>) -> Result<()> {
        let (native, handle) = game.native_game();
        native.framework_dispatcher_update(handle)?;
        game.audio_runtime().dispatch_pending()?;
        crate::media::MediaPlayer::update(game)?;
        game.media_runtime().dispatch_pending()?;
        Ok(())
    }
}

/// Opens files relative to the process title-container directory.
pub struct TitleContainer;

impl TitleContainer {
    pub fn OpenStream(name: &str) -> Result<Box<dyn Read>> {
        File::open(name)
            .map(|file| Box::new(file) as Box<dyn Read>)
            .map_err(|error| {
                CnaError::Io(format!("failed to open title-container stream: {error}"))
            })
    }
}
