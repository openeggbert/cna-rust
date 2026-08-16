//! Microsoft.Xna.Framework-compatible Rust API over CNA's stable C ABI.

#![forbid(unsafe_code)]

mod error;
mod game;
mod value;

pub use error::{CnaError, Result};
pub use game::run;

/// XNA 4.0-compatible API hierarchy backed by CNA.
#[allow(non_snake_case)]
pub mod Microsoft {
    /// Microsoft.Xna namespace.
    pub mod Xna {
        /// Microsoft.Xna.Framework compatibility facade.
        pub mod Framework {
            pub use crate::game::{Game, GameTime};
            pub use crate::value::{Color, Vector2};

            /// Microsoft.Xna.Framework.Graphics compatibility types.
            pub mod Graphics {
                use super::super::super::value::{Color, Matrix, Vector2};
                pub struct Viewport {
                    pub X: i32,
                    pub Y: i32,
                    pub Width: i32,
                    pub Height: i32,
                }
                pub struct GraphicsDevice {
                    pub Viewport: Viewport,
                }
                impl GraphicsDevice {
                    pub fn Clear(&self, _color: Color) {}
                }
                pub struct GraphicsDeviceManager {
                    pub GraphicsDevice: GraphicsDevice,
                }
                impl GraphicsDeviceManager {
                    pub fn new(_game: &impl crate::game::Game) -> Self {
                        Self {
                            GraphicsDevice: GraphicsDevice {
                                Viewport: Viewport { X: 0, Y: 0, Width: 1280, Height: 720 }
                            }
                        }
                    }
                }
                pub struct SpriteBatch {}
                impl SpriteBatch {
                    pub fn new(_device: &GraphicsDevice) -> Self { Self {} }
                    pub fn Begin(&self) {}
                    pub fn End(&self) {}
                    pub fn Draw(&self, _texture: &Texture2D, _position: Vector2, _color: Color) {}
                    pub fn DrawRect(&self, _texture: &Texture2D, _rect: [f32; 4], _color: Color) {}
                }
                pub struct Texture2D {
                    pub Width: i32,
                    pub Height: i32,
                }
                pub struct BasicEffect {
                    pub World: Matrix,
                    pub View: Matrix,
                    pub Projection: Matrix,
                    pub TextureEnabled: bool,
                    pub Texture: Option<Texture2D>,
                }
                impl BasicEffect {
                    pub fn new(_device: &GraphicsDevice) -> Self {
                        Self {
                            World: Matrix::CreateIdentity(),
                            View: Matrix::CreateIdentity(),
                            Projection: Matrix::CreateIdentity(),
                            TextureEnabled: false,
                            Texture: None,
                        }
                    }
                    pub fn Apply(&self) {}
                }
            }

            /// Microsoft.Xna.Framework.Input compatibility types.
            pub mod Input {
                pub enum Keys { Escape }
                pub struct KeyboardState {}
                impl KeyboardState {
                    pub fn IsKeyDown(&self, _key: Keys) -> bool { false }
                }
                pub struct Keyboard {}
                impl Keyboard {
                    pub fn GetState() -> KeyboardState { KeyboardState {} }
                }
            }

            /// Microsoft.Xna.Framework.Content compatibility types.
            pub mod Content {
                pub struct ContentManager {}
                impl ContentManager {
                    pub fn Load<T>(&self, _name: &str) -> Result<T, ()> { Err(()) }
                }
            }
        }
    }
}
