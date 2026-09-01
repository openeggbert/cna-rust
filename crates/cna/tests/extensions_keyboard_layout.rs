//! `input_keyboard.h`'s layout-aware routes against the live library.
//!
//! No key is held while a test runs, so the *values* here are mostly the
//! honest empty answer. What is worth qualifying is the shape of the mapping
//! itself, and that does not need a keypress:
//!
//! * a scancode round-trips through its own name;
//! * a virtual key round-trips through its own name;
//! * on this host's layout, scancode and virtual key agree for the letters --
//!   reported rather than asserted, because a machine configured for AZERTY
//!   would legitimately disagree and the test would be asserting the tester's
//!   keyboard;
//! * an unknown name answers "no key" rather than failing, which is the
//!   documented behaviour and the one a caller has to be able to act on.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cna::extensions::keyboard::{KeyModifiers, KeyboardLayout, KeyboardStateText};
use cna::Microsoft::Xna::Framework::Input::{Keyboard, KeyboardState, Keys};
use cna::Microsoft::Xna::Framework::{Game, GameContext};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

#[derive(Default)]
struct KeyboardGame {
    state: Arc<GameState>,
    ran: Arc<AtomicBool>,
}

impl GameStateAccess for KeyboardGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

/// The letters, which every Latin layout has somewhere.
const LETTERS: [Keys; 6] = [Keys::A, Keys::W, Keys::S, Keys::D, Keys::Q, Keys::Z];

impl Game for KeyboardGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        // --- names round-trip ------------------------------------------------
        let mut agreements = 0_usize;
        for key in LETTERS {
            let key_name = KeyboardLayout::key_name(game, key)?;
            assert!(!key_name.is_empty(), "{key:?} has a name");
            assert_eq!(
                KeyboardLayout::key_from_name(game, &key_name)?,
                Some(key),
                "a key's own name maps back to it: {key:?} -> {key_name:?}"
            );

            let scancode_name = KeyboardLayout::scancode_name(game, key)?;
            assert!(!scancode_name.is_empty(), "scancode {key:?} has a name");
            assert_eq!(
                KeyboardLayout::scancode_from_name(game, &scancode_name)?,
                Some(key),
                "a scancode's own name maps back to it: {key:?} -> {scancode_name:?}"
            );

            // Reported, not asserted: this is a fact about the host's layout.
            if KeyboardLayout::key_from_scancode(game, key)? == Some(key) {
                agreements += 1;
            }
        }
        println!(
            "NOTE: {agreements} of {} letters have scancode == virtual key on this layout",
            LETTERS.len()
        );

        // --- an unknown name is an answer, not a failure ---------------------
        let unknown = KeyboardLayout::key_from_name(game, "no key is called this")?;
        assert_eq!(
            unknown, None,
            "an unknown name answers the canonical none value rather than failing"
        );
        let unknown_scancode =
            KeyboardLayout::scancode_from_name(game, "no key is called this")?;
        assert_eq!(unknown_scancode, None);

        // --- modifiers -------------------------------------------------------
        let modifiers = KeyboardLayout::modifiers(game)?;
        println!("NOTE: modifiers held = {:#06x}", modifiers.bits());
        assert!(
            KeyModifiers::ALL.contains(modifiers),
            "CNA reported a modifier bit outside the set this ABI version defines: \
             {:#06x}. `bits` is published so this is visible rather than silently \
             dropped, and a new bit means the ALL constant needs updating.",
            modifiers.bits()
        );
        assert!(KeyModifiers::NONE.is_empty());
        assert!(
            (KeyModifiers::SHIFT | KeyModifiers::CTRL).contains(KeyModifiers::SHIFT),
            "the bit set composes"
        );
        assert!(
            !KeyModifiers::SHIFT.contains(KeyModifiers::CTRL),
            "and does not over-report"
        );

        // --- the snapshot's own text -----------------------------------------
        let live = Keyboard::GetState(game)?;
        println!("NOTE: live snapshot text = {:?}", live.native_to_string()?);

        // Measured, and not what a "state to text" route suggests: the string
        // does not describe the snapshot at all. It is the .NET default
        // `ToString()` for a struct -- the qualified type name -- and it is the
        // same for every snapshot.
        //
        // That is XNA-faithful. `KeyboardState` in the decompiled reference has
        // no `ToString` override, so `Object.ToString()` is what a real XNA
        // game sees, and this matches it exactly. The route is bound so the
        // Rust projection has XNA's `ToString` and reads it from CNA rather
        // than hardcoding the string.
        //
        // Worth noting beside `GraphicsResource`, which went the other way:
        // there CNA answers the *bare* type name where XNA answers the
        // qualified one. Here it answers the qualified one. So neither the
        // short nor the long form is a rule of this ABI, and each route's
        // answer has to be read rather than inferred from its neighbours.
        const XNA_DEFAULT: &str = "Microsoft.Xna.Framework.Input.KeyboardState";
        let built = KeyboardState::new(&[Keys::A, Keys::Escape]).native_to_string()?;
        let empty = KeyboardState::new(&[]).native_to_string()?;
        println!("NOTE: built snapshot text = {built:?}, empty = {empty:?}");
        assert_eq!(built, XNA_DEFAULT);
        assert_eq!(
            empty, XNA_DEFAULT,
            "the text does not vary with the snapshot, which is exactly what XNA's \
             own unoverridden ToString does"
        );

        self.ran.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn the_layout_answers_for_scancodes_names_and_modifiers() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let ran = Arc::new(AtomicBool::new(false));
    let game = KeyboardGame {
        state: Arc::new(GameState::default()),
        ran: Arc::clone(&ran),
    };
    run_for_frames(game, 1).expect("one frame with the keyboard layout");
    assert!(ran.load(Ordering::SeqCst), "LoadContent ran");
}
