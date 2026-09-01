#![allow(non_snake_case)]

use std::sync::Mutex;

use crate::error::{CnaError, Result};
use crate::extensions::events::EventHandler;
use crate::gamer_services::GamerServicesDispatcher;

use super::{
    Game, GameComponent, GameComponentBase, GameComponentRuntime, GameTime, IGameComponent,
    IUpdateable,
};

/// XNA's lifecycle bridge for the GamerServices dispatcher.
///
/// XNA's component is thirteen instructions of glue and every one of them
/// matters: `Initialize` hands the dispatcher the game window, subscribes
/// `InstallingTitleUpdate` to `Game.Exit`, and initialises the dispatcher from
/// the game's services; `Update` pumps the dispatcher before the base
/// component's own update. A game that adds this component and nothing else is
/// how `BeginShowMessageBox`, `BeginGetAchievements` and every other
/// asynchronous gamer-services call ever completes.
///
/// This projection did none of it. The component was an ordinary
/// `GameComponent` with the right name, so a game that added it got correct
/// enabled/update-order behaviour and a dispatcher that was never pumped. The
/// three calls are now here, in XNA's order.
///
/// # Where a refusal goes
///
/// XNA's `Initialize` and `Update` are `void` and throw. Both are `void` here
/// too, and Rust has nothing to throw, so the first refusal is remembered and
/// [`crate::extensions::gamer_services::TakeComponentError`] reports it. That
/// follows `GraphicsDeviceManager`, which records a callback refusal and
/// surfaces it at the next call that can carry one -- with the difference that
/// this component has no fallible XNA member of its own, so the seam is a CNA
/// extension rather than a strict one.
pub struct GamerServicesComponent {
    base: GameComponent,
    /// The `InstallingTitleUpdate` registration, released with the component.
    installing_title_update: Mutex<Option<u64>>,
}

impl GamerServicesComponent {
    #[must_use]
    pub fn new(game: &dyn Game) -> Self {
        Self {
            base: GameComponent::new(game),
            installing_title_update: Mutex::new(None),
        }
    }

    /// XNA `GamerServicesComponent.Initialize`.
    ///
    /// In XNA's order: the window handle, the title-update subscription, the
    /// dispatcher, then the base component.
    pub fn Initialize(&self) {
        if let Err(error) = self.initialize() {
            crate::extensions::gamer_services::record_component_error(error);
        }
        self.base.Initialize();
    }

    fn initialize(&self) -> Result<()> {
        let game = self
            .base
            .Game()
            .ok_or(CnaError::InvalidInput("the component's game is gone"))?;
        GamerServicesDispatcher::SetWindowHandle(game.Window().Handle())?;
        // XNA's handler is `Game.Exit()`: a title update is about to install,
        // so the game stops. The subscription is the component's, and is
        // released when the component is.
        let exiting = game.clone();
        let registration = GamerServicesDispatcher::AddInstallingTitleUpdateHandler(Box::new(
            move |_: &dyn std::any::Any, _| {
                let _ = exiting.exit();
            },
        ));
        *self
            .installing_title_update
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(registration);
        GamerServicesDispatcher::initialize_with_game(game.active()?.handle)
    }

    /// XNA `GamerServicesComponent.Update`.
    ///
    /// The dispatcher first, then the base component, which is the order XNA
    /// uses and the order that decides whether an asynchronous result is
    /// visible to a handler running this frame or the next one.
    pub fn Update(&self, gameTime: &GameTime) {
        if let Err(error) = GamerServicesDispatcher::Update() {
            crate::extensions::gamer_services::record_component_error(error);
        }
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
        if let Some(registration) = self
            .installing_title_update
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            let _ = GamerServicesDispatcher::RemoveInstallingTitleUpdateHandler(registration);
        }
        self.base.Dispose();
    }
}

impl Drop for GamerServicesComponent {
    fn drop(&mut self) {
        GameComponentBase::Dispose(self);
    }
}
