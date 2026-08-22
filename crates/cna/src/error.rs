use core::fmt;
use std::path::PathBuf;

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
    /// The library ABI does not exactly match this experimental binding.
    AbiVersionMismatch { expected: u32, actual: u32 },
    /// A required C ABI symbol is absent from the selected library.
    MissingSymbol(&'static str),
    /// CNA returned a non-success result.
    Native { code: u32, message: String },
    /// A Rust lifecycle callback returned an error or panicked.
    Callback(String),
    /// An input cannot be represented by the native ABI.
    InvalidInput(&'static str),
    /// A platform has no implemented dynamic-library loader yet.
    UnsupportedPlatform,
}

impl fmt::Display for CnaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NativeUnavailable { searched, details } => write!(
                formatter,
                "CNA native library unavailable; set CNA_NATIVE_LIBRARY to the exact library path or CNA_NATIVE_DIR to its directory; searched {searched:?}: {details}"
            ),
            Self::AbiVersionMismatch { expected, actual } => write!(
                formatter,
                "CNA ABI version mismatch: Rust declarations require {expected:#010x}, library reports {actual:#010x}"
            ),
            Self::MissingSymbol(symbol) => write!(formatter, "CNA library is missing required symbol {symbol}"),
            Self::Native { code, message } => write!(formatter, "CNA error {code}: {message}"),
            Self::Callback(message) => write!(formatter, "game callback failed: {message}"),
            Self::InvalidInput(message) => formatter.write_str(message),
            Self::UnsupportedPlatform => formatter.write_str("CNA dynamic loading is not implemented on this platform"),
        }
    }
}

impl std::error::Error for CnaError {}

/// Result type used by the safe CNA API.
pub type Result<T> = core::result::Result<T, CnaError>;
