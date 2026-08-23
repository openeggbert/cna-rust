#![allow(non_snake_case)]

use core::fmt;
use std::error::Error;

use crate::extensions::events::EventHandler;

use super::GraphicsDevice;

/// Rust projection of XNA's graphics-device service contract.
pub trait IGraphicsDeviceService {
    fn GraphicsDevice(&self) -> &GraphicsDevice;
    fn AddDeviceCreatedHandler(&self, handler: Box<dyn EventHandler>) -> u64;
    fn RemoveDeviceCreatedHandler(&self, registration: u64) -> bool;
    fn AddDeviceDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64;
    fn RemoveDeviceDisposingHandler(&self, registration: u64) -> bool;
    fn AddDeviceResetHandler(&self, handler: Box<dyn EventHandler>) -> u64;
    fn RemoveDeviceResetHandler(&self, registration: u64) -> bool;
    fn AddDeviceResettingHandler(&self, handler: Box<dyn EventHandler>) -> u64;
    fn RemoveDeviceResettingHandler(&self, registration: u64) -> bool;
}

macro_rules! graphics_exception {
    ($name:ident, $default:literal) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name {
            message: String,
            inner_message: Option<String>,
        }

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self {
                    message: $default.to_owned(),
                    inner_message: None,
                }
            }

            #[must_use]
            pub fn from_message(message: &str) -> Self {
                Self {
                    message: message.to_owned(),
                    inner_message: None,
                }
            }

            #[must_use]
            pub fn from_message_and_inner(message: &str, inner: &dyn Error) -> Self {
                Self {
                    message: message.to_owned(),
                    inner_message: Some(inner.to_string()),
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.message)?;
                if let Some(inner) = &self.inner_message {
                    write!(formatter, ": {inner}")?;
                }
                Ok(())
            }
        }

        impl Error for $name {}
    };
}

graphics_exception!(
    DeviceLostException,
    "The graphics device has been lost and cannot currently be used."
);
graphics_exception!(
    DeviceNotResetException,
    "The graphics device has not been reset after it was lost."
);
graphics_exception!(
    NoSuitableGraphicsDeviceException,
    "No suitable graphics device was found."
);
