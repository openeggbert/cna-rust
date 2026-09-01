#![allow(non_snake_case, non_upper_case_globals, clippy::missing_errors_doc)]

mod dynamic;
mod microphone;
mod runtime;
mod sound;
mod xact;

pub(crate) use runtime::AudioRuntime;
pub use dynamic::DynamicSoundEffectInstance;
pub use microphone::Microphone;
pub use sound::{SoundEffect, SoundEffectInstance, SoundEffectInstanceBase};
pub use xact::{AudioCategory, AudioEngine, Cue, SoundBank, WaveBank};
pub(crate) use xact::{
    cue_cna_raised_disposing, engine_cna_raised_disposing, sound_bank_cna_raised_disposing,
    wave_bank_cna_raised_disposing,
};

use core::any::Any;
use core::fmt;
use std::error::Error;

use crate::value::Vector3;
use crate::Microsoft::Xna::Framework::TimeSpan;

fn validate_pcm_format(sample_rate: i32, channels: AudioChannels) {
    assert!((8_000..=48_000).contains(&sample_rate), "sampleRate must be between 8000 and 48000");
    let _ = channels;
}

pub(crate) fn sample_duration(size_in_bytes: i32, sample_rate: i32, channels: AudioChannels) -> TimeSpan {
    assert!(size_in_bytes >= 0, "sizeInBytes must not be negative");
    validate_pcm_format(sample_rate, channels);
    if size_in_bytes == 0 { return TimeSpan::Zero; }
    let samples = size_in_bytes / (channels as i32 * 2);
    let milliseconds = (samples as f32 * 1000.0_f32 / sample_rate as f32) + 0.5_f32;
    TimeSpan::from_ticks((milliseconds as i64) * TimeSpan::TicksPerMillisecond)
}

pub(crate) fn sample_size(duration: TimeSpan, sample_rate: i32, channels: AudioChannels) -> i32 {
    let milliseconds = duration.TotalMilliseconds();
    assert!(
        (0.0..=f64::from(i32::MAX)).contains(&milliseconds),
        "duration is outside the supported range"
    );
    validate_pcm_format(sample_rate, channels);
    if duration == TimeSpan::Zero { return 0; }
    // XNA performs only the rate division in binary32, promotes that result to
    // double, then multiplies by TimeSpan.TotalMilliseconds. Keeping the mixed
    // precision is observable at 44.1 kHz (one mono second is 88,198 bytes).
    let sample_frames = milliseconds * f64::from(sample_rate as f32 / 1000.0_f32);
    assert!(sample_frames <= f64::from(i32::MAX), "duration is outside the supported range");
    let sample_frames = sample_frames as i32;
    let aligned_frames = sample_frames
        .checked_add(sample_frames % channels as i32)
        .expect("duration is outside the supported range");
    aligned_frames
        .checked_mul(channels as i32 * 2)
        .expect("duration is outside the supported range")
}

/// Number of PCM channels accepted by XNA audio constructors.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AudioChannels {
    Mono = 1,
    Stereo = 2,
}

/// Authored versus immediate XACT stop behavior.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AudioStopOptions {
    AsAuthored = 0,
    Immediate = 1,
}

/// Playback state of a sound instance or cue.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SoundState {
    Playing = 0,
    Paused = 1,
    Stopped = 2,
}

/// Capture state of a microphone.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MicrophoneState {
    Started = 0,
    Stopped = 1,
}

/// Position and orientation of the listener used for 3D audio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioListener {
    position: Vector3,
    velocity: Vector3,
    forward: Vector3,
    up: Vector3,
}

impl AudioListener {
    pub fn new() -> Self {
        Self {
            position: Vector3::Zero,
            velocity: Vector3::Zero,
            forward: Vector3::Forward,
            up: Vector3::Up,
        }
    }

    pub fn Position(&self) -> Vector3 { self.position }
    pub fn SetPosition(&mut self, value: Vector3) { self.position = value; }
    pub fn Velocity(&self) -> Vector3 { self.velocity }
    pub fn SetVelocity(&mut self, value: Vector3) { self.velocity = value; }
    pub fn Forward(&self) -> Vector3 { self.forward }
    pub fn SetForward(&mut self, value: Vector3) { self.forward = value; }
    pub fn Up(&self) -> Vector3 { self.up }
    pub fn SetUp(&mut self, value: Vector3) { self.up = value; }
}

impl Default for AudioListener {
    fn default() -> Self { Self::new() }
}

/// Position, orientation, velocity, and Doppler scale of a 3D emitter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioEmitter {
    position: Vector3,
    velocity: Vector3,
    forward: Vector3,
    up: Vector3,
    doppler_scale: f32,
}

impl AudioEmitter {
    pub fn new() -> Self {
        Self {
            position: Vector3::Zero,
            velocity: Vector3::Zero,
            forward: Vector3::Forward,
            up: Vector3::Up,
            doppler_scale: 1.0,
        }
    }

    pub fn Position(&self) -> Vector3 { self.position }
    pub fn SetPosition(&mut self, value: Vector3) { self.position = value; }
    pub fn Velocity(&self) -> Vector3 { self.velocity }
    pub fn SetVelocity(&mut self, value: Vector3) { self.velocity = value; }
    pub fn Forward(&self) -> Vector3 { self.forward }
    pub fn SetForward(&mut self, value: Vector3) { self.forward = value; }
    pub fn Up(&self) -> Vector3 { self.up }
    pub fn SetUp(&mut self, value: Vector3) { self.up = value; }
    pub fn DopplerScale(&self) -> f32 { self.doppler_scale }
    pub fn SetDopplerScale(&mut self, value: f32) {
        assert!(!(value < 0.0), "DopplerScale must not be negative");
        self.doppler_scale = value;
    }
}

impl Default for AudioEmitter {
    fn default() -> Self { Self::new() }
}

/// Immutable identity and display information for an audio renderer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RendererDetail {
    friendly_name: String,
    renderer_id: String,
}

impl RendererDetail {
    pub(crate) fn from_parts(friendly_name: String, renderer_id: String) -> Self {
        Self { friendly_name, renderer_id }
    }

    pub fn FriendlyName(&self) -> String { self.friendly_name.clone() }
    pub fn RendererId(&self) -> String { self.renderer_id.clone() }
    pub fn GetHashCode(&self) -> i32 {
        dotnet_string_hash(&self.friendly_name) ^ dotnet_string_hash(&self.renderer_id)
    }
    pub fn ToString(&self) -> String {
        "Microsoft.Xna.Framework.Audio.RendererDetail".to_owned()
    }
    pub fn Equals(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>().is_some_and(|value| {
            self.friendly_name == value.friendly_name && self.renderer_id == value.renderer_id
        })
    }
}

fn dotnet_string_hash(value: &str) -> i32 {
    let mut first = 5_381_i32;
    let mut second = first;
    let utf16 = value.encode_utf16().collect::<Vec<_>>();
    for pair in utf16.chunks(2) {
        first = first
            .wrapping_shl(5)
            .wrapping_add(first)
            ^ i32::from(pair[0]);
        if let Some(value) = pair.get(1) {
            second = second
                .wrapping_shl(5)
                .wrapping_add(second)
                ^ i32::from(*value);
        }
    }
    first.wrapping_add(second.wrapping_mul(1_566_083_941))
}

macro_rules! audio_exception {
    ($name:ident, $default:literal) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name {
            message: String,
            inner_message: Option<String>,
        }

        impl $name {
            pub fn new() -> Self {
                Self { message: $default.to_owned(), inner_message: None }
            }
            pub fn from_message(message: &str) -> Self {
                Self { message: message.to_owned(), inner_message: None }
            }
            pub fn from_message_and_inner(message: &str, inner: &dyn Error) -> Self {
                Self { message: message.to_owned(), inner_message: Some(inner.to_string()) }
            }
        }

        impl Default for $name {
            fn default() -> Self { Self::new() }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.message)?;
                if let Some(inner) = &self.inner_message { write!(formatter, ": {inner}")?; }
                Ok(())
            }
        }

        impl Error for $name {}
    };
}

audio_exception!(InstancePlayLimitException, "The sound effect instance limit was reached.");
audio_exception!(NoAudioHardwareException, "No audio hardware is available.");
audio_exception!(NoMicrophoneConnectedException, "No microphone is connected.");

#[cfg(test)]
mod tests {
    use super::RendererDetail;

    #[test]
    fn renderer_detail_uses_xna_value_identity_and_hash() {
        let detail = RendererDetail::from_parts("SDL3 Mixer".to_owned(), "sdl3_mixer".to_owned());
        let equal = RendererDetail::from_parts("SDL3 Mixer".to_owned(), "sdl3_mixer".to_owned());
        let different_name =
            RendererDetail::from_parts("Other Mixer".to_owned(), "sdl3_mixer".to_owned());
        assert!(detail.Equals(&equal));
        assert!(!detail.Equals(&different_name));
        assert_eq!(detail.GetHashCode(), 1_962_617_453);
        assert_eq!(detail.ToString(), "Microsoft.Xna.Framework.Audio.RendererDetail");
    }
}
