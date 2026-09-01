//! What a modern gamepad is, beyond XNA's four values.
//!
//! XNA's `GamePad` is two sticks, two triggers and a button mask, and
//! [`crate::input::gamepad`] projects exactly that. A controller made since
//! then also has a touchpad, motion sensors, a light bar, motors in its
//! triggers, a battery, a serial number, and its own idea of what its buttons
//! are called -- none of which is a value Rust holds, so all of it is here.
//!
//! # Absence is an answer here too
//!
//! Every accessor reports presence separately from value. A pad with no
//! gyroscope answers `None` rather than `(0, 0, 0)`, because a pad lying
//! perfectly still also reads zero and a game that cannot tell the two apart
//! will calibrate against a device that is not there.

#![allow(clippy::missing_errors_doc)]

use cna_sys as sys;

use crate::error::Result;
use crate::extensions::devices::PowerState;
use crate::game::GameContext;
use crate::input::PlayerIndex;

/// What a pad calls one of its buttons.
///
/// The same physical position is `A` on an Xbox pad and `Cross` on a
/// PlayStation one, and a game that draws a button prompt needs the pad's own
/// answer rather than XNA's name for the bit.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ButtonLabel {
    Unknown,
    A,
    B,
    X,
    Y,
    Cross,
    Circle,
    Square,
    Triangle,
    /// A label a newer CNA introduced.
    Unrecognized(u32),
}

impl ButtonLabel {
    const fn from_native(value: sys::CNA_GamePadButtonLabel) -> Self {
        match value {
            sys::CNA_GAMEPAD_BUTTON_LABEL_UNKNOWN => Self::Unknown,
            sys::CNA_GAMEPAD_BUTTON_LABEL_A => Self::A,
            sys::CNA_GAMEPAD_BUTTON_LABEL_B => Self::B,
            sys::CNA_GAMEPAD_BUTTON_LABEL_X => Self::X,
            sys::CNA_GAMEPAD_BUTTON_LABEL_Y => Self::Y,
            sys::CNA_GAMEPAD_BUTTON_LABEL_CROSS => Self::Cross,
            sys::CNA_GAMEPAD_BUTTON_LABEL_CIRCLE => Self::Circle,
            sys::CNA_GAMEPAD_BUTTON_LABEL_SQUARE => Self::Square,
            sys::CNA_GAMEPAD_BUTTON_LABEL_TRIANGLE => Self::Triangle,
            other => Self::Unrecognized(other),
        }
    }
}

/// How a pad is attached.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ConnectionState {
    Unknown,
    Wired,
    Wireless,
    /// A state a newer CNA introduced.
    Unrecognized(u32),
}

impl ConnectionState {
    const fn from_native(value: sys::CNA_GamePadConnectionState) -> Self {
        match value {
            sys::CNA_GAMEPAD_CONNECTION_STATE_UNKNOWN => Self::Unknown,
            sys::CNA_GAMEPAD_CONNECTION_STATE_WIRED => Self::Wired,
            sys::CNA_GAMEPAD_CONNECTION_STATE_WIRELESS => Self::Wireless,
            other => Self::Unrecognized(other),
        }
    }
}

/// One finger on a pad's touchpad.
///
/// `x` and `y` are normalised to the pad's surface; `pressure` is what the pad
/// reports and is not comparable between models.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct TouchpadFinger {
    pub is_down: bool,
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
}

/// A pad's battery, as a state and a percentage.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct PowerInfo {
    pub state: PowerState,
    /// Remaining charge, or `None` when the pad does not report one.
    pub percent: Option<i32>,
}

fn scalar<T: Copy + Default>(
    game: &GameContext<'_>,
    player: PlayerIndex,
    route: impl FnOnce(&crate::native::Native, sys::CNA_Handle, u32, *mut T) -> sys::CNA_Result,
) -> Result<T> {
    let (native, handle) = game.native_game();
    let mut value = T::default();
    native.check(route(native, handle, player as u32, &mut value))?;
    Ok(value)
}

/// Applies CNA's own axis dead-zone curve to one raw value.
///
/// The thresholds a caller passes are XNA's own constants. This exists because
/// a game that reads a raw axis from somewhere else -- a joystick, a network
/// packet -- and wants it to feel like a gamepad axis must use the same curve,
/// and re-deriving it is how two code paths end up feeling different.
pub fn exclude_axis_dead_zone(value: f32, dead_zone: f32) -> Result<f32> {
    let native = crate::native::Native::process()?;
    let mut result = 0.0_f32;
    // SAFETY: the output is a live local; both inputs are by value.
    native.check(unsafe {
        (native.gamepad_exclude_axis_dead_zone)(value, dead_zone, &mut result)
    })?;
    Ok(result)
}

/// The pad's motion sensors, touchpad, battery and identity.
pub mod pad {
    use super::{
        scalar, ButtonLabel, ConnectionState, PowerInfo, TouchpadFinger,
    };
    use crate::error::Result;
    use crate::extensions::devices::PowerState;
    use crate::game::GameContext;
    use crate::input::PlayerIndex;
    use crate::native::runtime::read_string;
    use crate::value::{Color, Vector3};
    use cna_sys as sys;

    /// The pad's accelerometer, or `None` when it has none.
    pub fn accelerometer(game: &GameContext<'_>, player: PlayerIndex) -> Result<Option<Vector3>> {
        motion(game, player, true)
    }

    /// The pad's gyroscope, or `None` when it has none.
    pub fn gyroscope(game: &GameContext<'_>, player: PlayerIndex) -> Result<Option<Vector3>> {
        motion(game, player, false)
    }

    fn motion(
        game: &GameContext<'_>,
        player: PlayerIndex,
        accelerometer: bool,
    ) -> Result<Option<Vector3>> {
        let (native, handle) = game.native_game();
        let mut value = sys::CNA_Vector3::default();
        let mut present = sys::CNA_FALSE;
        let route = if accelerometer {
            native.gamepad_get_accelerometer_ext
        } else {
            native.gamepad_get_gyro_ext
        };
        // SAFETY: the game handle is callback-live and both outputs are live
        // locals.
        native.check(unsafe { route(handle, player as u32, &mut value, &mut present) })?;
        Ok((present != sys::CNA_FALSE).then_some(Vector3 {
            X: value.x,
            Y: value.y,
            Z: value.z,
        }))
    }

    /// How many touchpads the pad has.
    pub fn touchpad_count(game: &GameContext<'_>, player: PlayerIndex) -> Result<i32> {
        scalar(game, player, |native, handle, index, out| {
            // SAFETY: callback-live handle, live output.
            unsafe { (native.gamepad_get_touchpad_count_ext)(handle, index, out) }
        })
    }

    /// How many fingers one touchpad can report at once.
    pub fn touchpad_finger_count(
        game: &GameContext<'_>,
        player: PlayerIndex,
        touchpad: i32,
    ) -> Result<i32> {
        let (native, handle) = game.native_game();
        let mut value = 0_i32;
        // SAFETY: callback-live handle, live output.
        native.check(unsafe {
            (native.gamepad_get_touchpad_finger_count_ext)(
                handle,
                player as u32,
                touchpad,
                &mut value,
            )
        })?;
        Ok(value)
    }

    /// One finger on one touchpad, or `None` when there is no such finger.
    pub fn touchpad_finger(
        game: &GameContext<'_>,
        player: PlayerIndex,
        touchpad: i32,
        finger: i32,
    ) -> Result<Option<TouchpadFinger>> {
        let (native, handle) = game.native_game();
        let mut value = sys::CNA_GamePadTouchpadFinger::default();
        let mut present = sys::CNA_FALSE;
        // SAFETY: callback-live handle, live outputs.
        native.check(unsafe {
            (native.gamepad_get_touchpad_finger_ext)(
                handle,
                player as u32,
                touchpad,
                finger,
                &mut value,
                &mut present,
            )
        })?;
        Ok((present != sys::CNA_FALSE).then_some(TouchpadFinger {
            is_down: value.is_down != sys::CNA_FALSE,
            x: value.x,
            y: value.y,
            pressure: value.pressure,
        }))
    }

    /// The pad's battery.
    pub fn power_info(game: &GameContext<'_>, player: PlayerIndex) -> Result<PowerInfo> {
        let (native, handle) = game.native_game();
        let mut state = 0_u32;
        let mut percent = -1_i32;
        // SAFETY: callback-live handle, live outputs.
        native.check(unsafe {
            (native.gamepad_get_power_info_ext)(handle, player as u32, &mut state, &mut percent)
        })?;
        Ok(PowerInfo {
            state: PowerState::from_native(state),
            // Upstream spells "the pad does not report one" as a negative.
            percent: (percent >= 0).then_some(percent),
        })
    }

    /// How the pad is attached.
    pub fn connection_state(
        game: &GameContext<'_>,
        player: PlayerIndex,
    ) -> Result<ConnectionState> {
        Ok(ConnectionState::from_native(scalar(
            game,
            player,
            |native, handle, index, out| {
                // SAFETY: callback-live handle, live output.
                unsafe { (native.gamepad_get_connection_state_ext)(handle, index, out) }
            },
        )?))
    }

    /// What the pad calls one of its buttons.
    pub fn button_label(
        game: &GameContext<'_>,
        player: PlayerIndex,
        button: u32,
    ) -> Result<ButtonLabel> {
        let (native, handle) = game.native_game();
        let mut value = 0_u32;
        // SAFETY: callback-live handle, live output.
        native.check(unsafe {
            (native.gamepad_get_button_label_ext)(handle, player as u32, button, &mut value)
        })?;
        Ok(ButtonLabel::from_native(value))
    }

    /// The pad's firmware version.
    pub fn firmware_version(game: &GameContext<'_>, player: PlayerIndex) -> Result<u16> {
        scalar(game, player, |native, handle, index, out| {
            // SAFETY: callback-live handle, live output.
            unsafe { (native.gamepad_get_firmware_version_ext)(handle, index, out) }
        })
    }

    /// The pad's Steam Input handle, or `None` when it has none.
    pub fn steam_handle(game: &GameContext<'_>, player: PlayerIndex) -> Result<Option<u64>> {
        let value: u64 = scalar(game, player, |native, handle, index, out| {
            // SAFETY: callback-live handle, live output.
            unsafe { (native.gamepad_get_steam_handle_ext)(handle, index, out) }
        })?;
        Ok((value != 0).then_some(value))
    }

    /// The pad's index as the platform assigns it, which is not XNA's.
    ///
    /// XNA's `PlayerIndex` is a slot a game addresses; this is the number the
    /// pad itself carries -- the lit quadrant on an Xbox pad -- and the two
    /// need not agree.
    pub fn player_index(game: &GameContext<'_>, player: PlayerIndex) -> Result<i32> {
        scalar(game, player, |native, handle, index, out| {
            // SAFETY: callback-live handle, live output.
            unsafe { (native.gamepad_get_player_index_ext)(handle, index, out) }
        })
    }

    /// Asks the pad to show a different index, answering whether it did.
    pub fn set_player_index(
        game: &GameContext<'_>,
        player: PlayerIndex,
        value: i32,
    ) -> Result<bool> {
        let (native, handle) = game.native_game();
        let mut applied = sys::CNA_FALSE;
        // SAFETY: callback-live handle, live output.
        native.check(unsafe {
            (native.gamepad_set_player_index_ext)(handle, player as u32, value, &mut applied)
        })?;
        Ok(applied != sys::CNA_FALSE)
    }

    /// Sets the pad's light bar, where it has one.
    ///
    /// Reports nothing about whether it took, because the route reports
    /// nothing: an empty slot accepts this as readily as a DualSense does --
    /// measured. Its neighbour [`set_trigger_vibration`] does answer, so where
    /// it matters, ask that one whether a pad is really there.
    pub fn set_light_bar(game: &GameContext<'_>, player: PlayerIndex, color: Color) -> Result<()> {
        let (native, handle) = game.native_game();
        let native_color = sys::CNA_Color {
            r: color.R(),
            g: color.G(),
            b: color.B(),
            a: color.A(),
        };
        // SAFETY: callback-live handle, colour by value.
        native.check(unsafe {
            (native.gamepad_set_light_bar_ext)(handle, player as u32, native_color)
        })
    }

    /// Drives the motors in the triggers, answering whether the pad has them.
    ///
    /// Distinct from the handle motors: a pad with trigger motors can push back
    /// against a finger, which is a different effect from shaking the case.
    pub fn set_trigger_vibration(
        game: &GameContext<'_>,
        player: PlayerIndex,
        left: f32,
        right: f32,
    ) -> Result<bool> {
        let (native, handle) = game.native_game();
        let mut applied = sys::CNA_FALSE;
        // SAFETY: callback-live handle, live output.
        native.check(unsafe {
            (native.gamepad_set_trigger_vibration_ext)(
                handle,
                player as u32,
                left,
                right,
                &mut applied,
            )
        })?;
        Ok(applied != sys::CNA_FALSE)
    }

    /// The pad's display name.
    pub fn name(game: &GameContext<'_>, player: PlayerIndex) -> Result<String> {
        text(game, player, TextField::Name)
    }

    /// The pad's stable identity, as the platform spells it.
    pub fn guid(game: &GameContext<'_>, player: PlayerIndex) -> Result<String> {
        text(game, player, TextField::Guid)
    }

    /// The device path the platform opened the pad at.
    pub fn path(game: &GameContext<'_>, player: PlayerIndex) -> Result<String> {
        text(game, player, TextField::Path)
    }

    /// The pad's serial number, empty when it reports none.
    pub fn serial(game: &GameContext<'_>, player: PlayerIndex) -> Result<String> {
        text(game, player, TextField::Serial)
    }

    #[derive(Clone, Copy)]
    enum TextField {
        Name,
        Guid,
        Path,
        Serial,
    }

    fn text(game: &GameContext<'_>, player: PlayerIndex, field: TextField) -> Result<String> {
        let (native, handle) = game.native_game();
        let index = player as u32;
        let (size, copy) = match field {
            TextField::Name => (
                native.gamepad_get_name_size_ext,
                native.gamepad_copy_name_ext,
            ),
            TextField::Guid => (
                native.gamepad_get_guid_size_ext,
                native.gamepad_copy_guid_ext,
            ),
            TextField::Path => (
                native.gamepad_get_path_size_ext,
                native.gamepad_copy_path_ext,
            ),
            TextField::Serial => (
                native.gamepad_get_serial_size_ext,
                native.gamepad_copy_serial_ext,
            ),
        };
        read_string(
            |value| native.check(value),
            // SAFETY: callback-live handle, live outputs; the size-then-copy
            // pair for one UTF-8 string.
            |bytes| unsafe { size(handle, index, bytes) },
            |destination, capacity, written| unsafe {
                copy(handle, index, destination, capacity, written)
            },
        )
    }
}
