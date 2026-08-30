//! XNA's gamer-services event payloads and the sign-in/out registries.
//!
//! # Callback ownership
//!
//! CNA's sign-in and sign-out events are process-global: subscribing returns a
//! registration handle and there is no per-object owner to tie it to. The
//! projection therefore keeps one process-wide registry per event, subscribes
//! to CNA the first time a handler is added, and unsubscribes when the last
//! one is removed. The trampoline reads nothing that can outlive the call: it
//! copies the gamer handle out of CNA's event record into a fresh borrowed
//! facade valid only for the handler's duration, and a panic in a handler is
//! caught before it can unwind into C.

#![allow(non_snake_case)]

use core::ffi::c_void;
use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, OnceLock};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::EventHandler;

use super::core::{GamerServicesRuntime, OwnedHandle};
use super::gamer::{GamerCore, SignedInGamer};

/// XNA `Microsoft.Xna.Framework.GamerServices.SignedInEventArgs`.
#[derive(Clone, Debug)]
pub struct SignedInEventArgs {
    gamer: SignedInGamer,
}

impl SignedInEventArgs {
    /// XNA `SignedInEventArgs(SignedInGamer)`.
    #[must_use]
    pub fn new(gamer: &SignedInGamer) -> Self {
        Self {
            gamer: gamer.clone(),
        }
    }

    /// XNA `SignedInEventArgs.Gamer`.
    #[must_use]
    pub const fn Gamer(&self) -> &SignedInGamer {
        &self.gamer
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.SignedOutEventArgs`.
#[derive(Clone, Debug)]
pub struct SignedOutEventArgs {
    gamer: SignedInGamer,
}

impl SignedOutEventArgs {
    /// XNA `SignedOutEventArgs(SignedInGamer)`.
    #[must_use]
    pub fn new(gamer: &SignedInGamer) -> Self {
        Self {
            gamer: gamer.clone(),
        }
    }

    /// XNA `SignedOutEventArgs.Gamer`.
    #[must_use]
    pub const fn Gamer(&self) -> &SignedInGamer {
        &self.gamer
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.InviteAcceptedEventArgs`.
#[derive(Clone, Debug)]
pub struct InviteAcceptedEventArgs {
    gamer: SignedInGamer,
    is_current_session: bool,
}

impl InviteAcceptedEventArgs {
    /// XNA `InviteAcceptedEventArgs(SignedInGamer, bool)`.
    #[must_use]
    pub fn new(gamer: &SignedInGamer, isCurrentSession: bool) -> Self {
        Self {
            gamer: gamer.clone(),
            is_current_session: isCurrentSession,
        }
    }

    /// XNA `InviteAcceptedEventArgs.Gamer`.
    #[must_use]
    pub const fn Gamer(&self) -> &SignedInGamer {
        &self.gamer
    }

    /// XNA `InviteAcceptedEventArgs.IsCurrentSession`.
    #[must_use]
    pub const fn IsCurrentSession(&self) -> bool {
        self.is_current_session
    }
}

type SignedInHandlers = Mutex<Vec<(u64, Arc<Mutex<Box<dyn EventHandler<SignedInEventArgs>>>>)>>;
type SignedOutHandlers = Mutex<Vec<(u64, Arc<Mutex<Box<dyn EventHandler<SignedOutEventArgs>>>>)>>;

/// One process-wide event registry plus its CNA subscription.
struct Registry {
    next: Mutex<u64>,
    signed_in: SignedInHandlers,
    signed_out: SignedOutHandlers,
    /// CNA registrations, held for exactly as long as a handler wants them.
    subscription: Mutex<(Option<sys::CNA_Handle>, Option<sys::CNA_Handle>)>,
    /// A subscription CNA refused, surfaced by `GamerServicesDispatcher.Update`.
    ///
    /// XNA's `+=` cannot fail, so the projection registers the handler and
    /// remembers the refusal rather than swallowing it. The dispatcher is
    /// where the CLR delivers these events from, so it is where the caller
    /// learns that none will arrive.
    pending: Mutex<Option<CnaError>>,
    runtime: GamerServicesRuntime,
}

/// Reports and clears any subscription CNA refused.
///
/// # Errors
///
/// Returns the error CNA reported when a sign-in or sign-out subscription
/// could not be established.
pub(crate) fn take_subscription_error() -> Result<()> {
    let Ok(registry) = registry() else {
        return Ok(());
    };
    let taken = registry
        .pending
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    match taken {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

fn registry() -> Result<&'static Registry> {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    if let Some(existing) = REGISTRY.get() {
        return Ok(existing);
    }
    let created = Registry {
        next: Mutex::new(0),
        signed_in: Mutex::new(Vec::new()),
        signed_out: Mutex::new(Vec::new()),
        subscription: Mutex::new((None, None)),
        pending: Mutex::new(None),
        runtime: GamerServicesRuntime::open()?,
    };
    Ok(REGISTRY.get_or_init(|| created))
}

fn next_registration(registry: &Registry) -> u64 {
    let mut next = registry
        .next
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *next = next.wrapping_add(1).max(1);
    *next
}

unsafe extern "C" fn signed_in_trampoline(
    _context: *mut c_void,
    info: *const sys::CNA_SignedInGamerEventInfo,
) {
    dispatch(info, |registry, gamer| {
        let handlers = registry
            .signed_in
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for (_, handler) in handlers {
            let args = SignedInEventArgs::new(gamer);
            let mut guard = handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            // A panicking handler must not unwind into CNA. The event is
            // dropped for that handler and the rest still run, which is what
            // the CLR multicast delegate would do for a handler that returned.
            let _ = catch_unwind(AssertUnwindSafe(|| guard.invoke(&() as &dyn Any, args)));
        }
    });
}

unsafe extern "C" fn signed_out_trampoline(
    _context: *mut c_void,
    info: *const sys::CNA_SignedInGamerEventInfo,
) {
    dispatch(info, |registry, gamer| {
        let handlers = registry
            .signed_out
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for (_, handler) in handlers {
            let args = SignedOutEventArgs::new(gamer);
            let mut guard = handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let _ = catch_unwind(AssertUnwindSafe(|| guard.invoke(&() as &dyn Any, args)));
        }
    });
}

/// Turns CNA's event record into a facade valid for the handler's duration.
fn dispatch(
    info: *const sys::CNA_SignedInGamerEventInfo,
    run: impl Fn(&Registry, &SignedInGamer),
) {
    if info.is_null() {
        return;
    }
    // SAFETY: CNA passes a live record for the duration of the callback and
    // the projection copies the handle out rather than retaining the pointer.
    let handle = unsafe { (*info).gamer };
    let Ok(registry) = registry() else {
        return;
    };
    if handle == 0 {
        return;
    }
    // The event's gamer belongs to CNA's roster, not to the handler: the
    // facade borrows it for this call and releases nothing.
    let borrow = OwnedHandle::borrowed_view(registry.runtime.clone());
    let gamer = SignedInGamer::from_core(GamerCore::borrowed(borrow, handle));
    run(registry, &gamer);
}

fn ensure_subscribed(registry: &'static Registry, signed_in: bool) -> Result<()> {
    let mut subscription = registry
        .subscription
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let slot = if signed_in {
        &mut subscription.0
    } else {
        &mut subscription.1
    };
    if slot.is_some() {
        return Ok(());
    }
    let mut handle = 0;
    let api = &registry.runtime.native().gamer_services;
    let result = if signed_in {
        // SAFETY: the trampoline is a plain C function and the output is live.
        unsafe {
            (api.signed_in_gamer_subscribe_signed_in_ext)(
                Some(signed_in_trampoline),
                core::ptr::null_mut(),
                &mut handle,
            )
        }
    } else {
        // SAFETY: as above for the sign-out event.
        unsafe {
            (api.signed_in_gamer_subscribe_signed_out_ext)(
                Some(signed_out_trampoline),
                core::ptr::null_mut(),
                &mut handle,
            )
        }
    };
    registry.runtime.check(result)?;
    *slot = Some(handle);
    Ok(())
}

fn release_if_empty(registry: &'static Registry, signed_in: bool, empty: bool) {
    if !empty {
        return;
    }
    let mut subscription = registry
        .subscription
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let slot = if signed_in {
        &mut subscription.0
    } else {
        &mut subscription.1
    };
    if let Some(handle) = slot.take() {
        // SAFETY: the registration came from the matching subscribe route and
        // is released exactly once.
        let _ = registry
            .runtime
            .check(unsafe { (registry.runtime.native().gamer_services.gamer_unsubscribe_ext)(handle) });
    }
}

pub(crate) fn add_signed_in(handler: Box<dyn EventHandler<SignedInEventArgs>>) -> u64 {
    let Ok(registry) = registry() else {
        return 0;
    };
    remember(registry, ensure_subscribed(registry, true));
    let registration = next_registration(registry);
    registry
        .signed_in
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push((registration, Arc::new(Mutex::new(handler))));
    registration
}

fn remember(registry: &'static Registry, result: Result<()>) {
    if let Err(error) = result {
        let mut pending = registry
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if pending.is_none() {
            *pending = Some(error);
        }
    }
}

pub(crate) fn remove_signed_in(registration: u64) -> bool {
    let Ok(registry) = registry() else {
        return false;
    };
    let (removed, empty) = {
        let mut handlers = registry
            .signed_in
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let before = handlers.len();
        handlers.retain(|(value, _)| *value != registration);
        (before != handlers.len(), handlers.is_empty())
    };
    release_if_empty(registry, true, empty);
    removed
}

pub(crate) fn add_signed_out(handler: Box<dyn EventHandler<SignedOutEventArgs>>) -> u64 {
    let Ok(registry) = registry() else {
        return 0;
    };
    remember(registry, ensure_subscribed(registry, false));
    let registration = next_registration(registry);
    registry
        .signed_out
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push((registration, Arc::new(Mutex::new(handler))));
    registration
}

type PlainHandlers = Mutex<Vec<(u64, Arc<Mutex<Box<dyn EventHandler>>>)>>;

static TITLE_UPDATE: OnceLock<(PlainHandlers, Mutex<Option<sys::CNA_Handle>>)> = OnceLock::new();

fn title_update() -> &'static (PlainHandlers, Mutex<Option<sys::CNA_Handle>>) {
    TITLE_UPDATE.get_or_init(|| (Mutex::new(Vec::new()), Mutex::new(None)))
}

unsafe extern "C" fn title_update_trampoline(_context: *mut c_void) {
    let handlers = title_update()
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    for (_, handler) in handlers {
        let mut guard = handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Contained: a panicking handler must not unwind into CNA.
        let _ = catch_unwind(AssertUnwindSafe(|| {
            guard.invoke(&() as &dyn Any, crate::extensions::events::EventArgs)
        }));
    }
}

pub(crate) fn add_installing_title_update(handler: Box<dyn EventHandler>) -> Result<u64> {
    let registry = registry()?;
    let state = title_update();
    {
        let mut subscription = state
            .1
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if subscription.is_none() {
            let mut handle = 0;
            // SAFETY: the trampoline is a plain C function and the output is live.
            registry.runtime.check(unsafe {
                (registry
                    .runtime
                    .native()
                    .gamer_services
                    .gamer_services_dispatcher_subscribe_installing_title_update_ext)(
                    Some(title_update_trampoline),
                    core::ptr::null_mut(),
                    &mut handle,
                )
            })?;
            *subscription = Some(handle);
        }
    }
    let registration = next_registration(registry);
    state
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push((registration, Arc::new(Mutex::new(handler))));
    Ok(registration)
}

static AVATAR_CHANGED: OnceLock<(PlainHandlers, Mutex<Option<sys::CNA_Handle>>)> = OnceLock::new();

fn avatar_changed() -> &'static (PlainHandlers, Mutex<Option<sys::CNA_Handle>>) {
    AVATAR_CHANGED.get_or_init(|| (Mutex::new(Vec::new()), Mutex::new(None)))
}

unsafe extern "C" fn avatar_changed_trampoline(_context: *mut c_void) {
    let handlers = avatar_changed()
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    for (_, handler) in handlers {
        let mut guard = handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Contained: a panicking handler must not unwind into CNA.
        let _ = catch_unwind(AssertUnwindSafe(|| {
            guard.invoke(&() as &dyn Any, crate::extensions::events::EventArgs)
        }));
    }
}

pub(crate) fn add_avatar_description_changed(handler: Box<dyn EventHandler>) -> Result<u64> {
    let registry = registry()?;
    let state = avatar_changed();
    {
        let mut subscription = state
            .1
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if subscription.is_none() {
            let mut handle = 0;
            // SAFETY: the trampoline is a plain C function and the output is live.
            registry.runtime.check(unsafe {
                (registry
                    .runtime
                    .native()
                    .gamer_services
                    .avatar_description_subscribe_changed_ext)(
                    Some(avatar_changed_trampoline),
                    core::ptr::null_mut(),
                    &mut handle,
                )
            })?;
            *subscription = Some(handle);
        }
    }
    let registration = next_registration(registry);
    state
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push((registration, Arc::new(Mutex::new(handler))));
    Ok(registration)
}

pub(crate) fn remove_avatar_description_changed(registration: u64) -> Result<bool> {
    let registry = registry()?;
    let state = avatar_changed();
    let (removed, empty) = {
        let mut handlers = state
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let before = handlers.len();
        handlers.retain(|(value, _)| *value != registration);
        (before != handlers.len(), handlers.is_empty())
    };
    if empty {
        let taken = state
            .1
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(handle) = taken {
            // SAFETY: the registration came from the subscribe route above.
            registry.runtime.check(unsafe {
                (registry.runtime.native().gamer_services.gamer_unsubscribe_ext)(handle)
            })?;
        }
    }
    Ok(removed)
}

pub(crate) fn remove_installing_title_update(registration: u64) -> Result<bool> {
    let registry = registry()?;
    let state = title_update();
    let (removed, empty) = {
        let mut handlers = state
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let before = handlers.len();
        handlers.retain(|(value, _)| *value != registration);
        (before != handlers.len(), handlers.is_empty())
    };
    if empty {
        let taken = state
            .1
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(handle) = taken {
            // SAFETY: the registration came from the subscribe route above.
            registry.runtime.check(unsafe {
                (registry.runtime.native().gamer_services.gamer_unsubscribe_ext)(handle)
            })?;
        }
    }
    Ok(removed)
}

pub(crate) fn remove_signed_out(registration: u64) -> bool {
    let Ok(registry) = registry() else {
        return false;
    };
    let (removed, empty) = {
        let mut handlers = registry
            .signed_out
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let before = handlers.len();
        handlers.retain(|(value, _)| *value != registration);
        (before != handlers.len(), handlers.is_empty())
    };
    release_if_empty(registry, false, empty);
    removed
}
