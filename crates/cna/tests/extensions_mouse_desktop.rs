//! `input_mouse.h`'s `_ext` routes and `input.h`'s dead-zone algorithm.
//!
//! Nothing here needs a hand on the mouse. The desktop position is read rather
//! than asserted; capture and warp are asked for and their *answer* is what is
//! checked, because a backend that declines is a documented outcome and not a
//! failure; and the clicked event is driven through CNA's own test hook, which
//! is the whole reason the hook exists.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cna::extensions::mouse::{apply_dead_zone, MouseDesktop, RawAnalogState};
use cna::Microsoft::Xna::Framework::Input::GamePadDeadZone;
use cna::Microsoft::Xna::Framework::{Game, GameContext, Vector2};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

#[derive(Default)]
struct MouseGame {
    state: Arc<GameState>,
    ran: Arc<AtomicBool>,
}

impl GameStateAccess for MouseGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for MouseGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        MouseDesktop::reset_for_tests(game)?;

        // --- the clicked event, driven through CNA's own hook ---------------
        let seen: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));
        let collected = Arc::clone(&seen);
        let subscription = MouseDesktop::on_clicked(move |button| {
            collected
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(button);
        })?;

        for button in [1_i32, 3, 2] {
            MouseDesktop::raise_clicked(game, button)?;
        }
        let observed = seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        assert_eq!(
            observed,
            vec![1, 3, 2],
            "every raised click reaches the handler, in order and with its own button"
        );

        // Withdrawing stops delivery. This is the half that matters for
        // safety: the boxed closure is freed here, so a click raised
        // afterwards must not reach it.
        subscription.unsubscribe()?;
        MouseDesktop::raise_clicked(game, 9)?;
        let after = seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        assert_eq!(
            after, observed,
            "a click raised after unsubscribing reaches nothing"
        );
        drop(subscription);

        // --- relative mode round-trips --------------------------------------
        let before = MouseDesktop::is_relative_mode(game)?;
        MouseDesktop::set_relative_mode(game, !before)?;
        let toggled = MouseDesktop::is_relative_mode(game)?;
        println!("NOTE: relative mode {before} -> {toggled}");
        MouseDesktop::set_relative_mode(game, before)?;
        assert_eq!(
            MouseDesktop::is_relative_mode(game)?,
            before,
            "relative mode is restored to what it was"
        );

        // --- capture and warp report acceptance, not success -----------------
        let captured = MouseDesktop::set_capture(game, true)?;
        println!("NOTE: the backend {} capture", if captured { "accepted" } else { "declined" });
        let released = MouseDesktop::set_capture(game, false)?;
        assert_eq!(
            captured, released,
            "a backend that can capture can release, and one that cannot does neither"
        );

        let (x, y) = MouseDesktop::global_position(game)?;
        println!("NOTE: cursor at desktop ({x}, {y})");
        let warped = MouseDesktop::warp_global(game, x, y)?;
        println!("NOTE: the backend {} the warp", if warped { "accepted" } else { "declined" });
        if warped {
            assert_eq!(
                MouseDesktop::global_position(game)?,
                (x, y),
                "a warp the backend accepted, to where the cursor already was, leaves it there"
            );
        }

        self.ran.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn the_desktop_mouse_routes_answer_and_the_click_hook_drives_the_event() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let ran = Arc::new(AtomicBool::new(false));
    let game = MouseGame {
        state: Arc::new(GameState::default()),
        ran: Arc::clone(&ran),
    };
    run_for_frames(game, 1).expect("one frame with the desktop mouse routes");
    assert!(ran.load(Ordering::SeqCst), "LoadContent ran");
}

#[test]
fn the_dead_zone_algorithm_is_cnas_and_answers_per_mode() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }

    // A stick just off centre: inside XNA's dead zone, outside nothing.
    let raw = RawAnalogState {
        left_thumb_stick: Vector2::from_x_and_y(0.1, 0.1),
        right_thumb_stick: Vector2::from_x_and_y(0.9, 0.0),
        left_trigger: 0.05,
        right_trigger: 0.8,
    };

    let none = apply_dead_zone(GamePadDeadZone::None, raw).expect("no dead zone");
    let independent =
        apply_dead_zone(GamePadDeadZone::IndependentAxes, raw).expect("independent axes");
    let circular = apply_dead_zone(GamePadDeadZone::Circular, raw).expect("circular");

    println!("NOTE: none        {:?}", none.thumb_sticks.Left());
    println!("NOTE: independent {:?}", independent.thumb_sticks.Left());
    println!("NOTE: circular    {:?}", circular.thumb_sticks.Left());

    // `None` passes the value straight through -- that is what the mode means,
    // and it is the one answer that does not depend on a threshold.
    assert_eq!(
        none.thumb_sticks.Left(),
        Vector2::from_x_and_y(0.1, 0.1),
        "the None mode applies no dead zone at all"
    );

    // The two thresholding modes both suppress a small deflection, and the
    // large one survives every mode. Which exact value each produces is CNA's
    // algorithm's business, which is precisely why this calls it instead of
    // restating it.
    for (name, processed) in [("independent", independent), ("circular", circular)] {
        let left = processed.thumb_sticks.Left();
        assert_eq!(
            left,
            Vector2::Zero,
            "{name} suppresses a deflection inside the dead zone"
        );
        let right = processed.thumb_sticks.Right();
        assert!(
            right.X > 0.5,
            "{name} leaves a large deflection largely intact: {right:?}"
        );
    }

    // The right trigger is well past any threshold and survives; the left is
    // not and does not.
    assert!(independent.triggers.Right() > 0.5);
    assert_eq!(independent.triggers.Left(), 0.0);
}
