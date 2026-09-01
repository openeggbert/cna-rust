//! CNA's own gamer-services capabilities, and the call spellings XNA cannot
//! express in Rust.
//!
//! Two different things live here, and the difference matters.
//!
//! **Rust spelling.** XNA overloads `this[...]` by integer and by string.
//! Rust cannot give two methods one name, so the strict type keeps the
//! metadata-selected string form and the integer operation arrives through a
//! trait. Same collection, same handle, same identity rule.
//!
//! **A platform's own publishing routes.** XNA has no way for a game to create
//! a `SignedInGamer` -- the platform supplies them -- and this projection has
//! none either, because inventing one would be exactly the fabrication the
//! strict surface must not do. CNA publishes `_ext` routes so a *platform
//! layer* can populate the roster, and that is what [`SignedInGamerPublisher`]
//! is. It is deliberately outside `cna::Microsoft::Xna::Framework`: a game
//! that calls it is acting as its own platform, and after it does, every
//! strict `Gamer.SignedInGamers` read reports the real CNA roster rather than
//! anything this crate made up.

#![allow(clippy::missing_errors_doc, non_snake_case)]

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::models::SkinnedModel;
use crate::game::TimeSpan;
use crate::gamer_services::{
    Achievement, AchievementCollection, AvatarAnimation, AvatarAnimationPreset, AvatarBodyType,
    AvatarRenderer, FriendCollection, SignedInGamer,
};
use crate::graphics::GraphicsDevice;
use crate::value::Color;
use crate::input::PlayerIndex;

/// The first refusal `GamerServicesComponent` could not throw.
///
/// XNA's `GamerServicesComponent.Initialize` and `Update` are `void` and
/// throw. Both are `void` in this projection too, and there is nothing to
/// throw them into: the game's component collection calls them from a
/// lifecycle callback that has no `Result` to carry a failure. So the first
/// one is kept here, exactly once, and a host that wants to know reads it.
///
/// Only the *first* is kept. A dispatcher that cannot initialise will refuse
/// every subsequent `Update` as well, and the hundredth copy of that says
/// nothing the first did not.
static COMPONENT_ERROR: Mutex<Option<CnaError>> = Mutex::new(None);

pub(crate) fn record_component_error(error: CnaError) {
    let mut pending = COMPONENT_ERROR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if pending.is_none() {
        *pending = Some(error);
    }
}

/// Reports and clears the refusal `GamerServicesComponent` could not throw.
///
/// Returns `Ok(())` when the component has not failed since the last read.
///
/// # Errors
///
/// The first error CNA reported from the component's `Initialize` or `Update`.
pub fn TakeComponentError() -> Result<()> {
    COMPONENT_ERROR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
        .map_or(Ok(()), Err)
}

/// The integer indexer XNA's achievement collection also declares.
pub trait AchievementCollectionExt {
    /// The achievement at a zero-based position.
    fn item_at(&self, index: i32) -> Result<Achievement>;
}

impl AchievementCollectionExt for AchievementCollection {
    fn item_at(&self, index: i32) -> Result<Achievement> {
        AchievementCollection::item_at(self, index)
    }
}

/// CNA's achievement equality, which XNA does not declare.
pub trait AchievementExt {
    /// Whether two achievements are the same achievement by value.
    ///
    /// A collection answers an owned copy rather than a view, so this is how
    /// a caller checks that a copy it kept is still the collection's.
    fn equals(&self, other: &Achievement) -> Result<bool>;
}

impl AchievementExt for Achievement {
    fn equals(&self, other: &Self) -> Result<bool> {
        Achievement::equals(self, other)
    }
}

/// What a platform layer knows about one signed-in gamer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedInGamerRegistration {
    /// The gamer's tag. CNA copies it during publication.
    pub gamertag: String,
    /// Whether the gamer is signed in to the online service.
    pub is_signed_in_to_live: bool,
    /// Whether the gamer is a guest of another signed-in gamer.
    pub is_guest: bool,
    /// The controller slot the gamer occupies.
    pub player_index: PlayerIndex,
}

/// Publishes a roster of signed-in gamers the way a platform layer would.
///
/// The publisher owns every gamer it creates and keeps them alive for as long
/// as CNA's process-wide collection references them: CNA's collection holds
/// non-owning pointers, and `cna_signed_in_gamer_destroy` refuses while a
/// gamer is still published. Dropping the publisher therefore clears the
/// roster first and only then releases the gamers, in that order.
#[derive(Debug)]
pub struct SignedInGamerPublisher {
    runtime: crate::gamer_services::GamerServicesRuntimeHandle,
    gamers: Vec<Arc<PublishedGamer>>,
}

#[derive(Debug)]
struct PublishedGamer {
    runtime: crate::gamer_services::GamerServicesRuntimeHandle,
    handle: sys::CNA_Handle,
}

impl Drop for PublishedGamer {
    fn drop(&mut self) {
        // SAFETY: the handle came from `cna_signed_in_gamer_create_ext` and the
        // publisher cleared the roster before dropping, so the refusal that
        // guards a published gamer no longer applies.
        let _ = unsafe {
            (self
                .runtime
                .native()
                .gamer_services
                .signed_in_gamer_destroy)(self.handle)
        };
    }
}

impl SignedInGamerPublisher {
    /// Publishes exactly this roster, replacing whatever CNA held before.
    ///
    /// An empty roster is legal and is how a platform reports that everyone
    /// signed out.
    pub fn publish(gamers: &[SignedInGamerRegistration]) -> Result<Self> {
        let runtime = crate::gamer_services::open_runtime()?;
        let mut created = Vec::with_capacity(gamers.len());
        for gamer in gamers {
            let view = crate::gamer_services::borrow_string(&gamer.gamertag)?;
            let mut handle = 0;
            // SAFETY: the view borrows the tag for the call; CNA copies it.
            runtime.check(unsafe {
                (runtime.native().gamer_services.signed_in_gamer_create_ext)(
                    view.value,
                    u8::from(gamer.is_signed_in_to_live).into(),
                    u8::from(gamer.is_guest).into(),
                    gamer.player_index as u32,
                    &mut handle,
                )
            })?;
            created.push(Arc::new(PublishedGamer {
                runtime: runtime.clone(),
                handle,
            }));
        }
        let handles: Vec<sys::CNA_Handle> = created.iter().map(|gamer| gamer.handle).collect();
        let count = u64::try_from(handles.len())
            .map_err(|_| CnaError::InvalidInput("the roster is too large"))?;
        let pointer = if handles.is_empty() {
            core::ptr::null()
        } else {
            handles.as_ptr()
        };
        // SAFETY: the array describes exactly `count` live handles and CNA
        // retains each one for as long as the collection references it.
        runtime.check(unsafe {
            (runtime.native().gamer_services.gamer_set_signed_in_gamers_ext)(pointer, count)
        })?;
        Ok(Self {
            runtime,
            gamers: created,
        })
    }

    /// The gamers this publisher put on the roster, as strict XNA objects.
    pub fn gamers(&self) -> Result<Vec<SignedInGamer>> {
        let roster = crate::Microsoft::Xna::Framework::GamerServices::Gamer::SignedInGamers()?;
        use crate::GamerCollectionBase as _;
        let count = roster.Count()?;
        (0..count).map(|index| roster.ItemAt(index)).collect()
    }

    /// Clears CNA's roster without waiting for the publisher to drop.
    pub fn retire(&mut self) -> Result<()> {
        // SAFETY: an empty roster is the canonical "nobody is signed in".
        self.runtime.check(unsafe {
            (self
                .runtime
                .native()
                .gamer_services
                .gamer_set_signed_in_gamers_ext)(core::ptr::null(), 0)
        })?;
        self.gamers.clear();
        Ok(())
    }
}

impl Drop for SignedInGamerPublisher {
    fn drop(&mut self) {
        // The roster must lose its references before any gamer is released:
        // CNA refuses to destroy a gamer the collection still names.
        let _ = self.retire();
    }
}

/// CNA's own control over a Guide request that is waiting for a person.
///
/// XNA's `Guide.BeginShowMessageBox` and `Guide.BeginShowKeyboardInput` are
/// answered by someone pressing a button or typing. CNA keeps the request
/// *pending* rather than inventing an answer, and publishes these routes so a
/// host -- a platform layer, a test, or a game drawing its own Guide -- can
/// resolve it. They are CNA's, not XNA's, which is why they are here.
///
/// Until a request is resolved the matching `End*` reports CNA's
/// not-answered-yet state rather than a fabricated choice.
pub struct PendingGuideRequest;

impl PendingGuideRequest {
    /// Whether a message box is waiting to be answered.
    pub fn has_message_box() -> Result<bool> {
        Self::flag(|api| api.guide_get_has_pending_message_box_ext)
    }

    /// The button a pending message box focuses.
    pub fn message_box_focus_button() -> Result<i32> {
        let runtime = crate::gamer_services::open_runtime()?;
        let mut value = 0;
        // SAFETY: the output is initialized and the route is process-global.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .guide_get_pending_message_box_focus_button_ext)(&mut value)
        })?;
        Ok(value)
    }

    /// Answers a pending message box as if someone chose that button.
    pub fn click_message_box(buttonIndex: i32) -> Result<()> {
        let runtime = crate::gamer_services::open_runtime()?;
        // SAFETY: the index is a plain scalar CNA range-checks.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .guide_simulate_message_box_click_ext)(buttonIndex)
        })
    }

    /// Discards a pending message box without answering it.
    pub fn reset_message_box() -> Result<()> {
        let runtime = crate::gamer_services::open_runtime()?;
        // SAFETY: the route is process-global and takes nothing.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .guide_reset_pending_message_box_ext)()
        })
    }

    /// Whether a keyboard-input request is waiting to be answered.
    pub fn has_keyboard_input() -> Result<bool> {
        Self::flag(|api| api.guide_get_has_pending_keyboard_input_ext)
    }

    /// Whether the last keyboard-input request was cancelled.
    pub fn keyboard_input_was_canceled() -> Result<bool> {
        Self::flag(|api| api.guide_was_keyboard_input_canceled_ext)
    }

    /// The title a pending keyboard-input request carries.
    pub fn keyboard_input_title() -> Result<String> {
        Self::text(|api| {
            (
                api.guide_get_pending_keyboard_input_title_size_ext,
                api.guide_copy_pending_keyboard_input_title_ext,
            )
        })
    }

    /// The description a pending keyboard-input request carries.
    pub fn keyboard_input_description() -> Result<String> {
        Self::text(|api| {
            (
                api.guide_get_pending_keyboard_input_description_size_ext,
                api.guide_copy_pending_keyboard_input_description_ext,
            )
        })
    }

    /// The text a pending keyboard-input request currently displays.
    ///
    /// Password mode is CNA's to apply: this answers what the Guide would
    /// show, which for a password request is the masked form.
    pub fn keyboard_input_display_text() -> Result<String> {
        Self::text(|api| {
            (
                api.guide_get_pending_keyboard_input_display_text_size_ext,
                api.guide_copy_pending_keyboard_input_display_text_ext,
            )
        })
    }

    /// Cancels a pending keyboard-input request.
    pub fn cancel_keyboard_input() -> Result<()> {
        let runtime = crate::gamer_services::open_runtime()?;
        // SAFETY: the route is process-global and takes nothing.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .guide_simulate_keyboard_input_cancel_ext)()
        })
    }

    /// Discards a pending keyboard-input request.
    pub fn reset_keyboard_input() -> Result<()> {
        let runtime = crate::gamer_services::open_runtime()?;
        // SAFETY: the route is process-global and takes nothing.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .guide_reset_pending_keyboard_input_ext)()
        })
    }

    /// CNA's visibility setter, which the canonical layer accepts and ignores.
    ///
    /// XNA's `Guide.IsVisible` is read-only, and CNA derives it from whether a
    /// message box or keyboard request is pending rather than from a stored
    /// flag. The canonical setter exists for symmetry and does nothing; the
    /// route is published here with that stated rather than left unreachable
    /// or described as if it worked.
    pub fn set_visible(visible: bool) -> Result<()> {
        let runtime = crate::gamer_services::open_runtime()?;
        // SAFETY: the argument is a canonical boolean.
        runtime.check(unsafe {
            (runtime.native().gamer_services.guide_set_is_visible)(u8::from(visible).into())
        })
    }

    /// Publishes whether the title is running as a trial.
    ///
    /// XNA's `Guide.IsTrialMode` is read-only for the same reason.
    pub fn set_trial_mode(trial: bool) -> Result<()> {
        let runtime = crate::gamer_services::open_runtime()?;
        // SAFETY: the argument is a canonical boolean.
        runtime.check(unsafe {
            (runtime.native().gamer_services.guide_set_is_trial_mode)(u8::from(trial).into())
        })
    }

    fn flag(
        select: impl Fn(
            &crate::native::gamer_services::GamerServicesApi,
        ) -> unsafe extern "C" fn(*mut sys::CNA_Bool) -> sys::CNA_Result,
    ) -> Result<bool> {
        let runtime = crate::gamer_services::open_runtime()?;
        let route = select(&runtime.native().gamer_services);
        let mut value = 0;
        // SAFETY: the output is initialized and the route is process-global.
        runtime.check(unsafe { route(&mut value) })?;
        Ok(value != 0)
    }

    #[allow(clippy::type_complexity)]
    fn text(
        select: impl Fn(
            &crate::native::gamer_services::GamerServicesApi,
        ) -> (
            unsafe extern "C" fn(*mut u64) -> sys::CNA_Result,
            unsafe extern "C" fn(*mut core::ffi::c_char, u64, *mut u64) -> sys::CNA_Result,
        ),
    ) -> Result<String> {
        let runtime = crate::gamer_services::open_runtime()?;
        let (size, copy) = select(&runtime.native().gamer_services);
        crate::native::runtime::read_string(
            |value| runtime.check(value),
            // SAFETY: the size query takes only its output.
            |bytes| unsafe { size(bytes) },
            // SAFETY: the destination has the reported capacity.
            |destination, capacity, written| unsafe { copy(destination, capacity, written) },
        )
    }
}

/// Pumps the dispatcher and reports whether there was work to do.
///
/// XNA's `GamerServicesDispatcher.Update` is `void`: a game pumps it every
/// frame and never learns whether anything happened. CNA publishes the same
/// step with an answer, which is what a host driving the dispatcher outside a
/// `Game` loop -- a test, a tool, a headless service -- needs in order to stop.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn UpdateDispatcherAsync() -> Result<bool> {
    let runtime = crate::gamer_services::open_runtime()?;
    let mut value = 0;
    // SAFETY: the output is initialized and the route is process-global.
    runtime.check(unsafe {
        (runtime
            .native()
            .gamer_services
            .gamer_services_dispatcher_update_async)(&mut value)
    })?;
    Ok(value != 0)
}

/// How many gamer objects the dispatcher has released.
///
/// A counter CNA keeps for its own leak checks, and the only way from this side
/// to tell "the roster was retired" from "the roster was retired and the gamers
/// were freed".
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn FreedGamerCount() -> Result<u64> {
    let runtime = crate::gamer_services::open_runtime()?;
    let mut value = 0;
    // SAFETY: the output is initialized and the route is process-global.
    runtime.check(unsafe {
        (runtime
            .native()
            .gamer_services
            .gamer_services_dispatcher_get_freed_gamer_count_ext)(&mut value)
    })?;
    Ok(value)
}

/// Sets a gamer's presence from free text rather than one of XNA's 60 modes.
///
/// `GamerPresence.PresenceMode` is an enum, so XNA's presence can only ever say
/// one of the things Microsoft enumerated. CNA carries the mode as a string
/// underneath and publishes this to set it directly, which is how a platform
/// layer shows something XNA has no ordinal for.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn SetPresenceModeText(gamer: &SignedInGamer, mode: &str) -> Result<()> {
    let runtime = crate::gamer_services::open_runtime()?;
    let handle = crate::gamer_services::signed_in_handle(gamer)?;
    let view = crate::gamer_services::borrow_string(mode)?;
    // SAFETY: the handle is live and the view borrows the text for the call.
    runtime.check(unsafe {
        (runtime
            .native()
            .gamer_services
            .signed_in_gamer_set_presence_mode_string_ext)(handle, view.value)
    })
}

/// What a platform layer knows about one achievement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AchievementRegistration {
    /// The achievement's key, which is also how a game looks it up.
    pub key: String,
    /// The achievement's display name.
    pub name: String,
    /// The achievement's description.
    pub description: String,
    /// Whether the achievement is shown before it is earned.
    pub display_before_earned: bool,
    /// Whether this gamer has earned it.
    pub is_earned: bool,
    /// When it was earned, in `DateTime` ticks. Zero when it was not.
    pub earned_ticks: i64,
}

/// Achievements a platform layer supplies in place of a title catalog.
///
/// `SignedInGamer.AwardAchievement` produces a real achievement on this
/// runtime, but CNA has no catalog behind it: the name, description and score
/// come back empty, so nine of `Achievement`'s ten properties can only ever be
/// measured against nothing. These routes build one with real values, which is
/// what a platform layer with its own catalog would publish.
pub struct AchievementInjection;

impl AchievementInjection {
    /// Builds one achievement.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn achievement(achievement: &AchievementRegistration) -> Result<Achievement> {
        let runtime = crate::gamer_services::open_runtime()?;
        let handle = Self::create(&runtime, achievement)?;
        Ok(Achievement::adopt(runtime, handle))
    }

    /// Builds the collection `GetAchievements` would have answered.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn collection(achievements: &[AchievementRegistration]) -> Result<AchievementCollection> {
        let runtime = crate::gamer_services::open_runtime()?;
        let mut handles = Vec::with_capacity(achievements.len());
        for achievement in achievements {
            match Self::create(&runtime, achievement) {
                Ok(handle) => handles.push(handle),
                Err(error) => {
                    for handle in &handles {
                        // SAFETY: each came from the create route and is
                        // released once.
                        let _ = unsafe {
                            (runtime.native().gamer_services.achievement_destroy)(*handle)
                        };
                    }
                    return Err(error);
                }
            }
        }
        let count = u64::try_from(handles.len())
            .map_err(|_| CnaError::InvalidInput("too many achievements"))?;
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
                .gamer_services
                .achievement_collection_create_ext)(pointer, count, &mut collection)
        });
        if let Err(error) = created {
            for handle in &handles {
                // SAFETY: as above.
                let _ =
                    unsafe { (runtime.native().gamer_services.achievement_destroy)(*handle) };
            }
            return Err(error);
        }
        Ok(AchievementCollection::adopt(runtime, collection))
    }

    fn create(
        runtime: &crate::gamer_services::GamerServicesRuntimeHandle,
        achievement: &AchievementRegistration,
    ) -> Result<sys::CNA_Handle> {
        let key = crate::gamer_services::borrow_string(&achievement.key)?;
        let name = crate::gamer_services::borrow_string(&achievement.name)?;
        let description = crate::gamer_services::borrow_string(&achievement.description)?;
        let mut handle = 0;
        // SAFETY: every view borrows for the call and CNA copies its bytes.
        runtime.check(unsafe {
            (runtime.native().gamer_services.achievement_create_ext)(
                key.value,
                name.value,
                description.value,
                u8::from(achievement.display_before_earned).into(),
                u8::from(achievement.is_earned).into(),
                achievement.earned_ticks,
                &mut handle,
            )
        })?;
        Ok(handle)
    }
}

/// What a platform layer knows about one friend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FriendRegistration {
    /// The friend's gamertag.
    pub gamertag: String,
    /// The friend's display name.
    pub display_name: String,
    /// Whether the friend is signed in.
    pub is_online: bool,
    /// Whether the friend is in a game.
    pub is_playing: bool,
    /// Whether the friend is marked away.
    pub is_away: bool,
    /// Whether the friend is marked busy.
    pub is_busy: bool,
    /// Whether this gamer has sent them a friend request.
    pub friend_request_sent_to: bool,
    /// Whether they have sent this gamer a friend request.
    pub friend_request_received_from: bool,
}

/// Friends a platform layer supplies in place of a social service.
///
/// `SignedInGamer.GetFriends` reaches a real CNA collection, and on a host with
/// no social service that collection is empty -- so `FriendGamer`'s eight
/// states have nothing to be read from. These routes build the roster a
/// platform layer would have published.
pub struct FriendInjection;

impl FriendInjection {
    /// Builds the collection `GetFriends` would have answered.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn collection(friends: &[FriendRegistration]) -> Result<FriendCollection> {
        let runtime = crate::gamer_services::open_runtime()?;
        let mut handles = Vec::with_capacity(friends.len());
        for friend in friends {
            match Self::create(&runtime, friend) {
                Ok(handle) => handles.push(handle),
                Err(error) => {
                    Self::release(&runtime, &handles);
                    return Err(error);
                }
            }
        }
        let count =
            u64::try_from(handles.len()).map_err(|_| CnaError::InvalidInput("too many friends"))?;
        let pointer = if handles.is_empty() {
            core::ptr::null()
        } else {
            handles.as_ptr()
        };
        let mut collection = 0;
        // SAFETY: the array describes exactly `count` live handles, and the
        // collection takes ownership of each on success.
        let created = runtime.check(unsafe {
            (runtime.native().gamer_services.friend_collection_create_ext)(
                pointer,
                count,
                &mut collection,
            )
        });
        if let Err(error) = created {
            Self::release(&runtime, &handles);
            return Err(error);
        }
        Ok(crate::gamer_services::adopt_friend_collection(
            runtime, collection,
        ))
    }

    fn create(
        runtime: &crate::gamer_services::GamerServicesRuntimeHandle,
        friend: &FriendRegistration,
    ) -> Result<sys::CNA_Handle> {
        let gamertag = crate::gamer_services::borrow_string(&friend.gamertag)?;
        let display_name = crate::gamer_services::borrow_string(&friend.display_name)?;
        let mut handle = 0;
        // SAFETY: both views borrow for the call and CNA copies their bytes.
        runtime.check(unsafe {
            (runtime.native().gamer_services.friend_gamer_create_ext)(
                gamertag.value,
                display_name.value,
                u8::from(friend.is_online).into(),
                u8::from(friend.is_playing).into(),
                u8::from(friend.is_away).into(),
                u8::from(friend.is_busy).into(),
                u8::from(friend.friend_request_sent_to).into(),
                u8::from(friend.friend_request_received_from).into(),
                &mut handle,
            )
        })?;
        Ok(handle)
    }

    fn release(
        runtime: &crate::gamer_services::GamerServicesRuntimeHandle,
        handles: &[sys::CNA_Handle],
    ) {
        for handle in handles {
            // SAFETY: each came from the create route above and is released once.
            let _ = unsafe { (runtime.native().gamer_services.gamer_destroy)(*handle) };
        }
    }
}

/// What CNA's avatar content layer names, which XNA never published.
///
/// XNA's avatar assets came from the console: a body type and an animation
/// preset were identities the platform resolved, and the names behind them were
/// never a game's business. There is no such platform here, so CNA publishes
/// the names and lets a game supply the content -- which means a game has to be
/// able to ask what content a given identity wants.
pub trait AvatarContentNames {
    /// The content asset name this identity maps to.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn content_name(&self) -> Result<String>;
}

impl AvatarContentNames for AvatarBodyType {
    fn content_name(&self) -> Result<String> {
        let runtime = crate::gamer_services::open_runtime()?;
        let api = &runtime.native().gamer_services;
        let (size, copy) = (
            api.avatar_body_type_get_content_name_size_ext,
            api.avatar_body_type_copy_content_name_ext,
        );
        let identity = *self as u32;
        crate::native::runtime::read_string(
            |value| runtime.check(value),
            // SAFETY: the identity is a plain scalar.
            |bytes| unsafe { size(identity, bytes) },
            // SAFETY: the destination has the reported capacity.
            |destination, capacity, written| unsafe {
                copy(identity, destination, capacity, written)
            },
        )
    }
}

impl AvatarContentNames for AvatarAnimationPreset {
    fn content_name(&self) -> Result<String> {
        let runtime = crate::gamer_services::open_runtime()?;
        let api = &runtime.native().gamer_services;
        let (size, copy) = (
            api.avatar_animation_preset_get_clip_name_size_ext,
            api.avatar_animation_preset_copy_clip_name_ext,
        );
        let identity = *self as u32;
        crate::native::runtime::read_string(
            |value| runtime.check(value),
            // SAFETY: the identity is a plain scalar.
            |bytes| unsafe { size(identity, bytes) },
            // SAFETY: the destination has the reported capacity.
            |destination, capacity, written| unsafe {
                copy(identity, destination, capacity, written)
            },
        )
    }
}

/// The clip an avatar animation actually plays, beyond XNA's 31 presets.
///
/// XNA's `AvatarAnimation` is constructed from an enum and plays whatever the
/// console's avatar service had for it. CNA lets a game name its own clip, so
/// an animation can play something Microsoft never enumerated.
pub trait AvatarAnimationClip {
    /// The clip name this animation plays.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn clip_name(&self) -> Result<String>;

    /// Plays a clip of the game's own naming instead of the preset's.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn set_clip_name(&self, value: &str) -> Result<()>;
}

impl AvatarAnimationClip for AvatarAnimation {
    fn clip_name(&self) -> Result<String> {
        let runtime = crate::gamer_services::open_runtime()?;
        let handle = crate::gamer_services::animation_handle(self)?;
        let api = &runtime.native().gamer_services;
        let (size, copy) = (
            api.avatar_animation_get_real_clip_name_size_ext,
            api.avatar_animation_copy_real_clip_name_ext,
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

    fn set_clip_name(&self, value: &str) -> Result<()> {
        let runtime = crate::gamer_services::open_runtime()?;
        let handle = crate::gamer_services::animation_handle(self)?;
        let view = crate::gamer_services::borrow_string(value)?;
        // SAFETY: the handle is live and the view borrows for the call.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .avatar_animation_set_real_clip_name_ext)(handle, view.value)
        })
    }
}

/// The colours a real-rendered avatar is drawn in.
///
/// XNA's `AvatarDescription` carried these inside an opaque 76-byte blob the
/// console decoded. There is no console here, so CNA takes them directly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarAppearance {
    /// Skin colour.
    pub skin: Color,
    /// Hair colour.
    pub hair: Color,
    /// Shirt colour.
    pub shirt: Color,
    /// Trouser colour.
    pub pants: Color,
    /// Shoe colour.
    pub shoes: Color,
}

/// Drawing a real avatar, which XNA got from the console and CNA does not have.
///
/// `AvatarRenderer::Draw` on this runtime draws a placeholder, because the
/// avatar asset service XNA relied on does not exist outside Xbox Live. CNA's
/// answer is to let a game supply its own skinned model and its own clip
/// names, and this is that path: point the renderer at a
/// [`SkinnedModel`] -- CNA's own engine-layer model, which is what the route
/// borrows despite the header typing it as an opaque handle -- give it the
/// colours, and draw a named clip at a position in it.
///
/// It is deliberately outside `cna::Microsoft::Xna::Framework`. XNA has no
/// member that takes a model, and the strict `Draw` keeps meaning exactly what
/// it means -- draw whatever this runtime has for this avatar.
pub trait AvatarRealRendering {
    /// Draws this model rather than the placeholder from now on.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn use_model(&self, device: &GraphicsDevice, model: &SkinnedModel) -> Result<()>;

    /// Sets the colours the real model is drawn in.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    fn set_appearance(&self, appearance: AvatarAppearance) -> Result<()>;

    /// Draws the real model at a position inside a named clip.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, including the refusal for a
    /// renderer that has no real model.
    fn draw_clip(&self, clipName: &str, position: TimeSpan, r#loop: bool) -> Result<()>;
}

fn color_to_native(value: Color) -> sys::CNA_Color {
    sys::CNA_Color {
        r: value.R(),
        g: value.G(),
        b: value.B(),
        a: value.A(),
    }
}

impl AvatarRealRendering for AvatarRenderer {
    fn use_model(&self, device: &GraphicsDevice, model: &SkinnedModel) -> Result<()> {
        let runtime = crate::gamer_services::open_runtime()?;
        let handle = crate::gamer_services::renderer_handle(self)?;
        // SAFETY: all three handles are live for the call.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .avatar_renderer_enable_real_rendering_ext)(
                handle,
                device.handle()?,
                model.native_handle()?,
            )
        })
    }

    fn set_appearance(&self, appearance: AvatarAppearance) -> Result<()> {
        let runtime = crate::gamer_services::open_runtime()?;
        let handle = crate::gamer_services::renderer_handle(self)?;
        let value = sys::CNA_AvatarAppearanceEXT {
            struct_size: core::mem::size_of::<sys::CNA_AvatarAppearanceEXT>() as u32,
            struct_version: 1,
            skin_color: color_to_native(appearance.skin),
            hair_color: color_to_native(appearance.hair),
            shirt_color: color_to_native(appearance.shirt),
            pants_color: color_to_native(appearance.pants),
            shoes_color: color_to_native(appearance.shoes),
        };
        // SAFETY: the handle is live and the descriptor is versioned.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .avatar_renderer_set_appearance_ext)(handle, &value)
        })
    }

    fn draw_clip(&self, clipName: &str, position: TimeSpan, r#loop: bool) -> Result<()> {
        let runtime = crate::gamer_services::open_runtime()?;
        let handle = crate::gamer_services::renderer_handle(self)?;
        let view = crate::gamer_services::borrow_string(clipName)?;
        // SAFETY: the handle is live and the view borrows for the call.
        runtime.check(unsafe {
            (runtime.native().gamer_services.avatar_renderer_draw_real_ext)(
                handle,
                view.value,
                position.Ticks(),
                u8::from(r#loop).into(),
            )
        })
    }
}
