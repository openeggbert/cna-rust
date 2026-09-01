//! `graphics_resource.h` -- what CNA itself records about a graphics resource.
//!
//! XNA's `GraphicsResource` base is already projected by
//! [`crate::Microsoft::Xna::Framework::Graphics::GraphicsResource`], and until
//! this module existed that projection kept `Name` and `Tag` in Rust and never
//! told CNA. That was a real divergence rather than a tidy simplification: a
//! name set through Rust was invisible to CNA, so the device's
//! `ResourceDestroyed` event -- which reports the name and tag a resource had
//! when it went -- reported an empty one for every resource a Rust caller had
//! named.
//!
//! `Name` and `SetName` now go through CNA. The rest of what `graphics_resource.h`
//! offers has no XNA counterpart to fold into, so it lives here:
//!
//! * [`NativeGraphicsResource::native_to_string`] -- CNA's own `ToString`,
//!   which is *not* XNA's. Both fall back to a type name when the resource is
//!   unnamed; XNA's fallback is `Object.ToString()`, the namespace-qualified
//!   name, and CNA's is the bare one. So an unnamed `Texture2D` is
//!   `"Microsoft.Xna.Framework.Graphics.Texture2D"` to
//!   [`GraphicsResource::ToString`] and `"Texture2D"` here. Neither is made to
//!   stand for the other.
//! * [`NativeGraphicsResource::native_is_disposed`] -- CNA's disposal flag,
//!   which is not the same question as XNA's `IsDisposed`. The Rust one asks
//!   whether the *handle* was released; this asks whether the native object was
//!   disposed. [`NativeGraphicsResource::dispose_in_place`] makes them differ
//!   on purpose.
//! * [`NativeGraphicsResource::native_tag`] -- the opaque `uint64` token CNA
//!   carries and reports through `ResourceDestroyed`. XNA's `Tag` is an
//!   arbitrary managed object, modelled here as `Arc<dyn Any>`, and a Rust
//!   object cannot cross into C. They are two properties, and both are kept.
//! * [`NativeGraphicsResource::device_in_callback`] -- the owning device as CNA
//!   will lend it, which is only inside a lifecycle callback.
//!
//! [`GraphicsResource::ToString`]: crate::Microsoft::Xna::Framework::Graphics::GraphicsResource::ToString

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::Result;
use crate::graphics::resource::ResourceState;
use crate::graphics::GraphicsDevice;
use crate::native::Native;

/// Implemented by every resource that owns a native handle.
///
/// Crate-private: it hands out the internal state, and exists only to give
/// [`NativeGraphicsResource`] one blanket implementation instead of a dozen
/// identical ones.
pub(crate) trait HasResourceState {
    fn resource_state(&self) -> &ResourceState;
}

/// What CNA records about a graphics resource, beyond XNA's own properties.
///
/// Implemented for every resource with a native handle. See the module
/// documentation for why each of these is separate from its XNA neighbour
/// rather than folded into it.
pub trait NativeGraphicsResource {
    /// CNA's `ToString`: the name when set, the bare type name otherwise.
    fn native_to_string(&self) -> Result<String>;

    /// Whether CNA considers the native object disposed.
    fn native_is_disposed(&self) -> Result<bool>;

    /// The opaque token CNA carries and reports through `ResourceDestroyed`.
    fn native_tag(&self) -> Result<u64>;

    /// Sets that token. Zero is the null token.
    fn set_native_tag(&self, tag: u64) -> Result<()>;

    /// Disposes the native object without releasing the C handle.
    ///
    /// After this the resource still answers its name and reports itself
    /// disposed to [`Self::native_is_disposed`]. Dropping the Rust value still
    /// releases the handle, and doing both is safe: repeated disposal is a
    /// documented no-op.
    fn dispose_in_place(&self) -> Result<()>;

    /// The owning device, as CNA will lend it.
    ///
    /// `Ok(None)` means the resource is standalone and has no owning device.
    /// A device-owned resource is only lent its device while its game is
    /// inside a lifecycle callback; outside one this fails rather than
    /// answering `None`, because "there is no device" and "the device is not
    /// borrowable right now" are different facts.
    fn device_in_callback(&self) -> Result<Option<&GraphicsDevice>>;

    /// Subscribes to CNA's own disposing event for this resource.
    ///
    /// Not the same event as XNA's `Disposing`, which
    /// [`GraphicsResource::AddDisposingHandler`] registers for. The XNA one is
    /// raised by the XNA `Dispose()` path and knows only about disposals a
    /// caller asked for. This one is CNA's, and fires whenever the native
    /// object is disposed -- including by a device reset or a content unload
    /// that Rust did not initiate.
    ///
    /// Both are kept because they answer different questions, and neither is
    /// emitted twice for the other's cause.
    ///
    /// The returned value withdraws the subscription when dropped, so it must
    /// be held for as long as the callback should run.
    ///
    /// [`GraphicsResource::AddDisposingHandler`]: crate::Microsoft::Xna::Framework::Graphics::GraphicsResource::AddDisposingHandler
    fn on_native_disposing(
        &self,
        callback: impl FnMut() + Send + 'static,
    ) -> Result<ResourceDisposingSubscription>;
}

/// A live registration on a resource's native disposing event.
///
/// Withdraws itself on drop, in the one order that is safe: the registration
/// is cancelled *before* the boxed closure behind it is freed, because the
/// reverse leaves CNA holding a pointer to a dead closure.
#[must_use = "dropping a ResourceDisposingSubscription immediately unsubscribes it"]
pub struct ResourceDisposingSubscription {
    native: Arc<Native>,
    registration: Mutex<sys::CNA_GraphicsResourceEventRegistrationHandle>,
    callback: Mutex<*mut core::ffi::c_void>,
}

// SAFETY: the pointer is an owned box this value alone frees, and the closure
// behind it is required to be `Send`.
unsafe impl Send for ResourceDisposingSubscription {}

type DisposingClosure = Box<dyn FnMut() + Send + 'static>;

impl ResourceDisposingSubscription {
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
        let result = self
            .native
            .unsubscribe_graphics_resource_disposing(registration);
        let mut callback = self
            .callback
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pointer = core::mem::replace(&mut *callback, core::ptr::null_mut());
        if !pointer.is_null() {
            // SAFETY: the pointer came from `Box::into_raw` in the matching
            // subscribe, and the registration naming it is already withdrawn.
            drop(unsafe { Box::from_raw(pointer.cast::<DisposingClosure>()) });
        }
        result
    }
}

impl Drop for ResourceDisposingSubscription {
    fn drop(&mut self) {
        let _ = self.unsubscribe();
    }
}

impl core::fmt::Debug for ResourceDisposingSubscription {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ResourceDisposingSubscription")
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

impl<T: HasResourceState> NativeGraphicsResource for T {
    fn native_to_string(&self) -> Result<String> {
        self.resource_state().native_string()
    }

    fn native_is_disposed(&self) -> Result<bool> {
        self.resource_state().native_is_disposed()
    }

    fn native_tag(&self) -> Result<u64> {
        self.resource_state().native_tag()
    }

    fn set_native_tag(&self, tag: u64) -> Result<()> {
        self.resource_state().set_native_tag(tag)
    }

    fn dispose_in_place(&self) -> Result<()> {
        self.resource_state().dispose_in_place()
    }

    fn device_in_callback(&self) -> Result<Option<&GraphicsDevice>> {
        let state = self.resource_state();
        Ok(state.native_device_handle()?.map(|_| state.device()))
    }

    fn on_native_disposing(
        &self,
        callback: impl FnMut() + Send + 'static,
    ) -> Result<ResourceDisposingSubscription> {
        unsafe extern "C" fn trampoline(
            _resource: sys::CNA_Handle,
            context: *mut core::ffi::c_void,
        ) {
            if context.is_null() {
                return;
            }
            // SAFETY: the context is the box the subscription owns and has not
            // been freed, because freeing happens only after the registration
            // naming it is withdrawn.
            let closure = unsafe { &mut *context.cast::<DisposingClosure>() };
            // A panic must not cross back into C, and a disposing callback has
            // nowhere to report one.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| closure()));
        }

        let state = self.resource_state();
        let handle = state.require_handle()?;
        let native = Arc::clone(state.device().state_native());
        let boxed: DisposingClosure = Box::new(callback);
        let context = Box::into_raw(Box::new(boxed)).cast::<core::ffi::c_void>();
        match native.subscribe_graphics_resource_disposing(handle, Some(trampoline), context) {
            Ok(registration) => Ok(ResourceDisposingSubscription {
                native,
                registration: Mutex::new(registration),
                callback: Mutex::new(context),
            }),
            Err(error) => {
                // CNA never took the pointer, so this is the only owner left.
                // SAFETY: the box was created immediately above and handed to
                // nothing that kept it.
                drop(unsafe { Box::from_raw(context.cast::<DisposingClosure>()) });
                Err(error)
            }
        }
    }
}
