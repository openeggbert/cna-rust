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
