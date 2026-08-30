//! Rust projections of the CLR event vocabulary XNA's public API uses.

use std::any::Any;

/// Rust value used for CLR's stateless `EventArgs` payload.
#[derive(Clone, Copy, Debug, Default)]
pub struct EventArgs;

/// Type-erased XNA event callback.
pub trait EventHandler<T = EventArgs>: Send {
    fn invoke(&mut self, sender: &dyn Any, args: T);
}

impl<F, T> EventHandler<T> for F
where
    F: FnMut(&dyn Any, T) + Send,
{
    fn invoke(&mut self, sender: &dyn Any, args: T) {
        self(sender, args);
    }
}
