#![allow(clippy::cast_possible_truncation, clippy::missing_errors_doc)]

use core::mem::size_of;

use cna_sys as sys;

use crate::error::Result;
use crate::game::GameContext;

/// XNA key identities retain their canonical Windows virtual-key values.
#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Keys {
    Escape = sys::CNA_KEY_ESCAPE,
}

/// Copy-oriented snapshot of all 256 native keyboard slots.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct KeyboardState {
    pressed_key_words: [u64; 4],
}

#[allow(non_snake_case)]
impl KeyboardState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pressed_key_words: [0; 4],
        }
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
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
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
        Ok(KeyboardState {
            pressed_key_words: state.pressed_key_words,
        })
    }
}
