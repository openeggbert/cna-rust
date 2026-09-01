//! What a modern gamepad reports, beyond XNA's four values.
//!
//! No pad is guaranteed to be attached here, and that is the first thing this
//! file measures rather than assumes: every accessor is asked for a slot that
//! may be empty, and what must hold is that an absent capability answers
//! *absent* rather than a plausible zero. A pad lying still also reads
//! `(0, 0, 0)` on its gyroscope, so a projection that could not tell the two
//! apart would let a game calibrate against a device that is not there.

use std::sync::{Arc, Mutex};

use cna::extensions::gamepad_ext::{exclude_axis_dead_zone, pad, ButtonLabel, ConnectionState};
use cna::Microsoft::Xna::Framework::Input::GamePad;
use cna::Microsoft::Xna::Framework::{Color, Game, GameContext, PlayerIndex};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

#[derive(Debug, Default)]
struct Observed {
    connected: bool,
    accelerometer: Option<Option<(f32, f32, f32)>>,
    gyroscope: Option<Option<(f32, f32, f32)>>,
    touchpads: Option<i32>,
    power_percent: Option<Option<i32>>,
    connection: Option<ConnectionState>,
    label: Option<ButtonLabel>,
    name: Option<String>,
    guid: Option<String>,
    steam_handle: Option<Option<u64>>,
    light_bar: Option<bool>,
    trigger_vibration: Option<bool>,
    notes: Vec<String>,
}

#[derive(Default)]
struct PadGame {
    state: Arc<GameState>,
    observed: Arc<Mutex<Observed>>,
}

impl GameStateAccess for PadGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for PadGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let slot = PlayerIndex::One;
        let mut observed = Observed::default();
        observed.connected = GamePad::GetState(game, slot)
            .map(|state| state.IsConnected())
            .unwrap_or(false);

        observed.accelerometer = pad::accelerometer(game, slot)
            .ok()
            .map(|value| value.map(|v| (v.X, v.Y, v.Z)));
        observed.gyroscope = pad::gyroscope(game, slot)
            .ok()
            .map(|value| value.map(|v| (v.X, v.Y, v.Z)));
        observed.touchpads = pad::touchpad_count(game, slot).ok();
        observed.power_percent = pad::power_info(game, slot).ok().map(|info| info.percent);
        observed.connection = pad::connection_state(game, slot).ok();
        // XNA's A button, whatever this pad happens to call it.
        observed.label = pad::button_label(game, slot, 1).ok();
        observed.name = pad::name(game, slot).ok();
        observed.guid = pad::guid(game, slot).ok();
        observed.steam_handle = pad::steam_handle(game, slot).ok();
        observed.light_bar = pad::set_light_bar(game, slot, Color::Red).map(|()| true).ok();
        observed.trigger_vibration = pad::set_trigger_vibration(game, slot, 0.5, 0.5).ok();

        if let Ok(count) = pad::touchpad_count(game, slot) {
            for touchpad in 0..count {
                let fingers = pad::touchpad_finger_count(game, slot, touchpad).unwrap_or(0);
                observed
                    .notes
                    .push(format!("touchpad {touchpad} reports {fingers} finger slot(s)"));
                for finger in 0..fingers {
                    if let Ok(Some(state)) = pad::touchpad_finger(game, slot, touchpad, finger) {
                        observed.notes.push(format!("  finger {finger}: {state:?}"));
                    }
                }
            }
        }

        *self
            .observed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = observed;
        Ok(())
    }
}

#[test]
fn an_absent_gamepad_capability_answers_absent() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let observed = Arc::new(Mutex::new(Observed::default()));
    let game = PadGame {
        state: Arc::new(GameState::default()),
        observed: Arc::clone(&observed),
    };
    run_for_frames(game, 1).expect("one frame reading player one");
    let observed = observed
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for note in &observed.notes {
        println!("NOTE: {note}");
    }
    println!(
        "NOTE: connected={} accelerometer={:?} gyroscope={:?} touchpads={:?}",
        observed.connected, observed.accelerometer, observed.gyroscope, observed.touchpads
    );
    println!(
        "NOTE: power={:?} connection={:?} label={:?} name={:?} guid={:?} steam={:?}",
        observed.power_percent,
        observed.connection,
        observed.label,
        observed.name,
        observed.guid,
        observed.steam_handle
    );
    println!(
        "NOTE: light_bar accepted={:?} trigger_vibration applied={:?}",
        observed.light_bar, observed.trigger_vibration
    );

    // The measurement that matters: a pad with no motion sensor answers None,
    // not a zero vector. If a pad *is* attached and does have one, the value is
    // whatever it is -- what must never happen is `Some((0.0, 0.0, 0.0))` from
    // a slot with no pad at all.
    if !observed.connected {
        assert_eq!(
            observed.accelerometer,
            Some(None),
            "an empty slot must report no accelerometer rather than a zero reading"
        );
        assert_eq!(
            observed.gyroscope,
            Some(None),
            "an empty slot must report no gyroscope rather than a zero reading"
        );
        assert_eq!(
            observed.touchpads,
            Some(0),
            "an empty slot has no touchpads"
        );
    }

    // Every text field must at least answer; an empty string is the honest
    // answer for a slot with no pad, and a failure would be a binding fault.
    assert!(
        observed.name.is_some(),
        "the pad name should answer even for an empty slot"
    );
    assert!(observed.guid.is_some());
}

#[test]
fn the_dead_zone_curve_is_cnas_own() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    // XNA's own left-stick threshold. What is asserted is the shape of the
    // curve rather than its exact values, because the curve is upstream's to
    // choose -- but a value inside the dead zone must come out at zero, one
    // outside must keep its sign, and the mapping must be monotonic, or a game
    // steering by it will feel wrong in a way no result code reports.
    const DEAD_ZONE: f32 = 0.24;

    let inside = exclude_axis_dead_zone(0.1, DEAD_ZONE).expect("inside the dead zone");
    assert_eq!(inside, 0.0, "a value inside the dead zone must come out zero");

    let outside = exclude_axis_dead_zone(0.8, DEAD_ZONE).expect("outside the dead zone");
    assert!(
        outside > 0.0 && outside <= 0.8,
        "a value outside must stay positive and must not be amplified, got {outside}"
    );

    let negative = exclude_axis_dead_zone(-0.8, DEAD_ZONE).expect("the negative side");
    assert!(
        (negative + outside).abs() < 1e-6,
        "the curve must be symmetric about zero: {outside} vs {negative}"
    );

    let mut previous = 0.0_f32;
    for step in 0..=10 {
        let raw = step as f32 / 10.0;
        let value = exclude_axis_dead_zone(raw, DEAD_ZONE).expect("a step");
        assert!(
            value >= previous,
            "the curve must be monotonic: {raw} gave {value} after {previous}"
        );
        previous = value;
    }
    println!("NOTE: 1.0 maps to {previous}");
}
