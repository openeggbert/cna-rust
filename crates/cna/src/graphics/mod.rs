//! XNA graphics projection backed by focused native resource implementations.

mod buffer;
mod device;
mod device_collections;
mod display;
mod effect;
mod kinds;
mod presentation;
mod render_target;
pub(crate) mod resource;
mod sprite_batch;
mod sprite_font;
mod states;
mod texture2d;
mod texture_cube;
mod vertex;
mod viewport;

pub use buffer::{
    DynamicIndexBuffer, DynamicVertexBuffer, IndexBuffer, IndexBufferBase, IndexData, VertexBuffer,
    VertexBufferBase, VertexBufferBinding, VertexData,
};
pub use device::{BackBufferData, GraphicsDevice};
pub use device_collections::{SamplerStateCollection, TextureCollection, TextureRuntime};
pub use display::{DisplayModeCollection, GraphicsAdapter};
pub use effect::{
    Effect, EffectAnnotation, EffectAnnotationCollection, EffectAnnotationDescriptor, EffectBase,
    EffectMaterial, EffectParameter, EffectParameterClass, EffectParameterCollection,
    EffectParameterDescriptor, EffectParameterType, EffectPass, EffectPassCollection,
    EffectTechnique, EffectTechniqueCollection, EffectTechniqueDescriptor,
};
pub use kinds::{
    Blend, BlendFunction, BufferUsage, ClearOptions, ColorWriteChannels, CompareFunction,
    CubeMapFace, CullMode, DepthFormat, FillMode, GraphicsDeviceStatus, GraphicsProfile,
    IndexElementSize, PresentInterval, PrimitiveType, RenderTargetUsage, SetDataOptions,
    SpriteEffects, SpriteSortMode, StencilOperation, SurfaceFormat, TextureAddressMode,
    TextureFilter, VertexElementFormat, VertexElementUsage,
};
pub use presentation::{
    DisplayMode, PresentationParameters, ResourceCreatedEventArgs, ResourceDestroyedEventArgs,
};
pub use render_target::{RenderTarget2D, RenderTargetBinding, RenderTargetCube};
pub use resource::{GraphicsResource, Texture};
pub use sprite_batch::SpriteBatch;
pub use sprite_font::SpriteFont;
pub use states::{BlendState, DepthStencilState, RasterizerState, SamplerState};
pub use texture2d::{Texture2D, Texture2DBase};
pub use texture_cube::{CubeTextureData, TextureCube, TextureCubeBase};
pub use vertex::{
    IVertexType, VertexDeclaration, VertexElement, VertexPositionColor, VertexPositionColorTexture,
    VertexPositionNormalTexture, VertexPositionTexture,
};
pub use viewport::Viewport;
