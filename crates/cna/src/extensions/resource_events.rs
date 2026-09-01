//! `ContentLost`, for the three resource families that carry it.
//!
//! A **dynamic** vertex buffer, a **dynamic** index buffer and a render target
//! each lose their backing store when the graphics device does -- a reset, a
//! lost device, a suspend and resume. XNA raises `ContentLost` so a game can
//! refill them, and a game that ignores it draws garbage after an alt-tab.
//!
//! Static buffers are not in that list, and the header says so: their contents
//! are the driver's to restore. Measured, `cna_vertex_buffer_subscribe_content_lost`
//! on a static buffer answers "ContentLost exists only on DynamicVertexBuffer",
//! which is why this trait is implemented for the dynamic types alone rather
//! than for every buffer with a handle -- an implementation that always refused
//! would be a worse projection than none.
//!
//! Nothing in Rust could raise this on its own: only CNA knows the device was
//! reset. That is why `DynamicVertexBuffer::AddContentLostHandler` and its
//! index counterpart used to **never fire** -- they were added, removed, and
//! emitted nowhere, so a caller that registered one waited forever.
//!
//! `RUST-EXT-018` closed that. Each dynamic buffer now installs **one** of
//! these subscriptions, on its first XNA-shaped handler, and the trampoline
//! delivers into the buffer's own handler list in registration order. So both
//! routes work and they are the same event: this one for a caller who wants
//! the subscription's lifetime in hand, `AddContentLostHandler` for a caller
//! writing XNA.
//!
//! All three follow one shape, so they share one subscription type. It
//! withdraws itself on drop, in the only order that is safe: the registration
//! is cancelled *before* the boxed closure behind it is freed, because the
//! reverse leaves CNA holding a pointer to a dead closure.

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::Result;
use crate::extensions::graphics_resource::HasResourceState;
use crate::native::Native;
use crate::Microsoft::Xna::Framework::Graphics::{
    DynamicIndexBuffer, DynamicVertexBuffer, RenderTarget2D, RenderTargetCube,
};

type ContentLostClosure = Box<dyn FnMut() + Send + 'static>;

/// How a registration is withdrawn, which differs per family.
#[derive(Clone, Copy)]
enum Withdraw {
    VertexBuffer,
    IndexBuffer,
    RenderTarget,
}

/// A live registration on a resource's `ContentLost` event.
#[must_use = "dropping a ContentLostSubscription immediately unsubscribes it"]
pub struct ContentLostSubscription {
    native: Arc<Native>,
    withdraw: Withdraw,
    registration: Mutex<sys::CNA_Handle>,
    callback: Mutex<*mut core::ffi::c_void>,
}

// SAFETY: the pointer is an owned box this value alone frees, and the closure
// behind it is required to be `Send`.
unsafe impl Send for ContentLostSubscription {}

impl ContentLostSubscription {
    /// Withdraws the subscription early. Idempotent.
    pub fn unsubscribe(&self) -> Result<()> {
        let mut guard = self
            .registration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let registration = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if registration == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        let result = match self.withdraw {
            Withdraw::VertexBuffer => self
                .native
                .unsubscribe_vertex_buffer_content_lost(registration),
            Withdraw::IndexBuffer => self
                .native
                .unsubscribe_index_buffer_content_lost(registration),
            Withdraw::RenderTarget => self
                .native
                .unsubscribe_render_target_content_lost(registration),
        };
        let mut callback = self
            .callback
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pointer = core::mem::replace(&mut *callback, core::ptr::null_mut());
        if !pointer.is_null() {
            // SAFETY: the pointer came from `Box::into_raw` below, and the
            // registration naming it is already withdrawn.
            drop(unsafe { Box::from_raw(pointer.cast::<ContentLostClosure>()) });
        }
        result
    }
}

impl Drop for ContentLostSubscription {
    fn drop(&mut self) {
        let _ = self.unsubscribe();
    }
}

impl core::fmt::Debug for ContentLostSubscription {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ContentLostSubscription")
            .field(
                "registration",
                &*self
                    .registration
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            )
            .finish_non_exhaustive()
    }
}

unsafe extern "C" fn trampoline(_resource: sys::CNA_Handle, context: *mut core::ffi::c_void) {
    if context.is_null() {
        return;
    }
    // SAFETY: the context is the box the subscription owns and is freed only
    // after the registration naming it has been withdrawn.
    let closure = unsafe { &mut *context.cast::<ContentLostClosure>() };
    // A panic must not cross back into C, and a device reset has nowhere to
    // report one.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| closure()));
}

/// Resources whose contents CNA can lose.
pub trait NotifiesContentLost {
    /// Subscribes to this resource's `ContentLost` event.
    ///
    /// The returned value withdraws the registration when dropped, so it must
    /// be held for as long as the callback should run.
    fn on_content_lost(
        &self,
        callback: impl FnMut() + Send + 'static,
    ) -> Result<ContentLostSubscription>;
}

fn subscribe(
    native: Arc<Native>,
    withdraw: Withdraw,
    handle: sys::CNA_Handle,
    callback: impl FnMut() + Send + 'static,
) -> Result<ContentLostSubscription> {
    let boxed: ContentLostClosure = Box::new(callback);
    let context = Box::into_raw(Box::new(boxed)).cast::<core::ffi::c_void>();
    let outcome = match withdraw {
        Withdraw::VertexBuffer => {
            native.subscribe_vertex_buffer_content_lost(handle, Some(trampoline), context)
        }
        Withdraw::IndexBuffer => {
            native.subscribe_index_buffer_content_lost(handle, Some(trampoline), context)
        }
        Withdraw::RenderTarget => {
            native.subscribe_render_target_content_lost(handle, Some(trampoline), context)
        }
    };
    match outcome {
        Ok(registration) => Ok(ContentLostSubscription {
            native,
            withdraw,
            registration: Mutex::new(registration),
            callback: Mutex::new(context),
        }),
        Err(error) => {
            // CNA never took the pointer, so this is the only owner left.
            // SAFETY: the box was created immediately above.
            drop(unsafe { Box::from_raw(context.cast::<ContentLostClosure>()) });
            Err(error)
        }
    }
}

macro_rules! notifies_content_lost {
    ($type:ty, $withdraw:expr) => {
        impl NotifiesContentLost for $type {
            fn on_content_lost(
                &self,
                callback: impl FnMut() + Send + 'static,
            ) -> Result<ContentLostSubscription> {
                let state = self.resource_state();
                subscribe(
                    Arc::clone(state.device().state_native()),
                    $withdraw,
                    state.require_handle()?,
                    callback,
                )
            }
        }
    };
}

notifies_content_lost!(DynamicVertexBuffer, Withdraw::VertexBuffer);
notifies_content_lost!(DynamicIndexBuffer, Withdraw::IndexBuffer);
notifies_content_lost!(RenderTarget2D, Withdraw::RenderTarget);
notifies_content_lost!(RenderTargetCube, Withdraw::RenderTarget);
