#![allow(non_snake_case, clippy::missing_errors_doc)]

use std::any::Any;
use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::EventHandler;

use super::resource::{ResourceKind, ResourceState};
use super::{GraphicsDevice, GraphicsResource};

/// Owned native XNA occlusion query.
pub struct OcclusionQuery {
    state: Arc<ResourceState>,
}

impl OcclusionQuery {
    pub fn new(graphicsDevice: &GraphicsDevice) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        graphicsDevice
            .state
            .native()
            .create_occlusion_query(graphicsDevice.handle()?, &mut handle)?;
        Ok(Self {
            state: ResourceState::new(graphicsDevice, handle, ResourceKind::OcclusionQuery),
        })
    }

    pub fn Begin(&self) -> Result<()> {
        if self.state.is_active() {
            return Err(CnaError::InvalidInput(
                "an occlusion query cannot be begun twice",
            ));
        }
        self.state
            .device()
            .state
            .native()
            .begin_occlusion_query(self.state.require_handle()?)?;
        self.state.set_active(true);
        Ok(())
    }

    pub fn End(&self) -> Result<()> {
        if !self.state.is_active() {
            return Err(CnaError::InvalidInput(
                "an occlusion query must be begun before it can be ended",
            ));
        }
        self.state
            .device()
            .state
            .native()
            .end_occlusion_query(self.state.require_handle()?)?;
        self.state.set_active(false);
        Ok(())
    }

    pub fn IsComplete(&self) -> Result<bool> {
        if self.state.is_active() {
            return Ok(false);
        }
        self.state
            .device()
            .state
            .native()
            .occlusion_query_is_complete(self.state.require_handle()?)
    }

    pub fn PixelCount(&self) -> Result<i32> {
        if self.state.is_active() {
            return Err(CnaError::InvalidInput(
                "an active occlusion query has no completed pixel count",
            ));
        }
        self.state
            .device()
            .state
            .native()
            .occlusion_query_pixel_count(self.state.require_handle()?)
    }

    pub fn Dispose(&mut self, value: bool) -> Result<()> {
        self.state.dispose_with_event(self, value)
    }
}

impl GraphicsResource for OcclusionQuery {
    fn GraphicsDevice(&self) -> Option<&GraphicsDevice> {
        Some(self.state.device())
    }
    fn IsDisposed(&self) -> bool {
        self.state.handle().is_none()
    }
    fn Name(&self) -> String {
        self.state.name()
    }
    fn SetName(&mut self, value: &str) {
        self.state.set_name(value);
    }
    fn Tag(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.state.tag()
    }
    fn SetTag(&mut self, value: Option<Arc<dyn Any + Send + Sync>>) {
        self.state.set_tag(value);
    }
    fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.add_disposing_handler(handler)
    }
    fn RemoveDisposingHandler(&self, registration: u64) -> bool {
        self.state.remove_disposing_handler(registration)
    }
    fn Dispose(&mut self, value: bool) -> Result<()> {
        Self::Dispose(self, value)
    }
}

impl Drop for OcclusionQuery {
    fn drop(&mut self) {}
}
