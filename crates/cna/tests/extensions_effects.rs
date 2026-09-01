//! The rest of `effects.h`: shader effects, the colour matrix, and what an
//! effect can say about itself.
//!
//! The measurement that matters most here is the one upstream warns about:
//! **creating a `ShaderEffect` succeeds whether or not the source compiles**,
//! and two of CNA's renderers disagree about the same nonsense text. So this
//! file asserts the contract that holds everywhere -- both sources empty is
//! refused, a created effect exists -- and *reports* the renderer-specific
//! answer rather than asserting one, because asserting it would be asserting
//! which renderer the artifact was built with.

use cna::extensions::effects::{create_sprite_effect, ColorMatrixEffect, EffectFactsExt};
use cna::extensions::pbr::TextureSlot;
use cna::extensions::shader_effect::ShaderEffect;
use cna::Microsoft::Xna::Framework::Graphics::{
    BasicEffect, GraphicsDevice, GraphicsProfile, PresentationParameters,
};
use cna::Microsoft::Xna::Framework::{GraphicsDeviceInformation, Vector4};
use cna::EffectBase;
use cna::{CnaError, ErrorCategory, Result};

const VERTEX: &str = "#version 300 es\nvoid main() { gl_Position = vec4(0.0); }\n";
const FRAGMENT: &str =
    "#version 300 es\nprecision mediump float;\nout vec4 c;\nvoid main() { c = vec4(1.0); }\n";

fn device() -> Option<GraphicsDevice> {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return None;
    }
    let parameters = PresentationParameters::new();
    parameters.SetBackBufferWidth(64);
    parameters.SetBackBufferHeight(64);
    match GraphicsDevice::new(
        &GraphicsDeviceInformation::new().Adapter(),
        GraphicsProfile::HiDef,
        &parameters,
    ) {
        Ok(device) => Some(device),
        Err(CnaError::Native {
            category: ErrorCategory::Platform,
            ref message,
            ..
        }) if message.contains("platform window id") => {
            println!("this renderer cannot create a device without a window: {message}");
            None
        }
        Err(error) => panic!(
            "independent GraphicsDevice construction failed with something other than \
             the renderer's no-window refusal, which is the only failure this skips. \
             Seen once under a parallel full-suite run and not reproduced since, so the \
             exact text matters: {error:?}"
        ),
    }
}

#[test]
fn creating_a_shader_effect_is_not_compiling_it() {
    let Some(device) = device() else { return };

    // The one refusal every renderer agrees on.
    assert!(
        ShaderEffect::new(&device, "", "").is_err(),
        "both sources empty is refused identically everywhere"
    );

    let effect = ShaderEffect::new(&device, VERTEX, FRAGMENT).expect("a plausible shader");
    println!(
        "NOTE: plausible source -> has_renderer={:?} is_valid={:?}",
        effect.has_renderer(),
        effect.is_valid()
    );

    // Nonsense text. Upstream is explicit that this is *accepted* by both
    // renderers it measured, and that they then disagree about validity -- so
    // what is asserted is that construction does not fail, and what is printed
    // is the renderer's own verdict.
    let nonsense = ShaderEffect::new(&device, "this is not a shader", "nor is this")
        .expect("construction does not compile, so nonsense source still makes an effect");
    let valid = nonsense.is_valid().expect("a validity answer");
    let error = nonsense.compile_error().expect("a compile-error answer");
    println!("NOTE: nonsense source -> is_valid={valid}, compile_error={error:?}");
    if !valid {
        assert!(
            !error.is_empty(),
            "a renderer that calls the effect invalid should say why"
        );
    }
}

#[test]
fn a_shader_effect_accepts_uniforms_of_every_shape() {
    let Some(device) = device() else { return };
    let effect = ShaderEffect::new(&device, VERTEX, FRAGMENT).expect("an effect");

    // A uniform name the shader does not declare is the renderer's business.
    // What is under test is that the marshalling reaches CNA at all and that
    // each shape is distinguishable -- an argument-order mistake would show up
    // as one of these failing where its neighbours pass.
    let outcomes: Vec<(&str, Result<()>)> = vec![
        ("float", effect.set_float("u_f", 1.5)),
        ("int32", effect.set_int32("u_i", -3)),
        (
            "vector2",
            effect.set_vector2("u_v2", cna::Microsoft::Xna::Framework::Vector2 { X: 1.0, Y: 2.0 }),
        ),
        (
            "vector3",
            effect.set_vector3(
                "u_v3",
                cna::Microsoft::Xna::Framework::Vector3 {
                    X: 1.0,
                    Y: 2.0,
                    Z: 3.0,
                },
            ),
        ),
        (
            "vector4",
            effect.set_vector4(
                "u_v4",
                Vector4 {
                    X: 1.0,
                    Y: 2.0,
                    Z: 3.0,
                    W: 4.0,
                },
            ),
        ),
        (
            "matrix",
            effect.set_matrix("u_m", cna::Microsoft::Xna::Framework::Matrix::Identity),
        ),
        ("float array", effect.set_float_array("u_fa", &[1.0, 2.0, 3.0])),
        (
            "mat4 array",
            effect.set_matrix_array("u_ma", &[0.0; 32], 2),
        ),
        ("vec3 array", effect.set_vector3_array("u_v3a", &[0.0; 9], 3)),
        (
            "uniform block",
            effect.declare_uniform_block(0, &[("a", 0), ("b", 16)]),
        ),
    ];
    for (shape, outcome) in &outcomes {
        println!("NOTE: set {shape} -> {outcome:?}");
    }
    // Whether a renderer accepts an unknown uniform name is its own decision,
    // so the assertion is about *consistency*: a build that accepts one shape
    // and refuses another of the same standing has a marshalling problem.
    let accepted = outcomes.iter().filter(|(_, r)| r.is_ok()).count();
    assert!(
        accepted == 0 || accepted == outcomes.len(),
        "the uniform setters should stand or fall together, got {accepted} of {}",
        outcomes.len()
    );
}

#[test]
fn a_shader_effect_carries_the_three_transforms() {
    let Some(device) = device() else { return };
    let effect = ShaderEffect::new(&device, VERTEX, FRAGMENT).expect("an effect");

    let mut world = cna::Microsoft::Xna::Framework::Matrix::Identity;
    world.M41 = 5.0;
    world.M42 = -2.0;
    if effect.set_world(world).is_err() {
        println!("NOTE: this renderer does not take the world transform; skipping");
        return;
    }
    assert_eq!(
        effect.world().expect("the world transform"),
        world,
        "a transform written back must read back, or the marshalling transposed it"
    );

    let mut view = cna::Microsoft::Xna::Framework::Matrix::Identity;
    view.M43 = 7.0;
    effect.set_view(view).expect("the view transform");
    assert_eq!(effect.view().expect("the view transform"), view);

    let mut projection = cna::Microsoft::Xna::Framework::Matrix::Identity;
    projection.M11 = 0.5;
    effect.set_projection(projection).expect("the projection");
    assert_eq!(effect.projection().expect("the projection"), projection);
}

#[test]
fn a_colour_matrix_effect_round_trips_its_transform() {
    let Some(device) = device() else { return };
    let Ok(effect) = ColorMatrixEffect::new(&device) else {
        println!("NOTE: this build has no ColorMatrixEffect; skipping");
        return;
    };

    let mut values = [0.0_f32; 16];
    for (index, value) in values.iter_mut().enumerate() {
        *value = index as f32 / 16.0;
    }
    effect.set_matrix(values).expect("set the transform");
    assert_eq!(
        effect.matrix().expect("read the transform"),
        values,
        "sixteen floats must survive the crossing in order"
    );

    let offset = Vector4 {
        X: 0.1,
        Y: 0.2,
        Z: 0.3,
        W: 0.4,
    };
    effect.set_offset(offset).expect("set the offset");
    assert_eq!(effect.offset().expect("read the offset"), offset);

    // Grayscale is CNA's own coefficients, so the assertion is that it changed
    // something rather than what it changed -- pinning the numbers here would
    // be pinning a choice that is upstream's to make.
    effect.set_grayscale().expect("grayscale");
    assert_ne!(
        effect.matrix().expect("read the transform"),
        values,
        "grayscale should replace the transform"
    );

    effect.reset().expect("reset");
    let identity = effect.matrix().expect("read the transform");
    assert_ne!(
        identity,
        [0.0; 16],
        "reset should restore a real transform, not zero it"
    );
    assert_eq!(
        effect.offset().expect("read the offset"),
        Vector4 {
            X: 0.0,
            Y: 0.0,
            Z: 0.0,
            W: 0.0
        },
        "reset should clear the offset too"
    );
}

#[test]
fn an_effect_can_say_what_it_is() {
    let Some(device) = device() else { return };

    let sprite = create_sprite_effect(&device).expect("the stock sprite effect");
    assert!(
        sprite
            .is_exact_stock_sprite_effect()
            .expect("the stock-sprite question"),
        "an effect made by the stock sprite factory is the stock sprite effect"
    );

    let basic = BasicEffect::from_device(&device).expect("a BasicEffect");
    assert!(
        !basic
            .AsEffect()
            .is_exact_stock_sprite_effect()
            .expect("the stock-sprite question"),
        "a BasicEffect is not the stock sprite effect"
    );

    // Two effects on the same device must agree about which device that is --
    // when the question can be asked at all. Upstream resolves it through the
    // effect's *parent game* rather than through the device it was made with,
    // so an effect on an independently constructed device is refused with an
    // invalid-game-handle failure. That is measured rather than documented, so
    // the refusal is reported and only the agreement is asserted.
    match (
        sprite.graphics_device_identity(),
        basic.AsEffect().graphics_device_identity(),
    ) {
        (Ok(first), Ok(second)) => assert_eq!(
            first, second,
            "effects on one device should report one device"
        ),
        (Err(first), Err(_)) => println!(
            "NOTE: the device identity needs a running Game; both effects refused: {first}"
        ),
        (first, second) => panic!(
            "two effects on one device disagreed about whether the question can be asked: \
             {first:?} vs {second:?}"
        ),
    }

    println!(
        "NOTE: sprite effect has_renderer={:?} is_compiled={:?}",
        sprite.has_renderer(),
        sprite.is_compiled()
    );
    // A stock effect is not built from caller source, so empty sources are the
    // expected answer rather than a failure.
    println!(
        "NOTE: sprite effect vertex source length={:?}",
        sprite.vertex_source().map(|text| text.len())
    );
}

#[test]
fn the_pbr_slot_order_matches_the_abi() {
    // No library needed: this is about the mapping, and it is the mapping that
    // was wrong. `CNA_PbrMaterialEXT::texture_coordinate_sets` is documented as
    // "Indexed in slot order: base color, normal, metallic-roughness, emissive,
    // occlusion, specular, specular color" -- emissive *before* occlusion,
    // which is not the order the Rust enum's variants are declared in. Reading
    // it the other way round silently addressed the wrong slot.
    assert_eq!(
        TextureSlot::ALL,
        [
            TextureSlot::BaseColor,
            TextureSlot::Normal,
            TextureSlot::MetallicRoughness,
            TextureSlot::Emissive,
            TextureSlot::Occlusion,
            TextureSlot::Specular,
            TextureSlot::SpecularColor,
        ],
        "the slot order must be the ABI's, not the enum's declaration order"
    );
}
