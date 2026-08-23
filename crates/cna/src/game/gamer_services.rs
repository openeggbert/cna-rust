#![allow(non_snake_case)]

use crate::extensions::events::EventHandler;

use super::{
    Game, GameComponent, GameComponentBase, GameComponentRuntime, GameTime, IGameComponent,
    IUpdateable,
};

/// XNA's lifecycle bridge for the optional GamerServices dispatcher.
///
/// The selected runtime profile contains no Gamer/Guide/network service graph.
/// The component therefore preserves ordinary `GameComponent` lifecycle and
/// ordering behavior without manufacturing unavailable services.
pub struct GamerServicesComponent {
    base: GameComponent,
}

impl GamerServicesComponent {
    #[must_use]
    pub fn new(game: &dyn Game) -> Self {
        Self {
            base: GameComponent::new(game),
        }
    }

    pub fn Initialize(&self) {
        self.base.Initialize();
    }

    pub fn Update(&self, gameTime: &GameTime) {
        self.base.Update(gameTime);
    }
}

impl IGameComponent for GamerServicesComponent {
    fn Initialize(&self) {
        Self::Initialize(self);
    }
}

impl IUpdateable for GamerServicesComponent {
    fn Enabled(&self) -> bool {
        self.base.Enabled()
    }

    fn UpdateOrder(&self) -> i32 {
        self.base.UpdateOrder()
    }

    fn AddEnabledChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.base.AddEnabledChangedHandler(handler)
    }

    fn RemoveEnabledChangedHandler(&self, registration: u64) -> bool {
        self.base.RemoveEnabledChangedHandler(registration)
    }

    fn AddUpdateOrderChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.base.AddUpdateOrderChangedHandler(handler)
    }

    fn RemoveUpdateOrderChangedHandler(&self, registration: u64) -> bool {
        self.base.RemoveUpdateOrderChangedHandler(registration)
    }

    fn Update(&self, gameTime: &GameTime) {
        Self::Update(self, gameTime);
    }
}

impl GameComponentRuntime for GamerServicesComponent {
    fn AsUpdateable(&self) -> Option<&dyn IUpdateable> {
        Some(self)
    }
}

impl GameComponentBase for GamerServicesComponent {
    fn Dispose(&mut self) {
        self.base.Dispose();
    }
}

impl Drop for GamerServicesComponent {
    fn drop(&mut self) {
        GameComponentBase::Dispose(self);
    }
}
