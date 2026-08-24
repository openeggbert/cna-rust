#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use std::sync::Arc;

use cna_sys as sys;

use crate::error::Result;
use crate::audio::AudioRuntime;
use crate::graphics::GraphicsDevice;
use crate::native::Native;
use crate::media::MediaRuntime;

/// Callback-scoped access to the host-owned XNA game state.
///
/// The context cannot escape a lifecycle callback. Its [`GraphicsDevice`]
/// result can: it is a stable shared Rust identity whose transient CNA handle
/// is rebound by the host at each callback boundary.
pub struct GameContext<'callback> {
    pub(crate) native: &'callback Arc<Native>,
    pub(crate) handle: sys::CNA_Handle,
    pub(crate) device: &'callback GraphicsDevice,
    pub(crate) audio: &'callback Arc<AudioRuntime>,
    pub(crate) media: &'callback Arc<MediaRuntime>,
    pub(crate) media_generation: u64,
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

    pub(crate) fn audio_runtime(&self) -> &Arc<AudioRuntime> {
        self.audio
    }

    pub(crate) fn native_game(&self) -> (&Arc<Native>, sys::CNA_Handle) {
        (self.native, self.handle)
    }

    pub(crate) fn media_runtime(&self) -> &Arc<MediaRuntime> {
        self.media
    }

    pub(crate) fn media_generation(&self) -> u64 {
        self.media_generation
    }
}
