//! CNA's process-global runtime identity and renderer selection.
//!
//! Everything measured here is process-global upstream, so the whole family is
//! exercised in one test: a second test in this binary would observe state the
//! first one latched.

use std::sync::Arc;

use cna::extensions::runtime::{
    active_renderer, automatic_renderer_fallback, available_renderers, current_backend_category,
    current_backend_maturity, current_renderer, desktop_operating_system, platform,
    platform_is_apple, platform_is_mobile, platform_name, renderer_fallbacks,
    renderer_selection_is_latched, selected_renderer, set_automatic_renderer_fallback,
    set_preferred_renderer, set_preferred_renderer_by_name, set_renderer_fallback_chain,
    BackendCategory, BackendMaturity, DesktopOperatingSystem, Platform, RendererFallbackReason,
    RendererType,
};
use cna::Microsoft::Xna::Framework::Game;
use cna::{run_for_frames, CnaError, GameState, GameStateAccess};

#[derive(Default)]
struct LatchGame {
    state: Arc<GameState>,
}

impl GameStateAccess for LatchGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for LatchGame {}

#[test]
fn runtime_identity_and_renderer_selection() {
    // --- platform identity -------------------------------------------------
    let platform = platform().expect("CNA reports a platform");
    let name = platform_name().expect("CNA names its platform");
    assert!(!name.is_empty());
    let is_apple = platform_is_apple().expect("CNA answers is-apple");
    let is_mobile = platform_is_mobile().expect("CNA answers is-mobile");
    match platform {
        Platform::Desktop => assert!(!is_mobile),
        Platform::Android => assert!(is_mobile && !is_apple),
        Platform::Ios => assert!(is_mobile && is_apple),
        _ => {}
    }
    let operating_system = desktop_operating_system().expect("CNA reports a desktop OS");
    if operating_system == DesktopOperatingSystem::MacOsX {
        assert!(is_apple);
    }
    if operating_system == DesktopOperatingSystem::Linux {
        assert!(!is_apple);
    }

    // --- renderer identity -------------------------------------------------
    let current = current_renderer().expect("CNA reports its renderer");
    let current_name = current.name().expect("CNA names the running renderer");
    assert!(!current_name.is_empty());
    assert_eq!(
        RendererType::parse(&current_name).expect("CNA parses its own spelling"),
        Some(current),
        "a renderer's own name must round-trip through CNA's parser",
    );
    assert_eq!(RendererType::parse("no-such-renderer-identity"), Ok(None));

    let available = available_renderers().expect("CNA lists compiled-in renderers");
    assert!(!available.is_empty());
    assert!(available.contains(&current));
    for renderer in &available {
        assert!(renderer.is_available().expect("availability is answerable"));
    }
    // Vulkan exists as an identity in every build; whether it is compiled in is
    // a build fact, and CNA answers it either way rather than failing.
    let vulkan = RendererType::VULKAN.is_available().expect("availability is answerable");
    assert_eq!(vulkan, available.contains(&RendererType::VULKAN));

    // Only the running renderer has a canonical name route.
    if current != RendererType::PIXIJS {
        assert!(matches!(
            RendererType::PIXIJS.name(),
            Err(CnaError::UnsupportedRuntime(_))
        ));
    }

    // --- backend classification -------------------------------------------
    assert_eq!(
        current.category().expect("a renderer has a category"),
        current_backend_category().expect("the running renderer has a category"),
    );
    assert_eq!(
        current.maturity().expect("a renderer has a maturity"),
        current_backend_maturity().expect("the running renderer has a maturity"),
    );
    for category in [
        BackendCategory::Native,
        BackendCategory::TranslationLayer,
        BackendCategory::Software,
        BackendCategory::Web,
        BackendCategory::Diagnostic,
    ] {
        assert!(!category.name().expect("a category is named").is_empty());
    }
    for maturity in [
        BackendMaturity::Production,
        BackendMaturity::Supported,
        BackendMaturity::Experimental,
        BackendMaturity::Historical,
        BackendMaturity::Deprecated,
    ] {
        assert!(!maturity.name().expect("a maturity is named").is_empty());
    }
    for reason in [
        RendererFallbackReason::NotCompiledIn,
        RendererFallbackReason::ProbeUnavailable,
        RendererFallbackReason::InitializationFailed,
        RendererFallbackReason::WindowKindConflict,
    ] {
        assert!(!reason.name().expect("a reason is named").is_empty());
    }

    // --- selection before anything is created ------------------------------
    assert_eq!(renderer_selection_is_latched(), Ok(false));
    // CNA refuses to guess which renderer is running before one exists.
    assert!(matches!(active_renderer(), Err(CnaError::Native { code: 3, .. })));

    set_preferred_renderer(current).expect("the preferred renderer may be set before latching");
    assert_eq!(selected_renderer(), Ok(current));
    set_preferred_renderer_by_name(&current_name).expect("the same choice by name");
    assert_eq!(selected_renderer(), Ok(current));

    set_renderer_fallback_chain(&available).expect("a fallback chain of compiled-in renderers");
    set_renderer_fallback_chain(&[]).expect("an empty fallback chain is a valid choice");
    set_automatic_renderer_fallback(true).expect("automatic fallback may be enabled");
    assert_eq!(automatic_renderer_fallback(), Ok(true));
    set_automatic_renderer_fallback(false).expect("automatic fallback may be disabled");
    assert_eq!(automatic_renderer_fallback(), Ok(false));

    // --- latching ----------------------------------------------------------
    run_for_frames(LatchGame::default(), 1).expect("one frame creates a renderer");
    assert_eq!(renderer_selection_is_latched(), Ok(true));
    assert_eq!(active_renderer(), Ok(current));
    // Once latched the choice is fixed, and CNA says so rather than accepting
    // a change it cannot honour.
    assert!(matches!(
        set_preferred_renderer(current),
        Err(CnaError::Native { code: 3, .. })
    ));

    // The fallback history is CNA's own account of what it tried; on a build
    // whose first choice worked it is empty, and no record is invented.
    for record in renderer_fallbacks().expect("the fallback history is readable") {
        assert!(available.contains(&record.renderer) || !record.message.is_empty());
        assert!(!record.reason.name().expect("a reason is named").is_empty());
    }
}
