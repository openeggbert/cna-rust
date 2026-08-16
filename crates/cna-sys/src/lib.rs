//! Raw Rust declarations for CNA's stable C ABI.
//!
//! The upstream ABI has not shipped yet, so this crate deliberately contains
//! no guessed layouts, constants, function signatures, or linker directives.
//! Those declarations will be generated or audited from CNA's canonical C
//! headers once they exist.

#![no_std]

/// Whether canonical native declarations are present in this crate.
pub const BINDINGS_AVAILABLE: bool = false;
