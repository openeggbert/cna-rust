//! `RUST-BEHAVIOR-008`: the visualization spectrum, on a real host audio backend.
//!
//! Every earlier audio run on this host used SDL's `dummy` driver, because the sandbox's
//! PulseAudio route could not wake its mainloop, and the 256 frequency bins CNA publishes
//! were recorded as `BACKEND_BLOCKED` -- measured for shape, never for content. The route
//! is now open: SDL selects `pulseaudio`, enumerates this host's devices and drains a
//! queued buffer, so the spectrum can be measured against a real device.
//!
//! Measuring it also corrected the reason the old runs saw zeros. The ownership stress
//! plays a 160-sample fixture of **pure silence**; a correct spectrum of silence is zero.
//! CNA's `OnPostMix` tap and its 512-point FFT were never the blocked part, and the
//! `dummy` driver reaches them too, because SDL's dummy device still consumes its buffer
//! on a callback thread at the real-time rate. What `dummy` genuinely cannot do is put
//! the samples on a device, which is why the real driver is the one that qualifies
//! playback -- and why this runs both and requires them to agree.
//!
//! Three properties make the answer falsifiable rather than merely non-zero:
//!
//! * a silent authored fixture must still read as an all-zero spectrum;
//! * a tone must peak in the bin its frequency selects -- CNA's mixer always requests a
//!   fixed 44,100 Hz and the FFT is 512 wide, so bin = round(hz * 512 / 44100);
//! * the captured sample peak must be the authored amplitude scaled by MediaPlayer's
//!   volume, since the tap sits after the mix.
//!
//! A spectrum reading noise, a stale ring, or an uninitialised buffer fails all three.

#![allow(non_snake_case, clippy::float_cmp)]

use std::f32::consts::TAU;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use cna::Microsoft::Xna::Framework::Media::{MediaPlayer, Song, VisualizationData};
use cna::Microsoft::Xna::Framework::{Game, GameContext, GameTime};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

const CHILD: &str = "CNA_RUST_AUDIO_BACKEND_CHILD";

/// CNA's mixer always requests this rate (`AudioMixer.hpp`, AUD-04-005), whatever the
/// device offers, so the bin a tone selects does not depend on the host.
const MIXER_RATE: f32 = 44_100.0;
/// `VisualizationFFT::InputSize`; its 256 bins are what `VisualizationData` publishes.
const FFT_WINDOW: f32 = 512.0;

/// Amplitude of the authored tone, and the volume it is played at. Their product is what
/// the post-mix tap must report, and 0.15 keeps a sine tone unobtrusive on a real device.
const TONE_AMPLITUDE: f32 = 0.8;
const PLAYBACK_VOLUME: f32 = 0.15;

fn expected_bin(hz: f32) -> usize {
    (hz * FFT_WINDOW / MIXER_RATE).round() as usize
}

/// Project-authored mono PCM16 WAV. No third-party asset is involved: every sample is
/// computed here, so the fixture is deterministic and redistributable. `hz` of zero
/// authors silence.
fn tone_wav(hz: f32, seconds: f32, rate: u32, amplitude: f32) -> Vec<u8> {
    let frames = (seconds * rate as f32) as usize;
    let mut samples = Vec::with_capacity(frames);
    for index in 0..frames {
        let value = if hz == 0.0 {
            0.0
        } else {
            (TAU * hz * (index as f32) / (rate as f32)).sin() * amplitude
        };
        samples.push((value * f32::from(i16::MAX)) as i16);
    }
    let data_len = (samples.len() * 2) as u32;
    let mut value = Vec::new();
    value.extend_from_slice(b"RIFF");
    value.extend_from_slice(&(36 + data_len).to_le_bytes());
    value.extend_from_slice(b"WAVEfmt ");
    value.extend_from_slice(&16_u32.to_le_bytes());
    value.extend_from_slice(&1_u16.to_le_bytes()); // PCM
    value.extend_from_slice(&1_u16.to_le_bytes()); // mono
    value.extend_from_slice(&rate.to_le_bytes());
    value.extend_from_slice(&(rate * 2).to_le_bytes()); // byte rate
    value.extend_from_slice(&2_u16.to_le_bytes()); // block align
    value.extend_from_slice(&16_u16.to_le_bytes()); // bits per sample
    value.extend_from_slice(b"data");
    value.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        value.extend_from_slice(&sample.to_le_bytes());
    }
    value
}

fn path(value: &Path) -> &str {
    value.to_str().expect("UTF-8 fixture path")
}

/// What one authored signal's playback produced.
#[derive(Clone, Copy, Debug, Default)]
struct Spectrum {
    /// Frames whose sample window held any non-zero value.
    non_silent_frames: usize,
    /// Largest absolute sample the capture ring reported.
    peak_sample: f32,
    /// Largest magnitude any bin reported, and where.
    peak_magnitude: f32,
    peak_bin: usize,
    /// Bins above a tenth of the peak: how concentrated the answer is.
    loud_bins: usize,
}

struct ToneGame {
    state: Arc<GameState>,
    song_path: PathBuf,
    frames: Arc<AtomicUsize>,
    result: Arc<Mutex<Spectrum>>,
}

impl GameStateAccess for ToneGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for ToneGame {
    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        MediaPlayer::SetVolume(game, PLAYBACK_VOLUME)?;
        MediaPlayer::SetIsVisualizationEnabled(game, true)?;
        let song = Song::FromUri(game, "authored-tone", path(&self.song_path))?;
        // Play resets CNA's capture ring, so each measurement starts from no data.
        MediaPlayer::Play(game, &song)?;
        Ok(())
    }

    fn Update(&mut self, game: &mut GameContext<'_>, _: &GameTime) -> Result<()> {
        self.frames.fetch_add(1, Ordering::SeqCst);
        let mut data = VisualizationData::new();
        MediaPlayer::GetVisualizationData(game, &mut data)?;

        let samples = data.Samples();
        let frequencies = data.Frequencies();
        let mut measured = self.result.lock().unwrap();

        let sample_peak = samples.iter().fold(0.0_f32, |peak, s| peak.max(s.abs()));
        if sample_peak > 0.0 {
            measured.non_silent_frames += 1;
        }
        measured.peak_sample = measured.peak_sample.max(sample_peak);

        let mut best = 0.0_f32;
        let mut best_bin = 0_usize;
        for (index, magnitude) in frequencies.iter().enumerate() {
            if *magnitude > best {
                best = *magnitude;
                best_bin = index;
            }
        }
        // Keep the loudest frame's answer, so a frame captured before the ring filled
        // cannot decide where the peak is.
        if best > measured.peak_magnitude {
            measured.peak_magnitude = best;
            measured.peak_bin = best_bin;
            measured.loud_bins = frequencies
                .iter()
                .filter(|magnitude| **magnitude > best * 0.1)
                .count();
        }
        Ok(())
    }
}

fn measure(root: &Path, name: &str, hz: f32, frames: u64) -> Spectrum {
    let song = root.join(format!("{name}.wav"));
    fs::write(&song, tone_wav(hz, 3.0, 44_100, TONE_AMPLITUDE)).expect("write authored fixture");
    let result = Arc::new(Mutex::new(Spectrum::default()));
    let counted = Arc::new(AtomicUsize::new(0));
    run_for_frames(
        ToneGame {
            state: Arc::new(GameState::new()),
            song_path: song,
            frames: Arc::clone(&counted),
            result: Arc::clone(&result),
        },
        frames,
    )
    .expect("authored fixture playback");
    let measured = *result.lock().unwrap();
    println!(
        "{name}: hz={hz} frames={} non_silent={} peak_sample={:.6} peak_bin={} peak_mag={:.6} loud_bins={}",
        counted.load(Ordering::SeqCst),
        measured.non_silent_frames,
        measured.peak_sample,
        measured.peak_bin,
        measured.peak_magnitude,
        measured.loud_bins,
    );
    measured
}

#[test]
fn audio_backend_real_visualization_spectrum() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    if std::env::var_os(CHILD).is_none() {
        // Both drivers, in one run, required to agree. `dummy` is the control that keeps
        // this test meaningful on a host with no audio device at all; `pulseaudio` is the
        // one that puts the samples on real hardware.
        for driver in ["dummy", "pulseaudio"] {
            let data = std::env::temp_dir()
                .join(format!("cna-rust-audio-data-{}-{driver}", std::process::id()));
            let status = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "audio_backend_real_visualization_spectrum",
                    "--nocapture",
                ])
                .env(CHILD, driver)
                .env("XDG_DATA_HOME", data)
                .env("SDL_AUDIODRIVER", driver)
                .status()
                .expect("start audio backend child");
            assert!(
                status.success(),
                "audio backend child ({driver}) failed: {status}"
            );
        }
        return;
    }

    let driver = std::env::var(CHILD).expect("driver stage");
    let root = std::env::temp_dir().join(format!("cna-rust-audio-{}", std::process::id()));
    fs::create_dir_all(&root).expect("create audio fixture directory");
    println!("--- SDL_AUDIODRIVER={driver} ---");

    // The control, and the explanation of every earlier all-zero reading: the fixture the
    // ownership stress plays is silent, and a correct spectrum of silence is zero.
    let silent = measure(&root, "authored-silence", 0.0, 60);
    assert_eq!(
        silent.peak_sample, 0.0,
        "a silent authored fixture produced samples"
    );
    assert_eq!(
        silent.peak_magnitude, 0.0,
        "a silent authored fixture produced a spectrum"
    );
    assert_eq!(
        silent.non_silent_frames, 0,
        "a silent authored fixture reported non-silent frames"
    );

    let low = measure(&root, "tone-low", 1_000.0, 90);
    let high = measure(&root, "tone-high", 4_000.0, 90);

    for (name, tone, hz) in [("low", low, 1_000.0_f32), ("high", high, 4_000.0_f32)] {
        assert!(
            tone.non_silent_frames > 0,
            "{driver}/{name}: captured no samples"
        );
        // The tap sits after the mix, so what it reports is the authored amplitude
        // scaled by MediaPlayer's volume. Anything else is not this signal.
        let expected_peak = TONE_AMPLITUDE * PLAYBACK_VOLUME;
        assert!(
            (tone.peak_sample - expected_peak).abs() < expected_peak * 0.05,
            "{driver}/{name}: sample peak {} is not the authored {expected_peak}",
            tone.peak_sample,
        );
        // The bin the tone's own frequency selects.
        assert_eq!(
            tone.peak_bin,
            expected_bin(hz),
            "{driver}/{name}: {hz} Hz peaked in bin {} rather than {}",
            tone.peak_bin,
            expected_bin(hz),
        );
        // A tone is one frequency: its energy must be concentrated, not spread.
        assert!(
            tone.loud_bins <= 8,
            "{driver}/{name}: {} bins within a tenth of the peak is not a tone",
            tone.loud_bins,
        );
    }

    // Rate-independent restatement of the same fact, so the pinned bins above cannot pass
    // by coincidence: four times the frequency, four times the bin.
    let ratio = high.peak_bin as f32 / low.peak_bin as f32;
    assert!(
        (3.5..=4.5).contains(&ratio),
        "{driver}: peak bin ratio {ratio} does not follow the 4x tone ratio ({} -> {})",
        low.peak_bin,
        high.peak_bin,
    );

    let _ = fs::remove_dir_all(&root);
}
