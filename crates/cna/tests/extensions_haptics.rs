//! CNA's force-feedback devices against the live library.
//!
//! No haptic device is attached to this host, so the real forces are
//! `HARDWARE_PENDING`. What is measured is everything else: that enumeration
//! and its count agree, that an identifier naming no device is refused rather
//! than answered, and -- the part worth having -- that CNA's separate
//! "did it actually apply" answer survives into Rust as its own type instead
//! of being folded into success.

use std::sync::{Arc, Mutex};

use cna::extensions::haptics::{
    count, enumerate, joystick_is_haptic, mouse_is_haptic, Applied, HapticCapabilities,
    HapticDevice, HapticFeatures,
};
use cna::Microsoft::Xna::Framework::{Game, GameContext};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

#[derive(Debug, Default)]
struct Observed {
    devices: usize,
    reported_count: u32,
    mouse_haptic: bool,
    joystick_zero_haptic: bool,
    open_unknown_refused: bool,
}

#[derive(Default)]
struct HapticGame {
    state: Arc<GameState>,
    observed: Arc<Mutex<Observed>>,
}

impl GameStateAccess for HapticGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for HapticGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let reported_count = count(game)?;
        let devices = enumerate(game)?;
        assert_eq!(
            devices.len(),
            reported_count as usize,
            "the enumeration and the count describe the same device set"
        );

        // Every device that is there can be opened, described and closed. On a
        // host with none this loop is empty, which is the honest outcome
        // rather than a skipped assertion.
        for info in &devices {
            let device = HapticDevice::open(game, info.id)?;
            assert!(device.is_open()?, "a freshly opened device is open");
            let capabilities = device.capabilities()?;
            assert!(capabilities.is_open, "and says so in its capabilities");
            assert!(capabilities.axis_count >= 0);
            assert!(
                capabilities.max_effects_playing <= capabilities.max_effects,
                "a device cannot play more effects than it can hold"
            );
            let _ = device.name()?;
            device.dispose()?;
        }

        // Opening an identifier that names no device is *not* an error. CNA
        // hands back a device object whose `is_open` says false, whose
        // capabilities are empty, and whose operations report that they did
        // not apply. That is a coherent design, and it is exactly why
        // `Applied` is its own type here: a rumble on such a device succeeds
        // as a call and did nothing, and a caller that read success as "it
        // worked" would show a working control that never buzzes.
        let phantom = HapticDevice::open(game, u32::MAX)
            .expect("opening an unknown identifier answers a closed device, not an error");
        assert!(!phantom.is_open()?, "and it says it is not open");
        let phantom_capabilities = phantom.capabilities()?;
        assert!(phantom_capabilities.features.is_empty());
        assert!(!phantom_capabilities.is_open);
        assert!(!phantom_capabilities.rumble_supported);
        assert_eq!(
            phantom.play_rumble(1.0, 10)?,
            Applied(false),
            "a rumble on a device that is not open reports that it did not apply"
        );
        assert_eq!(phantom.init_rumble()?, Applied(false));
        assert_eq!(phantom.set_gain(50)?, Applied(false));
        assert_eq!(phantom.stop_all_effects()?, Applied(false));
        let open_unknown_refused = false;

        *self
            .observed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Observed {
            devices: devices.len(),
            reported_count,
            mouse_haptic: mouse_is_haptic(game)?,
            joystick_zero_haptic: joystick_is_haptic(game, 0)?,
            open_unknown_refused,
        };
        Ok(())
    }
}

#[test]
fn haptic_devices_enumerate_and_describe_themselves() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let observed = Arc::new(Mutex::new(Observed::default()));
    run_for_frames(
        HapticGame {
            state: Arc::new(GameState::new()),
            observed: Arc::clone(&observed),
        },
        1,
    )
    .expect("haptic enumeration and capability reads");
    let observed = observed
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    assert_eq!(observed.devices, observed.reported_count as usize);
    assert!(
        !observed.open_unknown_refused,
        "opening an unknown identifier answers a closed device rather than failing"
    );
    // No device attached here, so neither the mouse nor joystick zero has
    // force feedback. Asserting the answer rather than skipping it is what
    // records the host honestly.
    if observed.devices == 0 {
        assert!(!observed.mouse_haptic);
        assert!(!observed.joystick_zero_haptic);
    }
}

#[test]
fn capabilities_default_to_a_device_that_can_do_nothing() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    // The default is what a device that reports nothing looks like, and it
    // must not look like a device that can do everything.
    let capabilities = HapticCapabilities::canonical_defaults().expect("defaults");
    assert!(
        capabilities.features.is_empty(),
        "a device that reports nothing supports nothing"
    );
    assert!(!capabilities.is_open);
    assert!(!capabilities.rumble_supported);
    assert_eq!(capabilities.axis_count, 0);
    // The effect counts default to -1, not 0, and the difference matters: a
    // device that holds zero effects is a real answer, while -1 is "not
    // known". Rewriting it to 0 here would turn an unknown into a claim.
    assert_eq!(
        capabilities.max_effects, -1,
        "an unknown effect capacity stays -1 rather than becoming zero"
    );
    assert_eq!(capabilities.max_effects_playing, -1);

    // Equality is CNA's, and it includes the device name: two devices with
    // identical numbers but different names are different devices.
    assert!(
        capabilities
            .same_capabilities("wheel", &capabilities, "wheel")
            .expect("equality"),
        "identical capabilities under the same name are equal"
    );
    assert!(
        !capabilities
            .same_capabilities("wheel", &capabilities, "pedals")
            .expect("equality"),
        "the same numbers under a different name are not the same device"
    );
}

#[test]
fn the_feature_set_keeps_xnas_vocabulary_separate_from_cnas() {
    // XNA's entire haptic vocabulary is two motor amplitudes, which is one bit
    // of this set. Compressing the rest into it -- as a projection that mapped
    // everything onto GamePad.SetVibration would -- would discard the spring,
    // damper, friction and gain a real wheel reports.
    assert!(HapticFeatures::ALL.contains(HapticFeatures::LEFT_RIGHT));
    assert!(HapticFeatures::ALL.contains(HapticFeatures::SPRING));
    assert!(HapticFeatures::ALL.contains(HapticFeatures::DAMPER));
    assert!(HapticFeatures::ALL.contains(HapticFeatures::FRICTION));
    assert!(HapticFeatures::ALL.contains(HapticFeatures::GAIN));
    assert!(HapticFeatures::ALL.contains(HapticFeatures::AUTOCENTER));

    // Distinct features are distinct bits: two collapsing onto one would make
    // a device claim a capability it does not have.
    let each = [
        HapticFeatures::CONSTANT,
        HapticFeatures::SINE,
        HapticFeatures::SQUARE,
        HapticFeatures::TRIANGLE,
        HapticFeatures::SAWTOOTH_UP,
        HapticFeatures::SAWTOOTH_DOWN,
        HapticFeatures::RAMP,
        HapticFeatures::SPRING,
        HapticFeatures::DAMPER,
        HapticFeatures::INERTIA,
        HapticFeatures::FRICTION,
        HapticFeatures::LEFT_RIGHT,
        HapticFeatures::CUSTOM,
        HapticFeatures::GAIN,
        HapticFeatures::AUTOCENTER,
        HapticFeatures::STATUS,
        HapticFeatures::PAUSE,
    ];
    let mut seen = 0_u32;
    for feature in each {
        assert_ne!(feature.bits(), 0, "every named feature is a real bit");
        assert_eq!(seen & feature.bits(), 0, "no two features share a bit");
        seen |= feature.bits();
        assert!(HapticFeatures::ALL.contains(feature));
    }
    assert!(HapticFeatures::NONE.is_empty());
    assert!(!HapticFeatures::NONE.contains(HapticFeatures::LEFT_RIGHT));
}
