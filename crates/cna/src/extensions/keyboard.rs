//! `input_keyboard.h`'s layout-aware routes: scancodes, key names, modifiers.
//!
//! XNA's `Keys` enum names *virtual* keys -- what a keypress means. A scancode
//! names a *physical* key -- where it is on the board. On a QWERTY layout the
//! two coincide and the distinction is invisible; on AZERTY or Dvorak they do
//! not, and a game that binds movement to WASD by virtual key ends up binding
//! three keys scattered across the board.
//!
//! XNA had no answer for that, so none of this has an XNA counterpart to fold
//! into. It is also not derivable in Rust: every route here asks the platform's
//! *current* layout a question, which is why each takes a live game.
//!
//! [`KeyboardState::ToString`] is here for a different reason -- it is a value
//! operation, but the text is CNA's own formatting rather than something the
//! bit set implies.

use cna_sys as sys;

use crate::error::Result;
use crate::input::{KeyboardState, Keys};
use crate::Microsoft::Xna::Framework::GameContext;

/// The modifier keys held right now, as a bit set.
///
/// Wider than XNA's view of the keyboard: `Caps`, `Num`, `Scroll` and `Mode`
/// are *latched* states rather than held keys, and `Gui` is the key XNA's
/// `Keys` calls `LeftWindows`/`RightWindows`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct KeyModifiers(u32);

impl KeyModifiers {
    /// No modifier at all.
    pub const NONE: Self = Self(sys::CNA_KEY_MODIFIER_NONE);
    /// Either shift key.
    pub const SHIFT: Self = Self(sys::CNA_KEY_MODIFIER_SHIFT);
    /// Either control key.
    pub const CTRL: Self = Self(sys::CNA_KEY_MODIFIER_CTRL);
    /// Either alt key.
    pub const ALT: Self = Self(sys::CNA_KEY_MODIFIER_ALT);
    /// Either windows/command key.
    pub const GUI: Self = Self(sys::CNA_KEY_MODIFIER_GUI);
    /// Caps lock is latched on.
    pub const CAPS: Self = Self(sys::CNA_KEY_MODIFIER_CAPS);
    /// Num lock is latched on.
    pub const NUM: Self = Self(sys::CNA_KEY_MODIFIER_NUM);
    /// Scroll lock is latched on.
    pub const SCROLL: Self = Self(sys::CNA_KEY_MODIFIER_SCROLL);
    /// The AltGr / mode-switch key.
    pub const MODE: Self = Self(sys::CNA_KEY_MODIFIER_MODE);
    /// Every bit this ABI defines.
    pub const ALL: Self = Self(sys::CNA_KEY_MODIFIER_ALL);

    /// The raw bits, exactly as CNA reported them.
    ///
    /// Published rather than hidden because `ALL` is the set this ABI version
    /// defines, not a guarantee that no future bit exists outside it.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Whether every bit of `other` is set here.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether any bit is set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl core::ops::BitOr for KeyModifiers {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// CNA's canonical "there is no such key" answer, folded into `None`.
///
/// The header says each of these routes "receives the identity, or the
/// canonical none value when the name is unknown", and that value is
/// [`Keys::None`]. A caller asking "which key is this?" wants `None` for that,
/// not a `Keys::None` they have to remember to compare against.
fn identified(key: Option<Keys>) -> Option<Keys> {
    key.filter(|key| *key != Keys::None)
}

/// The layout-aware keyboard queries, all scoped to a live game.
#[derive(Clone, Copy, Debug)]
pub struct KeyboardLayout;

impl KeyboardLayout {
    /// Which key the physical `scancode` produces under the current layout.
    ///
    /// Answers `None` when the layout maps it to something outside XNA's
    /// `Keys` -- a real answer, not a failure, and one a caller has to be able
    /// to tell apart from a refusal.
    pub fn key_from_scancode(game: &GameContext<'_>, scancode: Keys) -> Result<Option<Keys>> {
        let value = game
            .native
            .keyboard_key_from_scancode(game.handle, scancode as sys::CNA_Key)?;
        Ok(identified(Keys::from_key_code(value)))
    }

    /// The modifier keys held right now.
    pub fn modifiers(game: &GameContext<'_>) -> Result<KeyModifiers> {
        Ok(KeyModifiers(game.native.keyboard_modifiers(game.handle)?))
    }

    /// What the platform calls the physical key at `scancode`.
    ///
    /// This is the *position*'s name: on AZERTY the key where QWERTY has `Q`
    /// is named for what it produces there.
    pub fn scancode_name(game: &GameContext<'_>, scancode: Keys) -> Result<String> {
        game.native
            .keyboard_scancode_name(game.handle, scancode as sys::CNA_Key)
    }

    /// What the platform calls the virtual key `key`.
    pub fn key_name(game: &GameContext<'_>, key: Keys) -> Result<String> {
        game.native.keyboard_key_name(game.handle, key as sys::CNA_Key)
    }

    /// The scancode a name refers to, or `None` when the name is unknown.
    pub fn scancode_from_name(game: &GameContext<'_>, name: &str) -> Result<Option<Keys>> {
        let value = game.native.keyboard_scancode_from_name(game.handle, name)?;
        Ok(identified(Keys::from_key_code(value)))
    }

    /// The key a name refers to, or `None` when the name is unknown.
    pub fn key_from_name(game: &GameContext<'_>, name: &str) -> Result<Option<Keys>> {
        let value = game.native.keyboard_key_from_name(game.handle, name)?;
        Ok(identified(Keys::from_key_code(value)))
    }
}

/// CNA's own text for a keyboard snapshot.
pub trait KeyboardStateText {
    /// XNA's `ToString` for a keyboard snapshot.
    ///
    /// Measured: the string does not describe the snapshot. It is the .NET
    /// default for a struct -- `"Microsoft.Xna.Framework.Input.KeyboardState"`
    /// -- and every snapshot answers the same one.
    ///
    /// That is correct rather than a gap. XNA's `KeyboardState` has no
    /// `ToString` override, so this is what a real XNA game sees, and the route
    /// is bound so the projection reads the string from CNA rather than
    /// hardcoding it.
    fn native_to_string(&self) -> Result<String>;
}

impl KeyboardStateText for KeyboardState {
    fn native_to_string(&self) -> Result<String> {
        let native = self.to_native();
        crate::native::Native::process()?.keyboard_state_string(&native)
    }
}
