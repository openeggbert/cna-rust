//! CNA-specific functionality kept outside the strict XNA projection.
//!
//! Nothing here may appear inside `cna::Microsoft::Xna::Framework`: this module
//! is where CNA's own concepts live, and the strict API-compatibility verifier
//! measures the XNA hierarchy alone. Each submodule stays close to one
//! canonical CNA family so a route's authority is obvious from where it lands.

pub mod content;
pub mod devices;
pub mod effects;
pub mod engine;
pub mod events;
pub mod gamepad_ext;
pub mod gamer_services;
pub mod graphics;
pub mod graphics_device_ext;
pub mod haptics;
pub mod input;
pub mod input_devices;
pub mod logging;
pub mod media;
pub mod cnb;
pub mod models;
pub mod native_model;
pub mod net;
pub mod object_dictionary;
pub mod pbr;
pub mod runtime;
pub mod shader_effect;
pub mod sensors;
pub mod text_input;
pub mod window;
