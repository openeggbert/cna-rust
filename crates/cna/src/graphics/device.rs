#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]

use core::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::{EventArgs, EventHandler};
use crate::extensions::window::WindowHandle;
use crate::native::Native;
use crate::value::{Color, Rectangle, Vector4};

use super::resource::{EventHandlers, ResourceState};
use super::{
    BlendState, DepthStencilState, DisplayMode, GraphicsAdapter, GraphicsDeviceStatus,
    GraphicsProfile, PresentationParameters, RasterizerState, ResourceCreatedEventArgs,
    ResourceDestroyedEventArgs, SamplerStateCollection, TextureCollection, Viewport,
};

/// Shared validity and child-resource registry for one game-owned device.
pub(super) struct DeviceState {
    native: Arc<Native>,
    game: sys::CNA_Handle,
    handle: Mutex<sys::CNA_Handle>,
    alive: AtomicBool,
    resources: Mutex<Vec<Weak<ResourceState>>>,
    presentation_parameters: PresentationParameters,
    display_mode: OnceLock<DisplayMode>,
    disposing: EventHandlers<EventArgs>,
    resource_created: EventHandlers<ResourceCreatedEventArgs>,
    resource_destroyed: EventHandlers<ResourceDestroyedEventArgs>,
    device_lost: EventHandlers<EventArgs>,
    device_reset: EventHandlers<EventArgs>,
    device_resetting: EventHandlers<EventArgs>,
    blend_state: Mutex<Option<Arc<BlendState>>>,
    depth_stencil_state: Mutex<Option<Arc<DepthStencilState>>>,
    rasterizer_state: Mutex<Option<Arc<RasterizerState>>>,
    sampler_states: OnceLock<SamplerStateCollection>,
    vertex_sampler_states: OnceLock<SamplerStateCollection>,
    textures: OnceLock<TextureCollection>,
    vertex_textures: OnceLock<TextureCollection>,
    pub(super) adapters: OnceLock<Vec<GraphicsAdapter>>,
}

impl DeviceState {
    pub(super) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.ensure_alive()?;
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            Err(CnaError::InvalidInput(
                "graphics-device operations require an active game callback",
            ))
        } else {
            Ok(handle)
        }
    }

    pub(super) fn ensure_alive(&self) -> Result<()> {
        if self.alive.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(CnaError::InvalidInput("graphics device is disposed"))
        }
    }

    fn enter_callback(&self) -> Result<()> {
        self.ensure_alive()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        self.native.borrow_graphics_device(self.game, &mut handle)?;
        *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = handle;
        Ok(())
    }

    fn leave_callback(&self) {
        *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = sys::CNA_INVALID_HANDLE;
    }

    pub(super) fn native(&self) -> &Arc<Native> {
        &self.native
    }

    pub(super) fn register(&self, resource: &Arc<ResourceState>) {
        let mut resources = self
            .resources
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        resources.retain(|entry| entry.strong_count() != 0);
        resources.push(Arc::downgrade(resource));
    }

    pub(super) fn dispose_resources(&self) -> Result<()> {
        let resources = self
            .resources
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>();
        let mut first_error = None;
        for resource in resources.into_iter().rev() {
            if let Err(error) = resource.dispose_native() {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
        if let Some(error) = first_error {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub(super) fn invalidate(&self) -> bool {
        self.leave_callback();
        self.alive.swap(false, Ordering::AcqRel)
    }
}

/// Durable safe identity for a game-owned XNA graphics device.
///
/// Clones share one logical device; they never take ownership of CNA's native
/// device, which remains owned by the game host.
#[derive(Clone)]
pub struct GraphicsDevice {
    pub(super) state: Arc<DeviceState>,
}

#[allow(non_snake_case)]
impl GraphicsDevice {
    pub(crate) fn bind(native: &Arc<Native>, game: sys::CNA_Handle) -> Self {
        Self {
            state: Arc::new(DeviceState {
                native: Arc::clone(native),
                game,
                handle: Mutex::new(sys::CNA_INVALID_HANDLE),
                alive: AtomicBool::new(true),
                resources: Mutex::new(Vec::new()),
                presentation_parameters: PresentationParameters::new(),
                display_mode: OnceLock::new(),
                disposing: EventHandlers::new(),
                resource_created: EventHandlers::new(),
                resource_destroyed: EventHandlers::new(),
                device_lost: EventHandlers::new(),
                device_reset: EventHandlers::new(),
                device_resetting: EventHandlers::new(),
                blend_state: Mutex::new(None),
                depth_stencil_state: Mutex::new(None),
                rasterizer_state: Mutex::new(None),
                sampler_states: OnceLock::new(),
                vertex_sampler_states: OnceLock::new(),
                textures: OnceLock::new(),
                vertex_textures: OnceLock::new(),
                adapters: OnceLock::new(),
            }),
        }
    }

    pub(super) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.state.handle()
    }

    pub fn IsDisposed(&self) -> Result<bool> {
        Ok(!self.state.alive.load(Ordering::Acquire))
    }

    pub fn GraphicsDeviceStatus(&self) -> Result<GraphicsDeviceStatus> {
        let mut value = sys::CNA_GRAPHICS_DEVICE_STATUS_NORMAL;
        self.state
            .native
            .graphics_device_status(self.state.handle()?, &mut value)?;
        match value {
            sys::CNA_GRAPHICS_DEVICE_STATUS_NORMAL => Ok(GraphicsDeviceStatus::Normal),
            sys::CNA_GRAPHICS_DEVICE_STATUS_LOST => Ok(GraphicsDeviceStatus::Lost),
            sys::CNA_GRAPHICS_DEVICE_STATUS_NOT_RESET => Ok(GraphicsDeviceStatus::NotReset),
            _ => Err(CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "CNA returned an unknown GraphicsDeviceStatus".to_owned(),
            }),
        }
    }

    pub fn Adapter(&self) -> Result<&GraphicsAdapter> {
        let index = self
            .state
            .native
            .graphics_adapter_index(self.state.handle()?)?;
        GraphicsAdapter::all(self)?
            .get(index as usize)
            .ok_or(CnaError::Native {
                code: sys::CNA_RESULT_INVALID_STATE,
                message: "graphics-device adapter index is outside the current adapter set"
                    .to_owned(),
            })
    }

    pub fn GraphicsProfile(&self) -> Result<GraphicsProfile> {
        let mut value = sys::CNA_GRAPHICS_PROFILE_REACH;
        self.state
            .native
            .graphics_profile(self.state.handle()?, &mut value)?;
        match value {
            sys::CNA_GRAPHICS_PROFILE_REACH => Ok(GraphicsProfile::Reach),
            sys::CNA_GRAPHICS_PROFILE_HI_DEF => Ok(GraphicsProfile::HiDef),
            _ => Err(CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "CNA returned an unknown GraphicsProfile".to_owned(),
            }),
        }
    }

    pub fn PresentationParameters(&self) -> Result<&PresentationParameters> {
        let mut value = sys::CNA_PresentationParameters {
            struct_size: size_of::<sys::CNA_PresentationParameters>() as u32,
            struct_version: 1,
            ..sys::CNA_PresentationParameters::default()
        };
        self.state
            .native
            .presentation_parameters(self.state.handle()?, &mut value)?;
        let window_handle = WindowHandle(
            self.state
                .native
                .game_window_native_handle(self.state.game)?,
        );
        if !self
            .state
            .presentation_parameters
            .update_from_native(value, window_handle)
        {
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "CNA returned invalid PresentationParameters identities".to_owned(),
            });
        }
        Ok(&self.state.presentation_parameters)
    }

    pub fn DisplayMode(&self) -> Result<&DisplayMode> {
        if let Some(value) = self.state.display_mode.get() {
            return Ok(value);
        }
        let mut native = sys::CNA_DisplayMode {
            struct_size: size_of::<sys::CNA_DisplayMode>() as u32,
            struct_version: 1,
            ..sys::CNA_DisplayMode::default()
        };
        self.state
            .native
            .display_mode(self.state.handle()?, &mut native)?;
        let value = DisplayMode::from_native(native).ok_or_else(|| CnaError::Native {
            code: sys::CNA_RESULT_INTERNAL,
            message: "CNA returned an unknown display surface format".to_owned(),
        })?;
        let _ = self.state.display_mode.set(value);
        self.state.display_mode.get().ok_or(CnaError::Native {
            code: sys::CNA_RESULT_INTERNAL,
            message: "display-mode identity could not be initialized".to_owned(),
        })
    }

    pub fn SamplerStates(&self) -> Result<&SamplerStateCollection> {
        self.state.ensure_alive()?;
        Ok(self
            .state
            .sampler_states
            .get_or_init(|| SamplerStateCollection::pixel(&self.state)))
    }

    pub fn VertexSamplerStates(&self) -> Result<&SamplerStateCollection> {
        self.state.ensure_alive()?;
        Ok(self
            .state
            .vertex_sampler_states
            .get_or_init(|| SamplerStateCollection::vertex(&self.state)))
    }

    pub fn Textures(&self) -> Result<&TextureCollection> {
        self.state.ensure_alive()?;
        Ok(self
            .state
            .textures
            .get_or_init(|| TextureCollection::pixel(&self.state)))
    }

    pub fn VertexTextures(&self) -> Result<&TextureCollection> {
        self.state.ensure_alive()?;
        Ok(self
            .state
            .vertex_textures
            .get_or_init(|| TextureCollection::vertex(&self.state)))
    }

    pub fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.disposing.add(handler)
    }

    pub fn RemoveDisposingHandler(&self, registration: u64) -> bool {
        self.state.disposing.remove(registration)
    }

    pub fn AddResourceCreatedHandler(
        &self,
        handler: Box<dyn EventHandler<ResourceCreatedEventArgs>>,
    ) -> u64 {
        self.state.resource_created.add(handler)
    }

    pub fn RemoveResourceCreatedHandler(&self, registration: u64) -> bool {
        self.state.resource_created.remove(registration)
    }

    pub fn AddResourceDestroyedHandler(
        &self,
        handler: Box<dyn EventHandler<ResourceDestroyedEventArgs>>,
    ) -> u64 {
        self.state.resource_destroyed.add(handler)
    }

    pub fn RemoveResourceDestroyedHandler(&self, registration: u64) -> bool {
        self.state.resource_destroyed.remove(registration)
    }

    pub fn AddDeviceLostHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.device_lost.add(handler)
    }

    pub fn RemoveDeviceLostHandler(&self, registration: u64) -> bool {
        self.state.device_lost.remove(registration)
    }

    pub fn AddDeviceResetHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.device_reset.add(handler)
    }

    pub fn RemoveDeviceResetHandler(&self, registration: u64) -> bool {
        self.state.device_reset.remove(registration)
    }

    pub fn AddDeviceResettingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.device_resetting.add(handler)
    }

    pub fn RemoveDeviceResettingHandler(&self, registration: u64) -> bool {
        self.state.device_resetting.remove(registration)
    }

    pub(crate) fn is_same_device(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }

    pub fn Clear(
        &self,
        options: crate::Microsoft::Xna::Framework::Graphics::ClearOptions,
        color: Vector4,
        depth: f32,
        stencil: i32,
    ) -> Result<()> {
        let _ = (depth, stencil);
        if options != crate::Microsoft::Xna::Framework::Graphics::ClearOptions::Target {
            return Err(CnaError::UnsupportedRuntime(
                "CNA ABI 0.7 exposes color-target clear but not the mapped depth/stencil clear route",
            ));
        }
        self.clear_rgba([color.X, color.Y, color.Z, color.W])
    }

    pub fn ClearWithOptionsAndColorAndDepthAndStencil(
        &self,
        options: crate::Microsoft::Xna::Framework::Graphics::ClearOptions,
        color: Color,
        depth: f32,
        stencil: i32,
    ) -> Result<()> {
        self.Clear(options, color.ToVector4(), depth, stencil)
    }

    pub fn ClearWithColor(&self, color: Color) -> Result<()> {
        let scale = 1.0 / 255.0;
        self.clear_rgba([
            f32::from(color.R()) * scale,
            f32::from(color.G()) * scale,
            f32::from(color.B()) * scale,
            f32::from(color.A()) * scale,
        ])
    }

    fn clear_rgba(&self, rgba: [f32; 4]) -> Result<()> {
        self.state
            .native
            .clear_graphics_device(self.state.handle()?, rgba)
    }

    pub fn Viewport(&self) -> Result<Viewport> {
        let mut viewport = sys::CNA_Viewport::default();
        self.state
            .native
            .graphics_viewport(self.state.handle()?, &mut viewport)?;
        Ok(Viewport::from_native(viewport))
    }

    pub fn SetViewport(&mut self, value: Viewport) -> Result<()> {
        self.state
            .native
            .set_graphics_viewport(self.state.handle()?, value.to_native())
    }

    pub fn ScissorRectangle(&self) -> Result<Rectangle> {
        let mut value = sys::CNA_Rectangle::default();
        self.state
            .native
            .graphics_scissor_rectangle(self.state.handle()?, &mut value)?;
        Ok(Rectangle::new(value.x, value.y, value.width, value.height))
    }

    pub fn SetScissorRectangle(&mut self, value: Rectangle) -> Result<()> {
        self.state.native.set_graphics_scissor_rectangle(
            self.state.handle()?,
            sys::CNA_Rectangle {
                x: value.X,
                y: value.Y,
                width: value.Width,
                height: value.Height,
            },
        )
    }

    pub fn BlendFactor(&self) -> Result<Color> {
        let mut value = sys::CNA_Color::default();
        self.state
            .native
            .graphics_blend_factor(self.state.handle()?, &mut value)?;
        Ok(
            Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
                i32::from(value.r),
                i32::from(value.g),
                i32::from(value.b),
                i32::from(value.a),
            ),
        )
    }

    pub fn SetBlendFactor(&mut self, value: Color) -> Result<()> {
        self.state.native.set_graphics_blend_factor(
            self.state.handle()?,
            sys::CNA_Color {
                r: value.R(),
                g: value.G(),
                b: value.B(),
                a: value.A(),
            },
        )
    }

    pub fn MultiSampleMask(&self) -> Result<i32> {
        let mut value = 0;
        self.state
            .native
            .graphics_multi_sample_mask(self.state.handle()?, &mut value)?;
        Ok(value)
    }

    pub fn SetMultiSampleMask(&mut self, value: i32) -> Result<()> {
        self.state
            .native
            .set_graphics_multi_sample_mask(self.state.handle()?, value)
    }

    pub fn ReferenceStencil(&self) -> Result<i32> {
        let mut value = 0;
        self.state
            .native
            .graphics_reference_stencil(self.state.handle()?, &mut value)?;
        Ok(value)
    }

    pub fn SetReferenceStencil(&mut self, value: i32) -> Result<()> {
        self.state
            .native
            .set_graphics_reference_stencil(self.state.handle()?, value)
    }

    pub fn BlendState(&self) -> Result<Arc<BlendState>> {
        if let Some(value) = self
            .state
            .blend_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            return Ok(Arc::clone(value));
        }
        let mut native = sys::CNA_BlendState {
            struct_size: size_of::<sys::CNA_BlendState>() as u32,
            struct_version: 1,
            ..sys::CNA_BlendState::default()
        };
        self.state
            .native
            .blend_state(self.state.handle()?, &mut native)?;
        let value =
            Arc::new(
                BlendState::from_native(native, self).ok_or_else(|| CnaError::Native {
                    code: sys::CNA_RESULT_INTERNAL,
                    message: "CNA returned invalid BlendState identities".to_owned(),
                })?,
            );
        let mut cached = self
            .state
            .blend_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(Arc::clone(cached.get_or_insert(value)))
    }

    pub fn SetBlendState(&mut self, value: Arc<BlendState>) -> Result<()> {
        value.bind(self)?;
        self.state
            .native
            .set_graphics_blend_state(self.state.handle()?, &value.native())?;
        *self
            .state
            .blend_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(value);
        Ok(())
    }

    pub fn DepthStencilState(&self) -> Result<Arc<DepthStencilState>> {
        if let Some(value) = self
            .state
            .depth_stencil_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            return Ok(Arc::clone(value));
        }
        let mut native = sys::CNA_DepthStencilState {
            struct_size: size_of::<sys::CNA_DepthStencilState>() as u32,
            struct_version: 1,
            ..sys::CNA_DepthStencilState::default()
        };
        self.state
            .native
            .depth_stencil_state(self.state.handle()?, &mut native)?;
        let value = Arc::new(DepthStencilState::from_native(native, self).ok_or_else(|| {
            CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "CNA returned invalid DepthStencilState identities".to_owned(),
            }
        })?);
        let mut cached = self
            .state
            .depth_stencil_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(Arc::clone(cached.get_or_insert(value)))
    }

    pub fn SetDepthStencilState(&mut self, value: Arc<DepthStencilState>) -> Result<()> {
        value.bind(self)?;
        self.state
            .native
            .set_graphics_depth_stencil_state(self.state.handle()?, &value.native())?;
        *self
            .state
            .depth_stencil_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(value);
        Ok(())
    }

    pub fn RasterizerState(&self) -> Result<Arc<RasterizerState>> {
        if let Some(value) = self
            .state
            .rasterizer_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            return Ok(Arc::clone(value));
        }
        let mut native = sys::CNA_RasterizerState {
            struct_size: size_of::<sys::CNA_RasterizerState>() as u32,
            struct_version: 1,
            ..sys::CNA_RasterizerState::default()
        };
        self.state
            .native
            .rasterizer_state(self.state.handle()?, &mut native)?;
        let value = Arc::new(RasterizerState::from_native(native, self).ok_or_else(|| {
            CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "CNA returned invalid RasterizerState identities".to_owned(),
            }
        })?);
        let mut cached = self
            .state
            .rasterizer_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(Arc::clone(cached.get_or_insert(value)))
    }

    pub fn SetRasterizerState(&mut self, value: Arc<RasterizerState>) -> Result<()> {
        value.bind(self)?;
        self.state
            .native
            .set_graphics_rasterizer_state(self.state.handle()?, &value.native())?;
        *self
            .state
            .rasterizer_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(value);
        Ok(())
    }

    pub fn Present(
        &self,
        sourceRectangle: Option<Rectangle>,
        destinationRectangle: Option<Rectangle>,
        overrideWindowHandle: WindowHandle,
    ) -> Result<()> {
        if sourceRectangle.is_some()
            || destinationRectangle.is_some()
            || overrideWindowHandle != WindowHandle::default()
        {
            return Err(CnaError::UnsupportedRuntime(
                "CNA ABI 0.7 exposes only whole-backbuffer presentation to the current window",
            ));
        }
        self.PresentWithNoArguments()
    }

    pub fn PresentWithNoArguments(&self) -> Result<()> {
        self.state
            .native
            .present_graphics_device(self.state.handle()?)
    }

    pub fn Dispose(&mut self, value: bool) -> Result<()> {
        let _ = value;
        if self.IsDisposed()? {
            return Ok(());
        }
        Err(CnaError::UnsupportedRuntime(
            "CNA ABI 0.7 has no independent game-owned GraphicsDevice dispose route",
        ))
    }

    pub fn DisposeWithNoArguments(&mut self) -> Result<()> {
        self.Dispose(true)
    }

    pub fn Finalize(&self) {}

    pub(crate) fn dispose_resources(&self) -> Result<()> {
        self.state.dispose_resources()
    }

    pub(crate) fn enter_callback(&self) -> Result<()> {
        self.state.enter_callback()
    }

    pub(crate) fn leave_callback(&self) {
        self.state.leave_callback();
    }

    pub(crate) fn invalidate(&self) {
        if self.state.invalidate() {
            let _ = self.state.disposing.emit(self, EventArgs);
        }
    }

    pub(crate) fn renderer_info(&self) -> Result<(String, bool, bool, u32)> {
        let handle = self.state.handle()?;
        let mut info = sys::CNA_RendererInfo {
            struct_size: size_of::<sys::CNA_RendererInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_RendererInfo::default()
        };
        self.state.native.renderer_info(handle, &mut info)?;
        let mut name_size = 0_u64;
        self.state
            .native
            .renderer_name_size(handle, &mut name_size)?;
        let capacity = usize::try_from(name_size)
            .map_err(|_| CnaError::InvalidInput("renderer name is too large"))?;
        let mut bytes = vec![0_u8; capacity];
        let mut copied = 0_u64;
        self.state
            .native
            .copy_renderer_name(handle, &mut bytes, &mut copied)?;
        let name = String::from_utf8_lossy(&bytes).into_owned();
        let supports_3d =
            info.capability_flags & (1_u64 << sys::CNA_GRAPHICS_CAPABILITY_THREE_D) != 0;
        let supports_depth = info.capability_flags
            & (1_u64 << sys::CNA_GRAPHICS_CAPABILITY_DEPTH_STENCIL_BUFFER)
            != 0;
        Ok((
            name,
            supports_3d,
            supports_depth,
            info.max_texture_dimension,
        ))
    }
}

impl Drop for GraphicsDevice {
    fn drop(&mut self) {
        // The native handle is parent-owned. Dropping an alias releases only
        // this Rust reference; the host performs deterministic invalidation.
    }
}
