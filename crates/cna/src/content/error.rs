#![allow(non_snake_case)]

use core::fmt;
use std::error::Error;

/// Portable subset of CLR serialization information retained by the mapped
/// exception constructor.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SerializationInfo {
    pub(crate) message: String,
}

impl SerializationInfo {
    #[must_use]
    pub fn from_message(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    /// Replaces the serialized message, as a CLR `GetObjectData` override does.
    pub fn SetMessage(&mut self, value: &str) {
        self.message = value.to_owned();
    }
}

/// Portable subset of CLR streaming context retained by the mapped exception
/// constructor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StreamingContext {
    pub(crate) state: i32,
}

impl StreamingContext {
    #[must_use]
    pub const fn from_state(state: i32) -> Self {
        Self { state }
    }

    pub(crate) const fn state(self) -> i32 {
        self.state
    }
}

/// Error raised when an XNB asset cannot be opened or decoded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentLoadException {
    message: String,
    inner_message: Option<String>,
    streaming_context: Option<i32>,
}

impl ContentLoadException {
    #[must_use]
    pub fn new() -> Self {
        Self {
            message: "Error loading content".to_owned(),
            inner_message: None,
            streaming_context: None,
        }
    }

    #[must_use]
    pub fn from_message(message: &str) -> Self {
        Self {
            message: message.to_owned(),
            inner_message: None,
            streaming_context: None,
        }
    }

    #[must_use]
    pub fn from_info_and_context(info: SerializationInfo, context: StreamingContext) -> Self {
        Self {
            message: info.message,
            inner_message: None,
            streaming_context: Some(context.state),
        }
    }

    #[must_use]
    pub fn from_message_and_inner_exception(message: &str, innerException: &dyn Error) -> Self {
        Self {
            message: message.to_owned(),
            inner_message: Some(innerException.to_string()),
            streaming_context: None,
        }
    }

    pub(crate) fn with_inner_message(message: impl Into<String>, inner: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            inner_message: Some(inner.into()),
            streaming_context: None,
        }
    }
}

impl Default for ContentLoadException {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ContentLoadException {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)?;
        if let Some(inner) = &self.inner_message {
            write!(formatter, ": {inner}")?;
        }
        Ok(())
    }
}

impl Error for ContentLoadException {}
