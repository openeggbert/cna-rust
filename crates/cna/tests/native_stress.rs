//! Crash-isolated ownership and callback stress for an explicitly supplied CNA library.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::float_cmp,
    clippy::needless_pass_by_value,
    clippy::too_many_lines
)]

use std::any::TypeId;
use std::fs;
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use cna::extensions::graphics::{
    EffectAnnotationCollectionExt, EffectFactoryExt, EffectParameterCollectionExt,
    EffectPassCollectionExt, EffectTechniqueCollectionExt, FloatClearExt, ModelCollectionExt,
};
use cna::Microsoft::Xna::Framework::GamerServices::GamerServicesComponent;
use cna::Microsoft::Xna::Framework::Graphics::{
    AlphaTestEffect, BasicEffect, BlendState, BufferUsage, ClearOptions, CompareFunction,
    CubeMapFace,
    DepthFormat, DepthStencilState, DualTextureEffect, DynamicIndexBuffer, DynamicVertexBuffer,
    Effect,
    EffectMaterial, EffectParameterClass, EffectParameterType, EnvironmentMapEffect,
    GraphicsAdapter, GraphicsDevice, GraphicsDeviceStatus, GraphicsProfile, GraphicsResource,
    IndexBuffer, IndexElementSize, Model, ModelBone, OcclusionQuery, PrimitiveType,
    PresentationParameters, RasterizerState, RenderTarget2D, RenderTargetBinding, RenderTargetCube,
    SamplerState,
    SetDataOptions, SkinnedEffect, SpriteBatch, SpriteFont, SpriteSortMode, SurfaceFormat, Texture,
    Texture2D, Texture3D, TextureCube, VertexBuffer, VertexBufferBinding, VertexDeclaration,
    VertexElement, VertexElementFormat, VertexElementUsage, VertexPositionColor,
};
use cna::Microsoft::Xna::Framework::Input::Touch::{GestureType, TouchPanel};
use cna::Microsoft::Xna::Framework::Storage::StorageDevice;
use cna::Microsoft::Xna::Framework::{
    Color, Game, GameContext, GameTime, GraphicsDeviceInformation, GraphicsDeviceManager, IDrawable,
    IGameComponent,
    IUpdateable, Matrix, PreparingDeviceSettingsEventArgs, Rectangle, Vector2, Vector3, Vector4,
};
use cna::{
    run_for_frames, CnaError, EffectAnnotationDescriptor, EffectParameterDescriptor,
    EffectTechniqueDescriptor, ErrorCategory, GameComponentCollectionExt, GameComponentRuntime,
    GameState, GameStateAccess, Result, VertexData,
};

const CHILD_CASE: &str = "CNA_RUST_NATIVE_STRESS_CHILD";

#[derive(Default)]
struct EmptyGame {
    state: Arc<GameState>,
}

impl GameStateAccess for EmptyGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for EmptyGame {}

struct SmallFamilyStressGame {
    state: Arc<GameState>,
    manager: Option<GraphicsDeviceManager>,
}

impl GameStateAccess for SmallFamilyStressGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for SmallFamilyStressGame {
    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let _ = TouchPanel::GetCapabilities(game)?;
        let touches = TouchPanel::GetState(game)?;
        TouchPanel::SetEnabledGestures(game, GestureType::Tap | GestureType::DoubleTap)?;
        assert!(touches.Count() <= 8);
        self.manager
            .as_ref()
            .expect("small-family manager")
            .ApplyChanges()
    }
}

fn small_family_stress_game() -> SmallFamilyStressGame {
    let mut game = SmallFamilyStressGame {
        state: Arc::new(GameState::new()),
        manager: None,
    };
    let mut manager = GraphicsDeviceManager::new(&game);
    manager.SetPreferredBackBufferWidth(640).unwrap();
    manager.SetPreferredBackBufferHeight(360).unwrap();
    manager.AddPreparingDeviceSettingsHandler(Box::new(
        |sender: &dyn std::any::Any, args: PreparingDeviceSettingsEventArgs| {
            assert!(sender.downcast_ref::<GraphicsDeviceManager>().is_some());
            args.GraphicsDeviceInformation()
                .PresentationParameters()
                .SetBackBufferWidth(640);
        },
    ));
    let gamer: Arc<dyn IGameComponent> = Arc::new(GamerServicesComponent::new(&game));
    game.state.Components().Add(gamer);
    game.manager = Some(manager);
    game
}

struct PanicPreparingStressGame {
    state: Arc<GameState>,
    manager: Option<GraphicsDeviceManager>,
}

impl GameStateAccess for PanicPreparingStressGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for PanicPreparingStressGame {
    fn Initialize(&mut self, _: &mut GameContext<'_>) -> Result<()> {
        self.manager
            .as_ref()
            .expect("panic-stress manager")
            .ApplyChanges()
    }
}

fn panic_preparing_stress_game() -> PanicPreparingStressGame {
    let mut game = PanicPreparingStressGame {
        state: Arc::new(GameState::new()),
        manager: None,
    };
    let mut manager = GraphicsDeviceManager::new(&game);
    manager.SetPreferredBackBufferWidth(641).unwrap();
    manager.AddPreparingDeviceSettingsHandler(Box::new(
        |_: &dyn std::any::Any, _: PreparingDeviceSettingsEventArgs| {
            panic!("intentional crash-isolated PreparingDeviceSettings panic");
        },
    ));
    game.manager = Some(manager);
    game
}

fn storage_stress_cycle(index: usize) {
    let result = StorageDevice::BeginShowSelectorWithCallbackAndState(None, None)
        .expect("storage selector stress");
    let device = StorageDevice::EndShowSelector(&result).expect("storage selector End stress");
    let name = format!("cna-rust-native-stress-{}-{index}", std::process::id());
    let open = device
        .BeginOpenContainer(&name, None, None)
        .expect("storage container stress");
    let mut container = device
        .EndOpenContainer(&open)
        .expect("storage container End stress");
    let retained_device = container
        .StorageDevice()
        .expect("retained storage device")
        .clone();
    container.CreateDirectory("nested").unwrap();
    let mut stream = container.CreateFile("nested/value.bin").unwrap();
    stream.write_all(b"storage stress").unwrap();
    if index == 0 {
        stream = std::thread::spawn(move || {
            assert!(stream.Close().is_err());
            stream
        })
        .join()
        .expect("wrong-thread stream close is contained");
        stream.Close().expect("owner-thread stream close retry");
        container = std::thread::spawn(move || {
            assert!(container.Dispose().is_err());
            container
        })
        .join()
        .expect("wrong-thread container Dispose is contained");
    }
    drop(device);
    container
        .Dispose()
        .expect("container-before-stream shutdown");
    assert!(stream.write_all(b"closed").is_err());
    container.Dispose().expect("double container Dispose");
    retained_device.DeleteContainer(&name).unwrap();
}

#[derive(Default)]
struct PanicGame {
    state: Arc<GameState>,
}

impl GameStateAccess for PanicGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for PanicGame {
    fn Update(&mut self, game: &mut GameContext<'_>, time: &GameTime) -> Result<()> {
        let _ = (game, time);
        panic!("intentional callback stress panic")
    }
}

#[derive(Default)]
struct PanicEventGame {
    state: Arc<GameState>,
}

impl GameStateAccess for PanicEventGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for PanicEventGame {
    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let _ = game;
        self.AddExitingHandler(Box::new(|_: &dyn std::any::Any, _| {
            panic!("intentional lifecycle-event panic");
        }));
        Ok(())
    }
}

struct SuppressFirstDrawGame {
    state: Arc<GameState>,
    begin_draws: Arc<AtomicUsize>,
    draws: Arc<AtomicUsize>,
    end_draws: Arc<AtomicUsize>,
}

impl GameStateAccess for SuppressFirstDrawGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for SuppressFirstDrawGame {
    fn BeginDraw(&mut self) -> bool {
        self.begin_draws.fetch_add(1, Ordering::SeqCst) != 0
    }

    fn Draw(&mut self, game: &mut GameContext<'_>, time: &GameTime) -> Result<()> {
        let _ = (game, time);
        self.draws.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn EndDraw(&mut self) {
        self.end_draws.fetch_add(1, Ordering::SeqCst);
    }
}

struct RecordingComponent {
    name: &'static str,
    game: Weak<GameState>,
    log: Arc<Mutex<Vec<String>>>,
    update_order: i32,
    draw_order: i32,
    remove_during_update: Mutex<Option<Weak<dyn IGameComponent>>>,
}

impl RecordingComponent {
    fn record(&self, suffix: &str) {
        self.log
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(format!("{}.{}", self.name, suffix));
    }
}

impl GameComponentRuntime for RecordingComponent {
    fn AsUpdateable(&self) -> Option<&dyn IUpdateable> {
        Some(self)
    }

    fn AsDrawable(&self) -> Option<&dyn IDrawable> {
        Some(self)
    }
}

impl IGameComponent for RecordingComponent {
    fn Initialize(&self) {
        self.record("Initialize");
    }
}

impl IUpdateable for RecordingComponent {
    fn Enabled(&self) -> bool {
        true
    }

    fn UpdateOrder(&self) -> i32 {
        self.update_order
    }

    fn AddEnabledChangedHandler(
        &self,
        handler: Box<dyn cna::extensions::events::EventHandler>,
    ) -> u64 {
        drop(handler);
        0
    }

    fn RemoveEnabledChangedHandler(&self, registration: u64) -> bool {
        let _ = registration;
        false
    }

    fn AddUpdateOrderChangedHandler(
        &self,
        handler: Box<dyn cna::extensions::events::EventHandler>,
    ) -> u64 {
        drop(handler);
        0
    }

    fn RemoveUpdateOrderChangedHandler(&self, registration: u64) -> bool {
        let _ = registration;
        false
    }

    fn Update(&self, game_time: &GameTime) {
        let _ = game_time;
        self.record("Update");
        let target = self
            .remove_during_update
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .and_then(|target| target.upgrade());
        if let (Some(game), Some(target)) = (self.game.upgrade(), target) {
            assert!(game.Components().Remove(&target));
        }
    }
}

impl IDrawable for RecordingComponent {
    fn Visible(&self) -> bool {
        true
    }

    fn DrawOrder(&self) -> i32 {
        self.draw_order
    }

    fn AddVisibleChangedHandler(
        &self,
        handler: Box<dyn cna::extensions::events::EventHandler>,
    ) -> u64 {
        drop(handler);
        0
    }

    fn RemoveVisibleChangedHandler(&self, registration: u64) -> bool {
        let _ = registration;
        false
    }

    fn AddDrawOrderChangedHandler(
        &self,
        handler: Box<dyn cna::extensions::events::EventHandler>,
    ) -> u64 {
        drop(handler);
        0
    }

    fn RemoveDrawOrderChangedHandler(&self, registration: u64) -> bool {
        let _ = registration;
        false
    }

    fn Draw(&self, game_time: &GameTime) {
        let _ = game_time;
        self.record("Draw");
    }
}

struct ComponentOrderGame {
    state: Arc<GameState>,
    log: Arc<Mutex<Vec<String>>>,
    add_after_initialize: AtomicBool,
}

impl GameStateAccess for ComponentOrderGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for ComponentOrderGame {
    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let _ = game;
        self.log
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push("Game.Initialize".to_owned());
        Ok(())
    }

    fn Update(&mut self, game: &mut GameContext<'_>, time: &GameTime) -> Result<()> {
        let _ = (game, time);
        self.log
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push("Game.Update".to_owned());
        if !self.add_after_initialize.swap(true, Ordering::AcqRel) {
            let component: Arc<dyn IGameComponent> = Arc::new(RecordingComponent {
                name: "D",
                game: Arc::downgrade(&self.state),
                log: Arc::clone(&self.log),
                update_order: 0,
                draw_order: 0,
                remove_during_update: Mutex::new(None),
            });
            self.state.Components().Add(component);
        }
        Ok(())
    }

    fn Draw(&mut self, game: &mut GameContext<'_>, time: &GameTime) -> Result<()> {
        let _ = (game, time);
        self.log
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push("Game.Draw".to_owned());
        Ok(())
    }
}

fn component_order_game(log: &Arc<Mutex<Vec<String>>>) -> ComponentOrderGame {
    let state = Arc::new(GameState::new());
    let component = |name, update_order, draw_order| {
        Arc::new(RecordingComponent {
            name,
            game: Arc::downgrade(&state),
            log: Arc::clone(log),
            update_order,
            draw_order,
            remove_during_update: Mutex::new(None),
        })
    };
    let a = component("A", 0, 0);
    let b: Arc<dyn IGameComponent> = component("B", 0, 0);
    let c: Arc<dyn IGameComponent> = component("C", -1, -1);
    *a.remove_during_update
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::downgrade(&b));
    let a: Arc<dyn IGameComponent> = a;
    state.Components().Add(a);
    state.Components().Add(Arc::clone(&b));
    state.Components().Add(c);
    ComponentOrderGame {
        state,
        log: Arc::clone(log),
        add_after_initialize: AtomicBool::new(false),
    }
}

#[derive(Default)]
struct LifecycleEvidence {
    events: Vec<&'static str>,
    unload_resources_disposed: bool,
}

struct LifecycleEvidenceGame {
    state: Arc<GameState>,
    evidence: Arc<Mutex<LifecycleEvidence>>,
    device: Arc<Mutex<Option<GraphicsDevice>>>,
    texture: Arc<Mutex<Option<Texture2D>>>,
    batch: Arc<Mutex<Option<SpriteBatch>>>,
}

impl GameStateAccess for LifecycleEvidenceGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl LifecycleEvidenceGame {
    fn event(&self, name: &'static str) {
        self.evidence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .events
            .push(name);
    }
}

impl Game for LifecycleEvidenceGame {
    fn BeginRun(&mut self) {
        self.event("BeginRun");
    }

    fn EndRun(&mut self) {
        self.event("EndRun");
    }

    fn BeginDraw(&mut self) -> bool {
        self.event("BeginDraw");
        true
    }

    fn EndDraw(&mut self) {
        self.event("EndDraw");
    }

    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        self.event("Initialize");
        let exiting_evidence = Arc::clone(&self.evidence);
        self.AddExitingHandler(Box::new(move |sender: &dyn std::any::Any, _| {
            let state = sender
                .downcast_ref::<GameState>()
                .expect("GameState event sender");
            assert!(!state
                .GraphicsDevice()
                .expect("device during Exiting")
                .IsDisposed()
                .expect("device state during Exiting"));
            exiting_evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .events
                .push("Exiting");
        }));
        let disposed_evidence = Arc::clone(&self.evidence);
        self.AddDisposedHandler(Box::new(move |_: &dyn std::any::Any, _| {
            disposed_evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .events
                .push("DisposedEvent");
        }));
        let device = game.GraphicsDevice()?;
        let disposing_evidence = Arc::clone(&self.evidence);
        device.AddDisposingHandler(Box::new(move |sender: &dyn std::any::Any, _| {
            let device = sender
                .downcast_ref::<GraphicsDevice>()
                .expect("GraphicsDevice event sender");
            assert!(device.IsDisposed().expect("disposed event state"));
            disposing_evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .events
                .push("DeviceDisposing");
        }));
        *self
            .device
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(device);
        Ok(())
    }

    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        self.event("LoadContent");
        let first_device = game.GraphicsDevice()?;
        let second_device = game.GraphicsDevice()?;
        assert!(std::ptr::eq(
            first_device.PresentationParameters()?,
            second_device.PresentationParameters()?
        ));
        assert!(std::ptr::eq(
            first_device.Adapter()?,
            second_device.Adapter()?
        ));
        assert!(std::ptr::eq(
            first_device.SamplerStates()?,
            second_device.SamplerStates()?
        ));
        assert!(std::ptr::eq(
            first_device.VertexSamplerStates()?,
            second_device.VertexSamplerStates()?
        ));
        assert!(std::ptr::eq(
            first_device.Textures()?,
            second_device.Textures()?
        ));
        assert!(std::ptr::eq(
            first_device.VertexTextures()?,
            second_device.VertexTextures()?
        ));
        assert_eq!(
            first_device.GraphicsDeviceStatus()?,
            GraphicsDeviceStatus::Normal
        );
        assert!(matches!(
            first_device.GraphicsProfile()?,
            GraphicsProfile::Reach | GraphicsProfile::HiDef
        ));
        assert!(std::ptr::eq(
            first_device.Adapter()?,
            GraphicsAdapter::DefaultAdapter(&first_device)?
        ));
        assert_eq!(
            GraphicsAdapter::Adapters(&first_device)?.as_ptr(),
            GraphicsAdapter::Adapters(&second_device)?.as_ptr()
        );
        assert!(std::ptr::eq(
            first_device.Adapter()?.CurrentDisplayMode()?,
            second_device.Adapter()?.CurrentDisplayMode()?
        ));
        assert!(std::ptr::eq(
            first_device.Adapter()?.SupportedDisplayModes()?,
            second_device.Adapter()?.SupportedDisplayModes()?
        ));
        let mut encoded = Cursor::new(ONE_PIXEL_RGBA_PNG);
        *self
            .texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(
            Texture2D::FromStreamWithGraphicsDeviceAndStream(&first_device, &mut encoded)?,
        );
        *self
            .batch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some(SpriteBatch::new(&second_device)?);
        Ok(())
    }

    fn Update(&mut self, game: &mut GameContext<'_>, time: &GameTime) -> Result<()> {
        let _ = (game, time);
        self.event("Update");
        Ok(())
    }

    fn Draw(&mut self, game: &mut GameContext<'_>, time: &GameTime) -> Result<()> {
        let _ = (game, time);
        self.event("Draw");
        let texture = self
            .texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut batch = self
            .batch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let texture = texture
            .as_ref()
            .expect("texture created during LoadContent");
        let batch = batch.as_mut().expect("batch created during LoadContent");
        batch.Begin()?;
        // The texture and batch were constructed from distinct wrappers
        // returned for one durable device identity. Same-device validation
        // must still accept the draw.
        batch.Draw(texture, Vector2::Zero, Color::White)?;
        batch.End()
    }

    fn UnloadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let _ = game;
        self.event("UnloadContent");
        let texture_disposed = self
            .texture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .expect("texture retained through shutdown")
            .IsDisposed();
        let batch_disposed = self
            .batch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .expect("batch retained through shutdown")
            .IsDisposed();
        self.evidence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .unload_resources_disposed = texture_disposed && batch_disposed;
        Ok(())
    }

    fn Dispose(&mut self) {
        self.event("Dispose");
        self.DisposeWithDisposing(true);
    }
}

#[derive(Default)]
struct ResourceGame {
    state: Arc<GameState>,
    texture: Option<Texture2D>,
    batch: Option<SpriteBatch>,
}

impl GameStateAccess for ResourceGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for ResourceGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let device = game.GraphicsDevice()?;
        let mut encoded = Cursor::new(ONE_PIXEL_RGBA_PNG);
        self.texture = Some(Texture2D::FromStreamWithGraphicsDeviceAndStream(
            &device,
            &mut encoded,
        )?);
        self.batch = Some(SpriteBatch::new(&device)?);
        Ok(())
    }

    fn Draw(&mut self, game: &mut GameContext<'_>, time: &GameTime) -> Result<()> {
        let _ = (game, time);
        if let (Some(batch), Some(texture)) = (&mut self.batch, &self.texture) {
            batch.Begin()?;
            batch.Draw(texture, Vector2::Zero, Color::White)?;
            batch.End()?;
        }
        Ok(())
    }

    fn UnloadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let _ = game;
        if let Some(batch) = &mut self.batch {
            batch.DisposeWithNoArguments()?;
            batch.DisposeWithNoArguments()?;
        }
        if let Some(texture) = &mut self.texture {
            texture.DisposeWithNoArguments()?;
            texture.DisposeWithNoArguments()?;
        }
        self.batch = None;
        self.texture = None;
        Ok(())
    }
}

#[derive(Default)]
struct TextureTransferGame {
    state: Arc<GameState>,
}

impl GameStateAccess for TextureTransferGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for TextureTransferGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let mut device = game.GraphicsDevice()?;
        assert!(matches!(
            Texture2D::new(&device, 0, 1),
            Err(CnaError::InvalidInput(_))
        ));
        assert!(matches!(
            Texture2D::FromStreamWithGraphicsDeviceAndStream(
                &device,
                &mut Cursor::new(Vec::<u8>::new())
            ),
            Err(CnaError::InvalidInput(_))
        ));

        let mut texture = Texture2D::new(&device, 2, 2)?;
        let initial = [Color::Red, Color::Green, Color::Blue, Color::White];
        texture.SetData(&initial)?;
        let mut readback = [Color::Transparent; 4];
        texture.GetData(&mut readback)?;
        assert_eq!(readback, initial);

        let partial = [Color::Black, Color::Yellow];
        texture.SetDataWithLevelAndRectAndDataAndStartIndexAndElementCount(
            0,
            Some(Rectangle::new(1, 0, 1, 1)),
            &partial,
            1,
            1,
        )?;
        texture.GetData(&mut readback)?;
        assert_eq!(
            readback,
            [Color::Red, Color::Yellow, Color::Blue, Color::White]
        );

        assert!(matches!(
            texture.SetDataWithLevelAndRectAndDataAndStartIndexAndElementCount(
                1, None, &initial, 0, 4
            ),
            Err(CnaError::InvalidInput(_))
        ));
        assert!(matches!(
            texture.SetDataWithLevelAndRectAndDataAndStartIndexAndElementCount(
                0,
                Some(Rectangle::new(2, 0, 1, 1)),
                &initial,
                0,
                1
            ),
            Err(CnaError::InvalidInput(_))
        ));
        assert!(matches!(
            texture.SetData::<Color>(&[]),
            Err(CnaError::InvalidInput(_))
        ));
        assert!(matches!(
            texture.SetData(&[0_i32; 4]),
            Err(CnaError::InvalidInput(_))
        ));

        let mut png = Vec::new();
        texture.SaveAsPng(&mut png, 2, 2)?;
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        let mut jpeg = Vec::new();
        texture.SaveAsJpeg(&mut jpeg, 2, 2)?;
        assert_eq!(&jpeg[..2], &[0xff, 0xd8]);

        let tag: Arc<dyn std::any::Any + Send + Sync> = Arc::new(String::from("texture-tag"));
        texture.SetTag(Some(Arc::clone(&tag)));
        assert!(Arc::ptr_eq(&texture.Tag().expect("retained tag"), &tag));
        texture.SetName("transfer texture");
        assert_eq!(texture.ToString(), "transfer texture");

        let event_order = Arc::new(Mutex::new(Vec::new()));
        let first_order = Arc::clone(&event_order);
        let first = texture.AddDisposingHandler(Box::new(
            move |sender: &dyn std::any::Any, _: cna::extensions::events::EventArgs| {
                let texture = sender
                    .downcast_ref::<Texture2D>()
                    .expect("Texture2D event sender");
                assert!(!texture.IsDisposed());
                first_order.lock().expect("event order").push(1);
            },
        ));
        let removed = texture.AddDisposingHandler(Box::new(
            |_: &dyn std::any::Any, _: cna::extensions::events::EventArgs| {
                panic!("removed handler must not run");
            },
        ));
        let last_order = Arc::clone(&event_order);
        texture.AddDisposingHandler(Box::new(
            move |_: &dyn std::any::Any, _: cna::extensions::events::EventArgs| {
                last_order.lock().expect("event order").push(3);
            },
        ));
        assert!(texture.RemoveDisposingHandler(removed));
        assert!(!texture.RemoveDisposingHandler(removed));
        assert_ne!(first, removed);
        texture.DisposeWithNoArguments()?;
        texture.DisposeWithNoArguments()?;
        assert_eq!(*event_order.lock().expect("event order"), [1, 3]);
        assert!(texture.IsDisposed());
        assert!(matches!(
            texture.GetData(&mut readback),
            Err(CnaError::InvalidInput(_))
        ));

        let mut batch = SpriteBatch::new(&device)?;
        let alpha_blend = BlendState::AlphaBlend;
        batch.BeginWithSortModeAndBlendState(SpriteSortMode::Deferred, &alpha_blend)?;
        batch.End()?;
        let mut blend = BlendState::new();
        let sampler = SamplerState::new();
        let depth = DepthStencilState::new();
        let rasterizer = RasterizerState::new();
        batch.BeginWithSortModeAndBlendStateAndSamplerStateAndDepthStencilStateAndRasterizerState(
            SpriteSortMode::Immediate,
            &blend,
            &sampler,
            &depth,
            &rasterizer,
        )?;
        assert!(blend.GraphicsDevice().is_some());
        assert!(sampler.GraphicsDevice().is_some());
        assert!(depth.GraphicsDevice().is_some());
        assert!(rasterizer.GraphicsDevice().is_some());
        batch.End()?;
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            blend.SetMultiSampleMask(0);
        }))
        .is_err());
        let blend = Arc::new(blend);
        let depth = Arc::new(depth);
        let rasterizer = Arc::new(rasterizer);
        device.SetBlendState(Arc::clone(&blend))?;
        device.SetDepthStencilState(Arc::clone(&depth))?;
        device.SetRasterizerState(Arc::clone(&rasterizer))?;
        assert!(Arc::ptr_eq(&blend, &device.BlendState()?));
        assert!(Arc::ptr_eq(&depth, &device.DepthStencilState()?));
        assert!(Arc::ptr_eq(&rasterizer, &device.RasterizerState()?));
        let sampler = Arc::new(SamplerState::LinearClamp);
        let sampler_states = device.SamplerStates()?;
        sampler_states.SetItem(0, Arc::clone(&sampler))?;
        assert!(Arc::ptr_eq(&sampler, &sampler_states.Item(0)?));
        assert!(matches!(
            sampler_states.Item(-1),
            Err(CnaError::InvalidInput(_))
        ));
        let bound_texture: Arc<dyn Texture> = Arc::new(Texture2D::new(&device, 1, 1)?);
        device
            .Textures()?
            .SetItem(0, Some(Arc::clone(&bound_texture)))?;
        assert!(Arc::ptr_eq(
            &bound_texture,
            &device.Textures()?.Item(0)?.expect("bound texture")
        ));
        let associated_device = bound_texture
            .GraphicsDevice()
            .expect("texture retains device association");
        assert!(std::ptr::eq(
            associated_device.PresentationParameters()?,
            device.PresentationParameters()?
        ));
        device.Textures()?.SetItem(0, None)?;
        assert!(device.Textures()?.Item(0)?.is_none());
        assert!(matches!(
            device.Textures()?.Item(16),
            Err(CnaError::InvalidInput(_))
        ));
        device.SetBlendFactor(Color::Red)?;
        assert_eq!(device.BlendFactor()?, Color::Red);
        device.SetMultiSampleMask(0x1234_5678)?;
        assert_eq!(device.MultiSampleMask()?, 0x1234_5678);
        device.SetReferenceStencil(23)?;
        assert_eq!(device.ReferenceStencil()?, 23);
        let viewport = device.Viewport()?;
        device.SetViewport(viewport)?;
        device.SetScissorRectangle(Rectangle::new(0, 0, 1, 1))?;
        assert_eq!(device.ScissorRectangle()?, Rectangle::new(0, 0, 1, 1));
        device.PresentWithNoArguments()?;
        assert!(matches!(batch.End(), Err(CnaError::InvalidInput(_))));
        assert!(matches!(
            batch.Draw(&texture, Vector2::Zero, Color::White),
            Err(CnaError::InvalidInput(_))
        ));
        batch.Begin()?;
        assert!(matches!(batch.Begin(), Err(CnaError::InvalidInput(_))));
        assert!(matches!(
            batch.Draw(&texture, Vector2::Zero, Color::White),
            Err(CnaError::InvalidInput(_))
        ));
        batch.End()?;
        assert!(matches!(
            batch.Draw(&texture, Vector2::Zero, Color::White),
            Err(CnaError::InvalidInput(_))
        ));
        batch.Begin()?;
        batch.DisposeWithNoArguments()?;
        assert!(batch.IsDisposed());
        assert!(matches!(batch.Begin(), Err(CnaError::InvalidInput(_))));
        assert!(matches!(batch.End(), Err(CnaError::InvalidInput(_))));

        let mut panic_texture = Texture2D::new(&device, 1, 1)?;
        panic_texture.AddDisposingHandler(Box::new(
            |_: &dyn std::any::Any, _: cna::extensions::events::EventArgs| {
                panic!("intentional disposing-handler panic");
            },
        ));
        assert!(matches!(
            panic_texture.DisposeWithNoArguments(),
            Err(CnaError::Callback(_))
        ));
        assert!(panic_texture.IsDisposed());

        let mip = Texture2D::from_graphics_device_and_width_and_height_and_mip_map_and_format(
            &device,
            4,
            4,
            true,
            cna::Microsoft::Xna::Framework::Graphics::SurfaceFormat::Color,
        )?;
        assert_eq!(mip.LevelCount(), 3);
        mip.SetDataWithLevelAndRectAndDataAndStartIndexAndElementCount(1, None, &initial, 0, 4)?;
        let mut mip_readback = [Color::Transparent; 4];
        mip.GetDataWithLevelAndRectAndDataAndStartIndexAndElementCount(
            1,
            None,
            &mut mip_readback,
            0,
            4,
        )?;
        assert_eq!(mip_readback, initial);
        Ok(())
    }
}

#[derive(Clone, Copy, Default)]
struct CustomVertex {
    x: f32,
    y: f32,
}

impl VertexData for CustomVertex {
    fn vertex_declaration() -> &'static VertexDeclaration {
        static DECLARATION: OnceLock<VertexDeclaration> = OnceLock::new();
        DECLARATION.get_or_init(|| {
            VertexDeclaration::from_vertex_stride_and_elements(
                8,
                &[VertexElement::new(
                    0,
                    VertexElementFormat::Vector2,
                    VertexElementUsage::Position,
                    0,
                )],
            )
            .expect("valid custom vertex declaration")
        })
    }

    fn write_bytes(&self, destination: &mut Vec<u8>) {
        destination.extend_from_slice(&self.x.to_ne_bytes());
        destination.extend_from_slice(&self.y.to_ne_bytes());
    }

    fn read_bytes(source: &[u8]) -> Result<Self> {
        if source.len() != 8 {
            return Err(CnaError::InvalidInput(
                "custom vertex payload must contain eight bytes",
            ));
        }
        Ok(Self {
            x: f32::from_ne_bytes(source[0..4].try_into().expect("four-byte x")),
            y: f32::from_ne_bytes(source[4..8].try_into().expect("four-byte y")),
        })
    }
}

#[derive(Default)]
struct BufferTransferGame {
    state: Arc<GameState>,
    vertex: Option<VertexBuffer>,
    index: Option<IndexBuffer>,
    texture_cube: Option<TextureCube>,
    render_target2d: Option<RenderTarget2D>,
    render_target_cube: Option<RenderTargetCube>,
}

struct SpriteFontXnbGame {
    state: Arc<GameState>,
    root: PathBuf,
    retained_font: Arc<Mutex<Option<Arc<SpriteFont>>>>,
}

struct EffectXnbGame {
    state: Arc<GameState>,
    root: PathBuf,
}

struct ModelXnbGame {
    state: Arc<GameState>,
    root: PathBuf,
    retained_model: Arc<Mutex<Option<Arc<Model>>>>,
    retained_bone: Arc<Mutex<Option<Arc<ModelBone>>>>,
}

#[derive(Default)]
struct RemainingGraphicsGame {
    state: Arc<GameState>,
    texture3d: Option<Texture3D>,
}

impl GameStateAccess for ModelXnbGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for ModelXnbGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let _ = game;
        self.Content()
            .SetRootDirectory(self.root.to_str().expect("UTF-8 Model fixture path"))?;

        assert!(self.Content().Load::<Texture3D>("model").is_err());
        assert!(self.Content().Load::<Model>("bad-root").is_err());
        assert!(self.Content().Load::<Model>("missing-effect").is_err());

        let first = self.Content().Load::<Model>("model")?;
        let cached = self.Content().Load::<Model>("MODEL")?;
        assert!(Arc::ptr_eq(&first, &cached));
        assert_eq!(first.Bones()?.Count()?, 2);
        assert_eq!(first.Meshes()?.Count()?, 1);
        assert_eq!(first.Root()?.Name()?, "Root");

        match self.Content().Load::<Texture3D>("texture3d") {
            Ok(texture) => {
                assert_eq!(
                    (texture.Width(), texture.Height(), texture.Depth()),
                    (2, 2, 2)
                );
                assert_eq!(texture.LevelCount(), 2);
                assert!(Arc::ptr_eq(
                    &texture,
                    &self.Content().Load::<Texture3D>("TEXTURE3D")?
                ));
            }
            Err(CnaError::Content(error)) => {
                let message = error.to_string();
                assert!(
                    message.contains("CNA error 6")
                        && message.contains("does not support real volume (3D) texture storage"),
                    "unexpected Texture3D content error: {message}"
                );
            }
            Err(error) => return Err(error),
        }
        match self.Content().Load::<TextureCube>("texture-cube") {
            Ok(texture) => {
                assert_eq!(texture.Size(), 1);
                assert_eq!(texture.LevelCount(), 1);
                assert!(Arc::ptr_eq(
                    &texture,
                    &self.Content().Load::<TextureCube>("TEXTURE-CUBE")?
                ));
            }
            Err(CnaError::Content(error)) => {
                assert!(
                    error.to_string().contains("CNA error 6"),
                    "unexpected TextureCube content error: {error}"
                );
            }
            Err(error) => return Err(error),
        }
        let root_a = first.Bones()?.ItemAt(0)?;
        let root_b = first.Bones()?.ItemAt(0)?;
        assert!(Arc::ptr_eq(&root_a, &root_b));
        let child = first.Bones()?.ItemAt(1)?;
        assert!(Arc::ptr_eq(
            &child.Parent()?.expect("child parent"),
            &root_a
        ));
        assert_eq!(root_a.Children()?.Count()?, 1);
        assert!(Arc::ptr_eq(&root_a.Children()?.ItemAt(0)?, &child));

        let mesh = first.Meshes()?.ItemAt(0)?;
        assert_eq!(mesh.Name()?, "Triangle");
        assert!(std::ptr::eq(mesh.ParentBone()?, child.as_ref()));
        assert_eq!(mesh.MeshParts()?.Count()?, 2);
        assert_eq!(mesh.Effects()?.Count()?, 1);
        let part0 = mesh.MeshParts()?.ItemAt(0)?;
        let part1 = mesh.MeshParts()?.ItemAt(1)?;
        assert!(std::ptr::eq(part0.VertexBuffer()?, part1.VertexBuffer()?));
        assert!(std::ptr::eq(part0.IndexBuffer()?, part1.IndexBuffer()?));
        let effect0 = part0.Effect()?.expect("part 0 effect");
        let effect1 = part1.Effect()?.expect("part 1 effect");
        assert!(Arc::ptr_eq(&effect0, &effect1));
        assert_eq!(
            first
                .Tag()?
                .expect("model tag")
                .downcast::<String>()
                .expect("string model tag")
                .as_str(),
            "model-tag"
        );

        let mut local = vec![Matrix::Identity; 2];
        first.CopyBoneTransformsTo(&mut local)?;
        assert_eq!(local[1].M41, 2.0);
        let mut absolute = vec![Matrix::Identity; 2];
        first.CopyAbsoluteBoneTransformsTo(&mut absolute)?;
        assert_eq!(absolute[1].M41, 2.0);
        first.Draw(Matrix::Identity, Matrix::Identity, Matrix::Identity)?;

        let alpha = self.Content().Load::<AlphaTestEffect>("alpha")?;
        assert_eq!(alpha.AlphaFunction()?, CompareFunction::Greater);
        assert_eq!(alpha.ReferenceAlpha()?, 127);
        assert_eq!(alpha.Alpha()?, 0.75);
        apply_current_passes(&alpha)?;
        let dual = self.Content().Load::<DualTextureEffect>("dual")?;
        assert_eq!(dual.Alpha()?, 0.5);
        apply_current_passes(&dual)?;
        let environment = self.Content().Load::<EnvironmentMapEffect>("environment")?;
        assert_eq!(environment.EnvironmentMapAmount()?, 0.25);
        assert_eq!(environment.FresnelFactor()?, 0.75);
        apply_current_passes(&environment)?;
        let skinned = self.Content().Load::<SkinnedEffect>("skinned")?;
        assert_eq!(skinned.WeightsPerVertex()?, 2);
        assert_eq!(skinned.SpecularPower()?, 8.0);
        apply_current_passes(&skinned)?;

        self.Content().Unload()?;
        assert!(first.Bones().is_err());
        assert!(root_a.Name().is_err());
        assert!(part0.VertexBuffer().is_err());

        let reloaded = self.Content().Load::<Model>("model")?;
        assert!(!Arc::ptr_eq(&first, &reloaded));
        let retained_bone = reloaded.Bones()?.ItemAt(1)?;
        *self
            .retained_bone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(retained_bone);
        *self
            .retained_model
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(reloaded);
        Ok(())
    }
}

impl GameStateAccess for RemainingGraphicsGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for RemainingGraphicsGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let device = game.GraphicsDevice()?;
        let mut basic = BasicEffect::from_device(&device)?;
        assert!(std::ptr::eq(
            basic.DirectionalLight0()?,
            basic.DirectionalLight0()?
        ));
        basic.SetFogEnabled(true)?;
        basic.SetFogStart(2.0)?;
        basic.SetFogEnd(8.0)?;
        basic.SetLightingEnabled(true)?;
        basic.DirectionalLight0()?.SetEnabled(true)?;
        basic
            .DirectionalLight0()?
            .SetDirection(Vector3::from_x_and_y_and_z(0.0, -1.0, 0.0))?;
        let clone = BasicEffect::new(&basic)?;
        assert_eq!(clone.FogEnabled()?, basic.FogEnabled()?);
        assert_eq!(clone.FogStart()?, 2.0);
        apply_current_passes(&basic)?;
        apply_current_passes(&clone)?;

        let alpha = AlphaTestEffect::from_device(&device)?;
        apply_current_passes(&alpha)?;
        let dual = DualTextureEffect::from_device(&device)?;
        apply_current_passes(&dual)?;
        let environment = EnvironmentMapEffect::from_device(&device)?;
        assert!(std::ptr::eq(
            environment.DirectionalLight0()?,
            environment.DirectionalLight0()?
        ));
        apply_current_passes(&environment)?;
        let skinned = SkinnedEffect::from_device(&device)?;
        apply_current_passes(&skinned)?;

        match Texture3D::new(&device, 2, 2, 2, true, SurfaceFormat::Color) {
            Ok(texture) => {
                assert_eq!(texture.LevelCount(), 2);
                let voxels = [
                    Color::Red,
                    Color::Green,
                    Color::Blue,
                    Color::White,
                    Color::Black,
                    Color::Yellow,
                    Color::CornflowerBlue,
                    Color::Transparent,
                ];
                texture.SetData(&voxels)?;
                let mut readback = [Color::Transparent; 8];
                texture.GetData(&mut readback)?;
                assert_eq!(readback, voxels);
                texture.SetDataWithLevelAndLeftAndTopAndRightAndBottomAndFrontAndBackAndDataAndStartIndexAndElementCount(
                    1,
                    0,
                    0,
                    1,
                    1,
                    0,
                    1,
                    &[Color::Magenta],
                    0,
                    1,
                )?;
                assert!(texture
                    .SetDataWithLevelAndLeftAndTopAndRightAndBottomAndFrontAndBackAndDataAndStartIndexAndElementCount(
                        2, 0, 0, 1, 1, 0, 1, &[Color::White], 0, 1,
                    )
                    .is_err());
                self.texture3d = Some(texture);
            }
            Err(CnaError::Native { code: 6, .. }) => {}
            Err(error) => return Err(error),
        }

        let mut query = OcclusionQuery::new(&device)?;
        assert!(query.End().is_err());
        match query.Begin() {
            Ok(()) => {
                assert!(query.Begin().is_err());
                assert!(!query.IsComplete()?);
                assert!(query.PixelCount().is_err());
                query.End()?;
                match query.IsComplete() {
                    Ok(true) => {
                        assert_eq!(query.PixelCount()?, 1);
                    }
                    Ok(false) | Err(CnaError::Native { code: 6, .. }) => {}
                    Err(error) => return Err(error),
                }
            }
            Err(CnaError::Native { code: 6, .. }) => {}
            Err(error) => return Err(error),
        }
        query.Dispose(true)?;
        query.Dispose(true)?;
        assert!(query.Begin().is_err());
        let mut active_query = OcclusionQuery::new(&device)?;
        if active_query.Begin().is_ok() {
            active_query.Dispose(true)?;
            assert!(active_query.End().is_err());
        }
        Ok(())
    }
}

fn apply_current_passes(effect: &Effect) -> Result<()> {
    for pass in effect.CurrentTechnique()?.Passes()?.GetEnumerator()? {
        pass.Apply()?;
    }
    Ok(())
}

#[derive(Default)]
struct EffectStressGame {
    state: Arc<GameState>,
}

impl GameStateAccess for EffectStressGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for EffectStressGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let device = game.GraphicsDevice()?;
        let bool_bits = f32::from_bits(1);
        let mut effect = device.create_reflection_effect(
            &[
                EffectParameterDescriptor {
                    name: "Gain".to_owned(),
                    semantic: "SCALAR".to_owned(),
                    row_count: 1,
                    column_count: 1,
                    parameter_class: EffectParameterClass::Scalar,
                    parameter_type: EffectParameterType::Single,
                    annotations: vec![EffectAnnotationDescriptor {
                        name: "Visible".to_owned(),
                        semantic: String::new(),
                        row_count: 1,
                        column_count: 1,
                        parameter_class: EffectParameterClass::Scalar,
                        parameter_type: EffectParameterType::Bool,
                        data: vec![bool_bits],
                        cached_string: String::new(),
                    }],
                },
                EffectParameterDescriptor {
                    name: "Enabled".to_owned(),
                    semantic: String::new(),
                    row_count: 1,
                    column_count: 1,
                    parameter_class: EffectParameterClass::Scalar,
                    parameter_type: EffectParameterType::Bool,
                    annotations: Vec::new(),
                },
                EffectParameterDescriptor {
                    name: "Count".to_owned(),
                    semantic: String::new(),
                    row_count: 1,
                    column_count: 1,
                    parameter_class: EffectParameterClass::Scalar,
                    parameter_type: EffectParameterType::Int32,
                    annotations: Vec::new(),
                },
                EffectParameterDescriptor {
                    name: "Tint".to_owned(),
                    semantic: String::new(),
                    row_count: 1,
                    column_count: 4,
                    parameter_class: EffectParameterClass::Vector,
                    parameter_type: EffectParameterType::Single,
                    annotations: Vec::new(),
                },
                EffectParameterDescriptor {
                    name: "Transform".to_owned(),
                    semantic: String::new(),
                    row_count: 4,
                    column_count: 4,
                    parameter_class: EffectParameterClass::Matrix,
                    parameter_type: EffectParameterType::Single,
                    annotations: Vec::new(),
                },
            ],
            &[
                EffectTechniqueDescriptor {
                    name: "FirstTechnique".to_owned(),
                    passes: vec!["P0".to_owned(), "StatePass".to_owned()],
                },
                EffectTechniqueDescriptor {
                    name: "SecondTechnique".to_owned(),
                    passes: vec!["P1".to_owned()],
                },
            ],
        )?;

        let parameters = effect.Parameters()?;
        assert!(Arc::ptr_eq(&parameters, &effect.Parameters()?));
        assert_eq!(parameters.Count()?, 5);
        let gain = parameters.Item("Gain")?.expect("Gain parameter");
        assert!(Arc::ptr_eq(&gain, &parameters.item_at(0)?));
        assert_eq!(gain.Name()?, "Gain");
        assert_eq!(gain.Semantic()?, "SCALAR");
        assert!(Arc::ptr_eq(
            &gain,
            &parameters
                .GetParameterBySemantic("SCALAR")?
                .expect("semantic lookup")
        ));
        gain.SetValueWithValueAsSingle(0.75)?;
        assert_eq!(gain.GetValueSingle()?, 0.75);
        let annotations = gain.Annotations()?;
        assert_eq!(annotations.Count()?, 1);
        let visible = annotations.Item("Visible")?.expect("Visible annotation");
        assert!(Arc::ptr_eq(&visible, &annotations.item_at(0)?));
        assert!(visible.GetValueBoolean()?);
        assert_eq!(visible.ParameterType()?, EffectParameterType::Bool);

        let enabled = parameters.Item("Enabled")?.expect("Enabled parameter");
        enabled.SetValueWithValueAsBoolean(true)?;
        assert!(enabled.GetValueBoolean()?);
        let count = parameters.Item("Count")?.expect("Count parameter");
        count.SetValueWithValueAsInt32(7)?;
        assert_eq!(count.GetValueInt32()?, 7);
        let tint = parameters.Item("Tint")?.expect("Tint parameter");
        let tint_value = Vector4::from_x_and_y_and_z_and_w(0.1, 0.2, 0.3, 0.4);
        tint.SetValueWithValueAsVector4(tint_value)?;
        assert_eq!(tint.GetValueVector4()?, tint_value);
        let transform = parameters.Item("Transform")?.expect("Transform parameter");
        transform.SetValueWithValueAsMatrix(Matrix::Identity)?;
        assert_eq!(transform.GetValueMatrix()?, Matrix::Identity);

        let techniques = effect.Techniques()?;
        assert!(Arc::ptr_eq(&techniques, &effect.Techniques()?));
        // CNA's empty Effect starts with XNA's default technique; the two
        // reflected techniques follow it in insertion order.
        assert_eq!(techniques.Count()?, 3);
        let first = techniques.Item("FirstTechnique")?.expect("first technique");
        assert!(Arc::ptr_eq(&first, &techniques.item_at(1)?));
        assert!(Arc::ptr_eq(&first, &effect.CurrentTechnique()?));
        let passes = first.Passes()?;
        assert_eq!(passes.Count()?, 3);
        let first_pass = passes.Item("P0")?.expect("first pass");
        assert!(Arc::ptr_eq(&first_pass, &passes.item_at(0)?));
        match first_pass.Apply() {
            Ok(()) | Err(CnaError::Native { code: 6, .. }) => {}
            Err(error) => return Err(error),
        }
        let second = techniques
            .Item("SecondTechnique")?
            .expect("second technique");
        effect.SetCurrentTechnique(&second)?;
        assert!(Arc::ptr_eq(&second, &effect.CurrentTechnique()?));

        let cloned = effect.Clone()?;
        assert_eq!(cloned.Parameters()?.Count()?, 5);
        let material = EffectMaterial::new(&effect)?;
        assert_eq!(material.Parameters()?.Count()?, 5);

        let mut batch = SpriteBatch::new(&device)?;
        let blend = BlendState::AlphaBlend;
        let sampler = SamplerState::LinearClamp;
        let depth = DepthStencilState::None;
        let rasterizer = RasterizerState::CullCounterClockwise;
        let result = batch.BeginWithSortModeAndBlendStateAndSamplerStateAndDepthStencilStateAndRasterizerStateAndEffect(
            SpriteSortMode::Immediate, &blend, &sampler, &depth, &rasterizer, Some(&effect),
        );
        match result {
            Ok(()) => batch.End()?,
            Err(CnaError::Native { code: 6, .. }) => {
                batch.Begin()?;
                batch.End()?;
            }
            Err(error) => return Err(error),
        }
        batch.BeginWithSortModeAndBlendStateAndSamplerStateAndDepthStencilStateAndRasterizerStateAndEffectAndTransformMatrix(
            SpriteSortMode::Deferred, &blend, &sampler, &depth, &rasterizer, None, Matrix::Identity,
        )?;
        batch.End()?;

        effect.DisposeWithNoArguments()?;
        effect.DisposeWithNoArguments()?;
        assert!(gain.Name().is_err());
        assert!(matches!(
            batch.BeginWithSortModeAndBlendStateAndSamplerStateAndDepthStencilStateAndRasterizerStateAndEffect(
                SpriteSortMode::Deferred, &blend, &sampler, &depth, &rasterizer, Some(&effect),
            ),
            Err(CnaError::InvalidInput(_))
        ));
        batch.Begin()?;
        batch.End()?;
        Ok(())
    }
}

impl GameStateAccess for SpriteFontXnbGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for SpriteFontXnbGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        self.Content()
            .SetRootDirectory(self.root.to_str().expect("UTF-8 SpriteFont fixture path"))?;
        let font = self.Content().Load::<SpriteFont>("font")?;
        let cached = self.Content().Load::<SpriteFont>("FONT")?;
        assert!(Arc::ptr_eq(&font, &cached));
        assert_eq!(font.Characters(), ['?']);
        assert_eq!(font.DefaultCharacter(), Some('?'));
        assert_eq!(font.LineSpacing(), 2);
        assert_eq!(font.Spacing(), 0.0);
        assert_eq!(font.MeasureString("?")?, Vector2::from_x_and_y(1.0, 2.0));
        assert_eq!(
            font.MeasureStringWithText("??")?,
            Vector2::from_x_and_y(2.0, 2.0)
        );
        let multiline = font.MeasureString("?\n?")?;
        assert_eq!(multiline.X, 1.0);
        assert_eq!(multiline.Y, 4.0);
        font.SetLineSpacing(3)?;
        font.SetSpacing(0.25)?;
        assert_eq!(font.LineSpacing(), 3);
        assert_eq!(font.Spacing(), 0.25);
        assert!(font.SetSpacing(f32::NAN).is_err());
        assert!(font.SetDefaultCharacter(Some('Z')).is_err());

        let device = game.GraphicsDevice()?;
        let mut batch = SpriteBatch::new(&device)?;
        batch.Begin()?;
        batch.DrawString(&font, "", Vector2::Zero, Color::White)?;
        batch.DrawStringWithSpriteFontAndTextAndPositionAndColor(
            &font,
            "?",
            Vector2::Zero,
            Color::White,
        )?;
        batch.DrawStringWithSpriteFontAndTextAndPositionAndColorAndRotationAndOriginAndScaleAndEffectsAndLayerDepthAsSpriteFontAndStringAndVector2AndColorAndSingleAndVector2AndSingleAndSpriteEffectsAndSingle(
            &font, "?", Vector2::Zero, Color::White, 0.0, Vector2::Zero, 1.0,
            cna::Microsoft::Xna::Framework::Graphics::SpriteEffects::None, 0.0,
        )?;
        batch.DrawStringWithSpriteFontAndTextAndPositionAndColorAndRotationAndOriginAndScaleAndEffectsAndLayerDepthAsSpriteFontAndStringAndVector2AndColorAndSingleAndVector2AndVector2AndSpriteEffectsAndSingle(
            &font, "?", Vector2::Zero, Color::White, 0.0, Vector2::Zero, Vector2::One,
            cna::Microsoft::Xna::Framework::Graphics::SpriteEffects::FlipHorizontally, 0.0,
        )?;
        batch.DrawStringWithSpriteFontAndTextAndPositionAndColorAndRotationAndOriginAndScaleAndEffectsAndLayerDepthAsSpriteFontAndStringBuilderAndVector2AndColorAndSingleAndVector2AndSingleAndSpriteEffectsAndSingle(
            &font, "?", Vector2::Zero, Color::White, 0.0, Vector2::Zero, 1.0,
            cna::Microsoft::Xna::Framework::Graphics::SpriteEffects::None, 0.0,
        )?;
        batch.DrawStringWithSpriteFontAndTextAndPositionAndColorAndRotationAndOriginAndScaleAndEffectsAndLayerDepthAsSpriteFontAndStringBuilderAndVector2AndColorAndSingleAndVector2AndVector2AndSpriteEffectsAndSingle(
            &font, "?", Vector2::Zero, Color::White, 0.0, Vector2::Zero, Vector2::One,
            cna::Microsoft::Xna::Framework::Graphics::SpriteEffects::FlipVertically, 0.0,
        )?;
        font.SetDefaultCharacter(None)?;
        assert!(batch
            .DrawString(&font, "Z", Vector2::Zero, Color::White)
            .is_err());
        font.SetDefaultCharacter(Some('?'))?;
        batch.End()?;

        *self
            .retained_font
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(font);
        Ok(())
    }
}

impl GameStateAccess for EffectXnbGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for EffectXnbGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let _ = game;
        self.Content()
            .SetRootDirectory(self.root.to_str().expect("UTF-8 Effect fixture path"))?;
        match self.Content().Load::<Effect>("effect") {
            Ok(effect) => {
                let cached = self.Content().Load::<Effect>("EFFECT")?;
                assert!(Arc::ptr_eq(&effect, &cached));
                assert_eq!(effect.Name(), "effect");
                assert!(effect.Techniques()?.Count()? > 0);
            }
            Err(CnaError::Content(error)) => {
                let message = error.to_string();
                assert!(
                    message.contains("could not construct Effect content asset 'effect'"),
                    "unexpected Effect content error: {message}"
                );
                assert!(
                    message.contains("CNA error 6")
                        && message.contains("does not support compiled XNA/FNA Effect"),
                    "missing native inner error: {message}"
                );
                assert!(matches!(
                    self.Content().Load::<Effect>("effect"),
                    Err(CnaError::Content(_))
                ));
            }
            Err(error) => return Err(error),
        }
        Ok(())
    }
}

struct SpriteFontFixture(PathBuf);

impl SpriteFontFixture {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("cna-rust-sprite-font-xnb-{}", std::process::id()));
        fs::create_dir_all(&path).expect("create SpriteFont XNB fixture directory");
        fs::write(path.join("font.xnb"), sprite_font_xnb()).expect("write SpriteFont XNB fixture");
        Self(path)
    }
}

impl Drop for SpriteFontFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct EffectFixture(PathBuf);

impl EffectFixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("cna-rust-effect-xnb-{}", std::process::id()));
        fs::create_dir_all(&path).expect("create Effect XNB fixture directory");
        fs::write(path.join("effect.xnb"), effect_xnb()).expect("write Effect XNB fixture");
        Self(path)
    }
}

impl Drop for EffectFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct ModelFixture(PathBuf);

impl ModelFixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("cna-rust-model-xnb-{}", std::process::id()));
        fs::create_dir_all(&path).expect("create Model XNB fixture directory");
        fs::write(path.join("model.xnb"), compressed_xnb(&model_xnb(1, 3)))
            .expect("write valid compressed Model XNB fixture");
        fs::write(path.join("bad-root.xnb"), model_xnb(3, 3))
            .expect("write malformed-root Model XNB fixture");
        fs::write(path.join("missing-effect.xnb"), model_xnb(1, 0))
            .expect("write missing-effect Model XNB fixture");
        fs::write(path.join("alpha.xnb"), alpha_test_effect_xnb())
            .expect("write AlphaTestEffect XNB fixture");
        fs::write(path.join("dual.xnb"), dual_texture_effect_xnb())
            .expect("write DualTextureEffect XNB fixture");
        fs::write(path.join("environment.xnb"), environment_map_effect_xnb())
            .expect("write EnvironmentMapEffect XNB fixture");
        fs::write(path.join("skinned.xnb"), skinned_effect_xnb())
            .expect("write SkinnedEffect XNB fixture");
        fs::write(path.join("texture3d.xnb"), texture3d_xnb())
            .expect("write Texture3D XNB fixture");
        fs::write(path.join("texture-cube.xnb"), texture_cube_xnb())
            .expect("write TextureCube XNB fixture");
        Self(path)
    }
}

impl Drop for ModelFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn model_xnb(root_reference: u8, effect_shared_reference: u8) -> Vec<u8> {
    const READERS: &[&str] = &[
        "Microsoft.Xna.Framework.Content.ModelReader",
        "Microsoft.Xna.Framework.Content.StringReader",
        "Microsoft.Xna.Framework.Content.VertexBufferReader",
        "Microsoft.Xna.Framework.Content.IndexBufferReader",
        "Microsoft.Xna.Framework.Content.BasicEffectReader",
    ];
    let mut payload = Vec::new();
    write_7bit(&mut payload, READERS.len());
    for reader in READERS {
        write_xnb_string(&mut payload, reader);
        payload.extend_from_slice(&0_i32.to_le_bytes());
    }
    write_7bit(&mut payload, 3);

    payload.push(1); // ModelReader root object.
    payload.extend_from_slice(&2_u32.to_le_bytes());
    write_dispatched_string(&mut payload, "Root");
    write_matrix(&mut payload, Matrix::Identity);
    write_dispatched_string(&mut payload, "Child");
    let mut child_transform = Matrix::Identity;
    child_transform.M41 = 2.0;
    write_matrix(&mut payload, child_transform);

    payload.push(0); // Root parent.
    payload.extend_from_slice(&1_u32.to_le_bytes());
    payload.push(2); // Root child is Child.
    payload.push(1); // Child parent is Root (reader validates, hierarchy comes from children).
    payload.extend_from_slice(&0_u32.to_le_bytes());

    payload.extend_from_slice(&1_i32.to_le_bytes());
    write_dispatched_string(&mut payload, "Triangle");
    payload.push(2); // Parent bone is Child.
    for value in [0.0_f32, 0.0, 0.0, 2.0] {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    write_dispatched_string(&mut payload, "mesh-tag");
    payload.extend_from_slice(&2_i32.to_le_bytes());
    for part_index in 0..2_i32 {
        for value in [0_i32, 3, 0, 1] {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        write_dispatched_string(&mut payload, &format!("part-{part_index}"));
        write_7bit(&mut payload, 1);
        write_7bit(&mut payload, 2);
        write_7bit(&mut payload, usize::from(effect_shared_reference));
    }
    payload.push(root_reference);
    write_dispatched_string(&mut payload, "model-tag");

    payload.push(3); // Shared VertexBuffer.
    payload.extend_from_slice(&12_i32.to_le_bytes());
    payload.extend_from_slice(&1_i32.to_le_bytes());
    for value in [0_i32, 2, 0, 0] {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    payload.extend_from_slice(&3_u32.to_le_bytes());
    for value in [0.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
        payload.extend_from_slice(&value.to_le_bytes());
    }

    payload.push(4); // Shared IndexBuffer.
    payload.push(1); // Sixteen-bit indices.
    payload.extend_from_slice(&6_i32.to_le_bytes());
    for value in [0_u16, 1, 2] {
        payload.extend_from_slice(&value.to_le_bytes());
    }

    payload.push(5); // Shared BasicEffect.
    write_xnb_string(&mut payload, "");
    for value in [
        1.0_f32, 1.0, 1.0, // Diffuse.
        0.0, 0.0, 0.0, // Emissive.
        1.0, 1.0, 1.0, // Specular.
        16.0, 1.0, // Power and alpha.
    ] {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    payload.push(0); // VertexColorEnabled.

    let mut bytes = b"XNBw\x05\x00".to_vec();
    bytes.extend_from_slice(
        &u32::try_from(10 + payload.len())
            .expect("Model fixture size")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&payload);
    bytes
}

fn compressed_xnb(uncompressed: &[u8]) -> Vec<u8> {
    let payload = &uncompressed[10..];
    assert!(!payload.is_empty() && payload.len() <= 0x8000);
    let header_bits =
        (3_u32 << 28) | (u32::try_from(payload.len()).expect("LZX payload size") << 4);
    let mut block = vec![0; 16 + payload.len()];
    block[0] = u8::try_from((header_bits >> 16) & 0xff).expect("LZX header byte");
    block[1] = u8::try_from((header_bits >> 24) & 0xff).expect("LZX header byte");
    block[2] = u8::try_from(header_bits & 0xff).expect("LZX header byte");
    block[3] = u8::try_from((header_bits >> 8) & 0xff).expect("LZX header byte");
    block[4] = 1;
    block[8] = 1;
    block[12] = 1;
    block[16..].copy_from_slice(payload);

    let mut framed = vec![
        0xff,
        u8::try_from(payload.len() >> 8).expect("LZX frame size"),
        u8::try_from(payload.len() & 0xff).expect("LZX frame size"),
        u8::try_from(block.len() >> 8).expect("LZX block size"),
        u8::try_from(block.len() & 0xff).expect("LZX block size"),
    ];
    framed.extend_from_slice(&block);

    let mut bytes = b"XNBw\x05\x80".to_vec();
    bytes.extend_from_slice(
        &u32::try_from(14 + framed.len())
            .expect("compressed XNB size")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(
        &u32::try_from(payload.len())
            .expect("decompressed XNB size")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&framed);
    bytes
}

fn stock_effect_xnb(reader: &str, body: &[u8]) -> Vec<u8> {
    let mut payload = Vec::new();
    write_7bit(&mut payload, 1);
    write_xnb_string(&mut payload, reader);
    payload.extend_from_slice(&0_i32.to_le_bytes());
    write_7bit(&mut payload, 0);
    payload.push(1);
    payload.extend_from_slice(body);
    let mut bytes = b"XNBw\x05\x00".to_vec();
    bytes.extend_from_slice(
        &u32::try_from(10 + payload.len())
            .expect("stock-effect fixture size")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&payload);
    bytes
}

fn texture3d_xnb() -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&0_i32.to_le_bytes()); // SurfaceFormat.Color.
    body.extend_from_slice(&2_i32.to_le_bytes()); // Width.
    body.extend_from_slice(&2_i32.to_le_bytes()); // Height.
    body.extend_from_slice(&2_i32.to_le_bytes()); // Depth.
    body.extend_from_slice(&2_i32.to_le_bytes()); // Complete mip chain.
    body.extend_from_slice(&32_i32.to_le_bytes());
    body.extend_from_slice(&[255_u8; 32]);
    body.extend_from_slice(&4_i32.to_le_bytes());
    body.extend_from_slice(&[255_u8; 4]);
    stock_effect_xnb("Microsoft.Xna.Framework.Content.Texture3DReader", &body)
}

fn texture_cube_xnb() -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&0_i32.to_le_bytes()); // SurfaceFormat.Color.
    body.extend_from_slice(&1_i32.to_le_bytes()); // Size.
    body.extend_from_slice(&1_i32.to_le_bytes()); // One mip.
    for _ in 0..6 {
        body.extend_from_slice(&4_i32.to_le_bytes());
        body.extend_from_slice(&[255_u8; 4]);
    }
    stock_effect_xnb("Microsoft.Xna.Framework.Content.TextureCubeReader", &body)
}

fn write_vector3(bytes: &mut Vec<u8>, value: [f32; 3]) {
    for component in value {
        bytes.extend_from_slice(&component.to_le_bytes());
    }
}

fn alpha_test_effect_xnb() -> Vec<u8> {
    let mut body = Vec::new();
    write_xnb_string(&mut body, "");
    body.extend_from_slice(&6_i32.to_le_bytes());
    body.extend_from_slice(&127_u32.to_le_bytes());
    write_vector3(&mut body, [0.25, 0.5, 0.75]);
    body.extend_from_slice(&0.75_f32.to_le_bytes());
    body.push(1);
    stock_effect_xnb(
        "Microsoft.Xna.Framework.Content.AlphaTestEffectReader",
        &body,
    )
}

fn dual_texture_effect_xnb() -> Vec<u8> {
    let mut body = Vec::new();
    write_xnb_string(&mut body, "");
    write_xnb_string(&mut body, "");
    write_vector3(&mut body, [0.2, 0.4, 0.6]);
    body.extend_from_slice(&0.5_f32.to_le_bytes());
    body.push(1);
    stock_effect_xnb(
        "Microsoft.Xna.Framework.Content.DualTextureEffectReader",
        &body,
    )
}

fn environment_map_effect_xnb() -> Vec<u8> {
    let mut body = Vec::new();
    write_xnb_string(&mut body, "");
    write_xnb_string(&mut body, "");
    body.extend_from_slice(&0.25_f32.to_le_bytes());
    write_vector3(&mut body, [0.1, 0.2, 0.3]);
    body.extend_from_slice(&0.75_f32.to_le_bytes());
    write_vector3(&mut body, [0.4, 0.5, 0.6]);
    write_vector3(&mut body, [0.1, 0.0, 0.0]);
    body.extend_from_slice(&0.8_f32.to_le_bytes());
    stock_effect_xnb(
        "Microsoft.Xna.Framework.Content.EnvironmentMapEffectReader",
        &body,
    )
}

fn skinned_effect_xnb() -> Vec<u8> {
    let mut body = Vec::new();
    write_xnb_string(&mut body, "");
    body.extend_from_slice(&2_i32.to_le_bytes());
    write_vector3(&mut body, [0.7, 0.6, 0.5]);
    write_vector3(&mut body, [0.1, 0.2, 0.3]);
    write_vector3(&mut body, [0.9, 0.8, 0.7]);
    body.extend_from_slice(&8.0_f32.to_le_bytes());
    body.extend_from_slice(&0.9_f32.to_le_bytes());
    stock_effect_xnb("Microsoft.Xna.Framework.Content.SkinnedEffectReader", &body)
}

fn write_dispatched_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.push(2);
    write_xnb_string(bytes, value);
}

fn write_matrix(bytes: &mut Vec<u8>, value: Matrix) {
    for component in [
        value.M11, value.M12, value.M13, value.M14, value.M21, value.M22, value.M23, value.M24,
        value.M31, value.M32, value.M33, value.M34, value.M41, value.M42, value.M43, value.M44,
    ] {
        bytes.extend_from_slice(&component.to_le_bytes());
    }
}

/// Locates CNA's legal conformance Effect bytecode.
///
/// The path was previously relative to this crate's manifest, which assumed
/// one sibling checkout layout and broke the moment the crate was vendored
/// into a generated project. `CNA_ROOT` is the input the loader already
/// documents, so the fixture follows it; the sibling layout stays as the last
/// resort for a source checkout that sets nothing.
fn conformance_effect_path() -> PathBuf {
    const RELATIVE: &str = "modules/renderers/fna3d/effects/CnaConformanceEffect.fxb";
    if let Some(explicit) = std::env::var_os("CNA_CONFORMANCE_EFFECT") {
        return PathBuf::from(explicit);
    }
    if let Some(root) = std::env::var_os("CNA_ROOT") {
        return PathBuf::from(root).join(RELATIVE);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../cna")
        .join(RELATIVE)
}

fn effect_xnb() -> Vec<u8> {
    let path = conformance_effect_path();
    let effect_code = fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "read CNA's legal conformance Effect bytecode fixture at {}: {error}; \
             set CNA_ROOT to the CNA checkout or CNA_CONFORMANCE_EFFECT to the file",
            path.display()
        )
    });
    let mut payload = Vec::new();
    write_7bit(&mut payload, 1);
    write_xnb_string(&mut payload, "Microsoft.Xna.Framework.Content.EffectReader");
    payload.extend_from_slice(&0_i32.to_le_bytes());
    write_7bit(&mut payload, 0);
    payload.push(1);
    payload.extend_from_slice(
        &i32::try_from(effect_code.len())
            .expect("Effect fixture bytecode length")
            .to_le_bytes(),
    );
    payload.extend_from_slice(&effect_code);

    let mut bytes = b"XNBw\x05\x00".to_vec();
    bytes.extend_from_slice(
        &u32::try_from(10 + payload.len())
            .expect("Effect fixture size")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&payload);
    bytes
}

fn sprite_font_xnb() -> Vec<u8> {
    const READERS: &[&str] = &[
        "Microsoft.Xna.Framework.Content.SpriteFontReader",
        "Microsoft.Xna.Framework.Content.Texture2DReader",
        "Microsoft.Xna.Framework.Content.ListReader`1[[Microsoft.Xna.Framework.Rectangle]]",
        "Microsoft.Xna.Framework.Content.ListReader`1[[System.Char]]",
        "Microsoft.Xna.Framework.Content.ListReader`1[[Microsoft.Xna.Framework.Vector3]]",
        "Microsoft.Xna.Framework.Content.RectangleReader",
        "Microsoft.Xna.Framework.Content.CharReader",
        "Microsoft.Xna.Framework.Content.Vector3Reader",
    ];
    let mut payload = Vec::new();
    write_7bit(&mut payload, READERS.len());
    for reader in READERS {
        write_xnb_string(&mut payload, reader);
        payload.extend_from_slice(&0_i32.to_le_bytes());
    }
    write_7bit(&mut payload, 0);
    payload.push(1); // SpriteFont root reader.
    payload.push(2); // Texture2D atlas reader.
    payload.extend_from_slice(&0_i32.to_le_bytes()); // SurfaceFormat.Color.
    payload.extend_from_slice(&1_i32.to_le_bytes());
    payload.extend_from_slice(&1_i32.to_le_bytes());
    payload.extend_from_slice(&1_i32.to_le_bytes()); // One mip.
    payload.extend_from_slice(&4_i32.to_le_bytes());
    payload.extend_from_slice(&[255, 255, 255, 255]);
    payload.push(3); // Glyph Rectangle list.
    payload.extend_from_slice(&1_i32.to_le_bytes());
    write_rectangle(&mut payload, Rectangle::new(0, 0, 1, 1));
    payload.push(3); // Cropping Rectangle list.
    payload.extend_from_slice(&1_i32.to_le_bytes());
    write_rectangle(&mut payload, Rectangle::new(0, 0, 1, 1));
    payload.push(4); // Character list.
    payload.extend_from_slice(&1_i32.to_le_bytes());
    payload.extend_from_slice(&u16::from(b'?').to_le_bytes());
    payload.extend_from_slice(&2_i32.to_le_bytes()); // Line spacing.
    payload.extend_from_slice(&0_f32.to_le_bytes()); // Extra spacing.
    payload.push(5); // Kerning Vector3 list.
    payload.extend_from_slice(&1_i32.to_le_bytes());
    payload.extend_from_slice(&0_f32.to_le_bytes());
    payload.extend_from_slice(&1_f32.to_le_bytes());
    payload.extend_from_slice(&0_f32.to_le_bytes());
    payload.push(1); // Has default character.
    payload.extend_from_slice(&u16::from(b'?').to_le_bytes());

    let mut bytes = b"XNBw\x05\x00".to_vec();
    bytes.extend_from_slice(
        &u32::try_from(10 + payload.len())
            .expect("SpriteFont fixture size")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&payload);
    bytes
}

fn write_7bit(bytes: &mut Vec<u8>, mut value: usize) {
    loop {
        let mut next = u8::try_from(value & 0x7f).expect("seven-bit fixture chunk");
        value >>= 7;
        if value != 0 {
            next |= 0x80;
        }
        bytes.push(next);
        if value == 0 {
            return;
        }
    }
}

fn write_xnb_string(bytes: &mut Vec<u8>, value: &str) {
    write_7bit(bytes, value.len());
    bytes.extend_from_slice(value.as_bytes());
}

fn write_rectangle(bytes: &mut Vec<u8>, value: Rectangle) {
    bytes.extend_from_slice(&value.X.to_le_bytes());
    bytes.extend_from_slice(&value.Y.to_le_bytes());
    bytes.extend_from_slice(&value.Width.to_le_bytes());
    bytes.extend_from_slice(&value.Height.to_le_bytes());
}

impl GameStateAccess for BufferTransferGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for BufferTransferGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let mut device = game.GraphicsDevice()?;
        let declaration = VertexPositionColor::VertexDeclaration();
        assert_eq!(declaration.VertexStride(), 16);

        let mut vertex = VertexBuffer::new(&device, declaration, 3, BufferUsage::None)?;
        let vertices = [
            VertexPositionColor::new(Vector3::Zero, Color::Red),
            VertexPositionColor::new(Vector3::UnitX, Color::Green),
            VertexPositionColor::new(Vector3::UnitY, Color::Blue),
        ];
        vertex.SetData(&vertices)?;
        let mut vertex_readback = [VertexPositionColor::default(); 3];
        vertex.GetData(&mut vertex_readback)?;
        assert_eq!(vertex_readback, vertices);

        let typed = VertexBuffer::from_graphics_device_and_vertex_type_and_vertex_count_and_usage(
            &device,
            TypeId::of::<VertexPositionColor>(),
            3,
            BufferUsage::None,
        )?;
        typed.SetData(&vertices)?;
        assert!(typed
            .VertexDeclaration()
            .GetVertexElements()
            .iter()
            .eq(declaration.GetVertexElements().iter()));

        let custom = VertexBuffer::new(
            &device,
            CustomVertex::vertex_declaration(),
            2,
            BufferUsage::None,
        )?;
        let custom_data = [
            CustomVertex { x: 1.0, y: 2.0 },
            CustomVertex { x: 3.0, y: 4.0 },
        ];
        custom.SetData(&custom_data)?;
        let mut custom_readback = [CustomVertex::default(); 2];
        custom.GetData(&mut custom_readback)?;
        assert_eq!(custom_readback[0].x, 1.0);
        assert_eq!(custom_readback[1].y, 4.0);

        let dynamic = DynamicVertexBuffer::new(&device, declaration, 3, BufferUsage::None)?;
        dynamic.SetData(&vertices, 0, 3, SetDataOptions::Discard)?;
        dynamic.SetData(&vertices, 0, 3, SetDataOptions::NoOverwrite)?;
        assert!(!dynamic.IsContentLost()?);
        // A byte offset combined with a streaming hint. It is not enough that
        // the call succeeds: the vertex has to land in slot one and leave slot
        // zero alone, which is the whole meaning of `offsetInBytes`.
        dynamic
            .SetDataWithOffsetInBytesAndDataAndStartIndexAndElementCountAndVertexStrideAndOptions(
                16,
                &[VertexPositionColor::new(Vector3::UnitZ, Color::Yellow)],
                0,
                1,
                16,
                SetDataOptions::NoOverwrite,
            )?;
        let mut streamed = [VertexPositionColor::default(); 3];
        dynamic.GetData(&mut streamed)?;
        assert_eq!(streamed[0], vertices[0]);
        assert_eq!(
            streamed[1],
            VertexPositionColor::new(Vector3::UnitZ, Color::Yellow)
        );
        assert_eq!(streamed[2], vertices[2]);
        // A user-declared vertex type with a streaming hint. CNA's typed
        // transfer route only knows the built-in XNA layouts, so this is the
        // byte route carrying the option, and the readback proves it wrote.
        let dynamic_custom = DynamicVertexBuffer::new(
            &device,
            CustomVertex::vertex_declaration(),
            2,
            BufferUsage::None,
        )?;
        dynamic_custom.SetData(
            &[
                CustomVertex { x: 5.0, y: 6.0 },
                CustomVertex { x: 7.0, y: 8.0 },
            ],
            0,
            2,
            SetDataOptions::Discard,
        )?;
        let mut custom_streamed = [CustomVertex::default(); 2];
        dynamic_custom.GetData(&mut custom_streamed)?;
        assert_eq!(custom_streamed[0].x, 5.0);
        assert_eq!(custom_streamed[1].y, 8.0);

        let binding = VertexBufferBinding::from_vertex_buffer_and_vertex_offset(&vertex, 1)?;
        device.SetVertexBuffers(&[binding.clone()])?;
        assert!(device.GetVertexBuffers()? == [binding]);
        assert!(matches!(
            vertex.DisposeWithNoArguments(),
            Err(CnaError::InvalidInput(_))
        ));
        device.SetVertexBuffers(&[])?;
        vertex.DisposeWithNoArguments()?;
        vertex.DisposeWithNoArguments()?;

        let indices16 =
            IndexBuffer::new(&device, IndexElementSize::SixteenBits, 3, BufferUsage::None)?;
        indices16.SetData(&[0_u16, 1, 2])?;
        let mut read16 = [0_u16; 3];
        indices16.GetData(&mut read16)?;
        assert_eq!(read16, [0, 1, 2]);

        let indices32 = IndexBuffer::new(
            &device,
            IndexElementSize::ThirtyTwoBits,
            3,
            BufferUsage::None,
        )?;
        indices32.SetData(&[2_u32, 1, 0])?;
        let mut read32 = [0_u32; 3];
        indices32.GetData(&mut read32)?;
        assert_eq!(read32, [2, 1, 0]);

        let dynamic_index =
            DynamicIndexBuffer::new(&device, IndexElementSize::SixteenBits, 3, BufferUsage::None)?;
        dynamic_index.SetData(&[0_u16, 1, 2], 0, 3, SetDataOptions::Discard)?;
        dynamic_index.SetData(&[0_u16, 1, 2], 0, 3, SetDataOptions::NoOverwrite)?;
        assert!(matches!(
            dynamic_index.SetDataWithOffsetInBytesAndDataAndStartIndexAndElementCountAndOptions(
                2,
                &[2_u16],
                0,
                1,
                SetDataOptions::NoOverwrite,
            ),
            Err(CnaError::InvalidInput(_))
        ));
        dynamic_index.SetDataWithOffsetInBytesAndDataAndStartIndexAndElementCountAndOptions(
            2,
            &[2_u16],
            0,
            1,
            SetDataOptions::None,
        )?;
        assert!(!dynamic_index.IsContentLost()?);

        device.SetVertexBufferWithVertexBuffer(&typed)?;
        device.SetIndices(Some(&indices32))?;
        assert!(device.Indices()?.is_some());

        assert!(matches!(
            device.DrawPrimitives(PrimitiveType::TriangleList, 1, 1),
            Err(CnaError::InvalidInput(_))
        ));
        assert_missing_effect(device.DrawPrimitives(PrimitiveType::TriangleList, 0, 1));
        assert_missing_effect(device.DrawIndexedPrimitives(
            PrimitiveType::TriangleList,
            0,
            0,
            3,
            0,
            1,
        ));
        assert_missing_effect(device.DrawInstancedPrimitives(
            PrimitiveType::TriangleList,
            0,
            0,
            3,
            0,
            1,
            1,
        ));

        assert!(matches!(
            device.DrawUserPrimitives(PrimitiveType::TriangleList, &vertices[..2], 0, 1),
            Err(CnaError::InvalidInput(_))
        ));
        assert_missing_effect(device.DrawUserPrimitives(
            PrimitiveType::TriangleList,
            &vertices,
            0,
            1,
        ));
        assert_missing_effect(device.DrawUserPrimitivesWithPrimitiveTypeAndVertexDataAndVertexOffsetAndPrimitiveCountAndVertexDeclaration(
            PrimitiveType::TriangleList,
            &vertices,
            0,
            1,
            declaration,
        ));

        let indices_i32 = [0_i32, 1, 2];
        let indices_i16 = [0_i16, 1, 2];
        assert!(matches!(
            device.DrawUserIndexedPrimitives(
                PrimitiveType::TriangleList,
                &vertices,
                0,
                3,
                &[0_i32, 1, 3],
                0,
                1,
            ),
            Err(CnaError::InvalidInput(_))
        ));
        assert_missing_effect(device.DrawUserIndexedPrimitives(
            PrimitiveType::TriangleList,
            &vertices,
            0,
            3,
            &indices_i32,
            0,
            1,
        ));
        assert_missing_effect(device.DrawUserIndexedPrimitivesWithPrimitiveTypeAndVertexDataAndVertexOffsetAndNumVerticesAndIndexDataAndIndexOffsetAndPrimitiveCount(
            PrimitiveType::TriangleList,
            &vertices,
            0,
            3,
            &indices_i16,
            0,
            1,
        ));
        assert_missing_effect(device.DrawUserIndexedPrimitivesWithPrimitiveTypeAndVertexDataAndVertexOffsetAndNumVerticesAndIndexDataAndIndexOffsetAndPrimitiveCountAndVertexDeclarationAsPrimitiveTypeAnd0ArrayAndInt32AndInt32AndInt32ArrayAndInt32AndInt32AndVertexDeclaration(
            PrimitiveType::TriangleList,
            &vertices,
            0,
            3,
            &indices_i32,
            0,
            1,
            declaration,
        ));
        assert_missing_effect(device.DrawUserIndexedPrimitivesWithPrimitiveTypeAndVertexDataAndVertexOffsetAndNumVerticesAndIndexDataAndIndexOffsetAndPrimitiveCountAndVertexDeclarationAsPrimitiveTypeAnd0ArrayAndInt32AndInt32AndInt16ArrayAndInt32AndInt32AndVertexDeclaration(
            PrimitiveType::TriangleList,
            &vertices,
            0,
            3,
            &indices_i16,
            0,
            1,
            declaration,
        ));

        device.ClearWithColor(Color::CornflowerBlue)?;
        // The mapped depth/stencil clear, which is a route this binding used
        // to refuse outright. All three option masks reach CNA.
        device.ClearWithOptionsAndColorAndDepthAndStencil(
            ClearOptions::Target,
            Color::CornflowerBlue,
            1.0,
            0,
        )?;
        device.ClearWithOptionsAndColorAndDepthAndStencil(
            ClearOptions::Target | ClearOptions::DepthBuffer,
            Color::Black,
            1.0,
            0,
        )?;
        device.ClearWithOptionsAndColorAndDepthAndStencil(
            ClearOptions::DepthBuffer | ClearOptions::Stencil,
            Color::Black,
            0.5,
            7,
        )?;
        // XNA packs the Vector4 overload's color through `new Color(color)`
        // before the device sees it, so the two overloads are the same call.
        device.Clear(
            ClearOptions::Target,
            Color::CornflowerBlue.ToVector4(),
            1.0,
            0,
        )?;
        // CNA argument validation still reaches the caller unchanged.
        assert!(device
            .ClearWithOptionsAndColorAndDepthAndStencil(
                ClearOptions::Target,
                Color::Black,
                f32::NAN,
                0,
            )
            .is_err());
        // The CNA-only float clear keeps channels XNA cannot express.
        device.clear_color_channels([0.25, 0.5, 0.75, 1.0])?;
        assert!(device
            .clear_color_channels([f32::INFINITY, 0.0, 0.0, 1.0])
            .is_err());
        let mut region = [Color::Transparent; 2];
        assert_headless_readback_unsupported(device.GetBackBufferData(
            Some(Rectangle::new(0, 0, 1, 1)),
            &mut region,
            1,
            1,
        ));
        assert_eq!(region, [Color::Transparent; 2]);
        let parameters = device.PresentationParameters()?.Clone();
        let pixel_count = usize::try_from(
            parameters
                .BackBufferWidth()
                .checked_mul(parameters.BackBufferHeight())
                .expect("back-buffer pixel count"),
        )
        .expect("positive back-buffer pixel count");
        let mut complete = vec![Color::Transparent; pixel_count];
        assert_headless_readback_unsupported(device.GetBackBufferDataWithData(&mut complete));
        assert!(complete.iter().all(|pixel| *pixel == Color::Transparent));
        let mut offset_complete = vec![Color::Transparent; pixel_count + 1];
        assert_headless_readback_unsupported(
            device.GetBackBufferDataWithDataAndStartIndexAndElementCount(
                &mut offset_complete,
                1,
                pixel_count as i32,
            ),
        );
        assert_eq!(offset_complete[0], Color::Transparent);
        assert!(offset_complete[1..]
            .iter()
            .all(|pixel| *pixel == Color::Transparent));

        let reset_events = Arc::new(Mutex::new(Vec::new()));
        let resetting_events = Arc::clone(&reset_events);
        device.AddDeviceResettingHandler(Box::new(
            move |_: &dyn std::any::Any, _: cna::extensions::events::EventArgs| {
                resetting_events
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push("resetting");
            },
        ));
        let reset_complete_events = Arc::clone(&reset_events);
        device.AddDeviceResetHandler(Box::new(
            move |_: &dyn std::any::Any, _: cna::extensions::events::EventArgs| {
                reset_complete_events
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push("reset");
            },
        ));
        device.Reset()?;
        device.ResetWithPresentationParameters(&parameters)?;
        device.ResetWithPresentationParametersAndGraphicsAdapter(&parameters, device.Adapter()?)?;
        assert_eq!(
            *reset_events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            [
                "resetting",
                "reset",
                "resetting",
                "reset",
                "resetting",
                "reset"
            ]
        );
        assert_eq!(device.GetVertexBuffers()?.len(), 1);
        assert!(device.Indices()?.is_some());

        let texture_cube = TextureCube::new(&device, 1, false, SurfaceFormat::Color)?;
        let mut cube_readback = [Color::Transparent];
        match texture_cube.SetData(CubeMapFace::PositiveX, &[Color::Red]) {
            Ok(()) => {
                texture_cube.GetData(CubeMapFace::PositiveX, &mut cube_readback)?;
                assert_eq!(cube_readback, [Color::Red]);
            }
            Err(CnaError::Native { code: 6, .. }) => {
                assert_native_not_supported(
                    texture_cube.GetData(CubeMapFace::PositiveX, &mut cube_readback),
                );
                assert_eq!(cube_readback, [Color::Transparent]);
            }
            Err(error) => return Err(error),
        }

        let mut render_target2d = RenderTarget2D::new(&device, 1, 1)?;
        let render_target2d_other = RenderTarget2D::new(&device, 2, 2)?;
        let binding2d = RenderTargetBinding::new(&render_target2d)?;
        let binding2d_other = RenderTargetBinding::new(&render_target2d_other)?;
        assert!(matches!(
            device.SetRenderTargets(&[binding2d.clone(), binding2d.clone()]),
            Err(CnaError::InvalidInput(_))
        ));
        assert!(matches!(
            device.SetRenderTargets(&[binding2d.clone(), binding2d_other]),
            Err(CnaError::InvalidInput(_))
        ));
        match device.SetRenderTargetWithRenderTarget(Some(&render_target2d)) {
            Ok(()) => {
                let observed = device.GetRenderTargets();
                let dispose_while_bound = render_target2d.DisposeWithNoArguments();
                device.SetRenderTargets(&[])?;
                assert_eq!(observed?.len(), 1);
                assert!(matches!(
                    dispose_while_bound,
                    Err(CnaError::InvalidInput(_))
                ));
            }
            Err(CnaError::Native { code: 6, .. }) => {
                assert!(device.GetRenderTargets()?.is_empty());
            }
            Err(error) => return Err(error),
        }

        let render_target_cube = RenderTargetCube::new(
            &device,
            1,
            false,
            SurfaceFormat::Color,
            cna::Microsoft::Xna::Framework::Graphics::DepthFormat::None,
        )?;
        match device.SetRenderTarget(Some(&render_target_cube), CubeMapFace::NegativeZ) {
            Ok(()) => {
                let observed = device.GetRenderTargets();
                device.SetRenderTargets(&[])?;
                let observed = observed?;
                assert_eq!(observed.len(), 1);
                assert_eq!(observed[0].CubeMapFace(), CubeMapFace::NegativeZ);
            }
            Err(CnaError::Native { code: 6, .. }) => {
                assert!(device.GetRenderTargets()?.is_empty());
            }
            Err(error) => return Err(error),
        }
        device.SetRenderTargets(&[])?;

        self.vertex = Some(typed);
        self.index = Some(indices32);
        self.texture_cube = Some(texture_cube);
        self.render_target2d = Some(render_target2d);
        self.render_target_cube = Some(render_target_cube);
        Ok(())
    }

    fn UnloadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let _ = game;
        assert!(self
            .vertex
            .as_ref()
            .expect("retained vertex buffer")
            .IsDisposed());
        assert!(self
            .index
            .as_ref()
            .expect("retained index buffer")
            .IsDisposed());
        assert!(self
            .texture_cube
            .as_ref()
            .expect("retained cube texture")
            .IsDisposed());
        assert!(self
            .render_target2d
            .as_ref()
            .expect("retained 2D render target")
            .IsDisposed());
        assert!(self
            .render_target_cube
            .as_ref()
            .expect("retained cube render target")
            .IsDisposed());
        Ok(())
    }
}

fn assert_missing_effect(result: Result<()>) {
    // The category is CNA's own classification, read from the same
    // thread-local diagnostic as the message; asserting it here is what proves
    // the binding reads a real value rather than defaulting to None.
    assert!(matches!(
        result,
        Err(CnaError::Native { code: 12, category: ErrorCategory::Internal, message })
            if message.contains("no effect has been applied")
    ));
}

fn assert_headless_readback_unsupported(result: Result<()>) {
    assert!(matches!(
        result,
        Err(CnaError::Native { code: 6, category: ErrorCategory::NotSupported, message })
            if message.contains("Headless renderer does not rasterize")
    ));
}

fn assert_native_not_supported(result: Result<()>) {
    assert!(matches!(
        result,
        Err(CnaError::Native {
            code: 6,
            category: ErrorCategory::NotSupported,
            ..
        })
    ));
}

#[derive(Default)]
struct FaultTextureInfoGame {
    state: Arc<GameState>,
}

impl GameStateAccess for FaultTextureInfoGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for FaultTextureInfoGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let device = game.GraphicsDevice()?;
        let _ = Texture2D::new(&device, 1, 1)?;
        Ok(())
    }
}

#[test]
fn native_stress_isolated() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    if let Ok(case) = std::env::var(CHILD_CASE) {
        run_child_case(&case);
        return;
    }

    for case in [
        "lifecycle-100",
        "resources-25",
        "texture-transfers-10",
        "buffer-transfers-10",
        "sprite-font-xnb-10",
        "effect-xnb-1",
        "model-xnb-10",
        "remaining-graphics-10",
        "effect-graph-10",
        "small-families-10",
        "small-family-callback-panic",
        "lifecycle-order-and-identity",
        "component-order-and-mutation",
        "callback-panic",
        "callback-event-panic",
        "fault-game-create",
        "fault-texture-info",
        "fault-game-destroy",
        "independent-graphics-device",
    ] {
        let status = Command::new(std::env::current_exe().expect("current test executable"))
            .args(["--exact", "native_stress_isolated"])
            .env(CHILD_CASE, case)
            .status()
            .expect("start isolated native stress child");
        assert!(
            status.success(),
            "native stress child failed: {case}: {status}"
        );
    }
}

/// Presentation parameters for an independently constructed device.
fn independent_presentation(width: i32, height: i32) -> PresentationParameters {
    let parameters = PresentationParameters::new();
    parameters.SetBackBufferWidth(width);
    parameters.SetBackBufferHeight(height);
    parameters.SetBackBufferFormat(SurfaceFormat::Color);
    parameters.SetDepthStencilFormat(DepthFormat::Depth24);
    parameters
}

/// A `GraphicsAdapter` reachable before any device exists.
///
/// XNA's `GraphicsAdapter.DefaultAdapter` is static; this projection reaches
/// the same default through `GraphicsDeviceInformation`, whose public
/// constructor sets `Adapter` to it. Nothing here borrows a device, which is
/// the point: the first independent device has none to borrow.
fn independent_adapter() -> Arc<GraphicsAdapter> {
    GraphicsDeviceInformation::new().Adapter()
}

/// Proves that `GraphicsDevice::new` produces an *owned* device.
///
/// The distinction being tested is ownership, not construction: a game's
/// device is borrowed for a callback and refuses to dispose itself, while this
/// one is created, used, and destroyed entirely by this crate, with no game in
/// the process at all.
fn independent_graphics_device_case() {
    let adapter = independent_adapter();

    // 1. Construction with no game anywhere.
    let mut device =
        GraphicsDevice::new(&adapter, GraphicsProfile::Reach, &independent_presentation(320, 240))
            .expect("independent GraphicsDevice construction");

    // 2. State immediately after construction is the requested state.
    assert!(!device.IsDisposed().expect("fresh device is not disposed"));
    assert_eq!(
        device.GraphicsProfile().expect("profile of a fresh device"),
        GraphicsProfile::Reach
    );
    assert_eq!(
        device
            .GraphicsDeviceStatus()
            .expect("status of a fresh device"),
        GraphicsDeviceStatus::Normal
    );
    let parameters = device
        .PresentationParameters()
        .expect("presentation parameters of a fresh device");
    assert_eq!(parameters.BackBufferWidth(), 320);
    assert_eq!(parameters.BackBufferHeight(), 240);
    assert_eq!(parameters.BackBufferFormat(), SurfaceFormat::Color);
    assert_eq!(parameters.DepthStencilFormat(), DepthFormat::Depth24);
    // An independent device presents to no window, and says so rather than
    // naming some other window.
    assert_eq!(
        parameters.DeviceWindowHandle(),
        PresentationParameters::new().DeviceWindowHandle()
    );

    // A device outside a callback is nevertheless usable: that is exactly what
    // distinguishes it from the game-owned device, whose handle is borrowed
    // only for the duration of one callback.
    device
        .ClearWithColor(Color::CornflowerBlue)
        .expect("clear on an independent device outside any callback");

    // 3. Child resources, with exact data rather than a success code.
    let texture = Texture2D::new(&device, 2, 2).expect("texture on an independent device");
    assert_eq!((texture.Width(), texture.Height()), (2, 2));
    assert_eq!(texture.LevelCount(), 1);
    let written = [Color::Red, Color::Green, Color::Blue, Color::White];
    texture.SetData(&written).expect("upload to owned texture");
    let mut read = [Color::Transparent; 4];
    texture.GetData(&mut read).expect("download from owned texture");
    assert_eq!(read, written);

    // 4. Two independent devices coexist, and each one owns its own resources.
    let second =
        GraphicsDevice::new(&adapter, GraphicsProfile::HiDef, &independent_presentation(64, 48))
            .expect("a second independent device");
    assert_eq!(
        second.GraphicsProfile().expect("second device profile"),
        GraphicsProfile::HiDef
    );
    assert_eq!(
        second
            .PresentationParameters()
            .expect("second device parameters")
            .BackBufferWidth(),
        64
    );
    // Both stay usable while the other lives. Their distinctness is proved
    // below by the cross-device refusal and by one surviving the other's
    // disposal, rather than by an identity member XNA does not declare.
    device.ClearWithColor(Color::Black).expect("first device still usable");
    second.ClearWithColor(Color::White).expect("second device usable");

    // 5. A resource belongs to the device that made it. CNA itself was
    //    measured to accept a foreign texture in a sampler slot, so the
    //    refusal here is this crate's, and it must not weaken.
    let foreign = second
        .Textures()
        .expect("second device sampler collection")
        .SetItem(0, Some(Arc::new(Texture2D::new(&device, 1, 1).expect("device texture"))
            as Arc<dyn Texture>));
    assert!(
        matches!(foreign, Err(CnaError::InvalidInput(message))
            if message.contains("different graphics device")),
        "a texture from another device must be refused, got {foreign:?}"
    );

    // 6. Disposal raises Disposing exactly once and releases the children.
    let disposals = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&disposals);
    device.AddDisposingHandler(Box::new(move |sender: &dyn std::any::Any, _| {
        let device = sender
            .downcast_ref::<GraphicsDevice>()
            .expect("GraphicsDevice disposing sender");
        // XNA sets isDisposed before raising the event.
        assert!(device.IsDisposed().expect("state inside Disposing"));
        observed.fetch_add(1, Ordering::SeqCst);
    }));
    device
        .DisposeWithNoArguments()
        .expect("independent device disposes itself");
    assert_eq!(disposals.load(Ordering::SeqCst), 1);
    assert!(device.IsDisposed().expect("disposed device reports it"));
    // XNA's release disposes every device resource, and CNA leaves them alive,
    // so the crate must have released this one itself.
    assert!(texture.IsDisposed(), "device disposal releases its resources");

    // 7. Stale use refuses deterministically rather than reaching CNA.
    let stale = device.ClearWithColor(Color::Black);
    assert!(
        matches!(stale, Err(CnaError::InvalidInput("graphics device is disposed"))),
        "a disposed device refuses work, got {stale:?}"
    );
    assert!(device.PresentationParameters().is_err());
    assert!(Texture2D::new(&device, 1, 1).is_err());

    // 8. A repeated Dispose is silent and raises nothing further, which is
    //    what XNA's `~GraphicsDevice` does once `isDisposed` is set.
    device
        .DisposeWithNoArguments()
        .expect("repeating Dispose is a no-op");
    assert_eq!(disposals.load(Ordering::SeqCst), 1);

    // 9. The other device is untouched by the first one's disposal.
    assert!(!second.IsDisposed().expect("second device still alive"));
    second
        .ClearWithColor(Color::White)
        .expect("second device survives the first one's disposal");

    // 10. A game's device still refuses to dispose itself: adding an owned
    //     device must not have relaxed the borrowed one. The same game also
    //     creates a second device while it is running and keeps drawing,
    //     which is the Rust-side check on cnanext a2013068 "preserve active
    //     GL context across secondary devices": before that fix a secondary
    //     device stole the running game's context.
    run_for_frames(BorrowedDeviceDisposalGame::default(), 2)
        .expect("a game device refuses self-disposal and survives a secondary device");

    // 11. Dropping the last clone releases a device that was never disposed,
    //     and a live child keeps it alive until the child goes first.
    let dropped = GraphicsDevice::new(
        &adapter,
        GraphicsProfile::Reach,
        &independent_presentation(32, 32),
    )
    .expect("a device released by Drop alone");
    let child = Texture2D::new(&dropped, 1, 1).expect("child of the dropped device");
    drop(dropped);
    // The child holds a device clone, so it is still usable after the last
    // caller-visible handle is gone.
    child
        .SetData(&[Color::Red])
        .expect("child outlives the caller's device handle");
    drop(child);

    // 12. Repeated construction, use and release. A native handle this crate
    //     failed to destroy would not be visible through the safe API -- the
    //     wrapper has already forgotten it -- so this is a cycle test in the
    //     same sense as the other stress cases, and it is the sanitizer path
    //     rather than this assertion that would show a leak.
    for cycle in 0..25 {
        let mut device = GraphicsDevice::new(
            &adapter,
            if cycle % 2 == 0 {
                GraphicsProfile::Reach
            } else {
                GraphicsProfile::HiDef
            },
            &independent_presentation(16 + cycle, 16),
        )
        .expect("independent device cycle");
        let texture = Texture2D::new(&device, 1, 1).expect("cycle texture");
        texture.SetData(&[Color::Red]).expect("cycle upload");
        assert_eq!(
            device
                .PresentationParameters()
                .expect("cycle parameters")
                .BackBufferWidth(),
            16 + cycle
        );
        if cycle % 3 == 0 {
            // Release through Dispose on some cycles and through Drop on the
            // others, so neither path is exercised only once.
            device.DisposeWithNoArguments().expect("cycle disposal");
            assert!(texture.IsDisposed());
        }
    }
}

/// A game whose device must keep refusing to dispose itself.
#[derive(Default)]
struct BorrowedDeviceDisposalGame {
    state: Arc<GameState>,
    frames_seen: Arc<AtomicUsize>,
}

impl GameStateAccess for BorrowedDeviceDisposalGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for BorrowedDeviceDisposalGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let mut device = game.GraphicsDevice()?;
        let refusal = device.DisposeWithNoArguments();
        assert!(
            matches!(refusal, Err(CnaError::UnsupportedRuntime(message))
                if message.contains("reserves GraphicsDevice disposal to the owning Game")),
            "a borrowed device must refuse self-disposal, got {refusal:?}"
        );
        assert!(!device.IsDisposed()?, "a refused Dispose changes nothing");

        // Create an owned device while this game is running, use it, and
        // release it. The game's own device must keep working throughout and
        // afterwards; a secondary device that took the current context would
        // show up as the game's own draw failing.
        let mut secondary = GraphicsDevice::new(
            &independent_adapter(),
            GraphicsProfile::Reach,
            &independent_presentation(48, 48),
        )?;
        secondary.ClearWithColor(Color::Red)?;
        let owned = Texture2D::new(&secondary, 1, 1)?;
        owned.SetData(&[Color::Green])?;
        device.ClearWithColor(Color::CornflowerBlue)?;
        let borrowed = Texture2D::new(&device, 1, 1)?;
        borrowed.SetData(&[Color::Blue])?;
        let mut read = [Color::Transparent];
        borrowed.GetData(&mut read)?;
        assert_eq!(read, [Color::Blue], "the game's device still works");
        secondary.DisposeWithNoArguments()?;
        device.ClearWithColor(Color::CornflowerBlue)?;
        borrowed.GetData(&mut read)?;
        assert_eq!(
            read,
            [Color::Blue],
            "the game's device survives a secondary device's whole life"
        );
        self.frames_seen.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn Draw(&mut self, game: &mut GameContext<'_>, _time: &GameTime) -> Result<()> {
        // Drawing after the secondary device existed is the actual context
        // check: LoadContent alone would not exercise a later frame.
        let device = game.GraphicsDevice()?;
        device.ClearWithColor(Color::CornflowerBlue)?;
        self.frames_seen.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

fn run_child_case(case: &str) {
    match case {
        "independent-graphics-device" => independent_graphics_device_case(),
        "lifecycle-100" => {
            for _ in 0..100 {
                run_for_frames(EmptyGame::default(), 1).expect("create/run/destroy cycle");
            }
        }
        "resources-25" => {
            for _ in 0..25 {
                run_for_frames(ResourceGame::default(), 1)
                    .expect("resource create/double-dispose/drop cycle");
            }
        }
        "texture-transfers-10" => {
            for _ in 0..10 {
                run_for_frames(TextureTransferGame::default(), 1)
                    .expect("Texture2D transfer/validation/event cycle");
            }
        }
        "buffer-transfers-10" => {
            for _ in 0..10 {
                run_for_frames(BufferTransferGame::default(), 1)
                    .expect("buffer transfer/binding/ownership cycle");
            }
        }
        "sprite-font-xnb-10" => {
            let fixture = SpriteFontFixture::new();
            for _ in 0..10 {
                let retained_font = Arc::new(Mutex::new(None));
                run_for_frames(
                    SpriteFontXnbGame {
                        state: Arc::new(GameState::new()),
                        root: fixture.0.clone(),
                        retained_font: Arc::clone(&retained_font),
                    },
                    1,
                )
                .expect("SpriteFont XNB/content/draw-string ownership cycle");
                let font = retained_font
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take()
                    .expect("retained SpriteFont after game shutdown");
                assert!(font.MeasureString("?").is_err());
            }
        }
        "effect-xnb-1" => {
            let fixture = EffectFixture::new();
            run_for_frames(
                EffectXnbGame {
                    state: Arc::new(GameState::new()),
                    root: fixture.0.clone(),
                },
                1,
            )
            .expect("Effect XNB reader pipeline and backend failure cycle");
        }
        "model-xnb-10" => {
            let fixture = ModelFixture::new();
            for _ in 0..10 {
                let retained_model = Arc::new(Mutex::new(None));
                let retained_bone = Arc::new(Mutex::new(None));
                run_for_frames(
                    ModelXnbGame {
                        state: Arc::new(GameState::new()),
                        root: fixture.0.clone(),
                        retained_model: Arc::clone(&retained_model),
                        retained_bone: Arc::clone(&retained_bone),
                    },
                    1,
                )
                .expect("Model XNB/shared-resource/draw/unload ownership cycle");
                assert!(retained_model
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .as_ref()
                    .expect("retained Model")
                    .Bones()
                    .is_err());
                assert!(retained_bone
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .as_ref()
                    .expect("retained ModelBone")
                    .Name()
                    .is_err());
            }
        }
        "remaining-graphics-10" => {
            for _ in 0..10 {
                let game = RemainingGraphicsGame::default();
                run_for_frames(game, 1)
                    .expect("stock-effect/Texture3D/OcclusionQuery ownership cycle");
            }
        }
        "effect-graph-10" => {
            for _ in 0..10 {
                run_for_frames(EffectStressGame::default(), 1)
                    .expect("Effect reflection/parameter/pass/SpriteBatch ownership cycle");
            }
        }
        "small-families-10" => {
            let changed = StorageDevice::AddDeviceChangedHandler(Box::new(
                |_: &dyn std::any::Any, _: cna::extensions::events::EventArgs| {},
            ));
            assert!(StorageDevice::RemoveDeviceChangedHandler(changed));
            for index in 0..10 {
                run_for_frames(small_family_stress_game(), 1)
                    .expect("Framework/Touch/GamerServices ownership cycle");
                storage_stress_cycle(index);
            }
        }
        "small-family-callback-panic" => {
            assert!(matches!(
                run_for_frames(panic_preparing_stress_game(), 1),
                Err(CnaError::Callback(_))
            ));
            assert!(matches!(
                StorageDevice::BeginShowSelectorWithCallbackAndState(
                    Some(Box::new(|_| panic!("intentional storage completion panic"))),
                    None,
                ),
                Err(CnaError::Callback(_))
            ));
            run_for_frames(EmptyGame::default(), 1)
                .expect("game recreation after small-family callback panic");
        }
        "lifecycle-order-and-identity" => {
            let evidence = Arc::new(Mutex::new(LifecycleEvidence::default()));
            let device = Arc::new(Mutex::new(None));
            let texture = Arc::new(Mutex::new(None));
            let batch = Arc::new(Mutex::new(None));
            run_for_frames(
                LifecycleEvidenceGame {
                    state: Arc::new(GameState::new()),
                    evidence: Arc::clone(&evidence),
                    device: Arc::clone(&device),
                    texture: Arc::clone(&texture),
                    batch: Arc::clone(&batch),
                },
                1,
            )
            .expect("lifecycle/identity evidence game");

            let evidence = evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            assert_eq!(
                evidence.events,
                [
                    "Initialize",
                    "LoadContent",
                    "BeginRun",
                    "Update",
                    "BeginDraw",
                    "Draw",
                    "EndDraw",
                    "Exiting",
                    "EndRun",
                    "UnloadContent",
                    "DeviceDisposing",
                    "Dispose",
                    "DisposedEvent"
                ]
            );
            assert!(evidence.unload_resources_disposed);
            drop(evidence);

            assert!(device
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_ref()
                .expect("retained device")
                .IsDisposed()
                .expect("device state"));
            assert!(matches!(
                device
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .as_ref()
                    .expect("retained device")
                    .SamplerStates(),
                Err(CnaError::InvalidInput("graphics device is disposed"))
            ));
            let mut texture = texture
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let texture = texture.as_mut().expect("retained texture");
            assert!(texture.IsDisposed());
            texture
                .DisposeWithNoArguments()
                .expect("dispose texture after parent shutdown");
            let mut batch = batch
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let batch = batch.as_mut().expect("retained batch");
            assert!(batch.IsDisposed());
            batch
                .DisposeWithNoArguments()
                .expect("dispose batch after parent shutdown");
            assert!(matches!(
                batch.Begin(),
                Err(CnaError::InvalidInput("graphics device is disposed"))
            ));

            let begin_draws = Arc::new(AtomicUsize::new(0));
            let draws = Arc::new(AtomicUsize::new(0));
            let end_draws = Arc::new(AtomicUsize::new(0));
            run_for_frames(
                SuppressFirstDrawGame {
                    state: Arc::new(GameState::new()),
                    begin_draws: Arc::clone(&begin_draws),
                    draws: Arc::clone(&draws),
                    end_draws: Arc::clone(&end_draws),
                },
                1,
            )
            .expect("BeginDraw=false suppression then successful draw");
            assert_eq!(begin_draws.load(Ordering::SeqCst), 2);
            assert_eq!(draws.load(Ordering::SeqCst), 1);
            assert_eq!(end_draws.load(Ordering::SeqCst), 1);
        }
        "component-order-and-mutation" => {
            let log = Arc::new(Mutex::new(Vec::new()));
            run_for_frames(component_order_game(&log), 1)
                .expect("component initialization/order/mutation game");
            assert_eq!(
                *log.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
                [
                    "Game.Initialize",
                    "A.Initialize",
                    "B.Initialize",
                    "C.Initialize",
                    "Game.Update",
                    "D.Initialize",
                    "C.Update",
                    "A.Update",
                    // XNA snapshots the update list before iteration, so B
                    // still updates once after A removes it from Components.
                    "B.Update",
                    "D.Update",
                    "Game.Draw",
                    "C.Draw",
                    "A.Draw",
                    "D.Draw",
                ]
            );
        }
        "callback-panic" => {
            assert!(matches!(
                run_for_frames(PanicGame::default(), 1),
                Err(CnaError::Callback(_))
            ));
            run_for_frames(EmptyGame::default(), 1).expect("game recreation after contained panic");
        }
        "callback-event-panic" => {
            assert!(matches!(
                run_for_frames(PanicEventGame::default(), 1),
                Err(CnaError::Callback(_))
            ));
            run_for_frames(EmptyGame::default(), 1)
                .expect("game recreation after contained event-handler panic");
        }
        "fault-game-create" => {
            std::env::set_var("CNA_RUST_TEST_FAULT", "game-create");
            assert!(matches!(
                run_for_frames(EmptyGame::default(), 1),
                Err(CnaError::Native {
                    code: cna_sys::CNA_RESULT_INTERNAL,
                    ..
                })
            ));
            std::env::remove_var("CNA_RUST_TEST_FAULT");
        }
        "fault-texture-info" => {
            std::env::set_var("CNA_RUST_TEST_FAULT", "texture-info");
            assert!(matches!(
                run_for_frames(FaultTextureInfoGame::default(), 1),
                Err(CnaError::Native {
                    code: cna_sys::CNA_RESULT_INTERNAL,
                    ..
                })
            ));
            std::env::remove_var("CNA_RUST_TEST_FAULT");
            run_for_frames(EmptyGame::default(), 1).expect("recreate after texture rollback");
        }
        "fault-game-destroy" => {
            std::env::set_var("CNA_RUST_TEST_FAULT", "game-destroy");
            assert!(matches!(
                run_for_frames(EmptyGame::default(), 1),
                Err(CnaError::Native {
                    code: cna_sys::CNA_RESULT_INTERNAL,
                    ..
                })
            ));
            std::env::remove_var("CNA_RUST_TEST_FAULT");
            run_for_frames(EmptyGame::default(), 1).expect("recreate after destroy failure report");
        }
        _ => panic!("unknown native stress child case"),
    }
}

// A valid 1x1 RGBA PNG used only to exercise CNA's encoded texture route.
const ONE_PIXEL_RGBA_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0xf0,
    0x1f, 0x00, 0x05, 0x00, 0x01, 0xff, 0x89, 0x99, 0x3d, 0x1d, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];
