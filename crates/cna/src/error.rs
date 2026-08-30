use core::fmt;
use std::path::PathBuf;

use cna_sys as sys;

/// The stable category CNA attaches to a failed call.
///
/// It is the coarse identity behind a result code, and it is what lets the
/// binding tell, for example, a refused argument from a stale handle without
/// parsing an English message. Where an XNA member has a matching exception
/// identity, this is the evidence for choosing it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ErrorCategory {
    /// CNA reported no category, including when the diagnostic is unavailable.
    None,
    Argument,
    Handle,
    State,
    Memory,
    Io,
    NotSupported,
    Platform,
    Thread,
    Callback,
    Range,
    Encoding,
    Internal,
    ShuttingDown,
    /// A category a newer CNA introduced.
    Unrecognized(u32),
}

impl ErrorCategory {
    pub(crate) const fn from_native(value: sys::CNA_ErrorCategory) -> Self {
        match value {
            sys::CNA_ERROR_CATEGORY_NONE => Self::None,
            sys::CNA_ERROR_CATEGORY_ARGUMENT => Self::Argument,
            sys::CNA_ERROR_CATEGORY_HANDLE => Self::Handle,
            sys::CNA_ERROR_CATEGORY_STATE => Self::State,
            sys::CNA_ERROR_CATEGORY_MEMORY => Self::Memory,
            sys::CNA_ERROR_CATEGORY_IO => Self::Io,
            sys::CNA_ERROR_CATEGORY_NOT_SUPPORTED => Self::NotSupported,
            sys::CNA_ERROR_CATEGORY_PLATFORM => Self::Platform,
            sys::CNA_ERROR_CATEGORY_THREAD => Self::Thread,
            sys::CNA_ERROR_CATEGORY_CALLBACK => Self::Callback,
            sys::CNA_ERROR_CATEGORY_RANGE => Self::Range,
            sys::CNA_ERROR_CATEGORY_ENCODING => Self::Encoding,
            sys::CNA_ERROR_CATEGORY_INTERNAL => Self::Internal,
            sys::CNA_ERROR_CATEGORY_SHUTTING_DOWN => Self::ShuttingDown,
            other => Self::Unrecognized(other),
        }
    }
}

/// A failure reported by the safe CNA wrapper.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CnaError {
    /// No compatible CNA shared library could be opened.
    NativeUnavailable {
        /// Candidate paths/names attempted in discovery order.
        searched: Vec<PathBuf>,
        /// Platform loader diagnostics.
        details: String,
    },
    /// The library ABI is not admissible for this reviewed binding.
    ///
    /// `reason` names the canonical rule from CNA's ABI versioning contract
    /// that the library's reported version violates.
    AbiVersionMismatch {
        expected: u32,
        actual: u32,
        reason: &'static str,
    },
    /// A required C ABI symbol is absent from the selected library.
    MissingSymbol(&'static str),
    /// CNA returned a non-success result.
    ///
    /// `category` is CNA's own stable classification of the failure, read from
    /// the same thread-local diagnostic as `message`. It exists so a caller
    /// can act on the kind of failure without parsing prose.
    Native {
        code: u32,
        category: ErrorCategory,
        message: String,
    },
    /// A Rust lifecycle callback returned an error or panicked.
    Callback(String),
    /// An input cannot be represented by the native ABI.
    InvalidInput(&'static str),
    /// A platform has no implemented dynamic-library loader yet.
    UnsupportedPlatform,
    /// CNA's selected ABI does not expose the requested runtime route.
    UnsupportedRuntime(&'static str),
    /// A mapped XNA file/stream operation failed.
    Io(String),
    /// XNA content loading or XNB decoding failed.
    Content(crate::content::ContentLoadException),
}

impl fmt::Display for CnaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NativeUnavailable { searched, details } => write!(
                formatter,
                "CNA native library unavailable; set CNA_NATIVE_LIBRARY to the exact library path or CNA_NATIVE_DIR to its directory; searched {searched:?}: {details}"
            ),
            Self::AbiVersionMismatch {
                expected,
                actual,
                reason,
            } => write!(
                formatter,
                "CNA ABI version mismatch: Rust declarations were reviewed against {}.{}.{}, library reports {}.{}.{} ({reason})",
                cna_sys::cna_abi_version_major(*expected),
                cna_sys::cna_abi_version_minor(*expected),
                cna_sys::cna_abi_version_patch(*expected),
                cna_sys::cna_abi_version_major(*actual),
                cna_sys::cna_abi_version_minor(*actual),
                cna_sys::cna_abi_version_patch(*actual),
            ),
            Self::MissingSymbol(symbol) => write!(formatter, "CNA library is missing required symbol {symbol}"),
            Self::Native {
                code,
                category,
                message,
            } => write!(formatter, "CNA error {code} ({category:?}): {message}"),
            Self::Callback(message) => write!(formatter, "game callback failed: {message}"),
            Self::InvalidInput(message) | Self::UnsupportedRuntime(message) => {
                formatter.write_str(message)
            }
            Self::Io(message) => formatter.write_str(message),
            Self::Content(error) => error.fmt(formatter),
            Self::UnsupportedPlatform => formatter.write_str("CNA dynamic loading is not implemented on this platform"),
        }
    }
}

impl std::error::Error for CnaError {}

impl From<crate::content::ContentLoadException> for CnaError {
    fn from(value: crate::content::ContentLoadException) -> Self {
        Self::Content(value)
    }
}

/// Result type used by the safe CNA API.
pub type Result<T> = core::result::Result<T, CnaError>;
