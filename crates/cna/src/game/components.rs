#![allow(non_snake_case, clippy::missing_errors_doc)]

use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex, Weak};

use crate::error::Result;
use crate::extensions::events::{EventArgs, EventHandler};
use crate::graphics::resource::EventHandlers;
use crate::graphics::GraphicsDevice;

use super::{Game, GameState, GameTime};

/// Runtime query hooks needed because Rust cannot dynamically cast one trait
/// object to an unrelated trait object as the CLR can.
pub trait GameComponentRuntime: Any + Send + Sync {
    fn AsUpdateable(&self) -> Option<&dyn IUpdateable> {
        None
    }

    fn AsDrawable(&self) -> Option<&dyn IDrawable> {
        None
    }
}

pub trait IGameComponent: GameComponentRuntime {
    fn Initialize(&self);
}

pub trait IUpdateable: Send + Sync {
    fn Enabled(&self) -> bool;
    fn UpdateOrder(&self) -> i32;
    fn AddEnabledChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64;
    fn RemoveEnabledChangedHandler(&self, registration: u64) -> bool;
    fn AddUpdateOrderChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64;
    fn RemoveUpdateOrderChangedHandler(&self, registration: u64) -> bool;
    fn Update(&self, gameTime: &GameTime);
}

pub trait IDrawable: Send + Sync {
    fn Visible(&self) -> bool;
    fn DrawOrder(&self) -> i32;
    fn AddVisibleChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64;
    fn RemoveVisibleChangedHandler(&self, registration: u64) -> bool;
    fn AddDrawOrderChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64;
    fn RemoveDrawOrderChangedHandler(&self, registration: u64) -> bool;
    fn Draw(&self, gameTime: &GameTime);
}

/// Concrete XNA component base behavior represented through composition.
pub struct GameComponent {
    game: Weak<GameState>,
    enabled: AtomicBool,
    update_order: AtomicI32,
    disposed: AtomicBool,
    enabled_changed: EventHandlers<EventArgs>,
    update_order_changed: EventHandlers<EventArgs>,
    disposed_event: EventHandlers<EventArgs>,
}

impl GameComponent {
    #[must_use]
    pub fn new(game: &dyn Game) -> Self {
        Self::from_game_state(game.game_state())
    }

    fn from_game_state(game: &Arc<GameState>) -> Self {
        Self {
            game: Arc::downgrade(game),
            enabled: AtomicBool::new(true),
            update_order: AtomicI32::new(0),
            disposed: AtomicBool::new(false),
            enabled_changed: EventHandlers::new(),
            update_order_changed: EventHandlers::new(),
            disposed_event: EventHandlers::new(),
        }
    }

    #[must_use]
    pub fn Enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    pub fn SetEnabled(&mut self, value: bool) {
        if self.enabled.swap(value, Ordering::AcqRel) != value {
            self.OnEnabledChanged(self, EventArgs);
        }
    }

    #[must_use]
    pub fn UpdateOrder(&self) -> i32 {
        self.update_order.load(Ordering::Acquire)
    }

    pub fn SetUpdateOrder(&mut self, value: i32) {
        if self.update_order.swap(value, Ordering::AcqRel) != value {
            self.OnUpdateOrderChanged(self, EventArgs);
        }
    }

    #[must_use]
    pub fn Game(&self) -> Option<Arc<GameState>> {
        self.game.upgrade()
    }

    pub fn AddEnabledChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.enabled_changed.add(handler)
    }

    pub fn RemoveEnabledChangedHandler(&self, registration: u64) -> bool {
        self.enabled_changed.remove(registration)
    }

    pub fn AddUpdateOrderChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.update_order_changed.add(handler)
    }

    pub fn RemoveUpdateOrderChangedHandler(&self, registration: u64) -> bool {
        self.update_order_changed.remove(registration)
    }

    pub fn AddDisposedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.disposed_event.add(handler)
    }

    pub fn RemoveDisposedHandler(&self, registration: u64) -> bool {
        self.disposed_event.remove(registration)
    }

    pub fn Initialize(&self) {}

    pub fn Update(&self, gameTime: &GameTime) {
        let _ = gameTime;
    }

    pub fn Dispose(&mut self) {
        self.DisposeWithDisposing(true);
    }

    pub fn DisposeWithDisposing(&mut self, disposing: bool) {
        if disposing && !self.disposed.swap(true, Ordering::AcqRel) {
            if let Some(game) = self.game.upgrade() {
                game.Components().remove_instance(self);
            }
            let _ = self.disposed_event.emit(self, EventArgs);
        }
    }

    pub fn Finalize(&self) {}

    pub fn OnUpdateOrderChanged(&self, sender: &dyn Any, args: EventArgs) {
        let _ = self.update_order_changed.emit(sender, args);
    }

    pub fn OnEnabledChanged(&self, sender: &dyn Any, args: EventArgs) {
        let _ = self.enabled_changed.emit(sender, args);
    }
}

impl IGameComponent for GameComponent {
    fn Initialize(&self) {
        Self::Initialize(self);
    }
}

impl IUpdateable for GameComponent {
    fn Enabled(&self) -> bool {
        Self::Enabled(self)
    }
    fn UpdateOrder(&self) -> i32 {
        Self::UpdateOrder(self)
    }
    fn AddEnabledChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        Self::AddEnabledChangedHandler(self, handler)
    }
    fn RemoveEnabledChangedHandler(&self, registration: u64) -> bool {
        Self::RemoveEnabledChangedHandler(self, registration)
    }
    fn AddUpdateOrderChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        Self::AddUpdateOrderChangedHandler(self, handler)
    }
    fn RemoveUpdateOrderChangedHandler(&self, registration: u64) -> bool {
        Self::RemoveUpdateOrderChangedHandler(self, registration)
    }
    fn Update(&self, gameTime: &GameTime) {
        Self::Update(self, gameTime);
    }
}

impl GameComponentRuntime for GameComponent {
    fn AsUpdateable(&self) -> Option<&dyn IUpdateable> {
        Some(self)
    }
}

impl Drop for GameComponent {
    fn drop(&mut self) {
        self.DisposeWithDisposing(false);
    }
}

/// Drawable component composed from the updateable component base.
pub struct DrawableGameComponent {
    base: GameComponent,
    visible: AtomicBool,
    draw_order: AtomicI32,
    visible_changed: EventHandlers<EventArgs>,
    draw_order_changed: EventHandlers<EventArgs>,
}

impl DrawableGameComponent {
    #[must_use]
    pub fn new(game: &dyn Game) -> Self {
        Self {
            base: GameComponent::new(game),
            visible: AtomicBool::new(true),
            draw_order: AtomicI32::new(0),
            visible_changed: EventHandlers::new(),
            draw_order_changed: EventHandlers::new(),
        }
    }

    #[must_use]
    pub fn Visible(&self) -> bool {
        self.visible.load(Ordering::Acquire)
    }
    pub fn SetVisible(&mut self, value: bool) {
        if self.visible.swap(value, Ordering::AcqRel) != value {
            self.OnVisibleChanged(self, EventArgs);
        }
    }
    #[must_use]
    pub fn DrawOrder(&self) -> i32 {
        self.draw_order.load(Ordering::Acquire)
    }
    pub fn SetDrawOrder(&mut self, value: i32) {
        if self.draw_order.swap(value, Ordering::AcqRel) != value {
            self.OnDrawOrderChanged(self, EventArgs);
        }
    }
    pub fn GraphicsDevice(&self) -> Result<GraphicsDevice> {
        self.base
            .game
            .upgrade()
            .ok_or(crate::CnaError::InvalidInput("parent game is disposed"))?
            .GraphicsDevice()
            .cloned()
    }
    pub fn AddVisibleChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.visible_changed.add(handler)
    }
    pub fn RemoveVisibleChangedHandler(&self, registration: u64) -> bool {
        self.visible_changed.remove(registration)
    }
    pub fn AddDrawOrderChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.draw_order_changed.add(handler)
    }
    pub fn RemoveDrawOrderChangedHandler(&self, registration: u64) -> bool {
        self.draw_order_changed.remove(registration)
    }
    pub fn Initialize(&self) {
        self.base.Initialize();
        self.LoadContent();
    }
    pub fn Dispose(&mut self, disposing: bool) {
        if disposing {
            if let Some(game) = self.base.game.upgrade() {
                game.Components().remove_instance(self);
            }
            self.UnloadContent();
        }
        self.base.DisposeWithDisposing(disposing);
    }
    pub fn Draw(&self, gameTime: &GameTime) {
        let _ = gameTime;
    }
    pub fn LoadContent(&self) {}
    pub fn UnloadContent(&self) {}
    pub fn OnDrawOrderChanged(&self, sender: &dyn Any, args: EventArgs) {
        let _ = self.draw_order_changed.emit(sender, args);
    }
    pub fn OnVisibleChanged(&self, sender: &dyn Any, args: EventArgs) {
        let _ = self.visible_changed.emit(sender, args);
    }
}

impl IGameComponent for DrawableGameComponent {
    fn Initialize(&self) {
        Self::Initialize(self);
    }
}
impl IUpdateable for DrawableGameComponent {
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
        self.base.Update(gameTime);
    }
}
impl IDrawable for DrawableGameComponent {
    fn Visible(&self) -> bool {
        Self::Visible(self)
    }
    fn DrawOrder(&self) -> i32 {
        Self::DrawOrder(self)
    }
    fn AddVisibleChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        Self::AddVisibleChangedHandler(self, handler)
    }
    fn RemoveVisibleChangedHandler(&self, registration: u64) -> bool {
        Self::RemoveVisibleChangedHandler(self, registration)
    }
    fn AddDrawOrderChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        Self::AddDrawOrderChangedHandler(self, handler)
    }
    fn RemoveDrawOrderChangedHandler(&self, registration: u64) -> bool {
        Self::RemoveDrawOrderChangedHandler(self, registration)
    }
    fn Draw(&self, gameTime: &GameTime) {
        Self::Draw(self, gameTime);
    }
}
impl GameComponentRuntime for DrawableGameComponent {
    fn AsUpdateable(&self) -> Option<&dyn IUpdateable> {
        Some(self)
    }
    fn AsDrawable(&self) -> Option<&dyn IDrawable> {
        Some(self)
    }
}

/// Marker used by the verifier for the composed CLR base-class relationship.
pub trait GameComponentBase {}
impl GameComponentBase for DrawableGameComponent {}

impl Drop for DrawableGameComponent {
    fn drop(&mut self) {
        self.Dispose(false);
    }
}

#[derive(Clone)]
pub struct GameComponentCollectionEventArgs {
    component: Arc<dyn IGameComponent>,
}

impl GameComponentCollectionEventArgs {
    #[must_use]
    pub fn new(gameComponent: Arc<dyn IGameComponent>) -> Self {
        Self {
            component: gameComponent,
        }
    }
    #[must_use]
    pub fn GameComponent(&self) -> Arc<dyn IGameComponent> {
        Arc::clone(&self.component)
    }
}

pub struct GameComponentCollection {
    items: Mutex<Vec<Arc<dyn IGameComponent>>>,
    initialized: AtomicBool,
    component_added: EventHandlers<GameComponentCollectionEventArgs>,
    component_removed: EventHandlers<GameComponentCollectionEventArgs>,
}

impl GameComponentCollection {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
            initialized: AtomicBool::new(false),
            component_added: EventHandlers::new(),
            component_removed: EventHandlers::new(),
        }
    }
    pub fn AddComponentAddedHandler(
        &self,
        handler: Box<dyn EventHandler<GameComponentCollectionEventArgs>>,
    ) -> u64 {
        self.component_added.add(handler)
    }
    pub fn RemoveComponentAddedHandler(&self, registration: u64) -> bool {
        self.component_added.remove(registration)
    }
    pub fn AddComponentRemovedHandler(
        &self,
        handler: Box<dyn EventHandler<GameComponentCollectionEventArgs>>,
    ) -> u64 {
        self.component_removed.add(handler)
    }
    pub fn RemoveComponentRemovedHandler(&self, registration: u64) -> bool {
        self.component_removed.remove(registration)
    }

    /// # Panics
    ///
    /// Panics for a negative/out-of-range index or duplicate component, matching
    /// the mapped XNA collection exception contract.
    pub fn InsertItem(&self, index: i32, item: Arc<dyn IGameComponent>) {
        let index = usize::try_from(index).expect("component index must not be negative");
        let mut items = self
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            index <= items.len(),
            "component index is outside the collection"
        );
        assert!(
            !items.iter().any(|current| Arc::ptr_eq(current, &item)),
            "the same component cannot be added twice"
        );
        items.insert(index, Arc::clone(&item));
        drop(items);
        if self.initialized.load(Ordering::Acquire) {
            item.Initialize();
        }
        let _ = self
            .component_added
            .emit(self, GameComponentCollectionEventArgs::new(item));
    }
    /// # Panics
    ///
    /// Panics when `index` is outside the collection.
    pub fn RemoveItem(&self, index: i32) {
        let index = usize::try_from(index).expect("component index must not be negative");
        let item = self
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(index);
        let _ = self
            .component_removed
            .emit(self, GameComponentCollectionEventArgs::new(item));
    }
    /// # Panics
    ///
    /// Always panics because XNA forbids replacing collection entries.
    pub fn SetItem(&self, index: i32, item: Arc<dyn IGameComponent>) {
        let _ = (index, item);
        panic!("XNA GameComponentCollection does not support replacing items");
    }
    pub fn ClearItems(&self) {
        let removed = self
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for item in &removed {
            let _ = self.component_removed.emit(
                self,
                GameComponentCollectionEventArgs::new(Arc::clone(item)),
            );
        }
        self.items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    pub(crate) fn initialize_all(&self) {
        self.initialized.store(true, Ordering::Release);
        let snapshot = self.snapshot();
        for component in snapshot {
            component.Initialize();
        }
    }
    pub(crate) fn update_all(&self, time: &GameTime) {
        let mut snapshot = self
            .snapshot()
            .into_iter()
            .filter_map(|component| {
                let order = component.AsUpdateable()?.UpdateOrder();
                Some((order, component))
            })
            .collect::<Vec<_>>();
        snapshot.sort_by_key(|(order, _)| *order);
        for (_, component) in snapshot {
            if let Some(value) = component.AsUpdateable() {
                if value.Enabled() {
                    value.Update(time);
                }
            }
        }
    }
    pub(crate) fn draw_all(&self, time: &GameTime) {
        let mut snapshot = self
            .snapshot()
            .into_iter()
            .filter_map(|component| {
                let order = component.AsDrawable()?.DrawOrder();
                Some((order, component))
            })
            .collect::<Vec<_>>();
        snapshot.sort_by_key(|(order, _)| *order);
        for (_, component) in snapshot {
            if let Some(value) = component.AsDrawable() {
                if value.Visible() {
                    value.Draw(time);
                }
            }
        }
    }
    fn snapshot(&self) -> Vec<Arc<dyn IGameComponent>> {
        self.items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn remove_instance(&self, component: &dyn IGameComponent) -> bool {
        let component_data = (component as *const dyn IGameComponent).cast::<()>();
        let index = self
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .position(|current| {
                (current.as_ref() as *const dyn IGameComponent).cast::<()>() == component_data
            });
        if let Some(index) = index {
            self.RemoveItem(i32::try_from(index).expect("component index fits i32"));
            true
        } else {
            false
        }
    }
}

impl Default for GameComponentCollection {
    fn default() -> Self {
        Self::new()
    }
}

pub trait GameComponentCollectionExt {
    fn Add(&self, item: Arc<dyn IGameComponent>);
    fn Remove(&self, item: &Arc<dyn IGameComponent>) -> bool;
    fn Clear(&self);
    fn Count(&self) -> usize;
    fn Item(&self, index: usize) -> Arc<dyn IGameComponent>;
}

impl GameComponentCollectionExt for GameComponentCollection {
    fn Add(&self, item: Arc<dyn IGameComponent>) {
        self.InsertItem(
            i32::try_from(self.Count()).expect("component count fits i32"),
            item,
        );
    }
    fn Remove(&self, item: &Arc<dyn IGameComponent>) -> bool {
        let index = self
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .position(|current| Arc::ptr_eq(current, item));
        if let Some(index) = index {
            self.RemoveItem(i32::try_from(index).expect("component index fits i32"));
            true
        } else {
            false
        }
    }
    fn Clear(&self) {
        self.ClearItems();
    }
    fn Count(&self) -> usize {
        self.items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }
    fn Item(&self, index: usize) -> Arc<dyn IGameComponent> {
        Arc::clone(
            &self
                .items
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)[index],
        )
    }
}
