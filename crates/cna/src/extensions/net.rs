//! CNA-only access to the packet buffers XNA keeps internal.
//!
//! XNA's `NetworkSession` reaches `PacketWriter`'s bytes through an `internal`
//! member and fills a `PacketReader` the same way, so a game never touches
//! either. CNA has no session yet, and without these two routes the packet
//! types would be a write-only sink and a permanently empty source. They are
//! CNA additions and live here rather than in the strict XNA hierarchy.

#![allow(non_snake_case)]

use crate::net::{PacketReader, PacketWriter};

/// The bytes a `PacketWriter` has produced.
#[must_use]
pub fn PacketBytes(writer: &PacketWriter) -> &[u8] {
    writer.bytes()
}

/// Fills a `PacketReader` with one received packet and rewinds it.
pub fn FillPacket(reader: &mut PacketReader, bytes: &[u8]) {
    reader.fill(bytes);
}
