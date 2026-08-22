//! Safe Rust projection of Microsoft XNA Framework 4.0 over CNA's stable C ABI.

#![deny(unsafe_op_in_unsafe_fn)]

mod error;
mod game;
mod graphics;
mod input;
mod native;
mod value;

pub use error::{CnaError, Result};
pub use game::{run, run_for_frames};

/// XNA 4.0 compatibility hierarchy. Casing intentionally follows XNA.
#[allow(non_snake_case)]
pub mod Microsoft {
    pub mod Xna {
        pub mod Framework {
            pub use crate::game::{Game, GameContext, GameTime, TimeSpan};
            pub use crate::value::{
                BoundingBox, BoundingSphere, Color, MathHelper, Matrix, Plane, Point, Quaternion,
                Ray, Rectangle, Vector2, Vector3, Vector4,
            };

            #[allow(non_snake_case, clippy::module_name_repetitions)]
            pub mod Graphics {
                pub use crate::graphics::{
                    GraphicsDevice, GraphicsResource, SpriteBatch, Texture, Texture2D, Viewport,
                };
            }

            #[allow(non_snake_case)]
            pub mod Input {
                pub use crate::input::{Keyboard, KeyboardState, Keys};
            }

            /// Reserved strict namespace for the not-yet-implemented XNB facade.
            #[allow(non_snake_case)]
            pub mod Content {}
        }
    }
}

/// CNA-specific functionality kept outside the strict XNA projection.
pub mod extensions {
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

        impl RendererInfoExt for GraphicsDevice<'_> {
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
