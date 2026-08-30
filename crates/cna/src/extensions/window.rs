//! Opaque native window identity.

/// Opaque native window identity. It cannot be dereferenced or forged
/// through CNA-Rust's safe public API.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct WindowHandle(pub(crate) u64);
