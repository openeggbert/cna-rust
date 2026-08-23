//! Game lifecycle contracts, time values, callback context, and native host adapter.

mod context;
mod host;
mod lifecycle;
mod time;

pub use context::GameContext;
pub use host::{run, run_for_frames};
pub use lifecycle::Game;
pub use time::{GameTime, TimeSpan};

#[cfg(test)]
mod tests;
