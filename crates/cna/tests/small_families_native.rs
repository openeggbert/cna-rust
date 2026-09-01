//! Native qualification for the Framework, Touch, Storage, and `GamerServices` milestone.

#![allow(clippy::too_many_lines)]

use std::any::TypeId;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use cna::extensions::gamer_services::TakeComponentError;
use cna::extensions::window::WindowHandle;
use cna::Microsoft::Xna::Framework::GamerServices::{GamerServicesComponent, GamerServicesDispatcher};
use cna::Microsoft::Xna::Framework::Graphics::IGraphicsDeviceService;
use cna::Microsoft::Xna::Framework::Input::Touch::{GestureType, TouchPanel};
use cna::Microsoft::Xna::Framework::Input::Keyboard;
use cna::Microsoft::Xna::Framework::Storage::{StorageContainer, StorageDevice};
use cna::Microsoft::Xna::Framework::{
    Game, GameContext, GameTime, GraphicsDeviceManager, IGameComponent,
    IGraphicsDeviceManager, PlayerIndex, PreparingDeviceSettingsEventArgs,
};
use cna::{
    run_for_frames, CnaError, FileMode, GameComponentCollectionExt, GameState, GameStateAccess,
    Result, StorageAsyncState,
};

#[derive(Default)]
struct FrameworkEvidence {
    preparing: AtomicUsize,
    resetting: AtomicUsize,
    reset: AtomicUsize,
    disposed: AtomicUsize,
    keyboard_checked: AtomicUsize,
    self_removed: AtomicUsize,
    touch_checked: AtomicUsize,
}

struct SmallFamilyGame {
    state: Arc<GameState>,
    manager: Option<GraphicsDeviceManager>,
    evidence: Arc<FrameworkEvidence>,
}

#[derive(Default)]
struct EmptyQualificationGame {
    state: Arc<GameState>,
}

impl GameStateAccess for EmptyQualificationGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for EmptyQualificationGame {}

struct PanicPreparingGame {
    state: Arc<GameState>,
    manager: Option<GraphicsDeviceManager>,
}

impl GameStateAccess for PanicPreparingGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for PanicPreparingGame {
    fn Initialize(&mut self, _: &mut GameContext<'_>) -> Result<()> {
        self.manager
            .as_ref()
            .expect("manager constructed before Run")
            .ApplyChanges()
    }
}

impl GameStateAccess for SmallFamilyGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for SmallFamilyGame {
    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let manager = self
            .manager
            .as_ref()
            .expect("manager constructed before Run");
        assert!(!manager.GraphicsDevice()?.IsDisposed()?);
        assert_eq!(
            manager.GraphicsDevice()?.GraphicsProfile()?,
            manager.GraphicsProfile()?
        );

        let capabilities = TouchPanel::GetCapabilities(game)?;
        let touches = TouchPanel::GetState(game)?;
        if !capabilities.IsConnected() {
            assert_eq!(touches.Count(), 0);
            assert!(!touches.IsConnected());
        }
        TouchPanel::SetEnabledGestures(game, GestureType::Tap | GestureType::DoubleTap)?;
        assert_eq!(
            TouchPanel::EnabledGestures(game)?,
            GestureType::Tap | GestureType::DoubleTap
        );
        assert!(!TouchPanel::IsGestureAvailable(game)?);
        assert!(TouchPanel::ReadGesture(game).is_err());
        self.evidence.touch_checked.fetch_add(1, Ordering::SeqCst);

        // XNA's per-player Chatpad overload reaches a real canonical route
        // rather than the refusal it used to return. CNA has one keyboard, so
        // the documented answer is that every slot reports the shared
        // snapshot -- and a slot that disagreed with `GetState` would fail
        // here rather than pass silently.
        let shared = Keyboard::GetState(game)?;
        for player in [
            PlayerIndex::One,
            PlayerIndex::Two,
            PlayerIndex::Three,
            PlayerIndex::Four,
        ] {
            assert_eq!(Keyboard::GetStateWithPlayerIndex(game, player)?, shared);
        }
        assert_eq!(shared.GetPressedKeys().len(), 0);
        self.evidence.keyboard_checked.fetch_add(1, Ordering::SeqCst);

        manager.ApplyChanges()?;
        Ok(())
    }
}

fn native_enabled() -> bool {
    std::env::var_os("CNA_NATIVE_LIBRARY").is_some()
}

fn native_game_guard() -> MutexGuard<'static, ()> {
    static NATIVE_GAME: OnceLock<Mutex<()>> = OnceLock::new();
    NATIVE_GAME
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A game that reports its own window handle from inside a frame.
///
/// `GameWindow::Handle` is only meaningful while the game is bound, so the
/// comparison `GamerServicesComponent` exists to make -- did the dispatcher
/// get *this* window? -- has to be taken during a callback, not after `Run`.
#[derive(Default)]
struct WindowReportingGame {
    state: Arc<GameState>,
    seen: Arc<Mutex<Option<(WindowHandle, WindowHandle)>>>,
}

impl GameStateAccess for WindowReportingGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for WindowReportingGame {
    fn Update(&mut self, _: &mut GameContext<'_>, _: &GameTime) -> Result<()> {
        let pair = (
            self.state.Window().Handle(),
            GamerServicesDispatcher::WindowHandle()?,
        );
        *self
            .seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(pair);
        Ok(())
    }
}

fn seen_handles(seen: &Arc<Mutex<Option<(WindowHandle, WindowHandle)>>>) -> (WindowHandle, WindowHandle) {
    seen.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .expect("the game reported its window from inside a frame")
}

#[test]
fn the_gamer_services_component_hands_the_dispatcher_the_game_window() {
    if !native_enabled() {
        return;
    }
    let _native_game = native_game_guard();

    // The control, which is also what this component used to do: a game with
    // no `GamerServicesComponent` leaves the dispatcher exactly as it found
    // it. Before this milestone the *with*-component case did the same, which
    // is why a game that added the component still got a dispatcher nobody
    // ever initialised or pumped.
    GamerServicesDispatcher::SetWindowHandle(WindowHandle::default())
        .expect("the dispatcher accepts a window handle");
    let without = WindowReportingGame::default();
    let seen_without = Arc::clone(&without.seen);
    run_for_frames(without, 1).expect("a game with no gamer-services component runs");
    let (_, dispatcher_without) = seen_handles(&seen_without);
    assert_eq!(
        dispatcher_without,
        WindowHandle::default(),
        "nothing but the component may hand the dispatcher a window"
    );

    // With it, XNA's `Initialize` runs: the window handle first, then the
    // `InstallingTitleUpdate` subscription, then the dispatcher itself.
    GamerServicesDispatcher::SetWindowHandle(WindowHandle::default())
        .expect("the dispatcher accepts a window handle");
    let with = WindowReportingGame::default();
    let seen_with = Arc::clone(&with.seen);
    let component: Arc<dyn IGameComponent> = Arc::new(GamerServicesComponent::new(&with));
    with.state.Components().Add(component);
    run_for_frames(with, 1).expect("a game with a gamer-services component runs");

    let (window, dispatcher) = seen_handles(&seen_with);
    // HEADLESS has no window and reports handle zero for both sides, so this
    // equality only discriminates on a renderer that has one. The OPENGLES3
    // run is where it does: deleting the component's `SetWindowHandle` leaves
    // the dispatcher on the zero this test seeded and fails here.
    assert_eq!(
        dispatcher, window,
        "the dispatcher must hold the game's own window, not another one"
    );
    assert!(
        GamerServicesDispatcher::IsInitialized().expect("the dispatcher reports its state"),
        "the component initialises the dispatcher, which is what lets an \
         asynchronous gamer-services call ever complete"
    );
    TakeComponentError().expect("the component reported no refusal");
}

#[test]
fn framework_touch_and_gamer_services_use_the_game_owned_runtime() {
    if !native_enabled() {
        return;
    }
    let _native_game = native_game_guard();

    let evidence = Arc::new(FrameworkEvidence::default());
    let mut game = SmallFamilyGame {
        state: Arc::new(GameState::new()),
        manager: None,
        evidence: Arc::clone(&evidence),
    };
    let mut manager = GraphicsDeviceManager::new(&game);
    assert_eq!(manager.PreferredBackBufferWidth().unwrap(), 800);
    assert_eq!(manager.PreferredBackBufferHeight().unwrap(), 480);
    manager.SetPreferredBackBufferWidth(640).unwrap();
    manager.SetPreferredBackBufferHeight(360).unwrap();
    assert!(matches!(
        manager.ApplyChanges(),
        Err(CnaError::UnsupportedRuntime(_))
    ));

    let preparing = Arc::clone(&evidence);
    manager.AddPreparingDeviceSettingsHandler(Box::new(
        move |sender: &dyn std::any::Any, args: PreparingDeviceSettingsEventArgs| {
            assert!(sender.downcast_ref::<GraphicsDeviceManager>().is_some());
            preparing.preparing.fetch_add(1, Ordering::SeqCst);
            let information = args.GraphicsDeviceInformation();
            information.PresentationParameters().SetBackBufferWidth(640);
        },
    ));
    let self_removal_token = Arc::new(AtomicU64::new(0));
    let token_for_callback = Arc::clone(&self_removal_token);
    let self_removed = Arc::clone(&evidence);
    let registration = manager.AddPreparingDeviceSettingsHandler(Box::new(
        move |sender: &dyn std::any::Any, _: PreparingDeviceSettingsEventArgs| {
            let manager = sender
                .downcast_ref::<GraphicsDeviceManager>()
                .expect("GraphicsDeviceManager event sender");
            assert!(manager
                .RemovePreparingDeviceSettingsHandler(token_for_callback.load(Ordering::SeqCst)));
            self_removed.self_removed.fetch_add(1, Ordering::SeqCst);
        },
    ));
    self_removal_token.store(registration, Ordering::SeqCst);
    let resetting = Arc::clone(&evidence);
    manager.AddDeviceResettingHandler(Box::new(
        move |_: &dyn std::any::Any, _: cna::extensions::events::EventArgs| {
            resetting.resetting.fetch_add(1, Ordering::SeqCst);
        },
    ));
    let reset = Arc::clone(&evidence);
    manager.AddDeviceResetHandler(Box::new(
        move |_: &dyn std::any::Any, _: cna::extensions::events::EventArgs| {
            reset.reset.fetch_add(1, Ordering::SeqCst);
        },
    ));
    let disposed = Arc::clone(&evidence);
    manager.AddDisposedHandler(Box::new(
        move |sender: &dyn std::any::Any, _: cna::extensions::events::EventArgs| {
            assert!(sender.downcast_ref::<GraphicsDeviceManager>().is_some());
            disposed.disposed.fetch_add(1, Ordering::SeqCst);
        },
    ));
    manager
        .OnDeviceResetting(&(), cna::extensions::events::EventArgs)
        .unwrap();
    manager
        .OnDeviceReset(&(), cna::extensions::events::EventArgs)
        .unwrap();

    let proposal = manager.FindBestDevice(false).unwrap();
    assert_eq!(proposal.PresentationParameters().BackBufferWidth(), 640);
    assert!(proposal.Clone().Equals(&proposal as &dyn std::any::Any));
    let mut candidates = vec![proposal];
    assert!(matches!(
        manager.RankDevices(&mut candidates),
        Err(CnaError::UnsupportedRuntime(_))
    ));

    assert!(game
        .state
        .Services()
        .GetService(TypeId::of::<dyn IGraphicsDeviceManager>())
        .is_some());
    assert!(game
        .state
        .Services()
        .GetService(TypeId::of::<dyn IGraphicsDeviceService>())
        .is_some());

    let gamer: Arc<dyn IGameComponent> = Arc::new(GamerServicesComponent::new(&game));
    game.state.Components().Add(gamer);
    game.manager = Some(manager);
    run_for_frames(game, 1).expect("Framework manager, Touch, and GamerServices lifecycle");

    assert_eq!(evidence.touch_checked.load(Ordering::SeqCst), 1);
    assert_eq!(evidence.keyboard_checked.load(Ordering::SeqCst), 1);
    assert!(evidence.preparing.load(Ordering::SeqCst) >= 1);
    assert!(evidence.resetting.load(Ordering::SeqCst) >= 1);
    assert!(evidence.reset.load(Ordering::SeqCst) >= 1);
    assert_eq!(evidence.self_removed.load(Ordering::SeqCst), 1);
    assert_eq!(evidence.disposed.load(Ordering::SeqCst), 1);
}

#[test]
fn framework_callback_panic_is_contained_and_game_recreation_succeeds() {
    if !native_enabled() {
        return;
    }
    let _native_game = native_game_guard();

    let mut game = PanicPreparingGame {
        state: Arc::new(GameState::new()),
        manager: None,
    };
    let mut manager = GraphicsDeviceManager::new(&game);
    manager.SetPreferredBackBufferWidth(641).unwrap();
    manager.AddPreparingDeviceSettingsHandler(Box::new(
        |_: &dyn std::any::Any, _: PreparingDeviceSettingsEventArgs| {
            panic!("intentional PreparingDeviceSettings panic");
        },
    ));
    game.manager = Some(manager);
    assert!(matches!(
        run_for_frames(game, 1),
        Err(CnaError::Callback(_))
    ));
    run_for_frames(EmptyQualificationGame::default(), 1)
        .expect("Game recreation after contained framework callback panic");
}

#[test]
fn storage_async_containment_streams_and_disposal_use_cna() {
    if !native_enabled() {
        return;
    }
    let _native_game = native_game_guard();

    let device_changed = StorageDevice::AddDeviceChangedHandler(Box::new(
        |_: &dyn std::any::Any, _: cna::extensions::events::EventArgs| {},
    ));
    assert!(StorageDevice::RemoveDeviceChangedHandler(device_changed));
    assert!(!StorageDevice::RemoveDeviceChangedHandler(device_changed));

    assert!(matches!(
        StorageDevice::BeginShowSelectorWithCallbackAndState(
            Some(Box::new(|_| panic!("intentional selector callback panic"))),
            None,
        ),
        Err(CnaError::Callback(_))
    ));

    let callbacks = Arc::new(AtomicUsize::new(0));
    let callback_count = Arc::clone(&callbacks);
    let state_value = Arc::new(String::from("selector-state"));
    let state: StorageAsyncState = Some(state_value.clone());
    let selector = StorageDevice::BeginShowSelectorWithCallbackAndState(
        Some(Box::new(move |result| {
            callback_count.fetch_add(1, Ordering::SeqCst);
            assert!(result.CompletedSynchronously());
            assert!(result.IsCompleted());
            assert_eq!(
                result
                    .AsyncState()
                    .and_then(|value| value.downcast::<String>().ok())
                    .as_deref()
                    .map(String::as_str),
                Some("selector-state")
            );
        })),
        state,
    )
    .expect("native storage selector");
    assert_eq!(callbacks.load(Ordering::SeqCst), 1);
    let device = StorageDevice::EndShowSelector(&selector).expect("selector End");
    assert!(StorageDevice::EndShowSelector(&selector).is_err());
    assert!(device.IsConnected().expect("storage connection"));
    assert!(device.TotalSpace().expect("total space") >= 0);
    assert!(device.FreeSpace().expect("free space") >= 0);

    let other = StorageDevice::EndShowSelector(
        &StorageDevice::BeginShowSelectorWithCallbackAndState(None, None)
            .expect("second storage selector"),
    )
    .expect("second selector End");
    let name = format!("cna-rust-small-family-{}", std::process::id());
    let open = device
        .BeginOpenContainer(&name, None, None)
        .expect("open storage container");
    assert!(other.EndOpenContainer(&open).is_err());
    let mut container = device.EndOpenContainer(&open).expect("container End");
    assert_eq!(container.DisplayName().expect("display name"), name);
    assert!(container
        .StorageDevice()
        .expect("parent device")
        .IsConnected()
        .unwrap());

    for invalid in ["../escape", "/absolute", "C:\\escape", "a/../../escape"] {
        assert!(container.CreateDirectory(invalid).is_err(), "{invalid}");
        assert!(container.CreateFile(invalid).is_err(), "{invalid}");
    }
    container
        .CreateDirectory("nested")
        .expect("nested directory");
    assert!(container.DirectoryExists("nested").unwrap());
    let mut stream = container
        .CreateFile("nested/data.bin")
        .expect("create native storage stream");
    stream.write_all(b"CNA storage").expect("write stream");
    stream.flush().expect("flush stream");
    stream.seek(SeekFrom::Start(0)).expect("seek stream");
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).expect("read stream");
    assert_eq!(bytes, b"CNA storage");
    assert!(container.FileExists("nested/data.bin").unwrap());
    drop(stream);

    let mut reopened = container
        .OpenFile("nested/data.bin", FileMode::Open)
        .expect("reopen native storage stream");
    assert_eq!(reopened.Length().unwrap(), 11);
    let disposal_count = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&disposal_count);
    container.AddDisposingHandler(Box::new(
        move |sender: &dyn std::any::Any, _: cna::extensions::events::EventArgs| {
            let container = sender
                .downcast_ref::<StorageContainer>()
                .expect("StorageContainer event sender");
            assert!(container.IsDisposed());
            observed.fetch_add(1, Ordering::SeqCst);
        },
    ));
    container.Dispose().expect("container Dispose");
    container.Dispose().expect("idempotent container Dispose");
    assert_eq!(disposal_count.load(Ordering::SeqCst), 1);
    let mut byte = [0_u8; 1];
    assert!(reopened.read(&mut byte).is_err());
    device
        .DeleteContainer(&name)
        .expect("delete test container");

    let panic_name = format!("cna-rust-dispose-panic-{}", std::process::id());
    let result = device
        .BeginOpenContainer(&panic_name, None, None)
        .expect("open callback-panic container");
    let mut panic_container = device.EndOpenContainer(&result).unwrap();
    panic_container.AddDisposingHandler(Box::new(
        |_: &dyn std::any::Any, _: cna::extensions::events::EventArgs| {
            panic!("intentional StorageContainer.Disposing panic");
        },
    ));
    assert!(matches!(
        panic_container.Dispose(),
        Err(CnaError::Callback(_))
    ));
    assert!(panic_container.IsDisposed());
    device.DeleteContainer(&panic_name).unwrap();
}
