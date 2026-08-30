#![allow(clippy::cast_possible_truncation, clippy::missing_errors_doc)]

use core::{any::Any, mem::size_of};

use cna_sys as sys;

use crate::error::Result;
use crate::game::GameContext;

mod gamepad;
mod mouse;
mod touch;

pub use gamepad::{
    ButtonState, Buttons, GamePad, GamePadButtons, GamePadCapabilities, GamePadDPad,
    GamePadDeadZone, GamePadState, GamePadThumbSticks, GamePadTriggers, GamePadType,
};
pub use mouse::{Mouse, MouseState};
pub use touch::{
    GestureSample, GestureType, TouchCollection, TouchCollectionEnumerator, TouchLocation,
    TouchLocationState, TouchPanel, TouchPanelCapabilities,
};

// Generated from enum constants in the pinned Microsoft.Xna.Framework.dll
// SHA-256 38e7093f52d7474bbc6256906519781a1210d7da50a1c667b52716fcf49ca130.
macro_rules! xna_keys {
    ($($name:ident = $value:expr),+ $(,)?) => {
        /// XNA key identities with their exact signed CLR enum values.
        #[repr(i32)]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub enum Keys {
            $($name = $value),+
        }

        #[cfg(test)]
        const XNA_KEYS: &[(Keys, i32)] = &[$((Keys::$name, $value)),+];

        impl Keys {
            fn from_code(code: i32) -> Option<Self> {
                match code {
                    $($value => Some(Self::$name),)+
                    _ => None,
                }
            }
        }
    };
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KeyState {
    Up = 0,
    Down = 1,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PlayerIndex {
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
}

xna_keys! {
    A = 65,
    Add = 107,
    Apps = 93,
    Attn = 246,
    B = 66,
    Back = 8,
    BrowserBack = 166,
    BrowserFavorites = 171,
    BrowserForward = 167,
    BrowserHome = 172,
    BrowserRefresh = 168,
    BrowserSearch = 170,
    BrowserStop = 169,
    C = 67,
    CapsLock = 20,
    Crsel = 247,
    D = 68,
    D0 = 48,
    D1 = 49,
    D2 = 50,
    D3 = 51,
    D4 = 52,
    D5 = 53,
    D6 = 54,
    D7 = 55,
    D8 = 56,
    D9 = 57,
    Decimal = 110,
    Delete = 46,
    Divide = 111,
    Down = 40,
    E = 69,
    End = 35,
    Enter = 13,
    EraseEof = 249,
    Escape = 27,
    Execute = 43,
    Exsel = 248,
    F = 70,
    F1 = 112,
    F10 = 121,
    F11 = 122,
    F12 = 123,
    F13 = 124,
    F14 = 125,
    F15 = 126,
    F16 = 127,
    F17 = 128,
    F18 = 129,
    F19 = 130,
    F2 = 113,
    F20 = 131,
    F21 = 132,
    F22 = 133,
    F23 = 134,
    F24 = 135,
    F3 = 114,
    F4 = 115,
    F5 = 116,
    F6 = 117,
    F7 = 118,
    F8 = 119,
    F9 = 120,
    G = 71,
    H = 72,
    Help = 47,
    Home = 36,
    I = 73,
    ImeConvert = 28,
    ImeNoConvert = 29,
    Insert = 45,
    J = 74,
    K = 75,
    Kana = 21,
    Kanji = 25,
    L = 76,
    LaunchApplication1 = 182,
    LaunchApplication2 = 183,
    LaunchMail = 180,
    LeftControl = 162,
    Left = 37,
    LeftAlt = 164,
    LeftShift = 160,
    LeftWindows = 91,
    M = 77,
    MediaNextTrack = 176,
    MediaPlayPause = 179,
    MediaPreviousTrack = 177,
    MediaStop = 178,
    Multiply = 106,
    N = 78,
    None = 0,
    NumLock = 144,
    NumPad0 = 96,
    NumPad1 = 97,
    NumPad2 = 98,
    NumPad3 = 99,
    NumPad4 = 100,
    NumPad5 = 101,
    NumPad6 = 102,
    NumPad7 = 103,
    NumPad8 = 104,
    NumPad9 = 105,
    O = 79,
    OemAuto = 243,
    OemCopy = 242,
    OemEnlW = 244,
    OemSemicolon = 186,
    OemBackslash = 226,
    OemQuestion = 191,
    OemTilde = 192,
    OemOpenBrackets = 219,
    OemPipe = 220,
    OemCloseBrackets = 221,
    OemQuotes = 222,
    Oem8 = 223,
    OemClear = 254,
    OemComma = 188,
    OemMinus = 189,
    OemPeriod = 190,
    OemPlus = 187,
    P = 80,
    Pa1 = 253,
    PageDown = 34,
    PageUp = 33,
    Pause = 19,
    Play = 250,
    Print = 42,
    PrintScreen = 44,
    ProcessKey = 229,
    Q = 81,
    R = 82,
    RightControl = 163,
    Right = 39,
    RightAlt = 165,
    RightShift = 161,
    RightWindows = 92,
    S = 83,
    Scroll = 145,
    Select = 41,
    SelectMedia = 181,
    Separator = 108,
    Sleep = 95,
    Space = 32,
    Subtract = 109,
    T = 84,
    Tab = 9,
    U = 85,
    Up = 38,
    V = 86,
    VolumeDown = 174,
    VolumeMute = 173,
    VolumeUp = 175,
    W = 87,
    X = 88,
    Y = 89,
    Z = 90,
    Zoom = 251,
    ChatPadGreen = 202,
    ChatPadOrange = 203,
}

/// Copy-oriented snapshot of all 256 native keyboard slots.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct KeyboardState {
    pressed_key_words: [u64; 4],
}

#[allow(non_snake_case)]
impl KeyboardState {
    #[must_use]
    pub fn new(keys: &[Keys]) -> Self {
        let mut result = Self::empty();
        for key in keys {
            let code = *key as usize;
            result.pressed_key_words[code / 64] |= 1_u64 << (code % 64);
        }
        result
    }

    const fn empty() -> Self {
        Self {
            pressed_key_words: [0; 4],
        }
    }

    fn from_native_words(words: [u64; 4]) -> Self {
        let mut result = Self::empty();
        for code in 0..256 {
            if words[code / 64] & (1_u64 << (code % 64)) != 0
                && Keys::from_code(code as i32).is_some()
            {
                result.pressed_key_words[code / 64] |= 1_u64 << (code % 64);
            }
        }
        result
    }

    #[must_use]
    pub fn IsKeyDown(&self, key: Keys) -> bool {
        let key = key as usize;
        self.pressed_key_words[key / 64] & (1_u64 << (key % 64)) != 0
    }

    #[must_use]
    pub fn IsKeyUp(&self, key: Keys) -> bool {
        !self.IsKeyDown(key)
    }

    #[must_use]
    pub fn Item(&self, key: Keys) -> KeyState {
        if self.IsKeyDown(key) {
            KeyState::Down
        } else {
            KeyState::Up
        }
    }

    #[must_use]
    pub fn GetPressedKeys(&self) -> Vec<Keys> {
        (0..256)
            .filter_map(|code| {
                let key = Keys::from_code(code)?;
                self.IsKeyDown(key).then_some(key)
            })
            .collect()
    }

    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        self.pressed_key_words.iter().fold(0_u32, |hash, word| {
            hash ^ (*word as u32) ^ ((*word >> 32) as u32)
        }) as i32
    }

    #[must_use]
    pub fn Equals(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::empty()
    }
}

pub struct Keyboard;

#[allow(non_snake_case)]
impl Keyboard {
    /// The composed Rust Game projection passes its callback context explicitly.
    pub fn GetState(game: &GameContext<'_>) -> Result<KeyboardState> {
        let mut state = sys::CNA_KeyboardState {
            struct_size: size_of::<sys::CNA_KeyboardState>() as u32,
            struct_version: 1,
            pressed_key_words: [0; 4],
        };
        game.native.keyboard_state(game.handle, &mut state)?;
        Ok(KeyboardState::from_native_words(state.pressed_key_words))
    }

    /// XNA's per-player Chatpad overload.
    ///
    /// CNA has one keyboard, so every slot answers with the snapshot
    /// `GetState` produces; the canonical route exists because the canonical
    /// API has the overload, and the projection calls it rather than
    /// forwarding to `GetState` so a future per-slot backend is observed here
    /// without a Rust change.
    pub fn GetStateWithPlayerIndex(
        game: &GameContext<'_>,
        playerIndex: PlayerIndex,
    ) -> Result<KeyboardState> {
        let mut state = sys::CNA_KeyboardState {
            struct_size: size_of::<sys::CNA_KeyboardState>() as u32,
            struct_version: 1,
            pressed_key_words: [0; 4],
        };
        game.native
            .keyboard_state_for_player(game.handle, playerIndex as u32, &mut state)?;
        Ok(KeyboardState::from_native_words(state.pressed_key_words))
    }
}

#[cfg(test)]
mod tests {
    use super::{Keys, XNA_KEYS};
    use std::collections::BTreeSet;

    #[test]
    fn keys_match_the_complete_xna_metadata_table() {
        assert_eq!(XNA_KEYS.len(), 160);
        let values: BTreeSet<i32> = XNA_KEYS.iter().map(|(key, _)| *key as i32).collect();
        assert_eq!(values.len(), XNA_KEYS.len());
        for &(key, expected) in XNA_KEYS {
            assert_eq!(key as i32, expected);
        }
        assert_eq!(Keys::Escape as i32, 27);
        assert_eq!(Keys::ChatPadOrange as i32, 203);
    }
}
