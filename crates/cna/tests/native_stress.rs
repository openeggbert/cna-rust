//! Crash-isolated ownership and callback stress for an explicitly supplied CNA library.

use std::io::Cursor;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use cna::Microsoft::Xna::Framework::Graphics::{
    BlendState, DepthStencilState, GraphicsDevice, GraphicsResource, RasterizerState, SamplerState,
    SpriteBatch, SpriteSortMode, Texture, Texture2D,
};
use cna::Microsoft::Xna::Framework::{Color, Game, GameContext, GameTime, Rectangle, Vector2};
use cna::{run_for_frames, CnaError, Result};

const CHILD_CASE: &str = "CNA_RUST_NATIVE_STRESS_CHILD";

struct EmptyGame;

impl Game for EmptyGame {}

struct PanicGame;

impl Game for PanicGame {
    fn Update(&mut self, game: &mut GameContext<'_>, time: &GameTime) -> Result<()> {
        let _ = (game, time);
        panic!("intentional callback stress panic")
    }
}

struct SuppressFirstDrawGame {
    begin_draws: Arc<AtomicUsize>,
    draws: Arc<AtomicUsize>,
    end_draws: Arc<AtomicUsize>,
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

#[derive(Default)]
struct LifecycleEvidence {
    events: Vec<&'static str>,
    unload_resources_disposed: bool,
}

struct LifecycleEvidenceGame {
    evidence: Arc<Mutex<LifecycleEvidence>>,
    device: Arc<Mutex<Option<GraphicsDevice>>>,
    texture: Arc<Mutex<Option<Texture2D>>>,
    batch: Arc<Mutex<Option<SpriteBatch>>>,
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
        *self
            .device
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(game.GraphicsDevice()?);
        Ok(())
    }

    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        self.event("LoadContent");
        let first_device = game.GraphicsDevice()?;
        let second_device = game.GraphicsDevice()?;
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

    fn OnExiting(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        self.event("Exiting");
        assert!(!game.GraphicsDevice()?.IsDisposed()?);
        Ok(())
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
    }
}

#[derive(Default)]
struct ResourceGame {
    texture: Option<Texture2D>,
    batch: Option<SpriteBatch>,
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

struct TextureTransferGame;

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
        device.SetBlendState(&blend)?;
        device.SetDepthStencilState(&depth)?;
        device.SetRasterizerState(&rasterizer)?;
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
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            blend.SetMultiSampleMask(0);
        }))
        .is_err());
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

struct FaultTextureInfoGame;

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
        "lifecycle-order-and-identity",
        "callback-panic",
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
                run_for_frames(EmptyGame, 1).expect("create/run/destroy cycle");
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
                run_for_frames(TextureTransferGame, 1)
                    .expect("Texture2D transfer/validation/event cycle");
            }
        }
        "lifecycle-order-and-identity" => {
            let evidence = Arc::new(Mutex::new(LifecycleEvidence::default()));
            let device = Arc::new(Mutex::new(None));
            let texture = Arc::new(Mutex::new(None));
            let batch = Arc::new(Mutex::new(None));
            run_for_frames(
                LifecycleEvidenceGame {
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
                    "Dispose"
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
        "callback-panic" => {
            assert!(matches!(
                run_for_frames(PanicGame, 1),
                Err(CnaError::Callback(_))
            ));
            run_for_frames(EmptyGame, 1).expect("game recreation after contained panic");
        }
        "fault-game-create" => {
            std::env::set_var("CNA_RUST_TEST_FAULT", "game-create");
            assert!(matches!(
                run_for_frames(EmptyGame, 1),
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
                run_for_frames(FaultTextureInfoGame, 1),
                Err(CnaError::Native {
                    code: cna_sys::CNA_RESULT_INTERNAL,
                    ..
                })
            ));
            std::env::remove_var("CNA_RUST_TEST_FAULT");
            run_for_frames(EmptyGame, 1).expect("recreate after texture rollback");
        }
        "fault-game-destroy" => {
            std::env::set_var("CNA_RUST_TEST_FAULT", "game-destroy");
            assert!(matches!(
                run_for_frames(EmptyGame, 1),
                Err(CnaError::Native {
                    code: cna_sys::CNA_RESULT_INTERNAL,
                    ..
                })
            ));
            std::env::remove_var("CNA_RUST_TEST_FAULT");
            run_for_frames(EmptyGame, 1).expect("recreate after destroy failure report");
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
