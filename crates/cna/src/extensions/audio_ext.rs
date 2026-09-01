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

/// Whether CNA raised its own disposal notification for an XACT object.
///
/// XNA's `Disposing` is what a game programs against, and
/// `AddDisposingHandler` is where a game reads it. This says whether the
/// runtime *underneath* agreed that the object was disposed -- a different
/// question, with a different answer whenever the two get out of step.
///
/// They were out of step. Until `RUST-EXT-017` the Rust event fired only on an
/// explicit `Dispose`; an `AudioEngine` tearing down its own banks and cues
/// released their handles and raised nothing, which is exactly the path XNA
/// *does* raise on. CNA's notification fired the whole time. Binding it alone
/// would have made the disagreement visible without fixing it, so both landed
/// together: every path that raises the XNA event now raises this one, and
/// this is how a caller checks.
///
/// Answers `false` before disposal, and `false` afterwards on an artifact that
/// refused the subscription -- which is a fact about the artifact, not a
/// disposal that did not happen.
pub trait NativeDisposalNotice {
    /// Whether CNA raised its disposal notification for this object.
    fn cna_raised_disposing(&self) -> bool;
}

impl NativeDisposalNotice for crate::audio::AudioEngine {
    fn cna_raised_disposing(&self) -> bool {
        crate::audio::engine_cna_raised_disposing(self)
    }
}

impl NativeDisposalNotice for crate::audio::WaveBank {
    fn cna_raised_disposing(&self) -> bool {
        crate::audio::wave_bank_cna_raised_disposing(self)
    }
}

impl NativeDisposalNotice for crate::audio::SoundBank {
    fn cna_raised_disposing(&self) -> bool {
        crate::audio::sound_bank_cna_raised_disposing(self)
    }
}

impl NativeDisposalNotice for crate::audio::Cue {
    fn cna_raised_disposing(&self) -> bool {
        crate::audio::cue_cna_raised_disposing(self)
    }
}
