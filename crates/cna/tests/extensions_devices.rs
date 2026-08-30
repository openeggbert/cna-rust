//! CNA's device layer against the live library.
//!
//! The layer is a build option. This test asks CNA whether it is present and
//! then holds the answers to the matching standard, rather than assuming
//! either state.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cna::extensions::devices::{
    battery_percent, battery_seconds_remaining, clipboard_text, display_content_scale,
    display_safe_area, is_available, logical_cpu_core_count, power_state, preferred_locales,
    set_clipboard_text, system_ram_megabytes, PowerState,
};
use cna::Microsoft::Xna::Framework::{Game, GameContext};
use cna::{run_for_frames, CnaError, ErrorCategory, GameState, GameStateAccess, Result};

#[derive(Default)]
struct DeviceGame {
    state: Arc<GameState>,
    observed: Arc<AtomicBool>,
}

impl GameStateAccess for DeviceGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

/// A refusal is only acceptable when CNA says the layer is compiled out.
fn accept(available: bool, result: Result<()>) {
    match result {
        Ok(()) => {}
        Err(CnaError::Native {
            category: ErrorCategory::NotSupported,
            ..
        }) => assert!(
            !available,
            "the device layer reports as available but refused a route",
        ),
        Err(error) => panic!("unexpected device-layer failure: {error}"),
    }
}

impl Game for DeviceGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let available = is_available()?;

        accept(available, power_state(game).map(|state| {
            assert!(
                !matches!(state, PowerState::Unrecognized(_)),
                "unnamed power state: {state:?}",
            );
        }));

        // A percentage and a remaining time are Option: a host that does not
        // know reports nothing rather than a zero that would read as empty.
        accept(available, battery_percent(game).map(|percent| {
            if let Some(percent) = percent {
                assert!((0..=100).contains(&percent), "percent out of range: {percent}");
            }
        }));
        accept(available, battery_seconds_remaining(game).map(|seconds| {
            if let Some(seconds) = seconds {
                assert!(seconds >= 0);
            }
        }));

        accept(available, logical_cpu_core_count(game).map(|count| {
            assert!(count >= 1, "a host always has at least one core: {count}");
        }));
        accept(available, system_ram_megabytes(game).map(|megabytes| {
            assert!(megabytes >= 0);
        }));

        accept(available, preferred_locales(game).map(|locales| {
            for locale in locales {
                assert!(
                    !locale.language.is_empty(),
                    "a locale always names a language",
                );
            }
        }));

        // A headless session has no window, and CNA's canonical answer for
        // that is zero. The projection reports None so it cannot be mistaken
        // for a scale of zero.
        accept(available, display_content_scale(game).map(|scale| {
            if let Some(scale) = scale {
                assert!(scale > 0.0, "a real content scale is positive: {scale}");
            }
        }));
        accept(available, display_safe_area(game).map(|area| {
            assert!(area.Width >= 0 && area.Height >= 0);
        }));

        // Setting the clipboard succeeds when the request was made, which is
        // not the same as the clipboard changing. The test asserts the former
        // and deliberately does not assert the latter.
        accept(available, set_clipboard_text(game, "cna-rust clipboard probe"));
        accept(available, clipboard_text(game).map(|_| ()));

        self.observed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn the_device_layer_answers_or_says_it_is_absent() {
    let game = DeviceGame::default();
    let observed = Arc::clone(&game.observed);
    run_for_frames(game, 1).expect("one frame reaches LoadContent");
    assert!(observed.load(Ordering::SeqCst), "LoadContent ran");
}
