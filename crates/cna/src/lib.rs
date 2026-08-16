//! CNA and Microsoft.Xna.Framework-compatible Rust namespaces over CNA's
//! stable C ABI.

#![forbid(unsafe_code)]

mod error;
mod game;
mod value;

/// CNA-native API hierarchy.
#[allow(non_snake_case)]
pub mod CNA {
    /// CNA.Framework public values and lifecycle types.
    pub mod Framework {
        pub use crate::error::{CnaError, Result};
        pub use crate::game::{run, Game, GameContext, GameTime};
        pub use crate::value::{Color, Vector2};

        /// CNA.Framework.Graphics native resource wrappers.
        pub mod Graphics {}

        /// CNA.Framework.Input snapshots and enumerations.
        pub mod Input {}

        /// CNA.Framework.Content loading APIs.
        pub mod Content {}
    }

    /// Low-level ABI status. Application code should use `CNA::Framework`.
    pub mod Interop {
        /// Returns whether canonical CNA ABI declarations are available.
        #[must_use]
        pub const fn bindings_available() -> bool {
            cna_sys::BINDINGS_AVAILABLE
        }
    }
}

/// XNA 4.0-compatible API hierarchy backed by CNA.
#[allow(non_snake_case)]
pub mod Microsoft {
    /// Microsoft.Xna namespace.
    pub mod Xna {
        /// Microsoft.Xna.Framework compatibility facade.
        pub mod Framework {
            pub use crate::CNA::Framework::{
                run, CnaError, Color, Game, GameContext, GameTime, Result, Vector2,
            };

            /// Microsoft.Xna.Framework.Graphics compatibility types.
            pub mod Graphics {}

            /// Microsoft.Xna.Framework.Input compatibility types.
            pub mod Input {}

            /// Microsoft.Xna.Framework.Content compatibility types.
            pub mod Content {}
        }
    }
}
