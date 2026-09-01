//! Microsoft XNA 4.0 `GamerServices` and `Avatar`.
//!
//! These belong to the wider Windows runtime profile
//! (`tools/api-compat/profiles/xna40-windows-full.json`), not the selected
//! seven-assembly profile.
//!
//! The family divides on whether CLR metadata is the whole contract.
//! [`values`] is pure managed Rust because an enum ordinal or an exception
//! identity has nothing behind it. Everything else is an object graph over
//! `gamer_services.h`, and each handle in it carries one measured ownership
//! policy recorded next to the type that holds it.
//!
//! Every route here is process-global upstream: CNA's gamer services take no
//! game handle, exactly as XNA's `Gamer`, `Guide` and `GamerServicesDispatcher`
//! are static. Nothing in this module invents a signed-in gamer, a friend, a
//! profile or a Guide screen the host does not have. A headless host has no
//! gamer service, and an empty roster is the true answer rather than a
//! refusal.

mod achievements;
mod async_result;
mod avatar;
mod core;
mod dispatcher;
mod events;
mod gamer;
mod leaderboards;
mod values;

pub(crate) use self::core::GamerServicesRuntime as GamerServicesRuntimeHandle;
pub(crate) use async_result::with_callback;
pub(crate) use events::{add_invite_accepted, remove_invite_accepted};
pub(crate) use gamer::string_view as borrow_string;
pub(crate) use avatar::{animation_handle, renderer_handle};
pub(crate) use gamer::{adopt_friend_collection, signed_in_handle};

/// Opens, or reuses, the process CNA library for the extension surface.
pub(crate) fn open_runtime() -> crate::error::Result<GamerServicesRuntimeHandle> {
    GamerServicesRuntimeHandle::open()
}

pub use achievements::{Achievement, AchievementCollection};
pub use dispatcher::{GamerServicesDispatcher, Guide};
pub use events::{InviteAcceptedEventArgs, SignedInEventArgs, SignedOutEventArgs};
pub use avatar::{AvatarAnimation, AvatarDescription, AvatarRenderer, IAvatarAnimation};
pub use async_result::{GamerAsyncCallback, GamerAsyncResult, GamerAsyncState};
pub use leaderboards::{
    LeaderboardEntry, LeaderboardReader, LeaderboardWriter, PropertyDictionary, PropertyValueKind,
};
pub use gamer::{
    FriendCollection, FriendGamer, GameDefaults, Gamer, GamerBase, GamerCollection,
    GamerCollectionBase, GamerCollectionEnumerator, GamerPresence, GamerPrivileges, GamerProfile,
    SignedInGamer, SignedInGamerCollection,
};
pub use values::{
    AvatarAnimationPreset, AvatarBodyType, AvatarBone, AvatarExpression, AvatarEye, AvatarEyebrow,
    AvatarMouth, AvatarRendererState, ControllerSensitivity, GameDifficulty, GamerPresenceMode,
    GameUpdateRequiredException, GamerPrivilegeException, GamerPrivilegeSetting,
    GamerServicesNotAvailableException, GamerZone, GuideAlreadyVisibleException,
    LeaderboardIdentity, LeaderboardKey, LeaderboardOutcome, MessageBoxIcon, NetworkException,
    NetworkExceptionBase, NetworkNotAvailableException, NotificationPosition, RacingCameraAngle,
};
