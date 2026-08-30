//! CNA's renderer capability reporting for a strict XNA GraphicsDevice.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cna::extensions::graphics::{
    FeatureSupport, FormatUsage, RendererCapabilityExt, RendererFeature, RendererLimit,
    RendererInfoExt, ShaderDialect,
};
use cna::Microsoft::Xna::Framework::Graphics::SurfaceFormat;
use cna::Microsoft::Xna::Framework::{Game, GameContext};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

#[derive(Default)]
struct CapabilityGame {
    state: Arc<GameState>,
    observed: Arc<AtomicBool>,
}

impl GameStateAccess for CapabilityGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for CapabilityGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let device = game.GraphicsDevice()?;

        // Every named feature answers, and the answer is one of CNA's four
        // states rather than a bare boolean: "unknown" is a real answer.
        for feature in RendererFeature::ALL {
            let support = device.feature_support(feature)?;
            assert!(
                !matches!(support, FeatureSupport::Unrecognized(_)),
                "{feature:?} answered with an identity this build does not name: {support:?}",
            );
        }

        // HEADLESS has no 3D pipeline. Whatever it answers, it must not claim
        // support it does not have, and the renderer_info summary must agree
        // with the per-feature answer.
        let summary = RendererInfoExt::renderer_info(&device)?;
        let three_d = device.feature_support(RendererFeature::THREE_DIMENSIONAL_PIPELINE)?;
        assert_eq!(
            summary.supports_3d,
            matches!(support_is_usable(three_d), true),
            "the renderer summary and the per-feature answer disagree about 3D",
        );

        // A limit is Option: a renderer that does not publish one says so
        // rather than reporting a fabricated zero.
        let texture = device.limit(RendererLimit::MAX_TEXTURE_DIMENSION)?;
        assert_eq!(
            texture.map(|value| u32::try_from(value).unwrap_or(u32::MAX)),
            Some(summary.max_texture_dimension),
            "the summary's max texture dimension must be the published limit",
        );
        for limit in RendererLimit::ALL {
            // Asking is always valid; the answer may legitimately be None.
            let _ = device.limit(limit)?;
        }

        // Format support keeps "not asked" and "refused" apart.
        let color = device.format_support(SurfaceFormat::Color)?;
        assert!(
            color.supported.bits() & !color.known.bits() == 0,
            "a renderer cannot support a usage it has no answer for",
        );
        assert!(!color.supports(FormatUsage::ALL) || color.knows(FormatUsage::ALL));

        // The dialect is a named identity, not a free string.
        let dialect = device.shader_dialect()?;
        assert!(
            !matches!(dialect, ShaderDialect::Unrecognized(_)),
            "unnamed shader dialect: {dialect:?}",
        );

        // CNA's own report is text CNA produces, not something assembled here.
        let report = device.capability_report()?;
        assert!(!report.is_empty());

        self.observed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

const fn support_is_usable(support: FeatureSupport) -> bool {
    matches!(
        support,
        FeatureSupport::Supported | FeatureSupport::Restricted
    )
}

#[test]
fn renderer_capabilities_are_reported_by_cna() {
    let game = CapabilityGame::default();
    let observed = Arc::clone(&game.observed);
    run_for_frames(game, 1).expect("one frame reaches LoadContent");
    assert!(observed.load(Ordering::SeqCst), "LoadContent ran");
}
