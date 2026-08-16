//! Safe Rust frontend for the native CNA game framework.
//!
//! This is an early scaffold. Local values are usable now; native execution
//! will be implemented after CNA publishes its stable C ABI.

#![forbid(unsafe_code)]

mod error;
mod game;
mod value;

pub use error::{CnaError, Result};
pub use game::{run, Game, GameContext, GameTime};
pub use value::{Color, Vector2};

/// Returns whether the raw crate contains canonical CNA ABI declarations.
#[must_use]
pub const fn native_bindings_available() -> bool {
    cna_sys::BINDINGS_AVAILABLE
}
