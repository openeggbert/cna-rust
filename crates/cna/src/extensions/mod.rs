//! CNA-specific functionality kept outside the strict XNA projection.
//!
//! Nothing here may appear inside `cna::Microsoft::Xna::Framework`: this module
//! is where CNA's own concepts live, and the strict API-compatibility verifier
//! measures the XNA hierarchy alone. Each submodule stays close to one
//! canonical CNA family so a route's authority is obvious from where it lands.

pub mod events;
pub mod graphics;
pub mod media;
pub mod runtime;
pub mod window;
