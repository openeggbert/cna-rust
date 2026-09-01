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
use crate::game::TimeSpan;
use crate::net::{
    AvailableNetworkSession, AvailableNetworkSessionCollection, NetworkGamer, NetworkSession,
    NetworkSessionEndReason, NetworkSessionJoinError, NetworkSessionProperties,
    NetworkSessionState, NetworkSessionType, SendDataOptions,
};

/// Writes a mutated property list back onto a live session.
///
/// XNA's `NetworkSession.SessionProperties` is a **reference**: a game reads
/// it once, assigns a slot, and the session it came from is what changed.
/// A `CNA_Handle` cannot carry that reference across the ABI without letting a
/// caller keep a pointer into session state, so CNA's C API publishes a copy
/// and, since `CABI-49`, a route that applies one back. This is that route,
/// and it is the second half of XNA's assignment:
///
/// ```text
/// XNA     session.SessionProperties[0] = 5;
/// here    let mut p = session.SessionProperties()?;
///         p.SetItem(0, Some(5));
///         ApplySessionProperties(&session, &p)?;
/// ```
///
/// It lives outside `cna::Microsoft::Xna::Framework` because the two-step is
/// not XNA's shape. The strict getter still answers what the session holds,
/// and it is now something a game can change rather than a copy whose writes
/// went nowhere.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn ApplySessionProperties(
    session: &NetworkSession,
    properties: &NetworkSessionProperties,
) -> Result<()> {
    let runtime = crate::net::net_runtime()?;
    let handle = crate::net::session_handle(session)?;
    let published = crate::net::write_properties(&runtime, properties)?;
    // SAFETY: both handles are live, and CNA copies the values out of the
    // list rather than retaining it.
    let outcome = runtime.check(unsafe {
        (runtime
            .native()
            .net
            .network_session_replace_session_properties)(handle, published)
    });
    // The list is this call's to release whatever happened.
    // SAFETY: it came from `write_properties` and is released once.
    let _ = unsafe { (runtime.native().net.network_session_properties_destroy)(published) };
    outcome
}

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

    /// Drops every packet still queued on a local gamer.
    ///
    /// XNA has no such member: a game drains its queue by receiving from it,
    /// and a session that disposes takes the queue with it. CNA publishes the
    /// clear its own session uses at teardown, and a test that injected a
    /// packet and wants to start over needs it -- otherwise the next
    /// `ReceiveData` answers the previous test's bytes.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn clear_packets(gamer: &crate::net::LocalNetworkGamer) -> Result<()> {
        let runtime = crate::net::net_runtime()?;
        let handle = crate::net::local_gamer_handle(gamer)?;
        // SAFETY: the handle names a live local gamer.
        runtime.check(unsafe {
            (runtime
                .native()
                .net
                .local_network_gamer_clear_packet_queue_ext)(handle)
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

/// The join error the last failed join carried on this thread.
///
/// XNA raises `NetworkSessionJoinException`, and the reason a join failed --
/// not found, not joinable, full -- is a property on that exception object. An
/// exception object never crosses a C ABI, so CNA records the value per thread
/// beside the usual result and message, and this is where a Rust caller reads
/// it.
///
/// Answers `None` when the last failure on this thread was not a join failure,
/// or carried no join error. Reading does not clear it: a second read of the
/// same failure answers the same thing.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn LastJoinError() -> Result<Option<NetworkSessionJoinError>> {
    let runtime = crate::net::net_runtime()?;
    let (mut value, mut present) = (0, 0);
    // SAFETY: both outputs are initialized and the route is thread-local.
    runtime.check(unsafe {
        (runtime.native().net.net_get_last_join_error)(&mut value, &mut present)
    })?;
    if present == 0 {
        return Ok(None);
    }
    match value {
        sys::CNA_NETWORK_SESSION_JOIN_ERROR_SESSION_NOT_FOUND => {
            Ok(Some(NetworkSessionJoinError::SessionNotFound))
        }
        sys::CNA_NETWORK_SESSION_JOIN_ERROR_SESSION_NOT_JOINABLE => {
            Ok(Some(NetworkSessionJoinError::SessionNotJoinable))
        }
        sys::CNA_NETWORK_SESSION_JOIN_ERROR_SESSION_FULL => {
            Ok(Some(NetworkSessionJoinError::SessionFull))
        }
        _ => Err(CnaError::InvalidInput(
            "CNA reported a join error XNA does not declare",
        )),
    }
}

/// How many `NetworkSession` objects and pending creations CNA holds.
///
/// Both are process-wide counters CNA keeps for its own leak checks. They are
/// here for the same reason [`RemoteGamerInjection::owned_gamer_count`] is: a
/// test that disposes a session can assert the count went back down, which is
/// the difference between "Dispose returned Ok" and "Dispose released
/// something".
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn LiveSessionCount() -> Result<i32> {
    let runtime = crate::net::net_runtime()?;
    let mut value = 0;
    // SAFETY: the output is initialized and the route is process-global.
    runtime
        .check(unsafe { (runtime.native().net.network_session_get_instance_count_ext)(&mut value) })?;
    Ok(value)
}

/// How many session creations CNA has begun and not finished.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn PendingSessionActionCount() -> Result<i32> {
    let runtime = crate::net::net_runtime()?;
    let mut value = 0;
    // SAFETY: the output is initialized and the route is process-global.
    runtime.check(unsafe {
        (runtime.native().net.network_session_get_active_action_count_ext)(&mut value)
    })?;
    Ok(value)
}

/// What a host layer knows about a session it discovered.
///
/// Every field is what a real search would have reported; nothing here is
/// optional, because a discovered session with an invented gamertag or an
/// invented slot count is exactly the fabrication the strict surface refuses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredSession {
    /// The host's gamertag, copied during creation.
    pub host_gamertag: String,
    /// The address the host accepts connections on, copied during creation.
    pub host_address: String,
    /// The port the host accepts connections on.
    pub host_port: u16,
    /// How many gamers are already in the session.
    pub current_gamer_count: i32,
    /// How many public slots are unoccupied.
    pub open_public_gamer_slots: i32,
    /// How many private slots are unoccupied.
    pub open_private_gamer_slots: i32,
    /// The session type the host advertises.
    pub session_type: NetworkSessionType,
    /// The properties the host advertises.
    pub session_properties: NetworkSessionProperties,
    /// The measured round-trip to the host, or `None` for an unmeasured link.
    pub roundtrip: Option<TimeSpan>,
}

/// Discovered sessions a host supplies in place of a search.
///
/// `NetworkSession::Find` on one machine finds nothing, because there is
/// nothing on the network to find. That leaves `AvailableNetworkSession` -- a
/// whole XNA type with six properties -- unreachable, and its connect address,
/// port and advertised type unreachable with it. CNA publishes `_ext` routes
/// that build one, and this is how a host layer uses them.
///
/// Nothing in the strict projection reaches these routes, and nothing in it
/// invents a discovered session when they are not used: `Find` still answers
/// an empty collection.
pub struct DiscoveredSessionInjection;

impl DiscoveredSessionInjection {
    /// Builds one discovered session.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn session(session: &DiscoveredSession) -> Result<AvailableNetworkSession> {
        let runtime = crate::net::net_runtime()?;
        let handle = Self::create(&runtime, session)?;
        Ok(AvailableNetworkSession::adopt(runtime, handle))
    }

    /// Builds the read-only collection `Find` would have answered.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn collection(sessions: &[DiscoveredSession]) -> Result<AvailableNetworkSessionCollection> {
        let runtime = crate::net::net_runtime()?;
        let mut handles = Vec::with_capacity(sessions.len());
        for session in sessions {
            match Self::create(&runtime, session) {
                Ok(handle) => handles.push(handle),
                Err(error) => {
                    Self::release(&runtime, &handles);
                    return Err(error);
                }
            }
        }
        let count = u64::try_from(handles.len())
            .map_err(|_| CnaError::InvalidInput("too many discovered sessions"))?;
        let pointer = if handles.is_empty() {
            core::ptr::null()
        } else {
            handles.as_ptr()
        };
        let mut collection = 0;
        // SAFETY: the array describes exactly `count` live handles, and the
        // collection takes ownership of each on success.
        let created = runtime.check(unsafe {
            (runtime
                .native()
                .net
                .available_network_session_collection_create_ext)(
                pointer, count, &mut collection
            )
        });
        if let Err(error) = created {
            Self::release(&runtime, &handles);
            return Err(error);
        }
        Ok(AvailableNetworkSessionCollection::adopt(
            runtime, collection,
        ))
    }

    fn create(
        runtime: &crate::gamer_services::GamerServicesRuntimeHandle,
        session: &DiscoveredSession,
    ) -> Result<sys::CNA_Handle> {
        let gamertag = crate::gamer_services::borrow_string(&session.host_gamertag)?;
        let address = crate::gamer_services::borrow_string(&session.host_address)?;
        let properties = crate::net::write_properties(runtime, &session.session_properties)?;
        let info = sys::CNA_AvailableNetworkSessionCreateInfo {
            struct_size: core::mem::size_of::<sys::CNA_AvailableNetworkSessionCreateInfo>() as u32,
            struct_version: 1,
            current_gamer_count: session.current_gamer_count,
            open_private_gamer_slots: session.open_private_gamer_slots,
            open_public_gamer_slots: session.open_public_gamer_slots,
            session_type: session.session_type as i32 as u32,
            host_port: session.host_port,
            reserved: [0; 6],
            host_gamertag: gamertag.value,
            host_address: address.value,
            session_properties: properties,
        };
        let quality = session.roundtrip.map_or(
            sys::CNA_QualityOfService {
                struct_size: core::mem::size_of::<sys::CNA_QualityOfService>() as u32,
                struct_version: 1,
                ..sys::CNA_QualityOfService::default()
            },
            |roundtrip| sys::CNA_QualityOfService {
                struct_size: core::mem::size_of::<sys::CNA_QualityOfService>() as u32,
                struct_version: 1,
                is_available: sys::CNA_TRUE,
                reserved: [0; 7],
                average_roundtrip_ticks: roundtrip.Ticks(),
                minimum_roundtrip_ticks: roundtrip.Ticks(),
                bytes_per_second_downstream: 0,
                bytes_per_second_upstream: 0,
            },
        );
        let mut handle = 0;
        // SAFETY: both views borrow for the call, CNA copies their bytes, and
        // the properties handle is copied rather than retained.
        let created = runtime.check(unsafe {
            (runtime.native().net.available_network_session_create_ext)(
                &info,
                &quality,
                &mut handle,
            )
        });
        // The property list is this call's to release whatever happened.
        // SAFETY: it came from `write_properties` and is released once.
        let _ = unsafe { (runtime.native().net.network_session_properties_destroy)(properties) };
        created?;
        Ok(handle)
    }

    fn release(
        runtime: &crate::gamer_services::GamerServicesRuntimeHandle,
        handles: &[sys::CNA_Handle],
    ) {
        for handle in handles {
            // SAFETY: each came from the create route above and is released once.
            let _ = unsafe { (runtime.native().net.available_network_session_destroy)(*handle) };
        }
    }
}

/// CNA's own facts about a discovered session, beyond XNA's six properties.
///
/// XNA's `AvailableNetworkSession` never says *where* the session is: the
/// address and port are inside the framework, and `Join` is the only thing
/// that uses them. CNA publishes all three, and a host layer that has to make
/// its own connection decision needs them.
pub trait DiscoveredSessionExt {
    /// The address the host accepts connections on.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn connect_address(&self) -> Result<String>;

    /// The port the host accepts connections on.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn connect_port(&self) -> Result<u16>;

    /// The session type the host advertises.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn advertised_session_type(&self) -> Result<NetworkSessionType>;

    /// Whether CNA considers two discovered sessions the same one.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn is_same_session(&self, other: &Self) -> Result<bool>;
}

impl DiscoveredSessionExt for AvailableNetworkSession {
    fn connect_address(&self) -> Result<String> {
        let runtime = crate::net::net_runtime()?;
        let handle = crate::net::available_session_handle(self)?;
        let api = &runtime.native().net;
        let (size, copy) = (
            api.available_network_session_get_connect_address_size_ext,
            api.available_network_session_copy_connect_address_ext,
        );
        crate::native::runtime::read_string(
            |value| runtime.check(value),
            // SAFETY: the handle is live for the size query.
            |bytes| unsafe { size(handle, bytes) },
            // SAFETY: the destination has the reported capacity.
            |destination, capacity, written| unsafe {
                copy(handle, destination, capacity, written)
            },
        )
    }

    fn connect_port(&self) -> Result<u16> {
        let runtime = crate::net::net_runtime()?;
        let handle = crate::net::available_session_handle(self)?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        runtime.check(unsafe {
            (runtime
                .native()
                .net
                .available_network_session_get_connect_port_ext)(handle, &mut value)
        })?;
        Ok(value)
    }

    fn advertised_session_type(&self) -> Result<NetworkSessionType> {
        let runtime = crate::net::net_runtime()?;
        let handle = crate::net::available_session_handle(self)?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        runtime.check(unsafe {
            (runtime
                .native()
                .net
                .available_network_session_get_session_type_ext)(handle, &mut value)
        })?;
        NetworkSessionType::from_native(value).ok_or(CnaError::InvalidInput(
            "CNA reported a session type XNA does not declare",
        ))
    }

    fn is_same_session(&self, other: &Self) -> Result<bool> {
        let runtime = crate::net::net_runtime()?;
        let (left, right) = (
            crate::net::available_session_handle(self)?,
            crate::net::available_session_handle(other)?,
        );
        let (mut equal, mut different) = (0, 0);
        // SAFETY: both handles are live and both outputs are initialized.
        runtime.check(unsafe {
            (runtime.native().net.available_network_session_equals)(left, right, &mut equal)
        })?;
        // CNA publishes both halves of the comparison, and a projection that
        // called only one would not notice them disagreeing.
        runtime.check(unsafe {
            (runtime.native().net.available_network_session_not_equals)(
                left,
                right,
                &mut different,
            )
        })?;
        if (equal != 0) == (different != 0) {
            return Err(CnaError::InvalidInput(
                "CNA's equality and inequality disagree about two discovered sessions",
            ));
        }
        Ok(equal != 0)
    }
}
