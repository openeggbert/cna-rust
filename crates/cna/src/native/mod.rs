//! Audited dynamic C ABI boundary, partitioned by native concern.

mod abi;
mod api;
mod audio;
mod device_manager;
mod display;
pub(crate) mod engine;
mod error;
#[cfg(feature = "native-fault-injection")]
pub(crate) mod fault;
mod game;
pub(crate) mod gamer_services;
mod graphics;
mod input;
mod loader;
pub(crate) mod media;
pub(crate) mod net;
pub(crate) mod runtime;
mod storage;
mod window;

pub(crate) use api::Native;
pub(crate) use device_manager::NativeGraphicsPreferences;
pub(crate) use graphics::{
    BasicBoolProperty, BasicFloatProperty, BasicVector3Property, StockEffectKind,
    StockMatrixProperty,
};
