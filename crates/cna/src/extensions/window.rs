//! Opaque native window identity, and the window controls that reach past XNA.

use crate::error::Result;
use crate::Microsoft::Xna::Framework::GameContext;

/// Opaque native window identity. It cannot be dereferenced or forged
/// through CNA-Rust's safe public API.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct WindowHandle(pub(crate) u64);

/// The window controls `runtime_window.h` adds beyond XNA.
///
/// XNA's `GameWindow` has a title, a client bounds and an allow-resizing flag,
/// and nothing else: no borderless, no way to minimize but the user's, and no
/// route to the platform's own handle. These are all three.
#[derive(Clone, Copy, Debug)]
pub struct NativeWindow;

impl NativeWindow {
    /// Whether the window is borderless.
    pub fn is_borderless(game: &GameContext<'_>) -> Result<bool> {
        game.native.window_is_borderless(game.handle)
    }

    /// Turns the border on or off.
    pub fn set_borderless(game: &GameContext<'_>, borderless: bool) -> Result<()> {
        game.native.set_window_borderless(game.handle, borderless)
    }

    /// Minimizes the window.
    pub fn minimize(game: &GameContext<'_>) -> Result<()> {
        game.native.minimize_window(game.handle)
    }

    /// Restores the window from minimized.
    pub fn restore(game: &GameContext<'_>) -> Result<()> {
        game.native.restore_window(game.handle)
    }

    /// The platform's own window handles.
    ///
    /// **Every pointer in the result belongs to the platform.** None of it is
    /// the caller's to free, and all of it is valid only while the window is,
    /// which is why this answers the raw structure rather than something with a
    /// `Drop`. It is the escape hatch for embedding CNA in a larger
    /// application -- handing the window to an overlay, a debugger, or another
    /// toolkit -- and there is no safe way to do that without raw handles.
    ///
    /// `system` says which of the fields mean anything: on X11 it is `display`
    /// and `window_id`, on Wayland `display` and `surface`, on Win32 `window`.
    pub fn native_handles(game: &GameContext<'_>) -> Result<cna_sys::CNA_NativeWindowHandle> {
        game.native.native_window(game.handle)
    }
}
