//! Audited dynamic C ABI boundary, partitioned by native concern.

mod api;
mod error;
#[cfg(feature = "native-fault-injection")]
mod fault;
mod game;
mod graphics;
mod input;
mod loader;

pub(crate) use api::Native;
