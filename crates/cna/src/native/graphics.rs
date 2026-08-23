//! Native graphics-device and owned-resource calls.

#![allow(clippy::similar_names)]

use cna_sys as sys;

use crate::error::{CnaError, Result};

use super::Native;

impl Native {
    pub(crate) fn borrow_graphics_device(
        &self,
        game: sys::CNA_Handle,
        device: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the caller is callback-scoped and supplies a valid output.
        self.check(unsafe { (self.game_get_graphics_device)(game, device) })
    }

    pub(crate) fn clear_graphics_device(
        &self,
        device: sys::CNA_Handle,
        rgba: [f32; 4],
    ) -> Result<()> {
        // SAFETY: GraphicsDevice guarantees its callback-scoped handle.
        self.check(unsafe {
            (self.graphics_device_clear_rgba)(device, rgba[0], rgba[1], rgba[2], rgba[3])
        })
    }

    pub(crate) fn graphics_device_status(
        &self,
        device: sys::CNA_Handle,
        status: &mut sys::CNA_GraphicsDeviceStatus,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_status)(device, status) })
    }

    pub(crate) fn graphics_profile(
        &self,
        device: sys::CNA_Handle,
        profile: &mut sys::CNA_GraphicsProfile,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_graphics_profile)(device, profile) })
    }

    pub(crate) fn presentation_parameters(
        &self,
        device: sys::CNA_Handle,
        parameters: &mut sys::CNA_PresentationParameters,
    ) -> Result<()> {
        // SAFETY: the caller supplies a complete versioned writable output.
        self.check(unsafe {
            (self.graphics_device_get_presentation_parameters)(device, parameters)
        })
    }

    pub(crate) fn display_mode(
        &self,
        device: sys::CNA_Handle,
        mode: &mut sys::CNA_DisplayMode,
    ) -> Result<()> {
        // SAFETY: the caller supplies a complete versioned writable output.
        self.check(unsafe { (self.graphics_device_get_display_mode)(device, mode) })
    }

    pub(crate) fn blend_state(
        &self,
        device: sys::CNA_Handle,
        state: &mut sys::CNA_BlendState,
    ) -> Result<()> {
        // SAFETY: the caller supplies a complete versioned writable output.
        self.check(unsafe { (self.graphics_device_get_blend_state)(device, state) })
    }

    pub(crate) fn depth_stencil_state(
        &self,
        device: sys::CNA_Handle,
        state: &mut sys::CNA_DepthStencilState,
    ) -> Result<()> {
        // SAFETY: the caller supplies a complete versioned writable output.
        self.check(unsafe { (self.graphics_device_get_depth_stencil_state)(device, state) })
    }

    pub(crate) fn rasterizer_state(
        &self,
        device: sys::CNA_Handle,
        state: &mut sys::CNA_RasterizerState,
    ) -> Result<()> {
        // SAFETY: the caller supplies a complete versioned writable output.
        self.check(unsafe { (self.graphics_device_get_rasterizer_state)(device, state) })
    }

    pub(crate) fn sampler_state(
        &self,
        device: sys::CNA_Handle,
        stage: sys::CNA_ShaderStage,
        slot: u32,
        state: &mut sys::CNA_SamplerState,
    ) -> Result<()> {
        // SAFETY: stage/slot are validated by the collection and output is live.
        self.check(unsafe { (self.graphics_device_get_sampler_state)(device, stage, slot, state) })
    }

    pub(crate) fn set_sampler_state(
        &self,
        device: sys::CNA_Handle,
        stage: sys::CNA_ShaderStage,
        slot: u32,
        state: &sys::CNA_SamplerState,
    ) -> Result<()> {
        // SAFETY: the complete descriptor is copied synchronously.
        self.check(unsafe { (self.graphics_device_set_sampler_state)(device, stage, slot, state) })
    }

    pub(crate) fn texture_slot(
        &self,
        device: sys::CNA_Handle,
        stage: sys::CNA_ShaderStage,
        slot: u32,
        info: &mut sys::CNA_TextureSlotInfo,
    ) -> Result<()> {
        // SAFETY: stage/slot are validated by the collection and output is live.
        self.check(unsafe { (self.graphics_device_get_texture)(device, stage, slot, info) })
    }

    pub(crate) fn set_texture_slot(
        &self,
        device: sys::CNA_Handle,
        stage: sys::CNA_ShaderStage,
        slot: u32,
        texture: sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the collection and concrete texture wrapper validate both handles.
        self.check(unsafe { (self.graphics_device_set_texture)(device, stage, slot, texture) })
    }

    pub(crate) fn graphics_viewport(
        &self,
        device: sys::CNA_Handle,
        viewport: &mut sys::CNA_Viewport,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_viewport)(device, viewport) })
    }

    pub(crate) fn set_graphics_viewport(
        &self,
        device: sys::CNA_Handle,
        viewport: sys::CNA_Viewport,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and the POD value is copied.
        self.check(unsafe { (self.graphics_device_set_viewport)(device, viewport) })
    }

    pub(crate) fn graphics_scissor_rectangle(
        &self,
        device: sys::CNA_Handle,
        rectangle: &mut sys::CNA_Rectangle,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_scissor_rectangle)(device, rectangle) })
    }

    pub(crate) fn set_graphics_scissor_rectangle(
        &self,
        device: sys::CNA_Handle,
        rectangle: sys::CNA_Rectangle,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and the POD value is copied.
        self.check(unsafe { (self.graphics_device_set_scissor_rectangle)(device, rectangle) })
    }

    pub(crate) fn graphics_blend_factor(
        &self,
        device: sys::CNA_Handle,
        color: &mut sys::CNA_Color,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_blend_factor)(device, color) })
    }

    pub(crate) fn set_graphics_blend_factor(
        &self,
        device: sys::CNA_Handle,
        color: sys::CNA_Color,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and the POD value is copied.
        self.check(unsafe { (self.graphics_device_set_blend_factor)(device, color) })
    }

    pub(crate) fn graphics_multi_sample_mask(
        &self,
        device: sys::CNA_Handle,
        value: &mut i32,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is valid.
        self.check(unsafe { (self.graphics_device_get_multi_sample_mask)(device, value) })
    }

    pub(crate) fn set_graphics_multi_sample_mask(
        &self,
        device: sys::CNA_Handle,
        value: i32,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and the scalar is copied.
        self.check(unsafe { (self.graphics_device_set_multi_sample_mask)(device, value) })
    }

    pub(crate) fn graphics_reference_stencil(
        &self,
        device: sys::CNA_Handle,
        value: &mut i32,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is valid.
        self.check(unsafe { (self.graphics_device_get_reference_stencil)(device, value) })
    }

    pub(crate) fn set_graphics_reference_stencil(
        &self,
        device: sys::CNA_Handle,
        value: i32,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and the scalar is copied.
        self.check(unsafe { (self.graphics_device_set_reference_stencil)(device, value) })
    }

    pub(crate) fn set_graphics_blend_state(
        &self,
        device: sys::CNA_Handle,
        state: &sys::CNA_BlendState,
    ) -> Result<()> {
        // SAFETY: CNA copies the complete versioned descriptor synchronously.
        self.check(unsafe { (self.graphics_device_set_blend_state)(device, state) })
    }

    pub(crate) fn set_graphics_depth_stencil_state(
        &self,
        device: sys::CNA_Handle,
        state: &sys::CNA_DepthStencilState,
    ) -> Result<()> {
        // SAFETY: CNA copies the complete versioned descriptor synchronously.
        self.check(unsafe { (self.graphics_device_set_depth_stencil_state)(device, state) })
    }

    pub(crate) fn set_graphics_rasterizer_state(
        &self,
        device: sys::CNA_Handle,
        state: &sys::CNA_RasterizerState,
    ) -> Result<()> {
        // SAFETY: CNA copies the complete versioned descriptor synchronously.
        self.check(unsafe { (self.graphics_device_set_rasterizer_state)(device, state) })
    }

    pub(crate) fn present_graphics_device(&self, device: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the device is callback-scoped and CNA performs presentation synchronously.
        self.check(unsafe { (self.graphics_device_present)(device) })
    }

    pub(crate) fn renderer_info(
        &self,
        device: sys::CNA_Handle,
        info: &mut sys::CNA_RendererInfo,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_renderer_info)(device, info) })
    }

    pub(crate) fn renderer_name_size(&self, device: sys::CNA_Handle, size: &mut u64) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is valid.
        self.check(unsafe { (self.graphics_device_get_renderer_name_size)(device, size) })
    }

    pub(crate) fn copy_renderer_name(
        &self,
        device: sys::CNA_Handle,
        destination: &mut [u8],
        copied: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(destination.len())
            .map_err(|_| CnaError::InvalidInput("renderer-name buffer is too large"))?;
        // SAFETY: the slice describes exactly `capacity` writable bytes and all
        // references remain live through the synchronous call.
        self.check(unsafe {
            (self.graphics_device_copy_renderer_name)(
                device,
                destination.as_mut_ptr().cast(),
                capacity,
                copied,
            )
        })
    }

    pub(crate) fn create_texture_from_encoded(
        &self,
        device: sys::CNA_Handle,
        bytes: &[u8],
        decode: Option<&sys::CNA_Texture2DDecodeInfo>,
        texture: &mut sys::CNA_Handle,
    ) -> Result<()> {
        let count = u64::try_from(bytes.len())
            .map_err(|_| CnaError::InvalidInput("encoded texture is too large"))?;
        let decode = decode.map_or(core::ptr::null(), |value| value as *const _);
        // SAFETY: the encoded slice and optional decode structure remain live;
        // the output reference is valid for the synchronous call.
        self.check(unsafe {
            (self.texture2d_create_from_encoded_memory)(
                device,
                bytes.as_ptr(),
                count,
                decode,
                texture,
            )
        })
    }

    pub(crate) fn create_texture(
        &self,
        device: sys::CNA_Handle,
        info: &sys::CNA_Texture2DCreateInfo,
        texture: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the callback-scoped device and versioned input/output remain live.
        self.check(unsafe { (self.texture2d_create)(device, info, texture) })
    }

    pub(crate) fn texture_info(
        &self,
        texture: sys::CNA_Handle,
        info: &mut sys::CNA_Texture2DInfo,
    ) -> Result<()> {
        #[cfg(feature = "native-fault-injection")]
        super::fault::check("texture-info")?;
        // SAFETY: the owned texture handle and initialized output are live.
        self.check(unsafe { (self.texture2d_get_info)(texture, info) })
    }

    pub(crate) fn set_texture_data<T: Copy>(
        &self,
        texture: sys::CNA_Handle,
        data_type: sys::CNA_TextureDataType,
        transfer: &sys::CNA_Texture2DTransfer,
        data: &[T],
    ) -> Result<()> {
        let capacity = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("texture data array is too large"))?;
        // SAFETY: the reviewed data-type identity matches T's representation;
        // both the versioned transfer and source slice remain live for the call.
        self.check(unsafe {
            (self.texture2d_set_data)(texture, data_type, transfer, data.as_ptr().cast(), capacity)
        })
    }

    pub(crate) fn get_texture_data<T: Copy>(
        &self,
        texture: sys::CNA_Handle,
        data_type: sys::CNA_TextureDataType,
        transfer: &sys::CNA_Texture2DTransfer,
        data: &mut [T],
        required: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(data.len())
            .map_err(|_| CnaError::InvalidInput("texture data array is too large"))?;
        // SAFETY: the reviewed data-type identity matches T's representation;
        // the destination is valid for capacity elements and native writes atomically.
        self.check(unsafe {
            (self.texture2d_get_data)(
                texture,
                data_type,
                transfer,
                data.as_mut_ptr().cast(),
                capacity,
                required,
            )
        })
    }

    pub(crate) fn encoded_texture_size(
        &self,
        texture: sys::CNA_Handle,
        format: sys::CNA_TextureImageFormat,
        width: u32,
        height: u32,
        size: &mut u64,
    ) -> Result<()> {
        // SAFETY: the owned texture and output remain live.
        self.check(unsafe {
            (self.texture2d_get_encoded_byte_count)(texture, format, width, height, size)
        })
    }

    pub(crate) fn copy_encoded_texture(
        &self,
        texture: sys::CNA_Handle,
        format: sys::CNA_TextureImageFormat,
        width: u32,
        height: u32,
        destination: &mut [u8],
        copied: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(destination.len())
            .map_err(|_| CnaError::InvalidInput("encoded texture is too large"))?;
        // SAFETY: the destination describes capacity writable bytes and all
        // references remain live through the synchronous call.
        self.check(unsafe {
            (self.texture2d_copy_encoded)(
                texture,
                format,
                width,
                height,
                destination.as_mut_ptr(),
                capacity,
                copied,
            )
        })
    }

    pub(crate) fn destroy_texture(&self, texture: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the caller transfers exactly-once ownership of a live handle.
        self.check(unsafe { (self.texture2d_destroy)(texture) })
    }

    pub(crate) fn create_sprite_batch(
        &self,
        device: sys::CNA_Handle,
        batch: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is valid.
        self.check(unsafe { (self.sprite_batch_create)(device, batch) })
    }

    pub(crate) fn begin_sprite_batch(
        &self,
        batch: sys::CNA_Handle,
        info: &sys::CNA_SpriteBatchBeginInfo,
    ) -> Result<()> {
        // SAFETY: the owned handle and versioned input are live.
        self.check(unsafe { (self.sprite_batch_begin)(batch, info) })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn begin_sprite_batch_with_states(
        &self,
        batch: sys::CNA_Handle,
        sort_mode: sys::CNA_SpriteSortMode,
        blend: &sys::CNA_BlendState,
        sampler: &sys::CNA_SamplerState,
        depth_stencil: &sys::CNA_DepthStencilState,
        rasterizer: &sys::CNA_RasterizerState,
    ) -> Result<()> {
        // SAFETY: all descriptors are complete version-one POD values copied
        // synchronously by CNA and the batch handle remains owned by Rust.
        self.check(unsafe {
            (self.sprite_batch_begin_with_states)(
                batch,
                sort_mode,
                blend,
                sampler,
                depth_stencil,
                rasterizer,
            )
        })
    }

    pub(crate) fn submit_sprite(
        &self,
        batch: sys::CNA_Handle,
        command: &sys::CNA_SpriteCommand,
    ) -> Result<()> {
        // SAFETY: both the owned handle and POD command are live; count is one.
        self.check(unsafe { (self.sprite_batch_submit_many)(batch, command, 1) })
    }

    pub(crate) fn end_sprite_batch(&self, batch: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the wrapper enforces an active begin/end interval.
        self.check(unsafe { (self.sprite_batch_end)(batch) })
    }

    pub(crate) fn destroy_sprite_batch(&self, batch: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the caller transfers exactly-once ownership of a live handle.
        self.check(unsafe { (self.sprite_batch_destroy)(batch) })
    }
}
