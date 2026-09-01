//! XNA's `GamerServicesDispatcher` and `Guide`.
//!
//! Both are static facades over CNA's process-global gamer services, so
//! neither owns a handle and nothing here is bound to a `Game` generation.
//!
//! # What HEADLESS actually does
//!
//! CNA's Guide has no window to draw on and no sign-in service behind it, and
//! the projection does not pretend otherwise: `ShowSignIn`, `ShowFriends` and
//! the rest reach their canonical routes and report whatever CNA answers,
//! including a refusal. The one thing the projection never does is invent a
//! visible Guide, a signed-in gamer or a chosen message-box button.
//!
//! `BeginShowMessageBox` and `BeginShowKeyboardInput` are the exception worth
//! stating plainly. CNA leaves the request *pending* rather than completing
//! it, because the answer is a person's choice; the matching `End*` reports
//! that no answer exists yet. `cna::extensions::gamer_services` publishes the
//! pending state and the routes that resolve it, which is how a host without a
//! Guide UI can drive one deterministically.

#![allow(non_snake_case)]

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::EventHandler;
use crate::extensions::window::WindowHandle;
use crate::game::TimeSpan;
use crate::input::PlayerIndex;

use super::async_result::{with_callback, GamerAsyncCallback, GamerAsyncResult, GamerAsyncState};
use super::core::GamerServicesRuntime;
use super::gamer::{string_view, Gamer, GamerBase};
use super::values::{MessageBoxIcon, NotificationPosition};

/// XNA `Microsoft.Xna.Framework.GamerServices.GamerServicesDispatcher`.
pub struct GamerServicesDispatcher;

impl GamerServicesDispatcher {
    /// XNA `GamerServicesDispatcher.Initialize`.
    ///
    /// XNA takes the game's service provider. CNA's dispatcher takes the game
    /// handle itself, so the projection takes the callback-scoped
    /// [`crate::Microsoft::Xna::Framework::GameContext`] that names it -- the
    /// same explicit-context rule the Audio and Media families use.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Initialize(serviceProvider: &crate::game::GameContext<'_>) -> Result<()> {
        Self::initialize_with_game(serviceProvider.handle)
    }

    /// The same call, for a caller that already holds the game handle.
    ///
    /// `GamerServicesComponent` is the one: XNA's component initialises the
    /// dispatcher from `Game.Services` during `Game.Initialize`, which is
    /// inside a lifecycle callback but not inside a `GameContext`.
    pub(crate) fn initialize_with_game(game: sys::CNA_Handle) -> Result<()> {
        let runtime = GamerServicesRuntime::open()?;
        // SAFETY: the game handle is callback-scoped and live.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .gamer_services_dispatcher_initialize)(game)
        })
    }

    /// XNA `GamerServicesDispatcher.Update`.
    ///
    /// Also reports a sign-in or sign-out subscription CNA refused: XNA's `+=`
    /// cannot fail, so the refusal surfaces here, where the CLR delivers those
    /// events from.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn Update() -> Result<()> {
        let runtime = GamerServicesRuntime::open()?;
        // SAFETY: the route is process-global and takes nothing.
        runtime.check(unsafe {
            (runtime.native().gamer_services.gamer_services_dispatcher_update)()
        })?;
        super::events::take_subscription_error()
    }

    /// XNA `GamerServicesDispatcher.IsInitialized`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsInitialized() -> Result<bool> {
        let runtime = GamerServicesRuntime::open()?;
        let mut value = 0;
        // SAFETY: the output is initialized and the route is process-global.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .gamer_services_dispatcher_get_is_initialized)(&mut value)
        })?;
        Ok(value != 0)
    }

    /// XNA `GamerServicesDispatcher.WindowHandle`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn WindowHandle() -> Result<WindowHandle> {
        let runtime = GamerServicesRuntime::open()?;
        let mut value = 0;
        // SAFETY: the output is initialized and the route is process-global.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .gamer_services_dispatcher_get_window_handle)(&mut value)
        })?;
        Ok(WindowHandle(value))
    }

    /// XNA `GamerServicesDispatcher.WindowHandle` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetWindowHandle(value: WindowHandle) -> Result<()> {
        let runtime = GamerServicesRuntime::open()?;
        // SAFETY: the handle is an opaque platform value CNA stores unchanged.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .gamer_services_dispatcher_set_window_handle)(value.0)
        })
    }

    /// XNA `GamerServicesDispatcher.InstallingTitleUpdate` subscription.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    #[must_use]
    pub fn AddInstallingTitleUpdateHandler(handler: Box<dyn EventHandler>) -> u64 {
        super::events::add_installing_title_update(handler).unwrap_or(0)
    }

    /// XNA `GamerServicesDispatcher.InstallingTitleUpdate` removal.
    #[must_use]
    pub fn RemoveInstallingTitleUpdateHandler(registration: u64) -> bool {
        super::events::remove_installing_title_update(registration).unwrap_or(false)
    }
}

/// XNA `Microsoft.Xna.Framework.GamerServices.Guide`.
pub struct Guide;

macro_rules! guide_player_screen {
    ($($name:ident => $route:ident),+ $(,)?) => {
        impl Guide {
            $(
                #[doc = concat!("XNA `Guide.", stringify!($name), "`.")]
                ///
                /// # Errors
                ///
                /// Returns the exact error CNA reports, including its refusal
                /// on a host with no Guide.
                pub fn $name(player: PlayerIndex) -> Result<()> {
                    let runtime = GamerServicesRuntime::open()?;
                    // SAFETY: the slot identity is a plain scalar.
                    runtime.check(unsafe {
                        (runtime.native().gamer_services.$route)(player as u32)
                    })
                }
            )+
        }
    };
}

guide_player_screen! {
    ShowMessages => guide_show_messages,
    ShowFriends => guide_show_friends,
    ShowPlayers => guide_show_players,
    ShowParty => guide_show_party,
    ShowPartySessions => guide_show_party_sessions,
    ShowMarketplace => guide_show_marketplace,
}

macro_rules! guide_gamer_screen {
    ($($name:ident => $route:ident),+ $(,)?) => {
        impl Guide {
            $(
                #[doc = concat!("XNA `Guide.", stringify!($name), "`.")]
                ///
                /// # Errors
                ///
                /// Returns the exact error CNA reports.
                pub fn $name(player: PlayerIndex, gamer: &Gamer) -> Result<()> {
                    let runtime = GamerServicesRuntime::open()?;
                    let handle = gamer.handle_for_guide()?;
                    // SAFETY: the gamer handle is live for the call.
                    runtime.check(unsafe {
                        (runtime.native().gamer_services.$route)(player as u32, handle)
                    })
                }
            )+
        }
    };
}

guide_gamer_screen! {
    ShowFriendRequest => guide_show_friend_request,
    ShowPlayerReview => guide_show_player_review,
    ShowGamerCard => guide_show_gamer_card,
}

impl Guide {
    /// XNA `Guide.IsScreenSaverEnabled`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsScreenSaverEnabled() -> Result<bool> {
        Self::flag(|api| api.guide_get_is_screen_saver_enabled)
    }

    /// XNA `Guide.IsScreenSaverEnabled` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetIsScreenSaverEnabled(value: bool) -> Result<()> {
        Self::set_flag(|api| api.guide_set_is_screen_saver_enabled, value)
    }

    /// XNA `Guide.IsVisible`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsVisible() -> Result<bool> {
        Self::flag(|api| api.guide_get_is_visible)
    }

    /// XNA `Guide.IsTrialMode`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn IsTrialMode() -> Result<bool> {
        Self::flag(|api| api.guide_get_is_trial_mode)
    }

    /// XNA `Guide.SimulateTrialMode`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SimulateTrialMode() -> Result<bool> {
        Self::flag(|api| api.guide_get_simulate_trial_mode)
    }

    /// XNA `Guide.SimulateTrialMode` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetSimulateTrialMode(value: bool) -> Result<()> {
        Self::set_flag(|api| api.guide_set_simulate_trial_mode, value)
    }

    /// XNA `Guide.NotificationPosition`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, or the mapping error for a
    /// position XNA does not declare.
    pub fn NotificationPosition() -> Result<NotificationPosition> {
        let runtime = GamerServicesRuntime::open()?;
        let mut value = 0;
        // SAFETY: the output is initialized and the route is process-global.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .guide_get_notification_position)(&mut value)
        })?;
        NotificationPosition::from_native(value).ok_or(CnaError::InvalidInput(
            "CNA reported a notification position XNA does not declare",
        ))
    }

    /// XNA `Guide.NotificationPosition` assignment.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn SetNotificationPosition(value: NotificationPosition) -> Result<()> {
        let runtime = GamerServicesRuntime::open()?;
        // SAFETY: the identity is a plain scalar.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .guide_set_notification_position)(value as u32)
        })
    }

    /// XNA `Guide.ShowSignIn`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ShowSignIn(paneCount: i32, onlineOnly: bool) -> Result<()> {
        let runtime = GamerServicesRuntime::open()?;
        // SAFETY: both arguments are plain scalars.
        runtime.check(unsafe {
            (runtime.native().gamer_services.guide_show_sign_in)(
                paneCount,
                u8::from(onlineOnly).into(),
            )
        })
    }

    /// XNA `Guide.DelayNotifications`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn DelayNotifications(delay: TimeSpan) -> Result<()> {
        let runtime = GamerServicesRuntime::open()?;
        // SAFETY: the delay is a plain tick count.
        runtime.check(unsafe {
            (runtime.native().gamer_services.guide_delay_notifications)(delay.Ticks())
        })
    }

    /// XNA `Guide.ShowComposeMessage`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ShowComposeMessage(player: PlayerIndex, text: &str, recipients: &[Gamer]) -> Result<()> {
        let runtime = GamerServicesRuntime::open()?;
        let view = string_view(text)?;
        let handles = Self::handles(recipients)?;
        let (pointer, count) = Self::array(&handles)?;
        // SAFETY: the view borrows `text` and the array describes `count` live
        // handles for the duration of the call.
        runtime.check(unsafe {
            (runtime.native().gamer_services.guide_show_compose_message)(
                player as u32,
                view.value,
                pointer,
                count,
            )
        })
    }

    /// XNA `Guide.ShowGameInvite(player, recipients)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ShowGameInvite(player: PlayerIndex, recipients: &[Gamer]) -> Result<()> {
        let runtime = GamerServicesRuntime::open()?;
        let handles = Self::handles(recipients)?;
        let (pointer, count) = Self::array(&handles)?;
        // SAFETY: the array describes `count` live handles for the call.
        runtime.check(unsafe {
            (runtime.native().gamer_services.guide_show_game_invite)(player as u32, pointer, count)
        })
    }

    /// XNA `Guide.ShowGameInvite(sessionId)`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn ShowGameInviteWithSessionId(sessionId: &str) -> Result<()> {
        let runtime = GamerServicesRuntime::open()?;
        let view = string_view(sessionId)?;
        // SAFETY: the view borrows `sessionId` for the call.
        runtime.check(unsafe {
            (runtime
                .native()
                .gamer_services
                .guide_show_game_invite_for_session)(view.value)
        })
    }

    /// XNA `Guide.BeginShowMessageBox(player, ...)`.
    ///
    /// CNA leaves the request pending rather than answering it: the choice is
    /// a person's. [`Guide::EndShowMessageBox`] answers `None` until something
    /// resolves it, and `cna::extensions::gamer_services` publishes the
    /// pending state and the routes that do.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginShowMessageBox(
        player: PlayerIndex,
        title: &str,
        text: &str,
        buttons: &[&str],
        focusButton: i32,
        icon: MessageBoxIcon,
        callback: Option<GamerAsyncCallback>,
        state: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = GamerServicesRuntime::open()?;
        let title_view = string_view(title)?;
        let text_view = string_view(text)?;
        let button_views = buttons
            .iter()
            .map(|value| string_view(value).map(|view| view.value))
            .collect::<Result<Vec<_>>>()?;
        let count = u64::try_from(button_views.len())
            .map_err(|_| CnaError::InvalidInput("the button array is too large"))?;
        let pointer = if button_views.is_empty() {
            core::ptr::null()
        } else {
            button_views.as_ptr()
        };
        let route = runtime.native().gamer_services.guide_begin_show_message_box;
        let (result, _fired) = with_callback(state, callback, |trampoline, context| {
            // SAFETY: every view borrows its string for the call and the
            // button array describes exactly `count` views.
            runtime.check(unsafe {
                route(
                    player as u32,
                    title_view.value,
                    text_view.value,
                    pointer,
                    count,
                    focusButton,
                    icon as u32,
                    trampoline,
                    context,
                )
            })?;
            Ok(())
        })?;
        Ok(result)
    }

    /// XNA `Guide.BeginShowMessageBox(title, ...)`, for the first slot.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    #[allow(clippy::too_many_arguments)]
    pub fn BeginShowMessageBoxWithTitleAndTextAndButtonsAndFocusButtonAndIconAndCallbackAndState(
        title: &str,
        text: &str,
        buttons: &[&str],
        focusButton: i32,
        icon: MessageBoxIcon,
        callback: Option<GamerAsyncCallback>,
        state: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        Self::BeginShowMessageBox(
            PlayerIndex::One,
            title,
            text,
            buttons,
            focusButton,
            icon,
            callback,
            state,
        )
    }

    /// XNA `Guide.EndShowMessageBox`.
    ///
    /// `None` is CLR `null`: the message box was dismissed without a choice,
    /// which is also what a host with no Guide reports.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated, or the exact error
    /// CNA reports.
    pub fn EndShowMessageBox(result: &GamerAsyncResult) -> Result<Option<i32>> {
        result.end_once::<()>()?;
        let runtime = GamerServicesRuntime::open()?;
        let (mut has_choice, mut index) = (0, 0);
        // SAFETY: both outputs are initialized and the route is process-global.
        runtime.check(unsafe {
            (runtime.native().gamer_services.guide_end_show_message_box)(
                &mut has_choice,
                &mut index,
            )
        })?;
        Ok((has_choice != 0).then_some(index))
    }

    /// XNA `Guide.BeginShowKeyboardInput`.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn BeginShowKeyboardInput(
        player: PlayerIndex,
        title: &str,
        description: &str,
        defaultText: &str,
        callback: Option<GamerAsyncCallback>,
        state: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        Self::begin_keyboard(player, title, description, defaultText, false, callback, state)
    }

    /// XNA `Guide.BeginShowKeyboardInput` with password masking.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    #[allow(clippy::too_many_arguments)]
    pub fn BeginShowKeyboardInputWithPlayerAndTitleAndDescriptionAndDefaultTextAndCallbackAndStateAndUsePasswordMode(
        player: PlayerIndex,
        title: &str,
        description: &str,
        defaultText: &str,
        callback: Option<GamerAsyncCallback>,
        state: GamerAsyncState,
        usePasswordMode: bool,
    ) -> Result<GamerAsyncResult> {
        Self::begin_keyboard(
            player,
            title,
            description,
            defaultText,
            usePasswordMode,
            callback,
            state,
        )
    }

    /// XNA `Guide.EndShowKeyboardInput`.
    ///
    /// # Errors
    ///
    /// Returns the one-shot error when `End` is repeated, or the exact error
    /// CNA reports.
    pub fn EndShowKeyboardInput(result: &GamerAsyncResult) -> Result<String> {
        result.end_once::<()>()?;
        let runtime = GamerServicesRuntime::open()?;
        let api = &runtime.native().gamer_services;
        let (size, copy) = (
            api.guide_end_show_keyboard_input_size,
            api.guide_end_show_keyboard_input,
        );
        crate::native::runtime::read_string(
            |value| runtime.check(value),
            // SAFETY: the size query takes only its output.
            |bytes| unsafe { size(bytes) },
            // SAFETY: the destination has the reported capacity.
            |destination, capacity, written| unsafe { copy(destination, capacity, written) },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn begin_keyboard(
        player: PlayerIndex,
        title: &str,
        description: &str,
        default_text: &str,
        use_password_mode: bool,
        callback: Option<GamerAsyncCallback>,
        state: GamerAsyncState,
    ) -> Result<GamerAsyncResult> {
        let runtime = GamerServicesRuntime::open()?;
        let title_view = string_view(title)?;
        let description_view = string_view(description)?;
        let default_view = string_view(default_text)?;
        let route = runtime
            .native()
            .gamer_services
            .guide_begin_show_keyboard_input;
        let (result, _fired) = with_callback(state, callback, |trampoline, context| {
            // SAFETY: every view borrows its string for the call.
            runtime.check(unsafe {
                route(
                    player as u32,
                    title_view.value,
                    description_view.value,
                    default_view.value,
                    u8::from(use_password_mode).into(),
                    trampoline,
                    context,
                )
            })?;
            Ok(())
        })?;
        Ok(result)
    }

    fn handles(gamers: &[Gamer]) -> Result<Vec<sys::CNA_Handle>> {
        gamers
            .iter()
            .map(GamerBase::handle_for_guide)
            .collect::<Result<Vec<_>>>()
    }

    fn array(handles: &[sys::CNA_Handle]) -> Result<(*const sys::CNA_Handle, u64)> {
        let count = u64::try_from(handles.len())
            .map_err(|_| CnaError::InvalidInput("the gamer array is too large"))?;
        let pointer = if handles.is_empty() {
            core::ptr::null()
        } else {
            handles.as_ptr()
        };
        Ok((pointer, count))
    }

    fn flag(
        select: impl Fn(
            &crate::native::gamer_services::GamerServicesApi,
        ) -> unsafe extern "C" fn(*mut sys::CNA_Bool) -> sys::CNA_Result,
    ) -> Result<bool> {
        let runtime = GamerServicesRuntime::open()?;
        let route = select(&runtime.native().gamer_services);
        let mut value = 0;
        // SAFETY: the output is initialized and the route is process-global.
        runtime.check(unsafe { route(&mut value) })?;
        Ok(value != 0)
    }

    fn set_flag(
        select: impl Fn(
            &crate::native::gamer_services::GamerServicesApi,
        ) -> unsafe extern "C" fn(sys::CNA_Bool) -> sys::CNA_Result,
        value: bool,
    ) -> Result<()> {
        let runtime = GamerServicesRuntime::open()?;
        let route = select(&runtime.native().gamer_services);
        // SAFETY: the argument is a canonical boolean.
        runtime.check(unsafe { route(u8::from(value).into()) })
    }
}
