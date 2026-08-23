#![allow(
    non_snake_case,
    non_upper_case_globals,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use core::ops::{BitAnd, BitOr, BitOrAssign};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::error::Result;
use crate::extensions::events::{EventArgs, EventHandler};
use crate::extensions::window::WindowHandle;
use crate::graphics::resource::EventHandlers;
use crate::native::Native;
use crate::value::Rectangle;

use super::state::ActiveGame;

/// Open flags representation of XNA's supported display orientations.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct DisplayOrientation(i32);

impl DisplayOrientation {
    pub const Default: Self = Self(0);
    pub const LandscapeLeft: Self = Self(1);
    pub const LandscapeRight: Self = Self(2);
    pub const Portrait: Self = Self(4);

    pub(crate) const fn from_bits(value: i32) -> Self {
        Self(value)
    }
}

impl BitOr for DisplayOrientation {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for DisplayOrientation {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for DisplayOrientation {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

/// Stable managed identity for the one window owned by a game.
pub struct GameWindow {
    binding: Mutex<Option<ActiveGame>>,
    title: Mutex<String>,
    allow_user_resizing: AtomicBool,
    client_x: AtomicI32,
    client_y: AtomicI32,
    client_width: AtomicI32,
    client_height: AtomicI32,
    orientation: AtomicI32,
    native_handle: AtomicU64,
    screen_device_name: Mutex<String>,
    supported_orientations: AtomicI32,
    screen_device_name_changed: EventHandlers<EventArgs>,
    client_size_changed: EventHandlers<EventArgs>,
    orientation_changed: EventHandlers<EventArgs>,
}

impl GameWindow {
    pub(crate) fn new(title: &str) -> Self {
        Self {
            binding: Mutex::new(None),
            title: Mutex::new(title.to_owned()),
            allow_user_resizing: AtomicBool::new(false),
            client_x: AtomicI32::new(0),
            client_y: AtomicI32::new(0),
            client_width: AtomicI32::new(0),
            client_height: AtomicI32::new(0),
            orientation: AtomicI32::new(DisplayOrientation::Default.0),
            native_handle: AtomicU64::new(0),
            screen_device_name: Mutex::new(String::new()),
            supported_orientations: AtomicI32::new(DisplayOrientation::Default.0),
            screen_device_name_changed: EventHandlers::new(),
            client_size_changed: EventHandlers::new(),
            orientation_changed: EventHandlers::new(),
        }
    }

    pub(crate) fn attach(&self, native: &Arc<Native>, game: cna_sys::CNA_Handle) -> Result<()> {
        let requested_allow_user_resizing = self.allow_user_resizing.load(Ordering::Acquire);
        let binding = ActiveGame {
            native: Arc::clone(native),
            handle: game,
        };
        let mut allow_user_resizing = native.game_window_allow_user_resizing(game)?;
        if allow_user_resizing != requested_allow_user_resizing {
            native.set_game_window_allow_user_resizing(game, requested_allow_user_resizing)?;
            allow_user_resizing = requested_allow_user_resizing;
        }
        let bounds = native.game_window_client_bounds(game)?;
        let orientation =
            i32::try_from(native.game_window_current_orientation(game)?).map_err(|_| {
                crate::error::CnaError::InvalidInput(
                    "window orientation exceeds the Rust flags representation",
                )
            })?;
        let handle = native.game_window_native_handle(game)?;
        let screen_device_name = native.game_window_screen_device_name(game)?;
        let title = native.game_window_title(game)?;

        self.allow_user_resizing
            .store(allow_user_resizing, Ordering::Release);
        self.client_x.store(bounds.x, Ordering::Release);
        self.client_y.store(bounds.y, Ordering::Release);
        self.client_width.store(bounds.width, Ordering::Release);
        self.client_height.store(bounds.height, Ordering::Release);
        self.orientation.store(orientation, Ordering::Release);
        self.native_handle.store(handle, Ordering::Release);
        *self
            .screen_device_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = screen_device_name;
        *self
            .title
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = title;
        *self
            .binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(binding);
        Ok(())
    }

    pub(crate) fn detach(&self) {
        self.binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }

    fn active(&self) -> Option<ActiveGame> {
        self.binding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    #[must_use]
    pub fn Title(&self) -> String {
        self.title
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn SetTitle(&self, title: &str) -> Result<()> {
        if let Some(active) = self.active() {
            active.native.set_game_window_title(active.handle, title)?;
        }
        *self
            .title
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = title.to_owned();
        Ok(())
    }

    #[must_use]
    pub fn Handle(&self) -> WindowHandle {
        WindowHandle(self.native_handle.load(Ordering::Acquire))
    }

    #[must_use]
    pub fn AllowUserResizing(&self) -> bool {
        self.allow_user_resizing.load(Ordering::Acquire)
    }

    pub fn SetAllowUserResizing(&self, value: bool) -> Result<()> {
        if let Some(active) = self.active() {
            active
                .native
                .set_game_window_allow_user_resizing(active.handle, value)?;
        }
        self.allow_user_resizing.store(value, Ordering::Release);
        Ok(())
    }

    #[must_use]
    pub fn ClientBounds(&self) -> Rectangle {
        Rectangle::new(
            self.client_x.load(Ordering::Acquire),
            self.client_y.load(Ordering::Acquire),
            self.client_width.load(Ordering::Acquire),
            self.client_height.load(Ordering::Acquire),
        )
    }

    #[must_use]
    pub fn ScreenDeviceName(&self) -> String {
        self.screen_device_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    #[must_use]
    pub fn CurrentOrientation(&self) -> DisplayOrientation {
        DisplayOrientation(self.orientation.load(Ordering::Acquire))
    }

    pub fn AddScreenDeviceNameChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.screen_device_name_changed.add(handler)
    }

    pub fn RemoveScreenDeviceNameChangedHandler(&self, registration: u64) -> bool {
        self.screen_device_name_changed.remove(registration)
    }

    pub fn AddClientSizeChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.client_size_changed.add(handler)
    }

    pub fn RemoveClientSizeChangedHandler(&self, registration: u64) -> bool {
        self.client_size_changed.remove(registration)
    }

    pub fn AddOrientationChangedHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.orientation_changed.add(handler)
    }

    pub fn RemoveOrientationChangedHandler(&self, registration: u64) -> bool {
        self.orientation_changed.remove(registration)
    }

    pub fn BeginScreenDeviceChange(&self, willBeFullScreen: bool) -> Result<()> {
        if let Some(active) = self.active() {
            active
                .native
                .begin_game_screen_device_change(active.handle, willBeFullScreen)?;
        }
        Ok(())
    }

    pub fn EndScreenDeviceChange(
        &self,
        screenDeviceName: &str,
        clientWidth: i32,
        clientHeight: i32,
    ) -> Result<()> {
        if clientWidth < 0 || clientHeight < 0 {
            return Err(crate::error::CnaError::InvalidInput(
                "client dimensions must not be negative",
            ));
        }
        if let Some(active) = self.active() {
            active.native.end_game_screen_device_change(
                active.handle,
                screenDeviceName,
                clientWidth,
                clientHeight,
            )?;
        }
        self.client_width.store(clientWidth, Ordering::Release);
        self.client_height.store(clientHeight, Ordering::Release);
        *self
            .screen_device_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = screenDeviceName.to_owned();
        Ok(())
    }

    pub fn EndScreenDeviceChangeWithScreenDeviceName(&self, screenDeviceName: &str) -> Result<()> {
        let bounds = self.ClientBounds();
        self.EndScreenDeviceChange(screenDeviceName, bounds.Width.max(0), bounds.Height.max(0))
    }

    pub fn SetSupportedOrientations(&self, orientations: DisplayOrientation) {
        self.supported_orientations
            .store(orientations.0, Ordering::Release);
    }

    pub fn OnActivated(&self) {}

    pub fn OnDeactivated(&self) {}

    pub fn OnPaint(&self) {}

    pub fn OnScreenDeviceNameChanged(&self) {
        let _ = self.screen_device_name_changed.emit(self, EventArgs);
    }

    pub fn OnClientSizeChanged(&self) {
        let _ = self.client_size_changed.emit(self, EventArgs);
    }

    pub fn OnOrientationChanged(&self) {
        let _ = self.orientation_changed.emit(self, EventArgs);
    }

    pub(crate) fn native_client_size_changed(&self) -> Result<()> {
        let Some(active) = self.active() else {
            return Ok(());
        };
        let bounds = active.native.game_window_client_bounds(active.handle)?;
        self.client_x.store(bounds.x, Ordering::Release);
        self.client_y.store(bounds.y, Ordering::Release);
        self.client_width.store(bounds.width, Ordering::Release);
        self.client_height.store(bounds.height, Ordering::Release);
        self.OnClientSizeChanged();
        Ok(())
    }

    pub(crate) fn native_orientation_changed(&self) -> Result<()> {
        let Some(active) = self.active() else {
            return Ok(());
        };
        let orientation = i32::try_from(
            active
                .native
                .game_window_current_orientation(active.handle)?,
        )
        .map_err(|_| {
            crate::error::CnaError::InvalidInput(
                "window orientation exceeds the Rust flags representation",
            )
        })?;
        self.orientation.store(orientation, Ordering::Release);
        self.OnOrientationChanged();
        Ok(())
    }

    pub(crate) fn native_screen_device_name_changed(&self) -> Result<()> {
        let Some(active) = self.active() else {
            return Ok(());
        };
        let name = active
            .native
            .game_window_screen_device_name(active.handle)?;
        *self
            .screen_device_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = name;
        self.OnScreenDeviceNameChanged();
        Ok(())
    }
}
