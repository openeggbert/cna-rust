//! What CNA's substitute sensor backends make measurable.
//!
//! `extensions_sensors.rs` measures the honest report of a machine with no
//! sensors. This file measures the other half: with CNA's own backend
//! installed, a sensor reports supported, a reading survives the delivery path,
//! and a subscription's callback actually runs. Without these routes the
//! projection could only ever be exercised against `NotSupported`, and a
//! projection nobody has driven past its first refusal is a projection nobody
//! has checked.
//!
//! The two families take different shapes because upstream gives them
//! different shapes: the compass and the motion sensor take a whole substitute
//! backend, the accelerometer and gyroscope are steered field by field.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use cna::extensions::sensors::{
    Accelerometer, AttitudeReading, Compass, CompassReading, Gyroscope, Motion, MotionReading,
    SensorState, SensorTimestamp,
};
use cna::Microsoft::Xna::Framework::{Game, GameContext, Matrix, Quaternion, Vector3};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

#[derive(Debug, Default)]
struct Observed {
    state_before: Option<SensorState>,
    supported_before: bool,
    supported_after: bool,
    valid_after_inject: bool,
    state_after: Option<SensorState>,
    held_before_start: bool,
    held_after_start: bool,
    held_after_stop: bool,
    injected_reading: Option<(f32, f32, f32)>,
    callback_readings: Vec<(f32, f32, f32)>,
    readings_after_unsubscribe: usize,
    dispatch_exceptions_before: i32,
    dispatch_exceptions_after_panic: i32,
    motion_north_referenced: Option<bool>,
    motion_reading: Option<MotionReading>,
    compass_reading: Option<CompassReading>,
    notes: Vec<String>,
}

#[derive(Default)]
struct BackendGame {
    state: Arc<GameState>,
    observed: Arc<Mutex<Observed>>,
}

impl GameStateAccess for BackendGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

/// Standard gravity, which is the accelerometer's conversion factor.
const G: f32 = 9.806_65;

/// An injection whose numbers no real device would produce, so a value that
/// arrives can only have come from the injection under test.
///
/// In **metres per second squared**, because that is what the injector takes.
/// The reading comes back in `g`, which is the asymmetry this file pins down.
const X: f32 = 0.125;
const Y: f32 = -2.5;
const Z: f32 = 7.75;

impl Game for BackendGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let mut observed = Observed::default();

        // --- the accelerometer, steered field by field ---------------------
        let accelerometer = Accelerometer::new(game)?;
        // `is_supported` is the *platform* question -- `cna_accelerometer_get_is_supported`
        // takes the game, not a sensor -- so a per-instance hook cannot move
        // it. The instance's own state is what changes, and that is what a
        // caller reads before deciding there is nothing to read.
        observed.supported_before = Accelerometer::is_supported(game)?;
        observed.state_before = Some(accelerometer.state()?);
        accelerometer.set_supported_for_tests(true)?;
        observed.supported_after = Accelerometer::is_supported(game)?;
        observed.state_after = Some(accelerometer.state()?);

        // `start()` still refuses -- "no matching sensor could be opened".
        // `set_supported_for_tests` flips what the sensor *reports*, not what
        // the platform can open, so the accelerometer and gyroscope are driven
        // by forcing the started flag rather than by starting for real. That is
        // the difference between these hooks and the compass's whole
        // substitute backend, and it is measured here rather than assumed.
        observed.notes.push(match accelerometer.start() {
            Ok(()) => "accelerometer start() succeeded behind the test hooks".to_owned(),
            Err(error) => format!("accelerometer start() refused: {error}"),
        });
        observed.held_before_start = accelerometer.holds_subsystem_for_tests()?;
        accelerometer.set_started_for_tests(true)?;
        observed.held_after_start = accelerometer.holds_subsystem_for_tests()?;

        // `isSupported_` is what gates `CurrentValue` upstream, so the hook's
        // observable effect is here rather than in `is_supported` or `state`.
        // Whether a *valid* reading follows is a second question, asked next.
        accelerometer.inject(X, Y, Z)?;
        observed.valid_after_inject = accelerometer.is_data_valid()?;
        observed.injected_reading = accelerometer
            .current_value()?
            .map(|reading| {
                (
                    reading.acceleration.X,
                    reading.acceleration.Y,
                    reading.acceleration.Z,
                )
            });

        // --- a subscription, driven by the dispatch hook -------------------
        let seen: Arc<Mutex<Vec<(f32, f32, f32)>>> = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);
        let mut subscription = accelerometer.on_current_value_changed(move |reading| {
            sink.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push((
                    reading.acceleration.X,
                    reading.acceleration.Y,
                    reading.acceleration.Z,
                ));
        })?;
        accelerometer.register_started_instance_for_tests()?;
        Accelerometer::dispatch_to_for_tests(game, &[&accelerometer], X, Y, Z)?;
        observed.callback_readings = seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();

        // Dropping the subscription must stop the callbacks *and* free the
        // closure in that order. A dispatch after it is the check that CNA is
        // no longer holding a pointer into freed memory.
        subscription.unsubscribe()?;
        Accelerometer::dispatch_to_for_tests(game, &[&accelerometer], 1.0, 1.0, 1.0)?;
        observed.readings_after_unsubscribe = seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();

        // --- a panicking subscriber must not reach C -----------------------
        observed.dispatch_exceptions_before =
            Accelerometer::dispatch_exception_count_for_tests(game)?;
        let calls = Arc::new(AtomicU32::new(0));
        let counter = Arc::clone(&calls);
        // The panic below is deliberate; the default hook would print a
        // backtrace and make a passing run look like a failing one.
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let panicking = accelerometer.on_current_value_changed(move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
            panic!("a Rust sensor subscriber panicked on purpose");
        })?;
        Accelerometer::dispatch_to_for_tests(game, &[&accelerometer], X, Y, Z)?;
        drop(panicking);
        std::panic::set_hook(previous);
        observed.dispatch_exceptions_after_panic =
            Accelerometer::dispatch_exception_count_for_tests(game)?;
        observed.notes.push(format!(
            "the panicking subscriber ran {} time(s)",
            calls.load(Ordering::SeqCst)
        ));

        accelerometer.set_started_for_tests(false)?;
        observed.held_after_stop = accelerometer.holds_subsystem_for_tests()?;
        accelerometer.unregister_started_instance_for_tests()?;

        // --- the compass, with a whole substitute backend ------------------
        let compass = Compass::new(game)?;
        compass.set_test_backend(true, true)?;
        compass.start()?;
        let reading = CompassReading {
            timestamp: SensorTimestamp {
                ticks: 1,
                offset_ticks: 0,
            },
            heading_accuracy: 1.5,
            magnetic_heading: 42.5,
            true_heading: 43.25,
            magnetometer_reading: Vector3 {
                X: 1.0,
                Y: 2.0,
                Z: 3.0,
            },
        };
        compass.inject(reading)?;
        observed.compass_reading = compass.current_value()?;
        compass.stop()?;
        compass.set_test_backend(false, false)?;

        // --- the motion sensor, which only a backend can make answer -------
        let motion = Motion::new(game)?;
        motion.set_test_backend(true, true, true)?;
        observed.motion_north_referenced = Some(motion.is_attitude_north_referenced()?);
        motion.start()?;
        let fused = MotionReading {
            timestamp: SensorTimestamp {
                ticks: 2,
                offset_ticks: 0,
            },
            attitude: AttitudeReading {
                timestamp: SensorTimestamp {
                    ticks: 2,
                    offset_ticks: 0,
                },
                pitch: 0.25,
                roll: -0.5,
                yaw: 1.75,
                quaternion: Quaternion {
                    X: 0.0,
                    Y: 0.0,
                    Z: 0.0,
                    W: 1.0,
                },
                rotation_matrix: Matrix::Identity,
            },
            device_acceleration: Vector3 {
                X: X,
                Y: Y,
                Z: Z,
            },
            device_rotation_rate: Vector3 {
                X: 0.5,
                Y: 0.5,
                Z: 0.5,
            },
            gravity: Vector3 {
                X: 0.0,
                Y: -1.0,
                Z: 0.0,
            },
        };
        motion.inject(fused)?;
        observed.motion_reading = motion.current_value()?;
        motion.stop()?;
        motion.set_test_backend(false, false, false)?;

        // The gyroscope shares the accelerometer's shape; one check that the
        // macro really was instantiated for it too.
        let gyroscope = Gyroscope::new(game)?;
        gyroscope.set_supported_for_tests(true)?;
        observed
            .notes
            .push(format!("gyroscope supported: {}", Gyroscope::is_supported(game)?));

        *self
            .observed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = observed;
        Ok(())
    }
}

#[test]
fn a_substitute_backend_makes_the_whole_sensor_path_measurable() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let observed = Arc::new(Mutex::new(Observed::default()));
    let game = BackendGame {
        state: Arc::new(GameState::default()),
        observed: Arc::clone(&observed),
    };
    run_for_frames(game, 1).expect("one frame with the sensor backends installed");
    let observed = observed
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for note in &observed.notes {
        println!("NOTE: {note}");
    }

    // The point of the whole file: without the hook the answer is "no sensor",
    // with it the answer is a sensor.
    assert!(
        !observed.supported_before,
        "this host is expected to have no real accelerometer"
    );
    assert_eq!(
        observed.state_before,
        Some(SensorState::NotSupported),
        "with no device and no hook, the honest state is NotSupported"
    );
    // Measured, and not what the hook's name suggests: `state` is written only
    // by Start/Stop/Dispose upstream, so a per-instance support hook does not
    // move it. Stated here so the next reader does not go looking for a bug.
    println!(
        "NOTE: accelerometer state after set_supported_for_tests = {:?}",
        observed.state_after
    );
    // Measured, and worth stating: the hook is per instance, so the *platform*
    // question keeps its honest answer. A game asking "does this machine have
    // an accelerometer" is not told yes by a test hook.
    assert!(
        !observed.supported_after,
        "a per-instance test hook must not change what the platform reports"
    );

    // The platform hold is an ownership contract, and this is its observable
    // half. What the *forced* started flag does to it is upstream's business;
    // what matters here is that a sensor which has not started does not hold
    // the subsystem, and that clearing the flag leaves it not holding one.
    assert!(
        !observed.held_before_start,
        "a sensor that has not started must not hold the subsystem"
    );
    println!(
        "NOTE: subsystem held after forcing started = {}",
        observed.held_after_start
    );
    assert!(
        !observed.held_after_stop,
        "clearing the started flag should give the platform subsystem hold back"
    );

    // A reading that arrives with these numbers can only have come from the
    // injection: no device produces them. Whether one arrives at all depends on
    // `is_data_valid`, which the injection is what sets.
    println!(
        "NOTE: is_data_valid after injecting behind the hook = {}",
        observed.valid_after_inject
    );
    if observed.valid_after_inject {
        let (x, y, z) = observed
            .injected_reading
            .expect("a valid reading should be readable");
        // The injector takes metres per second squared and the reading is in
        // g. This is the assertion that caught the crate documenting the
        // injection as taking g -- it does not, and 9.80665 in reads 1.0 out.
        for (name, got, expected) in [("X", x, X / G), ("Y", y, Y / G), ("Z", z, Z / G)] {
            assert!(
                (got - expected).abs() < 1e-6,
                "{name}: injecting {} m/s^2 should read back {expected} g, got {got}",
                match name {
                    "X" => X,
                    "Y" => Y,
                    _ => Z,
                }
            );
        }
    } else {
        assert_eq!(
            observed.injected_reading, None,
            "a sensor with no valid data must answer None rather than a stale or zero reading"
        );
    }

    assert_eq!(
        observed.callback_readings.len(),
        1,
        "a subscription's callback should have run exactly once for one dispatch"
    );
    let (x, y, z) = observed.callback_readings[0];
    // Same conversion as the injector: the dispatch hook takes the platform
    // unit and the reading arrives in the canonical one.
    assert!(
        (x - X / G).abs() < 1e-6 && (y - Y / G).abs() < 1e-6 && (z - Z / G).abs() < 1e-6,
        "the callback should receive the dispatched reading in g, got {:?}",
        observed.callback_readings[0]
    );
    assert_eq!(
        observed.readings_after_unsubscribe,
        observed.callback_readings.len(),
        "a dispatch after unsubscribing must not reach the closure -- and must not \
         read the freed box either, which is what this dispatch is really checking"
    );

    // A Rust panic must be contained at the trampoline. CNA counts the
    // exceptions a subscriber threw, so if this number ever moves, a panic has
    // unwound into C.
    assert_eq!(
        observed.dispatch_exceptions_after_panic, observed.dispatch_exceptions_before,
        "a panicking Rust subscriber must not raise an exception CNA sees"
    );

    // The motion sensor exists only behind its backend, and north-referencing
    // is the one property the backend is asked to decide.
    assert_eq!(
        observed.motion_north_referenced,
        Some(true),
        "the backend was installed claiming a north-referenced attitude"
    );
    let motion = observed
        .motion_reading
        .expect("the motion sensor should report the injected fusion");
    assert_eq!(motion.device_acceleration.X, X);
    assert_eq!(motion.gravity.Y, -1.0);
    assert_eq!(motion.attitude.yaw, 1.75);

    let compass = observed
        .compass_reading
        .expect("the compass should report the injected reading behind its backend");
    assert_eq!(compass.magnetic_heading, 42.5);
    assert_eq!(compass.true_heading, 43.25);
    assert_eq!(compass.magnetometer_reading.Z, 3.0);
}

#[test]
fn a_readings_to_string_matches_xna_s_own_format() {
    // Pure formatting: no library, no host, no sensor. The expected strings are
    // read off cnanext's own ToString() implementations rather than invented,
    // which is what makes this a fidelity check and not a change detector.
    let reading = cna::extensions::sensors::AccelerometerReading {
        timestamp: SensorTimestamp {
            ticks: 0,
            offset_ticks: 0,
        },
        acceleration: Vector3 {
            X: 1.0,
            Y: 2.0,
            Z: 3.0,
        },
    };
    assert_eq!(reading.ToString(), "Acceleration:{X:1 Y:2 Z:3}");

    let attitude = AttitudeReading {
        timestamp: SensorTimestamp {
            ticks: 0,
            offset_ticks: 0,
        },
        pitch: 0.5,
        roll: 1.5,
        yaw: 2.5,
        quaternion: Quaternion {
            X: 0.0,
            Y: 0.0,
            Z: 0.0,
            W: 1.0,
        },
        rotation_matrix: Matrix::Identity,
    };
    assert_eq!(attitude.ToString(), "Pitch:0.5 Roll:1.5 Yaw:2.5");

    let motion = MotionReading {
        timestamp: SensorTimestamp {
            ticks: 0,
            offset_ticks: 0,
        },
        attitude,
        device_acceleration: Vector3 {
            X: 1.0,
            Y: 0.0,
            Z: 0.0,
        },
        device_rotation_rate: Vector3 {
            X: 0.0,
            Y: 0.0,
            Z: 0.0,
        },
        gravity: Vector3 {
            X: 0.0,
            Y: -1.0,
            Z: 0.0,
        },
    };
    assert_eq!(
        motion.ToString(),
        "DeviceAcceleration:{X:1 Y:0 Z:0} Gravity:{X:0 Y:-1 Z:0}"
    );

    let event = cna::extensions::sensors::AccelerometerReadingEvent {
        timestamp: SensorTimestamp {
            ticks: 0,
            offset_ticks: 0,
        },
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    assert_eq!(event.ToString(), "{X:1 Y:2 Z:3}");
}
