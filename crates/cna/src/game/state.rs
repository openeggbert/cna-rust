#![allow(
    non_snake_case,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use cna_sys as sys;

use crate::content::ContentManager;
use crate::error::{CnaError, Result};
use crate::extensions::events::{EventArgs, EventHandler};
use crate::graphics::resource::EventHandlers;
use crate::graphics::GraphicsDevice;
use crate::native::Native;

use super::{
    GameComponentCollection, GameServiceContainer, GameWindow, LaunchParameters, TimeSpan,
};

#[derive(Clone)]
pub(crate) struct ActiveGame {
    pub(crate) native: Arc<Native>,
    pub(crate) handle: sys::CNA_Handle,
}

/// Durable managed state composed into every user-defined XNA `Game`.
pub struct GameState {
    launch_parameters: LaunchParameters,
    components: GameComponentCollection,
    services: Arc<GameServiceContainer>,
    content: Mutex<Arc<ContentManager>>,
    window: GameWindow,
    is_active: AtomicBool,
    is_fixed_time_step: AtomicBool,
    is_mouse_visible: AtomicBool,
    target_elapsed_time_ticks: AtomicI64,
    inactive_sleep_time_ticks: AtomicI64,
    binding: Mutex<Option<ActiveGame>>,
    graphics_device: OnceLock<GraphicsDevice>,
    activated: EventHandlers<EventArgs>,
    deactivated: EventHandlers<EventArgs>,
    exiting: EventHandlers<EventArgs>,
    disposed: EventHandlers<EventArgs>,
    disposed_once: AtomicBool,
}

impl GameState {
    #[must_use]
    pub fn new() -> Self {
        let services = Arc::new(GameServiceContainer::new());
        let service_provider: Arc<dyn super::ServiceProvider> = Arc::clone(&services) as Arc<_>;
        Self {
            launch_parameters: LaunchParameters::new(),
            components: GameComponentCollection::new(),
            services,
            content: Mutex::new(Arc::new(ContentManager::new(service_provider))),
            window: GameWindow::new("CNA Rust"),
            is_active: AtomicBool::new(true),
            is_fixed_time_step: AtomicBool::new(true),
            is_mouse_visible: AtomicBool::new(false),
            target_elapsed_time_ticks: AtomicI64::new(166_667),
            inactive_sleep_time_ticks: AtomicI64::new(200_000),
            binding: Mutex::new(None),
            graphics_device: OnceLock::new(),
            activated: EventHandlers::new(),
            deactivated: EventHandlers::new(),
            exiting: EventHandlers::new(),
            disposed: EventHandlers::new(),
            disposed_once: AtomicBool::new(false),
        }
    }

    pub(crate) fn attach(
        &self,
        native: &Arc<Native>,
        handle: sys::CNA_Handle,
        device: &GraphicsDevice,
    ) -> Result<()> {
        if let Some(existing) = self.graphics_device.get() {
            if !existing.is_same_device(device) {
                return Err(CnaError::InvalidInput(
                    "an XNA Game cannot be attached to multiple graphics devices",
                ));
            }
        } else {
            self.graphics_device.set(device.clone()).map_err(|_| {
                CnaError::InvalidInput("graphics device identity was already initialized")
            })?;
        }
        self.Content().bind_graphics_device(device)?;
        let is_fixed_time_step = self.is_fixed_time_step.load(Ordering::Acquire);
        let is_mouse_visible = self.is_mouse_visible.load(Ordering::Acquire);
        let target_elapsed_time_ticks = self.target_elapsed_time_ticks.load(Ordering::Acquire);
        let inactive_sleep_time_ticks = self.inactive_sleep_time_ticks.load(Ordering::Acquire);
        self.window.attach(native, handle)?;
        *self
            .binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(ActiveGame {
            native: Arc::clone(native),
            handle,
        });
        let synchronized = (|| {
            native.set_game_is_fixed_time_step(handle, is_fixed_time_step)?;
            native.set_game_is_mouse_visible(handle, is_mouse_visible)?;
            native.set_game_target_elapsed_time_ticks(handle, target_elapsed_time_ticks)?;
            native.set_game_inactive_sleep_time_ticks(handle, inactive_sleep_time_ticks)
        })();
        if let Err(error) = synchronized {
            self.detach();
            return Err(error);
        }
        Ok(())
    }

    pub(crate) fn detach(&self) {
        self.window.detach();
        self.binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }

    pub(crate) fn active(&self) -> Result<ActiveGame> {
        self.binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .ok_or(CnaError::InvalidInput("game is not running"))
    }

    pub(crate) fn create_configuration(&self) -> (bool, i64, String) {
        (
            self.is_fixed_time_step.load(Ordering::Acquire),
            self.target_elapsed_time_ticks.load(Ordering::Acquire),
            self.window.Title(),
        )
    }

    #[must_use]
    pub fn LaunchParameters(&self) -> &LaunchParameters {
        &self.launch_parameters
    }

    #[must_use]
    pub fn Components(&self) -> &GameComponentCollection {
        &self.components
    }

    #[must_use]
    pub fn Services(&self) -> &GameServiceContainer {
        &self.services
    }

    #[must_use]
    pub fn Content(&self) -> Arc<ContentManager> {
        Arc::clone(
            &self
                .content
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }

    pub fn SetContent(&self, value: Arc<ContentManager>) {
        *self
            .content
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
    }

    pub(crate) fn cleanup_content(&self) -> Result<()> {
        self.Content().cleanup_for_game_shutdown()
    }

    #[must_use]
    pub fn Window(&self) -> &GameWindow {
        &self.window
    }

    pub fn GraphicsDevice(&self) -> Result<&GraphicsDevice> {
        self.graphics_device
            .get()
            .ok_or(CnaError::InvalidInput("graphics device is not initialized"))
    }

    #[must_use]
    pub fn IsActive(&self) -> bool {
        self.is_active.load(Ordering::Acquire)
    }

    #[must_use]
    pub fn IsFixedTimeStep(&self) -> bool {
        self.is_fixed_time_step.load(Ordering::Acquire)
    }

    pub fn SetIsFixedTimeStep(&self, value: bool) -> Result<()> {
        if let Ok(active) = self.active() {
            active
                .native
                .set_game_is_fixed_time_step(active.handle, value)?;
        }
        self.is_fixed_time_step.store(value, Ordering::Release);
        Ok(())
    }

    #[must_use]
    pub fn IsMouseVisible(&self) -> bool {
        self.is_mouse_visible.load(Ordering::Acquire)
    }

    pub fn SetIsMouseVisible(&self, value: bool) -> Result<()> {
        if let Ok(active) = self.active() {
            active
                .native
                .set_game_is_mouse_visible(active.handle, value)?;
        }
        self.is_mouse_visible.store(value, Ordering::Release);
        Ok(())
    }

    #[must_use]
    pub fn TargetElapsedTime(&self) -> TimeSpan {
        TimeSpan::from_ticks(self.target_elapsed_time_ticks.load(Ordering::Acquire))
    }

    pub fn SetTargetElapsedTime(&self, value: TimeSpan) -> Result<()> {
        if value.Ticks() <= 0 {
            return Err(CnaError::InvalidInput(
                "target elapsed time must be positive",
            ));
        }
        if let Ok(active) = self.active() {
            active
                .native
                .set_game_target_elapsed_time_ticks(active.handle, value.Ticks())?;
        }
        self.target_elapsed_time_ticks
            .store(value.Ticks(), Ordering::Release);
        Ok(())
    }

    #[must_use]
    pub fn InactiveSleepTime(&self) -> TimeSpan {
        TimeSpan::from_ticks(self.inactive_sleep_time_ticks.load(Ordering::Acquire))
    }

    pub fn SetInactiveSleepTime(&self, value: TimeSpan) -> Result<()> {
        if value.Ticks() < 0 {
            return Err(CnaError::InvalidInput(
                "inactive sleep time must not be negative",
            ));
        }
        if let Ok(active) = self.active() {
            active
                .native
                .set_game_inactive_sleep_time_ticks(active.handle, value.Ticks())?;
        }
        self.inactive_sleep_time_ticks
            .store(value.Ticks(), Ordering::Release);
        Ok(())
    }

    pub(crate) fn refresh_native_properties(&self) -> Result<()> {
        let active = self.active()?;
        self.is_active.store(
            active.native.game_is_active(active.handle)?,
            Ordering::Release,
        );
        self.is_mouse_visible.store(
            active.native.game_is_mouse_visible(active.handle)?,
            Ordering::Release,
        );
        self.is_fixed_time_step.store(
            active.native.game_is_fixed_time_step(active.handle)?,
            Ordering::Release,
        );
        self.target_elapsed_time_ticks.store(
            active
                .native
                .game_target_elapsed_time_ticks(active.handle)?,
            Ordering::Release,
        );
        self.inactive_sleep_time_ticks.store(
            active
                .native
                .game_inactive_sleep_time_ticks(active.handle)?,
            Ordering::Release,
        );
        Ok(())
    }

    pub(crate) fn initialize_components(&self) {
        self.components.initialize_all();
    }

    pub(crate) fn update_components(&self, time: &super::GameTime) {
        self.components.update_all(time);
    }

    pub(crate) fn draw_components(&self, time: &super::GameTime) {
        self.components.draw_all(time);
    }

    pub(crate) fn reset_elapsed_time(&self) -> Result<()> {
        let active = self.active()?;
        active.native.reset_game_elapsed_time(active.handle)
    }

    pub(crate) fn suppress_draw(&self) -> Result<()> {
        let active = self.active()?;
        active.native.suppress_game_draw(active.handle)
    }

    pub(crate) fn tick(&self) -> Result<()> {
        let active = self.active()?;
        active.native.tick_game(active.handle)
    }

    pub(crate) fn exit(&self) -> Result<()> {
        let active = self.active()?;
        active.native.request_game_exit(active.handle)
    }

    pub(crate) fn add_activated(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.activated.add(handler)
    }

    pub(crate) fn remove_activated(&self, registration: u64) -> bool {
        self.activated.remove(registration)
    }

    pub(crate) fn add_deactivated(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.deactivated.add(handler)
    }

    pub(crate) fn remove_deactivated(&self, registration: u64) -> bool {
        self.deactivated.remove(registration)
    }

    pub(crate) fn add_exiting(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.exiting.add(handler)
    }

    pub(crate) fn remove_exiting(&self, registration: u64) -> bool {
        self.exiting.remove(registration)
    }

    pub(crate) fn add_disposed(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.disposed.add(handler)
    }

    pub(crate) fn remove_disposed(&self, registration: u64) -> bool {
        self.disposed.remove(registration)
    }

    pub(crate) fn emit_activated(&self) -> bool {
        self.activated.emit(self, EventArgs)
    }

    pub(crate) fn emit_deactivated(&self) -> bool {
        self.deactivated.emit(self, EventArgs)
    }

    pub(crate) fn emit_exiting(&self) -> bool {
        self.exiting.emit(self, EventArgs)
    }

    pub(crate) fn emit_disposed(&self) -> bool {
        if self.disposed_once.swap(true, Ordering::AcqRel) {
            false
        } else {
            self.disposed.emit(self, EventArgs)
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

/// Composition hook that gives the XNA `Game` trait durable per-instance state.
pub trait GameStateAccess {
    fn game_state(&self) -> &Arc<GameState>;

    fn game_state_arc(&self) -> Arc<GameState> {
        Arc::clone(self.game_state())
    }
}
