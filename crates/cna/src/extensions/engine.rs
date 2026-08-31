//! CNA's engine layer: the render pipeline a frame is drawn through, and the
//! shadow maps it casts from.
//!
//! None of this is XNA. `Microsoft.Xna.Framework.Graphics` has no pipeline
//! object, no scene target, no post-process chain, no shadow map and no
//! per-pass GPU timing; a game drew straight to the back buffer or to a
//! `RenderTarget2D` it managed itself. CNA's engine layer is a different thing,
//! so it lives here.
//!
//! Availability is queried, never assumed: the engine layer is a build-time
//! choice upstream, and a library without it answers `NOT_SUPPORTED` rather
//! than pretending. [`super::pbr::engine_layer_version`] is the query to make
//! before constructing anything in this module. Nor does construction prove
//! capability: a shadow map creates on a renderer that cannot cast one, which
//! is why [`ShadowMap::is_supported`] exists as a separate question.
//!
//! ## Ownership
//!
//! [`RenderPipeline`] and [`ShadowMap`] are `OWNED`: each holds a handle it
//! releases exactly once. CNA counts both against the parent game's owned
//! children and refuses to destroy a game while one is outstanding, so each
//! also registers with its device: whichever comes first, the value's own
//! `Drop` or the device's shutdown, releases it, and the second finds nothing
//! to do.
//!
//! A pipeline's depth, normal and velocity inputs are `RETAINED_DEPENDENCY`.
//! CNA stores a raw `Texture2D*` for each and retains no resource -- the C
//! route's own retention guard is a local that dies with the call -- so the
//! caller is what keeps them alive. The safe API therefore *takes* the
//! textures rather than borrowing them. A pipeline's shadow map is retained on
//! the same terms, through an [`Arc`] the caller shares.
//!
//! The scene target is a `TRANSIENT_VIEW`: the pipeline owns it and replaces it
//! on resize, so the [`Texture2D`] handed out here never destroys the handle
//! and refuses once the view has expired. A shadow map's texture and caster
//! effects are `PARENT_OWNED` and carry the map's Rust lifetime, so the borrow
//! upstream refuses to outlive cannot be written here.

#![allow(clippy::missing_errors_doc)]

use core::ffi::{c_char, c_void};
use core::marker::PhantomData;
use std::cell::RefCell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::graphics::{
    BorrowedHandle, Effect, GraphicsDevice, OwnedEngineChild, SurfaceFormat, Texture2D,
};
use crate::native::Native;
use crate::value::{BoundingBox, Color, Matrix, Vector3};

use super::pbr::{EngineRenderSettings, ShadowQuality};

/// What one finished frame of the pipeline actually did.
///
/// Every field is a measured value rather than a status, which is what makes
/// this worth reading: `passes_run` distinguishes a chain that ran from one
/// that was merely configured, and `gpu_memory_estimate_bytes` is zero exactly
/// when the renderer allocated nothing.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct RenderPipelineFrameStatistics {
    /// How many post-process passes ran.
    pub passes_run: i32,
    /// How many times the render target changed.
    pub target_switches: i32,
    /// Whether the frame rendered through an offscreen scene target.
    pub used_scene_target: bool,
    /// Whether the skybox drew.
    pub drew_skybox: bool,
    /// Estimated bytes of GPU memory the pipeline's targets hold.
    pub gpu_memory_estimate_bytes: u64,
}

/// How long one named pass took on the GPU.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct PassTiming {
    /// The pass's own name, as the engine records it.
    pub name: String,
    /// How many samples the average is over; zero when the pass was not timed.
    pub sample_count: i32,
    /// Mean milliseconds the pass took on the GPU.
    pub milliseconds: f64,
}

/// The Rust side of a pipeline draw callback.
///
/// CNA keeps a raw callback and a raw context, so this lives behind an [`Arc`]
/// whose address is what CNA is given: the allocation cannot move while the
/// registration stands, and the pipeline holds the other reference.
struct SceneCallback {
    draw: RefCell<Box<dyn FnMut() -> Result<()>>>,
    /// Set when the callback returned an error or panicked, so `end_frame` can
    /// report the Rust cause rather than the native result it was turned into.
    failure: RefCell<Option<CnaError>>,
}

impl SceneCallback {
    fn new(draw: impl FnMut() -> Result<()> + 'static) -> Arc<Self> {
        Arc::new(Self {
            draw: RefCell::new(Box::new(draw)),
            failure: RefCell::new(None),
        })
    }

    fn context(self: &Arc<Self>) -> *mut c_void {
        Arc::as_ptr(self).cast::<c_void>().cast_mut()
    }

    fn take_failure(&self) -> Option<CnaError> {
        self.failure.borrow_mut().take()
    }
}

/// Receives one draw request from inside an open frame.
///
/// A panic is caught here rather than crossing the C frame: it becomes a
/// failing result, CNA raises that out of `end`, and `end_frame` reports the
/// Rust cause.
unsafe extern "C" fn scene_trampoline(context: *mut c_void) -> sys::CNA_Result {
    if context.is_null() {
        return sys::CNA_RESULT_INVALID_ARGUMENT;
    }
    // SAFETY: the pointer is the address of an `Arc<SceneCallback>`'s contents,
    // kept alive by the pipeline for as long as the registration stands, and
    // only ever shared.
    let scene = unsafe { &*context.cast::<SceneCallback>() };
    let Ok(mut draw) = scene.draw.try_borrow_mut() else {
        // The callback re-entered the frame that is calling it. Refusing keeps
        // the borrow sound and gives `end_frame` something exact to report.
        *scene.failure.borrow_mut() = Some(CnaError::Callback(
            "a render-pipeline draw callback re-entered the frame it was drawing".to_owned(),
        ));
        return sys::CNA_RESULT_INVALID_STATE;
    };
    match catch_unwind(AssertUnwindSafe(|| draw())) {
        Ok(Ok(())) => sys::CNA_RESULT_SUCCESS,
        Ok(Err(error)) => {
            *scene.failure.borrow_mut() = Some(error);
            sys::CNA_RESULT_INVALID_STATE
        }
        Err(_) => {
            *scene.failure.borrow_mut() = Some(CnaError::Callback(
                "a render-pipeline draw callback panicked".to_owned(),
            ));
            sys::CNA_RESULT_INVALID_STATE
        }
    }
}

/// A render target another engine object owns, viewed for a bounded borrow.
///
/// CNA does not hand back the owner's own handle: it publishes a *new* handle
/// that holds the owner alive, and that handle has to be released with
/// `cna_render_target_destroy` -- which releases the view, never the owner's
/// target. Leaking it keeps the owner alive past the device that made it, so
/// this is a value with a `Drop` rather than a plain [`Texture2D`].
///
/// The lifetime is the owner's. A pipeline cannot be resized, nor a shadow map
/// destroyed, while a view of it exists, so the stale-view case upstream
/// refuses at run time cannot be written here at all.
pub struct BorrowedRenderTarget<'owner> {
    native: Arc<Native>,
    handle: sys::CNA_Handle,
    texture: Texture2D,
    owner: PhantomData<&'owner ()>,
}

impl BorrowedRenderTarget<'_> {
    fn new(native: &Arc<Native>, device: &GraphicsDevice, handle: sys::CNA_Handle) -> Result<Self> {
        let texture =
            Texture2D::from_borrowed_handle(device, handle, Arc::new(ParentOwnedBorrow))?;
        Ok(Self {
            native: Arc::clone(native),
            handle,
            texture,
            owner: PhantomData,
        })
    }

    /// The texture itself, borrowed for as long as this view lives.
    #[must_use]
    pub const fn texture(&self) -> &Texture2D {
        &self.texture
    }
}

impl Drop for BorrowedRenderTarget<'_> {
    fn drop(&mut self) {
        // SAFETY: the handle is this view's own, released exactly once. It is
        // the view CNA published, not the owner's target.
        let _ = unsafe { (self.native.render_target_destroy)(self.handle) };
    }
}

/// An effect another engine object owns, viewed for a bounded borrow.
///
/// Like [`BorrowedRenderTarget`], CNA publishes a *new* handle here and counts
/// it against the owner: upstream refuses to destroy a shadow map while one of
/// its effects is still borrowed. Releasing it is this value's `Drop`, and the
/// owner's lifetime is what stops the borrow outliving what it points into.
pub struct BorrowedEffect<'owner> {
    native: Arc<Native>,
    handle: sys::CNA_Handle,
    effect: Effect,
    owner: PhantomData<&'owner ()>,
}

impl BorrowedEffect<'_> {
    fn new(native: &Arc<Native>, device: &GraphicsDevice, handle: sys::CNA_Handle) -> Self {
        Self {
            native: Arc::clone(native),
            handle,
            effect: Effect::from_borrowed_handle(device, handle, Arc::new(ParentOwnedBorrow)),
            owner: PhantomData,
        }
    }

    /// The effect itself, borrowed for as long as this view lives.
    #[must_use]
    pub const fn effect(&self) -> &Effect {
        &self.effect
    }
}

impl Drop for BorrowedEffect<'_> {
    fn drop(&mut self) {
        // SAFETY: the handle is this view's own, released exactly once. It
        // releases the borrow CNA published, never the owner's effect.
        let _ = unsafe { (self.native.effect_destroy)(self.handle) };
    }
}

/// A handle whose validity the Rust lifetime already guarantees.
///
/// Used for the borrows a `&self` method hands out with the owner's lifetime
/// attached. There is no expiry to detect; the type exists so the resource is
/// constructed in the *borrowed* state and never destroys the handle.
struct ParentOwnedBorrow;

impl BorrowedHandle for ParentOwnedBorrow {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// One owned engine handle, released exactly once.
///
/// The handle lives behind a mutex because two paths can release it and only
/// one may call CNA: the value's own `Drop`, and the device shutdown that has
/// to happen before CNA will destroy the game.
struct EngineHandle {
    native: Arc<Native>,
    handle: Mutex<sys::CNA_Handle>,
    destroy: unsafe extern "C" fn(sys::CNA_Handle) -> sys::CNA_Result,
    released: &'static str,
}

impl EngineHandle {
    fn get(&self) -> Result<sys::CNA_Handle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput(self.released));
        }
        Ok(handle)
    }

    fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle was published by this object's own create route
        // and is released exactly once, here.
        self.native.check(unsafe { (self.destroy)(handle) })
    }
}

impl OwnedEngineChild for EngineHandle {
    fn release_native(&self) -> Result<()> {
        self.release()
    }
}

/// One engine render pipeline on a graphics device.
///
/// The pipeline has no size when it is created; `begin_frame` before
/// [`RenderPipeline::resize`] is refused, and that refusal is a different state
/// -- with a different message -- from a frame that is already open.
pub struct RenderPipeline {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
    device: GraphicsDevice,
    depth: Option<Texture2D>,
    normals: Option<Texture2D>,
    velocity: Option<Texture2D>,
    transparent: Option<Arc<SceneCallback>>,
    shadow_casters: Option<Arc<SceneCallback>>,
    shadow_map: Option<Arc<ShadowMap>>,
}

impl RenderPipeline {
    /// Creates a pipeline on a device.
    ///
    /// The device is borrowed by CNA and cloned here, so the pipeline cannot
    /// outlive the device it allocates targets on.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.render_pipeline_create)(device.handle()?, &mut handle)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.render_pipeline_destroy,
            released: "the render pipeline has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
            device: device.clone(),
            depth: None,
            normals: None,
            velocity: None,
            transparent: None,
            shadow_casters: None,
            shadow_map: None,
        })
    }

    /// The device this pipeline allocates its targets on.
    #[must_use]
    pub const fn graphics_device(&self) -> &GraphicsDevice {
        &self.device
    }

    /// Copies the pipeline's settings out.
    ///
    /// The canonical getter returns a reference into the pipeline; this copies,
    /// so the result stays correct after the pipeline changes.
    pub fn settings(&self) -> Result<EngineRenderSettings> {
        let handle = self.core.get()?;
        let mut inner = sys::CNA_RenderPipelineSettingsEXT::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_get_settings)(handle, &mut inner)
        })?;
        Ok(EngineRenderSettings::from_native(inner))
    }

    /// Copies settings into the pipeline.
    ///
    /// Every field goes through the engine's own setter, so the corrections
    /// [`EngineRenderSettings::normalize`] reports apply here too: what comes
    /// back out of [`RenderPipeline::settings`] is what the engine kept, not
    /// what was handed in.
    pub fn set_settings(&self, settings: &EngineRenderSettings) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the structure is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_set_settings)(handle, settings.as_native())
        })
    }

    /// Sizes the pipeline's targets.
    ///
    /// This replaces the scene target, so every outstanding view of it stops
    /// validating -- which is why it takes `&mut self`.
    pub fn resize(&mut self, width: i32, height: i32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the sizes are by value.
        self.native
            .check(unsafe { (self.native.engine.render_pipeline_resize)(handle, width, height) })?;
        Ok(())
    }

    /// Opens a frame, clearing to a colour.
    pub fn begin_frame(&self, clear_color: Color) -> Result<()> {
        let handle = self.core.get()?;
        let color = sys::CNA_Color {
            r: clear_color.R(),
            g: clear_color.G(),
            b: clear_color.B(),
            a: clear_color.A(),
        };
        // SAFETY: the handle is owned and the colour is borrowed for the call.
        self.native
            .check(unsafe { (self.native.engine.render_pipeline_begin)(handle, &color) })
    }

    /// Closes the frame, running the shadow pass and the post-process chain.
    ///
    /// A draw callback that failed or panicked is reported as the Rust cause it
    /// had, not as the native result CNA turned it into.
    pub fn end_frame(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned; any registered callback is alive for the
        // duration of the call because this value holds the other reference.
        let result = self
            .native
            .check(unsafe { (self.native.engine.render_pipeline_end)(handle) });
        for scene in [self.transparent.as_ref(), self.shadow_casters.as_ref()]
            .into_iter()
            .flatten()
        {
            if let Some(failure) = scene.take_failure() {
                return Err(failure);
            }
        }
        result
    }

    /// Sets the camera the frame renders from.
    pub fn set_camera(
        &self,
        view: Matrix,
        projection: Matrix,
        near_plane: f32,
        far_plane: f32,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let view = native_matrix(view);
        let projection = native_matrix(projection);
        // SAFETY: the handle is owned and both matrices are borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_set_camera)(
                handle,
                &view,
                &projection,
                near_plane,
                far_plane,
            )
        })
    }

    /// Sets the camera the skybox draws with, when it differs from the scene's.
    pub fn set_skybox_camera(&self, view: Matrix, projection: Matrix) -> Result<()> {
        let handle = self.core.get()?;
        let view = native_matrix(view);
        let projection = native_matrix(projection);
        // SAFETY: the handle is owned and both matrices are borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_set_skybox_camera)(handle, &view, &projection)
        })
    }

    /// Gives the pipeline the depth and normal buffers its passes read.
    ///
    /// CNA keeps a raw pointer to each and retains neither, so this takes the
    /// textures: the pipeline holds them for exactly as long as CNA points at
    /// them, and passing `None` for both clears the inputs and releases the
    /// previous pair.
    pub fn set_depth_normal_inputs(
        &mut self,
        depth: Option<Texture2D>,
        normals: Option<Texture2D>,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let depth_handle = match depth.as_ref() {
            Some(texture) => texture.handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        let normal_handle = match normals.as_ref() {
            Some(texture) => texture.handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        // SAFETY: the handle is owned and both texture handles are live for the
        // call; retention is what this value does with them afterwards.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_set_depth_normal_inputs)(
                handle,
                depth_handle,
                normal_handle,
            )
        })?;
        // Only after the route accepted them: a refused call leaves CNA
        // pointing at the previous pair, so replacing the retention would drop
        // exactly the textures that are still in use.
        self.depth = depth;
        self.normals = normals;
        Ok(())
    }

    /// Gives the pipeline the velocity buffer motion blur reads.
    ///
    /// Retained on the same terms as the depth and normal inputs.
    pub fn set_velocity_input(&mut self, velocity: Option<Texture2D>) -> Result<()> {
        let handle = self.core.get()?;
        let texture_handle = match velocity.as_ref() {
            Some(texture) => texture.handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        // SAFETY: the handle is owned and the texture handle is live for the call.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_set_velocity_input_ext)(handle, texture_handle)
        })?;
        self.velocity = velocity;
        Ok(())
    }

    /// Registers the callback that draws transparent geometry inside the frame.
    ///
    /// The callback runs during [`RenderPipeline::end_frame`], but **only while
    /// the pipeline's transparency mode is not
    /// [`TransparencyMode::None`](super::pbr::TransparencyMode::None)**, which
    /// is the default. Registering a callback and never seeing it run is
    /// otherwise indistinguishable from a broken registration, so the mode is
    /// part of this contract rather than a detail: upstream returns from the
    /// transparent phase before consulting the callback at all.
    ///
    /// Returning an error from it fails that frame, and the error itself is
    /// what `end_frame` returns.
    pub fn set_transparent_scene(
        &mut self,
        draw: impl FnMut() -> Result<()> + 'static,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let scene = SceneCallback::new(draw);
        // SAFETY: the handle is owned, and `scene` is moved into this value
        // below so the context outlives the registration.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_set_transparent_scene)(
                handle,
                Some(scene_trampoline),
                scene.context(),
            )
        })?;
        self.transparent = Some(scene);
        Ok(())
    }

    /// Removes the transparent-scene callback.
    pub fn clear_transparent_scene(&mut self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned; a null callback is the documented clear.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_set_transparent_scene)(
                handle,
                None,
                core::ptr::null_mut(),
            )
        })?;
        // Only after CNA has forgotten the pointer.
        self.transparent = None;
        Ok(())
    }

    /// Registers the shadow map, light and caster callback for the frame.
    ///
    /// The map is borrowed by CNA and retained here through the [`Arc`], so a
    /// pipeline cannot end up pointing at a map nothing keeps alive.
    pub fn set_shadow_scene(
        &mut self,
        shadow_map: &Arc<ShadowMap>,
        light: DirectionalLight,
        scene_bounds: BoundingBox,
        draw_casters: impl FnMut() -> Result<()> + 'static,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let map_handle = shadow_map.core.get()?;
        let light = light.to_native();
        let bounds = native_bounds(scene_bounds);
        let scene = SceneCallback::new(draw_casters);
        // SAFETY: the handle is owned, the structures are borrowed for the
        // call, and both the map and the callback context are retained below.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_set_shadow_scene)(
                handle,
                map_handle,
                &light,
                &bounds,
                Some(scene_trampoline),
                scene.context(),
            )
        })?;
        self.shadow_casters = Some(scene);
        self.shadow_map = Some(Arc::clone(shadow_map));
        Ok(())
    }

    /// Removes the shadow scene.
    pub fn clear_shadow_scene(
        &mut self,
        light: DirectionalLight,
        scene_bounds: BoundingBox,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let light = light.to_native();
        let bounds = native_bounds(scene_bounds);
        // SAFETY: the handle is owned; an invalid map handle with a null
        // callback is the documented clear, and the structures are still
        // required arguments.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_set_shadow_scene)(
                handle,
                sys::CNA_INVALID_HANDLE,
                &light,
                &bounds,
                None,
                core::ptr::null_mut(),
            )
        })?;
        self.shadow_casters = None;
        self.shadow_map = None;
        Ok(())
    }

    /// The shadow map the pipeline is drawing from, if any.
    ///
    /// Asked of CNA and then reconciled with what this value retains. It is not
    /// a handle comparison: upstream publishes a *fresh* borrowed handle for
    /// every call, aliasing the pipeline rather than naming the caller's map,
    /// so the handles necessarily differ and identity lives on the Rust side.
    /// What is checked is that both agree a map is set -- the two disagreeing
    /// would mean the pipeline points at a map nothing here keeps alive, which
    /// is exactly what the retention exists to prevent. The borrow CNA
    /// published is released before returning, because leaving it outstanding
    /// keeps the pipeline alive past its own device.
    pub fn shadow_scene_map(&self) -> Result<Option<&Arc<ShadowMap>>> {
        let handle = self.core.get()?;
        let mut map_handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_get_shadow_map)(handle, &mut map_handle)
        })?;
        if map_handle != sys::CNA_INVALID_HANDLE {
            // SAFETY: the borrowed handle was published by the call above and
            // is released exactly once, here.
            let _ = unsafe { (self.native.engine.shadow_map_destroy)(map_handle) };
        }
        match (map_handle == sys::CNA_INVALID_HANDLE, self.shadow_map.as_ref()) {
            (true, None) => Ok(None),
            (false, Some(retained)) => Ok(Some(retained)),
            (true, Some(_)) => Err(CnaError::InvalidInput(
                "this binding retains a shadow map the pipeline says it does not have",
            )),
            (false, None) => Err(CnaError::InvalidInput(
                "the pipeline reports a shadow map this binding does not retain",
            )),
        }
    }

    /// Why the transparency mode fell back, or an empty string when it did not.
    pub fn transparency_fallback_reason(&self) -> Result<String> {
        let handle = self.core.get()?;
        self.copy_text(|api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe {
                (api.render_pipeline_copy_transparency_fallback_reason_ext)(
                    handle,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
    }

    /// Turns GPU timing on or off, answering what the renderer actually did.
    ///
    /// A renderer without GPU timers accepts the request and stays off rather
    /// than refusing, so the answer is read back rather than assumed.
    pub fn set_gpu_timing_enabled(&self, value: bool) -> Result<bool> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the flag is a canonical boolean.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_set_gpu_timing_enabled_ext)(
                handle,
                u8::from(value),
            )
        })?;
        self.is_gpu_timing_enabled()
    }

    /// Whether GPU timing is on.
    pub fn is_gpu_timing_enabled(&self) -> Result<bool> {
        self.flag(self.native.engine.render_pipeline_is_gpu_timing_enabled_ext)
    }

    /// Whether the skybox drew during the last frame.
    pub fn did_skybox_draw(&self) -> Result<bool> {
        self.flag(self.native.engine.render_pipeline_did_skybox_draw)
    }

    /// Whether the shadow pass ran during the last frame.
    pub fn did_shadow_pass_run(&self) -> Result<bool> {
        self.flag(self.native.engine.render_pipeline_did_shadow_pass_run)
    }

    /// Whether the pipeline renders through an offscreen target.
    pub fn is_using_scene_target(&self) -> Result<bool> {
        self.flag(self.native.engine.render_pipeline_is_using_scene_target)
    }

    /// A view of the offscreen scene target, or `None` when there is none.
    ///
    /// **Only inside an open frame.** Upstream hands the target out only while
    /// a frame is open *and* the pipeline is rendering offscreen, so a call
    /// between frames answers `None` even on a pipeline that has one and is
    /// reporting its bytes through
    /// [`RenderPipeline::gpu_memory_estimate_bytes`]. Whether the pipeline uses
    /// one at all is [`RenderPipeline::is_using_scene_target`], which answers
    /// at any time.
    ///
    /// The texture is the pipeline's, not the caller's: it is never destroyed
    /// here, and it stops answering as soon as the pipeline is resized, has its
    /// device resources released, or is dropped.
    pub fn scene_target(&self) -> Result<Option<BorrowedRenderTarget<'_>>> {
        let handle = self.core.get()?;
        let mut texture = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_get_scene_target)(handle, &mut texture)
        })?;
        if texture == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        BorrowedRenderTarget::new(&self.native, &self.device, texture).map(Some)
    }

    /// The scene target's surface format.
    pub fn scene_target_format(&self) -> Result<SurfaceFormat> {
        let handle = self.core.get()?;
        let mut format: sys::CNA_SurfaceFormat = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_get_scene_target_format)(handle, &mut format)
        })?;
        SurfaceFormat::from_native(format)
            .ok_or(CnaError::InvalidInput("native surface format is unknown"))
    }

    /// How many passes ran in the last frame.
    pub fn last_frame_pass_count(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_get_last_frame_pass_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// The estimated GPU memory the pipeline's targets hold.
    pub fn gpu_memory_estimate_bytes(&self) -> Result<u64> {
        let handle = self.core.get()?;
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_get_gpu_memory_estimate_bytes)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// What the last frame did.
    pub fn statistics(&self) -> Result<RenderPipelineFrameStatistics> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_RenderPipelineFrameStatisticsEXT::default();
        // SAFETY: the handle is owned and the output is a live local whose
        // versioning fields CNA fills itself.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_get_statistics)(handle, &mut value)
        })?;
        Ok(RenderPipelineFrameStatistics {
            passes_run: value.passes_run,
            target_switches: value.target_switches,
            used_scene_target: value.used_scene_target != 0,
            drew_skybox: value.drew_skybox != 0,
            gpu_memory_estimate_bytes: value.gpu_memory_estimate_bytes,
        })
    }

    /// Releases the pipeline's device resources without destroying it.
    ///
    /// Refused while a frame is open, because the open frame would draw into
    /// freed memory. Every scene-target view stops validating.
    pub fn release_device_resources(&mut self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_release_device_resources_ext)(handle)
        })
    }

    /// Every pass timing the pipeline's own chain recorded, in its order.
    ///
    /// The count and the values come from the engine; nothing here invents a
    /// timing for a pass that was not timed, which is why `sample_count` is
    /// reported rather than hidden.
    pub fn pass_timings(&self) -> Result<Vec<PassTiming>> {
        let handle = self.core.get()?;
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_get_pass_timing_count_ext)(handle, &mut count)
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported more timings than fit in memory"))?;
        let mut timings = Vec::with_capacity(count);
        for index in 0..count {
            let position = index as u64;
            let mut timing = sys::CNA_PassTimingEXT::default();
            // SAFETY: the handle is owned, the index is below the reported
            // count, and the output is a live local.
            self.native.check(unsafe {
                (self.native.engine.render_pipeline_get_pass_timing_ext)(
                    handle,
                    position,
                    &mut timing,
                )
            })?;
            let name = self.copy_text(|api, destination, capacity, out_bytes| {
                // SAFETY: the destination holds `capacity` writable bytes.
                unsafe {
                    (api.render_pipeline_copy_pass_timing_name_ext)(
                        handle,
                        position,
                        destination,
                        capacity,
                        out_bytes,
                    )
                }
            })?;
            timings.push(PassTiming {
                name,
                sample_count: timing.sample_count,
                milliseconds: timing.milliseconds,
            });
        }
        Ok(timings)
    }

    /// Releases the pipeline now rather than at drop.
    ///
    /// A game that keeps a pipeline past the end of its own run needs this:
    /// CNA refuses to destroy a game while an owned child is outstanding, and
    /// the device's shutdown does exactly this call for anything still alive.
    pub fn release(&mut self) -> Result<()> {
        self.transparent = None;
        self.shadow_casters = None;
        self.shadow_map = None;
        self.depth = None;
        self.normals = None;
        self.velocity = None;
        self.core.release()
    }

    fn flag(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_RenderPipelineHandle,
            *mut sys::CNA_Bool,
        ) -> sys::CNA_Result,
    ) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value != 0)
    }

    /// CNA's size-then-copy text protocol, where the size probe is allowed to
    /// answer `BUFFER_TOO_SMALL` rather than success.
    fn copy_text(
        &self,
        mut route: impl FnMut(
            &crate::native::engine::EngineApi,
            *mut c_char,
            u64,
            *mut u64,
        ) -> sys::CNA_Result,
    ) -> Result<String> {
        let api = &self.native.engine;
        let mut required = 0_u64;
        let probe = route(api, core::ptr::null_mut(), 0, &mut required);
        if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.native.check(probe)?;
        }
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("CNA text is too large"))?;
        if capacity == 0 {
            return Ok(String::new());
        }
        let mut buffer = vec![0_u8; capacity];
        let mut written = 0_u64;
        self.native.check(route(
            api,
            buffer.as_mut_ptr().cast::<c_char>(),
            required,
            &mut written,
        ))?;
        let written = usize::try_from(written)
            .map_err(|_| CnaError::InvalidInput("CNA text is too large"))?;
        buffer.truncate(written.min(capacity));
        while buffer.last() == Some(&0) {
            buffer.pop();
        }
        String::from_utf8(buffer).map_err(|_| CnaError::InvalidInput("CNA text is not valid UTF-8"))
    }
}

impl Drop for RenderPipeline {
    fn drop(&mut self) {
        // The pipeline is released first, so CNA has forgotten the callback
        // contexts and the retained input textures before this value's fields,
        // which own both, are dropped after this returns.
        let _ = self.core.release();
    }
}

/// A directional light, as CNA's engine layer models one.
///
/// A typed Rust value rather than the C structure: the ABI carries
/// `struct_size`, `struct_version` and three padding bytes, and none of those
/// are anything a caller should be able to set wrong.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct DirectionalLight {
    /// The direction the light travels.
    pub direction: Vector3,
    /// Linear RGB colour.
    pub color: Vector3,
    /// Scalar multiplier on [`DirectionalLight::color`].
    pub intensity: f32,
    /// Whether this light should be given a shadow map.
    pub casts_shadows: bool,
}

impl DirectionalLight {
    /// CNA's own defaults, asked of the library rather than restated here.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_DirectionalLightEXT::default();
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.engine.directional_light_ext_init)(&mut value) })?;
        Ok(Self::from_native(value))
    }

    fn from_native(value: sys::CNA_DirectionalLightEXT) -> Self {
        Self {
            direction: from_native_vector3(value.direction),
            color: from_native_vector3(value.color),
            intensity: value.intensity,
            casts_shadows: value.casts_shadows != 0,
        }
    }

    fn to_native(self) -> sys::CNA_DirectionalLightEXT {
        sys::CNA_DirectionalLightEXT {
            struct_size: core::mem::size_of::<sys::CNA_DirectionalLightEXT>() as u32,
            struct_version: 1,
            direction: native_vector3(self.direction),
            color: native_vector3(self.color),
            intensity: self.intensity,
            casts_shadows: u8::from(self.casts_shadows),
            reserved: [0; 3],
        }
    }
}

/// Whether the renderer can sample a shadow map inside a shader.
///
/// Ask this **as well as** [`ShadowMap::is_supported`], not instead of it. One
/// says whether the shadow can be drawn and the other whether anything can read
/// it, and a frame needs both: a map that rasters on a renderer that cannot
/// sample it produces a texture nothing reads, which looks exactly like a scene
/// with no occluders.
pub fn supports_shadow_sampling(device: &GraphicsDevice) -> Result<bool> {
    let native = device.state_native();
    let mut value: sys::CNA_Bool = 0;
    // SAFETY: the device handle is live and the output is a live local.
    native.check(unsafe {
        (native.engine.graphics_device_supports_shadow_sampling_ext)(device.handle()?, &mut value)
    })?;
    Ok(value != 0)
}

/// One directional-light shadow map.
pub struct ShadowMap {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
    device: GraphicsDevice,
}

impl ShadowMap {
    /// Creates a shadow map at a quality preset.
    ///
    /// Creation succeeds on a renderer that cannot cast shadows; ask
    /// [`ShadowMap::is_supported`] rather than reading success as capability.
    pub fn new(device: &GraphicsDevice, quality: ShadowQuality) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.shadow_map_create)(device.handle()?, quality.to_native(), &mut handle)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.shadow_map_destroy,
            released: "the shadow map has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
            device: device.clone(),
        })
    }

    /// The device this map renders on.
    #[must_use]
    pub const fn graphics_device(&self) -> &GraphicsDevice {
        &self.device
    }

    /// Whether this renderer can cast into the map.
    ///
    /// The honest answer rather than a guess from the renderer's name: the
    /// caster shader has to exist *and* link, and a renderer can advertise
    /// custom effects and still fail to compile this one.
    pub fn is_supported(&self) -> Result<bool> {
        self.flag(self.native.engine.shadow_map_is_supported)
    }

    /// Opens the shadow pass, binding the map and computing the light transform.
    pub fn begin(&self, light: DirectionalLight, scene_bounds: BoundingBox) -> Result<()> {
        let handle = self.core.get()?;
        let light = light.to_native();
        let bounds = native_bounds(scene_bounds);
        // SAFETY: the handle is owned and both structures are borrowed for the call.
        self.native
            .check(unsafe { (self.native.engine.shadow_map_begin)(handle, &light, &bounds) })
    }

    /// Closes the shadow pass.
    pub fn end(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.shadow_map_end)(handle) })
    }

    /// Applies the caster effect for a rigid draw inside the pass.
    pub fn apply_caster(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.shadow_map_apply_caster)(handle) })
    }

    /// Applies the skinned caster effect with a bone palette.
    pub fn apply_skinned_caster(
        &self,
        bone_transforms: &[Matrix],
        weights_per_vertex: i32,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let bones: Vec<sys::CNA_Matrix> =
            bone_transforms.iter().copied().map(native_matrix).collect();
        let pointer = if bones.is_empty() {
            core::ptr::null()
        } else {
            bones.as_ptr()
        };
        // SAFETY: the handle is owned and the palette is borrowed for the call,
        // with its own length passed alongside it.
        self.native.check(unsafe {
            (self.native.engine.shadow_map_apply_skinned_caster)(
                handle,
                pointer,
                bones.len() as u64,
                weights_per_vertex,
            )
        })
    }

    /// The map's own caster effect, borrowed for as long as the map lives.
    ///
    /// `None` when the renderer has no caster shader, which is the same fact
    /// [`ShadowMap::is_supported`] reports.
    pub fn caster_effect(&self) -> Result<Option<BorrowedEffect<'_>>> {
        self.borrowed_effect(self.native.engine.shadow_map_get_caster_effect)
    }

    /// The map's skinned caster effect, borrowed on the same terms.
    pub fn skinned_caster_effect(&self) -> Result<Option<BorrowedEffect<'_>>> {
        self.borrowed_effect(self.native.engine.shadow_map_get_skinned_caster_effect)
    }

    /// A borrowed view of the map's depth texture.
    ///
    /// `None` when the renderer has no shadow target to hand out.
    pub fn shadow_texture(&self) -> Result<Option<BorrowedRenderTarget<'_>>> {
        let handle = self.core.get()?;
        let mut texture = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.shadow_map_get_shadow_texture)(handle, &mut texture)
        })?;
        if texture == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        BorrowedRenderTarget::new(&self.native, &self.device, texture).map(Some)
    }

    /// The transform from world space into the map, as of the last `begin`.
    pub fn light_view_projection(&self) -> Result<Matrix> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_Matrix::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.shadow_map_get_light_view_projection)(handle, &mut value)
        })?;
        Ok(from_native_matrix(value))
    }

    /// The map's edge length in texels.
    pub fn size(&self) -> Result<i32> {
        self.count(self.native.engine.shadow_map_get_size)
    }

    /// The filter radius in texels the map's quality selects.
    pub fn filter_radius(&self) -> Result<i32> {
        self.count(self.native.engine.shadow_map_get_filter_radius)
    }

    /// The quality preset the map was created with.
    pub fn quality(&self) -> Result<ShadowQuality> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_ShadowQuality = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.shadow_map_get_quality)(handle, &mut value) })?;
        ShadowQuality::from_native(value)
            .ok_or(CnaError::InvalidInput("native shadow quality is unknown"))
    }

    /// The depth bias applied when casting.
    pub fn depth_bias(&self) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.shadow_map_get_depth_bias)(handle, &mut value) })?;
        Ok(value)
    }

    /// Sets the depth bias applied when casting.
    pub fn set_depth_bias(&self, value: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.native
            .check(unsafe { (self.native.engine.shadow_map_set_depth_bias)(handle, value) })
    }

    /// Releases the map now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }

    /// The map size a quality preset selects, without creating a map.
    pub fn size_for_quality(quality: ShadowQuality) -> Result<i32> {
        let native = Native::process()?;
        let mut value = 0_i32;
        // SAFETY: the identity is canonical and the output is a live local.
        native.check(unsafe {
            (native.engine.shadow_map_size_for_quality)(quality.to_native(), &mut value)
        })?;
        Ok(value)
    }

    /// The filter radius a quality preset selects, without creating a map.
    pub fn filter_radius_for_quality(quality: ShadowQuality) -> Result<i32> {
        let native = Native::process()?;
        let mut value = 0_i32;
        // SAFETY: the identity is canonical and the output is a live local.
        native.check(unsafe {
            (native.engine.shadow_map_filter_radius_for_quality)(quality.to_native(), &mut value)
        })?;
        Ok(value)
    }

    /// A directional light's view transform for a scene, without a map.
    ///
    /// A pure function of its arguments upstream, so it needs no map and works
    /// wherever the engine layer is present.
    pub fn compute_light_view(light: DirectionalLight, scene_bounds: BoundingBox) -> Result<Matrix> {
        let native = Native::process()?;
        let light = light.to_native();
        let bounds = native_bounds(scene_bounds);
        let mut value = sys::CNA_Matrix::default();
        // SAFETY: both inputs are borrowed for the call and the output is a live local.
        native.check(unsafe {
            (native.engine.shadow_map_compute_light_view)(&light, &bounds, &mut value)
        })?;
        Ok(from_native_matrix(value))
    }

    /// The projection that fits a scene into a light's view.
    pub fn compute_light_projection(
        light_view: Matrix,
        scene_bounds: BoundingBox,
    ) -> Result<Matrix> {
        let native = Native::process()?;
        let view = native_matrix(light_view);
        let bounds = native_bounds(scene_bounds);
        let mut value = sys::CNA_Matrix::default();
        // SAFETY: both inputs are borrowed for the call and the output is a live local.
        native.check(unsafe {
            (native.engine.shadow_map_compute_light_projection)(&view, &bounds, &mut value)
        })?;
        Ok(from_native_matrix(value))
    }

    fn flag(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_ShadowMapHandle,
            *mut sys::CNA_Bool,
        ) -> sys::CNA_Result,
    ) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value != 0)
    }

    fn count(
        &self,
        route: unsafe extern "C" fn(sys::CNA_ShadowMapHandle, *mut i32) -> sys::CNA_Result,
    ) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value)
    }

    fn borrowed_effect(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_ShadowMapHandle,
            *mut sys::CNA_EffectHandle,
        ) -> sys::CNA_Result,
    ) -> Result<Option<BorrowedEffect<'_>>> {
        let handle = self.core.get()?;
        let mut effect = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut effect) })?;
        if effect == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        Ok(Some(BorrowedEffect::new(
            &self.native,
            &self.device,
            effect,
        )))
    }
}

impl Drop for ShadowMap {
    fn drop(&mut self) {
        // Every borrow this map hands out carries its Rust lifetime, so none
        // can still be outstanding here.
        let _ = self.core.release();
    }
}

fn native_matrix(value: Matrix) -> sys::CNA_Matrix {
    sys::CNA_Matrix {
        m11: value.M11,
        m12: value.M12,
        m13: value.M13,
        m14: value.M14,
        m21: value.M21,
        m22: value.M22,
        m23: value.M23,
        m24: value.M24,
        m31: value.M31,
        m32: value.M32,
        m33: value.M33,
        m34: value.M34,
        m41: value.M41,
        m42: value.M42,
        m43: value.M43,
        m44: value.M44,
    }
}

fn from_native_matrix(value: sys::CNA_Matrix) -> Matrix {
    Matrix::new(
        value.m11, value.m12, value.m13, value.m14, value.m21, value.m22, value.m23, value.m24,
        value.m31, value.m32, value.m33, value.m34, value.m41, value.m42, value.m43, value.m44,
    )
}

fn native_vector3(value: Vector3) -> sys::CNA_Vector3 {
    sys::CNA_Vector3 {
        x: value.X,
        y: value.Y,
        z: value.Z,
    }
}

fn from_native_vector3(value: sys::CNA_Vector3) -> Vector3 {
    Vector3::from_x_and_y_and_z(value.x, value.y, value.z)
}

fn native_bounds(value: BoundingBox) -> sys::CNA_BoundingBox {
    sys::CNA_BoundingBox {
        min: native_vector3(value.Min),
        max: native_vector3(value.Max),
    }
}
