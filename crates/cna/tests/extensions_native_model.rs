//! The model CNA's own content pipeline loads, and the glTF facts it carries.
//!
//! This is not a second test for `Model`. `crates/cna/src/graphics/model.rs`
//! is the XNA `Model`, built by this crate's own `.xnb` reader, and
//! `content_xnb.rs` covers it. What is under test here is the half of CNA's
//! content story the Rust reader cannot reach: CNA's `ModelTypeReader` accepts
//! `.gltf` and `.glb` directly, and everything a glTF import knows -- the
//! import report, the cameras the scene declared, the skins, the material
//! variants -- exists only on a model CNA loaded.
//!
//! The asset is written by the test rather than checked in, so what each
//! assertion depends on is visible in the same file as the assertion.
//!
//! # Why every case runs in a child process
//!
//! Loading a model with a mesh part faults this process before it ends --
//! `RUST-UPSTREAM-021`, and leaking the handle does not avoid it because the C
//! API's handle registry runs the same destructor at exit. So each case runs in
//! a child that prints `OK: <case>` when its assertions pass, and the parent
//! reads that marker rather than the child's exit status. The status is
//! reported too: a child that exits *cleanly* means upstream has been fixed,
//! and the parent says so.

#![allow(clippy::needless_return)]

use std::path::Path;
use std::process::Command;

use cna::extensions::content::NativeContentManager;
use cna::extensions::gamer_services::{AvatarAppearance, AvatarRealRendering};
use cna::extensions::models::SkinnedModel;
use cna::extensions::native_model::{GltfImportKind, GltfImportSeverity, NativeModel};
use cna::Microsoft::Xna::Framework::Graphics::{
    GraphicsDevice, GraphicsProfile, PresentationParameters,
};
use cna::Microsoft::Xna::Framework::GamerServices::{AvatarDescription, AvatarRenderer};
use cna::Microsoft::Xna::Framework::{Color, GraphicsDeviceInformation, Matrix, TimeSpan};
use cna::{CnaError, ErrorCategory, Result};

/// The env var naming which case the child should run.
const STAGE: &str = "CNA_RUST_NATIVE_MODEL_STAGE";

/// Runs one case whose whole point is the teardown that faults today.
///
/// The child cannot reach its marker while `RUST-UPSTREAM-021` stands, so this
/// asserts the *absence* of the marker and the fault status. The day upstream
/// fixes it, the marker appears and this fails -- which is what will send
/// somebody to retire the finding.
fn in_child_blocked_by_upstream_021(case: &str, body: impl FnOnce()) {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    if std::env::var(STAGE).as_deref() == Ok(case) {
        body();
        println!("OK: {case}");
        return;
    }
    if std::env::var_os(STAGE).is_some() {
        return;
    }

    let Some(output) = spawn_case(case) else { return };
    let text = String::from_utf8_lossy(&output.stdout).into_owned()
        + &String::from_utf8_lossy(&output.stderr);
    if text.contains(&format!("SKIP: {case}")) {
        println!("SKIP: {case} -- the child could not build a host");
        return;
    }
    let reached_the_end = text.contains(&format!("OK: {case}"));
    println!(
        "MEASURED: `{case}` status={:?} reached-the-end={reached_the_end}",
        output.status
    );
    assert!(
        !reached_the_end,
        "RUST-UPSTREAM-021 no longer reproduces: destroying a content-loaded model with \
         mesh parts completed. Re-measure, retire the finding, and take the warning off \
         `NativeModel`."
    );
}

fn spawn_case(case: &str) -> Option<std::process::Output> {
    let exe = std::env::current_exe().expect("this test binary");
    Some(
        Command::new(exe)
            .args(["--test-threads=1", "--nocapture", "--exact", case])
            .env(STAGE, case)
            .output()
            .expect("run the case in a child process"),
    )
}

/// Runs one case in a child process and reports what it found.
///
/// In the child this simply calls `body`. In the parent it spawns the child,
/// requires the `OK:` marker, and treats a clean exit as news rather than as
/// the expected outcome.
fn in_child(case: &str, body: impl FnOnce()) {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    if std::env::var(STAGE).as_deref() == Ok(case) {
        body();
        println!("OK: {case}");
        return;
    }
    if std::env::var_os(STAGE).is_some() {
        // A different case's child; this one has nothing to do.
        return;
    }

    let Some(output) = spawn_case(case) else { return };
    let text = String::from_utf8_lossy(&output.stdout).into_owned()
        + &String::from_utf8_lossy(&output.stderr);

    for line in text.lines().filter(|line| {
        line.starts_with("OK:") || line.starts_with("SKIP:") || line.contains("panicked")
    }) {
        println!("    {line}");
    }

    if text.contains(&format!("SKIP: {case}")) {
        println!("SKIP: {case} -- the child could not build a host");
        return;
    }
    assert!(
        text.contains(&format!("OK: {case}")),
        "the child running `{case}` did not reach its end. status={:?}\n{text}",
        output.status
    );
    if output.status.success() {
        println!(
            "NOTE: `{case}` exited cleanly. Loading a model with a mesh part no longer \
             faults on this CNA build -- if that is a real upstream fix, RUST-UPSTREAM-021 \
             can be retired and `NativeModel`'s warning removed."
        );
    }
}

/// A triangle, a two-node skeleton, a camera, a skin and a material.
///
/// Deliberately small but not degenerate: every EXT family this module projects
/// needs something in the source to report, and a scene with one node and no
/// camera would let an accessor that always answers zero pass.
const GLTF: &str = r#"{
  "asset": { "version": "2.0", "generator": "cna-rust test" },
  "scene": 0,
  "scenes": [ { "nodes": [ 0, 3 ] } ],
  "nodes": [
    { "name": "Root",       "children": [ 1 ], "mesh": 0, "skin": 0 },
    { "name": "Child",      "translation": [ 0.0, 2.0, 0.0 ] },
    { "name": "Unattached", "translation": [ 5.0, 0.0, 0.0 ] },
    { "name": "CameraNode", "camera": 0, "translation": [ 0.0, 0.0, 4.0 ] }
  ],
  "cameras": [
    { "name": "MainCamera", "type": "perspective",
      "perspective": { "yfov": 0.8, "znear": 0.25, "zfar": 120.0, "aspectRatio": 1.5 } }
  ],
  "skins": [ { "name": "MainSkin", "joints": [ 0, 1 ], "inverseBindMatrices": 1 } ],
  "meshes": [
    { "name": "Triangle",
      "primitives": [ { "attributes": { "POSITION": 0, "JOINTS_0": 2, "WEIGHTS_0": 3 },
                        "material": 0 } ] }
  ],
  "materials": [
    { "name": "Red", "pbrMetallicRoughness": { "baseColorFactor": [ 1.0, 0.0, 0.0, 1.0 ] } }
  ],
  "accessors": [
    { "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3",
      "min": [ 0.0, 0.0, 0.0 ], "max": [ 1.0, 1.0, 0.0 ] },
    { "bufferView": 1, "componentType": 5126, "count": 2, "type": "MAT4" },
    { "bufferView": 2, "componentType": 5121, "count": 3, "type": "VEC4" },
    { "bufferView": 3, "componentType": 5126, "count": 3, "type": "VEC4" }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0,   "byteLength": 36  },
    { "buffer": 0, "byteOffset": 36,  "byteLength": 128 },
    { "buffer": 0, "byteOffset": 164, "byteLength": 12  },
    { "buffer": 0, "byteOffset": 176, "byteLength": 48  }
  ],
  "buffers": [ { "byteLength": 224, "uri": "data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAAAACAPwAAAAAAAAAAAAAAAAAAAAAAAIA/AAAAAAAAAAAAAAAAAAAAAAAAgD8AAAAAAAAAAAAAAAAAAAAAAACAPwAAgD8AAAAAAAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAAAAAAIA/AAAAAAAAAAAAAAAAAAAAAAAAgD8AAAAAAAAAAAEAAAAAAIA/AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAAAAAAAAAAAAA=" } ]
}"#;

/// A device, a content manager rooted at a fresh directory, and the asset in it.
///
/// `None` when this renderer cannot make a windowless device, which is a
/// renderer property rather than a binding fault.
fn loaded_model() -> Option<(GraphicsDevice, NativeContentManager, &'static NativeModel)> {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return None;
    }
    let device = independent_device_or_skip(|| {
        let parameters = PresentationParameters::new();
        parameters.SetBackBufferWidth(64);
        parameters.SetBackBufferHeight(64);
        GraphicsDevice::new(
            &GraphicsDeviceInformation::new().Adapter(),
            GraphicsProfile::HiDef,
            &parameters,
        )
    })?;
    let root = content_root();
    let manager = NativeContentManager::new(&device, root.to_str().expect("utf-8 content root"))
        .expect("native content manager on that device");
    let model = match NativeModel::load(&manager, "scene") {
        Ok(model) => model,
        Err(error) => {
            // A library built without the glTF importer is a real answer about
            // the artifact, not a failure of the binding -- but only that
            // answer. Anything else still fails.
            if matches!(
                error,
                CnaError::Native {
                    category: ErrorCategory::Io,
                    ..
                }
            ) {
                println!("this artifact does not load .gltf assets: {error}");
                return None;
            }
            panic!("loading the glTF model failed: {error}");
        }
    };
    // Leaked on purpose. Dropping it runs `cna_model_destroy`, which faults --
    // RUST-UPSTREAM-021 -- and a fault before the `OK:` marker would tell the
    // parent the assertions failed when they did not. The process still faults
    // at exit; the marker just gets out first.
    Some((device, manager, Box::leak(Box::new(model))))
}

/// Writes the asset once, under the target directory rather than a temporary
/// one, so a failing run leaves the exact input behind to look at.
fn content_root() -> std::path::PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-model-content");
    std::fs::create_dir_all(&root).expect("content root");
    std::fs::write(root.join("scene.gltf"), GLTF).expect("write the glTF asset");
    root
}

fn independent_device_or_skip(build: impl FnOnce() -> Result<GraphicsDevice>) -> Option<GraphicsDevice> {
    match build() {
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
fn a_gltf_scene_arrives_as_bones_and_meshes() {
    in_child("a_gltf_scene_arrives_as_bones_and_meshes", || {
        let Some((_device, _manager, model)) = loaded_model() else {
            println!("SKIP: {}", std::env::var(STAGE).unwrap_or_default());
            return;
        };

        let bones = model.bones().expect("bones");
        let names: Vec<String> = bones
            .iter()
            .map(|bone| bone.name().expect("bone name"))
            .collect();
        assert!(
            names.iter().any(|name| name == "Root"),
            "the imported bones should carry the source node names, got {names:?}"
        );
        assert!(
            names.iter().any(|name| name == "Child"),
            "a child node should import as a bone too, got {names:?}"
        );

        // Indices are the model's own and must agree with the collection order,
        // which is what every other index in this API is stated against.
        for (position, bone) in bones.iter().enumerate() {
            assert_eq!(
                bone.index().expect("bone index") as usize,
                position,
                "bone {position} reports a different index than its position"
            );
        }

        let meshes = model.meshes().expect("meshes");
        assert_eq!(meshes.len(), 1, "the source declares exactly one mesh");
        assert_eq!(meshes[0].name().expect("mesh name"), "Triangle");
        assert_eq!(
            meshes[0].part_count().expect("part count"),
            1,
            "one primitive is one part"
        );
    });
}


#[test]
fn a_bone_named_lookup_answers_the_same_bone_as_the_collection() {
    in_child("a_bone_named_lookup_answers_the_same_bone_as_the_collection", || {
        let Some((_device, _manager, model)) = loaded_model() else {
            println!("SKIP: {}", std::env::var(STAGE).unwrap_or_default());
            return;
        };

        let found = model
            .bone_named("Child")
            .expect("bone lookup")
            .expect("the source declares a Child node");
        let by_index = model.bones().expect("bones");
        let expected = by_index
            .iter()
            .find(|bone| bone.name().expect("bone name") == "Child")
            .expect("Child is in the collection");
        assert_eq!(
            found.index().expect("index"),
            expected.index().expect("index"),
            "lookup by name and lookup by index should reach the same bone"
        );

        assert!(
            model.bone_named("NoSuchBone").expect("bone lookup").is_none(),
            "a name the model does not carry answers None rather than failing"
        );
    });
}


#[test]
fn a_view_outlives_the_model_it_came_from() {
    in_child_blocked_by_upstream_021("a_view_outlives_the_model_it_came_from", || {
        let Some((_device, _manager, model)) = loaded_model() else {
            println!("SKIP: {}", std::env::var(STAGE).unwrap_or_default());
            return;
        };

        let bone = model
            .bone_named("Root")
            .expect("bone lookup")
            .expect("Root exists");
        let name_before = bone.name().expect("name before");
        let index_before = bone.index().expect("index before");

        // The whole reason `ModelBoneView` carries no lifetime parameter. Dropping
        // the model must not invalidate a view taken from it, heap state included;
        // if this ever changes, the type needs a lifetime and this fails first.
        //
        // On a CNA carrying RUST-UPSTREAM-021 the child does not return from
        // this release, so the assertions below are what a fixed CNA will
        // check. The pure-C probe measured the same property on a *hand-built*
        // model, where the teardown works, and that is the evidence the type's
        // missing lifetime parameter actually rests on.
        model.release().expect("release the model");

        assert_eq!(
            bone.name().expect("name after the model is gone"),
            name_before,
            "a bone view should keep its name after the model is released"
        );
        assert_eq!(
            bone.index().expect("index after the model is gone"),
            index_before
        );
    });
}


#[test]
fn the_import_report_counts_what_the_source_declared() {
    in_child("the_import_report_counts_what_the_source_declared", || {
        let Some((_device, _manager, model)) = loaded_model() else {
            println!("SKIP: {}", std::env::var(STAGE).unwrap_or_default());
            return;
        };

        let report = model.gltf_import_report().expect("import report");
        assert!(
            report.node_count >= 3,
            "the source declares four nodes, two of them in the scene; got {}",
            report.node_count
        );
        assert_eq!(
            report.camera_node_count, 1,
            "exactly one node references a camera"
        );
        assert_eq!(report.skin_count, 1, "the source declares one skin");
        assert!(
            report.distinct_mesh_count >= 1,
            "the source declares one mesh"
        );

        // Every diagnostic must decode into the closed Rust enums. An entry CNA
        // reports with an identity this crate does not know is a real drift signal,
        // so it fails here rather than being silently mapped to a default.
        for diagnostic in model.gltf_import_diagnostics().expect("diagnostics") {
            assert!(
                !diagnostic.code.is_empty(),
                "every diagnostic carries a stable code"
            );
            assert!(
                matches!(
                    diagnostic.severity,
                    GltfImportSeverity::Information | GltfImportSeverity::Warning
                ),
                "severity decoded outside the closed set"
            );
            assert!(
                matches!(
                    diagnostic.kind,
                    GltfImportKind::Information
                        | GltfImportKind::GeneratedData
                        | GltfImportKind::InvalidSourceData
                        | GltfImportKind::Approximation
                        | GltfImportKind::DroppedData
                        | GltfImportKind::UnsupportedFeature
                ),
                "kind decoded outside the closed set"
            );
            println!(
                "diagnostic {} [{:?}/{:?}] x{}: {}",
                diagnostic.code,
                diagnostic.severity,
                diagnostic.kind,
                diagnostic.count,
                diagnostic.message
            );
        }

        assert_eq!(
            report.anything_lost,
            report.warning_count > 0,
            "`anything_lost` is defined as 'at least one warning is present'"
        );
    });
}


#[test]
fn the_imported_camera_keeps_the_source_s_own_numbers() {
    in_child("the_imported_camera_keeps_the_source_s_own_numbers", || {
        let Some((_device, _manager, model)) = loaded_model() else {
            println!("SKIP: {}", std::env::var(STAGE).unwrap_or_default());
            return;
        };

        let cameras = model.cameras().expect("cameras");
        assert_eq!(cameras.len(), 1, "the source declares one camera");
        let camera = &cameras[0];

        assert_eq!(camera.name, "MainCamera");
        assert!(camera.is_perspective, "the source declares a perspective camera");
        assert!(
            !camera.has_infinite_far_plane,
            "the source declares zfar, so the far plane is finite"
        );
        assert!(
            camera.has_authored_aspect_ratio,
            "the source declares aspectRatio"
        );
        assert!(
            (camera.field_of_view - 0.8).abs() < 1e-5,
            "yfov should arrive verbatim, got {}",
            camera.field_of_view
        );
        assert!(
            (camera.near_plane_distance - 0.25).abs() < 1e-5,
            "znear should arrive verbatim, got {}",
            camera.near_plane_distance
        );
        assert!(
            (camera.far_plane_distance - 120.0).abs() < 1e-3,
            "zfar should arrive verbatim, got {}",
            camera.far_plane_distance
        );
        assert!(
            (camera.aspect_ratio - 1.5).abs() < 1e-5,
            "aspectRatio should arrive verbatim, got {}",
            camera.aspect_ratio
        );
        assert_ne!(
            camera.projection,
            Matrix::default(),
            "an imported camera should carry a real projection, not a zeroed one"
        );
    });
}


#[test]
fn the_imported_skin_names_the_meshes_it_poses() {
    in_child("the_imported_skin_names_the_meshes_it_poses", || {
        let Some((_device, _manager, model)) = loaded_model() else {
            println!("SKIP: {}", std::env::var(STAGE).unwrap_or_default());
            return;
        };

        let skins = model.skins().expect("skins");
        assert_eq!(skins.len(), 1, "the source declares one skin");
        let skin = &skins[0];
        assert_eq!(skin.name, "MainSkin");
        assert!(skin.has_skeleton, "the skin declares joints");

        let mesh_count = model.mesh_count().expect("mesh count");
        for index in &skin.mesh_indices {
            assert!(
                *index < mesh_count,
                "a skin's mesh index must address this model's own mesh collection, \
                 got {index} against {mesh_count} meshes"
            );
        }

        // The skeleton itself is not reachable for a skin the *content loader*
        // built: upstream answers INVALID_STATE, "The Model skin's skeleton was
        // not created through the C API". That is a real gap rather than a
        // binding fault -- `has_skeleton` above is true -- so it is asserted as
        // measured and recorded as RUST-UPSTREAM-022. When upstream closes it
        // this fails and says so.
        match model.skin_skeleton(0) {
            Err(CnaError::Native {
                category: ErrorCategory::State,
                ref message,
                ..
            }) if message.contains("not created through the C API") => {
                println!("MEASURED: a content-loaded skin's skeleton is unreachable: {message}");
            }
            Ok(Some(skeleton)) => {
                let bones = skeleton.bone_count().expect("skeleton bone count");
                assert!(bones > 0, "the skeleton should carry the source's joints");
                drop(skeleton);
                assert_eq!(
                    model.skins().expect("skins after dropping the skeleton").len(),
                    1,
                    "dropping the caller's skeleton handle must not remove the skin"
                );
                panic!(
                    "RUST-UPSTREAM-022 no longer reproduces: a content-loaded skin's \
                     skeleton is reachable now. Re-measure and retire the finding."
                );
            }
            Ok(None) => panic!(
                "the skin reports a skeleton but `skin_skeleton` answered None"
            ),
            Err(other) => panic!(
                "unexpected refusal for a content-loaded skin's skeleton: {other}"
            ),
        }
    });
}


#[test]
fn a_material_variant_selection_round_trips() {
    in_child("a_material_variant_selection_round_trips", || {
        let Some((_device, _manager, model)) = loaded_model() else {
            println!("SKIP: {}", std::env::var(STAGE).unwrap_or_default());
            return;
        };

        let variants = model.material_variants().expect("material variants");
        // The source declares no KHR_materials_variants extension, so the honest
        // answer is an empty list -- and selecting nothing must still be accepted.
        assert!(
            model.material_variant().expect("selected variant").is_none()
                || !variants.is_empty(),
            "a model with no variants must not report one selected"
        );

        model
            .set_material_variant(None)
            .expect("restoring the default materials is always allowed");
        assert_eq!(model.material_variant().expect("selected variant"), None);

        // Past the declared variants is refused rather than clamped, which is what
        // keeps a caller's index bug from silently drawing the wrong material.
        let past_the_end = variants.len() as u64;
        assert!(
            model.set_material_variant(Some(past_the_end)).is_err(),
            "selecting variant {past_the_end} of {} should be refused",
            variants.len()
        );
    });
}


#[test]
fn bone_transforms_round_trip_through_the_bulk_routes() {
    in_child("bone_transforms_round_trip_through_the_bulk_routes", || {
        let Some((_device, _manager, model)) = loaded_model() else {
            println!("SKIP: {}", std::env::var(STAGE).unwrap_or_default());
            return;
        };

        let count = model.bone_transform_count().expect("bone transform count");
        assert!(count > 0, "an imported scene should have bones");

        let local = model.bone_transforms().expect("local transforms");
        assert_eq!(local.len() as u64, count, "one matrix per bone");

        let absolute = model
            .absolute_bone_transforms()
            .expect("absolute transforms");
        assert_eq!(absolute.len() as u64, count);

        // Writing back what was read must change nothing observable. That is a
        // stronger check than writing identities: it exercises both directions of
        // the marshalling with real values, so a transposed matrix fails here.
        model
            .set_bone_transforms(&local)
            .expect("writing back the same transforms");
        assert_eq!(
            model.bone_transforms().expect("local transforms again"),
            local,
            "a read-write-read cycle should be the identity"
        );

        // Short input is refused rather than applied in part.
        if count > 1 {
            assert!(
                model.set_bone_transforms(&local[..local.len() - 1]).is_err(),
                "a transform array that does not cover every bone should be refused"
            );
            assert_eq!(
                model.bone_transforms().expect("local transforms after the refusal"),
                local,
                "a refused write must not have applied part of itself"
            );
        }
    });
}


#[test]
fn a_released_model_refuses_further_work() {
    in_child_blocked_by_upstream_021("a_released_model_refuses_further_work", || {
        let Some((_device, _manager, model)) = loaded_model() else {
            println!("SKIP: {}", std::env::var(STAGE).unwrap_or_default());
            return;
        };

        // `release` calls `cna_model_destroy` for real. On a CNA that still
        // carries RUST-UPSTREAM-021 the child does not come back from this --
        // which is exactly why the parent reads the `OK:` marker rather than
        // the exit status. Everything below runs only on a fixed CNA.
        model.release().expect("release the model");

        assert!(
            model.bones().is_err(),
            "a released model should refuse to answer rather than use a stale handle"
        );
        assert!(model.gltf_import_report().is_err());
        assert!(model.cameras().is_err());

        // Idempotent: a second call is a no-op, not a second refusal and not a
        // double free.
        model.release().expect("second release is a no-op");
    });
}


#[test]
fn a_missing_asset_fails_rather_than_answering_an_empty_model() {
    in_child("a_missing_asset_fails_rather_than_answering_an_empty_model", || {
        if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
            return;
        }
        let Some(device) = independent_device_or_skip(|| {
            let parameters = PresentationParameters::new();
            parameters.SetBackBufferWidth(64);
            parameters.SetBackBufferHeight(64);
            GraphicsDevice::new(
                &GraphicsDeviceInformation::new().Adapter(),
                GraphicsProfile::HiDef,
                &parameters,
            )
        }) else {
            return;
        };
        let root = content_root();
        let manager = NativeContentManager::new(&device, root.to_str().expect("utf-8 content root"))
            .expect("native content manager");
        assert!(
            NativeModel::load(&manager, "no-such-asset").is_err(),
            "a missing asset must fail the load, not answer a model with nothing in it"
        );
    });
}


#[test]
fn an_avatar_renderer_draws_a_model_a_game_supplied() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let Some(device) = independent_device_or_skip(|| {
        let parameters = PresentationParameters::new();
        parameters.SetBackBufferWidth(64);
        parameters.SetBackBufferHeight(64);
        GraphicsDevice::new(
            &GraphicsDeviceInformation::new().Adapter(),
            GraphicsProfile::HiDef,
            &parameters,
        )
    }) else {
        return;
    };
    // The renderer takes CNA's engine-layer model, which a build without the
    // engine layer does not have. That is an artifact fact, not a binding one.
    let Ok(model) = SkinnedModel::new() else {
        println!("this artifact has no engine layer, so it has no skinned model");
        return;
    };

    let description = AvatarDescription::CreateRandom().expect("a random avatar");
    let renderer = AvatarRenderer::new(&description).expect("a renderer for it");

    // The colours are the renderer's own state and need no model.
    renderer
        .set_appearance(AvatarAppearance {
            skin: Color::CornflowerBlue,
            hair: Color::Black,
            shirt: Color::White,
            pants: Color::DarkSlateGray,
            shoes: Color::Red,
        })
        .expect("an appearance the renderer keeps");

    // Drawing a clip before a model is named is refused, which is the
    // documented answer and the one that proves the route is not a no-op.
    assert!(
        matches!(
            renderer.draw_clip("Stand0", TimeSpan::Zero, false),
            Err(CnaError::Native { .. })
        ),
        "a renderer with no real model must refuse rather than draw nothing"
    );

    renderer
        .use_model(&device, &model)
        .expect("the renderer takes the game's own model");

    // With a model it reaches the renderer. A build that cannot draw answers a
    // refusal; what must not happen is a Rust-side no-op reported as a draw.
    match renderer.draw_clip("Stand0", TimeSpan::Zero, true) {
        Ok(()) => {}
        Err(CnaError::Native { .. }) => {}
        Err(other) => panic!("unexpected real avatar draw failure: {other:?}"),
    }
}
