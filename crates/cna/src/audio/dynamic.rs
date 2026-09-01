use std::sync::{Arc, Mutex, Weak};

use crate::error::{CnaError, Result};
use crate::extensions::events::{EventArgs, EventHandler};
use crate::game::{GameContext, TimeSpan};
use crate::graphics::resource::EventHandlers;

use super::runtime::{
    audio_event_trampoline, AudioCallbackToken, AudioEventTarget, AudioResourceCleanup,
    NativeAudioRegistration,
};
use super::sound::{native_channels, validate_range, InstanceState};
use super::{AudioChannels, AudioEmitter, AudioListener, SoundEffectInstanceBase, SoundState};

struct DynamicState {
    self_weak: Weak<DynamicState>,
    instance: Arc<InstanceState>,
    channels: AudioChannels,
    events: EventHandlers<EventArgs>,
    registration: Mutex<Option<NativeAudioRegistration>>,
}

impl DynamicState {
    fn dispose(&self) -> Result<()> {
        let mut registration = self.registration.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(value) = registration.as_mut() {
            value.token.deactivate();
            if let Err(error) = self.instance.native().unsubscribe_audio(value.handle) {
                value.token.reactivate();
                return Err(error);
            }
            registration.take();
        }
        drop(registration);
        self.instance.dispose()
    }

}

impl AudioResourceCleanup for DynamicState {
    fn dispose_for_game_shutdown(&self) -> Result<()> { self.dispose() }
}

impl AudioEventTarget for DynamicState {
    fn dispatch_audio_event(&self) -> bool {
        let Some(state) = self.self_weak.upgrade() else { return false; };
        let sender = DynamicSoundEffectInstance { state, owns_drop: false };
        self.events.emit(&sender, EventArgs)
    }
    fn accepts_audio_events(&self) -> bool { !self.instance.is_disposed() }
}

/// Procedural PCM16 playback fed by caller-submitted buffers.
pub struct DynamicSoundEffectInstance {
    state: Arc<DynamicState>,
    owns_drop: bool,
}

impl DynamicSoundEffectInstance {
    pub fn new(game: &GameContext<'_>, sampleRate: i32, channels: AudioChannels) -> Result<Self> {
        if !(8_000..=48_000).contains(&sampleRate) {
            return Err(CnaError::InvalidInput("sampleRate must be between 8000 and 48000"));
        }
        let runtime = Arc::clone(game.audio_runtime());
        let active = runtime.active()?;
        let handle = active.native.create_dynamic_instance(active.handle, sampleRate, native_channels(channels))?;
        let instance = InstanceState::new(Arc::clone(&runtime), Arc::clone(&active.native), active.generation, handle, None);
        let state = Arc::new_cyclic(|weak| DynamicState {
            self_weak: weak.clone(), instance, channels, events: EventHandlers::new(),
            registration: Mutex::new(None),
        });
        let target: Arc<dyn AudioEventTarget> = state.clone();
        let mut token = AudioCallbackToken::new(&runtime, &target);
        let registration = match active.native.subscribe_dynamic(handle, Some(audio_event_trampoline), token.context()) {
            Ok(registration) => registration,
            Err(error) => { let _ = active.native.destroy_sound_effect_instance(handle); return Err(error); }
        };
        *state.registration.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(NativeAudioRegistration { handle: registration, token });
        runtime.register(&state);
        Ok(Self { state, owns_drop: true })
    }

    pub fn IsLooped(&self) -> Result<bool> { self.state.instance.require_handle()?; Ok(false) }
    pub fn SetIsLooped(&mut self, value: bool) -> Result<()> { self.state.instance.require_handle()?; if value { Err(CnaError::InvalidInput("a DynamicSoundEffectInstance cannot be looped")) } else { Ok(()) } }
    pub fn PendingBufferCount(&self) -> Result<i32> { let handle = self.state.instance.require_handle()?; self.state.instance.native().dynamic_pending_count(handle) }
    pub fn AddBufferNeededHandler(&self, handler: Box<dyn EventHandler>) -> u64 { self.state.events.add(handler) }
    pub fn RemoveBufferNeededHandler(&self, registration: u64) -> bool { self.state.events.remove(registration) }
    pub fn Dispose(&mut self, disposing: bool) -> Result<()> {
        let _ = disposing;
        self.state.dispose()
    }

    pub fn SubmitBuffer(&self, buffer: &[u8]) -> Result<()> {
        let count = i32::try_from(buffer.len()).map_err(|_| CnaError::InvalidInput("audio buffer is too large"))?;
        self.SubmitBufferWithBufferAndOffsetAndCount(buffer, 0, count)
    }

    pub fn SubmitBufferWithBufferAndOffsetAndCount(&self, buffer: &[u8], offset: i32, count: i32) -> Result<()> {
        let alignment = self.state.channels as usize * 2;
        if buffer.is_empty() || buffer.len() % alignment != 0 {
            return Err(CnaError::InvalidInput("audio buffer must contain complete PCM16 channel frames"));
        }
        let offset_usize = usize::try_from(offset).map_err(|_| CnaError::InvalidInput("audio buffer offset is invalid"))?;
        let count_usize = usize::try_from(count).map_err(|_| CnaError::InvalidInput("audio buffer count is invalid"))?;
        let end = offset_usize.checked_add(count_usize).ok_or(CnaError::InvalidInput("audio buffer range overflows"))?;
        if count == 0 || offset_usize >= buffer.len() || end > buffer.len() || offset_usize % alignment != 0 || count_usize % alignment != 0 {
            return Err(CnaError::InvalidInput("audio buffer range must contain complete PCM16 channel frames"));
        }
        let handle = self.state.instance.require_handle()?;
        self.state.instance.native().dynamic_submit(handle, buffer, offset, count)
    }

    pub fn GetSampleDuration(&self, sizeInBytes: i32) -> Result<TimeSpan> {
        if sizeInBytes < 0 { return Err(CnaError::InvalidInput("sizeInBytes must not be negative")); }
        let ticks = self.state.instance.native().dynamic_duration(self.state.instance.require_handle()?, sizeInBytes)?;
        Ok(TimeSpan::from_ticks(ticks))
    }

    pub fn GetSampleSizeInBytes(&self, duration: TimeSpan) -> Result<i32> {
        if duration.Ticks() < 0 || duration.Ticks() > i64::from(i32::MAX) * TimeSpan::TicksPerMillisecond {
            return Err(CnaError::InvalidInput("duration is outside the supported range"));
        }
        self.state.instance.native().dynamic_size(self.state.instance.require_handle()?, duration.Ticks())
    }

    pub fn Play(&self) -> Result<()> { self.state.instance.transport(0, false) }
}

impl SoundEffectInstanceBase for DynamicSoundEffectInstance {
    fn IsDisposed(&self) -> bool { self.state.instance.is_disposed() }
    fn Volume(&self) -> f32 { self.state.instance.volume() }
    fn SetVolume(&mut self, value: f32) -> Result<()> {
        validate_range(value, 0.0, 1.0, "Volume must be within [0, 1]")?;
        self.state.instance.set_float(0, value)
    }
    fn Pitch(&self) -> f32 { self.state.instance.pitch() }
    fn SetPitch(&mut self, value: f32) -> Result<()> {
        validate_range(value, -1.0, 1.0, "Pitch must be within [-1, 1]")?;
        self.state.instance.set_float(1, value)
    }
    fn Pan(&self) -> f32 { self.state.instance.pan() }
    fn SetPan(&mut self, value: f32) -> Result<()> {
        validate_range(value, -1.0, 1.0, "Pan must be within [-1, 1]")?;
        self.state.instance.set_float(2, value)
    }
    fn IsLooped(&self) -> bool { false }
    fn SetIsLooped(&mut self, value: bool) -> Result<()> { self.SetIsLooped(value) }
    fn State(&self) -> Result<SoundState> { self.state.instance.state() }
    fn Play(&self) -> Result<()> { self.Play() }
    fn Stop(&self) -> Result<()> { self.state.instance.transport(3, true) }
    fn StopWithImmediate(&self, immediate: bool) -> Result<()> {
        self.state.instance.transport(3, immediate)
    }
    fn Pause(&self) -> Result<()> { self.state.instance.transport(1, false) }
    fn Resume(&self) -> Result<()> { self.state.instance.transport(2, false) }
    fn Apply3D(&self, listener: &AudioListener, emitter: &AudioEmitter) -> Result<()> {
        self.state.instance.apply_3d(core::slice::from_ref(listener), emitter)
    }
    fn Apply3DWithListenersAndEmitter(&self, listeners: &[AudioListener], emitter: &AudioEmitter) -> Result<()> {
        self.state.instance.apply_3d(listeners, emitter)
    }
}

impl Drop for DynamicSoundEffectInstance {
    fn drop(&mut self) { if self.owns_drop { let _ = self.state.dispose(); } }
}

/// The `audio.h` streaming routes with no XNA counterpart.
///
/// XNA's `DynamicSoundEffectInstance` takes 16-bit PCM bytes and nothing else.
/// These are the paths CNA adds around that: float samples rather than packed
/// bytes, the initial queue that primes playback before the first
/// `BufferNeeded`, an explicit clear, and the pump that retires finished
/// buffers.
///
/// [`Update`](Self::Update) is the one worth reading twice. XNA's instance
/// raises `BufferNeeded` from the audio engine's own servicing; a caller that
/// drives playback itself -- a test, or a game with its own mixer cadence --
/// has no way to make that happen. `Update` is that way.
impl DynamicSoundEffectInstance {
    /// Submits a range of 32-bit float samples, which CNA copies during the
    /// call.
    ///
    /// The whole slice is passed alongside the range, so the offset and count
    /// are checked against it here: an out-of-range pair would otherwise be a
    /// read past the buffer.
    pub fn SubmitFloatBuffer(&self, buffer: &[f32], offset: i32, count: i32) -> Result<()> {
        let handle = self.state.instance.require_handle()?;
        self.state
            .instance
            .native()
            .submit_dynamic_float_buffer(handle, buffer, offset, count)
    }

    /// Queues the buffers playback starts with.
    pub fn QueueInitialBuffers(&self) -> Result<()> {
        let handle = self.state.instance.require_handle()?;
        self.state
            .instance
            .native()
            .queue_dynamic_initial_buffers(handle)
    }

    /// Drops every queued buffer without playing it.
    pub fn ClearBuffers(&self) -> Result<()> {
        let handle = self.state.instance.require_handle()?;
        self.state.instance.native().clear_dynamic_buffers(handle)
    }

    /// Retires finished buffers and raises `BufferNeeded` for what that frees.
    ///
    /// Servicing the instance by hand, for a caller driving playback on its own
    /// cadence rather than the audio engine's.
    pub fn Update(&self) -> Result<()> {
        let handle = self.state.instance.require_handle()?;
        self.state.instance.native().update_dynamic_instance(handle)
    }
}
