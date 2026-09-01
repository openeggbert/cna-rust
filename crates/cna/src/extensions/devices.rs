//! CNA's device layer: power, system facts, locale, display and clipboard.
//!
//! None of it exists in XNA 4.0. All of it needs a live game, because CNA
//! reaches the host platform through the game's platform binding, so every
//! route here takes the same callback-scoped [`GameContext`] the strict XNA
//! context-injected members take.
//!
//! **A route answering is not the same as a device existing.** The layer is a
//! build option: every route is exported in both states, and the ones the
//! layer implements refuse with `NOT_SUPPORTED` when it is compiled out. Ask
//! [`is_available`] first rather than reading a refusal as a missing device.

#![allow(clippy::missing_errors_doc)]

use cna_sys as sys;

use crate::error::Result;
use crate::game::GameContext;
use crate::native::runtime::read_string;
use crate::value::Rectangle;

/// The host's power state.
///
/// It is the same identity a gamepad's battery reports: the six values answer
/// both questions, so a caller that handles one handles the other.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum PowerState {
    /// The platform could not determine the state.
    Error,
    Unknown,
    OnBattery,
    NoBattery,
    Charging,
    Charged,
    /// A state a newer CNA introduced.
    Unrecognized(u32),
}

impl PowerState {
    /// Maps CNA's power-state identity, which a joystick's battery shares.
    pub(super) const fn from_native_value(value: sys::CNA_PowerState) -> Self {
        Self::from_native(value)
    }

    const fn from_native(value: sys::CNA_PowerState) -> Self {
        match value {
            sys::CNA_POWER_STATE_ERROR => Self::Error,
            sys::CNA_POWER_STATE_UNKNOWN => Self::Unknown,
            sys::CNA_POWER_STATE_ON_BATTERY => Self::OnBattery,
            sys::CNA_POWER_STATE_NO_BATTERY => Self::NoBattery,
            sys::CNA_POWER_STATE_CHARGING => Self::Charging,
            sys::CNA_POWER_STATE_CHARGED => Self::Charged,
            other => Self::Unrecognized(other),
        }
    }
}

/// One preferred locale the host reports, most preferred first.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Locale {
    /// ISO 639 language code.
    pub language: String,
    /// ISO 3166 country code, empty when the host names none.
    pub country: String,
}

/// Whether this build contains the extended device layer.
///
/// A `false` here means every `NOT_SUPPORTED` below is a compiled-out layer
/// rather than a missing device.
pub fn is_available() -> Result<bool> {
    let native = crate::native::Native::process()?;
    let mut value = sys::CNA_FALSE;
    // SAFETY: the output is a live local of the declared type.
    native.check(unsafe { (native.runtime.devices_is_available)(&mut value) })?;
    Ok(value != sys::CNA_FALSE)
}

/// The host's power state.
pub fn power_state(game: &GameContext<'_>) -> Result<PowerState> {
    let (native, handle) = game.native_game();
    let mut value = 0;
    // SAFETY: the game handle is callback-live and the output is a live local.
    native.check(unsafe { (native.runtime.power_state)(handle, &mut value) })?;
    Ok(PowerState::from_native(value))
}

/// Remaining battery charge as a percentage, or `None` when unknown.
pub fn battery_percent(game: &GameContext<'_>) -> Result<Option<i32>> {
    let (native, handle) = game.native_game();
    let mut value = 0;
    // SAFETY: the game handle is callback-live and the output is a live local.
    native.check(unsafe { (native.runtime.power_battery_percent)(handle, &mut value) })?;
    Ok((value >= 0).then_some(value))
}

/// Remaining battery time in seconds, or `None` when unknown.
pub fn battery_seconds_remaining(game: &GameContext<'_>) -> Result<Option<i32>> {
    let (native, handle) = game.native_game();
    let mut value = 0;
    // SAFETY: the game handle is callback-live and the output is a live local.
    native.check(unsafe { (native.runtime.power_seconds_remaining)(handle, &mut value) })?;
    Ok((value >= 0).then_some(value))
}

/// Logical CPU cores the host reports.
pub fn logical_cpu_core_count(game: &GameContext<'_>) -> Result<i32> {
    let (native, handle) = game.native_game();
    let mut value = 0;
    // SAFETY: the game handle is callback-live and the output is a live local.
    native.check(unsafe { (native.runtime.system_cpu_core_count)(handle, &mut value) })?;
    Ok(value)
}

/// System RAM in megabytes, as the host reports it.
pub fn system_ram_megabytes(game: &GameContext<'_>) -> Result<i32> {
    let (native, handle) = game.native_game();
    let mut value = 0;
    // SAFETY: the game handle is callback-live and the output is a live local.
    native.check(unsafe { (native.runtime.system_ram_megabytes)(handle, &mut value) })?;
    Ok(value)
}

/// The host's preferred locales, most preferred first.
pub fn preferred_locales(game: &GameContext<'_>) -> Result<Vec<Locale>> {
    let (native, handle) = game.native_game();
    let api = &native.runtime;
    let mut count = 0_u64;
    // SAFETY: the game handle is callback-live and the output is a live local.
    native.check(unsafe { (api.locale_count)(handle, &mut count) })?;
    let mut result = Vec::new();
    for index in 0..count {
        let language = read_string(
            |value| native.check(value),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.locale_language_size)(handle, index, bytes) },
            |destination, capacity, written| unsafe {
                (api.locale_copy_language)(handle, index, destination, capacity, written)
            },
        )?;
        let country = read_string(
            |value| native.check(value),
            // SAFETY: as above.
            |bytes| unsafe { (api.locale_country_size)(handle, index, bytes) },
            |destination, capacity, written| unsafe {
                (api.locale_copy_country)(handle, index, destination, capacity, written)
            },
        )?;
        result.push(Locale { language, country });
    }
    Ok(result)
}

/// The game window's display content scale, or `None` when there is no window.
///
/// A headless or windowless session answers zero upstream, which is the
/// canonical answer rather than a failure. It becomes `None` here so a caller
/// cannot mistake "no window" for a scale of zero.
pub fn display_content_scale(game: &GameContext<'_>) -> Result<Option<f32>> {
    let (native, handle) = game.native_game();
    let mut value = 0.0;
    // SAFETY: the game handle is callback-live and the output is a live local.
    native.check(unsafe { (native.runtime.display_content_scale)(handle, &mut value) })?;
    Ok((value != 0.0).then_some(value))
}

/// The display's safe area, in the window's client coordinates.
pub fn display_safe_area(game: &GameContext<'_>) -> Result<Rectangle> {
    let (native, handle) = game.native_game();
    let mut value = sys::CNA_Rectangle::default();
    // SAFETY: the game handle is callback-live and the output is a live local.
    native.check(unsafe { (native.runtime.display_safe_area)(handle, &mut value) })?;
    Ok(Rectangle::new(value.x, value.y, value.width, value.height))
}

/// The system clipboard's current text.
///
/// An empty or unavailable clipboard answers with an empty string rather than
/// failing. The clipboard is process-external state: another application can
/// change it between this call and the next.
pub fn clipboard_text(game: &GameContext<'_>) -> Result<String> {
    let (native, handle) = game.native_game();
    let api = &native.runtime;
    read_string(
        |value| native.check(value),
        // SAFETY: both outputs are live locals; the two routes form CNA's
        // canonical size-then-copy pair for one UTF-8 string.
        |bytes| unsafe { (api.clipboard_text_size)(handle, bytes) },
        |destination, capacity, written| unsafe {
            (api.clipboard_copy_text)(handle, destination, capacity, written)
        },
    )
}

/// Places text on the system clipboard.
///
/// Success means the request was made, not that the clipboard changed: a
/// headless session with no clipboard, or a browser that requires a user
/// gesture, may ignore it. [`try_set_clipboard_text`] is the route that
/// answers *that* question, and is the better one to reach for.
pub fn set_clipboard_text(game: &GameContext<'_>, text: &str) -> Result<()> {
    let (native, handle) = game.native_game();
    let view = sys::CNA_StringView {
        data: text.as_ptr().cast::<core::ffi::c_char>(),
        byte_length: text.len() as u64,
    };
    // SAFETY: `view` borrows `text` for the duration of the call.
    native.check(unsafe { (native.runtime.clipboard_set_text)(handle, view) })
}

/// What a camera is currently doing.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CameraState {
    /// The platform reports no camera support at all.
    #[default]
    NotSupported,
    /// Present, not opened.
    Closed,
    /// Opening.
    Opening,
    /// The user or the platform refused access.
    Denied,
    /// Open and delivering frames.
    Ready,
    /// It was ready and the platform took it away.
    Lost,
}

impl CameraState {
    const fn from_native(value: sys::CNA_CameraState) -> Self {
        match value {
            sys::CNA_CAMERA_STATE_CLOSED => Self::Closed,
            sys::CNA_CAMERA_STATE_OPENING => Self::Opening,
            sys::CNA_CAMERA_STATE_DENIED => Self::Denied,
            sys::CNA_CAMERA_STATE_READY => Self::Ready,
            5 => Self::Lost,
            _ => Self::NotSupported,
        }
    }

    const fn to_native(self) -> sys::CNA_CameraState {
        match self {
            Self::NotSupported => sys::CNA_CAMERA_STATE_NOT_SUPPORTED,
            Self::Closed => sys::CNA_CAMERA_STATE_CLOSED,
            Self::Opening => sys::CNA_CAMERA_STATE_OPENING,
            Self::Denied => sys::CNA_CAMERA_STATE_DENIED,
            Self::Ready => sys::CNA_CAMERA_STATE_READY,
            Self::Lost => 5,
        }
    }
}

/// One camera the platform reports.
///
/// # This family is blocked upstream
///
/// `RUST-UPSTREAM-020`. `cna_camera_create_with_test_backend_ext` hands CNA's
/// **global** platform override a raw pointer into the camera resource, and
/// `cna_camera_destroy` frees that resource without clearing the override. Any
/// later call that consults the platform camera list reads freed memory.
///
/// The type is therefore deliberately small: it exists so
/// `tests/upstream_camera_destroy.rs` can drive the sequence and keep measuring
/// whether the defect is still there. It is **not** a projection anybody should
/// build on yet, and the safe API does not pretend the lifecycle is sound --
/// wrapping a crashing teardown in a friendly `Result` would hide it.
pub struct Camera {
    native: std::sync::Arc<crate::native::Native>,
    handle: std::sync::Mutex<sys::CNA_CameraHandle>,
}

impl Camera {
    /// How many cameras the platform reports.
    ///
    /// After any camera has been destroyed this reads through the dangling
    /// override described on [`Camera`]. That is the reproducer's payload.
    pub fn count(game: &GameContext<'_>) -> Result<u64> {
        let (native, handle) = game.native_game();
        let mut value = 0_u64;
        // SAFETY: the game handle is callback-live and the output is a live
        // local.
        native.check(unsafe { (native.engine.camera_get_count_ext)(handle, &mut value) })?;
        Ok(value)
    }

    /// Whether the platform supports cameras at all.
    pub fn is_supported(game: &GameContext<'_>) -> Result<bool> {
        let (native, handle) = game.native_game();
        let mut value = 0_u8;
        // SAFETY: the game handle is callback-live and the output is a live
        // local.
        native
            .check(unsafe { (native.engine.camera_get_is_supported_ext)(handle, &mut value) })?;
        Ok(value != 0)
    }

    /// Opens CNA's deterministic test camera.
    pub fn with_test_backend(game: &GameContext<'_>) -> Result<Self> {
        let (native, handle) = game.native_game();
        let mut camera = sys::CNA_INVALID_HANDLE;
        // SAFETY: the game handle is callback-live and the output is a live
        // local.
        native.check(unsafe {
            (native.engine.camera_create_with_test_backend_ext)(handle, &mut camera)
        })?;
        Ok(Self {
            native: std::sync::Arc::clone(native),
            handle: std::sync::Mutex::new(camera),
        })
    }

    fn get(&self) -> Result<sys::CNA_CameraHandle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(crate::error::CnaError::InvalidInput(
                "the camera has been released",
            ));
        }
        Ok(handle)
    }

    /// What the camera is doing.
    pub fn state(&self) -> Result<CameraState> {
        let handle = self.get()?;
        let mut value = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.camera_get_state_ext)(handle, &mut value) })?;
        Ok(CameraState::from_native(value))
    }

    /// Drives the test backend's state.
    pub fn set_test_state(&self, state: CameraState) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and the identity is canonical.
        self.native.check(unsafe {
            (self.native.engine.camera_set_test_state_ext)(handle, state.to_native())
        })
    }

    /// Releases the camera.
    ///
    /// This is the call that leaves CNA's platform override pointing at freed
    /// memory; see [`Camera`].
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = *guard;
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle was published by this object's own create route
        // and is released exactly once, here.
        self.native
            .check(unsafe { (self.native.engine.camera_destroy)(handle) })?;
        *guard = sys::CNA_INVALID_HANDLE;
        Ok(())
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// Whether the host is a real device or an emulator.
///
/// XNA's `Microsoft.Devices.Environment.DeviceType`, which a Windows Phone game
/// read to skip work an emulator could not do.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DeviceType {
    Device,
    Emulator,
    /// A kind a newer CNA introduced.
    Unrecognized(u32),
}

impl DeviceType {
    const fn from_native(value: sys::CNA_DeviceType) -> Self {
        match value {
            sys::CNA_DEVICE_TYPE_DEVICE => Self::Device,
            sys::CNA_DEVICE_TYPE_EMULATOR => Self::Emulator,
            other => Self::Unrecognized(other),
        }
    }
}

/// Whether this host is a real device or an emulator.
///
/// Takes no game: it is a property of the process, not of a window.
pub fn device_type() -> Result<DeviceType> {
    let native = crate::native::Native::process()?;
    let mut value = 0;
    // SAFETY: the output is a live local of the declared type.
    native.check(unsafe { (native.runtime.environment_get_device_type)(&mut value) })?;
    Ok(DeviceType::from_native(value))
}

/// Puts text on the host clipboard, and says whether it went.
///
/// The difference from [`set_clipboard_text`] is the answer: that one reports
/// only that the request was made. `false` here is an ordinary answer on a host
/// with no clipboard, which is why it is not a failure.
pub fn try_set_clipboard_text(game: &GameContext<'_>, text: &str) -> Result<bool> {
    let (native, handle) = game.native_game();
    let mut value = sys::CNA_FALSE;
    // SAFETY: the game handle is callback-live, the text is borrowed for the
    // call, and the output is a live local.
    native.check(unsafe {
        (native.runtime.devices_clipboard_set_text_ext)(handle, string_view(text), &mut value)
    })?;
    Ok(value != sys::CNA_FALSE)
}

/// Opens a URL in the host's browser.
///
/// Answers whether the platform accepted it. What "accepted" means is the
/// platform's -- a browser may still fail to load the page -- so this reports
/// the handoff rather than the outcome.
pub fn open_url(game: &GameContext<'_>, url: &str) -> Result<bool> {
    let (native, handle) = game.native_game();
    let mut value = sys::CNA_FALSE;
    // SAFETY: as above.
    native.check(unsafe {
        (native.runtime.url_launcher_open_ext)(handle, string_view(url), &mut value)
    })?;
    Ok(value != sys::CNA_FALSE)
}

fn string_view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast::<core::ffi::c_char>(),
        byte_length: value.len() as u64,
    }
}

/// How severe a message box is.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum MessageBoxType {
    #[default]
    Error,
    Warning,
    Information,
}

impl MessageBoxType {
    const fn to_native(self) -> sys::CNA_MessageBoxType {
        match self {
            Self::Error => sys::CNA_MESSAGE_BOX_TYPE_ERROR,
            Self::Warning => sys::CNA_MESSAGE_BOX_TYPE_WARNING,
            Self::Information => sys::CNA_MESSAGE_BOX_TYPE_INFORMATION,
        }
    }

    const fn from_native(value: sys::CNA_MessageBoxType) -> Option<Self> {
        Some(match value {
            sys::CNA_MESSAGE_BOX_TYPE_ERROR => Self::Error,
            sys::CNA_MESSAGE_BOX_TYPE_WARNING => Self::Warning,
            sys::CNA_MESSAGE_BOX_TYPE_INFORMATION => Self::Information,
            _ => return None,
        })
    }
}

/// What the substitute message-box backend recorded.
///
/// Counts and the last call's shape, which is how a test asserts that a game
/// asked the question it meant to ask -- on a machine with no desktop session
/// to answer it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct MessageBoxTestLog {
    pub simple_calls: u32,
    pub choice_calls: u32,
    /// The severity the last call passed, when it decoded to one this build
    /// knows.
    pub last_type: Option<MessageBoxType>,
    pub last_button_count: u32,
}

/// Message boxes, and the substitute backend that makes them testable.
pub mod message_box {
    use super::{string_view, MessageBoxTestLog, MessageBoxType};
    use crate::error::{CnaError, Result};
    use crate::game::GameContext;
    use cna_sys as sys;

    /// Whether this platform can show a message box at all.
    pub fn is_supported(game: &GameContext<'_>) -> Result<bool> {
        let (native, handle) = game.native_game();
        let mut value = sys::CNA_FALSE;
        // SAFETY: the game handle is callback-live and the output is a local.
        native
            .check(unsafe { (native.runtime.message_box_get_is_supported_ext)(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Shows a message box with no choice to make.
    pub fn show(
        game: &GameContext<'_>,
        kind: MessageBoxType,
        title: &str,
        message: &str,
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: the game handle is callback-live and both strings are
        // borrowed for the call.
        native.check(unsafe {
            (native.runtime.message_box_show_simple_ext)(
                handle,
                kind.to_native(),
                string_view(title),
                string_view(message),
            )
        })
    }

    /// Shows a message box with buttons, answering which was chosen.
    ///
    /// `None` when the dialog was dismissed without choosing.
    pub fn show_choice(
        game: &GameContext<'_>,
        kind: MessageBoxType,
        title: &str,
        message: &str,
        buttons: &[&str],
    ) -> Result<Option<usize>> {
        let (native, handle) = game.native_game();
        let views: Vec<sys::CNA_StringView> =
            buttons.iter().map(|label| string_view(label)).collect();
        let mut chosen = -1_i32;
        // SAFETY: the game handle is callback-live, and the strings and the
        // array they are in all outlive the call.
        native.check(unsafe {
            (native.runtime.message_box_show_ext)(
                handle,
                kind.to_native(),
                string_view(title),
                string_view(message),
                if views.is_empty() {
                    core::ptr::null()
                } else {
                    views.as_ptr()
                },
                views.len() as u64,
                &mut chosen,
            )
        })?;
        // Upstream spells "dismissed" as a negative index.
        Ok(usize::try_from(chosen).ok())
    }

    /// Installs or removes CNA's substitute backend.
    ///
    /// `chosen_button` is what [`show_choice`] will answer while it is
    /// installed, which is what lets a test drive both branches of a
    /// confirmation without a desktop session.
    pub fn set_test_backend(
        game: &GameContext<'_>,
        installed: bool,
        chosen_button: i32,
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: the game handle is callback-live and both values are by
        // value.
        native.check(unsafe {
            (native.runtime.message_box_set_test_backend_ext)(
                handle,
                u8::from(installed),
                chosen_button,
            )
        })
    }

    /// What the substitute backend has recorded.
    pub fn test_log(game: &GameContext<'_>) -> Result<MessageBoxTestLog> {
        let (native, handle) = game.native_game();
        let mut value = sys::CNA_MessageBoxTestLog {
            struct_size: core::mem::size_of::<sys::CNA_MessageBoxTestLog>() as u32,
            struct_version: 1,
            ..sys::CNA_MessageBoxTestLog::default()
        };
        // SAFETY: the game handle is callback-live and the output is a live
        // local whose size and version headers are set.
        native.check(unsafe { (native.runtime.message_box_get_test_log_ext)(handle, &mut value) })?;
        let _ = CnaError::InvalidInput;
        Ok(MessageBoxTestLog {
            simple_calls: value.simple_calls,
            choice_calls: value.choice_calls,
            last_type: MessageBoxType::from_native(value.last_type),
            last_button_count: value.last_button_count,
        })
    }
}

/// What the substitute vibration backend recorded.
///
/// Every field is a count or the last call's argument, which is what makes a
/// rumble pattern assertable on a machine with no gamepad: a test drives the
/// game and then reads what the game asked for.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct VibrationTestLog {
    pub start_calls: u32,
    pub stop_calls: u32,
    pub left_right_calls: u32,
    /// The duration the last call asked for, in 100-nanosecond ticks.
    pub last_duration_ticks: i64,
    pub last_intensity: f32,
    pub last_large_motor: f32,
    pub last_small_motor: f32,
}

/// The vibration motor, and the substitute backend that makes it testable.
///
/// Windows Phone's `VibrateController`, which XNA exposed as a duration and
/// nothing else. CNA adds an intensity and a separate left/right pair, because
/// a gamepad has two motors and a phone has one.
pub mod vibrate_controller {
    use super::{string_view, VibrationTestLog};
    use crate::error::Result;
    use crate::game::GameContext;
    use crate::native::runtime::read_string;
    use cna_sys as sys;

    /// Whether this host has a vibration motor at all.
    pub fn is_supported(game: &GameContext<'_>) -> Result<bool> {
        let (native, handle) = game.native_game();
        let mut value = sys::CNA_FALSE;
        // SAFETY: the game handle is callback-live and the output is a local.
        native.check(unsafe {
            (native.runtime.vibrate_controller_get_is_supported_ext)(handle, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// The motor's device name, empty when there is none.
    pub fn device_name(game: &GameContext<'_>) -> Result<String> {
        let (native, handle) = game.native_game();
        let api = &native.runtime;
        read_string(
            |value| native.check(value),
            // SAFETY: callback-live handle, live outputs; the size-then-copy
            // pair.
            |bytes| unsafe { (api.vibrate_controller_get_device_name_size_ext)(handle, bytes) },
            |destination, capacity, written| unsafe {
                (api.vibrate_controller_copy_device_name_ext)(handle, destination, capacity, written)
            },
        )
    }

    /// Vibrates for a duration, in 100-nanosecond ticks.
    pub fn start(game: &GameContext<'_>, duration_ticks: i64) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: the game handle is callback-live and the duration is by value.
        native.check(unsafe { (native.runtime.vibrate_controller_start)(handle, duration_ticks) })
    }

    /// Vibrates at an intensity between 0 and 1.
    pub fn start_with_intensity(
        game: &GameContext<'_>,
        duration_ticks: i64,
        intensity: f32,
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: as above.
        native.check(unsafe {
            (native.runtime.vibrate_controller_start_with_intensity_ext)(
                handle,
                duration_ticks,
                intensity,
            )
        })
    }

    /// Drives the two motors separately, as a gamepad has them.
    pub fn start_left_right(
        game: &GameContext<'_>,
        large_motor: f32,
        small_motor: f32,
        duration_ticks: i64,
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: as above.
        native.check(unsafe {
            (native.runtime.vibrate_controller_start_left_right_ext)(
                handle,
                large_motor,
                small_motor,
                duration_ticks,
            )
        })
    }

    /// Stops immediately.
    pub fn stop(game: &GameContext<'_>) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: the game handle is callback-live.
        native.check(unsafe { (native.runtime.vibrate_controller_stop)(handle) })
    }

    /// Installs or removes CNA's substitute motor.
    ///
    /// `supported` is what [`is_supported`] will answer while it is installed,
    /// and `device_name` what [`device_name`] will.
    pub fn set_test_backend(
        game: &GameContext<'_>,
        installed: bool,
        supported: bool,
        device_name: &str,
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: the game handle is callback-live and the name is borrowed for
        // the call.
        native.check(unsafe {
            (native.runtime.vibrate_controller_set_test_backend_ext)(
                handle,
                u8::from(installed),
                u8::from(supported),
                string_view(device_name),
            )
        })
    }

    /// What the substitute motor has recorded.
    pub fn test_log(game: &GameContext<'_>) -> Result<VibrationTestLog> {
        let (native, handle) = game.native_game();
        let mut value = sys::CNA_VibrationTestLog {
            struct_size: core::mem::size_of::<sys::CNA_VibrationTestLog>() as u32,
            struct_version: 1,
            ..sys::CNA_VibrationTestLog::default()
        };
        // SAFETY: the game handle is callback-live and the output is a live
        // local whose size and version headers are set.
        native.check(unsafe {
            (native.runtime.vibrate_controller_get_test_log_ext)(handle, &mut value)
        })?;
        Ok(VibrationTestLog {
            start_calls: value.start_calls,
            stop_calls: value.stop_calls,
            left_right_calls: value.left_right_calls,
            last_duration_ticks: value.last_duration_ticks,
            last_intensity: value.last_intensity,
            last_large_motor: value.last_large_motor,
            last_small_motor: value.last_small_motor,
        })
    }
}

/// One name-and-pattern pair a file dialog filters by.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileDialogFilter {
    /// What the dialog shows the user, such as "PNG image".
    pub name: String,
    /// The platform's own pattern, such as `*.png`.
    pub pattern: String,
}

/// Native file dialogs, and the substitute backend that makes them testable.
///
/// # These are asynchronous, and that decides the Rust shape
///
/// Upstream is explicit: a show route "returns once the request has been made,
/// and the handler runs whenever the platform answers -- which may be long
/// afterwards, or never if the process exits first". There is no route to
/// withdraw a pending dialog, so nothing on this side can know when the
/// closure stops being reachable.
///
/// The handler is therefore **leaked deliberately**: one small boxed closure
/// per dialog shown, never freed. Freeing it when the handler runs would leave
/// the other case -- a dialog the platform never answers -- freeing memory CNA
/// still points at, which is worse than a bounded leak in a program that shows
/// a handful of dialogs in its life. A test backend answers immediately, so
/// this is only a real cost in a real session.
pub mod file_dialog {
    use super::{string_view, FileDialogFilter};
    use crate::error::Result;
    use crate::game::GameContext;
    use cna_sys as sys;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    type Handler = Box<dyn FnMut(Vec<String>) + Send + 'static>;

    unsafe extern "C" fn trampoline(
        files: *const sys::CNA_StringView,
        count: u64,
        context: *mut core::ffi::c_void,
    ) {
        if context.is_null() {
            return;
        }
        // SAFETY: the context is the leaked box this module made, and it is
        // never freed, so it is live for as long as CNA can call this.
        let handler = unsafe { &mut *context.cast::<Handler>() };
        let mut chosen = Vec::new();
        if !files.is_null() {
            let length = usize::try_from(count).unwrap_or(0);
            // SAFETY: CNA documents the array as `count` views borrowed for the
            // duration of this call; every one is copied before it returns.
            let views = unsafe { core::slice::from_raw_parts(files, length) };
            for view in views {
                let bytes = usize::try_from(view.byte_length).unwrap_or(0);
                if view.data.is_null() || bytes == 0 {
                    chosen.push(String::new());
                    continue;
                }
                // SAFETY: the view's bytes are counted UTF-8 borrowed for this
                // call.
                let slice =
                    unsafe { core::slice::from_raw_parts(view.data.cast::<u8>(), bytes) };
                chosen.push(String::from_utf8_lossy(slice).into_owned());
            }
        }
        // A panic must not cross back into C, and there is nowhere to report
        // one from inside a platform callback.
        let _ = catch_unwind(AssertUnwindSafe(|| handler(chosen)));
    }

    fn leak(handler: impl FnMut(Vec<String>) + Send + 'static) -> *mut core::ffi::c_void {
        let boxed: Handler = Box::new(handler);
        Box::into_raw(Box::new(boxed)).cast::<core::ffi::c_void>()
    }

    fn native_filters(filters: &[FileDialogFilter]) -> Vec<sys::CNA_FileDialogFilter> {
        filters
            .iter()
            .map(|filter| sys::CNA_FileDialogFilter {
                struct_size: core::mem::size_of::<sys::CNA_FileDialogFilter>() as u32,
                struct_version: 1,
                name: string_view(&filter.name),
                pattern: string_view(&filter.pattern),
            })
            .collect()
    }

    /// Whether this platform can show a file dialog at all.
    pub fn is_supported(game: &GameContext<'_>) -> Result<bool> {
        let (native, handle) = game.native_game();
        let mut value = sys::CNA_FALSE;
        // SAFETY: the game handle is callback-live and the output is a local.
        native
            .check(unsafe { (native.runtime.file_dialog_get_is_supported_ext)(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Asks for one or more files to open.
    pub fn show_open_file(
        game: &GameContext<'_>,
        filters: &[FileDialogFilter],
        default_location: &str,
        allow_multiple: bool,
        on_result: impl FnMut(Vec<String>) + Send + 'static,
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        let native_filters = native_filters(filters);
        let context = leak(on_result);
        // SAFETY: the game handle is callback-live, the filters and their
        // strings outlive the call, and the context is a box this module never
        // frees -- see the module documentation for why.
        native.check(unsafe {
            (native.runtime.file_dialog_show_open_file_ext)(
                handle,
                Some(trampoline),
                context,
                if native_filters.is_empty() {
                    core::ptr::null()
                } else {
                    native_filters.as_ptr()
                },
                native_filters.len() as u64,
                string_view(default_location),
                u8::from(allow_multiple),
            )
        })
    }

    /// Asks for one or more folders.
    pub fn show_open_folder(
        game: &GameContext<'_>,
        default_location: &str,
        allow_multiple: bool,
        on_result: impl FnMut(Vec<String>) + Send + 'static,
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        let context = leak(on_result);
        // SAFETY: as above.
        native.check(unsafe {
            (native.runtime.file_dialog_show_open_folder_ext)(
                handle,
                Some(trampoline),
                context,
                string_view(default_location),
                u8::from(allow_multiple),
            )
        })
    }

    /// Asks where to save a file.
    pub fn show_save_file(
        game: &GameContext<'_>,
        filters: &[FileDialogFilter],
        default_location: &str,
        on_result: impl FnMut(Vec<String>) + Send + 'static,
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        let native_filters = native_filters(filters);
        let context = leak(on_result);
        // SAFETY: as above.
        native.check(unsafe {
            (native.runtime.file_dialog_show_save_file_ext)(
                handle,
                Some(trampoline),
                context,
                if native_filters.is_empty() {
                    core::ptr::null()
                } else {
                    native_filters.as_ptr()
                },
                native_filters.len() as u64,
                string_view(default_location),
            )
        })
    }

    /// Installs or removes CNA's substitute dialog.
    ///
    /// `paths` is what every show route will answer with while it is
    /// installed, immediately rather than whenever a platform gets round to
    /// it -- which is what makes a dialog assertable in a test at all.
    pub fn set_test_backend(
        game: &GameContext<'_>,
        installed: bool,
        paths: &[&str],
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        let views: Vec<sys::CNA_StringView> = paths.iter().map(|path| string_view(path)).collect();
        // SAFETY: the game handle is callback-live and the array and its
        // strings outlive the call.
        native.check(unsafe {
            (native.runtime.file_dialog_set_test_backend_ext)(
                handle,
                u8::from(installed),
                if views.is_empty() {
                    core::ptr::null()
                } else {
                    views.as_ptr()
                },
                views.len() as u64,
            )
        })
    }
}

/// A tray icon and its menu.
///
/// `OWNED`: it holds a handle it releases exactly once, and it owns the click
/// handlers its entries carry. Dropping it removes the icon and frees them, in
/// that order -- the reverse would leave CNA holding a pointer to a dead
/// closure until the icon went.
pub struct SystemTray {
    native: std::sync::Arc<crate::native::Native>,
    handle: std::sync::Mutex<sys::CNA_SystemTrayHandle>,
    /// The boxed click handlers CNA holds pointers into. Never read here; they
    /// exist so the allocations outlive the tray.
    handlers: std::sync::Mutex<Vec<*mut core::ffi::c_void>>,
}

// SAFETY: the pointers are owned boxes this value alone frees, and the closures
// behind them are required to be `Send`. Nothing is shared.
unsafe impl Send for SystemTray {}

type TrayHandler = Box<dyn FnMut() + Send + 'static>;

unsafe extern "C" fn tray_trampoline(context: *mut core::ffi::c_void) {
    if context.is_null() {
        return;
    }
    // SAFETY: the context is a box the tray owns and keeps alive.
    let handler = unsafe { &mut *context.cast::<TrayHandler>() };
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler()));
}

impl SystemTray {
    /// Whether this platform has a system tray at all.
    pub fn is_supported(game: &GameContext<'_>) -> Result<bool> {
        let (native, handle) = game.native_game();
        let mut value = sys::CNA_FALSE;
        // SAFETY: the game handle is callback-live and the output is a local.
        native
            .check(unsafe { (native.runtime.system_tray_get_is_supported_ext)(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Creates a tray icon with a tooltip.
    pub fn new(game: &GameContext<'_>, tooltip: &str) -> Result<Self> {
        Self::create(game, tooltip, false)
    }

    /// Creates a tray icon backed by CNA's substitute tray.
    ///
    /// The icon does not appear anywhere; entries can still be added, read
    /// back and clicked through [`Self::click_entry_for_tests`], which is what
    /// makes a tray menu assertable on a machine with no desktop session.
    pub fn with_test_backend(game: &GameContext<'_>, tooltip: &str) -> Result<Self> {
        Self::create(game, tooltip, true)
    }

    fn create(game: &GameContext<'_>, tooltip: &str, test_backend: bool) -> Result<Self> {
        let (native, game_handle) = game.native_game();
        let mut handle = sys::CNA_INVALID_HANDLE;
        let route = if test_backend {
            native.runtime.system_tray_create_with_test_backend_ext
        } else {
            native.runtime.system_tray_create
        };
        // SAFETY: the game handle is callback-live, the tooltip is borrowed for
        // the call, and the output is a live local.
        native.check(unsafe { route(game_handle, string_view(tooltip), &mut handle) })?;
        Ok(Self {
            native: std::sync::Arc::clone(native),
            handle: std::sync::Mutex::new(handle),
            handlers: std::sync::Mutex::new(Vec::new()),
        })
    }

    fn get(&self) -> Result<sys::CNA_SystemTrayHandle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(crate::error::CnaError::InvalidInput(
                "the system tray has been released",
            ));
        }
        Ok(handle)
    }

    /// Replaces the icon's tooltip.
    pub fn set_tooltip(&self, tooltip: &str) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and the tooltip is borrowed for the call.
        self.native.check(unsafe {
            (self.native.runtime.system_tray_set_tooltip)(handle, string_view(tooltip))
        })
    }

    /// Appends a menu entry, answering its index.
    ///
    /// `checkable` decides whether the entry carries a check mark at all,
    /// which is a different question from whether it is currently checked.
    pub fn add_entry(
        &self,
        label: &str,
        enabled: bool,
        checkable: bool,
        checked: bool,
        on_click: impl FnMut() + Send + 'static,
    ) -> Result<u64> {
        let handle = self.get()?;
        let boxed: TrayHandler = Box::new(on_click);
        let context = Box::into_raw(Box::new(boxed)).cast::<core::ffi::c_void>();
        let mut index = 0_u64;
        // SAFETY: the handle is owned, the label is borrowed for the call, and
        // the context is a box this value takes ownership of below.
        let result = self.native.check(unsafe {
            (self.native.runtime.system_tray_add_entry)(
                handle,
                string_view(label),
                u8::from(enabled),
                u8::from(checkable),
                u8::from(checked),
                Some(tray_trampoline),
                context,
                &mut index,
            )
        });
        match result {
            Ok(()) => {
                self.handlers
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(context);
                Ok(index)
            }
            Err(error) => {
                // CNA never took the pointer, so this side still owns it.
                // SAFETY: the box was made two statements ago and handed to
                // nobody.
                drop(unsafe { Box::from_raw(context.cast::<TrayHandler>()) });
                Err(error)
            }
        }
    }

    /// Replaces one entry's label.
    pub fn set_entry_label(&self, index: u64, label: &str) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and the label is borrowed for the call.
        self.native.check(unsafe {
            (self.native.runtime.system_tray_set_entry_label)(handle, index, string_view(label))
        })
    }

    /// Whether one entry is enabled.
    pub fn entry_enabled(&self, index: u64) -> Result<bool> {
        let handle = self.get()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.system_tray_get_entry_enabled)(handle, index, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Enables or disables one entry.
    pub fn set_entry_enabled(&self, index: u64, enabled: bool) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and both values are by value.
        self.native.check(unsafe {
            (self.native.runtime.system_tray_set_entry_enabled)(handle, index, u8::from(enabled))
        })
    }

    /// Whether one entry is checked.
    pub fn entry_checked(&self, index: u64) -> Result<bool> {
        let handle = self.get()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.system_tray_get_entry_checked)(handle, index, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Checks or unchecks one entry.
    pub fn set_entry_checked(&self, index: u64, checked: bool) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and both values are by value.
        self.native.check(unsafe {
            (self.native.runtime.system_tray_set_entry_checked)(handle, index, u8::from(checked))
        })
    }

    /// Clicks one entry, as a user would.
    ///
    /// Only the substitute tray answers this; a real one has a real user. It
    /// is what lets a test drive the handler a menu entry carries.
    pub fn click_entry_for_tests(&self, index: u64) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and the index is by value.
        self.native
            .check(unsafe { (self.native.runtime.system_tray_click_entry_for_tests_ext)(handle, index) })
    }

    /// Removes the icon early.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle is owned by this value and released exactly once.
        let result = self
            .native
            .check(unsafe { (self.native.runtime.system_tray_destroy)(handle) });
        // Only now are the handlers unreachable from C.
        let mut handlers = self
            .handlers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for context in handlers.drain(..) {
            // SAFETY: each pointer came from `Box::into_raw` in `add_entry`,
            // and the tray that could reach it is gone.
            drop(unsafe { Box::from_raw(context.cast::<TrayHandler>()) });
        }
        result
    }
}

impl core::fmt::Debug for SystemTray {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SystemTray")
            .field(
                "live",
                &(*self
                    .handle
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    != sys::CNA_INVALID_HANDLE),
            )
            .finish()
    }
}

impl Drop for SystemTray {
    fn drop(&mut self) {
        let _ = self.release();
    }
}
