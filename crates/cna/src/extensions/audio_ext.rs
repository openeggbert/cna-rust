//! `audio.h`'s process-level capability query.
//!
//! Everything else this header adds beyond XNA is a property of an object and
//! lives on that object. This one is a property of the process.

use crate::error::Result;
use crate::Microsoft::Xna::Framework::GameContext;

/// What the process can currently do with sound.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AudioCapabilities {
    /// Whether the native playback device can currently be opened.
    pub is_playback_available: bool,
}

/// Asks CNA what the process can currently do with sound.
///
/// The answer is about *now*, not about the build: a machine with no audio
/// hardware, a device another process holds exclusively, and a container with
/// no sound server all answer the same way, and all three can change while a
/// game runs. So this is a reading rather than a constant, and a caller that
/// wants to skip audio work should ask again rather than cache it.
pub fn capabilities(game: &GameContext<'_>) -> Result<AudioCapabilities> {
    Ok(AudioCapabilities {
        is_playback_available: game.native.audio_capabilities(game.handle)?,
    })
}
