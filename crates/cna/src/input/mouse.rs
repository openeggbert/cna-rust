#![allow(non_snake_case)]

use core::{any::Any, mem::size_of};

use cna_sys as sys;

use crate::error::Result;
use crate::extensions::window::WindowHandle;
use crate::game::GameContext;

use super::ButtonState;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MouseState {
    x: i32,
    y: i32,
    left_button: ButtonState,
    right_button: ButtonState,
    middle_button: ButtonState,
    x_button1: ButtonState,
    x_button2: ButtonState,
    wheel: i32,
}

impl MouseState {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        x: i32,
        y: i32,
        scrollWheel: i32,
        leftButton: ButtonState,
        middleButton: ButtonState,
        rightButton: ButtonState,
        xButton1: ButtonState,
        xButton2: ButtonState,
    ) -> Self {
        Self {
            x,
            y,
            left_button: leftButton,
            right_button: rightButton,
            middle_button: middleButton,
            x_button1: xButton1,
            x_button2: xButton2,
            wheel: scrollWheel,
        }
    }

    #[must_use]
    pub const fn X(&self) -> i32 {
        self.x
    }
    #[must_use]
    pub const fn Y(&self) -> i32 {
        self.y
    }
    #[must_use]
    pub const fn LeftButton(&self) -> ButtonState {
        self.left_button
    }
    #[must_use]
    pub const fn RightButton(&self) -> ButtonState {
        self.right_button
    }
    #[must_use]
    pub const fn MiddleButton(&self) -> ButtonState {
        self.middle_button
    }
    #[must_use]
    pub const fn XButton1(&self) -> ButtonState {
        self.x_button1
    }
    #[must_use]
    pub const fn XButton2(&self) -> ButtonState {
        self.x_button2
    }
    #[must_use]
    pub const fn ScrollWheelValue(&self) -> i32 {
        self.wheel
    }

    #[must_use]
    pub fn GetHashCode(&self) -> i32 {
        self.x
            ^ self.y
            ^ self.left_button as i32
            ^ self.right_button as i32
            ^ self.middle_button as i32
            ^ self.x_button1 as i32
            ^ self.x_button2 as i32
            ^ self.wheel
    }

    #[must_use]
    pub fn Equals(&self, obj: &dyn Any) -> bool {
        obj.downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }

    #[must_use]
    pub fn ToString(&self) -> String {
        let mut pressed = Vec::new();
        if self.left_button == ButtonState::Pressed {
            pressed.push("Left");
        }
        if self.right_button == ButtonState::Pressed {
            pressed.push("Right");
        }
        if self.middle_button == ButtonState::Pressed {
            pressed.push("Middle");
        }
        if self.x_button1 == ButtonState::Pressed {
            pressed.push("XButton1");
        }
        if self.x_button2 == ButtonState::Pressed {
            pressed.push("XButton2");
        }
        let buttons = if pressed.is_empty() {
            "None".to_owned()
        } else {
            pressed.join(" ")
        };
        format!(
            "{{X:{} Y:{} Buttons:{} Wheel:{}}}",
            self.x, self.y, buttons, self.wheel
        )
    }

    fn from_native(value: sys::CNA_MouseState) -> Self {
        let pressed = value.pressed_buttons;
        Self::new(
            value.x,
            value.y,
            value.scroll_wheel,
            button(pressed, sys::CNA_MOUSE_BUTTON_LEFT),
            button(pressed, sys::CNA_MOUSE_BUTTON_MIDDLE),
            button(pressed, sys::CNA_MOUSE_BUTTON_RIGHT),
            button(pressed, sys::CNA_MOUSE_BUTTON_X1),
            button(pressed, sys::CNA_MOUSE_BUTTON_X2),
        )
    }
}

pub struct Mouse;

impl Mouse {
    pub fn WindowHandle(game: &GameContext<'_>) -> Result<WindowHandle> {
        let mut value = 0;
        game.native.mouse_window_handle(game.handle, &mut value)?;
        Ok(WindowHandle(value))
    }

    pub fn SetWindowHandle(game: &GameContext<'_>, value: WindowHandle) -> Result<()> {
        game.native.set_mouse_window_handle(game.handle, value.0)
    }

    pub fn GetState(game: &GameContext<'_>) -> Result<MouseState> {
        let mut state = sys::CNA_MouseState {
            struct_size: size_of::<sys::CNA_MouseState>() as u32,
            struct_version: 1,
            ..sys::CNA_MouseState::default()
        };
        game.native.mouse_state(game.handle, &mut state)?;
        Ok(MouseState::from_native(state))
    }

    pub fn SetPosition(game: &GameContext<'_>, x: i32, y: i32) -> Result<()> {
        game.native.set_mouse_position(game.handle, x, y)
    }
}

const fn button(
    pressed: sys::CNA_MouseButtonFlags,
    value: sys::CNA_MouseButtonFlags,
) -> ButtonState {
    if pressed & value != 0 {
        ButtonState::Pressed
    } else {
        ButtonState::Released
    }
}
