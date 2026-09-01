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
/// gesture, may ignore it. Read it back if the value matters.
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
