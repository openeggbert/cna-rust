//! What a graphics device can say about itself, and the events it raises.
//!
//! The strict projection covers XNA's `GraphicsDevice`. This is the rest of
//! `graphics_device.h`: the renderer capabilities a game asks *before* choosing
//! a path, the device's own lifecycle events, and the render-state toggles that
//! sit underneath the state objects.
//!
//! # A capability is not a promise about a frame
//!
//! Every `supports_*` here answers what the renderer says it can do. It does
//! not say the operation will succeed on a given resource, and the crate's own
//! qualification does not treat it as if it did -- a route that answers `true`
//! and then refuses is a finding, not a contradiction to be smoothed over.

#![allow(clippy::missing_errors_doc)]

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::graphics::{Effect, GraphicsDevice, GraphicsProfile, SurfaceFormat, Texture2D};
use crate::native::Native;
use crate::value::Color;

/// What CNA does when a game makes a 3D call a renderer cannot serve.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Unsupported3DCallBehavior {
    /// Refuse the call, which is what a game wants while it is being written.
    #[default]
    Throw,
    /// Log once and carry on, which is what a shipped game wants on a host
    /// whose renderer is weaker than the one it was built against.
    WarnAndStub,
}

impl Unsupported3DCallBehavior {
    const fn to_native(self) -> sys::CNA_Unsupported3DGraphicsCallBehavior {
        match self {
            Self::Throw => sys::CNA_UNSUPPORTED_3D_GRAPHICS_CALL_BEHAVIOR_THROW,
            Self::WarnAndStub => sys::CNA_UNSUPPORTED_3D_GRAPHICS_CALL_BEHAVIOR_WARN_AND_STUB,
        }
    }

    const fn from_native(value: sys::CNA_Unsupported3DGraphicsCallBehavior) -> Option<Self> {
        Some(match value {
            sys::CNA_UNSUPPORTED_3D_GRAPHICS_CALL_BEHAVIOR_THROW => Self::Throw,
            sys::CNA_UNSUPPORTED_3D_GRAPHICS_CALL_BEHAVIOR_WARN_AND_STUB => Self::WarnAndStub,
            _ => return None,
        })
    }
}

/// One of the device's own lifecycle events.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum DeviceEvent {
    Disposing,
    DeviceLost,
    DeviceReset,
    DeviceResetting,
}

impl DeviceEvent {
    const fn to_native(self) -> sys::CNA_GraphicsDeviceEvent {
        match self {
            Self::Disposing => sys::CNA_GRAPHICS_DEVICE_EVENT_DISPOSING,
            Self::DeviceLost => sys::CNA_GRAPHICS_DEVICE_EVENT_DEVICE_LOST,
            Self::DeviceReset => sys::CNA_GRAPHICS_DEVICE_EVENT_DEVICE_RESET,
            Self::DeviceResetting => sys::CNA_GRAPHICS_DEVICE_EVENT_DEVICE_RESETTING,
        }
    }
}

// The two shapes almost every route in this module has. They stay inherent
// because they are private plumbing rather than public surface: an inherent
// `pub fn` on a strict XNA type is what this module exists not to add.
impl GraphicsDevice {
    fn flag(
        &self,
        route: impl FnOnce(&Native, sys::CNA_Handle, *mut sys::CNA_Bool) -> sys::CNA_Result,
    ) -> Result<bool> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        native.check(route(&native, handle, &mut value))?;
        Ok(value != sys::CNA_FALSE)
    }
}

/// Capabilities a game asks before choosing a rendering path.
///
/// Distinct from [`crate::extensions::graphics::RendererCapabilityExt`], which
/// is CNA's generic capability *tables* -- a feature enum, a limit enum, a
/// per-format usage mask. These are the individually typed questions
/// `graphics_device.h` publishes, and the colour space the display is
/// presenting in.
///
/// A CNA extension: import it to call these.
///
/// ```rust,ignore
/// use cna::extensions::graphics_device_ext::DeviceCapabilityExt;
/// if device.executes_shader_effect_source()? { /* ... */ }
/// ```
pub trait DeviceCapabilityExt {
    /// Whether this renderer compiles shader-effect source at all.
    ///
    /// `false` means `ShaderEffect` will construct and never draw, which is a
    /// different answer from a shader that failed to compile.
    fn executes_shader_effect_source(&self) -> Result<bool>;

    /// Whether this renderer can light from an environment map.
    fn supports_image_based_lighting(&self) -> Result<bool>;

    /// Whether a surface format can be drawn into rather than only sampled.
    ///
    /// The two are different capabilities and a renderer routinely has one
    /// without the other, which is why this is not `SurfaceFormat`'s own
    /// question.
    fn supports_surface_format_as_render_target(
        &self,
        format: SurfaceFormat,
    ) -> Result<bool>;

    /// Whether the display can present in that colour space.
    fn supports_display_color_space(&self, color_space: u32) -> Result<bool>;

    /// The colour space the display is currently presenting in.
    fn display_color_space(&self) -> Result<u32>;

    /// Asks the display to present in that colour space.
    ///
    /// Answers whether it took. A refusal is an ordinary answer on a display
    /// that cannot, which is why it is not a failure.
    fn set_display_color_space(&self, color_space: u32) -> Result<bool>;

    /// The largest compute work-group count on one axis.
    fn max_compute_work_group_count(&self, axis: i32) -> Result<i32>;

    /// The largest compute work-group size on one axis.
    fn max_compute_work_group_size(&self, axis: i32) -> Result<i32>;

    /// The largest number of invocations one work group may have.
    ///
    /// Separate from the per-axis sizes and smaller than their product on real
    /// hardware, which is why a dispatch sized from the axes alone can still be
    /// refused.
    fn max_compute_work_group_invocations(&self) -> Result<i32>;
}

impl DeviceCapabilityExt for GraphicsDevice {
    fn executes_shader_effect_source(&self) -> Result<bool> {
        self.flag(|native, handle, out| {
            // SAFETY: owned handle, live output.
            unsafe { (native.graphics_device_executes_shader_effect_source_ext)(handle, out) }
        })
    }

    fn supports_image_based_lighting(&self) -> Result<bool> {
        self.flag(|native, handle, out| {
            // SAFETY: owned handle, live output.
            unsafe { (native.graphics_device_supports_image_based_lighting_ext)(handle, out) }
        })
    }

    fn supports_surface_format_as_render_target(
        &self,
        format: SurfaceFormat,
    ) -> Result<bool> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe {
            (native.graphics_device_supports_surface_format_as_render_target_ext)(
                handle,
                format as u32,
                &mut value,
            )
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    fn supports_display_color_space(&self, color_space: u32) -> Result<bool> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe {
            (native.graphics_device_supports_display_color_space_ext)(
                handle,
                color_space,
                &mut value,
            )
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    fn display_color_space(&self) -> Result<u32> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = 0_u32;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe {
            (native.graphics_device_get_display_color_space_ext)(handle, &mut value)
        })?;
        Ok(value)
    }

    fn set_display_color_space(&self, color_space: u32) -> Result<bool> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe {
            (native.graphics_device_set_display_color_space_ext)(handle, color_space, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    fn max_compute_work_group_count(&self, axis: i32) -> Result<i32> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe {
            (native.graphics_device_get_max_compute_work_group_count_ext)(
                handle,
                axis,
                &mut value,
            )
        })?;
        Ok(value)
    }

    fn max_compute_work_group_size(&self, axis: i32) -> Result<i32> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe {
            (native.graphics_device_get_max_compute_work_group_size_ext)(handle, axis, &mut value)
        })?;
        Ok(value)
    }

    fn max_compute_work_group_invocations(&self) -> Result<i32> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = 0_i32;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe {
            (native.graphics_device_get_max_compute_work_group_invocations_ext)(handle, &mut value)
        })?;
        Ok(value)
    }
}

impl GraphicsDevice {
    fn toggle(
        &self,
        enabled: bool,
        route: impl FnOnce(&Native, sys::CNA_Handle, sys::CNA_Bool) -> sys::CNA_Result,
    ) -> Result<()> {
        let handle = self.handle()?;
        let native = Native::process()?;
        native.check(route(&native, handle, u8::from(enabled)))
    }
}

/// Device state, lifecycle, and the render-state toggles under the state objects.
///
/// XNA reaches render state only through `BlendState`, `DepthStencilState` and
/// their siblings, which set several things at once. CNA also publishes the
/// individual toggles underneath them, along with the device's own disposal,
/// resource accounting and renderer-rebuild routes.
///
/// A CNA extension: import it to call these.
///
/// ```rust,ignore
/// use cna::extensions::graphics_device_ext::DeviceStateExt;
/// device.set_depth_test_enabled(false)?;
/// ```
pub trait DeviceStateExt {
    /// Whether the device has been disposed.
    fn is_disposed_native(&self) -> Result<bool>;

    /// Disposes the device through CNA rather than through Rust's `Drop`.
    ///
    /// The strict projection's `Dispose` is the one to reach for; this exists
    /// because the route is part of the header and a caller holding a device
    /// CNA owns may need it.
    fn dispose_native(&self) -> Result<()>;

    /// How many resources the device is currently tracking.
    ///
    /// The count that must fall back to its starting value when everything a
    /// test made is dropped, which is what makes a leak assertable rather than
    /// merely unlikely.
    fn tracked_resource_count(&self) -> Result<u64>;

    /// Clears the colour and depth buffers in one call.
    fn clear_color_depth(&self, color: Color, depth: f32) -> Result<()>;

    /// Makes one effect the device's current one.
    fn set_current_effect(&self, effect: &Effect) -> Result<()>;

    /// Unbinds one texture from every slot it is bound to.
    ///
    /// What a caller does before destroying a texture the device may still be
    /// holding, which is otherwise a refusal at destroy time.
    fn unbind_texture(&self, texture: &Texture2D) -> Result<()>;

    /// Turns blending on or off directly.
    fn set_blend_enabled(&self, enabled: bool) -> Result<()>;

    /// Turns depth testing on or off directly.
    fn set_depth_test_enabled(&self, enabled: bool) -> Result<()>;

    /// Turns depth writing on or off directly.
    fn set_depth_write_enabled(&self, enabled: bool) -> Result<()>;

    /// Whether the device tries to recover a lost renderer context.
    fn set_context_recovery_enabled(&self, enabled: bool) -> Result<()>;

    /// What the device does when a 3D call cannot be served.
    fn unsupported_3d_call_behavior(&self) -> Result<Unsupported3DCallBehavior>;

    /// Chooses what the device does when a 3D call cannot be served.
    fn set_unsupported_3d_call_behavior(
        &self,
        behavior: Unsupported3DCallBehavior,
    ) -> Result<()>;

    /// Changes the profile the device reports and enforces.
    fn set_graphics_profile(&self, profile: GraphicsProfile) -> Result<()>;

    /// Rebuilds the renderer for a different multi-sample count.
    ///
    /// Heavier than a state change: it recreates the renderer, so every
    /// resource the device tracks sees a lost-and-reset cycle.
    fn recreate_renderer_for_multi_sample_count(&self, count: i32) -> Result<()>;

    /// Tells the device its content-loaded resources are gone.
    ///
    /// What a content manager calls when it unloads, so the device stops
    /// tracking resources whose backing has already been freed.
    fn notify_content_lost_resources(&self) -> Result<()>;

    /// Puts a named marker in the renderer's command stream.
    ///
    /// For a graphics debugger. A renderer with no marker support ignores it,
    /// which is why this reports nothing.
    fn set_string_marker(&self, text: &str) -> Result<()>;
}

impl DeviceStateExt for GraphicsDevice {
    fn is_disposed_native(&self) -> Result<bool> {
        self.flag(|native, handle, out| {
            // SAFETY: owned handle, live output.
            unsafe { (native.graphics_device_get_is_disposed)(handle, out) }
        })
    }

    fn dispose_native(&self) -> Result<()> {
        let handle = self.handle()?;
        let native = Native::process()?;
        // SAFETY: the handle is owned by a live device.
        native.check(unsafe { (native.graphics_device_dispose)(handle) })
    }

    fn tracked_resource_count(&self) -> Result<u64> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        native
            .check(unsafe { (native.graphics_device_get_tracked_resource_count)(handle, &mut value) })?;
        Ok(value)
    }

    fn clear_color_depth(&self, color: Color, depth: f32) -> Result<()> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let native_color = sys::CNA_Color {
            r: color.R(),
            g: color.G(),
            b: color.B(),
            a: color.A(),
        };
        // SAFETY: the handle is owned and both values are by value.
        native.check(unsafe {
            (native.graphics_device_clear_color_depth)(handle, native_color, depth)
        })
    }

    fn set_current_effect(&self, effect: &Effect) -> Result<()> {
        let handle = self.handle()?;
        let native = Native::process()?;
        // SAFETY: both handles belong to live values.
        native.check(unsafe {
            (native.graphics_device_set_current_effect)(handle, effect.handle()?)
        })
    }

    fn unbind_texture(&self, texture: &Texture2D) -> Result<()> {
        let handle = self.handle()?;
        let native = Native::process()?;
        // SAFETY: both handles belong to live values.
        native.check(unsafe { (native.graphics_device_unbind_texture)(handle, texture.handle()?) })
    }

    fn set_blend_enabled(&self, enabled: bool) -> Result<()> {
        self.toggle(enabled, |native, handle, value| {
            // SAFETY: owned handle, flag by value.
            unsafe { (native.graphics_device_set_blend_enabled)(handle, value) }
        })
    }

    fn set_depth_test_enabled(&self, enabled: bool) -> Result<()> {
        self.toggle(enabled, |native, handle, value| {
            // SAFETY: owned handle, flag by value.
            unsafe { (native.graphics_device_set_depth_test_enabled)(handle, value) }
        })
    }

    fn set_depth_write_enabled(&self, enabled: bool) -> Result<()> {
        self.toggle(enabled, |native, handle, value| {
            // SAFETY: owned handle, flag by value.
            unsafe { (native.graphics_device_set_depth_write_enabled)(handle, value) }
        })
    }

    fn set_context_recovery_enabled(&self, enabled: bool) -> Result<()> {
        self.toggle(enabled, |native, handle, value| {
            // SAFETY: owned handle, flag by value.
            unsafe { (native.graphics_device_set_context_recovery_enabled)(handle, value) }
        })
    }

    fn unsupported_3d_call_behavior(&self) -> Result<Unsupported3DCallBehavior> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let mut value = 0_u32;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe {
            (native.graphics_device_get_unsupported_3d_call_behavior)(handle, &mut value)
        })?;
        Unsupported3DCallBehavior::from_native(value).ok_or(CnaError::InvalidInput(
            "CNA reported an unsupported-3D-call behaviour this build does not know",
        ))
    }

    fn set_unsupported_3d_call_behavior(
        &self,
        behavior: Unsupported3DCallBehavior,
    ) -> Result<()> {
        let handle = self.handle()?;
        let native = Native::process()?;
        // SAFETY: the handle is owned and the value is by value.
        native.check(unsafe {
            (native.graphics_device_set_unsupported_3d_call_behavior)(
                handle,
                behavior.to_native(),
            )
        })
    }

    fn set_graphics_profile(&self, profile: GraphicsProfile) -> Result<()> {
        let handle = self.handle()?;
        let native = Native::process()?;
        // SAFETY: the handle is owned and the profile is by value.
        native.check(unsafe {
            (native.graphics_device_set_graphics_profile_ext)(handle, profile as u32)
        })
    }

    fn recreate_renderer_for_multi_sample_count(&self, count: i32) -> Result<()> {
        let handle = self.handle()?;
        let native = Native::process()?;
        // SAFETY: the handle is owned and the count is by value.
        native.check(unsafe {
            (native.graphics_device_recreate_renderer_for_multi_sample_count_ext)(handle, count)
        })
    }

    fn notify_content_lost_resources(&self) -> Result<()> {
        let handle = self.handle()?;
        let native = Native::process()?;
        // SAFETY: the handle is owned.
        native.check(unsafe { (native.graphics_device_notify_content_lost_resources_ext)(handle) })
    }

    fn set_string_marker(&self, text: &str) -> Result<()> {
        let handle = self.handle()?;
        let native = Native::process()?;
        let view = sys::CNA_StringView {
            data: text.as_ptr().cast::<core::ffi::c_char>(),
            byte_length: text.len() as u64,
        };
        // SAFETY: the handle is owned and the view borrows `text` for the call.
        native.check(unsafe { (native.graphics_device_set_string_marker_ext)(handle, view) })
    }
}

/// A live subscription to one of the device's events.
///
/// CNA takes a function pointer and an opaque `void*` and keeps both, so the
/// closure is boxed and its address becomes the context. This value is the only
/// thing that knows the box is still reachable, which is why it withdraws the
/// registration in `Drop` *before* freeing it: the reverse order leaves CNA
/// holding a pointer to a dead closure.
#[must_use = "dropping a DeviceSubscription immediately unsubscribes it"]
pub struct DeviceSubscription {
    native: Arc<Native>,
    registration: Mutex<sys::CNA_GraphicsDeviceEventRegistrationHandle>,
    callback: Mutex<*mut core::ffi::c_void>,
    free: unsafe fn(*mut core::ffi::c_void),
}

// SAFETY: the pointer is an owned box this value alone frees, and the closures
// behind it are required to be `Send`.
unsafe impl Send for DeviceSubscription {}

impl DeviceSubscription {
    /// Withdraws the subscription early. Idempotent.
    pub fn unsubscribe(&self) -> Result<()> {
        let mut guard = self
            .registration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let registration = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if registration == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the registration is this value's own, withdrawn once, and
        // before the box below is freed.
        let result = self
            .native
            .check(unsafe { (self.native.graphics_device_unsubscribe)(registration) });
        let mut callback = self
            .callback
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pointer = core::mem::replace(&mut *callback, core::ptr::null_mut());
        if !pointer.is_null() {
            // SAFETY: the pointer came from `Box::into_raw` in the matching
            // subscribe call, with the `free` recorded there for its type.
            unsafe { (self.free)(pointer) };
        }
        result
    }
}

impl core::fmt::Debug for DeviceSubscription {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("DeviceSubscription")
            .field(
                "live",
                &(*self
                    .registration
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    != sys::CNA_INVALID_HANDLE),
            )
            .finish()
    }
}

impl Drop for DeviceSubscription {
    fn drop(&mut self) {
        let _ = self.unsubscribe();
    }
}

unsafe fn free_boxed<F>(pointer: *mut core::ffi::c_void) {
    // SAFETY: the caller passes back exactly the pointer `Box::into_raw`
    // produced for this `F`.
    drop(unsafe { Box::from_raw(pointer.cast::<F>()) });
}

/// What a resource-destroyed event carries.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct ResourceDestroyed {
    /// Whether the resource carried a tag at all.
    pub has_tag: bool,
    /// The resource's name, empty when it had none.
    pub name: String,
}

/// The device's own lifecycle and resource-tracking events.
///
/// XNA raises `Disposing`, `DeviceLost`, `DeviceReset` and `DeviceResetting`
/// through CLR events, which the strict projection covers with its
/// `Add*Handler`/`Remove*Handler` pairs. These are CNA's native subscriptions
/// to the same transitions plus the two resource-tracking events XNA has no
/// counterpart for, and each returns a [`DeviceSubscription`] that withdraws
/// itself on drop.
///
/// A CNA extension: import it to call these.
///
/// ```rust,ignore
/// use cna::extensions::graphics_device_ext::{DeviceEvent, DeviceEventExt};
/// let subscription = device.on_event(DeviceEvent::DeviceReset, || {})?;
/// ```
pub trait DeviceEventExt {
    /// Calls `callback` when the device raises that event.
    fn on_event(
        &self,
        event: DeviceEvent,
        callback: impl FnMut() + Send + 'static,
    ) -> Result<DeviceSubscription>;

    /// Calls `callback` whenever the device starts tracking a resource.
    ///
    /// The event carries whether a resource is attached rather than the
    /// resource itself: the object is the device's, mid-construction, and
    /// handing a Rust value out here would be publishing a half-built one.
    fn on_resource_created(
        &self,
        callback: impl FnMut(bool) + Send + 'static,
    ) -> Result<DeviceSubscription>;

    /// Calls `callback` whenever the device stops tracking a resource.
    fn on_resource_destroyed(
        &self,
        callback: impl FnMut(ResourceDestroyed) + Send + 'static,
    ) -> Result<DeviceSubscription>;
}

impl DeviceEventExt for GraphicsDevice {
    fn on_event(
        &self,
        event: DeviceEvent,
        callback: impl FnMut() + Send + 'static,
    ) -> Result<DeviceSubscription> {
        type Closure = Box<dyn FnMut() + Send + 'static>;

        unsafe extern "C" fn trampoline(
            _device: sys::CNA_Handle,
            context: *mut core::ffi::c_void,
        ) {
            if context.is_null() {
                return;
            }
            // SAFETY: the context is the box the subscription owns.
            let closure = unsafe { &mut *context.cast::<Closure>() };
            // A panic must not cross back into C, and a device event has
            // nowhere to report one.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| closure()));
        }

        let handle = self.handle()?;
        let native = Native::process()?;
        let boxed: Closure = Box::new(callback);
        let context = Box::into_raw(Box::new(boxed)).cast::<core::ffi::c_void>();
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned, the trampoline has the audited
        // signature, and the context is a live box the returned value owns.
        let result = native.check(unsafe {
            (native.graphics_device_subscribe_event)(
                handle,
                event.to_native(),
                Some(trampoline),
                context,
                &mut registration,
            )
        });
        if let Err(error) = result {
            // SAFETY: CNA never took the pointer.
            unsafe { free_boxed::<Closure>(context) };
            return Err(error);
        }
        Ok(DeviceSubscription {
            native,
            registration: Mutex::new(registration),
            callback: Mutex::new(context),
            free: free_boxed::<Closure>,
        })
    }

    fn on_resource_created(
        &self,
        callback: impl FnMut(bool) + Send + 'static,
    ) -> Result<DeviceSubscription> {
        type Closure = Box<dyn FnMut(bool) + Send + 'static>;

        unsafe extern "C" fn trampoline(
            _device: sys::CNA_Handle,
            info: *const sys::CNA_ResourceCreatedEventInfo,
            context: *mut core::ffi::c_void,
        ) {
            if context.is_null() {
                return;
            }
            // SAFETY: the context is the box the subscription owns, and CNA
            // borrows `info` for the duration of this call.
            let closure = unsafe { &mut *context.cast::<Closure>() };
            let has_resource = !info.is_null() && unsafe { (*info).has_resource } != sys::CNA_FALSE;
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                closure(has_resource);
            }));
        }

        let handle = self.handle()?;
        let native = Native::process()?;
        let boxed: Closure = Box::new(callback);
        let context = Box::into_raw(Box::new(boxed)).cast::<core::ffi::c_void>();
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: as in `on_event`.
        let result = native.check(unsafe {
            (native.graphics_device_subscribe_resource_created)(
                handle,
                Some(trampoline),
                context,
                &mut registration,
            )
        });
        if let Err(error) = result {
            // SAFETY: CNA never took the pointer.
            unsafe { free_boxed::<Closure>(context) };
            return Err(error);
        }
        Ok(DeviceSubscription {
            native,
            registration: Mutex::new(registration),
            callback: Mutex::new(context),
            free: free_boxed::<Closure>,
        })
    }

    fn on_resource_destroyed(
        &self,
        callback: impl FnMut(ResourceDestroyed) + Send + 'static,
    ) -> Result<DeviceSubscription> {
        type Closure = Box<dyn FnMut(ResourceDestroyed) + Send + 'static>;

        unsafe extern "C" fn trampoline(
            _device: sys::CNA_Handle,
            info: *const sys::CNA_ResourceDestroyedEventInfo,
            context: *mut core::ffi::c_void,
        ) {
            if context.is_null() {
                return;
            }
            // SAFETY: the context is the box the subscription owns.
            let closure = unsafe { &mut *context.cast::<Closure>() };
            let mut value = ResourceDestroyed::default();
            if !info.is_null() {
                // SAFETY: CNA borrows `info` and its name for this call; the
                // bytes are copied before it returns.
                let info = unsafe { &*info };
                value.has_tag = info.has_tag != sys::CNA_FALSE;
                let length = usize::try_from(info.name.byte_length).unwrap_or(0);
                if !info.name.data.is_null() && length > 0 {
                    // SAFETY: counted UTF-8 borrowed for this call.
                    let bytes = unsafe {
                        core::slice::from_raw_parts(info.name.data.cast::<u8>(), length)
                    };
                    value.name = String::from_utf8_lossy(bytes).into_owned();
                }
            }
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| closure(value)));
        }

        let handle = self.handle()?;
        let native = Native::process()?;
        let boxed: Closure = Box::new(callback);
        let context = Box::into_raw(Box::new(boxed)).cast::<core::ffi::c_void>();
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: as in `on_event`.
        let result = native.check(unsafe {
            (native.graphics_device_subscribe_resource_destroyed)(
                handle,
                Some(trampoline),
                context,
                &mut registration,
            )
        });
        if let Err(error) = result {
            // SAFETY: CNA never took the pointer.
            unsafe { free_boxed::<Closure>(context) };
            return Err(error);
        }
        Ok(DeviceSubscription {
            native,
            registration: Mutex::new(registration),
            callback: Mutex::new(context),
            free: free_boxed::<Closure>,
        })
    }
}

/// How many vertices a primitive count needs, for one topology.
///
/// The arithmetic differs per topology -- a strip shares vertices where a list
/// does not -- which is why this is a route rather than a multiplication at the
/// call site.
pub fn primitive_vertex_count(
    primitive_type: crate::graphics::PrimitiveType,
    primitive_count: i32,
) -> Result<i32> {
    let native = Native::process()?;
    let mut value = 0_i32;
    // SAFETY: the output is a live local.
    native.check(unsafe {
        (native.primitive_type_get_vertex_count)(primitive_type as u32, primitive_count, &mut value)
    })?;
    Ok(value)
}

/// What CNA can say about an XNA `OcclusionQuery` that XNA cannot.
///
/// A CNA extension: import it to call these.
///
/// ```rust,ignore
/// use cna::extensions::graphics_device_ext::OcclusionQueryExt;
/// let exact = query.is_pixel_count_precise()?;
/// ```
pub trait OcclusionQueryExt {
    /// Whether a renderer is attached to answer this query at all.
    fn has_renderer(&self) -> Result<bool>;

    /// Whether the pixel count is exact rather than conservative.
    ///
    /// A renderer that only answers "some pixels passed" reports `false`, and a
    /// game that draws a lens flare proportional to the count needs to know
    /// which it is getting.
    fn is_pixel_count_precise(&self) -> Result<bool>;
}

impl OcclusionQueryExt for crate::graphics::OcclusionQuery {
    fn has_renderer(&self) -> Result<bool> {
        let handle = self.native_handle()?;
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe { (native.occlusion_query_has_renderer)(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    fn is_pixel_count_precise(&self) -> Result<bool> {
        let handle = self.native_handle()?;
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe {
            (native.occlusion_query_get_is_pixel_count_precise_ext)(handle, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }
}

/// The read-only view of what CNA is about to create a device with, and the
/// subscription that delivers it.
///
/// [`GraphicsDeviceManagerExt::ObserveDeviceSettings`] returns the subscription
/// and hands the view to a callback. Both are CNA's own -- XNA has
/// `PreparingDeviceSettings`, which can *change* the settings, and nothing
/// read-only beside it -- so they are named here rather than inside
/// `cna::Microsoft::Xna::Framework`, where until now they had no name at all:
/// the method was public and its return type was not exported, so nothing
/// outside this crate could hold what it answered.
///
/// [`PresentationMode`] had the same defect and is fixed the same way.
/// `GraphicsDeviceManagerExt::PreferredPresentationMode` answers one, the enum
/// was `pub` in a private module and re-exported nowhere, and a consumer could
/// call the method and not name what came back.
pub use crate::game::{DeviceSettingsObserver, ObservedDeviceSettings, PresentationMode};

/// The `runtime_graphics_manager.h` routes with no XNA counterpart.
///
/// `PreferredPresentationMode` is the one that adds a capability rather than a
/// reading: XNA scales the back buffer to the window one way and gives a game
/// no say. CNA has five, and letterboxing versus stretching is a decision a
/// game with a fixed-aspect design has to be able to make.
///
/// `ObserveDeviceSettings` is the read-only pair to XNA's
/// `PreparingDeviceSettings`, which the strict projection carries: that one
/// hands a handler a mutable configuration, this one a `*const`. Its return
/// types, [`DeviceSettingsObserver`] and [`ObservedDeviceSettings`], are named
/// in this module for the same reason the method now is.
///
/// A CNA extension: import it to call these.
///
/// ```rust,ignore
/// use cna::extensions::graphics_device_ext::GraphicsDeviceManagerExt;
/// let observer = manager.ObserveDeviceSettings(|settings| { /* ... */ })?;
/// ```
#[allow(non_snake_case)]
pub trait GraphicsDeviceManagerExt {
    /// The device CNA currently has for this manager, if any.
    ///
    /// `GraphicsDevice()` answers the Rust value, which exists from the moment
    /// the manager creates one. This asks CNA, and answers `None` before the
    /// device is created and after it is lost -- the two states in which the
    /// Rust value is present and the native one is not.
    fn HasNativeGraphicsDevice(&self) -> Result<bool>;

    /// How the back buffer is fitted to the window.
    fn PreferredPresentationMode(&self) -> Result<PresentationMode>;

    /// Sets how the back buffer is fitted to the window.
    fn SetPreferredPresentationMode(&self, value: PresentationMode) -> Result<()>;

    /// Watches the candidate device settings without being able to change them.
    ///
    /// The read-only pair to `PreparingDeviceSettings`, which hands a handler a
    /// mutable configuration so it can edit what the device is created with.
    /// This one is handed a `*const` and cannot, which is what makes it the
    /// right subscription for logging or asserting what was chosen: an observer
    /// that *could* write is an observer a reader has to check for writes.
    fn ObserveDeviceSettings(
        &self,
        callback: impl FnMut(ObservedDeviceSettings) + Send + 'static,
    ) -> Result<DeviceSettingsObserver>;
}
