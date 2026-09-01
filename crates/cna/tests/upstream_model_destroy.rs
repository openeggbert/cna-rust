//! Reproducer for the fault in a content-loaded `Model`'s teardown.
//!
//! `MeshResource::~MeshResource` hands each part back its standalone copy:
//!
//! ```text
//! part->parentMesh = nullptr;
//! part->value = std::move(part->detachedValue);
//! ```
//!
//! A hand-built part has a `detachedValue` and survives that. A content-loaded
//! part does not -- `MirrorLoadedModel` fills only `value`, with an aliasing
//! pointer into the model -- so the move assigns an **empty** `shared_ptr` over
//! a good one. `~PartResource` then dereferences it two lines later, without
//! the null check its own next line applies to `detachedValue`.
//!
//! This runs in a **child process** on purpose: the failure is a fault, not a
//! result code. Two stages run here:
//!
//! * `destroy` releases the model and does not come back.
//! * `leak` never releases it, reaches the end of its work, and *still* faults
//!   -- the C API's handle registry runs the same destructor at process exit.
//!   That stage is why nothing in the binding guards the teardown: there is no
//!   ordering on this side that avoids the fault, only one that hides it.
//!
//! The control that makes *content-loaded* rather than *has a mesh part* the
//! answer is `tools/reproducers/ext015g_handbuilt_mesh.c`: the same shape built by
//! hand destroys cleanly, because a hand-built part has the `detachedValue`
//! this one lacks. It stays in C because the model-construction routes are a
//! deliberate non-binding -- `crate::graphics::Model` is the Rust way to build
//! a model.
//!
//! Full write-up: `RUST-UPSTREAM-021` in `docs/upstream-findings.md`.

use std::path::{Path, PathBuf};
use std::process::Command;

use cna::extensions::content::NativeContentManager;
use cna::extensions::native_model::NativeModel;
use cna::Microsoft::Xna::Framework::Graphics::{
    GraphicsDevice, GraphicsProfile, PresentationParameters,
};
use cna::Microsoft::Xna::Framework::GraphicsDeviceInformation;
use cna::{CnaError, ErrorCategory, Result};

const STAGE: &str = "CNA_RUST_MODEL_DESTROY_STAGE";

/// One triangle under two nodes.
///
/// A glTF with no mesh at all is not a usable control: CNA's importer refuses
/// it outright -- "contains no mesh instances to import" -- so there is no such
/// thing as a loaded model without a part to compare against. The control is
/// the hand-built model in the C probe instead.
const ONE_PART: &str = r#"{
  "asset": { "version": "2.0" }, "scene": 0,
  "scenes": [ { "nodes": [ 0 ] } ],
  "nodes": [ { "name": "Root", "children": [ 1 ], "mesh": 0 }, { "name": "Child" } ],
  "meshes": [ { "name": "Triangle",
                "primitives": [ { "attributes": { "POSITION": 0 } } ] } ],
  "accessors": [ { "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3",
                   "min": [ 0.0, 0.0, 0.0 ], "max": [ 1.0, 1.0, 0.0 ] } ],
  "bufferViews": [ { "buffer": 0, "byteOffset": 0, "byteLength": 36 } ],
  "buffers": [ { "byteLength": 36, "uri": "data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAA" } ] }"#;

fn content_root() -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("model-destroy-repro");
    std::fs::create_dir_all(&root).expect("content root");
    std::fs::write(root.join("onepart.gltf"), ONE_PART).expect("write the one-part asset");
    root
}

fn host() -> Option<(GraphicsDevice, NativeContentManager)> {
    let parameters = PresentationParameters::new();
    parameters.SetBackBufferWidth(64);
    parameters.SetBackBufferHeight(64);
    let device = match GraphicsDevice::new(
        &GraphicsDeviceInformation::new().Adapter(),
        GraphicsProfile::HiDef,
        &parameters,
    ) {
        Ok(device) => device,
        Err(CnaError::Native {
            category: ErrorCategory::Platform,
            ref message,
            ..
        }) if message.contains("platform window id") => {
            println!("REPRO: skip -- this renderer needs a window: {message}");
            return None;
        }
        Err(error) => panic!("independent GraphicsDevice construction failed: {error}"),
    };
    let root = content_root();
    let manager = NativeContentManager::new(&device, root.to_str().expect("utf-8 root"))
        .expect("native content manager");
    Some((device, manager))
}

fn run_stage(stage: &str) -> Result<()> {
    let Some((_device, manager)) = host() else {
        return Ok(());
    };
    let model = NativeModel::load(&manager, "onepart")?;
    let meshes = model.mesh_count()?;
    let parts: u64 = model
        .meshes()?
        .iter()
        .map(|mesh| mesh.part_count().unwrap_or(0))
        .sum();
    println!("REPRO: loaded: {meshes} mesh(es), {parts} part(s)");
    if stage == "leak" {
        // Never released. The fault still comes, at process exit, from the same
        // destructor -- which is the point of this stage.
        core::mem::forget(model);
        println!("REPRO: leaked");
        return Ok(());
    }
    model.release()?;
    println!("REPRO: destroyed");
    Ok(())
}

#[test]
fn destroying_a_content_loaded_model_with_a_mesh_part_faults() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    if let Ok(stage) = std::env::var(STAGE) {
        run_stage(&stage).expect("the staged load and destroy");
        println!("REPRO: survived {stage}");
        return;
    }

    let exe = std::env::current_exe().expect("this test binary");
    let mut outcomes = Vec::new();
    for stage in ["destroy", "leak"] {
        let output = Command::new(&exe)
            .args([
                "--test-threads=1",
                "--nocapture",
                "--exact",
                "destroying_a_content_loaded_model_with_a_mesh_part_faults",
            ])
            .env(STAGE, stage)
            .output()
            .expect("run the staged child");
        let text = String::from_utf8_lossy(&output.stdout).into_owned()
            + &String::from_utf8_lossy(&output.stderr);
        outcomes.push((stage, output.status, text));
    }

    for (stage, status, text) in &outcomes {
        println!(
            "--- stage {stage}: status={status:?}, {} bytes captured ---",
            text.len()
        );
        for line in text.lines().filter(|line| line.contains("REPRO:")) {
            println!("    {}", line.trim());
        }
    }

    let (_, destroy_status, destroy_text) = &outcomes[0];
    if destroy_text.contains("REPRO: skip") {
        println!("SKIP: this renderer cannot make a windowless device");
        return;
    }
    assert!(
        destroy_text.contains("REPRO: loaded"),
        "the asset must load before its teardown can be measured"
    );

    // Stage one: releasing the model does not come back.
    let destroy_survived =
        destroy_status.success() && destroy_text.contains("REPRO: survived");

    // Stage two: not releasing it does not help. Reaching "leaked" and then
    // failing anyway is the whole finding -- the fault moves to process exit
    // rather than going away.
    let (_, leak_status, leak_text) = &outcomes[1];
    let leaked_then_faulted =
        leak_text.contains("REPRO: leaked") && !leak_status.success();
    if leaked_then_faulted {
        println!(
            "MEASURED: the model was never released and the process still failed: \
             {leak_status:?}. The handle registry runs the same destructor at exit, \
             which is why the binding does not guard the teardown."
        );
    }

    if destroy_survived {
        println!(
            "NOTE: destroying a content-loaded model with a mesh part no longer faults on \
             this CNA build. If that is a real upstream fix, RUST-UPSTREAM-021 can be \
             retired and `NativeModel`'s teardown warning removed."
        );
    } else {
        println!(
            "MEASURED: destroying a content-loaded model with one mesh part failed: \
             {destroy_status:?}. This is RUST-UPSTREAM-021."
        );
    }
    assert!(
        !destroy_survived,
        "RUST-UPSTREAM-021 no longer reproduces -- re-measure and retire the finding"
    );
    assert!(
        leaked_then_faulted,
        "leaking the model was expected to move the fault to process exit, not to avoid \
         it. If it is avoided now, the binding could guard the teardown after all -- \
         re-measure RUST-UPSTREAM-021. status={leak_status:?}"
    );
}
