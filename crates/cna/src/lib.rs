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
            pub mod Graphics {}

            /// Microsoft.Xna.Framework.Input compatibility types.
            pub mod Input {}

            /// Microsoft.Xna.Framework.Content compatibility types.
            pub mod Content {}
        }
    }
}
