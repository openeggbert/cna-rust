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

use crate::error::{CnaError, ErrorCategory, Result};
use crate::extensions::events::{EventArgs, EventHandler};
use crate::extensions::window::WindowHandle;
use crate::native::Native;
use crate::value::{Color, Rectangle, Vector4};

use super::buffer::encode_vertices;
use super::resource::{EventHandlers, ResourceKind, ResourceState};
use super::{
    BlendState, DepthStencilState, DisplayMode, GraphicsAdapter, GraphicsDeviceStatus,
    GraphicsProfile, IndexBuffer, PresentationParameters, PrimitiveType, RasterizerState,
    RenderTarget2D, RenderTargetBinding, RenderTargetCube, ResourceCreatedEventArgs,
    ResourceDestroyedEventArgs, SamplerStateCollection, TextureCollection, VertexBuffer,
    VertexBufferBinding, VertexData, VertexDeclaration, Viewport,
};

mod back_buffer_data_sealed {
    pub trait Sealed {}
}

/// Safe element contract for the ABI-0.7 RGBA8 back-buffer transfer route.
///
/// The trait is sealed because CNA currently guarantees only XNA `Color`
/// readback. Broader XNA element formats are rejected structurally instead of
/// being reinterpreted as arbitrary Rust memory.
pub trait BackBufferData: back_buffer_data_sealed::Sealed + Copy + Send + Sync + 'static {
    #[doc(hidden)]
    fn from_color(value: Color) -> Self;
}

impl back_buffer_data_sealed::Sealed for Color {}

impl BackBufferData for Color {
    fn from_color(value: Color) -> Self {
        value
    }
}

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
    bound_vertex_buffers: Mutex<Vec<VertexBufferBinding>>,
    bound_vertex_handles: Mutex<Vec<sys::CNA_Handle>>,
    bound_index_buffer: Mutex<Option<IndexBuffer>>,
    bound_index_handle: Mutex<sys::CNA_Handle>,
    bound_render_targets: Mutex<Vec<RenderTargetBinding>>,
    bound_render_target_handles: Mutex<Vec<sys::CNA_Handle>>,
    pub(super) adapters: OnceLock<Vec<GraphicsAdapter>>,
}

impl DeviceState {
    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
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

    pub(super) fn is_resource_bound(&self, kind: ResourceKind, handle: sys::CNA_Handle) -> bool {
        match kind {
            ResourceKind::VertexBuffer => self
                .bound_vertex_handles
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .contains(&handle),
            ResourceKind::IndexBuffer => {
                *self
                    .bound_index_handle
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    == handle
            }
            ResourceKind::RenderTarget2D | ResourceKind::RenderTargetCube => self
                .bound_render_target_handles
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .contains(&handle),
            ResourceKind::Texture2D
            | ResourceKind::Texture3D
            | ResourceKind::TextureCube
            | ResourceKind::SpriteBatch
            | ResourceKind::SpriteFont
            | ResourceKind::Effect
            | ResourceKind::OcclusionQuery => false,
        }
    }

    fn unbind_all_buffers(&self) -> Result<()> {
        let device = self.handle()?;
        self.native.set_graphics_vertex_buffers(device, &[])?;
        self.native
            .set_graphics_index_buffer(device, sys::CNA_INVALID_HANDLE)?;
        self.bound_vertex_buffers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.bound_vertex_handles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        *self
            .bound_index_buffer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        *self
            .bound_index_handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = sys::CNA_INVALID_HANDLE;
        Ok(())
    }

    fn unbind_all_render_targets(&self) -> Result<()> {
        self.native.set_render_targets(self.handle()?, &[])?;
        self.bound_render_targets
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.bound_render_target_handles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        Ok(())
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
                bound_vertex_buffers: Mutex::new(Vec::new()),
                bound_vertex_handles: Mutex::new(Vec::new()),
                bound_index_buffer: Mutex::new(None),
                bound_index_handle: Mutex::new(sys::CNA_INVALID_HANDLE),
                bound_render_targets: Mutex::new(Vec::new()),
                bound_render_target_handles: Mutex::new(Vec::new()),
                adapters: OnceLock::new(),
            }),
        }
    }

    pub fn new(
        adapter: &GraphicsAdapter,
        graphicsProfile: GraphicsProfile,
        presentationParameters: &PresentationParameters,
    ) -> Result<Self> {
        let _ = (adapter, graphicsProfile, presentationParameters);
        Err(CnaError::UnsupportedRuntime(
            "CNA ABI 0.7 exposes only the game-owned GraphicsDevice; it has no independent device constructor",
        ))
    }

    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
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
                category: ErrorCategory::None,
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
                category: ErrorCategory::None,
                message: "graphics-device adapter index is outside the current adapter set"
                    .to_owned(),
            })
    }

    pub(crate) fn proposal_adapter_index_for(&self, adapter: &GraphicsAdapter) -> Result<i32> {
        adapter.proposal_index_for(&self.state)
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
                category: ErrorCategory::None,
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
                category: ErrorCategory::None,
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
            category: ErrorCategory::None,
            message: "CNA returned an unknown display surface format".to_owned(),
        })?;
        let _ = self.state.display_mode.set(value);
        self.state.display_mode.get().ok_or(CnaError::Native {
            code: sys::CNA_RESULT_INTERNAL,
            category: ErrorCategory::None,
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
                    category: ErrorCategory::None,
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
                category: ErrorCategory::None,
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
                category: ErrorCategory::None,
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

    pub fn Indices(&self) -> Result<Option<IndexBuffer>> {
        let device = self.state.handle()?;
        let mut native_handle = sys::CNA_INVALID_HANDLE;
        self.state
            .native
            .graphics_index_buffer(device, &mut native_handle)?;
        let cached = self
            .state
            .bound_index_buffer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        match cached {
            Some(buffer) if buffer.handle()? == native_handle => Ok(Some(buffer)),
            None if native_handle == sys::CNA_INVALID_HANDLE => Ok(None),
            _ => Err(CnaError::Native {
                code: sys::CNA_RESULT_INVALID_STATE,
                category: ErrorCategory::None,
                message:
                    "native index-buffer binding changed outside CNA-Rust's safe identity registry"
                        .to_owned(),
            }),
        }
    }

    pub fn SetIndices(&mut self, value: Option<&IndexBuffer>) -> Result<()> {
        let native_handle = match value {
            Some(buffer) => {
                if !buffer.is_same_device(self) {
                    return Err(CnaError::InvalidInput(
                        "index buffer belongs to a different graphics device",
                    ));
                }
                buffer.handle()?
            }
            None => sys::CNA_INVALID_HANDLE,
        };
        self.state
            .native
            .set_graphics_index_buffer(self.state.handle()?, native_handle)?;
        *self
            .state
            .bound_index_buffer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value.cloned();
        *self
            .state
            .bound_index_handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = native_handle;
        Ok(())
    }

    pub fn GetVertexBuffers(&self) -> Result<Vec<VertexBufferBinding>> {
        let device = self.state.handle()?;
        let mut count = 0_u64;
        self.state
            .native
            .graphics_vertex_buffer_count(device, &mut count)?;
        let count_usize = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("native vertex binding count is too large"))?;
        let mut native_bindings = vec![sys::CNA_VertexBufferBinding::default(); count_usize];
        let mut copied = 0_u64;
        self.state.native.copy_graphics_vertex_buffers(
            device,
            &mut native_bindings,
            &mut copied,
        )?;
        if copied != count {
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                category: ErrorCategory::None,
                message: "CNA returned inconsistent vertex binding counts".to_owned(),
            });
        }
        let mut first = sys::CNA_INVALID_HANDLE;
        self.state
            .native
            .graphics_vertex_buffer(device, &mut first)?;
        let expected_first = native_bindings
            .first()
            .map_or(sys::CNA_INVALID_HANDLE, |binding| binding.vertex_buffer);
        if first != expected_first {
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                category: ErrorCategory::None,
                message: "CNA returned inconsistent first vertex-buffer identity".to_owned(),
            });
        }
        let cached = self
            .state
            .bound_vertex_buffers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if cached.len() != native_bindings.len() {
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INVALID_STATE,
                category: ErrorCategory::None,
                message: "native vertex bindings changed outside CNA-Rust's safe identity registry"
                    .to_owned(),
            });
        }
        for (logical, native) in cached.iter().zip(&native_bindings) {
            if logical.VertexBuffer().handle()? != native.vertex_buffer
                || logical.VertexOffset() != native.vertex_offset
                || logical.InstanceFrequency() != native.instance_frequency
            {
                return Err(CnaError::Native {
                    code: sys::CNA_RESULT_INVALID_STATE,
                    category: ErrorCategory::None,
                    message: "native vertex binding identity differs from CNA-Rust's safe registry"
                        .to_owned(),
                });
            }
        }
        Ok(cached)
    }

    pub fn SetVertexBuffer(&self, vertexBuffer: &VertexBuffer, vertexOffset: i32) -> Result<()> {
        if !vertexBuffer.is_same_device(self) {
            return Err(CnaError::InvalidInput(
                "vertex buffer belongs to a different graphics device",
            ));
        }
        let binding =
            VertexBufferBinding::from_vertex_buffer_and_vertex_offset(vertexBuffer, vertexOffset)?;
        self.state.native.set_graphics_vertex_buffer(
            self.state.handle()?,
            vertexBuffer.handle()?,
            vertexOffset,
        )?;
        *self
            .state
            .bound_vertex_buffers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = vec![binding];
        *self
            .state
            .bound_vertex_handles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = vec![vertexBuffer.handle()?];
        Ok(())
    }

    pub fn SetVertexBufferWithVertexBuffer(&self, vertexBuffer: &VertexBuffer) -> Result<()> {
        self.SetVertexBuffer(vertexBuffer, 0)
    }

    pub fn SetVertexBuffers(&self, vertexBuffers: &[VertexBufferBinding]) -> Result<()> {
        let mut native = Vec::with_capacity(vertexBuffers.len());
        let mut handles = Vec::with_capacity(vertexBuffers.len());
        for binding in vertexBuffers {
            if !binding.VertexBuffer().is_same_device(self) {
                return Err(CnaError::InvalidInput(
                    "vertex buffer belongs to a different graphics device",
                ));
            }
            native.push(binding.to_native()?);
            handles.push(binding.VertexBuffer().handle()?);
        }
        self.state
            .native
            .set_graphics_vertex_buffers(self.state.handle()?, &native)?;
        *self
            .state
            .bound_vertex_buffers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = vertexBuffers.to_vec();
        *self
            .state
            .bound_vertex_handles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = handles;
        Ok(())
    }

    pub fn DrawPrimitives(
        &self,
        primitiveType: PrimitiveType,
        startVertex: i32,
        primitiveCount: i32,
    ) -> Result<()> {
        let required = primitive_element_count(primitiveType, primitiveCount)?;
        let start = usize::try_from(startVertex)
            .map_err(|_| CnaError::InvalidInput("start vertex must not be negative"))?;
        let bindings = self
            .state
            .bound_vertex_buffers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let binding = bindings.first().ok_or(CnaError::InvalidInput(
            "DrawPrimitives requires a bound vertex buffer",
        ))?;
        binding.VertexBuffer().handle()?;
        let begin = start
            .checked_add(binding.VertexOffset() as usize)
            .ok_or(CnaError::InvalidInput("vertex draw range overflows"))?;
        let end = begin
            .checked_add(required)
            .ok_or(CnaError::InvalidInput("vertex draw range overflows"))?;
        if end > binding.VertexBuffer().VertexCount() as usize {
            return Err(CnaError::InvalidInput(
                "vertex draw range exceeds the bound buffer",
            ));
        }
        self.state.native.draw_primitives(
            self.state.handle()?,
            primitiveType as u32,
            startVertex,
            primitiveCount,
        )
    }

    pub fn DrawIndexedPrimitives(
        &self,
        primitiveType: PrimitiveType,
        baseVertex: i32,
        minVertexIndex: i32,
        numVertices: i32,
        startIndex: i32,
        primitiveCount: i32,
    ) -> Result<()> {
        self.validate_indexed_draw(
            primitiveType,
            baseVertex,
            minVertexIndex,
            numVertices,
            startIndex,
            primitiveCount,
        )?;
        self.state.native.draw_indexed_primitives(
            self.state.handle()?,
            primitiveType as u32,
            baseVertex,
            minVertexIndex,
            numVertices,
            startIndex,
            primitiveCount,
        )
    }

    pub fn DrawInstancedPrimitives(
        &self,
        primitiveType: PrimitiveType,
        baseVertex: i32,
        minVertexIndex: i32,
        numVertices: i32,
        startIndex: i32,
        primitiveCount: i32,
        instanceCount: i32,
    ) -> Result<()> {
        self.validate_indexed_draw(
            primitiveType,
            baseVertex,
            minVertexIndex,
            numVertices,
            startIndex,
            primitiveCount,
        )?;
        if instanceCount <= 0 {
            return Err(CnaError::InvalidInput(
                "instance count must be greater than zero",
            ));
        }
        self.state.native.draw_instanced_primitives(
            self.state.handle()?,
            primitiveType as u32,
            baseVertex,
            minVertexIndex,
            numVertices,
            startIndex,
            primitiveCount,
            instanceCount,
        )
    }

    pub fn DrawUserPrimitives<T: VertexData>(
        &self,
        primitiveType: PrimitiveType,
        vertexData: &[T],
        vertexOffset: i32,
        primitiveCount: i32,
    ) -> Result<()> {
        self.draw_user_primitives(
            primitiveType,
            vertexData,
            vertexOffset,
            primitiveCount,
            T::vertex_declaration(),
        )
    }

    pub fn DrawUserPrimitivesWithPrimitiveTypeAndVertexDataAndVertexOffsetAndPrimitiveCountAndVertexDeclaration<
        T: VertexData,
    >(
        &self,
        primitiveType: PrimitiveType,
        vertexData: &[T],
        vertexOffset: i32,
        primitiveCount: i32,
        vertexDeclaration: &VertexDeclaration,
    ) -> Result<()> {
        self.draw_user_primitives(
            primitiveType,
            vertexData,
            vertexOffset,
            primitiveCount,
            vertexDeclaration,
        )
    }

    pub fn DrawUserIndexedPrimitives<T: VertexData>(
        &self,
        primitiveType: PrimitiveType,
        vertexData: &[T],
        vertexOffset: i32,
        numVertices: i32,
        indexData: &[i32],
        indexOffset: i32,
        primitiveCount: i32,
    ) -> Result<()> {
        self.draw_user_indexed_primitives(
            primitiveType,
            vertexData,
            vertexOffset,
            numVertices,
            indexData,
            indexOffset,
            primitiveCount,
            T::vertex_declaration(),
        )
    }

    pub fn DrawUserIndexedPrimitivesWithPrimitiveTypeAndVertexDataAndVertexOffsetAndNumVerticesAndIndexDataAndIndexOffsetAndPrimitiveCount<
        T: VertexData,
    >(
        &self,
        primitiveType: PrimitiveType,
        vertexData: &[T],
        vertexOffset: i32,
        numVertices: i32,
        indexData: &[i16],
        indexOffset: i32,
        primitiveCount: i32,
    ) -> Result<()> {
        self.draw_user_indexed_primitives(
            primitiveType,
            vertexData,
            vertexOffset,
            numVertices,
            indexData,
            indexOffset,
            primitiveCount,
            T::vertex_declaration(),
        )
    }

    pub fn DrawUserIndexedPrimitivesWithPrimitiveTypeAndVertexDataAndVertexOffsetAndNumVerticesAndIndexDataAndIndexOffsetAndPrimitiveCountAndVertexDeclarationAsPrimitiveTypeAnd0ArrayAndInt32AndInt32AndInt32ArrayAndInt32AndInt32AndVertexDeclaration<
        T: VertexData,
    >(
        &self,
        primitiveType: PrimitiveType,
        vertexData: &[T],
        vertexOffset: i32,
        numVertices: i32,
        indexData: &[i32],
        indexOffset: i32,
        primitiveCount: i32,
        vertexDeclaration: &VertexDeclaration,
    ) -> Result<()> {
        self.draw_user_indexed_primitives(
            primitiveType,
            vertexData,
            vertexOffset,
            numVertices,
            indexData,
            indexOffset,
            primitiveCount,
            vertexDeclaration,
        )
    }

    pub fn DrawUserIndexedPrimitivesWithPrimitiveTypeAndVertexDataAndVertexOffsetAndNumVerticesAndIndexDataAndIndexOffsetAndPrimitiveCountAndVertexDeclarationAsPrimitiveTypeAnd0ArrayAndInt32AndInt32AndInt16ArrayAndInt32AndInt32AndVertexDeclaration<
        T: VertexData,
    >(
        &self,
        primitiveType: PrimitiveType,
        vertexData: &[T],
        vertexOffset: i32,
        numVertices: i32,
        indexData: &[i16],
        indexOffset: i32,
        primitiveCount: i32,
        vertexDeclaration: &VertexDeclaration,
    ) -> Result<()> {
        self.draw_user_indexed_primitives(
            primitiveType,
            vertexData,
            vertexOffset,
            numVertices,
            indexData,
            indexOffset,
            primitiveCount,
            vertexDeclaration,
        )
    }

    fn validate_indexed_draw(
        &self,
        primitive_type: PrimitiveType,
        base_vertex: i32,
        min_vertex_index: i32,
        num_vertices: i32,
        start_index: i32,
        primitive_count: i32,
    ) -> Result<()> {
        let index_elements = primitive_element_count(primitive_type, primitive_count)?;
        let min_vertex = usize::try_from(min_vertex_index)
            .map_err(|_| CnaError::InvalidInput("minimum vertex index must not be negative"))?;
        let vertex_count = usize::try_from(num_vertices)
            .map_err(|_| CnaError::InvalidInput("vertex count must not be negative"))?;
        let index_start = usize::try_from(start_index)
            .map_err(|_| CnaError::InvalidInput("start index must not be negative"))?;
        if vertex_count == 0 {
            return Err(CnaError::InvalidInput(
                "indexed draws require at least one vertex",
            ));
        }
        let vertex_bindings = self
            .state
            .bound_vertex_buffers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let vertex = vertex_bindings.first().ok_or(CnaError::InvalidInput(
            "indexed draws require a bound vertex buffer",
        ))?;
        vertex.VertexBuffer().handle()?;
        let first =
            i64::from(base_vertex) + i64::from(min_vertex_index) + i64::from(vertex.VertexOffset());
        let end = first
            .checked_add(i64::from(num_vertices))
            .ok_or(CnaError::InvalidInput("indexed vertex range overflows"))?;
        if first < 0 || end > i64::from(vertex.VertexBuffer().VertexCount()) {
            return Err(CnaError::InvalidInput(
                "indexed vertex range exceeds the bound vertex buffer",
            ));
        }
        let index = self
            .state
            .bound_index_buffer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .ok_or(CnaError::InvalidInput(
                "indexed draws require a bound index buffer",
            ))?;
        index.handle()?;
        let index_end = index_start
            .checked_add(index_elements)
            .ok_or(CnaError::InvalidInput("indexed draw range overflows"))?;
        if index_end > index.IndexCount() as usize
            || min_vertex
                .checked_add(vertex_count)
                .ok_or(CnaError::InvalidInput("indexed vertex range overflows"))?
                > vertex.VertexBuffer().VertexCount() as usize
        {
            return Err(CnaError::InvalidInput(
                "indexed draw range exceeds a bound buffer",
            ));
        }
        Ok(())
    }

    fn draw_user_primitives<T: VertexData>(
        &self,
        primitive_type: PrimitiveType,
        vertex_data: &[T],
        vertex_offset: i32,
        primitive_count: i32,
        declaration: &VertexDeclaration,
    ) -> Result<()> {
        let required = primitive_element_count(primitive_type, primitive_count)?;
        let offset = usize::try_from(vertex_offset)
            .map_err(|_| CnaError::InvalidInput("vertex offset must not be negative"))?;
        if offset
            .checked_add(required)
            .ok_or(CnaError::InvalidInput("user vertex range overflows"))?
            > vertex_data.len()
        {
            return Err(CnaError::InvalidInput(
                "user vertex range exceeds the supplied slice",
            ));
        }
        validate_user_declaration::<T>(declaration)?;
        let bytes = encode_vertices(vertex_data);
        self.with_user_declaration(declaration, |native_declaration| {
            let primitives = sys::CNA_UserPrimitives {
                struct_size: size_of::<sys::CNA_UserPrimitives>() as u32,
                struct_version: 1,
                primitive_type: primitive_type as u32,
                vertex_source: sys::CNA_USER_VERTEX_SOURCE_RAW_STREAM,
                vertex_data: bytes.as_ptr().cast(),
                vertex_declaration: native_declaration,
                vertex_offset,
                num_vertices: 0,
                primitive_count,
                reserved: 0,
            };
            self.state
                .native
                .draw_user_primitives(self.state.handle()?, &primitives)
        })
    }

    fn draw_user_indexed_primitives<T: VertexData, I: UserIndexData>(
        &self,
        primitive_type: PrimitiveType,
        vertex_data: &[T],
        vertex_offset: i32,
        num_vertices: i32,
        index_data: &[I],
        index_offset: i32,
        primitive_count: i32,
        declaration: &VertexDeclaration,
    ) -> Result<()> {
        let required_indices = primitive_element_count(primitive_type, primitive_count)?;
        let vertex_offset_usize = usize::try_from(vertex_offset)
            .map_err(|_| CnaError::InvalidInput("vertex offset must not be negative"))?;
        let vertex_count = usize::try_from(num_vertices)
            .map_err(|_| CnaError::InvalidInput("vertex count must not be negative"))?;
        let index_offset_usize = usize::try_from(index_offset)
            .map_err(|_| CnaError::InvalidInput("index offset must not be negative"))?;
        if vertex_count == 0
            || vertex_offset_usize
                .checked_add(vertex_count)
                .ok_or(CnaError::InvalidInput("user vertex range overflows"))?
                > vertex_data.len()
        {
            return Err(CnaError::InvalidInput(
                "user vertex range exceeds the supplied slice",
            ));
        }
        let index_end = index_offset_usize
            .checked_add(required_indices)
            .ok_or(CnaError::InvalidInput("user index range overflows"))?;
        let indices =
            index_data
                .get(index_offset_usize..index_end)
                .ok_or(CnaError::InvalidInput(
                    "user index range exceeds the supplied slice",
                ))?;
        if indices
            .iter()
            .any(|value| value.value() < 0 || value.value() as usize >= vertex_count)
        {
            return Err(CnaError::InvalidInput(
                "user index references a vertex outside numVertices",
            ));
        }
        validate_user_declaration::<T>(declaration)?;
        let bytes = encode_vertices(vertex_data);
        self.with_user_declaration(declaration, |native_declaration| {
            let primitives = sys::CNA_UserPrimitives {
                struct_size: size_of::<sys::CNA_UserPrimitives>() as u32,
                struct_version: 1,
                primitive_type: primitive_type as u32,
                vertex_source: sys::CNA_USER_VERTEX_SOURCE_RAW_STREAM,
                vertex_data: bytes.as_ptr().cast(),
                vertex_declaration: native_declaration,
                vertex_offset,
                num_vertices,
                primitive_count,
                reserved: 0,
            };
            let indices = sys::CNA_UserIndices {
                struct_size: size_of::<sys::CNA_UserIndices>() as u32,
                struct_version: 1,
                index_element_size: I::ELEMENT_SIZE,
                index_offset,
                index_data: index_data.as_ptr().cast(),
            };
            self.state.native.draw_user_indexed_primitives(
                self.state.handle()?,
                &primitives,
                &indices,
            )
        })
    }

    fn with_user_declaration(
        &self,
        declaration: &VertexDeclaration,
        operation: impl FnOnce(sys::CNA_VertexDeclarationHandle) -> Result<()>,
    ) -> Result<()> {
        declaration.ensure_open()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        self.state.native.create_vertex_declaration(
            declaration.VertexStride(),
            &declaration.native_elements(),
            &mut handle,
        )?;
        let result = operation(handle);
        let cleanup = self.state.native.destroy_vertex_declaration(handle);
        result?;
        cleanup
    }

    pub fn GetBackBufferData<T: BackBufferData>(
        &self,
        rect: Option<Rectangle>,
        data: &mut [T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        let start = usize::try_from(startIndex)
            .map_err(|_| CnaError::InvalidInput("start index must not be negative"))?;
        let count = usize::try_from(elementCount)
            .map_err(|_| CnaError::InvalidInput("element count must not be negative"))?;
        let end = start.checked_add(count).ok_or(CnaError::InvalidInput(
            "back-buffer destination range overflows",
        ))?;
        if end > data.len() {
            return Err(CnaError::InvalidInput(
                "back-buffer destination range exceeds the supplied slice",
            ));
        }
        if let Some(source) = rect {
            if source.X < 0 || source.Y < 0 || source.Width < 0 || source.Height < 0 {
                return Err(CnaError::InvalidInput(
                    "back-buffer source rectangle must not contain negative values",
                ));
            }
            let parameters = self.PresentationParameters()?;
            let right = source
                .X
                .checked_add(source.Width)
                .ok_or(CnaError::InvalidInput(
                    "back-buffer source rectangle overflows",
                ))?;
            let bottom = source
                .Y
                .checked_add(source.Height)
                .ok_or(CnaError::InvalidInput(
                    "back-buffer source rectangle overflows",
                ))?;
            if right > parameters.BackBufferWidth() || bottom > parameters.BackBufferHeight() {
                return Err(CnaError::InvalidInput(
                    "back-buffer source rectangle exceeds the back buffer",
                ));
            }
        }
        let source_rectangle =
            rect.map_or_else(sys::CNA_Rectangle::default, |value| sys::CNA_Rectangle {
                x: value.X,
                y: value.Y,
                width: value.Width,
                height: value.Height,
            });
        let readback = sys::CNA_BackBufferReadback {
            struct_size: size_of::<sys::CNA_BackBufferReadback>() as u32,
            struct_version: 1,
            has_source_rectangle: u8::from(rect.is_some()),
            reserved: [0; 3],
            source_rectangle,
            start_index: start as u64,
            element_count: count as u64,
        };
        let mut colors = vec![sys::CNA_Color::default(); data.len()];
        self.state.native.get_backbuffer_data_window(
            self.state.handle()?,
            &readback,
            &mut colors,
        )?;
        for (destination, value) in data[start..end].iter_mut().zip(&colors[start..end]) {
            *destination = T::from_color(
                Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
                    i32::from(value.r),
                    i32::from(value.g),
                    i32::from(value.b),
                    i32::from(value.a),
                ),
            );
        }
        Ok(())
    }

    pub fn GetBackBufferDataWithData<T: BackBufferData>(&self, data: &mut [T]) -> Result<()> {
        let count = i32::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("back-buffer destination is too large"))?;
        self.GetBackBufferData(None, data, 0, count)
    }

    pub fn GetBackBufferDataWithDataAndStartIndexAndElementCount<T: BackBufferData>(
        &self,
        data: &mut [T],
        startIndex: i32,
        elementCount: i32,
    ) -> Result<()> {
        self.GetBackBufferData(None, data, startIndex, elementCount)
    }

    pub fn Reset(&self) -> Result<()> {
        if self.state.device_resetting.emit(self, EventArgs) {
            return Err(CnaError::Callback(
                "GraphicsDevice.DeviceResetting handler panicked".to_owned(),
            ));
        }
        self.state
            .native
            .reset_graphics_device(self.state.handle()?)?;
        self.after_reset()?;
        if self.state.device_reset.emit(self, EventArgs) {
            Err(CnaError::Callback(
                "GraphicsDevice.DeviceReset handler panicked".to_owned(),
            ))
        } else {
            Ok(())
        }
    }

    pub fn ResetWithPresentationParameters(
        &self,
        presentationParameters: &PresentationParameters,
    ) -> Result<()> {
        self.reset_with_parameters(presentationParameters, None)
    }

    pub fn ResetWithPresentationParametersAndGraphicsAdapter(
        &self,
        presentationParameters: &PresentationParameters,
        graphicsAdapter: &GraphicsAdapter,
    ) -> Result<()> {
        let adapter_index = graphicsAdapter.index_for(&self.state)?;
        self.reset_with_parameters(presentationParameters, Some(adapter_index))
    }

    fn reset_with_parameters(
        &self,
        parameters: &PresentationParameters,
        adapter_index: Option<u32>,
    ) -> Result<()> {
        let mut current = sys::CNA_PresentationParameters {
            struct_size: size_of::<sys::CNA_PresentationParameters>() as u32,
            struct_version: 1,
            ..sys::CNA_PresentationParameters::default()
        };
        let handle = self.state.handle()?;
        self.state
            .native
            .presentation_parameters(handle, &mut current)?;
        let native = parameters.to_native(current.headless_ext != sys::CNA_FALSE);
        if self.state.device_resetting.emit(self, EventArgs) {
            return Err(CnaError::Callback(
                "GraphicsDevice.DeviceResetting handler panicked".to_owned(),
            ));
        }
        self.state
            .native
            .reset_graphics_device_with_parameters(handle, &native, adapter_index)?;
        self.after_reset()?;
        if self.state.device_reset.emit(self, EventArgs) {
            Err(CnaError::Callback(
                "GraphicsDevice.DeviceReset handler panicked".to_owned(),
            ))
        } else {
            Ok(())
        }
    }

    fn after_reset(&self) -> Result<()> {
        *self
            .state
            .blend_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        *self
            .state
            .depth_stencil_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        *self
            .state
            .rasterizer_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        if let Some(collection) = self.state.sampler_states.get() {
            collection.clear_cache();
        }
        if let Some(collection) = self.state.vertex_sampler_states.get() {
            collection.clear_cache();
        }
        if let Some(collection) = self.state.textures.get() {
            collection.clear_cache();
        }
        if let Some(collection) = self.state.vertex_textures.get() {
            collection.clear_cache();
        }
        let _ = self.PresentationParameters()?;
        Ok(())
    }

    pub fn SetRenderTarget(
        &self,
        renderTarget: Option<&RenderTargetCube>,
        cubeMapFace: super::CubeMapFace,
    ) -> Result<()> {
        match renderTarget {
            Some(target) => {
                self.SetRenderTargets(&[RenderTargetBinding::from_render_target_and_cube_map_face(
                    target,
                    cubeMapFace,
                )?])
            }
            None => self.SetRenderTargets(&[]),
        }
    }

    pub fn SetRenderTargetWithRenderTarget(
        &self,
        renderTarget: Option<&RenderTarget2D>,
    ) -> Result<()> {
        match renderTarget {
            Some(target) => self.SetRenderTargets(&[RenderTargetBinding::new(target)?]),
            None => self.SetRenderTargets(&[]),
        }
    }

    pub fn SetRenderTargets(&self, renderTargets: &[RenderTargetBinding]) -> Result<()> {
        let mut handles = Vec::with_capacity(renderTargets.len());
        let mut native = Vec::with_capacity(renderTargets.len());
        let mut expected_shape = None;
        for binding in renderTargets {
            if !binding.device().is_same_device(self) {
                return Err(CnaError::InvalidInput(
                    "render target belongs to another graphics device",
                ));
            }
            let handle = binding.handle()?;
            if handles.contains(&handle) {
                return Err(CnaError::InvalidInput(
                    "the same render target cannot occupy multiple binding slots",
                ));
            }
            let shape = binding.dimensions_and_samples();
            if expected_shape.is_some_and(|expected| expected != shape) {
                return Err(CnaError::InvalidInput(
                    "all render targets must have compatible dimensions and multisampling",
                ));
            }
            expected_shape = Some(shape);
            handles.push(handle);
            native.push(binding.to_native()?);
        }
        self.state
            .native
            .set_render_targets(self.state.handle()?, &native)?;
        *self
            .state
            .bound_render_targets
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = renderTargets.to_vec();
        *self
            .state
            .bound_render_target_handles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = handles;
        Ok(())
    }

    pub fn GetRenderTargets(&self) -> Result<Vec<RenderTargetBinding>> {
        let handle = self.state.handle()?;
        let mut count = 0;
        self.state.native.render_target_count(handle, &mut count)?;
        let length = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("render-target binding count is too large"))?;
        let mut native = vec![sys::CNA_RenderTargetBinding::default(); length];
        if length != 0 {
            self.state
                .native
                .copy_render_targets(handle, &mut native, &mut count)?;
        }
        let cached = self
            .state
            .bound_render_targets
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if cached.len() != length
            || cached.iter().zip(&native).any(|(logical, observed)| {
                logical.handle().ok() != Some(observed.render_target)
                    || logical.CubeMapFace() as u32 != observed.cube_map_face
                    || observed.array_slice != 0
            })
        {
            return Err(CnaError::Native {
                code: sys::CNA_RESULT_INVALID_STATE,
                category: ErrorCategory::None,
                message: "native render-target bindings changed outside CNA-Rust's safe identity registry"
                    .to_owned(),
            });
        }
        Ok(cached.clone())
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

    pub(crate) fn unbind_all_buffers(&self) -> Result<()> {
        self.state.unbind_all_buffers()
    }

    pub(crate) fn has_bound_buffer_handle(
        &self,
        vertex_handles: &[sys::CNA_Handle],
        index_handles: &[sys::CNA_Handle],
    ) -> bool {
        self.state
            .bound_vertex_handles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .any(|handle| vertex_handles.contains(handle))
            || index_handles.contains(
                &*self
                    .state
                    .bound_index_handle
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            )
    }

    pub(crate) fn unbind_all_render_targets(&self) -> Result<()> {
        self.state.unbind_all_render_targets()
    }

    pub(crate) fn invalidate(&self) {
        if self.state.invalidate() {
            let _ = self.state.disposing.emit(self, EventArgs);
        }
    }

    /// Creates one of CNA's extended effects, which share the XNA `Effect`
    /// handle kind.
    pub(crate) fn create_extended_effect(&self, crt: bool) -> Result<sys::CNA_Handle> {
        let device = self.state.handle()?;
        let native = self.state.native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        let result = if crt {
            // SAFETY: the device is live and the output is a live local.
            unsafe { (native.runtime.crt_effect_create)(device, &mut handle) }
        } else {
            // SAFETY: as above.
            unsafe { (native.runtime.depth_effect_create)(device, &mut handle) }
        };
        native.check(result)?;
        Ok(handle)
    }

    pub(crate) fn create_ascii_post_process_effect(&self) -> Result<sys::CNA_Handle> {
        let device = self.state.handle()?;
        let native = self.state.native();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the device is live and the output is a live local.
        native.check(unsafe { (native.runtime.ascii_effect_create)(device, &mut handle) })?;
        Ok(handle)
    }

    /// The native table, for an extension that owns a CNA-only handle.
    pub(crate) fn extended_effect_native(&self) -> Result<Arc<crate::native::Native>> {
        self.state.handle()?;
        Ok(Arc::clone(self.state.native()))
    }

    pub(crate) fn renderer_feature_support(&self, feature: u32) -> Result<u32> {
        let handle = self.state.handle()?;
        self.state.native.renderer_feature_support(handle, feature)
    }

    pub(crate) fn renderer_limit(&self, limit: u32) -> Result<Option<u64>> {
        let handle = self.state.handle()?;
        self.state.native.renderer_limit(handle, limit)
    }

    pub(crate) fn surface_format_support(&self, format: u32) -> Result<(u32, u32)> {
        let handle = self.state.handle()?;
        self.state.native.surface_format_support(handle, format)
    }

    pub(crate) fn shader_dialect(&self) -> Result<u32> {
        let handle = self.state.handle()?;
        self.state.native.shader_dialect(handle)
    }

    pub(crate) fn capability_report(&self) -> Result<String> {
        let handle = self.state.handle()?;
        self.state.native.capability_report(handle)
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

fn primitive_element_count(primitive_type: PrimitiveType, primitive_count: i32) -> Result<usize> {
    let count = usize::try_from(primitive_count)
        .map_err(|_| CnaError::InvalidInput("primitive count must be greater than zero"))?;
    if count == 0 {
        return Err(CnaError::InvalidInput(
            "primitive count must be greater than zero",
        ));
    }
    match primitive_type {
        PrimitiveType::TriangleList => count.checked_mul(3),
        PrimitiveType::TriangleStrip => count.checked_add(2),
        PrimitiveType::LineList => count.checked_mul(2),
        PrimitiveType::LineStrip => count.checked_add(1),
    }
    .ok_or(CnaError::InvalidInput("primitive element count overflows"))
}

fn validate_user_declaration<T: VertexData>(declaration: &VertexDeclaration) -> Result<()> {
    declaration.ensure_open()?;
    if declaration.structurally_equals(T::vertex_declaration()) {
        Ok(())
    } else {
        Err(CnaError::InvalidInput(
            "vertex declaration does not match the safe VertexData encoding",
        ))
    }
}

trait UserIndexData: Copy {
    const ELEMENT_SIZE: sys::CNA_IndexElementSize;
    fn value(self) -> i64;
}

impl UserIndexData for i16 {
    const ELEMENT_SIZE: sys::CNA_IndexElementSize = sys::CNA_INDEX_ELEMENT_SIZE_SIXTEEN_BITS;
    fn value(self) -> i64 {
        i64::from(self)
    }
}

impl UserIndexData for i32 {
    const ELEMENT_SIZE: sys::CNA_IndexElementSize = sys::CNA_INDEX_ELEMENT_SIZE_THIRTY_TWO_BITS;
    fn value(self) -> i64 {
        i64::from(self)
    }
}

impl Drop for GraphicsDevice {
    fn drop(&mut self) {
        // The native handle is parent-owned. Dropping an alias releases only
        // this Rust reference; the host performs deterministic invalidation.
    }
}
