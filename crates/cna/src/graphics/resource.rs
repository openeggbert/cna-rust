#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    non_snake_case
)]

use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::{EventArgs, EventHandler};

use super::GraphicsDevice;

#[derive(Clone, Copy)]
pub(super) enum ResourceKind {
    Texture2D,
    SpriteBatch,
}

pub(super) struct ResourceState {
    device: GraphicsDevice,
    handle: Mutex<sys::CNA_Handle>,
    kind: ResourceKind,
    active: Mutex<bool>,
    name: Mutex<String>,
    tag: Mutex<Option<Arc<dyn Any + Send + Sync>>>,
    disposing: EventHandlers,
}

type SharedHandler = Arc<Mutex<Box<dyn EventHandler>>>;

#[derive(Default)]
pub(super) struct EventHandlers {
    state: Mutex<EventHandlerState>,
}

#[derive(Default)]
struct EventHandlerState {
    next_registration: u64,
    entries: Vec<(u64, SharedHandler)>,
}

impl EventHandlers {
    pub(super) const fn new() -> Self {
        Self {
            state: Mutex::new(EventHandlerState {
                next_registration: 0,
                entries: Vec::new(),
            }),
        }
    }

    pub(super) fn add(&self, handler: Box<dyn EventHandler>) -> u64 {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.next_registration = state.next_registration.wrapping_add(1).max(1);
        let registration = state.next_registration;
        state
            .entries
            .push((registration, Arc::new(Mutex::new(handler))));
        registration
    }

    pub(super) fn remove(&self, registration: u64) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let length = state.entries.len();
        state.entries.retain(|(value, _)| *value != registration);
        length != state.entries.len()
    }

    pub(super) fn emit(&self, sender: &dyn Any) -> bool {
        let handlers = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entries
            .iter()
            .map(|(_, handler)| Arc::clone(handler))
            .collect::<Vec<_>>();
        let mut panicked = false;
        for handler in handlers {
            let result = catch_unwind(AssertUnwindSafe(|| {
                handler
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .invoke(sender, EventArgs);
            }));
            panicked |= result.is_err();
        }
        panicked
    }
}

impl ResourceState {
    pub(super) fn new(
        device: &GraphicsDevice,
        handle: sys::CNA_Handle,
        kind: ResourceKind,
    ) -> Arc<Self> {
        let state = Arc::new(Self {
            device: device.clone(),
            handle: Mutex::new(handle),
            kind,
            active: Mutex::new(false),
            name: Mutex::new(String::new()),
            tag: Mutex::new(None),
            disposing: EventHandlers::default(),
        });
        device.state.register(&state);
        state
    }

    pub(super) fn handle(&self) -> Option<sys::CNA_Handle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (handle != sys::CNA_INVALID_HANDLE).then_some(handle)
    }

    pub(super) fn require_handle(&self) -> Result<sys::CNA_Handle> {
        self.device.state.ensure_alive()?;
        self.handle()
            .ok_or(CnaError::InvalidInput("graphics resource is disposed"))
    }

    pub(super) fn device(&self) -> &GraphicsDevice {
        &self.device
    }

    pub(super) fn is_active(&self) -> bool {
        *self
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(super) fn set_active(&self, value: bool) {
        *self
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
    }

    pub(super) fn name(&self) -> String {
        self.name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub(super) fn set_name(&self, value: &str) {
        *self
            .name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value.to_owned();
    }

    pub(super) fn tag(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.tag
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub(super) fn set_tag(&self, value: Option<Arc<dyn Any + Send + Sync>>) {
        *self
            .tag
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
    }

    pub(super) fn add_disposing_handler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.disposing.add(handler)
    }

    pub(super) fn remove_disposing_handler(&self, registration: u64) -> bool {
        self.disposing.remove(registration)
    }

    pub(super) fn dispose_with_event(&self, sender: &dyn Any, disposing: bool) -> Result<()> {
        if self.handle().is_none() {
            return Ok(());
        }
        let handler_panicked = disposing && self.disposing.emit(sender);
        let result = self.dispose_native();
        if handler_panicked {
            Err(CnaError::Callback(
                "Rust event-handler panic was contained before the native boundary".to_owned(),
            ))
        } else {
            result
        }
    }

    pub(super) fn dispose_native(&self) -> Result<()> {
        let mut handle = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        if matches!(self.kind, ResourceKind::SpriteBatch) && self.is_active() {
            self.device.state.native().end_sprite_batch(*handle)?;
            self.set_active(false);
        }
        match self.kind {
            ResourceKind::Texture2D => self.device.state.native().destroy_texture(*handle)?,
            ResourceKind::SpriteBatch => {
                self.device.state.native().destroy_sprite_batch(*handle)?;
            }
        };
        *handle = sys::CNA_INVALID_HANDLE;
        Ok(())
    }
}

impl Drop for ResourceState {
    fn drop(&mut self) {
        let _ = self.dispose_native();
    }
}

/// Public behavior shared by XNA graphics resources.
pub trait GraphicsResource {
    fn GraphicsDevice(&self) -> Option<&GraphicsDevice>;
    fn IsDisposed(&self) -> bool;
    fn Name(&self) -> String;
    fn SetName(&mut self, value: &str);
    fn Tag(&self) -> Option<Arc<dyn Any + Send + Sync>>;
    fn SetTag(&mut self, value: Option<Arc<dyn Any + Send + Sync>>);
    fn AddDisposingHandler(&mut self, handler: Box<dyn EventHandler>) -> u64;
    fn RemoveDisposingHandler(&mut self, registration: u64) -> bool;
    fn Dispose(&mut self, value: bool) -> Result<()>;
    fn DisposeWithNoArguments(&mut self) -> Result<()> {
        self.Dispose(true)
    }
    fn Finalize(&self) {}
    fn ToString(&self) -> String {
        let name = self.Name();
        if name.is_empty() {
            format!(
                "Microsoft.Xna.Framework.Graphics.{}",
                std::any::type_name::<Self>()
                    .rsplit("::")
                    .next()
                    .unwrap_or("GraphicsResource")
            )
        } else {
            name
        }
    }
}

/// XNA base relationship projected as a Rust trait.
pub trait Texture: GraphicsResource {
    fn Format(&self) -> crate::Microsoft::Xna::Framework::Graphics::SurfaceFormat;
    fn LevelCount(&self) -> i32;
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use crate::extensions::events::EventArgs;

    use super::EventHandlers;

    #[test]
    fn event_registry_preserves_order_removal_and_self_removal() {
        let events = Arc::new(EventHandlers::default());
        let order = Arc::new(Mutex::new(Vec::new()));

        let first_order = Arc::clone(&order);
        let first = events.add(Box::new(move |_: &dyn Any, _: EventArgs| {
            first_order.lock().expect("order").push(1);
        }));
        let second_order = Arc::clone(&order);
        let calls = Arc::new(AtomicUsize::new(0));
        let self_registration = Arc::new(AtomicU64::new(0));
        let weak_events = Arc::downgrade(&events);
        let handler_calls = Arc::clone(&calls);
        let handler_registration = Arc::clone(&self_registration);
        let second = events.add(Box::new(move |_: &dyn Any, _: EventArgs| {
            second_order.lock().expect("order").push(2);
            handler_calls.fetch_add(1, Ordering::Relaxed);
            if let Some(events) = weak_events.upgrade() {
                events.remove(handler_registration.load(Ordering::Relaxed));
            }
        }));
        self_registration.store(second, Ordering::Relaxed);

        assert!(!events.emit(&()));
        assert_eq!(*order.lock().expect("order"), [1, 2]);
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        assert!(events.remove(first));
        assert!(!events.remove(first));
        assert!(!events.emit(&()));
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
}
