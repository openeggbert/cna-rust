//! CNA's engine-layer render pipeline against the live library.
//!
//! The engine layer is a build option upstream, so the test asks
//! `engine_layer_version` which build this artifact is before asserting
//! anything about a pipeline. What it will not do is accept "the call
//! succeeded" as evidence: every assertion below is a value -- a size, a count,
//! a byte estimate, an exact refusal message, a callback invocation count --
//! because a pipeline that constructs and does nothing is precisely the state
//! this project refused to bind on.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use cna::extensions::pbr::{
    GltfMaterialBridge, GltfMaterialExtensionSource, GltfMaterialExtensionTextures,
    GltfMaterialSource, PbrMaterialExtensions, PbrMaterialFull, SkinnedPbrEffect,
    ThinFilmIridescence,
};
use cna::extensions::engine::{
    supports_shadow_sampling, DirectionalLight, FxaaPass, GpuTimer, Particle,
    ParticleEmitterSettings, ParticleSystem, PostProcessChain, PostProcessContext, PostProcessPass,
    AerialPerspectivePass, AsciiPass, AtmosphericSky, BloomPass, ChromaticAberrationPass,
    ColorGradePass, ComputeShader, ContactShadowPass, DecalPass, DepthOfFieldPass, FilmGrainPass,
    FullscreenPass, HeightFogPass, LensFlarePass, LightShaftPass, LutInterpolation, MemoryBarrier,
    MotionBlurPass, RenderPipeline, ScopedRenderTarget, ShadowMap, Skybox, SpatialUpscalePass,
    CascadedShadowMap, CubeShadowMap, PointLight, PunctualLight, PunctualLightKind,
    AutoExposure, ClusteredLight, ClusteredLightAssignment, ClusteredLightGrid,
    ClusteredLightBuffer, ClusteredLightCompute, ClusteredLightSet, ClusteredLightType,
    ClusteredForwardEffect, ClusteredShadingMaterial, ClusteredShadowPolicy, CubeLut,
    DebugDraw, DepthEncoding, EnvironmentProcessor, ImageBasedLight, LightProbe,
    LightProbeBaker, LightProbeVolume, ShaderEffectFactory,
    DepthNormalPrepass, DisplayColorSpace, FrustumCuller, HdrDisplayOutput, LodGroup,
    LodSelectionMode, ShadowCascadeState, SpotLight, SpotShadowMap, SsaoPass,
    SsrPass, StorageBuffer, TonemapPass, TransparentDrawList, VolumetricFogPass,
    WeightedBlendedTransparency,
};
use cna::extensions::graphics::EffectFactoryExt;
use cna::extensions::pbr::{
    engine_layer_version, EngineRenderSettings, RenderQuality, ShadowQuality, TonemappingMode,
    TransparencyMode,
};
use cna::Microsoft::Xna::Framework::Graphics::{
    CubeMapFace, DepthFormat, SurfaceFormat, Texture2D, TextureCube,
};
use cna::Microsoft::Xna::Framework::{
    BoundingBox, BoundingFrustum, BoundingSphere, Color, Game, GameContext, GameTime, Matrix,
    Vector2, Vector3,
};
use cna::{run_for_frames, CnaError, ErrorCategory, GameState, GameStateAccess, Result};

/// One CNA `Game` at a time.
///
/// CNA's game host is process-global -- the renderer, the window and the
/// device manager are all one per process -- so two `run_for_frames` calls on
/// two test threads race for the same native state. Serialising them here is
/// the honest fix; running the whole binary single-threaded would hide the
/// constraint from anyone adding a third test.
static ONE_GAME_AT_A_TIME: Mutex<()> = Mutex::new(());

/// How many frames the qualification run draws.
const FRAMES: usize = 8;

/// What the in-game run measured, read back after the game has shut down.
#[derive(Default)]
struct Findings {
    engine_layer: i32,
    scene_target_size: Option<(i32, i32)>,
    scene_target_format: Option<String>,
    scene_target_pixels: Option<std::result::Result<(u32, bool, usize), String>>,
    uses_scene_target: bool,
    scene_target_between_frames: bool,
    gpu_memory_estimate: u64,
    statistics_passes: i32,
    last_frame_passes: i32,
    unsized_begin_refusal: Option<String>,
    double_begin_refusal: Option<String>,
    unopened_end_refusal: Option<String>,
    release_during_frame_refusal: Option<String>,
    exposure_round_trip: f32,
    tonemapping_round_trip: Option<TonemappingMode>,
    transparency_round_trip: Option<TransparencyMode>,
    gpu_timing_requested: bool,
    gpu_timing_reported: bool,
    pass_timing_names: Vec<String>,
    transparent_draws: usize,
    transparent_error: Option<String>,
    transparent_panic: Option<String>,
    frame_state_after_failed_end: Option<String>,
    cleared_frame: Option<String>,
    use_after_shutdown_release: Option<String>,
    frames_completed: usize,
}

fn native_message(error: &CnaError) -> String {
    match error {
        CnaError::Native { message, .. } => message.clone(),
        other => panic!("expected a native refusal, got {other}"),
    }
}

fn expect_state_refusal<T: std::fmt::Debug>(result: Result<T>, what: &str) -> String {
    match result {
        Ok(value) => panic!("{what} was accepted and answered {value:?}"),
        Err(error @ CnaError::Native { .. }) => {
            let CnaError::Native { category, .. } = &error else {
                unreachable!()
            };
            assert_eq!(
                *category,
                ErrorCategory::State,
                "{what} must be refused as a state error"
            );
            native_message(&error)
        }
        Err(other) => panic!("{what} failed with {other} rather than a native refusal"),
    }
}

struct PipelineGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<Findings>>,
    draws: Arc<AtomicUsize>,
    ready: Arc<AtomicBool>,
    pipeline: Option<RenderPipeline>,
    transparent_calls: Arc<AtomicUsize>,
    frame_limit: usize,
}

impl PipelineGame {
    fn new(findings: &Arc<Mutex<Findings>>) -> Self {
        Self {
            state: Arc::new(GameState::default()),
            findings: Arc::clone(findings),
            draws: Arc::new(AtomicUsize::new(0)),
            ready: Arc::new(AtomicBool::new(false)),
            pipeline: None,
            transparent_calls: Arc::new(AtomicUsize::new(0)),
            frame_limit: FRAMES,
        }
    }
}

impl PipelineGame {
    /// Reads the scene target back off the GPU.
    ///
    /// Only after a frame has actually run: the pipeline allocates the target
    /// lazily inside `begin`, so asking before the first frame answers "none"
    /// and would make a real absence indistinguishable from a resource that
    /// simply does not exist yet.
    fn measure_scene_target(pipeline: &RenderPipeline, shared: &Arc<Mutex<Findings>>) {
        let mut findings = shared.lock().expect("findings");
        findings.gpu_memory_estimate = pipeline
            .gpu_memory_estimate_bytes()
            .expect("the pipeline reports its target memory");
        findings.uses_scene_target = pipeline
            .is_using_scene_target()
            .expect("the pipeline says whether it renders offscreen");
        let Some(target) = pipeline
            .scene_target()
            .expect("the scene-target query answers")
        else {
            return;
        };
        let target = target.texture();
        let (width, height) = (target.Width(), target.Height());
        findings.scene_target_size = Some((width, height));
        findings.scene_target_format = pipeline.scene_target_format().ok().map(|f| format!("{f:?}"));

        // The frame cleared to CornflowerBlue. Reading the target back is the
        // difference between "a render target exists" and "the GPU actually
        // rasterised into it", so the pixels themselves are the assertion.
        let mut pixels =
            vec![Color::Transparent; (width.max(0) as usize) * (height.max(0) as usize)];
        findings.scene_target_pixels = match target.GetData(&mut pixels) {
            Ok(()) => {
                let first = pixels.first().copied().unwrap_or(Color::Transparent);
                let uniform = pixels.iter().all(|pixel| *pixel == first);
                Some(Ok((first.PackedValue(), uniform, pixels.len())))
            }
            Err(error) => Some(Err(error.to_string())),
        };
        drop(findings);

    }

    /// Runs the negative paths while the pipeline is still live.
    ///
    /// Not in `UnloadContent`: CNA delivers that from inside `destroy_game`,
    /// after the device shutdown has already released every engine child, so a
    /// pipeline call there measures a released handle rather than a refusal.
    fn measure_failing_callbacks(pipeline: &mut RenderPipeline, shared: &Arc<Mutex<Findings>>) {
        let findings = shared;
        let record = |slot: fn(&mut Findings) -> &mut Option<String>, outcome: Result<()>| {
            let mut findings = findings.lock().expect("findings");
            *slot(&mut findings) = match outcome {
                Ok(()) => Some(String::new()),
                Err(error) => Some(error.to_string()),
            };
        };

        if pipeline
            .set_transparent_scene(|| Err(CnaError::InvalidInput("the transparent scene refused")))
            .is_ok()
            && pipeline.begin_frame(Color::Black).is_ok()
        {
            record(|f| &mut f.transparent_error, pipeline.end_frame());
            // A frame whose draw callback failed: does CNA leave it open? The
            // answer is measured rather than assumed, because a caller that
            // guesses wrong either loses a frame or stalls its own loop.
            record(
                |f| &mut f.frame_state_after_failed_end,
                pipeline.begin_frame(Color::Black),
            );
        }

        let _ = pipeline.end_frame();
        if pipeline
            .set_transparent_scene(|| panic!("intentional transparent-scene panic"))
            .is_ok()
            && pipeline.begin_frame(Color::Black).is_ok()
        {
            record(|f| &mut f.transparent_panic, pipeline.end_frame());
        }

        // Clearing removes the registration, so a frame closes cleanly again.
        let _ = pipeline.end_frame();
        let _ = pipeline.clear_transparent_scene();
        let cleared = pipeline
            .begin_frame(Color::Black)
            .and_then(|()| pipeline.end_frame());
        record(|f| &mut f.cleared_frame, cleared);
    }
}

impl GameStateAccess for PipelineGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for PipelineGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let mut pipeline = RenderPipeline::new(&device)?;

        // A pipeline that has never been sized refuses to open a frame, and
        // that refusal is a *different* state from a frame already being open.
        // Asserting the two messages differ is what keeps the two states from
        // silently collapsing into one.
        let never_sized_begin =
            expect_state_refusal(pipeline.begin_frame(Color::CornflowerBlue), "an unsized begin");
        self.findings.lock().expect("findings").unsized_begin_refusal = Some(never_sized_begin);

        pipeline.resize(320, 180)?;

        // Settings round-trip through the engine's own setters. The value read
        // back is what the engine kept, which is the only reason reading it is
        // worth anything.
        let mut settings = pipeline.settings()?;
        settings.set_exposure(1.75);
        settings.set_tonemapping_mode(TonemappingMode::Aces);
        // Sorted rather than the default None: upstream leaves the transparent
        // phase before consulting the callback when the mode is None, so a test
        // that registered one and never set this would be measuring nothing.
        settings.set_transparency_mode(TransparencyMode::Sorted);
        pipeline.set_settings(&settings)?;
        let stored = pipeline.settings()?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.exposure_round_trip = stored.exposure();
            findings.tonemapping_round_trip = stored.tonemapping_mode().ok();
            findings.transparency_round_trip = stored.transparency_mode().ok();
        }

        // GPU timing is a request, not a command: a renderer without timers
        // accepts it and stays off. The test records both halves.
        let reported = pipeline.set_gpu_timing_enabled(true)?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.gpu_timing_requested = true;
            findings.gpu_timing_reported = reported;
        }

        let calls = Arc::clone(&self.transparent_calls);
        pipeline.set_transparent_scene(move || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })?;

        pipeline.set_camera(
            Matrix::CreateLookAt(
                Vector3::from_x_and_y_and_z(0.0, 0.0, 5.0),
                Vector3::Zero,
                Vector3::Up,
            ),
            Matrix::CreatePerspectiveFieldOfView(1.0, 16.0 / 9.0, 0.1, 100.0),
            0.1,
            100.0,
        )?;

        self.pipeline = Some(pipeline);
        self.ready.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn Draw(&mut self, game: &mut GameContext<'_>, _: &GameTime) -> Result<()> {
        let _ = game;
        let shared = Arc::clone(&self.findings);
        let calls = Arc::clone(&self.transparent_calls);
        let frame_limit = self.frame_limit;
        let Some(pipeline) = self.pipeline.as_mut() else {
            return Ok(());
        };
        let frame = self.draws.fetch_add(1, Ordering::SeqCst);
        pipeline.begin_frame(Color::CornflowerBlue)?;
        if frame == 0 {
            // Inside an open frame, a second begin and a device-resource
            // release are both refused, each with its own message.
            let double = expect_state_refusal(
                pipeline.begin_frame(Color::Black),
                "a second begin inside an open frame",
            );
            let release = expect_state_refusal(
                pipeline.release_device_resources(),
                "releasing device resources inside an open frame",
            );
            let mut findings = shared.lock().expect("findings");
            findings.double_begin_refusal = Some(double);
            findings.release_during_frame_refusal = Some(release);
        }
        if frame == 1 {
            // Inside the frame, because that is the only time upstream hands
            // the scene target out.
            Self::measure_scene_target(pipeline, &shared);
        }
        pipeline.end_frame()?;
        if frame == 0 {
            let unopened =
                expect_state_refusal(pipeline.end_frame(), "ending a frame that is not open");
            shared.lock().expect("findings").unopened_end_refusal = Some(unopened);
            // Between frames the same query answers "none" even though the
            // target exists, which is a contract worth pinning rather than a
            // detail to be surprised by later.
            shared.lock().expect("findings").scene_target_between_frames =
                pipeline.scene_target()?.is_some();
        }
        let mut findings = shared.lock().expect("findings");
        findings.frames_completed += 1;
        let statistics = pipeline.statistics()?;
        findings.statistics_passes = statistics.passes_run;
        findings.last_frame_passes = pipeline.last_frame_pass_count()?;
        findings.gpu_memory_estimate = statistics.gpu_memory_estimate_bytes;
        findings.pass_timing_names = pipeline
            .pass_timings()?
            .into_iter()
            .map(|timing| timing.name)
            .collect();
        findings.transparent_draws = calls.load(Ordering::SeqCst);
        let last = findings.frames_completed == frame_limit;
        drop(findings);
        if last {
            Self::measure_failing_callbacks(pipeline, &shared);
        }
        Ok(())
    }

    fn UnloadContent(&mut self, _: &mut GameContext<'_>) -> Result<()> {
        // By the time CNA delivers this, the device's shutdown has already
        // released every engine child -- it has to, because CNA refuses to
        // destroy a game that still owns one, and that check runs before this
        // callback. Asserting the pipeline reports *released* here is what
        // proves the release happened on the right side of that check rather
        // than by luck.
        let Some(pipeline) = self.pipeline.as_ref() else {
            return Ok(());
        };
        self.findings
            .lock()
            .expect("findings")
            .use_after_shutdown_release = match pipeline.last_frame_pass_count() {
            Ok(count) => Some(format!("answered {count}")),
            Err(error) => Some(error.to_string()),
        };
        Ok(())
    }
}

#[test]
fn the_render_pipeline_runs_real_frames_and_reports_what_they_did() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(Findings::default()));
    let game = PipelineGame::new(&findings);
    run_for_frames(game, FRAMES as u64).expect("every requested frame runs through the pipeline");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    assert_eq!(
        findings.frames_completed, FRAMES,
        "every requested frame opened and closed"
    );

    // Two distinct refusals, not one message reused.
    let never_sized = findings
        .unsized_begin_refusal
        .as_deref()
        .expect("an unsized begin is refused");
    let double = findings
        .double_begin_refusal
        .as_deref()
        .expect("a second begin is refused");
    let unopened = findings
        .unopened_end_refusal
        .as_deref()
        .expect("ending an unopened frame is refused");
    let release = findings
        .release_during_frame_refusal
        .as_deref()
        .expect("releasing resources mid-frame is refused");
    assert_ne!(
        never_sized, double,
        "never sized and already open are different states with different messages"
    );
    assert!(
        never_sized.contains("sized"),
        "the never-sized refusal names the missing resize: {never_sized}"
    );
    assert!(
        double.contains("already open"),
        "the double-begin refusal names the open frame: {double}"
    );
    assert!(
        unopened.contains("No frame is open"),
        "the unopened end refusal names the absent frame: {unopened}"
    );
    assert!(
        release.contains("frame"),
        "the mid-frame release refusal names the open frame: {release}"
    );

    // Settings came back as the engine stored them.
    assert!(
        (findings.exposure_round_trip - 1.75).abs() < 1e-6,
        "the exposure the engine kept is the one that was set: {}",
        findings.exposure_round_trip
    );
    assert_eq!(
        findings.tonemapping_round_trip,
        Some(TonemappingMode::Aces),
        "the tonemapping operator the engine kept is the one that was set"
    );
    assert_eq!(
        findings.transparency_round_trip,
        Some(TransparencyMode::Sorted),
        "the transparency mode the engine kept is the one that was set"
    );

    // The statistics structure and the standalone getter are two routes over
    // one fact and must agree.
    assert_eq!(
        findings.statistics_passes, findings.last_frame_passes,
        "the frame statistics and the pass-count route report the same frame"
    );

    // The transparent-scene callback ran once per closed frame, which is a
    // count rather than a "it was called at some point".
    assert_eq!(
        findings.transparent_draws, findings.frames_completed,
        "the transparent scene drew exactly once per frame"
    );

    let failure = findings
        .transparent_error
        .as_deref()
        .expect("the refusing-callback frame ran");
    assert!(
        failure.contains("the transparent scene refused"),
        "the frame reports the Rust cause rather than a bare native code: {failure:?}"
    );
    let panicked = findings
        .transparent_panic
        .as_deref()
        .expect("the panicking-callback frame ran");
    assert!(
        panicked.contains("panicked"),
        "a contained panic is reported as one: {panicked:?}"
    );
    assert_eq!(
        findings.cleared_frame.as_deref(),
        Some(""),
        "a frame closes cleanly once the failing callback is cleared"
    );
    let after_release = findings
        .use_after_shutdown_release
        .as_deref()
        .expect("UnloadContent ran");
    assert!(
        after_release.contains("released"),
        "the device shutdown released the pipeline before CNA's own UnloadContent: {after_release:?}"
    );
    println!(
        "state after a failed end_frame: {:?}",
        findings.frame_state_after_failed_end
    );

    match findings.scene_target_size {
        Some((width, height)) => {
            assert!(
                findings.uses_scene_target,
                "a pipeline that hands out a scene target says it is using one"
            );
            assert_eq!(
                (width, height),
                (320, 180),
                "the scene target is the size the pipeline was resized to"
            );
            // The estimate is the target's own bytes, and CNA computes it from
            // the same size: an off-by-one in either would part them.
            assert_eq!(
                findings.gpu_memory_estimate,
                (width as u64) * (height as u64) * 4,
                "the GPU memory estimate is this target's own bytes"
            );
            assert!(
                findings.gpu_memory_estimate > 0,
                "a GPU-backed pipeline allocates something, unlike the headless artifact \
                 this project measured at zero"
            );
            match findings.scene_target_pixels.as_ref() {
                Some(Ok((first, uniform, count))) => {
                    assert_eq!(
                        *count,
                        (width as usize) * (height as usize),
                        "the readback covered the whole target"
                    );
                    assert!(
                        *uniform,
                        "every pixel of a cleared target is the same colour; \
                         the first was {first:#010x}"
                    );
                    // The frame cleared to CornflowerBlue and nothing drew over
                    // it, so the target holds exactly that colour. This is the
                    // assertion a headless device could not make.
                    assert_eq!(
                        *first,
                        Color::CornflowerBlue.PackedValue(),
                        "the scene target holds the colour the frame cleared to"
                    );
                }
                Some(Err(reason)) => {
                    println!("scene-target readback refused by this renderer: {reason}");
                }
                None => panic!("the scene target was measured but never read"),
            }
            assert!(
                !findings.scene_target_between_frames,
                "the scene target is handed out only inside an open frame"
            );
        }
        None => println!("this renderer draws straight to the back buffer; no scene target"),
    }

    println!(
        "engine layer {} | scene target {:?} {:?} | gpu memory estimate {} bytes | \
         gpu timing requested={} reported={} | pass timings {:?}",
        findings.engine_layer,
        findings.scene_target_size,
        findings.scene_target_format,
        findings.gpu_memory_estimate,
        findings.gpu_timing_requested,
        findings.gpu_timing_reported,
        findings.pass_timing_names,
    );
}

/// What a shadow-map run measured.
#[derive(Default)]
struct ShadowFindings {
    engine_layer: i32,
    supported: bool,
    sampling_supported: bool,
    quality_sizes: Vec<(i32, i32)>,
    created_size: i32,
    created_radius: i32,
    created_quality: Option<ShadowQuality>,
    depth_bias_round_trip: f32,
    corners_inside_clip: Option<bool>,
    begin_matches_pure_functions: Option<bool>,
    texture_size: Option<(i32, i32)>,
    caster_effect_present: bool,
    caster_technique_count: Option<i32>,
    skinned_caster_effect_present: bool,
    shadow_pass_ran: bool,
    shadows_enabled: bool,
    caster_draws: usize,
    retained_map_matches: Option<bool>,
    frames_completed: usize,
}

struct ShadowGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<ShadowFindings>>,
    caster_calls: Arc<AtomicUsize>,
    pipeline: Option<RenderPipeline>,
    map: Option<Arc<ShadowMap>>,
    draws: Arc<AtomicUsize>,
}

impl ShadowGame {
    fn new(findings: &Arc<Mutex<ShadowFindings>>) -> Self {
        Self {
            state: Arc::new(GameState::default()),
            findings: Arc::clone(findings),
            caster_calls: Arc::new(AtomicUsize::new(0)),
            pipeline: None,
            map: None,
            draws: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl GameStateAccess for ShadowGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

/// The scene the shadow test casts over: a unit box away from the origin, so a
/// wrong translation in the light transform moves the corners out of clip space
/// instead of leaving them at zero where an identity matrix would.
fn scene_bounds() -> BoundingBox {
    BoundingBox::new(
        Vector3::from_x_and_y_and_z(-3.0, -1.0, 2.0),
        Vector3::from_x_and_y_and_z(1.0, 4.0, 7.0),
    )
}

fn sun() -> DirectionalLight {
    let mut light = DirectionalLight::canonical_defaults().expect("CNA's own light defaults");
    light.direction = Vector3::from_x_and_y_and_z(-0.4, -1.0, -0.3);
    light.casts_shadows = true;
    light
}

impl Game for ShadowGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;

        // The size and filter radius a preset selects are pure functions of the
        // preset. Reading all five is what turns "a number came back" into "the
        // presets are ordered and distinct", which is the property a renderer
        // quality dial actually depends on.
        let qualities = [
            ShadowQuality::Disabled,
            ShadowQuality::Low,
            ShadowQuality::Medium,
            ShadowQuality::High,
            ShadowQuality::Ultra,
        ];
        let mut sizes = Vec::new();
        for quality in qualities {
            sizes.push((
                ShadowMap::size_for_quality(quality)?,
                ShadowMap::filter_radius_for_quality(quality)?,
            ));
        }
        self.findings.lock().expect("findings").quality_sizes = sizes;

        let map = Arc::new(ShadowMap::new(&device, ShadowQuality::Medium)?);
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.supported = map.is_supported()?;
            findings.sampling_supported = supports_shadow_sampling(&device)?;
            findings.created_size = map.size()?;
            findings.created_radius = map.filter_radius()?;
            findings.created_quality = map.quality().ok();
        }

        map.set_depth_bias(0.0025)?;
        self.findings.lock().expect("findings").depth_bias_round_trip = map.depth_bias()?;

        // The light transform is a pure function of the light and the bounds,
        // so it can be checked against the thing it is for: every corner of the
        // scene must land inside the light's clip volume. A projection that did
        // not fit the scene would push at least one corner outside, and an
        // identity matrix would push all eight.
        let light = sun();
        let view = ShadowMap::compute_light_view(light, scene_bounds())?;
        let projection = ShadowMap::compute_light_projection(view, scene_bounds())?;
        let transform = view * projection;
        let inside = scene_bounds().GetCorners().into_iter().all(|corner| {
            let clip = Vector3::Transform(corner, transform);
            (-1.001..=1.001).contains(&clip.X)
                && (-1.001..=1.001).contains(&clip.Y)
                && (-0.001..=1.001).contains(&clip.Z)
        });
        self.findings.lock().expect("findings").corners_inside_clip = Some(inside);

        let mut pipeline = RenderPipeline::new(&device)?;
        pipeline.resize(256, 256)?;
        // The pipeline runs its shadow pass only while shadows are enabled in
        // its settings, so the test enables them and then reads back what the
        // engine kept rather than trusting the default either way.
        let mut settings = pipeline.settings()?;
        settings.set_shadows_enabled(true);
        settings.set_shadow_quality(ShadowQuality::Medium);
        pipeline.set_settings(&settings)?;
        self.findings.lock().expect("findings").shadows_enabled =
            pipeline.settings()?.shadows_enabled();
        let calls = Arc::clone(&self.caster_calls);
        pipeline.set_shadow_scene(&map, light, scene_bounds(), move || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })?;
        self.findings.lock().expect("findings").retained_map_matches = Some(
            pipeline
                .shadow_scene_map()?
                .is_some_and(|retained| Arc::ptr_eq(retained, &map)),
        );

        {
            let mut findings = self.findings.lock().expect("findings");
            findings.caster_effect_present = map.caster_effect()?.is_some();
            findings.skinned_caster_effect_present = map.skinned_caster_effect()?.is_some();
            // The caster effect is a real Effect, not a marker: reading its
            // technique count reaches the native object behind the borrow.
            findings.caster_technique_count = match map.caster_effect()? {
                Some(borrowed) => Some(borrowed.effect().Techniques()?.Count()?),
                None => None,
            };
        }

        self.pipeline = Some(pipeline);
        self.map = Some(map);
        Ok(())
    }

    fn Draw(&mut self, game: &mut GameContext<'_>, _: &GameTime) -> Result<()> {
        let _ = game;
        let shared = Arc::clone(&self.findings);
        let calls = Arc::clone(&self.caster_calls);
        let (Some(pipeline), Some(map)) = (self.pipeline.as_mut(), self.map.as_ref()) else {
            return Ok(());
        };
        let frame = self.draws.fetch_add(1, Ordering::SeqCst);
        pipeline.begin_frame(Color::Black)?;
        pipeline.end_frame()?;

        let mut findings = shared.lock().expect("findings");
        findings.frames_completed += 1;
        findings.shadow_pass_ran = pipeline.did_shadow_pass_run()?;
        findings.caster_draws = calls.load(Ordering::SeqCst);
        if frame == 0 {
            // After the pipeline's own shadow pass, the map's transform must be
            // exactly the composition of the two pure functions the caller can
            // compute itself. Anything else means the pipeline is casting from
            // a different frustum than the one the caller reasoned about.
            let light = sun();
            let view = ShadowMap::compute_light_view(light, scene_bounds())?;
            let projection = ShadowMap::compute_light_projection(view, scene_bounds())?;
            let stored = map.light_view_projection()?;
            findings.begin_matches_pure_functions = Some(stored == view * projection);
            findings.texture_size = map
                .shadow_texture()?
                .map(|view| (view.texture().Width(), view.texture().Height()));
        }
        Ok(())
    }
}

#[test]
fn a_shadow_map_reports_its_preset_and_casts_from_the_transform_it_publishes() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(ShadowFindings::default()));
    let game = ShadowGame::new(&findings);
    run_for_frames(game, 4).expect("four real frames with a shadow scene");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }
    assert_eq!(findings.frames_completed, 4, "every frame ran");

    // Disabled selects nothing; every other preset selects a real map, and the
    // sizes strictly increase. A preset table that had collapsed to one value
    // would still answer, which is why this is an ordering rather than a count.
    let sizes = &findings.quality_sizes;
    assert_eq!(sizes.len(), 5, "five presets were measured");
    let enabled = &sizes[1..];
    assert!(
        enabled.windows(2).all(|pair| pair[0].0 < pair[1].0),
        "each preset selects a strictly larger map than the one below it: {sizes:?}"
    );
    assert!(
        enabled
            .iter()
            .all(|(size, _)| u32::try_from(*size).is_ok_and(u32::is_power_of_two)),
        "every shadow-map size is a power of two: {sizes:?}"
    );

    assert_eq!(
        findings.created_quality,
        Some(ShadowQuality::Medium),
        "the map reports the preset it was created with"
    );
    assert_eq!(
        findings.created_size, sizes[2].0,
        "the created map's size is the one its preset selects"
    );
    assert_eq!(
        findings.created_radius, sizes[2].1,
        "the created map's filter radius is the one its preset selects"
    );
    assert!(
        (findings.depth_bias_round_trip - 0.0025).abs() < 1e-7,
        "the depth bias round-trips through CNA: {}",
        findings.depth_bias_round_trip
    );

    assert_eq!(
        findings.corners_inside_clip,
        Some(true),
        "every corner of the scene lands inside the light's clip volume"
    );
    assert_eq!(
        findings.retained_map_matches,
        Some(true),
        "the pipeline reports the very map this binding retains for it"
    );

    if findings.supported {
        assert!(
            findings.caster_effect_present,
            "a renderer that can cast has a caster effect to cast with"
        );
        assert!(
            findings.skinned_caster_effect_present,
            "a renderer that can cast has a skinned caster effect too"
        );
        assert!(
            findings.caster_technique_count.is_some_and(|count| count > 0),
            "the borrowed caster effect is a real effect with techniques: {:?}",
            findings.caster_technique_count
        );
        assert!(
            findings.shadows_enabled,
            "the engine kept the shadows-enabled flag the test set"
        );
        assert!(
            findings.shadow_pass_ran,
            "a supported shadow scene with shadows enabled runs its pass"
        );
        assert_eq!(
            findings.caster_draws, findings.frames_completed,
            "the caster callback drew exactly once per frame"
        );
        assert_eq!(
            findings.begin_matches_pure_functions,
            Some(true),
            "the pipeline casts from exactly the transform the pure functions compute"
        );
        match findings.texture_size {
            Some((width, height)) => assert_eq!(
                (width, height),
                (findings.created_size, findings.created_size),
                "the shadow texture is the map's own size"
            ),
            None => panic!("a supported map hands out its shadow texture"),
        }
    } else {
        println!(
            "this renderer cannot cast shadows: caster effect {}, sampling {}",
            findings.caster_effect_present, findings.sampling_supported
        );
    }

    println!(
        "shadow map: supported={} sampling={} size={} radius={} texture={:?} presets={:?}",
        findings.supported,
        findings.sampling_supported,
        findings.created_size,
        findings.created_radius,
        findings.texture_size,
        findings.quality_sizes,
    );
}

/// What a post-process run measured.
#[derive(Default)]
struct ChainFindings {
    engine_layer: i32,
    tonemap_curve: Vec<(String, f32, f32)>,
    fxaa_thresholds: Vec<(String, f32)>,
    fxaa_glsl_bytes: usize,
    fxaa_glsl_head: String,
    blit_name: String,
    blit_supported: bool,
    pass_counts: Vec<i32>,
    owned_transfer_refusal: Option<String>,
    pass_usable_after_refusal: Option<String>,
    pool_targets_before: u64,
    pool_targets_after: u64,
    pool_bytes: u64,
    two_slots_differ: Option<bool>,
    blit_source: Option<u32>,
    blit_destination: Option<std::result::Result<u32, String>>,
    pipeline_user_passes: Option<i32>,
    frames_completed: usize,
}

struct ChainGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<ChainFindings>>,
    chain: Option<PostProcessChain>,
    pipeline: Option<RenderPipeline>,
    draws: Arc<AtomicUsize>,
}

impl ChainGame {
    fn new(findings: &Arc<Mutex<ChainFindings>>) -> Self {
        Self {
            state: Arc::new(GameState::default()),
            findings: Arc::clone(findings),
            chain: None,
            pipeline: None,
            draws: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl GameStateAccess for ChainGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for ChainGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;

        // Pure functions first: they need no pass and no frame, and their
        // answers are exact.
        let mut curve = Vec::new();
        for mode in [
            TonemappingMode::None,
            TonemappingMode::Reinhard,
            TonemappingMode::Filmic,
            TonemappingMode::Aces,
        ] {
            for value in [0.0_f32, 0.5, 1.0, 4.0] {
                curve.push((
                    format!("{mode:?}"),
                    value,
                    TonemapPass::tonemap_channel(mode, value, 1.0, 1.0)?,
                ));
            }
        }
        self.findings.lock().expect("findings").tonemap_curve = curve;

        let mut thresholds = Vec::new();
        for quality in [
            RenderQuality::Low,
            RenderQuality::Medium,
            RenderQuality::High,
            RenderQuality::Ultra,
        ] {
            thresholds.push((
                format!("{quality:?}"),
                FxaaPass::edge_threshold_for_quality(quality)?,
            ));
        }
        let glsl = FxaaPass::fragment_glsl()?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.fxaa_thresholds = thresholds;
            findings.fxaa_glsl_bytes = glsl.len();
            findings.fxaa_glsl_head = glsl.chars().take(80).collect();
        }

        let mut chain = PostProcessChain::new(&device)?;
        let mut counts = vec![chain.pass_count()?];

        // A borrowed pass: the chain records it and the Arc is what keeps it
        // alive, so dropping the caller's handle while it is in the chain is
        // not expressible.
        let blit = Arc::new(PostProcessPass::blit(&device)?);
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.blit_name = blit.name()?;
            findings.blit_supported = blit.is_supported(&device)?;
        }
        chain.add_pass(&blit)?;
        counts.push(chain.pass_count()?);

        // A consuming transfer that must be refused, with the pass coming back.
        //
        // Upstream's own refusal -- a pass still lending its effect cannot be
        // handed over -- is unreachable from safe Rust: the borrow carries the
        // pass's lifetime, so `chain.add_owned_pass(lending)` while a borrow is
        // outstanding does not compile. That is a stronger guarantee than the
        // run-time check, and it is why the refusal exercised here is a
        // different one: a chain that has already been released.
        //
        // What is being asserted is the ownership contract itself. A transfer
        // that failed after dropping the pass would look identical from the
        // outside until something later used it, so the test uses it.
        let effect = device.create_empty_effect()?;
        let lending = PostProcessPass::from_effect(&device, effect, "lender")?;
        {
            let borrow = lending.effect()?;
            assert!(borrow.is_some(), "an effect pass lends the effect it holds");
        }
        let mut released = PostProcessChain::new(&device)?;
        released.release()?;
        match released.add_owned_pass(lending) {
            Ok(()) => panic!("a released chain accepted a pass"),
            Err(refused) => {
                let mut findings = self.findings.lock().expect("findings");
                findings.owned_transfer_refusal = Some(refused.error.to_string());
                // The pass came back. Using it proves it is still alive rather
                // than a handle CNA has already released.
                findings.pass_usable_after_refusal = Some(match refused.pass.name() {
                    Ok(name) => name,
                    Err(error) => format!("refused: {error}"),
                });
            }
        }
        counts.push(chain.pass_count()?);

        // A consuming transfer that must succeed.
        let owned = TonemapPass::new(&device)?;
        owned.set_exposure(1.5)?;
        owned.set_mode(TonemappingMode::Aces)?;
        chain
            .add_owned_pass(owned.into_pass())
            .expect("a pass that is not lending anything is handed over");
        counts.push(chain.pass_count()?);
        self.findings.lock().expect("findings").pass_counts = counts;

        // The pipeline's user-pass list is the same borrow.
        let mut pipeline = RenderPipeline::new(&device)?;
        pipeline.resize(64, 64)?;
        pipeline.add_user_pass(&blit)?;
        let mut settings = pipeline.settings()?;
        settings.set_tonemapping_mode(TonemappingMode::None);
        pipeline.set_settings(&settings)?;
        pipeline.begin_frame(Color::Black)?;
        pipeline.end_frame()?;
        self.findings.lock().expect("findings").pipeline_user_passes =
            Some(pipeline.last_frame_pass_count()?);
        pipeline.clear_user_passes()?;

        self.chain = Some(chain);
        self.pipeline = Some(pipeline);
        Ok(())
    }

    fn Draw(&mut self, game: &mut GameContext<'_>, _: &GameTime) -> Result<()> {
        let device = game.GraphicsDevice()?;
        let shared = Arc::clone(&self.findings);
        let Some(chain) = self.chain.as_mut() else {
            return Ok(());
        };
        let frame = self.draws.fetch_add(1, Ordering::SeqCst);
        shared.lock().expect("findings").frames_completed += 1;
        if frame != 0 {
            return Ok(());
        }

        // The pool is the chain's own, borrowed for as long as the chain lives.
        // Two slots of one shape must be two targets, not the same one twice.
        {
            let view = chain.target_pool()?;
            let pool = view.pool();
            shared.lock().expect("findings").pool_targets_before = pool.target_count()?;
            let first = pool.acquire(32, 32, SurfaceFormat::Color, DepthFormat::None, 0)?;
            let second = pool.acquire(32, 32, SurfaceFormat::Color, DepthFormat::None, 1)?;
            let differ = first.texture().Width() == second.texture().Width()
                && !std::ptr::eq(first.texture(), second.texture());
            let mut findings = shared.lock().expect("findings");
            findings.two_slots_differ = Some(differ);
            findings.pool_targets_after = pool.target_count()?;
            findings.pool_bytes = pool.estimated_bytes()?;
        }

        // A blit pass over a known source, read back off the GPU. This is the
        // pass's whole contract -- copy the source unchanged -- expressed as
        // pixels rather than as a success code. The destination is left unset,
        // which upstream defines as the back buffer, so the readback is the
        // frame itself rather than a target only this test can see.
        let parameters = device.PresentationParameters()?.Clone();
        let (width, height) = (parameters.BackBufferWidth(), parameters.BackBufferHeight());
        let source = Texture2D::new(&device, 4, 4)?;
        source.SetData(&vec![Color::Crimson; 16])?;
        let blit = PostProcessPass::blit(&device)?;
        let context = PostProcessContext::canonical_defaults()?
            .source(&source)?
            .size(width, height)
            .depth_range(0.1, 100.0);
        blit.apply(&context)?;
        let pixels = usize::try_from(width * height).expect("back-buffer pixel count");
        let mut read = vec![Color::Transparent; pixels];
        let outcome = device.GetBackBufferDataWithData(&mut read);
        let mut findings = shared.lock().expect("findings");
        findings.blit_source = Some(Color::Crimson.PackedValue());
        findings.blit_destination = Some(match outcome {
            Ok(()) => {
                let first = read[0];
                if read.iter().all(|pixel| *pixel == first) {
                    Ok(first.PackedValue())
                } else {
                    Err("the blitted frame is not one uniform colour".to_owned())
                }
            }
            Err(error) => Err(error.to_string()),
        });
        Ok(())
    }
}

#[test]
fn a_post_process_chain_owns_what_it_is_given_and_copies_what_it_is_asked_to() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(ChainFindings::default()));
    let game = ChainGame::new(&findings);
    run_for_frames(game, 2).expect("two frames with a post-process chain");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }
    assert_eq!(findings.frames_completed, 2, "every frame ran");

    // The tonemapping curve is a pure function, so it can be held to the shape
    // of the operator rather than to "a number came back". Black stays black
    // everywhere; `None` is a clamp, passing an in-range value through and
    // pinning anything above one; and a real operator compresses an over-range
    // value to *below* the clamp, which is what distinguishes it from `None`
    // and from the other operators being silently wired to the same code.
    println!("tonemap curve: {:?}", findings.tonemap_curve);
    let sample = |mode: &str, input: f32| -> f32 {
        findings
            .tonemap_curve
            .iter()
            .find(|(m, i, _)| m == mode && (*i - input).abs() < 1e-6)
            .map(|(_, _, o)| *o)
            .unwrap_or_else(|| panic!("no {mode} sample at {input}"))
    };
    for (mode, input, output) in &findings.tonemap_curve {
        if *input == 0.0 {
            assert!(
                output.abs() < 1e-6,
                "{mode} maps black to black, not {output}"
            );
        }
        assert!(
            *output <= 1.000_01,
            "{mode} never emits above the display range: {input} -> {output}"
        );
    }
    assert!(
        (sample("None", 0.5) - 0.5).abs() < 1e-5,
        "the None operator passes an in-range value through: {}",
        sample("None", 0.5)
    );
    assert!(
        (sample("None", 4.0) - 1.0).abs() < 1e-5,
        "the None operator clamps an over-range value to one: {}",
        sample("None", 4.0)
    );
    for mode in ["Reinhard", "Filmic", "Aces"] {
        assert!(
            sample(mode, 4.0) < sample("None", 4.0),
            "{mode} compresses where None clamps: {} vs {}",
            sample(mode, 4.0),
            sample("None", 4.0)
        );
        assert!(
            sample(mode, 0.5) > 0.0,
            "{mode} keeps a mid-grey visible: {}",
            sample(mode, 0.5)
        );
    }

    // FXAA's presets are ordered: a higher quality filters more edges, which
    // means a *lower* threshold. A table that had collapsed to one value would
    // still answer.
    let thresholds: Vec<f32> = findings.fxaa_thresholds.iter().map(|(_, v)| *v).collect();
    assert_eq!(thresholds.len(), 4, "four presets were measured");
    assert!(
        thresholds.windows(2).all(|pair| pair[0] >= pair[1]),
        "a higher quality preset asks for a threshold no higher than the one below it: {:?}",
        findings.fxaa_thresholds
    );
    assert!(
        thresholds[0] > thresholds[3],
        "the presets are not all the same value: {:?}",
        findings.fxaa_thresholds
    );
    assert!(
        findings.fxaa_glsl_bytes > 200,
        "the FXAA fragment shader is real source, {} bytes",
        findings.fxaa_glsl_bytes
    );
    assert!(
        findings.fxaa_glsl_head.contains("version") || findings.fxaa_glsl_head.contains("precision"),
        "the shader starts like GLSL: {:?}",
        findings.fxaa_glsl_head
    );

    assert!(
        !findings.blit_name.is_empty(),
        "a pass carries the name the engine gave it"
    );
    assert!(
        findings.blit_supported,
        "a copy is supported wherever the engine layer runs"
    );

    // Borrowed, refused, and consumed, in that order: 0 -> 1 -> 1 -> 2.
    assert_eq!(
        findings.pass_counts,
        vec![0, 1, 1, 2],
        "a borrowed pass and a consumed pass each add one, and a refused transfer adds none"
    );
    let refusal = findings
        .owned_transfer_refusal
        .as_deref()
        .expect("a pass lending its effect is refused");
    assert!(
        refusal.contains("released"),
        "the refusal names the released chain: {refusal}"
    );
    let after = findings
        .pass_usable_after_refusal
        .as_deref()
        .expect("the refused pass came back");
    assert_eq!(
        after, "lender",
        "the refused pass is still the caller's, and still answers: {after:?}"
    );

    assert_eq!(
        findings.two_slots_differ,
        Some(true),
        "two slots of one shape are two targets"
    );
    assert!(
        findings.pool_targets_after >= findings.pool_targets_before + 2,
        "acquiring two targets grows the pool: {} -> {}",
        findings.pool_targets_before,
        findings.pool_targets_after
    );
    // Two 32x32 four-byte targets and nothing else: the estimate is exactly
    // their bytes, which an off-by-one in either dimension or a double count
    // would break.
    assert_eq!(
        findings.pool_bytes,
        2 * 32 * 32 * 4,
        "a pool holding two 32x32 Color targets reports exactly their bytes"
    );

    match findings.blit_destination.as_ref() {
        Some(Ok(pixel)) => assert_eq!(
            Some(*pixel),
            findings.blit_source,
            "a blit pass copies its source unchanged"
        ),
        Some(Err(reason)) => println!("render-target readback refused: {reason}"),
        None => panic!("the blit pass never ran"),
    }

    println!(
        "post-process: passes {:?} | pool {} -> {} targets, {} bytes | pipeline pass count {:?} | \
         fxaa thresholds {:?}",
        findings.pass_counts,
        findings.pool_targets_before,
        findings.pool_targets_after,
        findings.pool_bytes,
        findings.pipeline_user_passes,
        findings.fxaa_thresholds,
    );
}

/// How many frames the GPU-timer and particle run draws.
///
/// A GPU timer query is non-blocking and resolves some frames after the work
/// it timed was submitted, so one frame would measure nothing on a renderer
/// that supports timing perfectly well.
const SIMULATION_FRAMES: usize = 30;

/// What a GPU-timer and particle run measured.
#[derive(Default)]
struct SimulationFindings {
    engine_layer: i32,
    timer_supported: bool,
    timer_unsupported_reason: String,
    timer_open_inside_range: Option<bool>,
    timer_open_after_end: Option<bool>,
    timer_samples: i32,
    timer_collections: usize,
    timer_available_frames: usize,
    timer_milliseconds: f64,
    random_values: Vec<f32>,
    random_is_deterministic: Option<bool>,
    step_free_fall: Option<(f32, f32)>,
    step_respawns: Option<(f32, f32)>,
    capacity: i32,
    settings_round_trip: Option<(f32, f32)>,
    emission_rate_clamped: Option<bool>,
    active_after_update: i32,
    particles_read: usize,
    aged_particles: usize,
    cpu_positions: Vec<(f32, f32, f32)>,
    gpu_positions: Vec<(f32, f32, f32)>,
    used_compute: Option<bool>,
    lookup_glsl_bytes: usize,
    frames_completed: usize,
}

struct SimulationGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<SimulationFindings>>,
    timer: Option<GpuTimer>,
    system: Option<ParticleSystem>,
    draws: Arc<AtomicUsize>,
}

impl SimulationGame {
    fn new(findings: &Arc<Mutex<SimulationFindings>>) -> Self {
        Self {
            state: Arc::new(GameState::default()),
            findings: Arc::clone(findings),
            timer: None,
            system: None,
            draws: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl GameStateAccess for SimulationGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

/// A deterministic emitter: no variance, no cone, one particle per second.
fn steady_emitter() -> Result<ParticleEmitterSettings> {
    let mut settings = ParticleEmitterSettings::canonical_defaults()?;
    settings.position = Vector3::Zero;
    settings.direction = Vector3::Up;
    settings.gravity = Vector3::from_x_and_y_and_z(0.0, -10.0, 0.0);
    settings.cone_angle = 0.0;
    settings.speed = 1.0;
    settings.speed_variance = 0.0;
    settings.lifetime = 4.0;
    settings.lifetime_variance = 0.0;
    settings.drag = 0.0;
    settings.emission_rate = 8.0;
    Ok(settings)
}

impl Game for SimulationGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;

        // The shader hash is a pure function and the two simulations agree on
        // it by construction upstream. Determinism and range are what a caller
        // can actually rely on, so both are asserted.
        let seeds = [0_u32, 1, 2, 7, 1_000, 4_294_967_295];
        let mut values = Vec::new();
        for seed in seeds {
            values.push(ParticleSystem::random(seed)?);
        }
        let again: Vec<f32> = seeds
            .iter()
            .map(|seed| ParticleSystem::random(*seed))
            .collect::<Result<_>>()?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.random_is_deterministic = Some(values == again);
            findings.random_values = values;
        }

        // The pure integrator. With no drag and a known gravity, one step is
        // arithmetic a test can do itself -- which is the difference between
        // checking the simulation and checking that a call returned.
        let settings = steady_emitter()?;
        let mut particle = Particle::canonical_defaults()?;
        particle.lifetime = 10.0;
        particle.velocity = Vector3::Zero;
        let stepped = particle.step(0, settings, 0.5)?;
        self.findings.lock().expect("findings").step_free_fall =
            Some((stepped.velocity.Y, stepped.age));

        // A slot whose age passes its lifetime respawns rather than dying, and
        // the respawn count is how a caller sees that happen.
        let mut expiring = Particle::canonical_defaults()?;
        expiring.lifetime = 0.25;
        expiring.age = 0.2;
        let respawned = expiring.step(3, settings, 0.5)?;
        self.findings.lock().expect("findings").step_respawns =
            Some((respawned.age, respawned.respawn_count));

        let system = ParticleSystem::with_capacity(&device, 64)?;
        system.set_settings(settings)?;
        let stored = system.settings()?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.capacity = system.capacity()?;
            findings.settings_round_trip = Some((stored.emission_rate, stored.lifetime));
        }

        // An emission rate the capacity cannot sustain is accepted and then
        // reported. The settings must still read back exactly as written --
        // that is the whole point of reporting rather than clamping.
        let mut greedy = settings;
        greedy.emission_rate = 10_000.0;
        system.set_settings(greedy)?;
        let clamped = system.is_emission_rate_clamped()?;
        let greedy_stored = system.settings()?;
        assert!(
            (greedy_stored.emission_rate - 10_000.0).abs() < 1e-3,
            "the settings are stored as given, not corrected: {}",
            greedy_stored.emission_rate
        );
        self.findings.lock().expect("findings").emission_rate_clamped = Some(clamped);
        system.set_settings(settings)?;
        system.reset()?;

        let timer = GpuTimer::new(&device)?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.timer_supported = timer.is_supported()?;
            findings.timer_unsupported_reason = timer.unsupported_reason()?;
        }

        self.findings.lock().expect("findings").lookup_glsl_bytes =
            ParticleSystem::particle_lookup_glsl()?.len();

        self.timer = Some(timer);
        self.system = Some(system);
        Ok(())
    }

    fn Draw(&mut self, game: &mut GameContext<'_>, _: &GameTime) -> Result<()> {
        let _ = game;
        let shared = Arc::clone(&self.findings);
        let (Some(timer), Some(system)) = (self.timer.as_ref(), self.system.as_ref()) else {
            return Ok(());
        };
        let frame = self.draws.fetch_add(1, Ordering::SeqCst);
        shared.lock().expect("findings").frames_completed += 1;

        // The poll comes *before* the next range opens. A GPU timer query is
        // non-blocking: the result of the range closed last frame is not ready
        // when `end` returns, and opening the next range re-issues the query.
        // Polling straight after `end` therefore collects nothing, for ever --
        // which is exactly what this test measured before the order changed.
        {
            let mut findings = shared.lock().expect("findings");
            findings.timer_available_frames += usize::from(timer.is_result_available()?);
            if timer.poll()? {
                findings.timer_collections += 1;
            }
            findings.timer_samples = timer.sample_count()?;
            let milliseconds = timer.last_milliseconds()?;
            if milliseconds > 0.0 {
                findings.timer_milliseconds = milliseconds;
            }
        }

        timer.begin()?;
        let open = timer.is_open()?;
        system.update(0.25)?;
        timer.end()?;
        let closed = timer.is_open()?;
        if frame == 0 {
            let mut findings = shared.lock().expect("findings");
            findings.timer_open_inside_range = Some(open);
            findings.timer_open_after_end = Some(closed);
        }

        if frame == 3 {
            let particles = system.particles()?;
            let mut findings = shared.lock().expect("findings");
            findings.active_after_update = system.active_count()?;
            findings.particles_read = particles.len();
            findings.aged_particles = particles.iter().filter(|p| p.age > 0.0).count();
            findings.used_compute = Some(system.uses_compute()?);
        }
        Ok(())
    }

    fn UnloadContent(&mut self, _: &mut GameContext<'_>) -> Result<()> {
        Ok(())
    }
}

/// Runs one deterministic simulation on the requested path and returns the
/// positions it produced.
fn simulate(system: &ParticleSystem, on_cpu: bool) -> Result<Vec<(f32, f32, f32)>> {
    system.set_simulation_on_cpu(on_cpu)?;
    system.reset()?;
    for _ in 0..8 {
        system.update(0.125)?;
    }
    Ok(system
        .particles()?
        .into_iter()
        .map(|p| (p.position.X, p.position.Y, p.position.Z))
        .collect())
}

struct PathComparisonGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<SimulationFindings>>,
}

impl GameStateAccess for PathComparisonGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for PathComparisonGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        if engine_layer_version()? == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let system = ParticleSystem::with_capacity(&device, 32)?;
        system.set_settings(steady_emitter()?)?;

        // Upstream states the CPU and GPU paths are one simulation. Running the
        // same eight steps on each and comparing the particles is what turns
        // that from a claim into a measurement.
        let cpu = simulate(&system, true)?;
        let gpu = simulate(&system, false)?;
        let mut findings = self.findings.lock().expect("findings");
        findings.cpu_positions = cpu;
        findings.gpu_positions = gpu;
        findings.used_compute = Some(system.uses_compute()?);
        Ok(())
    }
}

#[test]
fn gpu_timers_and_particles_report_what_they_measured() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(SimulationFindings::default()));
    let game = SimulationGame::new(&findings);
    run_for_frames(game, SIMULATION_FRAMES as u64).expect("every simulation frame runs");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }
    assert_eq!(findings.frames_completed, SIMULATION_FRAMES, "every frame ran");

    // The hash is deterministic and in range. A generator that returned the
    // same number for every seed would satisfy determinism alone, so both
    // properties are asserted.
    assert_eq!(
        findings.random_is_deterministic,
        Some(true),
        "the shader hash answers the same value for the same seed"
    );
    assert!(
        findings
            .random_values
            .iter()
            .all(|value| (0.0..1.0).contains(value)),
        "every hash value is a unit fraction: {:?}",
        findings.random_values
    );
    let distinct: std::collections::BTreeSet<u32> = findings
        .random_values
        .iter()
        .map(|value| value.to_bits())
        .collect();
    assert!(
        distinct.len() >= findings.random_values.len() - 1,
        "different seeds give different values: {:?}",
        findings.random_values
    );

    // Free fall for half a second under -10: the velocity is exactly -5 and the
    // age is exactly the step. Anything else is a different integrator.
    let (velocity, age) = findings.step_free_fall.expect("one step was taken");
    assert!(
        (velocity - -5.0).abs() < 1e-4,
        "half a second under gravity -10 gives velocity -5, not {velocity}"
    );
    assert!(
        (age - 0.5).abs() < 1e-6,
        "one 0.5 second step ages the particle by 0.5, not {age}"
    );

    // A slot past its lifetime respawns rather than disappearing, and the
    // overflow carries: a slot aged 0.2 with a lifetime of 0.25, stepped by
    // 0.5, comes back aged by exactly the 0.45 it overshot. Asserting the rule
    // rather than "the age got smaller" is what would catch a respawn that
    // reset to zero and silently lost a frame of motion.
    let (respawn_age, respawns) = findings.step_respawns.expect("one step was taken");
    assert!(
        respawns >= 1.0,
        "a slot past its lifetime respawns: count {respawns}"
    );
    let overflow = (0.2_f32 + 0.5) - 0.25;
    assert!(
        (respawn_age - overflow).abs() < 1e-5,
        "the respawned slot carries the overflow: expected {overflow}, got {respawn_age}"
    );

    assert_eq!(findings.capacity, 64, "the system allocated what was asked");
    let (rate, lifetime) = findings.settings_round_trip.expect("settings were read back");
    assert!(
        (rate - 8.0).abs() < 1e-4 && (lifetime - 4.0).abs() < 1e-4,
        "the emitter settings round-trip exactly: rate {rate}, lifetime {lifetime}"
    );
    // 10,000 per second for four seconds is 40,000 slots against a capacity of
    // 64, so the report is not merely "some number came back".
    assert_eq!(
        findings.emission_rate_clamped,
        Some(true),
        "a rate 600 times the capacity is reported as clamped"
    );

    assert!(
        findings.particles_read > 0,
        "the system hands its particles back"
    );
    assert!(
        findings.aged_particles > 0,
        "particles that have been updated carry a non-zero age: {} of {}",
        findings.aged_particles,
        findings.particles_read
    );
    assert!(
        findings.active_after_update > 0,
        "a system emitting at eight per second has active slots after a second"
    );

    if findings.timer_supported {
        assert_eq!(
            findings.timer_open_inside_range,
            Some(true),
            "a supported timer reports its range open between begin and end"
        );
        assert_eq!(
            findings.timer_open_after_end,
            Some(false),
            "and closed after end"
        );
        assert!(
            findings.timer_samples > 0,
            "a supported timer collects results across {SIMULATION_FRAMES} frames; \
             {} polls returned a result and {} frames reported one available",
            findings.timer_collections,
            findings.timer_available_frames
        );
        assert_eq!(
            findings.timer_samples as usize, findings.timer_collections,
            "every collected result is one sample"
        );
        assert!(
            findings.timer_milliseconds > 0.0,
            "a collected GPU sample is a real duration: {}",
            findings.timer_milliseconds
        );
        assert!(
            findings.timer_milliseconds < 1_000.0,
            "and a plausible one: {}",
            findings.timer_milliseconds
        );
    } else {
        assert!(
            !findings.timer_unsupported_reason.is_empty(),
            "an unsupported timer says why"
        );
        assert_eq!(
            findings.timer_samples, 0,
            "an unsupported timer collects nothing"
        );
        println!(
            "GPU timing unsupported by this renderer: {}",
            findings.timer_unsupported_reason
        );
    }

    assert!(
        findings.lookup_glsl_bytes > 50,
        "the particle lookup GLSL is real source, {} bytes",
        findings.lookup_glsl_bytes
    );

    println!(
        "particles: capacity {} active {} read {} aged {} | compute {:?} | \
         timer supported={} samples={} collections={} available-frames={} last={}ms",
        findings.capacity,
        findings.active_after_update,
        findings.particles_read,
        findings.aged_particles,
        findings.used_compute,
        findings.timer_supported,
        findings.timer_samples,
        findings.timer_collections,
        findings.timer_available_frames,
        findings.timer_milliseconds,
    );
}

#[test]
fn the_cpu_and_gpu_particle_paths_are_one_simulation() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(SimulationFindings::default()));
    let game = PathComparisonGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame comparing the two simulation paths");

    let findings = findings.lock().expect("findings");
    if findings.cpu_positions.is_empty() && findings.gpu_positions.is_empty() {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }
    assert_eq!(
        findings.cpu_positions.len(),
        findings.gpu_positions.len(),
        "both paths simulate the same number of slots"
    );
    assert!(
        !findings.cpu_positions.is_empty(),
        "the comparison ran over some particles"
    );
    let moved = findings
        .cpu_positions
        .iter()
        .filter(|(x, y, z)| x.abs() + y.abs() + z.abs() > 1e-6)
        .count();
    assert!(
        moved > 0,
        "the simulation actually moved particles: {:?}",
        &findings.cpu_positions[..findings.cpu_positions.len().min(4)]
    );

    // Bit-identical is upstream's claim for the hash; the integration is the
    // same four lines in the same order, so the two paths must agree to within
    // float reassociation rather than to within "roughly".
    let mut worst = 0.0_f32;
    for (cpu, gpu) in findings.cpu_positions.iter().zip(&findings.gpu_positions) {
        worst = worst
            .max((cpu.0 - gpu.0).abs())
            .max((cpu.1 - gpu.1).abs())
            .max((cpu.2 - gpu.2).abs());
    }
    println!(
        "cpu/gpu worst position difference: {worst} over {} slots (compute used: {:?})",
        findings.cpu_positions.len(),
        findings.used_compute
    );
    assert!(
        worst < 1e-3,
        "the CPU and GPU paths produce the same particles; worst difference {worst}"
    );
}

/// A compute shader with one deterministic answer.
///
/// Each invocation reads its own slot, multiplies by a uniform and adds its own
/// index. There is no reduction, no shared memory and no ordering, so the exact
/// output is a closed form the test computes itself -- which is the difference
/// between checking that a dispatch happened and checking what it computed.
const DOUBLE_AND_INDEX_GLSL: &str = "#version 310 es\n\
layout(local_size_x = 8) in;\n\
layout(std430, binding = 0) buffer Values { int values[]; };\n\
uniform int uFactor;\n\
void main() {\n\
    uint index = gl_GlobalInvocationID.x;\n\
    if (index >= uint(values.length())) { return; }\n\
    values[index] = values[index] * uFactor + int(index);\n\
}\n";

/// What a compute run measured.
#[derive(Default)]
struct ComputeFindings {
    engine_layer: i32,
    barrier_contains_self: Option<bool>,
    barrier_all_contains_storage: Option<bool>,
    barrier_storage_contains_all: Option<bool>,
    byte_buffer_size: u64,
    byte_round_trip: Option<Vec<u8>>,
    element_count: u64,
    element_byte_size: u64,
    element_round_trip: Option<Vec<i32>>,
    oversized_upload_refused: Option<String>,
    wrong_element_size_refused: Option<String>,
    shader_valid: Option<bool>,
    shader_compile_error: String,
    broken_shader_valid: Option<bool>,
    broken_shader_error: String,
    dispatched: Option<std::result::Result<Vec<i32>, String>>,
    image_binding_supported: Option<bool>,
}

struct ComputeGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<ComputeFindings>>,
}

impl GameStateAccess for ComputeGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for ComputeGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;

        // The barrier mask is CNA's, so the containment test is asked of it.
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.barrier_contains_self =
                Some(MemoryBarrier::SHADER_STORAGE.contains(MemoryBarrier::SHADER_STORAGE)?);
            findings.barrier_all_contains_storage =
                Some(MemoryBarrier::ALL.contains(MemoryBarrier::SHADER_STORAGE)?);
            findings.barrier_storage_contains_all =
                Some(MemoryBarrier::SHADER_STORAGE.contains(MemoryBarrier::ALL)?);
        }

        // A flat byte buffer round-trips its bytes.
        let bytes = StorageBuffer::with_byte_size(&device, 16)?;
        let payload: Vec<u8> = (0..16_u8).collect();
        bytes.set_bytes(&payload)?;
        let mut read = vec![0_u8; 16];
        bytes.get_bytes(&mut read)?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.byte_buffer_size = bytes.byte_size()?;
            findings.byte_round_trip = Some(read);
        }

        // A typed buffer remembers both numbers, which is what lets it refuse
        // an overlong upload and a mismatched element size instead of quietly
        // reinterpreting the bytes.
        let values = Arc::new(StorageBuffer::with_elements::<i32>(&device, 16)?);
        let input: Vec<i32> = (0..16_i32).map(|index| index + 1).collect();
        values.set_elements(&input)?;
        let mut back = vec![0_i32; 16];
        values.get_elements(&mut back)?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.element_count = values.element_count()?;
            findings.element_byte_size = values.element_byte_size()?;
            findings.element_round_trip = Some(back);
            findings.oversized_upload_refused = values
                .set_elements(&vec![0_i32; 17])
                .err()
                .map(|error| error.to_string());
            findings.wrong_element_size_refused = values
                .set_elements(&vec![0_i16; 16])
                .err()
                .map(|error| error.to_string());
        }

        // A source that cannot compile is refused at creation, with the
        // compiler's own diagnostic in the failure. Measuring that here is what
        // keeps `is_valid` honest: it describes a shader that was created, and
        // a caller does not get one that silently does nothing.
        let broken =
            ComputeShader::new(&device, "#version 310 es\nvoid main() { not_a_function(); }\n");
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.broken_shader_valid = Some(broken.is_ok());
            findings.broken_shader_error = match broken {
                Ok(_) => String::new(),
                Err(error) => error.to_string(),
            };
        }

        let mut shader = ComputeShader::new(&device, DOUBLE_AND_INDEX_GLSL)?;
        let valid = shader.is_valid()?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.shader_valid = Some(valid);
            findings.shader_compile_error = shader.compile_error()?;
            findings.image_binding_supported = Some(shader.is_image_binding_supported()?);
        }
        if !valid {
            return Ok(());
        }

        shader.bind_storage_buffer(0, &values)?;
        shader.set_uniform_int("uFactor", 3)?;
        shader.dispatch(2, 1, 1)?;
        shader.barrier(MemoryBarrier::SHADER_STORAGE | MemoryBarrier::BUFFER_UPDATE)?;
        let mut computed = vec![0_i32; 16];
        self.findings.lock().expect("findings").dispatched =
            Some(match values.get_elements(&mut computed) {
                Ok(()) => Ok(computed),
                Err(error) => Err(error.to_string()),
            });
        Ok(())
    }
}

#[test]
fn a_compute_dispatch_produces_the_exact_values_it_was_asked_for() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(ComputeFindings::default()));
    let game = ComputeGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame with a compute dispatch");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    // A mask contains itself and is contained by ALL, and the reverse is false.
    // The third assertion is the one that matters: a containment test that
    // always answered true would pass the first two.
    assert_eq!(findings.barrier_contains_self, Some(true));
    assert_eq!(findings.barrier_all_contains_storage, Some(true));
    assert_eq!(
        findings.barrier_storage_contains_all,
        Some(false),
        "one bit does not contain every bit"
    );

    assert_eq!(findings.byte_buffer_size, 16, "the buffer is the size asked");
    assert_eq!(
        findings.byte_round_trip.as_deref(),
        Some((0..16_u8).collect::<Vec<u8>>().as_slice()),
        "a byte buffer round-trips its bytes"
    );

    assert_eq!(findings.element_count, 16, "the element count is remembered");
    assert_eq!(
        findings.element_byte_size, 4,
        "and so is the element size, which is what makes a mismatch detectable"
    );
    assert_eq!(
        findings.element_round_trip.as_deref(),
        Some((1..=16_i32).collect::<Vec<i32>>().as_slice()),
        "a typed buffer round-trips its elements"
    );
    assert!(
        findings.oversized_upload_refused.is_some(),
        "seventeen elements into a sixteen-element buffer is refused"
    );
    assert!(
        findings.wrong_element_size_refused.is_some(),
        "a two-byte element into a four-byte buffer is refused rather than reinterpreted"
    );

    assert_eq!(
        findings.broken_shader_valid,
        Some(false),
        "a source that cannot compile is refused rather than producing a shader that does nothing"
    );
    assert!(
        findings.broken_shader_error.contains("did not compile"),
        "and the failure carries the compiler's own diagnostic: {:?}",
        findings.broken_shader_error
    );

    match findings.shader_valid {
        Some(true) => {
            assert!(
                findings.shader_compile_error.is_empty(),
                "a shader that compiled has no compile error: {:?}",
                findings.shader_compile_error
            );
            // The dispatch's whole answer, computed here from the same closed
            // form the shader implements: value * 3 + index over sixteen slots
            // across two groups of eight.
            let expected: Vec<i32> = (0..16_i32).map(|index| (index + 1) * 3 + index).collect();
            match findings.dispatched.as_ref() {
                Some(Ok(actual)) => assert_eq!(
                    actual, &expected,
                    "the dispatch computed value * 3 + index for every slot"
                ),
                Some(Err(reason)) => panic!("the readback after a dispatch failed: {reason}"),
                None => panic!("the dispatch never ran"),
            }
            println!(
                "compute: dispatch verified over 16 elements; image binding supported = {:?}",
                findings.image_binding_supported
            );
        }
        Some(false) => println!(
            "this renderer did not compile the compute shader: {}",
            findings.shader_compile_error
        ),
        None => panic!("the shader was never created"),
    }
}

/// What a decal and sky run measured.
#[derive(Default)]
struct SkyFindings {
    engine_layer: i32,
    inside_box: Vec<(f32, bool)>,
    decal_opacity: f32,
    decal_slope: f32,
    decal_tint: Option<(f32, f32, f32)>,
    skybox_supported: bool,
    skybox_has_environment_before: Option<bool>,
    skybox_has_environment_after: Option<bool>,
    skybox_environment_size: Option<i32>,
    skybox_yaw: f32,
    skybox_intensity: f32,
    view_ray_centre: Option<(f32, f32, f32)>,
    view_ray_yawed: Option<(f32, f32, f32)>,
    sky_supported: bool,
    sky_turbidity: f32,
    sky_profile: Vec<(usize, f32, f32, f32)>,
    sky_turbidity_response: Option<(f32, f32)>,
    sky_glsl_bytes: usize,
}

struct SkyGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<SkyFindings>>,
}

impl GameStateAccess for SkyGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for SkyGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;

        // The decal box is a unit box in its own space, so the boundary is a
        // pure fact a test can walk rather than a value to read back.
        let mut inside = Vec::new();
        for offset in [0.0_f32, 0.25, 0.49, 0.51, 1.0] {
            inside.push((
                offset,
                DecalPass::is_inside_decal_box(Vector3::from_x_and_y_and_z(offset, 0.0, 0.0))?,
            ));
        }
        self.findings.lock().expect("findings").inside_box = inside;

        let decal = DecalPass::new(&device)?;
        decal.set_opacity(0.625)?;
        decal.set_max_slope_angle(0.75)?;
        decal.set_tint(Vector3::from_x_and_y_and_z(0.1, 0.2, 0.3))?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.decal_opacity = decal.opacity()?;
            findings.decal_slope = decal.max_slope_angle()?;
            let tint = decal.tint()?;
            findings.decal_tint = Some((tint.X, tint.Y, tint.Z));
        }

        let mut skybox = Skybox::new(&device)?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.skybox_supported = skybox.is_supported()?;
            findings.skybox_has_environment_before = Some(skybox.has_environment()?);
        }
        // The consuming attach: on success the skybox owns the cube map and the
        // Rust value forgets its handle, so `has_environment` is asked of CNA
        // rather than of what this test still holds.
        let cube = TextureCube::new(&device, 4, false, SurfaceFormat::Color)?;
        skybox
            .set_owned_environment(cube)
            .expect("a live cube map is taken over");
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.skybox_has_environment_after = Some(skybox.has_environment()?);
            // The borrow reaches the cube map itself, not just its presence:
            // reading its size through the view is what proves the handle names
            // the environment that was handed over.
            findings.skybox_environment_size = match skybox.environment()? {
                Some(view) => Some(view.size()?),
                None => None,
            };
        }
        skybox.set_yaw(0.5)?;
        skybox.set_intensity(2.25)?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.skybox_yaw = skybox.yaw()?;
            findings.skybox_intensity = skybox.intensity()?;
        }

        // The view ray is a pure function, so the sky's rotation is checkable
        // without drawing: yawing the sky must turn the ray the centre pixel
        // looks along.
        let view = Matrix::CreateLookAt(
            Vector3::Zero,
            Vector3::from_x_and_y_and_z(0.0, 0.0, -1.0),
            Vector3::Up,
        );
        let projection = Matrix::CreatePerspectiveFieldOfView(1.0, 1.0, 0.1, 100.0);
        let centre = Skybox::compute_view_ray(view, projection, 0.0, 0.0, 0.0)?;
        let yawed = Skybox::compute_view_ray(view, projection, 0.0, 0.0, 1.0)?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.view_ray_centre = Some((centre.X, centre.Y, centre.Z));
            findings.view_ray_yawed = Some((yawed.X, yawed.Y, yawed.Z));
        }

        let sky = AtmosphericSky::new(&device)?;
        sky.set_turbidity(4.5)?;
        // A sun low in the sky, so a sweep of view directions at one elevation
        // varies only in its angle from the sun.
        let sun = Vector3::NormalizeWithValue(Vector3::from_x_and_y_and_z(1.0, 0.25, 0.0));
        sky.set_sun_direction(sun)?;
        let mut profile = Vec::new();
        for step in 0..5 {
            let angle = std::f32::consts::PI * (step as f32) / 4.0;
            let view = Vector3::NormalizeWithValue(Vector3::from_x_and_y_and_z(
                angle.cos(),
                0.25,
                angle.sin(),
            ));
            let radiance = AtmosphericSky::radiance(view, sun, 4.5)?;
            profile.push((step, radiance.X, radiance.Y, radiance.Z));
        }
        let hazy = AtmosphericSky::radiance(sun, sun, 9.0)?;
        let clear = AtmosphericSky::radiance(sun, sun, 2.0)?;
        {
            let mut findings = self.findings.lock().expect("findings");
            findings.sky_supported = sky.is_supported()?;
            findings.sky_turbidity = sky.turbidity()?;
            findings.sky_profile = profile;
            findings.sky_turbidity_response = Some((hazy.X, clear.X));
            findings.sky_glsl_bytes = AtmosphericSky::model_glsl()?.len();
        }
        Ok(())
    }
}

#[test]
fn decals_and_skies_answer_with_the_values_they_were_given() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(SkyFindings::default()));
    let game = SkyGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame with a decal pass and two skies");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    // The decal box has a boundary, and a containment test that always said
    // "yes" would pass every inside case. The outside cases are the assertion.
    println!("decal box along x: {:?}", findings.inside_box);
    assert!(
        findings.inside_box.iter().any(|(_, inside)| *inside),
        "some point is inside the decal box: {:?}",
        findings.inside_box
    );
    assert!(
        findings.inside_box.iter().any(|(_, inside)| !*inside),
        "and some point is outside it: {:?}",
        findings.inside_box
    );
    assert_eq!(
        findings.inside_box.first().map(|(_, inside)| *inside),
        Some(true),
        "the box's own centre is inside it"
    );
    assert_eq!(
        findings.inside_box.last().map(|(_, inside)| *inside),
        Some(false),
        "a point a whole unit off centre is outside it"
    );

    assert!((findings.decal_opacity - 0.625).abs() < 1e-6);
    assert!((findings.decal_slope - 0.75).abs() < 1e-6);
    assert_eq!(
        findings.decal_tint,
        Some((0.1, 0.2, 0.3)),
        "the tint round-trips through CNA channel for channel"
    );

    assert_eq!(
        findings.skybox_has_environment_before,
        Some(false),
        "a skybox created with no environment has none"
    );
    assert_eq!(
        findings.skybox_has_environment_after,
        Some(true),
        "and has one after a cube map is handed over"
    );
    assert_eq!(
        findings.skybox_environment_size,
        Some(4),
        "the borrowed environment is the four-texel cube map that was handed over"
    );
    assert!((findings.skybox_yaw - 0.5).abs() < 1e-6);
    assert!((findings.skybox_intensity - 2.25).abs() < 1e-6);

    // Yawing the sky must turn the ray. Two rays that agreed would mean the
    // rotation never reached the computation.
    let centre = findings.view_ray_centre.expect("a centre ray");
    let yawed = findings.view_ray_yawed.expect("a yawed ray");
    let length = (centre.0 * centre.0 + centre.1 * centre.1 + centre.2 * centre.2).sqrt();
    assert!(
        (length - 1.0).abs() < 1e-3,
        "the view ray is a unit direction, not {length}"
    );
    let difference = (centre.0 - yawed.0).abs() + (centre.1 - yawed.1).abs() + (centre.2 - yawed.2).abs();
    assert!(
        difference > 0.1,
        "a yaw of one radian turns the ray: {centre:?} vs {yawed:?}"
    );

    assert!((findings.sky_turbidity - 4.5).abs() < 1e-6);
    // The model is swept across five view directions at one elevation, from
    // the sun's azimuth round to the opposite side. Three properties are
    // asserted, and a stub returning a constant fails all three: radiance is
    // never negative, it changes monotonically with the angle from the sun, and
    // the two ends of the sweep are far apart rather than nearly equal.
    println!("atmospheric sky profile: {:?}", findings.sky_profile);
    assert_eq!(findings.sky_profile.len(), 5, "the sweep ran");
    assert!(
        findings
            .sky_profile
            .iter()
            .all(|(_, r, g, b)| *r >= 0.0 && *g >= 0.0 && *b >= 0.0),
        "radiance is never negative: {:?}",
        findings.sky_profile
    );
    let red: Vec<f32> = findings.sky_profile.iter().map(|(_, r, _, _)| *r).collect();
    // The Perez formulation this model follows brightens the sky both toward
    // the sun and away from it -- the circumsolar term at one end, the
    // `cos squared` term at the other -- so the sweep is U-shaped with its
    // minimum at right angles to the sun. Asserting that shape catches a model
    // that ignored the sun direction (flat), one that only had the circumsolar
    // term (monotonic), and one returning noise.
    let minimum = red
        .iter()
        .copied()
        .enumerate()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .expect("a minimum");
    assert!(
        (1..=3).contains(&minimum.0),
        "the sky is dimmest at right angles to the sun, not at an end: {red:?}"
    );
    assert!(
        red[0] > minimum.1 * 1.2 && red[4] > minimum.1 * 1.2,
        "and brighter than that both toward the sun and away from it: {red:?}"
    );
    // Which *end* is brighter says which way the sun direction points. It is
    // the far end, so `sun_direction` is the direction the light travels --
    // the same convention `DirectionalLightEXT` uses, and the opposite of the
    // one "the direction the sun is in" suggests.
    assert!(
        red[4] > red[0],
        "the brightest end is the one opposite the sun-direction vector: {red:?}"
    );

    // Turbidity is a real parameter of the model, not an ignored one.
    let (hazy, clear) = findings
        .sky_turbidity_response
        .expect("two turbidities were evaluated");
    assert!(
        (hazy - clear).abs() > 1e-9,
        "a hazier atmosphere gives a different radiance: {hazy} vs {clear}"
    );

    assert!(
        findings.sky_glsl_bytes > 100,
        "the sky model's GLSL is real source, {} bytes",
        findings.sky_glsl_bytes
    );

    println!(
        "sky: skybox supported={} | atmospheric supported={} | turbidity response {:?}",
        findings.skybox_supported, findings.sky_supported, findings.sky_turbidity_response
    );
}

/// What the screen-space pass run measured.
#[derive(Default)]
struct PassFindings {
    engine_layer: i32,
    bloom_extract: Vec<(f32, f32)>,
    bloom_iterations: Vec<i32>,
    ssao_samples: Vec<i32>,
    ssao_kernel: Vec<(f32, f32, f32)>,
    ssao_kernel_len: usize,
    coc: Vec<(f32, f32)>,
    optical_depth: Vec<(f32, f32)>,
    occluded: Vec<(f32, bool)>,
    combined_visibility: Vec<(f32, f32, f32)>,
    transmittance: Vec<(f32, f32)>,
    air_mass: Vec<(f32, f32)>,
    identity_scale: Vec<((i32, i32, i32, i32), bool)>,
    lut_size: Vec<((i32, i32), i32)>,
    identity_lut_size: Option<(i32, i32)>,
    has_strip_lut: Option<bool>,
    round_trips: Vec<(&'static str, f32, f32)>,
    supported: Vec<(&'static str, bool)>,
    scope_recorded: Option<bool>,
    ascii_cell_size: Option<(i32, i32)>,
    glsl_bytes: Vec<(&'static str, usize)>,
}

struct PassGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<PassFindings>>,
}

impl GameStateAccess for PassGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for PassGame {
    #[allow(clippy::too_many_lines)]
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let mut findings = PassFindings {
            engine_layer: version,
            ..PassFindings::default()
        };

        // --- pure functions, which need no pass ---------------------------
        for value in [0.0_f32, 0.5, 1.0, 2.0, 4.0] {
            findings
                .bloom_extract
                .push((value, BloomPass::extract_channel(value, 1.0)?));
        }
        for quality in [
            RenderQuality::Low,
            RenderQuality::Medium,
            RenderQuality::High,
            RenderQuality::Ultra,
        ] {
            findings
                .bloom_iterations
                .push(BloomPass::iterations_for_quality(quality)?);
            findings
                .ssao_samples
                .push(SsaoPass::sample_count_for_quality(quality)?);
        }
        for distance in [1.0_f32, 5.0, 10.0, 20.0] {
            findings.coc.push((
                distance,
                DepthOfFieldPass::circle_of_confusion_millimetres(distance, 10.0, 50.0, 2.8)?,
            ));
        }
        for distance in [0.0_f32, 10.0, 100.0] {
            findings.optical_depth.push((
                distance,
                // A level ray at the height the fog is densest at, so the
                // integral is density times distance and nothing else.
                HeightFogPass::optical_depth(0.0, 0.0, distance, 0.02, 0.1, 0.0)?,
            ));
        }
        // The ray is at view depth 5. A scene sample nearer than that by more
        // than the bias and by less than the thickness is an occluder; one
        // further away, or nearer by more than the thickness, is not.
        for scene_depth in [4.0_f32, 4.7, 4.99, 5.0, 6.0] {
            findings.occluded.push((
                scene_depth,
                ContactShadowPass::is_occluded(5.0, scene_depth, 0.01, 0.5)?,
            ));
        }
        for (map, contact) in [(1.0_f32, 1.0_f32), (1.0, 0.25), (0.5, 0.5), (0.0, 1.0)] {
            findings.combined_visibility.push((
                map,
                contact,
                ContactShadowPass::combine_visibility(map, contact)?,
            ));
        }
        for air_mass in [0.0_f32, 1.0, 4.0] {
            let value = AerialPerspectivePass::transmittance(air_mass, 3.0)?;
            findings.transmittance.push((air_mass, value.X));
        }
        for distance in [0.0_f32, 100.0, 1000.0] {
            findings.air_mass.push((
                distance,
                AerialPerspectivePass::air_mass_for_distance(
                    Vector3::from_x_and_y_and_z(0.0, 1.0, 0.0),
                    distance,
                    8000.0,
                )?,
            ));
        }
        for sizes in [(64, 64, 64, 64), (64, 64, 128, 128), (128, 64, 64, 64)] {
            findings.identity_scale.push((
                sizes,
                SpatialUpscalePass::is_identity_scale(sizes.0, sizes.1, sizes.2, sizes.3)?,
            ));
        }
        for strip in [(256, 16), (1024, 32)] {
            findings.lut_size.push((
                strip,
                ColorGradePass::lut_size_for_strip(strip.0, strip.1)?,
            ));
        }
        findings.glsl_bytes.push((
            "ssao",
            SsaoPass::occlusion_glsl(false)?.len(),
        ));
        findings.glsl_bytes.push((
            "contact-shadow",
            ContactShadowPass::occlusion_test_glsl()?.len(),
        ));

        // --- the passes themselves ----------------------------------------
        macro_rules! round_trip {
            ($label:literal, $pass:expr, $set:ident, $get:ident, $value:expr) => {{
                let pass = &$pass;
                pass.$set($value)?;
                findings.round_trips.push(($label, $value, pass.$get()?));
            }};
        }

        let bloom = BloomPass::new(&device)?;
        round_trip!("bloom.threshold", bloom, set_threshold, threshold, 0.875);
        round_trip!("bloom.intensity", bloom, set_intensity, intensity, 1.25);
        bloom.set_iterations(5)?;
        findings
            .round_trips
            .push(("bloom.iterations", 5.0, bloom.iterations()? as f32));
        bloom.reset_targets()?;
        findings
            .supported
            .push(("bloom", bloom.pass().is_supported(&device)?));

        let ssao = SsaoPass::new(&device)?;
        round_trip!("ssao.radius", ssao, set_radius, radius, 0.75);
        round_trip!("ssao.intensity", ssao, set_intensity, intensity, 1.5);
        ssao.set_sample_count(24)?;
        ssao.set_half_resolution(true)?;
        findings
            .round_trips
            .push(("ssao.sample_count", 24.0, ssao.sample_count()? as f32));
        findings.round_trips.push((
            "ssao.half_resolution",
            1.0,
            f32::from(u8::from(ssao.is_half_resolution()?)),
        ));
        let kernel = ssao.kernel()?;
        findings.ssao_kernel_len = kernel.len();
        findings.ssao_kernel = kernel.iter().map(|v| (v.X, v.Y, v.Z)).collect();
        ssao.reset_targets()?;
        findings
            .supported
            .push(("ssao", ssao.pass().is_supported(&device)?));

        let ssr = SsrPass::new(&device)?;
        round_trip!("ssr.max_distance", ssr, set_max_distance, max_distance, 40.0);
        round_trip!("ssr.thickness", ssr, set_thickness, thickness, 0.35);
        round_trip!("ssr.edge_fade", ssr, set_edge_fade, edge_fade, 0.2);
        round_trip!("ssr.intensity", ssr, set_intensity, intensity, 0.8);
        findings
            .supported
            .push(("ssr", ssr.pass().is_supported(&device)?));

        let dof = DepthOfFieldPass::new(&device)?;
        round_trip!("dof.focus_distance", dof, set_focus_distance, focus_distance, 12.5);
        round_trip!("dof.focal_length", dof, set_focal_length, focal_length, 35.0);
        round_trip!("dof.f_number", dof, set_f_number, f_number, 1.8);

        let contact = ContactShadowPass::new(&device)?;
        round_trip!("contact.max_distance", contact, set_max_distance, max_distance, 0.5);
        round_trip!("contact.thickness", contact, set_thickness, thickness, 0.08);
        contact.set_light_direction(Vector3::from_x_and_y_and_z(0.0, -1.0, 0.0))?;
        findings.round_trips.push((
            "contact.light_direction.y",
            -1.0,
            contact.light_direction()?.Y,
        ));
        findings
            .supported
            .push(("contact-shadow", contact.pass().is_supported(&device)?));

        let shafts = LightShaftPass::new(&device)?;
        round_trip!("shafts.decay", shafts, set_decay, decay, 0.95);
        shafts.set_light_screen_position(Vector2::from_x_and_y(0.25, 0.75))?;
        findings.round_trips.push((
            "shafts.light_x",
            0.25,
            shafts.light_screen_position()?.X,
        ));

        let fog = HeightFogPass::new(&device)?;
        round_trip!("fog.density", fog, set_density, density, 0.03);
        fog.set_color(Vector3::from_x_and_y_and_z(0.4, 0.5, 0.6))?;
        findings
            .round_trips
            .push(("fog.color.z", 0.6, fog.color()?.Z));

        let mut volumetric = VolumetricFogPass::new(&device)?;
        round_trip!("volumetric.density", volumetric, set_density, density, 0.15);
        round_trip!("volumetric.anisotropy", volumetric, set_anisotropy, anisotropy, 0.6);
        let map = Arc::new(ShadowMap::new(&device, ShadowQuality::Low)?);
        volumetric.set_light(
            &map,
            Vector3::from_x_and_y_and_z(0.0, -1.0, 0.0),
            Vector3::from_x_and_y_and_z(1.0, 0.9, 0.8),
        )?;

        let aerial = AerialPerspectivePass::new(&device)?;
        round_trip!("aerial.turbidity", aerial, set_turbidity, turbidity, 3.5);
        findings.glsl_bytes.push((
            "aerial-fallback",
            aerial.fallback_reason()?.len(),
        ));

        let grain = FilmGrainPass::new(&device)?;
        round_trip!("grain.intensity", grain, set_intensity, intensity, 0.05);
        let aberration = ChromaticAberrationPass::new(&device)?;
        round_trip!("aberration.strength", aberration, set_strength, strength, 0.004);
        let flare = LensFlarePass::new(&device)?;
        round_trip!("flare.dispersal", flare, set_dispersal, dispersal, 0.3);
        let blur = MotionBlurPass::new(&device)?;
        round_trip!("blur.strength", blur, set_strength, strength, 0.4);

        let mut grade = ColorGradePass::new(&device)?;
        round_trip!("grade.strength", grade, set_strength, strength, 0.65);
        grade.set_interpolation(LutInterpolation::Tetrahedral)?;
        findings.round_trips.push((
            "grade.interpolation",
            1.0,
            f32::from(u8::from(grade.interpolation()? == LutInterpolation::Tetrahedral)),
        ));
        let lut = ColorGradePass::create_identity_lut(&device, 16)?;
        findings.identity_lut_size = Some((lut.Width(), lut.Height()));
        grade.set_strip_lut(Some(lut))?;
        findings.has_strip_lut = Some(grade.has_strip_lut()?);

        let upscale = SpatialUpscalePass::new(&device)?;
        upscale.set_sharpness(0.55)?;
        upscale.set_edge_adaptive(true)?;
        findings
            .round_trips
            .push(("upscale.sharpness", 0.55, upscale.sharpness()?));

        let ascii = AsciiPass::new(&device)?;
        findings
            .supported
            .push(("ascii-effect", ascii.has_effect()?));
        if let Some(effect) = ascii.effect()? {
            let (width, height) = effect.cell_size()?;
            findings.ascii_cell_size = Some((width, height));
        }

        let _fullscreen = FullscreenPass::new(&device)?;

        // A scope records the binding it replaced, which is what lets it put
        // the old one back. Asking is the only way to know it recorded one.
        {
            let scope = ScopedRenderTarget::begin(&device, None)?;
            findings.scope_recorded = Some(scope.has_recorded_previous()?);
            scope.end()?;
        }

        *self.findings.lock().expect("findings") = findings;
        Ok(())
    }
}

#[test]
fn the_screen_space_passes_carry_their_knobs_and_compute_their_curves() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(PassFindings::default()));
    let game = PassGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame building every screen-space pass");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    // Every knob comes back as it was set. Fifty-odd of them at once is where a
    // macro-generated accessor wired to a neighbouring route shows up.
    for (label, expected, actual) in &findings.round_trips {
        assert!(
            (expected - actual).abs() < 1e-4,
            "{label} round-trips: set {expected}, read {actual}"
        );
    }
    assert!(
        findings.round_trips.len() >= 25,
        "every knob was exercised: {}",
        findings.round_trips.len()
    );

    // The bright pass keeps nothing of black, never brightens, and keeps more
    // of a brighter pixel. It is a soft knee rather than a hard cut -- a pixel
    // exactly at the threshold still contributes -- which is measured here
    // rather than assumed either way.
    println!("bloom extraction: {:?}", findings.bloom_extract);
    let extracted: Vec<f32> = findings.bloom_extract.iter().map(|(_, k)| *k).collect();
    assert!(
        extracted[0].abs() < 1e-6,
        "black contributes nothing: {:?}",
        findings.bloom_extract
    );
    for (value, kept) in &findings.bloom_extract {
        assert!(
            *kept >= 0.0 && kept <= value,
            "the bright pass never brightens: {value} -> {kept}"
        );
    }
    assert!(
        extracted.windows(2).all(|pair| pair[0] <= pair[1]),
        "a brighter pixel contributes at least as much: {:?}",
        findings.bloom_extract
    );
    assert!(
        extracted[4] > extracted[1],
        "and strictly more once it is well over the threshold: {:?}",
        findings.bloom_extract
    );

    // Quality presets are ordered and distinct for both passes.
    for (label, values) in [
        ("bloom iterations", &findings.bloom_iterations),
        ("ssao samples", &findings.ssao_samples),
    ] {
        assert_eq!(values.len(), 4, "{label}: four presets were measured");
        assert!(
            values.windows(2).all(|pair| pair[0] <= pair[1]),
            "{label} do not decrease with quality: {values:?}"
        );
        assert!(
            values[0] < values[3],
            "{label} are not all the same: {values:?}"
        );
    }

    // The SSAO kernel is the pass's own, and it is a unit hemisphere.
    // The kernel is upstream's fixed sample pool, not the frame's sample
    // count: asking for twenty-four samples does not shrink it.
    assert!(
        findings.ssao_kernel_len >= 24,
        "the kernel holds at least the requested sample count: {}",
        findings.ssao_kernel_len
    );
    for (x, y, z) in &findings.ssao_kernel {
        let length = (x * x + y * y + z * z).sqrt();
        assert!(
            length <= 1.001,
            "every kernel sample is inside the unit sphere: {length}"
        );
        assert!(
            *z >= -0.001,
            "and in the hemisphere the normal points into: {z}"
        );
    }

    // The lens equation: nothing is out of focus at the focus distance, and
    // more is out of focus the further away it gets.
    println!("circle of confusion: {:?}", findings.coc);
    let at_focus = findings
        .coc
        .iter()
        .find(|(distance, _)| (*distance - 10.0).abs() < 1e-6)
        .map(|(_, coc)| *coc)
        .expect("a sample at the focus distance");
    assert!(
        at_focus.abs() < 1e-4,
        "the circle of confusion vanishes at the focus distance: {at_focus}"
    );
    let far = findings
        .coc
        .iter()
        .find(|(distance, _)| (*distance - 20.0).abs() < 1e-6)
        .map(|(_, coc)| *coc)
        .expect("a far sample");
    assert!(far > at_focus, "and grows beyond it: {far} vs {at_focus}");

    // Optical depth is zero over no distance and grows with it.
    println!("optical depth: {:?}", findings.optical_depth);
    assert!(
        findings.optical_depth[0].1.abs() < 1e-6,
        "no distance means no fog: {:?}",
        findings.optical_depth[0]
    );
    assert!(
        findings.optical_depth[1].1 < findings.optical_depth[2].1,
        "more distance means more fog: {:?}",
        findings.optical_depth
    );
    // A level ray at the fog's own base height integrates to exactly density
    // times distance, which is the closed form the exponential collapses to.
    assert!(
        (findings.optical_depth[2].1 - 0.02 * 100.0).abs() < 1e-3,
        "a level ray at the base height integrates to density times distance: {:?}",
        findings.optical_depth[2]
    );

    // The contact-shadow march's own test has a boundary, and it is where the
    // sample lies in front of the ray by more than the bias and less than the
    // thickness.
    println!("contact-shadow occlusion: {:?}", findings.occluded);
    assert!(
        findings.occluded.iter().any(|(_, hit)| *hit),
        "some sample occludes: {:?}",
        findings.occluded
    );
    assert!(
        findings.occluded.iter().any(|(_, hit)| !*hit),
        "and some does not: {:?}",
        findings.occluded
    );

    // Combining a shadow map's visibility with a contact shadow's is the
    // product: neither can brighten the other, and full visibility on one side
    // leaves the other untouched.
    println!("combined visibility: {:?}", findings.combined_visibility);
    for (map, contact, combined) in &findings.combined_visibility {
        assert!(
            (*combined - map * contact).abs() < 1e-5,
            "the two visibilities multiply: {map} and {contact} gave {combined}"
        );
        assert!(
            *combined <= map.min(*contact) + 1e-6,
            "so a contact shadow only ever darkens: {map} and {contact} gave {combined}"
        );
    }

    // Transmittance is a fraction that never rises with air mass. It does not
    // start at one -- the model keeps a floor -- so the assertion is the
    // ordering and the range, which is what a caller can rely on.
    println!("transmittance: {:?}", findings.transmittance);
    assert!(
        findings
            .transmittance
            .iter()
            .all(|(_, value)| *value > 0.0 && *value <= 1.0),
        "transmittance is a fraction: {:?}",
        findings.transmittance
    );
    assert!(
        findings
            .transmittance
            .windows(2)
            .all(|pair| pair[0].1 >= pair[1].1),
        "more air never means more light: {:?}",
        findings.transmittance
    );
    assert!(
        findings.transmittance[0].1 > findings.transmittance[2].1,
        "and four air masses attenuate more than none: {:?}",
        findings.transmittance
    );
    assert!(
        findings.air_mass[0].1.abs() < 1e-6
            && findings.air_mass[1].1 < findings.air_mass[2].1,
        "air mass starts at zero and grows with distance: {:?}",
        findings.air_mass
    );

    // An identity resample is exactly the one where nothing changes size.
    for (sizes, identity) in &findings.identity_scale {
        let expected = sizes.0 == sizes.2 && sizes.1 == sizes.3;
        assert_eq!(
            *identity, expected,
            "{sizes:?} is an identity scale only when the sizes match"
        );
    }

    // A 256x16 strip carries sixteen slices of sixteen texels, and a 1024x32
    // strip carries thirty-two.
    assert_eq!(
        findings.lut_size,
        vec![((256, 16), 16), ((1024, 32), 32)],
        "a strip's slice count is its height"
    );
    assert_eq!(
        findings.identity_lut_size,
        Some((256, 16)),
        "a sixteen-slice identity table is a 256x16 strip"
    );
    assert_eq!(
        findings.has_strip_lut,
        Some(true),
        "the attached table is the one the pass reports"
    );

    assert_eq!(
        findings.scope_recorded,
        Some(true),
        "a scope records the binding it replaced"
    );
    // The borrowed ASCII effect is CNAEXT's own type, and reading its cell size
    // through the borrow is what proves the handle names that effect rather
    // than merely being non-zero.
    let (cell_width, cell_height) = findings
        .ascii_cell_size
        .expect("the ASCII pass lends its effect");
    assert!(
        cell_width > 0 && cell_height > 0,
        "the ASCII cell is a real size: {cell_width}x{cell_height}"
    );
    for (label, bytes) in &findings.glsl_bytes {
        if label.ends_with("fallback") {
            continue;
        }
        assert!(*bytes > 100, "{label} GLSL is real source, {bytes} bytes");
    }

    println!(
        "screen-space passes: {} knobs round-tripped | support {:?} | glsl {:?}",
        findings.round_trips.len(),
        findings.supported,
        findings.glsl_bytes
    );
}

/// What the shadow-variant run measured.
#[derive(Default)]
struct VariantFindings {
    engine_layer: i32,
    spot_supported: bool,
    spot_size: i32,
    spot_position: Option<(f32, f32, f32)>,
    spot_range: f32,
    spot_view_projection_matches: Option<bool>,
    cube_supported: bool,
    cube_size: i32,
    cube_size_for_quality: i32,
    cube_position: Option<(f32, f32, f32)>,
    cube_face_views: Vec<(f32, f32, f32)>,
    cube_face_projection_symmetric: Option<bool>,
    cascade_supported: bool,
    cascade_count: i32,
    cascade_size: i32,
    split_distances: Vec<f32>,
    selected: Vec<(f32, i32)>,
    frustum_corners: Vec<(f32, f32, f32)>,
    bounding_radius: f32,
    snap_moved: Option<(f32, f32)>,
    cascade_state_defaults: Option<(i32, f32, bool)>,
    punctual_defaults: Option<(PunctualLightKind, bool, bool)>,
    round_trips: Vec<(&'static str, f32, f32)>,
}

struct VariantGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<VariantFindings>>,
}

impl GameStateAccess for VariantGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for VariantGame {
    #[allow(clippy::too_many_lines)]
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let mut findings = VariantFindings {
            engine_layer: version,
            ..VariantFindings::default()
        };

        // Pure values first.
        let state = ShadowCascadeState::canonical_defaults()?;
        findings.cascade_state_defaults = Some((state.count, state.blend_band, state.debug_tint));
        let punctual = PunctualLight::canonical_defaults()?;
        findings.punctual_defaults = Some((
            punctual.kind,
            punctual.has_shadow_cube,
            punctual.has_shadow_map,
        ));

        // A spot light's shadow transform is a pure function of the light, and
        // the map must cast from exactly that. Comparing them is what catches a
        // map casting from a frustum the caller cannot predict.
        let mut light = SpotLight::canonical_defaults()?;
        light.position = Vector3::from_x_and_y_and_z(2.0, 6.0, -3.0);
        light.direction = Vector3::from_x_and_y_and_z(0.0, -1.0, 0.0);
        light.range = 25.0;
        light.inner_angle = 0.3;
        light.outer_angle = 0.6;

        let spot = SpotShadowMap::new(&device, ShadowQuality::Medium)?;
        findings.spot_supported = spot.is_supported()?;
        findings.spot_size = spot.size()?;
        spot.set_depth_bias(0.0015)?;
        findings
            .round_trips
            .push(("spot.depth_bias", 0.0015, spot.depth_bias()?));
        spot.begin(light)?;
        spot.end()?;
        let position = spot.light_position()?;
        findings.spot_position = Some((position.X, position.Y, position.Z));
        findings.spot_range = spot.light_range()?;
        let expected = light.compute_light_view()? * light.compute_light_projection()?;
        findings.spot_view_projection_matches = Some(spot.light_view_projection()? == expected);

        let mut point = PointLight::canonical_defaults()?;
        point.position = Vector3::from_x_and_y_and_z(-1.0, 3.0, 4.0);
        point.range = 12.0;

        let cube = CubeShadowMap::new(&device, ShadowQuality::Low)?;
        findings.cube_supported = cube.is_supported()?;
        findings.cube_size = cube.size()?;
        findings.cube_size_for_quality = CubeShadowMap::size_for_quality(ShadowQuality::Low)?;
        cube.set_depth_bias(0.002)?;
        findings
            .round_trips
            .push(("cube.depth_bias", 0.002, cube.depth_bias()?));
        cube.update(point)?;
        let position = cube.light_position()?;
        findings.cube_position = Some((position.X, position.Y, position.Z));
        findings
            .round_trips
            .push(("cube.light_range", 12.0, cube.light_range()?));

        // The six face views must look six different ways. Transforming the
        // same world point through each and collecting the results is how a
        // table that had collapsed to one face shows up.
        for face in 0..6 {
            let view = CubeShadowMap::compute_face_view(face, point.position)?;
            // An asymmetric offset, so no two faces can map it to the same
            // view-space point: a probe along one axis alone would collide on
            // the four faces perpendicular to it and prove nothing.
            let seen = Vector3::Transform(
                Vector3::from_x_and_y_and_z(
                    point.position.X + 0.3,
                    point.position.Y + 0.7,
                    point.position.Z + 1.1,
                ),
                view,
            );
            findings.cube_face_views.push((seen.X, seen.Y, seen.Z));
        }
        let projection = CubeShadowMap::compute_face_projection(point.range)?;
        findings.cube_face_projection_symmetric = Some(
            (projection.M11 - projection.M22).abs() < 1e-5
                && projection.M12.abs() < 1e-6
                && projection.M21.abs() < 1e-6,
        );

        let cascaded = CascadedShadowMap::new(&device, ShadowQuality::Medium, 4)?;
        findings.cascade_supported = cascaded.is_supported()?;
        findings.cascade_count = cascaded.cascade_count()?;
        findings.cascade_size = cascaded.cascade_size()?;
        cascaded.set_split_lambda(0.75)?;
        cascaded.set_blend_band(0.15)?;
        cascaded.set_debug_tint_enabled(true)?;
        findings
            .round_trips
            .push(("cascade.split_lambda", 0.75, cascaded.split_lambda()?));
        findings
            .round_trips
            .push(("cascade.blend_band", 0.15, cascaded.blend_band()?));
        findings.round_trips.push((
            "cascade.debug_tint",
            1.0,
            f32::from(u8::from(cascaded.is_debug_tint_enabled()?)),
        ));

        let sun = DirectionalLight::canonical_defaults()?;
        let camera_view = Matrix::CreateLookAt(
            Vector3::from_x_and_y_and_z(0.0, 2.0, 10.0),
            Vector3::Zero,
            Vector3::Up,
        );
        let camera_projection =
            Matrix::CreatePerspectiveFieldOfView(1.0, 16.0 / 9.0, 0.5, 200.0);
        cascaded.update(sun, camera_view, camera_projection)?;
        for index in 0..cascaded.cascade_count()? {
            findings.split_distances.push(cascaded.split_distance(index)?);
        }
        for depth in [0.0_f32, 1.0, 20.0, 150.0, 500.0] {
            findings.selected.push((depth, cascaded.select_cascade(depth)?));
        }

        let corners = CascadedShadowMap::compute_frustum_corners(camera_view, camera_projection)?;
        findings.frustum_corners = corners.iter().map(|c| (c.X, c.Y, c.Z)).collect();
        let (centre, radius) = CascadedShadowMap::compute_bounding_sphere(&corners)?;
        findings.bounding_radius = radius;
        let snapped = CascadedShadowMap::snap_to_texel_grid(centre, radius, 1024)?;
        findings.snap_moved = Some((
            (snapped.X - centre.X).abs(),
            2.0 * radius / 1024.0,
        ));

        *self.findings.lock().expect("findings") = findings;
        Ok(())
    }
}

#[test]
fn the_shadow_map_variants_cast_from_the_transforms_they_publish() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(VariantFindings::default()));
    let game = VariantGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame with three shadow-map variants");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    for (label, expected, actual) in &findings.round_trips {
        assert!(
            (expected - actual).abs() < 1e-5,
            "{label} round-trips: set {expected}, read {actual}"
        );
    }

    // A spot map casts from exactly the composition of the two pure functions,
    // and remembers the light it was opened for.
    assert_eq!(
        findings.spot_position,
        Some((2.0, 6.0, -3.0)),
        "the map remembers where its light was"
    );
    assert!(
        (findings.spot_range - 25.0).abs() < 1e-5,
        "and how far it reached: {}",
        findings.spot_range
    );
    assert_eq!(
        findings.spot_view_projection_matches,
        Some(true),
        "the spot map casts from the transform its own pure functions compute"
    );
    assert!(
        u32::try_from(findings.spot_size).is_ok_and(u32::is_power_of_two),
        "the spot map is a power-of-two square: {}",
        findings.spot_size
    );

    assert_eq!(
        findings.cube_position,
        Some((-1.0, 3.0, 4.0)),
        "the cube remembers where its light was"
    );
    assert_eq!(
        findings.cube_size, findings.cube_size_for_quality,
        "the cube's size is the one its preset selects"
    );
    // Six faces, six different views of one point.
    println!("cube face views: {:?}", findings.cube_face_views);
    let distinct: std::collections::BTreeSet<(u32, u32, u32)> = findings
        .cube_face_views
        .iter()
        .map(|(x, y, z)| (x.to_bits(), y.to_bits(), z.to_bits()))
        .collect();
    assert_eq!(
        distinct.len(),
        6,
        "the six cube faces look six different ways: {:?}",
        findings.cube_face_views
    );
    assert_eq!(
        findings.cube_face_projection_symmetric,
        Some(true),
        "a cube face's projection is the square ninety-degree frustum it has to be"
    );

    // Four cascades, strictly increasing splits, and a depth lands in the
    // cascade its split covers.
    assert_eq!(findings.cascade_count, 4, "four cascades were allocated");
    assert!(
        findings.cascade_size > 0,
        "each cascade has a size: {}",
        findings.cascade_size
    );
    println!("cascade splits: {:?}", findings.split_distances);
    assert_eq!(findings.split_distances.len(), 4, "one split per cascade");
    assert!(
        findings
            .split_distances
            .windows(2)
            .all(|pair| pair[0] < pair[1]),
        "the splits strictly increase: {:?}",
        findings.split_distances
    );
    println!("cascade selection: {:?}", findings.selected);
    for (depth, index) in &findings.selected {
        assert!(
            (0..findings.cascade_count).contains(index),
            "every depth lands in a real cascade: {depth} -> {index}"
        );
        let expected = findings
            .split_distances
            .iter()
            .position(|split| depth <= split)
            .unwrap_or(findings.split_distances.len() - 1);
        assert_eq!(
            *index as usize, expected,
            "a depth lands in the first cascade whose split covers it: {depth} -> {index}"
        );
    }

    // Eight corners, a sphere that encloses them, and a snap that moves the
    // centre by less than one texel of the map it snaps to.
    assert_eq!(findings.frustum_corners.len(), 8, "a frustum has eight corners");
    assert!(
        findings.bounding_radius > 0.0,
        "the enclosing sphere has a radius: {}",
        findings.bounding_radius
    );
    let (moved, texel) = findings.snap_moved.expect("a snapped centre");
    assert!(
        moved <= texel + 1e-4,
        "snapping moves the centre by at most one texel: moved {moved}, texel {texel}"
    );

    let (count, blend, tint) = findings
        .cascade_state_defaults
        .expect("the cascade state defaults");
    assert!(
        count >= 0 && blend >= 0.0 && !tint,
        "CNA's cascade-state defaults are a real, untinted starting state: {count}, {blend}, {tint}"
    );
    assert_eq!(
        findings.punctual_defaults,
        Some((PunctualLightKind::None, false, false)),
        "a default punctual light is no light with no shadow resources"
    );

    println!(
        "shadow variants: spot supported={} size={} | cube supported={} size={} | \
         cascaded supported={} count={} size={}",
        findings.spot_supported,
        findings.spot_size,
        findings.cube_supported,
        findings.cube_size,
        findings.cascade_supported,
        findings.cascade_count,
        findings.cascade_size,
    );
}

/// What the prepass and transparency run measured.
#[derive(Default)]
struct FrameFindings {
    engine_layer: i32,
    prepass_supported: bool,
    prepass_pass_count: i32,
    prepass_mrt: Option<bool>,
    prepass_packed: Option<bool>,
    prepass_device_packed: Option<bool>,
    prepass_depth_size: Option<(i32, i32)>,
    prepass_normal_size: Option<(i32, i32)>,
    prepass_velocity_before: Option<bool>,
    prepass_velocity_after: Option<bool>,
    depth_round_trip: Vec<(f32, f32)>,
    velocity_samples: Vec<(u32, bool, f32, f32)>,
    glsl_bytes: Vec<(&'static str, usize)>,
    round_trips: Vec<(&'static str, f32, f32)>,
    oit_supported: bool,
    oit_reason: String,
    oit_accumulating: Vec<bool>,
    oit_weights: Vec<(f32, f32)>,
    list_counts: Vec<u64>,
    list_order: Vec<i32>,
    list_draws: usize,
    sort_keys: Vec<(f32, f32)>,
    camera_position: Option<(f32, f32, f32)>,
}

struct FrameGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<FrameFindings>>,
}

impl GameStateAccess for FrameGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for FrameGame {
    #[allow(clippy::too_many_lines)]
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let mut findings = FrameFindings {
            engine_layer: version,
            ..FrameFindings::default()
        };

        // The packing pair is a pure round trip, and the only thing a packed
        // depth buffer relies on.
        for value in [0.0_f32, 0.125, 0.5, 0.874_321, 1.0] {
            let (r, g, b, a) = DepthNormalPrepass::pack_depth(value)?;
            findings
                .depth_round_trip
                .push((value, DepthNormalPrepass::unpack_depth(r, g, b, a)?));
        }
        // The velocity encoding's two routes have to agree: a texel that
        // "carries no velocity" must decode to no motion, and one that does
        // must decode to some. Sampling a spread of texels is how a predicate
        // that answered the same thing for all of them shows up.
        for texel in [
            Color::Transparent,
            Color::Black,
            Color::White,
            Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(128, 128, 0, 0),
            Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
                128, 128, 255, 255,
            ),
        ] {
            let motion = DepthNormalPrepass::decode_velocity(texel)?;
            findings.velocity_samples.push((
                texel.PackedValue(),
                DepthNormalPrepass::has_velocity(texel)?,
                motion.X,
                motion.Y,
            ));
        }
        findings.prepass_device_packed = Some(DepthNormalPrepass::uses_packed_depth(&device)?);
        findings.glsl_bytes.push((
            "depth-decode",
            DepthNormalPrepass::depth_decode_glsl(true)?.len(),
        ));
        findings.glsl_bytes.push((
            "velocity-decode",
            DepthNormalPrepass::velocity_decode_glsl()?.len(),
        ));
        findings.glsl_bytes.push((
            "oit-accumulation",
            WeightedBlendedTransparency::accumulation_glsl()?.len(),
        ));

        let prepass = DepthNormalPrepass::new(&device, 128, 96, DepthEncoding::Automatic)?;
        findings.prepass_supported = prepass.is_supported(&device)?;
        findings.prepass_pass_count = prepass.pass_count()?;
        findings.prepass_mrt = Some(prepass.is_using_multiple_render_targets()?);
        findings.prepass_packed = Some(prepass.is_depth_packed()?);
        findings.prepass_velocity_before = Some(prepass.is_velocity_enabled()?);
        prepass.set_velocity_enabled(true)?;
        findings.prepass_velocity_after = Some(prepass.is_velocity_enabled()?);
        prepass.set_roughness(0.375)?;
        findings
            .round_trips
            .push(("prepass.roughness", 0.375, prepass.roughness()?));
        if let Some(depth) = prepass.depth_texture()? {
            findings.prepass_depth_size =
                Some((depth.texture().Width(), depth.texture().Height()));
        }
        if let Some(normals) = prepass.normal_texture()? {
            findings.prepass_normal_size =
                Some((normals.texture().Width(), normals.texture().Height()));
        }

        let oit = WeightedBlendedTransparency::new(&device, 64, 64)?;
        findings.oit_supported = oit.is_supported()?;
        findings.oit_reason = oit.unsupported_reason()?;
        findings.oit_accumulating.push(oit.is_accumulating()?);
        if findings.oit_supported {
            oit.begin(100.0)?;
            findings.oit_accumulating.push(oit.is_accumulating()?);
            oit.end()?;
            findings.oit_accumulating.push(oit.is_accumulating()?);
        }
        for depth in [1.0_f32, 10.0, 50.0] {
            findings
                .oit_weights
                .push((depth, WeightedBlendedTransparency::weight(depth, 0.5, 100.0)?));
        }

        // The sorted list draws back to front, so the entry furthest from the
        // camera comes first. Submitting three boxes at known distances and
        // reading the order back is the whole of that claim.
        let view = Matrix::CreateLookAt(
            Vector3::from_x_and_y_and_z(0.0, 0.0, 0.0),
            Vector3::from_x_and_y_and_z(0.0, 0.0, -1.0),
            Vector3::Up,
        );
        let camera = TransparentDrawList::camera_position_of(view)?;
        findings.camera_position = Some((camera.X, camera.Y, camera.Z));
        for distance in [5.0_f32, 40.0] {
            let bounds = BoundingBox::new(
                Vector3::from_x_and_y_and_z(-1.0, -1.0, -distance - 1.0),
                Vector3::from_x_and_y_and_z(1.0, 1.0, -distance + 1.0),
            );
            findings
                .sort_keys
                .push((distance, TransparentDrawList::sort_key(bounds, camera)?));
        }

        let drawn = Arc::new(AtomicUsize::new(0));
        let mut list = TransparentDrawList::new()?;
        findings.list_counts.push(list.count()?);
        for distance in [5.0_f32, 40.0, 20.0] {
            let bounds = BoundingBox::new(
                Vector3::from_x_and_y_and_z(-1.0, -1.0, -distance - 1.0),
                Vector3::from_x_and_y_and_z(1.0, 1.0, -distance + 1.0),
            );
            let counter = Arc::clone(&drawn);
            list.submit(bounds, move || {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })?;
        }
        findings.list_counts.push(list.count()?);
        findings.list_order = list.sorted_order(view)?;
        list.draw_sorted(view)?;
        findings.list_draws = drawn.load(Ordering::SeqCst);
        list.clear()?;
        findings.list_counts.push(list.count()?);

        *self.findings.lock().expect("findings") = findings;
        Ok(())
    }
}

#[test]
fn the_prepass_packs_depth_and_the_transparency_sorts_back_to_front() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(FrameFindings::default()));
    let game = FrameGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame with a prepass and both transparency paths");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    for (label, expected, actual) in &findings.round_trips {
        assert!(
            (expected - actual).abs() < 1e-5,
            "{label} round-trips: set {expected}, read {actual}"
        );
    }

    // Packing and unpacking a depth is a round trip a packed buffer's every
    // reader depends on. Eight bits per channel over four channels leaves room
    // for far better than a thousandth.
    println!("depth pack round trip: {:?}", findings.depth_round_trip);
    for (value, back) in &findings.depth_round_trip {
        assert!(
            (value - back).abs() < 1e-3,
            "packing and unpacking {value} gives {back} back"
        );
    }
    println!("velocity samples: {:?}", findings.velocity_samples);
    for (texel, has, x, y) in &findings.velocity_samples {
        let moved = x.abs() + y.abs() > 1e-6;
        assert_eq!(
            *has, moved,
            "texel {texel:#010x}: has_velocity says {has} and decode gives ({x}, {y})"
        );
    }
    assert!(
        findings.velocity_samples.iter().any(|(_, has, _, _)| *has)
            && findings.velocity_samples.iter().any(|(_, has, _, _)| !*has),
        "some texels carry motion and some do not: {:?}",
        findings.velocity_samples
    );

    assert!(
        findings.prepass_pass_count >= 1,
        "the prepass needs at least one pass: {}",
        findings.prepass_pass_count
    );
    // One pass with multiple render targets, more without. The two answers are
    // the same fact, so they must agree.
    assert_eq!(
        findings.prepass_mrt == Some(true),
        findings.prepass_pass_count == 1,
        "a single-pass prepass is the one using multiple render targets: {} passes, mrt {:?}",
        findings.prepass_pass_count,
        findings.prepass_mrt
    );
    assert_eq!(
        findings.prepass_packed, findings.prepass_device_packed,
        "the prepass's encoding is the one the device would have chosen"
    );
    assert_eq!(
        findings.prepass_depth_size,
        Some((128, 96)),
        "the depth target is the size the prepass was created at"
    );
    assert_eq!(
        findings.prepass_normal_size,
        Some((128, 96)),
        "and so is the normal target"
    );
    assert_eq!(findings.prepass_velocity_before, Some(false));
    assert_eq!(
        findings.prepass_velocity_after,
        Some(true),
        "the velocity target turns on when asked"
    );

    // The weight falls off with depth, which is what makes the accumulation
    // order-independent: a nearer fragment must weigh more.
    println!("oit weights: {:?}", findings.oit_weights);
    assert!(
        findings
            .oit_weights
            .windows(2)
            .all(|pair| pair[0].1 > pair[1].1),
        "a nearer fragment weighs more: {:?}",
        findings.oit_weights
    );
    if findings.oit_supported {
        assert_eq!(
            findings.oit_accumulating,
            vec![false, true, false],
            "the accumulation is open only between begin and end"
        );
        assert!(
            findings.oit_reason.is_empty(),
            "a supported accumulation has nothing to explain: {:?}",
            findings.oit_reason
        );
    } else {
        assert!(
            !findings.oit_reason.is_empty(),
            "an unsupported accumulation says why"
        );
        println!("order-independent transparency unsupported: {}", findings.oit_reason);
    }

    // Three entries at 5, 40 and 20 units, drawn back to front: the order is
    // the middle index first, then the last, then the first.
    assert_eq!(
        findings.list_counts,
        vec![0, 3, 0],
        "the list counts what was submitted and forgets it on clear"
    );
    println!("sorted order: {:?}", findings.list_order);
    assert_eq!(
        findings.list_order,
        vec![1, 2, 0],
        "the list draws the furthest entry first: 40, then 20, then 5"
    );
    assert_eq!(
        findings.list_draws, 3,
        "every entry's callback ran exactly once"
    );
    assert_eq!(
        findings.camera_position,
        Some((0.0, 0.0, 0.0)),
        "the camera position is the one the view matrix implies"
    );
    // The sort key grows with distance, which is what the order above follows
    // from rather than restating it.
    assert!(
        findings.sort_keys[0].1 < findings.sort_keys[1].1,
        "a farther box sorts later: {:?}",
        findings.sort_keys
    );

    for (label, bytes) in &findings.glsl_bytes {
        assert!(*bytes > 50, "{label} GLSL is real source, {bytes} bytes");
    }

    println!(
        "frame plumbing: prepass supported={} passes={} packed={:?} | oit supported={} | \
         glsl {:?}",
        findings.prepass_supported,
        findings.prepass_pass_count,
        findings.prepass_packed,
        findings.oit_supported,
        findings.glsl_bytes,
    );
}

/// A three-entry `.cube` document, small enough to check by hand.
///
/// A size-two table has eight entries in blue-slowest order, and this one is
/// the identity over a non-unit domain -- which is what makes both the entry
/// lookup and the domain query say something a wrong parse would get wrong.
const IDENTITY_CUBE: &str = "TITLE \"probe\"\n\
LUT_3D_SIZE 2\n\
DOMAIN_MIN 0.0 0.0 0.0\n\
DOMAIN_MAX 2.0 2.0 2.0\n\
0.0 0.0 0.0\n\
1.0 0.0 0.0\n\
0.0 1.0 0.0\n\
1.0 1.0 0.0\n\
0.0 0.0 1.0\n\
1.0 0.0 1.0\n\
0.0 1.0 1.0\n\
1.0 1.0 1.0\n";

/// What the display and exposure run measured.
#[derive(Default)]
struct DisplayFindings {
    engine_layer: i32,
    hdr_supported: bool,
    hdr_color_space: Option<DisplayColorSpace>,
    pq_round_trip: Vec<(f32, f32)>,
    rec2020: Option<(f32, f32, f32)>,
    roll_off: Vec<(f32, f32)>,
    srgb_encode: Option<(f32, f32, f32)>,
    hdr10_encode: Option<(f32, f32, f32)>,
    exposure_round_trip: Vec<(&'static str, f32, f32)>,
    exposure_applied: Option<f32>,
    lut_title: String,
    lut_size: i32,
    lut_unit_domain: Option<bool>,
    lut_domain_max: Option<(f32, f32, f32)>,
    lut_entries: Vec<(i32, i32, i32, f32, f32, f32)>,
    lut_strip_size: Option<(i32, i32)>,
    lut_parse_refusal: Option<String>,
}

struct DisplayGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<DisplayFindings>>,
}

impl GameStateAccess for DisplayGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for DisplayGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let mut findings = DisplayFindings {
            engine_layer: version,
            ..DisplayFindings::default()
        };

        // The PQ curve is an exact pair, and this host has no HDR display to
        // show it on -- which is precisely why the pure functions matter.
        for nits in [0.1_f32, 1.0, 100.0, 1_000.0, 10_000.0] {
            let signal = HdrDisplayOutput::encode_pq(nits)?;
            findings
                .pq_round_trip
                .push((nits, HdrDisplayOutput::decode_pq(signal)?));
        }
        let wide = HdrDisplayOutput::rec709_to_rec2020(Vector3::from_x_and_y_and_z(1.0, 1.0, 1.0))?;
        findings.rec2020 = Some((wide.X, wide.Y, wide.Z));
        for value in [0.0_f32, 0.5, 1.0, 4.0] {
            findings
                .roll_off
                .push((value, HdrDisplayOutput::roll_off(value, 1.0)?));
        }
        let grey = Vector3::from_x_and_y_and_z(0.5, 0.5, 0.5);
        let srgb = HdrDisplayOutput::encode(DisplayColorSpace::Srgb, grey, 100.0, 1_000.0)?;
        let hdr10 = HdrDisplayOutput::encode(DisplayColorSpace::Hdr10, grey, 100.0, 1_000.0)?;
        findings.srgb_encode = Some((srgb.X, srgb.Y, srgb.Z));
        findings.hdr10_encode = Some((hdr10.X, hdr10.Y, hdr10.Z));

        let output = HdrDisplayOutput::new(&device)?;
        findings.hdr_supported = output.is_supported()?;
        output.set_color_space(DisplayColorSpace::Hdr10)?;
        output.set_paper_white_nits(203.0)?;
        output.set_peak_nits(1_500.0)?;
        findings.hdr_color_space = output.color_space().ok();
        findings.exposure_round_trip.push((
            "hdr.paper_white",
            203.0,
            output.paper_white_nits()?,
        ));
        findings
            .exposure_round_trip
            .push(("hdr.peak", 1_500.0, output.peak_nits()?));

        let exposure = AutoExposure::new(&device)?;
        exposure.set_exposure(1.75)?;
        exposure.set_key_value(0.18)?;
        exposure.set_adaptation_speeds(3.0, 1.5)?;
        exposure.set_exposure_range(0.05, 8.0)?;
        findings
            .exposure_round_trip
            .push(("exposure.value", 1.75, exposure.exposure()?));
        findings
            .exposure_round_trip
            .push(("exposure.key", 0.18, exposure.key_value()?));
        findings
            .exposure_round_trip
            .push(("exposure.brightening", 3.0, exposure.brightening_speed()?));
        findings
            .exposure_round_trip
            .push(("exposure.darkening", 1.5, exposure.darkening_speed()?));
        // Writing into the pipeline settings is the adaptation's whole output,
        // so the value that lands there must be the one it settled on.
        let mut settings = EngineRenderSettings::canonical_defaults()?;
        exposure.apply_to(&mut settings)?;
        findings.exposure_applied = Some(settings.exposure());

        let lut = CubeLut::parse(IDENTITY_CUBE)?;
        findings.lut_title = lut.title()?;
        findings.lut_size = lut.size()?;
        findings.lut_unit_domain = Some(lut.is_unit_domain()?);
        let domain_max = lut.domain_max()?;
        findings.lut_domain_max = Some((domain_max.X, domain_max.Y, domain_max.Z));
        for (r, g, b) in [(0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 1)] {
            let entry = lut.entry(r, g, b)?;
            findings
                .lut_entries
                .push((r, g, b, entry.X, entry.Y, entry.Z));
        }
        let strip = lut.create_strip_texture(&device)?;
        findings.lut_strip_size = Some((strip.Width(), strip.Height()));
        findings.lut_parse_refusal = CubeLut::parse("this is not a cube file")
            .err()
            .map(|error| error.to_string());

        *self.findings.lock().expect("findings") = findings;
        Ok(())
    }
}

#[test]
fn the_display_encode_and_the_lut_parser_answer_with_exact_values() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(DisplayFindings::default()));
    let game = DisplayGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame with an HDR output, an exposure and a LUT");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    for (label, expected, actual) in &findings.exposure_round_trip {
        assert!(
            (expected - actual).abs() < 1e-3,
            "{label} round-trips: set {expected}, read {actual}"
        );
    }
    assert_eq!(
        findings.hdr_color_space,
        Some(DisplayColorSpace::Hdr10),
        "the colour space round-trips as an identity, not as a number"
    );

    // PQ is a transfer function and its inverse, so the round trip is exact to
    // within float precision across four decades of brightness.
    println!("PQ round trip: {:?}", findings.pq_round_trip);
    for (nits, back) in &findings.pq_round_trip {
        assert!(
            (nits - back).abs() <= nits.abs() * 1e-3 + 1e-4,
            "encoding {nits} nits and decoding it back gives {back}"
        );
    }
    // And it is a real curve, not the identity: a hundred nits does not encode
    // to a hundred.
    let hundred = findings
        .pq_round_trip
        .iter()
        .find(|(nits, _)| (*nits - 100.0).abs() < 1e-6)
        .map(|(_, _)| HdrDisplayOutput::encode_pq(100.0).expect("encode"))
        .expect("a hundred-nit sample");
    assert!(
        (0.0..=1.0).contains(&hundred) && (hundred - 100.0).abs() > 1.0,
        "PQ maps nits into a unit signal: 100 nits -> {hundred}"
    );

    // Rec.709 white is Rec.2020 white: the conversion preserves the white
    // point, which is the one property that pins the matrix's rows.
    let (r, g, b) = findings.rec2020.expect("a converted white");
    assert!(
        (r - 1.0).abs() < 1e-3 && (g - 1.0).abs() < 1e-3 && (b - 1.0).abs() < 1e-3,
        "white stays white across the primaries: {r}, {g}, {b}"
    );

    // The roll-off never exceeds the peak and never lowers a value already
    // below it by more than the shoulder it applies.
    println!("roll off: {:?}", findings.roll_off);
    assert!(
        findings.roll_off[0].1.abs() < 1e-6,
        "black rolls off to black: {:?}",
        findings.roll_off[0]
    );
    for (value, rolled) in &findings.roll_off {
        assert!(
            *rolled <= 1.000_01,
            "nothing exceeds the peak after roll-off: {value} -> {rolled}"
        );
    }
    assert!(
        findings.roll_off.windows(2).all(|pair| pair[0].1 <= pair[1].1),
        "the roll-off is monotonic: {:?}",
        findings.roll_off
    );

    // sRGB and HDR10 are different encodes of one colour. Two colour spaces
    // that agreed would mean the space argument never reached the maths.
    let srgb = findings.srgb_encode.expect("an sRGB encode");
    let hdr10 = findings.hdr10_encode.expect("an HDR10 encode");
    assert!(
        (srgb.0 - hdr10.0).abs() > 1e-4,
        "the colour space changes the encode: {srgb:?} vs {hdr10:?}"
    );

    assert_eq!(
        findings.exposure_applied,
        Some(1.75),
        "the exposure the adaptation settled on is the one it writes into the settings"
    );

    // The parsed table is the document, entry for entry.
    assert_eq!(findings.lut_title, "probe", "the title comes from the document");
    assert_eq!(findings.lut_size, 2, "a LUT_3D_SIZE of two parses as two");
    assert_eq!(
        findings.lut_unit_domain,
        Some(false),
        "a domain of zero to two is not the unit cube"
    );
    assert_eq!(
        findings.lut_domain_max,
        Some((2.0, 2.0, 2.0)),
        "and the domain maximum is the one the document declared"
    );
    println!("lut entries: {:?}", findings.lut_entries);
    // Blue varies slowest in a `.cube` file, so entry (1, 0, 0) is the second
    // line and entry (0, 0, 1) is the fifth. Reading them back in the wrong
    // order is the single most likely parser bug there is.
    assert_eq!(
        findings.lut_entries,
        vec![
            (0, 0, 0, 0.0, 0.0, 0.0),
            (1, 0, 0, 1.0, 0.0, 0.0),
            (0, 1, 0, 0.0, 1.0, 0.0),
            (0, 0, 1, 0.0, 0.0, 1.0),
            (1, 1, 1, 1.0, 1.0, 1.0),
        ],
        "every entry is the line the document put at that grid position"
    );
    // A size-two table strips to two 2x2 slices side by side.
    assert_eq!(
        findings.lut_strip_size,
        Some((4, 2)),
        "a two-entry table strips to a 4x2 texture"
    );
    assert!(
        findings.lut_parse_refusal.is_some(),
        "text that is not a .cube document is refused rather than parsed into nothing"
    );

    println!(
        "display: HDR supported={} | sRGB {:?} vs HDR10 {:?} | LUT '{}' size {}",
        findings.hdr_supported,
        findings.srgb_encode,
        findings.hdr10_encode,
        findings.lut_title,
        findings.lut_size,
    );
}

/// What the culling, LOD and debug-draw run measured.
#[derive(Default)]
struct CullingFindings {
    engine_layer: i32,
    line_counts: Vec<(&'static str, i32)>,
    vertices_depth_tested: usize,
    vertices_overlay: usize,
    first_line: Option<((f32, f32, f32), u32)>,
    depth_tested_round_trip: Option<(bool, bool)>,
    box_visibility: Vec<((f32, f32, f32), bool)>,
    sphere_visibility: Vec<(f32, bool)>,
    culled_boxes: Vec<u64>,
    culled_transforms: usize,
    culled_short_bounds: usize,
    frustum_matrix_matches: Option<bool>,
    lod_levels: Vec<(f32, bool)>,
    lod_selection: Vec<(f32, i32)>,
    lod_mode_round_trip: Option<LodSelectionMode>,
    lod_hysteresis: f32,
    lod_sticky: Vec<(&'static str, i32)>,
    projected_radius: Vec<(f32, f32)>,
}

struct CullingGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<CullingFindings>>,
}

impl GameStateAccess for CullingGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

/// A camera looking down -Z from the origin, with a 90-degree field of view.
fn culling_camera() -> (Matrix, Matrix) {
    (
        Matrix::CreateLookAt(
            Vector3::Zero,
            Vector3::from_x_and_y_and_z(0.0, 0.0, -1.0),
            Vector3::Up,
        ),
        Matrix::CreatePerspectiveFieldOfView(std::f32::consts::FRAC_PI_2, 1.0, 1.0, 100.0),
    )
}

fn unit_box_at(x: f32, y: f32, z: f32) -> BoundingBox {
    BoundingBox::new(
        Vector3::from_x_and_y_and_z(x - 0.5, y - 0.5, z - 0.5),
        Vector3::from_x_and_y_and_z(x + 0.5, y + 0.5, z + 0.5),
    )
}

impl Game for CullingGame {
    #[allow(clippy::too_many_lines)]
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let mut findings = CullingFindings {
            engine_layer: version,
            ..CullingFindings::default()
        };
        let (view, projection) = culling_camera();

        // --- frustum culling -----------------------------------------------
        let culler = FrustumCuller::new()?;
        culler.set_camera(view, projection)?;
        // The culler's own frustum must be the camera's view-projection: a
        // culler testing against a different matrix would still answer, just
        // wrongly.
        findings.frustum_matrix_matches =
            Some(culler.frustum()?.Matrix() == view * projection);

        // Straight ahead is inside; behind, and far off to the side, are not.
        let probes = [
            (0.0_f32, 0.0_f32, -10.0_f32),
            (0.0, 0.0, 10.0),
            (0.0, 0.0, -1000.0),
            (100.0, 0.0, -10.0),
        ];
        let mut boxes = Vec::new();
        for (x, y, z) in probes {
            let bounds = unit_box_at(x, y, z);
            findings
                .box_visibility
                .push(((x, y, z), culler.is_box_visible(bounds)?));
            boxes.push(bounds);
        }
        findings.culled_boxes = culler.cull_boxes(&boxes)?;
        for radius in [0.5_f32, 50.0, 70.0] {
            findings.sphere_visibility.push((
                radius,
                culler.is_sphere_visible(BoundingSphere {
                    Center: Vector3::from_x_and_y_and_z(0.0, 0.0, 60.0),
                    Radius: radius,
                })?,
            ));
        }
        let transforms: Vec<Matrix> = probes
            .iter()
            .map(|(x, y, z)| Matrix::CreateTranslation(Vector3::from_x_and_y_and_z(*x, *y, *z)))
            .collect();
        findings.culled_transforms = culler.cull_transforms(&transforms, &boxes)?.len();
        // The documented tail rule: a transform with no bound of its own is
        // kept. Two bounds for four transforms must keep the visible one of the
        // first two plus both of the unpaired ones.
        findings.culled_short_bounds = culler
            .cull_transforms(&transforms, &boxes[..2])?
            .len();

        // --- levels of detail ----------------------------------------------
        let lod = LodGroup::new()?;
        for distance in [10.0_f32, 25.0, 60.0] {
            lod.add_level(distance)?;
        }
        findings.lod_levels = lod
            .levels()?
            .into_iter()
            .map(|level| (level.max_distance, level.has_part))
            .collect();
        lod.set_hysteresis(0.0)?;
        findings.lod_hysteresis = lod.hysteresis()?;
        for distance in [1.0_f32, 10.0, 20.0, 40.0, 1_000.0] {
            lod.reset_hysteresis()?;
            findings
                .lod_selection
                .push((distance, lod.select_index(distance)?));
        }
        // Hysteresis: having settled on a level, a distance that crosses the
        // next boundary by less than the margin must hold the old level, and
        // forgetting the old level must let it move.
        lod.set_hysteresis(3.0)?;
        lod.reset_hysteresis()?;
        findings
            .lod_sticky
            .push(("settled at 20", lod.select_index(20.0)?));
        findings
            .lod_sticky
            .push(("nudged to 26", lod.select_index(26.0)?));
        findings
            .lod_sticky
            .push(("pushed to 40", lod.select_index(40.0)?));
        lod.reset_hysteresis()?;
        findings
            .lod_sticky
            .push(("26 with no memory", lod.select_index(26.0)?));
        lod.set_hysteresis(0.0)?;
        lod.reset_hysteresis()?;

        lod.set_selection_mode(LodSelectionMode::ScreenSpaceError)?;
        findings.lod_mode_round_trip = lod.selection_mode().ok();
        lod.set_screen_space_parameters(1.0, std::f32::consts::FRAC_PI_2, 1_080.0)?;
        for distance in [1.0_f32, 10.0, 100.0] {
            findings
                .projected_radius
                .push((distance, lod.projected_radius_pixels(distance)?));
        }

        // --- debug drawing --------------------------------------------------
        let debug = DebugDraw::new(&device)?;
        findings.line_counts.push(("empty", debug.line_count()?));
        debug.add_line(
            Vector3::Zero,
            Vector3::from_x_and_y_and_z(1.0, 0.0, 0.0),
            Color::Red,
        )?;
        findings.line_counts.push(("one line", debug.line_count()?));
        debug.add_box(unit_box_at(0.0, 0.0, -5.0), Color::Lime)?;
        findings.line_counts.push(("plus a box", debug.line_count()?));
        debug.add_cross(Vector3::Zero, 1.0, Color::White)?;
        findings.line_counts.push(("plus a cross", debug.line_count()?));
        debug.add_sphere(Vector3::Zero, 1.0, Color::Blue, 8)?;
        findings.line_counts.push(("plus a sphere", debug.line_count()?));
        debug.add_bounding_sphere(
            BoundingSphere {
                Center: Vector3::Zero,
                Radius: 2.0,
            },
            Color::Yellow,
            8,
        )?;
        findings
            .line_counts
            .push(("plus a bounding sphere", debug.line_count()?));
        debug.add_frustum(&BoundingFrustum::new(view * projection), Color::Magenta)?;
        findings.line_counts.push(("plus a frustum", debug.line_count()?));

        let sun = DirectionalLight::canonical_defaults()?;
        debug.add_directional_light_gizmo(sun, Vector3::Zero, 4.0, Color::Orange)?;
        debug.add_point_light_gizmo(PointLight::canonical_defaults()?, Color::Cyan)?;
        debug.add_spot_light_gizmo(SpotLight::canonical_defaults()?, Color::Pink, 8)?;
        let cascades = CascadedShadowMap::new(&device, ShadowQuality::Low, 2)?;
        cascades.update(sun, view, projection)?;
        debug.add_cascade_gizmo(&cascades, Color::Gray)?;
        findings
            .line_counts
            .push(("plus every gizmo", debug.line_count()?));

        let depth_vertices = debug.vertices(true)?;
        let overlay_vertices = debug.vertices(false)?;
        findings.vertices_depth_tested = depth_vertices.len();
        findings.vertices_overlay = overlay_vertices.len();
        let queued = if depth_vertices.is_empty() {
            &overlay_vertices
        } else {
            &depth_vertices
        };
        findings.first_line = queued.first().map(|vertex| {
            (
                (vertex.Position.X, vertex.Position.Y, vertex.Position.Z),
                vertex.Color.PackedValue(),
            )
        });

        let before = debug.is_depth_tested()?;
        debug.set_depth_tested(!before)?;
        findings.depth_tested_round_trip = Some((before, debug.is_depth_tested()?));
        debug.set_depth_tested(before)?;

        debug.clear()?;
        findings.line_counts.push(("cleared", debug.line_count()?));

        *self.findings.lock().expect("findings") = findings;
        Ok(())
    }
}

#[test]
fn culling_lod_and_debug_drawing_count_exactly_what_they_produce() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(CullingFindings::default()));
    let game = CullingGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame with a culler, a LOD group and a debug drawer");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    // --- frustum culling ---------------------------------------------------
    assert_eq!(
        findings.frustum_matrix_matches,
        Some(true),
        "the culler tests against the camera it was given"
    );
    println!("box visibility: {:?}", findings.box_visibility);
    let visible: Vec<bool> = findings.box_visibility.iter().map(|(_, v)| *v).collect();
    assert_eq!(
        visible,
        vec![true, false, false, false],
        "only the box in front of the camera and inside the far plane is visible"
    );
    // The culled indices must be exactly the visible ones, in order.
    let expected: Vec<u64> = visible
        .iter()
        .enumerate()
        .filter(|(_, v)| **v)
        .map(|(index, _)| index as u64)
        .collect();
    assert_eq!(
        findings.culled_boxes, expected,
        "culling a list gives back the indices the per-box test agrees on"
    );
    assert_eq!(
        findings.culled_transforms, expected.len(),
        "culling transforms keeps as many as culling their boxes did"
    );
    assert_eq!(
        findings.culled_short_bounds,
        visible[..2].iter().filter(|v| **v).count() + 2,
        "a transform past the end of the bounds array is kept, not dropped"
    );
    // A sphere centred 60 units behind the camera becomes visible once it is
    // large enough to reach in front of the near plane, one unit ahead. Radius
    // is the only thing that changes between the three tests, and the answer
    // has to turn over between 50 and 70 rather than being constant.
    println!("sphere visibility: {:?}", findings.sphere_visibility);
    assert_eq!(
        findings
            .sphere_visibility
            .iter()
            .map(|(_, v)| *v)
            .collect::<Vec<bool>>(),
        vec![false, false, true],
        "a sphere behind the camera is culled until its radius reaches into view"
    );

    // --- levels of detail --------------------------------------------------
    assert_eq!(
        findings.lod_levels,
        vec![(10.0, false), (25.0, false), (60.0, false)],
        "the levels come back as they were added, each with no mesh part"
    );
    assert!(
        findings.lod_hysteresis.abs() < 1e-6,
        "hysteresis is off for the selection test: {}",
        findings.lod_hysteresis
    );
    println!("lod selection: {:?}", findings.lod_selection);
    // Each distance lands in the first level whose boundary is strictly above
    // it, and anything past the last boundary is dropped entirely rather than
    // falling back to the coarsest level.
    for (distance, index) in &findings.lod_selection {
        let expected = findings
            .lod_levels
            .iter()
            .position(|(boundary, _)| distance < boundary)
            .map_or(-1_i32, |position| position as i32);
        assert_eq!(
            *index, expected,
            "{distance} selects the first level strictly above it"
        );
    }
    // The boundary distance itself belongs to the next level, not to the level
    // it closes: a wrong comparison operator changes exactly this answer.
    assert_eq!(
        findings
            .lod_selection
            .iter()
            .find(|(distance, _)| (*distance - 10.0).abs() < 1e-6)
            .map(|(_, index)| *index),
        Some(1),
        "a distance sitting on a boundary belongs to the level above it"
    );
    println!("lod hysteresis: {:?}", findings.lod_sticky);
    let sticky: std::collections::HashMap<&str, i32> =
        findings.lod_sticky.iter().copied().collect();
    assert_eq!(sticky["settled at 20"], 1, "20 is in the middle level");
    assert_eq!(
        sticky["nudged to 26"], 1,
        "26 is one unit past the boundary, inside the three-unit margin, so the level holds"
    );
    assert_eq!(
        sticky["pushed to 40"], 2,
        "40 is well past the margin, so the level moves"
    );
    assert_eq!(
        sticky["26 with no memory"], 2,
        "and with the memory reset, 26 selects the level it really falls in"
    );
    assert_eq!(
        findings.lod_mode_round_trip,
        Some(LodSelectionMode::ScreenSpaceError),
        "the selection mode round-trips as an identity"
    );
    // Projected size falls with distance, which is the whole basis of the
    // screen-space rule -- and it falls by the documented law rather than by
    // some monotonic curve that merely looks right. With a unit radius, a
    // ninety-degree vertical field of view and a 1080-pixel viewport, the half
    // extent at distance d is 2d, so the answer must be 540/d exactly.
    println!("projected radius: {:?}", findings.projected_radius);
    for (distance, pixels) in &findings.projected_radius {
        let expected = 540.0 / distance;
        assert!(
            (pixels - expected).abs() < expected * 1e-4,
            "at {distance} units the projected radius is {pixels}, not {expected}"
        );
    }

    // --- debug drawing -----------------------------------------------------
    println!("line counts: {:?}", findings.line_counts);
    let counts: std::collections::HashMap<&str, i32> =
        findings.line_counts.iter().copied().collect();
    assert_eq!(counts["empty"], 0, "a fresh drawer has nothing queued");
    assert_eq!(counts["one line"], 1, "a line is one line");
    assert_eq!(counts["plus a box"], 1 + 12, "a box is its twelve edges");
    assert_eq!(counts["plus a cross"], 1 + 12 + 3, "a cross is three segments");
    assert_eq!(
        counts["plus a sphere"],
        1 + 12 + 3 + 8 * 3,
        "an eight-segment sphere is three eight-segment rings"
    );
    assert_eq!(
        counts["plus a bounding sphere"],
        1 + 12 + 3 + 8 * 3 + 8 * 3,
        "and a bounding sphere is drawn exactly the same way"
    );
    assert_eq!(
        counts["plus a frustum"],
        1 + 12 + 3 + 8 * 3 + 8 * 3 + 12,
        "a frustum is twelve edges like any other box"
    );
    assert!(
        counts["plus every gizmo"] > counts["plus a frustum"],
        "every gizmo adds lines: {:?}",
        findings.line_counts
    );
    assert_eq!(counts["cleared"], 0, "clearing leaves nothing queued");

    // Two vertices per line, and every line landed in the queue the drawer's
    // own depth-test flag names -- a drawer that filed them in the other queue
    // would draw the same lines with the wrong depth behaviour.
    let total_vertices = findings.vertices_depth_tested + findings.vertices_overlay;
    assert_eq!(
        total_vertices,
        counts["plus every gizmo"] as usize * 2,
        "every queued line is two vertices, across both queues"
    );
    let (depth_tested, _) = findings
        .depth_tested_round_trip
        .expect("the depth-test flag was read");
    let (used, empty) = if depth_tested {
        (findings.vertices_depth_tested, findings.vertices_overlay)
    } else {
        (findings.vertices_overlay, findings.vertices_depth_tested)
    };
    assert_eq!(used, total_vertices, "the lines went into the queue the flag selects");
    assert_eq!(empty, 0, "and the other queue stayed empty");
    let (position, color) = findings.first_line.expect("a first vertex");
    assert_eq!(
        position,
        (0.0, 0.0, 0.0),
        "the first vertex is where the first line started"
    );
    assert_eq!(
        color,
        Color::Red.PackedValue(),
        "and carries the colour that line was given"
    );

    let (before, after) = findings
        .depth_tested_round_trip
        .expect("the depth-test flag was toggled");
    assert_ne!(before, after, "the depth-test flag round-trips");

    println!(
        "culling: {} boxes visible of 4 | lod levels {:?} | debug lines {:?}",
        findings.culled_boxes.len(),
        findings.lod_levels,
        counts["plus every gizmo"],
    );
}

/// What the clustered-lighting run measured.
#[derive(Default)]
struct ClusteredFindings {
    engine_layer: i32,
    defaults: Option<ClusteredLight>,
    usable: Vec<(&'static str, bool, bool)>,
    set_counts: Vec<(&'static str, i32, bool)>,
    round_trip: Option<(ClusteredLight, ClusteredLight)>,
    lights_match_gets: bool,
    after_remove: Vec<f32>,
    bounds: Vec<((f32, f32, f32), f32, (f32, f32, f32), f32)>,
    bounds_match_per_index: bool,
    converted: Vec<(&'static str, ClusteredLightType, f32, f32)>,
    set_overflow: Option<String>,
    grid_shape: (i32, i32, i32, i32),
    cluster_indices: Vec<i32>,
    bad_coordinate: Option<String>,
    bad_grid_shape: Vec<(&'static str, bool)>,
    before_projection: Option<(bool, f32, Result<()>)>,
    bad_projection: Vec<(&'static str, bool)>,
    planes: (f32, f32),
    slice_distances: Vec<f32>,
    slice_lookup: Vec<(f32, i32)>,
    cluster_depth: Vec<(f32, f32)>,
    inverse_round_trip: f32,
    assignment_before: (i32, i32),
    assign_without_projection: Option<String>,
    assignment_after: (i32, i32, i32, i32),
    offsets: Vec<i32>,
    indices: Vec<i32>,
    runs_match_offsets: bool,
    clustered_light_reach: Vec<(&'static str, usize)>,
    after_clear: (i32, i32, Vec<i32>),
    adopted: Option<(i32, i32, Vec<i32>, Vec<i32>)>,
    bad_adoptions: Vec<(&'static str, bool)>,
}

struct ClusteredGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<ClusteredFindings>>,
}

impl GameStateAccess for ClusteredGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

fn clustered_light_at(x: f32, y: f32, z: f32, range: f32) -> Result<ClusteredLight> {
    let mut light = ClusteredLight::canonical_defaults()?;
    light.position = Vector3::from_x_and_y_and_z(x, y, z);
    light.range = range;
    Ok(light)
}

impl Game for ClusteredGame {
    #[allow(clippy::too_many_lines)]
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let mut findings = ClusteredFindings {
            engine_layer: version,
            ..ClusteredFindings::default()
        };

        // --- the light value ------------------------------------------------
        let defaults = ClusteredLight::canonical_defaults()?;
        findings.defaults = Some(defaults);

        // `is_usable` is documented as exactly the test `add` applies, so both
        // are recorded for every probe and compared against each other.
        let set = ClusteredLightSet::new(&device)?;
        let mut probe = |name: &'static str, light: ClusteredLight| -> Result<()> {
            let usable = light.is_usable()?;
            let accepted = set.add(light).is_ok();
            findings.usable.push((name, usable, accepted));
            set.clear()?;
            Ok(())
        };
        probe("the defaults", defaults)?;
        let mut negative_range = defaults;
        negative_range.range = -1.0;
        probe("a negative range", negative_range)?;
        let mut zero_range = defaults;
        zero_range.range = 0.0;
        probe("a zero range", zero_range)?;
        let mut negative_intensity = defaults;
        negative_intensity.intensity = -0.5;
        probe("a negative intensity", negative_intensity)?;
        let mut infinite_position = defaults;
        infinite_position.position = Vector3::from_x_and_y_and_z(f32::INFINITY, 0.0, 0.0);
        probe("an infinite position", infinite_position)?;
        let mut inverted_cone = defaults;
        inverted_cone.kind = ClusteredLightType::Spot;
        inverted_cone.inner_angle = 1.0;
        inverted_cone.outer_angle = 0.5;
        probe("a cone whose inner angle is wider", inverted_cone)?;
        let mut degenerate_spot = defaults;
        degenerate_spot.kind = ClusteredLightType::Spot;
        degenerate_spot.direction = Vector3::Zero;
        probe("a spot with no direction", degenerate_spot)?;
        drop(probe);

        // --- the set --------------------------------------------------------
        findings
            .set_counts
            .push(("fresh", set.count()?, set.is_empty()?));
        let first = clustered_light_at(1.0, 0.0, 0.0, 3.0)?;
        let second = clustered_light_at(2.0, 0.0, 0.0, 5.0)?;
        let third = clustered_light_at(3.0, 0.0, 0.0, 7.0)?;
        assert_eq!(set.add(first)?, 0);
        assert_eq!(set.add(second)?, 1);
        assert_eq!(set.add(third)?, 2);
        findings
            .set_counts
            .push(("three added", set.count()?, set.is_empty()?));
        findings.round_trip = Some((second, set.get(1)?));
        let listed = set.lights()?;
        findings.lights_match_gets = listed.len() == 3
            && (0..3).all(|index| set.get(index).ok().as_ref() == listed.get(index as usize));

        // The bounds are what an assignment sorts, so their rule matters: each
        // is the light's own position and range, not a default sphere.
        for index in 0..3 {
            let light = set.get(index)?;
            let sphere = set.bounds_at(index)?;
            findings.bounds.push((
                (light.position.X, light.position.Y, light.position.Z),
                light.range,
                (sphere.Center.X, sphere.Center.Y, sphere.Center.Z),
                sphere.Radius,
            ));
        }
        let all_bounds = set.bounds()?;
        findings.bounds_match_per_index = all_bounds.len() == 3
            && (0..3).all(|index| {
                set.bounds_at(index).ok().as_ref() == all_bounds.get(index as usize)
            });

        set.remove_at(0)?;
        findings.after_remove = set.lights()?.into_iter().map(|light| light.range).collect();
        set.replace_at(0, clustered_light_at(9.0, 0.0, 0.0, 11.0)?)?;
        findings
            .set_counts
            .push(("after replace", set.count()?, set.is_empty()?));
        findings.after_remove.push(set.get(0)?.range);
        set.clear()?;
        findings
            .set_counts
            .push(("cleared", set.count()?, set.is_empty()?));

        // A point light and a spot light converted into the set keep their kind.
        let mut point = PointLight::canonical_defaults()?;
        point.range = 13.0;
        set.add_point(point)?;
        let converted = set.get(0)?;
        findings.converted.push((
            "a point light",
            converted.kind,
            converted.range,
            converted.outer_angle,
        ));
        let mut spot = SpotLight::canonical_defaults()?;
        spot.range = 17.0;
        spot.outer_angle = 0.75;
        set.add_spot(spot)?;
        let converted = set.get(1)?;
        findings.converted.push((
            "a spot light",
            converted.kind,
            converted.range,
            converted.outer_angle,
        ));
        set.clear()?;

        // The documented ceiling is a refusal, not a silent drop.
        for _ in 0..ClusteredLight::SET_MAX {
            set.add(defaults)?;
        }
        findings.set_overflow = set.add(defaults).err().map(|error| error.to_string());
        set.clear()?;

        // --- the grid -------------------------------------------------------
        let grid = ClusteredLightGrid::new(&device, 4, 3, 8)?;
        findings.grid_shape = (
            grid.tiles_x()?,
            grid.tiles_y()?,
            grid.slice_count()?,
            grid.cluster_count()?,
        );
        for slice in 0..8 {
            for y in 0..3 {
                for x in 0..4 {
                    findings.cluster_indices.push(grid.cluster_index(x, y, slice)?);
                }
            }
        }
        findings.bad_coordinate = grid.cluster_index(4, 0, 0).err().map(|e| e.to_string());
        for (name, tiles_x, tiles_y, slices) in [
            ("zero tiles", 0, 4, 4),
            ("too many tiles", ClusteredLightGrid::MAX_TILES_PER_AXIS + 1, 4, 4),
            ("too many slices", 4, 4, ClusteredLightGrid::MAX_SLICE_COUNT + 1),
        ] {
            findings.bad_grid_shape.push((
                name,
                ClusteredLightGrid::new(&device, tiles_x, tiles_y, slices).is_err(),
            ));
        }

        findings.before_projection = Some((
            grid.has_projection()?,
            grid.slice_distance(0)?,
            grid.cluster_bounds(0, 0, 0).map(|_| ()),
        ));

        let projection =
            Matrix::CreatePerspectiveFieldOfView(std::f32::consts::FRAC_PI_2, 1.0, 1.0, 100.0);
        for (name, near, far) in [
            ("a zero near plane", 0.0_f32, 100.0_f32),
            ("a negative near plane", -1.0, 100.0),
            ("an inverted pair", 100.0, 1.0),
        ] {
            findings
                .bad_projection
                .push((name, grid.set_projection(projection, near, far).is_err()));
        }
        grid.set_projection(projection, 1.0, 100.0)?;
        findings.planes = (grid.near_plane()?, grid.far_plane()?);
        for slice in 0..=8 {
            findings.slice_distances.push(grid.slice_distance(slice)?);
        }
        for distance in [-5.0_f32, 0.5, 1.0, 10.0, 99.0, 100.0, 1.0e6] {
            findings
                .slice_lookup
                .push((distance, grid.slice_for_view_distance(distance)?));
        }
        for slice in 0..8 {
            let bounds = grid.cluster_bounds(0, 0, slice)?;
            findings
                .cluster_depth
                .push((bounds.Min.Z.abs().min(bounds.Max.Z.abs()), bounds.Min.Z.abs().max(bounds.Max.Z.abs())));
        }
        let identity = projection * grid.inverse_projection()?;
        findings.inverse_round_trip = [
            identity.M11 - 1.0,
            identity.M12,
            identity.M21,
            identity.M22 - 1.0,
            identity.M33 - 1.0,
            identity.M44 - 1.0,
        ]
        .into_iter()
        .fold(0.0_f32, |worst, value| worst.max(value.abs()));

        // --- the assignment --------------------------------------------------
        let assignment = ClusteredLightAssignment::new(&device)?;
        findings.assignment_before = (assignment.light_count()?, assignment.cluster_count()?);

        let bare_grid = ClusteredLightGrid::new(&device, 2, 2, 2)?;
        findings.assign_without_projection = assignment
            .assign(&bare_grid, Matrix::Identity, &[])
            .err()
            .map(|error| error.to_string());
        bare_grid.release()?;

        // One light dead ahead inside the frustum, one behind the camera and
        // far outside it. The camera looks down -Z from the origin.
        set.clear()?;
        set.add(clustered_light_at(0.0, 0.0, -20.0, 6.0)?)?;
        set.add(clustered_light_at(0.0, 0.0, 5000.0, 1.0)?)?;
        let view = Matrix::CreateLookAt(
            Vector3::Zero,
            Vector3::from_x_and_y_and_z(0.0, 0.0, -1.0),
            Vector3::Up,
        );
        assignment.assign(&grid, view, &set.bounds()?)?;
        findings.assignment_after = (
            assignment.light_count()?,
            assignment.cluster_count()?,
            assignment.total_reference_count()?,
            assignment.max_lights_per_cluster()?,
        );
        findings.offsets = assignment.offsets()?;
        findings.indices = assignment.indices()?;
        let cluster_count = assignment.cluster_count()?;
        let mut runs_match = findings.offsets.len() == cluster_count as usize + 1;
        let mut reach = [0_usize; 2];
        for cluster in 0..cluster_count {
            let run = assignment.lights_in_cluster(cluster)?;
            let start = findings.offsets[cluster as usize] as usize;
            let end = findings.offsets[cluster as usize + 1] as usize;
            if findings.indices.get(start..end) != Some(run.as_slice()) {
                runs_match = false;
            }
            for light in run {
                if let Some(slot) = reach.get_mut(light as usize) {
                    *slot += 1;
                }
            }
        }
        findings.runs_match_offsets = runs_match;
        findings
            .clustered_light_reach
            .push(("the light inside the frustum", reach[0]));
        findings
            .clustered_light_reach
            .push(("the light behind the camera", reach[1]));

        assignment.clear()?;
        findings.after_clear = (
            assignment.total_reference_count()?,
            assignment.cluster_count()?,
            assignment.offsets()?,
        );

        // --- adoption --------------------------------------------------------
        let offsets = vec![0, 2, 2, 3];
        let indices = vec![1, 0, 2];
        assignment.adopt(3, &offsets, &indices)?;
        findings.adopted = Some((
            assignment.light_count()?,
            assignment.cluster_count()?,
            assignment.offsets()?,
            assignment.indices()?,
        ));
        for (name, light_count, offsets, indices) in [
            ("offsets that do not begin at zero", 3, vec![1, 2, 3], vec![1, 0, 2]),
            ("offsets that go backwards", 3, vec![0, 2, 1, 3], vec![1, 0, 2]),
            ("offsets that end short", 3, vec![0, 1, 2], vec![1, 0, 2]),
            ("an index past the light count", 2, vec![0, 2, 2, 3], vec![1, 0, 2]),
        ] {
            findings.bad_adoptions.push((
                name,
                assignment.adopt(light_count, &offsets, &indices).is_err(),
            ));
        }

        *self.findings.lock().expect("findings") = findings;
        Ok(())
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn clustered_lighting_sorts_the_lights_it_is_given_into_the_grid_it_is_given() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(ClusteredFindings::default()));
    let game = ClusteredGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame with a clustered light set, grid and assignment");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    // --- the light value ---------------------------------------------------
    let defaults = findings.defaults.expect("CNA's own defaults");
    println!("clustered light defaults: {defaults:?}");
    assert_eq!(defaults.kind, ClusteredLightType::Point);
    assert!(!defaults.casts_shadows);
    assert_eq!(defaults.position, Vector3::Zero);
    assert_eq!(
        defaults.direction,
        Vector3::from_x_and_y_and_z(0.0, -1.0, 0.0),
        "the default spot direction points straight down"
    );
    assert_eq!(defaults.color, Vector3::from_x_and_y_and_z(1.0, 1.0, 1.0));
    assert!((defaults.intensity - 1.0).abs() < 1e-6);
    assert!((defaults.range - 20.0).abs() < 1e-6);
    assert!((defaults.inner_angle - 0.35).abs() < 1e-6);
    assert!((defaults.outer_angle - 0.5).abs() < 1e-6);

    println!("usability: {:?}", findings.usable);
    for (name, usable, accepted) in &findings.usable {
        assert_eq!(
            usable, accepted,
            "{name}: `is_usable` must answer exactly what `add` does"
        );
    }
    assert_eq!(
        findings.usable.iter().map(|(_, u, _)| *u).collect::<Vec<bool>>(),
        vec![true, false, false, false, false, false, false],
        "only the defaults are usable: {:?}",
        findings.usable
    );

    // --- the set -----------------------------------------------------------
    println!("set counts: {:?}", findings.set_counts);
    assert_eq!(findings.set_counts[0], ("fresh", 0, true));
    assert_eq!(findings.set_counts[1], ("three added", 3, false));
    assert_eq!(findings.set_counts[2], ("after replace", 2, false));
    assert_eq!(findings.set_counts[3], ("cleared", 0, true));

    let (written, read) = findings.round_trip.expect("a light read back");
    assert_eq!(written, read, "a light comes back as it went in");
    assert!(
        findings.lights_match_gets,
        "copying every light agrees with reading them one at a time"
    );

    // The bounds are the light's own sphere, which is what makes the
    // assignment's answer depend on where the lights actually are.
    println!("bounds: {:?}", findings.bounds);
    for (position, range, center, radius) in &findings.bounds {
        assert_eq!(center, position, "a light's bounds are centred on it");
        assert!(
            (radius - range).abs() < 1e-6,
            "and reach as far as its range: {radius} against {range}"
        );
    }
    assert!(
        findings.bounds_match_per_index,
        "copying every sphere agrees with reading them one at a time"
    );

    // Removing index 0 shifts the rest down rather than leaving a hole.
    assert_eq!(
        findings.after_remove,
        vec![5.0, 7.0, 11.0],
        "removal shifts the tail down, and the replacement lands where it was asked to"
    );

    println!("converted: {:?}", findings.converted);
    let (_, kind, range, _) = findings.converted[0];
    assert_eq!(kind, ClusteredLightType::Point, "a point light stays a point");
    assert!((range - 13.0).abs() < 1e-6, "and keeps its range");
    let (_, kind, range, outer) = findings.converted[1];
    assert_eq!(kind, ClusteredLightType::Spot, "a spot light stays a spot");
    assert!((range - 17.0).abs() < 1e-6, "and keeps its range");
    assert!((outer - 0.75).abs() < 1e-6, "and its cone");

    let overflow = findings
        .set_overflow
        .as_deref()
        .expect("the set refuses light 257");
    println!("set overflow: {overflow}");

    // --- the grid ----------------------------------------------------------
    assert_eq!(
        findings.grid_shape,
        (4, 3, 8, 96),
        "the cluster count is the product of the three dimensions"
    );
    // Every coordinate maps to its own index, and together they cover the
    // range exactly: an index function that collided or left gaps would size
    // the shader's light list wrongly.
    let mut sorted = findings.cluster_indices.clone();
    sorted.sort_unstable();
    assert_eq!(
        sorted,
        (0..96).collect::<Vec<i32>>(),
        "the cluster index is a bijection onto the cluster range"
    );
    assert!(
        findings.bad_coordinate.is_some(),
        "a coordinate outside the grid is refused"
    );
    for (name, refused) in &findings.bad_grid_shape {
        assert!(refused, "{name} is refused rather than clamped");
    }

    let (has_projection, distance, bounds) = findings
        .before_projection
        .as_ref()
        .expect("the grid was asked before it had a shape");
    assert!(!has_projection, "a fresh grid has no projection");
    assert_eq!(*distance, 0.0, "and no slice distances");
    assert!(bounds.is_err(), "and refuses to bound a cluster");

    for (name, refused) in &findings.bad_projection {
        assert!(refused, "{name} is refused");
    }
    assert_eq!(findings.planes, (1.0, 100.0), "the planes round-trip");

    // The slice boundaries are logarithmic in the plane ratio: there is one
    // more boundary than slice, the first is the near plane and the last the
    // far plane. A linear split would pass a monotonicity check and fail this.
    println!("slice distances: {:?}", findings.slice_distances);
    assert_eq!(findings.slice_distances.len(), 9, "one more boundary than slice");
    for (slice, distance) in findings.slice_distances.iter().enumerate() {
        let expected = 1.0_f32 * (100.0_f32 / 1.0).powf(slice as f32 / 8.0);
        assert!(
            (distance - expected).abs() < expected * 1e-3,
            "boundary {slice} is at {distance}, not {expected}"
        );
    }

    // Placing a distance is clamped into the grid at both ends.
    println!("slice lookup: {:?}", findings.slice_lookup);
    for (distance, slice) in &findings.slice_lookup {
        assert!(
            (0..8).contains(slice),
            "{distance} was placed outside the grid, at slice {slice}"
        );
        let expected = findings
            .slice_distances
            .windows(2)
            .position(|pair| *distance >= pair[0] && *distance < pair[1])
            .map_or_else(
                || if *distance < 1.0 { 0 } else { 7 },
                |position| position,
            );
        assert_eq!(
            *slice as usize, expected,
            "{distance} belongs to slice {expected}, not {slice}"
        );
    }

    // A cluster's depth extent is the pair of slice boundaries around it.
    println!("cluster depth: {:?}", findings.cluster_depth);
    for (slice, (near, far)) in findings.cluster_depth.iter().enumerate() {
        assert!(
            (near - findings.slice_distances[slice]).abs() < findings.slice_distances[slice] * 1e-3,
            "cluster {slice} starts at {near}, not {}",
            findings.slice_distances[slice]
        );
        assert!(
            (far - findings.slice_distances[slice + 1]).abs()
                < findings.slice_distances[slice + 1] * 1e-3,
            "cluster {slice} ends at {far}, not {}",
            findings.slice_distances[slice + 1]
        );
    }

    assert!(
        findings.inverse_round_trip < 1e-4,
        "the stored inverse really inverts the projection: worst term {}",
        findings.inverse_round_trip
    );

    // --- the assignment ----------------------------------------------------
    assert_eq!(
        findings.assignment_before,
        (0, 0),
        "a fresh assignment describes nothing"
    );
    assert!(
        findings.assign_without_projection.is_some(),
        "a grid with no shape cannot be sorted into"
    );

    let (lights, clusters, references, max_per_cluster) = findings.assignment_after;
    println!(
        "assignment: {lights} lights, {clusters} clusters, {references} references, at most {max_per_cluster} per cluster"
    );
    assert_eq!(lights, 2, "the assignment describes the two lights it sorted");
    assert_eq!(clusters, 96, "over the grid's own clusters");
    assert_eq!(
        findings.offsets.len(),
        clusters as usize + 1,
        "one more offset than cluster"
    );
    assert_eq!(findings.offsets[0], 0, "the offsets begin at zero");
    assert!(
        findings.offsets.windows(2).all(|pair| pair[0] <= pair[1]),
        "and never go backwards: {:?}",
        findings.offsets
    );
    assert_eq!(
        *findings.offsets.last().expect("a last offset") as usize,
        findings.indices.len(),
        "and end at the index count"
    );
    assert_eq!(
        references as usize,
        findings.indices.len(),
        "the total reference count is the length of the index array"
    );
    assert!(
        findings.runs_match_offsets,
        "each cluster's run is exactly the slice its offsets name"
    );
    let longest_run = findings
        .offsets
        .windows(2)
        .map(|pair| (pair[1] - pair[0]) as i32)
        .max()
        .unwrap_or(0);
    assert_eq!(
        max_per_cluster, longest_run,
        "the reported maximum is the longest run there actually is"
    );

    // The whole point: a light inside the frustum reaches clusters and one
    // behind the camera reaches none.
    println!("light reach: {:?}", findings.clustered_light_reach);
    assert!(
        findings.clustered_light_reach[0].1 > 0,
        "a light twenty units ahead of the camera lands in at least one cluster"
    );
    assert_eq!(
        findings.clustered_light_reach[1].1, 0,
        "and one five thousand units behind it lands in none"
    );

    // Clearing forgets the clusters as well as the references, and keeps the
    // "one more offset than cluster" invariant while doing it: a cleared
    // assignment is an empty grid, not a broken one.
    println!("after clear: {:?}", findings.after_clear);
    assert_eq!(
        findings.after_clear,
        (0, 0, vec![0]),
        "clearing leaves no references, no clusters, and the single zero offset that empties them"
    );

    // --- adoption ----------------------------------------------------------
    let (lights, clusters, offsets, indices) = findings.adopted.clone().expect("an adopted assignment");
    assert_eq!(lights, 3, "adoption takes the light count it is given");
    assert_eq!(clusters, 3, "and one fewer clusters than offsets");
    assert_eq!(offsets, vec![0, 2, 2, 3], "the offsets come back unchanged");
    assert_eq!(indices, vec![1, 0, 2], "and so do the indices, in order");

    println!("bad adoptions: {:?}", findings.bad_adoptions);
    for (name, refused) in &findings.bad_adoptions {
        assert!(refused, "{name} is refused");
    }
}

/// Where the shadow-budget probes put their five casters, nearest first.
const SHADOW_DISTANCES: [f32; 5] = [2.0, 4.0, 6.0, 8.0, 10.0];

/// What the shadow-budget, upload-buffer and compute-assignment run measured.
#[derive(Default)]
struct ClusteredGpuFindings {
    engine_layer: i32,
    budget_round_trip: (i32, i32),
    hysteresis_round_trip: (f32, f32),
    fresh_policy: (usize, i32, i32),
    scores: Vec<f32>,
    selection: Vec<i32>,
    selection_counts: (i32, i32),
    is_selected_agrees: bool,
    generous_budget: (usize, i32, i32),
    after_policy_reset: (usize, i32, i32),
    sticky_selection: Vec<(&'static str, Vec<i32>)>,
    glsl: String,
    fresh_buffer: (bool, i32, i32, i32),
    bind_before_upload: Option<String>,
    after_upload: (bool, i32, i32, i32),
    expected_upload: (i32, i32, i32),
    bind_after_upload: Option<String>,
    mismatched_grid: Option<String>,
    mismatched_lights: Option<String>,
    bad_stride: bool,
    stride_round_trip: i32,
    compute_supported: bool,
    compute_reason: String,
    compute_offsets: Vec<i32>,
    compute_indices: Vec<i32>,
    cpu_offsets: Vec<i32>,
    cpu_indices: Vec<i32>,
    used_compute: bool,
    overflow: Vec<(&'static str, bool)>,
}

struct ClusteredGpuGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<ClusteredGpuFindings>>,
}

impl GameStateAccess for ClusteredGpuGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for ClusteredGpuGame {
    #[allow(clippy::too_many_lines)]
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let mut findings = ClusteredGpuFindings {
            engine_layer: version,
            ..ClusteredGpuFindings::default()
        };
        let (view, projection) = culling_camera();
        let camera = Vector3::Zero;

        // --- the shadow budget ----------------------------------------------
        let policy = ClusteredShadowPolicy::new(&device, 2)?;
        let first_budget = policy.budget()?;
        policy.set_budget(5)?;
        findings.budget_round_trip = (first_budget, policy.budget()?);
        let first_hysteresis = policy.hysteresis()?;
        policy.set_hysteresis(2.5)?;
        findings.hysteresis_round_trip = (first_hysteresis, policy.hysteresis()?);
        policy.set_hysteresis(first_hysteresis)?;
        policy.set_budget(2)?;
        findings.fresh_policy = (
            policy.selected()?.len(),
            policy.request_count()?,
            policy.refused_count()?,
        );

        // Five casters at increasing distances, plus two that do not ask.
        // Every one is inside the frustum *and* inside its own range: CNA's
        // falloff is zero at or beyond the range, so a light further away than
        // it reaches scores nothing and never becomes a candidate at all.
        let lights = ClusteredLightSet::new(&device)?;
        for distance in SHADOW_DISTANCES {
            let mut light = clustered_light_at(0.0, 0.0, -distance, 20.0)?;
            light.casts_shadows = true;
            lights.add(light)?;
        }
        for step in 0..2 {
            lights.add(clustered_light_at(0.0, 0.0, -3.0 - 2.0 * step as f32, 20.0)?)?;
        }
        policy.select(&lights, view, projection, camera)?;
        findings.scores = (0..lights.count()?)
            .map(|index| policy.score(index))
            .collect::<Result<Vec<f32>>>()?;
        findings.selection = policy.selected()?;
        findings.selection_counts = (policy.request_count()?, policy.refused_count()?);
        findings.is_selected_agrees = (0..lights.count()?)
            .map(|index| {
                Ok(policy.is_selected(index)? == findings.selection.contains(&index))
            })
            .collect::<Result<Vec<bool>>>()?
            .into_iter()
            .all(|agrees| agrees);

        policy.reset()?;
        policy.set_budget(50)?;
        policy.select(&lights, view, projection, camera)?;
        findings.generous_budget = (
            policy.selected()?.len(),
            policy.request_count()?,
            policy.refused_count()?,
        );

        policy.reset()?;
        findings.after_policy_reset = (
            policy.selected()?.len(),
            policy.request_count()?,
            policy.refused_count()?,
        );

        // Hysteresis: with a margin nothing can beat, the incumbent survives a
        // scene that has turned around underneath it; forgetting the incumbent
        // lets the new best win.
        policy.set_budget(1)?;
        policy.set_hysteresis(1.0e6)?;
        policy.select(&lights, view, projection, camera)?;
        findings
            .sticky_selection
            .push(("settled", policy.selected()?));
        for (index, distance) in SHADOW_DISTANCES.iter().rev().enumerate() {
            let mut moved = lights.get(index as i32)?;
            moved.position = Vector3::from_x_and_y_and_z(0.0, 0.0, -distance);
            lights.replace_at(index as i32, moved)?;
        }
        policy.select(&lights, view, projection, camera)?;
        findings
            .sticky_selection
            .push(("scene reversed", policy.selected()?));
        policy.reset()?;
        policy.select(&lights, view, projection, camera)?;
        findings
            .sticky_selection
            .push(("memory reset", policy.selected()?));
        policy.set_hysteresis(ClusteredShadowPolicy::DEFAULT_HYSTERESIS)?;

        // --- the upload buffer ------------------------------------------------
        findings.glsl = ClusteredLightBuffer::light_lookup_glsl()?;
        let buffer = ClusteredLightBuffer::new(&device)?;
        findings.fresh_buffer = (
            buffer.is_uploaded()?,
            buffer.light_count()?,
            buffer.cluster_count()?,
            buffer.reference_count()?,
        );

        let grid = ClusteredLightGrid::new(&device, 4, 3, 8)?;
        grid.set_projection(projection, 1.0, 100.0)?;
        let assignment = ClusteredLightAssignment::new(&device)?;
        assignment.assign(&grid, view, &lights.bounds()?)?;

        // A shadow map's caster effect is the only effect reachable without
        // content; binding into it is what the refusal-before-upload case needs.
        let shadow = ShadowMap::new(&device, ShadowQuality::Low)?;
        if let Some(caster) = shadow.caster_effect()? {
            findings.bind_before_upload =
                buffer.bind(caster.effect(), 0).err().map(|e| e.to_string());
        }

        buffer.upload(&lights, &grid, &assignment)?;
        findings.after_upload = (
            buffer.is_uploaded()?,
            buffer.light_count()?,
            buffer.cluster_count()?,
            buffer.reference_count()?,
        );
        findings.expected_upload = (
            lights.count()?,
            grid.cluster_count()?,
            assignment.total_reference_count()?,
        );
        if let Some(caster) = shadow.caster_effect()? {
            findings.bind_after_upload =
                buffer.bind(caster.effect(), 0).err().map(|e| e.to_string());
        }

        // A trio that disagrees is refused rather than uploaded.
        let other_grid = ClusteredLightGrid::new(&device, 2, 2, 2)?;
        other_grid.set_projection(projection, 1.0, 100.0)?;
        findings.mismatched_grid = buffer
            .upload(&lights, &other_grid, &assignment)
            .err()
            .map(|e| e.to_string());
        let wider = ClusteredLightAssignment::new(&device)?;
        wider.adopt(lights.count()? + 5, &vec![0; grid.cluster_count()? as usize + 1], &[])?;
        findings.mismatched_lights = buffer
            .upload(&lights, &grid, &wider)
            .err()
            .map(|e| e.to_string());
        other_grid.release()?;
        wider.release()?;

        // --- the compute assignment -------------------------------------------
        findings.bad_stride = ClusteredLightCompute::new(&device, 0).is_err();
        let compute = ClusteredLightCompute::new(&device, 32)?;
        findings.stride_round_trip = compute.stride()?;
        findings.compute_supported = compute.is_supported()?;
        findings.compute_reason = compute.unsupported_reason()?;

        let computed = ClusteredLightAssignment::new(&device)?;
        compute.assign(&grid, view, &lights.bounds()?, &computed)?;
        findings.used_compute = compute.used_compute()?;
        findings.compute_offsets = computed.offsets()?;
        findings.compute_indices = computed.indices()?;
        findings.cpu_offsets = assignment.offsets()?;
        findings.cpu_indices = assignment.indices()?;

        findings
            .overflow
            .push(("a stride of thirty-two", compute.has_overflowed()?));
        let narrow = ClusteredLightCompute::new(&device, 1)?;
        narrow.assign(&grid, view, &lights.bounds()?, &computed)?;
        findings
            .overflow
            .push(("a stride of one", narrow.has_overflowed()?));
        narrow.release()?;

        *self.findings.lock().expect("findings") = findings;
        Ok(())
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn the_shadow_budget_the_upload_buffer_and_the_compute_sort_agree_with_the_cpu() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(ClusteredGpuFindings::default()));
    let game = ClusteredGpuGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame with a shadow policy, a light buffer and a compute sort");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    // --- the shadow budget -------------------------------------------------
    assert_eq!(
        findings.budget_round_trip,
        (2, 5),
        "the budget is what it was created with, and then what it was set to"
    );
    println!("hysteresis: {:?}", findings.hysteresis_round_trip);
    assert!(
        (findings.hysteresis_round_trip.0 - ClusteredShadowPolicy::DEFAULT_HYSTERESIS).abs() < 1e-6,
        "a fresh policy carries CNA's own default margin"
    );
    assert!(
        (findings.hysteresis_round_trip.1 - 2.5).abs() < 1e-6,
        "and the margin round-trips"
    );
    assert_eq!(
        findings.fresh_policy,
        (0, 0, 0),
        "a policy that has not scored anything admits, requests and refuses nothing"
    );

    println!("scores: {:?}", findings.scores);
    println!("selection: {:?}", findings.selection);
    assert_eq!(
        findings.selection_counts,
        (5, 3),
        "five lights asked to cast, and with a budget of two, three were refused"
    );
    assert_eq!(
        findings.selection.len(),
        2,
        "the budget is a ceiling on the selection, not a suggestion"
    );
    assert!(
        findings.is_selected_agrees,
        "asking about one light agrees with the list of all of them"
    );
    // The two admitted are the two highest-scoring: a policy that admitted the
    // first two it saw, or the last two, would pass every count above.
    let mut ranked: Vec<(usize, f32)> = findings.scores.iter().copied().enumerate().collect();
    ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
    let mut expected: Vec<i32> = ranked[..2].iter().map(|(index, _)| *index as i32).collect();
    expected.sort_unstable();
    let mut admitted = findings.selection.clone();
    admitted.sort_unstable();
    assert_eq!(
        admitted, expected,
        "the admitted lights are the highest-scoring ones: scores {:?}",
        findings.scores
    );
    // The two lights that never asked score nothing and are never admitted.
    assert!(
        findings.scores[5..].iter().all(|score| *score == 0.0),
        "a light that does not cast shadows is not scored: {:?}",
        findings.scores
    );

    assert_eq!(
        findings.generous_budget,
        (5, 5, 0),
        "a budget larger than the demand admits every caster and refuses none"
    );
    assert_eq!(
        findings.after_policy_reset,
        (0, 0, 0),
        "resetting forgets the selection, the requests and the refusals"
    );

    println!("sticky selection: {:?}", findings.sticky_selection);
    let settled = &findings.sticky_selection[0].1;
    let reversed = &findings.sticky_selection[1].1;
    let after_reset = &findings.sticky_selection[2].1;
    assert_eq!(settled.len(), 1, "a budget of one admits one light");
    assert_eq!(
        reversed, settled,
        "a margin nothing can beat keeps the incumbent even when the scene turns around"
    );
    assert_ne!(
        after_reset, settled,
        "and forgetting the incumbent lets the new best win"
    );

    // --- the upload buffer -------------------------------------------------
    println!(
        "light lookup GLSL: {} bytes, first line {:?}",
        findings.glsl.len(),
        findings.glsl.lines().next()
    );
    assert!(
        !findings.glsl.is_empty(),
        "the shader-side lookup is published, not left to the caller to guess"
    );
    assert!(
        findings.glsl.contains("cluster") || findings.glsl.contains("Cluster"),
        "and it is the cluster lookup: {:?}",
        findings.glsl.lines().next()
    );

    assert_eq!(
        findings.fresh_buffer,
        (false, 0, 0, 0),
        "a buffer that has uploaded nothing says so and counts nothing"
    );
    assert!(
        findings.bind_before_upload.is_some(),
        "binding a buffer that holds no light list is refused"
    );

    let (uploaded, lights, clusters, references) = findings.after_upload;
    assert!(uploaded, "the upload succeeded");
    assert_eq!(
        (lights, clusters, references),
        findings.expected_upload,
        "and carried exactly the set, the grid and the assignment it was given"
    );
    println!("bind after upload: {:?}", findings.bind_after_upload);

    assert!(
        findings.mismatched_grid.is_some(),
        "a grid whose cluster count the assignment does not describe is refused"
    );
    assert!(
        findings.mismatched_lights.is_some(),
        "and so is an assignment naming more lights than the set holds"
    );

    // --- the compute assignment --------------------------------------------
    assert!(findings.bad_stride, "a non-positive stride is refused");
    assert_eq!(findings.stride_round_trip, 32, "the stride round-trips");
    println!(
        "compute supported: {} reason: {:?}",
        findings.compute_supported, findings.compute_reason
    );
    assert_eq!(
        findings.compute_reason.is_empty(),
        findings.compute_supported,
        "the reason is empty exactly when the program compiled"
    );
    assert_eq!(
        findings.used_compute, findings.compute_supported,
        "the GPU path ran exactly when it was available"
    );

    // The whole promise of the fallback: the same assignment either way.
    assert_eq!(
        findings.compute_offsets, findings.cpu_offsets,
        "the compute sort produces the CPU sort's cluster offsets"
    );
    let mut computed = findings.compute_indices.clone();
    let mut expected = findings.cpu_indices.clone();
    computed.sort_unstable();
    expected.sort_unstable();
    assert_eq!(
        computed, expected,
        "and the same light references: {:?} against {:?}",
        findings.compute_indices, findings.cpu_indices
    );

    println!("overflow: {:?}", findings.overflow);
    assert!(
        !findings.overflow[0].1,
        "thirty-two lights per cluster is room to spare for seven lights"
    );
    assert!(
        findings.overflow[1].1,
        "one light per cluster is not, and the flag says so rather than the call failing"
    );
}

/// What the clustered forward effect run measured.
#[derive(Default)]
struct ForwardFindings {
    engine_layer: i32,
    supported: bool,
    effect_present: bool,
    release_while_borrowed: Option<String>,
    release_after_borrow: Option<String>,
    base_color: Vec<((f32, f32, f32), (f32, f32, f32))>,
    metallic: Vec<(f32, f32)>,
    roughness: Vec<(f32, f32)>,
    ior: Vec<(f32, f32)>,
    ambient: Vec<((f32, f32, f32), (f32, f32, f32))>,
    bound_extras: (bool, bool),
    cleared_extras: (bool, bool),
    begin_before_upload: Option<String>,
    begin_after_upload: Option<String>,
    frame_before: bool,
    frame_after: Option<(i32, i32)>,
    frame_cleared: bool,
    attenuation: Vec<((f32, f32), (f32, f32, f32))>,
    contributions: Vec<(&'static str, (f32, f32, f32))>,
}

struct ForwardGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<ForwardFindings>>,
}

impl GameStateAccess for ForwardGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

fn triple(value: Vector3) -> (f32, f32, f32) {
    (value.X, value.Y, value.Z)
}

impl Game for ForwardGame {
    #[allow(clippy::too_many_lines)]
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let mut findings = ForwardFindings {
            engine_layer: version,
            ..ForwardFindings::default()
        };
        let (view, projection) = culling_camera();

        let mut effect = ClusteredForwardEffect::new(&device)?;
        findings.supported = effect.is_supported()?;
        findings.effect_present = effect.effect()?.is_some();

        // The shader effect is a counted borrow: releasing the owner while one
        // is outstanding is refused, and allowed once it is dropped.
        {
            let borrowed = effect.effect()?;
            findings.release_while_borrowed = effect.release().err().map(|e| e.to_string());
            drop(borrowed);
        }

        // --- the material terms ---------------------------------------------
        for probe in [
            Vector3::from_x_and_y_and_z(0.25, 0.5, 0.75),
            Vector3::from_x_and_y_and_z(2.0, -1.0, 0.5),
        ] {
            effect.set_base_color(probe)?;
            findings
                .base_color
                .push((triple(probe), triple(effect.base_color()?)));
        }
        for probe in [0.25_f32, -1.0, 5.0] {
            effect.set_metallic(probe)?;
            findings.metallic.push((probe, effect.metallic()?));
        }
        for probe in [0.5_f32, 0.0, -1.0, 5.0] {
            effect.set_roughness(probe)?;
            findings.roughness.push((probe, effect.roughness()?));
        }
        for probe in [1.5_f32, 2.4] {
            effect.set_ior(probe)?;
            findings.ior.push((probe, effect.ior()?));
        }
        for probe in [
            Vector3::from_x_and_y_and_z(0.1, 0.2, 0.3),
            Vector3::from_x_and_y_and_z(-1.0, 0.5, 2.0),
        ] {
            effect.set_ambient(probe)?;
            findings
                .ambient
                .push((triple(probe), triple(effect.ambient()?)));
        }

        findings.bound_extras = (effect.has_area_light()?, effect.has_light_probe()?);
        effect.clear_area_light()?;
        effect.clear_light_probe()?;
        findings.cleared_extras = (effect.has_area_light()?, effect.has_light_probe()?);

        // --- beginning a frame ------------------------------------------------
        let lights = ClusteredLightSet::new(&device)?;
        let mut lamp = clustered_light_at(0.0, 0.0, -10.0, 30.0)?;
        lamp.intensity = 2.0;
        lights.add(lamp)?;
        let grid = ClusteredLightGrid::new(&device, 4, 3, 8)?;
        grid.set_projection(projection, 1.0, 100.0)?;
        let assignment = ClusteredLightAssignment::new(&device)?;
        assignment.assign(&grid, view, &lights.bounds()?)?;
        let buffer = ClusteredLightBuffer::new(&device)?;

        findings.begin_before_upload = effect
            .begin(Matrix::Identity, view, projection, Vector3::Zero, &buffer)
            .err()
            .map(|e| e.to_string());
        buffer.upload(&lights, &grid, &assignment)?;
        findings.begin_after_upload = effect
            .begin(Matrix::Identity, view, projection, Vector3::Zero, &buffer)
            .err()
            .map(|e| e.to_string());

        // --- the opaque frame --------------------------------------------------
        findings.frame_before = effect.has_opaque_frame()?;
        let frame = Texture2D::new(&device, 40, 24)?;
        effect.set_opaque_frame(Some(frame))?;
        findings.frame_after = effect
            .opaque_frame()?
            .map(|view| (view.texture().Width(), view.texture().Height()));
        effect.set_opaque_frame(None)?;
        findings.frame_cleared = effect.has_opaque_frame()?;

        // --- the pure functions -------------------------------------------------
        let half = Vector3::from_x_and_y_and_z(0.5, 0.5, 0.5);
        for (distance, thickness) in [(1.0_f32, 0.0_f32), (1.0, 1.0), (1.0, 2.0), (2.0, 1.0)] {
            findings.attenuation.push((
                (distance, thickness),
                triple(ClusteredForwardEffect::volume_attenuation(
                    half, distance, thickness,
                )?),
            ));
        }

        let surface = Vector3::Zero;
        let up = Vector3::Up;
        let camera = Vector3::from_x_and_y_and_z(0.0, 5.0, 0.0);
        let mut overhead = ClusteredLight::canonical_defaults()?;
        overhead.position = Vector3::from_x_and_y_and_z(0.0, 3.0, 0.0);
        overhead.range = 20.0;
        let material = ClusteredShadingMaterial::default();
        findings.contributions.push((
            "a lamp overhead",
            triple(ClusteredForwardEffect::contribution(
                overhead, surface, up, camera, material,
            )?),
        ));
        let mut brighter = overhead;
        brighter.intensity = 2.0;
        findings.contributions.push((
            "twice as bright",
            triple(ClusteredForwardEffect::contribution(
                brighter, surface, up, camera, material,
            )?),
        ));
        let mut below = overhead;
        below.position = Vector3::from_x_and_y_and_z(0.0, -3.0, 0.0);
        findings.contributions.push((
            "the same lamp underneath",
            triple(ClusteredForwardEffect::contribution(
                below, surface, up, camera, material,
            )?),
        ));
        let mut out_of_range = overhead;
        out_of_range.range = 1.0;
        findings.contributions.push((
            "out of its own range",
            triple(ClusteredForwardEffect::contribution(
                out_of_range,
                surface,
                up,
                camera,
                material,
            )?),
        ));

        findings.release_after_borrow = effect.release().err().map(|e| e.to_string());

        *self.findings.lock().expect("findings") = findings;
        Ok(())
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn the_clustered_forward_effect_clamps_what_it_documents_and_shades_what_it_is_given() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(ForwardFindings::default()));
    let game = ForwardGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame with a clustered forward effect");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    println!(
        "supported: {} effect present: {}",
        findings.supported, findings.effect_present
    );
    assert_eq!(
        findings.effect_present, findings.supported,
        "the shader effect is handed out exactly when the shader linked"
    );

    // Ownership: the borrow is counted, and the count is what refuses.
    if findings.effect_present {
        assert!(
            findings.release_while_borrowed.is_some(),
            "releasing the effect while its shader is borrowed is refused"
        );
    }
    assert!(
        findings.release_after_borrow.is_none(),
        "and allowed once the borrow is gone: {:?}",
        findings.release_after_borrow
    );

    // --- the material terms ------------------------------------------------
    println!("base colour: {:?}", findings.base_color);
    assert_eq!(
        findings.base_color[0].1,
        (0.25, 0.5, 0.75),
        "a colour inside the range round-trips untouched"
    );
    assert_eq!(
        findings.base_color[1].1,
        (1.0, 0.0, 0.5),
        "and one outside it is clamped per channel, not refused or scaled"
    );

    println!("metallic: {:?}", findings.metallic);
    assert_eq!(
        findings.metallic,
        vec![(0.25, 0.25), (-1.0, 0.0), (5.0, 1.0)],
        "metallic is clamped to zero-to-one"
    );

    // The roughness floor is 0.04, not zero: a smooth surface collapses the
    // specular lobe, so the setter clamps rather than accepting it.
    println!("roughness: {:?}", findings.roughness);
    assert_eq!(
        findings.roughness,
        vec![(0.5, 0.5), (0.0, 0.04), (-1.0, 0.04), (5.0, 1.0)],
        "roughness is clamped to 0.04-to-one"
    );

    println!("ior: {:?}", findings.ior);
    for (written, read) in &findings.ior {
        assert!(
            (written - read).abs() < 1e-6,
            "the index of refraction round-trips: {written} came back as {read}"
        );
    }

    // The ambient term is *floored*, not clamped: a channel above one survives.
    println!("ambient: {:?}", findings.ambient);
    assert_eq!(
        findings.ambient[0].1,
        (0.1, 0.2, 0.3),
        "an ambient inside the range round-trips"
    );
    assert_eq!(
        findings.ambient[1].1,
        (0.0, 0.5, 2.0),
        "a negative channel is floored at zero and a bright one is kept"
    );

    assert_eq!(
        findings.bound_extras,
        (false, false),
        "a fresh effect has neither an area light nor a light probe"
    );
    assert_eq!(
        findings.cleared_extras,
        (false, false),
        "and clearing what was never bound is a no-op rather than an error"
    );

    // --- beginning a frame -------------------------------------------------
    assert!(
        findings.begin_before_upload.is_some(),
        "beginning against a buffer holding no light list is refused"
    );
    assert!(
        findings.begin_after_upload.is_none(),
        "and allowed once one has been uploaded: {:?}",
        findings.begin_after_upload
    );

    // --- the opaque frame --------------------------------------------------
    assert!(!findings.frame_before, "no frame is bound to start with");
    assert_eq!(
        findings.frame_after,
        Some((40, 24)),
        "the frame that comes back is the one that went in, at its own size"
    );
    assert!(!findings.frame_cleared, "and unbinding leaves none");

    // --- the pure functions ------------------------------------------------
    // Beer-Lambert: no thickness is no attenuation, and the attenuation colour
    // is reached at exactly the attenuation distance. Doubling the thickness
    // squares it; doubling the distance takes its square root.
    println!("attenuation: {:?}", findings.attenuation);
    let value = |index: usize| findings.attenuation[index].1;
    assert_eq!(value(0), (1.0, 1.0, 1.0), "zero thickness attenuates nothing");
    for channel in [value(1).0, value(1).1, value(1).2] {
        assert!(
            (channel - 0.5).abs() < 1e-4,
            "at the attenuation distance the colour is reached exactly: {channel}"
        );
    }
    assert!(
        (value(2).0 - 0.25).abs() < 1e-4,
        "twice the thickness squares the attenuation: {}",
        value(2).0
    );
    assert!(
        (value(3).0 - 0.5_f32.sqrt()).abs() < 1e-4,
        "twice the distance takes its square root: {}",
        value(3).0
    );

    println!("contributions: {:?}", findings.contributions);
    let contribution = |name: &str| -> (f32, f32, f32) {
        findings
            .contributions
            .iter()
            .find(|(label, _)| *label == name)
            .expect("a recorded contribution")
            .1
    };
    let lit = contribution("a lamp overhead");
    assert!(
        lit.0 > 0.0 && lit.1 > 0.0 && lit.2 > 0.0,
        "a lamp above a surface facing up lights it: {lit:?}"
    );
    let brighter = contribution("twice as bright");
    assert!(
        (brighter.0 - lit.0 * 2.0).abs() < lit.0 * 1e-3,
        "and the contribution is linear in intensity: {} against {}",
        brighter.0,
        lit.0 * 2.0
    );
    assert_eq!(
        contribution("the same lamp underneath"),
        (0.0, 0.0, 0.0),
        "a lamp behind the surface contributes nothing"
    );
    assert_eq!(
        contribution("out of its own range"),
        (0.0, 0.0, 0.0),
        "and neither does one further away than it reaches"
    );
}

/// What the light-probe and image-based-lighting run measured.
#[derive(Default)]
struct ProbeFindings {
    engine_layer: i32,
    fresh: (f32, bool, bool, usize),
    positioned: (f32, f32, f32),
    bad_coefficient: Vec<(&'static str, bool)>,
    dc_irradiance: Vec<(f32, f32, f32)>,
    directional_irradiance: Vec<(&'static str, (f32, f32, f32))>,
    scaled: Vec<(&'static str, (f32, f32, f32), bool)>,
    equality: Vec<(&'static str, bool)>,
    visibility: Vec<(&'static str, f32, f32)>,
    bad_visibility: Vec<(&'static str, bool)>,
    weights: Vec<(&'static str, f32)>,
    glsl_bytes: usize,
    ibl_defaults: (bool, i32, f32),
    ibl_states: Vec<(&'static str, bool)>,
    mip_ramp: Vec<(f32, f32, f32)>,
    degenerate_ramp: Vec<(&'static str, f32)>,
    hammersley: Vec<(i32, f32, f32)>,
    ggx_smooth: (f32, f32, f32),
    face_directions: Vec<(f32, f32, f32)>,
    equirectangular: Vec<(&'static str, f32, f32)>,
    brdf_lut: Option<(i32, i32)>,
    generators: Vec<(&'static str, std::result::Result<i32, String>)>,
    bad_generator_arguments: Vec<(&'static str, bool)>,
    generated_probe: Option<(f32, f32, f32)>,
}

struct ProbeGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<ProbeFindings>>,
}

impl GameStateAccess for ProbeGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for ProbeGame {
    #[allow(clippy::too_many_lines)]
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let mut findings = ProbeFindings {
            engine_layer: version,
            ..ProbeFindings::default()
        };

        // --- the probe as a value ---------------------------------------------
        let probe = LightProbe::new()?;
        let position = probe.position()?;
        findings.fresh = (
            position.X.abs() + position.Y.abs() + position.Z.abs(),
            probe.is_zero()?,
            probe.has_visibility()?,
            probe.coefficients()?.len(),
        );
        let placed = LightProbe::at(Vector3::from_x_and_y_and_z(3.0, -4.0, 5.0))?;
        findings.positioned = triple(placed.position()?);

        for (name, index) in [
            ("one past the table", LightProbe::COEFFICIENT_COUNT),
            ("before the table", -1),
        ] {
            findings
                .bad_coefficient
                .push((name, probe.coefficient(index).is_err()));
            findings.bad_coefficient.push((
                name,
                probe.set_coefficient(index, Vector3::Zero).is_err(),
            ));
        }

        // The band-zero term is directionally flat: with only it set, every
        // normal must receive the same irradiance.
        probe.set_coefficient(0, Vector3::from_x_and_y_and_z(1.0, 1.0, 1.0))?;
        for normal in [
            Vector3::Up,
            Vector3::from_x_and_y_and_z(1.0, 0.0, 0.0),
            Vector3::from_x_and_y_and_z(0.0, 0.0, -1.0),
        ] {
            findings.dc_irradiance.push(triple(probe.irradiance(normal)?));
        }

        // A band-one term is not: it must brighten one hemisphere and darken
        // the other, and the dark side is floored at zero rather than going
        // negative.
        probe.set_coefficient(1, Vector3::from_x_and_y_and_z(4.0, 4.0, 4.0))?;
        for (name, normal) in [
            ("along the band-one axis", Vector3::Up),
            ("against it", Vector3::from_x_and_y_and_z(0.0, -1.0, 0.0)),
            ("across it", Vector3::from_x_and_y_and_z(1.0, 0.0, 0.0)),
        ] {
            findings
                .directional_irradiance
                .push((name, triple(probe.irradiance(normal)?)));
        }

        probe.set_coefficient(1, Vector3::Zero)?;
        probe.scale(2.0)?;
        findings
            .scaled
            .push(("doubled", triple(probe.coefficient(0)?), probe.is_zero()?));
        probe.scale(0.0)?;
        findings
            .scaled
            .push(("scaled to nothing", triple(probe.coefficient(0)?), probe.is_zero()?));

        // Copying makes two probes equal by value; changing one coefficient
        // stops them being equal.
        probe.set_coefficient(0, Vector3::from_x_and_y_and_z(0.5, 0.25, 0.125))?;
        probe.set_position(Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0))?;
        let copy = LightProbe::new()?;
        findings
            .equality
            .push(("before copying", copy.value_eq(&probe)?));
        copy.copy_from(&probe)?;
        findings
            .equality
            .push(("after copying", copy.value_eq(&probe)?));
        copy.set_coefficient(4, Vector3::from_x_and_y_and_z(0.0, 0.0, 1.0))?;
        findings
            .equality
            .push(("after changing one coefficient", copy.value_eq(&probe)?));

        // --- visibility --------------------------------------------------------
        findings.weights.push((
            "with no visibility stored",
            probe.visibility_weight(Vector3::Up, 10.0)?,
        ));
        probe.set_visibility(0, 5.0, 30.0)?;
        findings.visibility.push((
            "as written",
            probe.visibility_mean(0)?,
            probe.visibility_mean_squared(0)?,
        ));
        probe.set_visibility(1, -5.0, -30.0)?;
        findings.visibility.push((
            "written negative",
            probe.visibility_mean(1)?,
            probe.visibility_mean_squared(1)?,
        ));
        findings
            .visibility
            .push(("never written", probe.visibility_mean(3)?, probe.visibility_mean_squared(3)?));
        for (name, direction) in [
            ("before the table", -1),
            ("one past the table", LightProbe::VISIBILITY_DIRECTIONS),
        ] {
            findings
                .bad_visibility
                .push((name, probe.set_visibility(direction, 1.0, 1.0).is_err()));
            findings
                .bad_visibility
                .push((name, probe.visibility_mean(direction).is_err()));
        }
        // The six slots are +X, -X, +Y, -Y, +Z, -Z in that order, and the weight
        // blends across the axes the query direction actually points along --
        // so a query straight up reads slot two and nothing else.
        probe.set_visibility(2, 5.0, 30.0)?;
        findings.weights.push((
            "at no distance at all",
            probe.visibility_weight(Vector3::Up, 0.0)?,
        ));
        findings.weights.push((
            "well inside the mean",
            probe.visibility_weight(Vector3::Up, 1.0)?,
        ));
        findings.weights.push((
            "well beyond the mean",
            probe.visibility_weight(Vector3::Up, 50.0)?,
        ));
        findings.weights.push((
            "along an axis with nothing recorded",
            probe.visibility_weight(Vector3::from_x_and_y_and_z(0.0, 0.0, 1.0), 50.0)?,
        ));
        probe.set_coefficient(0, Vector3::from_x_and_y_and_z(1.0, 1.0, 1.0))?;
        probe.scale(-1.0)?;
        findings.scaled.push((
            "scaled by a negative factor",
            triple(probe.coefficient(0)?),
            probe.is_zero()?,
        ));
        findings.glsl_bytes = LightProbe::evaluation_glsl()?.len();

        // --- image-based lighting ----------------------------------------------
        let mut ibl = ImageBasedLight::canonical_defaults()?;
        findings.ibl_defaults = (
            ibl.is_valid()?,
            ibl.prefiltered_mip_count(),
            ibl.intensity(),
        );
        let irradiance = TextureCube::new(&device, 4, false, SurfaceFormat::Color)?;
        let specular = TextureCube::new(&device, 8, true, SurfaceFormat::Color)?;
        let lut = Texture2D::new(&device, 8, 8)?;
        ibl.set_irradiance(Some(irradiance));
        findings.ibl_states.push(("with one texture", ibl.is_valid()?));
        ibl.set_prefiltered_specular(Some(specular), 4);
        findings.ibl_states.push(("with two", ibl.is_valid()?));
        ibl.set_brdf_lut(Some(lut));
        findings.ibl_states.push(("with all three", ibl.is_valid()?));
        ibl.set_prefiltered_specular(None, 4);
        findings
            .ibl_states
            .push(("with the specular cube taken away", ibl.is_valid()?));

        // --- the static maths ---------------------------------------------------
        for roughness in [0.0_f32, 0.25, 0.5, 1.0] {
            let mip = EnvironmentProcessor::mip_for_roughness(roughness, 5)?;
            findings.mip_ramp.push((
                roughness,
                mip,
                EnvironmentProcessor::roughness_for_mip(mip, 5)?,
            ));
        }
        for (name, roughness, mip_count) in [
            ("a single mip", 0.75_f32, 1_i32),
            ("no mips at all", 0.75, 0),
            ("a roughness below zero", -1.0, 5),
            ("a roughness above one", 2.0, 5),
        ] {
            findings.degenerate_ramp.push((
                name,
                EnvironmentProcessor::mip_for_roughness(roughness, mip_count)?,
            ));
        }
        for index in 0..4 {
            let point = EnvironmentProcessor::hammersley(index, 4)?;
            findings.hammersley.push((index, point.X, point.Y));
        }
        findings.ggx_smooth = triple(EnvironmentProcessor::importance_sample_ggx(
            EnvironmentProcessor::hammersley(1, 4)?,
            Vector3::Up,
            0.0,
        )?);
        for face in [
            CubeMapFace::PositiveX,
            CubeMapFace::NegativeX,
            CubeMapFace::PositiveY,
            CubeMapFace::NegativeY,
            CubeMapFace::PositiveZ,
            CubeMapFace::NegativeZ,
        ] {
            findings
                .face_directions
                .push(triple(EnvironmentProcessor::face_direction(face, 0.5, 0.5)?));
        }
        for (name, direction) in [
            ("straight up", Vector3::Up),
            ("straight down", Vector3::from_x_and_y_and_z(0.0, -1.0, 0.0)),
            ("along +X", Vector3::from_x_and_y_and_z(1.0, 0.0, 0.0)),
        ] {
            let uv = EnvironmentProcessor::direction_to_equirectangular(direction)?;
            findings.equirectangular.push((name, uv.X, uv.Y));
        }

        // --- the generators ------------------------------------------------------
        let processor = EnvironmentProcessor::new(&device)?;
        findings.brdf_lut = match processor.brdf_lut(16, 8) {
            Ok(texture) => Some((texture.Width(), texture.Height())),
            Err(_) => None,
        };
        let panorama = Texture2D::new(&device, 16, 8)?;
        panorama.SetData(&vec![Color::White; 16 * 8])?;
        let environment = match processor.convert_equirectangular(&panorama, 8) {
            Ok(cube) => {
                findings
                    .generators
                    .push(("the equirectangular conversion", Ok(cube.Size())));
                Some(cube)
            }
            Err(error) => {
                findings
                    .generators
                    .push(("the equirectangular conversion", Err(error.to_string())));
                None
            }
        };
        if let Some(environment) = environment.as_ref() {
            findings.generators.push((
                "the irradiance convolution",
                processor
                    .irradiance(environment, 4, 8)
                    .map(|cube| cube.Size())
                    .map_err(|error| error.to_string()),
            ));
            findings.generators.push((
                "the prefiltered specular chain",
                processor
                    .prefiltered_specular(environment, 8, 3, 8)
                    .map(|cube| cube.Size())
                    .map_err(|error| error.to_string()),
            ));
            findings.generated_probe = processor
                .probe(environment, Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0))
                .ok()
                .map(|probe| triple(probe.position().unwrap_or(Vector3::Zero)));
            for (name, refused) in [
                ("a zero irradiance size", processor.irradiance(environment, 0, 8).is_err()),
                ("a zero sample count", processor.irradiance(environment, 4, 0).is_err()),
                (
                    "a zero mip count",
                    processor.prefiltered_specular(environment, 8, 0, 8).is_err(),
                ),
            ] {
                findings.bad_generator_arguments.push((name, refused));
            }
        }
        findings.bad_generator_arguments.push((
            "a zero face size",
            processor.convert_equirectangular(&panorama, 0).is_err(),
        ));
        findings
            .bad_generator_arguments
            .push(("a zero table size", processor.brdf_lut(0, 8).is_err()));

        *self.findings.lock().expect("findings") = findings;
        Ok(())
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn light_probes_store_directional_light_and_the_processor_makes_the_maps_that_fill_them() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(ProbeFindings::default()));
    let game = ProbeGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame with a light probe and an environment processor");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    // --- the probe as a value ----------------------------------------------
    assert_eq!(
        findings.fresh,
        (0.0, true, false, LightProbe::COEFFICIENT_COUNT as usize),
        "a fresh probe sits at the origin, stores no light and no visibility, \
         and carries exactly the documented number of coefficients"
    );
    assert_eq!(
        findings.positioned,
        (3.0, -4.0, 5.0),
        "a probe created at a position remembers it"
    );
    for (name, refused) in &findings.bad_coefficient {
        assert!(
            refused,
            "a coefficient index {name} is refused rather than clamped"
        );
    }

    // Band zero is directionally flat: with only it set, every normal gets the
    // same irradiance, and it is not zero.
    println!("band-zero irradiance: {:?}", findings.dc_irradiance);
    let first = findings.dc_irradiance[0];
    assert!(first.0 > 0.0, "the band-zero term lights the surface");
    for value in &findings.dc_irradiance {
        assert_eq!(
            *value, first,
            "band zero is the same in every direction: {:?}",
            findings.dc_irradiance
        );
    }

    // Band one is not flat, and the dark side is floored at zero rather than
    // going negative.
    println!("band-one irradiance: {:?}", findings.directional_irradiance);
    let along = findings.directional_irradiance[0].1;
    let against = findings.directional_irradiance[1].1;
    let across = findings.directional_irradiance[2].1;
    assert!(
        along.0 > across.0 && across.0 >= against.0,
        "a band-one term brightens one hemisphere and darkens the other: {:?}",
        findings.directional_irradiance
    );
    assert_eq!(
        against,
        (0.0, 0.0, 0.0),
        "and a reconstruction that would go negative is floored at zero, not signed"
    );

    println!("scaled: {:?}", findings.scaled);
    let (_, doubled, still_lit) = findings.scaled[0];
    assert_eq!(doubled, (2.0, 2.0, 2.0), "scaling multiplies the coefficients");
    assert!(!still_lit, "a scaled probe still stores light");
    let (_, emptied, is_zero) = findings.scaled[1];
    assert_eq!(emptied, (0.0, 0.0, 0.0), "scaling by zero empties them");
    assert!(is_zero, "and the probe says it stores nothing");
    let (_, unchanged, _) = findings.scaled[2];
    assert_eq!(
        unchanged,
        (1.0, 1.0, 1.0),
        "a negative factor is ignored outright, not applied and then floored"
    );

    assert_eq!(
        findings.equality,
        vec![
            ("before copying", false),
            ("after copying", true),
            ("after changing one coefficient", false),
        ],
        "probes compare by content, across every coefficient"
    );

    // --- visibility ---------------------------------------------------------
    println!("visibility: {:?}", findings.visibility);
    assert_eq!(
        findings.visibility[0],
        ("as written", 5.0, 30.0),
        "visibility round-trips"
    );
    assert_eq!(
        findings.visibility[1],
        ("written negative", 0.0, 0.0),
        "a negative distance is floored at zero, not stored"
    );
    assert_eq!(
        findings.visibility[2],
        ("never written", 0.0, 0.0),
        "and a direction never written carries nothing"
    );
    for (name, refused) in &findings.bad_visibility {
        assert!(refused, "a visibility direction {name} is refused");
    }

    println!("weights: {:?}", findings.weights);
    let weight = |name: &str| -> f32 {
        findings
            .weights
            .iter()
            .find(|(label, _)| *label == name)
            .expect("a recorded weight")
            .1
    };
    assert_eq!(
        weight("with no visibility stored"),
        1.0,
        "an empty probe hides nothing"
    );
    assert_eq!(
        weight("at no distance at all"),
        1.0,
        "and neither does a point at no distance"
    );
    assert_eq!(
        weight("well inside the mean"),
        1.0,
        "a point closer than the mean occluder is not shadowed at all"
    );
    // Chebyshev, exactly as a variance shadow map computes it: with a mean of
    // five and a mean square of thirty the variance is five, so a point fifty
    // units out is weighted 5 / (5 + 45^2). A weight that merely *fell* with
    // distance would pass a monotonicity check and fail this.
    let expected = 5.0_f32 / (5.0 + 45.0 * 45.0);
    assert!(
        (weight("well beyond the mean") - expected).abs() < expected * 1e-3,
        "the weight beyond the mean is {}, not {expected}",
        weight("well beyond the mean")
    );
    assert_eq!(
        weight("along an axis with nothing recorded"),
        1.0,
        "an axis with no occluder statistics is trusted rather than discarded"
    );
    for (name, value) in &findings.weights {
        assert!(
            (0.0..=1.0).contains(value),
            "{name} produced a weight outside zero-to-one: {value}"
        );
    }
    assert!(findings.glsl_bytes > 0, "the evaluation GLSL is published");

    // --- image-based lighting -----------------------------------------------
    println!("image-based light defaults: {:?}", findings.ibl_defaults);
    let (valid, mips, intensity) = findings.ibl_defaults;
    assert!(!valid, "a light with no textures cannot shade");
    assert!(mips >= 1, "the default mip count is at least one: {mips}");
    assert!(intensity > 0.0, "and the default intensity is not zero");
    assert_eq!(
        findings.ibl_states,
        vec![
            ("with one texture", false),
            ("with two", false),
            ("with all three", true),
            ("with the specular cube taken away", false),
        ],
        "a nearly complete light is invalid, which is the failure the check exists for"
    );

    // --- the static maths ----------------------------------------------------
    // The roughness ramp and its inverse must agree; a one-way monotonic map
    // would satisfy every other property here.
    println!("mip ramp: {:?}", findings.mip_ramp);
    for (roughness, mip, back) in &findings.mip_ramp {
        assert!(
            (0.0..=4.0).contains(mip),
            "roughness {roughness} mapped outside the five-mip chain: {mip}"
        );
        assert!(
            (roughness - back).abs() < 1e-4,
            "roughness {roughness} came back as {back} through mip {mip}"
        );
    }
    assert!(
        findings
            .mip_ramp
            .windows(2)
            .all(|pair| pair[0].1 < pair[1].1),
        "and a rougher surface reads a higher mip: {:?}",
        findings.mip_ramp
    );

    println!("degenerate ramp: {:?}", findings.degenerate_ramp);
    assert_eq!(
        findings.degenerate_ramp[0].1, 0.0,
        "a single-mip chain has no ramp to index, so the answer is mip zero"
    );
    assert_eq!(
        findings.degenerate_ramp[1].1, 0.0,
        "and neither does an empty one"
    );
    assert_eq!(
        findings.degenerate_ramp[2].1, 0.0,
        "a roughness below zero is clamped into the ramp"
    );
    assert_eq!(
        findings.degenerate_ramp[3].1, 4.0,
        "and one above one is clamped to the last mip"
    );

    // Hammersley's first coordinate is the *centre* of the index's stratum,
    // `(i + 0.5) / n`, not `i / n`: the sequence samples texel centres, so no
    // point sits on the zero edge. Its second coordinate is the radical
    // inverse in base two, which is what makes the pairs low-discrepancy.
    println!("hammersley: {:?}", findings.hammersley);
    assert_eq!(
        findings.hammersley.iter().map(|(_, _, y)| *y).collect::<Vec<f32>>(),
        vec![0.0, 0.5, 0.25, 0.75],
        "the second coordinate is the base-two radical inverse"
    );
    for (index, x, y) in &findings.hammersley {
        let stratum = (*index as f32 + 0.5) / 4.0;
        assert!(
            (x - stratum).abs() < 1e-6,
            "point {index} has first coordinate {x}, not {stratum}"
        );
        assert!(
            (0.0..1.0).contains(y),
            "point {index} has second coordinate {y} outside the unit interval"
        );
    }
    let seconds: Vec<f32> = findings.hammersley.iter().map(|(_, _, y)| *y).collect();
    for (index, first) in seconds.iter().enumerate() {
        for second in &seconds[index + 1..] {
            assert!(
                (first - second).abs() > 1e-6,
                "the sequence repeats a point: {seconds:?}"
            );
        }
    }

    // At zero roughness the GGX lobe collapses onto the normal.
    println!("ggx at zero roughness: {:?}", findings.ggx_smooth);
    let (x, y, z) = findings.ggx_smooth;
    assert!(
        x.abs() < 1e-4 && (y - 1.0).abs() < 1e-4 && z.abs() < 1e-4,
        "a mirror-smooth lobe samples along the normal itself: {:?}",
        findings.ggx_smooth
    );

    // The six face centres are the six axes: distinct, and unit length.
    println!("face directions: {:?}", findings.face_directions);
    assert_eq!(findings.face_directions.len(), 6);
    for (index, first) in findings.face_directions.iter().enumerate() {
        let length = (first.0 * first.0 + first.1 * first.1 + first.2 * first.2).sqrt();
        assert!(
            (length - 1.0).abs() < 1e-4,
            "face {index} looks along a direction of length {length}"
        );
        for second in &findings.face_directions[index + 1..] {
            assert!(
                first != second,
                "two faces look the same way: {:?}",
                findings.face_directions
            );
        }
    }

    // The panorama mapping puts the poles at the top and bottom edges.
    println!("equirectangular: {:?}", findings.equirectangular);
    for (name, u, v) in &findings.equirectangular {
        assert!(
            (0.0..=1.0).contains(u) && (0.0..=1.0).contains(v),
            "{name} mapped outside the panorama: ({u}, {v})"
        );
    }
    let up = findings.equirectangular[0].2;
    let down = findings.equirectangular[1].2;
    assert!(
        (up - down).abs() > 0.9,
        "up and down land at opposite edges: {up} against {down}"
    );

    // --- the generators -------------------------------------------------------
    assert_eq!(
        findings.brdf_lut,
        Some((16, 16)),
        "the BRDF table is square at the size it was asked for, on every renderer"
    );
    println!("generators: {:?}", findings.generators);
    for (name, outcome) in &findings.generators {
        match outcome {
            Ok(size) => assert!(*size > 0, "{name} produced a cube of size {size}"),
            Err(message) => println!("{name} is unavailable on this renderer: {message}"),
        }
    }
    if let Some((label, Ok(size))) = findings.generators.first() {
        assert_eq!(*size, 8, "{label} honoured the face size it was given");
        assert_eq!(
            findings.generated_probe,
            Some((1.0, 2.0, 3.0)),
            "and a probe projected from it records the position it was asked for"
        );
    }
    println!("bad generator arguments: {:?}", findings.bad_generator_arguments);
    for (name, refused) in &findings.bad_generator_arguments {
        assert!(refused, "{name} is refused");
    }
}

/// What the probe-volume and probe-baker run measured.
#[derive(Default)]
struct VolumeFindings {
    engine_layer: i32,
    shape: (i32, i32, i32, i32),
    bounds: ((f32, f32, f32), (f32, f32, f32)),
    corner_positions: Vec<(f32, f32, f32)>,
    bad_index: bool,
    containment: Vec<(&'static str, bool)>,
    bad_volumes: Vec<(&'static str, bool)>,
    zero_states: Vec<(&'static str, bool)>,
    round_trip: Vec<(&'static str, bool)>,
    relocated: ((f32, f32, f32), (f32, f32, f32)),
    sample_at_probe: bool,
    sample_outside: bool,
    irradiance_matches_sample: bool,
    face_count: (i32, i32),
    face_sizes: Vec<(&'static str, i32)>,
    bad_face_size: bool,
    baker_supported: bool,
    planes: Vec<(&'static str, f32, f32)>,
    bad_planes: Vec<(&'static str, bool)>,
    face_views: Vec<(f32, f32, f32)>,
    bad_faces: Vec<(&'static str, bool)>,
    bake_calls: Option<u32>,
    bake_failure: Option<(String, u32)>,
    volume_light_calls: Option<u32>,
    volume_visibility_calls: Option<u32>,
    unsupported_bake: Option<String>,
}

struct VolumeGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<VolumeFindings>>,
}

impl GameStateAccess for VolumeGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

/// A closure that counts its calls, and the counter it writes to.
fn counting_draw() -> (Arc<Mutex<u32>>, impl FnMut(Matrix, Matrix) -> Result<()> + 'static) {
    let calls = Arc::new(Mutex::new(0_u32));
    let counter = Arc::clone(&calls);
    (calls, move |_view, _projection| {
        *counter.lock().expect("call counter") += 1;
        Ok(())
    })
}

impl Game for VolumeGame {
    #[allow(clippy::too_many_lines)]
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let mut findings = VolumeFindings {
            engine_layer: version,
            ..VolumeFindings::default()
        };

        // --- the volume -------------------------------------------------------
        let box_min = Vector3::from_x_and_y_and_z(-1.0, -2.0, -3.0);
        let box_max = Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0);
        let bounds = BoundingBox::new(box_min, box_max);
        let volume = LightProbeVolume::new(bounds, 2, 3, 4)?;
        findings.shape = (
            volume.count_x()?,
            volume.count_y()?,
            volume.count_z()?,
            volume.probe_count()?,
        );
        let read_back = volume.bounds()?;
        findings.bounds = (triple(read_back.Min), triple(read_back.Max));
        findings
            .corner_positions
            .push(triple(volume.probe_position(0, 0, 0)?));
        findings
            .corner_positions
            .push(triple(volume.probe_position(1, 2, 3)?));
        findings.bad_index = volume.probe_position(2, 0, 0).is_err();
        findings
            .containment
            .push(("the centre", volume.contains(Vector3::Zero)?));
        findings.containment.push((
            "well outside",
            volume.contains(Vector3::from_x_and_y_and_z(100.0, 0.0, 0.0))?,
        ));

        for (name, count_x, count_y, count_z, bounds) in [
            ("a count below one", 0, 2, 2, bounds),
            (
                "more probes than the ceiling",
                34,
                34,
                34,
                bounds,
            ),
            (
                "a box whose maximum is below its minimum",
                2,
                2,
                2,
                BoundingBox::new(box_max, box_min),
            ),
        ] {
            findings.bad_volumes.push((
                name,
                LightProbeVolume::new(bounds, count_x, count_y, count_z).is_err(),
            ));
        }

        findings.zero_states.push(("fresh", volume.is_zero()?));
        let lit = LightProbe::new()?;
        lit.set_coefficient(0, Vector3::from_x_and_y_and_z(1.0, 0.5, 0.25))?;
        lit.set_position(Vector3::from_x_and_y_and_z(9.0, 9.0, 9.0))?;
        volume.set_probe(0, 0, 0, &lit)?;
        findings.zero_states.push(("with one lit probe", volume.is_zero()?));

        let scratch = LightProbe::new()?;
        volume.copy_probe_into(0, 0, 0, &scratch)?;
        // The grid relocates what it stores, so the probe that comes back sits
        // at the cell rather than where the caller had put it.
        findings.relocated = (triple(lit.position()?), triple(scratch.position()?));
        findings
            .round_trip
            .push(("straight back out", scratch.value_eq(&lit)?));
        findings.round_trip.push((
            "its light",
            triple(scratch.coefficient(0)?) == triple(lit.coefficient(0)?),
        ));
        lit.set_position(volume.probe_position(0, 0, 0)?)?;
        findings
            .round_trip
            .push(("once the original is moved there too", scratch.value_eq(&lit)?));
        // Equality is position and coefficients only: CNA's header claims the
        // visibility table is compared too, and it is not.
        scratch.set_visibility(0, 12.0, 400.0)?;
        findings
            .round_trip
            .push(("with visibility on one side only", scratch.value_eq(&lit)?));

        // Sampling exactly at a probe's grid position must reproduce that
        // probe: every interpolation weight but one is zero there.
        let corner = volume.probe_position(0, 0, 0)?;
        let sampled = LightProbe::new()?;
        volume.sample_into(corner, &sampled)?;
        let expected = LightProbe::new()?;
        volume.copy_probe_into(0, 0, 0, &expected)?;
        expected.set_position(sampled.position()?)?;
        findings.sample_at_probe = sampled.value_eq(&expected)?;

        // A position outside the box is clamped into it, so it gives the same
        // answer as the nearest corner rather than an error or an empty probe.
        let outside = LightProbe::new()?;
        volume.sample_into(
            Vector3::from_x_and_y_and_z(-100.0, -100.0, -100.0),
            &outside,
        )?;
        outside.set_position(sampled.position()?)?;
        findings.sample_outside = outside.value_eq(&sampled)?;

        let normal = Vector3::from_x_and_y_and_z(0.0, 1.0, 0.0);
        let direct = volume.irradiance(corner, normal)?;
        let through_probe = sampled.irradiance(normal)?;
        findings.irradiance_matches_sample = triple(direct) == triple(through_probe);

        // --- the baker ---------------------------------------------------------
        findings.face_count = (LightProbeBaker::face_count()?, LightProbeBaker::FACE_COUNT);
        let baker = LightProbeBaker::new(&device)?;
        findings
            .face_sizes
            .push(("the default", baker.face_size()?));
        let small = LightProbeBaker::with_face_size(&device, 16)?;
        findings.face_sizes.push(("asked for sixteen", small.face_size()?));
        small.release()?;
        findings.bad_face_size = LightProbeBaker::with_face_size(&device, 0).is_err();
        findings.baker_supported = baker.is_supported()?;

        findings
            .planes
            .push(("as created", baker.near_plane()?, baker.far_plane()?));
        baker.set_planes(0.5, 250.0)?;
        findings
            .planes
            .push(("after setting a pair", baker.near_plane()?, baker.far_plane()?));
        for (name, near, far) in [
            ("a zero near plane", 0.0_f32, 250.0_f32),
            ("a negative near plane", -1.0, 250.0),
            ("a far plane below the near one", 10.0, 5.0),
        ] {
            findings
                .bad_planes
                .push((name, baker.set_planes(near, far).is_err()));
        }
        // The pair is refused as a pair: neither half may have been applied.
        findings
            .planes
            .push(("after three refusals", baker.near_plane()?, baker.far_plane()?));

        let position = Vector3::from_x_and_y_and_z(1.0, 2.0, 3.0);
        for face in 0..LightProbeBaker::FACE_COUNT {
            let view = baker.face_view(face, position)?;
            findings.face_views.push((view.M41, view.M42, view.M43));
        }
        for (name, face) in [
            ("before the first", -1),
            ("past the last", LightProbeBaker::FACE_COUNT),
        ] {
            findings
                .bad_faces
                .push((name, baker.face_view(face, position).is_err()));
        }

        if findings.baker_supported {
            let (calls, draw) = counting_draw();
            let probe = baker.bake_probe(position, draw)?;
            findings.bake_calls = Some(*calls.lock().expect("call counter"));
            drop(probe);

            // A failing callback cannot stop the capture -- the C callback
            // returns nothing -- so every face still runs and the error is
            // reported afterwards.
            let calls = Arc::new(Mutex::new(0_u32));
            let counter = Arc::clone(&calls);
            let failure = baker
                .bake_probe(position, move |_view, _projection| {
                    *counter.lock().expect("call counter") += 1;
                    Err(cna::CnaError::InvalidInput("the scene refused to draw"))
                })
                .err()
                .map(|error| error.to_string());
            findings.bake_failure =
                failure.map(|message| (message, *calls.lock().expect("call counter")));

            let small_volume = LightProbeVolume::new(bounds, 2, 1, 1)?;
            let (calls, draw) = counting_draw();
            baker.bake_light(&small_volume, draw)?;
            findings.volume_light_calls = Some(*calls.lock().expect("call counter"));
            let (calls, draw) = counting_draw();
            baker.bake_visibility(&small_volume, draw)?;
            findings.volume_visibility_calls = Some(*calls.lock().expect("call counter"));
        } else {
            let (_, draw) = counting_draw();
            findings.unsupported_bake = baker
                .bake_probe(position, draw)
                .err()
                .map(|error| error.to_string());
        }

        *self.findings.lock().expect("findings") = findings;
        Ok(())
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn a_probe_volume_interpolates_its_grid_and_a_baker_draws_six_faces_per_probe() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(VolumeFindings::default()));
    let game = VolumeGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame with a probe volume and a probe baker");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    // --- the volume ---------------------------------------------------------
    assert_eq!(
        findings.shape,
        (2, 3, 4, 24),
        "the probe count is the product of the three grid dimensions"
    );
    assert_eq!(
        findings.bounds,
        ((-1.0, -2.0, -3.0), (1.0, 2.0, 3.0)),
        "the box round-trips"
    );
    // The grid's first and last cells sit on the box corners, so the whole box
    // is covered rather than the cell centres.
    println!("corner positions: {:?}", findings.corner_positions);
    assert_eq!(
        findings.corner_positions,
        vec![(-1.0, -2.0, -3.0), (1.0, 2.0, 3.0)],
        "the outermost probes sit on the box's own corners"
    );
    assert!(findings.bad_index, "an index outside the grid is refused");
    assert_eq!(
        findings.containment,
        vec![("the centre", true), ("well outside", false)],
        "containment is the box's own test"
    );
    for (name, refused) in &findings.bad_volumes {
        assert!(refused, "{name} is refused");
    }

    assert_eq!(
        findings.zero_states,
        vec![("fresh", true), ("with one lit probe", false)],
        "a volume is dark until one of its probes is not"
    );
    // The grid decides where a probe is: what comes back has been moved to the
    // cell, and only matches the original once the original is moved there too.
    println!("relocation: {:?}", findings.relocated);
    assert_eq!(
        findings.relocated,
        ((9.0, 9.0, 9.0), (-1.0, -2.0, -3.0)),
        "storing a probe relocates the copy to its cell and leaves the caller's alone"
    );
    println!("round trip: {:?}", findings.round_trip);
    assert_eq!(
        findings.round_trip,
        vec![
            ("straight back out", false),
            ("its light", true),
            ("once the original is moved there too", true),
            ("with visibility on one side only", true),
        ],
        "the light survives the round trip, the position does not,          and visibility is not part of the comparison at all"
    );
    assert!(
        findings.sample_at_probe,
        "sampling at a probe's own position reproduces that probe exactly"
    );
    assert!(
        findings.sample_outside,
        "and a position outside the box is clamped into it rather than refused"
    );
    assert!(
        findings.irradiance_matches_sample,
        "the volume's irradiance is the sampled probe's irradiance, not a second calculation"
    );

    // --- the baker ----------------------------------------------------------
    assert_eq!(
        findings.face_count.0, findings.face_count.1,
        "the constant and the call agree about six faces"
    );
    assert_eq!(findings.face_count.1, 6, "and it is six");
    println!("face sizes: {:?}", findings.face_sizes);
    assert_eq!(
        findings.face_sizes[0].1,
        LightProbeBaker::DEFAULT_FACE_SIZE,
        "a default baker captures at the documented default size"
    );
    assert_eq!(
        findings.face_sizes[1].1, 16,
        "and one asked for sixteen captures at sixteen"
    );
    assert!(findings.bad_face_size, "a face size below one is refused");

    println!("planes: {:?}", findings.planes);
    let (_, near, far) = findings.planes[1];
    assert_eq!((near, far), (0.5, 250.0), "an ordered pair is applied");
    for (name, refused) in &findings.bad_planes {
        assert!(refused, "{name} is refused");
    }
    // The pair rule: a refused call leaves *both* halves as they were. An
    // implementation that wrote the near plane before validating the far one
    // would fail exactly here and nowhere else.
    assert_eq!(
        findings.planes[2],
        ("after three refusals", 0.5, 250.0),
        "three refused pairs left both distances untouched"
    );

    // Six faces, six distinct view translations, each the negated capture
    // position rotated into that face's basis.
    println!("face views: {:?}", findings.face_views);
    assert_eq!(findings.face_views.len(), 6);
    for (index, first) in findings.face_views.iter().enumerate() {
        for second in &findings.face_views[index + 1..] {
            assert!(
                first != second,
                "two faces capture with the same view: {:?}",
                findings.face_views
            );
        }
        let length = (first.0 * first.0 + first.1 * first.1 + first.2 * first.2).sqrt();
        let distance = (1.0_f32 * 1.0 + 2.0 * 2.0 + 3.0 * 3.0).sqrt();
        assert!(
            (length - distance).abs() < 1e-3,
            "face {index} places the eye {length} from the origin, not {distance}"
        );
    }
    for (name, refused) in &findings.bad_faces {
        assert!(refused, "a face index {name} is refused");
    }

    println!("baker supported: {}", findings.baker_supported);
    if findings.baker_supported {
        assert_eq!(
            findings.bake_calls,
            Some(6),
            "one probe is six faces, once each"
        );
        let (message, calls) = findings
            .bake_failure
            .clone()
            .expect("a failing callback reported its cause");
        println!("bake failure: {message} after {calls} faces");
        assert!(
            message.contains("the scene refused to draw"),
            "the Rust cause is what comes back, not the native code it became: {message}"
        );
        assert_eq!(
            calls, 6,
            "and every face still ran, because the C callback has no way to refuse"
        );
        assert_eq!(
            findings.volume_light_calls,
            Some(12),
            "a two-probe volume is two captures, six faces each"
        );
        assert_eq!(
            findings.volume_visibility_calls,
            Some(12),
            "and the visibility bake is a second pass over the same captures"
        );
    } else {
        let message = findings
            .unsupported_bake
            .as_deref()
            .expect("a baker that cannot capture refuses to");
        println!("baking is unavailable on this renderer: {message}");
    }
}

/// What the PBR texture-slot, material-identity and thin-film run measured.
#[derive(Default)]
struct MaterialFindings {
    engine_layer: i32,
    fresh_slots: Vec<(&'static str, bool, bool)>,
    slot_widths: Vec<(&'static str, i32)>,
    after_clearing_one: Vec<(&'static str, bool)>,
    extensions_text: String,
    material_identity: Vec<(&'static str, bool, bool)>,
    material_text: String,
    thin_film: Vec<(&'static str, (f32, f32, f32))>,
    thin_film_glsl: usize,
    weights: Vec<(i32, i32)>,
    bones: Vec<(f32, f32, f32)>,
    skinned_material: Option<(f32, bool)>,
}

struct MaterialGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<MaterialFindings>>,
}

impl GameStateAccess for MaterialGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

/// The nine texture slots, each with the width its probe texture is given.
const TEXTURE_SLOTS: [(&str, i32); 9] = [
    ("clearcoat", 2),
    ("clearcoat roughness", 3),
    ("clearcoat normal", 4),
    ("sheen colour", 5),
    ("sheen roughness", 6),
    ("transmission", 7),
    ("thickness", 8),
    ("iridescence", 9),
    ("iridescence thickness", 10),
];

impl Game for MaterialGame {
    #[allow(clippy::too_many_lines)]
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let mut findings = MaterialFindings {
            engine_layer: version,
            ..MaterialFindings::default()
        };

        // --- the nine texture slots -------------------------------------------
        let mut extensions = PbrMaterialExtensions::new()?;
        macro_rules! slot_state {
            ($label:literal, $has:ident, $get:ident) => {
                findings.fresh_slots.push((
                    $label,
                    extensions.$has(),
                    extensions.$get()?.is_some(),
                ));
            };
        }
        slot_state!("clearcoat", has_clearcoat_texture, clearcoat_texture);
        slot_state!(
            "clearcoat roughness",
            has_clearcoat_roughness_texture,
            clearcoat_roughness_texture
        );
        slot_state!(
            "clearcoat normal",
            has_clearcoat_normal_texture,
            clearcoat_normal_texture
        );
        slot_state!("sheen colour", has_sheen_color_texture, sheen_color_texture);
        slot_state!(
            "sheen roughness",
            has_sheen_roughness_texture,
            sheen_roughness_texture
        );
        slot_state!(
            "transmission",
            has_transmission_texture,
            transmission_texture
        );
        slot_state!("thickness", has_thickness_texture, thickness_texture);
        slot_state!("iridescence", has_iridescence_texture, iridescence_texture);
        slot_state!(
            "iridescence thickness",
            has_iridescence_thickness_texture,
            iridescence_thickness_texture
        );

        // Every slot gets a texture of its own distinct width, so a getter
        // wired to the wrong native route reads the wrong number rather than
        // merely "a texture".
        let width = |label: &str| -> i32 {
            TEXTURE_SLOTS
                .iter()
                .find(|(name, _)| *name == label)
                .map_or(1, |(_, width)| *width)
        };
        extensions.set_clearcoat_texture(Some(Texture2D::new(&device, width("clearcoat"), 1)?))?;
        extensions.set_clearcoat_roughness_texture(Some(Texture2D::new(
            &device,
            width("clearcoat roughness"),
            1,
        )?))?;
        extensions.set_clearcoat_normal_texture(Some(Texture2D::new(
            &device,
            width("clearcoat normal"),
            1,
        )?))?;
        extensions.set_sheen_color_texture(Some(Texture2D::new(
            &device,
            width("sheen colour"),
            1,
        )?))?;
        extensions.set_sheen_roughness_texture(Some(Texture2D::new(
            &device,
            width("sheen roughness"),
            1,
        )?))?;
        extensions.set_transmission_texture(Some(Texture2D::new(
            &device,
            width("transmission"),
            1,
        )?))?;
        extensions.set_thickness_texture(Some(Texture2D::new(&device, width("thickness"), 1)?))?;
        extensions.set_iridescence_texture(Some(Texture2D::new(
            &device,
            width("iridescence"),
            1,
        )?))?;
        extensions.set_iridescence_thickness_texture(Some(Texture2D::new(
            &device,
            width("iridescence thickness"),
            1,
        )?))?;

        macro_rules! slot_width {
            ($label:literal, $get:ident) => {
                findings.slot_widths.push((
                    $label,
                    extensions
                        .$get()?
                        .map_or(0, |view| view.texture().Width()),
                ));
            };
        }
        slot_width!("clearcoat", clearcoat_texture);
        slot_width!("clearcoat roughness", clearcoat_roughness_texture);
        slot_width!("clearcoat normal", clearcoat_normal_texture);
        slot_width!("sheen colour", sheen_color_texture);
        slot_width!("sheen roughness", sheen_roughness_texture);
        slot_width!("transmission", transmission_texture);
        slot_width!("thickness", thickness_texture);
        slot_width!("iridescence", iridescence_texture);
        slot_width!("iridescence thickness", iridescence_thickness_texture);

        extensions.set_transmission_texture(None)?;
        macro_rules! slot_has {
            ($label:literal, $has:ident) => {
                findings
                    .after_clearing_one
                    .push(($label, extensions.$has()));
            };
        }
        slot_has!("clearcoat", has_clearcoat_texture);
        slot_has!("transmission", has_transmission_texture);
        slot_has!("thickness", has_thickness_texture);
        findings.extensions_text = extensions.to_native_string()?;

        // --- material identity --------------------------------------------------
        let first = PbrMaterialFull::canonical_defaults()?;
        let same = PbrMaterialFull::canonical_defaults()?;
        findings.material_identity.push((
            "two sets of defaults",
            first.same_material(&same)?,
            first.hash_code()? == same.hash_code()?,
        ));
        let mut different = PbrMaterialFull::canonical_defaults()?;
        different.set_emissive_factor(Vector3::from_x_and_y_and_z(0.75, 0.25, 0.5));
        findings.material_identity.push((
            "one field changed",
            first.same_material(&different)?,
            first.hash_code()? == different.hash_code()?,
        ));
        findings.material_text = first.to_native_string()?;

        // --- thin-film iridescence ----------------------------------------------
        let base = Vector3::from_x_and_y_and_z(0.04, 0.05, 0.06);
        for (name, outside, film, cos_theta, thickness) in [
            ("no film at all", 1.0_f32, 1.3_f32, 1.0_f32, 0.0_f32),
            ("no film, seen edge-on", 1.0, 1.3, 0.2, 0.0),
            ("a film matching the air", 1.0, 1.0, 1.0, 400.0),
            ("total internal reflection", 1.5, 1.0, 0.1, 400.0),
            ("four hundred nanometres", 1.0, 1.3, 1.0, 400.0),
            ("eight hundred nanometres", 1.0, 1.3, 1.0, 800.0),
            ("at a grazing angle", 1.0, 1.3, 0.2, 400.0),
        ] {
            findings.thin_film.push((
                name,
                triple(ThinFilmIridescence::evaluate(
                    outside, film, cos_theta, thickness, base,
                )?),
            ));
        }
        findings.thin_film_glsl = ThinFilmIridescence::glsl()?.len();

        // --- the skinned PBR effect ---------------------------------------------
        let skinned = SkinnedPbrEffect::new(&device)?;
        for asked in [1_i32, 2, 4] {
            skinned.set_weights_per_vertex(asked)?;
            findings.weights.push((asked, skinned.weights_per_vertex()?));
        }
        let palette = vec![
            Matrix::CreateTranslation(Vector3::from_x_and_y_and_z(1.0, 0.0, 0.0)),
            Matrix::CreateTranslation(Vector3::from_x_and_y_and_z(0.0, 2.0, 0.0)),
            Matrix::CreateTranslation(Vector3::from_x_and_y_and_z(0.0, 0.0, 3.0)),
        ];
        skinned.set_bone_transforms(&palette)?;
        findings.bones = skinned
            .bone_transforms(palette.len())?
            .into_iter()
            .map(|bone| (bone.M41, bone.M42, bone.M43))
            .collect();

        let mut material = PbrMaterialFull::canonical_defaults()?;
        material.set_emissive_factor(Vector3::from_x_and_y_and_z(0.125, 0.25, 0.5));
        skinned.apply_full(&material)?;
        let extracted = skinned.extract_full()?;
        findings.skinned_material = Some((
            extracted.emissive_factor().X,
            extracted.same_material(&material)?,
        ));

        *self.findings.lock().expect("findings") = findings;
        Ok(())
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn the_material_extension_texture_slots_are_nine_independent_retained_dependencies() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(MaterialFindings::default()));
    let game = MaterialGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame with material extensions and a skinned PBR effect");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    // --- the nine texture slots ---------------------------------------------
    println!("fresh slots: {:?}", findings.fresh_slots);
    assert_eq!(findings.fresh_slots.len(), 9, "there are nine slots");
    for (name, has, published) in &findings.fresh_slots {
        assert!(!has, "the {name} slot starts empty");
        assert!(!published, "and publishes no view while it is");
    }

    // Each slot reports the width of the texture that slot was given, so a
    // getter wired to a neighbouring route reads the wrong number.
    println!("slot widths: {:?}", findings.slot_widths);
    let expected: Vec<(&str, i32)> = TEXTURE_SLOTS.to_vec();
    assert_eq!(
        findings.slot_widths, expected,
        "every slot reports its own texture, not a neighbour's"
    );

    assert_eq!(
        findings.after_clearing_one,
        vec![
            ("clearcoat", true),
            ("transmission", false),
            ("thickness", true),
        ],
        "clearing one slot leaves the others alone"
    );
    assert!(
        !findings.extensions_text.is_empty(),
        "the extension set renders itself as text"
    );

    // --- material identity ---------------------------------------------------
    println!("material identity: {:?}", findings.material_identity);
    assert_eq!(
        findings.material_identity[0],
        ("two sets of defaults", true, true),
        "two identical materials compare equal and hash equally"
    );
    let (_, same, same_hash) = findings.material_identity[1];
    assert!(!same, "changing a field makes them different materials");
    assert!(
        !same_hash,
        "and the hash follows the field rather than ignoring it"
    );
    assert!(
        !findings.material_text.is_empty(),
        "and a material renders itself as text"
    );

    // --- thin-film iridescence ------------------------------------------------
    println!("thin film: {:?}", findings.thin_film);
    let film = |name: &str| -> (f32, f32, f32) {
        findings
            .thin_film
            .iter()
            .find(|(label, _)| *label == name)
            .expect("a recorded film value")
            .1
    };
    // A film of no thickness is *exactly* the base Schlick reflectance -- a
    // deliberate departure from the glTF reference, whose 1e-5 floor leaves a
    // coloured residue of about 0.007 with the film switched off. Head-on the
    // Schlick term vanishes and the answer is the base itself; edge-on it is
    // the base plus `(1 - base) * (1 - cos)^5`, which is a number rather than a
    // direction.
    let base = (0.04_f32, 0.05_f32, 0.06_f32);
    let head_on = film("no film at all");
    assert!(
        (head_on.0 - base.0).abs() < 1e-6
            && (head_on.1 - base.1).abs() < 1e-6
            && (head_on.2 - base.2).abs() < 1e-6,
        "a film of no thickness is the base reflectance exactly, and gave {head_on:?}"
    );
    let edge_on = film("no film, seen edge-on");
    let schlick = (1.0_f32 - 0.2).powi(5);
    for (channel, base_channel) in [
        (edge_on.0, base.0),
        (edge_on.1, base.1),
        (edge_on.2, base.2),
    ] {
        let expected = base_channel + (1.0 - base_channel) * schlick;
        assert!(
            (channel - expected).abs() < 1e-4,
            "edge-on with no film is Schlick's own curve: {channel} against {expected}"
        );
    }
    // A film whose index matches the medium around it is *not* short-circuited:
    // the Airy summation still runs and shifts the colour slightly. Close to
    // the base, but not the base -- which is worth knowing before treating
    // "same index" as "no film".
    let matched = film("a film matching the air");
    assert!(
        matched != base,
        "a matching index is not a special case upstream: {matched:?}"
    );
    for (channel, base_channel) in [
        (matched.0, base.0),
        (matched.1, base.1),
        (matched.2, base.2),
    ] {
        assert!(
            (channel - base_channel).abs() < 0.02,
            "but it stays near the base: {channel} against {base_channel}"
        );
    }
    // Light that cannot enter the film all comes back.
    assert_eq!(
        film("total internal reflection"),
        (1.0, 1.0, 1.0),
        "total internal reflection returns everything"
    );
    // A real film interferes, and differently at different thicknesses and
    // angles: three values that were all equal would mean the film was ignored.
    let four_hundred = film("four hundred nanometres");
    let eight_hundred = film("eight hundred nanometres");
    let grazing = film("at a grazing angle");
    assert!(
        four_hundred != base,
        "a four-hundred-nanometre film changes the reflectance: {four_hundred:?}"
    );
    assert!(
        four_hundred != eight_hundred,
        "and doubling its thickness changes it again: {four_hundred:?} against {eight_hundred:?}"
    );
    assert!(
        four_hundred != grazing,
        "and so does looking along it: {four_hundred:?} against {grazing:?}"
    );
    for (name, value) in &findings.thin_film {
        for channel in [value.0, value.1, value.2] {
            assert!(
                (0.0..=1.0).contains(&channel),
                "{name} produced a reflectance outside zero-to-one: {value:?}"
            );
        }
    }
    assert!(
        findings.thin_film_glsl > 0,
        "the shader-side evaluation is published rather than left to be reimplemented"
    );

    // --- the skinned PBR effect ------------------------------------------------
    println!("weights per vertex: {:?}", findings.weights);
    for (asked, got) in &findings.weights {
        assert_eq!(asked, got, "the bone-weight count round-trips");
    }
    assert_eq!(
        findings.bones,
        vec![(1.0, 0.0, 0.0), (0.0, 2.0, 0.0), (0.0, 0.0, 3.0)],
        "the bone palette comes back in the order it went in"
    );
    let (emissive, same) = findings
        .skinned_material
        .expect("a material through the skinned effect");
    assert!(
        (emissive - 0.125).abs() < 1e-4,
        "a material applied to a skinned effect reads back as itself: {emissive}"
    );
    assert!(
        same,
        "and CNA agrees it is the same material that went in"
    );
}

/// What the factory, glTF bridge and newly unblocked routes measured.
#[derive(Default)]
struct BridgeFindings {
    engine_layer: i32,
    factory_states: Vec<(&'static str, bool, u64)>,
    factory_clear_while_borrowed: Option<String>,
    factory_clear_after: Option<String>,
    pipeline_skybox: Vec<(&'static str, bool, bool)>,
    gizmo_lines: Vec<(&'static str, i32)>,
    extensions_round_trip: (f32, f32),
    probe_contribution: Vec<(&'static str, (f32, f32, f32))>,
    gltf_defaults: Option<(f32, f32, f32, f32)>,
    gltf_material: Option<(f32, f32, f32)>,
    gltf_extension_defaults: Option<(f32, f32, f32, f32)>,
    built_attenuation: (f32, f32),
    gltf_extensions_built: Option<(f32, f32, bool)>,
}

struct BridgeGame {
    state: Arc<GameState>,
    findings: Arc<Mutex<BridgeFindings>>,
}

impl GameStateAccess for BridgeGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

const TRIVIAL_VERTEX: &str = "#version 300 es\nvoid main() { gl_Position = vec4(0.0, 0.0, 0.0, 1.0); }\n";
const TRIVIAL_FRAGMENT: &str =
    "#version 300 es\nprecision mediump float;\nout vec4 c;\nvoid main() { c = vec4(1.0); }\n";

impl Game for BridgeGame {
    #[allow(clippy::too_many_lines)]
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let version = engine_layer_version()?;
        self.findings.lock().expect("findings").engine_layer = version;
        if version == 0 {
            return Ok(());
        }
        let device = game.GraphicsDevice()?;
        let mut findings = BridgeFindings {
            engine_layer: version,
            ..BridgeFindings::default()
        };
        let (_view, projection) = culling_camera();

        // --- the shader effect factory ------------------------------------------
        let factory = ShaderEffectFactory::new(&device)?;
        findings.factory_states.push((
            "before anything is acquired",
            factory.contains("probe")?,
            factory.compile_count()?,
        ));
        {
            let first = factory.acquire("probe", TRIVIAL_VERTEX, TRIVIAL_FRAGMENT)?;
            findings.factory_states.push((
                "after one acquire",
                factory.contains("probe")?,
                factory.compile_count()?,
            ));
            let second = factory.acquire("probe", TRIVIAL_VERTEX, TRIVIAL_FRAGMENT)?;
            findings.factory_states.push((
                "after the same name again",
                factory.contains("probe")?,
                factory.compile_count()?,
            ));
            let other = factory.acquire("other", TRIVIAL_VERTEX, TRIVIAL_FRAGMENT)?;
            findings.factory_states.push((
                "after a second name",
                factory.contains("other")?,
                factory.compile_count()?,
            ));
            findings.factory_clear_while_borrowed =
                factory.clear().err().map(|error| error.to_string());
            drop(first);
            drop(second);
            drop(other);
        }
        findings.factory_clear_after = factory.clear().err().map(|error| error.to_string());
        factory.release()?;

        // --- the pipeline's skybox -----------------------------------------------
        let mut pipeline = RenderPipeline::new(&device)?;
        pipeline.resize(64, 48)?;
        findings.pipeline_skybox.push((
            "before one is set",
            pipeline.drawn_skybox()?.is_some(),
            pipeline.retained_skybox().is_some(),
        ));
        let environment = TextureCube::new(&device, 4, false, SurfaceFormat::Color)?;
        let sky = Arc::new(Skybox::with_environment(&device, environment)?);
        pipeline.set_skybox(Some(&sky))?;
        let drawn = pipeline.drawn_skybox()?;
        findings.pipeline_skybox.push((
            "after one is set",
            drawn.is_some(),
            pipeline.retained_skybox().is_some(),
        ));
        findings.pipeline_skybox.push((
            "and it carries the environment",
            match drawn.as_ref() {
                Some(view) => view.environment()?.is_some(),
                None => false,
            },
            true,
        ));
        drop(drawn);
        pipeline.set_skybox(None)?;
        findings.pipeline_skybox.push((
            "after clearing",
            pipeline.drawn_skybox()?.is_some(),
            pipeline.retained_skybox().is_some(),
        ));

        // --- the two gizmos that were waiting on other families -------------------
        let debug = DebugDraw::new(&device)?;
        findings.gizmo_lines.push(("empty", debug.line_count()?));
        let volume = Arc::new(LightProbeVolume::new(
            BoundingBox::new(
                Vector3::from_x_and_y_and_z(-1.0, -1.0, -1.0),
                Vector3::from_x_and_y_and_z(1.0, 1.0, 1.0),
            ),
            2,
            2,
            2,
        )?);
        debug.add_probe_volume_gizmo(&volume, Color::Cyan, 0.25)?;
        findings
            .gizmo_lines
            .push(("plus a probe volume", debug.line_count()?));
        let grid = ClusteredLightGrid::new(&device, 2, 2, 2)?;
        debug.add_cluster_slice_gizmo(&grid, Matrix::Identity, Color::Magenta)?;
        findings
            .gizmo_lines
            .push(("plus a grid with no projection", debug.line_count()?));
        grid.set_projection(projection, 1.0, 100.0)?;
        debug.add_cluster_slice_gizmo(&grid, Matrix::Identity, Color::Magenta)?;
        findings
            .gizmo_lines
            .push(("plus the same grid with one", debug.line_count()?));
        debug.clear()?;

        // --- the clustered forward effect's newly reachable routes -----------------
        let mut forward = ClusteredForwardEffect::new(&device)?;
        let extensions = PbrMaterialExtensions::new()?;
        extensions.set_clearcoat_factor(0.75)?;
        extensions.set_sheen_roughness(0.25)?;
        forward.set_material_extensions(&extensions)?;
        let borrowed = forward.material_extensions()?;
        findings.extensions_round_trip = (
            borrowed.extensions().clearcoat_factor()?,
            borrowed.extensions().sheen_roughness()?,
        );
        drop(borrowed);

        let probe = LightProbe::new()?;
        probe.set_coefficient(0, Vector3::from_x_and_y_and_z(1.0, 1.0, 1.0))?;
        forward.set_light_probe(&probe)?;
        forward.set_light_probe_volume(Some(&volume))?;
        forward.set_light_probe_volume(None)?;

        let mut lamp = ClusteredLight::canonical_defaults()?;
        lamp.position = Vector3::from_x_and_y_and_z(0.0, 3.0, 0.0);
        lamp.range = 20.0;
        let surface = Vector3::Zero;
        let normal = Vector3::Up;
        let camera = Vector3::from_x_and_y_and_z(0.0, 5.0, 0.0);
        let base = Vector3::from_x_and_y_and_z(1.0, 1.0, 1.0);
        let neutral = PbrMaterialExtensions::new()?;
        findings.probe_contribution.push((
            "with neutral extensions",
            triple(ClusteredForwardEffect::contribution_with_extensions(
                lamp, surface, normal, camera, base, 0.0, 0.5, &neutral,
            )?),
        ));
        let sheened = PbrMaterialExtensions::new()?;
        sheened.set_sheen_color_factor(Vector3::from_x_and_y_and_z(1.0, 0.0, 0.0))?;
        sheened.set_sheen_roughness(0.5)?;
        findings.probe_contribution.push((
            "with red sheen",
            triple(ClusteredForwardEffect::contribution_with_extensions(
                lamp, surface, normal, camera, base, 0.0, 0.5, &sheened,
            )?),
        ));

        // --- the glTF bridge --------------------------------------------------------
        let source = GltfMaterialSource::canonical_defaults()?;
        findings.gltf_defaults = Some((
            source.base_color_factor.W,
            source.metallic_factor,
            source.roughness_factor,
            source.ior,
        ));
        let mut authored = source;
        authored.metallic_factor = 0.25;
        authored.roughness_factor = 0.75;
        authored.emissive_factor = Vector3::from_x_and_y_and_z(0.1, 0.2, 0.3);
        let slots: [Option<&Texture2D>; 7] = [None; 7];
        let built = GltfMaterialBridge::build_material(authored, &slots)?;
        findings.gltf_material = Some((
            built.emissive_factor().X,
            built.emissive_factor().Y,
            built.emissive_factor().Z,
        ));

        let extension_source = GltfMaterialExtensionSource::canonical_defaults()?;
        findings.gltf_extension_defaults = Some((
            extension_source.iridescence_ior,
            extension_source.attenuation_distance,
            extension_source.iridescence_thickness_minimum,
            extension_source.iridescence_thickness_maximum,
        ));
        let untouched = PbrMaterialExtensions::new()?;
        untouched.set_attenuation_distance(-5.0)?;
        let from_defaults = PbrMaterialExtensions::new()?;
        from_defaults.set_attenuation_distance(2.5)?;
        GltfMaterialBridge::build_extensions(
            extension_source,
            &GltfMaterialExtensionTextures::default(),
            &from_defaults,
        )?;
        findings.built_attenuation = (
            untouched.attenuation_distance()?,
            from_defaults.attenuation_distance()?,
        );
        let mut authored = extension_source;
        authored.clearcoat_factor = 0.5;
        authored.transmission_factor = 0.25;
        let destination = PbrMaterialExtensions::new()?;
        GltfMaterialBridge::build_extensions(
            authored,
            &GltfMaterialExtensionTextures::default(),
            &destination,
        )?;
        findings.gltf_extensions_built = Some((
            destination.clearcoat_factor()?,
            destination.transmission_factor()?,
            destination.is_transmission_enabled()?,
        ));

        *self.findings.lock().expect("findings") = findings;
        Ok(())
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn the_factory_caches_by_name_and_the_gltf_bridge_builds_what_the_importer_read() {
    let _one_game = ONE_GAME_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let findings = Arc::new(Mutex::new(BridgeFindings::default()));
    let game = BridgeGame {
        state: Arc::new(GameState::default()),
        findings: Arc::clone(&findings),
    };
    run_for_frames(game, 1).expect("one frame with a shader factory and the glTF bridge");

    let findings = findings.lock().expect("findings");
    if findings.engine_layer == 0 {
        println!("engine layer absent from this artifact; nothing to qualify");
        return;
    }

    // --- the shader effect factory -------------------------------------------
    println!("factory: {:?}", findings.factory_states);
    let (_, contains, compiles) = findings.factory_states[0];
    assert!(!contains, "a fresh factory caches nothing");
    assert_eq!(compiles, 0, "and has compiled nothing");
    let (_, contains, first_compiles) = findings.factory_states[1];
    assert!(contains, "acquiring a name caches it");
    assert_eq!(first_compiles, 1, "and compiles it once");
    let (_, _, repeat_compiles) = findings.factory_states[2];
    assert_eq!(
        repeat_compiles, first_compiles,
        "asking for the same name again is a cache hit, not a second compile"
    );
    let (_, contains, second_compiles) = findings.factory_states[3];
    assert!(contains, "a second name is cached too");
    assert_eq!(
        second_compiles,
        first_compiles + 1,
        "and compiled once, so the count follows names rather than requests"
    );
    assert!(
        findings.factory_clear_while_borrowed.is_some(),
        "clearing while a view it published is alive is refused"
    );
    assert!(
        findings.factory_clear_after.is_none(),
        "and allowed once every view is gone: {:?}",
        findings.factory_clear_after
    );

    // --- the pipeline's skybox ------------------------------------------------
    println!("pipeline skybox: {:?}", findings.pipeline_skybox);
    assert_eq!(
        findings.pipeline_skybox[0],
        ("before one is set", false, false),
        "a fresh pipeline draws no skybox and retains none"
    );
    assert_eq!(
        findings.pipeline_skybox[1],
        ("after one is set", true, true),
        "setting one makes CNA report it and this crate keep it alive"
    );
    assert!(
        findings.pipeline_skybox[2].1,
        "and the borrowed view reaches the environment through it"
    );
    assert_eq!(
        findings.pipeline_skybox[3],
        ("after clearing", false, false),
        "clearing drops both CNA's pointer and the retained Arc"
    );

    // --- the two gizmos --------------------------------------------------------
    println!("gizmo lines: {:?}", findings.gizmo_lines);
    let count = |name: &str| -> i32 {
        findings
            .gizmo_lines
            .iter()
            .find(|(label, _)| *label == name)
            .expect("a recorded line count")
            .1
    };
    // A 2x2x2 volume is twelve box edges plus three cross segments per probe.
    assert_eq!(
        count("plus a probe volume"),
        12 + 8 * 3,
        "a probe volume is its box plus a cross at every probe"
    );
    // A grid with no projection has nothing to place, and draws nothing rather
    // than refusing.
    assert_eq!(
        count("plus a grid with no projection"),
        count("plus a probe volume"),
        "a grid with no projection adds no lines and does not fail"
    );
    assert!(
        count("plus the same grid with one") > count("plus a grid with no projection"),
        "and the same grid with a projection does: {:?}",
        findings.gizmo_lines
    );

    // --- the clustered forward effect -------------------------------------------
    let (clearcoat, sheen) = findings.extensions_round_trip;
    assert!(
        (clearcoat - 0.75).abs() < 1e-4 && (sheen - 0.25).abs() < 1e-4,
        "the extensions copied into the effect read back through the borrow: {clearcoat}, {sheen}"
    );
    println!("contribution: {:?}", findings.probe_contribution);
    let neutral = findings.probe_contribution[0].1;
    let sheened = findings.probe_contribution[1].1;
    assert!(
        neutral.0 > 0.0,
        "a lit surface with neutral extensions receives light: {neutral:?}"
    );
    assert!(
        sheened != neutral,
        "and a sheen changes what it receives: {sheened:?} against {neutral:?}"
    );
    assert!(
        sheened.0 > neutral.0 && (sheened.2 - neutral.2).abs() < neutral.2.max(1e-6) * 0.5,
        "a red sheen adds to the red channel: {sheened:?} against {neutral:?}"
    );

    // --- the glTF bridge ---------------------------------------------------------
    let (alpha, metallic, roughness, ior) = findings.gltf_defaults.expect("glTF defaults");
    println!("glTF defaults: alpha {alpha} metallic {metallic} roughness {roughness} ior {ior}");
    assert!(
        (alpha - 1.0).abs() < 1e-6,
        "the default base colour is opaque"
    );
    assert!(
        (metallic - 1.0).abs() < 1e-6 && (roughness - 1.0).abs() < 1e-6,
        "and glTF's own metallic and roughness defaults are one, not zero"
    );
    assert!(
        (ior - 1.5).abs() < 1e-6,
        "and the default index of refraction is glTF's 1.5, not 1.0"
    );
    assert_eq!(
        findings.gltf_material,
        Some((0.1, 0.2, 0.3)),
        "what the importer read reaches the built material"
    );

    let (iridescence_ior, attenuation, thin, thick) = findings
        .gltf_extension_defaults
        .expect("glTF extension defaults");
    println!(
        "glTF extension defaults: ior {iridescence_ior} attenuation {attenuation}          thickness {thin}..{thick}"
    );
    assert!(
        (iridescence_ior - 1.3).abs() < 1e-6,
        "the iridescence index of refraction defaults to glTF's 1.3"
    );
    assert!(
        (thin - 100.0).abs() < 1e-6 && (thick - 400.0).abs() < 1e-6,
        "and the iridescence film thickness range to glTF's 100..400 nanometres"
    );
    // glTF spells "no volume absorption" as an `attenuationDistance` of
    // `+Infinity`. CNA spells the same thing as **zero**, consistently: the
    // importer structure's initializer leaves it at zero, a fresh extension set
    // is zero, the setter floors negatives to zero, and the shader gates
    // absorption on `uAttenuationDistance > 0`. An importer that translates
    // glTF's infinity literally would get a very large finite distance and
    // almost no absorption, which looks right; one that translates "absent" to
    // zero gets what CNA means.
    println!("attenuation: {:?}", findings.built_attenuation);
    assert_eq!(
        attenuation, 0.0,
        "CNA's initializer leaves the attenuation distance at zero"
    );
    let (floored, overwritten) = findings.built_attenuation;
    assert_eq!(
        floored, 0.0,
        "a negative attenuation distance is floored at zero rather than kept"
    );
    assert_eq!(
        overwritten, 0.0,
        "and the bridge writes the source's zero over a distance that had been set,          rather than treating zero as 'leave it alone'"
    );

    let (clearcoat, transmission, transmits) = findings
        .gltf_extensions_built
        .expect("built glTF extensions");
    assert!(
        (clearcoat - 0.5).abs() < 1e-4 && (transmission - 0.25).abs() < 1e-4,
        "the extension factors reach the built set: {clearcoat}, {transmission}"
    );
    assert!(
        transmits,
        "and a non-zero transmission factor turns transmission on"
    );
}
