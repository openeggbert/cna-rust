//! CNA's motion sensors against the live library.
//!
//! This host has no accelerometer, compass or gyroscope, and that is the point
//! of most of what follows: the honest report of a missing sensor is a state
//! that says so, not a reading of zero. A device in free fall really does
//! report `(0, 0, 0)` g, so a projection that returned that for "no sensor"
//! would make the two indistinguishable.
//!
//! What can still be measured without hardware is everything except the
//! numbers a real device would produce: enumeration, construction, state,
//! validity, the sampling interval, and -- through CNA's own injection routes
//! -- that a reading really travels the delivery path.

use std::sync::{Arc, Mutex};

use cna::extensions::sensors::{
    count, enumerate, Accelerometer, Compass, CompassReading, Gyroscope, SensorKind, SensorState,
    SensorTimestamp,
};
use cna::Microsoft::Xna::Framework::{Game, GameContext, TimeSpan, Vector3};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

#[derive(Debug, Default)]
struct Observed {
    sensor_count: u32,
    kinds: Vec<SensorKind>,
    accelerometer_supported: bool,
    accelerometer_state: Option<SensorState>,
    accelerometer_valid: bool,
    accelerometer_reading: Option<(f32, f32, f32)>,
    accelerometer_after_inject: Option<(f32, f32, f32)>,
    gyroscope_state: Option<SensorState>,
    compass_state: Option<SensorState>,
    interval_round_trip: Option<(i64, i64)>,
    inject_refusal: Option<String>,
    gyroscope_inject: Option<String>,
    gyroscope_after_inject: Option<(f32, f32, f32)>,
    compass_inject: Option<String>,
}

#[derive(Default)]
struct SensorGame {
    state: Arc<GameState>,
    observed: Arc<Mutex<Observed>>,
}

impl GameStateAccess for SensorGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for SensorGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let mut observed = Observed {
            sensor_count: count(game)?,
            ..Observed::default()
        };
        let sensors = enumerate(game)?;
        assert_eq!(
            sensors.len(),
            observed.sensor_count as usize,
            "the enumeration and the count describe the same sensor set"
        );
        observed.kinds = sensors.iter().map(|sensor| sensor.kind).collect();

        // Support is answered without constructing anything, because the
        // answer here is "no" and constructing one to find out would be the
        // wrong shape.
        observed.accelerometer_supported = Accelerometer::is_supported(game)?;

        // Constructing succeeds even with no hardware: the object exists so a
        // game can ask *why* there are no readings.
        let accelerometer = Accelerometer::new(game)?;
        observed.accelerometer_state = Some(accelerometer.state()?);
        observed.accelerometer_valid = accelerometer.is_data_valid()?;
        observed.accelerometer_reading = accelerometer
            .current_value()?
            .map(|reading| {
                (
                    reading.acceleration.X,
                    reading.acceleration.Y,
                    reading.acceleration.Z,
                )
            });

        // The sampling interval is a real setting with a real read-back.
        let before = accelerometer.time_between_updates()?.Ticks();
        accelerometer.set_time_between_updates(TimeSpan::from_ticks(200_000))?;
        let after = accelerometer.time_between_updates()?.Ticks();
        observed.interval_round_trip = Some((before, after));

        // Starting a sensor the host does not have must not pretend to work
        // silently; whichever way CNA answers, the state afterwards is what a
        // game would act on.
        let _ = accelerometer.start();
        // Injection needs a sensor test backend installed and started. The
        // routes that install one are CNA's own test seams, which this binding
        // deliberately does not bind -- calling them would fake runtime state.
        // So on this host injection is refused, and the refusal is exact.
        observed.inject_refusal = accelerometer
            .inject(0.25, -0.5, 9.75)
            .err()
            .map(|error| error.to_string());
        observed.accelerometer_after_inject = accelerometer
            .current_value()?
            .map(|reading| {
                (
                    reading.acceleration.X,
                    reading.acceleration.Y,
                    reading.acceleration.Z,
                )
            });
        let _ = accelerometer.stop();
        accelerometer.dispose()?;

        let gyroscope = Gyroscope::new(game)?;
        observed.gyroscope_state = Some(gyroscope.state()?);
        observed.gyroscope_inject = gyroscope
            .inject(0.1, 0.2, 0.3)
            .err()
            .map(|error| error.to_string());
        observed.gyroscope_after_inject = gyroscope.current_value()?.map(|reading| {
            (
                reading.rotation_rate.X,
                reading.rotation_rate.Y,
                reading.rotation_rate.Z,
            )
        });

        let compass = Compass::new(game)?;
        observed.compass_state = Some(compass.state()?);
        let compass_refusal = compass.inject(CompassReading {
            timestamp: SensorTimestamp {
                ticks: 637_000_000_000_000_000,
                offset_ticks: 0,
            },
            heading_accuracy: 1.5,
            magnetic_heading: 42.0,
            true_heading: 43.25,
            magnetometer_reading: Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0),
        });
        observed.compass_inject = compass_refusal.err().map(|error| error.to_string());

        *self
            .observed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = observed;
        Ok(())
    }
}

#[test]
fn sensors_report_absence_rather_than_a_zero_reading() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let observed = Arc::new(Mutex::new(Observed::default()));
    run_for_frames(
        SensorGame {
            state: Arc::new(GameState::new()),
            observed: Arc::clone(&observed),
        },
        1,
    )
    .expect("sensor enumeration, construction and state");
    let observed = observed
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    // Every enumerated kind is one this build knows; an unknown identity is a
    // typed error rather than a silently mapped neighbour.
    assert_eq!(observed.kinds.len(), observed.sensor_count as usize);

    // The state is the honest answer for this host, whatever it is, and it is
    // never conflated with "a reading of zero".
    let state = observed
        .accelerometer_state
        .expect("the accelerometer reported a state");
    if observed.accelerometer_supported {
        assert_ne!(
            state,
            SensorState::NotSupported,
            "a supported accelerometer must not report NotSupported"
        );
    } else {
        assert_eq!(
            state,
            SensorState::NotSupported,
            "an unsupported accelerometer says so"
        );
        assert!(
            !observed.accelerometer_valid,
            "an unsupported sensor has no valid data"
        );
        assert_eq!(
            observed.accelerometer_reading, None,
            "no sensor means no reading -- not a reading of (0, 0, 0), which is \
             what a device in free fall reports"
        );
    }

    // The gyroscope and compass answer the same way, so absence is uniform
    // rather than per-family guesswork.
    assert_eq!(
        observed.gyroscope_state.expect("gyroscope state"),
        state,
        "every sensor family reports absence the same way on one host"
    );
    assert_eq!(observed.compass_state.expect("compass state"), state);

    // The sampling interval is a real setting: it reads back what was set.
    let (before, after) = observed
        .interval_round_trip
        .expect("an interval round trip was measured");
    assert_ne!(before, i64::MIN);
    assert_eq!(
        after, 200_000,
        "the requested sampling interval reads back exactly"
    );

    // Injection is refused here, and the refusal names the reason rather than
    // failing vaguely. The routes that would install a sensor test backend are
    // CNA's own test seams, classified TOOLING_ONLY and deliberately unbound:
    // a binding that called them would fake runtime state.
    // The three families do not agree about injection, and the difference is
    // measured rather than smoothed over. The accelerometer and the gyroscope
    // accept an injected reading on this host; the compass refuses, because it
    // needs a sensor test backend installed first. The route that would
    // install one is CNA's own test seam, classified TOOLING_ONLY and
    // deliberately unbound: a binding that called it would fake runtime state.
    assert_eq!(
        observed.inject_refusal, None,
        "the accelerometer accepts an injected reading on this host"
    );
    assert_eq!(
        observed.gyroscope_inject, None,
        "so does the gyroscope"
    );
    let compass = observed
        .compass_inject
        .as_deref()
        .expect("the compass refuses injection without a test backend");
    assert!(
        compass.contains("No test backend is installed and started for this sensor"),
        "the compass refusal states why, got {compass}"
    );

    // An accepted injection does **not** make the reading readable here, and
    // that is the most important measurement in this file. The sensor is still
    // NotSupported, so `is_data_valid` stays false, so `current_value` stays
    // `None`. CNA took the value without claiming the device now exists, and
    // the projection passes that through instead of surfacing an injected
    // number as though hardware had reported it.
    assert_eq!(
        observed.accelerometer_after_inject, None,
        "an injected value does not become a valid reading on a host with no sensor"
    );
    assert_eq!(observed.gyroscope_after_inject, None);
    assert!(
        !observed.accelerometer_supported,
        "and injecting does not conjure an accelerometer either"
    );
}
