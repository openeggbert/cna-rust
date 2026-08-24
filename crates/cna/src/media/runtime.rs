use core::ffi::c_void;
use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::thread::{self, ThreadId};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::{EventArgs, EventHandler};
use crate::graphics::resource::EventHandlers;
use crate::native::Native;

pub(crate) trait MediaInvalidatable: Send + Sync {
    fn invalidate_for_game_shutdown(&self);
}

#[derive(Clone)]
struct Binding {
    native: Arc<Native>,
    game: sys::CNA_Handle,
    generation: u64,
    owner: ThreadId,
}

#[derive(Clone, Copy)]
enum PendingEvent {
    ActiveSongChanged(u64),
    MediaStateChanged(u64),
}

struct Registrations {
    native: Arc<Native>,
    active_song: sys::CNA_MediaPlayerEventRegistrationHandle,
    media_state: sys::CNA_MediaPlayerEventRegistrationHandle,
}

struct DeferredRelease {
    native: Arc<Native>,
    destroy: unsafe extern "C" fn(sys::CNA_Handle) -> sys::CNA_Result,
    handle: sys::CNA_Handle,
}

pub(crate) struct MediaRuntime {
    next_generation: AtomicU64,
    dispatching: AtomicBool,
    binding: Mutex<Option<Binding>>,
    resources: Mutex<Vec<Weak<dyn MediaInvalidatable>>>,
    deferred_releases: Mutex<Vec<DeferredRelease>>,
    pending: Mutex<Vec<(u64, PendingEvent)>>,
    active_song_changed: EventHandlers<EventArgs>,
    media_state_changed: EventHandlers<EventArgs>,
    registrations: OnceLock<Registrations>,
}

static PROCESS_RUNTIME: OnceLock<Arc<MediaRuntime>> = OnceLock::new();

impl MediaRuntime {
    pub(crate) fn process() -> Arc<Self> {
        Arc::clone(PROCESS_RUNTIME.get_or_init(|| {
            Arc::new(Self {
                next_generation: AtomicU64::new(1),
                dispatching: AtomicBool::new(false),
                binding: Mutex::new(None),
                resources: Mutex::new(Vec::new()),
                deferred_releases: Mutex::new(Vec::new()),
                pending: Mutex::new(Vec::new()),
                active_song_changed: EventHandlers::new(),
                media_state_changed: EventHandlers::new(),
                registrations: OnceLock::new(),
            })
        }))
    }

    pub(crate) fn attach(
        self: &Arc<Self>,
        native: &Arc<Native>,
        game: sys::CNA_Handle,
    ) -> Result<u64> {
        let mut binding = self
            .binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if binding.is_some() {
            return Err(CnaError::InvalidInput(
                "MediaPlayer cannot bind two live Games in one process",
            ));
        }
        self.ensure_registrations(native)?;
        let generation = self.next_generation.fetch_add(1, Ordering::AcqRel);
        *binding = Some(Binding {
            native: Arc::clone(native),
            game,
            generation,
            owner: thread::current().id(),
        });
        Ok(generation)
    }

    fn ensure_registrations(self: &Arc<Self>, native: &Arc<Native>) -> Result<()> {
        if self.registrations.get().is_some() {
            return Ok(());
        }
        let context = Arc::as_ptr(self).cast_mut().cast::<c_void>();
        let active_fn: sys::cna_media_player_subscribe_active_song_changed_ext_fn = unsafe {
            super::native_function(native.media.media_player_subscribe_active_song_changed_ext)
        };
        let state_fn: sys::cna_media_player_subscribe_media_state_changed_ext_fn = unsafe {
            super::native_function(native.media.media_player_subscribe_media_state_changed_ext)
        };
        let unsubscribe: sys::cna_media_player_unsubscribe_ext_fn = unsafe {
            super::native_function(native.media.media_player_unsubscribe_ext)
        };
        let mut active_song = 0;
        // SAFETY: the process runtime address is stable in a process-lifetime Arc and the
        // callback catches panics before returning through C.
        native.check(unsafe {
            active_fn(Some(active_song_trampoline), context, &mut active_song)
        })?;
        let mut media_state = 0;
        // SAFETY: same registration lifetime and callback contract as above.
        if let Err(error) = native.check(unsafe {
            state_fn(Some(media_state_trampoline), context, &mut media_state)
        }) {
            // SAFETY: this is the owned registration returned immediately above.
            let _ = native.check(unsafe { unsubscribe(active_song) });
            return Err(error);
        }
        let _ = self.registrations.set(Registrations {
            native: Arc::clone(native),
            active_song,
            media_state,
        });
        Ok(())
    }

    pub(crate) fn binding(&self) -> Result<(Arc<Native>, sys::CNA_Handle, u64)> {
        let binding = self
            .binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .ok_or(CnaError::InvalidInput("Media requires an active Game callback"))?;
        if binding.owner != thread::current().id() {
            return Err(CnaError::InvalidInput(
                "Media operations must run on the Game owner thread",
            ));
        }
        Ok((binding.native, binding.game, binding.generation))
    }

    pub(crate) fn event_binding(&self) -> Result<(Arc<Native>, sys::CNA_Handle, u64)> {
        if !self.dispatching.load(Ordering::Acquire) {
            return Err(CnaError::InvalidInput(
                "reentrant Media control is available only inside a MediaPlayer handler",
            ));
        }
        self.binding()
    }

    pub(crate) fn is_generation_active(&self, generation: u64) -> bool {
        self.binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|binding| binding.generation == generation)
    }

    pub(crate) fn register_resource(&self, resource: &Arc<dyn MediaInvalidatable>) {
        let mut resources = self
            .resources
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        resources.retain(|item| item.strong_count() != 0);
        resources.push(Arc::downgrade(resource));
    }

    pub(crate) fn release_or_defer(
        &self,
        native: &Arc<Native>,
        destroy: unsafe extern "C" fn(sys::CNA_Handle) -> sys::CNA_Result,
        handle: sys::CNA_Handle,
    ) {
        if handle == 0 {
            return;
        }
        let on_owner = self
            .binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|binding| binding.owner == thread::current().id());
        if on_owner {
            // SAFETY: each ResourceCore supplies the matching destroy route and its owned handle.
            let _ = native.check(unsafe { destroy(handle) });
        } else {
            self.deferred_releases
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(DeferredRelease {
                    native: Arc::clone(native),
                    destroy,
                    handle,
                });
        }
    }

    fn drain_deferred_releases(&self) {
        let releases = self
            .deferred_releases
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain(..)
            .collect::<Vec<_>>();
        for release in releases {
            // SAFETY: deferred records contain the exact destroy route and owned handle.
            let _ = release
                .native
                .check(unsafe { (release.destroy)(release.handle) });
        }
    }

    pub(crate) fn detach(&self) {
        let binding = self
            .binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let Some(binding) = binding else {
            return;
        };
        let program_exit: sys::cna_media_player_program_exit_ext_fn = unsafe {
            super::native_function(binding.native.media.media_player_program_exit_ext)
        };
        // SAFETY: the game is still live while GameState performs media cleanup.
        let _ = binding.native.check(unsafe { program_exit(binding.game) });
        let resources = self
            .resources
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain(..)
            .filter_map(|resource| resource.upgrade())
            .collect::<Vec<_>>();
        for resource in resources.iter().rev() {
            resource.invalidate_for_game_shutdown();
        }
        self.drain_deferred_releases();
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    fn enqueue_active_song_changed(&self) {
        self.enqueue(PendingEvent::ActiveSongChanged(
            self.active_song_changed.registration_cutoff(),
        ));
    }

    fn enqueue_media_state_changed(&self) {
        self.enqueue(PendingEvent::MediaStateChanged(
            self.media_state_changed.registration_cutoff(),
        ));
    }

    fn enqueue(&self, event: PendingEvent) {
        if let Some(binding) = self
            .binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            self.pending
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push((binding.generation, event));
        }
    }

    pub(crate) fn dispatch_pending(&self) -> Result<()> {
        self.drain_deferred_releases();
        let generation = self
            .binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .map(|binding| binding.generation);
        let Some(generation) = generation else {
            return Ok(());
        };
        if self
            .dispatching
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(CnaError::InvalidInput(
                "MediaPlayer event dispatch cannot be pumped reentrantly",
            ));
        }
        struct DispatchGuard<'a>(&'a AtomicBool);
        impl Drop for DispatchGuard<'_> {
            fn drop(&mut self) {
                self.0.store(false, Ordering::Release);
            }
        }
        let _dispatch_guard = DispatchGuard(&self.dispatching);
        let pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain(..)
            .collect::<Vec<_>>();
        let mut panicked = false;
        for (event_generation, event) in pending {
            if event_generation != generation {
                continue;
            }
            match event {
                PendingEvent::ActiveSongChanged(cutoff) => {
                    panicked |= self.active_song_changed.emit_through(
                        &() as &dyn Any,
                        EventArgs,
                        cutoff,
                    );
                }
                PendingEvent::MediaStateChanged(cutoff) => {
                    panicked |= self.media_state_changed.emit_through(
                        &() as &dyn Any,
                        EventArgs,
                        cutoff,
                    );
                }
            }
        }
        if panicked {
            Err(CnaError::Callback(
                "Rust MediaPlayer event-handler panic was contained before the native boundary"
                    .to_owned(),
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn add_active_song_handler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.active_song_changed.add(handler)
    }

    pub(crate) fn remove_active_song_handler(&self, token: u64) -> bool {
        self.active_song_changed.remove(token)
    }

    pub(crate) fn add_media_state_handler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.media_state_changed.add(handler)
    }

    pub(crate) fn remove_media_state_handler(&self, token: u64) -> bool {
        self.media_state_changed.remove(token)
    }
}

impl Drop for MediaRuntime {
    fn drop(&mut self) {
        if let Some(registrations) = self.registrations.get() {
            let unsubscribe: sys::cna_media_player_unsubscribe_ext_fn = unsafe {
                super::native_function(registrations.native.media.media_player_unsubscribe_ext)
            };
            // SAFETY: both handles are owned by this runtime.
            let _ = registrations.native.check(unsafe { unsubscribe(registrations.active_song) });
            let _ = registrations.native.check(unsafe { unsubscribe(registrations.media_state) });
        }
    }
}

unsafe extern "C" fn active_song_trampoline(context: *mut c_void) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !context.is_null() {
            // SAFETY: registration stores the process-runtime address for its full lifetime.
            unsafe { &*context.cast::<MediaRuntime>() }.enqueue_active_song_changed();
        }
    }));
}

unsafe extern "C" fn media_state_trampoline(context: *mut c_void) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !context.is_null() {
            // SAFETY: registration stores the process-runtime address for its full lifetime.
            unsafe { &*context.cast::<MediaRuntime>() }.enqueue_media_state_changed();
        }
    }));
}
