//! `storage.h`'s process-level settings: what the save directory is called and
//! where it is.
//!
//! XNA decides both and tells a game neither. The directory is named after the
//! title and lives wherever the platform puts per-title data, and a game that
//! wants to know -- or a tool that wants to read a game's saves, or a test that
//! wants them somewhere disposable -- has no route to either.
//!
//! Both are process-wide and are best set before the first `StorageDevice` is
//! opened: a container already opened keeps the path it was opened with.

use crate::error::Result;
use crate::native::Native;

/// Sets the directory name saves are stored under.
pub fn set_app_name(name: &str) -> Result<()> {
    Native::process()?.set_storage_app_name(name)
}

/// The root directory saves are stored under.
pub fn root() -> Result<String> {
    Native::process()?.storage_root()
}

/// The `storage.h` container facts CNA holds that Rust does not.
///
/// `NativeIsDisposed` is the storage family's counterpart to
/// [`crate::extensions::audio_ext::NativeDisposalState`]: it asks CNA what
/// state the container is in, where the strict projection's `IsDisposed`
/// answers from Rust's own record. They differ when the device goes away
/// underneath a container the Rust value still holds.
///
/// A CNA extension: import it to call these.
///
/// ```rust,ignore
/// use cna::extensions::storage_ext::StorageContainerExt;
/// if container.NativeIsDisposed()? { /* the device went away */ }
/// ```
pub trait StorageContainerExt {
    /// Whether CNA considers the container disposed.
    ///
    /// A different question from `StorageContainer::IsDisposed`, which asks
    /// whether Rust released it. They differ when the device goes away
    /// underneath a container the Rust value still holds.
    fn NativeIsDisposed(&self) -> Result<bool>;

    /// Whether CNA still associates this container with a storage device.
    ///
    /// `StorageDevice` is already reachable through the Rust value; this asks
    /// CNA, and answers `false` for a container whose device it has dropped --
    /// the state that makes every subsequent file operation fail, and the one
    /// a caller otherwise learns about only from that failure.
    fn HasNativeStorageDevice(&self) -> Result<bool>;
}
