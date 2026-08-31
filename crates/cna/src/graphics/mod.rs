//! XNA graphics projection backed by focused native resource implementations.

mod buffer;
mod device;
mod device_collections;
mod display;
mod effect;
mod kinds;
pub(crate) mod model;
mod occlusion_query;
mod presentation;
mod render_target;
pub(crate) mod resource;
mod sprite_batch;
mod sprite_font;
mod states;
mod stock_effect;
mod support;
mod texture2d;
mod texture3d;
mod texture_cube;
mod vertex;
mod viewport;

pub use buffer::{
    DynamicIndexBuffer, DynamicVertexBuffer, IndexBuffer, IndexBufferBase, IndexData, VertexBuffer,
    VertexBufferBase, VertexBufferBinding, VertexData,
};
pub use device::{BackBufferData, GraphicsDevice};
pub(crate) use device::OwnedEngineChild;
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
pub use model::{
    Model, ModelBone, ModelBoneCollection, ModelBoneCollectionEnumerator, ModelEffectCollection,
    ModelEffectCollectionEnumerator, ModelMesh, ModelMeshCollection, ModelMeshCollectionEnumerator,
    ModelMeshPart, ModelMeshPartCollection, ModelMeshPartCollectionEnumerator,
};
pub use occlusion_query::OcclusionQuery;
pub use presentation::{
    DisplayMode, PresentationParameters, ResourceCreatedEventArgs, ResourceDestroyedEventArgs,
};
pub use render_target::{RenderTarget2D, RenderTargetBinding, RenderTargetCube};
pub(crate) use effect::{from_native_matrix, native_matrix};
pub(crate) use resource::BorrowedHandle;
pub use resource::{GraphicsResource, Texture};
pub use sprite_batch::SpriteBatch;
pub use sprite_font::SpriteFont;
pub use states::{BlendState, DepthStencilState, RasterizerState, SamplerState};
pub use stock_effect::{
    AlphaTestEffect, BasicEffect, DirectionalLight, DualTextureEffect, EnvironmentMapEffect,
    IEffectFog, IEffectLights, IEffectMatrices, SkinnedEffect,
};
pub use support::{
    DeviceLostException, DeviceNotResetException, IGraphicsDeviceService,
    NoSuitableGraphicsDeviceException,
};
pub use texture2d::{Texture2D, Texture2DBase};
pub use texture3d::{Texture3D, Texture3DData};
pub use texture_cube::{CubeTextureData, TextureCube, TextureCubeBase};
pub use vertex::{
    IVertexType, VertexDeclaration, VertexElement, VertexPositionColor, VertexPositionColorTexture,
    VertexPositionNormalTexture, VertexPositionTexture,
};
pub use viewport::Viewport;
