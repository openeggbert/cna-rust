#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use core::mem::size_of;
use std::any::Any;
use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::EventHandler;
use crate::value::{Color, Matrix, Rectangle, Vector2};

use super::resource::{ResourceKind, ResourceState};
use super::{
    BlendState, DepthStencilState, Effect, GraphicsDevice, GraphicsResource, RasterizerState,
    SamplerState, SpriteEffects, SpriteFont, SpriteSortMode, Texture2D,
};

/// Owned `SpriteBatch` associated with one durable graphics-device identity.
pub struct SpriteBatch {
    state: Arc<ResourceState>,
}

#[allow(non_snake_case)]
impl SpriteBatch {
    /// The live handle, for the guide renderer that draws through this batch.
    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
        self.state.require_handle()
    }

    pub fn new(graphicsDevice: &GraphicsDevice) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        graphicsDevice
            .state
            .native()
            .create_sprite_batch(graphicsDevice.handle()?, &mut handle)?;
        Ok(Self {
            state: ResourceState::new(graphicsDevice, handle, ResourceKind::SpriteBatch),
        })
    }

    pub fn Begin(&mut self) -> Result<()> {
        if self.state.is_active() {
            return Err(CnaError::InvalidInput("SpriteBatch.Begin was called twice"));
        }
        let info = sys::CNA_SpriteBatchBeginInfo {
            struct_size: size_of::<sys::CNA_SpriteBatchBeginInfo>() as u32,
            struct_version: 1,
            sort_mode: sys::CNA_SPRITE_SORT_MODE_DEFERRED,
            reserved: 0,
        };
        self.state
            .device()
            .state
            .native()
            .begin_sprite_batch(self.state.require_handle()?, &info)?;
        self.state.set_active(true);
        Ok(())
    }

    pub fn BeginWithSortModeAndBlendState(
        &mut self,
        sortMode: SpriteSortMode,
        blendState: &BlendState,
    ) -> Result<()> {
        let sampler_state = SamplerState::LinearClamp;
        let depth_stencil_state = DepthStencilState::None;
        let rasterizer_state = RasterizerState::CullCounterClockwise;
        self.begin_with_states(
            sortMode,
            blendState,
            &sampler_state,
            &depth_stencil_state,
            &rasterizer_state,
        )
    }

    pub fn BeginWithSortModeAndBlendStateAndSamplerStateAndDepthStencilStateAndRasterizerState(
        &mut self,
        sortMode: SpriteSortMode,
        blendState: &BlendState,
        samplerState: &SamplerState,
        depthStencilState: &DepthStencilState,
        rasterizerState: &RasterizerState,
    ) -> Result<()> {
        self.begin_with_states(
            sortMode,
            blendState,
            samplerState,
            depthStencilState,
            rasterizerState,
        )
    }

    fn begin_with_states(
        &mut self,
        sort_mode: SpriteSortMode,
        blend_state: &BlendState,
        sampler_state: &SamplerState,
        depth_stencil_state: &DepthStencilState,
        rasterizer_state: &RasterizerState,
    ) -> Result<()> {
        if self.state.is_active() {
            return Err(CnaError::InvalidInput("SpriteBatch.Begin was called twice"));
        }
        let device = self.state.device();
        blend_state.bind(device)?;
        sampler_state.bind(device)?;
        depth_stencil_state.bind(device)?;
        rasterizer_state.bind(device)?;
        let blend = blend_state.native();
        let sampler = sampler_state.native();
        let depth_stencil = depth_stencil_state.native();
        let rasterizer = rasterizer_state.native();
        device.state.native().begin_sprite_batch_with_states(
            self.state.require_handle()?,
            sort_mode as u32,
            &blend,
            &sampler,
            &depth_stencil,
            &rasterizer,
        )?;
        self.state.set_active(true);
        Ok(())
    }

    pub fn BeginWithSortModeAndBlendStateAndSamplerStateAndDepthStencilStateAndRasterizerStateAndEffect(
        &mut self,
        sortMode: SpriteSortMode,
        blendState: &BlendState,
        samplerState: &SamplerState,
        depthStencilState: &DepthStencilState,
        rasterizerState: &RasterizerState,
        effect: Option<&Effect>,
    ) -> Result<()> {
        self.begin_with_effect(
            sortMode,
            blendState,
            samplerState,
            depthStencilState,
            rasterizerState,
            effect,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn BeginWithSortModeAndBlendStateAndSamplerStateAndDepthStencilStateAndRasterizerStateAndEffectAndTransformMatrix(
        &mut self,
        sortMode: SpriteSortMode,
        blendState: &BlendState,
        samplerState: &SamplerState,
        depthStencilState: &DepthStencilState,
        rasterizerState: &RasterizerState,
        effect: Option<&Effect>,
        transformMatrix: Matrix,
    ) -> Result<()> {
        self.begin_with_effect(
            sortMode,
            blendState,
            samplerState,
            depthStencilState,
            rasterizerState,
            effect,
            Some(transformMatrix),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn begin_with_effect(
        &mut self,
        sort_mode: SpriteSortMode,
        blend_state: &BlendState,
        sampler_state: &SamplerState,
        depth_stencil_state: &DepthStencilState,
        rasterizer_state: &RasterizerState,
        effect: Option<&Effect>,
        transform: Option<Matrix>,
    ) -> Result<()> {
        if self.state.is_active() {
            return Err(CnaError::InvalidInput("SpriteBatch.Begin was called twice"));
        }
        let device = self.state.device();
        blend_state.bind(device)?;
        sampler_state.bind(device)?;
        depth_stencil_state.bind(device)?;
        rasterizer_state.bind(device)?;
        let effect_handle = if let Some(effect) = effect {
            if !effect.is_same_device(device) {
                return Err(CnaError::InvalidInput(
                    "SpriteBatch Effect belongs to a different graphics device",
                ));
            }
            effect.handle()?
        } else {
            sys::CNA_INVALID_HANDLE
        };
        let blend = blend_state.native();
        let sampler = sampler_state.native();
        let depth_stencil = depth_stencil_state.native();
        let rasterizer = rasterizer_state.native();
        let transform = transform.map(super::effect::native_matrix);
        device.state.native().begin_sprite_batch_with_effect(
            self.state.require_handle()?,
            sort_mode as u32,
            &blend,
            &sampler,
            &depth_stencil,
            &rasterizer,
            effect_handle,
            transform.as_ref(),
        )?;
        self.state.set_active(true);
        Ok(())
    }

    pub fn Draw(&mut self, texture: &Texture2D, position: Vector2, color: Color) -> Result<()> {
        self.submit(
            texture,
            Rectangle::new(
                position.X as i32,
                position.Y as i32,
                texture.Width(),
                texture.Height(),
            ),
            None,
            color,
            0.0,
            Vector2::Zero,
            SpriteEffects::None,
            0.0,
        )
    }

    pub fn DrawWithTextureAndDestinationRectangleAndColor(
        &mut self,
        texture: &Texture2D,
        destinationRectangle: Rectangle,
        color: Color,
    ) -> Result<()> {
        self.submit(
            texture,
            destinationRectangle,
            None,
            color,
            0.0,
            Vector2::Zero,
            SpriteEffects::None,
            0.0,
        )
    }

    pub fn DrawWithTextureAndDestinationRectangleAndSourceRectangleAndColor(
        &mut self,
        texture: &Texture2D,
        destinationRectangle: Rectangle,
        sourceRectangle: Option<Rectangle>,
        color: Color,
    ) -> Result<()> {
        self.submit(
            texture,
            destinationRectangle,
            sourceRectangle,
            color,
            0.0,
            Vector2::Zero,
            SpriteEffects::None,
            0.0,
        )
    }

    pub fn DrawWithTextureAndDestinationRectangleAndSourceRectangleAndColorAndRotationAndOriginAndEffectsAndLayerDepth(
        &mut self,
        texture: &Texture2D,
        destinationRectangle: Rectangle,
        sourceRectangle: Option<Rectangle>,
        color: Color,
        rotation: f32,
        origin: Vector2,
        effects: SpriteEffects,
        layerDepth: f32,
    ) -> Result<()> {
        self.submit(
            texture,
            destinationRectangle,
            sourceRectangle,
            color,
            rotation,
            origin,
            effects,
            layerDepth,
        )
    }

    pub fn DrawWithTextureAndPositionAndSourceRectangleAndColor(
        &mut self,
        texture: &Texture2D,
        position: Vector2,
        sourceRectangle: Option<Rectangle>,
        color: Color,
    ) -> Result<()> {
        let size = sourceRectangle.unwrap_or(texture.Bounds());
        self.submit(
            texture,
            Rectangle::new(
                position.X as i32,
                position.Y as i32,
                size.Width,
                size.Height,
            ),
            sourceRectangle,
            color,
            0.0,
            Vector2::Zero,
            SpriteEffects::None,
            0.0,
        )
    }

    pub fn DrawWithTextureAndPositionAndSourceRectangleAndColorAndRotationAndOriginAndScaleAndEffectsAndLayerDepthAsTexture2DAndVector2AndRectangleAndColorAndSingleAndVector2AndSingleAndSpriteEffectsAndSingle(
        &mut self,
        texture: &Texture2D,
        position: Vector2,
        sourceRectangle: Option<Rectangle>,
        color: Color,
        rotation: f32,
        origin: Vector2,
        scale: f32,
        effects: SpriteEffects,
        layerDepth: f32,
    ) -> Result<()> {
        self.draw_scaled(
            texture,
            position,
            sourceRectangle,
            color,
            rotation,
            origin,
            Vector2::new(scale),
            effects,
            layerDepth,
        )
    }

    pub fn DrawWithTextureAndPositionAndSourceRectangleAndColorAndRotationAndOriginAndScaleAndEffectsAndLayerDepthAsTexture2DAndVector2AndRectangleAndColorAndSingleAndVector2AndVector2AndSpriteEffectsAndSingle(
        &mut self,
        texture: &Texture2D,
        position: Vector2,
        sourceRectangle: Option<Rectangle>,
        color: Color,
        rotation: f32,
        origin: Vector2,
        scale: Vector2,
        effects: SpriteEffects,
        layerDepth: f32,
    ) -> Result<()> {
        self.draw_scaled(
            texture,
            position,
            sourceRectangle,
            color,
            rotation,
            origin,
            scale,
            effects,
            layerDepth,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_scaled(
        &mut self,
        texture: &Texture2D,
        position: Vector2,
        source_rectangle: Option<Rectangle>,
        color: Color,
        rotation: f32,
        origin: Vector2,
        scale: Vector2,
        effects: SpriteEffects,
        layer_depth: f32,
    ) -> Result<()> {
        let size = source_rectangle.unwrap_or(texture.Bounds());
        self.submit(
            texture,
            Rectangle::new(
                position.X as i32,
                position.Y as i32,
                (size.Width as f32 * scale.X) as i32,
                (size.Height as f32 * scale.Y) as i32,
            ),
            source_rectangle,
            color,
            rotation,
            origin,
            effects,
            layer_depth,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn submit(
        &mut self,
        texture: &Texture2D,
        destination_rectangle: Rectangle,
        source_rectangle: Option<Rectangle>,
        color: Color,
        rotation: f32,
        origin: Vector2,
        effects: SpriteEffects,
        layer_depth: f32,
    ) -> Result<()> {
        if !self.state.is_active() {
            return Err(CnaError::InvalidInput(
                "SpriteBatch.Draw requires an active Begin/End interval",
            ));
        }
        if !texture
            .GraphicsDevice()
            .is_some_and(|device| self.state.device().is_same_device(device))
        {
            return Err(CnaError::InvalidInput(
                "SpriteBatch and Texture2D belong to different graphics devices",
            ));
        }
        let source_rectangle = source_rectangle.unwrap_or_default();
        let command = sys::CNA_SpriteCommand {
            struct_size: size_of::<sys::CNA_SpriteCommand>() as u32,
            struct_version: 1,
            texture: texture.handle()?,
            destination: sys::CNA_Rectangle {
                x: destination_rectangle.X,
                y: destination_rectangle.Y,
                width: destination_rectangle.Width,
                height: destination_rectangle.Height,
            },
            source: sys::CNA_Rectangle {
                x: source_rectangle.X,
                y: source_rectangle.Y,
                width: source_rectangle.Width,
                height: source_rectangle.Height,
            },
            color: sys::CNA_Color {
                r: color.R(),
                g: color.G(),
                b: color.B(),
                a: color.A(),
            },
            rotation,
            origin: sys::CNA_Vector2 {
                x: origin.X,
                y: origin.Y,
            },
            effects: effects.bits(),
            layer_depth,
        };
        self.state
            .device()
            .state
            .native()
            .submit_sprite(self.state.require_handle()?, &command)
    }

    pub fn DrawString(
        &mut self,
        spriteFont: &SpriteFont,
        text: &str,
        position: Vector2,
        color: Color,
    ) -> Result<()> {
        self.submit_string(
            spriteFont,
            text,
            position,
            color,
            0.0,
            Vector2::Zero,
            Vector2::One,
            SpriteEffects::None,
            0.0,
        )
    }

    pub fn DrawStringWithSpriteFontAndTextAndPositionAndColor(
        &mut self,
        spriteFont: &SpriteFont,
        text: &str,
        position: Vector2,
        color: Color,
    ) -> Result<()> {
        self.DrawString(spriteFont, text, position, color)
    }

    pub fn DrawStringWithSpriteFontAndTextAndPositionAndColorAndRotationAndOriginAndScaleAndEffectsAndLayerDepthAsSpriteFontAndStringAndVector2AndColorAndSingleAndVector2AndSingleAndSpriteEffectsAndSingle(
        &mut self,
        spriteFont: &SpriteFont,
        text: &str,
        position: Vector2,
        color: Color,
        rotation: f32,
        origin: Vector2,
        scale: f32,
        effects: SpriteEffects,
        layerDepth: f32,
    ) -> Result<()> {
        self.submit_string(
            spriteFont,
            text,
            position,
            color,
            rotation,
            origin,
            Vector2::new(scale),
            effects,
            layerDepth,
        )
    }

    pub fn DrawStringWithSpriteFontAndTextAndPositionAndColorAndRotationAndOriginAndScaleAndEffectsAndLayerDepthAsSpriteFontAndStringAndVector2AndColorAndSingleAndVector2AndVector2AndSpriteEffectsAndSingle(
        &mut self,
        spriteFont: &SpriteFont,
        text: &str,
        position: Vector2,
        color: Color,
        rotation: f32,
        origin: Vector2,
        scale: Vector2,
        effects: SpriteEffects,
        layerDepth: f32,
    ) -> Result<()> {
        self.submit_string(
            spriteFont, text, position, color, rotation, origin, scale, effects, layerDepth,
        )
    }

    pub fn DrawStringWithSpriteFontAndTextAndPositionAndColorAndRotationAndOriginAndScaleAndEffectsAndLayerDepthAsSpriteFontAndStringBuilderAndVector2AndColorAndSingleAndVector2AndSingleAndSpriteEffectsAndSingle(
        &mut self,
        spriteFont: &SpriteFont,
        text: &str,
        position: Vector2,
        color: Color,
        rotation: f32,
        origin: Vector2,
        scale: f32,
        effects: SpriteEffects,
        layerDepth: f32,
    ) -> Result<()> {
        self.DrawStringWithSpriteFontAndTextAndPositionAndColorAndRotationAndOriginAndScaleAndEffectsAndLayerDepthAsSpriteFontAndStringAndVector2AndColorAndSingleAndVector2AndSingleAndSpriteEffectsAndSingle(
            spriteFont,
            text,
            position,
            color,
            rotation,
            origin,
            scale,
            effects,
            layerDepth,
        )
    }

    pub fn DrawStringWithSpriteFontAndTextAndPositionAndColorAndRotationAndOriginAndScaleAndEffectsAndLayerDepthAsSpriteFontAndStringBuilderAndVector2AndColorAndSingleAndVector2AndVector2AndSpriteEffectsAndSingle(
        &mut self,
        spriteFont: &SpriteFont,
        text: &str,
        position: Vector2,
        color: Color,
        rotation: f32,
        origin: Vector2,
        scale: Vector2,
        effects: SpriteEffects,
        layerDepth: f32,
    ) -> Result<()> {
        self.DrawStringWithSpriteFontAndTextAndPositionAndColorAndRotationAndOriginAndScaleAndEffectsAndLayerDepthAsSpriteFontAndStringAndVector2AndColorAndSingleAndVector2AndVector2AndSpriteEffectsAndSingle(
            spriteFont,
            text,
            position,
            color,
            rotation,
            origin,
            scale,
            effects,
            layerDepth,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn submit_string(
        &mut self,
        sprite_font: &SpriteFont,
        text: &str,
        position: Vector2,
        color: Color,
        rotation: f32,
        origin: Vector2,
        scale: Vector2,
        effects: SpriteEffects,
        layer_depth: f32,
    ) -> Result<()> {
        if !self.state.is_active() {
            return Err(CnaError::InvalidInput(
                "SpriteBatch.DrawString requires an active Begin/End interval",
            ));
        }
        if !sprite_font.is_same_device(self.state.device()) {
            return Err(CnaError::InvalidInput(
                "SpriteBatch and SpriteFont belong to different graphics devices",
            ));
        }
        if !position.X.is_finite()
            || !position.Y.is_finite()
            || !rotation.is_finite()
            || !origin.X.is_finite()
            || !origin.Y.is_finite()
            || !scale.X.is_finite()
            || !scale.Y.is_finite()
            || !layer_depth.is_finite()
        {
            return Err(CnaError::InvalidInput(
                "SpriteBatch.DrawString transform values must be finite",
            ));
        }
        let command = sys::CNA_SpriteTextCommand {
            struct_size: size_of::<sys::CNA_SpriteTextCommand>() as u32,
            struct_version: 1,
            sprite_font: sprite_font.handle()?,
            text: sys::CNA_StringView {
                data: text.as_ptr().cast(),
                byte_length: u64::try_from(text.len())
                    .map_err(|_| CnaError::InvalidInput("SpriteBatch text is too large"))?,
            },
            position: sys::CNA_Vector2 {
                x: position.X,
                y: position.Y,
            },
            color: sys::CNA_Color {
                r: color.R(),
                g: color.G(),
                b: color.B(),
                a: color.A(),
            },
            rotation,
            origin: sys::CNA_Vector2 {
                x: origin.X,
                y: origin.Y,
            },
            scale: sys::CNA_Vector2 {
                x: scale.X,
                y: scale.Y,
            },
            effects: effects.bits(),
            layer_depth,
        };
        self.state
            .device()
            .state
            .native()
            .draw_sprite_string(self.state.require_handle()?, &command)
    }

    pub fn End(&mut self) -> Result<()> {
        if !self.state.is_active() {
            return Err(CnaError::InvalidInput(
                "SpriteBatch.End requires an active interval",
            ));
        }
        let result = self
            .state
            .device()
            .state
            .native()
            .end_sprite_batch(self.state.require_handle()?);
        if result.is_ok() {
            self.state.set_active(false);
        }
        result
    }

    pub fn Dispose(&mut self, disposing: bool) -> Result<()> {
        self.state.dispose_with_event(self, disposing)
    }
}

impl GraphicsResource for SpriteBatch {
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

impl Drop for SpriteBatch {
    fn drop(&mut self) {
        let _ = self.state.dispose_native();
    }
}


impl crate::extensions::graphics_resource::HasResourceState for SpriteBatch {
    fn resource_state(&self) -> &super::resource::ResourceState {
        &self.state
    }
}
