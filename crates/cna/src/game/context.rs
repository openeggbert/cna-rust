#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use std::sync::Arc;

use cna_sys as sys;

use crate::error::Result;
use crate::graphics::GraphicsDevice;
use crate::native::Native;

/// Callback-scoped access to the host-owned XNA game state.
///
/// The context cannot escape a lifecycle callback. Its [`GraphicsDevice`]
/// result can: it is a stable shared Rust identity whose transient CNA handle
/// is rebound by the host at each callback boundary.
pub struct GameContext<'callback> {
    pub(crate) native: &'callback Arc<Native>,
    pub(crate) handle: sys::CNA_Handle,
    pub(crate) device: &'callback GraphicsDevice,
}

#[allow(non_snake_case)]
impl GameContext<'_> {
    pub fn GraphicsDevice(&self) -> Result<GraphicsDevice> {
        if self.device.IsDisposed()? {
            Err(crate::error::CnaError::InvalidInput(
                "graphics device is disposed",
            ))
        } else {
            Ok(self.device.clone())
        }
    }

    pub fn Exit(&self) -> Result<()> {
        self.native.request_game_exit(self.handle)
    }
}
