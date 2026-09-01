//! CNA's substitute touch panel: `input_touch.h` beyond XNA's `TouchPanel`.
//!
//! XNA's touch API reads a device. CNA also publishes the panel's own state --
//! whether a touch device exists at all, whether a mouse is reported as a
//! finger, where a finger is, and the frame boundary that turns queued input
//! into a `TouchCollection`. None of that is XNA, so none of it belongs in the
//! strict hierarchy.

use cna_sys as sys;

use crate::error::Result;
use crate::game::GameContext;
use crate::input::{GestureSample, TouchLocationState};
use crate::value::Vector2;

/// CNA's substitute touch panel.
///
/// No machine this crate is verified on has a touchscreen. Upstream ships a
/// panel that answers as one would, and without it the whole touch projection
/// could only ever be exercised against "no touch device" -- a gesture would
/// never be recognised, a `TouchCollection` would never hold anything, and the
/// `TouchPanel` accessors would only ever return their defaults.
///
/// These routes move CNA's own state rather than a device's, so nothing here
/// is something a game asks of its hardware. That does not make it private:
/// eight of the nine canonical routes behind it are `CNA_EXTENSION_BACKING`
/// -- a software panel, mouse-touch emulation, raising an event through the
/// path a device's would take -- and only `cna_touch_panel_reset_for_tests_ext`
/// is `TOOLING_ONLY`. So it is public, and it is here rather than in
/// `cna::Microsoft::Xna::Framework::Input::Touch`, where it was until
/// `RUST-SURFACE-001`: XNA declares no such type, and the strict hierarchy
/// carries only what XNA declares.
///
/// It still speaks in strict XNA values -- [`TouchLocationState`],
/// [`GestureSample`], [`Vector2`] -- because what it drives is XNA's
/// `TouchPanel`. Accepting an XNA type is not being one.
pub struct TouchPanelTestBackend;

#[allow(non_snake_case)]
impl TouchPanelTestBackend {
    /// Whether CNA currently believes a touch device exists.
    pub fn touch_device_exists(game: &GameContext<'_>) -> Result<bool> {
        let (native, handle) = game.native_game();
        let mut value = sys::CNA_FALSE;
        // SAFETY: the game handle is callback-live and the output is a local.
        native.check(unsafe {
            (native.runtime.touch_panel_get_touch_device_exists_ext)(handle, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Tells CNA whether a touch device exists.
    ///
    /// `TouchPanel::GetCapabilities` reads this, so it is the switch that
    /// turns the whole family from "unsupported" into something to measure.
    pub fn set_touch_device_exists(game: &GameContext<'_>, exists: bool) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: the game handle is callback-live and the flag is by value.
        native.check(unsafe {
            (native.runtime.touch_panel_set_touch_device_exists_ext)(handle, u8::from(exists))
        })
    }

    /// Whether a mouse is being reported as a finger.
    pub fn mouse_touch_emulation_enabled(game: &GameContext<'_>) -> Result<bool> {
        let (native, handle) = game.native_game();
        let mut value = sys::CNA_FALSE;
        // SAFETY: callback-live handle, live output.
        native.check(unsafe {
            (native.runtime.touch_panel_get_mouse_touch_emulation_enabled_ext)(handle, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Reports the mouse as a finger, or stops.
    pub fn set_mouse_touch_emulation_enabled(
        game: &GameContext<'_>,
        enabled: bool,
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: callback-live handle, flag by value.
        native.check(unsafe {
            (native.runtime.touch_panel_set_mouse_touch_emulation_enabled_ext)(
                handle,
                u8::from(enabled),
            )
        })
    }

    /// Places one finger at a position, as a device would report it.
    pub fn set_finger(
        game: &GameContext<'_>,
        device: i32,
        finger: i32,
        position: Vector2,
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: callback-live handle; the position is by value.
        native.check(unsafe {
            (native.runtime.touch_panel_set_finger_ext)(
                handle,
                device,
                finger,
                sys::CNA_Vector2 {
                    x: position.X,
                    y: position.Y,
                },
            )
        })
    }

    /// Raises one touch event through the same path a device's would take.
    pub fn raise_touch_event(
        game: &GameContext<'_>,
        finger: i32,
        state: TouchLocationState,
        x: f32,
        y: f32,
        pressure: f32,
        timestamp_seconds: f32,
    ) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: callback-live handle; every argument is by value.
        native.check(unsafe {
            (native.runtime.touch_panel_raise_touch_event_ext)(
                handle,
                finger,
                state as u32,
                x,
                y,
                pressure,
                timestamp_seconds,
            )
        })
    }

    /// Queues one gesture for `TouchPanel::ReadGesture` to hand back.
    ///
    /// The recogniser is CNA's and needs real timing to produce a pinch or a
    /// flick; this is how a test asserts what a game *does* with one without
    /// reproducing the input that would cause it.
    pub fn enqueue_gesture(game: &GameContext<'_>, sample: &GestureSample) -> Result<()> {
        let (native, handle) = game.native_game();
        let native_sample = sys::CNA_GestureSample {
            struct_size: core::mem::size_of::<sys::CNA_GestureSample>() as u32,
            struct_version: 1,
            gesture_type: sample.GestureType().bits() as u32,
            finger_id_ext: 0,
            finger_id2_ext: 0,
            reserved: 0,
            timestamp_ticks: sample.Timestamp().Ticks(),
            position: sys::CNA_Vector2 {
                x: sample.Position().X,
                y: sample.Position().Y,
            },
            position2: sys::CNA_Vector2 {
                x: sample.Position2().X,
                y: sample.Position2().Y,
            },
            delta: sys::CNA_Vector2 {
                x: sample.Delta().X,
                y: sample.Delta().Y,
            },
            delta2: sys::CNA_Vector2 {
                x: sample.Delta2().X,
                y: sample.Delta2().Y,
            },
        };
        // SAFETY: callback-live handle and a live local CNA copies.
        native.check(unsafe {
            (native.runtime.touch_panel_enqueue_gesture_ext)(handle, &native_sample)
        })
    }

    /// Advances the panel a frame, turning queued input into state.
    pub fn update(game: &GameContext<'_>) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: the game handle is callback-live.
        native.check(unsafe { (native.runtime.touch_panel_update_ext)(handle) })
    }

    /// Clears every finger, gesture and flag the panel is holding.
    ///
    /// What a test calls between cases so one does not leak into the next --
    /// the panel is process-global, exactly like the sensors.
    pub fn reset(game: &GameContext<'_>) -> Result<()> {
        let (native, handle) = game.native_game();
        // SAFETY: the game handle is callback-live.
        native.check(unsafe { (native.runtime.touch_panel_reset_for_tests_ext)(handle) })
    }
}
