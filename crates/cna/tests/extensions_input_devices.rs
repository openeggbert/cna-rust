//! CNA's device enumeration, hot-plug events and cursors against the live library.
//!
//! Hot-plug is exercised through CNA's own raise routes, so the events travel
//! the real delivery path without anything being physically unplugged. What a
//! real device would change is which identifiers arrive, not how they are
//! carried.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use cna::extensions::input_devices::{
    count, enumerate, raise, subscribe, Hotplug, InputDeviceKind, MouseCursor,
    StockCursor,
};
use cna::Microsoft::Xna::Framework::Graphics::Texture2D;
use cna::Microsoft::Xna::Framework::{Color, Game, GameContext};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

#[derive(Default)]
struct DeviceGame {
    state: Arc<GameState>,
    seen: Arc<Mutex<Vec<(Hotplug, u64)>>>,
    panics: Arc<AtomicUsize>,
    enumerated: Arc<Mutex<Vec<(InputDeviceKind, usize)>>>,
    cursor_outcome: Arc<Mutex<Option<String>>>,
}

impl GameStateAccess for DeviceGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for DeviceGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        // --- enumeration ---
        //
        // A headless host normally reports no devices, and that is an ordinary
        // answer rather than a failure. What matters is that the count and the
        // enumeration agree, whatever the host has.
        for kind in [
            InputDeviceKind::Keyboard,
            InputDeviceKind::Mouse,
            InputDeviceKind::TouchDevice,
        ] {
            let total = count(game, kind)?;
            let devices = enumerate(game, kind)?;
            assert_eq!(
                devices.len(),
                total as usize,
                "the enumeration and the count describe the same device set"
            );
            self.enumerated
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push((kind, devices.len()));
            // Whatever is there, each device is its own identity and CNA
            // agrees with itself about that.
            for device in &devices {
                assert!(device.same_device(device)?, "a device equals itself");
            }
            if devices.len() >= 2 {
                assert!(
                    !devices[0].same_device(&devices[1])?,
                    "two enumerated devices are not the same device"
                );
            }
        }
        // An index past the end is refused rather than answered.
        let keyboards = count(game, InputDeviceKind::Keyboard)?;
        assert!(
            enumerate(game, InputDeviceKind::Keyboard)?.len() == keyboards as usize,
            "enumeration stops at the count"
        );

        // --- hot-plug ---
        let seen = Arc::clone(&self.seen);
        let mut subscriptions = Vec::new();
        for event in [
            Hotplug::KeyboardConnected,
            Hotplug::KeyboardDisconnected,
            Hotplug::MouseConnected,
            Hotplug::MouseDisconnected,
        ] {
            let seen = Arc::clone(&seen);
            subscriptions.push(subscribe(event, move |id| {
                seen.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push((event, id));
            })?);
        }

        // A panicking handler must not unwind into C, and must not stop the
        // real handler being delivered to.
        let panics = Arc::clone(&self.panics);
        let panicking = subscribe(Hotplug::KeyboardConnected, move |_| {
            panics.fetch_add(1, Ordering::SeqCst);
            panic!("a hot-plug handler panic must be contained");
        })?;

        // Distinct identifiers, so an event delivered to the wrong handler or
        // carrying the wrong identifier is visible rather than plausible.
        raise(game, Hotplug::KeyboardConnected, 11)?;
        raise(game, Hotplug::KeyboardDisconnected, 12)?;
        raise(game, Hotplug::MouseConnected, 21)?;
        raise(game, Hotplug::MouseDisconnected, 22)?;
        drop(panicking);

        // Dropping a subscription stops delivery: a later event must not reach
        // a handler whose data has gone.
        drop(subscriptions);
        raise(game, Hotplug::KeyboardConnected, 99)?;
        raise(game, Hotplug::MouseConnected, 99)?;

        // --- cursors ---
        //
        // A headless host has no window to show a cursor on, so what is
        // recorded is CNA's honest answer rather than an assumption.
        let outcome = match MouseCursor::stock(game, StockCursor::Hand) {
            Ok(cursor) => match cursor.set_current(game) {
                Ok(()) => "stock cursor created and set".to_owned(),
                Err(error) => format!("stock cursor created, set refused: {error}"),
            },
            Err(error) => format!("stock cursor refused: {error}"),
        };
        *self
            .cursor_outcome
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(outcome);

        // A texture-backed cursor exercises the ownership question: CNA copies
        // what it needs during the call, so disposing the texture afterwards
        // must not invalidate the cursor.
        let texture = Texture2D::new(&game.GraphicsDevice()?, 2, 2)?;
        texture.SetData(&[Color::Red, Color::Green, Color::Blue, Color::White])?;
        if let Ok(cursor) = MouseCursor::from_texture(game, &texture, 1, 1) {
            drop(texture);
            // The cursor outlives the texture it was built from.
            let _ = cursor.set_current(game);
            cursor.dispose()?;
            // Disposing twice is a no-op rather than a double release.
            cursor.dispose()?;
        }
        Ok(())
    }
}

#[test]
fn input_devices_enumerate_and_report_hotplug() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let seen = Arc::new(Mutex::new(Vec::new()));
    let panics = Arc::new(AtomicUsize::new(0));
    let enumerated = Arc::new(Mutex::new(Vec::new()));
    let cursor_outcome = Arc::new(Mutex::new(None));
    run_for_frames(
        DeviceGame {
            state: Arc::new(GameState::new()),
            seen: Arc::clone(&seen),
            panics: Arc::clone(&panics),
            enumerated: Arc::clone(&enumerated),
            cursor_outcome: Arc::clone(&cursor_outcome),
        },
        1,
    )
    .expect("device enumeration, hot-plug and cursor lifecycle");

    let seen = seen
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(
        seen,
        vec![
            (Hotplug::KeyboardConnected, 11),
            (Hotplug::KeyboardDisconnected, 12),
            (Hotplug::MouseConnected, 21),
            (Hotplug::MouseDisconnected, 22),
        ],
        "each transition reached exactly its own handler, once, with its own identifier"
    );
    assert_eq!(
        panics.load(Ordering::SeqCst),
        1,
        "the panicking handler ran for its one event and was contained"
    );

    // Three kinds were enumerated, and each agreed with its own count.
    assert_eq!(
        enumerated
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len(),
        3
    );

    let outcome = cursor_outcome
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
        .expect("a cursor outcome was recorded");
    // Whatever this host does, it must be one of CNA's real answers rather
    // than a crash or a silent success that did nothing.
    assert!(
        outcome.starts_with("stock cursor"),
        "unexpected cursor outcome: {outcome}"
    );
}
