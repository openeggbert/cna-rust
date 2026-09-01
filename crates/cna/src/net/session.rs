//! XNA's network session object graph.
//!
//! # Ownership
//!
//! | Handle | Policy | Why |
//! |---|---|---|
//! | `NetworkSession` | owned | `cna_network_session_create*` answers an owned session, and `destroy` refuses while a gamer view is open |
//! | a roster gamer | owned view over a session-owned gamer | `cna_network_session_get_gamer` answers a **view** that must be released before the session |
//! | `NetworkMachine` | owned copy | `cna_network_gamer_copy_machine` hands back an independent copy, which is observationally identical because the canonical machine has no mutator |
//! | `AvailableNetworkSession` | owned copy | `cna_available_network_session_collection_copy_session` documents the copy as staying valid after its collection is disposed |
//! | `LocalNetworkGamer.SignedInGamer` | owned view over a borrowed gamer | released with `cna_signed_in_gamer_destroy`, which releases the view and not the gamer |
//!
//! The refusal is the interesting one. A session cannot be released while a
//! roster view it handed out is still open, so [`NetworkSession`] releases
//! every cached gamer facade *before* releasing its own handle -- in `Dispose`
//! and in `Drop`. Getting that order wrong leaks the session rather than
//! failing loudly, which is why the disposal test checks the session actually
//! reports disposed afterwards.

#![allow(non_snake_case)]

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::disposal::Disposable;
use crate::error::{CnaError, Result};
use crate::extensions::events::EventHandler;
use crate::game::TimeSpan;
use crate::gamer_services::{
    with_callback, Gamer, GamerAsyncCallback, GamerAsyncResult, GamerAsyncState, GamerBase,
    InviteAcceptedEventArgs, GamerCollection, GamerProfile, SignedInGamer,
};

use super::events::{
    GameEndedEventArgs, GameStartedEventArgs, GamerJoinedEventArgs, GamerLeftEventArgs,
    HostChangedEventArgs, NetworkSessionEndedEventArgs, WriteLeaderboardsEventArgs,
};
use super::values::NetworkSessionEndReason;
use super::values::{
    NetworkSessionProperties, NetworkSessionState, NetworkSessionType, PacketReader, PacketWriter,
    SendDataOptions,
};

use crate::gamer_services::GamerServicesRuntimeHandle;

/// Opens, or reuses, the process CNA library for the Net family.
///
/// The Net routes are process-global like the gamer-services ones, so they
/// share the same audited table rather than opening a second one.
pub(crate) fn net_runtime() -> Result<GamerServicesRuntimeHandle> {
    crate::gamer_services::open_runtime()
}

/// The live handle behind a session, for the CNA-only extension surface.
pub(crate) fn session_handle(session: &NetworkSession) -> Result<sys::CNA_Handle> {
    session.handle()
}

/// The live handle behind a network machine, for the same surface.
pub(crate) fn machine_handle(machine: &NetworkMachine) -> Result<sys::CNA_Handle> {
    machine.owner.get()
}

/// The live handle behind a discovered session, for the same surface.
pub(crate) fn available_session_handle(
    session: &AvailableNetworkSession,
) -> Result<sys::CNA_Handle> {
    session.owner.get()
}

/// The live handle behind a network gamer, for the same surface.
pub(crate) fn gamer_handle(gamer: &NetworkGamer) -> Result<sys::CNA_Handle> {
    gamer.gamer.handle()
}

/// The live handle behind a local network gamer, for the same surface.
pub(crate) fn local_gamer_handle(gamer: &LocalNetworkGamer) -> Result<sys::CNA_Handle> {
    gamer.gamer.handle()
}

/// XNA `Microsoft.Xna.Framework.Net.QualityOfService`.
///
/// A read-only snapshot of what CNA measured for one discovered session.
#[derive(Clone, Copy, Debug)]
pub struct QualityOfService(sys::CNA_QualityOfService);

impl QualityOfService {
    pub(crate) const fn from_native(value: sys::CNA_QualityOfService) -> Self {
        Self(value)
    }

    /// XNA `QualityOfService.IsAvailable`.
    ///
    /// `false` is the honest answer wherever CNA measured nothing; the other
    /// four properties are then whatever CNA reported for an unmeasured link
    /// rather than an invented latency.
    #[must_use]
    pub const fn IsAvailable(&self) -> bool {
        self.0.is_available != 0
    }

    /// XNA `QualityOfService.BytesPerSecondUpstream`.
    #[must_use]
    pub const fn BytesPerSecondUpstream(&self) -> i32 {
        self.0.bytes_per_second_upstream
    }

    /// XNA `QualityOfService.BytesPerSecondDownstream`.
    #[must_use]
    pub const fn BytesPerSecondDownstream(&self) -> i32 {
        self.0.bytes_per_second_downstream
    }

    /// XNA `QualityOfService.AverageRoundtripTime`.
    #[must_use]
    pub const fn AverageRoundtripTime(&self) -> TimeSpan {
        TimeSpan::from_ticks(self.0.average_roundtrip_ticks)
    }

    /// XNA `QualityOfService.MinimumRoundtripTime`.
    #[must_use]
    pub const fn MinimumRoundtripTime(&self) -> TimeSpan {
        TimeSpan::from_ticks(self.0.minimum_roundtrip_ticks)
    }
}

/// One owned native handle in this family, released exactly once.
#[derive(Debug)]
pub(crate) struct NetHandle {
    runtime: GamerServicesRuntimeHandle,
    handle: Mutex<sys::CNA_Handle>,
    release: unsafe extern "C" fn(sys::CNA_Handle) -> sys::CNA_Result,
}

impl NetHandle {
    pub(crate) fn new(
        runtime: GamerServicesRuntimeHandle,
        handle: sys::CNA_Handle,
        release: unsafe extern "C" fn(sys::CNA_Handle) -> sys::CNA_Result,
    ) -> Arc<Self> {
        Arc::new(Self {
            runtime,
            handle: Mutex::new(handle),
            release,
        })
    }

    pub(crate) fn get(&self) -> Result<sys::CNA_Handle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (handle != 0)
            .then_some(handle)
            .ok_or(CnaError::InvalidInput("the network object is disposed"))
    }

    pub(crate) fn is_released(&self) -> bool {
        *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            == 0
    }

    pub(crate) fn runtime(&self) -> &GamerServicesRuntimeHandle {
        &self.runtime
    }

    pub(crate) fn check(&self, result: sys::CNA_Result) -> Result<()> {
        self.runtime.check(result)
    }

    pub(crate) fn release(&self) -> Result<()> {
        let mut slot = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = *slot;
        if handle == 0 {
            return Ok(());
        }
        *slot = 0;
        // SAFETY: the handle came from the matching canonical constructor and
        // is released exactly once.
        let result = unsafe { (self.release)(handle) };
        if result == sys::CNA_RESULT_SUCCESS {
            return Ok(());
        }
        // CNA kept the resource -- a session with an open gamer view is the
        // documented case -- so the release is still owed.
        *slot = handle;
        self.runtime.check(result)
    }
}

impl Drop for NetHandle {
    fn drop(&mut self) {
        let handle = *self
            .handle
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == 0 {
            return;
        }
        // SAFETY: the same exactly-once release the explicit path performs.
        let _ = unsafe { (self.release)(handle) };
    }
}

/// The public contract `NetworkGamer` adds to `Gamer`.
///
/// `LocalNetworkGamer` derives from `NetworkGamer` in XNA, and Rust has no
/// class inheritance, so the relationship is stated through this trait exactly
/// as `GamerBase` states the one above it.
pub trait NetworkGamerBase {
    /// XNA `NetworkGamer.Session`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn Session(&self) -> Result<NetworkSession>;

    /// XNA `NetworkGamer.Machine`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn Machine(&self) -> Result<NetworkMachine>;

    /// XNA `NetworkGamer.IsHost`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn IsHost(&self) -> Result<bool>;

    /// XNA `NetworkGamer.IsLocal`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn IsLocal(&self) -> Result<bool>;

    /// XNA `NetworkGamer.IsPrivateSlot`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn IsPrivateSlot(&self) -> Result<bool>;

    /// XNA `NetworkGamer.IsReady`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn IsReady(&self) -> Result<bool>;

    /// XNA `NetworkGamer.IsReady` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn SetIsReady(&self, value: bool) -> Result<()>;

    /// XNA `NetworkGamer.HasVoice`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn HasVoice(&self) -> Result<bool>;

    /// XNA `NetworkGamer.IsTalking`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn IsTalking(&self) -> Result<bool>;

    /// XNA `NetworkGamer.IsMutedByLocalUser`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn IsMutedByLocalUser(&self) -> Result<bool>;

    /// XNA `NetworkGamer.IsGuest`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn IsGuest(&self) -> Result<bool>;

    /// XNA `NetworkGamer.RoundtripTime`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn RoundtripTime(&self) -> Result<TimeSpan>;

    /// XNA `NetworkGamer.Id`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn Id(&self) -> Result<u8>;

    /// XNA `NetworkGamer.HasLeftSession`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn HasLeftSession(&self) -> Result<bool>;
}

/// The shared state behind a network gamer facade.
#[derive(Clone, Debug)]
pub(crate) struct NetworkGamerCore {
    owner: Arc<NetHandle>,
}

impl NetworkGamerCore {
    fn adopt(runtime: GamerServicesRuntimeHandle, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime.native().net.network_gamer_destroy;
        Self {
            owner: NetHandle::new(runtime, handle, destroy),
        }
    }

    fn handle(&self) -> Result<sys::CNA_Handle> {
        self.owner.get()
    }

    fn api(&self) -> &crate::native::net::NetApi {
        &self.owner.runtime().native().net
    }

    fn flag(
        &self,
        route: unsafe extern "C" fn(sys::CNA_Handle, *mut sys::CNA_Bool) -> sys::CNA_Result,
    ) -> Result<bool> {
        let handle = self.handle()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe { route(handle, &mut value) })?;
        Ok(value != 0)
    }
}

macro_rules! network_gamer_base {
    ($name:ty) => {
        impl NetworkGamerBase for $name {
            fn Session(&self) -> Result<NetworkSession> {
                let handle = self.gamer.handle()?;
                let mut session = 0;
                // SAFETY: the handle is live and the output is initialized.
                self.gamer.owner.check(unsafe {
                    (self.gamer.api().network_gamer_get_session)(handle, &mut session)
                })?;
                if session == 0 {
                    return Err(CnaError::InvalidInput("the gamer belongs to no session"));
                }
                // The gamer names the session it was created in; the session is
                // not the gamer's to release, so the facade borrows it.
                Ok(NetworkSession::borrowed(
                    self.gamer.owner.runtime().clone(),
                    session,
                ))
            }

            fn Machine(&self) -> Result<NetworkMachine> {
                let handle = self.gamer.handle()?;
                let mut machine = 0;
                // SAFETY: the handle is live and the output receives an owned copy.
                self.gamer.owner.check(unsafe {
                    (self.gamer.api().network_gamer_copy_machine)(handle, &mut machine)
                })?;
                Ok(NetworkMachine::adopt(
                    self.gamer.owner.runtime().clone(),
                    machine,
                ))
            }

            fn IsHost(&self) -> Result<bool> {
                self.gamer.flag(self.gamer.api().network_gamer_get_is_host)
            }

            fn IsLocal(&self) -> Result<bool> {
                self.gamer.flag(self.gamer.api().network_gamer_get_is_local)
            }

            fn IsPrivateSlot(&self) -> Result<bool> {
                self.gamer
                    .flag(self.gamer.api().network_gamer_get_is_private_slot)
            }

            fn IsReady(&self) -> Result<bool> {
                self.gamer.flag(self.gamer.api().network_gamer_get_is_ready)
            }

            fn SetIsReady(&self, value: bool) -> Result<()> {
                let handle = self.gamer.handle()?;
                // SAFETY: the handle is live and the flag is canonical.
                self.gamer.owner.check(unsafe {
                    (self.gamer.api().network_gamer_set_is_ready)(handle, u8::from(value).into())
                })
            }

            fn HasVoice(&self) -> Result<bool> {
                self.gamer.flag(self.gamer.api().network_gamer_get_has_voice)
            }

            fn IsTalking(&self) -> Result<bool> {
                self.gamer
                    .flag(self.gamer.api().network_gamer_get_is_talking)
            }

            fn IsMutedByLocalUser(&self) -> Result<bool> {
                self.gamer
                    .flag(self.gamer.api().network_gamer_get_is_muted_by_local_user)
            }

            fn IsGuest(&self) -> Result<bool> {
                self.gamer.flag(self.gamer.api().network_gamer_get_is_guest)
            }

            fn RoundtripTime(&self) -> Result<TimeSpan> {
                let handle = self.gamer.handle()?;
                let mut ticks = 0;
                // SAFETY: the handle is live and the output is initialized.
                self.gamer.owner.check(unsafe {
                    (self.gamer.api().network_gamer_get_roundtrip_ticks)(handle, &mut ticks)
                })?;
                Ok(TimeSpan::from_ticks(ticks))
            }

            fn Id(&self) -> Result<u8> {
                let handle = self.gamer.handle()?;
                let mut value = 0;
                // SAFETY: the handle is live and the output is initialized.
                self.gamer
                    .owner
                    .check(unsafe { (self.gamer.api().network_gamer_get_id)(handle, &mut value) })?;
                Ok(value)
            }

            fn HasLeftSession(&self) -> Result<bool> {
                self.gamer
                    .flag(self.gamer.api().network_gamer_get_has_left_session)
            }
        }

        impl GamerBase for $name {
            fn Gamertag(&self) -> Result<String> {
                self.as_gamer()?.Gamertag()
            }

            fn DisplayName(&self) -> Result<String> {
                self.as_gamer()?.DisplayName()
            }

            fn Tag(&self) -> Result<u64> {
                self.as_gamer()?.Tag()
            }

            fn SetTag(&self, value: u64) -> Result<()> {
                self.as_gamer()?.SetTag(value)
            }

            fn IsDisposed(&self) -> Result<bool> {
                if self.gamer.owner.is_released() {
                    return Ok(true);
                }
                self.as_gamer()?.IsDisposed()
            }

            fn ToString(&self) -> Result<String> {
                self.as_gamer()?.ToString()
            }

            fn GetProfile(&self) -> Result<GamerProfile> {
                self.as_gamer()?.GetProfile()
            }

            fn LeaderboardWriter(&self) -> Result<&crate::gamer_services::LeaderboardWriter> {
                Err(CnaError::InvalidInput(
                    "a network gamer's leaderboard writer is reached through its Gamer view",
                ))
            }

            fn handle_for_guide(&self) -> Result<sys::CNA_Handle> {
                self.gamer.handle()
            }
        }
    };
}

/// XNA `Microsoft.Xna.Framework.Net.NetworkGamer`.
#[derive(Clone, Debug)]
pub struct NetworkGamer {
    pub(crate) gamer: NetworkGamerCore,
}

network_gamer_base!(NetworkGamer);

impl NetworkGamer {
    pub(crate) fn adopt(runtime: GamerServicesRuntimeHandle, handle: sys::CNA_Handle) -> Self {
        Self {
            gamer: NetworkGamerCore::adopt(runtime, handle),
        }
    }

    /// The gamer-base view of this network gamer.
    ///
    /// Every `cna_gamer_*` route accepts a network gamer handle, because the
    /// canonical surface belongs to the gamer base both derive from.
    fn as_gamer(&self) -> Result<Gamer> {
        Ok(Gamer::from_borrowed_handle(
            self.gamer.owner.runtime().clone(),
            self.gamer.handle()?,
        ))
    }

    pub(crate) fn release(&self) -> Result<()> {
        self.gamer.owner.release()
    }

    /// Adopts a gamer a host created and admitted to a session.
    ///
    /// The session owns it from that point, so the facade must not release it:
    /// the session's own disposal does.
    pub(crate) fn from_admitted(
        runtime: GamerServicesRuntimeHandle,
        handle: sys::CNA_Handle,
    ) -> Self {
        Self {
            gamer: NetworkGamerCore {
                owner: NetHandle::new(runtime, handle, no_release),
            },
        }
    }
}

impl NetworkGamer {
    /// XNA `NetworkGamer.Session`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Session(&self) -> Result<NetworkSession> {
        NetworkGamerBase::Session(self)
    }

    /// XNA `NetworkGamer.Machine`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Machine(&self) -> Result<NetworkMachine> {
        NetworkGamerBase::Machine(self)
    }

    /// XNA `NetworkGamer.IsHost`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsHost(&self) -> Result<bool> {
        NetworkGamerBase::IsHost(self)
    }

    /// XNA `NetworkGamer.IsLocal`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsLocal(&self) -> Result<bool> {
        NetworkGamerBase::IsLocal(self)
    }

    /// XNA `NetworkGamer.IsPrivateSlot`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsPrivateSlot(&self) -> Result<bool> {
        NetworkGamerBase::IsPrivateSlot(self)
    }

    /// XNA `NetworkGamer.IsReady`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsReady(&self) -> Result<bool> {
        NetworkGamerBase::IsReady(self)
    }

    /// XNA `NetworkGamer.IsReady` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetIsReady(&self, value: bool) -> Result<()> {
        NetworkGamerBase::SetIsReady(self, value)
    }

    /// XNA `NetworkGamer.HasVoice`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn HasVoice(&self) -> Result<bool> {
        NetworkGamerBase::HasVoice(self)
    }

    /// XNA `NetworkGamer.IsTalking`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsTalking(&self) -> Result<bool> {
        NetworkGamerBase::IsTalking(self)
    }

    /// XNA `NetworkGamer.IsMutedByLocalUser`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsMutedByLocalUser(&self) -> Result<bool> {
        NetworkGamerBase::IsMutedByLocalUser(self)
    }

    /// XNA `NetworkGamer.IsGuest`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsGuest(&self) -> Result<bool> {
        NetworkGamerBase::IsGuest(self)
    }

    /// XNA `NetworkGamer.RoundtripTime`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn RoundtripTime(&self) -> Result<TimeSpan> {
        NetworkGamerBase::RoundtripTime(self)
    }

    /// XNA `NetworkGamer.Id`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Id(&self) -> Result<u8> {
        NetworkGamerBase::Id(self)
    }

    /// XNA `NetworkGamer.HasLeftSession`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn HasLeftSession(&self) -> Result<bool> {
        NetworkGamerBase::HasLeftSession(self)
    }
}

/// The `ReadOnlyCollection<T>` contract a discovered-session collection
/// inherits.
///
/// `Count` and the integer indexer are the BCL base's members, not
/// `AvailableNetworkSessionCollection`'s own, so they arrive through this
/// trait rather than as members Microsoft never declared on it.
pub trait ReadOnlyCollectionBase<T> {
    /// CLR `ReadOnlyCollection<T>.Count`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn Count(&self) -> Result<i32>;

    /// CLR `ReadOnlyCollection<T>.this[int]`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn ItemAt(&self, index: i32) -> Result<T>;
}

impl ReadOnlyCollectionBase<AvailableNetworkSession> for AvailableNetworkSessionCollection {
    fn Count(&self) -> Result<i32> {
        self.count()
    }

    fn ItemAt(&self, index: i32) -> Result<AvailableNetworkSession> {
        self.item_at(index)
    }
}

/// XNA `Microsoft.Xna.Framework.Net.LocalNetworkGamer`.
#[derive(Clone, Debug)]
pub struct LocalNetworkGamer {
    pub(crate) gamer: NetworkGamerCore,
}

network_gamer_base!(LocalNetworkGamer);

impl LocalNetworkGamer {
    pub(crate) fn adopt(runtime: GamerServicesRuntimeHandle, handle: sys::CNA_Handle) -> Self {
        Self {
            gamer: NetworkGamerCore::adopt(runtime, handle),
        }
    }

    fn as_gamer(&self) -> Result<Gamer> {
        Ok(Gamer::from_borrowed_handle(
            self.gamer.owner.runtime().clone(),
            self.gamer.handle()?,
        ))
    }

    pub(crate) fn release(&self) -> Result<()> {
        self.gamer.owner.release()
    }

    /// XNA `LocalNetworkGamer.SignedInGamer`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, or the input error when the local
    /// gamer has no signed-in gamer behind it.
    pub fn SignedInGamer(&self) -> Result<SignedInGamer> {
        let handle = self.gamer.handle()?;
        let mut signed_in = 0;
        // SAFETY: the handle is live and the output receives a view handle.
        self.gamer.owner.check(unsafe {
            (self.gamer.api().local_network_gamer_get_signed_in_gamer)(handle, &mut signed_in)
        })?;
        if signed_in == 0 {
            return Err(CnaError::InvalidInput(
                "the local gamer has no signed-in gamer",
            ));
        }
        Ok(SignedInGamer::from_signed_in_view(
            self.gamer.owner.runtime().clone(),
            signed_in,
        ))
    }

    /// XNA `LocalNetworkGamer.IsDataAvailable`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsDataAvailable(&self) -> Result<bool> {
        self.gamer
            .flag(self.gamer.api().local_network_gamer_get_is_data_available)
    }

    /// XNA `LocalNetworkGamer.EnableSendVoice`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn EnableSendVoice(&self, remoteGamer: &NetworkGamer, enable: bool) -> Result<()> {
        let handle = self.gamer.handle()?;
        let remote = remoteGamer.gamer.handle()?;
        // SAFETY: both handles are live and the flag is canonical.
        self.gamer.owner.check(unsafe {
            (self.gamer.api().local_network_gamer_enable_send_voice)(
                handle,
                remote,
                u8::from(enable).into(),
            )
        })
    }

    /// XNA `LocalNetworkGamer.SendPartyInvites`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SendPartyInvites(&self) -> Result<()> {
        let handle = self.gamer.handle()?;
        // SAFETY: the handle is live.
        self.gamer
            .owner
            .check(unsafe { (self.gamer.api().local_network_gamer_send_party_invites)(handle) })
    }

    /// XNA `LocalNetworkGamer.SendData(data, options)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SendData(&self, data: &[u8], options: SendDataOptions) -> Result<()> {
        let handle = self.gamer.handle()?;
        let (pointer, count) = byte_array(data)?;
        // SAFETY: the array describes exactly `count` readable bytes, copied
        // during the call.
        self.gamer.owner.check(unsafe {
            (self.gamer.api().local_network_gamer_send_data)(
                handle,
                pointer,
                count,
                options.bits(),
            )
        })
    }

    /// XNA `LocalNetworkGamer.SendData(data, options, recipient)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SendDataWithDataAndOptionsAndRecipientAsByteArrayAndSendDataOptionsAndNetworkGamer(
        &self,
        data: &[u8],
        options: SendDataOptions,
        recipient: &NetworkGamer,
    ) -> Result<()> {
        let handle = self.gamer.handle()?;
        let target = recipient.gamer.handle()?;
        let (pointer, count) = byte_array(data)?;
        // SAFETY: as above, with a live recipient handle.
        self.gamer.owner.check(unsafe {
            (self.gamer.api().local_network_gamer_send_data_to)(
                handle,
                pointer,
                count,
                options.bits(),
                target,
            )
        })
    }

    /// XNA `LocalNetworkGamer.SendData(data, offset, count, options)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SendDataWithDataAndOffsetAndCountAndOptions(
        &self,
        data: &[u8],
        offset: i32,
        count: i32,
        options: SendDataOptions,
    ) -> Result<()> {
        let handle = self.gamer.handle()?;
        let (pointer, length) = byte_array(data)?;
        // SAFETY: CNA range-checks the window against `length`.
        self.gamer.owner.check(unsafe {
            (self.gamer.api().local_network_gamer_send_data_range)(
                handle,
                pointer,
                length,
                offset,
                count,
                options.bits(),
            )
        })
    }

    /// XNA `LocalNetworkGamer.SendData(data, offset, count, options, recipient)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SendDataWithDataAndOffsetAndCountAndOptionsAndRecipient(
        &self,
        data: &[u8],
        offset: i32,
        count: i32,
        options: SendDataOptions,
        recipient: &NetworkGamer,
    ) -> Result<()> {
        let handle = self.gamer.handle()?;
        let target = recipient.gamer.handle()?;
        let (pointer, length) = byte_array(data)?;
        // SAFETY: as above, with a live recipient handle.
        self.gamer.owner.check(unsafe {
            (self.gamer.api().local_network_gamer_send_data_range_to)(
                handle,
                pointer,
                length,
                offset,
                count,
                options.bits(),
                target,
            )
        })
    }

    /// XNA `LocalNetworkGamer.SendData(PacketWriter, options)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SendDataWithDataAndOptions(
        &self,
        data: &PacketWriter,
        options: SendDataOptions,
    ) -> Result<()> {
        self.SendData(data.bytes(), options)
    }

    /// XNA `LocalNetworkGamer.SendData(PacketWriter, options, recipient)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SendDataWithDataAndOptionsAndRecipientAsPacketWriterAndSendDataOptionsAndNetworkGamer(
        &self,
        data: &PacketWriter,
        options: SendDataOptions,
        recipient: &NetworkGamer,
    ) -> Result<()> {
        self.SendDataWithDataAndOptionsAndRecipientAsByteArrayAndSendDataOptionsAndNetworkGamer(
            data.bytes(),
            options,
            recipient,
        )
    }

    /// XNA `LocalNetworkGamer.ReceiveData(data, out sender)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ReceiveData(&self, data: &mut [u8], sender: &mut Option<NetworkGamer>) -> Result<i32> {
        let handle = self.gamer.handle()?;
        let capacity = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("the packet buffer is too large"))?;
        let pointer = if data.is_empty() {
            core::ptr::null_mut()
        } else {
            data.as_mut_ptr()
        };
        let (mut from, mut received) = (0, 0);
        // SAFETY: the destination has exactly `capacity` writable bytes.
        self.gamer.owner.check(unsafe {
            (self.gamer.api().local_network_gamer_receive_data)(
                handle,
                pointer,
                capacity,
                &mut from,
                &mut received,
            )
        })?;
        *sender = (from != 0)
            .then(|| NetworkGamer::adopt(self.gamer.owner.runtime().clone(), from));
        i32::try_from(received)
            .map_err(|_| CnaError::InvalidInput("CNA reported an impossible packet length"))
    }

    /// XNA `LocalNetworkGamer.ReceiveData(data, offset, out sender)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ReceiveDataWithDataAndOffsetAndSender(
        &self,
        data: &mut [u8],
        offset: i32,
        sender: &mut Option<NetworkGamer>,
    ) -> Result<i32> {
        let handle = self.gamer.handle()?;
        let capacity = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("the packet buffer is too large"))?;
        let pointer = if data.is_empty() {
            core::ptr::null_mut()
        } else {
            data.as_mut_ptr()
        };
        let (mut from, mut received) = (0, 0);
        // SAFETY: CNA range-checks the offset against `capacity`.
        self.gamer.owner.check(unsafe {
            (self.gamer.api().local_network_gamer_receive_data_at)(
                handle,
                pointer,
                capacity,
                offset,
                &mut from,
                &mut received,
            )
        })?;
        *sender = (from != 0)
            .then(|| NetworkGamer::adopt(self.gamer.owner.runtime().clone(), from));
        i32::try_from(received)
            .map_err(|_| CnaError::InvalidInput("CNA reported an impossible packet length"))
    }

    /// XNA `LocalNetworkGamer.ReceiveData(PacketReader, out sender)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ReceiveDataWithDataAndSender(
        &self,
        data: &mut PacketReader,
        sender: &mut Option<NetworkGamer>,
    ) -> Result<i32> {
        // One byte more than the largest packet this overload delivers, so a
        // packet that fills the buffer is a packet that did not fit. See
        // LARGEST_PACKET_INTO_A_READER.
        let capacity = LARGEST_PACKET_INTO_A_READER + 1;
        let received = data.receive(capacity, |buffer| self.ReceiveData(buffer, sender))?;
        if usize::try_from(received).unwrap_or(0) > LARGEST_PACKET_INTO_A_READER {
            return Err(CnaError::InvalidInput(
                "the received packet is larger than a PacketReader can be given",
            ));
        }
        Ok(received)
    }
}

/// XNA `Microsoft.Xna.Framework.Net.NetworkMachine`.
#[derive(Debug)]
pub struct NetworkMachine {
    owner: Arc<NetHandle>,
    gamers: Mutex<Vec<NetworkGamer>>,
}

impl NetworkMachine {
    pub(crate) fn adopt(runtime: GamerServicesRuntimeHandle, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime.native().net.network_machine_destroy;
        Self {
            owner: NetHandle::new(runtime, handle, destroy),
            gamers: Mutex::new(Vec::new()),
        }
    }

    /// XNA `NetworkMachine.Gamers`.
    ///
    /// Only a session populates a machine's gamer collection, so a machine
    /// created outside one legitimately reports none.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Gamers(&self) -> Result<GamerCollection<NetworkGamer>> {
        let handle = self.owner.get()?;
        let mut count = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self.owner.runtime().native().net.network_machine_get_gamer_count)(
                handle, &mut count,
            )
        })?;
        let mut cached = self
            .gamers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        cached.clear();
        for index in 0..count {
            let mut gamer = 0;
            // SAFETY: the index is inside the reported count.
            self.owner.check(unsafe {
                (self.owner.runtime().native().net.network_machine_get_gamer)(
                    handle, index, &mut gamer,
                )
            })?;
            cached.push(NetworkGamer::adopt(self.owner.runtime().clone(), gamer));
        }
        Ok(GamerCollection::from_values(cached.clone()))
    }

    /// XNA `NetworkMachine.RemoveFromSession`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn RemoveFromSession(&self) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live.
        self.owner.check(unsafe {
            (self
                .owner
                .runtime()
                .native()
                .net
                .network_machine_remove_from_session)(handle)
        })
    }
}

impl Drop for NetworkMachine {
    fn drop(&mut self) {
        // Views the machine handed out must go first: CNA refuses to release a
        // machine while one is open.
        self.gamers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        let _ = self.owner.release();
    }
}

/// XNA `Microsoft.Xna.Framework.Net.AvailableNetworkSession`.
#[derive(Debug)]
pub struct AvailableNetworkSession {
    owner: Arc<NetHandle>,
}

impl AvailableNetworkSession {
    pub(crate) fn adopt(runtime: GamerServicesRuntimeHandle, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime.native().net.available_network_session_destroy;
        Self {
            owner: NetHandle::new(runtime, handle, destroy),
        }
    }

    fn api(&self) -> &crate::native::net::NetApi {
        &self.owner.runtime().native().net
    }

    /// XNA `AvailableNetworkSession.HostGamertag`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn HostGamertag(&self) -> Result<String> {
        let handle = self.owner.get()?;
        let api = self.api();
        let (size, copy) = (
            api.available_network_session_get_host_gamertag_size,
            api.available_network_session_copy_host_gamertag,
        );
        crate::native::runtime::read_string(
            |value| self.owner.check(value),
            // SAFETY: the handle is live for the size query.
            |bytes| unsafe { size(handle, bytes) },
            // SAFETY: the destination has the reported capacity.
            |destination, capacity, written| unsafe {
                copy(handle, destination, capacity, written)
            },
        )
    }

    /// XNA `AvailableNetworkSession.CurrentGamerCount`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn CurrentGamerCount(&self) -> Result<i32> {
        self.count(self.api().available_network_session_get_current_gamer_count)
    }

    /// XNA `AvailableNetworkSession.OpenPublicGamerSlots`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn OpenPublicGamerSlots(&self) -> Result<i32> {
        self.count(self.api().available_network_session_get_open_public_gamer_slots)
    }

    /// XNA `AvailableNetworkSession.OpenPrivateGamerSlots`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn OpenPrivateGamerSlots(&self) -> Result<i32> {
        self.count(self.api().available_network_session_get_open_private_gamer_slots)
    }

    /// XNA `AvailableNetworkSession.QualityOfService`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn QualityOfService(&self) -> Result<QualityOfService> {
        let handle = self.owner.get()?;
        let mut value = sys::CNA_QualityOfService {
            struct_size: core::mem::size_of::<sys::CNA_QualityOfService>() as u32,
            struct_version: 1,
            ..sys::CNA_QualityOfService::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.owner.check(unsafe {
            (self.api().available_network_session_get_quality_of_service)(handle, &mut value)
        })?;
        Ok(QualityOfService::from_native(value))
    }

    /// XNA `AvailableNetworkSession.SessionProperties`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SessionProperties(&self) -> Result<NetworkSessionProperties> {
        let handle = self.owner.get()?;
        let mut properties = 0;
        // SAFETY: the handle is live and the output receives an owned handle.
        self.owner.check(unsafe {
            (self.api().available_network_session_copy_session_properties)(handle, &mut properties)
        })?;
        read_properties(self.owner.runtime(), properties)
    }

    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.owner.get()
    }

    fn count(
        &self,
        route: unsafe extern "C" fn(sys::CNA_Handle, *mut i32) -> sys::CNA_Result,
    ) -> Result<i32> {
        let handle = self.owner.get()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe { route(handle, &mut value) })?;
        Ok(value)
    }
}

/// XNA `Microsoft.Xna.Framework.Net.AvailableNetworkSessionCollection`.
#[derive(Debug)]
pub struct AvailableNetworkSessionCollection {
    owner: Arc<NetHandle>,
}

impl AvailableNetworkSessionCollection {
    pub(crate) fn adopt(runtime: GamerServicesRuntimeHandle, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime
            .native()
            .net
            .available_network_session_collection_destroy;
        Self {
            owner: NetHandle::new(runtime, handle, destroy),
        }
    }

    fn count(&self) -> Result<i32> {
        let handle = self.owner.get()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self
                .owner
                .runtime()
                .native()
                .net
                .available_network_session_collection_get_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// The answer owns an independent copy, so it stays usable after this
    /// collection is disposed -- which is what CNA documents and what a caller
    /// joining a discovered session needs.
    fn item_at(&self, index: i32) -> Result<AvailableNetworkSession> {
        let handle = self.owner.get()?;
        let mut session = 0;
        // SAFETY: CNA range-checks the index and answers an owned copy.
        self.owner.check(unsafe {
            (self
                .owner
                .runtime()
                .native()
                .net
                .available_network_session_collection_copy_session)(
                handle, index, &mut session
            )
        })?;
        Ok(AvailableNetworkSession::adopt(
            self.owner.runtime().clone(),
            session,
        ))
    }

    /// XNA `AvailableNetworkSessionCollection.IsDisposed`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsDisposed(&self) -> Result<bool> {
        if self.owner.is_released() {
            return Ok(true);
        }
        let handle = self.owner.get()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self
                .owner
                .runtime()
                .native()
                .net
                .available_network_session_collection_get_is_disposed)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// XNA `AvailableNetworkSessionCollection.Dispose`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Dispose(&self) -> Result<()> {
        if self.owner.is_released() {
            return Ok(());
        }
        let handle = self.owner.get()?;
        // SAFETY: the handle is live; disposal is idempotent upstream.
        self.owner.check(unsafe {
            (self
                .owner
                .runtime()
                .native()
                .net
                .available_network_session_collection_dispose)(handle)
        })
    }

    /// XNA `AvailableNetworkSessionCollection.Finalize`.
    #[allow(clippy::unused_self)]
    pub fn Finalize(&self) {}
}

impl Disposable for AvailableNetworkSessionCollection {
    fn Dispose(&mut self) {
        let _ = AvailableNetworkSessionCollection::Dispose(&*self);
    }
}

impl Drop for AvailableNetworkSessionCollection {
    fn drop(&mut self) {
        let _ = self.owner.release();
    }
}

fn byte_array(data: &[u8]) -> Result<(*const u8, u64)> {
    let count = u64::try_from(data.len())
        .map_err(|_| CnaError::InvalidInput("the packet is too large"))?;
    let pointer = if data.is_empty() {
        core::ptr::null()
    } else {
        data.as_ptr()
    };
    Ok((pointer, count))
}

/// The largest packet `ReceiveData(PacketReader, out sender)` will deliver.
///
/// XNA has no such limit and needs none: it peeks the queue, resizes the
/// reader to `Peek().Size`, and receives into an array that is exactly big
/// enough. `net_sessions.h` publishes no route that reports a queued packet's
/// size, and CNA's own array receive **truncates** to the buffer it is given
/// -- `len = min(packet.size, data.size)` -- where XNA throws
/// `ArgumentException` for an array too small. So a projection over the array
/// route has to choose a size in advance, and the two ways to get that wrong
/// are silence and a fixed number nobody wrote down.
///
/// This was `vec![0_u8; 4096]`, unexplained, with the short read reported as a
/// success. It is now a stated ceiling, and a packet above it is an error
/// rather than a packet with its tail removed. Sixty-four kibibytes is far
/// above any packet XNA's session layer produces and above ENet's unfragmented
/// payload; raising it is one constant, and removing the ceiling needs the
/// peek route `RUST-UPSTREAM-028` asks for.
const LARGEST_PACKET_INTO_A_READER: usize = 64 * 1024;

/// Reads a CNA properties handle into XNA's managed eight-slot value.
fn read_properties(
    runtime: &GamerServicesRuntimeHandle,
    handle: sys::CNA_Handle,
) -> Result<NetworkSessionProperties> {
    let api = &runtime.native().net;
    let mut properties = NetworkSessionProperties::new();
    let mut count = 0;
    // SAFETY: the handle is live and the output is initialized.
    let read = runtime.check(unsafe { (api.network_session_properties_get_count)(handle, &mut count) });
    let outcome = read.and_then(|()| {
        for index in 0..count.min(8) {
            let mut value = sys::CNA_OptionalInt32::default();
            // SAFETY: the index is inside the reported count.
            runtime.check(unsafe {
                (api.network_session_properties_get_item)(handle, index, &mut value)
            })?;
            properties.SetItem(index, (value.has_value != 0).then_some(value.value));
        }
        Ok(())
    });
    // The handle is this call's to release whatever happened.
    // SAFETY: it came from a canonical copy route and is released once.
    let _ = unsafe { (api.network_session_properties_destroy)(handle) };
    outcome?;
    Ok(properties)
}

/// Publishes XNA's managed eight-slot value as a CNA properties handle.
///
/// The handle is the caller's to release; every route that takes one either
/// copies from it or retains it, and the session routes copy.
pub(crate) fn write_properties(
    runtime: &GamerServicesRuntimeHandle,
    properties: &NetworkSessionProperties,
) -> Result<sys::CNA_Handle> {
    let api = &runtime.native().net;
    let mut handle = 0;
    // SAFETY: the output receives an owned handle.
    runtime.check(unsafe { (api.network_session_properties_create)(&mut handle) })?;
    for (index, slot) in properties.as_ref().iter().enumerate() {
        let index = i32::try_from(index)
            .map_err(|_| CnaError::InvalidInput("the property index is out of range"))?;
        let value = sys::CNA_OptionalInt32 {
            has_value: u8::from(slot.is_some()).into(),
            reserved: [0; 3],
            value: slot.unwrap_or(0),
        };
        // SAFETY: the index is inside the canonical eight slots.
        if let Err(error) =
            runtime.check(unsafe { (api.network_session_properties_set_item)(handle, index, value) })
        {
            // SAFETY: release what this call created before reporting.
            let _ = unsafe { (api.network_session_properties_destroy)(handle) };
            return Err(error);
        }
    }
    Ok(handle)
}

/// XNA `Microsoft.Xna.Framework.Net.NetworkSession`.
///
/// Owned unless it was reached through a gamer, which names the session it was
/// created in without owning it.
#[derive(Debug)]
pub struct NetworkSession {
    owner: Arc<NetHandle>,
    /// Roster views this session handed out. CNA refuses to release a session
    /// while one is open, so they are released first.
    views: Mutex<Vec<NetworkGamer>>,
    locals: Mutex<Vec<LocalNetworkGamer>>,
    events: Arc<SessionEvents>,
    owned: bool,
}

impl NetworkSession {
    /// XNA `NetworkSession.MaxSupportedGamers`.
    #[allow(non_upper_case_globals)]
    pub const MaxSupportedGamers: i32 = 31;

    /// XNA `NetworkSession.MaxPreviousGamers`.
    #[allow(non_upper_case_globals)]
    pub const MaxPreviousGamers: i32 = 100;

    fn adopt(runtime: GamerServicesRuntimeHandle, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime.native().net.network_session_destroy;
        Self {
            owner: NetHandle::new(runtime, handle, destroy),
            views: Mutex::new(Vec::new()),
            locals: Mutex::new(Vec::new()),
            events: Arc::default(),
            owned: true,
        }
    }

    pub(crate) fn borrowed(runtime: GamerServicesRuntimeHandle, handle: sys::CNA_Handle) -> Self {
        // A session named by one of its gamers is not that gamer's to release.
        Self {
            owner: NetHandle::new(runtime, handle, no_release),
            views: Mutex::new(Vec::new()),
            locals: Mutex::new(Vec::new()),
            events: Arc::default(),
            owned: false,
        }
    }

    fn api(&self) -> &crate::native::net::NetApi {
        &self.owner.runtime().native().net
    }

    /// XNA `NetworkSession.Create(sessionType, maxLocalGamers, maxGamers)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Create(
        sessionType: NetworkSessionType,
        maxLocalGamers: i32,
        maxGamers: i32,
    ) -> Result<Self> {
        let runtime = net_runtime()?;
        let mut handle = 0;
        // SAFETY: every argument is a plain scalar and the output is initialized.
        runtime.check(unsafe {
            (runtime.native().net.network_session_create)(
                sessionType as u32,
                maxLocalGamers,
                maxGamers,
                &mut handle,
            )
        })?;
        Ok(Self::adopt(runtime, handle))
    }

    /// XNA `NetworkSession.Create` with private slots and properties.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn CreateWithSessionTypeAndMaxLocalGamersAndMaxGamersAndPrivateGamerSlotsAndSessionProperties(
        sessionType: NetworkSessionType,
        maxLocalGamers: i32,
        maxGamers: i32,
        privateGamerSlots: i32,
        sessionProperties: &NetworkSessionProperties,
    ) -> Result<Self> {
        let runtime = net_runtime()?;
        let properties = write_properties(&runtime, sessionProperties)?;
        let mut handle = 0;
        // SAFETY: the properties handle is live for the call, which copies it.
        let result = runtime.check(unsafe {
            (runtime.native().net.network_session_create_with_properties)(
                sessionType as u32,
                maxLocalGamers,
                maxGamers,
                privateGamerSlots,
                properties,
                &mut handle,
            )
        });
        // SAFETY: the properties handle is this call's to release.
        let _ = unsafe { (runtime.native().net.network_session_properties_destroy)(properties) };
        result?;
        Ok(Self::adopt(runtime, handle))
    }

    /// XNA `NetworkSession.Create` with an explicit local-gamer list.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn CreateWithSessionTypeAndLocalGamersAndMaxGamersAndPrivateGamerSlotsAndSessionProperties(
        sessionType: NetworkSessionType,
        localGamers: &[SignedInGamer],
        maxGamers: i32,
        privateGamerSlots: i32,
        sessionProperties: &NetworkSessionProperties,
    ) -> Result<Self> {
        let runtime = net_runtime()?;
        let handles = localGamers
            .iter()
            .map(GamerBase::handle_for_guide)
            .collect::<Result<Vec<_>>>()?;
        let properties = write_properties(&runtime, sessionProperties)?;
        let count = u64::try_from(handles.len())
            .map_err(|_| CnaError::InvalidInput("the local gamer array is too large"))?;
        let pointer = if handles.is_empty() {
            core::ptr::null()
        } else {
            handles.as_ptr()
        };
        let mut handle = 0;
        // SAFETY: the array describes exactly `count` live gamer handles.
        let result = runtime.check(unsafe {
            (runtime.native().net.network_session_create_with_local_gamers)(
                sessionType as u32,
                pointer,
                count,
                maxGamers,
                privateGamerSlots,
                properties,
                &mut handle,
            )
        });
        // SAFETY: the properties handle is this call's to release.
        let _ = unsafe { (runtime.native().net.network_session_properties_destroy)(properties) };
        result?;
        Ok(Self::adopt(runtime, handle))
    }

    /// XNA `NetworkSession.Find`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Find(
        sessionType: NetworkSessionType,
        maxLocalGamers: i32,
        searchProperties: &NetworkSessionProperties,
    ) -> Result<AvailableNetworkSessionCollection> {
        let runtime = net_runtime()?;
        let properties = write_properties(&runtime, searchProperties)?;
        let mut collection = 0;
        // SAFETY: the properties handle is live for the call.
        let result = runtime.check(unsafe {
            (runtime.native().net.network_session_find)(
                sessionType as u32,
                maxLocalGamers,
                properties,
                &mut collection,
            )
        });
        // SAFETY: the properties handle is this call's to release.
        let _ = unsafe { (runtime.native().net.network_session_properties_destroy)(properties) };
        result?;
        Ok(AvailableNetworkSessionCollection::adopt(runtime, collection))
    }

    /// XNA `NetworkSession.Find` with an explicit local-gamer list.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn FindWithSessionTypeAndLocalGamersAndSearchProperties(
        sessionType: NetworkSessionType,
        localGamers: &[SignedInGamer],
        searchProperties: &NetworkSessionProperties,
    ) -> Result<AvailableNetworkSessionCollection> {
        let runtime = net_runtime()?;
        let handles = localGamers
            .iter()
            .map(GamerBase::handle_for_guide)
            .collect::<Result<Vec<_>>>()?;
        let properties = write_properties(&runtime, searchProperties)?;
        let count = u64::try_from(handles.len())
            .map_err(|_| CnaError::InvalidInput("the local gamer array is too large"))?;
        let pointer = if handles.is_empty() {
            core::ptr::null()
        } else {
            handles.as_ptr()
        };
        let mut collection = 0;
        // SAFETY: the array describes exactly `count` live gamer handles.
        let result = runtime.check(unsafe {
            (runtime.native().net.network_session_find_with_local_gamers)(
                sessionType as u32,
                pointer,
                count,
                properties,
                &mut collection,
            )
        });
        // SAFETY: the properties handle is this call's to release.
        let _ = unsafe { (runtime.native().net.network_session_properties_destroy)(properties) };
        result?;
        Ok(AvailableNetworkSessionCollection::adopt(runtime, collection))
    }

    /// XNA `NetworkSession.Join`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Join(availableSession: &AvailableNetworkSession) -> Result<Self> {
        let runtime = net_runtime()?;
        let discovered = availableSession.handle()?;
        let mut handle = 0;
        // SAFETY: the discovered-session handle is live for the call.
        runtime.check(unsafe {
            (runtime.native().net.network_session_join)(discovered, &mut handle)
        })?;
        Ok(Self::adopt(runtime, handle))
    }

    /// XNA `NetworkSession.JoinInvited(maxLocalGamers)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn JoinInvited(maxLocalGamers: i32) -> Result<Self> {
        let runtime = net_runtime()?;
        let mut handle = 0;
        // SAFETY: the argument is a plain scalar.
        runtime.check(unsafe {
            (runtime.native().net.network_session_join_invited)(maxLocalGamers, &mut handle)
        })?;
        Ok(Self::adopt(runtime, handle))
    }

    /// XNA `NetworkSession.JoinInvited(localGamers)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn JoinInvitedWithLocalGamers(localGamers: &[SignedInGamer]) -> Result<Self> {
        let runtime = net_runtime()?;
        let handles = localGamers
            .iter()
            .map(GamerBase::handle_for_guide)
            .collect::<Result<Vec<_>>>()?;
        let count = u64::try_from(handles.len())
            .map_err(|_| CnaError::InvalidInput("the local gamer array is too large"))?;
        let pointer = if handles.is_empty() {
            core::ptr::null()
        } else {
            handles.as_ptr()
        };
        let mut handle = 0;
        // SAFETY: the array describes exactly `count` live gamer handles.
        runtime.check(unsafe {
            (runtime
                .native()
                .net
                .network_session_join_invited_with_local_gamers)(pointer, count, &mut handle)
        })?;
        Ok(Self::adopt(runtime, handle))
    }

    /// XNA `NetworkSession.Update`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Update(&self) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live.
        self.owner
            .check(unsafe { (self.api().network_session_update)(handle) })?;
        self.events.take_pending()
    }

    /// XNA `NetworkSession.StartGame`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn StartGame(&self) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live.
        self.owner
            .check(unsafe { (self.api().network_session_start_game)(handle) })
    }

    /// XNA `NetworkSession.EndGame`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn EndGame(&self) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live.
        self.owner
            .check(unsafe { (self.api().network_session_end_game)(handle) })
    }

    /// XNA `NetworkSession.ResetReady`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ResetReady(&self) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live.
        self.owner
            .check(unsafe { (self.api().network_session_reset_ready)(handle) })
    }

    /// XNA `NetworkSession.AddLocalGamer`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn AddLocalGamer(&self, gamer: &SignedInGamer) -> Result<()> {
        let handle = self.owner.get()?;
        let signed_in = gamer.handle_for_guide()?;
        // SAFETY: both handles are live for the call.
        self.owner.check(unsafe {
            (self.api().network_session_add_local_gamer)(handle, signed_in)
        })
    }

    /// XNA `NetworkSession.FindGamerById`.
    ///
    /// `None` is CLR `null`: no gamer in the session carries that identifier.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn FindGamerById(&self, gamerId: u8) -> Result<Option<NetworkGamer>> {
        let handle = self.owner.get()?;
        let mut gamer = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self.api().network_session_find_gamer_by_id)(handle, gamerId, &mut gamer)
        })?;
        Ok((gamer != 0).then(|| self.retain_view(gamer)))
    }

    /// XNA `NetworkSession.Host`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, or the input error when the
    /// session has no host.
    pub fn Host(&self) -> Result<NetworkGamer> {
        let handle = self.owner.get()?;
        let mut gamer = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner
            .check(unsafe { (self.api().network_session_get_host)(handle, &mut gamer) })?;
        if gamer == 0 {
            return Err(CnaError::InvalidInput("the session has no host"));
        }
        Ok(self.retain_view(gamer))
    }

    /// XNA `NetworkSession.AllGamers`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn AllGamers(&self) -> Result<GamerCollection<NetworkGamer>> {
        self.roster(sys::CNA_NETWORK_SESSION_ROSTER_ALL)
    }

    /// XNA `NetworkSession.RemoteGamers`.
    ///
    /// Legitimately empty in a single process: nothing here invents a peer.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn RemoteGamers(&self) -> Result<GamerCollection<NetworkGamer>> {
        self.roster(sys::CNA_NETWORK_SESSION_ROSTER_REMOTE)
    }

    /// XNA `NetworkSession.PreviousGamers`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn PreviousGamers(&self) -> Result<GamerCollection<NetworkGamer>> {
        self.roster(sys::CNA_NETWORK_SESSION_ROSTER_PREVIOUS)
    }

    /// XNA `NetworkSession.LocalGamers`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn LocalGamers(&self) -> Result<GamerCollection<LocalNetworkGamer>> {
        let handle = self.owner.get()?;
        let count = self.roster_count(sys::CNA_NETWORK_SESSION_ROSTER_LOCAL)?;
        let mut gamers = Vec::with_capacity(count.max(0) as usize);
        for index in 0..count {
            let mut gamer = 0;
            // SAFETY: the index is inside the reported roster count.
            self.owner.check(unsafe {
                (self.api().network_session_get_gamer)(
                    handle,
                    sys::CNA_NETWORK_SESSION_ROSTER_LOCAL,
                    index,
                    &mut gamer,
                )
            })?;
            let local = LocalNetworkGamer::adopt(self.owner.runtime().clone(), gamer);
            self.locals
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(local.clone());
            gamers.push(local);
        }
        Ok(GamerCollection::from_values(gamers))
    }

    /// XNA `NetworkSession.IsHost`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsHost(&self) -> Result<bool> {
        self.flag(self.api().network_session_get_is_host)
    }

    /// XNA `NetworkSession.IsEveryoneReady`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsEveryoneReady(&self) -> Result<bool> {
        self.flag(self.api().network_session_get_is_everyone_ready)
    }

    /// XNA `NetworkSession.IsDisposed`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsDisposed(&self) -> Result<bool> {
        if self.owner.is_released() {
            return Ok(true);
        }
        self.flag(self.api().network_session_get_is_disposed)
    }

    /// XNA `NetworkSession.AllowHostMigration`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn AllowHostMigration(&self) -> Result<bool> {
        self.flag(self.api().network_session_get_allow_host_migration)
    }

    /// XNA `NetworkSession.AllowHostMigration` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetAllowHostMigration(&self, value: bool) -> Result<()> {
        self.set_flag(self.api().network_session_set_allow_host_migration, value)
    }

    /// XNA `NetworkSession.AllowJoinInProgress`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn AllowJoinInProgress(&self) -> Result<bool> {
        self.flag(self.api().network_session_get_allow_join_in_progress)
    }

    /// XNA `NetworkSession.AllowJoinInProgress` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetAllowJoinInProgress(&self, value: bool) -> Result<()> {
        self.set_flag(self.api().network_session_set_allow_join_in_progress, value)
    }

    /// XNA `NetworkSession.MaxGamers`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn MaxGamers(&self) -> Result<i32> {
        self.count(self.api().network_session_get_max_gamers)
    }

    /// XNA `NetworkSession.MaxGamers` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetMaxGamers(&self, value: i32) -> Result<()> {
        self.set_count(self.api().network_session_set_max_gamers, value)
    }

    /// XNA `NetworkSession.PrivateGamerSlots`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn PrivateGamerSlots(&self) -> Result<i32> {
        self.count(self.api().network_session_get_private_gamer_slots)
    }

    /// XNA `NetworkSession.PrivateGamerSlots` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetPrivateGamerSlots(&self, value: i32) -> Result<()> {
        self.set_count(self.api().network_session_set_private_gamer_slots, value)
    }

    /// XNA `NetworkSession.BytesPerSecondSent`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BytesPerSecondSent(&self) -> Result<i32> {
        self.count(self.api().network_session_get_bytes_per_second_sent)
    }

    /// XNA `NetworkSession.BytesPerSecondReceived`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BytesPerSecondReceived(&self) -> Result<i32> {
        self.count(self.api().network_session_get_bytes_per_second_received)
    }

    /// XNA `NetworkSession.SimulatedPacketLoss`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SimulatedPacketLoss(&self) -> Result<f32> {
        let handle = self.owner.get()?;
        let mut value = 0.0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self.api().network_session_get_simulated_packet_loss)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// XNA `NetworkSession.SimulatedPacketLoss` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetSimulatedPacketLoss(&self, value: f32) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live and the value is a plain scalar.
        self.owner.check(unsafe {
            (self.api().network_session_set_simulated_packet_loss)(handle, value)
        })
    }

    /// XNA `NetworkSession.SimulatedLatency`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SimulatedLatency(&self) -> Result<TimeSpan> {
        let handle = self.owner.get()?;
        let mut ticks = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self.api().network_session_get_simulated_latency_ticks)(handle, &mut ticks)
        })?;
        Ok(TimeSpan::from_ticks(ticks))
    }

    /// XNA `NetworkSession.SimulatedLatency` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetSimulatedLatency(&self, value: TimeSpan) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live and the value is a tick count.
        self.owner.check(unsafe {
            (self.api().network_session_set_simulated_latency_ticks)(handle, value.Ticks())
        })
    }

    /// XNA `NetworkSession.SessionType`.
    ///
    /// # Errors
    ///
    /// Returns the mapping error for a session type XNA does not declare.
    pub fn SessionType(&self) -> Result<NetworkSessionType> {
        let handle = self.owner.get()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self.api().network_session_get_session_type)(handle, &mut value)
        })?;
        NetworkSessionType::from_native(value).ok_or(CnaError::InvalidInput(
            "CNA reported a session type XNA does not declare",
        ))
    }

    /// XNA `NetworkSession.SessionState`.
    ///
    /// # Errors
    ///
    /// Returns the mapping error for a state XNA does not declare.
    pub fn SessionState(&self) -> Result<NetworkSessionState> {
        let handle = self.owner.get()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self.api().network_session_get_session_state)(handle, &mut value)
        })?;
        NetworkSessionState::from_native(value).ok_or(CnaError::InvalidInput(
            "CNA reported a session state XNA does not declare",
        ))
    }

    /// XNA `NetworkSession.SessionProperties`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SessionProperties(&self) -> Result<NetworkSessionProperties> {
        let handle = self.owner.get()?;
        let mut properties = 0;
        // SAFETY: the handle is live and the output receives an owned handle.
        self.owner.check(unsafe {
            (self.api().network_session_copy_session_properties)(handle, &mut properties)
        })?;
        read_properties(self.owner.runtime(), properties)
    }

    /// XNA `NetworkSession.Dispose`.
    ///
    /// Releases every roster view this session handed out first: CNA refuses
    /// to release a session while one is still open.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Dispose(&self) -> Result<()> {
        self.events.release_all();
        self.release_views();
        if self.owner.is_released() {
            return Ok(());
        }
        let handle = self.owner.get()?;
        // SAFETY: the handle is live; disposal is idempotent upstream.
        self.owner
            .check(unsafe { (self.api().network_session_dispose)(handle) })
    }

    /// XNA `NetworkSession.Finalize`.
    #[allow(clippy::unused_self)]
    pub fn Finalize(&self) {}

    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.owner.get()
    }


    fn retain_view(&self, handle: sys::CNA_Handle) -> NetworkGamer {
        let gamer = NetworkGamer::adopt(self.owner.runtime().clone(), handle);
        self.views
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(gamer.clone());
        gamer
    }

    fn release_views(&self) {
        for gamer in self
            .views
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain(..)
        {
            let _ = gamer.release();
        }
        for gamer in self
            .locals
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain(..)
        {
            let _ = gamer.release();
        }
    }

    fn roster_count(&self, roster: u32) -> Result<i32> {
        let handle = self.owner.get()?;
        let mut count = 0;
        // SAFETY: the handle is live and the roster identity is canonical.
        self.owner.check(unsafe {
            (self.api().network_session_get_gamer_count)(handle, roster, &mut count)
        })?;
        Ok(count)
    }

    fn roster(&self, roster: u32) -> Result<GamerCollection<NetworkGamer>> {
        let handle = self.owner.get()?;
        let count = self.roster_count(roster)?;
        let mut gamers = Vec::with_capacity(count.max(0) as usize);
        for index in 0..count {
            let mut gamer = 0;
            // SAFETY: the index is inside the reported roster count.
            self.owner.check(unsafe {
                (self.api().network_session_get_gamer)(handle, roster, index, &mut gamer)
            })?;
            gamers.push(self.retain_view(gamer));
        }
        Ok(GamerCollection::from_values(gamers))
    }

    fn flag(
        &self,
        route: unsafe extern "C" fn(sys::CNA_Handle, *mut sys::CNA_Bool) -> sys::CNA_Result,
    ) -> Result<bool> {
        let handle = self.owner.get()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe { route(handle, &mut value) })?;
        Ok(value != 0)
    }

    fn set_flag(
        &self,
        route: unsafe extern "C" fn(sys::CNA_Handle, sys::CNA_Bool) -> sys::CNA_Result,
        value: bool,
    ) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live and the flag is canonical.
        self.owner
            .check(unsafe { route(handle, u8::from(value).into()) })
    }

    fn count(
        &self,
        route: unsafe extern "C" fn(sys::CNA_Handle, *mut i32) -> sys::CNA_Result,
    ) -> Result<i32> {
        let handle = self.owner.get()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe { route(handle, &mut value) })?;
        Ok(value)
    }

    fn set_count(
        &self,
        route: unsafe extern "C" fn(sys::CNA_Handle, i32) -> sys::CNA_Result,
        value: i32,
    ) -> Result<()> {
        let handle = self.owner.get()?;
        // SAFETY: the handle is live and the value is a plain scalar.
        self.owner.check(unsafe { route(handle, value) })
    }
}

impl Disposable for NetworkSession {
    fn Dispose(&mut self) {
        let _ = NetworkSession::Dispose(&*self);
    }
}

impl Drop for NetworkSession {
    fn drop(&mut self) {
        if !self.owned {
            return;
        }
        // Order matters: CNA refuses to release a session while one of its
        // gamer views is still open, and a live subscription would outlive the
        // session it names.
        self.events.release_all();
        self.release_views();
        let _ = self.owner.release();
    }
}

unsafe extern "C" fn no_release(_handle: sys::CNA_Handle) -> sys::CNA_Result {
    sys::CNA_RESULT_SUCCESS
}

/// One session's event registries and the CNA subscriptions behind them.
///
/// # Callback ownership
///
/// A subscription's `context` is a raw pointer to this state, kept alive by an
/// `Arc` the registration owns. The registration is released -- and the `Arc`
/// dropped -- when the last handler for that event is removed and when the
/// session is disposed, so no callback can reach freed state. The trampolines
/// copy every handle out of CNA's record before running a handler and contain
/// any panic at the boundary.
#[derive(Default)]
pub(crate) struct SessionEvents {
    next: Mutex<u64>,
    game_started: Mutex<Vec<(u64, SharedHandler<GameStartedEventArgs>)>>,
    game_ended: Mutex<Vec<(u64, SharedHandler<GameEndedEventArgs>)>>,
    gamer_joined: Mutex<Vec<(u64, SharedHandler<GamerJoinedEventArgs>)>>,
    gamer_left: Mutex<Vec<(u64, SharedHandler<GamerLeftEventArgs>)>>,
    host_changed: Mutex<Vec<(u64, SharedHandler<HostChangedEventArgs>)>>,
    session_ended: Mutex<Vec<(u64, SharedHandler<NetworkSessionEndedEventArgs>)>>,
    write_arbitrated: Mutex<Vec<(u64, SharedHandler<WriteLeaderboardsEventArgs>)>>,
    write_unarbitrated: Mutex<Vec<(u64, SharedHandler<WriteLeaderboardsEventArgs>)>>,
    write_true_skill: Mutex<Vec<(u64, SharedHandler<WriteLeaderboardsEventArgs>)>>,
    /// Live CNA registrations, released when their last handler goes away.
    registrations: Mutex<Vec<(SessionEvent, sys::CNA_Handle, *const SessionEvents)>>,
    /// A subscription CNA refused, surfaced by `NetworkSession.Update`.
    pending: Mutex<Option<CnaError>>,
    runtime: Mutex<Option<GamerServicesRuntimeHandle>>,
}

// SAFETY: every field is behind a mutex, and the raw pointer is only ever the
// address of an `Arc<SessionEvents>` this state itself keeps alive.
unsafe impl Send for SessionEvents {}
// SAFETY: as above.
unsafe impl Sync for SessionEvents {}

type SharedHandler<T> = Arc<Mutex<Box<dyn EventHandler<T>>>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SessionEvent {
    GameStarted,
    GameEnded,
    GamerJoined,
    GamerLeft,
    HostChanged,
    SessionEnded,
    WriteArbitrated,
    WriteUnarbitrated,
    WriteTrueSkill,
}

impl core::fmt::Debug for SessionEvents {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("SessionEvents")
    }
}

impl SessionEvents {
    fn next(&self) -> u64 {
        let mut next = self
            .next
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *next = next.wrapping_add(1).max(1);
        *next
    }

    /// Remembers the first subscription CNA refused.
    fn remember(&self, error: CnaError) {
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if pending.is_none() {
            *pending = Some(error);
        }
    }

    /// Reports and clears any subscription CNA refused.
    fn take_pending(&self) -> Result<()> {
        let taken = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        match taken {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Releases every CNA subscription this state owns.
    fn release_all(&self) {
        let taken: Vec<_> = self
            .registrations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain(..)
            .collect();
        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for (_, handle, context) in taken {
            if let Some(runtime) = &runtime {
                // SAFETY: the registration came from the matching subscribe
                // route and is released exactly once.
                let _ = unsafe { (runtime.native().net.network_session_unsubscribe)(handle) };
            }
            // SAFETY: the context is the raw `Arc` this state handed out when
            // it subscribed, reclaimed exactly once here.
            drop(unsafe { Arc::from_raw(context) });
        }
    }
}

macro_rules! session_event {
    (
        $add:ident, $remove:ident, $field:ident, $args:ty, $variant:ident,
        $subscribe:ident, $trampoline:ident, $info:ty, $build:expr
    ) => {
        unsafe extern "C" fn $trampoline(
            _session: sys::CNA_NetworkSessionHandle,
            info: *const $info,
            context: *mut core::ffi::c_void,
        ) {
            if context.is_null() {
                return;
            }
            // SAFETY: the context is the `Arc<SessionEvents>` the subscription
            // owns, which outlives every callback it can produce.
            let state = unsafe { &*context.cast::<SessionEvents>() };
            let handlers = state
                .$field
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let runtime = state
                .runtime
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            for (_, handler) in handlers {
                #[allow(clippy::redundant_closure_call)]
                #[allow(unused_unsafe)]
                let Some(args) = (unsafe { $build(info, runtime.as_ref()) }) else {
                    continue;
                };
                let mut guard = handler
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                // Contained: a panicking handler must not unwind into CNA.
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    guard.invoke(&() as &dyn core::any::Any, args)
                }));
            }
        }

        impl NetworkSession {
            #[doc = concat!("XNA `NetworkSession.", stringify!($variant), "` subscription.")]
            ///
            /// # Errors
            ///
            /// Returns the exact error CNA reports.
            #[must_use]
            pub fn $add(&self, handler: Box<dyn EventHandler<$args>>) -> u64 {
                let Ok(handle) = self.owner.get() else {
                    return 0;
                };
                *self
                    .events
                    .runtime
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) =
                    Some(self.owner.runtime().clone());
                let already = self
                    .events
                    .registrations
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .iter()
                    .any(|(kind, _, _)| *kind == SessionEvent::$variant);
                if !already {
                    let context = Arc::into_raw(Arc::clone(&self.events));
                    let mut registration = 0;
                    // SAFETY: the trampoline is a plain C function and the
                    // context is an `Arc` this state now owns.
                    let result = self.owner.check(unsafe {
                        (self.api().$subscribe)(
                            handle,
                            Some($trampoline),
                            context.cast::<core::ffi::c_void>().cast_mut(),
                            &mut registration,
                        )
                    });
                    if let Err(error) = result {
                        // SAFETY: reclaim the `Arc` the failed subscribe left.
                        drop(unsafe { Arc::from_raw(context) });
                        // XNA's `+=` cannot fail. The handler is still
                        // registered, and `NetworkSession.Update` is where a
                        // caller learns CNA refused -- the same rule the
                        // gamer-services events follow.
                        self.events.remember(error);
                    }
                    self.events
                        .registrations
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push((SessionEvent::$variant, registration, context));
                }
                let registration = self.events.next();
                self.events
                    .$field
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push((registration, Arc::new(Mutex::new(handler))));
                registration
            }

            #[doc = concat!("XNA `NetworkSession.", stringify!($variant), "` removal.")]
            #[must_use]
            pub fn $remove(&self, registration: u64) -> bool {
                let mut handlers = self
                    .events
                    .$field
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let before = handlers.len();
                handlers.retain(|(value, _)| *value != registration);
                before != handlers.len()
            }
        }
    };
}

/// Rebuilds a network gamer facade from a handle CNA delivered in an event.
///
/// The gamer belongs to the session, so the facade borrows it for the
/// handler's duration and releases nothing.
unsafe fn event_gamer(
    handle: sys::CNA_NetworkGamerHandle,
    runtime: Option<&GamerServicesRuntimeHandle>,
) -> Option<NetworkGamer> {
    let runtime = runtime?;
    (handle != 0).then(|| NetworkGamer::from_admitted(runtime.clone(), handle))
}

session_event!(
    AddGameStartedHandler,
    RemoveGameStartedHandler,
    game_started,
    GameStartedEventArgs,
    GameStarted,
    network_session_subscribe_game_started,
    game_started_trampoline,
    sys::CNA_GameStartedEventInfo,
    |_info, _runtime| Some(GameStartedEventArgs::new())
);

session_event!(
    AddGameEndedHandler,
    RemoveGameEndedHandler,
    game_ended,
    GameEndedEventArgs,
    GameEnded,
    network_session_subscribe_game_ended,
    game_ended_trampoline,
    sys::CNA_GameEndedEventInfo,
    |_info, _runtime| Some(GameEndedEventArgs::new())
);

session_event!(
    AddGamerJoinedHandler,
    RemoveGamerJoinedHandler,
    gamer_joined,
    GamerJoinedEventArgs,
    GamerJoined,
    network_session_subscribe_gamer_joined,
    gamer_joined_trampoline,
    sys::CNA_GamerJoinedEventInfo,
    |info: *const sys::CNA_GamerJoinedEventInfo, runtime| {
        if info.is_null() {
            None
        } else {
            event_gamer((*info).gamer, runtime).map(|gamer| GamerJoinedEventArgs::new(&gamer))
        }
    }
);

session_event!(
    AddGamerLeftHandler,
    RemoveGamerLeftHandler,
    gamer_left,
    GamerLeftEventArgs,
    GamerLeft,
    network_session_subscribe_gamer_left,
    gamer_left_trampoline,
    sys::CNA_GamerLeftEventInfo,
    |info: *const sys::CNA_GamerLeftEventInfo, runtime| {
        if info.is_null() {
            None
        } else {
            event_gamer((*info).gamer, runtime).map(|gamer| GamerLeftEventArgs::new(&gamer))
        }
    }
);

session_event!(
    AddHostChangedHandler,
    RemoveHostChangedHandler,
    host_changed,
    HostChangedEventArgs,
    HostChanged,
    network_session_subscribe_host_changed,
    host_changed_trampoline,
    sys::CNA_HostChangedEventInfo,
    |info: *const sys::CNA_HostChangedEventInfo, runtime| {
        if info.is_null() {
            None
        } else {
            let old = event_gamer((*info).old_host, runtime);
            let new = event_gamer((*info).new_host, runtime);
            match (old, new) {
                (Some(old), Some(new)) => Some(HostChangedEventArgs::new(&old, &new)),
                _ => None,
            }
        }
    }
);

session_event!(
    AddSessionEndedHandler,
    RemoveSessionEndedHandler,
    session_ended,
    NetworkSessionEndedEventArgs,
    SessionEnded,
    network_session_subscribe_session_ended,
    session_ended_trampoline,
    sys::CNA_NetworkSessionEndedEventInfo,
    |info: *const sys::CNA_NetworkSessionEndedEventInfo, _runtime| {
        if info.is_null() {
            None
        } else {
            NetworkSessionEndReason::from_native((*info).end_reason)
                .map(NetworkSessionEndedEventArgs::new)
        }
    }
);

session_event!(
    AddWriteArbitratedLeaderboardHandler,
    RemoveWriteArbitratedLeaderboardHandler,
    write_arbitrated,
    WriteLeaderboardsEventArgs,
    WriteArbitrated,
    network_session_subscribe_write_arbitrated_leaderboard,
    write_arbitrated_trampoline,
    sys::CNA_WriteLeaderboardsEventInfo,
    |info: *const sys::CNA_WriteLeaderboardsEventInfo, runtime| {
        if info.is_null() {
            None
        } else {
            event_gamer((*info).gamer, runtime).map(|gamer| {
                WriteLeaderboardsEventArgs::from_parts(gamer, (*info).is_leaving != 0)
            })
        }
    }
);

session_event!(
    AddWriteUnarbitratedLeaderboardHandler,
    RemoveWriteUnarbitratedLeaderboardHandler,
    write_unarbitrated,
    WriteLeaderboardsEventArgs,
    WriteUnarbitrated,
    network_session_subscribe_write_unarbitrated_leaderboard,
    write_unarbitrated_trampoline,
    sys::CNA_WriteLeaderboardsEventInfo,
    |info: *const sys::CNA_WriteLeaderboardsEventInfo, runtime| {
        if info.is_null() {
            None
        } else {
            event_gamer((*info).gamer, runtime).map(|gamer| {
                WriteLeaderboardsEventArgs::from_parts(gamer, (*info).is_leaving != 0)
            })
        }
    }
);

session_event!(
    AddWriteTrueSkillHandler,
    RemoveWriteTrueSkillHandler,
    write_true_skill,
    WriteLeaderboardsEventArgs,
    WriteTrueSkill,
    network_session_subscribe_write_true_skill,
    write_true_skill_trampoline,
    sys::CNA_WriteLeaderboardsEventInfo,
    |info: *const sys::CNA_WriteLeaderboardsEventInfo, runtime| {
        if info.is_null() {
            None
        } else {
            event_gamer((*info).gamer, runtime).map(|gamer| {
                WriteLeaderboardsEventArgs::from_parts(gamer, (*info).is_leaving != 0)
            })
        }
    }
);

/// XNA's `Begin*`/`End*` pattern for network sessions.
///
/// CNA's `*_async` routes complete before returning and then invoke the
/// caller's callback, exactly as the gamer-services ones do, so the projection
/// reuses [`GamerAsyncResult`] rather than inventing a second async vocabulary.
/// A native trampoline is installed so CNA's callback path genuinely runs, the
/// produced value is stored before the Rust callback fires -- which is what
/// makes a callback that immediately calls `End` work -- and `End` stays
/// one-shot and typed.
impl NetworkSession {
    /// XNA `NetworkSession.BeginCreate(sessionType, maxLocalGamers, maxGamers, ...)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginCreate(
        sessionType: NetworkSessionType,
        maxLocalGamers: i32,
        maxGamers: i32,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = net_runtime()?;
        let adopted = runtime.clone();
        let (result, _fired) = with_callback(asyncState, callback, |trampoline, context| {
            let mut handle = 0;
            // SAFETY: every argument is a plain scalar and the context
            // outlives the call.
            runtime.check(unsafe {
                (runtime.native().net.network_session_create_async)(
                    sessionType as u32,
                    maxLocalGamers,
                    maxGamers,
                    trampoline,
                    context,
                    &mut handle,
                )
            })?;
            Ok(Self::adopt(adopted, handle))
        })?;
        Ok(result)
    }

    /// XNA `NetworkSession.BeginCreate` with private slots and properties.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    #[allow(clippy::too_many_arguments)]
    pub fn BeginCreateWithSessionTypeAndMaxLocalGamersAndMaxGamersAndPrivateGamerSlotsAndSessionPropertiesAndCallbackAndAsyncState(
        sessionType: NetworkSessionType,
        maxLocalGamers: i32,
        maxGamers: i32,
        privateGamerSlots: i32,
        sessionProperties: &NetworkSessionProperties,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = net_runtime()?;
        let properties = write_properties(&runtime, sessionProperties)?;
        let adopted = runtime.clone();
        let outcome = with_callback(asyncState, callback, |trampoline, context| {
            let mut handle = 0;
            // SAFETY: the properties handle is live for the call, which copies it.
            runtime.check(unsafe {
                (runtime.native().net.network_session_create_with_properties_async)(
                    sessionType as u32,
                    maxLocalGamers,
                    maxGamers,
                    privateGamerSlots,
                    properties,
                    trampoline,
                    context,
                    &mut handle,
                )
            })?;
            Ok(Self::adopt(adopted, handle))
        });
        // SAFETY: the properties handle is this call's to release.
        let _ = unsafe { (runtime.native().net.network_session_properties_destroy)(properties) };
        Ok(outcome?.0)
    }

    /// XNA `NetworkSession.BeginCreate` with an explicit local-gamer list.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    #[allow(clippy::too_many_arguments)]
    pub fn BeginCreateWithSessionTypeAndLocalGamersAndMaxGamersAndPrivateGamerSlotsAndSessionPropertiesAndCallbackAndAsyncState(
        sessionType: NetworkSessionType,
        localGamers: &[SignedInGamer],
        maxGamers: i32,
        privateGamerSlots: i32,
        sessionProperties: &NetworkSessionProperties,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = net_runtime()?;
        let handles = localGamers
            .iter()
            .map(GamerBase::handle_for_guide)
            .collect::<Result<Vec<_>>>()?;
        let properties = write_properties(&runtime, sessionProperties)?;
        let (pointer, count) = handle_array(&handles)?;
        let adopted = runtime.clone();
        let outcome = with_callback(asyncState, callback, |trampoline, context| {
            let mut handle = 0;
            // SAFETY: the array describes exactly `count` live gamer handles.
            runtime.check(unsafe {
                (runtime
                    .native()
                    .net
                    .network_session_create_with_local_gamers_async)(
                    sessionType as u32,
                    pointer,
                    count,
                    maxGamers,
                    privateGamerSlots,
                    properties,
                    trampoline,
                    context,
                    &mut handle,
                )
            })?;
            Ok(Self::adopt(adopted, handle))
        });
        // SAFETY: the properties handle is this call's to release.
        let _ = unsafe { (runtime.native().net.network_session_properties_destroy)(properties) };
        Ok(outcome?.0)
    }

    /// XNA `NetworkSession.EndCreate`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated.
    pub fn EndCreate(result: &GamerAsyncResult) -> Result<Self> {
        result.end_once::<Self>()
    }

    /// XNA `NetworkSession.BeginFind`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginFind(
        sessionType: NetworkSessionType,
        maxLocalGamers: i32,
        searchProperties: &NetworkSessionProperties,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = net_runtime()?;
        let properties = write_properties(&runtime, searchProperties)?;
        let adopted = runtime.clone();
        let outcome = with_callback(asyncState, callback, |trampoline, context| {
            let mut collection = 0;
            // SAFETY: the properties handle is live for the call.
            runtime.check(unsafe {
                (runtime.native().net.network_session_find_async)(
                    sessionType as u32,
                    maxLocalGamers,
                    properties,
                    trampoline,
                    context,
                    &mut collection,
                )
            })?;
            Ok(AvailableNetworkSessionCollection::adopt(adopted, collection))
        });
        // SAFETY: the properties handle is this call's to release.
        let _ = unsafe { (runtime.native().net.network_session_properties_destroy)(properties) };
        Ok(outcome?.0)
    }

    /// XNA `NetworkSession.BeginFind` with an explicit local-gamer list.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginFindWithSessionTypeAndLocalGamersAndSearchPropertiesAndCallbackAndAsyncState(
        sessionType: NetworkSessionType,
        localGamers: &[SignedInGamer],
        searchProperties: &NetworkSessionProperties,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = net_runtime()?;
        let handles = localGamers
            .iter()
            .map(GamerBase::handle_for_guide)
            .collect::<Result<Vec<_>>>()?;
        let properties = write_properties(&runtime, searchProperties)?;
        let (pointer, count) = handle_array(&handles)?;
        let adopted = runtime.clone();
        let outcome = with_callback(asyncState, callback, |trampoline, context| {
            let mut collection = 0;
            // SAFETY: the array describes exactly `count` live gamer handles.
            runtime.check(unsafe {
                (runtime
                    .native()
                    .net
                    .network_session_find_with_local_gamers_async)(
                    sessionType as u32,
                    pointer,
                    count,
                    properties,
                    trampoline,
                    context,
                    &mut collection,
                )
            })?;
            Ok(AvailableNetworkSessionCollection::adopt(adopted, collection))
        });
        // SAFETY: the properties handle is this call's to release.
        let _ = unsafe { (runtime.native().net.network_session_properties_destroy)(properties) };
        Ok(outcome?.0)
    }

    /// XNA `NetworkSession.EndFind`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated.
    pub fn EndFind(result: &GamerAsyncResult) -> Result<AvailableNetworkSessionCollection> {
        result.end_once::<AvailableNetworkSessionCollection>()
    }

    /// XNA `NetworkSession.BeginJoin`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginJoin(
        availableSession: &AvailableNetworkSession,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = net_runtime()?;
        let discovered = availableSession.handle()?;
        let adopted = runtime.clone();
        let (result, _fired) = with_callback(asyncState, callback, |trampoline, context| {
            let mut handle = 0;
            // SAFETY: the discovered-session handle is live for the call.
            runtime.check(unsafe {
                (runtime.native().net.network_session_join_async)(
                    discovered,
                    trampoline,
                    context,
                    &mut handle,
                )
            })?;
            Ok(Self::adopt(adopted, handle))
        })?;
        Ok(result)
    }

    /// XNA `NetworkSession.EndJoin`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated.
    pub fn EndJoin(result: &GamerAsyncResult) -> Result<Self> {
        result.end_once::<Self>()
    }

    /// XNA `NetworkSession.BeginJoinInvited(maxLocalGamers, ...)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginJoinInvited(
        maxLocalGamers: i32,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = net_runtime()?;
        let adopted = runtime.clone();
        let (result, _fired) = with_callback(asyncState, callback, |trampoline, context| {
            let mut handle = 0;
            // SAFETY: the argument is a plain scalar.
            runtime.check(unsafe {
                (runtime.native().net.network_session_join_invited_async)(
                    maxLocalGamers,
                    trampoline,
                    context,
                    &mut handle,
                )
            })?;
            Ok(Self::adopt(adopted, handle))
        })?;
        Ok(result)
    }

    /// XNA `NetworkSession.BeginJoinInvited(localGamers, ...)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginJoinInvitedWithLocalGamersAndCallbackAndAsyncState(
        localGamers: &[SignedInGamer],
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = net_runtime()?;
        let handles = localGamers
            .iter()
            .map(GamerBase::handle_for_guide)
            .collect::<Result<Vec<_>>>()?;
        let (pointer, count) = handle_array(&handles)?;
        let adopted = runtime.clone();
        let (result, _fired) = with_callback(asyncState, callback, |trampoline, context| {
            let mut handle = 0;
            // SAFETY: the array describes exactly `count` live gamer handles.
            runtime.check(unsafe {
                (runtime
                    .native()
                    .net
                    .network_session_join_invited_with_local_gamers_async)(
                    pointer, count, trampoline, context, &mut handle,
                )
            })?;
            Ok(Self::adopt(adopted, handle))
        })?;
        Ok(result)
    }

    /// XNA `NetworkSession.EndJoinInvited`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated.
    pub fn EndJoinInvited(result: &GamerAsyncResult) -> Result<Self> {
        result.end_once::<Self>()
    }

    /// XNA `NetworkSession.InviteAccepted` subscription.
    ///
    /// The event is static in XNA and process-global in CNA, so the
    /// subscription belongs to the process rather than to one session.
    #[must_use]
    pub fn AddInviteAcceptedHandler(
        handler: Box<dyn EventHandler<InviteAcceptedEventArgs>>,
    ) -> u64 {
        crate::gamer_services::add_invite_accepted(handler)
    }

    /// XNA `NetworkSession.InviteAccepted` removal.
    #[must_use]
    pub fn RemoveInviteAcceptedHandler(registration: u64) -> bool {
        crate::gamer_services::remove_invite_accepted(registration)
    }
}

fn handle_array(handles: &[sys::CNA_Handle]) -> Result<(*const sys::CNA_Handle, u64)> {
    let count = u64::try_from(handles.len())
        .map_err(|_| CnaError::InvalidInput("the local gamer array is too large"))?;
    let pointer = if handles.is_empty() {
        core::ptr::null()
    } else {
        handles.as_ptr()
    };
    Ok((pointer, count))
}
