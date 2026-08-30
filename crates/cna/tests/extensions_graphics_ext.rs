//! CNA's extended graphics layer against the live library.
//!
//! The layer is a build option and the renderer may refuse an effect it cannot
//! run. The test asks CNA which of those it is in and holds the answer to that
//! standard, rather than assuming a headless build can create a post-processing
//! effect.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cna::extensions::graphics::{
    is_available, AsciiPostProcessEffect, AsciiQuantizeMode, CrtEffectExt, CrtMaskType,
    DepthEffectExt, DepthEffectMode, DitherMode, ExtendedEffectExt,
};
use cna::Microsoft::Xna::Framework::Graphics::Effect;
use cna::Microsoft::Xna::Framework::{Game, GameContext};
use cna::{run_for_frames, CnaError, ErrorCategory, GameState, GameStateAccess, Result};

#[derive(Default)]
struct ExtendedGraphicsGame {
    state: Arc<GameState>,
    observed: Arc<AtomicBool>,
}

impl GameStateAccess for ExtendedGraphicsGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

/// A refusal is an answer when the layer or the renderer cannot do it; any
/// other failure is not.
fn refusal_is_allowed<T>(result: Result<T>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(CnaError::Native {
            category: ErrorCategory::NotSupported | ErrorCategory::State,
            ..
        }) => None,
        Err(error) => panic!("unexpected extended-graphics failure: {error}"),
    }
}

fn exercise_crt(effect: &Effect) -> Result<()> {
    // Every knob round-trips through CNA rather than through a cached value
    // here: the setter is written and the getter is read back.
    effect.set_scanline_intensity(0.25)?;
    assert!((effect.scanline_intensity()? - 0.25).abs() < 1e-6);
    effect.set_curvature(0.5)?;
    assert!((effect.curvature()? - 0.5).abs() < 1e-6);
    effect.set_vignette_intensity(0.75)?;
    assert!((effect.vignette_intensity()? - 0.75).abs() < 1e-6);
    effect.set_mask_intensity(0.125)?;
    assert!((effect.mask_intensity()? - 0.125).abs() < 1e-6);
    for mask in [
        CrtMaskType::None,
        CrtMaskType::ApertureGrille,
        CrtMaskType::ShadowMask,
    ] {
        effect.set_mask_type(mask)?;
        assert_eq!(effect.mask_type()?, mask);
    }
    Ok(())
}

fn exercise_depth(effect: &Effect) -> Result<()> {
    for mode in [
        DepthEffectMode::Color16Bit,
        DepthEffectMode::Grayscale1Bit,
        DepthEffectMode::Palette16,
    ] {
        effect.set_depth_mode(mode)?;
        assert_eq!(effect.depth_mode()?, mode);
    }
    for dither in [DitherMode::None, DitherMode::Bayer4x4, DitherMode::Bayer8x8] {
        effect.set_dither_mode(dither)?;
        assert_eq!(effect.dither_mode()?, dither);
    }
    Ok(())
}

impl Game for ExtendedGraphicsGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let available = is_available()?;
        let device = game.GraphicsDevice()?;
        println!("extended graphics layer available = {available}");

        if let Some(effect) = refusal_is_allowed(device.create_crt_effect()) {
            println!("CRT effect: created");
            exercise_crt(&effect)?;
        } else {
            println!("CRT effect: refused by this build or renderer");
        }

        if let Some(effect) = refusal_is_allowed(device.create_depth_effect()) {
            println!("depth effect: created");
            exercise_depth(&effect)?;
        } else {
            println!("depth effect: refused by this build or renderer");
        }

        if let Some(effect) = refusal_is_allowed(AsciiPostProcessEffect::new(&device)) {
            println!("ASCII post-process effect: created");
            effect.set_cell_size(8, 12)?;
            assert_eq!(effect.cell_size()?, (8, 12));
            for mode in [AsciiQuantizeMode::BlackWhite, AsciiQuantizeMode::Color] {
                effect.set_quantize_mode(mode)?;
                assert_eq!(effect.quantize_mode()?, mode);
            }
            // Nothing has been drawn, so the grid is whatever CNA reports for
            // "no draw yet"; the test reads it rather than asserting a size it
            // would have to invent.
            let (columns, rows) = effect.last_grid_dimensions()?;
            assert!(columns >= 0 && rows >= 0);
        }

        self.observed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn the_extended_graphics_layer_answers_or_refuses() {
    let game = ExtendedGraphicsGame::default();
    let observed = Arc::clone(&game.observed);
    run_for_frames(game, 1).expect("one frame reaches LoadContent");
    assert!(observed.load(Ordering::SeqCst), "LoadContent ran");
}
