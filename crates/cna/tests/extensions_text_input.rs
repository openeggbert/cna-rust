//! CNA's text input, IME composition and candidate lists against the live library.
//!
//! No keyboard or IME is attached to this host, and none is needed: CNA
//! provides the same raise routes its own tests use, so every event travels the
//! real delivery path rather than a Rust shortcut around it. What a physical
//! IME would change is which code units arrive, not how they are carried.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use cna::extensions::text_input::{
    TextEditing, TextEditingCandidates, TextInput, TextInputType, Utf16Assembler, Utf16Unit,
};
use cna::Microsoft::Xna::Framework::{Game, GameContext, Rectangle};
use cna::{run_for_frames, CnaError, GameState, GameStateAccess, Result};

#[derive(Default)]
struct TextInputGame {
    state: Arc<GameState>,
    committed: Arc<Mutex<String>>,
    unpaired: Arc<AtomicUsize>,
    editing: Arc<Mutex<Vec<TextEditing>>>,
    candidates: Arc<Mutex<Vec<TextEditingCandidates>>>,
    panics: Arc<AtomicUsize>,
}

impl GameStateAccess for TextInputGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for TextInputGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        // --- lifecycle ---
        //
        // `start` is accepted on every host, but activation is the platform's
        // answer and not this call's. A HEADLESS host has no text-input
        // service to activate, and CNA says so rather than reporting an
        // activation that nothing backs -- measured with
        // `build-probe/ext014_text.c`, where `start` succeeds and `is_active`
        // stays false. A host with a real window manager would answer true,
        // and nothing below depends on which it is.
        assert!(!TextInput::is_active(game)?, "text input starts inactive");
        TextInput::start(game)?;
        let active_after_start = TextInput::is_active(game)?;
        TextInput::stop(game)?;
        assert!(
            !TextInput::is_active(game)?,
            "stop leaves it inactive on any host"
        );
        TextInput::start_with_type(game, TextInputType::Email)?;
        assert_eq!(
            TextInput::is_active(game)?,
            active_after_start,
            "a typed start activates exactly as much as a plain one"
        );

        // The rectangle is a hint the platform uses to keep a candidate window
        // clear of the text; it must be accepted, not refused.
        TextInput::set_input_rectangle(game, Rectangle::new(10, 20, 300, 40))?;

        // Whether a screen keyboard is shown is a real query with a real
        // answer, even when the answer is "no" on a headless host.
        let _shown = TextInput::is_screen_keyboard_shown(game)?;

        // --- committed text, including astral characters ---
        let committed = Arc::clone(&self.committed);
        let unpaired = Arc::clone(&self.unpaired);
        let mut assembler = Utf16Assembler::new();
        let text_subscription = TextInput::subscribe_text(move |unit| {
            let push = assembler.push(unit);
            if push.orphaned.is_some() {
                unpaired.fetch_add(1, Ordering::SeqCst);
            }
            match push.unit {
                Utf16Unit::Character(value) => committed
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(value),
                Utf16Unit::PendingHighSurrogate => {}
                Utf16Unit::UnpairedSurrogate(_) => {
                    unpaired.fetch_add(1, Ordering::SeqCst);
                }
            }
        })?;

        // A handler that panics must not unwind into C, and must not stop the
        // other handlers from being delivered to.
        let panics = Arc::clone(&self.panics);
        let panicking = TextInput::subscribe_text(move |_| {
            panics.fetch_add(1, Ordering::SeqCst);
            panic!("a text-input handler panic must be contained");
        })?;

        // Not ASCII: an accented character, a CJK character and an emoji, the
        // last of which really does arrive as a surrogate pair.
        for unit in "aé中🎮".encode_utf16() {
            TextInput::raise_text(game, unit)?;
        }
        drop(panicking);

        // --- IME composition ---
        let editing = Arc::clone(&self.editing);
        let editing_subscription = TextInput::subscribe_editing(move |value| {
            editing
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(value.clone());
        })?;
        TextInput::raise_editing(game, "にほんご", 3, 6)?;
        TextInput::raise_editing(game, "", 0, 0)?;

        // --- IME candidates ---
        let candidates = Arc::clone(&self.candidates);
        let candidates_subscription = TextInput::subscribe_candidates(move |value| {
            candidates
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(value.clone());
        })?;
        TextInput::raise_candidates(game, &["日本語", "にほんご", "ニホンゴ"], Some(1), true)?;
        TextInput::raise_candidates(game, &[], None, false)?;

        // Dropping a subscription stops delivery; a later event must not
        // reach a handler whose data has gone.
        drop(text_subscription);
        drop(editing_subscription);
        drop(candidates_subscription);
        TextInput::raise_text(game, u16::from(b'Z'))?;
        TextInput::raise_editing(game, "ignored", 0, 0)?;
        TextInput::raise_candidates(game, &["ignored"], None, false)?;

        // CNA's selected-candidate field is a signed 32-bit index. A `usize`
        // that cannot fit would truncate into a valid-looking index and select
        // an unrelated candidate, so it is refused here rather than narrowed.
        let refused = TextInput::raise_candidates(game, &["only"], Some(usize::MAX), false);
        assert!(
            matches!(refused, Err(CnaError::InvalidInput(message))
                if message.contains("candidate index is out of range")),
            "an unrepresentable candidate index must be refused, got {refused:?}"
        );

        TextInput::stop(game)?;
        Ok(())
    }
}

#[test]
fn text_input_carries_unicode_composition_and_candidates() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let committed = Arc::new(Mutex::new(String::new()));
    let unpaired = Arc::new(AtomicUsize::new(0));
    let editing = Arc::new(Mutex::new(Vec::new()));
    let candidates = Arc::new(Mutex::new(Vec::new()));
    let panics = Arc::new(AtomicUsize::new(0));
    run_for_frames(
        TextInputGame {
            state: Arc::new(GameState::new()),
            committed: Arc::clone(&committed),
            unpaired: Arc::clone(&unpaired),
            editing: Arc::clone(&editing),
            candidates: Arc::clone(&candidates),
            panics: Arc::clone(&panics),
        },
        1,
    )
    .expect("text input lifecycle and delivery");

    // Exact text, not a length: an emoji that lost its pair or a CJK character
    // mangled into replacement characters would still have the right count.
    assert_eq!(
        *committed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        "aé中🎮",
        "every committed code unit arrived and the surrogate pair rejoined"
    );
    assert_eq!(
        unpaired.load(Ordering::SeqCst),
        0,
        "no surrogate was orphaned"
    );
    assert_eq!(
        panics.load(Ordering::SeqCst),
        "aé中🎮".encode_utf16().count(),
        "the panicking handler ran once per code *unit* -- five, not four, \
         because the emoji is a surrogate pair -- and was contained each time"
    );

    let editing = editing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(editing.len(), 2, "both composition updates arrived");
    assert_eq!(editing[0].text, "にほんご", "the draft is copied out exactly");
    assert_eq!((editing[0].start, editing[0].length), (3, 6));
    assert_eq!(editing[1].text, "", "an empty draft stays empty");

    let candidates = candidates
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(candidates.len(), 2, "both candidate lists arrived");
    assert_eq!(
        candidates[0].candidates,
        vec!["日本語", "にほんご", "ニホンゴ"],
        "every candidate is copied out of its borrowed view, in order"
    );
    assert_eq!(candidates[0].selected, Some(1));
    assert!(candidates[0].horizontal);
    assert!(candidates[1].candidates.is_empty());
    assert_eq!(
        candidates[1].selected, None,
        "the container's -1 becomes None rather than an index"
    );
    assert!(!candidates[1].horizontal);
}

#[test]
fn text_input_needs_a_running_game() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    // Subscribing is process-wide and needs no game, but it must also not
    // leave a handler behind when nothing ever delivers to it.
    let seen = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&seen);
    let subscription =
        TextInput::subscribe_text(move |_| {
            counted.fetch_add(1, Ordering::SeqCst);
        })
        .expect("a subscription needs no game");
    drop(subscription);
    assert_eq!(seen.load(Ordering::SeqCst), 0);
}

