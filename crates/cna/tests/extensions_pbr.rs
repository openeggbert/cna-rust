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
    engine_layer_version, engine_layer_version_string, AlphaMode, PbrEffect, PbrMaterial,
    RenderPipelineSettings, RenderQuality, ShadowQuality, TonemappingMode,
};
use cna::Microsoft::Xna::Framework::Graphics::{
    GraphicsDevice, GraphicsProfile, PresentationParameters,
};
use cna::Microsoft::Xna::Framework::{GraphicsDeviceInformation, Vector3};

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
