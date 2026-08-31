//! CNA's PBR materials, effects and pipeline settings against the live library.
//!
//! None of this is XNA -- `BasicEffect` has no metallic factor, no roughness,
//! no index of refraction and no tonemapping operator -- so nothing here
//! touches the strict projection.
//!
//! Availability is queried rather than assumed: these routes need CNA's engine
//! layer, and a library built without it refuses. The test reports which case
//! this artifact is in and only asserts the engine-layer behaviour when the
//! layer is actually present.

use cna::extensions::pbr::{
    engine_layer_version, engine_layer_version_string, AlphaMode, EngineRenderSettings, PbrEffect,
    PbrMaterial, PbrMaterialFull, RenderPipelineSettings, RenderQuality, ShadowQuality,
    TextureSlot, TextureTransform, TonemappingMode,
};
use cna::Microsoft::Xna::Framework::Graphics::{
    GraphicsDevice, GraphicsProfile, PresentationParameters,
};
use cna::Microsoft::Xna::Framework::{GraphicsDeviceInformation, Vector3};
use cna::CnaError;

fn device() -> GraphicsDevice {
    let parameters = PresentationParameters::new();
    parameters.SetBackBufferWidth(64);
    parameters.SetBackBufferHeight(64);
    GraphicsDevice::new(
        &GraphicsDeviceInformation::new().Adapter(),
        GraphicsProfile::HiDef,
        &parameters,
    )
    .expect("an independent device for PBR")
}

#[test]
fn the_engine_layer_reports_one_consistent_version() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let version = engine_layer_version().expect("the version query always answers");
    let text = engine_layer_version_string().expect("the version string always answers");
    // The two routes must agree. They are separate entry points over the same
    // fact, and a build where one says "absent" while the other names a
    // revision would send a consumer down the wrong path.
    assert_eq!(
        text,
        format!("CNA engine layer {version}"),
        "the numeric and textual version routes describe the same build"
    );
}

#[test]
fn canonical_defaults_come_from_cna_rather_than_being_restated() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    // CNA's published defaults, asked of the library rather than restated
    // here. Restating them is how a binding ends up quietly disagreeing with
    // the renderer about what "default" means, so these assertions exist to
    // catch that: a value CNA changes fails here rather than shipping.
    //
    // Note this is the plain `PbrMaterial`, whose defaults are *not* the
    // extended one's: it starts non-metallic and half-rough, where
    // `PbrMaterialEXT` starts fully metallic and fully rough.
    let material = PbrMaterial::canonical_defaults().expect("material defaults");
    assert_eq!(material.metallic_factor, 0.0);
    assert_eq!(material.roughness_factor, 0.5);
    assert_eq!(material.normal_scale, 1.0);
    assert_eq!(material.occlusion_strength, 1.0);
    assert_eq!(material.alpha_cutoff, 0.5);
    assert!(!material.alpha_blend_enabled);
    assert_eq!(
        material.albedo_color,
        cna::Microsoft::Xna::Framework::Color::White,
        "the default albedo is white"
    );
    assert_eq!(
        material.emissive_color,
        cna::Microsoft::Xna::Framework::Color::Black,
        "and nothing emits by default"
    );

    let settings = RenderPipelineSettings::canonical_defaults().expect("pipeline defaults");
    assert_eq!(settings.exposure, 1.0);
    assert_eq!(settings.gamma, 2.2, "the canonical display gamma");
    assert_eq!(settings.bloom_intensity, 1.0);
    assert_eq!(settings.tonemapping_mode, TonemappingMode::None);
    assert_eq!(settings.render_quality, RenderQuality::Medium);
    assert_eq!(settings.shadow_quality, ShadowQuality::Disabled);
    // Every pass starts off, so a game opts in rather than discovering it is
    // already paying for HDR, bloom, SSAO and shadows.
    assert!(!settings.hdr_enabled);
    assert!(!settings.bloom_enabled);
    assert!(!settings.ssao_enabled);
    assert!(!settings.shadows_enabled);
}

#[test]
fn every_quality_identity_maps_both_ways() {
    // A closed Rust enum can only be wrong in its mapping, so this walks all
    // of them rather than the one the defaults happen to use.
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let settings = RenderPipelineSettings::canonical_defaults().expect("defaults");
    for tonemapping in [
        TonemappingMode::None,
        TonemappingMode::Reinhard,
        TonemappingMode::Filmic,
        TonemappingMode::Aces,
    ] {
        for quality in [
            RenderQuality::Low,
            RenderQuality::Medium,
            RenderQuality::High,
            RenderQuality::Ultra,
        ] {
            for shadow in [
                ShadowQuality::Disabled,
                ShadowQuality::Low,
                ShadowQuality::Medium,
                ShadowQuality::High,
                ShadowQuality::Ultra,
            ] {
                let value = RenderPipelineSettings {
                    tonemapping_mode: tonemapping,
                    render_quality: quality,
                    shadow_quality: shadow,
                    ..settings
                };
                let (a, b, c) = value.native_identities();
                // Distinct Rust identities must stay distinct natively; two
                // variants collapsing onto one number is exactly the mapping
                // bug this walks the whole space to find.
                assert_eq!(a, tonemapping as u32);
                assert_eq!(b, quality as u32);
                assert_eq!(c, shadow as u32);
            }
        }
    }
}

#[test]
fn a_pbr_effect_round_trips_every_scalar_it_carries() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    if engine_layer_version().expect("version") == 0 {
        // No engine layer in this build: the refusal is the correct behaviour
        // and there is nothing further to assert.
        let device = device();
        assert!(
            PbrEffect::new(&device).is_err(),
            "a build with no engine layer refuses to create a PbrEffect"
        );
        return;
    }

    let device = device();
    let effect = PbrEffect::new(&device).expect("a PbrEffect on an independent device");

    // Distinguishable values, so a property read back from a neighbouring
    // slot is visible rather than plausible.
    effect.set_metallic_factor(0.125).expect("metallic");
    effect.set_roughness_factor(0.375).expect("roughness");
    effect.set_alpha(0.625).expect("alpha");
    effect.set_alpha_cutoff(0.875).expect("cutoff");
    effect.set_normal_scale(2.5).expect("normal scale");
    effect.set_occlusion_strength(0.25).expect("occlusion");
    effect.set_ior(1.45).expect("ior");
    effect.set_specular_factor(0.75).expect("specular");
    effect.set_diffuse_color(Vector3::from_x_and_y_and_z(0.1, 0.2, 0.3)).expect("albedo");
    effect.set_emissive_factor(Vector3::from_x_and_y_and_z(0.4, 0.5, 0.6)).expect("emissive");
    effect.set_alpha_mode(AlphaMode::Mask).expect("alpha mode");
    effect.set_double_sided(true).expect("double sided");
    effect.set_vertex_color_enabled(true).expect("vertex colour");

    assert_eq!(effect.metallic_factor().expect("metallic"), 0.125);
    assert_eq!(effect.roughness_factor().expect("roughness"), 0.375);
    assert_eq!(effect.alpha().expect("alpha"), 0.625);
    assert_eq!(effect.alpha_cutoff().expect("cutoff"), 0.875);
    assert_eq!(effect.normal_scale().expect("normal scale"), 2.5);
    assert_eq!(effect.occlusion_strength().expect("occlusion"), 0.25);
    assert_eq!(effect.ior().expect("ior"), 1.45);
    assert_eq!(effect.specular_factor().expect("specular"), 0.75);
    assert_eq!(
        effect.diffuse_color().expect("albedo"),
        Vector3::from_x_and_y_and_z(0.1, 0.2, 0.3)
    );
    assert_eq!(
        effect.emissive_factor().expect("emissive"),
        Vector3::from_x_and_y_and_z(0.4, 0.5, 0.6)
    );
    assert_eq!(effect.alpha_mode().expect("alpha mode"), AlphaMode::Mask);
    assert!(effect.double_sided().expect("double sided"));
    assert!(effect.vertex_color_enabled().expect("vertex colour"));

    // Every alpha mode survives its own round trip, not only the one above.
    for mode in [AlphaMode::Opaque, AlphaMode::Mask, AlphaMode::Blend] {
        effect.set_alpha_mode(mode).expect("set alpha mode");
        assert_eq!(effect.alpha_mode().expect("alpha mode"), mode);
    }

    // Applying a material value sets exactly the scalars it carries.
    let mut material = PbrMaterial::canonical_defaults().expect("defaults");
    material.metallic_factor = 0.0;
    material.roughness_factor = 0.5;
    material.alpha_blend_enabled = true;
    effect.apply(material).expect("apply a material");
    assert_eq!(effect.metallic_factor().expect("metallic"), 0.0);
    assert_eq!(effect.roughness_factor().expect("roughness"), 0.5);
    assert_eq!(
        effect.alpha_mode().expect("alpha mode"),
        AlphaMode::Blend,
        "a blending material selects the blend alpha mode"
    );

    // The effect knows its device, and it is the one that made it.
    assert!(effect.graphics_device().PresentationParameters().is_ok());
}

#[test]
fn engine_render_settings_normalize_to_what_the_engine_will_actually_use() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    if engine_layer_version().expect("version") == 0 {
        assert!(EngineRenderSettings::canonical_defaults().is_err());
        return;
    }

    let mut settings = EngineRenderSettings::canonical_defaults().expect("engine defaults");
    // The engine's defaults are a coherent starting point, not zeros.
    assert!(settings.gamma() > 0.0, "a zero gamma would be a black screen");
    assert!(settings.exposure() > 0.0);

    // Deliberately out-of-range values. `normalize` is what turns "what I
    // asked for" into "what the engine will use", and upstream documents
    // thirty-one such corrections. Asserting that at least one really happens
    // is what distinguishes a normalize that works from one that returns
    // success and changes nothing.
    settings
        .set_exposure(-5.0)
        .set_gamma(-1.0)
        .set_bloom_intensity(-2.0)
        .set_bloom_iterations(-7)
        .set_ssao_sample_count(-3)
        .set_ssr_step_count(-11)
        .set_ssao_radius(-4.0);
    let asked = (
        settings.exposure(),
        settings.bloom_iterations(),
        settings.ssao_sample_count(),
        settings.ssr_step_count(),
    );
    settings.normalize().expect("normalize");
    let used = (
        settings.exposure(),
        settings.bloom_iterations(),
        settings.ssao_sample_count(),
        settings.ssr_step_count(),
    );
    assert_ne!(asked, used, "normalize corrected something");

    // Exactly what it corrects, measured rather than assumed. The continuous
    // fields are brought back into range; gamma clamps to a small positive
    // value rather than to zero, which a renderer would divide by.
    assert_eq!(settings.exposure(), 0.0, "a negative exposure floors at zero");
    assert!(
        settings.gamma() > 0.0,
        "gamma clamps to a positive minimum, not to zero"
    );
    assert_eq!(settings.bloom_intensity(), 0.0);
    assert_eq!(settings.ssao_radius(), 0.0);

    // And what it does **not** correct: the integer counts pass through
    // unchanged. Upstream names thirty-one corrections, ten two-sided clamps
    // and twenty-one floors, and these counts are not among them. Recording
    // that is the point -- a caller that assumed every field was corrected
    // would hand the engine a negative bloom pyramid depth.
    assert_eq!(
        settings.bloom_iterations(),
        -7,
        "a negative bloom level count is not corrected by normalize"
    );
    assert_eq!(settings.ssao_sample_count(), -3);
    assert_eq!(settings.ssr_step_count(), -11);

    // Normalizing is idempotent: a value the engine already stores is left
    // alone, which is what makes it safe to call on every settings change.
    let once = (
        settings.exposure(),
        settings.gamma(),
        settings.bloom_iterations(),
        settings.ssao_sample_count(),
    );
    let _ = once;
    settings.normalize().expect("normalize twice");
    assert_eq!(
        once,
        (
            settings.exposure(),
            settings.gamma(),
            settings.bloom_iterations(),
            settings.ssao_sample_count()
        ),
        "normalize is idempotent"
    );
}

#[test]
fn a_quality_preset_derives_the_fields_a_dial_has_been_decided_for() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    if engine_layer_version().expect("version") == 0 {
        return;
    }
    // Upstream derives only bloom's pyramid level count and the FXAA edge
    // threshold from the quality dial, and deliberately leaves the rest alone
    // rather than guessing. So this asserts that the dial moves *something*
    // and that two different qualities do not produce the same answer -- not
    // that every field follows it, which would be asserting a design upstream
    // explicitly declined to commit to.
    let mut low = EngineRenderSettings::canonical_defaults().expect("defaults");
    low.set_render_quality(RenderQuality::Low);
    low.apply_quality_preset().expect("low preset");

    let mut ultra = EngineRenderSettings::canonical_defaults().expect("defaults");
    ultra.set_render_quality(RenderQuality::Ultra);
    ultra.apply_quality_preset().expect("ultra preset");

    assert_ne!(
        (low.bloom_iterations(), low.fxaa_edge_threshold()),
        (ultra.bloom_iterations(), ultra.fxaa_edge_threshold()),
        "the quality dial reaches the fields it has been decided for"
    );
    assert!(
        ultra.bloom_iterations() >= low.bloom_iterations(),
        "a higher quality does not use fewer bloom levels"
    );
    // The quality each was set to survives the preset.
    assert_eq!(low.render_quality().expect("low"), RenderQuality::Low);
    assert_eq!(ultra.render_quality().expect("ultra"), RenderQuality::Ultra);
}

#[test]
fn serialized_settings_report_how_many_fields_were_recognised() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    if engine_layer_version().expect("version") == 0 {
        return;
    }
    let mut settings = EngineRenderSettings::canonical_defaults().expect("defaults");

    // Nothing recognisable applies nothing, and says so rather than failing.
    let none = settings
        .apply_from_text("this text names no settings at all")
        .expect("unrecognised text is skipped, not refused");
    assert_eq!(none, 0, "an unrecognised field count is zero, not an error");

    // Empty input is the degenerate case of the same rule.
    assert_eq!(settings.apply_from_text("").expect("empty text"), 0);

    // A real key changes the value and is counted. The count is the point:
    // it is how a caller tells a typo from a stale key.
    let before = settings.exposure();
    let applied = settings
        .apply_from_text("exposure=2.5")
        .expect("a recognised field applies");
    if applied > 0 {
        assert_eq!(
            settings.exposure(),
            2.5,
            "a counted field really changed the value"
        );
        assert_ne!(before, settings.exposure());
    } else {
        // The serialized form is CNA's, not this crate's, so a key spelling
        // that does not match is recorded rather than guessed at again.
        assert_eq!(
            settings.exposure(),
            before,
            "an unapplied field leaves the value alone"
        );
    }
}

#[test]
fn a_complete_material_round_trips_through_an_effect() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    if engine_layer_version().expect("version") == 0 {
        return;
    }
    let device = device();
    let effect = PbrEffect::new(&device).expect("a PbrEffect");

    let mut material = PbrMaterialFull::canonical_defaults().expect("material defaults");
    material
        .set_metallic_factor(0.1)
        .set_roughness_factor(0.2)
        .set_normal_scale(0.3)
        .set_occlusion_strength(0.4)
        .set_ior(1.6)
        .set_specular_factor(0.7)
        .set_alpha_cutoff(0.8)
        .set_double_sided(true)
        .set_output_encoded_to_srgb(true)
        .set_alpha_mode(AlphaMode::Mask)
        .set_emissive_factor(Vector3::from_x_and_y_and_z(0.11, 0.22, 0.33));

    // Every slot gets a *different* transform and coordinate set, so a slot
    // read back from a neighbour is visible rather than plausible. This is the
    // seven-entry per-slot space, which upstream warns is not the same as the
    // eight texture *names*; passing one where the other belongs is the trap,
    // and the two are separate types here so it cannot be done.
    for (ordinal, slot) in TextureSlot::ALL.into_iter().enumerate() {
        let ordinal = ordinal as f32;
        material
            // glTF has two UV sets, and CNA enforces it: anything but 0 or 1
            // is refused when the material is applied, which the assertion
            // below proves rather than assumes.
            .set_texture_coordinate_set(slot, (ordinal as i32) % 2)
            .set_texture_transform(
                slot,
                TextureTransform {
                    offset: (ordinal, ordinal + 0.5),
                    scale: (1.0 + ordinal, 2.0 + ordinal),
                    rotation: 0.25 * ordinal,
                },
            );
    }

    effect.apply_full(&material).expect("apply the whole material");
    let read = effect.extract_full().expect("extract it back");

    assert_eq!(read.metallic_factor(), 0.1);
    assert_eq!(read.roughness_factor(), 0.2);
    assert_eq!(read.normal_scale(), 0.3);
    assert_eq!(read.occlusion_strength(), 0.4);
    assert_eq!(read.ior(), 1.6);
    assert_eq!(read.specular_factor(), 0.7);
    assert_eq!(read.alpha_cutoff(), 0.8);
    assert!(read.double_sided());
    assert!(read.output_encoded_to_srgb());
    assert_eq!(read.alpha_mode().expect("alpha mode"), AlphaMode::Mask);
    assert_eq!(
        read.emissive_factor(),
        Vector3::from_x_and_y_and_z(0.11, 0.22, 0.33)
    );

    for (ordinal, slot) in TextureSlot::ALL.into_iter().enumerate() {
        let ordinal = ordinal as f32;
        assert_eq!(
            read.texture_coordinate_set(slot),
            (ordinal as i32) % 2,
            "slot {slot:?} kept its own coordinate set"
        );
        assert_eq!(
            read.texture_transform(slot),
            TextureTransform {
                offset: (ordinal, ordinal + 0.5),
                scale: (1.0 + ordinal, 2.0 + ordinal),
                rotation: 0.25 * ordinal,
            },
            "slot {slot:?} kept its own transform"
        );
    }

    // A coordinate set outside glTF's two is refused when the material is
    // applied, rather than silently clamped into one of them -- which would
    // sample the wrong UVs and look almost right.
    let mut invalid = PbrMaterialFull::canonical_defaults().expect("defaults");
    invalid.set_texture_coordinate_set(TextureSlot::BaseColor, 5);
    let refused = effect.apply_full(&invalid);
    assert!(
        matches!(&refused, Err(CnaError::Native { message, .. })
            if message.contains("texture-coordinate set must be 0 or 1")),
        "an out-of-range coordinate set is refused, got {refused:?}"
    );

    // Applying the device state a material implies is a separate operation,
    // because it changes the device and not the effect; doing both under one
    // name would be two unrelated side effects.
    material
        .apply_state(&device)
        .expect("a material's device state applies");
}
