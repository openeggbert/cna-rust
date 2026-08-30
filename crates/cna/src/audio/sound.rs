use core::mem::size_of;
use std::io::Read;
use std::sync::{Arc, Mutex, Weak};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::game::{GameContext, TimeSpan};
use crate::native::Native;

use super::runtime::{AudioResourceCleanup, AudioRuntime};
use super::{sample_duration, sample_size, AudioChannels, AudioEmitter, AudioListener, SoundState};

pub(super) fn native_channels(channels: AudioChannels) -> sys::CNA_AudioChannels {
    channels as i32 as u32
}

fn validate_format(sample_rate: i32, channels: AudioChannels) -> Result<usize> {
    if !(8_000..=48_000).contains(&sample_rate) {
        return Err(CnaError::InvalidInput("sampleRate must be between 8000 and 48000"));
    }
    Ok(channels as usize * 2)
}

pub(super) fn validate_range(value: f32, minimum: f32, maximum: f32, name: &'static str) -> Result<()> {
    if value.is_nan() || value < minimum || value > maximum {
        Err(CnaError::InvalidInput(name))
    } else {
        Ok(())
    }
}

fn native_vector3(value: crate::value::Vector3) -> sys::CNA_Vector3 {
    sys::CNA_Vector3 { x: value.X, y: value.Y, z: value.Z }
}

pub(super) fn native_listener(value: &AudioListener) -> sys::CNA_AudioListener {
    sys::CNA_AudioListener {
        struct_size: size_of::<sys::CNA_AudioListener>() as u32,
        struct_version: 1,
        forward: native_vector3(value.Forward()),
        position: native_vector3(value.Position()),
        up: native_vector3(value.Up()),
        velocity: native_vector3(value.Velocity()),
    }
}

pub(super) fn native_emitter(value: &AudioEmitter) -> sys::CNA_AudioEmitter {
    sys::CNA_AudioEmitter {
        struct_size: size_of::<sys::CNA_AudioEmitter>() as u32,
        struct_version: 1,
        doppler_scale: value.DopplerScale(),
        forward: native_vector3(value.Forward()),
        position: native_vector3(value.Position()),
        up: native_vector3(value.Up()),
        velocity: native_vector3(value.Velocity()),
    }
}

pub(super) struct SoundEffectState {
    runtime: Arc<AudioRuntime>,
    native: Arc<Native>,
    generation: u64,
    handle: Mutex<sys::CNA_Handle>,
    duration: TimeSpan,
    name: Mutex<String>,
    children: Mutex<Vec<Weak<InstanceState>>>,
}

impl SoundEffectState {
    fn require_handle(&self) -> Result<sys::CNA_Handle> {
        self.runtime.ensure_generation(self.generation)?;
        let handle = *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        (handle != sys::CNA_INVALID_HANDLE)
            .then_some(handle)
            .ok_or(CnaError::InvalidInput("SoundEffect is disposed"))
    }

    fn dispose(&self) -> Result<()> {
        if *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
            == sys::CNA_INVALID_HANDLE
        {
            return Ok(());
        }
        let children = self
            .children
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>();
        for child in children.into_iter().rev() {
            child.dispose()?;
        }
        let mut handle = self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if *handle != sys::CNA_INVALID_HANDLE {
            self.native.destroy_sound_effect(*handle)?;
            *handle = sys::CNA_INVALID_HANDLE;
        }
        self.children.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clear();
        Ok(())
    }
}

impl AudioResourceCleanup for SoundEffectState {
    fn dispose_for_game_shutdown(&self) -> Result<()> { self.dispose() }
}

/// An owned native sound resource created from PCM16 or encoded audio.
pub struct SoundEffect {
    state: Arc<SoundEffectState>,
}

impl SoundEffect {
    pub fn new(game: &GameContext<'_>, buffer: &[u8], sampleRate: i32, channels: AudioChannels) -> Result<Self> {
        let alignment = validate_format(sampleRate, channels)?;
        if buffer.is_empty() || buffer.len() % alignment != 0 {
            return Err(CnaError::InvalidInput("buffer must contain complete PCM16 channel frames"));
        }
        Self::create(game, buffer, sampleRate, channels, None)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_buffer_and_offset_and_count_and_sample_rate_and_channels_and_loop_start_and_loop_length(
        game: &GameContext<'_>, buffer: &[u8], offset: i32, count: i32, sampleRate: i32,
        channels: AudioChannels, loopStart: i32, loopLength: i32,
    ) -> Result<Self> {
        let alignment = validate_format(sampleRate, channels)?;
        if buffer.is_empty() || buffer.len() % alignment != 0 {
            return Err(CnaError::InvalidInput("buffer must contain complete PCM16 channel frames"));
        }
        let offset_usize = usize::try_from(offset).map_err(|_| CnaError::InvalidInput("offset is outside the audio buffer"))?;
        let count_usize = usize::try_from(count).map_err(|_| CnaError::InvalidInput("count is outside the audio buffer"))?;
        let end = offset_usize.checked_add(count_usize).ok_or(CnaError::InvalidInput("offset and count overflow"))?;
        if count == 0 || offset_usize >= buffer.len() || end > buffer.len() || offset_usize % alignment != 0 || count_usize % alignment != 0 {
            return Err(CnaError::InvalidInput("offset and count must select complete PCM16 frames"));
        }
        let frames = count_usize / alignment;
        let loop_start = usize::try_from(loopStart).map_err(|_| CnaError::InvalidInput("loopStart is outside the selected audio"))?;
        let loop_length = usize::try_from(loopLength).map_err(|_| CnaError::InvalidInput("loopLength is outside the selected audio"))?;
        if loop_start > frames || loop_start.checked_add(loop_length).is_none_or(|value| value > frames) {
            return Err(CnaError::InvalidInput("loop range is outside the selected audio"));
        }
        let (loopStart, loopLength) = if loopLength == 0 {
            (0, i32::try_from(frames).map_err(|_| CnaError::InvalidInput("audio sample count is too large"))?)
        } else {
            (loopStart, loopLength)
        };
        Self::create(game, buffer, sampleRate, channels, Some((offset, count, loopStart, loopLength)))
    }

    fn create(game: &GameContext<'_>, buffer: &[u8], sample_rate: i32, channels: AudioChannels, range: Option<(i32, i32, i32, i32)>) -> Result<Self> {
        let runtime = Arc::clone(game.audio_runtime());
        let active = runtime.active()?;
        let handle = active.native.create_sound_effect(active.handle, buffer, sample_rate, native_channels(channels), range)?;
        let duration = match active.native.sound_effect_duration(handle) {
            Ok(ticks) => TimeSpan::from_ticks(ticks),
            Err(error) => { let _ = active.native.destroy_sound_effect(handle); return Err(error); }
        };
        let state = Arc::new(SoundEffectState {
            runtime: Arc::clone(&runtime), native: active.native, generation: active.generation,
            handle: Mutex::new(handle), duration, name: Mutex::new(String::new()),
            children: Mutex::new(Vec::new()),
        });
        runtime.register(&state);
        Ok(Self { state })
    }

    pub fn FromStream<R: Read>(game: &GameContext<'_>, stream: &mut R) -> Result<Self> {
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).map_err(|error| CnaError::Io(format!("failed to read audio stream: {error}")))?;
        if bytes.is_empty() { return Err(CnaError::InvalidInput("audio stream is empty")); }
        let runtime = Arc::clone(game.audio_runtime());
        let active = runtime.active()?;
        let handle = active.native.create_encoded_sound_effect(active.handle, &bytes)?;
        let duration = match active.native.sound_effect_duration(handle) {
            Ok(ticks) => TimeSpan::from_ticks(ticks),
            Err(error) => { let _ = active.native.destroy_sound_effect(handle); return Err(error); }
        };
        let state = Arc::new(SoundEffectState {
            runtime: Arc::clone(&runtime),
            native: active.native,
            generation: active.generation,
            handle: Mutex::new(handle),
            duration,
            name: Mutex::new(String::new()),
            children: Mutex::new(Vec::new()),
        });
        runtime.register(&state);
        Ok(Self { state })
    }

    pub fn IsDisposed(&self) -> bool { *self.state.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner) == sys::CNA_INVALID_HANDLE }
    pub fn Name(&self) -> Result<String> {
        Ok(self
            .state
            .name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
    pub fn SetName(&mut self, value: &str) -> Result<()> {
        if value.is_empty() {
            return Err(CnaError::InvalidInput("SoundEffect.Name must not be empty"));
        }
        self.state
            .native
            .set_sound_effect_name(self.state.require_handle()?, value)?;
        let value = self
            .state
            .native
            .sound_effect_name(self.state.require_handle()?)?;
        *self
            .state
            .name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
        Ok(())
    }
    pub fn Duration(&self) -> Result<TimeSpan> { Ok(self.state.duration) }

    pub fn MasterVolume(game: &GameContext<'_>) -> Result<f32> { let active = game.audio_runtime().active()?; active.native.sound_effect_setting(active.handle, 0) }
    pub fn SetMasterVolume(game: &GameContext<'_>, value: f32) -> Result<()> { validate_range(value, 0.0, 1.0, "MasterVolume must be within [0, 1]")?; let active = game.audio_runtime().active()?; active.native.set_sound_effect_setting(active.handle, 0, value) }
    pub fn DistanceScale(game: &GameContext<'_>) -> Result<f32> { let active = game.audio_runtime().active()?; active.native.sound_effect_setting(active.handle, 1) }
    pub fn SetDistanceScale(game: &GameContext<'_>, value: f32) -> Result<()> { if value < 0.0 { return Err(CnaError::InvalidInput("DistanceScale must not be negative")); } let value = if value <= f32::from_bits(1) { f32::from_bits(1) } else { value }; let active = game.audio_runtime().active()?; active.native.set_sound_effect_setting(active.handle, 1, value) }
    pub fn DopplerScale(game: &GameContext<'_>) -> Result<f32> { let active = game.audio_runtime().active()?; active.native.sound_effect_setting(active.handle, 2) }
    pub fn SetDopplerScale(game: &GameContext<'_>, value: f32) -> Result<()> { if value.is_nan() || value < 0.0 { return Err(CnaError::InvalidInput("DopplerScale must not be negative or NaN")); } let active = game.audio_runtime().active()?; active.native.set_sound_effect_setting(active.handle, 2, value) }
    pub fn SpeedOfSound(game: &GameContext<'_>) -> Result<f32> { let active = game.audio_runtime().active()?; active.native.sound_effect_setting(active.handle, 3) }
    pub fn SetSpeedOfSound(game: &GameContext<'_>, value: f32) -> Result<()> { if value.is_nan() || value <= 0.0 { return Err(CnaError::InvalidInput("SpeedOfSound must be positive and not NaN")); } let active = game.audio_runtime().active()?; active.native.set_sound_effect_setting(active.handle, 3, value) }

    pub fn CreateInstance(&self) -> Result<SoundEffectInstance> {
        let handle = self.state.native.create_sound_effect_instance(self.state.require_handle()?)?;
        let instance = InstanceState::new(Arc::clone(&self.state.runtime), Arc::clone(&self.state.native), self.state.generation, handle, Some(Arc::clone(&self.state)));
        self.state.children.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(Arc::downgrade(&instance));
        self.state.runtime.register(&instance);
        Ok(SoundEffectInstance { state: instance })
    }

    pub fn Play(&self) -> Result<bool> { self.state.native.play_sound_effect(self.state.require_handle()?, None) }
    pub fn PlayWithVolumeAndPitchAndPan(&self, volume: f32, pitch: f32, pan: f32) -> Result<bool> {
        validate_range(volume, 0.0, 1.0, "volume must be within [0, 1]")?;
        validate_range(pitch, -1.0, 1.0, "pitch must be within [-1, 1]")?;
        validate_range(pan, -1.0, 1.0, "pan must be within [-1, 1]")?;
        self.state.native.play_sound_effect(self.state.require_handle()?, Some((volume, pitch, pan)))
    }

    pub fn GetSampleDuration(sizeInBytes: i32, sampleRate: i32, channels: AudioChannels) -> TimeSpan { sample_duration(sizeInBytes, sampleRate, channels) }
    pub fn GetSampleSizeInBytes(duration: TimeSpan, sampleRate: i32, channels: AudioChannels) -> i32 { sample_size(duration, sampleRate, channels) }
    pub fn Finalize(&self) {}
    pub fn Dispose(&mut self) -> Result<()> { self.state.dispose() }
}

impl Drop for SoundEffect {
    fn drop(&mut self) { let _ = self.state.dispose(); }
}

#[derive(Clone, Copy)]
struct CachedInstance {
    volume: f32,
    pitch: f32,
    pan: f32,
    looped: bool,
    is_3d: bool,
    has_played: bool,
}

pub(super) struct InstanceState {
    runtime: Arc<AudioRuntime>,
    native: Arc<Native>,
    generation: u64,
    handle: Mutex<sys::CNA_Handle>,
    cached: Mutex<CachedInstance>,
    _parent: Option<Arc<SoundEffectState>>,
}

impl InstanceState {
    pub(super) fn new(runtime: Arc<AudioRuntime>, native: Arc<Native>, generation: u64, handle: sys::CNA_Handle, parent: Option<Arc<SoundEffectState>>) -> Arc<Self> {
        Arc::new(Self { runtime, native, generation, handle: Mutex::new(handle), cached: Mutex::new(CachedInstance { volume: 1.0, pitch: 0.0, pan: 0.0, looped: false, is_3d: false, has_played: false }), _parent: parent })
    }
    pub(super) fn require_handle(&self) -> Result<sys::CNA_Handle> { self.runtime.ensure_generation(self.generation)?; let handle = *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner); (handle != 0).then_some(handle).ok_or(CnaError::InvalidInput("SoundEffectInstance is disposed")) }
    pub(super) fn native(&self) -> &Native { &self.native }
    pub(super) fn dispose(&self) -> Result<()> { let mut handle = self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner); if *handle != 0 { self.native.destroy_sound_effect_instance(*handle)?; *handle = 0; } Ok(()) }
    pub(super) fn is_disposed(&self) -> bool { *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner) == 0 }
    pub(super) fn volume(&self) -> f32 { self.cached.lock().unwrap_or_else(std::sync::PoisonError::into_inner).volume }
    pub(super) fn pitch(&self) -> f32 { self.cached.lock().unwrap_or_else(std::sync::PoisonError::into_inner).pitch }
    pub(super) fn pan(&self) -> f32 { self.cached.lock().unwrap_or_else(std::sync::PoisonError::into_inner).pan }
    fn cached(&self) -> CachedInstance { *self.cached.lock().unwrap_or_else(std::sync::PoisonError::into_inner) }
    pub(super) fn set_float(&self, property: u8, value: f32) -> Result<()> {
        if property == 2 {
            let mut cached = self
                .cached
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !cached.has_played {
                cached.is_3d = false;
            }
            if cached.is_3d {
                return Err(CnaError::InvalidInput(
                    "Pan cannot be set after spatial playback has begun",
                ));
            }
        }
        self.native.set_instance_float(self.require_handle()?, property, value)?;
        let mut cached = self.cached.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        match property { 0 => cached.volume = value, 1 => cached.pitch = value, _ => cached.pan = value };
        Ok(())
    }
    pub(super) fn set_looped(&self, value: bool) -> Result<()> {
        if self
            .cached
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .has_played
        {
            return Err(CnaError::InvalidInput(
                "IsLooped cannot be changed after the first Play",
            ));
        }
        self.native.set_instance_looped(self.require_handle()?, value)?;
        self.cached.lock().unwrap_or_else(std::sync::PoisonError::into_inner).looped = value;
        Ok(())
    }
    pub(super) fn state(&self) -> Result<SoundState> { Ok(match self.native.instance_info(self.require_handle()?)?.state { sys::CNA_SOUND_STATE_PLAYING => SoundState::Playing, sys::CNA_SOUND_STATE_PAUSED => SoundState::Paused, _ => SoundState::Stopped }) }
    pub(super) fn transport(&self, action: u8, immediate: bool) -> Result<()> {
        self.native.instance_transport(self.require_handle()?, action, immediate)?;
        if action == 0 {
            self.cached
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .has_played = true;
        }
        Ok(())
    }
    pub(super) fn apply_3d(&self, listeners: &[AudioListener], emitter: &AudioEmitter) -> Result<()> {
        if listeners.is_empty() {
            return Err(CnaError::InvalidInput("at least one AudioListener is required"));
        }
        {
            let cached = self.cached.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if cached.has_played && !cached.is_3d {
                return Err(CnaError::InvalidInput(
                    "Apply3D must be called before the first non-spatial Play",
                ));
            }
        }
        let native = listeners.iter().map(native_listener).collect::<Vec<_>>();
        self.native.apply_instance_3d(
            self.require_handle()?,
            &native,
            &native_emitter(emitter),
        )?;
        self.cached
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_3d = true;
        Ok(())
    }
}

impl AudioResourceCleanup for InstanceState { fn dispose_for_game_shutdown(&self) -> Result<()> { self.dispose() } }

/// Public behavior composed by normal and dynamic sound-effect instances.
pub trait SoundEffectInstanceBase {
    fn IsDisposed(&self) -> bool;
    fn Volume(&self) -> f32;
    fn SetVolume(&mut self, value: f32) -> Result<()>;
    fn Pitch(&self) -> f32;
    fn SetPitch(&mut self, value: f32) -> Result<()>;
    fn Pan(&self) -> f32;
    fn SetPan(&mut self, value: f32) -> Result<()>;
    fn IsLooped(&self) -> bool;
    fn SetIsLooped(&mut self, value: bool) -> Result<()>;
    fn State(&self) -> Result<SoundState>;
    fn Play(&self) -> Result<()>;
    fn Stop(&self) -> Result<()>;
    fn StopWithImmediate(&self, immediate: bool) -> Result<()>;
    fn Pause(&self) -> Result<()>;
    fn Resume(&self) -> Result<()>;
    fn Apply3D(&self, listener: &AudioListener, emitter: &AudioEmitter) -> Result<()>;
    fn Apply3DWithListenersAndEmitter(&self, listeners: &[AudioListener], emitter: &AudioEmitter) -> Result<()>;
}

pub struct SoundEffectInstance { pub(super) state: Arc<InstanceState> }

impl SoundEffectInstanceBase for SoundEffectInstance {
    fn IsDisposed(&self) -> bool { self.state.is_disposed() }
    fn Volume(&self) -> f32 { self.state.cached().volume }
    fn SetVolume(&mut self, value: f32) -> Result<()> { validate_range(value, 0.0, 1.0, "Volume must be within [0, 1]")?; self.state.set_float(0, value) }
    fn Pitch(&self) -> f32 { self.state.cached().pitch }
    fn SetPitch(&mut self, value: f32) -> Result<()> { validate_range(value, -1.0, 1.0, "Pitch must be within [-1, 1]")?; self.state.set_float(1, value) }
    fn Pan(&self) -> f32 { self.state.cached().pan }
    fn SetPan(&mut self, value: f32) -> Result<()> { validate_range(value, -1.0, 1.0, "Pan must be within [-1, 1]")?; self.state.set_float(2, value) }
    fn IsLooped(&self) -> bool { self.state.cached().looped }
    fn SetIsLooped(&mut self, value: bool) -> Result<()> { self.state.set_looped(value) }
    fn State(&self) -> Result<SoundState> { self.state.state() }
    fn Play(&self) -> Result<()> { self.state.transport(0, false) }
    fn Stop(&self) -> Result<()> { self.StopWithImmediate(true) }
    fn StopWithImmediate(&self, immediate: bool) -> Result<()> { self.state.transport(3, immediate) }
    fn Pause(&self) -> Result<()> { self.state.transport(1, false) }
    fn Resume(&self) -> Result<()> { self.state.transport(2, false) }
    fn Apply3D(&self, listener: &AudioListener, emitter: &AudioEmitter) -> Result<()> { self.state.apply_3d(core::slice::from_ref(listener), emitter) }
    fn Apply3DWithListenersAndEmitter(&self, listeners: &[AudioListener], emitter: &AudioEmitter) -> Result<()> { self.state.apply_3d(listeners, emitter) }
}

impl SoundEffectInstance {
    pub fn IsDisposed(&self) -> bool { <Self as SoundEffectInstanceBase>::IsDisposed(self) }
    pub fn Volume(&self) -> f32 { <Self as SoundEffectInstanceBase>::Volume(self) }
    pub fn SetVolume(&mut self, value: f32) -> Result<()> { <Self as SoundEffectInstanceBase>::SetVolume(self, value) }
    pub fn Pitch(&self) -> f32 { <Self as SoundEffectInstanceBase>::Pitch(self) }
    pub fn SetPitch(&mut self, value: f32) -> Result<()> { <Self as SoundEffectInstanceBase>::SetPitch(self, value) }
    pub fn Pan(&self) -> f32 { <Self as SoundEffectInstanceBase>::Pan(self) }
    pub fn SetPan(&mut self, value: f32) -> Result<()> { <Self as SoundEffectInstanceBase>::SetPan(self, value) }
    pub fn IsLooped(&self) -> bool { <Self as SoundEffectInstanceBase>::IsLooped(self) }
    pub fn SetIsLooped(&mut self, value: bool) -> Result<()> { <Self as SoundEffectInstanceBase>::SetIsLooped(self, value) }
    pub fn State(&self) -> Result<SoundState> { <Self as SoundEffectInstanceBase>::State(self) }
    pub fn Play(&self) -> Result<()> { <Self as SoundEffectInstanceBase>::Play(self) }
    pub fn Stop(&self) -> Result<()> { <Self as SoundEffectInstanceBase>::Stop(self) }
    pub fn StopWithImmediate(&self, immediate: bool) -> Result<()> { <Self as SoundEffectInstanceBase>::StopWithImmediate(self, immediate) }
    pub fn Pause(&self) -> Result<()> { <Self as SoundEffectInstanceBase>::Pause(self) }
    pub fn Resume(&self) -> Result<()> { <Self as SoundEffectInstanceBase>::Resume(self) }
    pub fn Apply3D(&self, listener: &AudioListener, emitter: &AudioEmitter) -> Result<()> { <Self as SoundEffectInstanceBase>::Apply3D(self, listener, emitter) }
    pub fn Apply3DWithListenersAndEmitter(&self, listeners: &[AudioListener], emitter: &AudioEmitter) -> Result<()> { <Self as SoundEffectInstanceBase>::Apply3DWithListenersAndEmitter(self, listeners, emitter) }
    pub fn Finalize(&self) {}
    pub fn Dispose(&mut self) -> Result<()> { self.state.dispose() }
    pub fn DisposeWithDisposing(&mut self, disposing: bool) -> Result<()> {
        let _ = disposing;
        self.state.dispose()
    }
}

impl Drop for SoundEffectInstance { fn drop(&mut self) { let _ = self.state.dispose(); } }
