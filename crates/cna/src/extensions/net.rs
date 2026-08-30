//! CNA-only network capabilities, and the packet buffers XNA keeps internal.
//!
//! XNA's `NetworkSession` reaches `PacketWriter`'s bytes through an `internal`
//! member and fills a `PacketReader` the same way, so a game never touches
//! either. Those two routes are here because they are CNA additions, not XNA
//! members.
//!
//! The rest of this module is how a single process drives a session's
//! lifecycle without a peer. CNA publishes routes for injecting a remote gamer
//! and delivering a packet, and a game that calls them is standing in for the
//! network -- which is exactly why they are outside the strict hierarchy. What
//! the strict surface must never do is invent a peer; what a test or a
//! platform layer may do is supply one deliberately.

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


use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::net::{
    NetworkGamer, NetworkSession, NetworkSessionEndReason, NetworkSessionState, SendDataOptions,
};

/// A remote participant a host supplies in place of the network.
///
/// CNA's own `_ext` routes create and admit it. Nothing in the strict XNA
/// projection can reach them, and nothing in the strict projection invents a
/// gamer when they are not used: `RemoteGamers` stays empty.
pub struct RemoteGamerInjection;

impl RemoteGamerInjection {
    /// Creates a remote gamer for a session and admits it to the roster.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn admit(session: &NetworkSession, gamertag: &str) -> Result<NetworkGamer> {
        let runtime = crate::net::net_runtime()?;
        let handle = crate::net::session_handle(session)?;
        let view = crate::gamer_services::borrow_string(gamertag)?;
        let mut gamer = 0;
        // SAFETY: the view borrows the tag for the call and the session handle
        // is live.
        runtime.check(unsafe {
            (runtime.native().net.network_gamer_create)(handle, view.value, &mut gamer)
        })?;
        // SAFETY: the gamer was created for this session and is admitted once.
        runtime.check(unsafe {
            (runtime.native().net.network_session_add_remote_gamer_ext)(handle, gamer)
        })?;
        Ok(NetworkGamer::from_admitted(runtime, gamer))
    }

    /// Removes a gamer from a session with an explicit reason.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn remove(
        session: &NetworkSession,
        gamer: &NetworkGamer,
        reason: NetworkSessionEndReason,
    ) -> Result<()> {
        let runtime = crate::net::net_runtime()?;
        let handle = crate::net::session_handle(session)?;
        let target = crate::net::gamer_handle(gamer)?;
        // SAFETY: both handles are live and the reason is canonical.
        runtime.check(unsafe {
            (runtime.native().net.network_session_remove_gamer_ext)(handle, target, reason as u32)
        })
    }

    /// How many gamers the session owns, including ones it created itself.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn owned_gamer_count(session: &NetworkSession) -> Result<u64> {
        let runtime = crate::net::net_runtime()?;
        let handle = crate::net::session_handle(session)?;
        let mut count = 0;
        // SAFETY: the handle is live and the output is initialized.
        runtime.check(unsafe {
            (runtime.native().net.network_session_get_owned_gamer_count_ext)(handle, &mut count)
        })?;
        Ok(count)
    }
}

/// A network event a host delivers in place of the network.
///
/// CNA's session raises its own events from these, so a single process can
/// prove that a subscription fires, fires once, and stops firing after it is
/// removed -- none of which a lone process could otherwise observe.
pub struct NetworkEventInjection;

impl NetworkEventInjection {
    /// Delivers a state change to a session.
    ///
    /// The session raises `GameStarted`, `GameEnded` or `SessionEnded` from
    /// this on its next `Update`, exactly as it would from a real peer.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn state(session: &NetworkSession, state: NetworkSessionState) -> Result<()> {
        Self::send(session, |info| {
            info.r#type = sys::CNA_NETWORK_EVENT_TYPE_STATE_CHANGE;
            info.state = state as u32;
        })
    }

    /// Delivers a session-ended state change with an explicit reason.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ended(session: &NetworkSession, reason: NetworkSessionEndReason) -> Result<()> {
        Self::send(session, |info| {
            info.r#type = sys::CNA_NETWORK_EVENT_TYPE_STATE_CHANGE;
            info.state = NetworkSessionState::Ended as u32;
            info.reason = reason as u32;
        })
    }

    /// Delivers a gamer joining, which raises `GamerJoined` on the next update.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn joined(session: &NetworkSession, gamer: &NetworkGamer) -> Result<()> {
        let handle = crate::net::gamer_handle(gamer)?;
        Self::send(session, |info| {
            info.r#type = sys::CNA_NETWORK_EVENT_TYPE_GAMER_JOIN;
            info.gamer = handle;
        })
    }

    /// Delivers a gamer leaving, which raises `GamerLeft` on the next update.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn left(session: &NetworkSession, gamer: &NetworkGamer) -> Result<()> {
        let handle = crate::net::gamer_handle(gamer)?;
        Self::send(session, |info| {
            info.r#type = sys::CNA_NETWORK_EVENT_TYPE_GAMER_LEAVE;
            info.gamer = handle;
        })
    }

    /// Delivers a packet to a local gamer's receive queue.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn packet(
        gamer: &crate::net::LocalNetworkGamer,
        sender: &NetworkGamer,
        payload: &[u8],
        options: SendDataOptions,
    ) -> Result<()> {
        let runtime = crate::net::net_runtime()?;
        let handle = crate::net::local_gamer_handle(gamer)?;
        let from = crate::net::gamer_handle(sender)?;
        let count = u64::try_from(payload.len())
            .map_err(|_| CnaError::InvalidInput("the packet is too large"))?;
        let pointer = if payload.is_empty() {
            core::ptr::null()
        } else {
            payload.as_ptr()
        };
        let info = sys::CNA_NetworkEventInfo {
            struct_size: core::mem::size_of::<sys::CNA_NetworkEventInfo>() as u32,
            struct_version: 1,
            r#type: sys::CNA_NETWORK_EVENT_TYPE_PACKET_SEND,
            reliable: crate::net::send_data_bits(options),
            state: 0,
            reason: 0,
            gamer: 0,
            sender: from,
            packet: pointer,
            packet_byte_count: count,
        };
        // SAFETY: the payload describes exactly `count` readable bytes and is
        // copied during the call.
        runtime.check(unsafe {
            (runtime.native().net.local_network_gamer_enqueue_packet_ext)(handle, &info)
        })
    }

    fn send(
        session: &NetworkSession,
        fill: impl FnOnce(&mut sys::CNA_NetworkEventInfo),
    ) -> Result<()> {
        let runtime = crate::net::net_runtime()?;
        let handle = crate::net::session_handle(session)?;
        let mut info = sys::CNA_NetworkEventInfo {
            struct_size: core::mem::size_of::<sys::CNA_NetworkEventInfo>() as u32,
            struct_version: 1,
            r#type: sys::CNA_NETWORK_EVENT_TYPE_PACKET_SEND,
            reliable: 0,
            state: 0,
            reason: 0,
            gamer: 0,
            sender: 0,
            packet: core::ptr::null(),
            packet_byte_count: 0,
        };
        fill(&mut info);
        // SAFETY: the descriptor is versioned and outlives the call.
        runtime.check(unsafe {
            (runtime.native().net.network_session_send_network_event_ext)(handle, &info)
        })
    }
}

/// Keeps `Arc` referenced for the doc links above.
#[allow(dead_code)]
type Retained = Arc<()>;
