#![allow(
    non_snake_case,
    clippy::cast_possible_truncation,
    clippy::missing_errors_doc
)]

use core::mem::size_of;
use std::any::Any;
use std::sync::{Arc, Mutex, Weak};

use cna_sys as sys;

use crate::error::{CnaError, Result};

use super::device::DeviceState;
use super::{GraphicsDevice, SamplerState, Texture};

/// Internal-safe behavior required by every concrete XNA texture wrapper.
///
/// This extension contract deliberately exposes no native handle. It lets a
/// texture validate its device association and perform one reviewed binding
/// operation on behalf of a `TextureCollection`.
pub trait TextureRuntime: Any + Send + Sync {
    fn bind_texture_slot(
        &self,
        device: &GraphicsDevice,
        vertex_stage: bool,
        index: u32,
    ) -> Result<()>;
}

#[derive(Clone, Copy)]
enum ShaderStage {
    Pixel,
    Vertex,
}

impl ShaderStage {
    const fn native(self) -> sys::CNA_ShaderStage {
        match self {
            Self::Pixel => sys::CNA_SHADER_STAGE_PIXEL,
            Self::Vertex => sys::CNA_SHADER_STAGE_VERTEX,
        }
    }

    const fn is_vertex(self) -> bool {
        matches!(self, Self::Vertex)
    }
}

/// Stable device-owned sampler-state collection identity.
pub struct SamplerStateCollection {
    device: Weak<DeviceState>,
    stage: ShaderStage,
    cached: Mutex<Vec<Option<Arc<SamplerState>>>>,
}

impl SamplerStateCollection {
    pub(super) fn pixel(device: &Arc<DeviceState>) -> Self {
        Self::new(device, ShaderStage::Pixel)
    }

    pub(super) fn vertex(device: &Arc<DeviceState>) -> Self {
        Self::new(device, ShaderStage::Vertex)
    }

    fn new(device: &Arc<DeviceState>, stage: ShaderStage) -> Self {
        Self {
            device: Arc::downgrade(device),
            stage,
            cached: Mutex::new(vec![None; sys::CNA_MAX_SAMPLERS as usize]),
        }
    }

    pub fn Item(&self, index: i32) -> Result<Arc<SamplerState>> {
        let (index, native_index) = checked_index(index, sys::CNA_MAX_SAMPLERS, "sampler")?;
        if let Some(value) = self
            .cached
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[index]
            .as_ref()
        {
            return Ok(Arc::clone(value));
        }
        let state = self.device()?;
        let device = GraphicsDevice {
            state: Arc::clone(&state),
        };
        let mut native = sys::CNA_SamplerState {
            struct_size: size_of::<sys::CNA_SamplerState>() as u32,
            struct_version: 1,
            ..sys::CNA_SamplerState::default()
        };
        state.native().sampler_state(
            state.handle()?,
            self.stage.native(),
            native_index,
            &mut native,
        )?;
        let value = Arc::new(SamplerState::from_native(native, &device).ok_or_else(|| {
            CnaError::Native {
                code: sys::CNA_RESULT_INTERNAL,
                message: "CNA returned invalid SamplerState identities".to_owned(),
            }
        })?);
        let mut cached = self
            .cached
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(Arc::clone(cached[index].get_or_insert(value)))
    }

    pub fn SetItem(&self, index: i32, value: Arc<SamplerState>) -> Result<()> {
        let (index, native_index) = checked_index(index, sys::CNA_MAX_SAMPLERS, "sampler")?;
        let state = self.device()?;
        let device = GraphicsDevice {
            state: Arc::clone(&state),
        };
        value.bind(&device)?;
        state.native().set_sampler_state(
            state.handle()?,
            self.stage.native(),
            native_index,
            &value.native(),
        )?;
        self.cached
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[index] = Some(value);
        Ok(())
    }

    fn device(&self) -> Result<Arc<DeviceState>> {
        let state = self
            .device
            .upgrade()
            .ok_or(CnaError::InvalidInput("graphics device is disposed"))?;
        state.ensure_alive()?;
        Ok(state)
    }
}

/// Stable device-owned texture collection with safe logical-object caching.
pub struct TextureCollection {
    device: Weak<DeviceState>,
    stage: ShaderStage,
    cached: Mutex<Vec<Option<Arc<dyn Texture>>>>,
}

impl TextureCollection {
    pub(super) fn pixel(device: &Arc<DeviceState>) -> Self {
        Self::new(device, ShaderStage::Pixel)
    }

    pub(super) fn vertex(device: &Arc<DeviceState>) -> Self {
        Self::new(device, ShaderStage::Vertex)
    }

    fn new(device: &Arc<DeviceState>, stage: ShaderStage) -> Self {
        Self {
            device: Arc::downgrade(device),
            stage,
            cached: Mutex::new(vec![
                None;
                sys::CNA_TEXTURE_COLLECTION_MAX_TEXTURES as usize
            ]),
        }
    }

    pub fn Item(&self, index: i32) -> Result<Option<Arc<dyn Texture>>> {
        let (index, native_index) =
            checked_index(index, sys::CNA_TEXTURE_COLLECTION_MAX_TEXTURES, "texture")?;
        let state = self.device()?;
        let mut info = sys::CNA_TextureSlotInfo {
            struct_size: size_of::<sys::CNA_TextureSlotInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_TextureSlotInfo::default()
        };
        state.native().texture_slot(
            state.handle()?,
            self.stage.native(),
            native_index,
            &mut info,
        )?;
        let mut cached = self
            .cached
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if info.bound == sys::CNA_FALSE {
            cached[index] = None;
            return Ok(None);
        }
        if info.texture == sys::CNA_INVALID_HANDLE {
            cached[index] = None;
            return Err(CnaError::UnsupportedRuntime(
                "CNA reports a canonically bound texture without a reversible C handle",
            ));
        }
        cached[index]
            .as_ref()
            .map(Arc::clone)
            .map(Some)
            .ok_or(CnaError::UnsupportedRuntime(
                "CNA cannot reconstruct a safe Rust texture wrapper from a native handle",
            ))
    }

    pub fn SetItem(&self, index: i32, value: Option<Arc<dyn Texture>>) -> Result<()> {
        let (index, native_index) =
            checked_index(index, sys::CNA_TEXTURE_COLLECTION_MAX_TEXTURES, "texture")?;
        let state = self.device()?;
        let device = GraphicsDevice { state };
        if let Some(texture) = &value {
            texture.bind_texture_slot(&device, self.stage.is_vertex(), native_index)?;
        } else {
            device.state.native().set_texture_slot(
                device.state.handle()?,
                self.stage.native(),
                native_index,
                sys::CNA_INVALID_HANDLE,
            )?;
        }
        self.cached
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[index] = value;
        Ok(())
    }

    fn device(&self) -> Result<Arc<DeviceState>> {
        let state = self
            .device
            .upgrade()
            .ok_or(CnaError::InvalidInput("graphics device is disposed"))?;
        state.ensure_alive()?;
        Ok(state)
    }
}

fn checked_index(value: i32, limit: u32, kind: &'static str) -> Result<(usize, u32)> {
    let index = usize::try_from(value)
        .map_err(|_| CnaError::InvalidInput("collection index must not be negative"))?;
    if index >= limit as usize {
        return Err(CnaError::InvalidInput(match kind {
            "sampler" => "sampler index is outside the collection",
            _ => "texture index is outside the collection",
        }));
    }
    let native = u32::try_from(index)
        .map_err(|_| CnaError::InvalidInput("collection index exceeds the native range"))?;
    Ok((index, native))
}
