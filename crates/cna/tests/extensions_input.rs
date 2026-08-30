//! CNA's raw joystick input against the live library.
//!
//! No joystick is attached to this host, so what is measured is the honest
//! answer for that: an empty enumeration, and a refusal for an identifier that
//! names no device. A host with a device would run the rest of the same path.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cna::extensions::input::{capabilities, capture, count, enumerate, JoystickType};
use cna::Microsoft::Xna::Framework::{Game, GameContext};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

#[derive(Default)]
struct JoystickGame {
    state: Arc<GameState>,
    observed: Arc<AtomicBool>,
}

impl GameStateAccess for JoystickGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for JoystickGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let total = count(game)?;
        let devices = enumerate(game)?;
        assert_eq!(
            devices.len(),
            total as usize,
            "enumeration must return exactly the reported count",
        );
        println!("joysticks connected = {total}");

        for device in &devices {
            assert!(!device.name.is_empty(), "a device always has a name");
            assert!(
                !matches!(device.kind, JoystickType::Unrecognized(_)),
                "unnamed joystick type: {:?}",
                device.kind,
            );

            // The identifier the other routes take is the device's id, not its
            // position in this list.
            let capability = capabilities(game, device.id)?;
            assert_eq!(capability.kind, device.kind);
            assert!(capability.axis_count >= 0 && capability.button_count >= 0);
            if let Some(percent) = capability.power_percent {
                assert!((0..=100).contains(&percent));
            }

            let snapshot = capture(game, device.id)?;
            assert_eq!(snapshot.axes()?.len(), capability.axis_count as usize);
            assert_eq!(snapshot.buttons()?.len(), capability.button_count as usize);
            assert_eq!(snapshot.hats()?.len(), capability.hat_count as usize);
            assert_eq!(snapshot.balls()?.len(), capability.ball_count as usize);

            // Two snapshots of an idle device agree, and CNA is what decides.
            let again = capture(game, device.id)?;
            assert_eq!(snapshot.equals(&again), Ok(true));
        }

        // An identifier no device carries is not an error upstream: the capture
        // succeeds with every array empty. The projection preserves that rather
        // than turning it into a failure, and `capabilities` is what
        // distinguishes an absent device from an idle one.
        let unused = devices.iter().map(|device| device.id).max().unwrap_or(0) + 1_000;
        let empty = capture(game, unused).expect("an absent device captures as empty");
        assert!(empty.axes()?.is_empty());
        assert!(empty.buttons()?.is_empty());
        assert!(empty.hats()?.is_empty());
        assert!(empty.balls()?.is_empty());
        assert!(!capabilities(game, unused)?.is_connected);

        self.observed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn joystick_enumeration_and_capture() {
    let game = JoystickGame::default();
    let observed = Arc::clone(&game.observed);
    run_for_frames(game, 1).expect("one frame reaches LoadContent");
    assert!(observed.load(Ordering::SeqCst), "LoadContent ran");
}
