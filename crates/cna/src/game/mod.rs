//! Game lifecycle contracts, time values, callback context, and native host adapter.

mod components;
mod context;
mod host;
mod lifecycle;
mod misc;
mod services;
mod state;
mod time;
mod window;

pub use components::{
    DrawableGameComponent, GameComponent, GameComponentBase, GameComponentCollection,
    GameComponentCollectionEventArgs, IDrawable, IGameComponent, IUpdateable,
};
pub use components::{GameComponentCollectionExt, GameComponentRuntime};
pub use context::GameContext;
pub use host::{run, run_for_frames};
pub use lifecycle::Game;
pub use misc::{FrameworkDispatcher, TitleContainer};
pub use services::{GameServiceContainer, LaunchParameters, LaunchParametersExt, ServiceProvider};
pub use state::{GameState, GameStateAccess};
pub use time::{GameTime, TimeSpan};
pub use window::{DisplayOrientation, GameWindow};

#[cfg(test)]
mod tests;
