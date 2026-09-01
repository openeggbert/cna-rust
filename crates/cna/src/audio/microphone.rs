use std::sync::{Arc, Mutex, Weak};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::{EventArgs, EventHandler};
use crate::game::{GameContext, TimeSpan};
use crate::graphics::resource::EventHandlers;
use crate::native::Native;
use crate::extensions::audio_ext::MicrophoneExt;

use super::runtime::{
    audio_event_trampoline, AudioCallbackToken, AudioEventTarget, AudioResourceCleanup,
    AudioRuntime, NativeAudioRegistration,
};
use super::MicrophoneState;

struct MicrophoneInner {
    runtime: Arc<AudioRuntime>,
    native: Arc<Native>,
    game: sys::CNA_Handle,
    generation: u64,
    index: u64,
    sender: Mutex<Weak<Microphone>>,
    events: EventHandlers<EventArgs>,
    registration: Mutex<Option<NativeAudioRegistration>>,
}

impl MicrophoneInner {
    fn ensure_active(&self) -> Result<()> {
        self.runtime.ensure_generation(self.generation).map(|_| ())
    }

    fn dispose_registration(&self) -> Result<()> {
        let mut registration = self.registration.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(value) = registration.as_mut() {
            value.token.deactivate();
            if let Err(error) = self.native.unsubscribe_audio(value.handle) {
                value.token.reactivate();
                return Err(error);
            }
            registration.take();
        }
        Ok(())
    }
}

impl AudioResourceCleanup for MicrophoneInner {
    fn dispose_for_game_shutdown(&self) -> Result<()> { self.dispose_registration() }
}

impl AudioEventTarget for MicrophoneInner {
    fn dispatch_audio_event(&self) -> bool {
        self.sender
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .upgrade()
            .is_some_and(|sender| self.events.emit(sender.as_ref(), EventArgs))
    }
    fn accepts_audio_events(&self) -> bool { self.ensure_active().is_ok() }
}

/// Stable facade for a runtime-owned capture device.
pub struct Microphone {
    pub Name: String,
    inner: Arc<MicrophoneInner>,
}

impl Microphone {
    pub fn All(game: &GameContext<'_>) -> Result<Vec<Arc<Microphone>>> {
        let runtime = Arc::clone(game.audio_runtime());
        let active = runtime.active()?;
        if let Some(values) = runtime.microphones(active.generation) { return Ok(values); }
        let count = active.native.microphone_count(active.handle)?;
        let capacity = usize::try_from(count).map_err(|_| CnaError::InvalidInput("microphone count is too large"))?;
        let mut values = Vec::with_capacity(capacity);
        for index in 0..count {
            let name = active.native.microphone_name(active.handle, index)?;
            let inner = Arc::new(MicrophoneInner {
                runtime: Arc::clone(&runtime), native: Arc::clone(&active.native), game: active.handle,
                generation: active.generation, index, sender: Mutex::new(Weak::new()),
                events: EventHandlers::new(), registration: Mutex::new(None),
            });
            let microphone = Arc::new(Self { Name: name, inner: Arc::clone(&inner) });
            *inner.sender.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Arc::downgrade(&microphone);
            let target: Arc<dyn AudioEventTarget> = inner.clone();
            let mut token = AudioCallbackToken::new(&runtime, &target);
            let registration = active.native.subscribe_microphone(active.handle, index, Some(audio_event_trampoline), token.context())?;
            *inner.registration.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(NativeAudioRegistration { handle: registration, token });
            runtime.register(&inner);
            values.push(microphone);
        }
        runtime.set_microphones(active.generation, &values);
        Ok(values)
    }

    pub fn Default(game: &GameContext<'_>) -> Result<Option<Arc<Microphone>>> {
        let active = game.audio_runtime().active()?;
        let index = active.native.microphone_default(active.handle)?;
        let all = Self::All(game)?;
        index.map_or(Ok(None), |index| {
            let index = usize::try_from(index).map_err(|_| CnaError::InvalidInput("default microphone index is too large"))?;
            all.get(index).cloned().map(Some).ok_or(CnaError::InvalidInput("default microphone index is outside Microphone.All"))
        })
    }

    pub fn State(&self) -> Result<MicrophoneState> {
        self.inner.ensure_active()?;
        Ok(match self.inner.native.microphone_state(self.inner.game, self.inner.index)? {
            sys::CNA_MICROPHONE_STATE_STARTED => MicrophoneState::Started,
            _ => MicrophoneState::Stopped,
        })
    }
    pub fn BufferDuration(&self) -> Result<TimeSpan> { self.inner.ensure_active()?; Ok(TimeSpan::from_ticks(self.inner.native.microphone_buffer_duration(self.inner.game, self.inner.index)?)) }
    pub fn SetBufferDuration(&self, value: TimeSpan) -> Result<()> {
        const TICKS_PER_MILLISECOND: i64 = 10_000;
        let ticks = value.Ticks();
        if !(100 * TICKS_PER_MILLISECOND..=1_000 * TICKS_PER_MILLISECOND).contains(&ticks)
            || ticks % (10 * TICKS_PER_MILLISECOND) != 0
        {
            return Err(CnaError::InvalidInput(
                "BufferDuration must be 100 through 1000 milliseconds in 10 millisecond increments",
            ));
        }
        self.inner.ensure_active()?;
        self.inner
            .native
            .set_microphone_buffer_duration(self.inner.game, self.inner.index, ticks)
    }
    pub fn SampleRate(&self) -> Result<i32> { self.inner.ensure_active()?; self.inner.native.microphone_sample_rate(self.inner.game, self.inner.index) }
    pub fn IsHeadset(&self) -> Result<bool> { self.inner.ensure_active()?; self.inner.native.microphone_is_headset(self.inner.game, self.inner.index) }
    pub fn AddBufferReadyHandler(&self, handler: Box<dyn EventHandler>) -> u64 { self.inner.events.add(handler) }
    pub fn RemoveBufferReadyHandler(&self, registration: u64) -> bool { self.inner.events.remove(registration) }
    pub fn Finalize(&self) {}
    pub fn GetSampleSizeInBytes(&self, duration: TimeSpan) -> Result<i32> {
        if duration.Ticks() < 0
            || duration.Ticks() > i64::from(i32::MAX) * TimeSpan::TicksPerMillisecond
        {
            return Err(CnaError::InvalidInput("duration is outside the supported range"));
        }
        self.inner.ensure_active()?;
        self.inner
            .native
            .microphone_size(self.inner.game, self.inner.index, duration.Ticks())
    }
    pub fn GetSampleDuration(&self, sizeInBytes: i32) -> Result<TimeSpan> { if sizeInBytes < 0 { return Err(CnaError::InvalidInput("sizeInBytes must not be negative")); } self.inner.ensure_active()?; Ok(TimeSpan::from_ticks(self.inner.native.microphone_duration(self.inner.game, self.inner.index, sizeInBytes)?)) }
    pub fn Start(&self) -> Result<()> { self.inner.ensure_active()?; self.inner.native.microphone_transport(self.inner.game, self.inner.index, true) }
    pub fn Stop(&self) -> Result<()> { self.inner.ensure_active()?; self.inner.native.microphone_transport(self.inner.game, self.inner.index, false) }
    pub fn GetData(&self, buffer: &mut [u8]) -> Result<i32> {
        let count = i32::try_from(buffer.len()).map_err(|_| CnaError::InvalidInput("capture buffer is too large"))?;
        self.GetDataWithBufferAndOffsetAndCount(buffer, 0, count)
    }
    pub fn GetDataWithBufferAndOffsetAndCount(&self, buffer: &mut [u8], offset: i32, count: i32) -> Result<i32> {
        if buffer.is_empty() || buffer.len() % 2 != 0 {
            return Err(CnaError::InvalidInput(
                "capture buffer must contain aligned 16-bit mono samples",
            ));
        }
        let offset = usize::try_from(offset).map_err(|_| CnaError::InvalidInput("capture offset is invalid"))?;
        let count = usize::try_from(count).map_err(|_| CnaError::InvalidInput("capture count is invalid"))?;
        let end = offset.checked_add(count).ok_or(CnaError::InvalidInput("capture range overflows"))?;
        if offset >= buffer.len() || offset % 2 != 0 || count == 0 || count % 2 != 0 || end > buffer.len() {
            return Err(CnaError::InvalidInput("capture range is outside the aligned sample buffer"));
        }
        self.inner.ensure_active()?;
        let count_i32 = i32::try_from(count)
            .map_err(|_| CnaError::InvalidInput("capture count is too large"))?;
        if self
            .inner
            .native
            .microphone_duration(self.inner.game, self.inner.index, count_i32)?
            == 0
        {
            return Err(CnaError::InvalidInput("capture range duration rounds to zero"));
        }
        if self.State()? != MicrophoneState::Started {
            return Ok(0);
        }
        let copied = self.inner.native.microphone_data(self.inner.game, self.inner.index, &mut buffer[offset..end])?;
        i32::try_from(copied).map_err(|_| CnaError::InvalidInput("captured byte count is too large"))
    }
}

impl MicrophoneExt for Microphone {
    fn CheckAllBuffers(game: &GameContext<'_>) -> Result<()> {
        let (native, handle) = game.native_game();
        native.check_all_microphone_buffers(handle)
    }
}
