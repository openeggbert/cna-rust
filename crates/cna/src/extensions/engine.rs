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
use crate::graphics::{
    DepthFormat, RenderTarget2D, SamplerState, Texture3D, TextureCube, VertexPositionColor,
};
use crate::value::{
    BoundingBox, BoundingFrustum, BoundingSphere, Color, Matrix, Vector2, Vector3, Vector4,
};

use super::pbr::{EngineRenderSettings, RenderQuality, ShadowQuality, TonemappingMode};

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

    /// Drops the handle without releasing it, for a route that consumed it.
    fn forget(&self) {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = sys::CNA_INVALID_HANDLE;
    }

    /// Releases the handle, keeping it when CNA refuses to destroy it.
    ///
    /// The refusal is a reachable state, not a theoretical one: upstream
    /// declines to destroy an object while a counted borrow taken from it is
    /// still outstanding. Clearing the slot first and reporting the error
    /// afterwards would drop the only handle anyone had to a live native
    /// object -- every later call would answer "has been released" and the
    /// process would abort at exit with the child still owned. So the slot is
    /// cleared only once the destroy has actually succeeded.
    fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = *guard;
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle was published by this object's own create route
        // and is released exactly once, here -- the slot is cleared only on
        // success, so a refused destroy leaves it callable rather than lost.
        self.native.check(unsafe { (self.destroy)(handle) })?;
        *guard = sys::CNA_INVALID_HANDLE;
        Ok(())
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
    user_passes: Vec<Arc<PostProcessPass>>,
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
            user_passes: Vec::new(),
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

    /// Appends a caller-owned post-process pass to run after the built-in ones.
    ///
    /// **Borrowed:** the pipeline records the pass and never owns it, so the
    /// [`Arc`] is what keeps it alive for as long as it is registered.
    pub fn add_user_pass(&mut self, pass: &Arc<PostProcessPass>) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: both handles are live; retention follows on success.
        self.native.check(unsafe {
            (self.native.engine.render_pipeline_add_user_pass)(handle, pass.core.get()?)
        })?;
        self.user_passes.push(Arc::clone(pass));
        Ok(())
    }

    /// Removes every caller-owned pass.
    pub fn clear_user_passes(&mut self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.render_pipeline_clear_user_passes)(handle) })?;
        self.user_passes.clear();
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
        self.user_passes.clear();
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

fn from_native_bounds(value: sys::CNA_BoundingBox) -> BoundingBox {
    BoundingBox::new(
        from_native_vector3(value.min),
        from_native_vector3(value.max),
    )
}

/// One frame's inputs to a post-process pass or chain.
///
/// A typed Rust value rather than the C structure. The ABI's version-2 form
/// grew a borrowed `settings` pointer, so a caller that filled the structure by
/// hand would have to get `struct_size` right or silently lose fields; here the
/// versioning is filled from CNA's own initializer and the borrow is a Rust
/// reference with a lifetime.
///
/// The source and destination are `BORROWED`: CNA reads them for the call and
/// retains nothing, and the Rust references say so.
pub struct PostProcessContext<'frame> {
    inner: sys::CNA_PostProcessContext,
    frame: PhantomData<&'frame ()>,
}

impl<'frame> PostProcessContext<'frame> {
    /// CNA's own defaults: no textures, zero size, identity matrices, no
    /// settings and no previous frame.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut inner = sys::CNA_PostProcessContext::default();
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.engine.post_process_context_init)(&mut inner) })?;
        Ok(Self {
            inner,
            frame: PhantomData,
        })
    }

    /// The colour input the pass reads.
    #[must_use]
    pub fn source(mut self, texture: &'frame Texture2D) -> Result<Self> {
        self.inner.source = texture.handle()?;
        Ok(self)
    }

    /// The linear-depth input, for a pass that reads depth.
    #[must_use]
    pub fn source_depth(mut self, texture: &'frame Texture2D) -> Result<Self> {
        self.inner.source_depth = texture.handle()?;
        Ok(self)
    }

    /// The normals input, for a pass that reads normals.
    #[must_use]
    pub fn source_normals(mut self, texture: &'frame Texture2D) -> Result<Self> {
        self.inner.source_normals = texture.handle()?;
        Ok(self)
    }

    /// The velocity input, for a pass that reads velocity.
    #[must_use]
    pub fn source_velocity(mut self, texture: &'frame Texture2D) -> Result<Self> {
        self.inner.source_velocity = texture.handle()?;
        Ok(self)
    }

    /// The destination render target; the back buffer when this is not set.
    #[must_use]
    pub fn destination(mut self, target: &'frame RenderTarget2D) -> Result<Self> {
        self.inner.destination = target.handle()?;
        Ok(self)
    }

    /// The destination size in pixels.
    #[must_use]
    pub const fn size(mut self, width: i32, height: i32) -> Self {
        self.inner.width = width;
        self.inner.height = height;
        self
    }

    /// Seconds since the previous frame.
    #[must_use]
    pub const fn elapsed_seconds(mut self, value: f32) -> Self {
        self.inner.elapsed_seconds = value;
        self
    }

    /// The camera's depth range.
    #[must_use]
    pub const fn depth_range(mut self, near_plane: f32, far_plane: f32) -> Self {
        self.inner.near_plane = near_plane;
        self.inner.far_plane = far_plane;
        self
    }

    /// The camera matrices a reprojecting pass reads.
    #[must_use]
    pub fn camera(mut self, projection: Matrix, inverse_projection: Matrix, inverse_view: Matrix) -> Self {
        self.inner.projection = native_matrix(projection);
        self.inner.inverse_projection = native_matrix(inverse_projection);
        self.inner.inverse_view = native_matrix(inverse_view);
        self
    }

    /// The previous frame's view-projection, which marks the frame as having one.
    #[must_use]
    pub fn previous_view_projection(mut self, value: Matrix) -> Self {
        self.inner.previous_view_projection = native_matrix(value);
        self.inner.has_previous_frame = sys::CNA_TRUE;
        self
    }

    /// The settings a pass reads, borrowed for the frame.
    ///
    /// The pointer CNA keeps is the caller's, not a copy, which is why this
    /// takes a reference whose lifetime the context carries.
    #[must_use]
    pub fn settings(mut self, settings: &'frame EngineRenderSettings) -> Self {
        self.inner.settings = settings.as_native();
        self
    }

    const fn as_native(&self) -> &sys::CNA_PostProcessContext {
        &self.inner
    }
}

/// One engine-layer post-process pass.
///
/// `OWNED`: created by [`PostProcessPass::blit`] or
/// [`PostProcessPass::from_effect`], released exactly once. A pass created with
/// [`PostProcessPass::owning_effect`] releases the effect with it.
pub struct PostProcessPass {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
    device: GraphicsDevice,
    /// The effect a borrowing pass draws through, kept alive by this value
    /// because CNA holds a raw pointer to it and retains nothing.
    borrowed_effect: Option<Effect>,
}

impl PostProcessPass {
    /// A pass that copies its source to its destination unchanged.
    pub fn blit(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe { (native.engine.blit_pass_create)(device.handle()?, &mut handle) })?;
        Ok(Self::adopt(native, device, handle, None))
    }

    /// A pass that draws its source through an effect it does **not** own.
    ///
    /// CNA keeps a raw pointer to the effect and retains nothing, so the pass
    /// takes it: Rust is then what guarantees the effect outlives the pass,
    /// which is the invariant upstream states and cannot enforce.
    pub fn from_effect(device: &GraphicsDevice, effect: Effect, name: &str) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        let view = string_view(name);
        // SAFETY: the device and effect handles are live, and `name` is
        // borrowed for the duration of the call.
        native.check(unsafe {
            (native.engine.post_process_effect_pass_create)(
                device.handle()?,
                effect.native_handle()?,
                view,
                &mut handle,
            )
        })?;
        Ok(Self::adopt(native, device, handle, Some(effect)))
    }

    /// A pass that **takes over** an effect.
    ///
    /// The consuming form. On success CNA owns the effect and the Rust value
    /// hands over its handle without destroying it; on failure the effect comes
    /// back untouched, so a refused call never strands a resource. That is why
    /// the error carries the effect rather than dropping it.
    pub fn owning_effect(
        device: &GraphicsDevice,
        effect: Effect,
        name: &str,
    ) -> std::result::Result<Self, EffectNotTransferred> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        let view = string_view(name);
        let device_handle = match device.handle() {
            Ok(value) => value,
            Err(error) => return Err(EffectNotTransferred { effect, error }),
        };
        let effect_handle = match effect.native_handle() {
            Ok(value) => value,
            Err(error) => return Err(EffectNotTransferred { effect, error }),
        };
        // SAFETY: the handles are live and `name` is borrowed for the call. The
        // effect is relinquished only after the route reports success, so a
        // refusal leaves this value still owning it.
        let result = native.check(unsafe {
            (native.engine.post_process_effect_pass_create_owning)(
                device_handle,
                effect_handle,
                view,
                &mut handle,
            )
        });
        match result {
            Ok(()) => {
                effect.relinquish();
                Ok(Self::adopt(native, device, handle, None))
            }
            Err(error) => Err(EffectNotTransferred { effect, error }),
        }
    }

    fn adopt(
        native: &Arc<Native>,
        device: &GraphicsDevice,
        handle: sys::CNA_Handle,
        borrowed_effect: Option<Effect>,
    ) -> Self {
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.post_process_pass_destroy,
            released: "the post-process pass has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Self {
            core,
            native: Arc::clone(native),
            device: device.clone(),
            borrowed_effect,
        }
    }

    /// The pass's own name, as the engine records it.
    pub fn name(&self) -> Result<String> {
        let handle = self.core.get()?;
        copy_text(&self.native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe { (api.post_process_pass_copy_name)(handle, destination, capacity, out_bytes) }
        })
    }

    /// Whether the pass can do its real work on a device.
    ///
    /// A pass that answers `false` is not broken: upstream's contract is that
    /// such a pass degrades -- typically to a copy -- rather than failing. Ask
    /// this to know which you will get, not to decide whether calling is safe.
    pub fn is_supported(&self, device: &GraphicsDevice) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: both handles are live and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.post_process_pass_is_supported)(
                handle,
                device.handle()?,
                &mut value,
            )
        })?;
        Ok(value != 0)
    }

    /// Runs the pass over one frame's inputs.
    pub fn apply(&self, context: &PostProcessContext<'_>) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the context is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.post_process_pass_apply)(handle, context.as_native())
        })
    }

    /// The effect an effect pass draws through, borrowed from the pass.
    pub fn effect(&self) -> Result<Option<BorrowedEffect<'_>>> {
        let handle = self.core.get()?;
        let mut effect = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.post_process_effect_pass_get_effect)(handle, &mut effect)
        })?;
        if effect == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        Ok(Some(BorrowedEffect::new(
            &self.native,
            &self.device,
            effect,
        )))
    }

    /// Replaces the effect an effect pass draws through, borrowing the new one.
    ///
    /// A pass created by [`PostProcessPass::owning_effect`] still owns the
    /// effect it was given: setting a new one does not release it, exactly as
    /// the canonical setter does not.
    pub fn set_effect(&mut self, effect: Option<Effect>) -> Result<()> {
        let handle = self.core.get()?;
        let effect_handle = match effect.as_ref() {
            Some(value) => value.native_handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        // SAFETY: the handle is owned and the effect handle is live for the call.
        self.native.check(unsafe {
            (self.native.engine.post_process_effect_pass_set_effect)(handle, effect_handle)
        })?;
        self.borrowed_effect = effect;
        Ok(())
    }

    /// Releases the pass now rather than at drop.
    pub fn release(&mut self) -> Result<()> {
        let result = self.core.release();
        self.borrowed_effect = None;
        result
    }
}

impl Drop for PostProcessPass {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// An effect a consuming route refused to take over.
///
/// Upstream's contract is that a refused transfer leaves the caller owning what
/// it owned, and this type is how that reaches Rust: the effect comes back with
/// the failure rather than being dropped inside a call that did nothing.
pub struct EffectNotTransferred {
    /// The effect, still owned by the caller.
    pub effect: Effect,
    /// Why the transfer was refused.
    pub error: CnaError,
}

impl core::fmt::Debug for EffectNotTransferred {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("EffectNotTransferred")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl core::fmt::Display for EffectNotTransferred {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "the effect was not taken over: {}", self.error)
    }
}

impl std::error::Error for EffectNotTransferred {}

/// A pass a consuming route refused to take over.
pub struct PassNotTransferred {
    /// The pass, still owned by the caller.
    pub pass: PostProcessPass,
    /// Why the transfer was refused.
    pub error: CnaError,
}

impl core::fmt::Debug for PassNotTransferred {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PassNotTransferred")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl core::fmt::Display for PassNotTransferred {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "the pass was not taken over: {}", self.error)
    }
}

impl std::error::Error for PassNotTransferred {}

/// An ordered chain of post-process passes over pooled intermediate targets.
pub struct PostProcessChain {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
    device: GraphicsDevice,
    /// Passes the chain only borrows. CNA keeps raw pointers to them, so this
    /// value is what keeps them alive for as long as they are in the chain.
    borrowed: Vec<Arc<PostProcessPass>>,
    /// Effects that belonged to passes the chain has taken over. The chain owns
    /// the passes now, but a borrowing pass's effect was never the pass's to
    /// own, so it moves here and outlives the chain's use of it.
    owned_effects: Vec<Effect>,
}

impl PostProcessChain {
    /// Creates an empty chain.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.post_process_chain_create)(device.handle()?, &mut handle)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.post_process_chain_destroy,
            released: "the post-process chain has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
            device: device.clone(),
            borrowed: Vec::new(),
            owned_effects: Vec::new(),
        })
    }

    /// The device this chain allocates its intermediates on.
    #[must_use]
    pub const fn graphics_device(&self) -> &GraphicsDevice {
        &self.device
    }

    /// Appends a pass the caller keeps owning.
    ///
    /// The chain holds a raw pointer to it and releases nothing, so the [`Arc`]
    /// is what keeps it alive: a pass dropped while still in a chain would
    /// leave CNA reading freed memory, which is precisely what the shared
    /// reference prevents.
    pub fn add_pass(&mut self, pass: &Arc<PostProcessPass>) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: both handles are live; retention follows on success.
        self.native.check(unsafe {
            (self.native.engine.post_process_chain_add_pass)(handle, pass.core.get()?)
        })?;
        self.borrowed.push(Arc::clone(pass));
        Ok(())
    }

    /// Appends a pass and hands ownership of it to the chain.
    ///
    /// The consuming form, and the one route in the engine layer that
    /// invalidates a handle the caller still holds. On success the chain owns
    /// the pass and the Rust value forgets its handle; on failure -- a pass
    /// that is still lending its effect is the documented case -- the pass
    /// comes back untouched.
    pub fn add_owned_pass(
        &mut self,
        mut pass: PostProcessPass,
    ) -> std::result::Result<(), PassNotTransferred> {
        let handle = match self.core.get() {
            Ok(value) => value,
            Err(error) => return Err(PassNotTransferred { pass, error }),
        };
        let pass_handle = match pass.core.get() {
            Ok(value) => value,
            Err(error) => return Err(PassNotTransferred { pass, error }),
        };
        // SAFETY: both handles are live. The Rust handle is forgotten only
        // after the route reports success.
        let result = self.native.check(unsafe {
            (self.native.engine.post_process_chain_add_owned_pass)(handle, pass_handle)
        });
        match result {
            Ok(()) => {
                pass.core.forget();
                // The effect a borrowing pass kept alive has to outlive the
                // chain now, so it moves here rather than dying with the
                // Rust-side pass value.
                if let Some(effect) = pass.borrowed_effect.take() {
                    self.owned_effects.push(effect);
                }
                Ok(())
            }
            Err(error) => Err(PassNotTransferred { pass, error }),
        }
    }

    /// Removes every pass, releasing the ones the chain owns.
    pub fn clear(&mut self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.post_process_chain_clear)(handle) })?;
        self.borrowed.clear();
        self.owned_effects.clear();
        Ok(())
    }

    /// How many passes the chain holds.
    pub fn pass_count(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.post_process_chain_get_pass_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Runs every pass in order, ping-ponging between pooled targets.
    pub fn apply(&self, context: &PostProcessContext<'_>) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the context is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.post_process_chain_apply)(handle, context.as_native())
        })
    }

    /// Releases the chain's pooled intermediate targets.
    pub fn reset_targets(&mut self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.post_process_chain_reset_targets)(handle) })
    }

    /// Turns GPU timing on or off, answering what the renderer actually did.
    pub fn set_gpu_timing_enabled(&self, value: bool) -> Result<bool> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the flag is a canonical boolean.
        self.native.check(unsafe {
            (self.native.engine.post_process_chain_set_gpu_timing_enabled)(
                handle,
                u8::from(value),
            )
        })?;
        self.is_gpu_timing_enabled()
    }

    /// Whether GPU timing is on.
    pub fn is_gpu_timing_enabled(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.post_process_chain_is_gpu_timing_enabled)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Every pass timing the chain recorded, in its order.
    pub fn pass_timings(&self) -> Result<Vec<PassTiming>> {
        let handle = self.core.get()?;
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.post_process_chain_get_pass_timing_count)(handle, &mut count)
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
                (self.native.engine.post_process_chain_get_pass_timing)(
                    handle,
                    position,
                    &mut timing,
                )
            })?;
            let name = copy_text(&self.native, |api, destination, capacity, out_bytes| {
                // SAFETY: the destination holds `capacity` writable bytes.
                unsafe {
                    (api.post_process_chain_copy_pass_timing_name)(
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

    /// The chain's render-target pool, borrowed.
    ///
    /// A counted borrow upstream: destroying the chain is refused while the
    /// pool handle is outstanding, which the Rust lifetime makes unwritable.
    pub fn target_pool(&self) -> Result<RenderTargetPoolView<'_>> {
        let handle = self.core.get()?;
        let mut pool = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.post_process_chain_get_target_pool)(handle, &mut pool)
        })?;
        Ok(RenderTargetPoolView {
            pool: RenderTargetPool {
                core: Arc::new(EngineHandle {
                    native: Arc::clone(&self.native),
                    handle: Mutex::new(pool),
                    destroy: self.native.engine.render_target_pool_destroy,
                    released: "the render-target pool borrow has been released",
                }),
                native: Arc::clone(&self.native),
                device: self.device.clone(),
            },
            owner: PhantomData,
        })
    }

    /// Releases the chain now rather than at drop.
    pub fn release(&mut self) -> Result<()> {
        let result = self.core.release();
        self.borrowed.clear();
        self.owned_effects.clear();
        result
    }
}

impl Drop for PostProcessChain {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// A pool of reusable render targets.
pub struct RenderTargetPool {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
    device: GraphicsDevice,
}

/// A chain's own pool, borrowed for as long as the chain lives.
pub struct RenderTargetPoolView<'chain> {
    pool: RenderTargetPool,
    owner: PhantomData<&'chain ()>,
}

impl RenderTargetPoolView<'_> {
    /// The pool itself.
    #[must_use]
    pub const fn pool(&self) -> &RenderTargetPool {
        &self.pool
    }
}

impl RenderTargetPool {
    /// The number of targets the pool currently holds.
    pub fn target_count(&self) -> Result<u64> {
        let handle = self.core.get()?;
        let mut value = 0_u64;
        // SAFETY: the handle is live and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.render_target_pool_get_target_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// The estimated bytes the pool's targets hold.
    pub fn estimated_bytes(&self) -> Result<u64> {
        let handle = self.core.get()?;
        let mut value = 0_u64;
        // SAFETY: the handle is live and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.render_target_pool_get_estimated_bytes)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Releases every target the pool holds.
    pub fn reset(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is live.
        self.native
            .check(unsafe { (self.native.engine.render_target_pool_reset)(handle) })
    }

    /// Borrows a target of the requested shape from the pool.
    /// `slot` distinguishes two targets of the same shape, so a pass that needs
    /// two intermediates of one size asks for slot 0 and slot 1 rather than
    /// getting the same texture twice.
    pub fn acquire(
        &self,
        width: i32,
        height: i32,
        format: SurfaceFormat,
        depth: DepthFormat,
        slot: i32,
    ) -> Result<BorrowedRenderTarget<'_>> {
        let handle = self.core.get()?;
        let mut target = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is live and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.render_target_pool_acquire)(
                handle,
                width,
                height,
                format as u32,
                depth as u32,
                slot,
                &mut target,
            )
        })?;
        BorrowedRenderTarget::new(&self.native, &self.device, target)
    }
}

/// CNA's size-then-copy text protocol over the engine table.
fn copy_text(
    native: &Arc<Native>,
    mut route: impl FnMut(
        &crate::native::engine::EngineApi,
        *mut c_char,
        u64,
        *mut u64,
    ) -> sys::CNA_Result,
) -> Result<String> {
    let api = &native.engine;
    let mut required = 0_u64;
    let probe = route(api, core::ptr::null_mut(), 0, &mut required);
    if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
        native.check(probe)?;
    }
    let capacity =
        usize::try_from(required).map_err(|_| CnaError::InvalidInput("CNA text is too large"))?;
    if capacity == 0 {
        return Ok(String::new());
    }
    let mut buffer = vec![0_u8; capacity];
    let mut written = 0_u64;
    native.check(route(
        api,
        buffer.as_mut_ptr().cast::<c_char>(),
        required,
        &mut written,
    ))?;
    let written =
        usize::try_from(written).map_err(|_| CnaError::InvalidInput("CNA text is too large"))?;
    buffer.truncate(written.min(capacity));
    while buffer.last() == Some(&0) {
        buffer.pop();
    }
    String::from_utf8(buffer).map_err(|_| CnaError::InvalidInput("CNA text is not valid UTF-8"))
}

fn string_view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast::<c_char>(),
        byte_length: value.len() as u64,
    }
}

impl RenderTargetPool {
    /// Creates a pool of reusable render targets on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.render_target_pool_create)(device.handle()?, &mut handle)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.render_target_pool_destroy,
            released: "the render-target pool has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
            device: device.clone(),
        })
    }
}

impl Drop for RenderTargetPool {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// A tonemapping post-process pass.
///
/// Wraps a [`PostProcessPass`] rather than replacing it: upstream's concrete
/// passes are the same handle type driven through the same shared operations,
/// so `apply`, `name` and `is_supported` come from the pass itself and only the
/// tonemapping knobs live here.
pub struct TonemapPass {
    pass: PostProcessPass,
}

impl TonemapPass {
    /// Creates a tonemapping pass on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native
            .check(unsafe { (native.engine.tonemap_pass_create)(device.handle()?, &mut handle) })?;
        Ok(Self {
            pass: PostProcessPass::adopt(native, device, handle, None),
        })
    }

    /// The pass itself, for the operations every pass shares.
    #[must_use]
    pub const fn pass(&self) -> &PostProcessPass {
        &self.pass
    }

    /// Hands the pass over, for adding to a chain.
    #[must_use]
    pub fn into_pass(self) -> PostProcessPass {
        self.pass
    }

    /// The tonemapping operator.
    pub fn mode(&self) -> Result<TonemappingMode> {
        let handle = self.pass.core.get()?;
        let mut value: sys::CNA_TonemappingMode = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.pass
            .native
            .check(unsafe { (self.pass.native.engine.tonemap_pass_get_mode)(handle, &mut value) })?;
        TonemappingMode::from_native(value)
            .ok_or(CnaError::InvalidInput("native tonemapping mode is unknown"))
    }

    /// Sets the tonemapping operator.
    pub fn set_mode(&self, value: TonemappingMode) -> Result<()> {
        let handle = self.pass.core.get()?;
        // SAFETY: the handle is owned and the identity is canonical.
        self.pass.native.check(unsafe {
            (self.pass.native.engine.tonemap_pass_set_mode)(handle, value.to_native())
        })
    }

    /// Whether the pass dithers its output to hide banding.
    pub fn is_deband_enabled(&self) -> Result<bool> {
        let handle = self.pass.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.pass.native.check(unsafe {
            (self.pass.native.engine.tonemap_pass_is_deband_enabled)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Turns output dithering on or off.
    pub fn set_deband_enabled(&self, value: bool) -> Result<()> {
        let handle = self.pass.core.get()?;
        // SAFETY: the handle is owned and the flag is a canonical boolean.
        self.pass.native.check(unsafe {
            (self.pass.native.engine.tonemap_pass_set_deband_enabled)(handle, u8::from(value))
        })
    }

    /// The exposure multiplier.
    pub fn exposure(&self) -> Result<f32> {
        self.scalar(self.pass.native.engine.tonemap_pass_get_exposure)
    }

    /// Sets the exposure multiplier.
    pub fn set_exposure(&self, value: f32) -> Result<()> {
        self.set_scalar(self.pass.native.engine.tonemap_pass_set_exposure, value)
    }

    /// The gamma the pass encodes with.
    pub fn gamma(&self) -> Result<f32> {
        self.scalar(self.pass.native.engine.tonemap_pass_get_gamma)
    }

    /// Sets the gamma the pass encodes with.
    pub fn set_gamma(&self, value: f32) -> Result<()> {
        self.set_scalar(self.pass.native.engine.tonemap_pass_set_gamma, value)
    }

    /// How strongly the output is dithered.
    pub fn deband_strength(&self) -> Result<f32> {
        self.scalar(self.pass.native.engine.tonemap_pass_get_deband_strength)
    }

    /// Sets how strongly the output is dithered.
    pub fn set_deband_strength(&self, value: f32) -> Result<()> {
        self.set_scalar(self.pass.native.engine.tonemap_pass_set_deband_strength, value)
    }

    fn scalar(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_PostProcessPassHandle,
            *mut f32,
        ) -> sys::CNA_Result,
    ) -> Result<f32> {
        let handle = self.pass.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.pass.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value)
    }

    fn set_scalar(
        &self,
        route: unsafe extern "C" fn(sys::CNA_PostProcessPassHandle, f32) -> sys::CNA_Result,
        value: f32,
    ) -> Result<()> {
        let handle = self.pass.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.pass.native.check(unsafe { route(handle, value) })
    }

    /// The tonemapped value of one scene-linear channel.
    ///
    /// A pure function of its arguments upstream, which is what makes it worth
    /// binding on its own: a caller can check what the operator will do to a
    /// value without rendering anything, and a test can assert the curve rather
    /// than the fact that a pass exists.
    pub fn tonemap_channel(
        mode: TonemappingMode,
        value: f32,
        exposure: f32,
        gamma: f32,
    ) -> Result<f32> {
        let native = Native::process()?;
        let mut out = 0.0_f32;
        // SAFETY: every input is by value and the output is a live local.
        native.check(unsafe {
            (native.engine.tonemap_pass_tonemap_channel)(
                mode.to_native(),
                value,
                exposure,
                gamma,
                &mut out,
            )
        })?;
        Ok(out)
    }
}

/// An FXAA post-process pass.
pub struct FxaaPass {
    pass: PostProcessPass,
}

impl FxaaPass {
    /// Creates an FXAA pass on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe { (native.engine.fxaa_pass_create)(device.handle()?, &mut handle) })?;
        Ok(Self {
            pass: PostProcessPass::adopt(native, device, handle, None),
        })
    }

    /// The pass itself, for the operations every pass shares.
    #[must_use]
    pub const fn pass(&self) -> &PostProcessPass {
        &self.pass
    }

    /// Hands the pass over, for adding to a chain.
    #[must_use]
    pub fn into_pass(self) -> PostProcessPass {
        self.pass
    }

    /// The luminance difference at which the filter starts working.
    pub fn edge_threshold(&self) -> Result<f32> {
        let handle = self.pass.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.pass.native.check(unsafe {
            (self.pass.native.engine.fxaa_pass_get_edge_threshold)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Sets the luminance difference at which the filter starts working.
    pub fn set_edge_threshold(&self, value: f32) -> Result<()> {
        let handle = self.pass.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.pass.native.check(unsafe {
            (self.pass.native.engine.fxaa_pass_set_edge_threshold)(handle, value)
        })
    }

    /// The edge threshold a render-quality preset asks for.
    pub fn edge_threshold_for_quality(quality: RenderQuality) -> Result<f32> {
        let native = Native::process()?;
        let mut value = 0.0_f32;
        // SAFETY: the identity is canonical and the output is a live local.
        native.check(unsafe {
            (native.engine.fxaa_pass_edge_threshold_for_quality)(quality.to_native(), &mut value)
        })?;
        Ok(value)
    }

    /// The pass's own fragment shader, as the engine compiles it.
    pub fn fragment_glsl() -> Result<String> {
        let native = Native::process()?;
        copy_text(&native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe { (api.fxaa_pass_copy_fragment_glsl)(destination, capacity, out_bytes) }
        })
    }
}

/// A non-blocking GPU timer.
///
/// `OWNED`. Creation succeeds where the renderer has no timer query at all, so
/// [`GpuTimer::is_supported`] and [`GpuTimer::unsupported_reason`] are the
/// questions to ask -- and an unsupported timer's `begin` and `end` do nothing
/// rather than failing, which is why a caller that reads success as evidence
/// would measure nothing and never find out.
pub struct GpuTimer {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl GpuTimer {
    /// Creates a GPU timer for a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe { (native.engine.gpu_timer_create)(device.handle()?, &mut handle) })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.gpu_timer_destroy,
            released: "the GPU timer has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// Whether the renderer supplied a timer query.
    pub fn is_supported(&self) -> Result<bool> {
        self.flag(self.native.engine.gpu_timer_is_supported)
    }

    /// Why the timer is unsupported; empty when it is supported.
    pub fn unsupported_reason(&self) -> Result<String> {
        let handle = self.core.get()?;
        copy_text(&self.native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe {
                (api.gpu_timer_copy_unsupported_reason)(handle, destination, capacity, out_bytes)
            }
        })
    }

    /// Opens the timed range, or does nothing when unsupported or already open.
    pub fn begin(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.gpu_timer_begin)(handle) })
    }

    /// Closes the timed range, or does nothing when unsupported or not open.
    pub fn end(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.gpu_timer_end)(handle) })
    }

    /// Whether a timed range is currently open.
    pub fn is_open(&self) -> Result<bool> {
        self.flag(self.native.engine.gpu_timer_is_open)
    }

    /// Whether the last closed range can be collected without blocking.
    pub fn is_result_available(&self) -> Result<bool> {
        self.flag(self.native.engine.gpu_timer_is_result_available)
    }

    /// Collects a finished result without blocking.
    ///
    /// Answers `true` only when a *new* result was collected, which is what
    /// makes polling in a loop terminate rather than spin.
    pub fn poll(&self) -> Result<bool> {
        self.flag(self.native.engine.gpu_timer_poll)
    }

    /// The most recently collected GPU time, or zero before the first result.
    pub fn last_milliseconds(&self) -> Result<f64> {
        let handle = self.core.get()?;
        let mut value = 0.0_f64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.gpu_timer_get_last_milliseconds)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// How many results have been collected.
    pub fn sample_count(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.gpu_timer_get_sample_count)(handle, &mut value) })?;
        Ok(value)
    }

    /// Releases the timer now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }

    fn flag(
        &self,
        route: unsafe extern "C" fn(sys::CNA_GpuTimerHandle, *mut sys::CNA_Bool) -> sys::CNA_Result,
    ) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value != 0)
    }
}

impl Drop for GpuTimer {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// What an emitter throws, how fast, and what happens to it afterwards.
///
/// Assigned as given: no field is clamped or refused on the way in. An emission
/// rate the capacity cannot sustain is accepted and then *reported* by
/// [`ParticleSystem::is_emission_rate_clamped`], so the settings read back
/// exactly as they were written.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ParticleEmitterSettings {
    /// Where particles are born, in world space.
    pub position: Vector3,
    /// The centre of the emission cone; normalised internally.
    pub direction: Vector3,
    /// Constant acceleration, in units per second squared.
    pub gravity: Vector3,
    /// Colour at birth, unclamped so it can be an HDR emitter.
    pub start_color: Vector4,
    /// Colour at death.
    pub end_color: Vector4,
    /// The cone's half angle in radians; zero emits a line, pi a full sphere.
    pub cone_angle: f32,
    /// How fast a particle leaves, in units per second.
    pub speed: f32,
    /// How much that speed varies, as a fraction of it.
    pub speed_variance: f32,
    /// How long a particle lives, in seconds.
    pub lifetime: f32,
    /// How much that lifetime varies, as a fraction of it.
    pub lifetime_variance: f32,
    /// Linear drag per second; zero is a vacuum.
    pub drag: f32,
    /// How many particles are born per second.
    pub emission_rate: f32,
    /// A particle's size at birth, in world units.
    pub start_size: f32,
    /// Its size at death.
    pub end_size: f32,
}

impl ParticleEmitterSettings {
    /// CNA's own defaults, asked of the library rather than restated here.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_ParticleEmitterSettings::default();
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.engine.particle_emitter_settings_init)(&mut value) })?;
        Ok(Self::from_native(value))
    }

    fn from_native(value: sys::CNA_ParticleEmitterSettings) -> Self {
        Self {
            position: from_native_vector3(value.position),
            direction: from_native_vector3(value.direction),
            gravity: from_native_vector3(value.gravity),
            start_color: from_native_vector4(value.start_color),
            end_color: from_native_vector4(value.end_color),
            cone_angle: value.cone_angle,
            speed: value.speed,
            speed_variance: value.speed_variance,
            lifetime: value.lifetime,
            lifetime_variance: value.lifetime_variance,
            drag: value.drag,
            emission_rate: value.emission_rate,
            start_size: value.start_size,
            end_size: value.end_size,
        }
    }

    fn to_native(self) -> sys::CNA_ParticleEmitterSettings {
        sys::CNA_ParticleEmitterSettings {
            struct_size: core::mem::size_of::<sys::CNA_ParticleEmitterSettings>() as u32,
            struct_version: 1,
            position: native_vector3(self.position),
            direction: native_vector3(self.direction),
            gravity: native_vector3(self.gravity),
            start_color: native_vector4(self.start_color),
            end_color: native_vector4(self.end_color),
            cone_angle: self.cone_angle,
            speed: self.speed,
            speed_variance: self.speed_variance,
            lifetime: self.lifetime,
            lifetime_variance: self.lifetime_variance,
            drag: self.drag,
            emission_rate: self.emission_rate,
            start_size: self.start_size,
            end_size: self.end_size,
        }
    }
}

/// One particle, in the layout both the compute shader and the CPU simulation use.
///
/// The fourth component of `position` and `velocity` is padding `std430`
/// requires, not a `w` anything reads, so it is not exposed. `state` carries
/// age, lifetime, the seed the last spawn used, and how many times the slot has
/// respawned, which are four different things and are named as such.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct Particle {
    /// Position in world space.
    pub position: Vector3,
    /// Velocity in world space.
    pub velocity: Vector3,
    /// How long this slot's current particle has lived, in seconds.
    pub age: f32,
    /// How long it will live.
    pub lifetime: f32,
    /// The seed its last spawn used.
    pub seed: f32,
    /// How many times the slot has respawned.
    pub respawn_count: f32,
}

impl Particle {
    /// CNA's own defaults: at the origin, at rest, aged zero with a lifetime of one.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_Particle::default();
        // SAFETY: the structure is a caller-owned output.
        native.check(unsafe { (native.engine.particle_init)(&mut value) })?;
        Ok(Self::from_native(value))
    }

    /// Advances one particle by one step, exactly as either simulation does.
    ///
    /// The pure form of the integrator, so a caller can predict what a system
    /// will do to a slot without running one -- and a test can assert the
    /// arithmetic rather than the fact that a call returned.
    pub fn step(
        self,
        index: i32,
        settings: ParticleEmitterSettings,
        elapsed_seconds: f32,
    ) -> Result<Self> {
        let native = Native::process()?;
        let mut particle = self.to_native();
        let settings = settings.to_native();
        // SAFETY: the particle is a live local updated in place and the
        // settings are borrowed for the call.
        native.check(unsafe {
            (native.engine.particle_system_step)(
                &mut particle,
                index,
                &settings,
                elapsed_seconds,
            )
        })?;
        Ok(Self::from_native(particle))
    }

    fn from_native(value: sys::CNA_Particle) -> Self {
        Self {
            position: Vector3::from_x_and_y_and_z(
                value.position.x,
                value.position.y,
                value.position.z,
            ),
            velocity: Vector3::from_x_and_y_and_z(
                value.velocity.x,
                value.velocity.y,
                value.velocity.z,
            ),
            age: value.state.x,
            lifetime: value.state.y,
            seed: value.state.z,
            respawn_count: value.state.w,
        }
    }

    fn to_native(self) -> sys::CNA_Particle {
        sys::CNA_Particle {
            position: sys::CNA_Vector4 {
                x: self.position.X,
                y: self.position.Y,
                z: self.position.Z,
                w: 0.0,
            },
            velocity: sys::CNA_Vector4 {
                x: self.velocity.X,
                y: self.velocity.Y,
                z: self.velocity.Z,
                w: 0.0,
            },
            state: sys::CNA_Vector4 {
                x: self.age,
                y: self.lifetime,
                z: self.seed,
                w: self.respawn_count,
            },
        }
    }
}

/// An emitter, a simulation and a draw.
///
/// The simulation runs on the GPU where the device has compute and on the CPU
/// where it does not, and upstream states these are one simulation rather than
/// two implementations of one idea. [`ParticleSystem::uses_compute`] says which
/// path the last update took, and
/// [`ParticleSystem::set_simulation_on_cpu`] forces the other one -- which is
/// what makes the claim checkable rather than a promise.
pub struct ParticleSystem {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
    /// The depth image particles fade against. CNA keeps a raw pointer to it.
    depth: Option<Texture2D>,
}

impl ParticleSystem {
    /// Creates a system at CNA's default capacity.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.particle_system_create)(device.handle()?, &mut handle)
        })?;
        Ok(Self::adopt(native, device, handle))
    }

    /// Creates a system with a chosen number of slots.
    pub fn with_capacity(device: &GraphicsDevice, capacity: i32) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.particle_system_create_with_capacity)(
                device.handle()?,
                capacity,
                &mut handle,
            )
        })?;
        Ok(Self::adopt(native, device, handle))
    }

    fn adopt(native: &Arc<Native>, device: &GraphicsDevice, handle: sys::CNA_Handle) -> Self {
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.particle_system_destroy,
            released: "the particle system has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Self {
            core,
            native: Arc::clone(native),
            depth: None,
        }
    }

    /// The emitter settings, exactly as they were set.
    ///
    /// This is the one engine getter that validates its *output* structure on
    /// the way in: upstream refuses a destination whose `struct_size` and
    /// `struct_version` are not filled, so a zeroed one is rejected as
    /// malformed rather than being filled in. The versioning is written here
    /// before the call for exactly that reason.
    pub fn settings(&self) -> Result<ParticleEmitterSettings> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_ParticleEmitterSettings {
            struct_size: core::mem::size_of::<sys::CNA_ParticleEmitterSettings>() as u32,
            struct_version: 1,
            ..sys::CNA_ParticleEmitterSettings::default()
        };
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.particle_system_get_settings)(handle, &mut value)
        })?;
        Ok(ParticleEmitterSettings::from_native(value))
    }

    /// Replaces the emitter settings.
    pub fn set_settings(&self, settings: ParticleEmitterSettings) -> Result<()> {
        let handle = self.core.get()?;
        let value = settings.to_native();
        // SAFETY: the handle is owned and the structure is borrowed for the call.
        self.native
            .check(unsafe { (self.native.engine.particle_system_set_settings)(handle, &value) })
    }

    /// How many slots the system allocated.
    pub fn capacity(&self) -> Result<i32> {
        self.count(self.native.engine.particle_system_get_capacity)
    }

    /// How many slots are actually in use.
    pub fn active_count(&self) -> Result<i32> {
        self.count(self.native.engine.particle_system_get_active_count)
    }

    /// Whether the emission rate exceeds what the capacity can sustain.
    pub fn is_emission_rate_clamped(&self) -> Result<bool> {
        self.flag(self.native.engine.particle_system_is_emission_rate_clamped)
    }

    /// Advances the simulation.
    pub fn update(&self, elapsed_seconds: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.native.check(unsafe {
            (self.native.engine.particle_system_update)(handle, elapsed_seconds)
        })
    }

    /// Returns every slot to its unspawned state.
    pub fn reset(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.particle_system_reset)(handle) })
    }

    /// Draws every active particle as one instanced draw.
    pub fn draw(&self, view: Matrix, projection: Matrix, texture: &Texture2D) -> Result<()> {
        let handle = self.core.get()?;
        let view = native_matrix(view);
        let projection = native_matrix(projection);
        // SAFETY: the handle is owned, both matrices are borrowed for the call,
        // and the texture handle is live.
        self.native.check(unsafe {
            (self.native.engine.particle_system_draw)(
                handle,
                &view,
                &projection,
                texture.handle()?,
            )
        })
    }

    /// Copies the particles out, whichever path is simulating them.
    pub fn particles(&self) -> Result<Vec<Particle>> {
        let handle = self.core.get()?;
        let capacity = usize::try_from(self.capacity()?.max(0))
            .map_err(|_| CnaError::InvalidInput("the particle capacity does not fit in memory"))?;
        let mut buffer = vec![sys::CNA_Particle::default(); capacity];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `capacity`
        // writable particles, which is the count passed alongside it.
        self.native.check(unsafe {
            (self.native.engine.particle_system_copy_particles_ext)(
                handle,
                buffer.as_mut_ptr(),
                capacity as u64,
                &mut count,
            )
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported more particles than fit in memory"))?;
        Ok(buffer
            .into_iter()
            .take(count.min(capacity))
            .map(Particle::from_native)
            .collect())
    }

    /// Whether the last [`ParticleSystem::update`] ran on the GPU.
    pub fn uses_compute(&self) -> Result<bool> {
        self.flag(self.native.engine.particle_system_uses_compute)
    }

    /// Whether the CPU path has been forced.
    pub fn is_simulation_on_cpu(&self) -> Result<bool> {
        self.flag(self.native.engine.particle_system_is_simulation_on_cpu_ext)
    }

    /// Forces the simulation onto the CPU, or lets it use the GPU again.
    pub fn set_simulation_on_cpu(&self, forced: bool) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the flag is a canonical boolean.
        self.native.check(unsafe {
            (self.native.engine.particle_system_set_simulation_on_cpu_ext)(
                handle,
                u8::from(forced),
            )
        })
    }

    /// Why the GPU path was not taken; empty when it was.
    pub fn unsupported_reason(&self) -> Result<String> {
        let handle = self.core.get()?;
        copy_text(&self.native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe {
                (api.particle_system_copy_unsupported_reason)(
                    handle,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
    }

    /// How sharply particles fade into the depth behind them.
    pub fn softness(&self) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.particle_system_get_softness_ext)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Sets how sharply particles fade into the depth behind them.
    pub fn set_softness(&self, value: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.native
            .check(unsafe { (self.native.engine.particle_system_set_softness_ext)(handle, value) })
    }

    /// Supplies the depth image particles fade against.
    ///
    /// CNA keeps a raw pointer to it and retains nothing, so the system takes
    /// the texture and holds it for exactly as long as CNA points at it.
    pub fn set_depth_input(&mut self, depth: Option<Texture2D>, far_plane: f32) -> Result<()> {
        let handle = self.core.get()?;
        let texture_handle = match depth.as_ref() {
            Some(texture) => texture.handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        // SAFETY: the handle is owned and the texture handle is live for the call.
        self.native.check(unsafe {
            (self.native.engine.particle_system_set_depth_input_ext)(
                handle,
                texture_handle,
                far_plane,
            )
        })?;
        self.depth = depth;
        Ok(())
    }

    /// Releases the system now rather than at drop.
    pub fn release(&mut self) -> Result<()> {
        let result = self.core.release();
        self.depth = None;
        result
    }

    /// The same pseudo-random value the shader's hash returns for a seed.
    ///
    /// Bit-identical in GLSL and C++ upstream, which is what makes the CPU and
    /// GPU paths one simulation rather than two.
    pub fn random(seed: u32) -> Result<f32> {
        let native = Native::process()?;
        let mut value = 0.0_f32;
        // SAFETY: the seed is by value and the output is a live local.
        native.check(unsafe { (native.engine.particle_system_random)(seed, &mut value) })?;
        Ok(value)
    }

    /// The GLSL a vertex shader includes to read a particle.
    pub fn particle_lookup_glsl() -> Result<String> {
        let native = Native::process()?;
        copy_text(&native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe { (api.particle_system_copy_particle_lookup_glsl)(destination, capacity, out_bytes) }
        })
    }

    fn flag(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_ParticleSystemHandle,
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
        route: unsafe extern "C" fn(sys::CNA_ParticleSystemHandle, *mut i32) -> sys::CNA_Result,
    ) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value)
    }
}

impl Drop for ParticleSystem {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

fn native_vector4(value: Vector4) -> sys::CNA_Vector4 {
    sys::CNA_Vector4 {
        x: value.X,
        y: value.Y,
        z: value.Z,
        w: value.W,
    }
}

fn from_native_vector4(value: sys::CNA_Vector4) -> Vector4 {
    Vector4::from_x_and_y_and_z_and_w(value.x, value.y, value.z, value.w)
}

/// What a memory barrier orders against later commands.
///
/// A bit set rather than an enum: upstream folds several orderings into one
/// mask, and `CNA_GRAPHICS_MEMORY_BARRIER_ALL` is exactly the union of the
/// rest, so a Rust value that could only hold one would not express what the
/// route takes.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct MemoryBarrier(sys::CNA_GraphicsMemoryBarrier);

impl MemoryBarrier {
    /// No ordering at all.
    pub const NONE: Self = Self(sys::CNA_GRAPHICS_MEMORY_BARRIER_NONE);
    /// Orders vertex-attribute array reads.
    pub const VERTEX_ATTRIB_ARRAY: Self =
        Self(sys::CNA_GRAPHICS_MEMORY_BARRIER_VERTEX_ATTRIB_ARRAY);
    /// Orders element-array reads.
    pub const ELEMENT_ARRAY: Self = Self(sys::CNA_GRAPHICS_MEMORY_BARRIER_ELEMENT_ARRAY);
    /// Orders uniform reads.
    pub const UNIFORM: Self = Self(sys::CNA_GRAPHICS_MEMORY_BARRIER_UNIFORM);
    /// Orders texture fetches.
    pub const TEXTURE_FETCH: Self = Self(sys::CNA_GRAPHICS_MEMORY_BARRIER_TEXTURE_FETCH);
    /// Orders shader image accesses.
    pub const SHADER_IMAGE_ACCESS: Self =
        Self(sys::CNA_GRAPHICS_MEMORY_BARRIER_SHADER_IMAGE_ACCESS);
    /// Orders shader storage-buffer accesses.
    pub const SHADER_STORAGE: Self = Self(sys::CNA_GRAPHICS_MEMORY_BARRIER_SHADER_STORAGE);
    /// Orders buffer updates.
    pub const BUFFER_UPDATE: Self = Self(sys::CNA_GRAPHICS_MEMORY_BARRIER_BUFFER_UPDATE);
    /// Orders framebuffer accesses.
    pub const FRAMEBUFFER: Self = Self(sys::CNA_GRAPHICS_MEMORY_BARRIER_FRAMEBUFFER);
    /// Orders indirect-command reads.
    pub const INDIRECT_COMMAND: Self = Self(sys::CNA_GRAPHICS_MEMORY_BARRIER_INDIRECT_COMMAND);
    /// Every bit above, folded together.
    pub const ALL: Self = Self(sys::CNA_GRAPHICS_MEMORY_BARRIER_ALL);

    /// Whether this mask contains every bit of another.
    ///
    /// Asked of CNA rather than computed here: the mask is the ABI's, and a
    /// Rust reimplementation of the test would agree right up until upstream
    /// added a bit.
    pub fn contains(self, bits: Self) -> Result<bool> {
        let native = Native::process()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: both masks are by value and the output is a live local.
        native.check(unsafe {
            (native.engine.graphics_memory_barrier_has)(self.0, bits.0, &mut value)
        })?;
        Ok(value != 0)
    }
}

impl core::ops::BitOr for MemoryBarrier {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// How a compute shader may touch a bound image.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ImageAccess {
    /// The shader only reads.
    ReadOnly,
    /// The shader only writes.
    WriteOnly,
    /// The shader does both.
    ReadWrite,
}

impl ImageAccess {
    const fn to_native(self) -> sys::CNA_GraphicsImageAccess {
        match self {
            Self::ReadOnly => sys::CNA_GRAPHICS_IMAGE_ACCESS_READ_ONLY,
            Self::WriteOnly => sys::CNA_GRAPHICS_IMAGE_ACCESS_WRITE_ONLY,
            Self::ReadWrite => sys::CNA_GRAPHICS_IMAGE_ACCESS_READ_WRITE,
        }
    }
}

/// A shader-visible buffer of bytes.
///
/// `OWNED`. Created either as a flat byte range or as a count of fixed-size
/// elements; the second form is the C shape of upstream's `StorageBufferT<T>`,
/// and it remembers both numbers so a mismatched element size is refused rather
/// than silently reinterpreted.
pub struct StorageBuffer {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl StorageBuffer {
    /// Creates a buffer of a given size in bytes.
    pub fn with_byte_size(device: &GraphicsDevice, byte_size: u64) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.storage_buffer_create)(device.handle()?, byte_size, &mut handle)
        })?;
        Ok(Self::adopt(native, device, handle))
    }

    /// Creates a buffer sized as a count of fixed-size elements.
    ///
    /// `T` must be a plain value: bytes are what reach the GPU, which is the
    /// same requirement upstream's template asserts.
    pub fn with_elements<T: Copy>(device: &GraphicsDevice, element_count: u64) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.storage_buffer_create_typed)(
                device.handle()?,
                element_count,
                core::mem::size_of::<T>() as u64,
                &mut handle,
            )
        })?;
        Ok(Self::adopt(native, device, handle))
    }

    fn adopt(native: &Arc<Native>, device: &GraphicsDevice, handle: sys::CNA_Handle) -> Self {
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.storage_buffer_destroy,
            released: "the storage buffer has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Self {
            core,
            native: Arc::clone(native),
        }
    }

    /// The buffer's size in bytes.
    pub fn byte_size(&self) -> Result<u64> {
        self.size(self.native.engine.storage_buffer_get_byte_size)
    }

    /// How many elements the buffer was created to hold; zero for a byte buffer.
    pub fn element_count(&self) -> Result<u64> {
        self.size(self.native.engine.storage_buffer_get_element_count)
    }

    /// The element size the buffer was created with; zero for a byte buffer.
    pub fn element_byte_size(&self) -> Result<u64> {
        self.size(self.native.engine.storage_buffer_get_element_byte_size)
    }

    /// Uploads raw bytes.
    pub fn set_bytes(&self, data: &[u8]) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the slice is borrowed for the call
        // with its own length passed alongside it.
        self.native.check(unsafe {
            (self.native.engine.storage_buffer_set_bytes)(
                handle,
                data.as_ptr().cast::<c_void>(),
                data.len() as u64,
            )
        })
    }

    /// Reads raw bytes back.
    pub fn get_bytes(&self, destination: &mut [u8]) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the destination holds its own length
        // in writable bytes, which is the count passed alongside it.
        self.native.check(unsafe {
            (self.native.engine.storage_buffer_get_bytes)(
                handle,
                destination.as_mut_ptr().cast::<c_void>(),
                destination.len() as u64,
            )
        })
    }

    /// Uploads elements, refusing more than the buffer holds.
    pub fn set_elements<T: Copy>(&self, data: &[T]) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the slice is borrowed for the call
        // with its element count and element size passed alongside it.
        self.native.check(unsafe {
            (self.native.engine.storage_buffer_set_elements)(
                handle,
                data.as_ptr().cast::<c_void>(),
                data.len() as u64,
                core::mem::size_of::<T>() as u64,
            )
        })
    }

    /// Reads the buffer's whole element range back.
    pub fn get_elements<T: Copy>(&self, destination: &mut [T]) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the destination holds its own element
        // count of writable elements of the size passed alongside it.
        self.native.check(unsafe {
            (self.native.engine.storage_buffer_get_elements)(
                handle,
                destination.as_mut_ptr().cast::<c_void>(),
                destination.len() as u64,
                core::mem::size_of::<T>() as u64,
            )
        })
    }

    /// Releases the buffer now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }

    fn size(
        &self,
        route: unsafe extern "C" fn(sys::CNA_StorageBufferHandle, *mut u64) -> sys::CNA_Result,
    ) -> Result<u64> {
        let handle = self.core.get()?;
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value)
    }
}

impl Drop for StorageBuffer {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// A compute shader compiled from GLSL ES 3.10 source.
///
/// `OWNED`. Creation succeeds even where the source did not compile, so
/// [`ComputeShader::is_valid`] and [`ComputeShader::compile_error`] are the
/// questions to ask: a handle is not a program.
pub struct ComputeShader {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
    /// Buffers and textures bound to the shader. CNA keeps the binding, not the
    /// resource, so this value is what keeps them alive while it points at them.
    bound_buffers: Vec<Arc<StorageBuffer>>,
    bound_textures: Vec<Texture2D>,
}

impl ComputeShader {
    /// Compiles a compute shader from GLSL ES 3.10 source.
    pub fn new(device: &GraphicsDevice, source: &str) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        let view = string_view(source);
        // SAFETY: the device handle is live, `source` is borrowed for the call,
        // and the output is a live local.
        native.check(unsafe {
            (native.engine.compute_shader_create)(device.handle()?, view, &mut handle)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.compute_shader_destroy,
            released: "the compute shader has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
            bound_buffers: Vec::new(),
            bound_textures: Vec::new(),
        })
    }

    /// Whether the shader compiled.
    pub fn is_valid(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.compute_shader_is_valid)(handle, &mut value) })?;
        Ok(value != 0)
    }

    /// Why the shader did not compile; empty when it did.
    pub fn compile_error(&self) -> Result<String> {
        let handle = self.core.get()?;
        copy_text(&self.native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe { (api.compute_shader_copy_compile_error)(handle, destination, capacity, out_bytes) }
        })
    }

    /// Whether this renderer supports binding an image for shader read/write.
    pub fn is_image_binding_supported(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.compute_shader_is_image_binding_supported)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Sets a signed-integer uniform.
    pub fn set_uniform_int(&self, name: &str, value: i32) -> Result<()> {
        let handle = self.core.get()?;
        let view = string_view(name);
        // SAFETY: the handle is owned and `name` is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.compute_shader_set_uniform_int)(handle, view, value)
        })
    }

    /// Sets a floating-point uniform.
    pub fn set_uniform_float(&self, name: &str, value: f32) -> Result<()> {
        let handle = self.core.get()?;
        let view = string_view(name);
        // SAFETY: the handle is owned and `name` is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.compute_shader_set_uniform_float)(handle, view, value)
        })
    }

    /// Binds a storage buffer to a numbered binding point.
    ///
    /// CNA records the binding, not the resource, so the [`Arc`] is what keeps
    /// the buffer alive for as long as the shader points at it.
    pub fn bind_storage_buffer(&mut self, binding: i32, buffer: &Arc<StorageBuffer>) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: both handles are live; retention follows on success.
        self.native.check(unsafe {
            (self.native.engine.compute_shader_bind_storage_buffer)(
                handle,
                binding,
                buffer.core.get()?,
            )
        })?;
        self.bound_buffers.push(Arc::clone(buffer));
        Ok(())
    }

    /// Binds a texture to a numbered sampler unit.
    pub fn bind_texture(&mut self, unit: i32, sampler: &str, texture: Texture2D) -> Result<()> {
        let handle = self.core.get()?;
        let view = string_view(sampler);
        // SAFETY: the handle is owned, `sampler` is borrowed for the call, and
        // the texture handle is live; retention follows on success.
        self.native.check(unsafe {
            (self.native.engine.compute_shader_bind_texture)(
                handle,
                unit,
                view,
                texture.handle()?,
            )
        })?;
        self.bound_textures.push(texture);
        Ok(())
    }

    /// Binds a texture as a read/write image.
    pub fn bind_image(&mut self, unit: i32, texture: Texture2D, access: ImageAccess) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the texture handle is live;
        // retention follows on success.
        self.native.check(unsafe {
            (self.native.engine.compute_shader_bind_image)(
                handle,
                unit,
                texture.handle()?,
                access.to_native(),
            )
        })?;
        self.bound_textures.push(texture);
        Ok(())
    }

    /// Dispatches the shader over a group grid.
    pub fn dispatch(&self, groups_x: i32, groups_y: i32, groups_z: i32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the counts are by value.
        self.native.check(unsafe {
            (self.native.engine.compute_shader_dispatch)(handle, groups_x, groups_y, groups_z)
        })
    }

    /// Orders the given memory accesses against later commands.
    pub fn barrier(&self, bits: MemoryBarrier) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the mask is by value.
        self.native
            .check(unsafe { (self.native.engine.compute_shader_barrier)(handle, bits.0) })
    }

    /// Releases the shader now rather than at drop.
    pub fn release(&mut self) -> Result<()> {
        let result = self.core.release();
        self.bound_buffers.clear();
        self.bound_textures.clear();
        result
    }
}

impl Drop for ComputeShader {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// A deferred-decal projector.
///
/// `OWNED`. Projects a decal texture into whatever target is bound, reading the
/// depth and normal buffers a prepass produced. Those inputs are
/// `RETAINED_DEPENDENCY` on the same terms as the pipeline's: CNA keeps raw
/// pointers and retains nothing.
pub struct DecalPass {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
    depth: Option<Texture2D>,
    normals: Option<Texture2D>,
}

impl DecalPass {
    /// Creates a decal pass on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe { (native.engine.decal_pass_create)(device.handle()?, &mut handle) })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.decal_pass_destroy,
            released: "the decal pass has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
            depth: None,
            normals: None,
        })
    }

    /// Gives the pass the depth and normal buffers it projects against.
    pub fn set_prepass_inputs(
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
            (self.native.engine.decal_pass_set_prepass_inputs)(handle, depth_handle, normal_handle)
        })?;
        self.depth = depth;
        self.normals = normals;
        Ok(())
    }

    /// Sets the camera the decal pass unprojects with.
    ///
    /// A far plane that is not positive is *ignored* upstream, because the
    /// unprojection divides by it. That is a silent no-op rather than a
    /// refusal, so it is stated here rather than left to be discovered.
    pub fn set_camera(&self, view: Matrix, projection: Matrix, far_plane: f32) -> Result<()> {
        let handle = self.core.get()?;
        let view = native_matrix(view);
        let projection = native_matrix(projection);
        // SAFETY: the handle is owned and both matrices are borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.decal_pass_set_camera)(handle, &view, &projection, far_plane)
        })
    }

    /// Projects one decal into the current target.
    pub fn draw(
        &self,
        decal: &Texture2D,
        decal_world: Matrix,
        width: i32,
        height: i32,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let world = native_matrix(decal_world);
        // SAFETY: the handle is owned, the texture handle is live and the
        // matrix is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.decal_pass_draw)(
                handle,
                decal.handle()?,
                &world,
                width,
                height,
            )
        })
    }

    /// Whether a point in the decal box's local space falls inside it.
    ///
    /// A pure function of the point, so a caller can reason about coverage
    /// without drawing -- and a test can assert the box's own boundary.
    pub fn is_inside_decal_box(local_position: Vector3) -> Result<bool> {
        let native = Native::process()?;
        let point = native_vector3(local_position);
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the point is borrowed for the call and the output is a live local.
        native.check(unsafe {
            (native.engine.decal_pass_is_inside_decal_box)(&point, &mut value)
        })?;
        Ok(value != 0)
    }

    /// How opaque the projection is.
    pub fn opacity(&self) -> Result<f32> {
        self.scalar(self.native.engine.decal_pass_get_opacity)
    }

    /// Sets how opaque the projection is.
    pub fn set_opacity(&self, value: f32) -> Result<()> {
        self.set_scalar(self.native.engine.decal_pass_set_opacity, value)
    }

    /// The steepest surface the decal will still project onto, in radians.
    pub fn max_slope_angle(&self) -> Result<f32> {
        self.scalar(self.native.engine.decal_pass_get_max_slope_angle)
    }

    /// Sets the steepest surface the decal will still project onto.
    pub fn set_max_slope_angle(&self, value: f32) -> Result<()> {
        self.set_scalar(self.native.engine.decal_pass_set_max_slope_angle, value)
    }

    /// The colour the decal is multiplied by.
    pub fn tint(&self) -> Result<Vector3> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.decal_pass_get_tint)(handle, &mut value) })?;
        Ok(from_native_vector3(value))
    }

    /// Sets the colour the decal is multiplied by.
    pub fn set_tint(&self, value: Vector3) -> Result<()> {
        let handle = self.core.get()?;
        let tint = native_vector3(value);
        // SAFETY: the handle is owned and the colour is borrowed for the call.
        self.native
            .check(unsafe { (self.native.engine.decal_pass_set_tint)(handle, &tint) })
    }

    /// Releases the pass now rather than at drop.
    pub fn release(&mut self) -> Result<()> {
        let result = self.core.release();
        self.depth = None;
        self.normals = None;
        result
    }

    fn scalar(
        &self,
        route: unsafe extern "C" fn(sys::CNA_DecalPassHandle, *mut f32) -> sys::CNA_Result,
    ) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value)
    }

    fn set_scalar(
        &self,
        route: unsafe extern "C" fn(sys::CNA_DecalPassHandle, f32) -> sys::CNA_Result,
        value: f32,
    ) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.native.check(unsafe { route(handle, value) })
    }
}

impl Drop for DecalPass {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// A cube map another engine object owns, viewed for a bounded borrow.
///
/// Like [`BorrowedRenderTarget`], the handle CNA publishes here is a new one
/// that aliases its owner and must be released. Leaking it keeps the owner
/// alive past its device, which CNA reports by refusing to destroy the game.
pub struct BorrowedTextureCube<'owner> {
    native: Arc<Native>,
    handle: sys::CNA_Handle,
    owner: PhantomData<&'owner ()>,
}

impl BorrowedTextureCube<'_> {
    /// The cube map's native size, read through the borrow.
    pub fn size(&self) -> Result<i32> {
        let mut info = sys::CNA_TextureCubeInfo {
            struct_size: core::mem::size_of::<sys::CNA_TextureCubeInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_TextureCubeInfo::default()
        };
        self.native.texture_cube_info(self.handle, &mut info)?;
        i32::try_from(info.size)
            .map_err(|_| CnaError::InvalidInput("cube texture size exceeds i32"))
    }
}

impl Drop for BorrowedTextureCube<'_> {
    fn drop(&mut self) {
        // SAFETY: the handle is this view's own, released exactly once. It
        // aliases the owner, so releasing it releases nothing but the view.
        let _ = self.native.destroy_texture_cube(self.handle);
    }
}

/// A texture another engine object owns, viewed for a bounded borrow.
///
/// The same shape as [`BorrowedTextureCube`]: the handle CNA publishes aliases
/// its owner and has to be released, or the owner outlives its own device.
pub struct BorrowedTexture2D<'owner> {
    native: Arc<Native>,
    handle: sys::CNA_Handle,
    owner: PhantomData<&'owner ()>,
}

impl BorrowedTexture2D<'_> {
    /// The texture's size, read through the borrow.
    pub fn size(&self) -> Result<(i32, i32)> {
        let mut info = sys::CNA_Texture2DInfo {
            struct_size: core::mem::size_of::<sys::CNA_Texture2DInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_Texture2DInfo::default()
        };
        self.native.texture_info(self.handle, &mut info)?;
        let width = i32::try_from(info.width)
            .map_err(|_| CnaError::InvalidInput("texture width exceeds i32"))?;
        let height = i32::try_from(info.height)
            .map_err(|_| CnaError::InvalidInput("texture height exceeds i32"))?;
        Ok((width, height))
    }
}

impl Drop for BorrowedTexture2D<'_> {
    fn drop(&mut self) {
        // SAFETY: the handle is this view's own, released exactly once.
        let _ = self.native.destroy_texture(self.handle);
    }
}

/// A volume texture another engine object owns, viewed for a bounded borrow.
pub struct BorrowedTexture3D<'owner> {
    native: Arc<Native>,
    handle: sys::CNA_Handle,
    owner: PhantomData<&'owner ()>,
}

impl Drop for BorrowedTexture3D<'_> {
    fn drop(&mut self) {
        // SAFETY: the handle is this view's own, released exactly once.
        let _ = self.native.destroy_texture3d(self.handle);
    }
}

/// CNAEXT's ASCII post-process effect, owned by an [`AsciiPass`].
pub struct BorrowedAsciiEffect<'owner> {
    native: Arc<Native>,
    handle: sys::CNA_AsciiPostProcessEffectHandle,
    owner: PhantomData<&'owner ()>,
}

impl BorrowedAsciiEffect<'_> {
    /// The character cell size in pixels.
    pub fn cell_size(&self) -> Result<(i32, i32)> {
        let mut width = 0_i32;
        let mut height = 0_i32;
        // SAFETY: the handle is this view's own and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.runtime.ascii_get_cell_size)(self.handle, &mut width, &mut height)
        })?;
        Ok((width, height))
    }

    /// The grid the last draw produced, in cells.
    pub fn last_grid_dimensions(&self) -> Result<(i32, i32)> {
        let mut columns = 0_i32;
        let mut rows = 0_i32;
        // SAFETY: the handle is this view's own and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.runtime.ascii_get_last_grid_dimensions)(
                self.handle,
                &mut columns,
                &mut rows,
            )
        })?;
        Ok((columns, rows))
    }
}

impl Drop for BorrowedAsciiEffect<'_> {
    fn drop(&mut self) {
        // SAFETY: the handle is this view's own, released exactly once, and
        // through the ASCII effect's own destroy rather than the generic one.
        let _ = unsafe { (self.native.runtime.ascii_effect_destroy)(self.handle) };
    }
}

/// A cube map the skybox refused to take over.
pub struct EnvironmentNotTransferred {
    /// The cube map, still owned by the caller.
    pub environment: TextureCube,
    /// Why the transfer was refused.
    pub error: CnaError,
}

impl core::fmt::Debug for EnvironmentNotTransferred {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("EnvironmentNotTransferred")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl core::fmt::Display for EnvironmentNotTransferred {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "the environment was not taken over: {}", self.error)
    }
}

impl std::error::Error for EnvironmentNotTransferred {}

/// A cube-map sky.
///
/// `OWNED`. Its environment is either `RETAINED_DEPENDENCY` -- borrowed by CNA
/// and kept alive here -- or handed over outright through
/// [`Skybox::set_owned_environment`], which is the consuming form.
pub struct Skybox {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
    /// The environment CNA borrows. `None` once one has been handed over, since
    /// the skybox owns that one itself.
    borrowed_environment: Option<TextureCube>,
}

impl Skybox {
    /// Creates a skybox with no environment yet.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        Self::create(device, sys::CNA_INVALID_HANDLE, None)
    }

    /// Creates a skybox over an environment CNA borrows.
    ///
    /// The cube map is taken rather than referenced: CNA keeps a raw pointer to
    /// it, so Rust is what guarantees it outlives the skybox.
    pub fn with_environment(device: &GraphicsDevice, environment: TextureCube) -> Result<Self> {
        let handle = environment.native_handle()?;
        Self::create(device, handle, Some(environment))
    }

    fn create(
        device: &GraphicsDevice,
        environment: sys::CNA_Handle,
        retained: Option<TextureCube>,
    ) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device and environment handles are live and the output is
        // a live local.
        native.check(unsafe {
            (native.engine.skybox_create)(device.handle()?, environment, &mut handle)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.skybox_destroy,
            released: "the skybox has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
            borrowed_environment: retained,
        })
    }

    /// Whether this renderer can draw a sky.
    pub fn is_supported(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.skybox_is_supported)(handle, &mut value) })?;
        Ok(value != 0)
    }

    /// Whether an environment is attached.
    ///
    /// Answered from CNA rather than from the Rust-side retention, because a
    /// handed-over environment is the skybox's and this value keeps nothing.
    ///
    /// The query is not free: upstream publishes a *new* handle aliasing the
    /// skybox, and one that is never released keeps the skybox alive past its
    /// device -- which CNA then refuses to shut the game down over. It is
    /// released here before answering.
    pub fn has_environment(&self) -> Result<bool> {
        Ok(self.environment()?.is_some())
    }

    /// The attached environment, borrowed for as long as the skybox lives.
    ///
    /// `None` when none is attached. The handle CNA publishes here aliases the
    /// skybox rather than the cube map, so releasing it releases nothing but
    /// the view -- which is what [`BorrowedTextureCube`] does on drop.
    pub fn environment(&self) -> Result<Option<BorrowedTextureCube<'_>>> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.skybox_get_environment)(handle, &mut value) })?;
        if value == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        Ok(Some(BorrowedTextureCube {
            native: Arc::clone(&self.native),
            handle: value,
            owner: PhantomData,
        }))
    }

    /// Attaches an environment CNA borrows, replacing any previous one.
    pub fn set_environment(&mut self, environment: Option<TextureCube>) -> Result<()> {
        let handle = self.core.get()?;
        let cube = match environment.as_ref() {
            Some(value) => value.native_handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        // SAFETY: the handle is owned and the cube handle is live for the call.
        self.native
            .check(unsafe { (self.native.engine.skybox_set_environment)(handle, cube) })?;
        self.borrowed_environment = environment;
        Ok(())
    }

    /// Attaches an environment and hands ownership of it to the skybox.
    ///
    /// The consuming form. On success the skybox owns the cube map and this
    /// value forgets its handle; on failure the cube map comes back untouched,
    /// because upstream releases the handle last and a refusal never leaves the
    /// caller holding nothing.
    pub fn set_owned_environment(
        &mut self,
        environment: TextureCube,
    ) -> std::result::Result<(), EnvironmentNotTransferred> {
        let handle = match self.core.get() {
            Ok(value) => value,
            Err(error) => return Err(EnvironmentNotTransferred { environment, error }),
        };
        let cube = match environment.native_handle() {
            Ok(value) => value,
            Err(error) => return Err(EnvironmentNotTransferred { environment, error }),
        };
        // SAFETY: both handles are live; the cube is relinquished only after
        // the route reports success.
        let result = self
            .native
            .check(unsafe { (self.native.engine.skybox_set_owned_environment)(handle, cube) });
        match result {
            Ok(()) => {
                environment.relinquish();
                self.borrowed_environment = None;
                Ok(())
            }
            Err(error) => Err(EnvironmentNotTransferred { environment, error }),
        }
    }

    /// Draws the sky over whatever target is currently bound.
    pub fn draw(&self, view: Matrix, projection: Matrix, width: i32, height: i32) -> Result<()> {
        let handle = self.core.get()?;
        let view = native_matrix(view);
        let projection = native_matrix(projection);
        // SAFETY: the handle is owned and both matrices are borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.skybox_draw)(handle, &view, &projection, width, height)
        })
    }

    /// The world direction one screen point looks along, through the rotated sky.
    ///
    /// A pure function of its arguments, which is what makes the sky's rotation
    /// checkable without drawing it.
    pub fn compute_view_ray(
        view: Matrix,
        projection: Matrix,
        ndc_x: f32,
        ndc_y: f32,
        yaw: f32,
    ) -> Result<Vector3> {
        let native = Native::process()?;
        let view = native_matrix(view);
        let projection = native_matrix(projection);
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: both matrices are borrowed for the call and the output is a
        // live local.
        native.check(unsafe {
            (native.engine.skybox_compute_view_ray)(
                &view,
                &projection,
                ndc_x,
                ndc_y,
                yaw,
                &mut value,
            )
        })?;
        Ok(from_native_vector3(value))
    }

    /// How bright the sky is drawn.
    pub fn intensity(&self) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.skybox_get_intensity)(handle, &mut value) })?;
        Ok(value)
    }

    /// Sets how bright the sky is drawn.
    pub fn set_intensity(&self, value: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.native
            .check(unsafe { (self.native.engine.skybox_set_intensity)(handle, value) })
    }

    /// How far the sky is rotated about the vertical axis, in radians.
    pub fn yaw(&self) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.skybox_get_yaw)(handle, &mut value) })?;
        Ok(value)
    }

    /// Sets how far the sky is rotated about the vertical axis.
    pub fn set_yaw(&self, value: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.native
            .check(unsafe { (self.native.engine.skybox_set_yaw)(handle, value) })
    }

    /// The colour the sky is multiplied by.
    pub fn tint(&self) -> Result<Vector3> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.skybox_get_tint)(handle, &mut value) })?;
        Ok(from_native_vector3(value))
    }

    /// Sets the colour the sky is multiplied by.
    pub fn set_tint(&self, value: Vector3) -> Result<()> {
        let handle = self.core.get()?;
        let tint = native_vector3(value);
        // SAFETY: the handle is owned and the colour is borrowed for the call.
        self.native
            .check(unsafe { (self.native.engine.skybox_set_tint)(handle, &tint) })
    }

    /// Releases the skybox now rather than at drop.
    pub fn release(&mut self) -> Result<()> {
        let result = self.core.release();
        self.borrowed_environment = None;
        result
    }
}

impl Drop for Skybox {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// An analytic sky: a Preetham-style model rather than a cube map.
pub struct AtmosphericSky {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl AtmosphericSky {
    /// Creates an atmospheric sky on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.atmospheric_sky_create)(device.handle()?, &mut handle)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.atmospheric_sky_destroy,
            released: "the atmospheric sky has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// Whether this renderer can draw the model.
    pub fn is_supported(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.atmospheric_sky_is_supported)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Draws the sky over whatever target is currently bound.
    pub fn draw(&self, view: Matrix, projection: Matrix, width: i32, height: i32) -> Result<()> {
        let handle = self.core.get()?;
        let view = native_matrix(view);
        let projection = native_matrix(projection);
        // SAFETY: the handle is owned and both matrices are borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.atmospheric_sky_draw)(handle, &view, &projection, width, height)
        })
    }

    /// The direction the sun is in.
    pub fn sun_direction(&self) -> Result<Vector3> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.atmospheric_sky_get_sun_direction)(handle, &mut value)
        })?;
        Ok(from_native_vector3(value))
    }

    /// Sets the direction the sun is in.
    pub fn set_sun_direction(&self, value: Vector3) -> Result<()> {
        let handle = self.core.get()?;
        let direction = native_vector3(value);
        // SAFETY: the handle is owned and the direction is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.atmospheric_sky_set_sun_direction)(handle, &direction)
        })
    }

    /// How hazy the atmosphere is.
    pub fn turbidity(&self) -> Result<f32> {
        self.scalar(self.native.engine.atmospheric_sky_get_turbidity)
    }

    /// Sets how hazy the atmosphere is.
    pub fn set_turbidity(&self, value: f32) -> Result<()> {
        self.set_scalar(self.native.engine.atmospheric_sky_set_turbidity, value)
    }

    /// How bright the sky is drawn.
    pub fn intensity(&self) -> Result<f32> {
        self.scalar(self.native.engine.atmospheric_sky_get_intensity)
    }

    /// Sets how bright the sky is drawn.
    pub fn set_intensity(&self, value: f32) -> Result<()> {
        self.set_scalar(self.native.engine.atmospheric_sky_set_intensity, value)
    }

    /// The model's radiance along one direction, for a sun and a turbidity.
    ///
    /// A pure function, so the model can be evaluated -- and asserted -- without
    /// a device or a frame.
    pub fn radiance(direction: Vector3, sun_direction: Vector3, turbidity: f32) -> Result<Vector3> {
        let native = Native::process()?;
        let direction = native_vector3(direction);
        let sun = native_vector3(sun_direction);
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: both directions are borrowed for the call and the output is a
        // live local.
        native.check(unsafe {
            (native.engine.atmospheric_sky_radiance)(&direction, &sun, turbidity, &mut value)
        })?;
        Ok(from_native_vector3(value))
    }

    /// The model's own GLSL, as the engine compiles it.
    pub fn model_glsl() -> Result<String> {
        let native = Native::process()?;
        copy_text(&native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe { (api.atmospheric_sky_copy_model_glsl)(destination, capacity, out_bytes) }
        })
    }

    /// Releases the sky now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }

    fn scalar(
        &self,
        route: unsafe extern "C" fn(sys::CNA_AtmosphericSkyHandle, *mut f32) -> sys::CNA_Result,
    ) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value)
    }

    fn set_scalar(
        &self,
        route: unsafe extern "C" fn(sys::CNA_AtmosphericSkyHandle, f32) -> sys::CNA_Result,
        value: f32,
    ) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.native.check(unsafe { route(handle, value) })
    }
}

impl Drop for AtmosphericSky {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// Defines one concrete post-process pass over the shared `PostProcessPass`.
///
/// Upstream's concrete passes are the same handle type driven through the same
/// shared operations, so `apply`, `name` and `is_supported` come from the pass
/// itself and only the pass's own knobs are declared here.
macro_rules! concrete_pass {
    ($name:ident, $create:ident, $doc:literal) => {
        #[doc = $doc]
        pub struct $name {
            pass: PostProcessPass,
        }

        impl $name {
            #[doc = "Creates the pass on a device."]
            pub fn new(device: &GraphicsDevice) -> Result<Self> {
                let native = device.state_native();
                let mut handle = sys::CNA_INVALID_HANDLE;
                // SAFETY: the device handle is live and the output is a live local.
                native
                    .check(unsafe { (native.engine.$create)(device.handle()?, &mut handle) })?;
                Ok(Self {
                    pass: PostProcessPass::adopt(native, device, handle, None),
                })
            }

            /// The pass itself, for the operations every pass shares.
            #[must_use]
            pub const fn pass(&self) -> &PostProcessPass {
                &self.pass
            }

            /// Hands the pass over, for adding to a chain.
            #[must_use]
            pub fn into_pass(self) -> PostProcessPass {
                self.pass
            }

            #[allow(dead_code)]
            fn scalar(
                &self,
                route: unsafe extern "C" fn(
                    sys::CNA_PostProcessPassHandle,
                    *mut f32,
                ) -> sys::CNA_Result,
            ) -> Result<f32> {
                let handle = self.pass.core.get()?;
                let mut value = 0.0_f32;
                // SAFETY: the handle is owned and the output is a live local.
                self.pass.native.check(unsafe { route(handle, &mut value) })?;
                Ok(value)
            }

            #[allow(dead_code)]
            fn set_scalar(
                &self,
                route: unsafe extern "C" fn(
                    sys::CNA_PostProcessPassHandle,
                    f32,
                ) -> sys::CNA_Result,
                value: f32,
            ) -> Result<()> {
                let handle = self.pass.core.get()?;
                // SAFETY: the handle is owned and the value is by value.
                self.pass.native.check(unsafe { route(handle, value) })
            }

            #[allow(dead_code)]
            fn count(
                &self,
                route: unsafe extern "C" fn(
                    sys::CNA_PostProcessPassHandle,
                    *mut i32,
                ) -> sys::CNA_Result,
            ) -> Result<i32> {
                let handle = self.pass.core.get()?;
                let mut value = 0_i32;
                // SAFETY: the handle is owned and the output is a live local.
                self.pass.native.check(unsafe { route(handle, &mut value) })?;
                Ok(value)
            }

            #[allow(dead_code)]
            fn set_count(
                &self,
                route: unsafe extern "C" fn(
                    sys::CNA_PostProcessPassHandle,
                    i32,
                ) -> sys::CNA_Result,
                value: i32,
            ) -> Result<()> {
                let handle = self.pass.core.get()?;
                // SAFETY: the handle is owned and the value is by value.
                self.pass.native.check(unsafe { route(handle, value) })
            }

            #[allow(dead_code)]
            fn flag(
                &self,
                route: unsafe extern "C" fn(
                    sys::CNA_PostProcessPassHandle,
                    *mut sys::CNA_Bool,
                ) -> sys::CNA_Result,
            ) -> Result<bool> {
                let handle = self.pass.core.get()?;
                let mut value: sys::CNA_Bool = 0;
                // SAFETY: the handle is owned and the output is a live local.
                self.pass.native.check(unsafe { route(handle, &mut value) })?;
                Ok(value != 0)
            }

            #[allow(dead_code)]
            fn set_flag(
                &self,
                route: unsafe extern "C" fn(
                    sys::CNA_PostProcessPassHandle,
                    sys::CNA_Bool,
                ) -> sys::CNA_Result,
                value: bool,
            ) -> Result<()> {
                let handle = self.pass.core.get()?;
                // SAFETY: the handle is owned and the flag is a canonical boolean.
                self.pass
                    .native
                    .check(unsafe { route(handle, u8::from(value)) })
            }

            #[allow(dead_code)]
            fn vector3(
                &self,
                route: unsafe extern "C" fn(
                    sys::CNA_PostProcessPassHandle,
                    *mut sys::CNA_Vector3,
                ) -> sys::CNA_Result,
            ) -> Result<Vector3> {
                let handle = self.pass.core.get()?;
                let mut value = sys::CNA_Vector3::default();
                // SAFETY: the handle is owned and the output is a live local.
                self.pass.native.check(unsafe { route(handle, &mut value) })?;
                Ok(from_native_vector3(value))
            }

            #[allow(dead_code)]
            fn set_vector3(
                &self,
                route: unsafe extern "C" fn(
                    sys::CNA_PostProcessPassHandle,
                    *const sys::CNA_Vector3,
                ) -> sys::CNA_Result,
                value: Vector3,
            ) -> Result<()> {
                let handle = self.pass.core.get()?;
                let vector = native_vector3(value);
                // SAFETY: the handle is owned and the vector is borrowed for the call.
                self.pass.native.check(unsafe { route(handle, &vector) })
            }

            #[allow(dead_code)]
            fn text(
                &self,
                route: impl Fn(
                    &crate::native::engine::EngineApi,
                    sys::CNA_PostProcessPassHandle,
                    *mut c_char,
                    u64,
                    *mut u64,
                ) -> sys::CNA_Result,
            ) -> Result<String> {
                let handle = self.pass.core.get()?;
                copy_text(&self.pass.native, |api, destination, capacity, out_bytes| {
                    route(api, handle, destination, capacity, out_bytes)
                })
            }
        }
    };
}

/// Declares one `f32` knob on a concrete pass.
macro_rules! pass_scalar {
    ($get:ident, $set:ident, $get_route:ident, $set_route:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $get(&self) -> Result<f32> {
            self.scalar(self.pass.native.engine.$get_route)
        }

        #[doc = $doc]
        pub fn $set(&self, value: f32) -> Result<()> {
            self.set_scalar(self.pass.native.engine.$set_route, value)
        }
    };
}

/// Declares one `i32` knob on a concrete pass.
macro_rules! pass_count {
    ($get:ident, $set:ident, $get_route:ident, $set_route:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $get(&self) -> Result<i32> {
            self.count(self.pass.native.engine.$get_route)
        }

        #[doc = $doc]
        pub fn $set(&self, value: i32) -> Result<()> {
            self.set_count(self.pass.native.engine.$set_route, value)
        }
    };
}

/// Declares one boolean knob on a concrete pass.
macro_rules! pass_flag {
    ($get:ident, $set:ident, $get_route:ident, $set_route:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $get(&self) -> Result<bool> {
            self.flag(self.pass.native.engine.$get_route)
        }

        #[doc = $doc]
        pub fn $set(&self, value: bool) -> Result<()> {
            self.set_flag(self.pass.native.engine.$set_route, value)
        }
    };
}

/// Declares one `Vector3` knob on a concrete pass.
macro_rules! pass_vector3 {
    ($get:ident, $set:ident, $get_route:ident, $set_route:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $get(&self) -> Result<Vector3> {
            self.vector3(self.pass.native.engine.$get_route)
        }

        #[doc = $doc]
        pub fn $set(&self, value: Vector3) -> Result<()> {
            self.set_vector3(self.pass.native.engine.$set_route, value)
        }
    };
}

concrete_pass!(
    BloomPass,
    bloom_pass_create,
    "A bloom pass: a bright-pass extraction and a blur pyramid."
);

impl BloomPass {
    pass_scalar!(
        threshold, set_threshold, bloom_pass_get_threshold, bloom_pass_set_threshold,
        "The luminance above which a pixel contributes to the bloom."
    );
    pass_scalar!(
        intensity, set_intensity, bloom_pass_get_intensity, bloom_pass_set_intensity,
        "How strongly the bloom is added back."
    );
    pass_count!(
        iterations, set_iterations, bloom_pass_get_iterations, bloom_pass_set_iterations,
        "How many pyramid levels the blur uses; stored as given and clamped where the pyramid is built."
    );

    /// Releases the pass's pooled pyramid targets.
    pub fn reset_targets(&self) -> Result<()> {
        let handle = self.pass.core.get()?;
        // SAFETY: the handle is owned.
        self.pass
            .native
            .check(unsafe { (self.pass.native.engine.bloom_pass_reset_targets)(handle) })
    }

    /// What the bright pass keeps of one channel above a threshold.
    ///
    /// A pure function, so the extraction curve is assertable without a frame.
    pub fn extract_channel(value: f32, threshold: f32) -> Result<f32> {
        let native = Native::process()?;
        let mut out = 0.0_f32;
        // SAFETY: both inputs are by value and the output is a live local.
        native
            .check(unsafe { (native.engine.bloom_pass_extract_channel)(value, threshold, &mut out) })?;
        Ok(out)
    }

    /// How many pyramid levels a render-quality preset asks for.
    pub fn iterations_for_quality(quality: RenderQuality) -> Result<i32> {
        let native = Native::process()?;
        let mut value = 0_i32;
        // SAFETY: the identity is canonical and the output is a live local.
        native.check(unsafe {
            (native.engine.bloom_pass_iterations_for_quality)(quality.to_native(), &mut value)
        })?;
        Ok(value)
    }
}

concrete_pass!(
    ChromaticAberrationPass,
    chromatic_aberration_pass_create,
    "A chromatic-aberration pass: the channels are sampled at slightly different radii."
);

impl ChromaticAberrationPass {
    pass_scalar!(
        strength, set_strength,
        chromatic_aberration_pass_get_strength, chromatic_aberration_pass_set_strength,
        "How far apart the channels are sampled."
    );
}

concrete_pass!(FilmGrainPass, film_grain_pass_create, "A film-grain pass.");

impl FilmGrainPass {
    pass_scalar!(
        intensity, set_intensity, film_grain_pass_get_intensity, film_grain_pass_set_intensity,
        "How strong the grain is."
    );
}

concrete_pass!(LensFlarePass, lens_flare_pass_create, "A lens-flare pass.");

impl LensFlarePass {
    pass_scalar!(
        intensity, set_intensity, lens_flare_pass_get_intensity, lens_flare_pass_set_intensity,
        "How strong the flare is."
    );
    pass_scalar!(
        threshold, set_threshold, lens_flare_pass_get_threshold, lens_flare_pass_set_threshold,
        "The luminance above which a pixel produces a ghost."
    );
    pass_scalar!(
        dispersal, set_dispersal, lens_flare_pass_get_dispersal, lens_flare_pass_set_dispersal,
        "How far apart the ghost images are spread."
    );
}

concrete_pass!(MotionBlurPass, motion_blur_pass_create, "A motion-blur pass.");

impl MotionBlurPass {
    pass_scalar!(
        strength, set_strength, motion_blur_pass_get_strength, motion_blur_pass_set_strength,
        "How far the blur reaches along the velocity vector."
    );
    pass_scalar!(
        max_distance, set_max_distance,
        motion_blur_pass_get_max_distance, motion_blur_pass_set_max_distance,
        "The furthest the blur will reach, whatever the velocity says."
    );
}

concrete_pass!(
    HeightFogPass,
    height_fog_pass_create,
    "A height-fog pass: density falls off with altitude."
);

impl HeightFogPass {
    pass_scalar!(
        density, set_density, height_fog_pass_get_density, height_fog_pass_set_density,
        "The fog's density at the base height."
    );
    pass_scalar!(
        falloff, set_falloff, height_fog_pass_get_falloff, height_fog_pass_set_falloff,
        "How quickly the density falls off with altitude."
    );
    pass_scalar!(
        base_height, set_base_height,
        height_fog_pass_get_base_height, height_fog_pass_set_base_height,
        "The altitude the density is quoted at."
    );
    pass_vector3!(
        color, set_color, height_fog_pass_get_color, height_fog_pass_set_color,
        "The fog's colour."
    );

    /// The optical depth along a ray through the fog.
    ///
    /// A pure function of the geometry, so the fog's integral is assertable
    /// without rendering it. The ray starts at `camera_height` and gains
    /// `ray_height_step` of altitude per unit travelled.
    #[allow(clippy::too_many_arguments)]
    pub fn optical_depth(
        camera_height: f32,
        ray_height_step: f32,
        distance: f32,
        density: f32,
        falloff: f32,
        base_height: f32,
    ) -> Result<f32> {
        let native = Native::process()?;
        let mut value = 0.0_f32;
        // SAFETY: every input is by value and the output is a live local.
        native.check(unsafe {
            (native.engine.height_fog_pass_optical_depth)(
                camera_height,
                ray_height_step,
                distance,
                density,
                falloff,
                base_height,
                &mut value,
            )
        })?;
        Ok(value)
    }
}

concrete_pass!(SsaoPass, ssao_pass_create, "A screen-space ambient-occlusion pass.");

impl SsaoPass {
    pass_scalar!(
        radius, set_radius, ssao_pass_get_radius, ssao_pass_set_radius,
        "How far the occlusion search reaches, in world units."
    );
    pass_scalar!(
        intensity, set_intensity, ssao_pass_get_intensity, ssao_pass_set_intensity,
        "How strongly the occlusion darkens."
    );
    pass_count!(
        sample_count, set_sample_count,
        ssao_pass_get_sample_count, ssao_pass_set_sample_count,
        "How many samples the kernel takes."
    );
    pass_flag!(
        is_half_resolution, set_half_resolution,
        ssao_pass_get_half_resolution, ssao_pass_set_half_resolution,
        "Whether the pass runs at half resolution."
    );

    /// Releases the pass's pooled targets.
    pub fn reset_targets(&self) -> Result<()> {
        let handle = self.pass.core.get()?;
        // SAFETY: the handle is owned.
        self.pass
            .native
            .check(unsafe { (self.pass.native.engine.ssao_pass_reset_targets)(handle) })
    }

    /// The pass's own sample kernel, as it will use it.
    ///
    /// Reading it is what makes "the kernel is a hemisphere" checkable rather
    /// than a comment: every sample is a direction the pass will actually take.
    ///
    /// The size is asked for rather than assumed. The kernel is not the sample
    /// count: upstream keeps a fixed pool and the count selects how much of it
    /// a frame uses, so sizing the destination from `sample_count` refuses with
    /// `BUFFER_TOO_SMALL` -- and that particular refusal carries no message of
    /// its own, so it surfaces whatever the last failing call happened to say.
    pub fn kernel(&self) -> Result<Vec<Vector3>> {
        let handle = self.pass.core.get()?;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        let probe = unsafe {
            (self.pass.native.engine.ssao_pass_copy_kernel)(
                handle,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        };
        if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.pass.native.check(probe)?;
        }
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("the kernel size does not fit in memory"))?;
        if capacity == 0 {
            return Ok(Vec::new());
        }
        let mut buffer = vec![sys::CNA_Vector3::default(); capacity];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `capacity`
        // writable vectors, which is the count passed alongside it.
        self.pass.native.check(unsafe {
            (self.pass.native.engine.ssao_pass_copy_kernel)(
                handle,
                buffer.as_mut_ptr(),
                required,
                &mut count,
            )
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported a kernel larger than memory"))?;
        Ok(buffer
            .into_iter()
            .take(count.min(capacity))
            .map(from_native_vector3)
            .collect())
    }

    /// How many samples a render-quality preset asks for.
    pub fn sample_count_for_quality(quality: RenderQuality) -> Result<i32> {
        let native = Native::process()?;
        let mut value = 0_i32;
        // SAFETY: the identity is canonical and the output is a live local.
        native.check(unsafe {
            (native.engine.ssao_pass_sample_count_for_quality)(quality.to_native(), &mut value)
        })?;
        Ok(value)
    }

    /// The occlusion GLSL, at full or half resolution.
    pub fn occlusion_glsl(half_resolution: bool) -> Result<String> {
        let native = Native::process()?;
        copy_text(&native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe {
                (api.ssao_pass_copy_occlusion_glsl)(
                    u8::from(half_resolution),
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
    }
}

concrete_pass!(SsrPass, ssr_pass_create, "A screen-space reflection pass.");

impl SsrPass {
    pass_scalar!(
        max_distance, set_max_distance, ssr_pass_get_max_distance, ssr_pass_set_max_distance,
        "How far a reflection ray marches."
    );
    pass_scalar!(
        thickness, set_thickness, ssr_pass_get_thickness, ssr_pass_set_thickness,
        "How thick a depth sample is treated as being."
    );
    pass_scalar!(
        depth_bias, set_depth_bias, ssr_pass_get_depth_bias, ssr_pass_set_depth_bias,
        "The bias applied to the ray's depth comparison."
    );
    pass_scalar!(
        edge_fade, set_edge_fade, ssr_pass_get_edge_fade, ssr_pass_set_edge_fade,
        "How far from the screen edge the reflection fades out."
    );
    pass_scalar!(
        roughness_blur, set_roughness_blur,
        ssr_pass_get_roughness_blur, ssr_pass_set_roughness_blur,
        "How much a rough surface blurs its reflection."
    );
    pass_scalar!(
        intensity, set_intensity, ssr_pass_get_intensity, ssr_pass_set_intensity,
        "How strongly the reflection is added."
    );
    pass_count!(
        step_count, set_step_count, ssr_pass_get_step_count, ssr_pass_set_step_count,
        "How many steps the ray march takes; clamped to the engine's own bounds."
    );
}

concrete_pass!(
    DepthOfFieldPass,
    depth_of_field_pass_create,
    "A depth-of-field pass, parameterised as a physical lens."
);

impl DepthOfFieldPass {
    pass_scalar!(
        focus_distance, set_focus_distance,
        depth_of_field_pass_get_focus_distance, depth_of_field_pass_set_focus_distance,
        "The distance in focus, in world units."
    );
    pass_scalar!(
        focal_length, set_focal_length,
        depth_of_field_pass_get_focal_length, depth_of_field_pass_set_focal_length,
        "The lens's focal length in millimetres."
    );
    pass_scalar!(
        f_number, set_f_number, depth_of_field_pass_get_f_number, depth_of_field_pass_set_f_number,
        "The lens's f-number."
    );
    pass_scalar!(
        max_radius, set_max_radius,
        depth_of_field_pass_get_max_radius, depth_of_field_pass_set_max_radius,
        "The largest circle of confusion the pass will draw."
    );

    /// The circle of confusion, in millimetres, for one distance.
    ///
    /// The lens equation itself, as a pure function: a caller can check the
    /// focus falls where it asked without rendering anything.
    pub fn circle_of_confusion_millimetres(
        distance: f32,
        focus_distance: f32,
        focal_length: f32,
        f_number: f32,
    ) -> Result<f32> {
        let native = Native::process()?;
        let mut value = 0.0_f32;
        // SAFETY: every input is by value and the output is a live local.
        native.check(unsafe {
            (native.engine.depth_of_field_pass_circle_of_confusion_millimetres)(
                distance,
                focus_distance,
                focal_length,
                f_number,
                &mut value,
            )
        })?;
        Ok(value)
    }
}

concrete_pass!(LightShaftPass, light_shaft_pass_create, "A light-shaft (god-ray) pass.");

impl LightShaftPass {
    pass_scalar!(
        intensity, set_intensity, light_shaft_pass_get_intensity, light_shaft_pass_set_intensity,
        "How strong the shafts are."
    );
    pass_scalar!(
        decay, set_decay, light_shaft_pass_get_decay, light_shaft_pass_set_decay,
        "How quickly a shaft fades along its length."
    );
    pass_scalar!(
        threshold, set_threshold, light_shaft_pass_get_threshold, light_shaft_pass_set_threshold,
        "The luminance above which a pixel contributes."
    );

    /// Where the light is on screen, in normalised device coordinates.
    pub fn light_screen_position(&self) -> Result<Vector2> {
        let handle = self.pass.core.get()?;
        let mut value = sys::CNA_Vector2::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.pass.native.check(unsafe {
            (self.pass.native.engine.light_shaft_pass_get_light_screen_position)(
                handle,
                &mut value,
            )
        })?;
        Ok(Vector2::from_x_and_y(value.x, value.y))
    }

    /// Sets where the light is on screen.
    pub fn set_light_screen_position(&self, value: Vector2) -> Result<()> {
        let handle = self.pass.core.get()?;
        let position = sys::CNA_Vector2 {
            x: value.X,
            y: value.Y,
        };
        // SAFETY: the handle is owned and the position is borrowed for the call.
        self.pass.native.check(unsafe {
            (self.pass.native.engine.light_shaft_pass_set_light_screen_position)(
                handle,
                &position,
            )
        })
    }
}

concrete_pass!(VolumetricFogPassCore, volumetric_fog_pass_create, "A volumetric-fog pass.");

/// A volumetric-fog pass.
///
/// Distinct from the other concrete passes because it retains something: CNA
/// borrows the shadow map it scatters through and keeps a raw pointer to it.
pub struct VolumetricFogPass {
    inner: VolumetricFogPassCore,
    shadow_map: Option<Arc<ShadowMap>>,
}

impl VolumetricFogPass {
    /// Creates the pass on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        Ok(Self {
            inner: VolumetricFogPassCore::new(device)?,
            shadow_map: None,
        })
    }

    /// The pass itself, for the operations every pass shares.
    #[must_use]
    pub const fn pass(&self) -> &PostProcessPass {
        self.inner.pass()
    }

    /// Hands the pass over, for adding to a chain.
    ///
    /// The shadow map goes with it, because CNA still points at it.
    #[must_use]
    pub fn into_pass(self) -> (PostProcessPass, Option<Arc<ShadowMap>>) {
        (self.inner.into_pass(), self.shadow_map)
    }
}

impl VolumetricFogPassCore {
    pass_scalar!(
        density, set_density, volumetric_fog_pass_get_density, volumetric_fog_pass_set_density,
        "How dense the medium is."
    );
    pass_scalar!(
        anisotropy, set_anisotropy,
        volumetric_fog_pass_get_anisotropy, volumetric_fog_pass_set_anisotropy,
        "The phase function's forward-scattering parameter."
    );
    pass_scalar!(
        range, set_range, volumetric_fog_pass_get_range, volumetric_fog_pass_set_range,
        "How far the march reaches."
    );

}

impl VolumetricFogPass {
    /// How dense the medium is.
    pub fn density(&self) -> Result<f32> {
        self.inner.density()
    }

    /// Sets how dense the medium is.
    pub fn set_density(&self, value: f32) -> Result<()> {
        self.inner.set_density(value)
    }

    /// The phase function's forward-scattering parameter.
    pub fn anisotropy(&self) -> Result<f32> {
        self.inner.anisotropy()
    }

    /// Sets the phase function's forward-scattering parameter.
    pub fn set_anisotropy(&self, value: f32) -> Result<()> {
        self.inner.set_anisotropy(value)
    }

    /// How far the march reaches.
    pub fn range(&self) -> Result<f32> {
        self.inner.range()
    }

    /// Sets how far the march reaches.
    pub fn set_range(&self, value: f32) -> Result<()> {
        self.inner.set_range(value)
    }

    /// Gives the pass the light it scatters, and the shadow map it reads.
    ///
    /// The shadow map is borrowed, so the [`Arc`] is what keeps it alive for as
    /// long as the pass points at it.
    pub fn set_light(
        &mut self,
        shadow_map: &Arc<ShadowMap>,
        direction: Vector3,
        color: Vector3,
    ) -> Result<()> {
        let handle = self.inner.pass.core.get()?;
        let direction = native_vector3(direction);
        let color = native_vector3(color);
        // SAFETY: both handles are live and the vectors are borrowed for the
        // call; retention follows on success.
        self.inner.pass.native.check(unsafe {
            (self.inner.pass.native.engine.volumetric_fog_pass_set_light)(
                handle,
                shadow_map.core.get()?,
                &direction,
                &color,
            )
        })?;
        self.shadow_map = Some(Arc::clone(shadow_map));
        Ok(())
    }
}

concrete_pass!(
    AerialPerspectivePass,
    aerial_perspective_pass_create,
    "An aerial-perspective pass: distance haze from the same atmosphere the sky uses."
);

impl AerialPerspectivePass {
    pass_scalar!(
        turbidity, set_turbidity,
        aerial_perspective_pass_get_turbidity, aerial_perspective_pass_set_turbidity,
        "How hazy the atmosphere is."
    );
    pass_scalar!(
        intensity, set_intensity,
        aerial_perspective_pass_get_intensity, aerial_perspective_pass_set_intensity,
        "How strongly the haze is applied."
    );
    pass_scalar!(
        scale_height, set_scale_height,
        aerial_perspective_pass_get_scale_height, aerial_perspective_pass_set_scale_height,
        "The atmosphere's scale height."
    );
    pass_vector3!(
        sun_direction, set_sun_direction,
        aerial_perspective_pass_get_sun_direction, aerial_perspective_pass_set_sun_direction,
        "The sun direction the haze is coloured from."
    );

    /// Why the pass fell back; empty when it did not.
    pub fn fallback_reason(&self) -> Result<String> {
        self.text(|api, handle, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe {
                (api.aerial_perspective_pass_copy_fallback_reason)(
                    handle,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
    }

    /// The air mass along a view direction out to a distance.
    pub fn air_mass_for_distance(
        view_direction: Vector3,
        distance: f32,
        scale_height: f32,
    ) -> Result<f32> {
        let native = Native::process()?;
        let direction = native_vector3(view_direction);
        let mut value = 0.0_f32;
        // SAFETY: the direction is borrowed for the call and the output is a
        // live local.
        native.check(unsafe {
            (native.engine.aerial_perspective_pass_air_mass_for_distance)(
                &direction,
                distance,
                scale_height,
                &mut value,
            )
        })?;
        Ok(value)
    }

    /// What fraction of each channel survives one air mass at a turbidity.
    pub fn transmittance(air_mass: f32, turbidity: f32) -> Result<Vector3> {
        let native = Native::process()?;
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: both inputs are by value and the output is a live local.
        native.check(unsafe {
            (native.engine.aerial_perspective_pass_transmittance)(
                air_mass,
                turbidity,
                &mut value,
            )
        })?;
        Ok(from_native_vector3(value))
    }
}

concrete_pass!(
    ContactShadowPass,
    contact_shadow_pass_create,
    "A contact-shadow pass: a short screen-space ray march for the shadows a map misses."
);

impl ContactShadowPass {
    pass_scalar!(
        max_distance, set_max_distance,
        contact_shadow_pass_get_max_distance, contact_shadow_pass_set_max_distance,
        "How far the march reaches, in world units."
    );
    pass_scalar!(
        thickness, set_thickness,
        contact_shadow_pass_get_thickness, contact_shadow_pass_set_thickness,
        "How thick a depth sample is treated as being."
    );
    pass_scalar!(
        bias, set_bias, contact_shadow_pass_get_bias, contact_shadow_pass_set_bias,
        "The bias applied to the depth comparison."
    );
    pass_scalar!(
        intensity, set_intensity,
        contact_shadow_pass_get_intensity, contact_shadow_pass_set_intensity,
        "How strongly the contact shadow darkens."
    );
    pass_count!(
        step_count, set_step_count,
        contact_shadow_pass_get_step_count, contact_shadow_pass_set_step_count,
        "How many steps the march takes."
    );
    pass_vector3!(
        light_direction, set_light_direction,
        contact_shadow_pass_get_light_direction, contact_shadow_pass_set_light_direction,
        "The direction the light arrives from."
    );

    /// Why the pass fell back; empty when it did not.
    pub fn fallback_reason(&self) -> Result<String> {
        self.text(|api, handle, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe {
                (api.contact_shadow_pass_copy_fallback_reason)(
                    handle,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
    }

    /// Whether one march sample counts as occluded.
    ///
    /// The march's own test, as a pure function.
    pub fn is_occluded(
        ray_view_depth: f32,
        scene_view_depth: f32,
        bias: f32,
        thickness: f32,
    ) -> Result<bool> {
        let native = Native::process()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: every input is by value and the output is a live local.
        native.check(unsafe {
            (native.engine.contact_shadow_pass_is_occluded)(
                ray_view_depth,
                scene_view_depth,
                bias,
                thickness,
                &mut value,
            )
        })?;
        Ok(value != 0)
    }

    /// How a contact shadow's visibility combines with a shadow map's.
    pub fn combine_visibility(map_visibility: f32, contact_visibility: f32) -> Result<f32> {
        let native = Native::process()?;
        let mut value = 0.0_f32;
        // SAFETY: both inputs are by value and the output is a live local.
        native.check(unsafe {
            (native.engine.contact_shadow_pass_combine_visibility)(
                map_visibility,
                contact_visibility,
                &mut value,
            )
        })?;
        Ok(value)
    }

    /// The occlusion test's own GLSL.
    pub fn occlusion_test_glsl() -> Result<String> {
        let native = Native::process()?;
        copy_text(&native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe { (api.contact_shadow_pass_copy_occlusion_test_glsl)(destination, capacity, out_bytes) }
        })
    }
}

/// How a colour-grading lookup table is sampled between its slices.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum LutInterpolation {
    /// Eight-corner trilinear.
    Trilinear,
    /// Four-corner tetrahedral, which avoids trilinear's diagonal artefacts.
    Tetrahedral,
}

impl LutInterpolation {
    const fn from_native(value: sys::CNA_LutInterpolation) -> Option<Self> {
        Some(match value {
            sys::CNA_LUT_INTERPOLATION_TRILINEAR => Self::Trilinear,
            sys::CNA_LUT_INTERPOLATION_TETRAHEDRAL => Self::Tetrahedral,
            _ => return None,
        })
    }

    const fn to_native(self) -> sys::CNA_LutInterpolation {
        match self {
            Self::Trilinear => sys::CNA_LUT_INTERPOLATION_TRILINEAR,
            Self::Tetrahedral => sys::CNA_LUT_INTERPOLATION_TETRAHEDRAL,
        }
    }
}

concrete_pass!(
    ColorGradePassCore,
    color_grade_pass_create,
    "A colour-grading pass driven by a lookup table."
);

impl ColorGradePassCore {
    pass_scalar!(
        strength, set_strength, color_grade_pass_get_strength, color_grade_pass_set_strength,
        "How far the grade is applied, from zero through one."
    );
}

/// A colour-grading pass driven by a lookup table.
///
/// Distinct from the other concrete passes because it retains its tables: CNA
/// borrows the strip or volume LUT and keeps a raw pointer to it.
pub struct ColorGradePass {
    inner: ColorGradePassCore,
    strip_lut: Option<Texture2D>,
    volume_lut: Option<Texture3D>,
}

impl ColorGradePass {
    /// Creates the pass on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        Ok(Self {
            inner: ColorGradePassCore::new(device)?,
            strip_lut: None,
            volume_lut: None,
        })
    }

    /// The pass itself, for the operations every pass shares.
    #[must_use]
    pub const fn pass(&self) -> &PostProcessPass {
        self.inner.pass()
    }

    /// Hands the pass over, along with the tables CNA still points at.
    #[must_use]
    pub fn into_pass(self) -> (PostProcessPass, Option<Texture2D>, Option<Texture3D>) {
        (self.inner.into_pass(), self.strip_lut, self.volume_lut)
    }

    /// How far the grade is applied, from zero through one.
    pub fn strength(&self) -> Result<f32> {
        self.inner.strength()
    }

    /// Sets how far the grade is applied.
    pub fn set_strength(&self, value: f32) -> Result<()> {
        self.inner.set_strength(value)
    }

    /// How the table is sampled between its slices.
    pub fn interpolation(&self) -> Result<LutInterpolation> {
        let handle = self.inner.pass.core.get()?;
        let mut value: sys::CNA_LutInterpolation = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.inner.pass.native.check(unsafe {
            (self.inner.pass.native.engine.color_grade_pass_get_interpolation)(handle, &mut value)
        })?;
        LutInterpolation::from_native(value)
            .ok_or(CnaError::InvalidInput("native LUT interpolation is unknown"))
    }

    /// Sets how the table is sampled between its slices.
    pub fn set_interpolation(&self, value: LutInterpolation) -> Result<()> {
        let handle = self.inner.pass.core.get()?;
        // SAFETY: the handle is owned and the identity is canonical.
        self.inner.pass.native.check(unsafe {
            (self.inner.pass.native.engine.color_grade_pass_set_interpolation)(
                handle,
                value.to_native(),
            )
        })
    }

    /// The attached strip table, borrowed for as long as the pass lives.
    ///
    /// Like every engine query that answers with a handle, this one publishes a
    /// *new* handle aliasing the pass. The view releases it on drop; a caller
    /// that only compared it against `CNA_INVALID_HANDLE` would leak one per
    /// call and only find out at game shutdown.
    pub fn strip_lut(&self) -> Result<Option<BorrowedTexture2D<'_>>> {
        let handle = self.inner.pass.core.get()?;
        let mut value = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.inner.pass.native.check(unsafe {
            (self.inner.pass.native.engine.color_grade_pass_get_lut)(handle, &mut value)
        })?;
        if value == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        Ok(Some(BorrowedTexture2D {
            native: Arc::clone(&self.inner.pass.native),
            handle: value,
            owner: PhantomData,
        }))
    }

    /// Whether a strip table is attached.
    pub fn has_strip_lut(&self) -> Result<bool> {
        Ok(self.strip_lut()?.is_some())
    }

    /// The attached volume table, borrowed on the same terms.
    pub fn volume_lut(&self) -> Result<Option<BorrowedTexture3D<'_>>> {
        let handle = self.inner.pass.core.get()?;
        let mut value = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.inner.pass.native.check(unsafe {
            (self.inner.pass.native.engine.color_grade_pass_get_volume_lut)(handle, &mut value)
        })?;
        if value == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        Ok(Some(BorrowedTexture3D {
            native: Arc::clone(&self.inner.pass.native),
            handle: value,
            owner: PhantomData,
        }))
    }

    /// Whether a volume table is attached.
    pub fn has_volume_lut(&self) -> Result<bool> {
        Ok(self.volume_lut()?.is_some())
    }

    /// Attaches a strip lookup table, which the pass borrows.
    pub fn set_strip_lut(&mut self, lut: Option<Texture2D>) -> Result<()> {
        let handle = self.inner.pass.core.get()?;
        let texture = match lut.as_ref() {
            Some(value) => value.handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        // SAFETY: the handle is owned and the texture handle is live for the call.
        self.inner.pass.native.check(unsafe {
            (self.inner.pass.native.engine.color_grade_pass_set_lut)(handle, texture)
        })?;
        self.strip_lut = lut;
        Ok(())
    }

    /// Attaches a volume lookup table, which the pass borrows.
    pub fn set_volume_lut(&mut self, lut: Option<Texture3D>) -> Result<()> {
        let handle = self.inner.pass.core.get()?;
        let texture = match lut.as_ref() {
            Some(value) => value.native_handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        // SAFETY: the handle is owned and the texture handle is live for the call.
        self.inner.pass.native.check(unsafe {
            (self.inner.pass.native.engine.color_grade_pass_set_volume_lut)(handle, texture)
        })?;
        self.volume_lut = lut;
        Ok(())
    }

    /// Creates a strip table that grades nothing.
    ///
    /// The texture is the caller's: CNA allocates it and hands it over.
    pub fn create_identity_lut(device: &GraphicsDevice, size: i32) -> Result<Texture2D> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.color_grade_pass_create_identity_lut)(
                device.handle()?,
                size,
                &mut handle,
            )
        })?;
        Texture2D::from_owned_handle(device, handle)
    }

    /// The slice count a strip of the given pixel dimensions carries.
    ///
    /// A pure function, so a caller can validate a strip before uploading it.
    pub fn lut_size_for_strip(width: i32, height: i32) -> Result<i32> {
        let native = Native::process()?;
        let mut value = 0_i32;
        // SAFETY: both inputs are by value and the output is a live local.
        native.check(unsafe {
            (native.engine.color_grade_pass_lut_size_for_strip)(width, height, &mut value)
        })?;
        Ok(value)
    }
}

concrete_pass!(
    AsciiPass,
    ascii_pass_create,
    "An ASCII-art pass over CNAEXT's own ASCII post-process effect."
);

impl AsciiPass {
    /// The effect the pass draws through, borrowed for as long as the pass lives.
    ///
    /// Deliberately **not** a [`BorrowedEffect`]: the handle upstream publishes
    /// here names CNAEXT's own `AsciiPostProcessEffect`, not a generic
    /// `Effect`, and releasing it through the generic effect destroy leaves the
    /// process aborting at exit rather than failing at the call. The type is
    /// separate so the wrong release cannot be written.
    pub fn effect(&self) -> Result<Option<BorrowedAsciiEffect<'_>>> {
        let handle = self.pass.core.get()?;
        let mut value = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.pass.native.check(unsafe {
            (self.pass.native.engine.ascii_pass_get_effect)(handle, &mut value)
        })?;
        if value == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        Ok(Some(BorrowedAsciiEffect {
            native: Arc::clone(&self.pass.native),
            handle: value,
            owner: PhantomData,
        }))
    }

    /// Whether the pass carries an effect to draw through.
    pub fn has_effect(&self) -> Result<bool> {
        Ok(self.effect()?.is_some())
    }
}

/// A spatial upscaler: a sharpening resample from one size to another.
pub struct SpatialUpscalePass {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl SpatialUpscalePass {
    /// Creates an upscaler on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.spatial_upscale_pass_create)(device.handle()?, &mut handle)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.spatial_upscale_pass_destroy,
            released: "the spatial upscale pass has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// How strongly the resample sharpens.
    pub fn sharpness(&self) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.spatial_upscale_pass_get_sharpness)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Sets how strongly the resample sharpens.
    pub fn set_sharpness(&self, value: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.native
            .check(unsafe { (self.native.engine.spatial_upscale_pass_set_sharpness)(handle, value) })
    }

    /// Whether the filter follows edges rather than resampling uniformly.
    pub fn is_edge_adaptive(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.spatial_upscale_pass_get_edge_adaptive)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Sets whether the filter follows edges.
    pub fn set_edge_adaptive(&self, value: bool) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the flag is a canonical boolean.
        self.native.check(unsafe {
            (self.native.engine.spatial_upscale_pass_set_edge_adaptive)(handle, u8::from(value))
        })
    }

    /// Draws the source into the current target at the destination size.
    pub fn draw(
        &self,
        source: &Texture2D,
        source_width: i32,
        source_height: i32,
        destination_width: i32,
        destination_height: i32,
    ) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned, the texture handle is live and the
        // sizes are by value.
        self.native.check(unsafe {
            (self.native.engine.spatial_upscale_pass_draw)(
                handle,
                source.handle()?,
                source_width,
                source_height,
                destination_width,
                destination_height,
            )
        })
    }

    /// Whether a source and destination size pair is a no-op resample.
    ///
    /// A pure function, so a caller can skip the pass rather than run an
    /// identity through it.
    pub fn is_identity_scale(
        source_width: i32,
        source_height: i32,
        destination_width: i32,
        destination_height: i32,
    ) -> Result<bool> {
        let native = Native::process()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: every input is by value and the output is a live local.
        native.check(unsafe {
            (native.engine.spatial_upscale_pass_is_identity_scale)(
                source_width,
                source_height,
                destination_width,
                destination_height,
                &mut value,
            )
        })?;
        Ok(value != 0)
    }

    /// Releases the pass now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }
}

impl Drop for SpatialUpscalePass {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// A one-triangle fullscreen draw, for a pass written outside this chain.
///
/// `OWNED`. The effect it draws through is `BORROWED` for the call only, which
/// is why the draw takes it rather than the pass holding one.
pub struct FullscreenPass {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl FullscreenPass {
    /// Creates a fullscreen pass on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native
            .check(unsafe { (native.engine.fullscreen_pass_create)(device.handle()?, &mut handle) })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.fullscreen_pass_destroy,
            released: "the fullscreen pass has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// Draws the source into a destination render target.
    ///
    /// A `None` destination is the back buffer, which is what upstream's
    /// invalid handle means.
    pub fn draw(
        &self,
        source: &Texture2D,
        destination: Option<&RenderTarget2D>,
        effect: Option<&Effect>,
        width: i32,
        height: i32,
        sampler: &SamplerState,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let destination_handle = match destination {
            Some(target) => target.handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        let effect_handle = match effect {
            Some(value) => value.native_handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        let state = sampler.native();
        // SAFETY: the handle is owned, every resource handle is live and the
        // sampler state is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.fullscreen_pass_draw)(
                handle,
                source.handle()?,
                destination_handle,
                effect_handle,
                width,
                height,
                &state,
            )
        })
    }

    /// Draws the source over whatever target is already bound.
    pub fn draw_over_current_target(
        &self,
        source: &Texture2D,
        effect: Option<&Effect>,
        width: i32,
        height: i32,
        sampler: &SamplerState,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let effect_handle = match effect {
            Some(value) => value.native_handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        let state = sampler.native();
        // SAFETY: the handle is owned, every resource handle is live and the
        // sampler state is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.fullscreen_pass_draw_over_current_target)(
                handle,
                source.handle()?,
                effect_handle,
                width,
                height,
                &state,
            )
        })
    }

    /// Releases the pass now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }
}

impl Drop for FullscreenPass {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// A render-target binding that is put back when the scope ends.
///
/// `OWNED`, and deliberately not `Copy` or `Clone`: the scope is the thing that
/// restores the previous binding, so ending it twice or letting two values
/// think they own it would restore the wrong target.
pub struct ScopedRenderTarget {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl ScopedRenderTarget {
    /// Records the current binding and binds a destination until the scope ends.
    ///
    /// A `None` destination is the back buffer.
    pub fn begin(device: &GraphicsDevice, destination: Option<&RenderTarget2D>) -> Result<Self> {
        let native = device.state_native();
        let destination_handle = match destination {
            Some(target) => target.handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: both handles are live and the output is a live local.
        native.check(unsafe {
            (native.engine.scoped_render_target_begin)(
                device.handle()?,
                destination_handle,
                &mut handle,
            )
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.scoped_render_target_end,
            released: "the render-target scope has already ended",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// Whether the scope recorded a previous binding to restore.
    pub fn has_recorded_previous(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.scoped_render_target_get_has_recorded_previous)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Ends the scope now rather than at drop, reporting any failure.
    ///
    /// `Drop` ends it too, and cannot report; this is for a caller that wants
    /// to know whether the restore worked.
    pub fn end(self) -> Result<()> {
        self.core.release()
    }
}

impl Drop for ScopedRenderTarget {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// A point light: colour radiating from a position, falling off to a range.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct PointLight {
    /// World-space position.
    pub position: Vector3,
    /// Linear RGB colour.
    pub color: Vector3,
    /// Scalar multiplier on [`PointLight::color`].
    pub intensity: f32,
    /// Distance at which the light stops contributing.
    pub range: f32,
    /// Whether this light should be given a shadow cube.
    pub casts_shadows: bool,
}

impl PointLight {
    /// CNA's own defaults, asked of the library rather than restated here.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_PointLightEXT::default();
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.engine.point_light_ext_init)(&mut value) })?;
        Ok(Self {
            position: from_native_vector3(value.position),
            color: from_native_vector3(value.color),
            intensity: value.intensity,
            range: value.range,
            casts_shadows: value.casts_shadows != 0,
        })
    }

    fn to_native(self) -> sys::CNA_PointLightEXT {
        sys::CNA_PointLightEXT {
            struct_size: core::mem::size_of::<sys::CNA_PointLightEXT>() as u32,
            struct_version: 1,
            position: native_vector3(self.position),
            color: native_vector3(self.color),
            intensity: self.intensity,
            range: self.range,
            casts_shadows: u8::from(self.casts_shadows),
            reserved: [0; 3],
        }
    }
}

/// A spot light: a cone of colour from a position along a direction.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct SpotLight {
    /// World-space position.
    pub position: Vector3,
    /// The cone's axis.
    pub direction: Vector3,
    /// Linear RGB colour.
    pub color: Vector3,
    /// Scalar multiplier on [`SpotLight::color`].
    pub intensity: f32,
    /// Distance at which the light stops contributing.
    pub range: f32,
    /// The half angle the cone is at full brightness within, in radians.
    pub inner_angle: f32,
    /// The half angle the cone falls to nothing at, in radians.
    pub outer_angle: f32,
    /// Whether this light should be given a shadow map.
    pub casts_shadows: bool,
}

impl SpotLight {
    /// CNA's own defaults, asked of the library rather than restated here.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_SpotLightEXT::default();
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.engine.spot_light_ext_init)(&mut value) })?;
        Ok(Self {
            position: from_native_vector3(value.position),
            direction: from_native_vector3(value.direction),
            color: from_native_vector3(value.color),
            intensity: value.intensity,
            range: value.range,
            inner_angle: value.inner_angle,
            outer_angle: value.outer_angle,
            casts_shadows: value.casts_shadows != 0,
        })
    }

    fn to_native(self) -> sys::CNA_SpotLightEXT {
        sys::CNA_SpotLightEXT {
            struct_size: core::mem::size_of::<sys::CNA_SpotLightEXT>() as u32,
            struct_version: 1,
            position: native_vector3(self.position),
            direction: native_vector3(self.direction),
            color: native_vector3(self.color),
            intensity: self.intensity,
            range: self.range,
            inner_angle: self.inner_angle,
            outer_angle: self.outer_angle,
            casts_shadows: u8::from(self.casts_shadows),
            reserved: [0; 3],
        }
    }

    /// The view transform a spot light's shadow map casts from.
    pub fn compute_light_view(self) -> Result<Matrix> {
        let native = Native::process()?;
        let light = self.to_native();
        let mut value = sys::CNA_Matrix::default();
        // SAFETY: the light is borrowed for the call and the output is a live local.
        native.check(unsafe {
            (native.engine.spot_shadow_map_compute_light_view)(&light, &mut value)
        })?;
        Ok(from_native_matrix(value))
    }

    /// The projection a spot light's shadow map casts with.
    pub fn compute_light_projection(self) -> Result<Matrix> {
        let native = Native::process()?;
        let light = self.to_native();
        let mut value = sys::CNA_Matrix::default();
        // SAFETY: the light is borrowed for the call and the output is a live local.
        native.check(unsafe {
            (native.engine.spot_shadow_map_compute_light_projection)(&light, &mut value)
        })?;
        Ok(from_native_matrix(value))
    }
}

/// Declares one shadow-map variant over its own handle type.
macro_rules! shadow_variant {
    ($name:ident, $handle:ty, $destroy:ident, $released:literal, $doc:literal) => {
        #[doc = $doc]
        pub struct $name {
            core: Arc<EngineHandle>,
            native: Arc<Native>,
            device: GraphicsDevice,
        }

        impl $name {
            fn adopt(
                native: &Arc<Native>,
                device: &GraphicsDevice,
                handle: sys::CNA_Handle,
            ) -> Self {
                let core = Arc::new(EngineHandle {
                    native: Arc::clone(native),
                    handle: Mutex::new(handle),
                    destroy: native.engine.$destroy,
                    released: $released,
                });
                let child: Arc<dyn OwnedEngineChild> =
                    Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
                device.register_engine_child(&child);
                Self {
                    core,
                    native: Arc::clone(native),
                    device: device.clone(),
                }
            }

            /// Releases the map now rather than at drop.
            pub fn release(&self) -> Result<()> {
                self.core.release()
            }

            #[allow(dead_code)]
            fn flag(
                &self,
                route: unsafe extern "C" fn($handle, *mut sys::CNA_Bool) -> sys::CNA_Result,
            ) -> Result<bool> {
                let handle = self.core.get()?;
                let mut value: sys::CNA_Bool = 0;
                // SAFETY: the handle is owned and the output is a live local.
                self.native.check(unsafe { route(handle, &mut value) })?;
                Ok(value != 0)
            }

            #[allow(dead_code)]
            fn scalar(
                &self,
                route: unsafe extern "C" fn($handle, *mut f32) -> sys::CNA_Result,
            ) -> Result<f32> {
                let handle = self.core.get()?;
                let mut value = 0.0_f32;
                // SAFETY: the handle is owned and the output is a live local.
                self.native.check(unsafe { route(handle, &mut value) })?;
                Ok(value)
            }

            #[allow(dead_code)]
            fn set_scalar(
                &self,
                route: unsafe extern "C" fn($handle, f32) -> sys::CNA_Result,
                value: f32,
            ) -> Result<()> {
                let handle = self.core.get()?;
                // SAFETY: the handle is owned and the value is by value.
                self.native.check(unsafe { route(handle, value) })
            }

            #[allow(dead_code)]
            fn count(
                &self,
                route: unsafe extern "C" fn($handle, *mut i32) -> sys::CNA_Result,
            ) -> Result<i32> {
                let handle = self.core.get()?;
                let mut value = 0_i32;
                // SAFETY: the handle is owned and the output is a live local.
                self.native.check(unsafe { route(handle, &mut value) })?;
                Ok(value)
            }

            #[allow(dead_code)]
            fn matrix(
                &self,
                route: unsafe extern "C" fn($handle, *mut sys::CNA_Matrix) -> sys::CNA_Result,
            ) -> Result<Matrix> {
                let handle = self.core.get()?;
                let mut value = sys::CNA_Matrix::default();
                // SAFETY: the handle is owned and the output is a live local.
                self.native.check(unsafe { route(handle, &mut value) })?;
                Ok(from_native_matrix(value))
            }

            #[allow(dead_code)]
            fn vector3(
                &self,
                route: unsafe extern "C" fn($handle, *mut sys::CNA_Vector3) -> sys::CNA_Result,
            ) -> Result<Vector3> {
                let handle = self.core.get()?;
                let mut value = sys::CNA_Vector3::default();
                // SAFETY: the handle is owned and the output is a live local.
                self.native.check(unsafe { route(handle, &mut value) })?;
                Ok(from_native_vector3(value))
            }

            #[allow(dead_code)]
            fn borrowed_texture(
                &self,
                route: unsafe extern "C" fn($handle, *mut sys::CNA_Handle) -> sys::CNA_Result,
            ) -> Result<Option<BorrowedRenderTarget<'_>>> {
                let handle = self.core.get()?;
                let mut texture = sys::CNA_INVALID_HANDLE;
                // SAFETY: the handle is owned and the output is a live local.
                self.native.check(unsafe { route(handle, &mut texture) })?;
                if texture == sys::CNA_INVALID_HANDLE {
                    return Ok(None);
                }
                BorrowedRenderTarget::new(&self.native, &self.device, texture).map(Some)
            }

            #[allow(dead_code)]
            fn borrowed_effect(
                &self,
                route: unsafe extern "C" fn(
                    $handle,
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

        impl Drop for $name {
            fn drop(&mut self) {
                let _ = self.core.release();
            }
        }
    };
}

shadow_variant!(
    SpotShadowMap,
    sys::CNA_SpotShadowMapHandle,
    spot_shadow_map_destroy,
    "the spot shadow map has been released",
    "A spot-light shadow map: one square depth target, cast from a cone."
);

impl SpotShadowMap {
    /// Creates a spot-light shadow map at a quality preset.
    pub fn new(device: &GraphicsDevice, quality: ShadowQuality) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.spot_shadow_map_create)(
                device.handle()?,
                quality.to_native(),
                &mut handle,
            )
        })?;
        Ok(Self::adopt(native, device, handle))
    }

    /// Whether this renderer can cast into the map.
    pub fn is_supported(&self) -> Result<bool> {
        self.flag(self.native.engine.spot_shadow_map_is_supported)
    }

    /// Opens the shadow pass for a spot light.
    pub fn begin(&self, light: SpotLight) -> Result<()> {
        let handle = self.core.get()?;
        let light = light.to_native();
        // SAFETY: the handle is owned and the light is borrowed for the call.
        self.native
            .check(unsafe { (self.native.engine.spot_shadow_map_begin)(handle, &light) })
    }

    /// Closes the shadow pass.
    pub fn end(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.spot_shadow_map_end)(handle) })
    }

    /// The transform from world space into the map, as of the last `begin`.
    pub fn light_view_projection(&self) -> Result<Matrix> {
        self.matrix(self.native.engine.spot_shadow_map_get_light_view_projection)
    }

    /// Where the light the map was last opened for is.
    pub fn light_position(&self) -> Result<Vector3> {
        self.vector3(self.native.engine.spot_shadow_map_get_light_position)
    }

    /// How far that light reaches.
    pub fn light_range(&self) -> Result<f32> {
        self.scalar(self.native.engine.spot_shadow_map_get_light_range)
    }

    /// The map's edge length in texels.
    pub fn size(&self) -> Result<i32> {
        self.count(self.native.engine.spot_shadow_map_get_size)
    }

    /// The quality preset the map was created with.
    pub fn quality(&self) -> Result<ShadowQuality> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_ShadowQuality = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.spot_shadow_map_get_quality)(handle, &mut value)
        })?;
        ShadowQuality::from_native(value)
            .ok_or(CnaError::InvalidInput("native shadow quality is unknown"))
    }

    /// The depth bias applied when casting.
    pub fn depth_bias(&self) -> Result<f32> {
        self.scalar(self.native.engine.spot_shadow_map_get_depth_bias)
    }

    /// Sets the depth bias applied when casting.
    pub fn set_depth_bias(&self, value: f32) -> Result<()> {
        self.set_scalar(self.native.engine.spot_shadow_map_set_depth_bias, value)
    }

    /// A borrowed view of the map's depth texture.
    pub fn shadow_texture(&self) -> Result<Option<BorrowedRenderTarget<'_>>> {
        self.borrowed_texture(self.native.engine.spot_shadow_map_get_shadow_texture)
    }

    /// The map's caster effect, borrowed for as long as the map lives.
    pub fn caster_effect(&self) -> Result<Option<BorrowedEffect<'_>>> {
        self.borrowed_effect(self.native.engine.spot_shadow_map_get_caster_effect)
    }
}

shadow_variant!(
    CubeShadowMap,
    sys::CNA_CubeShadowMapHandle,
    cube_shadow_map_destroy,
    "the cube shadow map has been released",
    "A point-light shadow cube: six faces, cast from a position."
);

impl CubeShadowMap {
    /// Creates a shadow cube at a quality preset.
    pub fn new(device: &GraphicsDevice, quality: ShadowQuality) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.cube_shadow_map_create)(
                device.handle()?,
                quality.to_native(),
                &mut handle,
            )
        })?;
        Ok(Self::adopt(native, device, handle))
    }

    /// Whether this renderer can cast into the cube.
    pub fn is_supported(&self) -> Result<bool> {
        self.flag(self.native.engine.cube_shadow_map_is_supported)
    }

    /// Gives the cube the light it casts from.
    pub fn update(&self, light: PointLight) -> Result<()> {
        let handle = self.core.get()?;
        let light = light.to_native();
        // SAFETY: the handle is owned and the light is borrowed for the call.
        self.native
            .check(unsafe { (self.native.engine.cube_shadow_map_update)(handle, &light) })
    }

    /// Opens the shadow pass for one face, numbered as XNA numbers cube faces.
    pub fn begin_face(&self, face: i32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the index is by value.
        self.native
            .check(unsafe { (self.native.engine.cube_shadow_map_begin)(handle, face) })
    }

    /// Closes the face's shadow pass.
    pub fn end_face(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.cube_shadow_map_end)(handle) })
    }

    /// Where the light the cube was last updated for is.
    pub fn light_position(&self) -> Result<Vector3> {
        self.vector3(self.native.engine.cube_shadow_map_get_light_position)
    }

    /// How far that light reaches.
    pub fn light_range(&self) -> Result<f32> {
        self.scalar(self.native.engine.cube_shadow_map_get_light_range)
    }

    /// The cube's edge length in texels.
    pub fn size(&self) -> Result<i32> {
        self.count(self.native.engine.cube_shadow_map_get_size)
    }

    /// The quality preset the cube was created with.
    pub fn quality(&self) -> Result<ShadowQuality> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_ShadowQuality = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.cube_shadow_map_get_quality)(handle, &mut value)
        })?;
        ShadowQuality::from_native(value)
            .ok_or(CnaError::InvalidInput("native shadow quality is unknown"))
    }

    /// The depth bias applied when casting.
    pub fn depth_bias(&self) -> Result<f32> {
        self.scalar(self.native.engine.cube_shadow_map_get_depth_bias)
    }

    /// Sets the depth bias applied when casting.
    pub fn set_depth_bias(&self, value: f32) -> Result<()> {
        self.set_scalar(self.native.engine.cube_shadow_map_set_depth_bias, value)
    }

    /// A borrowed view of the cube's depth texture.
    pub fn shadow_texture(&self) -> Result<Option<BorrowedRenderTarget<'_>>> {
        self.borrowed_texture(self.native.engine.cube_shadow_map_get_shadow_texture)
    }

    /// The cube's caster effect, borrowed for as long as it lives.
    pub fn caster_effect(&self) -> Result<Option<BorrowedEffect<'_>>> {
        self.borrowed_effect(self.native.engine.cube_shadow_map_get_caster_effect)
    }

    /// The cube edge length a quality preset selects.
    pub fn size_for_quality(quality: ShadowQuality) -> Result<i32> {
        let native = Native::process()?;
        let mut value = 0_i32;
        // SAFETY: the identity is canonical and the output is a live local.
        native.check(unsafe {
            (native.engine.cube_shadow_map_size_for_quality)(quality.to_native(), &mut value)
        })?;
        Ok(value)
    }

    /// The projection every cube face casts with, for a light range.
    ///
    /// One ninety-degree frustum, so this is a pure function of the range.
    pub fn compute_face_projection(light_range: f32) -> Result<Matrix> {
        let native = Native::process()?;
        let mut value = sys::CNA_Matrix::default();
        // SAFETY: the range is by value and the output is a live local.
        native.check(unsafe {
            (native.engine.cube_shadow_map_compute_face_projection)(light_range, &mut value)
        })?;
        Ok(from_native_matrix(value))
    }

    /// The view transform one cube face casts with.
    pub fn compute_face_view(face: i32, light_position: Vector3) -> Result<Matrix> {
        let native = Native::process()?;
        let position = native_vector3(light_position);
        let mut value = sys::CNA_Matrix::default();
        // SAFETY: the position is borrowed for the call and the output is a
        // live local.
        native.check(unsafe {
            (native.engine.cube_shadow_map_compute_face_view)(
                face as u32,
                &position,
                &mut value,
            )
        })?;
        Ok(from_native_matrix(value))
    }
}

/// What a cascaded shadow map published for a receiver to read.
///
/// A typed Rust value over the ABI's fixed four-slot arrays: `count` says how
/// many of them a frame actually filled, and the rest is padding no shader
/// reads.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ShadowCascadeState {
    /// How many cascades the state describes.
    pub count: i32,
    /// How wide the blend between neighbouring cascades is.
    pub blend_band: f32,
    /// Each cascade's world-to-atlas transform.
    pub world_to_atlas: [Matrix; 4],
    /// Each cascade's far split distance.
    pub split_distance: [f32; 4],
    /// The camera view the splits were computed against.
    pub camera_view: Matrix,
    /// Whether each cascade is tinted for debugging.
    pub debug_tint: bool,
}

impl ShadowCascadeState {
    /// CNA's own defaults, asked of the library rather than restated here.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_ShadowCascadeStateEXT::default();
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.engine.shadow_cascade_state_ext_init)(&mut value) })?;
        Ok(Self {
            count: value.count,
            blend_band: value.blend_band,
            world_to_atlas: value.world_to_atlas.map(from_native_matrix),
            split_distance: value.split_distance,
            camera_view: from_native_matrix(value.camera_view),
            debug_tint: value.debug_tint != 0,
        })
    }
}

shadow_variant!(
    CascadedShadowMap,
    sys::CNA_CascadedShadowMapHandle,
    cascaded_shadow_map_destroy,
    "the cascaded shadow map has been released",
    "A cascaded directional shadow map: several splits into one atlas."
);

impl CascadedShadowMap {
    /// Creates a cascaded shadow map at a quality preset and cascade count.
    pub fn new(
        device: &GraphicsDevice,
        quality: ShadowQuality,
        cascade_count: i32,
    ) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.cascaded_shadow_map_create)(
                device.handle()?,
                quality.to_native(),
                cascade_count,
                &mut handle,
            )
        })?;
        Ok(Self::adopt(native, device, handle))
    }

    /// Whether this renderer can cast into the atlas.
    pub fn is_supported(&self) -> Result<bool> {
        self.flag(self.native.engine.cascaded_shadow_map_is_supported)
    }

    /// Recomputes every cascade for a light and a camera.
    pub fn update(
        &self,
        light: DirectionalLight,
        camera_view: Matrix,
        camera_projection: Matrix,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let light = light.to_native();
        let view = native_matrix(camera_view);
        let projection = native_matrix(camera_projection);
        // SAFETY: the handle is owned and all three structures are borrowed for
        // the call.
        self.native.check(unsafe {
            (self.native.engine.cascaded_shadow_map_update)(handle, &light, &view, &projection)
        })
    }

    /// Opens the shadow pass for one cascade.
    pub fn begin_cascade(&self, index: i32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the index is by value.
        self.native
            .check(unsafe { (self.native.engine.cascaded_shadow_map_begin)(handle, index) })
    }

    /// Closes the cascade's shadow pass.
    pub fn end_cascade(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.cascaded_shadow_map_end)(handle) })
    }

    /// Gives a receiving effect every cascade's transform and split.
    pub fn apply_to_receiver(&self, effect: &Effect) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: both handles are live for the call.
        self.native.check(unsafe {
            (self.native.engine.cascaded_shadow_map_apply_to_receiver)(
                handle,
                effect.native_handle()?,
            )
        })
    }

    /// How many cascades the map holds.
    pub fn cascade_count(&self) -> Result<i32> {
        self.count(self.native.engine.cascaded_shadow_map_get_cascade_count)
    }

    /// Each cascade's edge length in texels.
    pub fn cascade_size(&self) -> Result<i32> {
        self.count(self.native.engine.cascaded_shadow_map_get_cascade_size)
    }

    /// One cascade's world-to-atlas transform.
    pub fn cascade_matrix(&self, index: i32) -> Result<Matrix> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_Matrix::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.cascaded_shadow_map_get_cascade_matrix)(
                handle,
                index,
                &mut value,
            )
        })?;
        Ok(from_native_matrix(value))
    }

    /// One cascade's far split distance.
    pub fn split_distance(&self, index: i32) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.cascaded_shadow_map_get_split_distance)(handle, index, &mut value)
        })?;
        Ok(value)
    }

    /// Which cascade a view-space depth falls in.
    pub fn select_cascade(&self, view_depth: f32) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.cascaded_shadow_map_select_cascade)(handle, view_depth, &mut value)
        })?;
        Ok(value)
    }

    /// How the practical split scheme is weighted between uniform and logarithmic.
    pub fn split_lambda(&self) -> Result<f32> {
        self.scalar(self.native.engine.cascaded_shadow_map_get_split_lambda)
    }

    /// Sets how the split scheme is weighted.
    pub fn set_split_lambda(&self, value: f32) -> Result<()> {
        self.set_scalar(self.native.engine.cascaded_shadow_map_set_split_lambda, value)
    }

    /// How wide the blend between neighbouring cascades is.
    pub fn blend_band(&self) -> Result<f32> {
        self.scalar(self.native.engine.cascaded_shadow_map_get_blend_band)
    }

    /// Sets how wide that blend is.
    pub fn set_blend_band(&self, value: f32) -> Result<()> {
        self.set_scalar(self.native.engine.cascaded_shadow_map_set_blend_band, value)
    }

    /// Whether each cascade is tinted so it can be told apart.
    pub fn is_debug_tint_enabled(&self) -> Result<bool> {
        self.flag(self.native.engine.cascaded_shadow_map_is_debug_tint_enabled)
    }

    /// Turns the per-cascade debug tint on or off.
    pub fn set_debug_tint_enabled(&self, value: bool) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the flag is a canonical boolean.
        self.native.check(unsafe {
            (self.native.engine.cascaded_shadow_map_set_debug_tint_enabled)(
                handle,
                u8::from(value),
            )
        })
    }

    /// A borrowed view of the atlas.
    pub fn shadow_texture(&self) -> Result<Option<BorrowedRenderTarget<'_>>> {
        self.borrowed_texture(self.native.engine.cascaded_shadow_map_get_shadow_texture)
    }

    /// The map's caster effect, borrowed for as long as the map lives.
    pub fn caster_effect(&self) -> Result<Option<BorrowedEffect<'_>>> {
        self.borrowed_effect(self.native.engine.cascaded_shadow_map_get_caster_effect)
    }

    /// The practical split scheme's distances, without a map.
    pub fn compute_split_distances(
        near_plane: f32,
        far_plane: f32,
        cascade_count: i32,
        lambda: f32,
    ) -> Result<Vec<f32>> {
        let native = Native::process()?;
        let capacity = usize::try_from(cascade_count.max(0))
            .map_err(|_| CnaError::InvalidInput("the cascade count does not fit in memory"))?;
        let mut buffer = vec![0.0_f32; capacity];
        let mut count = 0_u64;
        // SAFETY: the destination holds `capacity` writable floats, which is
        // the count passed alongside it.
        native.check(unsafe {
            (native.engine.cascaded_shadow_map_compute_split_distances)(
                near_plane,
                far_plane,
                cascade_count,
                lambda,
                buffer.as_mut_ptr(),
                capacity as u64,
                &mut count,
            )
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported more splits than fit in memory"))?;
        buffer.truncate(count.min(capacity));
        Ok(buffer)
    }

    /// The eight world-space corners of a camera frustum.
    pub fn compute_frustum_corners(view: Matrix, projection: Matrix) -> Result<[Vector3; 8]> {
        let native = Native::process()?;
        let view = native_matrix(view);
        let projection = native_matrix(projection);
        let mut corners = [sys::CNA_Vector3::default(); 8];
        // SAFETY: both matrices are borrowed for the call and the destination
        // is the eight corners upstream documents.
        native.check(unsafe {
            (native.engine.cascaded_shadow_map_compute_frustum_corners)(
                &view,
                &projection,
                corners.as_mut_ptr(),
            )
        })?;
        Ok(corners.map(from_native_vector3))
    }

    /// The sphere that encloses eight corners.
    pub fn compute_bounding_sphere(corners: &[Vector3; 8]) -> Result<(Vector3, f32)> {
        let native = Native::process()?;
        let native_corners = corners.map(native_vector3);
        let mut centre = sys::CNA_Vector3::default();
        let mut radius = 0.0_f32;
        // SAFETY: the corners are borrowed for the call and both outputs are
        // live locals.
        native.check(unsafe {
            (native.engine.cascaded_shadow_map_compute_bounding_sphere)(
                native_corners.as_ptr(),
                &mut centre,
                &mut radius,
            )
        })?;
        Ok((from_native_vector3(centre), radius))
    }

    /// A centre snapped to the shadow map's texel grid.
    ///
    /// The pure function behind cascade stability: snapping is what stops a
    /// cascade's texels swimming as the camera moves.
    pub fn snap_to_texel_grid(centre: Vector3, radius: f32, size: i32) -> Result<Vector3> {
        let native = Native::process()?;
        let centre = native_vector3(centre);
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the centre is borrowed for the call and the output is a live local.
        native.check(unsafe {
            (native.engine.cascaded_shadow_map_snap_to_texel_grid)(
                &centre,
                radius,
                size,
                &mut value,
            )
        })?;
        Ok(from_native_vector3(value))
    }
}

/// Which kind of punctual light a [`PunctualLight`] describes.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum PunctualLightKind {
    /// No light at all.
    #[default]
    None,
    /// A point light.
    Point,
    /// A spot light.
    Spot,
}

impl PunctualLightKind {
    const fn from_native(value: sys::CNA_PunctualLightKindEXT) -> Option<Self> {
        Some(match value {
            sys::CNA_PUNCTUAL_LIGHT_KIND_EXT_NONE => Self::None,
            sys::CNA_PUNCTUAL_LIGHT_KIND_EXT_POINT => Self::Point,
            sys::CNA_PUNCTUAL_LIGHT_KIND_EXT_SPOT => Self::Spot,
            _ => return None,
        })
    }
}

/// One punctual light as a shading effect reads it.
///
/// The union of a point and a spot light, plus the shadow resources a receiver
/// samples. Its two shadow slots are non-owning handles upstream, so they are
/// reported as *presence* rather than published as values -- a safe type
/// holding a raw handle would be exactly the leak this crate refuses.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct PunctualLight {
    /// Which kind of light this is.
    pub kind: PunctualLightKind,
    /// World-space position.
    pub position: Vector3,
    /// The cone's axis, for a spot light.
    pub direction: Vector3,
    /// Linear RGB colour.
    pub diffuse_color: Vector3,
    /// Distance at which the light stops contributing.
    pub range: f32,
    /// The half angle the cone is at full brightness within.
    pub inner_angle: f32,
    /// The half angle the cone falls to nothing at.
    pub outer_angle: f32,
    /// The bias a receiver applies when sampling the shadow.
    pub shadow_depth_bias: f32,
    /// Whether a shadow cube is attached.
    pub has_shadow_cube: bool,
    /// Whether a shadow map is attached.
    pub has_shadow_map: bool,
    /// The transform a receiver samples the shadow map with.
    pub shadow_view_projection: Matrix,
}

impl PunctualLight {
    /// CNA's own defaults, asked of the library rather than restated here.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_PunctualLightEXT::default();
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.engine.punctual_light_ext_init)(&mut value) })?;
        Ok(Self {
            kind: PunctualLightKind::from_native(value.kind).ok_or(CnaError::InvalidInput(
                "native punctual light kind is unknown",
            ))?,
            position: from_native_vector3(value.position),
            direction: from_native_vector3(value.direction),
            diffuse_color: from_native_vector3(value.diffuse_color),
            range: value.range,
            inner_angle: value.inner_angle,
            outer_angle: value.outer_angle,
            shadow_depth_bias: value.shadow_depth_bias,
            has_shadow_cube: value.shadow_cube != sys::CNA_INVALID_HANDLE,
            has_shadow_map: value.shadow_map != sys::CNA_INVALID_HANDLE,
            shadow_view_projection: from_native_matrix(value.shadow_view_projection),
        })
    }
}

/// How a depth/normal prepass stores its linear depth.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DepthEncoding {
    /// Let the prepass choose what the renderer can do.
    #[default]
    Automatic,
    /// Four eight-bit channels packed together.
    Packed,
    /// A single half-float channel.
    HalfFloat,
}

impl DepthEncoding {
    const fn to_native(self) -> sys::CNA_DepthEncoding {
        match self {
            Self::Automatic => sys::CNA_DEPTH_ENCODING_AUTOMATIC,
            Self::Packed => sys::CNA_DEPTH_ENCODING_PACKED,
            Self::HalfFloat => sys::CNA_DEPTH_ENCODING_HALF_FLOAT,
        }
    }
}

/// A depth and normal prepass: the buffers SSAO, SSR and decals read.
///
/// `OWNED`. Whether the depth is packed into four channels or kept as a
/// half-float is a *renderer* answer under [`DepthEncoding::Automatic`], which
/// is why [`DepthNormalPrepass::is_depth_packed`] exists as its own question
/// rather than being inferred from what was asked for.
pub struct DepthNormalPrepass {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
    device: GraphicsDevice,
}

impl DepthNormalPrepass {
    /// Creates a prepass at a size and depth encoding.
    pub fn new(
        device: &GraphicsDevice,
        width: i32,
        height: i32,
        encoding: DepthEncoding,
    ) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.depth_normal_prepass_create)(
                device.handle()?,
                width,
                height,
                encoding.to_native(),
                &mut handle,
            )
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.depth_normal_prepass_destroy,
            released: "the depth/normal prepass has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
            device: device.clone(),
        })
    }

    /// Whether this renderer can run the prepass on a device.
    pub fn is_supported(&self, device: &GraphicsDevice) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: both handles are live and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.depth_normal_prepass_is_supported)(
                handle,
                device.handle()?,
                &mut value,
            )
        })?;
        Ok(value != 0)
    }

    /// Sizes the prepass's targets.
    pub fn resize(&self, width: i32, height: i32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the sizes are by value.
        self.native.check(unsafe {
            (self.native.engine.depth_normal_prepass_resize)(handle, width, height)
        })
    }

    /// Opens the prepass for one pass index and camera.
    pub fn begin(
        &self,
        pass_index: i32,
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
            (self.native.engine.depth_normal_prepass_begin)(
                handle,
                pass_index,
                &view,
                &projection,
                near_plane,
                far_plane,
            )
        })
    }

    /// Closes the prepass.
    pub fn end(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.depth_normal_prepass_end)(handle) })
    }

    /// How many passes the prepass needs on this renderer.
    ///
    /// One with multiple render targets, more without: the count is what a
    /// caller loops over, so it is read rather than assumed.
    pub fn pass_count(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.depth_normal_prepass_get_pass_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Whether the prepass writes depth and normals in one pass.
    pub fn is_using_multiple_render_targets(&self) -> Result<bool> {
        self.flag(self.native.engine.depth_normal_prepass_is_using_multiple_render_targets)
    }

    /// Whether the depth ended up packed into four channels.
    pub fn is_depth_packed(&self) -> Result<bool> {
        self.flag(self.native.engine.depth_normal_prepass_is_depth_packed)
    }

    /// Whether the velocity target is on.
    pub fn is_velocity_enabled(&self) -> Result<bool> {
        self.flag(self.native.engine.depth_normal_prepass_is_velocity_enabled_ext)
    }

    /// Turns the velocity target on or off.
    pub fn set_velocity_enabled(&self, value: bool) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the flag is a canonical boolean.
        self.native.check(unsafe {
            (self.native.engine.depth_normal_prepass_set_velocity_enabled_ext)(
                handle,
                u8::from(value),
            )
        })
    }

    /// The roughness the prepass writes alongside the normals.
    pub fn roughness(&self) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.depth_normal_prepass_get_roughness)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Sets the roughness the prepass writes.
    pub fn set_roughness(&self, value: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.native.check(unsafe {
            (self.native.engine.depth_normal_prepass_set_roughness)(handle, value)
        })
    }

    /// Gives the prepass the previous frame's camera, for velocity.
    pub fn set_previous_camera(&self, view: Matrix, projection: Matrix) -> Result<()> {
        let handle = self.core.get()?;
        let view = native_matrix(view);
        let projection = native_matrix(projection);
        // SAFETY: the handle is owned and both matrices are borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.depth_normal_prepass_set_previous_camera_ext)(
                handle,
                &view,
                &projection,
            )
        })
    }

    /// Gives the prepass the previous frame's world transform for the next draw.
    pub fn set_previous_world(&self, world: Matrix) -> Result<()> {
        let handle = self.core.get()?;
        let world = native_matrix(world);
        // SAFETY: the handle is owned and the matrix is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.depth_normal_prepass_set_previous_world_ext)(handle, &world)
        })
    }

    /// A borrowed view of the depth target.
    pub fn depth_texture(&self) -> Result<Option<BorrowedRenderTarget<'_>>> {
        self.borrowed_texture(self.native.engine.depth_normal_prepass_get_depth_texture)
    }

    /// A borrowed view of the normal target.
    pub fn normal_texture(&self) -> Result<Option<BorrowedRenderTarget<'_>>> {
        self.borrowed_texture(self.native.engine.depth_normal_prepass_get_normal_texture)
    }

    /// A borrowed view of the velocity target, when one is enabled.
    pub fn velocity_texture(&self) -> Result<Option<BorrowedRenderTarget<'_>>> {
        self.borrowed_texture(self.native.engine.depth_normal_prepass_get_velocity_texture_ext)
    }

    /// The prepass effect, borrowed for as long as the prepass lives.
    pub fn prepass_effect(&self) -> Result<Option<BorrowedEffect<'_>>> {
        self.borrowed_effect(self.native.engine.depth_normal_prepass_get_prepass_effect)
    }

    /// The skinned prepass effect, borrowed on the same terms.
    pub fn skinned_prepass_effect(&self) -> Result<Option<BorrowedEffect<'_>>> {
        self.borrowed_effect(self.native.engine.depth_normal_prepass_get_skinned_prepass_effect)
    }

    /// Releases the prepass now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }

    /// Packs a linear depth into four channel values.
    ///
    /// The inverse of [`DepthNormalPrepass::unpack_depth`], and the pair is the
    /// whole of what a packed-depth renderer relies on: a round trip that lost
    /// precision here would lose it in every shader that reads the buffer.
    pub fn pack_depth(value: f32) -> Result<(f32, f32, f32, f32)> {
        let native = Native::process()?;
        let (mut r, mut g, mut b, mut a) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
        // SAFETY: the value is by value and all four outputs are live locals.
        native.check(unsafe {
            (native.engine.depth_normal_prepass_pack_depth)(
                value, &mut r, &mut g, &mut b, &mut a,
            )
        })?;
        Ok((r, g, b, a))
    }

    /// Unpacks four channel values back into a linear depth.
    pub fn unpack_depth(r: f32, g: f32, b: f32, a: f32) -> Result<f32> {
        let native = Native::process()?;
        let mut value = 0.0_f32;
        // SAFETY: every input is by value and the output is a live local.
        native.check(unsafe {
            (native.engine.depth_normal_prepass_unpack_depth)(r, g, b, a, &mut value)
        })?;
        Ok(value)
    }

    /// Whether a device's prepass would use packed depth.
    pub fn uses_packed_depth(device: &GraphicsDevice) -> Result<bool> {
        let native = device.state_native();
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.depth_normal_prepass_uses_packed_depth_ext)(
                device.handle()?,
                &mut value,
            )
        })?;
        Ok(value != 0)
    }

    /// Whether an encoded velocity texel carries a velocity at all.
    pub fn has_velocity(texel: Color) -> Result<bool> {
        let native = Native::process()?;
        let texel = native_color(texel);
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the texel is by value and the output is a live local.
        native.check(unsafe {
            (native.engine.depth_normal_prepass_has_velocity_ext)(texel, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Decodes a velocity texel back into screen-space motion.
    pub fn decode_velocity(texel: Color) -> Result<Vector2> {
        let native = Native::process()?;
        let texel = native_color(texel);
        let mut value = sys::CNA_Vector2::default();
        // SAFETY: the texel is by value and the output is a live local.
        native.check(unsafe {
            (native.engine.depth_normal_prepass_decode_velocity_ext)(texel, &mut value)
        })?;
        Ok(Vector2::from_x_and_y(value.x, value.y))
    }

    /// The GLSL a shader includes to decode this prepass's depth.
    pub fn depth_decode_glsl(packed: bool) -> Result<String> {
        let native = Native::process()?;
        copy_text(&native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe {
                (api.depth_normal_prepass_copy_depth_decode_glsl)(
                    u8::from(packed),
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
    }

    /// The GLSL a shader includes to decode the velocity target.
    pub fn velocity_decode_glsl() -> Result<String> {
        let native = Native::process()?;
        copy_text(&native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe {
                (api.depth_normal_prepass_copy_velocity_decode_glsl)(
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
    }

    fn flag(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_DepthNormalPrepassHandle,
            *mut sys::CNA_Bool,
        ) -> sys::CNA_Result,
    ) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value != 0)
    }

    fn borrowed_texture(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_DepthNormalPrepassHandle,
            *mut sys::CNA_Handle,
        ) -> sys::CNA_Result,
    ) -> Result<Option<BorrowedRenderTarget<'_>>> {
        let handle = self.core.get()?;
        let mut texture = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut texture) })?;
        if texture == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        BorrowedRenderTarget::new(&self.native, &self.device, texture).map(Some)
    }

    fn borrowed_effect(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_DepthNormalPrepassHandle,
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

impl Drop for DepthNormalPrepass {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

fn native_color(value: Color) -> sys::CNA_Color {
    sys::CNA_Color {
        r: value.R(),
        g: value.G(),
        b: value.B(),
        a: value.A(),
    }
}

/// Weighted-blended order-independent transparency.
///
/// `OWNED`. Creation succeeds on a renderer that cannot run it; ask
/// [`WeightedBlendedTransparency::is_supported`] and, when it answers no,
/// [`WeightedBlendedTransparency::unsupported_reason`] for the reason the
/// pipeline's own fallback message quotes.
pub struct WeightedBlendedTransparency {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
    device: GraphicsDevice,
}

impl WeightedBlendedTransparency {
    /// Creates the accumulation targets at a size.
    pub fn new(device: &GraphicsDevice, width: i32, height: i32) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.weighted_blended_transparency_create)(
                device.handle()?,
                width,
                height,
                &mut handle,
            )
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.weighted_blended_transparency_destroy,
            released: "the weighted-blended transparency has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
            device: device.clone(),
        })
    }

    /// Whether this renderer can accumulate.
    pub fn is_supported(&self) -> Result<bool> {
        self.flag(self.native.engine.weighted_blended_transparency_is_supported)
    }

    /// Whether an accumulation is currently open.
    pub fn is_accumulating(&self) -> Result<bool> {
        self.flag(self.native.engine.weighted_blended_transparency_is_accumulating)
    }

    /// Why the renderer cannot accumulate; empty when it can.
    pub fn unsupported_reason(&self) -> Result<String> {
        let handle = self.core.get()?;
        copy_text(&self.native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe {
                (api.weighted_blended_transparency_copy_unsupported_reason)(
                    handle,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
    }

    /// Sizes the accumulation targets.
    pub fn resize(&self, width: i32, height: i32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the sizes are by value.
        self.native.check(unsafe {
            (self.native.engine.weighted_blended_transparency_resize)(handle, width, height)
        })
    }

    /// Opens the accumulation, for a camera far plane.
    pub fn begin(&self, far_plane: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.native.check(unsafe {
            (self.native.engine.weighted_blended_transparency_begin)(handle, far_plane)
        })
    }

    /// Closes the accumulation.
    pub fn end(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.weighted_blended_transparency_end)(handle) })
    }

    /// Resolves the accumulation into whatever target is bound.
    pub fn resolve(&self, width: i32, height: i32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the sizes are by value.
        self.native.check(unsafe {
            (self.native.engine.weighted_blended_transparency_resolve)(handle, width, height)
        })
    }

    /// A borrowed view of the accumulation target.
    pub fn accumulation_texture(&self) -> Result<Option<BorrowedRenderTarget<'_>>> {
        self.borrowed_texture(
            self.native
                .engine
                .weighted_blended_transparency_get_accumulation_texture_ext,
        )
    }

    /// A borrowed view of the revealage target.
    pub fn revealage_texture(&self) -> Result<Option<BorrowedRenderTarget<'_>>> {
        self.borrowed_texture(
            self.native
                .engine
                .weighted_blended_transparency_get_revealage_texture_ext,
        )
    }

    /// Releases the targets now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }

    /// The weight one fragment contributes, from its alpha and depth.
    ///
    /// The technique's whole weighting function, as a pure value: a nearer
    /// fragment must weigh more than a farther one at the same alpha, and that
    /// is what makes the result order-independent.
    pub fn weight(view_depth: f32, alpha: f32, far_plane: f32) -> Result<f32> {
        let native = Native::process()?;
        let mut value = 0.0_f32;
        // SAFETY: every input is by value and the output is a live local.
        native.check(unsafe {
            (native.engine.weighted_blended_transparency_weight)(
                view_depth, alpha, far_plane, &mut value,
            )
        })?;
        Ok(value)
    }

    /// The accumulation shader's own GLSL.
    pub fn accumulation_glsl() -> Result<String> {
        let native = Native::process()?;
        copy_text(&native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe {
                (api.weighted_blended_transparency_copy_accumulation_glsl)(
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
    }

    fn flag(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_WeightedBlendedTransparencyHandle,
            *mut sys::CNA_Bool,
        ) -> sys::CNA_Result,
    ) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value != 0)
    }

    fn borrowed_texture(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_WeightedBlendedTransparencyHandle,
            *mut sys::CNA_Handle,
        ) -> sys::CNA_Result,
    ) -> Result<Option<BorrowedRenderTarget<'_>>> {
        let handle = self.core.get()?;
        let mut texture = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut texture) })?;
        if texture == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        BorrowedRenderTarget::new(&self.native, &self.device, texture).map(Some)
    }
}

impl Drop for WeightedBlendedTransparency {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// The sorted-draw path for transparency, when order matters.
///
/// `OWNED`, and device-free: the list is bookkeeping, so it needs no graphics
/// device and is not registered against one.
pub struct TransparentDrawList {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
    entries: Vec<Arc<SceneCallback>>,
}

impl TransparentDrawList {
    /// Creates an empty list.
    pub fn new() -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local.
        native.check(unsafe { (native.engine.transparent_draw_list_create)(&mut handle) })?;
        Ok(Self {
            core: Arc::new(EngineHandle {
                native: Arc::clone(&native),
                handle: Mutex::new(handle),
                destroy: native.engine.transparent_draw_list_destroy,
                released: "the transparent draw list has been released",
            }),
            native,
            entries: Vec::new(),
        })
    }

    /// Submits one entry with the bounds it occupies.
    ///
    /// The callback runs during [`TransparentDrawList::draw_sorted`], in the
    /// order the list decides. CNA keeps the raw context, so the closure is
    /// retained here for as long as the entry is in the list.
    pub fn submit(
        &mut self,
        bounds: BoundingBox,
        draw: impl FnMut() -> Result<()> + 'static,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let bounds = native_bounds(bounds);
        let entry = SceneCallback::new(draw);
        // SAFETY: the handle is owned, the bounds are borrowed for the call and
        // the context is retained below.
        self.native.check(unsafe {
            (self.native.engine.transparent_draw_list_submit)(
                handle,
                &bounds,
                Some(scene_trampoline),
                entry.context(),
            )
        })?;
        self.entries.push(entry);
        Ok(())
    }

    /// How many entries the list holds.
    pub fn count(&self) -> Result<u64> {
        let handle = self.core.get()?;
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.transparent_draw_list_get_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Removes every entry.
    pub fn clear(&mut self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.transparent_draw_list_clear)(handle) })?;
        self.entries.clear();
        Ok(())
    }

    /// Draws every entry back to front for a camera.
    ///
    /// A callback that failed or panicked is reported as the Rust cause it had.
    pub fn draw_sorted(&self, view: Matrix) -> Result<()> {
        let handle = self.core.get()?;
        let view = native_matrix(view);
        // SAFETY: the handle is owned, the matrix is borrowed for the call, and
        // every registered callback is alive because this value holds it.
        let result = self.native.check(unsafe {
            (self.native.engine.transparent_draw_list_draw_sorted)(handle, &view)
        });
        for entry in &self.entries {
            if let Some(failure) = entry.take_failure() {
                return Err(failure);
            }
        }
        result
    }

    /// The order the list would draw its entries in, without drawing them.
    pub fn sorted_order(&self, view: Matrix) -> Result<Vec<i32>> {
        let handle = self.core.get()?;
        let view = native_matrix(view);
        let capacity = usize::try_from(self.count()?)
            .map_err(|_| CnaError::InvalidInput("the entry count does not fit in memory"))?;
        let mut buffer = vec![0_i32; capacity];
        let mut count = 0_u64;
        // SAFETY: the handle is owned, the matrix is borrowed for the call and
        // the destination holds `capacity` writable indices.
        self.native.check(unsafe {
            (self.native.engine.transparent_draw_list_copy_sorted_order_ext)(
                handle,
                &view,
                buffer.as_mut_ptr(),
                capacity as u64,
                &mut count,
            )
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported more entries than fit in memory"))?;
        buffer.truncate(count.min(capacity));
        Ok(buffer)
    }

    /// Releases the list now rather than at drop.
    pub fn release(&mut self) -> Result<()> {
        let result = self.core.release();
        self.entries.clear();
        result
    }

    /// The camera position a view matrix implies.
    pub fn camera_position_of(view: Matrix) -> Result<Vector3> {
        let native = Native::process()?;
        let view = native_matrix(view);
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the matrix is borrowed for the call and the output is a live local.
        native.check(unsafe {
            (native.engine.transparent_draw_list_camera_position_of)(&view, &mut value)
        })?;
        Ok(from_native_vector3(value))
    }

    /// The key the list sorts one entry by.
    ///
    /// A pure function of the bounds and the camera, so the ordering is
    /// predictable without submitting anything.
    pub fn sort_key(bounds: BoundingBox, camera_position: Vector3) -> Result<f32> {
        let native = Native::process()?;
        let bounds = native_bounds(bounds);
        let camera = native_vector3(camera_position);
        let mut value = 0.0_f32;
        // SAFETY: both inputs are borrowed for the call and the output is a
        // live local.
        native.check(unsafe {
            (native.engine.transparent_draw_list_sort_key)(&bounds, &camera, &mut value)
        })?;
        Ok(value)
    }
}

impl Drop for TransparentDrawList {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// Which colour space a display expects.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DisplayColorSpace {
    /// Ordinary sRGB, the only one a non-HDR display has.
    #[default]
    Srgb,
    /// Linear scRGB, where values above one are brighter than white.
    ScRgb,
    /// HDR10: Rec.2020 primaries with the PQ transfer function.
    Hdr10,
}

impl DisplayColorSpace {
    const fn from_native(value: sys::CNA_DisplayColorSpace) -> Option<Self> {
        Some(match value {
            sys::CNA_DISPLAY_COLOR_SPACE_SRGB => Self::Srgb,
            sys::CNA_DISPLAY_COLOR_SPACE_SCRGB => Self::ScRgb,
            sys::CNA_DISPLAY_COLOR_SPACE_HDR10 => Self::Hdr10,
            _ => return None,
        })
    }

    const fn to_native(self) -> sys::CNA_DisplayColorSpace {
        match self {
            Self::Srgb => sys::CNA_DISPLAY_COLOR_SPACE_SRGB,
            Self::ScRgb => sys::CNA_DISPLAY_COLOR_SPACE_SCRGB,
            Self::Hdr10 => sys::CNA_DISPLAY_COLOR_SPACE_HDR10,
        }
    }
}

/// The final encode into whatever the display expects.
///
/// `OWNED`. Creation succeeds on a display that is not HDR at all; ask
/// [`HdrDisplayOutput::is_supported`] rather than reading success as capability.
pub struct HdrDisplayOutput {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl HdrDisplayOutput {
    /// Creates the output on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.hdr_display_output_create)(device.handle()?, &mut handle)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.hdr_display_output_destroy,
            released: "the HDR display output has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// Whether this renderer and display can present HDR at all.
    pub fn is_supported(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.hdr_display_output_is_supported)(handle, &mut value) })?;
        Ok(value != 0)
    }

    /// The colour space the output encodes into.
    pub fn color_space(&self) -> Result<DisplayColorSpace> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_DisplayColorSpace = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.hdr_display_output_get_color_space)(handle, &mut value)
        })?;
        DisplayColorSpace::from_native(value)
            .ok_or(CnaError::InvalidInput("native display colour space is unknown"))
    }

    /// Sets the colour space the output encodes into.
    pub fn set_color_space(&self, value: DisplayColorSpace) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the identity is canonical.
        self.native.check(unsafe {
            (self.native.engine.hdr_display_output_set_color_space)(handle, value.to_native())
        })
    }

    /// How bright diffuse white is, in nits.
    pub fn paper_white_nits(&self) -> Result<f32> {
        self.scalar(self.native.engine.hdr_display_output_get_paper_white_nits)
    }

    /// Sets how bright diffuse white is.
    pub fn set_paper_white_nits(&self, value: f32) -> Result<()> {
        self.set_scalar(self.native.engine.hdr_display_output_set_paper_white_nits, value)
    }

    /// The brightest the display can go, in nits.
    pub fn peak_nits(&self) -> Result<f32> {
        self.scalar(self.native.engine.hdr_display_output_get_peak_nits)
    }

    /// Sets the brightest the display can go.
    pub fn set_peak_nits(&self, value: f32) -> Result<()> {
        self.set_scalar(self.native.engine.hdr_display_output_set_peak_nits, value)
    }

    /// Encodes a scene-referred source into the bound target.
    pub fn draw(&self, source: &Texture2D, width: i32, height: i32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned, the texture handle is live and the
        // destination is the currently bound target.
        self.native.check(unsafe {
            (self.native.engine.hdr_display_output_draw)(
                handle,
                source.handle()?,
                sys::CNA_INVALID_HANDLE,
                width,
                height,
            )
        })
    }

    /// Releases the output now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }

    /// Encodes one linear colour for a colour space, as the shader would.
    ///
    /// A pure function, so the whole encode is checkable without a display that
    /// can show it -- which matters here more than anywhere else, because no
    /// display on this host can.
    pub fn encode(
        space: DisplayColorSpace,
        linear: Vector3,
        paper_white_nits: f32,
        peak_nits: f32,
    ) -> Result<Vector3> {
        let native = Native::process()?;
        let linear = native_vector3(linear);
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the colour is borrowed for the call and the output is a live local.
        native.check(unsafe {
            (native.engine.hdr_display_output_encode)(
                space.to_native(),
                &linear,
                paper_white_nits,
                peak_nits,
                &mut value,
            )
        })?;
        Ok(from_native_vector3(value))
    }

    /// The PQ transfer function, from nits to a signal value.
    pub fn encode_pq(nits: f32) -> Result<f32> {
        let native = Native::process()?;
        let mut value = 0.0_f32;
        // SAFETY: the value is by value and the output is a live local.
        native.check(unsafe { (native.engine.hdr_display_output_encode_pq)(nits, &mut value) })?;
        Ok(value)
    }

    /// The inverse PQ transfer function, from a signal value back to nits.
    pub fn decode_pq(signal: f32) -> Result<f32> {
        let native = Native::process()?;
        let mut value = 0.0_f32;
        // SAFETY: the value is by value and the output is a live local.
        native.check(unsafe { (native.engine.hdr_display_output_decode_pq)(signal, &mut value) })?;
        Ok(value)
    }

    /// Converts a Rec.709 colour into Rec.2020 primaries.
    pub fn rec709_to_rec2020(color: Vector3) -> Result<Vector3> {
        let native = Native::process()?;
        let color = native_vector3(color);
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the colour is borrowed for the call and the output is a live local.
        native.check(unsafe {
            (native.engine.hdr_display_output_rec709_to_rec2020)(&color, &mut value)
        })?;
        Ok(from_native_vector3(value))
    }

    /// Rolls a value off towards a peak, so nothing clips hard.
    pub fn roll_off(value: f32, peak: f32) -> Result<f32> {
        let native = Native::process()?;
        let mut out = 0.0_f32;
        // SAFETY: both inputs are by value and the output is a live local.
        native.check(unsafe { (native.engine.hdr_display_output_roll_off)(value, peak, &mut out) })?;
        Ok(out)
    }

    fn scalar(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_HdrDisplayOutputHandle,
            *mut f32,
        ) -> sys::CNA_Result,
    ) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value)
    }

    fn set_scalar(
        &self,
        route: unsafe extern "C" fn(sys::CNA_HdrDisplayOutputHandle, f32) -> sys::CNA_Result,
        value: f32,
    ) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.native.check(unsafe { route(handle, value) })
    }
}

impl Drop for HdrDisplayOutput {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// Eye adaptation: an exposure that follows the scene's own brightness.
pub struct AutoExposure {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl AutoExposure {
    /// Creates the adaptation on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe {
            (native.engine.auto_exposure_ext_create)(device.handle()?, &mut handle)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.auto_exposure_ext_destroy,
            released: "the auto exposure has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// The exposure the adaptation has settled on.
    pub fn exposure(&self) -> Result<f32> {
        self.scalar(self.native.engine.auto_exposure_ext_get_exposure)
    }

    /// Sets the exposure directly, as a starting point.
    pub fn set_exposure(&self, value: f32) -> Result<()> {
        self.set_scalar(self.native.engine.auto_exposure_ext_set_exposure, value)
    }

    /// The middle-grey the adaptation aims the average luminance at.
    pub fn key_value(&self) -> Result<f32> {
        self.scalar(self.native.engine.auto_exposure_ext_get_key_value)
    }

    /// Sets the middle-grey it aims for.
    pub fn set_key_value(&self, value: f32) -> Result<()> {
        self.set_scalar(self.native.engine.auto_exposure_ext_set_key_value, value)
    }

    /// How quickly the exposure rises when the scene darkens.
    pub fn brightening_speed(&self) -> Result<f32> {
        self.scalar(self.native.engine.auto_exposure_ext_get_brightening_speed)
    }

    /// How quickly it falls when the scene brightens.
    pub fn darkening_speed(&self) -> Result<f32> {
        self.scalar(self.native.engine.auto_exposure_ext_get_darkening_speed)
    }

    /// Sets both adaptation speeds.
    ///
    /// One route rather than two, because upstream sets them together: an eye
    /// that brightened and darkened at unrelated rates would be a different
    /// model, not a differently configured one.
    pub fn set_adaptation_speeds(&self, brightening: f32, darkening: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and both values are by value.
        self.native.check(unsafe {
            (self.native.engine.auto_exposure_ext_set_adaptation_speeds)(
                handle,
                brightening,
                darkening,
            )
        })
    }

    /// Bounds the exposure the adaptation may reach.
    pub fn set_exposure_range(&self, minimum: f32, maximum: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and both values are by value.
        self.native.check(unsafe {
            (self.native.engine.auto_exposure_ext_set_exposure_range)(handle, minimum, maximum)
        })
    }

    /// The average luminance of a scene texture, as the adaptation measures it.
    pub fn measure_average_luminance(&self, scene: &Texture2D) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: both handles are live and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.auto_exposure_ext_measure_average_luminance)(
                handle,
                scene.handle()?,
                &mut value,
            )
        })?;
        Ok(value)
    }

    /// Advances the adaptation by one frame, answering the new exposure.
    pub fn update(&self, scene: &Texture2D, elapsed_seconds: f32) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: both handles are live and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.auto_exposure_ext_update)(
                handle,
                scene.handle()?,
                elapsed_seconds,
                &mut value,
            )
        })?;
        Ok(value)
    }

    /// Writes the settled exposure into a pipeline's settings.
    pub fn apply_to(&self, settings: &mut EngineRenderSettings) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the structure is updated in place.
        self.native.check(unsafe {
            (self.native.engine.auto_exposure_ext_apply_to)(handle, settings.as_native_mut())
        })
    }

    /// Releases the adaptation now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }

    fn scalar(
        &self,
        route: unsafe extern "C" fn(sys::CNA_AutoExposureHandle, *mut f32) -> sys::CNA_Result,
    ) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value)
    }

    fn set_scalar(
        &self,
        route: unsafe extern "C" fn(sys::CNA_AutoExposureHandle, f32) -> sys::CNA_Result,
        value: f32,
    ) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.native.check(unsafe { route(handle, value) })
    }
}

impl Drop for AutoExposure {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// A parsed Adobe `.cube` colour lookup table.
///
/// `OWNED`, and device-free: parsing is text work, so the table exists before
/// any device does and is not registered against one. The textures it builds
/// *are* the caller's, handed over outright rather than borrowed.
pub struct CubeLut {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl CubeLut {
    /// Parses a `.cube` document from text.
    pub fn parse(text: &str) -> Result<Self> {
        let native = Native::process()?;
        let view = string_view(text);
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the text is borrowed for the call and the output is a live local.
        native.check(unsafe { (native.engine.cube_lut_parse)(view, &mut handle) })?;
        Ok(Self::adopt(&native, handle))
    }

    /// Reads and parses a `.cube` file.
    pub fn load_from_file(path: &str) -> Result<Self> {
        let native = Native::process()?;
        let view = string_view(path);
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the path is borrowed for the call and the output is a live local.
        native.check(unsafe { (native.engine.cube_lut_load_from_file)(view, &mut handle) })?;
        Ok(Self::adopt(&native, handle))
    }

    fn adopt(native: &Arc<Native>, handle: sys::CNA_Handle) -> Self {
        Self {
            core: Arc::new(EngineHandle {
                native: Arc::clone(native),
                handle: Mutex::new(handle),
                destroy: native.engine.cube_lut_destroy,
                released: "the cube LUT has been released",
            }),
            native: Arc::clone(native),
        }
    }

    /// The table's edge length in entries.
    pub fn size(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.cube_lut_get_size)(handle, &mut value) })?;
        Ok(value)
    }

    /// The table's own title, as the document declared it.
    pub fn title(&self) -> Result<String> {
        let handle = self.core.get()?;
        copy_text(&self.native, |api, destination, capacity, out_bytes| {
            // SAFETY: the destination holds `capacity` writable bytes.
            unsafe { (api.cube_lut_copy_title)(handle, destination, capacity, out_bytes) }
        })
    }

    /// One entry, by its three grid indices.
    pub fn entry(&self, red: i32, green: i32, blue: i32) -> Result<Vector3> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.cube_lut_get_entry)(handle, red, green, blue, &mut value)
        })?;
        Ok(from_native_vector3(value))
    }

    /// The lowest input the table maps.
    pub fn domain_min(&self) -> Result<Vector3> {
        self.vector3(self.native.engine.cube_lut_get_domain_min)
    }

    /// The highest input the table maps.
    pub fn domain_max(&self) -> Result<Vector3> {
        self.vector3(self.native.engine.cube_lut_get_domain_max)
    }

    /// Whether the domain is the unit cube, which a shader can skip rescaling.
    pub fn is_unit_domain(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.cube_lut_is_unit_domain)(handle, &mut value) })?;
        Ok(value != 0)
    }

    /// Builds the strip texture a colour-grade pass samples.
    ///
    /// The texture is the caller's: CNA allocates it and hands it over.
    pub fn create_strip_texture(&self, device: &GraphicsDevice) -> Result<Texture2D> {
        let handle = self.core.get()?;
        let mut texture = sys::CNA_INVALID_HANDLE;
        // SAFETY: both handles are live and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.cube_lut_create_strip_texture)(
                handle,
                device.handle()?,
                &mut texture,
            )
        })?;
        Texture2D::from_owned_handle(device, texture)
    }

    /// Builds the volume texture a colour-grade pass samples.
    pub fn create_volume_texture(&self, device: &GraphicsDevice) -> Result<Texture3D> {
        let handle = self.core.get()?;
        let mut texture = sys::CNA_INVALID_HANDLE;
        // SAFETY: both handles are live and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.cube_lut_create_volume_texture)(
                handle,
                device.handle()?,
                &mut texture,
            )
        })?;
        Texture3D::from_owned_handle(device, texture)
    }

    /// Releases the table now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }

    fn vector3(
        &self,
        route: unsafe extern "C" fn(
            sys::CNA_CubeLutHandle,
            *mut sys::CNA_Vector3,
        ) -> sys::CNA_Result,
    ) -> Result<Vector3> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(from_native_vector3(value))
    }
}

impl Drop for CubeLut {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// Immediate-mode line drawing for diagnostics.
///
/// `OWNED`. Everything it draws is a line list, so
/// [`DebugDraw::line_count`] and [`DebugDraw::vertices`] are exact values: a
/// box is twelve lines, a cross is three, and a sphere is however many
/// segments it was asked for, three times over.
pub struct DebugDraw {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl DebugDraw {
    /// Creates the debug drawer on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live and the output is a live local.
        native.check(unsafe { (native.engine.debug_draw_create)(device.handle()?, &mut handle) })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(handle),
            destroy: native.engine.debug_draw_destroy,
            released: "the debug drawer has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// Opens a batch for a camera.
    pub fn begin(&self, view: Matrix, projection: Matrix) -> Result<()> {
        let handle = self.core.get()?;
        let view = native_matrix(view);
        let projection = native_matrix(projection);
        // SAFETY: the handle is owned and both matrices are borrowed for the call.
        self.native
            .check(unsafe { (self.native.engine.debug_draw_begin)(handle, &view, &projection) })
    }

    /// Draws the batch and closes it.
    pub fn end(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.debug_draw_end)(handle) })
    }

    /// Discards everything queued without drawing it.
    pub fn clear(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.debug_draw_clear)(handle) })
    }

    /// How many lines are queued.
    pub fn line_count(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.debug_draw_get_line_count)(handle, &mut value) })?;
        Ok(value)
    }

    /// Whether the lines are depth tested against the scene.
    pub fn is_depth_tested(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.debug_draw_is_depth_tested)(handle, &mut value) })?;
        Ok(value != 0)
    }

    /// Turns depth testing on or off.
    pub fn set_depth_tested(&self, value: bool) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the flag is a canonical boolean.
        self.native.check(unsafe {
            (self.native.engine.debug_draw_set_depth_tested)(handle, u8::from(value))
        })
    }

    /// Queues one line.
    pub fn add_line(&self, from: Vector3, to: Vector3, color: Color) -> Result<()> {
        let handle = self.core.get()?;
        let from = native_vector3(from);
        let to = native_vector3(to);
        // SAFETY: the handle is owned and both points are borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.debug_draw_add_line)(handle, &from, &to, native_color(color))
        })
    }

    /// Queues the twelve edges of a box.
    pub fn add_box(&self, bounds: BoundingBox, color: Color) -> Result<()> {
        let handle = self.core.get()?;
        let bounds = native_bounds(bounds);
        // SAFETY: the handle is owned and the bounds are borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.debug_draw_add_box)(handle, &bounds, native_color(color))
        })
    }

    /// Queues three axis-aligned segments through a point.
    pub fn add_cross(&self, centre: Vector3, size: f32, color: Color) -> Result<()> {
        let handle = self.core.get()?;
        let centre = native_vector3(centre);
        // SAFETY: the handle is owned and the point is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.debug_draw_add_cross)(handle, &centre, size, native_color(color))
        })
    }

    /// Queues three rings approximating a sphere.
    pub fn add_sphere(
        &self,
        centre: Vector3,
        radius: f32,
        color: Color,
        segments: i32,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let centre = native_vector3(centre);
        // SAFETY: the handle is owned and the point is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.debug_draw_add_sphere)(
                handle,
                &centre,
                radius,
                native_color(color),
                segments,
            )
        })
    }

    /// Queues the same three rings around a bounding sphere.
    pub fn add_bounding_sphere(
        &self,
        sphere: BoundingSphere,
        color: Color,
        segments: i32,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let sphere = sys::CNA_BoundingSphere {
            center: native_vector3(sphere.Center),
            radius: sphere.Radius,
        };
        // SAFETY: the handle is owned and the sphere is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.debug_draw_add_bounding_sphere)(
                handle,
                &sphere,
                native_color(color),
                segments,
            )
        })
    }

    /// Queues a camera frustum's own twelve edges.
    pub fn add_frustum(&self, frustum: &BoundingFrustum, color: Color) -> Result<()> {
        let handle = self.core.get()?;
        let frustum = sys::CNA_BoundingFrustum {
            matrix: native_matrix(frustum.Matrix()),
        };
        // SAFETY: the handle is owned and the frustum is by value.
        self.native.check(unsafe {
            (self.native.engine.debug_draw_add_frustum)(handle, frustum, native_color(color))
        })
    }

    /// Queues a gizmo showing where a directional light points.
    pub fn add_directional_light_gizmo(
        &self,
        light: DirectionalLight,
        origin: Vector3,
        length: f32,
        color: Color,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let light = light.to_native();
        let origin = native_vector3(origin);
        // SAFETY: the handle is owned and both structures are borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.debug_draw_add_directional_light_gizmo)(
                handle,
                &light,
                &origin,
                length,
                native_color(color),
            )
        })
    }

    /// Queues a gizmo showing a point light's position and range.
    pub fn add_point_light_gizmo(&self, light: PointLight, color: Color) -> Result<()> {
        let handle = self.core.get()?;
        let light = light.to_native();
        // SAFETY: the handle is owned and the light is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.debug_draw_add_point_light_gizmo)(
                handle,
                &light,
                native_color(color),
            )
        })
    }

    /// Queues a gizmo showing a spot light's cone.
    pub fn add_spot_light_gizmo(
        &self,
        light: SpotLight,
        color: Color,
        segments: i32,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let light = light.to_native();
        // SAFETY: the handle is owned and the light is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.debug_draw_add_spot_light_gizmo)(
                handle,
                &light,
                native_color(color),
                segments,
            )
        })
    }

    /// Queues a gizmo showing each of a cascaded shadow map's splits.
    pub fn add_cascade_gizmo(&self, map: &CascadedShadowMap, color: Color) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: both handles are live for the call.
        self.native.check(unsafe {
            (self.native.engine.debug_draw_add_cascade_gizmo)(
                handle,
                map.core.get()?,
                native_color(color),
            )
        })
    }

    /// The queued lines as vertices, ready to draw elsewhere.
    ///
    /// `depth_tested` selects which of the two queues to read: the drawer keeps
    /// them apart because they need different device state, and a caller
    /// reading only one and finding it short would otherwise have no way to
    /// tell which.
    pub fn vertices(&self, depth_tested: bool) -> Result<Vec<VertexPositionColor>> {
        let handle = self.core.get()?;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        let probe = unsafe {
            (self.native.engine.debug_draw_copy_vertices)(
                handle,
                u8::from(depth_tested),
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        };
        if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.native.check(probe)?;
        }
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("the vertex count does not fit in memory"))?;
        if capacity == 0 {
            return Ok(Vec::new());
        }
        let mut buffer = vec![sys::CNA_VertexPositionColor::default(); capacity];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `capacity`
        // writable vertices, which is the count passed alongside it.
        self.native.check(unsafe {
            (self.native.engine.debug_draw_copy_vertices)(
                handle,
                u8::from(depth_tested),
                buffer.as_mut_ptr(),
                required,
                &mut count,
            )
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported more vertices than fit in memory"))?;
        Ok(buffer
            .into_iter()
            .take(count.min(capacity))
            .map(|vertex| VertexPositionColor {
                Position: from_native_vector3(vertex.position),
                Color: Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
                    i32::from(vertex.color.r),
                    i32::from(vertex.color.g),
                    i32::from(vertex.color.b),
                    i32::from(vertex.color.a),
                ),
            })
            .collect())
    }

    /// Releases the drawer now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }
}

impl Drop for DebugDraw {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// A camera frustum, and the visibility questions it answers.
///
/// `OWNED`, and device-free: culling is arithmetic, so the culler needs no
/// graphics device and is not registered against one.
pub struct FrustumCuller {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl FrustumCuller {
    /// Creates a culler with no camera yet.
    pub fn new() -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local.
        native.check(unsafe { (native.engine.frustum_culler_ext_create)(&mut handle) })?;
        Ok(Self {
            core: Arc::new(EngineHandle {
                native: Arc::clone(&native),
                handle: Mutex::new(handle),
                destroy: native.engine.frustum_culler_ext_destroy,
                released: "the frustum culler has been released",
            }),
            native,
        })
    }

    /// Sets the camera from a view and a projection.
    pub fn set_camera(&self, view: Matrix, projection: Matrix) -> Result<()> {
        let handle = self.core.get()?;
        let view = native_matrix(view);
        let projection = native_matrix(projection);
        // SAFETY: the handle is owned and both matrices are borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.frustum_culler_ext_set_camera)(handle, &view, &projection)
        })
    }

    /// Sets the camera from a combined view-projection.
    pub fn set_view_projection(&self, value: Matrix) -> Result<()> {
        let handle = self.core.get()?;
        let value = native_matrix(value);
        // SAFETY: the handle is owned and the matrix is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.frustum_culler_ext_set_view_projection)(handle, &value)
        })
    }

    /// The frustum the culler is testing against.
    pub fn frustum(&self) -> Result<BoundingFrustum> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_BoundingFrustum::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.frustum_culler_ext_get_frustum)(handle, &mut value)
        })?;
        Ok(BoundingFrustum::new(from_native_matrix(value.matrix)))
    }

    /// Whether one box is at least partly inside the frustum.
    pub fn is_box_visible(&self, bounds: BoundingBox) -> Result<bool> {
        let handle = self.core.get()?;
        let bounds = native_bounds(bounds);
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned, the bounds are borrowed for the call and
        // the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.frustum_culler_ext_is_box_visible)(handle, &bounds, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Whether one sphere is at least partly inside the frustum.
    pub fn is_sphere_visible(&self, sphere: BoundingSphere) -> Result<bool> {
        let handle = self.core.get()?;
        let sphere = sys::CNA_BoundingSphere {
            center: native_vector3(sphere.Center),
            radius: sphere.Radius,
        };
        let mut value: sys::CNA_Bool = 0;
        // SAFETY: the handle is owned, the sphere is borrowed for the call and
        // the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.frustum_culler_ext_is_sphere_visible)(handle, &sphere, &mut value)
        })?;
        Ok(value != 0)
    }

    /// The indices of the boxes that survive the frustum, in order.
    pub fn cull_boxes(&self, boxes: &[BoundingBox]) -> Result<Vec<u64>> {
        let handle = self.core.get()?;
        let native_boxes: Vec<sys::CNA_BoundingBox> =
            boxes.iter().copied().map(native_bounds).collect();
        let mut visible = vec![0_u64; boxes.len()];
        let mut count = 0_u64;
        // SAFETY: the handle is owned, the input is borrowed for the call with
        // its own length, and the destination holds one index per input.
        self.native.check(unsafe {
            (self.native.engine.frustum_culler_ext_cull_boxes)(
                handle,
                native_boxes.as_ptr(),
                native_boxes.len() as u64,
                visible.as_mut_ptr(),
                visible.len() as u64,
                &mut count,
            )
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported more indices than fit in memory"))?;
        visible.truncate(count.min(boxes.len()));
        Ok(visible)
    }

    /// The indices of the spheres that survive the frustum, in order.
    pub fn cull_spheres(&self, spheres: &[BoundingSphere]) -> Result<Vec<u64>> {
        let handle = self.core.get()?;
        let native_spheres: Vec<sys::CNA_BoundingSphere> = spheres
            .iter()
            .map(|sphere| sys::CNA_BoundingSphere {
                center: native_vector3(sphere.Center),
                radius: sphere.Radius,
            })
            .collect();
        let mut visible = vec![0_u64; spheres.len()];
        let mut count = 0_u64;
        // SAFETY: the handle is owned, the input is borrowed for the call with
        // its own length, and the destination holds one index per input.
        self.native.check(unsafe {
            (self.native.engine.frustum_culler_ext_cull_spheres)(
                handle,
                native_spheres.as_ptr(),
                native_spheres.len() as u64,
                visible.as_mut_ptr(),
                visible.len() as u64,
                &mut count,
            )
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported more indices than fit in memory"))?;
        visible.truncate(count.min(spheres.len()));
        Ok(visible)
    }

    /// The transforms whose bounds survive the frustum, in order.
    ///
    /// The instancing path: many world transforms, each paired with the world
    /// bounds of the instance it places, and what comes back is the transforms
    /// themselves rather than their indices, ready to upload.
    ///
    /// `bounds` is parallel to `transforms` and may be **shorter**. CNA's rule
    /// for the tail is that a transform with no bound of its own is *kept*, not
    /// dropped, which is the opposite of what a caller who passed a short array
    /// by accident would expect; it is stated in CNA's own header and passed
    /// through here unchanged rather than papered over.
    pub fn cull_transforms(
        &self,
        transforms: &[Matrix],
        bounds: &[BoundingBox],
    ) -> Result<Vec<Matrix>> {
        let handle = self.core.get()?;
        let native_transforms: Vec<sys::CNA_Matrix> =
            transforms.iter().copied().map(native_matrix).collect();
        let bounds: Vec<sys::CNA_BoundingBox> =
            bounds.iter().copied().map(native_bounds).collect();
        let mut visible = vec![sys::CNA_Matrix::default(); transforms.len()];
        let mut count = 0_u64;
        // SAFETY: the handle is owned, both inputs are borrowed for the call
        // with their own lengths, and the destination holds one transform per
        // input.
        self.native.check(unsafe {
            (self.native.engine.frustum_culler_ext_cull_transforms)(
                handle,
                native_transforms.as_ptr(),
                native_transforms.len() as u64,
                bounds.as_ptr(),
                bounds.len() as u64,
                visible.as_mut_ptr(),
                visible.len() as u64,
                &mut count,
            )
        })?;
        let count = usize::try_from(count).map_err(|_| {
            CnaError::InvalidInput("CNA reported more transforms than fit in memory")
        })?;
        visible.truncate(count.min(transforms.len()));
        Ok(visible.into_iter().map(from_native_matrix).collect())
    }

    /// Releases the culler now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }
}

impl Drop for FrustumCuller {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// How a level-of-detail group decides which level to use.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum LodSelectionMode {
    /// By distance alone.
    #[default]
    Distance,
    /// By how many pixels the object covers.
    ScreenSpaceError,
}

impl LodSelectionMode {
    const fn from_native(value: sys::CNA_LodSelectionMode) -> Option<Self> {
        Some(match value {
            sys::CNA_LOD_SELECTION_MODE_DISTANCE => Self::Distance,
            sys::CNA_LOD_SELECTION_MODE_SCREEN_SPACE_ERROR => Self::ScreenSpaceError,
            _ => return None,
        })
    }

    const fn to_native(self) -> sys::CNA_LodSelectionMode {
        match self {
            Self::Distance => sys::CNA_LOD_SELECTION_MODE_DISTANCE,
            Self::ScreenSpaceError => sys::CNA_LOD_SELECTION_MODE_SCREEN_SPACE_ERROR,
        }
    }
}

/// One level in a [`LodGroup`].
///
/// The mesh part is reported as *presence* rather than published: CNA names it
/// with a `ModelMeshPart` handle, and this crate's `ModelMeshPart` is a managed
/// projection with no native handle of its own, so a value carrying one would
/// be a raw handle in the safe API.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct LodLevel {
    /// The distance beyond which the next level takes over.
    pub max_distance: f32,
    /// Whether a mesh part is attached to this level.
    pub has_part: bool,
}

/// A set of detail levels and the rule that picks between them.
///
/// `OWNED`, and device-free. Levels may be added without a mesh part, which is
/// what makes the selection rule usable from Rust: a caller adds the distances,
/// asks [`LodGroup::select_index`] which level a distance falls in, and keeps
/// its own table of what to draw for each.
pub struct LodGroup {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl LodGroup {
    /// Creates an empty group.
    pub fn new() -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local.
        native.check(unsafe { (native.engine.lod_group_ext_create)(&mut handle) })?;
        Ok(Self {
            core: Arc::new(EngineHandle {
                native: Arc::clone(&native),
                handle: Mutex::new(handle),
                destroy: native.engine.lod_group_ext_destroy,
                released: "the LOD group has been released",
            }),
            native,
        })
    }

    /// Adds a level that takes over out to a distance.
    ///
    /// The mesh part upstream also accepts is left out deliberately: see
    /// [`LodLevel`].
    pub fn add_level(&self, max_distance: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned; an invalid part handle is upstream's
        // documented "no part", and the distance is by value.
        self.native.check(unsafe {
            (self.native.engine.lod_group_ext_add_level)(
                handle,
                max_distance,
                sys::CNA_INVALID_HANDLE,
            )
        })
    }

    /// Removes every level.
    pub fn clear(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.lod_group_ext_clear)(handle) })
    }

    /// Every level, in the order they were added.
    pub fn levels(&self) -> Result<Vec<LodLevel>> {
        let handle = self.core.get()?;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        let probe = unsafe {
            (self.native.engine.lod_group_ext_copy_levels)(
                handle,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        };
        if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.native.check(probe)?;
        }
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("the level count does not fit in memory"))?;
        if capacity == 0 {
            return Ok(Vec::new());
        }
        let mut buffer = vec![sys::CNA_LodLevelEXT::default(); capacity];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `capacity`
        // writable levels, which is the count passed alongside it.
        self.native.check(unsafe {
            (self.native.engine.lod_group_ext_copy_levels)(
                handle,
                buffer.as_mut_ptr(),
                required,
                &mut count,
            )
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported more levels than fit in memory"))?;
        Ok(buffer
            .into_iter()
            .take(count.min(capacity))
            .map(|level| LodLevel {
                max_distance: level.max_distance,
                has_part: level.part != sys::CNA_INVALID_HANDLE,
            })
            .collect())
    }

    /// Which level a distance falls in, or `-1` for none.
    ///
    /// The boundary is **exclusive**: a level added with `max_distance` covers
    /// distances strictly below it, so a distance sitting exactly on a boundary
    /// belongs to the *next* level. Past the last boundary the answer is `-1`,
    /// meaning "draw nothing at all" -- the group does not fall back to its
    /// coarsest level. A negative distance is clamped to zero rather than
    /// refused, and an empty group answers `-1`.
    ///
    /// This call is **not** a pure query: it remembers the level it chose, and
    /// [`set_hysteresis`](Self::set_hysteresis) makes the next call sticky near
    /// that level's boundary. Call [`reset_hysteresis`](Self::reset_hysteresis)
    /// to ask again without that memory.
    pub fn select_index(&self, distance: f32) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.lod_group_ext_select_index)(handle, distance, &mut value)
        })?;
        Ok(value)
    }

    /// How the group decides which level to use.
    pub fn selection_mode(&self) -> Result<LodSelectionMode> {
        let handle = self.core.get()?;
        let mut value: sys::CNA_LodSelectionMode = 0;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.lod_group_ext_get_selection_mode)(handle, &mut value)
        })?;
        LodSelectionMode::from_native(value)
            .ok_or(CnaError::InvalidInput("native LOD selection mode is unknown"))
    }

    /// Sets how the group decides.
    pub fn set_selection_mode(&self, value: LodSelectionMode) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the identity is canonical.
        self.native.check(unsafe {
            (self.native.engine.lod_group_ext_set_selection_mode)(handle, value.to_native())
        })
    }

    /// How far past a boundary the group holds its current level.
    ///
    /// Hysteresis is what stops an object at a boundary flickering between two
    /// levels, so it is state rather than a pure rule -- which is why
    /// [`LodGroup::reset_hysteresis`] exists.
    pub fn hysteresis(&self) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.engine.lod_group_ext_get_hysteresis)(handle, &mut value) })?;
        Ok(value)
    }

    /// Sets how far past a boundary the group holds its level.
    pub fn set_hysteresis(&self, value: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and the value is by value.
        self.native
            .check(unsafe { (self.native.engine.lod_group_ext_set_hysteresis)(handle, value) })
    }

    /// Forgets which level was last selected.
    pub fn reset_hysteresis(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.lod_group_ext_reset_hysteresis)(handle) })
    }

    /// The camera the screen-space rule measures against.
    pub fn set_screen_space_parameters(
        &self,
        object_radius: f32,
        vertical_field_of_view: f32,
        viewport_height: f32,
    ) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and every value is by value.
        self.native.check(unsafe {
            (self.native.engine.lod_group_ext_set_screen_space_parameters)(
                handle,
                object_radius,
                vertical_field_of_view,
                viewport_height,
            )
        })
    }

    /// How many pixels the object covers at a distance.
    pub fn projected_radius_pixels(&self, distance: f32) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.lod_group_ext_projected_radius_pixels)(handle, distance, &mut value)
        })?;
        Ok(value)
    }

    /// Releases the group now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }
}

impl Drop for LodGroup {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// Which kind of light a [`ClusteredLight`] is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ClusteredLightType {
    /// A point light: a sphere of colour around a position.
    Point,
    /// A spot light: a cone of colour from a position along a direction.
    Spot,
}

impl ClusteredLightType {
    const fn to_native(self) -> sys::CNA_ClusteredLightType {
        match self {
            Self::Point => sys::CNA_CLUSTERED_LIGHT_TYPE_POINT,
            Self::Spot => sys::CNA_CLUSTERED_LIGHT_TYPE_SPOT,
        }
    }

    const fn from_native(value: sys::CNA_ClusteredLightType) -> Option<Self> {
        match value {
            sys::CNA_CLUSTERED_LIGHT_TYPE_POINT => Some(Self::Point),
            sys::CNA_CLUSTERED_LIGHT_TYPE_SPOT => Some(Self::Spot),
            _ => None,
        }
    }
}

/// One light in a [`ClusteredLightSet`].
///
/// A value, not a resource: a set holds copies, so a light read back out stays
/// correct after the set changes and nothing has to be released.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ClusteredLight {
    /// Which kind of light this is.
    pub kind: ClusteredLightType,
    /// Whether this light should be given a shadow.
    pub casts_shadows: bool,
    /// World-space position.
    pub position: Vector3,
    /// The direction a spot light points; ignored for a point light.
    pub direction: Vector3,
    /// Linear RGB colour.
    pub color: Vector3,
    /// Scalar multiplier on [`ClusteredLight::color`]; must not be negative.
    pub intensity: f32,
    /// Distance at which the light stops contributing; must be positive.
    pub range: f32,
    /// The half angle a spot light is at full strength within, in radians.
    pub inner_angle: f32,
    /// The half angle a spot light has fallen to nothing at, in radians.
    pub outer_angle: f32,
}

impl ClusteredLight {
    /// The greatest number of lights one [`ClusteredLightSet`] holds.
    pub const SET_MAX: i32 = sys::CNA_CLUSTERED_LIGHT_SET_MAX_EXT;

    /// CNA's own defaults, asked of the library rather than restated here.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_ClusteredLightEXT::default();
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.engine.clustered_light_ext_init)(&mut value) })?;
        Self::from_native(value)
    }

    /// Whether a set would accept this light.
    ///
    /// The same test [`ClusteredLightSet::add`] applies, exposed so a caller
    /// can ask before being refused: a positive range, a non-negative finite
    /// intensity, finite vectors, and for a spot light a non-degenerate
    /// direction whose inner angle is no wider than its outer.
    pub fn is_usable(&self) -> Result<bool> {
        let native = Native::process()?;
        let value = self.to_native();
        let mut usable = 0_u8;
        // SAFETY: the light is borrowed for the call and the output is a live
        // local.
        native
            .check(unsafe { (native.engine.clustered_light_set_is_usable)(&value, &mut usable) })?;
        Ok(usable != 0)
    }

    fn to_native(self) -> sys::CNA_ClusteredLightEXT {
        sys::CNA_ClusteredLightEXT {
            struct_size: core::mem::size_of::<sys::CNA_ClusteredLightEXT>() as u32,
            struct_version: 1,
            r#type: self.kind.to_native(),
            casts_shadows: u8::from(self.casts_shadows),
            reserved: [0; 3],
            position: native_vector3(self.position),
            direction: native_vector3(self.direction),
            color: native_vector3(self.color),
            intensity: self.intensity,
            range: self.range,
            inner_angle: self.inner_angle,
            outer_angle: self.outer_angle,
        }
    }

    fn from_native(value: sys::CNA_ClusteredLightEXT) -> Result<Self> {
        Ok(Self {
            kind: ClusteredLightType::from_native(value.r#type)
                .ok_or(CnaError::InvalidInput("native clustered light type is unknown"))?,
            casts_shadows: value.casts_shadows != 0,
            position: from_native_vector3(value.position),
            direction: from_native_vector3(value.direction),
            color: from_native_vector3(value.color),
            intensity: value.intensity,
            range: value.range,
            inner_angle: value.inner_angle,
            outer_angle: value.outer_angle,
        })
    }
}

/// The lights a clustered renderer sorts into its grid.
///
/// `OWNED`. The set holds values, so nothing it returns keeps it alive and
/// nothing it lends has to be released; it needs no device, but CNA parents it
/// to the game so its lifetime is accounted for like any other owned resource,
/// which is why it registers with the device the way every other engine object
/// here does.
pub struct ClusteredLightSet {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl ClusteredLightSet {
    /// Creates an empty set.
    ///
    /// The argument is the **graphics device**, not the game, despite CNA's own
    /// header naming the parameter `game` and describing it as "the owning
    /// game": the implementation resolves it with `GetBorrowedGraphicsDevice`
    /// and takes the game from the device. Passing the game handle answers
    /// "the graphics-device handle is invalid for this call". The same is true
    /// of [`ClusteredLightGrid`] and [`ClusteredLightAssignment`].
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut set = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live for the call and the output is a
        // live local.
        native
            .check(unsafe { (native.engine.clustered_light_set_create)(device.handle()?, &mut set) })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(set),
            destroy: native.engine.clustered_light_set_destroy,
            released: "the clustered light set has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// Adds a light, answering the index it landed at.
    pub fn add(&self, light: ClusteredLight) -> Result<i32> {
        let handle = self.core.get()?;
        let value = light.to_native();
        let mut index = 0_i32;
        // SAFETY: the handle is owned, the light is borrowed for the call, and
        // the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_set_add)(handle, &value, &mut index)
        })?;
        Ok(index)
    }

    /// Adds a [`PointLight`], converted to a clustered light.
    pub fn add_point(&self, light: PointLight) -> Result<i32> {
        let handle = self.core.get()?;
        let value = light.to_native();
        let mut index = 0_i32;
        // SAFETY: the handle is owned, the light is borrowed for the call, and
        // the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_set_add_point)(handle, &value, &mut index)
        })?;
        Ok(index)
    }

    /// Adds a [`SpotLight`], converted to a clustered light.
    pub fn add_spot(&self, light: SpotLight) -> Result<i32> {
        let handle = self.core.get()?;
        let value = light.to_native();
        let mut index = 0_i32;
        // SAFETY: the handle is owned, the light is borrowed for the call, and
        // the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_set_add_spot)(handle, &value, &mut index)
        })?;
        Ok(index)
    }

    /// Replaces the light at an index.
    pub fn replace_at(&self, index: i32, light: ClusteredLight) -> Result<()> {
        let handle = self.core.get()?;
        let value = light.to_native();
        // SAFETY: the handle is owned and the light is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_set_replace_at)(handle, index, &value)
        })
    }

    /// Removes the light at an index, shifting the ones after it down.
    pub fn remove_at(&self, index: i32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.clustered_light_set_remove_at)(handle, index) })
    }

    /// Removes every light.
    pub fn clear(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.clustered_light_set_clear)(handle) })
    }

    /// How many lights the set holds.
    pub fn count(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_set_get_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Whether the set holds no lights.
    pub fn is_empty(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value = 0_u8;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_set_is_empty)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// A copy of the light at an index.
    pub fn get(&self, index: i32) -> Result<ClusteredLight> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_ClusteredLightEXT::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_set_get_at)(handle, index, &mut value)
        })?;
        ClusteredLight::from_native(value)
    }

    /// A copy of every light the set holds.
    pub fn lights(&self) -> Result<Vec<ClusteredLight>> {
        let handle = self.core.get()?;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        let probe = unsafe {
            (self.native.engine.clustered_light_set_copy_lights)(
                handle,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        };
        if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.native.check(probe)?;
        }
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("the light count does not fit in memory"))?;
        if capacity == 0 {
            return Ok(Vec::new());
        }
        let mut buffer = vec![sys::CNA_ClusteredLightEXT::default(); capacity];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `capacity`
        // writable lights, which is the count passed alongside it.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_set_copy_lights)(
                handle,
                buffer.as_mut_ptr(),
                required,
                &mut count,
            )
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported more lights than fit in memory"))?;
        buffer
            .into_iter()
            .take(count.min(capacity))
            .map(ClusteredLight::from_native)
            .collect()
    }

    /// The bounding sphere of the light at an index.
    pub fn bounds_at(&self, index: i32) -> Result<BoundingSphere> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_BoundingSphere::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_set_get_bounds_at)(handle, index, &mut value)
        })?;
        Ok(BoundingSphere {
            Center: from_native_vector3(value.center),
            Radius: value.radius,
        })
    }

    /// The bounding sphere of every light, in light-index order.
    ///
    /// This is what [`ClusteredLightAssignment::assign`] sorts, so a caller
    /// sorts the set it already built rather than describing the lights twice.
    pub fn bounds(&self) -> Result<Vec<BoundingSphere>> {
        let handle = self.core.get()?;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        let probe = unsafe {
            (self.native.engine.clustered_light_set_copy_bounds)(
                handle,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        };
        if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.native.check(probe)?;
        }
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("the sphere count does not fit in memory"))?;
        if capacity == 0 {
            return Ok(Vec::new());
        }
        let mut buffer = vec![sys::CNA_BoundingSphere::default(); capacity];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `capacity`
        // writable spheres, which is the count passed alongside it.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_set_copy_bounds)(
                handle,
                buffer.as_mut_ptr(),
                required,
                &mut count,
            )
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported more spheres than fit in memory"))?;
        Ok(buffer
            .into_iter()
            .take(count.min(capacity))
            .map(|sphere| BoundingSphere {
                Center: from_native_vector3(sphere.center),
                Radius: sphere.radius,
            })
            .collect())
    }

    /// Releases the set now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }
}

impl Drop for ClusteredLightSet {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// The view frustum cut into clusters a light list is sorted into.
///
/// `OWNED`, and a pure CPU object: it is parented to the game only so its
/// lifetime is accounted for. Until [`set_projection`](Self::set_projection) it
/// has no shape, and [`cluster_bounds`](Self::cluster_bounds) refuses.
pub struct ClusteredLightGrid {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl ClusteredLightGrid {
    /// The most tiles a grid takes along either screen axis.
    pub const MAX_TILES_PER_AXIS: i32 = sys::CNA_CLUSTER_GRID_MAX_TILES_PER_AXIS_EXT;
    /// The most depth slices a grid takes.
    pub const MAX_SLICE_COUNT: i32 = sys::CNA_CLUSTER_GRID_MAX_SLICE_COUNT_EXT;
    /// The default tile count along X.
    pub const DEFAULT_TILES_X: i32 = sys::CNA_CLUSTER_GRID_DEFAULT_TILES_X_EXT;
    /// The default tile count along Y.
    pub const DEFAULT_TILES_Y: i32 = sys::CNA_CLUSTER_GRID_DEFAULT_TILES_Y_EXT;
    /// The default depth-slice count.
    pub const DEFAULT_SLICE_COUNT: i32 = sys::CNA_CLUSTER_GRID_DEFAULT_SLICE_COUNT_EXT;

    /// Creates a grid of the given shape.
    ///
    /// A dimension outside its range is refused rather than clamped: the
    /// cluster count is what the light-index list is sized from.
    pub fn new(
        device: &GraphicsDevice,
        tiles_x: i32,
        tiles_y: i32,
        slice_count: i32,
    ) -> Result<Self> {
        let native = device.state_native();
        let mut grid = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live for the call and the output is a
        // live local.
        native.check(unsafe {
            (native.engine.clustered_light_grid_create)(
                device.handle()?,
                tiles_x,
                tiles_y,
                slice_count,
                &mut grid,
            )
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(grid),
            destroy: native.engine.clustered_light_grid_destroy,
            released: "the cluster grid has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// Creates a grid of CNA's default shape.
    pub fn with_canonical_shape(device: &GraphicsDevice) -> Result<Self> {
        Self::new(
            device,
            Self::DEFAULT_TILES_X,
            Self::DEFAULT_TILES_Y,
            Self::DEFAULT_SLICE_COUNT,
        )
    }

    fn count(&self, route: unsafe extern "C" fn(sys::CNA_Handle, *mut i32) -> sys::CNA_Result) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value)
    }

    /// Tiles along X.
    pub fn tiles_x(&self) -> Result<i32> {
        self.count(self.native.engine.clustered_light_grid_get_tiles_x)
    }

    /// Tiles along Y.
    pub fn tiles_y(&self) -> Result<i32> {
        self.count(self.native.engine.clustered_light_grid_get_tiles_y)
    }

    /// Depth slices.
    pub fn slice_count(&self) -> Result<i32> {
        self.count(self.native.engine.clustered_light_grid_get_slice_count)
    }

    /// How many clusters the grid holds.
    pub fn cluster_count(&self) -> Result<i32> {
        self.count(self.native.engine.clustered_light_grid_get_cluster_count)
    }

    /// The flat index of a cluster coordinate.
    pub fn cluster_index(&self, x: i32, y: i32, slice: i32) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_grid_cluster_index)(handle, x, y, slice, &mut value)
        })?;
        Ok(value)
    }

    /// Gives the grid its shape from a camera projection.
    ///
    /// The slice spacing is logarithmic in the ratio of the two planes, so a
    /// zero near plane has no logarithm and an inverted pair has no grid: both
    /// are refused rather than clamped.
    pub fn set_projection(&self, projection: Matrix, near_plane: f32, far_plane: f32) -> Result<()> {
        let handle = self.core.get()?;
        let projection = native_matrix(projection);
        // SAFETY: the handle is owned and the matrix is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_grid_set_projection)(
                handle,
                &projection,
                near_plane,
                far_plane,
            )
        })
    }

    /// Whether a projection has been set.
    pub fn has_projection(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value = 0_u8;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_grid_has_projection)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// The near distance the grid was given.
    pub fn near_plane(&self) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_grid_get_near_plane)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// The far distance the grid was given.
    pub fn far_plane(&self) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_grid_get_far_plane)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// The inverse of the projection the grid was given.
    pub fn inverse_projection(&self) -> Result<Matrix> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_Matrix::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_grid_get_inverse_projection)(handle, &mut value)
        })?;
        Ok(from_native_matrix(value))
    }

    /// The view distance a depth-slice boundary sits at.
    ///
    /// The slice count itself is a valid argument and names the far edge of the
    /// last slice: there is **one more boundary than slice**. The answer is
    /// zero until a projection has been set.
    pub fn slice_distance(&self, slice: i32) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_grid_slice_distance)(handle, slice, &mut value)
        })?;
        Ok(value)
    }

    /// Which slice covers a view distance.
    ///
    /// **Clamped, not refused**: a point in front of the near plane belongs to
    /// the first slice and one beyond the far plane to the last, which is what
    /// a renderer wants when a light straddles the frustum edge.
    pub fn slice_for_view_distance(&self, view_distance: f32) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_grid_slice_for_view_distance)(
                handle,
                view_distance,
                &mut value,
            )
        })?;
        Ok(value)
    }

    /// The view-space bounds of one cluster.
    pub fn cluster_bounds(&self, x: i32, y: i32, slice: i32) -> Result<BoundingBox> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_BoundingBox::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_grid_cluster_bounds)(
                handle,
                x,
                y,
                slice,
                &mut value,
            )
        })?;
        Ok(from_native_bounds(value))
    }

    /// Releases the grid now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }
}

impl Drop for ClusteredLightGrid {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// Which lights reach which clusters.
///
/// `OWNED`, and a pure CPU object. Its index and offset arrays are read by
/// copy, so nothing it returns keeps it alive.
pub struct ClusteredLightAssignment {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl ClusteredLightAssignment {
    /// The most lights one assignment sorts.
    pub const MAX_LIGHTS: i32 = sys::CNA_CLUSTERED_ASSIGNMENT_MAX_LIGHTS_EXT;

    /// Creates an empty assignment.
    ///
    /// Takes the graphics device, for the reason
    /// [`ClusteredLightSet::new`] gives.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut assignment = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live for the call and the output is a
        // live local.
        native.check(unsafe {
            (native.engine.clustered_light_assignment_create)(device.handle()?, &mut assignment)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(assignment),
            destroy: native.engine.clustered_light_assignment_destroy,
            released: "the clustered light assignment has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// Sorts light bounds into a grid's clusters.
    ///
    /// The bounds are what [`ClusteredLightSet::bounds`] produces, in
    /// light-index order. The grid must already have a projection.
    pub fn assign(
        &self,
        grid: &ClusteredLightGrid,
        view: Matrix,
        bounds: &[BoundingSphere],
    ) -> Result<()> {
        let handle = self.core.get()?;
        let grid_handle = grid.core.get()?;
        let view = native_matrix(view);
        let native_bounds: Vec<sys::CNA_BoundingSphere> = bounds
            .iter()
            .map(|sphere| sys::CNA_BoundingSphere {
                center: native_vector3(sphere.Center),
                radius: sphere.Radius,
            })
            .collect();
        // SAFETY: both handles are owned, and the matrix and the sphere array
        // are borrowed for the call with the array's own length.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_assignment_assign)(
                handle,
                grid_handle,
                &view,
                native_bounds.as_ptr(),
                native_bounds.len() as u64,
            )
        })
    }

    /// Forgets every assignment.
    pub fn clear(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.clustered_light_assignment_clear)(handle) })
    }

    /// Takes an assignment computed elsewhere -- on the GPU, or by a caller's
    /// own sorter.
    ///
    /// `offsets` says where each cluster's run of indices begins, so there is
    /// **one more offset than cluster**: it must start at zero, never go
    /// backwards, and end at `indices.len()`. Every index must name a light
    /// below `light_count`.
    pub fn adopt(&self, light_count: i32, offsets: &[i32], indices: &[i32]) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned and both arrays are borrowed for the call
        // with their own lengths.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_assignment_adopt)(
                handle,
                light_count,
                offsets.as_ptr(),
                offsets.len() as u64,
                indices.as_ptr(),
                indices.len() as u64,
            )
        })
    }

    fn count(&self, route: unsafe extern "C" fn(sys::CNA_Handle, *mut i32) -> sys::CNA_Result) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe { route(handle, &mut value) })?;
        Ok(value)
    }

    /// How many lights the assignment describes.
    pub fn light_count(&self) -> Result<i32> {
        self.count(self.native.engine.clustered_light_assignment_get_light_count)
    }

    /// How many clusters the assignment describes.
    pub fn cluster_count(&self) -> Result<i32> {
        self.count(self.native.engine.clustered_light_assignment_get_cluster_count)
    }

    /// How many light references the assignment holds in total.
    pub fn total_reference_count(&self) -> Result<i32> {
        self.count(
            self.native
                .engine
                .clustered_light_assignment_get_total_reference_count,
        )
    }

    /// The largest number of lights any one cluster holds.
    ///
    /// The number worth watching: it sizes the shader's per-cluster loop.
    pub fn max_lights_per_cluster(&self) -> Result<i32> {
        self.count(
            self.native
                .engine
                .clustered_light_assignment_get_max_lights_per_cluster,
        )
    }

    fn copy_i32(
        &self,
        route: unsafe extern "C" fn(sys::CNA_Handle, *mut i32, u64, *mut u64) -> sys::CNA_Result,
        what: &'static str,
    ) -> Result<Vec<i32>> {
        let handle = self.core.get()?;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        let probe = unsafe { route(handle, core::ptr::null_mut(), 0, &mut required) };
        if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.native.check(probe)?;
        }
        let capacity = usize::try_from(required).map_err(|_| CnaError::InvalidInput(what))?;
        if capacity == 0 {
            return Ok(Vec::new());
        }
        let mut buffer = vec![0_i32; capacity];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `capacity`
        // writable values, which is the count passed alongside it.
        self.native
            .check(unsafe { route(handle, buffer.as_mut_ptr(), required, &mut count) })?;
        let count = usize::try_from(count).map_err(|_| CnaError::InvalidInput(what))?;
        buffer.truncate(count.min(capacity));
        Ok(buffer)
    }

    /// The light indices assigned to one cluster.
    pub fn lights_in_cluster(&self, cluster_index: i32) -> Result<Vec<i32>> {
        let handle = self.core.get()?;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        let probe = unsafe {
            (self
                .native
                .engine
                .clustered_light_assignment_copy_lights_in_cluster)(
                handle,
                cluster_index,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        };
        if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.native.check(probe)?;
        }
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("the cluster's light count does not fit in memory"))?;
        if capacity == 0 {
            return Ok(Vec::new());
        }
        let mut buffer = vec![0_i32; capacity];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `capacity`
        // writable indices, which is the count passed alongside it.
        self.native.check(unsafe {
            (self
                .native
                .engine
                .clustered_light_assignment_copy_lights_in_cluster)(
                handle,
                cluster_index,
                buffer.as_mut_ptr(),
                required,
                &mut count,
            )
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported more indices than fit in memory"))?;
        buffer.truncate(count.min(capacity));
        Ok(buffer)
    }

    /// The whole index array, cluster runs back to back.
    pub fn indices(&self) -> Result<Vec<i32>> {
        self.copy_i32(
            self.native.engine.clustered_light_assignment_copy_indices,
            "the index count does not fit in memory",
        )
    }

    /// The whole offset array: one more entry than the cluster count.
    pub fn offsets(&self) -> Result<Vec<i32>> {
        self.copy_i32(
            self.native.engine.clustered_light_assignment_copy_offsets,
            "the offset count does not fit in memory",
        )
    }

    /// Releases the assignment now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }
}

impl Drop for ClusteredLightAssignment {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// Which of a scene's lights are allowed to cast shadows this frame.
///
/// `OWNED`, and a pure CPU object. A scene may hold hundreds of clustered
/// lights and a renderer can afford shadow maps for a handful, so the policy
/// scores them and admits the best few; the hysteresis margin is what stops two
/// similarly-scored lights swapping the same slot every frame.
///
/// Takes the graphics device, for the reason [`ClusteredLightSet::new`] gives.
pub struct ClusteredShadowPolicy {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl ClusteredShadowPolicy {
    /// How many shadow-casting lights a policy admits unless told otherwise.
    pub const DEFAULT_BUDGET: i32 = sys::CNA_CLUSTERED_SHADOW_DEFAULT_BUDGET_EXT;
    /// The score margin a light must beat to displace one already selected.
    pub const DEFAULT_HYSTERESIS: f32 = sys::CNA_CLUSTERED_SHADOW_DEFAULT_HYSTERESIS_EXT;

    /// Creates a policy with a shadow budget.
    pub fn new(device: &GraphicsDevice, budget: i32) -> Result<Self> {
        let native = device.state_native();
        let mut policy = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live for the call and the output is a
        // live local.
        native.check(unsafe {
            (native.engine.clustered_shadow_policy_create)(device.handle()?, budget, &mut policy)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(policy),
            destroy: native.engine.clustered_shadow_policy_destroy,
            released: "the clustered shadow policy has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// Scores a light set and selects which of its lights may cast.
    pub fn select(
        &self,
        lights: &ClusteredLightSet,
        view: Matrix,
        projection: Matrix,
        camera_position: Vector3,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let set = lights.core.get()?;
        let view = native_matrix(view);
        let projection = native_matrix(projection);
        let camera_position = native_vector3(camera_position);
        // SAFETY: both handles are owned, and the matrices and the position are
        // borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.clustered_shadow_policy_select)(
                handle,
                set,
                &view,
                &projection,
                &camera_position,
            )
        })
    }

    /// How many lights may cast shadows at once.
    pub fn budget(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_shadow_policy_get_budget)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Sets how many lights may cast shadows at once.
    pub fn set_budget(&self, budget: i32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.clustered_shadow_policy_set_budget)(handle, budget) })
    }

    /// The margin a light must beat to displace one already selected.
    pub fn hysteresis(&self) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_shadow_policy_get_hysteresis)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Sets that margin.
    pub fn set_hysteresis(&self, hysteresis: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self.native.engine.clustered_shadow_policy_set_hysteresis)(handle, hysteresis)
        })
    }

    /// The indices of the lights currently admitted.
    pub fn selected(&self) -> Result<Vec<i32>> {
        let handle = self.core.get()?;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        let probe = unsafe {
            (self.native.engine.clustered_shadow_policy_copy_selected)(
                handle,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        };
        if probe != sys::CNA_RESULT_SUCCESS && probe != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.native.check(probe)?;
        }
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("the selection does not fit in memory"))?;
        if capacity == 0 {
            return Ok(Vec::new());
        }
        let mut buffer = vec![0_i32; capacity];
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the destination holds `capacity`
        // writable indices, which is the count passed alongside it.
        self.native.check(unsafe {
            (self.native.engine.clustered_shadow_policy_copy_selected)(
                handle,
                buffer.as_mut_ptr(),
                required,
                &mut count,
            )
        })?;
        let count = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("CNA reported more indices than fit in memory"))?;
        buffer.truncate(count.min(capacity));
        Ok(buffer)
    }

    /// Whether one light index is currently admitted.
    pub fn is_selected(&self, light_index: i32) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value = 0_u8;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_shadow_policy_is_selected)(
                handle,
                light_index,
                &mut value,
            )
        })?;
        Ok(value != 0)
    }

    /// The score the policy last computed for one light.
    pub fn score(&self, light_index: i32) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_shadow_policy_get_score)(handle, light_index, &mut value)
        })?;
        Ok(value)
    }

    /// How many lights asked to cast a shadow.
    pub fn request_count(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_shadow_policy_get_request_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// How many asked and were refused.
    ///
    /// The number that says whether the budget is too small, which the
    /// selection alone does not.
    pub fn refused_count(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_shadow_policy_get_refused_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Forgets every selection and score.
    pub fn reset(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.clustered_shadow_policy_reset)(handle) })
    }

    /// Releases the policy now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }
}

impl Drop for ClusteredShadowPolicy {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// The uploaded light list a clustered shader reads.
///
/// `OWNED`. The buffer owns three textures and **lends none of them**: there is
/// no accessor for them, only [`bind`](Self::bind), so nothing here can outlive
/// the buffer and destruction is never refused for an outstanding view.
pub struct ClusteredLightBuffer {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl ClusteredLightBuffer {
    /// Creates a buffer on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut buffer = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live for the call and the output is a
        // live local.
        native.check(unsafe {
            (native.engine.clustered_light_buffer_create)(device.handle()?, &mut buffer)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(buffer),
            destroy: native.engine.clustered_light_buffer_destroy,
            released: "the clustered light buffer has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// The GLSL a shader needs to read an uploaded light list.
    ///
    /// A property of the format rather than of any one buffer, so it needs no
    /// buffer to ask.
    pub fn light_lookup_glsl() -> Result<String> {
        let native = Native::process()?;
        copy_text(&native, |api, destination, capacity, out_bytes| {
            // SAFETY: CNA's size-then-copy protocol, driven by `copy_text`.
            unsafe {
                (api.clustered_light_buffer_copy_light_lookup_glsl)(
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
    }

    /// Uploads a set, a grid and an assignment as one consistent trio.
    ///
    /// The three must agree -- the assignment's light indices are positions in
    /// the set and its cluster indices positions in the grid. A mismatched trio
    /// is refused, because uploading it would light the wrong objects with the
    /// wrong lamps rather than fail visibly.
    pub fn upload(
        &self,
        lights: &ClusteredLightSet,
        grid: &ClusteredLightGrid,
        assignment: &ClusteredLightAssignment,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let lights = lights.core.get()?;
        let grid = grid.core.get()?;
        let assignment = assignment.core.get()?;
        // SAFETY: all four handles are owned and live for the call.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_buffer_upload)(handle, lights, grid, assignment)
        })
    }

    /// Binds the uploaded textures to three consecutive units of an effect.
    pub fn bind(&self, effect: &Effect, first_unit: i32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the buffer handle is owned, the effect is borrowed for the
        // call, and the unit is passed through.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_buffer_bind)(
                handle,
                effect.native_handle()?,
                first_unit,
            )
        })
    }

    /// Whether anything has been uploaded yet.
    pub fn is_uploaded(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value = 0_u8;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_buffer_is_uploaded)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// How many lights the last upload carried.
    pub fn light_count(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_buffer_get_light_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// How many clusters the last upload carried.
    pub fn cluster_count(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_buffer_get_cluster_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// How many light references the last upload carried.
    pub fn reference_count(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_buffer_get_reference_count)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Releases the buffer and its textures now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }
}

impl Drop for ClusteredLightBuffer {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// The GPU program that sorts lights into clusters.
///
/// `OWNED`, and it **degrades rather than refuses**: on a renderer without
/// compute shaders [`assign`](Self::assign) still produces the same assignment
/// on the CPU, and [`used_compute`](Self::used_compute) says which path ran.
/// [`is_supported`](Self::is_supported) answering `false` is therefore a fact
/// about performance, not about correctness.
pub struct ClusteredLightCompute {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
}

impl ClusteredLightCompute {
    /// The per-cluster light capacity used unless told otherwise.
    pub const DEFAULT_STRIDE: i32 = sys::CNA_CLUSTERED_COMPUTE_DEFAULT_STRIDE_EXT;

    /// Creates the program with a per-cluster light capacity.
    pub fn new(device: &GraphicsDevice, stride: i32) -> Result<Self> {
        let native = device.state_native();
        let mut compute = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live for the call and the output is a
        // live local.
        native.check(unsafe {
            (native.engine.clustered_light_compute_create)(device.handle()?, stride, &mut compute)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(compute),
            destroy: native.engine.clustered_light_compute_destroy,
            released: "the clustered light compute program has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
        })
    }

    /// Whether the GPU path compiled.
    pub fn is_supported(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value = 0_u8;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_compute_is_supported)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Why the GPU path is unavailable; empty when it compiled.
    pub fn unsupported_reason(&self) -> Result<String> {
        let handle = self.core.get()?;
        copy_text(&self.native, |api, destination, capacity, out_bytes| {
            // SAFETY: the handle is owned and this is CNA's size-then-copy
            // protocol, driven by `copy_text`.
            unsafe {
                (api.clustered_light_compute_copy_unsupported_reason)(
                    handle,
                    destination,
                    capacity,
                    out_bytes,
                )
            }
        })
    }

    /// The per-cluster light capacity.
    pub fn stride(&self) -> Result<i32> {
        let handle = self.core.get()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_compute_get_stride)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Sorts light bounds into a grid's clusters, filling an assignment.
    pub fn assign(
        &self,
        grid: &ClusteredLightGrid,
        view: Matrix,
        bounds: &[BoundingSphere],
        assignment: &ClusteredLightAssignment,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let grid_handle = grid.core.get()?;
        let out = assignment.core.get()?;
        let view = native_matrix(view);
        let native_bounds: Vec<sys::CNA_BoundingSphere> = bounds
            .iter()
            .map(|sphere| sys::CNA_BoundingSphere {
                center: native_vector3(sphere.Center),
                radius: sphere.Radius,
            })
            .collect();
        // SAFETY: all three handles are owned, and the matrix and sphere array
        // are borrowed for the call with the array's own length.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_compute_assign)(
                handle,
                grid_handle,
                &view,
                native_bounds.as_ptr(),
                native_bounds.len() as u64,
                out,
            )
        })
    }

    /// Whether the last assignment ran on the GPU.
    pub fn used_compute(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value = 0_u8;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_compute_used_compute)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Whether the last assignment overflowed a cluster's capacity.
    ///
    /// A cluster holding more lights than the stride drops the excess, so this
    /// says a larger stride is needed rather than that anything failed.
    pub fn has_overflowed(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value = 0_u8;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_light_compute_has_overflowed)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Releases the program now rather than at drop.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }
}

impl Drop for ClusteredLightCompute {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}

/// The material terms one clustered light shades a surface point with.
///
/// A value, and the reason [`ClusteredForwardEffect::contribution`] takes one:
/// the C route has sixteen inputs, eight of which the canonical overload
/// defaults and C cannot. [`Default`] fills in exactly those documented
/// neutral values, so "the effects you are not using" costs nothing to get
/// right.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ClusteredShadingMaterial {
    /// The material's base colour.
    pub base_color: Vector3,
    /// How metallic the material is.
    pub metallic: f32,
    /// The material's roughness.
    pub roughness: f32,
    /// Clearcoat strength.
    pub clearcoat: f32,
    /// Clearcoat roughness.
    pub clearcoat_roughness: f32,
    /// Sheen colour.
    pub sheen_color: Vector3,
    /// Sheen roughness.
    pub sheen_roughness: f32,
    /// Iridescence strength.
    pub iridescence: f32,
    /// Iridescence index of refraction; CNA's neutral value is `1.3`.
    pub iridescence_ior: f32,
    /// Iridescence film thickness in nanometres; CNA's neutral value is `400`.
    pub iridescence_thickness: f32,
    /// Subsurface colour.
    pub subsurface_color: Vector3,
    /// How far light wraps around the terminator; CNA's neutral value is `0.5`.
    pub subsurface_wrap: f32,
}

impl Default for ClusteredShadingMaterial {
    fn default() -> Self {
        Self {
            base_color: Vector3::from_x_and_y_and_z(1.0, 1.0, 1.0),
            metallic: 0.0,
            roughness: 0.5,
            clearcoat: 0.0,
            clearcoat_roughness: 0.0,
            sheen_color: Vector3::Zero,
            sheen_roughness: 0.0,
            iridescence: 0.0,
            iridescence_ior: 1.3,
            iridescence_thickness: 400.0,
            subsurface_color: Vector3::Zero,
            subsurface_wrap: 0.5,
        }
    }
}

/// The shader that walks a cluster's light list and shades a surface with it.
///
/// `OWNED`. Its shader effect is a **counted borrow**: releasing this effect is
/// refused while a [`BorrowedEffect`] taken from it is still alive, which is
/// why [`effect`](Self::effect) hands out a lifetime-bound view.
///
/// The opaque frame it refracts against is a `RETAINED_DEPENDENCY`: CNA stores
/// a raw `Texture2D*` and retains nothing, so
/// [`set_opaque_frame`](Self::set_opaque_frame) *takes* the texture and this
/// value holds it for exactly as long as CNA points at it.
///
/// Creation succeeds on a renderer that cannot run the shader; ask
/// [`is_supported`](Self::is_supported).
pub struct ClusteredForwardEffect {
    core: Arc<EngineHandle>,
    native: Arc<Native>,
    device: GraphicsDevice,
    opaque_frame: Option<Texture2D>,
}

impl ClusteredForwardEffect {
    /// The most lights one fragment walks before the shader stops accumulating.
    pub const MAX_LIGHTS_PER_FRAGMENT: i32 =
        sys::CNA_CLUSTERED_FORWARD_MAX_LIGHTS_PER_FRAGMENT_EXT;

    /// Creates the effect on a device.
    pub fn new(device: &GraphicsDevice) -> Result<Self> {
        let native = device.state_native();
        let mut effect = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device handle is live for the call and the output is a
        // live local.
        native.check(unsafe {
            (native.engine.clustered_forward_effect_create)(device.handle()?, &mut effect)
        })?;
        let core = Arc::new(EngineHandle {
            native: Arc::clone(native),
            handle: Mutex::new(effect),
            destroy: native.engine.clustered_forward_effect_destroy,
            released: "the clustered forward effect has been released",
        });
        let child: Arc<dyn OwnedEngineChild> = Arc::clone(&core) as Arc<dyn OwnedEngineChild>;
        device.register_engine_child(&child);
        Ok(Self {
            core,
            native: Arc::clone(native),
            device: device.clone(),
            opaque_frame: None,
        })
    }

    /// Whether the effect's shader exists and links.
    pub fn is_supported(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value = 0_u8;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_is_supported)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Prepares the effect to shade with an uploaded light buffer.
    ///
    /// Refuses when the buffer holds nothing -- there is no cluster table for
    /// the shader to walk -- and when the material transmits without an opaque
    /// frame to refract against, which is a refusal rather than an
    /// approximation because a transmissive material drawn without one is an
    /// opaque object where a glass one was asked for.
    pub fn begin(
        &self,
        world: Matrix,
        view: Matrix,
        projection: Matrix,
        camera_position: Vector3,
        lights: &ClusteredLightBuffer,
    ) -> Result<()> {
        let handle = self.core.get()?;
        let buffer = lights.core.get()?;
        let world = native_matrix(world);
        let view = native_matrix(view);
        let projection = native_matrix(projection);
        let camera_position = native_vector3(camera_position);
        // SAFETY: both handles are owned, and the matrices and the position are
        // borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_begin)(
                handle,
                &world,
                &view,
                &projection,
                &camera_position,
                buffer,
            )
        })
    }

    /// The shader effect, borrowed for as long as the view lives.
    ///
    /// `None` on a renderer where the shader did not link.
    pub fn effect(&self) -> Result<Option<BorrowedEffect<'_>>> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_get_effect)(handle, &mut value)
        })?;
        if value == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        Ok(Some(BorrowedEffect::new(&self.native, &self.device, value)))
    }

    /// Whether an area light is bound.
    pub fn has_area_light(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value = 0_u8;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_has_area_light)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Unbinds any area light.
    pub fn clear_area_light(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.clustered_forward_effect_clear_area_light)(handle) })
    }

    /// Whether a light probe is bound.
    pub fn has_light_probe(&self) -> Result<bool> {
        let handle = self.core.get()?;
        let mut value = 0_u8;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_has_light_probe)(handle, &mut value)
        })?;
        Ok(value != 0)
    }

    /// Unbinds any light probe.
    pub fn clear_light_probe(&self) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_clear_light_probe)(handle)
        })
    }

    /// The material's base colour.
    pub fn base_color(&self) -> Result<Vector3> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_get_base_color)(handle, &mut value)
        })?;
        Ok(from_native_vector3(value))
    }

    /// Sets it, **clamping** each channel to zero-to-one rather than refusing.
    pub fn set_base_color(&self, color: Vector3) -> Result<()> {
        let handle = self.core.get()?;
        let color = native_vector3(color);
        // SAFETY: the handle is owned and the colour is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_set_base_color)(handle, &color)
        })
    }

    /// How metallic the material is.
    pub fn metallic(&self) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_get_metallic)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Sets it, **clamped** to zero-to-one.
    pub fn set_metallic(&self, metallic: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_set_metallic)(handle, metallic)
        })
    }

    /// The material's roughness.
    pub fn roughness(&self) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_get_roughness)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Sets it, **clamped to 0.04-to-one**.
    ///
    /// The floor is not zero and is not a typo: a perfectly smooth surface
    /// collapses the specular lobe to a point the shader cannot integrate, so
    /// the canonical setter refuses to go below it by clamping.
    pub fn set_roughness(&self, roughness: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_set_roughness)(handle, roughness)
        })
    }

    /// The material's index of refraction.
    pub fn ior(&self) -> Result<f32> {
        let handle = self.core.get()?;
        let mut value = 0.0_f32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_get_ior)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// Sets it.
    pub fn set_ior(&self, ior: f32) -> Result<()> {
        let handle = self.core.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.engine.clustered_forward_effect_set_ior)(handle, ior) })
    }

    /// The ambient term.
    pub fn ambient(&self) -> Result<Vector3> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_get_ambient)(handle, &mut value)
        })?;
        Ok(from_native_vector3(value))
    }

    /// Sets it, **flooring** each channel at zero -- and only flooring: a
    /// channel above one is kept, because an ambient brighter than white is a
    /// choice while a negative one would subtract light that was never added.
    pub fn set_ambient(&self, ambient: Vector3) -> Result<()> {
        let handle = self.core.get()?;
        let ambient = native_vector3(ambient);
        // SAFETY: the handle is owned and the term is borrowed for the call.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_set_ambient)(handle, &ambient)
        })
    }

    /// The opaque frame the effect refracts against, viewed for a borrow.
    pub fn opaque_frame(&self) -> Result<Option<BorrowedRenderTarget<'_>>> {
        let handle = self.core.get()?;
        let mut value = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_get_opaque_frame)(handle, &mut value)
        })?;
        if value == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        BorrowedRenderTarget::new(&self.native, &self.device, value).map(Some)
    }

    /// Whether a frame to refract against is bound.
    pub fn has_opaque_frame(&self) -> Result<bool> {
        Ok(self.opaque_frame()?.is_some())
    }

    /// Gives the effect a copy of the opaque frame to refract against.
    ///
    /// CNA keeps a raw pointer and retains nothing, so this **takes** the
    /// texture: the effect holds it for exactly as long as CNA points at it,
    /// and `None` unbinds and releases the previous one.
    pub fn set_opaque_frame(&mut self, frame: Option<Texture2D>) -> Result<()> {
        let handle = self.core.get()?;
        let frame_handle = match frame.as_ref() {
            Some(texture) => texture.handle()?,
            None => sys::CNA_INVALID_HANDLE,
        };
        // SAFETY: the handle is owned and the texture handle is live for the
        // call, kept alive afterwards by the value this stores.
        self.native.check(unsafe {
            (self.native.engine.clustered_forward_effect_set_opaque_frame)(handle, frame_handle)
        })?;
        self.opaque_frame = frame;
        Ok(())
    }

    /// The volume attenuation of a transmissive material.
    ///
    /// A pure function of its arguments, so it needs no effect.
    pub fn volume_attenuation(
        attenuation_color: Vector3,
        attenuation_distance: f32,
        thickness: f32,
    ) -> Result<Vector3> {
        let native = Native::process()?;
        let color = native_vector3(attenuation_color);
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the colour is borrowed for the call and the output is a live
        // local.
        native.check(unsafe {
            (native.engine.clustered_forward_effect_volume_attenuation)(
                &color,
                attenuation_distance,
                thickness,
                &mut value,
            )
        })?;
        Ok(from_native_vector3(value))
    }

    /// One light's contribution to a surface point.
    ///
    /// A pure function of its arguments, so it needs no effect either.
    pub fn contribution(
        light: ClusteredLight,
        surface: Vector3,
        normal: Vector3,
        camera_position: Vector3,
        material: ClusteredShadingMaterial,
    ) -> Result<Vector3> {
        let native = Native::process()?;
        let light = light.to_native();
        let surface = native_vector3(surface);
        let normal = native_vector3(normal);
        let camera_position = native_vector3(camera_position);
        let base_color = native_vector3(material.base_color);
        let sheen_color = native_vector3(material.sheen_color);
        let subsurface_color = native_vector3(material.subsurface_color);
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: every pointer is a live local borrowed for the call, and the
        // output is one too.
        native.check(unsafe {
            (native.engine.clustered_forward_effect_contribution)(
                &light,
                &surface,
                &normal,
                &camera_position,
                &base_color,
                material.metallic,
                material.roughness,
                material.clearcoat,
                material.clearcoat_roughness,
                &sheen_color,
                material.sheen_roughness,
                material.iridescence,
                material.iridescence_ior,
                material.iridescence_thickness,
                &subsurface_color,
                material.subsurface_wrap,
                &mut value,
            )
        })?;
        Ok(from_native_vector3(value))
    }

    /// Releases the effect now rather than at drop.
    ///
    /// Refused while a [`BorrowedEffect`] taken from it is still alive.
    pub fn release(&self) -> Result<()> {
        self.core.release()
    }
}

impl Drop for ClusteredForwardEffect {
    fn drop(&mut self) {
        let _ = self.core.release();
    }
}
