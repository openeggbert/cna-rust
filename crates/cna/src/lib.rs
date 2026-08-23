//! Safe Rust projection of Microsoft XNA Framework 4.0 over CNA's stable C ABI.

#![deny(unsafe_op_in_unsafe_fn)]

mod error;
mod game;
mod graphics;
mod input;
mod native;
mod packed;
mod value;

pub use error::{CnaError, Result};
pub use game::{
    run, run_for_frames, GameComponentBase, GameComponentCollectionExt, GameComponentRuntime,
    GameState, GameStateAccess, LaunchParametersExt, ServiceProvider,
};
pub use graphics::TextureRuntime;

/// XNA 4.0 compatibility hierarchy. Casing intentionally follows XNA.
#[allow(non_snake_case)]
pub mod Microsoft {
    pub mod Xna {
        pub mod Framework {
            pub use crate::game::{
                DisplayOrientation, DrawableGameComponent, FrameworkDispatcher, Game,
                GameComponent, GameComponentCollection, GameComponentCollectionEventArgs,
                GameContext, GameServiceContainer, GameTime, GameWindow, IDrawable, IGameComponent,
                IUpdateable, LaunchParameters, TimeSpan, TitleContainer,
            };
            pub use crate::input::PlayerIndex;
            pub use crate::value::{
                BoundingBox, BoundingFrustum, BoundingSphere, Color, ContainmentType, Curve,
                CurveContinuity, CurveKey, CurveKeyCollection, CurveLoopType, CurveTangent,
                MathHelper, Matrix, Plane, PlaneIntersectionType, Point, Quaternion, Ray,
                Rectangle, Vector2, Vector3, Vector4,
            };

            #[allow(non_snake_case, clippy::module_name_repetitions)]
            pub mod Graphics {
                pub use crate::graphics::{
                    Blend, BlendFunction, BlendState, ClearOptions, ColorWriteChannels,
                    CompareFunction, CullMode, DepthFormat, DepthStencilState, DisplayMode,
                    DisplayModeCollection, FillMode, GraphicsAdapter, GraphicsDevice,
                    GraphicsDeviceStatus, GraphicsProfile, GraphicsResource, PresentInterval,
                    PresentationParameters, RasterizerState, RenderTargetUsage,
                    ResourceCreatedEventArgs, ResourceDestroyedEventArgs, SamplerState,
                    SamplerStateCollection, SpriteBatch, SpriteEffects, SpriteSortMode,
                    StencilOperation, SurfaceFormat, Texture, Texture2D, TextureAddressMode,
                    TextureCollection, TextureFilter, Viewport,
                };

                #[allow(non_snake_case)]
                pub mod PackedVector {
                    pub use crate::packed::{
                        Alpha8, Bgr565, Bgra4444, Bgra5551, Byte4, HalfSingle, HalfVector2,
                        HalfVector4, IPackedVector, IPackedVectorOfT, NormalizedByte2,
                        NormalizedByte4, NormalizedShort2, NormalizedShort4, Rg32, Rgba1010102,
                        Rgba64, Short2, Short4,
                    };
                }
            }

            #[allow(non_snake_case)]
            pub mod Input {
                pub use crate::input::{
                    ButtonState, Buttons, GamePad, GamePadButtons, GamePadCapabilities,
                    GamePadDPad, GamePadDeadZone, GamePadState, GamePadThumbSticks,
                    GamePadTriggers, GamePadType, KeyState, Keyboard, KeyboardState, Keys, Mouse,
                    MouseState,
                };

                #[allow(non_snake_case)]
                pub mod Touch {
                    pub use crate::input::{
                        TouchCollection, TouchCollectionEnumerator, TouchLocation,
                        TouchLocationState, TouchPanelCapabilities,
                    };
                }
            }

            /// Reserved strict namespace for the not-yet-implemented XNB facade.
            #[allow(non_snake_case)]
            pub mod Content {}
        }
    }
}

/// CNA-specific functionality kept outside the strict XNA projection.
pub mod extensions {
    pub mod events {
        use std::any::Any;

        /// Rust value used for CLR's stateless `EventArgs` payload.
        #[derive(Clone, Copy, Debug, Default)]
        pub struct EventArgs;

        /// Type-erased XNA event callback.
        pub trait EventHandler<T = EventArgs>: Send {
            fn invoke(&mut self, sender: &dyn Any, args: T);
        }

        impl<F, T> EventHandler<T> for F
        where
            F: FnMut(&dyn Any, T) + Send,
        {
            fn invoke(&mut self, sender: &dyn Any, args: T) {
                self(sender, args);
            }
        }
    }

    pub mod window {
        /// Opaque native window identity. It cannot be dereferenced or forged
        /// through CNA-Rust's safe public API.
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
        pub struct WindowHandle(pub(crate) u64);
    }

    pub mod graphics {
        use crate::error::Result;
        use crate::graphics::GraphicsDevice;

        /// Renderer facts queried from CNA rather than inferred from a name.
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct RendererInfo {
            pub name: String,
            pub supports_3d: bool,
            pub supports_depth_stencil: bool,
            pub max_texture_dimension: u32,
        }

        /// CNA renderer diagnostics for a strict XNA `GraphicsDevice`.
        pub trait RendererInfoExt {
            /// Queries CNA's active native renderer.
            ///
            /// # Errors
            ///
            /// Returns the exact error reported by CNA.
            fn renderer_info(&self) -> Result<RendererInfo>;
        }

        impl RendererInfoExt for GraphicsDevice {
            fn renderer_info(&self) -> Result<RendererInfo> {
                let (name, supports_3d, supports_depth_stencil, max_texture_dimension) =
                    self.renderer_info()?;
                Ok(RendererInfo {
                    name,
                    supports_3d,
                    supports_depth_stencil,
                    max_texture_dimension,
                })
            }
        }
    }
}
