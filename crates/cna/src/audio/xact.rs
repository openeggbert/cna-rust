use core::any::Any;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::{EventArgs, EventHandler};
use crate::game::{GameContext, TimeSpan};
use crate::graphics::resource::EventHandlers;
use crate::native::Native;
use crate::extensions::audio_ext::AudioEngineExt;
use crate::extensions::audio_ext::NativeDisposalState;

use super::runtime::{AudioResourceCleanup, AudioRuntime};
use super::sound::{native_emitter, native_listener};
use super::{AudioEmitter, AudioListener, AudioStopOptions, RendererDetail};

fn validate_xact_header(filename: &str, expected: [u8; 4], kind: &'static str) -> Result<()> {
    let mut file = File::open(filename)
        .map_err(|error| CnaError::Io(format!("failed to open {kind} file: {error}")))?;
    let length = file
        .metadata()
        .map_err(|error| CnaError::Io(format!("failed to inspect {kind} file: {error}")))?
        .len();
    let mut signature = [0_u8; 4];
    if length <= 4 || file.read_exact(&mut signature).is_err() || signature != expected {
        return Err(CnaError::InvalidInput("invalid XACT content signature"));
    }
    Ok(())
}

/// CNA's own disposal notification, kept beside the XNA-shaped event.
///
/// `RUST-EXT-017` is two halves. The Rust `Disposing` had to start firing where
/// XNA's does -- an engine's teardown of its banks and cues, which released the
/// handle and said nothing. Binding CNA's `*_subscribe_disposing_ext` alone
/// would have made the two visibly disagree instead of fixing either, so both
/// land together and each one measures the other: every path that raises the
/// XNA event also raises CNA's, and a test can ask both.
///
/// The subscription is the object's own and is released *before* the handle it
/// watches, so no callback can arrive against a destroyed object. The context
/// is a boxed probe the object owns for exactly as long as the subscription.
struct DisposingProbe {
    fired: AtomicBool,
}

/// A live native subscription and the box CNA holds a pointer into.
struct NativeDisposing {
    registration: sys::CNA_Handle,
    probe: Box<DisposingProbe>,
}

/// CNA's disposal notification, arriving on the thread doing the disposing.
///
/// It records and returns. Nothing here can panic across C: the flag is an
/// atomic store, and the `catch_unwind` is there because a future body might
/// not be.
unsafe extern "C" fn disposing_probe_trampoline(context: *mut core::ffi::c_void) {
    if context.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: the owning state keeps this box alive until it has
        // unsubscribed, and it unsubscribes before releasing its handle.
        let probe = unsafe { &*context.cast::<DisposingProbe>() };
        probe.fired.store(true, Ordering::Release);
    }));
}

/// Subscribes an object to CNA's disposal notification.
///
/// A refusal is not fatal: the XNA-shaped event is the contract and this is the
/// cross-check, so an artifact that will not publish the notification leaves
/// `cna_raised_disposing` answering `false` rather than failing construction.
fn watch_disposal(
    native: &Arc<Native>,
    kind: u8,
    handle: sys::CNA_Handle,
) -> Mutex<Option<NativeDisposing>> {
    let mut probe = Box::new(DisposingProbe {
        fired: AtomicBool::new(false),
    });
    let context = core::ptr::addr_of_mut!(*probe).cast();
    match native.subscribe_xact_disposing(kind, handle, Some(disposing_probe_trampoline), context) {
        Ok(registration) => Mutex::new(Some(NativeDisposing {
            registration,
            probe,
        })),
        Err(_) => Mutex::new(None),
    }
}

/// Releases the subscription, after the destruction it exists to observe.
///
/// The order is deliberate and is the opposite of the usual rule. CNA raises
/// this notification *during* `cna_*_destroy`, so unsubscribing first would
/// remove the only thing the cross-check measures. What has to hold instead is
/// that no callback can arrive against a released box, and it does: the box
/// lives in the object's own state and is dropped here, after the destroy has
/// returned and the registration has been released.
fn stop_watching(native: &Arc<Native>, slot: &Mutex<Option<NativeDisposing>>) -> bool {
    let taken = slot
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    match taken {
        Some(watch) => {
            let fired = watch.probe.fired.load(Ordering::Acquire);
            let _ = native.unsubscribe_audio(watch.registration);
            // The box drops here, after CNA has released the registration.
            drop(watch.probe);
            fired
        }
        None => false,
    }
}

/// A wave bank, sound bank or cue an `AudioEngine` disposes with itself.
///
/// XNA's `AudioEngine.Dispose` calls `NotifyDestroyedEngine`, which walks the
/// process-wide instance registry and calls the **public** `Dispose()` on
/// every `Cue`, `SoundBank` and `WaveBank` created against that engine -- so
/// each one's `Disposing` fires, exactly as it would have on an explicit
/// dispose. That is what this path has to reproduce, and what it did not:
/// it called the private teardown, which releases the handle and says nothing.
trait XactChild: Send + Sync {
    /// Disposes the child the way its owning engine does: raising `Disposing`.
    fn dispose_owned_by_engine(&self) -> Result<()>;
}

struct AudioEngineState {
    runtime: Arc<AudioRuntime>,
    native: Arc<Native>,
    generation: u64,
    handle: Mutex<sys::CNA_Handle>,
    children: Mutex<Vec<Weak<dyn XactChild>>>,
    categories: Mutex<HashMap<String, Weak<AudioCategoryState>>>,
    disposing: EventHandlers<EventArgs>,
    watch: Mutex<Option<NativeDisposing>>,
    cna_raised: AtomicBool,
}

impl AudioEngineState {
    fn require_handle(&self) -> Result<sys::CNA_Handle> {
        self.runtime.ensure_generation(self.generation)?;
        let handle = *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        (handle != 0).then_some(handle).ok_or(CnaError::InvalidInput("AudioEngine is disposed"))
    }
    fn register_child<T>(&self, child: &Arc<T>) where T: XactChild + 'static {
        let erased: Arc<dyn XactChild> = child.clone();
        self.children.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(Arc::downgrade(&erased));
    }
    fn dispose(&self) -> Result<()> {
        if *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner) == 0 { return Ok(()); }
        let children = self.children.lock().unwrap_or_else(std::sync::PoisonError::into_inner).iter().filter_map(Weak::upgrade).collect::<Vec<_>>();
        for child in children.into_iter().rev() { child.dispose_owned_by_engine()?; }
        let mut handle = self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if *handle != 0 { self.native.destroy_audio_engine(*handle)?; *handle = 0; }
        if stop_watching(&self.native, &self.watch) { self.cna_raised.store(true, Ordering::Release); }
        self.children.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clear();
        self.categories.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clear();
        Ok(())
    }
}

impl AudioResourceCleanup for AudioEngineState { fn dispose_for_game_shutdown(&self) -> Result<()> { self.dispose() } }

pub struct AudioEngine { state: Arc<AudioEngineState> }

impl AudioEngine {
    pub const ContentVersion: i32 = 39;

    pub fn new(game: &GameContext<'_>, settingsFile: &str) -> Result<Self> {
        Self::create(game, settingsFile, None)
    }
    pub fn from_settings_file_and_look_ahead_time_and_renderer_id(game: &GameContext<'_>, settingsFile: &str, lookAheadTime: TimeSpan, rendererId: &str) -> Result<Self> {
        Self::create(game, settingsFile, Some((lookAheadTime.Ticks(), rendererId)))
    }
    fn create(game: &GameContext<'_>, file: &str, renderer: Option<(i64, &str)>) -> Result<Self> {
        if file.is_empty() { return Err(CnaError::InvalidInput("settingsFile must not be empty")); }
        validate_xact_header(file, *b"XGSF", "XACT settings")?;
        let runtime = Arc::clone(game.audio_runtime());
        let active = runtime.active()?;
        let handle = active.native.create_audio_engine(active.handle, file, renderer)?;
        let state = Arc::new(AudioEngineState {
            watch: watch_disposal(&active.native, 0, handle),
            runtime: Arc::clone(&runtime), native: active.native, generation: active.generation,
            handle: Mutex::new(handle), children: Mutex::new(Vec::new()),
            categories: Mutex::new(HashMap::new()), disposing: EventHandlers::new(),
            cna_raised: AtomicBool::new(false),
        });
        runtime.register(&state);
        Ok(Self { state })
    }
    pub fn RendererDetails(&self) -> Result<Vec<RendererDetail>> {
        self.state.native.audio_renderers(self.state.require_handle()?).map(|values| values.into_iter().map(|(friendly, id)| RendererDetail::from_parts(friendly, id)).collect())
    }
    pub fn IsDisposed(&self) -> bool { *self.state.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner) == 0 }
    pub fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 { self.state.disposing.add(handler) }
    pub fn RemoveDisposingHandler(&self, registration: u64) -> bool { self.state.disposing.remove(registration) }
    pub fn GetCategory(&self, name: &str) -> Result<AudioCategory> {
        if name.is_empty() { return Err(CnaError::InvalidInput("category name must not be empty")); }
        if let Some(state) = self.state.categories.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(name).and_then(Weak::upgrade) {
            return Ok(AudioCategory { state });
        }
        let handle = self.state.native.audio_category(self.state.require_handle()?, name)?;
        let canonical_name = match self.state.native.audio_category_name(handle) {
            Ok(value) => value,
            Err(error) => { let _ = self.state.native.destroy_audio_category(handle); return Err(error); }
        };
        let state = Arc::new(AudioCategoryState { engine: Arc::clone(&self.state), handle: Mutex::new(handle), name: canonical_name.clone() });
        self.state.register_child(&state);
        self.state.runtime.register(&state);
        self.state.categories.lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(canonical_name, Arc::downgrade(&state));
        Ok(AudioCategory { state })
    }
    pub fn GetGlobalVariable(&self, name: &str) -> Result<f32> {
        if name.is_empty() {
            return Err(CnaError::InvalidInput("global variable name must not be empty"));
        }
        self.state.native.audio_engine_global(self.state.require_handle()?, name)
    }
    pub fn SetGlobalVariable(&self, name: &str, value: f32) -> Result<()> {
        if name.is_empty() {
            return Err(CnaError::InvalidInput("global variable name must not be empty"));
        }
        self.state.native.set_audio_engine_global(self.state.require_handle()?, name, value)
    }
    pub fn Update(&self) -> Result<()> { self.state.native.update_audio_engine(self.state.require_handle()?) }
    pub fn Finalize(&self) {}
    pub fn Dispose(&mut self) -> Result<()> {
        let first_disposal = !self.IsDisposed();
        self.state.dispose()?;
        if first_disposal && self.state.disposing.emit(self, EventArgs) {
            Err(CnaError::Callback("AudioEngine disposing handler panicked".to_owned()))
        } else {
            Ok(())
        }
    }
    pub fn DisposeWithDisposing(&mut self, disposing: bool) -> Result<()> { if disposing { self.Dispose() } else { self.state.dispose() } }
}

impl Drop for AudioEngine { fn drop(&mut self) { let _ = self.state.dispose(); } }

struct AudioCategoryState { engine: Arc<AudioEngineState>, handle: Mutex<sys::CNA_Handle>, name: String }
impl AudioCategoryState {
    fn require_handle(&self) -> Result<sys::CNA_Handle> { self.engine.require_handle()?; let value = *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner); (value != 0).then_some(value).ok_or(CnaError::InvalidInput("AudioCategory is invalid")) }
    fn dispose(&self) -> Result<()> { let mut handle = self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner); if *handle != 0 { self.engine.native.destroy_audio_category(*handle)?; *handle = 0; } Ok(()) }
}
// XNA's `AudioCategory` is a value type with no `Disposing` event, so the
// engine's teardown of one raises nothing.
impl XactChild for AudioCategoryState { fn dispose_owned_by_engine(&self) -> Result<()> { self.dispose() } }
impl AudioResourceCleanup for AudioCategoryState { fn dispose_for_game_shutdown(&self) -> Result<()> { self.dispose() } }

#[derive(Clone)]
pub struct AudioCategory { state: Arc<AudioCategoryState> }
impl AudioCategory {
    pub fn Name(&self) -> String { self.state.name.clone() }
    pub fn SetVolume(&mut self, volume: f32) -> Result<()> {
        if volume < 0.0 {
            return Err(CnaError::InvalidInput("category volume must not be negative"));
        }
        self.state.engine.native.audio_category_action(self.state.require_handle()?, 2, volume, 0)
    }
    pub fn Pause(&mut self) -> Result<()> { self.state.engine.native.audio_category_action(self.state.require_handle()?, 0, 0.0, 0) }
    pub fn Resume(&mut self) -> Result<()> { self.state.engine.native.audio_category_action(self.state.require_handle()?, 1, 0.0, 0) }
    pub fn Stop(&mut self, options: AudioStopOptions) -> Result<()> { self.state.engine.native.audio_category_action(self.state.require_handle()?, 3, 0.0, options as i32 as u32) }
    pub fn ToString(&self) -> String { self.Name() }
    pub fn Equals(&self, other: Self) -> bool {
        Arc::ptr_eq(&self.state.engine, &other.state.engine) && self.state.name == other.state.name
    }
    pub fn EqualsWithObj(&self, obj: &dyn Any) -> Result<bool> {
        let Some(other) = obj.downcast_ref::<Self>() else { return Ok(false); };
        self.state.engine.native.audio_categories_equal(
            self.state.require_handle()?,
            other.state.require_handle()?,
        )
    }
    pub fn GetHashCode(&self) -> Result<i32> { self.state.engine.native.audio_category_hash(self.state.require_handle()?) }
}
impl PartialEq for AudioCategory {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state.engine, &other.state.engine) && self.state.name == other.state.name
    }
}

struct WaveBankState {
    engine: Arc<AudioEngineState>,
    handle: Mutex<sys::CNA_Handle>,
    disposing: EventHandlers<EventArgs>,
    /// This state, so a teardown that has only the state can hand the public
    /// `WaveBank` to a handler as XNA's sender. Set once, immediately after
    /// construction.
    me: std::sync::OnceLock<Weak<WaveBankState>>,
    watch: Mutex<Option<NativeDisposing>>,
    cna_raised: AtomicBool,
}
impl WaveBankState {
    fn require_handle(&self) -> Result<sys::CNA_Handle> { self.engine.require_handle()?; let value = *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner); (value != 0).then_some(value).ok_or(CnaError::InvalidInput("WaveBank is disposed")) }
    fn dispose(&self) -> Result<()> {
        let mut handle = self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if *handle == 0 { return Ok(()); }
        self.engine.native.destroy_wave_bank(*handle)?;
        *handle = 0;
        if stop_watching(&self.engine.native, &self.watch) { self.cna_raised.store(true, Ordering::Release); }
        Ok(())
    }
}
impl WaveBankState {
    /// Disposes and raises `Disposing`, which is what every path but the
    /// finalizer does.
    fn dispose_raising(&self) -> Result<()> {
        let first_disposal =
            *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner) != 0;
        self.dispose()?;
        if !first_disposal {
            return Ok(());
        }
        let sender = self.me.get().and_then(Weak::upgrade).map(|state| WaveBank { state });
        let panicked = sender.as_ref().map_or_else(
            || self.disposing.emit(self, EventArgs),
            |bank| self.disposing.emit(bank, EventArgs),
        );
        if panicked {
            Err(CnaError::Callback("WaveBank disposing handler panicked".to_owned()))
        } else {
            Ok(())
        }
    }
}
impl XactChild for WaveBankState { fn dispose_owned_by_engine(&self) -> Result<()> { self.dispose_raising() } }
impl AudioResourceCleanup for WaveBankState { fn dispose_for_game_shutdown(&self) -> Result<()> { self.dispose() } }

pub struct WaveBank { state: Arc<WaveBankState> }
impl WaveBank {
    pub fn new(audioEngine: &AudioEngine, nonStreamingWaveBankFilename: &str) -> Result<Self> { Self::create(audioEngine, nonStreamingWaveBankFilename, None) }
    pub fn from_audio_engine_and_streaming_wave_bank_filename_and_offset_and_packetsize(audioEngine: &AudioEngine, streamingWaveBankFilename: &str, offset: i32, packetsize: i16) -> Result<Self> {
        Self::create(audioEngine, streamingWaveBankFilename, Some((offset, packetsize)))
    }
    fn create(engine: &AudioEngine, file: &str, streaming: Option<(i32, i16)>) -> Result<Self> {
        if file.is_empty() { return Err(CnaError::InvalidInput("wave-bank filename must not be empty")); }
        validate_xact_header(file, *b"WBND", "wave-bank")?;
        let handle = engine.state.native.create_wave_bank(engine.state.require_handle()?, file, streaming)?;
        let state = Arc::new(WaveBankState {
            engine: Arc::clone(&engine.state), handle: Mutex::new(handle),
            disposing: EventHandlers::new(), me: std::sync::OnceLock::new(),
            watch: watch_disposal(&engine.state.native, 1, handle),
            cna_raised: AtomicBool::new(false),
        });
        let _ = state.me.set(Arc::downgrade(&state));
        engine.state.register_child(&state); engine.state.runtime.register(&state); Ok(Self { state })
    }
    pub fn IsInUse(&self) -> Result<bool> { self.state.engine.native.wave_bank_flag(self.state.require_handle()?, false) }
    pub fn IsPrepared(&self) -> Result<bool> { self.state.engine.native.wave_bank_flag(self.state.require_handle()?, true) }
    pub fn IsDisposed(&self) -> bool { *self.state.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner) == 0 }
    pub fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 { self.state.disposing.add(handler) }
    pub fn RemoveDisposingHandler(&self, registration: u64) -> bool { self.state.disposing.remove(registration) }
    pub fn Finalize(&self) {}
    pub fn Dispose(&mut self) -> Result<()> { self.DisposeWithDisposing(true) }
    pub fn DisposeWithDisposing(&mut self, disposing: bool) -> Result<()> {
        // `disposing == false` is XNA's finalizer path, and XNA's
        // `Dispose(bool)` raises `Disposing` only when it is true.
        if disposing {
            self.state.dispose_raising()
        } else {
            self.state.dispose()
        }
    }
}
impl Drop for WaveBank { fn drop(&mut self) { let _ = self.state.dispose(); } }

struct SoundBankState {
    engine: Arc<AudioEngineState>,
    handle: Mutex<sys::CNA_Handle>,
    cues: Mutex<Vec<Weak<CueState>>>,
    disposing: EventHandlers<EventArgs>,
    me: std::sync::OnceLock<Weak<SoundBankState>>,
    watch: Mutex<Option<NativeDisposing>>,
    cna_raised: AtomicBool,
}
impl SoundBankState {
    fn require_handle(&self) -> Result<sys::CNA_Handle> { self.engine.require_handle()?; let value = *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner); (value != 0).then_some(value).ok_or(CnaError::InvalidInput("SoundBank is disposed")) }
    fn dispose(&self) -> Result<()> { if *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner) == 0 { return Ok(()); } let cues = self.cues.lock().unwrap_or_else(std::sync::PoisonError::into_inner).iter().filter_map(Weak::upgrade).collect::<Vec<_>>(); for cue in cues.into_iter().rev() { cue.dispose_raising()?; } let mut handle = self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner); if *handle != 0 { self.engine.native.destroy_sound_bank(*handle)?; *handle = 0; } if stop_watching(&self.engine.native, &self.watch) { self.cna_raised.store(true, Ordering::Release); } Ok(()) }
}
impl SoundBankState {
    fn dispose_raising(&self) -> Result<()> {
        let first_disposal =
            *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner) != 0;
        self.dispose()?;
        if !first_disposal {
            return Ok(());
        }
        let sender = self.me.get().and_then(Weak::upgrade).map(|state| SoundBank { state });
        let panicked = sender.as_ref().map_or_else(
            || self.disposing.emit(self, EventArgs),
            |bank| self.disposing.emit(bank, EventArgs),
        );
        if panicked {
            Err(CnaError::Callback("SoundBank disposing handler panicked".to_owned()))
        } else {
            Ok(())
        }
    }
}
impl XactChild for SoundBankState { fn dispose_owned_by_engine(&self) -> Result<()> { self.dispose_raising() } }
impl AudioResourceCleanup for SoundBankState { fn dispose_for_game_shutdown(&self) -> Result<()> { self.dispose() } }

pub struct SoundBank { state: Arc<SoundBankState> }
impl SoundBank {
    pub fn new(audioEngine: &AudioEngine, filename: &str) -> Result<Self> {
        if filename.is_empty() { return Err(CnaError::InvalidInput("sound-bank filename must not be empty")); }
        validate_xact_header(filename, *b"SDBK", "sound-bank")?;
        let handle = audioEngine.state.native.create_sound_bank(audioEngine.state.require_handle()?, filename)?;
        let state = Arc::new(SoundBankState {
            engine: Arc::clone(&audioEngine.state), handle: Mutex::new(handle),
            cues: Mutex::new(Vec::new()), disposing: EventHandlers::new(),
            me: std::sync::OnceLock::new(),
            watch: watch_disposal(&audioEngine.state.native, 2, handle),
            cna_raised: AtomicBool::new(false),
        });
        let _ = state.me.set(Arc::downgrade(&state));
        audioEngine.state.register_child(&state); audioEngine.state.runtime.register(&state); Ok(Self { state })
    }
    pub fn IsInUse(&self) -> Result<bool> { self.state.engine.native.sound_bank_is_in_use(self.state.require_handle()?) }
    pub fn IsDisposed(&self) -> bool { *self.state.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner) == 0 }
    pub fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 { self.state.disposing.add(handler) }
    pub fn RemoveDisposingHandler(&self, registration: u64) -> bool { self.state.disposing.remove(registration) }
    pub fn GetCue(&self, name: &str) -> Result<Cue> {
        if name.is_empty() { return Err(CnaError::InvalidInput("cue name must not be empty")); }
        let handle = self.state.engine.native.sound_bank_get_cue(self.state.require_handle()?, name)?;
        let cue_name = match self.state.engine.native.cue_name(handle) { Ok(value) => value, Err(error) => { let _ = self.state.engine.native.destroy_cue(handle); return Err(error); } };
        let state = Arc::new(CueState {
            engine: Arc::clone(&self.state.engine),
            handle: Mutex::new(handle),
            name: cue_name,
            played: AtomicBool::new(false),
            applied_3d: AtomicBool::new(false),
            disposing: EventHandlers::new(),
            me: std::sync::OnceLock::new(),
            watch: watch_disposal(&self.state.engine.native, 3, handle),
            cna_raised: AtomicBool::new(false),
        });
        let _ = state.me.set(Arc::downgrade(&state));
        self.state.cues.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(Arc::downgrade(&state));
        self.state.engine.register_child(&state);
        self.state.engine.runtime.register(&state); Ok(Cue { state })
    }
    pub fn PlayCue(&self, name: &str) -> Result<()> {
        if name.is_empty() { return Err(CnaError::InvalidInput("cue name must not be empty")); }
        self.state.engine.native.sound_bank_play(self.state.require_handle()?, name, None)
    }
    pub fn PlayCueWithNameAndListenerAndEmitter(&self, name: &str, listener: &AudioListener, emitter: &AudioEmitter) -> Result<()> {
        if name.is_empty() { return Err(CnaError::InvalidInput("cue name must not be empty")); }
        let listener = native_listener(listener);
        let emitter = native_emitter(emitter);
        self.state.engine.native.sound_bank_play(self.state.require_handle()?, name, Some((&listener, &emitter)))
    }
    pub fn Finalize(&self) {}
    pub fn Dispose(&mut self) -> Result<()> { self.DisposeWithDisposing(true) }
    pub fn DisposeWithDisposing(&mut self, disposing: bool) -> Result<()> {
        if disposing {
            self.state.dispose_raising()
        } else {
            self.state.dispose()
        }
    }
}
impl Drop for SoundBank { fn drop(&mut self) { let _ = self.state.dispose(); } }

struct CueState {
    engine: Arc<AudioEngineState>,
    handle: Mutex<sys::CNA_Handle>,
    name: String,
    played: AtomicBool,
    applied_3d: AtomicBool,
    disposing: EventHandlers<EventArgs>,
    me: std::sync::OnceLock<Weak<CueState>>,
    watch: Mutex<Option<NativeDisposing>>,
    cna_raised: AtomicBool,
}
impl CueState {
    fn require_handle(&self) -> Result<sys::CNA_Handle> { self.engine.require_handle()?; let value = *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner); (value != 0).then_some(value).ok_or(CnaError::InvalidInput("Cue is disposed")) }
    fn dispose(&self) -> Result<()> {
        let mut handle = self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if *handle == 0 { return Ok(()); }
        self.engine.native.destroy_cue(*handle)?;
        *handle = 0;
        if stop_watching(&self.engine.native, &self.watch) { self.cna_raised.store(true, Ordering::Release); }
        Ok(())
    }
    fn info(&self) -> Result<sys::CNA_CueInfo> { self.engine.native.cue_info(self.require_handle()?) }

    fn dispose_raising(&self) -> Result<()> {
        let first_disposal =
            *self.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner) != 0;
        self.dispose()?;
        if !first_disposal {
            return Ok(());
        }
        let sender = self.me.get().and_then(Weak::upgrade).map(|state| Cue { state });
        let panicked = sender.as_ref().map_or_else(
            || self.disposing.emit(self, EventArgs),
            |cue| self.disposing.emit(cue, EventArgs),
        );
        if panicked {
            Err(CnaError::Callback("Cue disposing handler panicked".to_owned()))
        } else {
            Ok(())
        }
    }
}
impl XactChild for CueState { fn dispose_owned_by_engine(&self) -> Result<()> { self.dispose_raising() } }
impl AudioResourceCleanup for CueState { fn dispose_for_game_shutdown(&self) -> Result<()> { self.dispose() } }

pub struct Cue { state: Arc<CueState> }
impl Cue {
    pub fn Name(&self) -> Result<String> { Ok(self.state.name.clone()) }
    pub fn IsCreated(&self) -> Result<bool> { Ok(self.state.info()?.is_created != 0) }
    pub fn IsPreparing(&self) -> Result<bool> { Ok(self.state.info()?.is_preparing != 0) }
    pub fn IsPrepared(&self) -> Result<bool> { Ok(self.state.info()?.is_prepared != 0) }
    pub fn IsPlaying(&self) -> Result<bool> { Ok(self.state.info()?.is_playing != 0) }
    pub fn IsStopping(&self) -> Result<bool> { Ok(self.state.info()?.is_stopping != 0) }
    pub fn IsStopped(&self) -> Result<bool> { Ok(self.state.info()?.is_stopped != 0) }
    pub fn IsPaused(&self) -> Result<bool> { Ok(self.state.info()?.is_paused != 0) }
    pub fn IsDisposed(&self) -> bool { *self.state.handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner) == 0 }
    pub fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 { self.state.disposing.add(handler) }
    pub fn RemoveDisposingHandler(&self, registration: u64) -> bool { self.state.disposing.remove(registration) }
    pub fn Play(&self) -> Result<()> {
        self.state.engine.native.cue_transport(self.state.require_handle()?, 0, 0)?;
        self.state.played.store(true, Ordering::Release);
        Ok(())
    }
    pub fn Pause(&self) -> Result<()> { self.state.engine.native.cue_transport(self.state.require_handle()?, 1, 0) }
    pub fn Resume(&self) -> Result<()> { self.state.engine.native.cue_transport(self.state.require_handle()?, 2, 0) }
    pub fn Stop(&self, options: AudioStopOptions) -> Result<()> { self.state.engine.native.cue_transport(self.state.require_handle()?, 3, options as i32 as u32) }
    pub fn GetVariable(&self, name: &str) -> Result<f32> {
        if name.is_empty() { return Err(CnaError::InvalidInput("cue variable name must not be empty")); }
        self.state.engine.native.cue_variable(self.state.require_handle()?, name)
    }
    pub fn SetVariable(&self, name: &str, value: f32) -> Result<()> {
        if name.is_empty() { return Err(CnaError::InvalidInput("cue variable name must not be empty")); }
        self.state.engine.native.set_cue_variable(self.state.require_handle()?, name, value)
    }
    pub fn Apply3D(&self, listener: &AudioListener, emitter: &AudioEmitter) -> Result<()> {
        if self.state.played.load(Ordering::Acquire)
            && !self.state.applied_3d.load(Ordering::Acquire)
        {
            return Err(CnaError::InvalidInput("Cue.Apply3D must precede the first Play"));
        }
        let listener = native_listener(listener);
        let emitter = native_emitter(emitter);
        self.state.engine.native.cue_apply_3d(self.state.require_handle()?, &listener, &emitter)?;
        self.state.applied_3d.store(true, Ordering::Release);
        Ok(())
    }
    pub fn Finalize(&self) {}
    pub fn Dispose(&mut self) -> Result<()> { self.state.dispose_raising() }
}
impl Drop for Cue { fn drop(&mut self) { let _ = self.state.dispose(); } }

/// Whether CNA considers the engine disposed. The answer that differs from
/// `IsDisposed` when an engine disposes and takes its banks and cues with it.
impl NativeDisposalState for AudioEngine {
    fn NativeIsDisposed(&self) -> Result<bool> {
        self.state
            .native
            .audio_engine_is_disposed(self.state.require_handle()?)
    }
}

impl AudioEngineExt for AudioEngine {
    fn RendererText(&self, index: u64) -> Result<String> {
        self.state
            .native
            .audio_engine_renderer_text(self.state.require_handle()?, index)
    }

    fn RendererHashCode(&self, index: u64) -> Result<i32> {
        self.state
            .native
            .audio_engine_renderer_hash(self.state.require_handle()?, index)
    }

    fn RenderersEqual(&self, left: u64, right: u64) -> Result<bool> {
        self.state.native.audio_engine_renderers_equal(
            self.state.require_handle()?,
            left,
            right,
        )
    }
}

/// Whether CNA considers the wave bank disposed.
impl NativeDisposalState for WaveBank {
    fn NativeIsDisposed(&self) -> Result<bool> {
        self.state
            .engine
            .native
            .wave_bank_is_disposed(self.state.require_handle()?)
    }
}

/// Whether CNA considers the sound bank disposed.
impl NativeDisposalState for SoundBank {
    fn NativeIsDisposed(&self) -> Result<bool> {
        self.state
            .engine
            .native
            .sound_bank_is_disposed(self.state.require_handle()?)
    }
}

/// Whether CNA raised its own disposal notification for each XACT object.
///
/// The strict projection's `Disposing` is the contract a game programs
/// against; this says whether the runtime underneath agreed. `RUST-EXT-017`
/// bound the four `*_subscribe_disposing_ext` routes for exactly this reason:
/// the Rust event was not firing on the path where an engine disposes its
/// banks and cues, and binding the native event alone would have made the two
/// disagree rather than fixed either. Now both fire on the same paths, and a
/// caller -- a test above all -- can check that they did.
///
/// `false` before disposal, and `false` afterwards on an artifact that refused
/// the subscription. It is never a substitute for the XNA event.
pub(crate) fn engine_cna_raised_disposing(engine: &AudioEngine) -> bool {
    engine.state.cna_raised.load(Ordering::Acquire)
}

pub(crate) fn wave_bank_cna_raised_disposing(bank: &WaveBank) -> bool {
    bank.state.cna_raised.load(Ordering::Acquire)
}

pub(crate) fn sound_bank_cna_raised_disposing(bank: &SoundBank) -> bool {
    bank.state.cna_raised.load(Ordering::Acquire)
}

pub(crate) fn cue_cna_raised_disposing(cue: &Cue) -> bool {
    cue.state.cna_raised.load(Ordering::Acquire)
}
