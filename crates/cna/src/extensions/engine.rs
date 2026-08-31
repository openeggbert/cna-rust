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
use crate::graphics::{DepthFormat, RenderTarget2D, TextureCube};
use crate::value::{BoundingBox, Color, Matrix, Vector3, Vector4};

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
