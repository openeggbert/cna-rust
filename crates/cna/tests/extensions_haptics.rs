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
    HapticDevice, HapticDirection, HapticDirectionType, HapticEffect, HapticEffectType,
    HapticFeatures,
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
    effect_steps: Vec<(&'static str, bool)>,
    effect_supported: bool,
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

        // The whole effect lifecycle on that same closed device. No haptic
        // hardware is attached, so real forces are HARDWARE_PENDING; what can
        // be measured is that every step answers honestly rather than
        // pretending to play.
        let effect = HapticEffect::new(HapticEffectType::LeftRight)?;
        let effect_supported = phantom.is_effect_supported(&effect)?;
        let mut effect_steps: Vec<(&'static str, bool)> = Vec::new();
        if let Ok(id) = phantom.create_effect(&effect) {
            effect_steps.push(("run", phantom.run_effect(id, 1)?.0));
            effect_steps.push(("stop", phantom.stop_effect(id)?.0));
            effect_steps.push(("update", phantom.update_effect(id, &effect)?.0));
            effect_steps.push(("status", phantom.effect_is_playing(id)?));
            let _ = phantom.destroy_effect(id);
        } else {
            effect_steps.push(("create refused", false));
        }

        *self
            .observed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Observed {
            devices: devices.len(),
            reported_count,
            mouse_haptic: mouse_is_haptic(game)?,
            joystick_zero_haptic: joystick_is_haptic(game, 0)?,
            open_unknown_refused,
            effect_steps,
            effect_supported,
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

    // Real forces are HARDWARE_PENDING. What is asserted is that a device
    // which is not open supports no effect and that no step of the effect
    // lifecycle claims to have applied -- a device pretending to play would
    // be worse than one that says it cannot.
    assert!(
        !observed.effect_supported,
        "a device that is not open supports no effect"
    );
    assert!(!observed.effect_steps.is_empty(), "the lifecycle ran");
    for (step, reported) in &observed.effect_steps {
        assert!(
            !reported,
            "{step} on a device that is not open must not report that it applied"
        );
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

#[test]
fn every_effect_family_and_direction_space_maps_both_ways() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    // Thirteen effect families, of which LeftRight is the one XNA could
    // express. Walking all of them proves no two collapse onto one identity,
    // which would let a caller build a spring and get a sawtooth.
    let families = [
        HapticEffectType::Constant,
        HapticEffectType::Sine,
        HapticEffectType::Square,
        HapticEffectType::Triangle,
        HapticEffectType::SawtoothUp,
        HapticEffectType::SawtoothDown,
        HapticEffectType::Ramp,
        HapticEffectType::Spring,
        HapticEffectType::Damper,
        HapticEffectType::Inertia,
        HapticEffectType::Friction,
        HapticEffectType::LeftRight,
        HapticEffectType::Custom,
    ];
    let mut seen = Vec::new();
    for family in families {
        let effect = HapticEffect::new(family).expect("effect defaults");
        let kind = effect.kind().expect("kind round-trips");
        assert_eq!(kind, family, "an effect keeps the family it was built for");
        assert!(!seen.contains(&kind), "no two families share an identity");
        seen.push(kind);
    }
    assert_eq!(seen.len(), 13);

    let direction = HapticDirection::canonical_defaults().expect("direction defaults");
    assert!(
        direction
            .same_direction(&direction)
            .expect("a direction equals itself"),
    );
    let other = HapticDirection {
        kind: HapticDirectionType::Cartesian,
        values: [1, 2, 3],
    };
    if direction.kind != HapticDirectionType::Cartesian || direction.values != [1, 2, 3] {
        assert!(
            !direction
                .same_direction(&other)
                .expect("two different directions differ"),
        );
    }
}

#[test]
fn an_effect_carries_its_own_fields_and_its_custom_samples() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let mut effect = HapticEffect::new(HapticEffectType::LeftRight).expect("effect");
    effect
        .set_length(250)
        .set_magnitude(-1234)
        .set_rumble_magnitudes(0xC000, 0x3000);
    assert_eq!(effect.length(), 250);
    assert_eq!(effect.magnitude(), -1234);
    assert_eq!(effect.rumble_magnitudes(), (0xC000, 0x3000));

    // The two motor amplitudes are one effect family's two fields here, not
    // the whole haptic vocabulary as they were in XNA.
    assert_eq!(effect.kind().expect("kind"), HapticEffectType::LeftRight);

    // Custom samples are copied in, so nothing borrows the caller's buffer.
    let mut custom = HapticEffect::new(HapticEffectType::Custom).expect("custom effect");
    {
        let samples: Vec<u16> = (0..8).map(|value| value * 1000).collect();
        custom.set_custom_samples(&samples);
        drop(samples);
    }
    assert_eq!(
        custom.custom_samples(),
        &[0, 1000, 2000, 3000, 4000, 5000, 6000, 7000],
        "the samples survived the buffer they came from"
    );

    // CNA's equality compares the sample data too, so two effects that differ
    // only in their samples are different effects.
    let mut same = HapticEffect::new(HapticEffectType::Custom).expect("custom effect");
    same.set_custom_samples(&[0, 1000, 2000, 3000, 4000, 5000, 6000, 7000]);
    assert!(custom.same_effect(&same).expect("equal effects"));
    let mut different = HapticEffect::new(HapticEffectType::Custom).expect("custom effect");
    different.set_custom_samples(&[9, 9, 9]);
    assert!(
        !custom.same_effect(&different).expect("different effects"),
        "effects differing only in their samples are not equal"
    );
}

