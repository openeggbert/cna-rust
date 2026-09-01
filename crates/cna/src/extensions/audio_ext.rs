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

/// Whether CNA considers an audio object disposed.
///
/// A different question from the strict projection's `IsDisposed`, which
/// answers from the Rust side's own record of whether it released the handle.
/// The two disagree in exactly the case that matters: an `AudioEngine` that
/// disposes takes its banks and cues down with it, so the Rust values still
/// exist and CNA's objects do not. This is what sees that.
///
/// Distinct again from [`NativeDisposalNotice`], which says whether CNA
/// *raised* its disposal notification rather than what state the object is in
/// now.
///
/// A CNA extension: import it to call this.
///
/// ```rust,ignore
/// use cna::extensions::audio_ext::NativeDisposalState;
/// assert!(bank.NativeIsDisposed()?);
/// ```
#[allow(non_snake_case)]
pub trait NativeDisposalState {
    /// Whether CNA considers this object disposed.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, including the refusal for a handle
    /// this side has already released.
    fn NativeIsDisposed(&self) -> Result<bool>;
}

/// The `audio.h` streaming routes with no XNA counterpart.
///
/// XNA's `DynamicSoundEffectInstance` takes 16-bit PCM bytes and nothing else.
/// These are the paths CNA adds around that: float samples rather than packed
/// bytes, the initial queue that primes playback before the first
/// `BufferNeeded`, an explicit clear, and the pump that retires finished
/// buffers.
///
/// `Update` is the one worth reading twice. XNA's instance raises
/// `BufferNeeded` from the audio engine's own servicing; a caller that drives
/// playback itself -- a test, or a game with its own mixer cadence -- has no
/// way to make that happen. `Update` is that way.
///
/// A CNA extension: import it to call these.
///
/// ```rust,ignore
/// use cna::extensions::audio_ext::DynamicSoundEffectInstanceExt;
/// instance.SubmitFloatBuffer(&samples, 0, samples.len() as i32)?;
/// ```
#[allow(non_snake_case)]
pub trait DynamicSoundEffectInstanceExt {
    /// Submits a range of 32-bit float samples, which CNA copies during the
    /// call.
    ///
    /// The whole slice is passed alongside the range, so the offset and count
    /// are checked against it here: an out-of-range pair would otherwise be a
    /// read past the buffer.
    fn SubmitFloatBuffer(&self, buffer: &[f32], offset: i32, count: i32) -> Result<()>;

    /// Queues the buffers playback starts with.
    fn QueueInitialBuffers(&self) -> Result<()>;

    /// Drops every queued buffer without playing it.
    fn ClearBuffers(&self) -> Result<()>;

    /// Retires finished buffers and raises `BufferNeeded` for what that frees.
    ///
    /// Servicing the instance by hand, for a caller driving playback on its own
    /// cadence rather than the audio engine's.
    fn Update(&self) -> Result<()>;
}

/// The `audio.h` route that services every microphone at once.
///
/// Process-wide rather than per-device, which is why it is an associated
/// function: XNA's `Microphone` raises `BufferReady` from the platform's own
/// servicing, and a caller driving capture on its own cadence -- a test, or a
/// game polling rather than waiting on events -- has no other way to make that
/// happen.
///
/// A CNA extension: import it, and `Microphone::CheckAllBuffers` resolves
/// through the trait.
///
/// ```rust,ignore
/// use cna::extensions::audio_ext::MicrophoneExt;
/// Microphone::CheckAllBuffers(game)?;
/// ```
#[allow(non_snake_case)]
pub trait MicrophoneExt {
    /// Checks every microphone's capture buffer and raises what is due.
    ///
    /// Process-wide rather than per-device, which is why it is an associated
    /// function: XNA's `Microphone` raises `BufferReady` from the platform's
    /// own servicing, and a caller driving capture on its own cadence -- a
    /// test, or a game polling rather than waiting on events -- has no other
    /// way to make that happen.
    fn CheckAllBuffers(game: &GameContext<'_>) -> Result<()>;
}

/// The `xact.h` renderer identity routes with no XNA counterpart.
///
/// XNA's `AudioEngine` reports a renderer's friendly name and identifier
/// separately, which the strict projection carries as `RendererDetails`. These
/// are CNA's single-string spelling of the same renderer, and the hash and
/// equality computed over it.
///
/// A CNA extension: import it to call these.
///
/// ```rust,ignore
/// use cna::extensions::audio_ext::AudioEngineExt;
/// let text = engine.RendererText(0)?;
/// ```
#[allow(non_snake_case)]
pub trait AudioEngineExt {
    /// The renderer descriptor at an index, as CNA formats it.
    ///
    /// `AudioEngine::RendererDetails` already reports the friendly
    /// name and identifier separately; this is CNA's single-string spelling of
    /// the same renderer, and it is what the hash and the equality below are
    /// computed over.
    fn RendererText(&self, index: u64) -> Result<String>;

    /// CNA's hash for the renderer at an index.
    fn RendererHashCode(&self, index: u64) -> Result<i32>;

    /// Whether CNA considers two renderers the same.
    ///
    /// Asked of CNA rather than answered by comparing the descriptors here:
    /// XNA compares renderer descriptors by value, and which fields take part
    /// is this ABI's decision, not the binding's.
    fn RenderersEqual(&self, left: u64, right: u64) -> Result<bool>;
}

/// The `audio.h` producer that loads a `SoundEffect` from content by name.
///
/// XNA has no counterpart: a game gets a sound effect from the content manager
/// or from `SoundEffect.FromStream`, and the strict projection carries both.
/// This asks CNA's own asset layer for one instead, which is what a caller with
/// an asset name and no `ContentManager` has.
///
/// A CNA extension: import it, and `SoundEffect::FromAsset` resolves through
/// the trait.
///
/// ```rust,ignore
/// use cna::extensions::audio_ext::SoundEffectExt;
/// let effect = SoundEffect::FromAsset(game, "explosion")?;
/// ```
#[allow(non_snake_case)]
pub trait SoundEffectExt: Sized {
    /// Loads a sound effect from CNA's asset layer by name.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, including the refusal for an asset
    /// the layer cannot find.
    fn FromAsset(game: &GameContext<'_>, assetName: &str) -> Result<Self>;
}
