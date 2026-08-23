#![allow(non_snake_case, non_upper_case_globals)]

use core::any::Any;
use core::mem::size_of;
use core::ops::{BitAnd, BitOr, BitOrAssign};

use cna_sys as sys;

use crate::error::Result;
use crate::game::GameContext;
use crate::value::{vector_support::xna_f32_hash, Vector2};

use super::PlayerIndex;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ButtonState {
    Released = 0,
    Pressed = 1,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self::Released
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Buttons(i32);

impl Buttons {
    pub const A: Self = Self(0x1000);
    pub const B: Self = Self(0x2000);
    pub const X: Self = Self(0x4000);
    pub const Y: Self = Self(0x8000);
    pub const Back: Self = Self(0x20);
    pub const Start: Self = Self(0x10);
    pub const DPadUp: Self = Self(1);
    pub const DPadDown: Self = Self(2);
    pub const DPadLeft: Self = Self(4);
    pub const DPadRight: Self = Self(8);
    pub const LeftShoulder: Self = Self(0x100);
    pub const RightShoulder: Self = Self(0x200);
    pub const LeftStick: Self = Self(0x40);
    pub const RightStick: Self = Self(0x80);
    pub const BigButton: Self = Self(0x800);
    pub const LeftThumbstickLeft: Self = Self(0x20_0000);
    pub const LeftThumbstickRight: Self = Self(0x4000_0000);
    pub const LeftThumbstickDown: Self = Self(0x2000_0000);
    pub const LeftThumbstickUp: Self = Self(0x1000_0000);
    pub const RightThumbstickLeft: Self = Self(0x0800_0000);
    pub const RightThumbstickRight: Self = Self(0x0400_0000);
    pub const RightThumbstickDown: Self = Self(0x0200_0000);
    pub const RightThumbstickUp: Self = Self(0x0100_0000);
    pub const LeftTrigger: Self = Self(0x0080_0000);
    pub const RightTrigger: Self = Self(0x0040_0000);

    const Empty: Self = Self(0);
    const PhysicalMask: Self = Self(64_511);
    const fn contains(self, value: Self) -> bool {
        self.0 & value.0 == value.0
    }
}

impl BitAnd for Buttons {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitOr for Buttons {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl BitOrAssign for Buttons {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

fn state(value: Buttons, flag: Buttons) -> ButtonState {
    if value.contains(flag) {
        ButtonState::Pressed
    } else {
        ButtonState::Released
    }
}

fn smart_hash(words: &[i32]) -> i32 {
    let value = words.iter().fold(0, |hash, word| hash ^ word);
    if value == 0 {
        i32::MAX
    } else {
        value
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GamePadButtons {
    a: ButtonState,
    b: ButtonState,
    x: ButtonState,
    y: ButtonState,
    left_stick: ButtonState,
    right_stick: ButtonState,
    left_shoulder: ButtonState,
    right_shoulder: ButtonState,
    back: ButtonState,
    start: ButtonState,
    big_button: ButtonState,
}

impl GamePadButtons {
    #[must_use]
    pub fn new(buttons: Buttons) -> Self {
        Self {
            a: state(buttons, Buttons::A),
            b: state(buttons, Buttons::B),
            x: state(buttons, Buttons::X),
            y: state(buttons, Buttons::Y),
            left_stick: state(buttons, Buttons::LeftStick),
            right_stick: state(buttons, Buttons::RightStick),
            left_shoulder: state(buttons, Buttons::LeftShoulder),
            right_shoulder: state(buttons, Buttons::RightShoulder),
            back: state(buttons, Buttons::Back),
            start: state(buttons, Buttons::Start),
            big_button: state(buttons, Buttons::BigButton),
        }
    }
    #[must_use]
    pub const fn A(&self) -> ButtonState {
        self.a
    }
    #[must_use]
    pub const fn B(&self) -> ButtonState {
        self.b
    }
    #[must_use]
    pub const fn X(&self) -> ButtonState {
        self.x
    }
    #[must_use]
    pub const fn Y(&self) -> ButtonState {
        self.y
    }
    #[must_use]
    pub const fn Back(&self) -> ButtonState {
        self.back
    }
    #[must_use]
    pub const fn Start(&self) -> ButtonState {
        self.start
    }
    #[must_use]
    pub const fn LeftShoulder(&self) -> ButtonState {
        self.left_shoulder
    }
    #[must_use]
    pub const fn LeftStick(&self) -> ButtonState {
        self.left_stick
    }
    #[must_use]
    pub const fn RightShoulder(&self) -> ButtonState {
        self.right_shoulder
    }
    #[must_use]
    pub const fn RightStick(&self) -> ButtonState {
        self.right_stick
    }
    #[must_use]
    pub const fn BigButton(&self) -> ButtonState {
        self.big_button
    }
    #[must_use]
    pub fn Equals(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }
    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        smart_hash(&[
            self.a as i32,
            self.b as i32,
            self.x as i32,
            self.y as i32,
            self.left_shoulder as i32,
            self.right_shoulder as i32,
            self.left_stick as i32,
            self.right_stick as i32,
            self.start as i32,
            self.back as i32,
            self.big_button as i32,
        ])
    }
    #[must_use]
    pub fn ToString(&self) -> String {
        let values = [
            (self.a, "A"),
            (self.b, "B"),
            (self.x, "X"),
            (self.y, "Y"),
            (self.left_shoulder, "LeftShoulder"),
            (self.right_shoulder, "RightShoulder"),
            (self.left_stick, "LeftStick"),
            (self.right_stick, "RightStick"),
            (self.start, "Start"),
            (self.back, "Back"),
            (self.big_button, "BigButton"),
        ];
        let names: Vec<_> = values
            .iter()
            .filter_map(|(value, name)| (*value == ButtonState::Pressed).then_some(*name))
            .collect();
        format!(
            "{{Buttons:{}}}",
            if names.is_empty() {
                "None".to_owned()
            } else {
                names.join(" ")
            }
        )
    }
    fn add_to(self, result: &mut Buttons) {
        for (value, flag) in [
            (self.a, Buttons::A),
            (self.b, Buttons::B),
            (self.x, Buttons::X),
            (self.y, Buttons::Y),
            (self.back, Buttons::Back),
            (self.start, Buttons::Start),
            (self.big_button, Buttons::BigButton),
            (self.left_shoulder, Buttons::LeftShoulder),
            (self.right_shoulder, Buttons::RightShoulder),
            (self.left_stick, Buttons::LeftStick),
            (self.right_stick, Buttons::RightStick),
        ] {
            if value == ButtonState::Pressed {
                *result |= flag;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GamePadDPad {
    up: ButtonState,
    right: ButtonState,
    down: ButtonState,
    left: ButtonState,
}
impl GamePadDPad {
    #[must_use]
    pub const fn new(
        upValue: ButtonState,
        downValue: ButtonState,
        leftValue: ButtonState,
        rightValue: ButtonState,
    ) -> Self {
        Self {
            up: upValue,
            right: rightValue,
            down: downValue,
            left: leftValue,
        }
    }
    #[must_use]
    pub const fn Up(&self) -> ButtonState {
        self.up
    }
    #[must_use]
    pub const fn Down(&self) -> ButtonState {
        self.down
    }
    #[must_use]
    pub const fn Right(&self) -> ButtonState {
        self.right
    }
    #[must_use]
    pub const fn Left(&self) -> ButtonState {
        self.left
    }
    #[must_use]
    pub fn Equals(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }
    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        smart_hash(&[
            self.up as i32,
            self.down as i32,
            self.left as i32,
            self.right as i32,
        ])
    }
    #[must_use]
    pub fn ToString(&self) -> String {
        let values = [
            (self.up, "Up"),
            (self.down, "Down"),
            (self.left, "Left"),
            (self.right, "Right"),
        ];
        let names: Vec<_> = values
            .iter()
            .filter_map(|(value, name)| (*value == ButtonState::Pressed).then_some(*name))
            .collect();
        format!(
            "{{DPad:{}}}",
            if names.is_empty() {
                "None".to_owned()
            } else {
                names.join(" ")
            }
        )
    }
    fn add_to(self, result: &mut Buttons) {
        for (value, flag) in [
            (self.up, Buttons::DPadUp),
            (self.down, Buttons::DPadDown),
            (self.left, Buttons::DPadLeft),
            (self.right, Buttons::DPadRight),
        ] {
            if value == ButtonState::Pressed {
                *result |= flag;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GamePadTriggers {
    left: f32,
    right: f32,
}
impl GamePadTriggers {
    #[must_use]
    pub fn new(leftTrigger: f32, rightTrigger: f32) -> Self {
        fn clamp(value: f32) -> f32 {
            let value = if value.is_nan() {
                value
            } else if value < 1.0 {
                value
            } else {
                1.0
            };
            if value.is_nan() {
                value
            } else if value > 0.0 {
                value
            } else {
                0.0
            }
        }
        Self {
            left: clamp(leftTrigger),
            right: clamp(rightTrigger),
        }
    }
    #[must_use]
    pub const fn Left(&self) -> f32 {
        self.left
    }
    #[must_use]
    pub const fn Right(&self) -> f32 {
        self.right
    }
    #[must_use]
    pub fn Equals(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }
    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        smart_hash(&[xna_f32_hash(self.left), xna_f32_hash(self.right)])
    }
    #[must_use]
    pub fn ToString(&self) -> String {
        format!("{{Left:{} Right:{}}}", self.left, self.right)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GamePadThumbSticks {
    left: Vector2,
    right: Vector2,
}
impl GamePadThumbSticks {
    #[must_use]
    pub fn new(leftThumbstick: Vector2, rightThumbstick: Vector2) -> Self {
        Self {
            left: Vector2::Max(Vector2::Min(leftThumbstick, Vector2::One), -Vector2::One),
            right: Vector2::Max(Vector2::Min(rightThumbstick, Vector2::One), -Vector2::One),
        }
    }
    #[must_use]
    pub const fn Left(&self) -> Vector2 {
        self.left
    }
    #[must_use]
    pub const fn Right(&self) -> Vector2 {
        self.right
    }
    #[must_use]
    pub fn Equals(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }
    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        smart_hash(&[
            xna_f32_hash(self.left.X),
            xna_f32_hash(self.left.Y),
            xna_f32_hash(self.right.X),
            xna_f32_hash(self.right.Y),
        ])
    }
    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "{{Left:{} Right:{}}}",
            self.left.ToString(),
            self.right.ToString()
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GamePadState {
    connected: bool,
    packet: i32,
    thumbs: GamePadThumbSticks,
    triggers: GamePadTriggers,
    buttons: GamePadButtons,
    dpad: GamePadDPad,
    raw_buttons: Buttons,
}

impl GamePadState {
    #[must_use]
    pub fn new(
        thumbSticks: GamePadThumbSticks,
        triggers: GamePadTriggers,
        buttons: GamePadButtons,
        dPad: GamePadDPad,
    ) -> Self {
        let raw_buttons = Self::build_raw_buttons(thumbSticks, triggers, buttons, dPad);
        Self {
            connected: true,
            packet: 0,
            thumbs: thumbSticks,
            triggers,
            buttons,
            dpad: dPad,
            raw_buttons,
        }
    }
    #[must_use]
    pub fn from_left_thumb_stick_and_right_thumb_stick_and_left_trigger_and_right_trigger_and_buttons(
        leftThumbStick: Vector2,
        rightThumbStick: Vector2,
        leftTrigger: f32,
        rightTrigger: f32,
        buttons: &[Buttons],
    ) -> Self {
        let combined = buttons.iter().copied().fold(Buttons::Empty, BitOr::bitor);
        let thumb_sticks = GamePadThumbSticks::new(leftThumbStick, rightThumbStick);
        let triggers = GamePadTriggers::new(leftTrigger, rightTrigger);
        let gamepad_buttons = GamePadButtons::new(combined & Buttons::PhysicalMask);
        let dpad = GamePadDPad::new(
            state(combined, Buttons::DPadUp),
            state(combined, Buttons::DPadDown),
            state(combined, Buttons::DPadLeft),
            state(combined, Buttons::DPadRight),
        );
        Self::new(thumb_sticks, triggers, gamepad_buttons, dpad)
    }
    #[must_use]
    pub const fn Buttons(&self) -> GamePadButtons {
        self.buttons
    }
    #[must_use]
    pub const fn DPad(&self) -> GamePadDPad {
        self.dpad
    }
    #[must_use]
    pub const fn IsConnected(&self) -> bool {
        self.connected
    }
    #[must_use]
    pub const fn PacketNumber(&self) -> i32 {
        self.packet
    }
    #[must_use]
    pub const fn ThumbSticks(&self) -> GamePadThumbSticks {
        self.thumbs
    }
    #[must_use]
    pub const fn Triggers(&self) -> GamePadTriggers {
        self.triggers
    }
    #[must_use]
    pub const fn IsButtonDown(&self, button: Buttons) -> bool {
        self.raw_buttons.contains(button)
    }
    #[must_use]
    pub const fn IsButtonUp(&self, button: Buttons) -> bool {
        !self.IsButtonDown(button)
    }
    #[must_use]
    pub fn Equals(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }
    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        self.thumbs.GetHashCode()
            ^ self.triggers.GetHashCode()
            ^ self.buttons.GetHashCode()
            ^ i32::from(self.connected)
            ^ self.dpad.GetHashCode()
            ^ self.packet
    }
    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "{{IsConnected:{}}}",
            if self.connected { "True" } else { "False" }
        )
    }

    fn build_raw_buttons(
        thumbs: GamePadThumbSticks,
        triggers: GamePadTriggers,
        buttons: GamePadButtons,
        dpad: GamePadDPad,
    ) -> Buttons {
        let mut result = Buttons::Empty;
        let left_x = (thumbs.left.X * 32_767.0) as i16;
        let left_y = (thumbs.left.Y * 32_767.0) as i16;
        let right_x = (thumbs.right.X * 32_767.0) as i16;
        let right_y = (thumbs.right.Y * 32_767.0) as i16;
        let left_trigger = (triggers.left * 255.0) as u8;
        let right_trigger = (triggers.right * 255.0) as u8;
        if left_x < -7849 {
            result |= Buttons::LeftThumbstickLeft;
        }
        if left_x > 7849 {
            result |= Buttons::LeftThumbstickRight;
        }
        if left_y < -7849 {
            result |= Buttons::LeftThumbstickDown;
        }
        if left_y > 7849 {
            result |= Buttons::LeftThumbstickUp;
        }
        if right_x < -8689 {
            result |= Buttons::RightThumbstickLeft;
        }
        if right_x > 8689 {
            result |= Buttons::RightThumbstickRight;
        }
        if right_y < -8689 {
            result |= Buttons::RightThumbstickDown;
        }
        if right_y > 8689 {
            result |= Buttons::RightThumbstickUp;
        }
        if left_trigger > 30 {
            result |= Buttons::LeftTrigger;
        }
        if right_trigger > 30 {
            result |= Buttons::RightTrigger;
        }
        buttons.add_to(&mut result);
        dpad.add_to(&mut result);
        result
    }

    fn from_native(value: sys::CNA_GamePadState) -> Self {
        let raw_buttons = Buttons(value.pressed_buttons as i32);
        let thumbs = GamePadThumbSticks::new(
            Vector2::from_x_and_y(
                value.analog.left_thumb_stick.x,
                value.analog.left_thumb_stick.y,
            ),
            Vector2::from_x_and_y(
                value.analog.right_thumb_stick.x,
                value.analog.right_thumb_stick.y,
            ),
        );
        let triggers = GamePadTriggers::new(value.analog.left_trigger, value.analog.right_trigger);
        let buttons = GamePadButtons::new(raw_buttons & Buttons::PhysicalMask);
        let dpad = GamePadDPad::new(
            state(raw_buttons, Buttons::DPadUp),
            state(raw_buttons, Buttons::DPadDown),
            state(raw_buttons, Buttons::DPadLeft),
            state(raw_buttons, Buttons::DPadRight),
        );
        Self {
            connected: value.is_connected != sys::CNA_FALSE,
            packet: value.packet_number,
            thumbs,
            triggers,
            buttons,
            dpad,
            raw_buttons,
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GamePadDeadZone {
    None = 0,
    IndependentAxes = 1,
    Circular = 2,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GamePadType {
    Unknown = 0,
    ArcadeStick = 3,
    DancePad = 5,
    FlightStick = 4,
    GamePad = 1,
    Wheel = 2,
    Guitar = 6,
    DrumKit = 8,
    AlternateGuitar = 7,
    BigButtonPad = 768,
}
impl Default for GamePadType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GamePadCapabilities {
    connected: bool,
    gamepad_type: GamePadType,
    values: [bool; 23],
    voice_support: bool,
}
impl GamePadCapabilities {
    #[must_use]
    pub const fn GamePadType(&self) -> GamePadType {
        self.gamepad_type
    }
    #[must_use]
    pub const fn IsConnected(&self) -> bool {
        self.connected
    }
    #[must_use]
    pub const fn HasAButton(&self) -> bool {
        self.values[0]
    }
    #[must_use]
    pub const fn HasBackButton(&self) -> bool {
        self.values[1]
    }
    #[must_use]
    pub const fn HasBButton(&self) -> bool {
        self.values[2]
    }
    #[must_use]
    pub const fn HasDPadDownButton(&self) -> bool {
        self.values[3]
    }
    #[must_use]
    pub const fn HasDPadLeftButton(&self) -> bool {
        self.values[4]
    }
    #[must_use]
    pub const fn HasDPadRightButton(&self) -> bool {
        self.values[5]
    }
    #[must_use]
    pub const fn HasDPadUpButton(&self) -> bool {
        self.values[6]
    }
    #[must_use]
    pub const fn HasLeftShoulderButton(&self) -> bool {
        self.values[7]
    }
    #[must_use]
    pub const fn HasLeftStickButton(&self) -> bool {
        self.values[8]
    }
    #[must_use]
    pub const fn HasRightShoulderButton(&self) -> bool {
        self.values[9]
    }
    #[must_use]
    pub const fn HasRightStickButton(&self) -> bool {
        self.values[10]
    }
    #[must_use]
    pub const fn HasStartButton(&self) -> bool {
        self.values[11]
    }
    #[must_use]
    pub const fn HasXButton(&self) -> bool {
        self.values[12]
    }
    #[must_use]
    pub const fn HasYButton(&self) -> bool {
        self.values[13]
    }
    #[must_use]
    pub const fn HasBigButton(&self) -> bool {
        self.values[14]
    }
    #[must_use]
    pub const fn HasLeftXThumbStick(&self) -> bool {
        self.values[15]
    }
    #[must_use]
    pub const fn HasLeftYThumbStick(&self) -> bool {
        self.values[16]
    }
    #[must_use]
    pub const fn HasRightXThumbStick(&self) -> bool {
        self.values[17]
    }
    #[must_use]
    pub const fn HasRightYThumbStick(&self) -> bool {
        self.values[18]
    }
    #[must_use]
    pub const fn HasLeftTrigger(&self) -> bool {
        self.values[19]
    }
    #[must_use]
    pub const fn HasRightTrigger(&self) -> bool {
        self.values[20]
    }
    #[must_use]
    pub const fn HasLeftVibrationMotor(&self) -> bool {
        self.values[21]
    }
    #[must_use]
    pub const fn HasRightVibrationMotor(&self) -> bool {
        self.values[22]
    }
    #[must_use]
    pub const fn HasVoiceSupport(&self) -> bool {
        self.voice_support
    }

    fn from_native(value: sys::CNA_GamePadCapabilities) -> Self {
        Self {
            connected: value.is_connected != sys::CNA_FALSE,
            gamepad_type: GamePadType::from_native(value.gamepad_type),
            values: [
                value.has_a_button != sys::CNA_FALSE,
                value.has_back_button != sys::CNA_FALSE,
                value.has_b_button != sys::CNA_FALSE,
                value.has_dpad_down_button != sys::CNA_FALSE,
                value.has_dpad_left_button != sys::CNA_FALSE,
                value.has_dpad_right_button != sys::CNA_FALSE,
                value.has_dpad_up_button != sys::CNA_FALSE,
                value.has_left_shoulder_button != sys::CNA_FALSE,
                value.has_left_stick_button != sys::CNA_FALSE,
                value.has_right_shoulder_button != sys::CNA_FALSE,
                value.has_right_stick_button != sys::CNA_FALSE,
                value.has_start_button != sys::CNA_FALSE,
                value.has_x_button != sys::CNA_FALSE,
                value.has_y_button != sys::CNA_FALSE,
                value.has_big_button != sys::CNA_FALSE,
                value.has_left_x_thumb_stick != sys::CNA_FALSE,
                value.has_left_y_thumb_stick != sys::CNA_FALSE,
                value.has_right_x_thumb_stick != sys::CNA_FALSE,
                value.has_right_y_thumb_stick != sys::CNA_FALSE,
                value.has_left_trigger != sys::CNA_FALSE,
                value.has_right_trigger != sys::CNA_FALSE,
                value.has_left_vibration_motor != sys::CNA_FALSE,
                value.has_right_vibration_motor != sys::CNA_FALSE,
            ],
            voice_support: value.has_voice_support != sys::CNA_FALSE,
        }
    }
}

impl GamePadType {
    const fn from_native(value: sys::CNA_GamePadType) -> Self {
        match value {
            sys::CNA_GAMEPAD_TYPE_GAMEPAD => Self::GamePad,
            sys::CNA_GAMEPAD_TYPE_WHEEL => Self::Wheel,
            sys::CNA_GAMEPAD_TYPE_ARCADE_STICK => Self::ArcadeStick,
            sys::CNA_GAMEPAD_TYPE_FLIGHT_STICK => Self::FlightStick,
            sys::CNA_GAMEPAD_TYPE_DANCE_PAD => Self::DancePad,
            sys::CNA_GAMEPAD_TYPE_GUITAR => Self::Guitar,
            sys::CNA_GAMEPAD_TYPE_ALTERNATE_GUITAR => Self::AlternateGuitar,
            sys::CNA_GAMEPAD_TYPE_DRUM_KIT => Self::DrumKit,
            sys::CNA_GAMEPAD_TYPE_BIG_BUTTON_PAD => Self::BigButtonPad,
            _ => Self::Unknown,
        }
    }
}

pub struct GamePad;

impl GamePad {
    pub fn GetState(game: &GameContext<'_>, playerIndex: PlayerIndex) -> Result<GamePadState> {
        let mut state = new_native_state();
        game.native
            .gamepad_state(game.handle, playerIndex as u32, &mut state)?;
        Ok(GamePadState::from_native(state))
    }

    pub fn GetStateWithPlayerIndexAndDeadZoneMode(
        game: &GameContext<'_>,
        playerIndex: PlayerIndex,
        deadZoneMode: GamePadDeadZone,
    ) -> Result<GamePadState> {
        let mut state = new_native_state();
        game.native.gamepad_state_with_dead_zone(
            game.handle,
            playerIndex as u32,
            deadZoneMode as u32,
            &mut state,
        )?;
        Ok(GamePadState::from_native(state))
    }

    pub fn GetCapabilities(
        game: &GameContext<'_>,
        playerIndex: PlayerIndex,
    ) -> Result<GamePadCapabilities> {
        let mut capabilities = sys::CNA_GamePadCapabilities {
            struct_size: size_of::<sys::CNA_GamePadCapabilities>() as u32,
            struct_version: 1,
            ..sys::CNA_GamePadCapabilities::default()
        };
        game.native
            .gamepad_capabilities(game.handle, playerIndex as u32, &mut capabilities)?;
        Ok(GamePadCapabilities::from_native(capabilities))
    }

    pub fn SetVibration(
        game: &GameContext<'_>,
        playerIndex: PlayerIndex,
        leftMotor: f32,
        rightMotor: f32,
    ) -> Result<bool> {
        let mut applied = sys::CNA_FALSE;
        game.native.set_gamepad_vibration(
            game.handle,
            playerIndex as u32,
            leftMotor,
            rightMotor,
            &mut applied,
        )?;
        Ok(applied != sys::CNA_FALSE)
    }
}

fn new_native_state() -> sys::CNA_GamePadState {
    sys::CNA_GamePadState {
        struct_size: size_of::<sys::CNA_GamePadState>() as u32,
        struct_version: 1,
        ..sys::CNA_GamePadState::default()
    }
}
