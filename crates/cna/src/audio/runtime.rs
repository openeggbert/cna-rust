use core::any::Any;
use core::ffi::c_void;
use std::collections::VecDeque;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread::{self, ThreadId};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::native::Native;
use super::Microphone;

pub(super) trait AudioResourceCleanup: Send + Sync {
    fn dispose_for_game_shutdown(&self) -> Result<()>;
}

pub(super) trait AudioEventTarget: Any + Send + Sync {
    fn dispatch_audio_event(&self) -> bool;
    fn accepts_audio_events(&self) -> bool;
}

#[derive(Clone)]
pub(crate) struct ActiveAudioGame {
    pub(crate) native: Arc<Native>,
    pub(crate) handle: sys::CNA_Handle,
    pub(crate) owner_thread: ThreadId,
    pub(crate) generation: u64,
}

pub(crate) struct AudioRuntime {
    binding: Mutex<Option<ActiveAudioGame>>,
    generation: AtomicU64,
    resources: Mutex<Vec<Arc<dyn AudioResourceCleanup>>>,
    pending: Mutex<VecDeque<Weak<dyn AudioEventTarget>>>,
    microphones: Mutex<Option<(u64, Vec<Arc<Microphone>>)>>,
}

impl AudioRuntime {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            binding: Mutex::new(None),
            generation: AtomicU64::new(0),
            resources: Mutex::new(Vec::new()),
            pending: Mutex::new(VecDeque::new()),
            microphones: Mutex::new(None),
        })
    }

    pub(crate) fn attach(&self, native: &Arc<Native>, handle: sys::CNA_Handle) -> Result<()> {
        let mut binding = self.binding.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if binding.is_some() {
            return Err(CnaError::InvalidInput("audio runtime is already attached"));
        }
        let generation = self.generation.fetch_add(1, Ordering::AcqRel).wrapping_add(1);
        *binding = Some(ActiveAudioGame {
            native: Arc::clone(native),
            handle,
            owner_thread: thread::current().id(),
            generation,
        });
        Ok(())
    }

    pub(crate) fn active(&self) -> Result<ActiveAudioGame> {
        self.binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .ok_or(CnaError::InvalidInput("game audio runtime is not active"))
    }

    pub(crate) fn ensure_generation(&self, generation: u64) -> Result<ActiveAudioGame> {
        let active = self.active()?;
        if active.generation != generation {
            Err(CnaError::InvalidInput("audio object belongs to an inactive Game generation"))
        } else {
            Ok(active)
        }
    }

    pub(super) fn register<T>(&self, state: &Arc<T>)
    where
        T: AudioResourceCleanup + 'static,
    {
        let erased: Arc<dyn AudioResourceCleanup> = state.clone();
        self.resources
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(erased);
    }

    pub(crate) fn cleanup(&self) -> Result<()> {
        let resources = self
            .resources
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let mut first_error = None;
        for resource in resources.into_iter().rev() {
            if let Err(error) = resource.dispose_for_game_shutdown() {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
        if first_error.is_none() {
            self.resources
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clear();
        }
        first_error.map_or(Ok(()), Err)
    }

    pub(crate) fn detach(&self) {
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.microphones
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        self.binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }

    pub(super) fn enqueue(&self, target: Weak<dyn AudioEventTarget>) {
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push_back(target);
    }

    pub(super) fn microphones(&self, generation: u64) -> Option<Vec<Arc<Microphone>>> {
        let cache = self.microphones.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let (cached_generation, values) = cache.as_ref()?;
        if *cached_generation != generation { return None; }
        Some(values.clone())
    }

    pub(super) fn set_microphones(&self, generation: u64, values: &[Arc<Microphone>]) {
        *self.microphones.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some((
            generation,
            values.to_vec(),
        ));
    }

    pub(crate) fn dispatch_pending(&self) -> Result<()> {
        let active = self.active()?;
        if thread::current().id() != active.owner_thread {
            return Err(CnaError::InvalidInput(
                "audio callbacks must be dispatched on the Game owner thread",
            ));
        }
        let pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain(..)
            .collect::<Vec<_>>();
        let mut panicked = false;
        for target in pending {
            if let Some(target) = target.upgrade() {
                if target.accepts_audio_events() {
                    panicked |= target.dispatch_audio_event();
                }
            }
        }
        if panicked {
            Err(CnaError::Callback(
                "Rust audio event-handler panic was contained before the native boundary".to_owned(),
            ))
        } else {
            Ok(())
        }
    }
}

pub(super) struct AudioCallbackToken {
    runtime: Weak<AudioRuntime>,
    target: Weak<dyn AudioEventTarget>,
    active: AtomicBool,
}

impl AudioCallbackToken {
    pub(super) fn new(runtime: &Arc<AudioRuntime>, target: &Arc<dyn AudioEventTarget>) -> Box<Self> {
        Box::new(Self {
            runtime: Arc::downgrade(runtime),
            target: Arc::downgrade(target),
            active: AtomicBool::new(true),
        })
    }

    pub(super) fn context(&mut self) -> *mut c_void {
        // `ptr::from_mut` is stable only from 1.76; `addr_of_mut!` has been
        // stable since 1.51 and produces the same pointer without an
        // intermediate reference, so the declared 1.74 MSRV holds.
        core::ptr::addr_of_mut!(*self).cast()
    }

    pub(super) fn deactivate(&self) {
        self.active.store(false, Ordering::Release);
    }

    pub(super) fn reactivate(&self) {
        self.active.store(true, Ordering::Release);
    }
}

pub(super) struct NativeAudioRegistration {
    pub(super) handle: sys::CNA_Handle,
    pub(super) token: Box<AudioCallbackToken>,
}

pub(super) unsafe extern "C" fn audio_event_trampoline(context: *mut c_void) {
    if context.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: the owned registration keeps this boxed token stable until CNA accepts
        // unsubscribe; the callback is never retained after that call succeeds.
        let token = unsafe { &*context.cast::<AudioCallbackToken>() };
        if !token.active.load(Ordering::Acquire) {
            return;
        }
        if let Some(runtime) = token.runtime.upgrade() {
            runtime.enqueue(token.target.clone());
        }
    }));
}
