//! Projection of CLR `System.IDisposable` for types that inherit it.
//!
//! Most XNA types declare `Dispose` themselves, and their projection declares
//! it too. A few inherit disposability from a BCL base -- `PacketReader` and
//! `PacketWriter` from `BinaryReader` and `BinaryWriter` -- and declare
//! nothing. Giving those an inherent `Dispose` would add a member Microsoft
//! did not declare, so the contract arrives through this trait instead.
//!
//! `Drop` is Rust lifetime safety and never replaces `Dispose`: the observable
//! release is `Dispose`, and it is idempotent, so calling it before `Drop`
//! changes nothing.

#![allow(non_snake_case)]

/// A type whose CLR counterpart inherits `System.IDisposable`.
pub trait Disposable {
    /// Releases what the value owns. Repeating it is a no-op.
    fn Dispose(&mut self);
}
