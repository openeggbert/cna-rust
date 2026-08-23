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
use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use cna::extensions::graphics::{
    EffectAnnotationCollectionExt, EffectFactoryExt, EffectParameterCollectionExt,
    EffectPassCollectionExt, EffectTechniqueCollectionExt,
};
use cna::Microsoft::Xna::Framework::Graphics::{
    BlendState, BufferUsage, CubeMapFace, DepthStencilState, DynamicIndexBuffer,
    DynamicVertexBuffer, Effect, EffectMaterial, EffectParameterClass, EffectParameterType,
    GraphicsAdapter, GraphicsDevice, GraphicsDeviceStatus, GraphicsProfile, GraphicsResource,
    IndexBuffer, IndexElementSize, PrimitiveType, RasterizerState, RenderTarget2D,
    RenderTargetBinding, RenderTargetCube, SamplerState, SetDataOptions, SpriteBatch, SpriteFont,
    SpriteSortMode, SurfaceFormat, Texture, Texture2D, TextureCube, VertexBuffer,
    VertexBufferBinding, VertexDeclaration, VertexElement, VertexElementFormat, VertexElementUsage,
    VertexPositionColor,
};
use cna::Microsoft::Xna::Framework::{
    Color, Game, GameContext, GameTime, IDrawable, IGameComponent, IUpdateable, Matrix, Rectangle,
    Vector2, Vector3, Vector4,
};
use cna::{
    run_for_frames, CnaError, EffectAnnotationDescriptor, EffectParameterDescriptor,
    EffectTechniqueDescriptor, GameComponentCollectionExt, GameComponentRuntime, GameState,
    GameStateAccess, Result, VertexData,
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

fn effect_xnb() -> Vec<u8> {
    let effect_code = fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../cna/modules/renderers/fna3d/effects/CnaConformanceEffect.fxb"),
    )
    .expect("read CNA's legal conformance Effect bytecode fixture");
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
        assert!(matches!(
            dynamic.SetDataWithOffsetInBytesAndDataAndStartIndexAndElementCountAndVertexStrideAndOptions(
                16,
                &vertices,
                0,
                1,
                16,
                SetDataOptions::Discard,
            ),
            Err(CnaError::InvalidInput(_))
        ));

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
    assert!(matches!(
        result,
        Err(CnaError::Native { code: 12, message })
            if message.contains("no effect has been applied")
    ));
}

fn assert_headless_readback_unsupported(result: Result<()>) {
    assert!(matches!(
        result,
        Err(CnaError::Native { code: 6, message })
            if message.contains("Headless renderer does not rasterize")
    ));
}

fn assert_native_not_supported(result: Result<()>) {
    assert!(matches!(result, Err(CnaError::Native { code: 6, .. })));
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
        "effect-graph-10",
        "lifecycle-order-and-identity",
        "component-order-and-mutation",
        "callback-panic",
        "callback-event-panic",
        "fault-game-create",
        "fault-texture-info",
        "fault-game-destroy",
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

fn run_child_case(case: &str) {
    match case {
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
        "effect-graph-10" => {
            for _ in 0..10 {
                run_for_frames(EffectStressGame::default(), 1)
                    .expect("Effect reflection/parameter/pass/SpriteBatch ownership cycle");
            }
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
