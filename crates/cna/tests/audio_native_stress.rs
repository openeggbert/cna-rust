//! Crash-isolated Audio/XACT ownership, callback, and failure-path stress.

#![allow(
    clippy::float_cmp,
    clippy::needless_pass_by_value,
    clippy::too_many_lines
)]

use std::any::Any;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use cna::extensions::events::EventArgs;
use cna::Microsoft::Xna::Framework::Audio::{
    AudioChannels, AudioEmitter, AudioEngine, AudioListener, AudioStopOptions,
    DynamicSoundEffectInstance, Microphone, SoundBank, SoundEffect, WaveBank,
};
use cna::Microsoft::Xna::Framework::{
    FrameworkDispatcher, Game, GameContext, GameTime, TimeSpan, Vector3,
};
use cna::{run_for_frames, CnaError, GameState, GameStateAccess, Result};

const CHILD_CASE: &str = "CNA_RUST_AUDIO_STRESS_CHILD";

fn pcm16_wav(sample_rate: u32, channels: u16, samples: &[i16]) -> Vec<u8> {
    let data_len = u32::try_from(samples.len() * 2).expect("small WAV fixture");
    let block_align = channels * 2;
    let mut wav = Vec::with_capacity(44 + data_len as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * u32::from(block_align)).to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

#[derive(Default)]
struct SoundStressGame {
    state: Arc<GameState>,
    callback_handlers: Arc<AtomicUsize>,
    callback_sources: Arc<AtomicUsize>,
    dynamics: Vec<DynamicSoundEffectInstance>,
    initialized: bool,
}

impl GameStateAccess for SoundStressGame {
    fn game_state(&self) -> &Arc<GameState> { &self.state }
}

impl Game for SoundStressGame {
    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let pcm = vec![0_u8; 320];
        let listener = AudioListener::new();
        let emitter = AudioEmitter::new();

        for cycle in 0..25 {
            let mut effect = SoundEffect::new(game, &pcm, 8_000, AudioChannels::Mono)?;
            assert_eq!(effect.Duration()?, TimeSpan::FromMilliseconds(20.0));
            effect.SetName(&format!("sound-cycle-{cycle}"))?;
            assert_eq!(effect.Name()?, format!("sound-cycle-{cycle}"));

            let mut instance = effect.CreateInstance()?;
            assert_eq!(instance.Volume(), 1.0);
            assert_eq!(instance.Pitch(), 0.0);
            assert_eq!(instance.Pan(), 0.0);
            assert!(!instance.IsLooped());
            instance.SetVolume(0.5)?;
            instance.SetPitch(-0.25)?;
            instance.SetPan(0.25)?;
            instance.SetIsLooped(true)?;
            instance.Apply3D(&listener, &emitter)?;
            // XNA permits Pan before the packet's first Play and thereby clears the 3D mode.
            instance.SetPan(-0.25)?;
            instance.Apply3D(&listener, &emitter)?;
            // ABI 0.9.0 lifted the single-listener restriction: XNA copies the
            // whole array to XACT with no count restriction, and CNA now accepts
            // any positive count, letting the nearest listener decide the
            // applied attenuation, pan and Doppler.
            let mut far = AudioListener::new();
            far.SetPosition(Vector3::from_x_and_y_and_z(0.0, 0.0, 1_000.0));
            instance.Apply3DWithListenersAndEmitter(&[listener, listener], &emitter)?;
            instance.Apply3DWithListenersAndEmitter(&[far, listener, far], &emitter)?;
            // A count of zero stays refused rather than guessed at.
            assert!(matches!(
                instance.Apply3DWithListenersAndEmitter(&[], &emitter),
                Err(CnaError::InvalidInput(_))
            ));
            instance.Play()?;
            assert!(instance.SetIsLooped(false).is_err());
            assert!(instance.SetPan(0.0).is_err());
            instance.Pause()?;
            instance.Resume()?;
            instance.StopWithImmediate(cycle % 2 == 0)?;

            if cycle % 2 == 0 {
                instance.Dispose()?;
                instance.Dispose()?;
                effect.Dispose()?;
            } else {
                effect.Dispose()?;
                assert!(instance.IsDisposed());
                assert_eq!(instance.Volume(), 0.5);
                assert_eq!(instance.Pitch(), -0.25);
                assert_eq!(instance.Pan(), -0.25);
                assert!(instance.State().is_err());
            }
            effect.Dispose()?;
            assert_eq!(effect.Name()?, format!("sound-cycle-{cycle}"));
            assert_eq!(effect.Duration()?, TimeSpan::FromMilliseconds(20.0));
        }

        let wav = pcm16_wav(8_000, 1, &[0; 160]);
        let mut prefixed = b"ignored prefix".to_vec();
        let wav_position = prefixed.len() as u64;
        prefixed.extend_from_slice(&wav);
        let mut stream = Cursor::new(prefixed);
        stream.set_position(wav_position);
        let mut streamed = SoundEffect::FromStream(game, &mut stream)?;
        assert_eq!(stream.position() as usize, stream.get_ref().len());
        assert_eq!(streamed.Duration()?, TimeSpan::FromMilliseconds(20.0));
        streamed.Dispose()?;
        assert!(SoundEffect::FromStream(game, &mut Cursor::new(b"RIFFbad".to_vec())).is_err());

        let mut wrong_thread = SoundEffect::new(game, &pcm, 8_000, AudioChannels::Mono)?;
        wrong_thread = std::thread::spawn(move || {
            assert!(wrong_thread.Dispose().is_err());
            wrong_thread
        })
        .join()
        .expect("wrong-thread SoundEffect disposal remains contained");
        wrong_thread.Dispose()?;

        let mut instance_parent = SoundEffect::new(game, &pcm, 8_000, AudioChannels::Mono)?;
        let mut wrong_instance = instance_parent.CreateInstance()?;
        wrong_instance = std::thread::spawn(move || {
            assert!(wrong_instance.Dispose().is_err());
            wrong_instance
        })
        .join()
        .expect("wrong-thread SoundEffectInstance disposal remains contained");
        wrong_instance.Dispose()?;
        instance_parent.Dispose()?;

        let mut wrong_dynamic =
            DynamicSoundEffectInstance::new(game, 8_000, AudioChannels::Mono)?;
        wrong_dynamic = std::thread::spawn(move || {
            assert!(wrong_dynamic.Dispose(true).is_err());
            wrong_dynamic
        })
        .join()
        .expect("wrong-thread dynamic disposal remains contained");
        wrong_dynamic.Dispose(true)?;

        let reentered = Arc::new(AtomicBool::new(false));
        for _ in 0..25 {
            let dynamic = DynamicSoundEffectInstance::new(game, 8_000, AudioChannels::Mono)?;
            assert_eq!(dynamic.PendingBufferCount()?, 0);
            dynamic.SubmitBuffer(&pcm)?;
            assert!(dynamic.PendingBufferCount()? >= 1);

            let registration = Arc::new(AtomicU64::new(0));
            let registration_for_handler = Arc::clone(&registration);
            let source_count = Arc::clone(&self.callback_sources);
            let reentered_for_handler = Arc::clone(&reentered);
            let first = dynamic.AddBufferNeededHandler(Box::new(move |sender: &dyn Any, _: EventArgs| {
                let dynamic = sender
                    .downcast_ref::<DynamicSoundEffectInstance>()
                    .expect("dynamic callback sender");
                source_count.fetch_add(1, Ordering::SeqCst);
                let id = registration_for_handler.load(Ordering::Acquire);
                assert!(dynamic.RemoveBufferNeededHandler(id));
                if !reentered_for_handler.swap(true, Ordering::AcqRel) {
                    dynamic
                        .SubmitBuffer(&[0_u8; 320])
                        .expect("reentrant SubmitBuffer");
                }
            }));
            registration.store(first, Ordering::Release);

            let handler_count = Arc::clone(&self.callback_handlers);
            dynamic.AddBufferNeededHandler(Box::new(move |sender: &dyn Any, _: EventArgs| {
                assert!(sender.downcast_ref::<DynamicSoundEffectInstance>().is_some());
                handler_count.fetch_add(1, Ordering::SeqCst);
            }));
            dynamic.Play()?;
            self.dynamics.push(dynamic);
        }

        let all = Microphone::All(game)?;
        for _ in 0..20 {
            let repeated = Microphone::All(game)?;
            assert_eq!(repeated.len(), all.len());
            for (left, right) in all.iter().zip(&repeated) {
                assert!(Arc::ptr_eq(left, right));
            }
        }
        let default = Microphone::Default(game)?;
        if all.is_empty() {
            assert!(default.is_none());
        } else if let Some(default) = default {
            assert!(all.iter().any(|microphone| Arc::ptr_eq(microphone, &default)));
        }

        self.initialized = true;
        Ok(())
    }

    fn Update(&mut self, game: &mut GameContext<'_>, _: &GameTime) -> Result<()> {
        assert!(self.initialized);
        // This is the same framework dispatcher used by the Game host. It advances native
        // dynamic audio and drains queued Rust handlers only on the owner thread.
        FrameworkDispatcher::Update(game)
    }
}

#[derive(Default)]
struct CallbackPanicGame {
    state: Arc<GameState>,
    later_handler: Arc<AtomicUsize>,
    dynamic: Option<DynamicSoundEffectInstance>,
}

impl GameStateAccess for CallbackPanicGame {
    fn game_state(&self) -> &Arc<GameState> { &self.state }
}

impl Game for CallbackPanicGame {
    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let dynamic = DynamicSoundEffectInstance::new(game, 8_000, AudioChannels::Mono)?;
        dynamic.AddBufferNeededHandler(Box::new(|_: &dyn Any, _: EventArgs| {
            panic!("intentional BufferNeeded handler panic");
        }));
        let later = Arc::clone(&self.later_handler);
        dynamic.AddBufferNeededHandler(Box::new(move |_: &dyn Any, _: EventArgs| {
            later.fetch_add(1, Ordering::SeqCst);
        }));
        dynamic.SubmitBuffer(&[0_u8; 320])?;
        dynamic.Play()?;
        self.dynamic = Some(dynamic);
        Ok(())
    }

    fn Update(&mut self, game: &mut GameContext<'_>, _: &GameTime) -> Result<()> {
        FrameworkDispatcher::Update(game)
    }
}

struct GlobalAudioGame {
    state: Arc<GameState>,
    verify_persistence: bool,
}

impl GameStateAccess for GlobalAudioGame {
    fn game_state(&self) -> &Arc<GameState> { &self.state }
}

impl Game for GlobalAudioGame {
    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        if self.verify_persistence {
            // These XNA fields are process-static. The process-global Media callback
            // registration now keeps the exact ABI-0.7 library generation loaded across Game
            // recreation, so CNA's static values retain the XNA-observable state as well.
            assert_eq!(SoundEffect::MasterVolume(game)?, 0.25);
            assert_eq!(SoundEffect::DistanceScale(game)?, 2.0);
            assert_eq!(SoundEffect::DopplerScale(game)?, 3.0);
            assert_eq!(SoundEffect::SpeedOfSound(game)?, 300.0);
        } else {
            SoundEffect::SetMasterVolume(game, 0.25)?;
            SoundEffect::SetDistanceScale(game, 2.0)?;
            SoundEffect::SetDopplerScale(game, 3.0)?;
            SoundEffect::SetSpeedOfSound(game, 300.0)?;
        }
        Ok(())
    }
}

fn append_u16(data: &mut Vec<u8>, value: u16) { data.extend_from_slice(&value.to_le_bytes()); }
fn append_u32(data: &mut Vec<u8>, value: u32) { data.extend_from_slice(&value.to_le_bytes()); }

fn xgs_with_categories() -> Vec<u8> {
    const HEADER_SIZE: u32 = 65;
    const CATEGORY_SIZE: u32 = 10;
    let category_offset = HEADER_SIZE;
    let variable_offset = category_offset + 2 * CATEGORY_SIZE;
    let category_name_offset = variable_offset;
    let variable_name_offset = category_name_offset + 8 + 7;
    let mut data = b"XGSF".to_vec();
    append_u16(&mut data, 46);
    append_u16(&mut data, 0);
    append_u16(&mut data, 0);
    data.extend_from_slice(&[0; 8]);
    data.push(3);
    for value in [2_u16, 0, 0, 0, 0, 0, 0] { append_u16(&mut data, value); }
    for value in [
        category_offset,
        variable_offset,
        0,
        0,
        0,
        0,
        category_name_offset,
        variable_name_offset,
    ] {
        append_u32(&mut data, value);
    }
    for _ in 0..2 {
        data.push(0xff);
        append_u16(&mut data, 0);
        append_u16(&mut data, 0);
        data.push(0);
        append_u16(&mut data, 0xffff);
        data.push(0xff);
        data.push(0);
    }
    data.extend_from_slice(b"Default\0Combat\0");
    data
}

struct XactFixtures {
    root: PathBuf,
    xgs: PathBuf,
    bad_xwb: PathBuf,
    bad_xsb: PathBuf,
}

impl XactFixtures {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "cna-rust-audio-stress-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create XACT fixture directory");
        let xgs = root.join("categories.xgs");
        let bad_xwb = root.join("malformed.xwb");
        let bad_xsb = root.join("malformed.xsb");
        fs::write(&xgs, xgs_with_categories()).expect("write parser-valid XGS fixture");
        fs::write(&bad_xwb, [b"WBND".as_slice(), &[0; 76]].concat())
            .expect("write malformed XWB fixture");
        fs::write(&bad_xsb, [b"SDBK".as_slice(), &[0; 76]].concat())
            .expect("write malformed XSB fixture");
        Self { root, xgs, bad_xwb, bad_xsb }
    }
}

impl Drop for XactFixtures {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); }
}

struct XactStressGame {
    state: Arc<GameState>,
    xgs: PathBuf,
    bad_xwb: PathBuf,
    bad_xsb: PathBuf,
}

impl GameStateAccess for XactStressGame {
    fn game_state(&self) -> &Arc<GameState> { &self.state }
}

impl Game for XactStressGame {
    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        for cycle in 0..20 {
            let mut engine = AudioEngine::new(game, path(&self.xgs))?;
            assert!(!engine.IsDisposed());
            let first = engine.GetCategory("Default")?;
            let second = engine.GetCategory("Default")?;
            assert!(first == second);
            assert!(first.Equals(second.clone()));
            assert!(first.EqualsWithObj(&second)?);
            assert_eq!(first.Name(), "Default");
            assert_eq!(first.GetHashCode()?, second.GetHashCode()?);
            let mut category = first;
            category.SetVolume(0.5)?;
            category.Pause()?;
            category.Resume()?;
            category.Stop(AudioStopOptions::Immediate)?;
            let _ = engine.RendererDetails()?;
            engine.Update()?;

            // CNA logs the parse failures but still publishes handles for these malformed
            // banks. Exercise and dispose those real handles; docs classify the missing
            // constructor failure as an upstream semantic gap.
            let mut malformed_wave = WaveBank::new(&engine, path(&self.bad_xwb))?;
            malformed_wave.Dispose()?;
            let mut malformed_streaming = WaveBank::from_audio_engine_and_streaming_wave_bank_filename_and_offset_and_packetsize(
                &engine,
                path(&self.bad_xwb),
                -1,
                -1,
            )?;
            malformed_streaming.Dispose()?;
            let mut malformed_sound = SoundBank::new(&engine, path(&self.bad_xsb))?;
            assert!(malformed_sound.GetCue("missing").is_err());
            assert!(malformed_sound.PlayCue("missing").is_err());
            malformed_sound.Dispose()?;

            if cycle == 0 {
                engine = std::thread::spawn(move || {
                    assert!(engine.Dispose().is_err());
                    engine
                })
                .join()
                .expect("wrong-thread AudioEngine disposal remains contained");
            }
            engine.Dispose()?;
            engine.Dispose()?;
            assert!(engine.IsDisposed());
            assert!(category.Pause().is_err());
        }

        let mut ignored_arguments = AudioEngine::from_settings_file_and_look_ahead_time_and_renderer_id(
            game,
            path(&self.xgs),
            TimeSpan::from_ticks(-1),
            "renderer-not-representable-by-cna-0.7",
        )?;
        ignored_arguments.Dispose()?;
        Ok(())
    }
}

fn path(value: &Path) -> &str { value.to_str().expect("UTF-8 fixture path") }

#[test]
fn audio_native_stress_isolated() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    if let Ok(case) = std::env::var(CHILD_CASE) {
        run_child_case(&case);
        return;
    }
    for case in ["sound-dynamic-microphone", "callback-panic", "xact", "global-state"] {
        let status = Command::new(std::env::current_exe().expect("current test executable"))
            .args(["--exact", "audio_native_stress_isolated"])
            .env(CHILD_CASE, case)
            .env("SDL_AUDIODRIVER", "dummy")
            .status()
            .expect("start isolated Audio stress child");
        assert!(status.success(), "Audio stress child failed: {case}: {status}");
    }
}

fn run_child_case(case: &str) {
    match case {
        "sound-dynamic-microphone" => {
            let handlers = Arc::new(AtomicUsize::new(0));
            let sources = Arc::new(AtomicUsize::new(0));
            run_for_frames(
                SoundStressGame {
                    state: Arc::new(GameState::new()),
                    callback_handlers: Arc::clone(&handlers),
                    callback_sources: Arc::clone(&sources),
                    dynamics: Vec::new(),
                    initialized: false,
                },
                10,
            )
            .expect("SoundEffect/Dynamic/Microphone stress");
            assert!(sources.load(Ordering::SeqCst) >= 25);
            assert!(handlers.load(Ordering::SeqCst) >= 25);
            let delivered = handlers.load(Ordering::SeqCst) + sources.load(Ordering::SeqCst);
            assert!(delivered >= 50, "expected at least 50 callback deliveries, got {delivered}");
            let after_shutdown = delivered;
            run_for_frames(SoundStressGame::default(), 1)
                .expect("new Game after Audio callback shutdown");
            assert_eq!(
                handlers.load(Ordering::SeqCst) + sources.load(Ordering::SeqCst),
                after_shutdown
            );
        }
        "callback-panic" => {
            let later = Arc::new(AtomicUsize::new(0));
            let result = run_for_frames(
                CallbackPanicGame {
                    state: Arc::new(GameState::new()),
                    later_handler: Arc::clone(&later),
                    dynamic: None,
                },
                10,
            );
            assert!(matches!(result, Err(CnaError::Callback(_))));
            assert!(later.load(Ordering::SeqCst) >= 1);
            run_for_frames(SoundStressGame::default(), 1)
                .expect("new Game after contained Audio callback panic");
        }
        "xact" => {
            let fixtures = XactFixtures::new();
            run_for_frames(
                XactStressGame {
                    state: Arc::new(GameState::new()),
                    xgs: fixtures.xgs.clone(),
                    bad_xwb: fixtures.bad_xwb.clone(),
                    bad_xsb: fixtures.bad_xsb.clone(),
                },
                1,
            )
            .expect("XACT dependency/error/ownership stress");
        }
        "global-state" => {
            run_for_frames(
                GlobalAudioGame { state: Arc::new(GameState::new()), verify_persistence: false },
                1,
            )
            .expect("set process-global SoundEffect state");
            run_for_frames(
                GlobalAudioGame { state: Arc::new(GameState::new()), verify_persistence: true },
                1,
            )
            .expect("verify XNA SoundEffect static-state persistence after Game recreation");
        }
        _ => panic!("unknown Audio stress child case"),
    }
}
