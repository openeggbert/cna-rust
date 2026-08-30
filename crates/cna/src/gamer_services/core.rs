//! Handle ownership shared by the GamerServices, Avatar and Net object graphs.
//!
//! CNA's gamer services are process-global: no route in `gamer_services.h`
//! takes a game handle, so nothing here is bound to a `Game` generation the
//! way Media and Audio are. What the family does need is one honest answer to
//! "who destroys this handle", because the canonical routes hand out three
//! different things that all look like `CNA_Handle`:
//!
//! - an **owned** handle the caller created and must release, such as a gamer
//!   profile or a friend collection;
//! - a **borrowed** handle valid only while its parent lives, such as the
//!   gamer a `cna_gamer_collection_get_at` answers;
//! - an **owned handle over a borrowed object** -- a fresh handle the caller
//!   must release that aliases an object it does not own. The process-wide
//!   signed-in collection answers these, and releasing one releases the view
//!   rather than the gamer.
//!
//! [`OwnedHandle`] covers the first and third; the second is expressed by a
//! Rust borrow of the parent and never reaches this type.

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::native::Native;

/// A canonical destroy or dispose route.
pub(crate) type ReleaseFn = unsafe extern "C" fn(sys::CNA_Handle) -> sys::CNA_Result;

/// The sentinel an owner that holds nothing carries.
///
/// It is not a CNA handle and is never passed to CNA: `no_release` refuses it,
/// and every borrowed child reads its own handle rather than this one.
const BORROWED_VIEW: sys::CNA_Handle = u64::MAX;

unsafe extern "C" fn no_release(handle: sys::CNA_Handle) -> sys::CNA_Result {
    debug_assert_eq!(handle, BORROWED_VIEW);
    sys::CNA_RESULT_SUCCESS
}

/// The process-wide gamer-services runtime.
///
/// CNA's gamer services have no per-game state, so this is only the audited
/// function table. It exists as a named type rather than a bare `Arc<Native>`
/// so the object graph reads as one family and a future per-game requirement
/// has somewhere to land.
#[derive(Clone, Debug)]
pub(crate) struct GamerServicesRuntime {
    native: Arc<Native>,
}

impl GamerServicesRuntime {
    /// Opens, or reuses, the process CNA library.
    pub(crate) fn open() -> Result<Self> {
        Ok(Self {
            native: Native::process()?,
        })
    }

    pub(crate) fn native(&self) -> &Arc<Native> {
        &self.native
    }

    pub(crate) fn check(&self, result: sys::CNA_Result) -> Result<()> {
        self.native.check(result)
    }
}

/// One native handle this wrapper releases exactly once.
///
/// Release is idempotent and never double-frees: the handle is taken out of
/// the cell before the call, and a failed release puts it back so the caller
/// or `Drop` can retry, which is the contract the canonical destroy routes
/// document.
#[derive(Debug)]
pub(crate) struct OwnedHandle {
    runtime: GamerServicesRuntime,
    handle: Mutex<sys::CNA_Handle>,
    release: ReleaseFn,
}

impl OwnedHandle {
    pub(crate) fn new(
        runtime: GamerServicesRuntime,
        handle: sys::CNA_Handle,
        release: ReleaseFn,
    ) -> Arc<Self> {
        Arc::new(Self {
            runtime,
            handle: Mutex::new(handle),
            release,
        })
    }

    /// An owner that holds no handle, for a facade that only borrows.
    ///
    /// A gamer delivered through a CNA event belongs to CNA's roster: the
    /// handler may read it but must not release it, and it stops being valid
    /// when the callback returns. Pairing that gamer with a live-but-empty
    /// owner keeps the borrowed rule -- there is nothing here to release, and
    /// `release` is a documented no-op for an already-empty slot.
    pub(crate) fn borrowed_view(runtime: GamerServicesRuntime) -> Arc<Self> {
        Arc::new(Self {
            runtime,
            handle: Mutex::new(BORROWED_VIEW),
            release: no_release,
        })
    }

    pub(crate) fn runtime(&self) -> &GamerServicesRuntime {
        &self.runtime
    }

    pub(crate) fn native(&self) -> &Arc<Native> {
        self.runtime.native()
    }

    pub(crate) fn check(&self, result: sys::CNA_Result) -> Result<()> {
        self.runtime.check(result)
    }

    /// The live handle, or the disposed error a released wrapper must report.
    pub(crate) fn get(&self) -> Result<sys::CNA_Handle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (handle != 0)
            .then_some(handle)
            .ok_or(CnaError::InvalidInput("the gamer-services object is disposed"))
    }

    pub(crate) fn is_released(&self) -> bool {
        *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            == 0
    }

    /// Releases the handle once. A second call is a success that does nothing.
    pub(crate) fn release(&self) -> Result<()> {
        let mut slot = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = *slot;
        if handle == 0 {
            return Ok(());
        }
        // Cleared first so a panic between here and the call cannot leave a
        // handle that `Drop` would release a second time.
        *slot = 0;
        // SAFETY: the handle came from the matching canonical constructor and
        // is released exactly once.
        let result = unsafe { (self.release)(handle) };
        if result == sys::CNA_RESULT_SUCCESS {
            return Ok(());
        }
        // CNA kept the resource, so the wrapper keeps the handle: releasing is
        // still owed, and a caller or `Drop` may retry it.
        *slot = handle;
        self.runtime.check(result)
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        let handle = *self
            .handle
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == 0 {
            return;
        }
        // SAFETY: the same exactly-once release the explicit path performs.
        let _ = unsafe { (self.release)(handle) };
    }
}

/// Reads a CNA UTF-8 string through a handle's canonical size/copy pair.
pub(crate) fn read_owned_string(
    owner: &OwnedHandle,
    size: impl Fn(sys::CNA_Handle, *mut u64) -> sys::CNA_Result,
    copy: impl Fn(sys::CNA_Handle, *mut core::ffi::c_char, u64, *mut u64) -> sys::CNA_Result,
) -> Result<String> {
    let handle = owner.get()?;
    crate::native::runtime::read_string(
        |result| owner.check(result),
        |bytes| size(handle, bytes),
        |destination, capacity, written| copy(handle, destination, capacity, written),
    )
}
