//! XNA's gamer object graph: `Gamer` and everything that derives from it.
//!
//! # Ownership
//!
//! | Handle | Policy | Why |
//! |---|---|---|
//! | `Gamer`, `SignedInGamer`, `FriendGamer` | owned | the caller created it, or CNA answered a fresh view handle it must release |
//! | a gamer read out of a `GamerCollection` | borrowed | `cna_gamer_collection_get_at` documents the handle as valid while the collection lives |
//! | `Gamer.SignedInGamers[i]` | owned view over a borrowed gamer | `cna_gamer_get_signed_in_gamer_at` answers a **new** handle aliasing the roster's gamer; releasing it releases the view, never the gamer |
//! | `GamerProfile` | owned | `cna_gamer_get_profile` answers an owned handle |
//! | `FriendCollection` | owned | `cna_signed_in_gamer_get_friends` answers an owned collection that keeps its friends alive |
//!
//! A borrowed gamer therefore never gets a `Drop` that destroys anything, and
//! the roster view's `Drop` releases the view alone. Nothing in this module
//! destroys a handle it did not receive ownership of.
//!
//! # Identity
//!
//! CNA answers a different handle for each read of the same roster position,
//! so raw handle equality is not gamer identity here. The static
//! [`SignedInGamerCollection`] therefore caches one facade per position and
//! re-checks the cached entry's gamertag against the live roster before
//! answering, which keeps `Gamer.SignedInGamers[0]` the same logical object
//! across reads while a roster change still replaces it.

#![allow(non_snake_case)]

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::disposal::Disposable;
use crate::error::{CnaError, Result};
use crate::extensions::events::EventHandler;
use crate::input::PlayerIndex;
use crate::value::Color;

use super::async_result::{GamerAsyncCallback, GamerAsyncResult, GamerAsyncState};
use super::achievements::AchievementCollection;
use super::events::{SignedInEventArgs, SignedOutEventArgs};
use super::core::{read_owned_string, GamerServicesRuntime, OwnedHandle};
use super::leaderboards::LeaderboardWriter;
use super::values::{
    ControllerSensitivity, GameDifficulty, GamerPresenceMode, GamerPrivilegeSetting, GamerZone,
    RacingCameraAngle,
};

/// The public contract every XNA gamer inherits from `Gamer`.
///
/// XNA's `Gamer` is a class other gamer types derive from. Rust has no class
/// inheritance, so the projection composes `Gamer` and states the relationship
/// through this trait, exactly as the graphics and component families do.
pub trait GamerBase {
    /// XNA `Gamer.Gamertag`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, including the disposed error for a
    /// gamer whose handle has been released.
    fn Gamertag(&self) -> Result<String>;

    /// XNA `Gamer.DisplayName`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn DisplayName(&self) -> Result<String>;

    /// XNA `Gamer.Tag`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn Tag(&self) -> Result<u64>;

    /// XNA `Gamer.Tag` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn SetTag(&self, value: u64) -> Result<()>;

    /// XNA `Gamer.IsDisposed`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn IsDisposed(&self) -> Result<bool>;

    /// XNA `Gamer.ToString`, which is the gamer's display name.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn ToString(&self) -> Result<String>;

    /// XNA `Gamer.GetProfile`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn GetProfile(&self) -> Result<GamerProfile>;

    /// XNA `Gamer.LeaderboardWriter`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn LeaderboardWriter(&self) -> Result<&LeaderboardWriter>;

    /// The live handle a Guide screen needs, without exposing it publicly.
    #[doc(hidden)]
    fn handle_for_guide(&self) -> Result<sys::CNA_Handle>;
}

/// One gamer's shared state.
///
/// `Arc` rather than a plain handle because a gamer is reachable from several
/// places at once -- a collection facade, a network gamer, a leaderboard entry
/// -- and all of them must see one disposal.
#[derive(Clone, Debug)]
pub(crate) struct GamerCore {
    /// The handle whose lifetime bounds this gamer: its own when owned, its
    /// collection's when borrowed.
    owner: Arc<OwnedHandle>,
    /// Set only for a gamer the owner holds rather than is. A borrowed gamer
    /// never releases anything; releasing the collection is what ends it.
    borrowed: Option<sys::CNA_Handle>,
    /// XNA's `Gamer.LeaderboardWriter` is the gamer's own object, created on
    /// first ask and then stable, so edits through it accumulate.
    writer: Arc<std::sync::OnceLock<LeaderboardWriter>>,
}

impl GamerCore {
    pub(crate) fn adopt(runtime: GamerServicesRuntime, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime.native().gamer_services.gamer_destroy;
        Self {
            owner: OwnedHandle::new(runtime, handle, destroy),
            borrowed: None,
            writer: Arc::new(std::sync::OnceLock::new()),
        }
    }

    /// A gamer the parent collection owns.
    ///
    /// `cna_gamer_collection_get_at` documents its handle as valid while the
    /// collection lives, so the parent is retained and the element has no
    /// release of its own.
    pub(crate) fn borrowed(parent: Arc<OwnedHandle>, handle: sys::CNA_Handle) -> Self {
        Self {
            owner: parent,
            borrowed: Some(handle),
            writer: Arc::new(std::sync::OnceLock::new()),
        }
    }

    /// Adopts a handle CNA releases through the signed-in gamer route.
    ///
    /// A roster view and a caller-created signed-in gamer are both released
    /// this way; the route refuses while the process roster still publishes
    /// the gamer, which is the state that keeps a published gamer alive.
    pub(crate) fn adopt_signed_in(runtime: GamerServicesRuntime, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime.native().gamer_services.signed_in_gamer_destroy;
        Self {
            owner: OwnedHandle::new(runtime, handle, destroy),
            borrowed: None,
            writer: Arc::new(std::sync::OnceLock::new()),
        }
    }

    pub(crate) fn owner(&self) -> &Arc<OwnedHandle> {
        &self.owner
    }

    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
        match self.borrowed {
            // A borrowed gamer is exactly as valid as the collection holding
            // it, so the parent is what gets checked.
            Some(handle) => self.owner.get().map(|_| handle),
            None => self.owner.get(),
        }
    }

    fn is_released(&self) -> bool {
        self.owner.is_released()
    }

    pub(crate) fn runtime(&self) -> &GamerServicesRuntime {
        self.owner.runtime()
    }

    fn api(&self) -> &crate::native::gamer_services::GamerServicesApi {
        &self.owner.native().gamer_services
    }

    fn leaderboard_writer(&self) -> Result<&LeaderboardWriter> {
        Ok(self
            .writer
            .get_or_init(|| LeaderboardWriter::for_gamer(self.clone())))
    }

    fn Gamertag(&self) -> Result<String> {
        let api = self.api();
        let (size, copy) = (api.gamer_get_gamertag_size, api.gamer_copy_gamertag);
        // SAFETY: both routes take this gamer handle and a caller buffer.
        self.read_string(
            |handle, bytes| unsafe { size(handle, bytes) },
            |handle, destination, capacity, written| unsafe {
                copy(handle, destination, capacity, written)
            },
        )
    }

    fn DisplayName(&self) -> Result<String> {
        let api = self.api();
        let (size, copy) = (api.gamer_get_display_name_size, api.gamer_copy_display_name);
        // SAFETY: both routes take this gamer handle and a caller buffer.
        self.read_string(
            |handle, bytes| unsafe { size(handle, bytes) },
            |handle, destination, capacity, written| unsafe {
                copy(handle, destination, capacity, written)
            },
        )
    }

    fn Text(&self) -> Result<String> {
        let api = self.api();
        let (size, copy) = (api.gamer_get_text_size, api.gamer_copy_text);
        // SAFETY: both routes take this gamer handle and a caller buffer.
        self.read_string(
            |handle, bytes| unsafe { size(handle, bytes) },
            |handle, destination, capacity, written| unsafe {
                copy(handle, destination, capacity, written)
            },
        )
    }

    fn Tag(&self) -> Result<u64> {
        let handle = self.handle()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner
            .check(unsafe { (self.api().gamer_get_tag)(handle, &mut value) })?;
        Ok(value)
    }

    fn SetTag(&self, value: u64) -> Result<()> {
        let handle = self.handle()?;
        // SAFETY: the handle is live and the tag is a plain scalar.
        self.owner
            .check(unsafe { (self.api().gamer_set_tag)(handle, value) })
    }

    fn read_string(
        &self,
        size: impl Fn(sys::CNA_Handle, *mut u64) -> sys::CNA_Result,
        copy: impl Fn(sys::CNA_Handle, *mut core::ffi::c_char, u64, *mut u64) -> sys::CNA_Result,
    ) -> Result<String> {
        let handle = self.handle()?;
        crate::native::runtime::read_string(
            |result| self.owner.check(result),
            |bytes| size(handle, bytes),
            |destination, capacity, written| copy(handle, destination, capacity, written),
        )
    }

    fn IsDisposed(&self) -> Result<bool> {
        if self.is_released() {
            return Ok(true);
        }
        let handle = self.handle()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner
            .check(unsafe { (self.api().gamer_get_is_disposed)(handle, &mut value) })?;
        Ok(value != 0)
    }

    fn GetProfile(&self) -> Result<GamerProfile> {
        let handle = self.handle()?;
        let mut profile = 0;
        // SAFETY: the handle is live and the output receives an owned handle.
        self.owner
            .check(unsafe { (self.api().gamer_get_profile)(handle, &mut profile) })?;
        Ok(GamerProfile::adopt(self.runtime().clone(), profile))
    }
}

macro_rules! gamer_base {
    ($name:ty) => {
        impl GamerBase for $name {
            fn Gamertag(&self) -> Result<String> {
                self.gamer.Gamertag()
            }

            fn DisplayName(&self) -> Result<String> {
                self.gamer.DisplayName()
            }

            fn Tag(&self) -> Result<u64> {
                self.gamer.Tag()
            }

            fn SetTag(&self, value: u64) -> Result<()> {
                self.gamer.SetTag(value)
            }

            fn IsDisposed(&self) -> Result<bool> {
                self.gamer.IsDisposed()
            }

            fn ToString(&self) -> Result<String> {
                self.gamer.Text()
            }

            fn GetProfile(&self) -> Result<GamerProfile> {
                self.gamer.GetProfile()
            }

            fn LeaderboardWriter(&self) -> Result<&LeaderboardWriter> {
                self.gamer.leaderboard_writer()
            }

            fn handle_for_guide(&self) -> Result<sys::CNA_Handle> {
                self.gamer.handle()
            }
        }
    };
}

/// XNA `Microsoft.Xna.Framework.GamerServices.Gamer`.
#[derive(Clone, Debug)]
pub struct Gamer {
    pub(crate) gamer: GamerCore,
}

gamer_base!(Gamer);

impl Gamer {
    pub(crate) fn from_core(gamer: GamerCore) -> Self {
        Self { gamer }
    }

    /// XNA `Gamer.Gamertag`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Gamertag(&self) -> Result<String> {
        self.gamer.Gamertag()
    }

    /// XNA `Gamer.DisplayName`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn DisplayName(&self) -> Result<String> {
        self.gamer.DisplayName()
    }

    /// XNA `Gamer.Tag`.
    ///
    /// XNA's tag is any boxed CLR value. CNA publishes a caller-owned 64-bit
    /// value instead, the same choice the graphics resources made: a
    /// pointer-sized key a caller can index its own table with, rather than an
    /// opaque box C cannot open.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Tag(&self) -> Result<u64> {
        self.gamer.Tag()
    }

    /// XNA `Gamer.Tag` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetTag(&self, value: u64) -> Result<()> {
        self.gamer.SetTag(value)
    }

    /// XNA `Gamer.IsDisposed`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsDisposed(&self) -> Result<bool> {
        self.gamer.IsDisposed()
    }

    /// XNA `Gamer.ToString`, which answers the display name.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ToString(&self) -> Result<String> {
        self.gamer.Text()
    }

    /// XNA `Gamer.GetProfile`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetProfile(&self) -> Result<GamerProfile> {
        self.gamer.GetProfile()
    }

    /// XNA `Gamer.BeginGetProfile`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, or the callback error when the
    /// completion callback panics.
    pub fn BeginGetProfile(
        &self,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let handle = self.gamer.handle()?;
        let owner = Arc::clone(self.gamer.owner());
        let route = owner.native().gamer_services.gamer_begin_get_profile;
        let runtime = self.gamer.runtime().clone();
        let (result, _fired) = super::async_result::with_callback(
            asyncState,
            callback,
            |trampoline, context| {
                let mut profile = 0;
                // SAFETY: the handle is live, the context outlives the call,
                // and the output receives an owned handle.
                owner.check(unsafe { route(handle, trampoline, context, &mut profile) })?;
                Ok(GamerProfile::adopt(runtime, profile))
            },
        )?;
        Ok(result)
    }

    /// XNA `Gamer.EndGetProfile`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated, or the identity
    /// error when the result belongs to another operation.
    pub fn EndGetProfile(&self, result: &GamerAsyncResult) -> Result<GamerProfile> {
        result.end_once::<GamerProfile>()
    }

    /// XNA `Gamer.BeginGetFromGamertag`.
    ///
    /// # Errors
    ///
    /// Returns CNA's refusal on a runtime with no directory service.
    pub fn BeginGetFromGamertag(
        gamertag: &str,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = GamerServicesRuntime::open()?;
        let view = string_view(gamertag)?;
        let route = runtime.native().gamer_services.gamer_begin_get_from_gamertag;
        let adopted = runtime.clone();
        let (result, _fired) = super::async_result::with_callback(
            asyncState,
            callback,
            |trampoline, context| {
                let mut handle = 0;
                // SAFETY: the view borrows `gamertag` for the call and the
                // context outlives it.
                runtime.check(unsafe { route(view.value, trampoline, context, &mut handle) })?;
                Ok(Self::from_core(GamerCore::adopt(adopted, handle)))
            },
        )?;
        Ok(result)
    }

    /// XNA `Gamer.EndGetFromGamertag`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated.
    pub fn EndGetFromGamertag(result: &GamerAsyncResult) -> Result<Gamer> {
        result.end_once::<Gamer>()
    }

    /// XNA `Gamer.BeginGetPartnerToken`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginGetPartnerToken(
        audienceUri: &str,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = GamerServicesRuntime::open()?;
        let view = string_view(audienceUri)?;
        let route = runtime.native().gamer_services.gamer_begin_get_partner_token;
        let (result, _fired) = super::async_result::with_callback(
            asyncState,
            callback,
            |trampoline, context| {
                // The canonical begin route both queries and copies through one
                // call, so the size is asked for first and the copy repeats it.
                let mut bytes = 0_u64;
                // SAFETY: a null destination with zero capacity is the
                // canonical size query; the view borrows `audienceUri`.
                runtime.check(unsafe {
                    route(view.value, trampoline, context, core::ptr::null_mut(), 0, &mut bytes)
                })?;
                let capacity = usize::try_from(bytes)
                    .map_err(|_| CnaError::InvalidInput("CNA text is too large"))?;
                if capacity == 0 {
                    return Ok(String::new());
                }
                let mut buffer = vec![0_u8; capacity];
                let mut written = 0_u64;
                // SAFETY: the destination has exactly the reported capacity.
                runtime.check(unsafe {
                    route(
                        view.value,
                        None,
                        core::ptr::null_mut(),
                        buffer.as_mut_ptr().cast::<core::ffi::c_char>(),
                        bytes,
                        &mut written,
                    )
                })?;
                let written = usize::try_from(written)
                    .map_err(|_| CnaError::InvalidInput("CNA text is too large"))?;
                buffer.truncate(written.min(capacity));
                String::from_utf8(buffer)
                    .map_err(|_| CnaError::InvalidInput("CNA text is not valid UTF-8"))
            },
        )?;
        Ok(result)
    }

    /// XNA `Gamer.EndGetPartnerToken`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated.
    pub fn EndGetPartnerToken(result: &GamerAsyncResult) -> Result<String> {
        result.end_once::<String>()
    }

    /// XNA `Gamer.LeaderboardWriter`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn LeaderboardWriter(&self) -> Result<&LeaderboardWriter> {
        self.gamer.leaderboard_writer()
    }

    /// XNA `Gamer.SignedInGamers`, the process-wide signed-in roster.
    ///
    /// The collection is a facade over CNA's process-global roster, not a
    /// snapshot: it re-reads the roster on every access. On a host with no
    /// gamer service the roster is legitimately empty, and an empty collection
    /// is the answer rather than an error.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, including the loader error when no
    /// CNA library is available.
    pub fn SignedInGamers() -> Result<&'static SignedInGamerCollection> {
        // XNA hands out one collection object, so the facade is process-wide
        // and stable. It holds no snapshot: every read goes to CNA's roster.
        static ROSTER: std::sync::OnceLock<SignedInGamerCollection> = std::sync::OnceLock::new();
        if let Some(existing) = ROSTER.get() {
            return Ok(existing);
        }
        let created = SignedInGamerCollection::process()?;
        Ok(ROSTER.get_or_init(|| created))
    }

    /// XNA `Gamer.GetFromGamertag`.
    ///
    /// # Errors
    ///
    /// Returns CNA's refusal on a runtime with no directory service, which is
    /// every runtime this ABI builds on. The projection reports that refusal
    /// rather than inventing a gamer or answering an empty result.
    pub fn GetFromGamertag(gamertag: &str) -> Result<Self> {
        let runtime = GamerServicesRuntime::open()?;
        let view = string_view(gamertag)?;
        let mut handle = 0;
        // SAFETY: the view borrows `gamertag` for the duration of the call.
        runtime.check(unsafe {
            (runtime.native().gamer_services.gamer_get_from_gamertag)(view.value, &mut handle)
        })?;
        Ok(Self::from_core(GamerCore::adopt(runtime, handle)))
    }

    /// XNA `Gamer.GetPartnerToken`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetPartnerToken(audienceUri: &str) -> Result<String> {
        let runtime = GamerServicesRuntime::open()?;
        let view = string_view(audienceUri)?;
        let api = &runtime.native().gamer_services;
        let (size, copy) = (
            api.gamer_get_partner_token_size,
            api.gamer_copy_partner_token,
        );
        crate::native::runtime::read_string(
            |result| runtime.check(result),
            // SAFETY: the view borrows `audienceUri` for the call.
            |bytes| unsafe { size(view.value, bytes) },
            // SAFETY: the destination has the reported capacity.
            |destination, capacity, written| unsafe {
                copy(view.value, destination, capacity, written)
            },
        )
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.SignedInGamer`.
///
/// XNA declares no public constructor: a signed-in gamer arrives from the
/// platform. This projection declares none either, so a headless host reports
/// an empty roster instead of a fabricated player. CNA's own
/// `cna_signed_in_gamer_create_ext` is how a platform layer publishes one, and
/// it is exposed as a deliberate CNA extension rather than as an XNA member.
#[derive(Clone, Debug)]
pub struct SignedInGamer {
    pub(crate) gamer: GamerCore,
}

gamer_base!(SignedInGamer);

impl SignedInGamer {
    pub(crate) fn from_core(gamer: GamerCore) -> Self {
        Self { gamer }
    }

    fn api(&self) -> &crate::native::gamer_services::GamerServicesApi {
        &self.gamer.owner().native().gamer_services
    }

    /// XNA `SignedInGamer.PlayerIndex`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, or the mapping error for a slot
    /// identity outside XNA's four.
    pub fn PlayerIndex(&self) -> Result<PlayerIndex> {
        let handle = self.gamer.handle()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.gamer
            .owner()
            .check(unsafe { (self.api().signed_in_gamer_get_player_index)(handle, &mut value) })?;
        match value {
            sys::CNA_PLAYER_INDEX_ONE => Ok(PlayerIndex::One),
            sys::CNA_PLAYER_INDEX_TWO => Ok(PlayerIndex::Two),
            sys::CNA_PLAYER_INDEX_THREE => Ok(PlayerIndex::Three),
            sys::CNA_PLAYER_INDEX_FOUR => Ok(PlayerIndex::Four),
            _ => Err(CnaError::InvalidInput(
                "CNA reported a player slot XNA does not declare",
            )),
        }
    }

    /// XNA `SignedInGamer.IsSignedInToLive`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsSignedInToLive(&self) -> Result<bool> {
        self.flag(self.api().signed_in_gamer_get_is_signed_in_to_live)
    }

    /// XNA `SignedInGamer.IsGuest`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsGuest(&self) -> Result<bool> {
        self.flag(self.api().signed_in_gamer_get_is_guest)
    }

    /// XNA `SignedInGamer.PartySize`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn PartySize(&self) -> Result<i32> {
        let handle = self.gamer.handle()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.gamer
            .owner()
            .check(unsafe { (self.api().signed_in_gamer_get_party_size)(handle, &mut value) })?;
        Ok(value)
    }

    /// XNA `SignedInGamer.Presence`.
    ///
    /// The value is read from CNA and written back through
    /// [`GamerPresence::SetPresenceMode`] and
    /// [`GamerPresence::SetPresenceValue`], so the object keeps XNA's mutable
    /// reference semantics rather than becoming a detached copy.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Presence(&self) -> Result<GamerPresence> {
        Ok(GamerPresence {
            gamer: self.gamer.clone(),
        })
    }

    /// XNA `SignedInGamer.Privileges`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Privileges(&self) -> Result<GamerPrivileges> {
        let handle = self.gamer.handle()?;
        let mut value = sys::CNA_GamerPrivileges {
            struct_size: core::mem::size_of::<sys::CNA_GamerPrivileges>() as u32,
            struct_version: 1,
            ..sys::CNA_GamerPrivileges::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.gamer
            .owner()
            .check(unsafe { (self.api().signed_in_gamer_get_privileges)(handle, &mut value) })?;
        Ok(GamerPrivileges(value))
    }

    /// XNA `SignedInGamer.GameDefaults`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GameDefaults(&self) -> Result<GameDefaults> {
        let handle = self.gamer.handle()?;
        let mut value = sys::CNA_GameDefaults {
            struct_size: core::mem::size_of::<sys::CNA_GameDefaults>() as u32,
            struct_version: 1,
            ..sys::CNA_GameDefaults::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.gamer
            .owner()
            .check(unsafe { (self.api().signed_in_gamer_get_game_defaults)(handle, &mut value) })?;
        Ok(GameDefaults(value))
    }

    /// XNA `SignedInGamer.IsFriend`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsFriend(&self, gamer: &Gamer) -> Result<bool> {
        let handle = self.gamer.handle()?;
        let other = gamer.gamer.handle()?;
        let mut value = 0;
        // SAFETY: both handles are live and the output is initialized.
        self.gamer.owner().check(unsafe {
            (self.api().signed_in_gamer_is_friend)(handle, other, &mut value)
        })?;
        Ok(value != 0)
    }

    /// XNA `SignedInGamer.GetFriends`.
    ///
    /// A host with no friend service answers an empty collection, and that
    /// success is the true answer rather than a refusal.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetFriends(&self) -> Result<FriendCollection> {
        let handle = self.gamer.handle()?;
        let mut collection = 0;
        // SAFETY: the handle is live and the output receives an owned handle.
        self.gamer
            .owner()
            .check(unsafe { (self.api().signed_in_gamer_get_friends)(handle, &mut collection) })?;
        Ok(FriendCollection {
            collection: GamerCollection::adopt(
                self.gamer.runtime().clone(),
                collection,
                |parent, handle| FriendGamer::from_core(GamerCore::borrowed(parent, handle)),
            ),
        })
    }

    /// XNA `SignedInGamer.IsHeadset`.
    ///
    /// XNA asks whether a microphone is this gamer's headset. CNA identifies a
    /// microphone by its enumeration index, so the projection takes that index
    /// rather than a `Microphone` the canonical route cannot accept.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsHeadset(&self, microphone: u64) -> Result<bool> {
        let handle = self.gamer.handle()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.gamer.owner().check(unsafe {
            (self.api().signed_in_gamer_is_headset)(handle, microphone, &mut value)
        })?;
        Ok(value != 0)
    }

    /// XNA `SignedInGamer.AwardAchievement`.
    ///
    /// CNA persists the award, so an achievement earned in one process run is
    /// still earned in the next.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn AwardAchievement(&self, achievementKey: &str) -> Result<()> {
        let handle = self.gamer.handle()?;
        let view = string_view(achievementKey)?;
        // SAFETY: the view borrows the key for the call.
        self.gamer
            .owner()
            .check(unsafe { (self.api().signed_in_gamer_award_achievement)(handle, view.value) })
    }

    /// XNA `SignedInGamer.BeginAwardAchievement`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginAwardAchievement(
        &self,
        achievementKey: &str,
        callback: Option<GamerAsyncCallback>,
        state: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let handle = self.gamer.handle()?;
        let owner = Arc::clone(self.gamer.owner());
        let view = string_view(achievementKey)?;
        let route = owner
            .native()
            .gamer_services
            .signed_in_gamer_begin_award_achievement;
        let (result, _fired) =
            super::async_result::with_callback(state, callback, |trampoline, context| {
                // SAFETY: the view borrows the key and the context outlives the call.
                owner.check(unsafe { route(handle, view.value, trampoline, context) })?;
                Ok(())
            })?;
        Ok(result)
    }

    /// XNA `SignedInGamer.EndAwardAchievement`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated.
    pub fn EndAwardAchievement(&self, result: &GamerAsyncResult) -> Result<()> {
        result.end_once::<()>()
    }

    /// XNA `SignedInGamer.GetAchievements`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetAchievements(&self) -> Result<AchievementCollection> {
        let handle = self.gamer.handle()?;
        let mut collection = 0;
        // SAFETY: the handle is live and the output receives an owned handle.
        self.gamer.owner().check(unsafe {
            (self.api().signed_in_gamer_get_achievements)(handle, &mut collection)
        })?;
        Ok(AchievementCollection::adopt(
            self.gamer.runtime().clone(),
            collection,
        ))
    }

    /// XNA `SignedInGamer.BeginGetAchievements`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginGetAchievements(
        &self,
        callback: Option<GamerAsyncCallback>,
        asyncState: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let handle = self.gamer.handle()?;
        let owner = Arc::clone(self.gamer.owner());
        let runtime = self.gamer.runtime().clone();
        let route = owner
            .native()
            .gamer_services
            .signed_in_gamer_begin_get_achievements;
        let (result, _fired) =
            super::async_result::with_callback(asyncState, callback, |trampoline, context| {
                let mut collection = 0;
                // SAFETY: the handle is live and the context outlives the call.
                owner.check(unsafe { route(handle, trampoline, context, &mut collection) })?;
                Ok(AchievementCollection::adopt(runtime, collection))
            })?;
        Ok(result)
    }

    /// XNA `SignedInGamer.EndGetAchievements`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated.
    pub fn EndGetAchievements(&self, result: &GamerAsyncResult) -> Result<AchievementCollection> {
        result.end_once::<AchievementCollection>()
    }

    /// XNA `SignedInGamer.SignedIn` subscription.
    ///
    /// The event is static in XNA and process-global in CNA, so the
    /// subscription belongs to the process rather than to one gamer facade.
    /// CNA is subscribed on the first handler and unsubscribed when the last
    /// one is removed.
    ///
    /// A CNA subscription this call could not establish is not lost: it is
    /// reported by `GamerServicesDispatcher::Update`, which is where the CLR
    /// delivers these events from. XNA's `+=` has no way to fail.
    #[must_use]
    pub fn AddSignedInHandler(handler: Box<dyn EventHandler<SignedInEventArgs>>) -> u64 {
        super::events::add_signed_in(handler)
    }

    /// XNA `SignedInGamer.SignedIn` removal.
    #[must_use]
    pub fn RemoveSignedInHandler(registration: u64) -> bool {
        super::events::remove_signed_in(registration)
    }

    /// XNA `SignedInGamer.SignedOut` subscription.
    #[must_use]
    pub fn AddSignedOutHandler(handler: Box<dyn EventHandler<SignedOutEventArgs>>) -> u64 {
        super::events::add_signed_out(handler)
    }

    /// XNA `SignedInGamer.SignedOut` removal.
    #[must_use]
    pub fn RemoveSignedOutHandler(registration: u64) -> bool {
        super::events::remove_signed_out(registration)
    }

    fn flag(
        &self,
        route: unsafe extern "C" fn(sys::CNA_Handle, *mut sys::CNA_Bool) -> sys::CNA_Result,
    ) -> Result<bool> {
        let handle = self.gamer.handle()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.gamer.owner().check(unsafe { route(handle, &mut value) })?;
        Ok(value != 0)
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.FriendGamer`.
#[derive(Clone, Debug)]
pub struct FriendGamer {
    pub(crate) gamer: GamerCore,
}

gamer_base!(FriendGamer);

macro_rules! friend_flags {
    ($($name:ident => $field:ident),+ $(,)?) => {
        impl FriendGamer {
            $(
                #[doc = concat!("XNA `FriendGamer.", stringify!($name), "`.")]
                ///
                /// # Errors
                ///
                /// Returns the exact error CNA reports.
                pub fn $name(&self) -> Result<bool> {
                    Ok(self.info()?.$field != 0)
                }
            )+
        }
    };
}

friend_flags! {
    IsOnline => is_online,
    IsPlaying => is_playing,
    IsJoinable => is_joinable,
    IsAway => is_away,
    IsBusy => is_busy,
    HasVoice => has_voice,
    FriendRequestReceivedFrom => friend_request_received_from,
    FriendRequestSentTo => friend_request_sent_to,
    InviteReceivedFrom => invite_received_from,
    InviteSentTo => invite_sent_to,
    InviteAccepted => invite_accepted,
    InviteRejected => invite_rejected,
}

impl FriendGamer {
    pub(crate) fn from_core(gamer: GamerCore) -> Self {
        Self { gamer }
    }

    fn info(&self) -> Result<sys::CNA_FriendGamerInfo> {
        let handle = self.gamer.handle()?;
        let mut value = sys::CNA_FriendGamerInfo {
            struct_size: core::mem::size_of::<sys::CNA_FriendGamerInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_FriendGamerInfo::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.gamer.owner().check(unsafe {
            (self.gamer.owner().native().gamer_services.friend_gamer_get_info)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// XNA `FriendGamer.Presence`, which is a free-text status string rather
    /// than the structured [`GamerPresence`] a signed-in gamer publishes.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Presence(&self) -> Result<String> {
        let api = &self.gamer.owner().native().gamer_services;
        let (size, copy) = (
            api.friend_gamer_get_presence_size,
            api.friend_gamer_copy_presence,
        );
        // SAFETY: both routes take this gamer handle and a caller buffer.
        self.gamer.read_string(
            |handle, bytes| unsafe { size(handle, bytes) },
            |handle, destination, capacity, written| unsafe {
                copy(handle, destination, capacity, written)
            },
        )
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.GamerPresence`.
///
/// A live view of one signed-in gamer's presence, not a copy: XNA hands out
/// the gamer's own presence object, and writing through it changes the gamer.
#[derive(Clone, Debug)]
pub struct GamerPresence {
    gamer: GamerCore,
}

impl GamerPresence {
    fn read(&self) -> Result<sys::CNA_GamerPresence> {
        let handle = self.gamer.handle()?;
        let mut value = sys::CNA_GamerPresence {
            struct_size: core::mem::size_of::<sys::CNA_GamerPresence>() as u32,
            struct_version: 1,
            ..sys::CNA_GamerPresence::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.gamer.owner().check(unsafe {
            (self.gamer.owner().native().gamer_services.signed_in_gamer_get_presence)(
                handle, &mut value,
            )
        })?;
        Ok(value)
    }

    fn write(&self, value: &sys::CNA_GamerPresence) -> Result<()> {
        let handle = self.gamer.handle()?;
        // SAFETY: the handle is live and the descriptor is versioned.
        self.gamer.owner().check(unsafe {
            (self.gamer.owner().native().gamer_services.signed_in_gamer_set_presence)(handle, value)
        })
    }

    /// XNA `GamerPresence.PresenceMode`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, or the mapping error for a mode
    /// identity XNA does not declare.
    pub fn PresenceMode(&self) -> Result<GamerPresenceMode> {
        let value = self.read()?;
        GamerPresenceMode::from_native(value.presence_mode).ok_or(CnaError::InvalidInput(
            "CNA reported a presence mode XNA does not declare",
        ))
    }

    /// XNA `GamerPresence.PresenceMode` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetPresenceMode(&self, value: GamerPresenceMode) -> Result<()> {
        let mut presence = self.read()?;
        presence.presence_mode = value as u32;
        self.write(&presence)
    }

    /// XNA `GamerPresence.PresenceValue`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn PresenceValue(&self) -> Result<i32> {
        Ok(self.read()?.presence_value)
    }

    /// XNA `GamerPresence.PresenceValue` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetPresenceValue(&self, value: i32) -> Result<()> {
        let mut presence = self.read()?;
        presence.presence_value = value;
        self.write(&presence)
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.GamerPrivileges`.
///
/// A read-only snapshot: XNA exposes only getters, and CNA answers the whole
/// privilege set in one descriptor.
#[derive(Clone, Copy, Debug)]
pub struct GamerPrivileges(sys::CNA_GamerPrivileges);

macro_rules! privilege_flags {
    ($($name:ident => $field:ident),+ $(,)?) => {
        impl GamerPrivileges {
            $(
                #[doc = concat!("XNA `GamerPrivileges.", stringify!($name), "`.")]
                ///
                /// # Errors
                ///
                /// Never fails; the family reports through `Result` so one
                /// gamer object reads the same way throughout.
                pub const fn $name(&self) -> Result<bool> {
                    Ok(self.0.$field != 0)
                }
            )+
        }
    };
}

privilege_flags! {
    AllowOnlineSessions => allow_online_sessions,
    AllowTradeContent => allow_trade_content,
    AllowPurchaseContent => allow_purchase_content,
    AllowPremiumContent => allow_premium_content,
}

impl GamerPrivileges {
    /// XNA `GamerPrivileges.AllowCommunication`.
    ///
    /// # Errors
    ///
    /// Returns the mapping error for a setting XNA does not declare.
    pub fn AllowCommunication(&self) -> Result<GamerPrivilegeSetting> {
        privilege_setting(self.0.allow_communication)
    }

    /// XNA `GamerPrivileges.AllowProfileViewing`.
    ///
    /// # Errors
    ///
    /// Returns the mapping error for a setting XNA does not declare.
    pub fn AllowProfileViewing(&self) -> Result<GamerPrivilegeSetting> {
        privilege_setting(self.0.allow_profile_viewing)
    }

    /// XNA `GamerPrivileges.AllowUserCreatedContent`.
    ///
    /// # Errors
    ///
    /// Returns the mapping error for a setting XNA does not declare.
    pub fn AllowUserCreatedContent(&self) -> Result<GamerPrivilegeSetting> {
        privilege_setting(self.0.allow_user_created_content)
    }
}

fn privilege_setting(value: u32) -> Result<GamerPrivilegeSetting> {
    match value {
        sys::CNA_GAMER_PRIVILEGE_SETTING_BLOCKED => Ok(GamerPrivilegeSetting::Blocked),
        sys::CNA_GAMER_PRIVILEGE_SETTING_EVERYONE => Ok(GamerPrivilegeSetting::Everyone),
        sys::CNA_GAMER_PRIVILEGE_SETTING_FRIENDS_ONLY => Ok(GamerPrivilegeSetting::FriendsOnly),
        _ => Err(CnaError::InvalidInput(
            "CNA reported a privilege setting XNA does not declare",
        )),
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.GameDefaults`.
///
/// A read-only snapshot of one gamer's game preferences.
#[derive(Clone, Copy, Debug)]
pub struct GameDefaults(sys::CNA_GameDefaults);

macro_rules! game_default_flags {
    ($($name:ident => $field:ident),+ $(,)?) => {
        impl GameDefaults {
            $(
                #[doc = concat!("XNA `GameDefaults.", stringify!($name), "`.")]
                ///
                /// # Errors
                ///
                /// Never fails; the whole family reports through `Result` so a
                /// caller reads one gamer object the same way throughout.
                pub const fn $name(&self) -> Result<bool> {
                    Ok(self.0.$field != 0)
                }
            )+
        }
    };
}

game_default_flags! {
    AutoAim => auto_aim,
    AutoCenter => auto_center,
    MoveWithRightThumbStick => move_with_right_thumb_stick,
    InvertYAxis => invert_y_axis,
    ManualTransmission => manual_transmission,
    AccelerateWithButtons => accelerate_with_buttons,
    BrakeWithButtons => brake_with_buttons,
}

impl GameDefaults {
    /// XNA `GameDefaults.GameDifficulty`.
    ///
    /// # Errors
    ///
    /// Returns the mapping error for a difficulty XNA does not declare.
    pub fn GameDifficulty(&self) -> Result<GameDifficulty> {
        match self.0.game_difficulty {
            sys::CNA_GAME_DIFFICULTY_EASY => Ok(GameDifficulty::Easy),
            sys::CNA_GAME_DIFFICULTY_NORMAL => Ok(GameDifficulty::Normal),
            sys::CNA_GAME_DIFFICULTY_HARD => Ok(GameDifficulty::Hard),
            _ => Err(CnaError::InvalidInput(
                "CNA reported a difficulty XNA does not declare",
            )),
        }
    }

    /// XNA `GameDefaults.ControllerSensitivity`.
    ///
    /// # Errors
    ///
    /// Returns the mapping error for a sensitivity XNA does not declare.
    pub fn ControllerSensitivity(&self) -> Result<ControllerSensitivity> {
        match self.0.controller_sensitivity {
            sys::CNA_CONTROLLER_SENSITIVITY_LOW => Ok(ControllerSensitivity::Low),
            sys::CNA_CONTROLLER_SENSITIVITY_MEDIUM => Ok(ControllerSensitivity::Medium),
            sys::CNA_CONTROLLER_SENSITIVITY_HIGH => Ok(ControllerSensitivity::High),
            _ => Err(CnaError::InvalidInput(
                "CNA reported a controller sensitivity XNA does not declare",
            )),
        }
    }

    /// XNA `GameDefaults.RacingCameraAngle`.
    ///
    /// # Errors
    ///
    /// Returns the mapping error for a camera angle XNA does not declare.
    pub fn RacingCameraAngle(&self) -> Result<RacingCameraAngle> {
        match self.0.racing_camera_angle {
            sys::CNA_RACING_CAMERA_ANGLE_BACK => Ok(RacingCameraAngle::Back),
            sys::CNA_RACING_CAMERA_ANGLE_FRONT => Ok(RacingCameraAngle::Front),
            sys::CNA_RACING_CAMERA_ANGLE_INSIDE => Ok(RacingCameraAngle::Inside),
            _ => Err(CnaError::InvalidInput(
                "CNA reported a racing camera angle XNA does not declare",
            )),
        }
    }

    /// XNA `GameDefaults.PrimaryColor`.
    ///
    /// `None` is CLR `null`: the gamer expressed no preference. A colour is
    /// never invented for an unset preference.
    /// # Errors
    ///
    /// Never fails; the family reports through `Result` throughout.
    pub fn PrimaryColor(&self) -> Result<Option<Color>> {
        Ok((self.0.has_primary_color != 0).then(|| native_color(self.0.primary_color)))
    }

    /// XNA `GameDefaults.SecondaryColor`.
    ///
    /// `None` is CLR `null`, exactly as for [`GameDefaults::PrimaryColor`].
    /// # Errors
    ///
    /// Never fails; the family reports through `Result` throughout.
    pub fn SecondaryColor(&self) -> Result<Option<Color>> {
        Ok((self.0.has_secondary_color != 0).then(|| native_color(self.0.secondary_color)))
    }
}

fn native_color(value: sys::CNA_Color) -> Color {
    Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
        i32::from(value.r),
        i32::from(value.g),
        i32::from(value.b),
        i32::from(value.a),
    )
}

/// XNA `Microsoft.Xna.Framework.GamerServices.GamerProfile`.
///
/// Owned: `cna_gamer_get_profile` answers a handle the caller releases.
#[derive(Debug)]
pub struct GamerProfile {
    owner: Arc<OwnedHandle>,
}

impl GamerProfile {
    fn adopt(runtime: GamerServicesRuntime, handle: sys::CNA_Handle) -> Self {
        let destroy = runtime.native().gamer_services.gamer_profile_destroy;
        Self {
            owner: OwnedHandle::new(runtime, handle, destroy),
        }
    }

    fn info(&self) -> Result<sys::CNA_GamerProfileInfo> {
        let handle = self.owner.get()?;
        let mut value = sys::CNA_GamerProfileInfo {
            struct_size: core::mem::size_of::<sys::CNA_GamerProfileInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_GamerProfileInfo::default()
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.gamer_profile_get_info)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// XNA `GamerProfile.GamerScore`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GamerScore(&self) -> Result<i32> {
        Ok(self.info()?.gamer_score)
    }

    /// XNA `GamerProfile.TitlesPlayed`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn TitlesPlayed(&self) -> Result<i32> {
        Ok(self.info()?.titles_played)
    }

    /// XNA `GamerProfile.TotalAchievements`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn TotalAchievements(&self) -> Result<i32> {
        Ok(self.info()?.total_achievements)
    }

    /// XNA `GamerProfile.Reputation`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Reputation(&self) -> Result<f32> {
        Ok(self.info()?.reputation)
    }

    /// XNA `GamerProfile.IsDisposed`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsDisposed(&self) -> Result<bool> {
        if self.owner.is_released() {
            return Ok(true);
        }
        Ok(self.info()?.is_disposed != 0)
    }

    /// XNA `GamerProfile.GamerZone`.
    ///
    /// # Errors
    ///
    /// Returns the mapping error for a zone XNA does not declare.
    pub fn GamerZone(&self) -> Result<GamerZone> {
        match self.info()?.gamer_zone {
            sys::CNA_GAMER_ZONE_UNKNOWN => Ok(GamerZone::Unknown),
            sys::CNA_GAMER_ZONE_RECREATION => Ok(GamerZone::Recreation),
            sys::CNA_GAMER_ZONE_PRO => Ok(GamerZone::Pro),
            sys::CNA_GAMER_ZONE_FAMILY => Ok(GamerZone::Family),
            sys::CNA_GAMER_ZONE_UNDERGROUND => Ok(GamerZone::Underground),
            _ => Err(CnaError::InvalidInput(
                "CNA reported a gamer zone XNA does not declare",
            )),
        }
    }

    /// XNA `GamerProfile.Motto`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Motto(&self) -> Result<String> {
        let api = &self.owner.native().gamer_services;
        let (size, copy) = (
            api.gamer_profile_get_motto_size,
            api.gamer_profile_copy_motto,
        );
        // SAFETY: both routes take this profile handle and a caller buffer.
        read_owned_string(
            &self.owner,
            |handle, bytes| unsafe { size(handle, bytes) },
            |handle, destination, capacity, written| unsafe {
                copy(handle, destination, capacity, written)
            },
        )
    }

    /// XNA `GamerProfile.Region`, projected as its CLR region name.
    ///
    /// `System.Globalization.RegionInfo` has no Rust counterpart and no CNA
    /// representation beyond its name, so the projection answers the name
    /// rather than inventing a culture object.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Region(&self) -> Result<String> {
        let api = &self.owner.native().gamer_services;
        let (size, copy) = (
            api.gamer_profile_get_region_name_size,
            api.gamer_profile_copy_region_name,
        );
        // SAFETY: both routes take this profile handle and a caller buffer.
        read_owned_string(
            &self.owner,
            |handle, bytes| unsafe { size(handle, bytes) },
            |handle, destination, capacity, written| unsafe {
                copy(handle, destination, capacity, written)
            },
        )
    }

    /// XNA `GamerProfile.GetGamerPicture`, as the picture's byte length.
    ///
    /// CNA publishes only the size on this runtime, which is zero when no
    /// picture service exists. No stream is fabricated for a picture that is
    /// not there.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetGamerPicture(&self) -> Result<Option<u64>> {
        let handle = self.owner.get()?;
        let (mut has_picture, mut bytes) = (0, 0);
        // SAFETY: the handle is live and both outputs are initialized.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.gamer_profile_get_picture_size)(
                handle,
                &mut has_picture,
                &mut bytes,
            )
        })?;
        Ok((has_picture != 0).then_some(bytes))
    }

    /// XNA `GamerProfile.Dispose`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports; the handle is retained on failure
    /// so the release is still owed.
    pub fn Dispose(&self) -> Result<()> {
        self.owner.release()
    }
}

impl Disposable for GamerProfile {
    fn Dispose(&mut self) {
        let _ = GamerProfile::Dispose(&*self);
    }
}

impl Drop for GamerProfile {
    fn drop(&mut self) {
        let _ = self.owner.release();
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.GamerCollection<T>`.
///
/// A read-only view over an owned CNA gamer collection. Elements are borrowed
/// from the collection -- `cna_gamer_collection_get_at` documents the handle
/// as valid while the collection lives -- so the projection copies the values
/// it needs and never gives an element its own destroy.
#[derive(Debug)]
pub struct GamerCollection<T> {
    owner: Arc<OwnedHandle>,
    cache: Mutex<Vec<Option<T>>>,
    /// How to wrap one borrowed element. A stored constructor rather than a
    /// trait bound, so no crate-private trait appears in a public signature.
    element: fn(Arc<OwnedHandle>, sys::CNA_Handle) -> T,
}

/// The `ReadOnlyCollection<T>` contract every XNA gamer collection inherits.
///
/// `Count` and the integer indexer are declared by the BCL base rather than by
/// `GamerCollection<T>`, `SignedInGamerCollection` or `FriendCollection`, so
/// they arrive through this trait. That keeps them available on every gamer
/// collection without adding members Microsoft never declared on those types.
pub trait GamerCollectionBase<T> {
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
    /// Returns the range error the CLR indexer raises, or the exact error CNA
    /// reports.
    fn ItemAt(&self, index: i32) -> Result<T>;

    /// XNA `GamerCollection<T>.GetEnumerator`.
    ///
    /// Declared on `GamerCollection<T>` and inherited by the two sealed
    /// collections, so it reaches all three through this contract.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn Enumerate(&self) -> Result<GamerCollectionEnumerator<T>>;
}

impl<T: Clone> GamerCollectionBase<T> for GamerCollection<T> {
    fn Count(&self) -> Result<i32> {
        self.count()
    }

    fn ItemAt(&self, index: i32) -> Result<T> {
        self.item(index)
    }

    fn Enumerate(&self) -> Result<GamerCollectionEnumerator<T>> {
        self.GetEnumerator()
    }
}

impl<T: Clone> GamerCollection<T> {
    pub(crate) fn adopt(
        runtime: GamerServicesRuntime,
        handle: sys::CNA_Handle,
        element: fn(Arc<OwnedHandle>, sys::CNA_Handle) -> T,
    ) -> Self {
        let destroy = runtime.native().gamer_services.gamer_collection_destroy;
        Self {
            owner: OwnedHandle::new(runtime, handle, destroy),
            cache: Mutex::new(Vec::new()),
            element,
        }
    }

    fn count(&self) -> Result<i32> {
        let handle = self.owner.get()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.gamer_collection_get_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Repeated reads of one position answer the same logical gamer: the
    /// facade is cached, because CNA's borrowed element handle is stable while
    /// the collection lives.
    fn item(&self, index: i32) -> Result<T> {
        let count = self.count()?;
        if index < 0 || index >= count {
            return Err(CnaError::InvalidInput(
                "the gamer collection index is out of range",
            ));
        }
        let position = usize::try_from(index)
            .map_err(|_| CnaError::InvalidInput("the gamer collection index is out of range"))?;
        let mut cache = self
            .cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if cache.len() < count as usize {
            cache.resize_with(count as usize, || None);
        }
        if let Some(existing) = cache[position].clone() {
            return Ok(existing);
        }
        let handle = self.owner.get()?;
        let mut element = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.owner.check(unsafe {
            (self.owner.native().gamer_services.gamer_collection_get_at)(
                handle,
                index,
                &mut element,
            )
        })?;
        let value = (self.element)(Arc::clone(&self.owner), element);
        cache[position] = Some(value.clone());
        Ok(value)
    }

    /// XNA `GamerCollection<T>.GetEnumerator`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn GetEnumerator(&self) -> Result<GamerCollectionEnumerator<T>> {
        let count = self.count()?;
        let mut items = Vec::with_capacity(count.max(0) as usize);
        for index in 0..count {
            items.push(self.item(index)?);
        }
        Ok(GamerCollectionEnumerator::new(items))
    }
}

/// XNA `GamerCollection<T>.GamerCollectionEnumerator`.
///
/// A CLR value type over a snapshot of the collection. It is `Clone` but not
/// `Copy`: cloning a cursor is meaningful, while silently copying one on every
/// use would make `MoveNext` behave differently from the CLR struct it
/// projects.
#[derive(Clone, Debug)]
pub struct GamerCollectionEnumerator<T> {
    items: Vec<T>,
    position: Option<usize>,
    disposed: bool,
}

impl<T: Clone> GamerCollectionEnumerator<T> {
    fn new(items: Vec<T>) -> Self {
        Self {
            items,
            position: None,
            disposed: false,
        }
    }

    /// CLR `IEnumerator.MoveNext`.
    ///
    /// # Errors
    ///
    /// Returns the input error once the enumerator has been disposed.
    #[must_use]
    pub fn MoveNext(&mut self) -> bool {
        if self.disposed {
            return false;
        }
        let next = self.position.map_or(0, |value| value.saturating_add(1));
        self.position = Some(next);
        next < self.items.len()
    }

    /// CLR `IEnumerator<T>.Current`.
    ///
    /// # Panics
    ///
    /// Panics when there is no current element, which is what the CLR
    /// enumerator does before the first `MoveNext`, after the last one, and
    /// once it has been disposed.
    #[must_use]
    pub fn Current(&self) -> T {
        assert!(!self.disposed, "the gamer collection enumerator is disposed");
        self.position
            .and_then(|position| self.items.get(position))
            .cloned()
            .expect("the gamer collection enumerator has no current element")
    }

    /// CLR `IEnumerator.Dispose`.
    ///
    /// A value type with no owned handle, so this marks the cursor spent and
    /// the type deliberately gains no `Drop`.
    pub fn Dispose(&mut self) {
        self.disposed = true;
        self.position = None;
    }
}

impl<T: PartialEq> PartialEq for GamerCollectionEnumerator<T> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
            && self.position == other.position
            && self.disposed == other.disposed
    }
}

impl<T: Clone> Iterator for GamerCollectionEnumerator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.disposed {
            return None;
        }
        let next = self.position.map_or(0, |value| value.saturating_add(1));
        self.position = Some(next);
        self.items.get(next).cloned()
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.FriendCollection`.
#[derive(Debug)]
pub struct FriendCollection {
    collection: GamerCollection<FriendGamer>,
}

impl GamerCollectionBase<FriendGamer> for FriendCollection {
    fn Count(&self) -> Result<i32> {
        self.collection.count()
    }

    fn ItemAt(&self, index: i32) -> Result<FriendGamer> {
        self.collection.item(index)
    }

    fn Enumerate(&self) -> Result<GamerCollectionEnumerator<FriendGamer>> {
        self.collection.GetEnumerator()
    }
}

impl FriendCollection {
    /// XNA `FriendCollection.IsDisposed`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsDisposed(&self) -> Result<bool> {
        if self.collection.owner.is_released() {
            return Ok(true);
        }
        let handle = self.collection.owner.get()?;
        let mut value = 0;
        // SAFETY: the handle is live and the output is initialized.
        self.collection.owner.check(unsafe {
            (self
                .collection
                .owner
                .native()
                .gamer_services
                .friend_collection_get_is_disposed)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// XNA `FriendCollection.Dispose`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Dispose(&self) -> Result<()> {
        self.collection.owner.release()
    }

    /// XNA `FriendCollection.Finalize`.
    ///
    /// The CLR finalizer has no observable effect once `Dispose` and `Drop`
    /// both release exactly once; it exists here because Microsoft declared it.
    #[allow(clippy::unused_self)]
    pub fn Finalize(&self) {}
}

impl Disposable for FriendCollection {
    fn Dispose(&mut self) {
        let _ = FriendCollection::Dispose(&*self);
    }
}

impl Drop for FriendCollection {
    fn drop(&mut self) {
        // Idempotent: an explicit `Dispose` already cleared the handle, so this
        // is the safety net rather than a second release.
        let _ = self.collection.owner.release();
    }
}

impl IntoIterator for &FriendCollection {
    type Item = FriendGamer;
    type IntoIter = GamerCollectionEnumerator<FriendGamer>;

    fn into_iter(self) -> Self::IntoIter {
        GamerCollectionEnumerator::new(
            self.Enumerate().map_or_else(|_| Vec::new(), Iterator::collect),
        )
    }
}

impl<T: Clone> IntoIterator for &GamerCollection<T> {
    type Item = T;
    type IntoIter = GamerCollectionEnumerator<T>;

    fn into_iter(self) -> Self::IntoIter {
        GamerCollectionEnumerator::new(
            self.Enumerate().map_or_else(|_| Vec::new(), Iterator::collect),
        )
    }
}

impl IntoIterator for &SignedInGamerCollection {
    type Item = SignedInGamer;
    type IntoIter = GamerCollectionEnumerator<SignedInGamer>;

    fn into_iter(self) -> Self::IntoIter {
        GamerCollectionEnumerator::new(
            self.Enumerate().map_or_else(|_| Vec::new(), Iterator::collect),
        )
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.SignedInGamerCollection`.
///
/// A facade over CNA's process-global roster rather than a native collection
/// handle: CNA publishes the roster through position and player-index routes,
/// not through a collection object.
#[derive(Debug)]
pub struct SignedInGamerCollection {
    runtime: GamerServicesRuntime,
    cache: Mutex<Vec<(String, SignedInGamer)>>,
}

impl SignedInGamerCollection {
    fn process() -> Result<Self> {
        Ok(Self {
            runtime: GamerServicesRuntime::open()?,
            cache: Mutex::new(Vec::new()),
        })
    }

    fn count(&self) -> Result<i32> {
        let mut value = 0;
        // SAFETY: the output is initialized and the route is process-global.
        self.runtime.check(unsafe {
            (self
                .runtime
                .native()
                .gamer_services
                .gamer_get_signed_in_gamer_count)(&mut value)
        })?;
        Ok(value)
    }

    fn item_at(&self, index: i32) -> Result<SignedInGamer> {
        let count = self.count()?;
        if index < 0 || index >= count {
            return Err(CnaError::InvalidInput(
                "the signed-in gamer index is out of range",
            ));
        }
        let mut handle = 0;
        // SAFETY: the index was range-checked and the output is initialized.
        self.runtime.check(unsafe {
            (self
                .runtime
                .native()
                .gamer_services
                .gamer_get_signed_in_gamer_at)(index, &mut handle)
        })?;
        let fresh = SignedInGamer::from_core(GamerCore::adopt_signed_in(
            self.runtime.clone(),
            handle,
        ));
        let gamertag = fresh.Gamertag()?;
        let position = usize::try_from(index)
            .map_err(|_| CnaError::InvalidInput("the signed-in gamer index is out of range"))?;
        let mut cache = self
            .cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if cache.len() < count as usize {
            cache.resize_with(count as usize, || (String::new(), fresh.clone()));
        }
        // CNA answers a new view handle for every read, so handle equality is
        // not identity here. The cached facade is kept while the roster still
        // holds the same gamer at this position and replaced when it does not.
        if let Some((cached_tag, cached)) = cache.get(position) {
            if *cached_tag == gamertag && !cached.gamer.owner().is_released() {
                return Ok(cached.clone());
            }
        }
        cache[position] = (gamertag, fresh.clone());
        Ok(fresh)
    }

    /// XNA `SignedInGamerCollection.this[PlayerIndex]`.
    ///
    /// `None` is CLR `null`: no gamer is signed in on that controller slot.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Item(&self, index: PlayerIndex) -> Result<Option<SignedInGamer>> {
        let (mut has_gamer, mut handle) = (0, 0);
        // SAFETY: both outputs are initialized and the route is process-global.
        self.runtime.check(unsafe {
            (self
                .runtime
                .native()
                .gamer_services
                .gamer_get_signed_in_gamer_at_player_index)(
                index as u32, &mut has_gamer, &mut handle
            )
        })?;
        if has_gamer == 0 {
            return Ok(None);
        }
        Ok(Some(SignedInGamer::from_core(GamerCore::adopt_signed_in(
            self.runtime.clone(),
            handle,
        ))))
    }

}

impl GamerCollectionBase<SignedInGamer> for SignedInGamerCollection {
    fn Count(&self) -> Result<i32> {
        self.count()
    }

    fn ItemAt(&self, index: i32) -> Result<SignedInGamer> {
        self.item_at(index)
    }

    fn Enumerate(&self) -> Result<GamerCollectionEnumerator<SignedInGamer>> {
        let count = self.count()?;
        let mut items = Vec::with_capacity(count.max(0) as usize);
        for index in 0..count {
            items.push(self.item_at(index)?);
        }
        Ok(GamerCollectionEnumerator::new(items))
    }
}

/// A borrowed UTF-8 view over a Rust string for one canonical call.
pub(crate) struct StringView<'a> {
    pub(crate) value: sys::CNA_StringView,
    _borrow: core::marker::PhantomData<&'a str>,
}

/// Borrows a Rust string as a canonical view for the duration of one call.
///
/// # Errors
///
/// Returns the input error for a string CNA cannot measure.
pub(crate) fn string_view(value: &str) -> Result<StringView<'_>> {
    let byte_length = u64::try_from(value.len())
        .map_err(|_| CnaError::InvalidInput("the text is too large for CNA"))?;
    Ok(StringView {
        value: sys::CNA_StringView {
            data: value.as_ptr().cast::<core::ffi::c_char>(),
            byte_length,
        },
        _borrow: core::marker::PhantomData,
    })
}
