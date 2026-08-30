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

use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::gamer_services::{Achievement, AchievementCollection, SignedInGamer};
use crate::input::PlayerIndex;

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
