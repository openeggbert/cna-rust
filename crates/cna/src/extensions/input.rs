//! CNA's raw joystick input.
//!
//! XNA had `GamePad` and nothing else: a device that is not an Xbox-shaped
//! controller could not be read at all. CNA exposes the raw device -- its
//! axes, buttons, hats and trackballs -- and that is a CNA concept, so it
//! lives here rather than beside `GamePad`.
//!
//! Reading a joystick needs a live game, because CNA reaches the input
//! subsystem through the game's platform binding.

#![allow(clippy::missing_errors_doc)]

use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::game::GameContext;
use crate::native::runtime::read_string;
use crate::native::Native;
use crate::value::Point;

/// What kind of device a joystick reports itself to be.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum JoystickType {
    Unknown,
    GamePad,
    Wheel,
    ArcadeStick,
    FlightStick,
    DancePad,
    Guitar,
    DrumKit,
    ArcadePad,
    Throttle,
    /// A type a newer CNA introduced.
    Unrecognized(u32),
}

impl JoystickType {
    const fn from_native(value: sys::CNA_JoystickType) -> Self {
        match value {
            sys::CNA_JOYSTICK_TYPE_UNKNOWN => Self::Unknown,
            sys::CNA_JOYSTICK_TYPE_GAMEPAD => Self::GamePad,
            sys::CNA_JOYSTICK_TYPE_WHEEL => Self::Wheel,
            sys::CNA_JOYSTICK_TYPE_ARCADE_STICK => Self::ArcadeStick,
            sys::CNA_JOYSTICK_TYPE_FLIGHT_STICK => Self::FlightStick,
            sys::CNA_JOYSTICK_TYPE_DANCE_PAD => Self::DancePad,
            sys::CNA_JOYSTICK_TYPE_GUITAR => Self::Guitar,
            sys::CNA_JOYSTICK_TYPE_DRUM_KIT => Self::DrumKit,
            sys::CNA_JOYSTICK_TYPE_ARCADE_PAD => Self::ArcadePad,
            sys::CNA_JOYSTICK_TYPE_THROTTLE => Self::Throttle,
            other => Self::Unrecognized(other),
        }
    }
}

/// Where a hat switch is pointing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum HatPosition {
    Centered,
    Up,
    Right,
    Down,
    Left,
    RightUp,
    RightDown,
    LeftUp,
    LeftDown,
    /// A position a newer CNA introduced.
    Unrecognized(u32),
}

impl HatPosition {
    const fn from_native(value: sys::CNA_JoystickHatPosition) -> Self {
        match value {
            sys::CNA_JOYSTICK_HAT_POSITION_CENTERED => Self::Centered,
            sys::CNA_JOYSTICK_HAT_POSITION_UP => Self::Up,
            sys::CNA_JOYSTICK_HAT_POSITION_RIGHT => Self::Right,
            sys::CNA_JOYSTICK_HAT_POSITION_DOWN => Self::Down,
            sys::CNA_JOYSTICK_HAT_POSITION_LEFT => Self::Left,
            sys::CNA_JOYSTICK_HAT_POSITION_RIGHT_UP => Self::RightUp,
            sys::CNA_JOYSTICK_HAT_POSITION_RIGHT_DOWN => Self::RightDown,
            sys::CNA_JOYSTICK_HAT_POSITION_LEFT_UP => Self::LeftUp,
            sys::CNA_JOYSTICK_HAT_POSITION_LEFT_DOWN => Self::LeftDown,
            other => Self::Unrecognized(other),
        }
    }
}

/// One connected joystick, as enumeration reports it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JoystickInfo {
    /// The instance identifier the capability and capture routes take.
    pub id: u32,
    pub kind: JoystickType,
    pub name: String,
}

/// What one joystick can do.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JoystickCapabilities {
    pub axis_count: i32,
    pub button_count: i32,
    pub hat_count: i32,
    pub ball_count: i32,
    pub kind: JoystickType,
    pub power_state: super::devices::PowerState,
    /// Battery charge, or `None` when unknown or disconnected.
    pub power_percent: Option<i32>,
    pub is_connected: bool,
    pub name: String,
    /// The device GUID, stable across reconnections of the same hardware.
    pub guid: String,
}

/// One captured joystick snapshot.
///
/// XNA's input states are values; CNA's raw joystick state is an owned native
/// object, because a device's axis and button counts are not known until it is
/// read. It is released by `Drop`.
#[derive(Debug)]
pub struct JoystickState {
    native: Arc<Native>,
    handle: sys::CNA_JoystickStateHandle,
}

/// How many joysticks are connected.
pub fn count(game: &GameContext<'_>) -> Result<u32> {
    let (native, handle) = game.native_game();
    let mut value = 0;
    // SAFETY: the game handle is callback-live and the output is a live local.
    native.check(unsafe { (native.runtime.joysticks_count)(handle, &mut value) })?;
    Ok(value)
}

/// Enumerates the connected joysticks in index order.
///
/// The index is a position in this list and is **not** the identifier the
/// other routes take; that is `JoystickInfo::id`, which survives another
/// device disconnecting.
pub fn enumerate(game: &GameContext<'_>) -> Result<Vec<JoystickInfo>> {
    let (native, handle) = game.native_game();
    let api = &native.runtime;
    let total = count(game)?;
    let mut result = Vec::new();
    for index in 0..total {
        let mut info = sys::CNA_JoystickInfo {
            struct_size: core::mem::size_of::<sys::CNA_JoystickInfo>() as u32,
            struct_version: sys::CNA_JOYSTICK_STRUCT_VERSION,
            ..sys::CNA_JoystickInfo::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output whose
        // prefix this build declares exactly.
        native.check(unsafe { (api.joysticks_info_at)(handle, index, &mut info) })?;
        let name = read_string(
            |value| native.check(value),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.joysticks_name_size_at)(handle, index, bytes) },
            |destination, capacity, written| unsafe {
                (api.joysticks_copy_name_at)(handle, index, destination, capacity, written)
            },
        )?;
        result.push(JoystickInfo {
            id: info.id,
            kind: JoystickType::from_native(info.r#type),
            name,
        });
    }
    Ok(result)
}

/// What the joystick with this instance identifier can do.
pub fn capabilities(game: &GameContext<'_>, id: u32) -> Result<JoystickCapabilities> {
    let (native, handle) = game.native_game();
    let api = &native.runtime;
    let mut value = sys::CNA_JoystickCapabilities {
        struct_size: core::mem::size_of::<sys::CNA_JoystickCapabilities>() as u32,
        struct_version: sys::CNA_JOYSTICK_STRUCT_VERSION,
        ..sys::CNA_JoystickCapabilities::default()
    };
    // SAFETY: the descriptor is a caller-owned versioned output whose prefix
    // this build declares exactly.
    native.check(unsafe { (api.joysticks_capabilities)(handle, id, &mut value) })?;
    let name = read_string(
        |result| native.check(result),
        // SAFETY: both outputs are live locals; the two routes form CNA's
        // canonical size-then-copy pair for one UTF-8 string.
        |bytes| unsafe { (api.joysticks_capabilities_name_size)(handle, id, bytes) },
        |destination, capacity, written| unsafe {
            (api.joysticks_copy_capabilities_name)(handle, id, destination, capacity, written)
        },
    )?;
    let guid = read_string(
        |result| native.check(result),
        // SAFETY: as above.
        |bytes| unsafe { (api.joysticks_capabilities_guid_size)(handle, id, bytes) },
        |destination, capacity, written| unsafe {
            (api.joysticks_copy_capabilities_guid)(handle, id, destination, capacity, written)
        },
    )?;
    Ok(JoystickCapabilities {
        axis_count: value.axis_count,
        button_count: value.button_count,
        hat_count: value.hat_count,
        ball_count: value.ball_count,
        kind: JoystickType::from_native(value.r#type),
        power_state: super::devices::PowerState::from_native_value(value.power_state),
        power_percent: (value.power_percent >= 0).then_some(value.power_percent),
        is_connected: value.is_connected != sys::CNA_FALSE,
        name,
        guid,
    })
}

/// Captures the current state of one joystick.
///
/// An identifier that names no connected device is **not** an error: the
/// capture succeeds and every array is empty, which is what the canonical
/// query does. Use [`capabilities`] and its `is_connected` to tell an absent
/// device from an idle one.
///
/// Trackball values are relative motion since the previous read, so capturing
/// consumes them: two captures in a row report the movement once.
pub fn capture(game: &GameContext<'_>, id: u32) -> Result<JoystickState> {
    let (native, handle) = game.native_game();
    let mut state = sys::CNA_INVALID_HANDLE;
    // SAFETY: the game handle is callback-live and the output receives a newly
    // owned snapshot handle.
    native.check(unsafe { (native.runtime.joysticks_capture_state)(handle, id, &mut state) })?;
    Ok(JoystickState {
        native: Arc::clone(native),
        handle: state,
    })
}

macro_rules! snapshot_vector {
    ($name:ident, $element:ty, $count_slot:ident, $copy_slot:ident, $map:expr) => {
        pub fn $name(&self) -> Result<Vec<$element>> {
            let api = &self.native.runtime;
            let mut count = 0_u32;
            // SAFETY: the snapshot handle is owned and the output is live.
            self.native
                .check(unsafe { (api.$count_slot)(self.handle, &mut count) })?;
            let capacity = usize::try_from(count)
                .map_err(|_| CnaError::InvalidInput("joystick element count is too large"))?;
            let mut raw = vec![Default::default(); capacity];
            if capacity == 0 {
                return Ok(Vec::new());
            }
            let mut copied = 0_u64;
            // SAFETY: `raw` holds exactly `count` writable elements.
            self.native.check(unsafe {
                (api.$copy_slot)(self.handle, raw.as_mut_ptr(), count as u64, &mut copied)
            })?;
            let copied = usize::try_from(copied)
                .map_err(|_| CnaError::InvalidInput("joystick element count is too large"))?;
            raw.truncate(copied.min(capacity));
            Ok(raw.into_iter().map($map).collect())
        }
    };
}

impl JoystickState {
    snapshot_vector!(axes, i16, joystick_axis_count, joystick_copy_axes, |value| value);
    snapshot_vector!(
        buttons,
        bool,
        joystick_button_count,
        joystick_copy_buttons,
        |value: sys::CNA_Bool| value != sys::CNA_FALSE
    );
    snapshot_vector!(
        hats,
        HatPosition,
        joystick_hat_count,
        joystick_copy_hats,
        HatPosition::from_native
    );
    snapshot_vector!(
        balls,
        Point,
        joystick_ball_count,
        joystick_copy_balls,
        |value: sys::CNA_Point| Point::new(value.x, value.y)
    );

    /// Whether two snapshots describe the same device state.
    ///
    /// The comparison is CNA's, not a field-by-field one here: it is what
    /// decides whether anything the device reports has changed.
    pub fn equals(&self, other: &Self) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: both snapshot handles are owned and live; the output is a
        // live local.
        self.native.check(unsafe {
            (self.native.runtime.joystick_state_equals)(self.handle, other.handle, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }
}

impl Drop for JoystickState {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        let _ = unsafe { (self.native.runtime.joystick_state_destroy)(self.handle) };
    }
}
