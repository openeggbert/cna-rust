use core::fmt;

/// A failure reported by the safe CNA wrapper.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CnaError {
    /// CNA has not published the native ABI required by this scaffold.
    NativeUnavailable,
}

impl fmt::Display for CnaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NativeUnavailable => formatter.write_str("CNA native C ABI is not available yet"),
        }
    }
}

impl std::error::Error for CnaError {}

/// Result type used by the safe CNA API.
pub type Result<T> = core::result::Result<T, CnaError>;
