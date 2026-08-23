//! XNA graphics projection backed by focused native resource implementations.

mod device;
mod device_collections;
mod display;
mod kinds;
mod presentation;
pub(crate) mod resource;
mod sprite_batch;
mod states;
mod texture2d;
mod viewport;

pub use device::GraphicsDevice;
pub use device_collections::{SamplerStateCollection, TextureCollection, TextureRuntime};
pub use display::{DisplayModeCollection, GraphicsAdapter};
pub use kinds::{
    Blend, BlendFunction, ClearOptions, ColorWriteChannels, CompareFunction, CullMode, DepthFormat,
    FillMode, GraphicsDeviceStatus, GraphicsProfile, PresentInterval, RenderTargetUsage,
    SpriteEffects, SpriteSortMode, StencilOperation, SurfaceFormat, TextureAddressMode,
    TextureFilter,
};
pub use presentation::{
    DisplayMode, PresentationParameters, ResourceCreatedEventArgs, ResourceDestroyedEventArgs,
};
pub use resource::{GraphicsResource, Texture};
pub use sprite_batch::SpriteBatch;
pub use states::{BlendState, DepthStencilState, RasterizerState, SamplerState};
pub use texture2d::Texture2D;
pub use viewport::Viewport;
