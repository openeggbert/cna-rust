//! What a content manager can see, and CNA's substitute touch panel.
//!
//! The manifest is the interesting half: it says which assets exist, what form
//! each is in, and -- through the reader usage -- whether this *build* can load
//! them. That last question is the one a packaging step needs and the one no
//! `Result` answers, because a file whose reader is missing loads fine on the
//! machine that built it.
//!
//! The touch panel is here for the same reason the sensors' backends are: no
//! machine this runs on has a touchscreen, so without it the whole touch
//! projection could only ever be exercised against "no touch device".

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use cna::extensions::content::{AssetTypeId, CnbWriter, NativeContentManager};
use cna::Microsoft::Xna::Framework::Graphics::{
    GraphicsDevice, GraphicsProfile, PresentationParameters,
};
use cna::Microsoft::Xna::Framework::Input::Touch::{
    TouchLocationState, TouchPanel, TouchPanelTestBackend,
};
use cna::Microsoft::Xna::Framework::{
    Game, GameContext, GraphicsDeviceInformation, Vector2,
};
use cna::{run_for_frames, CnaError, ErrorCategory, GameState, GameStateAccess, Result};

/// A content root with one real `.cnb` in it, written by the test.
fn content_root() -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("manifest-content");
    std::fs::create_dir_all(&root).expect("content root");
    let asset_type = AssetTypeId::custom("CnaRust.Test.Manifest").expect("a custom type");
    let writer = CnbWriter::new(asset_type, 1).expect("a writer");
    writer
        .set_metadata("CnaRust.Test.Manifest", "listed")
        .expect("metadata");
    let bytes = writer.build().expect("build");
    std::fs::write(root.join("listed.cnb"), bytes).expect("write the asset");
    root
}

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
             the renderer's no-window refusal: {error:?}"
        ),
    }
}

#[test]
fn a_manager_can_say_what_it_can_see_and_what_it_can_read() {
    let Some(device) = device() else { return };
    let root = content_root();
    let manager = NativeContentManager::new(&device, root.to_str().expect("utf-8 root"))
        .expect("a content manager");

    assert_eq!(
        manager.root_directory().expect("the root"),
        root.to_str().expect("utf-8 root"),
        "the manager should report the root it was made with"
    );

    manager
        .refresh_content_manifest()
        .expect("read the content root");
    let count = manager.manifest_entry_count().expect("a manifest count");
    println!("NOTE: the manifest holds {count} entr(ies)");
    let mut paths = Vec::new();
    for index in 0..count {
        let entry = manager.manifest_entry(index).expect("a manifest entry");
        println!("NOTE: {entry:?}");
        paths.push(entry.relative_path);
    }
    assert!(
        paths.iter().any(|path| path.contains("listed")),
        "the asset the test wrote should be in the manifest, got {paths:?}"
    );

    // Two names that normalise to one key are one asset. The normalisation is
    // exactly two rules -- backslashes to forward slashes, then lowercase --
    // and it deliberately does not resolve a path, which is why `./listed` is
    // a *different* asset from `listed`. Measured, because the first version
    // of this test assumed otherwise.
    let windows = manager.normalized_key("Textures\\Hero").expect("a key");
    let posix = manager.normalized_key("textures/hero").expect("a key");
    println!("NOTE: 'Textures\\Hero' -> {windows:?}, 'textures/hero' -> {posix:?}");
    assert_eq!(
        windows, posix,
        "a Windows and a POSIX spelling of one name must be one cache key"
    );

    let plain = manager.normalized_key("listed").expect("a key");
    let dotted = manager.normalized_key("./listed").expect("a key");
    println!("NOTE: 'listed' -> {plain:?}, './listed' -> {dotted:?}");
    assert_ne!(
        plain, dotted,
        "normalisation does not resolve a path, so these are two cache entries"
    );

    // The question a packaging step has and no result code answers: will this
    // build's readers serve this content?
    let readers = manager
        .xnb_reader_usage_count()
        .expect("a reader-usage count");
    for index in 0..readers {
        let usage = manager.xnb_reader_usage(index).expect("one reader usage");
        println!("NOTE: reader {:?}", usage);
        assert!(
            !usage.name.is_empty(),
            "a reader-usage entry must name a reader"
        );
    }

    // Changing the root changes what a name resolves to, which is the whole
    // point of the setter.
    manager
        .set_root_directory("/nonexistent")
        .expect("set a new root");
    assert_eq!(manager.root_directory().expect("the new root"), "/nonexistent");
    manager
        .set_root_directory(root.to_str().expect("utf-8 root"))
        .expect("restore the root");

    // The device the manager loads onto must be the one it was made with.
    assert_ne!(
        manager
            .graphics_device_identity()
            .expect("a device identity"),
        0,
        "a manager made on a device should name one"
    );

    manager.unload().expect("dropping the cache is always allowed");
}

#[derive(Debug, Default)]
struct TouchObserved {
    exists_before: bool,
    exists_after: bool,
    emulation_round_trip: Option<(bool, bool)>,
    touches_after_event: Option<usize>,
    notes: Vec<String>,
}

#[derive(Default)]
struct TouchGame {
    state: Arc<GameState>,
    observed: Arc<Mutex<TouchObserved>>,
}

impl GameStateAccess for TouchGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for TouchGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let mut observed = TouchObserved::default();

        TouchPanelTestBackend::reset(game)?;
        observed.exists_before = TouchPanelTestBackend::touch_device_exists(game)?;

        // The switch that turns the whole family from "unsupported" into
        // something to measure.
        TouchPanelTestBackend::set_touch_device_exists(game, true)?;
        observed.exists_after = TouchPanelTestBackend::touch_device_exists(game)?;

        let before = TouchPanelTestBackend::mouse_touch_emulation_enabled(game)?;
        TouchPanelTestBackend::set_mouse_touch_emulation_enabled(game, !before)?;
        let after = TouchPanelTestBackend::mouse_touch_emulation_enabled(game)?;
        observed.emulation_round_trip = Some((before, after));
        TouchPanelTestBackend::set_mouse_touch_emulation_enabled(game, before)?;

        // One finger down, one frame, and then read what the panel says --
        // through the ordinary `TouchPanel::GetState`, not a back door.
        TouchPanelTestBackend::set_finger(game, 0, 0, Vector2 { X: 0.5, Y: 0.25 })?;
        TouchPanelTestBackend::raise_touch_event(
            game,
            7,
            TouchLocationState::Pressed,
            120.0,
            80.0,
            1.0,
            0.0,
        )?;
        TouchPanelTestBackend::update(game)?;
        let state = TouchPanel::GetState(game)?;
        observed.touches_after_event = Some(state.Count() as usize);
        for index in 0..state.Count() {
            observed.notes.push(format!("touch {index}: {:?}", state.Item(index)));
        }

        TouchPanelTestBackend::reset(game)?;
        observed
            .notes
            .push(format!("after reset: {} touch(es)", TouchPanel::GetState(game)?.Count()));

        *self
            .observed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = observed;
        Ok(())
    }
}

#[test]
fn the_substitute_touch_panel_makes_a_touch_measurable() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let observed = Arc::new(Mutex::new(TouchObserved::default()));
    let game = TouchGame {
        state: Arc::new(GameState::default()),
        observed: Arc::clone(&observed),
    };
    run_for_frames(game, 1).expect("one frame driving the touch panel");
    let observed = observed
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for note in &observed.notes {
        println!("NOTE: {note}");
    }
    println!(
        "NOTE: device exists {} -> {}, emulation {:?}, touches {:?}",
        observed.exists_before,
        observed.exists_after,
        observed.emulation_round_trip,
        observed.touches_after_event
    );

    assert!(
        !observed.exists_before,
        "after a reset the panel should report no touch device"
    );
    assert!(
        observed.exists_after,
        "the backend's switch should make a touch device exist"
    );
    if let Some((before, after)) = observed.emulation_round_trip {
        assert_ne!(
            before, after,
            "the mouse-emulation flag must read back as it was set"
        );
    }
    assert!(
        observed.touches_after_event.is_some(),
        "the panel should answer a state once a device exists"
    );
}
