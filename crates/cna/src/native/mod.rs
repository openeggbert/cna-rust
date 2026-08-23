//! Audited dynamic C ABI boundary, partitioned by native concern.

mod api;
mod audio;
mod device_manager;
mod display;
mod error;
#[cfg(feature = "native-fault-injection")]
mod fault;
mod game;
mod graphics;
mod input;
mod loader;
mod storage;
mod window;

pub(crate) use api::Native;
pub(crate) use device_manager::NativeGraphicsPreferences;
pub(crate) use graphics::{
    BasicBoolProperty, BasicFloatProperty, BasicVector3Property, StockEffectKind,
    StockMatrixProperty,
};
