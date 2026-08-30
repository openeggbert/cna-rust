//! The XNA `Begin*`/`End*` pattern for gamer services.
//!
//! CNA's gamer-services `begin_*` routes complete before they return and then
//! invoke the caller's callback, exactly as the Storage selectors do. The
//! projection therefore mirrors [`crate::StorageAsyncResult`]: a concrete
//! result type rather than CLR `IAsyncResult`, a thread pool, or a fabricated
//! pending task, and the observable CLR `End` rules are enforced -- `End` is
//! one-shot and a result is only valid for the operation that produced it.
//!
//! The Rust callback runs from inside CNA's own callback, not after the native
//! call returns, so the canonical callback path is genuinely exercised. A
//! panic in it is caught at the boundary and reported as
//! [`crate::CnaError::Callback`]; nothing unwinds through C.

#![allow(non_snake_case)]

use core::ffi::c_void;
use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::error::{CnaError, Result};

/// The caller state XNA passes through a `Begin*` call unchanged.
pub type GamerAsyncState = Option<Arc<dyn Any + Send + Sync>>;

/// The one-shot completion callback a `Begin*` call accepts.
pub type GamerAsyncCallback = Box<dyn FnOnce(&GamerAsyncResult) + Send>;

static NEXT_ORIGIN: AtomicU64 = AtomicU64::new(1);

struct Inner {
    state: Mutex<GamerAsyncState>,
    /// The value the completed operation produced, taken by the one `End`.
    ///
    /// Type-erased so this module stays independent of the object graph it
    /// carries; every `End*` downcasts to the one type its own `Begin*` stored,
    /// and a result handed to the wrong `End` fails rather than reinterprets.
    value: Mutex<Option<Box<dyn Any + Send>>>,
    completed: AtomicBool,
    ended: AtomicBool,
    origin: u64,
}

impl core::fmt::Debug for Inner {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("GamerAsyncResult")
            .field("origin", &self.origin)
            .field("completed", &self.completed.load(Ordering::Acquire))
            .field("ended", &self.ended.load(Ordering::Acquire))
            .finish()
    }
}

/// XNA's `IAsyncResult` for one gamer-services operation.
#[derive(Clone, Debug)]
pub struct GamerAsyncResult {
    inner: Arc<Inner>,
}

impl GamerAsyncResult {
    pub(crate) fn completed(state: GamerAsyncState) -> Self {
        Self {
            inner: Arc::new(Inner {
                state: Mutex::new(state),
                value: Mutex::new(None),
                completed: AtomicBool::new(true),
                ended: AtomicBool::new(false),
                origin: NEXT_ORIGIN.fetch_add(1, Ordering::Relaxed),
            }),
        }
    }

    /// CLR `IAsyncResult.AsyncState`.
    #[must_use]
    pub fn AsyncState(&self) -> GamerAsyncState {
        self.inner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// CLR `IAsyncResult.CompletedSynchronously`.
    ///
    /// Always true: CNA completes every gamer-services operation before its
    /// `begin_*` route returns.
    #[must_use]
    pub const fn CompletedSynchronously(&self) -> bool {
        true
    }

    /// CLR `IAsyncResult.IsCompleted`.
    #[must_use]
    pub fn IsCompleted(&self) -> bool {
        self.inner.completed.load(Ordering::Acquire)
    }

    pub(crate) fn store(&self, value: Box<dyn Any + Send>) {
        *self
            .inner
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(value);
    }

    /// Consumes the one permitted `End` and answers what `Begin` produced.
    ///
    /// Enforces both observable CLR rules: `End` is one-shot, and a result is
    /// valid only for the operation that created it -- which here means the
    /// `End` whose value type matches.
    pub(crate) fn end_once<T: 'static>(&self) -> Result<T> {
        if self.inner.ended.swap(true, Ordering::AcqRel) {
            return Err(CnaError::InvalidInput(
                "a gamer-services End method cannot be called twice",
            ));
        }
        let taken = self
            .inner
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .ok_or(CnaError::InvalidInput(
                "the gamer-services asynchronous result has no value",
            ))?;
        taken.downcast::<T>().map(|value| *value).map_err(|_| {
            CnaError::InvalidInput(
                "the asynchronous result belongs to a different gamer-services operation",
            )
        })
    }
}

/// How many times CNA invoked the native completion callback.
struct Fired(usize);

/// Runs one `begin_*` route and then completes the Rust callback.
///
/// A native trampoline is installed so CNA's own callback path is genuinely
/// exercised and its exactly-once behaviour is observable, but the Rust
/// closure runs **after** the native call returns and after the produced value
/// is stored. That ordering is what makes XNA's usual idiom work: a completion
/// callback that immediately calls the matching `End` finds the value there.
/// CNA completes these operations before returning, so nothing is observable
/// between the two points.
///
/// The trampoline itself only counts, so no Rust value and no panic can cross
/// the C boundary at all.
pub(crate) fn with_callback<T: Any + Send>(
    state: GamerAsyncState,
    callback: Option<GamerAsyncCallback>,
    native: impl FnOnce(sys_callback, *mut c_void) -> Result<T>,
) -> Result<(GamerAsyncResult, usize)> {
    let mut fired = Fired(0);
    let context: *mut c_void = core::ptr::addr_of_mut!(fired).cast();
    let value = native(Some(trampoline), context)?;
    let result = GamerAsyncResult::completed(state);
    result.store(Box::new(value));
    if let Some(callback) = callback {
        catch_unwind(AssertUnwindSafe(|| callback(&result))).map_err(|_| {
            CnaError::Callback("a gamer-services completion callback panicked".to_owned())
        })?;
    }
    Ok((result, fired.0))
}

/// The canonical `void (*)(void*)` shape every gamer-services begin route takes.
#[allow(non_camel_case_types)]
pub(crate) type sys_callback = cna_sys::CNA_GamerAsyncCallback;

unsafe extern "C" fn trampoline(context: *mut c_void) {
    if context.is_null() {
        return;
    }
    // SAFETY: `with_callback` keeps the counter alive across the native call,
    // and CNA invokes the callback on the calling thread before that call
    // returns, so no other reference to it exists here.
    let fired = unsafe { &mut *context.cast::<Fired>() };
    fired.0 = fired.0.saturating_add(1);
}
