// Static XNA stock-state properties are projected as associated constants.
// Their private binding cell is intentionally interior-mutable so a named
// stock descriptor can acquire its first real device association safely.
#![allow(
    non_snake_case,
    non_upper_case_globals,
    clippy::declare_interior_mutable_const
)]

use core::mem::size_of;
use std::any::Any;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::EventHandler;
use crate::value::Color;

use super::resource::EventHandlers;
use super::{
    Blend, BlendFunction, ColorWriteChannels, CompareFunction, CullMode, FillMode, GraphicsDevice,
    GraphicsResource, StencilOperation, TextureAddressMode, TextureFilter,
};

const fn native_blend_function(value: BlendFunction) -> sys::CNA_BlendFunction {
    match value {
        BlendFunction::Min => sys::CNA_BLEND_FUNCTION_MIN,
        BlendFunction::Max => sys::CNA_BLEND_FUNCTION_MAX,
        _ => value as u32,
    }
}

fn blend_from_native(value: u32) -> Option<Blend> {
    Some(match value {
        0 => Blend::One,
        1 => Blend::Zero,
        2 => Blend::SourceColor,
        3 => Blend::InverseSourceColor,
        4 => Blend::SourceAlpha,
        5 => Blend::InverseSourceAlpha,
        6 => Blend::DestinationColor,
        7 => Blend::InverseDestinationColor,
        8 => Blend::DestinationAlpha,
        9 => Blend::InverseDestinationAlpha,
        10 => Blend::BlendFactor,
        11 => Blend::InverseBlendFactor,
        12 => Blend::SourceAlphaSaturation,
        _ => return None,
    })
}

fn blend_function_from_native(value: u32) -> Option<BlendFunction> {
    Some(match value {
        0 => BlendFunction::Add,
        1 => BlendFunction::Subtract,
        2 => BlendFunction::ReverseSubtract,
        sys::CNA_BLEND_FUNCTION_MIN => BlendFunction::Min,
        sys::CNA_BLEND_FUNCTION_MAX => BlendFunction::Max,
        _ => return None,
    })
}

fn compare_from_native(value: u32) -> Option<CompareFunction> {
    Some(match value {
        0 => CompareFunction::Always,
        1 => CompareFunction::Never,
        2 => CompareFunction::Less,
        3 => CompareFunction::LessEqual,
        4 => CompareFunction::Equal,
        5 => CompareFunction::GreaterEqual,
        6 => CompareFunction::Greater,
        7 => CompareFunction::NotEqual,
        _ => return None,
    })
}

fn stencil_from_native(value: u32) -> Option<StencilOperation> {
    Some(match value {
        0 => StencilOperation::Keep,
        1 => StencilOperation::Zero,
        2 => StencilOperation::Replace,
        3 => StencilOperation::Increment,
        4 => StencilOperation::Decrement,
        5 => StencilOperation::IncrementSaturation,
        6 => StencilOperation::DecrementSaturation,
        7 => StencilOperation::Invert,
        _ => return None,
    })
}

fn texture_address_from_native(value: u32) -> Option<TextureAddressMode> {
    Some(match value {
        0 => TextureAddressMode::Wrap,
        1 => TextureAddressMode::Clamp,
        2 => TextureAddressMode::Mirror,
        _ => return None,
    })
}

fn texture_filter_from_native(value: u32) -> Option<TextureFilter> {
    Some(match value {
        0 => TextureFilter::Linear,
        1 => TextureFilter::Point,
        2 => TextureFilter::Anisotropic,
        3 => TextureFilter::LinearMipPoint,
        4 => TextureFilter::PointMipLinear,
        5 => TextureFilter::MinLinearMagPointMipLinear,
        6 => TextureFilter::MinLinearMagPointMipPoint,
        7 => TextureFilter::MinPointMagLinearMipLinear,
        8 => TextureFilter::MinPointMagLinearMipPoint,
        _ => return None,
    })
}

/// Shared managed state for XNA graphics-state resources.
///
/// XNA permits constructing these resources before a device exists and attaches
/// them on first use. `OnceLock` preserves that one-device identity without a
/// fake handle or an extended callback borrow.
struct StateResource {
    device: OnceLock<GraphicsDevice>,
    disposed: AtomicBool,
    stock: bool,
    stock_name: &'static str,
    name: Mutex<Option<String>>,
    tag: Mutex<Option<Arc<dyn Any + Send + Sync>>>,
    disposing: EventHandlers,
}

impl StateResource {
    const fn new(stock: bool, stock_name: &'static str) -> Self {
        Self {
            device: OnceLock::new(),
            disposed: AtomicBool::new(false),
            stock,
            stock_name,
            name: Mutex::new(None),
            tag: Mutex::new(None),
            disposing: EventHandlers::new(),
        }
    }

    fn bind(&self, device: &GraphicsDevice) -> Result<()> {
        if self.is_disposed() {
            return Err(CnaError::InvalidInput("graphics state is disposed"));
        }
        if let Some(current) = self.device.get() {
            return if current.is_same_device(device) {
                Ok(())
            } else {
                Err(CnaError::InvalidInput(
                    "graphics state is already bound to a different device",
                ))
            };
        }
        let _ = self.device.set(device.clone());
        Ok(())
    }

    fn assert_mutable(&self) {
        assert!(
            !self.stock && self.device.get().is_none(),
            "an XNA graphics state cannot be modified after it is bound"
        );
        assert!(!self.is_disposed(), "an XNA graphics state is disposed");
    }

    fn is_disposed(&self) -> bool {
        self.disposed.load(Ordering::Acquire)
    }

    fn dispose(&self, sender: &dyn Any, disposing: bool) {
        if self.disposed.swap(true, Ordering::AcqRel) {
            return;
        }
        if disposing {
            let _ = self
                .disposing
                .emit(sender, crate::extensions::events::EventArgs);
        }
    }

    fn dispose_without_event(&self) {
        self.disposed.store(true, Ordering::Release);
    }

    fn name(&self) -> String {
        self.name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .unwrap_or_else(|| self.stock_name.to_owned())
    }

    fn set_name(&self, value: &str) {
        *self
            .name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(value.to_owned());
    }

    fn tag(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.tag
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn set_tag(&self, value: Option<Arc<dyn Any + Send + Sync>>) {
        *self
            .tag
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
    }
}

macro_rules! graphics_state_resource {
    ($type:ty) => {
        impl GraphicsResource for $type {
            fn GraphicsDevice(&self) -> Option<&GraphicsDevice> {
                self.resource.device.get()
            }

            fn IsDisposed(&self) -> bool {
                self.resource.is_disposed()
            }

            fn Name(&self) -> String {
                self.resource.name()
            }

            fn SetName(&mut self, value: &str) {
                self.resource.set_name(value);
            }

            fn Tag(&self) -> Option<Arc<dyn Any + Send + Sync>> {
                self.resource.tag()
            }

            fn SetTag(&mut self, value: Option<Arc<dyn Any + Send + Sync>>) {
                self.resource.set_tag(value);
            }

            fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
                self.resource.disposing.add(handler)
            }

            fn RemoveDisposingHandler(&self, registration: u64) -> bool {
                self.resource.disposing.remove(registration)
            }

            fn Dispose(&mut self, disposing: bool) -> Result<()> {
                <$type>::Dispose(self, disposing);
                Ok(())
            }
        }

        impl Drop for $type {
            fn drop(&mut self) {
                self.resource.dispose_without_event();
            }
        }
    };
}

/// Complete managed XNA blend-state descriptor.
pub struct BlendState {
    resource: StateResource,
    alpha_blend_function: BlendFunction,
    alpha_destination_blend: Blend,
    alpha_source_blend: Blend,
    color_blend_function: BlendFunction,
    color_destination_blend: Blend,
    color_source_blend: Blend,
    color_write_channels: ColorWriteChannels,
    color_write_channels1: ColorWriteChannels,
    color_write_channels2: ColorWriteChannels,
    color_write_channels3: ColorWriteChannels,
    blend_factor: Color,
    multi_sample_mask: i32,
}

impl BlendState {
    const fn preset(
        stock: bool,
        name: &'static str,
        color_source: Blend,
        alpha_source: Blend,
        color_destination: Blend,
        alpha_destination: Blend,
    ) -> Self {
        Self {
            resource: StateResource::new(stock, name),
            alpha_blend_function: BlendFunction::Add,
            alpha_destination_blend: alpha_destination,
            alpha_source_blend: alpha_source,
            color_blend_function: BlendFunction::Add,
            color_destination_blend: color_destination,
            color_source_blend: color_source,
            color_write_channels: ColorWriteChannels::All,
            color_write_channels1: ColorWriteChannels::All,
            color_write_channels2: ColorWriteChannels::All,
            color_write_channels3: ColorWriteChannels::All,
            blend_factor: Color::White,
            multi_sample_mask: -1,
        }
    }

    pub const Opaque: Self = Self::preset(
        true,
        "BlendState.Opaque",
        Blend::One,
        Blend::One,
        Blend::Zero,
        Blend::Zero,
    );
    pub const AlphaBlend: Self = Self::preset(
        true,
        "BlendState.AlphaBlend",
        Blend::One,
        Blend::One,
        Blend::InverseSourceAlpha,
        Blend::InverseSourceAlpha,
    );
    pub const Additive: Self = Self::preset(
        true,
        "BlendState.Additive",
        Blend::SourceAlpha,
        Blend::SourceAlpha,
        Blend::One,
        Blend::One,
    );
    pub const NonPremultiplied: Self = Self::preset(
        true,
        "BlendState.NonPremultiplied",
        Blend::SourceAlpha,
        Blend::SourceAlpha,
        Blend::InverseSourceAlpha,
        Blend::InverseSourceAlpha,
    );

    #[must_use]
    pub const fn new() -> Self {
        Self::preset(false, "", Blend::One, Blend::One, Blend::Zero, Blend::Zero)
    }

    pub(super) fn from_native(value: sys::CNA_BlendState, device: &GraphicsDevice) -> Option<Self> {
        let result = Self {
            resource: StateResource::new(false, ""),
            alpha_blend_function: blend_function_from_native(value.alpha_blend_function)?,
            alpha_destination_blend: blend_from_native(value.alpha_destination_blend)?,
            alpha_source_blend: blend_from_native(value.alpha_source_blend)?,
            color_blend_function: blend_function_from_native(value.color_blend_function)?,
            color_destination_blend: blend_from_native(value.color_destination_blend)?,
            color_source_blend: blend_from_native(value.color_source_blend)?,
            color_write_channels: ColorWriteChannels::from_bits(value.color_write_channels),
            color_write_channels1: ColorWriteChannels::from_bits(value.color_write_channels1),
            color_write_channels2: ColorWriteChannels::from_bits(value.color_write_channels2),
            color_write_channels3: ColorWriteChannels::from_bits(value.color_write_channels3),
            blend_factor: Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
                i32::from(value.blend_factor.r),
                i32::from(value.blend_factor.g),
                i32::from(value.blend_factor.b),
                i32::from(value.blend_factor.a),
            ),
            multi_sample_mask: value.multi_sample_mask,
        };
        result.resource.bind(device).ok()?;
        Some(result)
    }

    pub fn AlphaBlendFunction(&self) -> BlendFunction {
        self.alpha_blend_function
    }
    pub fn SetAlphaBlendFunction(&mut self, value: BlendFunction) {
        self.resource.assert_mutable();
        self.alpha_blend_function = value;
    }
    pub fn AlphaDestinationBlend(&self) -> Blend {
        self.alpha_destination_blend
    }
    pub fn SetAlphaDestinationBlend(&mut self, value: Blend) {
        self.resource.assert_mutable();
        self.alpha_destination_blend = value;
    }
    pub fn AlphaSourceBlend(&self) -> Blend {
        self.alpha_source_blend
    }
    pub fn SetAlphaSourceBlend(&mut self, value: Blend) {
        self.resource.assert_mutable();
        self.alpha_source_blend = value;
    }
    pub fn ColorBlendFunction(&self) -> BlendFunction {
        self.color_blend_function
    }
    pub fn SetColorBlendFunction(&mut self, value: BlendFunction) {
        self.resource.assert_mutable();
        self.color_blend_function = value;
    }
    pub fn ColorDestinationBlend(&self) -> Blend {
        self.color_destination_blend
    }
    pub fn SetColorDestinationBlend(&mut self, value: Blend) {
        self.resource.assert_mutable();
        self.color_destination_blend = value;
    }
    pub fn ColorSourceBlend(&self) -> Blend {
        self.color_source_blend
    }
    pub fn SetColorSourceBlend(&mut self, value: Blend) {
        self.resource.assert_mutable();
        self.color_source_blend = value;
    }
    pub fn ColorWriteChannels(&self) -> ColorWriteChannels {
        self.color_write_channels
    }
    pub fn SetColorWriteChannels(&mut self, value: ColorWriteChannels) {
        self.resource.assert_mutable();
        self.color_write_channels = value;
    }
    pub fn ColorWriteChannels1(&self) -> ColorWriteChannels {
        self.color_write_channels1
    }
    pub fn SetColorWriteChannels1(&mut self, value: ColorWriteChannels) {
        self.resource.assert_mutable();
        self.color_write_channels1 = value;
    }
    pub fn ColorWriteChannels2(&self) -> ColorWriteChannels {
        self.color_write_channels2
    }
    pub fn SetColorWriteChannels2(&mut self, value: ColorWriteChannels) {
        self.resource.assert_mutable();
        self.color_write_channels2 = value;
    }
    pub fn ColorWriteChannels3(&self) -> ColorWriteChannels {
        self.color_write_channels3
    }
    pub fn SetColorWriteChannels3(&mut self, value: ColorWriteChannels) {
        self.resource.assert_mutable();
        self.color_write_channels3 = value;
    }
    pub fn BlendFactor(&self) -> Color {
        self.blend_factor
    }
    pub fn SetBlendFactor(&mut self, value: Color) {
        self.resource.assert_mutable();
        self.blend_factor = value;
    }
    pub fn MultiSampleMask(&self) -> i32 {
        self.multi_sample_mask
    }
    pub fn SetMultiSampleMask(&mut self, value: i32) {
        self.resource.assert_mutable();
        self.multi_sample_mask = value;
    }
    pub fn Dispose(&mut self, value: bool) {
        self.resource.dispose(self, value);
    }

    pub(super) fn bind(&self, device: &GraphicsDevice) -> Result<()> {
        self.resource.bind(device)
    }
    pub(super) fn native(&self) -> sys::CNA_BlendState {
        sys::CNA_BlendState {
            struct_size: u32::try_from(size_of::<sys::CNA_BlendState>())
                .expect("CNA blend-state layout fits u32"),
            struct_version: 1,
            alpha_blend_function: native_blend_function(self.alpha_blend_function),
            alpha_destination_blend: self.alpha_destination_blend as u32,
            alpha_source_blend: self.alpha_source_blend as u32,
            color_blend_function: native_blend_function(self.color_blend_function),
            color_destination_blend: self.color_destination_blend as u32,
            color_source_blend: self.color_source_blend as u32,
            color_write_channels: self.color_write_channels.bits(),
            color_write_channels1: self.color_write_channels1.bits(),
            color_write_channels2: self.color_write_channels2.bits(),
            color_write_channels3: self.color_write_channels3.bits(),
            blend_factor: sys::CNA_Color {
                r: self.blend_factor.R(),
                g: self.blend_factor.G(),
                b: self.blend_factor.B(),
                a: self.blend_factor.A(),
            },
            multi_sample_mask: self.multi_sample_mask,
        }
    }
}

impl Default for BlendState {
    fn default() -> Self {
        Self::new()
    }
}
graphics_state_resource!(BlendState);

/// Complete managed XNA depth/stencil-state descriptor.
pub struct DepthStencilState {
    resource: StateResource,
    depth_buffer_enable: bool,
    depth_buffer_write_enable: bool,
    depth_buffer_function: CompareFunction,
    stencil_enable: bool,
    stencil_function: CompareFunction,
    stencil_mask: i32,
    stencil_write_mask: i32,
    reference_stencil: i32,
    stencil_fail: StencilOperation,
    stencil_depth_buffer_fail: StencilOperation,
    stencil_pass: StencilOperation,
    two_sided_stencil_mode: bool,
    counter_clockwise_stencil_function: CompareFunction,
    counter_clockwise_stencil_fail: StencilOperation,
    counter_clockwise_stencil_depth_buffer_fail: StencilOperation,
    counter_clockwise_stencil_pass: StencilOperation,
}

impl DepthStencilState {
    const fn preset(
        stock: bool,
        name: &'static str,
        depth_enable: bool,
        depth_write: bool,
    ) -> Self {
        Self {
            resource: StateResource::new(stock, name),
            depth_buffer_enable: depth_enable,
            depth_buffer_write_enable: depth_write,
            depth_buffer_function: CompareFunction::LessEqual,
            stencil_enable: false,
            stencil_function: CompareFunction::Always,
            stencil_mask: i32::MAX,
            stencil_write_mask: i32::MAX,
            reference_stencil: 0,
            stencil_fail: StencilOperation::Keep,
            stencil_depth_buffer_fail: StencilOperation::Keep,
            stencil_pass: StencilOperation::Keep,
            two_sided_stencil_mode: false,
            counter_clockwise_stencil_function: CompareFunction::Always,
            counter_clockwise_stencil_fail: StencilOperation::Keep,
            counter_clockwise_stencil_depth_buffer_fail: StencilOperation::Keep,
            counter_clockwise_stencil_pass: StencilOperation::Keep,
        }
    }

    pub const None: Self = Self::preset(true, "DepthStencilState.None", false, false);
    pub const Default: Self = Self::preset(true, "DepthStencilState.Default", true, true);
    pub const DepthRead: Self = Self::preset(true, "DepthStencilState.DepthRead", true, false);
    #[must_use]
    pub const fn new() -> Self {
        Self::preset(false, "", true, true)
    }

    pub(super) fn from_native(
        value: sys::CNA_DepthStencilState,
        device: &GraphicsDevice,
    ) -> Option<Self> {
        let result = Self {
            resource: StateResource::new(false, ""),
            depth_buffer_enable: value.depth_buffer_enable != sys::CNA_FALSE,
            depth_buffer_write_enable: value.depth_buffer_write_enable != sys::CNA_FALSE,
            depth_buffer_function: compare_from_native(value.depth_buffer_function)?,
            stencil_enable: value.stencil_enable != sys::CNA_FALSE,
            stencil_function: compare_from_native(value.stencil_function)?,
            stencil_mask: value.stencil_mask,
            stencil_write_mask: value.stencil_write_mask,
            reference_stencil: value.reference_stencil,
            stencil_fail: stencil_from_native(value.stencil_fail)?,
            stencil_depth_buffer_fail: stencil_from_native(value.stencil_depth_buffer_fail)?,
            stencil_pass: stencil_from_native(value.stencil_pass)?,
            two_sided_stencil_mode: value.two_sided_stencil_mode != sys::CNA_FALSE,
            counter_clockwise_stencil_function: compare_from_native(
                value.counter_clockwise_stencil_function,
            )?,
            counter_clockwise_stencil_fail: stencil_from_native(
                value.counter_clockwise_stencil_fail,
            )?,
            counter_clockwise_stencil_depth_buffer_fail: stencil_from_native(
                value.counter_clockwise_stencil_depth_buffer_fail,
            )?,
            counter_clockwise_stencil_pass: stencil_from_native(
                value.counter_clockwise_stencil_pass,
            )?,
        };
        result.resource.bind(device).ok()?;
        Some(result)
    }

    pub fn DepthBufferEnable(&self) -> bool {
        self.depth_buffer_enable
    }
    pub fn SetDepthBufferEnable(&mut self, value: bool) {
        self.resource.assert_mutable();
        self.depth_buffer_enable = value;
    }
    pub fn DepthBufferWriteEnable(&self) -> bool {
        self.depth_buffer_write_enable
    }
    pub fn SetDepthBufferWriteEnable(&mut self, value: bool) {
        self.resource.assert_mutable();
        self.depth_buffer_write_enable = value;
    }
    pub fn DepthBufferFunction(&self) -> CompareFunction {
        self.depth_buffer_function
    }
    pub fn SetDepthBufferFunction(&mut self, value: CompareFunction) {
        self.resource.assert_mutable();
        self.depth_buffer_function = value;
    }
    pub fn StencilEnable(&self) -> bool {
        self.stencil_enable
    }
    pub fn SetStencilEnable(&mut self, value: bool) {
        self.resource.assert_mutable();
        self.stencil_enable = value;
    }
    pub fn StencilFunction(&self) -> CompareFunction {
        self.stencil_function
    }
    pub fn SetStencilFunction(&mut self, value: CompareFunction) {
        self.resource.assert_mutable();
        self.stencil_function = value;
    }
    pub fn StencilMask(&self) -> i32 {
        self.stencil_mask
    }
    pub fn SetStencilMask(&mut self, value: i32) {
        self.resource.assert_mutable();
        self.stencil_mask = value;
    }
    pub fn StencilWriteMask(&self) -> i32 {
        self.stencil_write_mask
    }
    pub fn SetStencilWriteMask(&mut self, value: i32) {
        self.resource.assert_mutable();
        self.stencil_write_mask = value;
    }
    pub fn ReferenceStencil(&self) -> i32 {
        self.reference_stencil
    }
    pub fn SetReferenceStencil(&mut self, value: i32) {
        self.resource.assert_mutable();
        self.reference_stencil = value;
    }
    pub fn StencilFail(&self) -> StencilOperation {
        self.stencil_fail
    }
    pub fn SetStencilFail(&mut self, value: StencilOperation) {
        self.resource.assert_mutable();
        self.stencil_fail = value;
    }
    pub fn StencilDepthBufferFail(&self) -> StencilOperation {
        self.stencil_depth_buffer_fail
    }
    pub fn SetStencilDepthBufferFail(&mut self, value: StencilOperation) {
        self.resource.assert_mutable();
        self.stencil_depth_buffer_fail = value;
    }
    pub fn StencilPass(&self) -> StencilOperation {
        self.stencil_pass
    }
    pub fn SetStencilPass(&mut self, value: StencilOperation) {
        self.resource.assert_mutable();
        self.stencil_pass = value;
    }
    pub fn TwoSidedStencilMode(&self) -> bool {
        self.two_sided_stencil_mode
    }
    pub fn SetTwoSidedStencilMode(&mut self, value: bool) {
        self.resource.assert_mutable();
        self.two_sided_stencil_mode = value;
    }
    pub fn CounterClockwiseStencilFunction(&self) -> CompareFunction {
        self.counter_clockwise_stencil_function
    }
    pub fn SetCounterClockwiseStencilFunction(&mut self, value: CompareFunction) {
        self.resource.assert_mutable();
        self.counter_clockwise_stencil_function = value;
    }
    pub fn CounterClockwiseStencilFail(&self) -> StencilOperation {
        self.counter_clockwise_stencil_fail
    }
    pub fn SetCounterClockwiseStencilFail(&mut self, value: StencilOperation) {
        self.resource.assert_mutable();
        self.counter_clockwise_stencil_fail = value;
    }
    pub fn CounterClockwiseStencilDepthBufferFail(&self) -> StencilOperation {
        self.counter_clockwise_stencil_depth_buffer_fail
    }
    pub fn SetCounterClockwiseStencilDepthBufferFail(&mut self, value: StencilOperation) {
        self.resource.assert_mutable();
        self.counter_clockwise_stencil_depth_buffer_fail = value;
    }
    pub fn CounterClockwiseStencilPass(&self) -> StencilOperation {
        self.counter_clockwise_stencil_pass
    }
    pub fn SetCounterClockwiseStencilPass(&mut self, value: StencilOperation) {
        self.resource.assert_mutable();
        self.counter_clockwise_stencil_pass = value;
    }
    pub fn Dispose(&mut self, value: bool) {
        self.resource.dispose(self, value);
    }

    pub(super) fn bind(&self, device: &GraphicsDevice) -> Result<()> {
        self.resource.bind(device)
    }
    pub(super) fn native(&self) -> sys::CNA_DepthStencilState {
        sys::CNA_DepthStencilState {
            struct_size: u32::try_from(size_of::<sys::CNA_DepthStencilState>())
                .expect("CNA depth-stencil-state layout fits u32"),
            struct_version: 1,
            depth_buffer_enable: u8::from(self.depth_buffer_enable),
            depth_buffer_write_enable: u8::from(self.depth_buffer_write_enable),
            stencil_enable: u8::from(self.stencil_enable),
            two_sided_stencil_mode: u8::from(self.two_sided_stencil_mode),
            depth_buffer_function: self.depth_buffer_function as u32,
            stencil_function: self.stencil_function as u32,
            stencil_mask: self.stencil_mask,
            stencil_write_mask: self.stencil_write_mask,
            reference_stencil: self.reference_stencil,
            stencil_fail: self.stencil_fail as u32,
            stencil_depth_buffer_fail: self.stencil_depth_buffer_fail as u32,
            stencil_pass: self.stencil_pass as u32,
            counter_clockwise_stencil_function: self.counter_clockwise_stencil_function as u32,
            counter_clockwise_stencil_fail: self.counter_clockwise_stencil_fail as u32,
            counter_clockwise_stencil_depth_buffer_fail: self
                .counter_clockwise_stencil_depth_buffer_fail
                as u32,
            counter_clockwise_stencil_pass: self.counter_clockwise_stencil_pass as u32,
            reserved: 0,
        }
    }
}

impl Default for DepthStencilState {
    fn default() -> Self {
        Self::new()
    }
}
graphics_state_resource!(DepthStencilState);

/// Complete managed XNA rasterizer-state descriptor.
pub struct RasterizerState {
    resource: StateResource,
    cull_mode: CullMode,
    fill_mode: FillMode,
    depth_bias: f32,
    slope_scale_depth_bias: f32,
    multi_sample_anti_alias: bool,
    scissor_test_enable: bool,
}

impl RasterizerState {
    const fn preset(stock: bool, name: &'static str, cull_mode: CullMode) -> Self {
        Self {
            resource: StateResource::new(stock, name),
            cull_mode,
            fill_mode: FillMode::Solid,
            depth_bias: 0.0,
            slope_scale_depth_bias: 0.0,
            multi_sample_anti_alias: true,
            scissor_test_enable: false,
        }
    }

    pub const CullNone: Self = Self::preset(true, "RasterizerState.CullNone", CullMode::None);
    pub const CullClockwise: Self = Self::preset(
        true,
        "RasterizerState.CullClockwise",
        CullMode::CullClockwiseFace,
    );
    pub const CullCounterClockwise: Self = Self::preset(
        true,
        "RasterizerState.CullCounterClockwise",
        CullMode::CullCounterClockwiseFace,
    );
    #[must_use]
    pub const fn new() -> Self {
        Self::preset(false, "", CullMode::CullCounterClockwiseFace)
    }

    pub(super) fn from_native(
        value: sys::CNA_RasterizerState,
        device: &GraphicsDevice,
    ) -> Option<Self> {
        let result = Self {
            resource: StateResource::new(false, ""),
            cull_mode: match value.cull_mode {
                0 => CullMode::None,
                1 => CullMode::CullClockwiseFace,
                2 => CullMode::CullCounterClockwiseFace,
                _ => return None,
            },
            fill_mode: match value.fill_mode {
                0 => FillMode::Solid,
                1 => FillMode::WireFrame,
                _ => return None,
            },
            depth_bias: value.depth_bias,
            slope_scale_depth_bias: value.slope_scale_depth_bias,
            multi_sample_anti_alias: value.multi_sample_anti_alias != sys::CNA_FALSE,
            scissor_test_enable: value.scissor_test_enable != sys::CNA_FALSE,
        };
        result.resource.bind(device).ok()?;
        Some(result)
    }

    pub fn CullMode(&self) -> CullMode {
        self.cull_mode
    }
    pub fn SetCullMode(&mut self, value: CullMode) {
        self.resource.assert_mutable();
        self.cull_mode = value;
    }
    pub fn FillMode(&self) -> FillMode {
        self.fill_mode
    }
    pub fn SetFillMode(&mut self, value: FillMode) {
        self.resource.assert_mutable();
        self.fill_mode = value;
    }
    pub fn DepthBias(&self) -> f32 {
        self.depth_bias
    }
    pub fn SetDepthBias(&mut self, value: f32) {
        self.resource.assert_mutable();
        self.depth_bias = value;
    }
    pub fn SlopeScaleDepthBias(&self) -> f32 {
        self.slope_scale_depth_bias
    }
    pub fn SetSlopeScaleDepthBias(&mut self, value: f32) {
        self.resource.assert_mutable();
        self.slope_scale_depth_bias = value;
    }
    pub fn MultiSampleAntiAlias(&self) -> bool {
        self.multi_sample_anti_alias
    }
    pub fn SetMultiSampleAntiAlias(&mut self, value: bool) {
        self.resource.assert_mutable();
        self.multi_sample_anti_alias = value;
    }
    pub fn ScissorTestEnable(&self) -> bool {
        self.scissor_test_enable
    }
    pub fn SetScissorTestEnable(&mut self, value: bool) {
        self.resource.assert_mutable();
        self.scissor_test_enable = value;
    }
    pub fn Dispose(&mut self, value: bool) {
        self.resource.dispose(self, value);
    }

    pub(super) fn bind(&self, device: &GraphicsDevice) -> Result<()> {
        self.resource.bind(device)
    }
    pub(super) fn native(&self) -> sys::CNA_RasterizerState {
        sys::CNA_RasterizerState {
            struct_size: u32::try_from(size_of::<sys::CNA_RasterizerState>())
                .expect("CNA rasterizer-state layout fits u32"),
            struct_version: 1,
            cull_mode: self.cull_mode as u32,
            fill_mode: self.fill_mode as u32,
            depth_bias: self.depth_bias,
            slope_scale_depth_bias: self.slope_scale_depth_bias,
            multi_sample_anti_alias: u8::from(self.multi_sample_anti_alias),
            scissor_test_enable: u8::from(self.scissor_test_enable),
            reserved: [0; 2],
        }
    }
}

impl Default for RasterizerState {
    fn default() -> Self {
        Self::new()
    }
}
graphics_state_resource!(RasterizerState);

/// Complete managed XNA sampler-state descriptor.
pub struct SamplerState {
    resource: StateResource,
    address_u: TextureAddressMode,
    address_v: TextureAddressMode,
    address_w: TextureAddressMode,
    filter: TextureFilter,
    max_anisotropy: i32,
    max_mip_level: i32,
    mip_map_level_of_detail_bias: f32,
}

impl SamplerState {
    const fn preset(
        stock: bool,
        name: &'static str,
        filter: TextureFilter,
        address: TextureAddressMode,
    ) -> Self {
        Self {
            resource: StateResource::new(stock, name),
            address_u: address,
            address_v: address,
            address_w: address,
            filter,
            max_anisotropy: 4,
            max_mip_level: 0,
            mip_map_level_of_detail_bias: 0.0,
        }
    }

    pub const PointWrap: Self = Self::preset(
        true,
        "SamplerState.PointWrap",
        TextureFilter::Point,
        TextureAddressMode::Wrap,
    );
    pub const PointClamp: Self = Self::preset(
        true,
        "SamplerState.PointClamp",
        TextureFilter::Point,
        TextureAddressMode::Clamp,
    );
    pub const LinearWrap: Self = Self::preset(
        true,
        "SamplerState.LinearWrap",
        TextureFilter::Linear,
        TextureAddressMode::Wrap,
    );
    pub const LinearClamp: Self = Self::preset(
        true,
        "SamplerState.LinearClamp",
        TextureFilter::Linear,
        TextureAddressMode::Clamp,
    );
    pub const AnisotropicWrap: Self = Self::preset(
        true,
        "SamplerState.AnisotropicWrap",
        TextureFilter::Anisotropic,
        TextureAddressMode::Wrap,
    );
    pub const AnisotropicClamp: Self = Self::preset(
        true,
        "SamplerState.AnisotropicClamp",
        TextureFilter::Anisotropic,
        TextureAddressMode::Clamp,
    );
    #[must_use]
    pub const fn new() -> Self {
        Self::preset(false, "", TextureFilter::Linear, TextureAddressMode::Wrap)
    }

    pub(super) fn from_native(
        value: sys::CNA_SamplerState,
        device: &GraphicsDevice,
    ) -> Option<Self> {
        let result = Self {
            resource: StateResource::new(false, ""),
            address_u: texture_address_from_native(value.address_u)?,
            address_v: texture_address_from_native(value.address_v)?,
            address_w: texture_address_from_native(value.address_w)?,
            filter: texture_filter_from_native(value.filter)?,
            max_anisotropy: value.max_anisotropy,
            max_mip_level: value.max_mip_level,
            mip_map_level_of_detail_bias: value.mip_map_level_of_detail_bias,
        };
        result.resource.bind(device).ok()?;
        Some(result)
    }

    pub fn Filter(&self) -> TextureFilter {
        self.filter
    }
    pub fn SetFilter(&mut self, value: TextureFilter) {
        self.resource.assert_mutable();
        self.filter = value;
    }
    pub fn AddressU(&self) -> TextureAddressMode {
        self.address_u
    }
    pub fn SetAddressU(&mut self, value: TextureAddressMode) {
        self.resource.assert_mutable();
        self.address_u = value;
    }
    pub fn AddressV(&self) -> TextureAddressMode {
        self.address_v
    }
    pub fn SetAddressV(&mut self, value: TextureAddressMode) {
        self.resource.assert_mutable();
        self.address_v = value;
    }
    pub fn AddressW(&self) -> TextureAddressMode {
        self.address_w
    }
    pub fn SetAddressW(&mut self, value: TextureAddressMode) {
        self.resource.assert_mutable();
        self.address_w = value;
    }
    pub fn MaxAnisotropy(&self) -> i32 {
        self.max_anisotropy
    }
    pub fn SetMaxAnisotropy(&mut self, value: i32) {
        self.resource.assert_mutable();
        self.max_anisotropy = value;
    }
    pub fn MaxMipLevel(&self) -> i32 {
        self.max_mip_level
    }
    pub fn SetMaxMipLevel(&mut self, value: i32) {
        self.resource.assert_mutable();
        self.max_mip_level = value;
    }
    pub fn MipMapLevelOfDetailBias(&self) -> f32 {
        self.mip_map_level_of_detail_bias
    }
    pub fn SetMipMapLevelOfDetailBias(&mut self, value: f32) {
        self.resource.assert_mutable();
        self.mip_map_level_of_detail_bias = value;
    }
    pub fn Dispose(&mut self, value: bool) {
        self.resource.dispose(self, value);
    }

    pub(super) fn bind(&self, device: &GraphicsDevice) -> Result<()> {
        self.resource.bind(device)
    }
    pub(crate) fn native(&self) -> sys::CNA_SamplerState {
        sys::CNA_SamplerState {
            struct_size: u32::try_from(size_of::<sys::CNA_SamplerState>())
                .expect("CNA sampler-state layout fits u32"),
            struct_version: 1,
            address_u: self.address_u as u32,
            address_v: self.address_v as u32,
            address_w: self.address_w as u32,
            filter: self.filter as u32,
            max_anisotropy: self.max_anisotropy,
            max_mip_level: self.max_mip_level,
            mip_map_level_of_detail_bias: self.mip_map_level_of_detail_bias,
            reserved: 0,
        }
    }
}

impl Default for SamplerState {
    fn default() -> Self {
        Self::new()
    }
}
graphics_state_resource!(SamplerState);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xna_defaults_and_stock_states_are_complete() {
        let blend = BlendState::new();
        assert_eq!(blend.ColorSourceBlend(), Blend::One);
        assert_eq!(blend.ColorDestinationBlend(), Blend::Zero);
        assert_eq!(blend.BlendFactor(), Color::White);
        assert_eq!(blend.MultiSampleMask(), -1);
        let additive = BlendState::Additive;
        let alpha_blend = BlendState::AlphaBlend;
        assert_eq!(additive.ColorSourceBlend(), Blend::SourceAlpha);
        assert_eq!(alpha_blend.AlphaSourceBlend(), Blend::One);

        let depth = DepthStencilState::new();
        assert!(depth.DepthBufferEnable());
        assert!(depth.DepthBufferWriteEnable());
        assert_eq!(depth.DepthBufferFunction(), CompareFunction::LessEqual);
        let no_depth = DepthStencilState::None;
        let depth_read = DepthStencilState::DepthRead;
        assert!(!no_depth.DepthBufferEnable());
        assert!(!depth_read.DepthBufferWriteEnable());

        assert_eq!(
            RasterizerState::new().CullMode(),
            CullMode::CullCounterClockwiseFace
        );
        let cull_none = RasterizerState::CullNone;
        assert_eq!(cull_none.CullMode(), CullMode::None);
        assert_eq!(SamplerState::new().MaxAnisotropy(), 4);
        let linear_clamp = SamplerState::LinearClamp;
        assert_eq!(linear_clamp.AddressU(), TextureAddressMode::Clamp);
        let point_wrap = SamplerState::PointWrap;
        assert_eq!(point_wrap.Filter(), TextureFilter::Point);
    }
}
