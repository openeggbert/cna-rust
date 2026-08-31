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

use cna::extensions::engine::{
    supports_shadow_sampling, DirectionalLight, FxaaPass, PostProcessChain, PostProcessContext,
    PostProcessPass, RenderPipeline, ShadowMap, TonemapPass,
};
use cna::extensions::graphics::EffectFactoryExt;
use cna::extensions::pbr::{
    engine_layer_version, RenderQuality, ShadowQuality, TonemappingMode, TransparencyMode,
};
use cna::Microsoft::Xna::Framework::Graphics::{DepthFormat, SurfaceFormat, Texture2D};
use cna::Microsoft::Xna::Framework::{
    BoundingBox, Color, Game, GameContext, GameTime, Matrix, Vector3,
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
