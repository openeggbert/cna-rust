#![allow(non_snake_case, non_upper_case_globals, clippy::missing_errors_doc)]

use core::any::{Any, TypeId};
use core::ffi::c_void;
use core::mem::size_of;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::{EventArgs, EventHandler};
use crate::extensions::window::WindowHandle;
use crate::graphics::resource::EventHandlers;
use crate::graphics::{
    DepthFormat, GraphicsAdapter, GraphicsDevice, GraphicsProfile, IGraphicsDeviceService,
    PresentationParameters, SurfaceFormat,
};
use crate::native::{Native, NativeGraphicsPreferences};

use super::{DisplayOrientation, Game, GameState};

#[derive(Clone)]
struct GraphicsDeviceInformationData {
    adapter: Arc<GraphicsAdapter>,
    graphics_profile: GraphicsProfile,
    presentation_parameters: Arc<PresentationParameters>,
}

/// Mutable XNA device proposal with CLR-style shared reference identity.
pub struct GraphicsDeviceInformation {
    data: Arc<Mutex<GraphicsDeviceInformationData>>,
}

impl GraphicsDeviceInformation {
    #[must_use]
    pub fn new() -> Self {
        Self::from_parts(
            Arc::new(GraphicsAdapter::default_placeholder()),
            GraphicsProfile::Reach,
            Arc::new(PresentationParameters::new()),
        )
    }

    fn from_parts(
        adapter: Arc<GraphicsAdapter>,
        graphics_profile: GraphicsProfile,
        presentation_parameters: Arc<PresentationParameters>,
    ) -> Self {
        Self {
            data: Arc::new(Mutex::new(GraphicsDeviceInformationData {
                adapter,
                graphics_profile,
                presentation_parameters,
            })),
        }
    }

    #[must_use]
    pub fn Adapter(&self) -> Arc<GraphicsAdapter> {
        Arc::clone(&self.read().adapter)
    }

    pub fn SetAdapter(&self, value: Arc<GraphicsAdapter>) {
        self.write().adapter = value;
    }

    #[must_use]
    pub fn GraphicsProfile(&self) -> GraphicsProfile {
        self.read().graphics_profile
    }

    pub fn SetGraphicsProfile(&self, value: GraphicsProfile) {
        self.write().graphics_profile = value;
    }

    #[must_use]
    pub fn PresentationParameters(&self) -> Arc<PresentationParameters> {
        Arc::clone(&self.read().presentation_parameters)
    }

    pub fn SetPresentationParameters(&self, value: Arc<PresentationParameters>) {
        self.write().presentation_parameters = value;
    }

    #[must_use]
    pub fn Equals(&self, obj: &dyn Any) -> bool {
        let Some(other) = obj.downcast_ref::<Self>() else {
            return false;
        };
        let left = self.read();
        let right = other.read();
        left.adapter.same_identity(&right.adapter)
            && left.graphics_profile == right.graphics_profile
            && presentation_parameters_equal(
                &left.presentation_parameters,
                &right.presentation_parameters,
            )
    }

    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        let value = self.read();
        let parameters = &value.presentation_parameters;
        value.adapter.identity_hash()
            ^ value.graphics_profile as i32
            ^ parameters.BackBufferWidth()
            ^ parameters.BackBufferHeight()
            ^ parameters.BackBufferFormat() as i32
            ^ parameters.DepthStencilFormat() as i32
            ^ parameters.MultiSampleCount()
            ^ parameters.DisplayOrientation().bits()
            ^ parameters.PresentationInterval() as i32
            ^ parameters.RenderTargetUsage() as i32
            ^ hash_u64(parameters.DeviceWindowHandle().0)
            ^ i32::from(parameters.IsFullScreen())
    }

    /// XNA's explicit clone deep-copies mutable presentation state.
    #[must_use]
    pub fn Clone(&self) -> Self {
        let value = self.read();
        Self::from_parts(
            Arc::clone(&value.adapter),
            value.graphics_profile,
            Arc::new(value.presentation_parameters.Clone()),
        )
    }

    fn from_native(
        value: sys::CNA_GraphicsDeviceInformation,
        _device: &GraphicsDevice,
    ) -> Result<Self> {
        if value.struct_size as usize != size_of::<sys::CNA_GraphicsDeviceInformation>()
            || value.struct_version != 1
        {
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INVALID_STATE,
                message: "CNA supplied an incompatible GraphicsDeviceInformation structure"
                    .to_owned(),
            });
        }
        let graphics_profile = graphics_profile(value.graphics_profile)?;
        let presentation_parameters = Arc::new(PresentationParameters::new());
        if !presentation_parameters
            .update_from_native(value.presentation_parameters, WindowHandle::default())
        {
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INVALID_STATE,
                message: "CNA supplied invalid presentation parameters".to_owned(),
            });
        }
        let adapter = if value.adapter_index < 0 {
            Arc::new(GraphicsAdapter::default_placeholder())
        } else {
            let index = u32::try_from(value.adapter_index)
                .map_err(|_| CnaError::InvalidInput("graphics adapter index is negative"))?;
            Arc::new(GraphicsAdapter::proposal_placeholder(index))
        };
        Ok(Self::from_parts(
            adapter,
            graphics_profile,
            presentation_parameters,
        ))
    }

    fn copy_to_native(
        &self,
        destination: &mut sys::CNA_GraphicsDeviceInformation,
        device: &GraphicsDevice,
    ) -> Result<()> {
        let value = self.read();
        let adapter_index = device.proposal_adapter_index_for(&value.adapter)?;
        let headless = destination.presentation_parameters.headless_ext != sys::CNA_FALSE;
        *destination = sys::CNA_GraphicsDeviceInformation {
            struct_size: size_of::<sys::CNA_GraphicsDeviceInformation>() as u32,
            struct_version: 1,
            adapter_index,
            graphics_profile: value.graphics_profile as u32,
            presentation_parameters: value.presentation_parameters.to_native(headless),
        };
        Ok(())
    }

    fn read(&self) -> GraphicsDeviceInformationData {
        self.data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn write(&self) -> std::sync::MutexGuard<'_, GraphicsDeviceInformationData> {
        self.data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Clone for GraphicsDeviceInformation {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

impl Default for GraphicsDeviceInformation {
    fn default() -> Self {
        Self::new()
    }
}

fn presentation_parameters_equal(
    left: &PresentationParameters,
    right: &PresentationParameters,
) -> bool {
    left.BackBufferWidth() == right.BackBufferWidth()
        && left.BackBufferHeight() == right.BackBufferHeight()
        && left.BackBufferFormat() == right.BackBufferFormat()
        && left.DepthStencilFormat() == right.DepthStencilFormat()
        && left.MultiSampleCount() == right.MultiSampleCount()
        && left.DisplayOrientation() == right.DisplayOrientation()
        && left.PresentationInterval() == right.PresentationInterval()
        && left.RenderTargetUsage() == right.RenderTargetUsage()
        && left.DeviceWindowHandle() == right.DeviceWindowHandle()
        && left.IsFullScreen() == right.IsFullScreen()
}

const fn hash_u64(value: u64) -> i32 {
    (value ^ (value >> 32)) as i32
}

/// Event payload retaining the mutable candidate by shared identity.
#[derive(Clone)]
pub struct PreparingDeviceSettingsEventArgs {
    information: Arc<GraphicsDeviceInformation>,
}

impl PreparingDeviceSettingsEventArgs {
    #[must_use]
    pub fn new(graphicsDeviceInformation: Arc<GraphicsDeviceInformation>) -> Self {
        Self {
            information: graphicsDeviceInformation,
        }
    }

    #[must_use]
    pub fn GraphicsDeviceInformation(&self) -> Arc<GraphicsDeviceInformation> {
        Arc::clone(&self.information)
    }
}

/// XNA's manager interface used by the game loop.
pub trait IGraphicsDeviceManager: Send + Sync {
    fn CreateDevice(&self);
    fn BeginDraw(&self) -> bool;
    fn EndDraw(&self);
}

#[derive(Clone, Copy)]
struct GraphicsPreferences {
    graphics_profile: GraphicsProfile,
    preferred_depth_stencil_format: DepthFormat,
    preferred_back_buffer_format: SurfaceFormat,
    preferred_back_buffer_width: i32,
    preferred_back_buffer_height: i32,
    is_full_screen: bool,
    synchronize_with_vertical_retrace: bool,
    prefer_multi_sampling: bool,
    supported_orientations: DisplayOrientation,
}

impl Default for GraphicsPreferences {
    fn default() -> Self {
        Self {
            graphics_profile: GraphicsProfile::Reach,
            preferred_depth_stencil_format: DepthFormat::Depth24,
            preferred_back_buffer_format: SurfaceFormat::Color,
            preferred_back_buffer_width: GraphicsDeviceManager::DefaultBackBufferWidth,
            preferred_back_buffer_height: GraphicsDeviceManager::DefaultBackBufferHeight,
            is_full_screen: false,
            synchronize_with_vertical_retrace: true,
            prefer_multi_sampling: false,
            supported_orientations: DisplayOrientation::Default,
        }
    }
}

struct NativeManagerBinding {
    native: Arc<Native>,
    handle: sys::CNA_GraphicsDeviceManagerHandle,
    registrations: Vec<sys::CNA_GameEventRegistrationHandle>,
}

pub(crate) struct GraphicsDeviceManagerState {
    self_weak: Weak<GraphicsDeviceManagerState>,
    game: Weak<GameState>,
    device: std::sync::OnceLock<GraphicsDevice>,
    preferences: Mutex<GraphicsPreferences>,
    binding: Mutex<Option<NativeManagerBinding>>,
    disposed: AtomicBool,
    disposed_event_raised: AtomicBool,
    pending_callback_error: Mutex<Option<CnaError>>,
    device_created: EventHandlers<EventArgs>,
    device_disposing: EventHandlers<EventArgs>,
    device_reset: EventHandlers<EventArgs>,
    device_resetting: EventHandlers<EventArgs>,
    preparing_device_settings: EventHandlers<PreparingDeviceSettingsEventArgs>,
    disposed_event: EventHandlers<EventArgs>,
}

impl GraphicsDeviceManagerState {
    fn new(game: &Arc<GameState>) -> Arc<Self> {
        Arc::new_cyclic(|self_weak| Self {
            self_weak: self_weak.clone(),
            game: Arc::downgrade(game),
            device: std::sync::OnceLock::new(),
            preferences: Mutex::new(GraphicsPreferences::default()),
            binding: Mutex::new(None),
            disposed: AtomicBool::new(false),
            disposed_event_raised: AtomicBool::new(false),
            pending_callback_error: Mutex::new(None),
            device_created: EventHandlers::new(),
            device_disposing: EventHandlers::new(),
            device_reset: EventHandlers::new(),
            device_resetting: EventHandlers::new(),
            preparing_device_settings: EventHandlers::new(),
            disposed_event: EventHandlers::new(),
        })
    }

    fn ensure_open(&self) -> Result<()> {
        if self.disposed.load(Ordering::Acquire) {
            Err(CnaError::InvalidInput(
                "graphics device manager is disposed",
            ))
        } else {
            Ok(())
        }
    }

    fn game(&self) -> Result<Arc<GameState>> {
        self.game.upgrade().ok_or(CnaError::InvalidInput(
            "graphics device manager's game is disposed",
        ))
    }

    fn native(&self) -> Result<(Arc<Native>, sys::CNA_GraphicsDeviceManagerHandle)> {
        self.ensure_open()?;
        let binding = self
            .binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let binding = binding.as_ref().ok_or(CnaError::UnsupportedRuntime(
            "GraphicsDeviceManager requires an actively running CNA Game",
        ))?;
        Ok((Arc::clone(&binding.native), binding.handle))
    }

    pub(crate) fn attach(
        self: &Arc<Self>,
        native: &Arc<Native>,
        game: sys::CNA_Handle,
        device: &GraphicsDevice,
    ) -> Result<()> {
        self.ensure_open()?;
        if self
            .binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
        {
            return Ok(());
        }
        if let Some(existing) = self.device.get() {
            if !existing.is_same_device(device) {
                return Err(CnaError::InvalidInput(
                    "graphics device manager cannot change Game device identity",
                ));
            }
        } else {
            let _ = self.device.set(device.clone());
        }

        let handle = native.create_graphics_device_manager(game)?;
        let context = Arc::as_ptr(self).cast_mut().cast::<c_void>();
        let mut registrations = Vec::new();
        let attached = (|| {
            registrations.push(native.subscribe_graphics_device_manager_event(
                handle,
                sys::CNA_GRAPHICS_DEVICE_MANAGER_EVENT_DISPOSED,
                graphics_manager_event::<0>,
                context,
            )?);
            registrations.push(native.subscribe_graphics_device_manager_event(
                handle,
                sys::CNA_GRAPHICS_DEVICE_MANAGER_EVENT_DEVICE_CREATED,
                graphics_manager_event::<1>,
                context,
            )?);
            registrations.push(native.subscribe_graphics_device_manager_event(
                handle,
                sys::CNA_GRAPHICS_DEVICE_MANAGER_EVENT_DEVICE_DISPOSING,
                graphics_manager_event::<2>,
                context,
            )?);
            registrations.push(native.subscribe_graphics_device_manager_event(
                handle,
                sys::CNA_GRAPHICS_DEVICE_MANAGER_EVENT_DEVICE_RESET,
                graphics_manager_event::<3>,
                context,
            )?);
            registrations.push(native.subscribe_graphics_device_manager_event(
                handle,
                sys::CNA_GRAPHICS_DEVICE_MANAGER_EVENT_DEVICE_RESETTING,
                graphics_manager_event::<4>,
                context,
            )?);
            registrations.push(native.subscribe_preparing_device_settings(
                handle,
                preparing_device_settings,
                context,
            )?);
            self.apply_cached_preferences(native, handle)
        })();
        if let Err(error) = attached {
            for registration in registrations.drain(..).rev() {
                let _ = native.unsubscribe_game_event(registration);
            }
            let _ = native.destroy_graphics_device_manager(handle);
            return Err(error);
        }
        *self
            .binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(NativeManagerBinding {
            native: Arc::clone(native),
            handle,
            registrations,
        });
        Ok(())
    }

    fn apply_cached_preferences(
        &self,
        native: &Native,
        handle: sys::CNA_GraphicsDeviceManagerHandle,
    ) -> Result<()> {
        let value = *self
            .preferences
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        native
            .set_graphics_device_manager_graphics_profile(handle, value.graphics_profile as u32)?;
        native.set_graphics_device_manager_is_full_screen(handle, value.is_full_screen)?;
        native.set_graphics_device_manager_prefer_multi_sampling(
            handle,
            value.prefer_multi_sampling,
        )?;
        native.set_graphics_device_manager_back_buffer_format(
            handle,
            value.preferred_back_buffer_format as u32,
        )?;
        native.set_graphics_device_manager_back_buffer_width(
            handle,
            value.preferred_back_buffer_width,
        )?;
        native.set_graphics_device_manager_back_buffer_height(
            handle,
            value.preferred_back_buffer_height,
        )?;
        native.set_graphics_device_manager_depth_stencil_format(
            handle,
            value.preferred_depth_stencil_format as u32,
        )?;
        native.set_graphics_device_manager_vertical_retrace(
            handle,
            value.synchronize_with_vertical_retrace,
        )?;
        native.set_graphics_device_manager_supported_orientations(
            handle,
            value.supported_orientations.bits() as u32,
        )
    }

    fn refresh_preferences(
        &self,
        native: &Native,
        handle: sys::CNA_GraphicsDeviceManagerHandle,
    ) -> Result<()> {
        let value = native.graphics_device_manager_preferences(handle)?;
        *self
            .preferences
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = graphics_preferences(value)?;
        Ok(())
    }

    fn operation(&self, call: impl FnOnce(&Native, sys::CNA_Handle) -> Result<()>) -> Result<()> {
        let (native, handle) = self.native()?;
        let result = call(&native, handle);
        let refresh = if result.is_ok() {
            self.refresh_preferences(&native, handle)
        } else {
            Ok(())
        };
        result?;
        refresh?;
        self.take_callback_error()
    }

    fn begin_draw(&self) -> Result<bool> {
        let (native, handle) = self.native()?;
        let value = native.begin_graphics_device_manager_draw(handle)?;
        self.take_callback_error()?;
        Ok(value)
    }

    fn end_draw(&self) -> Result<()> {
        let (native, handle) = self.native()?;
        native.end_graphics_device_manager_draw(handle)?;
        self.take_callback_error()
    }

    fn take_callback_error(&self) -> Result<()> {
        self.pending_callback_error
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .map_or(Ok(()), Err)
    }

    fn record_callback_error(&self, error: CnaError) {
        let mut pending = self
            .pending_callback_error
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if pending.is_none() {
            *pending = Some(error);
        }
    }

    fn emit_event(&self, event: u8) {
        let sender = self
            .self_weak
            .upgrade()
            .map(GraphicsDeviceManager::callback_sender);
        let sender: &dyn Any = sender
            .as_ref()
            .map_or(self as &dyn Any, |manager| manager as &dyn Any);
        let panicked = match event {
            0 => {
                if self.disposed_event_raised.swap(true, Ordering::AcqRel) {
                    false
                } else {
                    self.disposed_event.emit(sender, EventArgs)
                }
            }
            1 => self.device_created.emit(sender, EventArgs),
            2 => self.device_disposing.emit(sender, EventArgs),
            3 => self.device_reset.emit(sender, EventArgs),
            _ => self.device_resetting.emit(sender, EventArgs),
        };
        if panicked {
            self.record_callback_error(CnaError::Callback(
                "GraphicsDeviceManager event handler panicked".to_owned(),
            ));
        }
    }

    fn prepare_device_settings(
        &self,
        value: &mut sys::CNA_GraphicsDeviceInformation,
    ) -> Result<()> {
        let device = self.device.get().ok_or(CnaError::InvalidInput(
            "graphics device manager has no Game-owned device",
        ))?;
        let information = Arc::new(GraphicsDeviceInformation::from_native(*value, device)?);
        let args = PreparingDeviceSettingsEventArgs::new(Arc::clone(&information));
        let sender = self
            .self_weak
            .upgrade()
            .map(GraphicsDeviceManager::callback_sender);
        let sender: &dyn Any = sender
            .as_ref()
            .map_or(self as &dyn Any, |manager| manager as &dyn Any);
        if self.preparing_device_settings.emit(sender, args) {
            return Err(CnaError::Callback(
                "PreparingDeviceSettings handler panicked".to_owned(),
            ));
        }
        information.copy_to_native(value, device)
    }

    pub(crate) fn dispose(&self, disposing: bool) -> Result<()> {
        if !disposing || self.disposed.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let mut first_error = None;
        let binding = self
            .binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(mut binding) = binding {
            if let Err(error) = binding
                .native
                .dispose_graphics_device_manager(binding.handle)
            {
                first_error = Some(error);
            }
            for registration in binding.registrations.drain(..).rev() {
                if let Err(error) = binding.native.unsubscribe_game_event(registration) {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
            if let Err(error) = binding
                .native
                .destroy_graphics_device_manager(binding.handle)
            {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        } else {
            self.emit_event(0);
        }
        if let Some(game) = self.game.upgrade() {
            game.unregister_graphics_device_manager(self);
        }
        if first_error.is_none() {
            first_error = self
                .pending_callback_error
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
        }
        first_error.map_or(Ok(()), Err)
    }
}

impl Drop for GraphicsDeviceManagerState {
    fn drop(&mut self) {
        let _ = self.dispose(true);
    }
}

unsafe extern "C" fn graphics_manager_event<const EVENT: u8>(context: *mut c_void) {
    if context.is_null() {
        return;
    }
    // SAFETY: every registration is released before its owning state can drop.
    let state = unsafe { &*context.cast::<GraphicsDeviceManagerState>() };
    let result = catch_unwind(AssertUnwindSafe(|| state.emit_event(EVENT)));
    if result.is_err() {
        state.record_callback_error(CnaError::Callback(
            "Rust panic was contained in a GraphicsDeviceManager callback".to_owned(),
        ));
    }
}

unsafe extern "C" fn preparing_device_settings(
    information: *mut sys::CNA_GraphicsDeviceInformation,
    context: *mut c_void,
) {
    if information.is_null() || context.is_null() {
        return;
    }
    // SAFETY: CNA borrows both pointers synchronously for this callback.
    let state = unsafe { &*context.cast::<GraphicsDeviceManagerState>() };
    let result = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: non-null was checked and CNA owns the mutable candidate for this call.
        state.prepare_device_settings(unsafe { &mut *information })
    }));
    let error = match result {
        Ok(Ok(())) => return,
        Ok(Err(error)) => error,
        Err(_) => {
            CnaError::Callback("Rust panic was contained in PreparingDeviceSettings".to_owned())
        }
    };
    state.record_callback_error(error);
}

/// Game-associated device preferences and lifecycle integration.
pub struct GraphicsDeviceManager {
    state: Arc<GraphicsDeviceManagerState>,
    dispose_on_drop: bool,
}

impl GraphicsDeviceManager {
    pub const DefaultBackBufferWidth: i32 = 800;
    pub const DefaultBackBufferHeight: i32 = 480;

    #[must_use]
    pub fn new(game: &dyn Game) -> Self {
        let game = game.game_state_arc();
        let state = GraphicsDeviceManagerState::new(&game);
        game.register_graphics_device_manager(Arc::clone(&state))
            .expect("an XNA Game accepts exactly one GraphicsDeviceManager");
        Self {
            state,
            dispose_on_drop: true,
        }
    }

    fn callback_sender(state: Arc<GraphicsDeviceManagerState>) -> Self {
        Self {
            state,
            dispose_on_drop: false,
        }
    }

    pub fn GraphicsProfile(&self) -> Result<GraphicsProfile> {
        self.state.ensure_open()?;
        Ok(self.preferences().graphics_profile)
    }

    pub fn SetGraphicsProfile(&mut self, value: GraphicsProfile) -> Result<()> {
        self.set_preference(
            |native, handle| {
                native.set_graphics_device_manager_graphics_profile(handle, value as u32)
            },
            |preferences| preferences.graphics_profile = value,
        )
    }

    pub fn PreferredDepthStencilFormat(&self) -> Result<DepthFormat> {
        self.state.ensure_open()?;
        Ok(self.preferences().preferred_depth_stencil_format)
    }

    pub fn SetPreferredDepthStencilFormat(&mut self, value: DepthFormat) -> Result<()> {
        self.set_preference(
            |native, handle| {
                native.set_graphics_device_manager_depth_stencil_format(handle, value as u32)
            },
            |preferences| preferences.preferred_depth_stencil_format = value,
        )
    }

    pub fn PreferredBackBufferFormat(&self) -> Result<SurfaceFormat> {
        self.state.ensure_open()?;
        Ok(self.preferences().preferred_back_buffer_format)
    }

    pub fn SetPreferredBackBufferFormat(&mut self, value: SurfaceFormat) -> Result<()> {
        self.set_preference(
            |native, handle| {
                native.set_graphics_device_manager_back_buffer_format(handle, value as u32)
            },
            |preferences| preferences.preferred_back_buffer_format = value,
        )
    }

    pub fn PreferredBackBufferWidth(&self) -> Result<i32> {
        self.state.ensure_open()?;
        Ok(self.preferences().preferred_back_buffer_width)
    }

    pub fn SetPreferredBackBufferWidth(&mut self, value: i32) -> Result<()> {
        self.set_preference(
            |native, handle| native.set_graphics_device_manager_back_buffer_width(handle, value),
            |preferences| preferences.preferred_back_buffer_width = value,
        )
    }

    pub fn PreferredBackBufferHeight(&self) -> Result<i32> {
        self.state.ensure_open()?;
        Ok(self.preferences().preferred_back_buffer_height)
    }

    pub fn SetPreferredBackBufferHeight(&mut self, value: i32) -> Result<()> {
        self.set_preference(
            |native, handle| native.set_graphics_device_manager_back_buffer_height(handle, value),
            |preferences| preferences.preferred_back_buffer_height = value,
        )
    }

    pub fn IsFullScreen(&self) -> Result<bool> {
        self.state.ensure_open()?;
        Ok(self.preferences().is_full_screen)
    }

    pub fn SetIsFullScreen(&mut self, value: bool) -> Result<()> {
        self.set_preference(
            |native, handle| native.set_graphics_device_manager_is_full_screen(handle, value),
            |preferences| preferences.is_full_screen = value,
        )
    }

    pub fn SynchronizeWithVerticalRetrace(&self) -> Result<bool> {
        self.state.ensure_open()?;
        Ok(self.preferences().synchronize_with_vertical_retrace)
    }

    pub fn SetSynchronizeWithVerticalRetrace(&mut self, value: bool) -> Result<()> {
        self.set_preference(
            |native, handle| native.set_graphics_device_manager_vertical_retrace(handle, value),
            |preferences| preferences.synchronize_with_vertical_retrace = value,
        )
    }

    pub fn PreferMultiSampling(&self) -> Result<bool> {
        self.state.ensure_open()?;
        Ok(self.preferences().prefer_multi_sampling)
    }

    pub fn SetPreferMultiSampling(&mut self, value: bool) -> Result<()> {
        self.set_preference(
            |native, handle| {
                native.set_graphics_device_manager_prefer_multi_sampling(handle, value)
            },
            |preferences| preferences.prefer_multi_sampling = value,
        )
    }

    pub fn SupportedOrientations(&self) -> Result<DisplayOrientation> {
        self.state.ensure_open()?;
        Ok(self.preferences().supported_orientations)
    }

    pub fn SetSupportedOrientations(&mut self, value: DisplayOrientation) -> Result<()> {
        self.set_preference(
            |native, handle| {
                native
                    .set_graphics_device_manager_supported_orientations(handle, value.bits() as u32)
            },
            |preferences| preferences.supported_orientations = value,
        )?;
        if let Ok(game) = self.state.game() {
            game.Window().SetSupportedOrientations(value);
        }
        Ok(())
    }

    pub fn GraphicsDevice(&self) -> Result<&GraphicsDevice> {
        self.state.ensure_open()?;
        self.state
            .device
            .get()
            .ok_or(CnaError::InvalidInput("graphics device is not initialized"))
    }

    pub fn AddDeviceCreatedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.device_created.add(handler)
    }

    pub fn RemoveDeviceCreatedHandler(&self, registration: u64) -> bool {
        self.state.device_created.remove(registration)
    }

    pub fn AddDeviceResettingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.device_resetting.add(handler)
    }

    pub fn RemoveDeviceResettingHandler(&self, registration: u64) -> bool {
        self.state.device_resetting.remove(registration)
    }

    pub fn AddDeviceResetHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.device_reset.add(handler)
    }

    pub fn RemoveDeviceResetHandler(&self, registration: u64) -> bool {
        self.state.device_reset.remove(registration)
    }

    pub fn AddDeviceDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.device_disposing.add(handler)
    }

    pub fn RemoveDeviceDisposingHandler(&self, registration: u64) -> bool {
        self.state.device_disposing.remove(registration)
    }

    pub fn AddPreparingDeviceSettingsHandler(
        &self,
        handler: Box<dyn EventHandler<PreparingDeviceSettingsEventArgs>>,
    ) -> u64 {
        self.state.preparing_device_settings.add(handler)
    }

    pub fn RemovePreparingDeviceSettingsHandler(&self, registration: u64) -> bool {
        self.state.preparing_device_settings.remove(registration)
    }

    pub fn AddDisposedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.disposed_event.add(handler)
    }

    pub fn RemoveDisposedHandler(&self, registration: u64) -> bool {
        self.state.disposed_event.remove(registration)
    }

    pub fn ApplyChanges(&self) -> Result<()> {
        self.state
            .operation(Native::apply_graphics_device_manager_changes)
    }

    pub fn ToggleFullScreen(&self) -> Result<()> {
        self.state
            .operation(Native::toggle_graphics_device_manager_full_screen)
    }

    pub fn FindBestDevice(&self, anySuitableDevice: bool) -> Result<GraphicsDeviceInformation> {
        self.state.ensure_open()?;
        let preferences = self.preferences();
        let adapter = match self.state.device.get() {
            Some(device) => Arc::new(GraphicsAdapter::DefaultAdapter(device)?.clone()),
            None => Arc::new(GraphicsAdapter::default_placeholder()),
        };
        let parameters = Arc::new(PresentationParameters::new());
        parameters.SetBackBufferWidth(preferences.preferred_back_buffer_width);
        parameters.SetBackBufferHeight(preferences.preferred_back_buffer_height);
        parameters.SetBackBufferFormat(preferences.preferred_back_buffer_format);
        parameters.SetDepthStencilFormat(preferences.preferred_depth_stencil_format);
        parameters.SetIsFullScreen(preferences.is_full_screen);
        let _ = anySuitableDevice;
        Ok(GraphicsDeviceInformation::from_parts(
            adapter,
            preferences.graphics_profile,
            parameters,
        ))
    }

    pub fn CanResetDevice(&self, newDeviceInfo: &GraphicsDeviceInformation) -> Result<bool> {
        let device = self.GraphicsDevice()?;
        Ok(!device.IsDisposed()? && device.GraphicsProfile()? == newDeviceInfo.GraphicsProfile())
    }

    pub fn RankDevices(&self, foundDevices: &mut Vec<GraphicsDeviceInformation>) -> Result<()> {
        let _ = foundDevices;
        Err(CnaError::UnsupportedRuntime(
            "CNA ABI 0.7 does not expose GraphicsDeviceManager device-candidate ranking",
        ))
    }

    pub fn OnDeviceCreated(&self, sender: &dyn Any, args: EventArgs) -> Result<()> {
        event_result(
            self.state.device_created.emit(sender, args),
            "DeviceCreated",
        )
    }

    pub fn OnDeviceDisposing(&self, sender: &dyn Any, args: EventArgs) -> Result<()> {
        event_result(
            self.state.device_disposing.emit(sender, args),
            "DeviceDisposing",
        )
    }

    pub fn OnDeviceReset(&self, sender: &dyn Any, args: EventArgs) -> Result<()> {
        event_result(self.state.device_reset.emit(sender, args), "DeviceReset")
    }

    pub fn OnDeviceResetting(&self, sender: &dyn Any, args: EventArgs) -> Result<()> {
        event_result(
            self.state.device_resetting.emit(sender, args),
            "DeviceResetting",
        )
    }

    pub fn Dispose(&mut self, disposing: bool) -> Result<()> {
        self.state.dispose(disposing)
    }

    pub fn OnPreparingDeviceSettings(
        &self,
        sender: &dyn Any,
        args: &PreparingDeviceSettingsEventArgs,
    ) -> Result<()> {
        event_result(
            self.state
                .preparing_device_settings
                .emit(sender, args.clone()),
            "PreparingDeviceSettings",
        )
    }

    fn preferences(&self) -> GraphicsPreferences {
        *self
            .state
            .preferences
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn set_preference(
        &self,
        native_call: impl FnOnce(&Native, sys::CNA_Handle) -> Result<()>,
        update: impl FnOnce(&mut GraphicsPreferences),
    ) -> Result<()> {
        self.state.ensure_open()?;
        if let Ok((native, handle)) = self.state.native() {
            native_call(&native, handle)?;
        }
        update(
            &mut self
                .state
                .preferences
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        Ok(())
    }
}

impl IGraphicsDeviceManager for GraphicsDeviceManager {
    fn CreateDevice(&self) {
        self.state
            .operation(Native::create_graphics_device_manager_device)
            .unwrap_or_else(|error| panic!("GraphicsDeviceManager.CreateDevice failed: {error}"));
    }

    fn BeginDraw(&self) -> bool {
        self.state
            .begin_draw()
            .unwrap_or_else(|error| panic!("GraphicsDeviceManager.BeginDraw failed: {error}"))
    }

    fn EndDraw(&self) {
        self.state
            .end_draw()
            .unwrap_or_else(|error| panic!("GraphicsDeviceManager.EndDraw failed: {error}"));
    }
}

impl IGraphicsDeviceService for GraphicsDeviceManager {
    fn GraphicsDevice(&self) -> &GraphicsDevice {
        GraphicsDeviceManager::GraphicsDevice(self)
            .unwrap_or_else(|error| panic!("GraphicsDeviceManager.GraphicsDevice failed: {error}"))
    }

    fn AddDeviceCreatedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        Self::AddDeviceCreatedHandler(self, handler)
    }

    fn RemoveDeviceCreatedHandler(&self, registration: u64) -> bool {
        Self::RemoveDeviceCreatedHandler(self, registration)
    }

    fn AddDeviceDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        Self::AddDeviceDisposingHandler(self, handler)
    }

    fn RemoveDeviceDisposingHandler(&self, registration: u64) -> bool {
        Self::RemoveDeviceDisposingHandler(self, registration)
    }

    fn AddDeviceResetHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        Self::AddDeviceResetHandler(self, handler)
    }

    fn RemoveDeviceResetHandler(&self, registration: u64) -> bool {
        Self::RemoveDeviceResetHandler(self, registration)
    }

    fn AddDeviceResettingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        Self::AddDeviceResettingHandler(self, handler)
    }

    fn RemoveDeviceResettingHandler(&self, registration: u64) -> bool {
        Self::RemoveDeviceResettingHandler(self, registration)
    }
}

impl Drop for GraphicsDeviceManager {
    fn drop(&mut self) {
        if self.dispose_on_drop {
            let _ = self.state.dispose(true);
        }
    }
}

fn event_result(panicked: bool, name: &'static str) -> Result<()> {
    if panicked {
        Err(CnaError::Callback(format!(
            "GraphicsDeviceManager.{name} handler panicked"
        )))
    } else {
        Ok(())
    }
}

fn graphics_preferences(value: NativeGraphicsPreferences) -> Result<GraphicsPreferences> {
    Ok(GraphicsPreferences {
        graphics_profile: graphics_profile(value.graphics_profile)?,
        preferred_depth_stencil_format: depth_format(value.depth_stencil_format)?,
        preferred_back_buffer_format: SurfaceFormat::from_native(value.back_buffer_format)
            .ok_or_else(|| CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "CNA returned an unknown preferred back-buffer format".to_owned(),
            })?,
        preferred_back_buffer_width: value.back_buffer_width,
        preferred_back_buffer_height: value.back_buffer_height,
        is_full_screen: value.is_full_screen,
        synchronize_with_vertical_retrace: value.synchronize_with_vertical_retrace,
        prefer_multi_sampling: value.prefer_multi_sampling,
        supported_orientations: DisplayOrientation::from_bits(
            i32::try_from(value.supported_orientations).map_err(|_| {
                CnaError::InvalidInput("supported orientations exceed the Rust flags value")
            })?,
        ),
    })
}

fn graphics_profile(value: u32) -> Result<GraphicsProfile> {
    match value {
        sys::CNA_GRAPHICS_PROFILE_REACH => Ok(GraphicsProfile::Reach),
        sys::CNA_GRAPHICS_PROFILE_HI_DEF => Ok(GraphicsProfile::HiDef),
        _ => Err(CnaError::Native {
            code: sys::CNA_RESULT_INTERNAL,
            message: "CNA returned an unknown graphics profile".to_owned(),
        }),
    }
}

fn depth_format(value: u32) -> Result<DepthFormat> {
    match value {
        sys::CNA_DEPTH_FORMAT_NONE => Ok(DepthFormat::None),
        sys::CNA_DEPTH_FORMAT_DEPTH16 => Ok(DepthFormat::Depth16),
        sys::CNA_DEPTH_FORMAT_DEPTH24 => Ok(DepthFormat::Depth24),
        sys::CNA_DEPTH_FORMAT_DEPTH24_STENCIL8 => Ok(DepthFormat::Depth24Stencil8),
        _ => Err(CnaError::Native {
            code: sys::CNA_RESULT_INTERNAL,
            message: "CNA returned an unknown depth/stencil format".to_owned(),
        }),
    }
}

pub(crate) fn manager_service_type_ids() -> (TypeId, TypeId) {
    (
        TypeId::of::<dyn IGraphicsDeviceManager>(),
        TypeId::of::<dyn IGraphicsDeviceService>(),
    )
}
