//! `audio.h` and `xact.h` beyond XNA, and the two helpers where CNA and the
//! Rust projection disagree.
//!
//! The disagreement is the reason this file exists. `SoundEffect`'s two static
//! helpers are implemented in Rust, faithfully, and CNA has routes for the same
//! questions that answer differently -- `RUST-UPSTREAM-027`. The C routes are
//! therefore a deliberate non-binding, and these tests pin the Rust answers to
//! the values the decompiled XNA assemblies give, so a change to either side is
//! caught here rather than shipped.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cna::extensions::audio_ext::{
    capabilities, DynamicSoundEffectInstanceExt, MicrophoneExt, NativeDisposalState,
    SoundEffectExt,
};
use cna::Microsoft::Xna::Framework::Audio::{
    AudioChannels, DynamicSoundEffectInstance, Microphone, SoundEffect,
};
use cna::Microsoft::Xna::Framework::{Game, GameContext, TimeSpan};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

/// The values XNA's own `AudioFormat` produces, read off the decompiled
/// `DurationFromSize` and `SizeFromDuration`.
///
/// `TimeSpan.FromMilliseconds` rounds, so a 100-byte stereo buffer at 44.1 kHz
/// is one millisecond and not zero; and `SizeFromDuration` does the rate
/// division in binary32 and aligns the frame count, so one mono second at
/// 44.1 kHz is 88,198 bytes and not 88,200. Both are the exact cases CNA gets
/// wrong.
#[test]
fn the_rust_sample_helpers_answer_what_the_xna_reference_does() {
    // No native library needed: this is Rust arithmetic, and that is the point.
    assert_eq!(
        SoundEffect::GetSampleDuration(100, 44100, AudioChannels::Stereo),
        TimeSpan::FromMilliseconds(1.0),
        "XNA rounds to the nearest millisecond, so a 100-byte stereo buffer at \
         44.1 kHz lasts one -- CNA truncates and answers zero (RUST-UPSTREAM-027)"
    );
    assert_eq!(
        SoundEffect::GetSampleDuration(88198, 44100, AudioChannels::Mono),
        TimeSpan::FromMilliseconds(1000.0),
        "and a full second rounds up to a second rather than down to 999 ms"
    );

    assert_eq!(
        SoundEffect::GetSampleSizeInBytes(TimeSpan::FromSeconds(1.0), 44100, AudioChannels::Mono),
        88198,
        "XNA's binary32 rate division and frame alignment make one mono second \
         88,198 bytes, not the 88,200 a double-precision multiply gives"
    );
    assert_eq!(
        SoundEffect::GetSampleSizeInBytes(
            TimeSpan::FromMilliseconds(1.0),
            11025,
            AudioChannels::Stereo
        ),
        48,
        "and the `num + num % Channels` alignment makes one stereo millisecond \
         at 11.025 kHz 48 bytes, not 44"
    );

    // The round trip XNA's own helpers make: a size, to a duration, and back.
    for (rate, channels) in [
        (8000, AudioChannels::Mono),
        (44100, AudioChannels::Mono),
        (48000, AudioChannels::Stereo),
    ] {
        let duration = SoundEffect::GetSampleDuration(88200, rate, channels);
        let size = SoundEffect::GetSampleSizeInBytes(duration, rate, channels);
        println!("NOTE: {rate} Hz {channels:?}: 88200 bytes -> {duration:?} -> {size} bytes");
        assert!(size > 0, "a round trip through both helpers stays positive");
    }
}

#[derive(Default)]
struct AudioGame {
    state: Arc<GameState>,
    ran: Arc<AtomicBool>,
}

impl GameStateAccess for AudioGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for AudioGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        // --- what the process can do right now --------------------------------
        let audio = capabilities(game)?;
        println!("NOTE: playback available = {}", audio.is_playback_available);

        // --- a sound effect from a path ---------------------------------------
        // An empty path is documented to create a *silent* effect rather than
        // failing, which is worth pinning: a caller that treats `Ok` as "the
        // file was there" would be wrong.
        let silent = SoundEffect::FromAsset(game, "")?;
        println!("NOTE: empty path -> duration {:?}", silent.Duration()?);
        assert!(
            !silent.NativeIsDisposed()?,
            "a freshly created effect is not disposed"
        );
        assert!(!silent.IsDisposed(), "and Rust agrees");

        let instance = silent.CreateInstance()?;
        assert!(
            !instance.NativeIsDisposed()?,
            "nor is a fresh instance"
        );

        // --- the streaming paths ----------------------------------------------
        let dynamic = DynamicSoundEffectInstance::new(game, 44100, AudioChannels::Mono)?;
        assert_eq!(dynamic.PendingBufferCount()?, 0, "nothing queued yet");

        let samples = vec![0.0_f32; 4410];
        dynamic.SubmitFloatBuffer(&samples, 0, samples.len() as i32)?;
        let after_submit = dynamic.PendingBufferCount()?;
        println!("NOTE: after one float buffer, pending = {after_submit}");
        assert!(
            after_submit > 0,
            "a submitted buffer is queued -- which is what says the float path \
             reached the mixer rather than being accepted and dropped"
        );

        // A range outside the buffer is refused before it reaches C, where it
        // would be a read past the slice.
        assert!(
            dynamic.SubmitFloatBuffer(&samples, 0, samples.len() as i32 + 1).is_err(),
            "a count past the end of the buffer is refused"
        );
        assert!(
            dynamic.SubmitFloatBuffer(&samples, -1, 1).is_err(),
            "and so is a negative offset"
        );

        dynamic.ClearBuffers()?;
        assert_eq!(
            dynamic.PendingBufferCount()?,
            0,
            "clearing drops every queued buffer"
        );

        dynamic.QueueInitialBuffers()?;
        dynamic.Update()?;
        println!("NOTE: after queue+update, pending = {}", dynamic.PendingBufferCount()?);

        // --- servicing every microphone at once -------------------------------
        Microphone::CheckAllBuffers(game)?;

        self.ran.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn the_audio_extensions_answer_against_the_live_library() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let ran = Arc::new(AtomicBool::new(false));
    let game = AudioGame {
        state: Arc::new(GameState::default()),
        ran: Arc::clone(&ran),
    };
    run_for_frames(game, 1).expect("one frame with the audio extensions");
    assert!(ran.load(Ordering::SeqCst), "LoadContent ran");
}
