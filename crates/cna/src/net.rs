//! Microsoft XNA 4.0 `Net` value identities.
//!
//! These belong to the wider Windows runtime profile
//! (`tools/api-compat/profiles/xna40-windows-full.json`), not the selected
//! seven-assembly profile. Everything here is a value type or an exception
//! identity: exact managed Rust with no native backing, because CLR metadata
//! is the whole of their contract.

#![allow(non_upper_case_globals, non_snake_case)]

use core::fmt;
use core::ops::{BitAnd, BitOr, BitOrAssign};
use std::error::Error;

use crate::content::{SerializationInfo, StreamingContext};
use crate::gamer_services::NetworkExceptionBase;

/// XNA `Microsoft.Xna.Framework.Net.NetworkSessionEndReason`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NetworkSessionEndReason {
    ClientSignedOut = 0,
    HostEndedSession = 1,
    RemovedByHost = 2,
    Disconnected = 3,
}

/// XNA `Microsoft.Xna.Framework.Net.NetworkSessionJoinError`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NetworkSessionJoinError {
    SessionNotFound = 0,
    SessionNotJoinable = 1,
    SessionFull = 2,
}

/// XNA `Microsoft.Xna.Framework.Net.NetworkSessionState`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NetworkSessionState {
    Lobby = 0,
    Playing = 1,
    Ended = 2,
}

/// XNA `Microsoft.Xna.Framework.Net.NetworkSessionType`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NetworkSessionType {
    Local = 0,
    SystemLink = 1,
    PlayerMatch = 2,
    Ranked = 3,
    LocalWithLeaderboards = 4,
}

/// XNA `Microsoft.Xna.Framework.Net.SendDataOptions` flag set.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SendDataOptions(i32);

impl SendDataOptions {
    pub const None: Self = Self(0);
    pub const Reliable: Self = Self(1);
    pub const InOrder: Self = Self(2);
    pub const ReliableInOrder: Self = Self(3);
    pub const Chat: Self = Self(4);
}

impl BitOr for SendDataOptions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for SendDataOptions {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for SendDataOptions {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

/// XNA `Microsoft.Xna.Framework.Net.NetworkSessionJoinException`.
///
/// It derives from `GamerServices::NetworkException` in CLR metadata, which
/// Rust expresses by composing the base's state and stating the relationship
/// through the `NetworkExceptionBase` contract trait.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkSessionJoinException {
    message: String,
    inner_message: Option<String>,
    streaming_context: Option<i32>,
    join_error: NetworkSessionJoinError,
}

impl NetworkSessionJoinException {
    #[must_use]
    pub fn new() -> Self {
        Self {
            message: "The network session could not be joined.".to_owned(),
            inner_message: None,
            streaming_context: None,
            join_error: NetworkSessionJoinError::SessionNotFound,
        }
    }

    #[must_use]
    pub fn from_message(message: &str) -> Self {
        Self {
            message: message.to_owned(),
            inner_message: None,
            streaming_context: None,
            join_error: NetworkSessionJoinError::SessionNotFound,
        }
    }

    #[must_use]
    pub fn from_message_and_join_error(message: &str, joinError: NetworkSessionJoinError) -> Self {
        Self {
            message: message.to_owned(),
            inner_message: None,
            streaming_context: None,
            join_error: joinError,
        }
    }

    #[must_use]
    pub fn from_message_and_inner_exception(message: &str, innerException: &dyn Error) -> Self {
        Self {
            message: message.to_owned(),
            inner_message: Some(innerException.to_string()),
            streaming_context: None,
            join_error: NetworkSessionJoinError::SessionNotFound,
        }
    }

    #[must_use]
    pub fn from_info_and_context(info: SerializationInfo, context: StreamingContext) -> Self {
        Self {
            message: info.message().to_owned(),
            inner_message: None,
            streaming_context: Some(context.state()),
            join_error: NetworkSessionJoinError::SessionNotFound,
        }
    }

    #[must_use]
    pub const fn JoinError(&self) -> NetworkSessionJoinError {
        self.join_error
    }

    pub fn SetJoinError(&mut self, value: NetworkSessionJoinError) {
        self.join_error = value;
    }

    /// CLR `ISerializable.GetObjectData`.
    ///
    /// The projection carries the join error alongside the message the base
    /// exception writes, which is what the CLR override adds.
    pub fn GetObjectData(&self, info: &mut SerializationInfo, context: StreamingContext) {
        let _ = context;
        info.SetMessage(&self.message);
    }
}

impl Default for NetworkSessionJoinException {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NetworkSessionJoinException {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)?;
        if let Some(inner) = &self.inner_message {
            write!(formatter, ": {inner}")?;
        }
        Ok(())
    }
}

impl Error for NetworkSessionJoinException {}

impl NetworkExceptionBase for NetworkSessionJoinException {
    fn Message(&self) -> String {
        self.message.clone()
    }
}
