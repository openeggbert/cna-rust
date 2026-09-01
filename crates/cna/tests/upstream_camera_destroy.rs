//! Reproducer for the camera test backend's dangling platform override.
//!
//! `cna_camera_create_with_test_backend_ext` hands CNA's *global* platform
//! override a raw pointer into the camera resource:
//!
//! ```text
//! CNA::C::Detail::GetPlatformOverride().SetCamera(resource->testService.get());
//! ```
//!
//! `cna_camera_destroy` releases the resource -- freeing the `unique_ptr` that
//! owned that provider -- and never clears the override. Anything that consults
//! the platform camera list afterwards reads freed memory.
//!
//! This runs the sequence in a **child process** on purpose. The failure is a
//! fault, not a result code, so a test that ran it in-process would take the
//! whole suite down and prove nothing repeatable.

#![allow(non_snake_case)]

use std::process::Command;

use cna::extensions::devices::Camera;
use cna::Microsoft::Xna::Framework::{Game, GameContext};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

/// The env var that tells the child which half of the sequence to run.
const STAGE: &str = "CNA_RUST_CAMERA_REPRO_STAGE";

struct CameraGame {
    state: std::sync::Arc<GameState>,
}

impl GameStateAccess for CameraGame {
    fn game_state(&self) -> &std::sync::Arc<GameState> {
        &self.state
    }
}

impl Game for CameraGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let stage = std::env::var(STAGE).unwrap_or_default();
        // Baseline: a test-backend camera, used and then destroyed, with
        // nothing consulting the platform list afterwards.
        let camera = Camera::with_test_backend(game)?;
        camera.set_test_state(cna::extensions::devices::CameraState::Ready)?;
        println!("REPRO: created, state={:?}", camera.state()?);
        camera.release()?;
        println!("REPRO: destroyed");
        if stage == "after-destroy" {
            // The one extra call. It walks Camera::getAvailableCamerasProperty,
            // which consults the override the destroyed resource still owns.
            let count = Camera::count(game)?;
            println!("REPRO: count after destroy = {count}");
        }
        println!("REPRO: survived");
        Ok(())
    }
}

fn run_stage() -> Result<()> {
    let game = CameraGame {
        state: std::sync::Arc::new(GameState::default()),
    };
    run_for_frames(game, 1)
}

#[test]
fn a_destroyed_test_camera_leaves_the_platform_override_dangling() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    if std::env::var(STAGE).is_ok() {
        // We are the child: run the stage and let the fault speak for itself.
        run_stage().expect("the staged camera sequence");
        return;
    }

    let exe = std::env::current_exe().expect("this test binary");
    let mut outcomes = Vec::new();
    for stage in ["baseline", "after-destroy"] {
        let output = Command::new(&exe)
            .args([
                "--test-threads=1",
                "--nocapture",
                "--exact",
                "a_destroyed_test_camera_leaves_the_platform_override_dangling",
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
            "--- stage {stage}: status={status:?} survived={} ---",
            text.contains("REPRO: survived")
        );
        for line in text.lines().filter(|l| l.starts_with("REPRO:")) {
            println!("    {line}");
        }
    }

    let (_, baseline_status, baseline_text) = &outcomes[0];
    assert!(
        baseline_text.contains("REPRO: destroyed"),
        "the baseline created and destroyed a test-backend camera"
    );
    assert!(
        baseline_status.success() && baseline_text.contains("REPRO: survived"),
        "creating and destroying a test camera is itself fine: {baseline_status:?}"
    );

    // The one call that reads the freed provider. This assertion states what
    // was measured; if upstream fixes the dangling override it will fail here
    // and this reproducer becomes the thing that says so.
    let (_, after_status, after_text) = &outcomes[1];
    let survived = after_status.success() && after_text.contains("REPRO: survived");
    if survived {
        println!(
            "NOTE: querying the camera list after destroy no longer faults on this CNA build. \
             If that is a real upstream fix, RUST-UPSTREAM-020 can be retired and the camera \
             family reclassified from BLOCKED_UPSTREAM."
        );
    } else {
        println!(
            "MEASURED: querying the camera list after destroy failed: {after_status:?}. \
             This is RUST-UPSTREAM-020."
        );
    }
    assert!(
        !survived,
        "RUST-UPSTREAM-020 no longer reproduces -- re-measure and reclassify the camera family"
    );
}
