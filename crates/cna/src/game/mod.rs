//! Game lifecycle contracts, time values, callback context, and native host adapter.

mod components;
mod context;
mod device_manager;
mod gamer_services;
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
pub(crate) use device_manager::{manager_service_type_ids, GraphicsDeviceManagerState};
pub use device_manager::{
    GraphicsDeviceInformation, GraphicsDeviceManager, IGraphicsDeviceManager,
    PreparingDeviceSettingsEventArgs,
};
pub use gamer_services::GamerServicesComponent;
pub use host::{run, run_for_frames};
pub use lifecycle::Game;
pub use misc::{FrameworkDispatcher, TitleContainer};
pub use services::{GameServiceContainer, LaunchParameters, LaunchParametersExt, ServiceProvider};
pub use state::{GameState, GameStateAccess};
pub use time::{GameTime, TimeSpan};
pub use window::{DisplayOrientation, GameWindow};

#[cfg(test)]
mod tests;
