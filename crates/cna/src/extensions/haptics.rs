//! CNA's force-feedback devices.
//!
//! XNA had exactly one haptic operation: `GamePad.SetVibration(index, left,
//! right)`, two motor amplitudes on a controller. CNA reports the device --
//! how many axes it has, how many effects it can hold, how many can play at
//! once, which waveform and condition families it supports, and whether it
//! takes a global gain or an autocentre setting.
//!
//! Those are not the same thing, and this module does **not** compress the
//! second into the first. `GamePad.SetVibration` stays exactly where XNA put
//! it; a wheel that supports spring, damper and friction conditions is
//! described here, because describing it as "left motor, right motor" would
//! discard almost everything true about it.

#![allow(clippy::missing_errors_doc)]

use std::sync::Arc;

use cna_sys as sys;

use crate::error::Result;
use crate::game::GameContext;
use crate::native::runtime::read_string;
use crate::native::Native;

/// What one haptic device can do.
///
/// A bit set rather than a list of booleans, because that is what CNA reports
/// and because a device supports an arbitrary combination.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HapticFeatures(u32);

macro_rules! feature {
    ($name:ident, $constant:ident, $doc:literal) => {
        #[doc = $doc]
        pub const $name: Self = Self(sys::$constant);
    };
}

impl HapticFeatures {
    feature!(NONE, CNA_HAPTIC_FEATURE_NONE, "No feature at all.");
    feature!(CONSTANT, CNA_HAPTIC_FEATURE_CONSTANT, "A constant force.");
    feature!(SINE, CNA_HAPTIC_FEATURE_SINE, "A sine waveform.");
    feature!(SQUARE, CNA_HAPTIC_FEATURE_SQUARE, "A square waveform.");
    feature!(TRIANGLE, CNA_HAPTIC_FEATURE_TRIANGLE, "A triangle waveform.");
    feature!(SAWTOOTH_UP, CNA_HAPTIC_FEATURE_SAWTOOTH_UP, "A rising sawtooth.");
    feature!(SAWTOOTH_DOWN, CNA_HAPTIC_FEATURE_SAWTOOTH_DOWN, "A falling sawtooth.");
    feature!(RAMP, CNA_HAPTIC_FEATURE_RAMP, "A ramp from one force to another.");
    feature!(SPRING, CNA_HAPTIC_FEATURE_SPRING, "A spring condition.");
    feature!(DAMPER, CNA_HAPTIC_FEATURE_DAMPER, "A damper condition.");
    feature!(INERTIA, CNA_HAPTIC_FEATURE_INERTIA, "An inertia condition.");
    feature!(FRICTION, CNA_HAPTIC_FEATURE_FRICTION, "A friction condition.");
    feature!(LEFT_RIGHT, CNA_HAPTIC_FEATURE_LEFT_RIGHT, "Two-motor rumble -- XNA's whole haptic vocabulary.");
    feature!(CUSTOM, CNA_HAPTIC_FEATURE_CUSTOM, "A caller-supplied waveform.");
    feature!(GAIN, CNA_HAPTIC_FEATURE_GAIN, "A global output gain.");
    feature!(AUTOCENTER, CNA_HAPTIC_FEATURE_AUTOCENTER, "A self-centring force.");
    feature!(STATUS, CNA_HAPTIC_FEATURE_STATUS, "Per-effect playback status.");
    feature!(PAUSE, CNA_HAPTIC_FEATURE_PAUSE, "Pausing and resuming playback.");
    feature!(ALL, CNA_HAPTIC_FEATURE_ALL, "Every feature this ABI names.");

    /// The raw bit set.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Whether every feature in `other` is present.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Whether any feature at all is present.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// One haptic device's shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HapticCapabilities {
    pub features: HapticFeatures,
    /// How many axes forces can be directed along.
    pub axis_count: i32,
    /// How many effects the device can hold at once.
    pub max_effects: i32,
    /// How many of those can play simultaneously.
    pub max_effects_playing: i32,
    pub is_open: bool,
    /// Whether the simple rumble path works, which is the one XNA had.
    pub rumble_supported: bool,
}

impl HapticCapabilities {
    /// CNA's defaults for a device that reports nothing.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_HapticCapabilities {
            struct_size: core::mem::size_of::<sys::CNA_HapticCapabilities>() as u32,
            struct_version: 1,
            ..sys::CNA_HapticCapabilities::default()
        };
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.runtime.haptic_capabilities_init)(&mut value) })?;
        Ok(Self::from_native(value))
    }

    /// Whether CNA considers these the same capabilities as `other`.
    ///
    /// Asks CNA rather than comparing the Rust fields, for the same reason
    /// device identity does: CNA defines what equality means here -- it
    /// compares the device name alongside the numbers -- and a derived `==`
    /// that agreed today would be a guess.
    pub fn same_capabilities(&self, name: &str, other: &Self, other_name: &str) -> Result<bool> {
        let native = Native::process()?;
        let left = self.to_native();
        let right = other.to_native();
        let mut equal = sys::CNA_FALSE;
        // SAFETY: both descriptors and both names are live for the call, and
        // the output is a live local.
        native.check(unsafe {
            (native.runtime.haptic_capabilities_equals)(
                &left,
                view(name),
                &right,
                view(other_name),
                &mut equal,
            )
        })?;
        Ok(equal != sys::CNA_FALSE)
    }

    fn to_native(self) -> sys::CNA_HapticCapabilities {
        sys::CNA_HapticCapabilities {
            struct_size: core::mem::size_of::<sys::CNA_HapticCapabilities>() as u32,
            struct_version: 1,
            features: self.features.0,
            axis_count: self.axis_count,
            max_effects: self.max_effects,
            max_effects_playing: self.max_effects_playing,
            is_open: u8::from(self.is_open),
            rumble_supported: u8::from(self.rumble_supported),
            reserved: [0; 2],
        }
    }

    const fn from_native(value: sys::CNA_HapticCapabilities) -> Self {
        Self {
            features: HapticFeatures(value.features),
            axis_count: value.axis_count,
            max_effects: value.max_effects,
            max_effects_playing: value.max_effects_playing,
            is_open: value.is_open != sys::CNA_FALSE,
            rumble_supported: value.rumble_supported != sys::CNA_FALSE,
        }
    }
}

fn view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast::<core::ffi::c_char>(),
        byte_length: value.len() as u64,
    }
}

/// One enumerated haptic device, before it is opened.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HapticInfo {
    /// The device identifier, which is what [`open`] takes.
    ///
    /// The index it was enumerated at is not a durable reference, the same way
    /// it is not for input devices; the identifier is.
    pub id: u32,
    pub name: String,
}

/// How many haptic devices the host reports.
///
/// Zero is an ordinary answer, and the usual one on a machine with no wheel or
/// force-feedback pad attached.
pub fn count(game: &GameContext<'_>) -> Result<u32> {
    let (native, handle) = game.native_game();
    let mut value = 0_u32;
    // SAFETY: the game handle is live and the output is a live local.
    native.check(unsafe { (native.runtime.haptics_get_count)(handle, &mut value) })?;
    Ok(value)
}

/// Every haptic device, as one snapshot.
pub fn enumerate(game: &GameContext<'_>) -> Result<Vec<HapticInfo>> {
    let total = count(game)?;
    let (native, handle) = game.native_game();
    let api = &native.runtime;
    let mut devices = Vec::with_capacity(total as usize);
    for index in 0..total {
        let mut id = 0_u32;
        // SAFETY: the index is below the count this call just read.
        native.check(unsafe { (api.haptics_get_id_at)(handle, index, &mut id) })?;
        let name = read_string(
            |value| native.check(value),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.haptics_get_name_size_at)(handle, index, bytes) },
            |destination, capacity, written| unsafe {
                (api.haptics_copy_name_at)(handle, index, destination, capacity, written)
            },
        )?;
        devices.push(HapticInfo { id, name });
    }
    Ok(devices)
}

/// Whether a joystick has force feedback.
pub fn joystick_is_haptic(game: &GameContext<'_>, joystick_id: u32) -> Result<bool> {
    let (native, handle) = game.native_game();
    let mut value = sys::CNA_FALSE;
    // SAFETY: the output is a live local of the declared type.
    native.check(unsafe {
        (native.runtime.haptics_get_is_joystick_haptic)(handle, joystick_id, &mut value)
    })?;
    Ok(value != sys::CNA_FALSE)
}

/// Whether the mouse has force feedback.
pub fn mouse_is_haptic(game: &GameContext<'_>) -> Result<bool> {
    let (native, handle) = game.native_game();
    let mut value = sys::CNA_FALSE;
    // SAFETY: the output is a live local of the declared type.
    native.check(unsafe { (native.runtime.haptics_get_is_mouse_haptic)(handle, &mut value) })?;
    Ok(value != sys::CNA_FALSE)
}

/// An opened haptic device this value owns.
#[derive(Debug)]
pub struct HapticDevice {
    native: Arc<Native>,
    handle: sys::CNA_HapticDeviceHandle,
}

/// Whether an operation the device may not support actually took effect.
///
/// CNA answers this separately from success, and the distinction is the whole
/// point: setting a gain on a device with no gain control is not an error, but
/// it also did not happen, and a caller that treated success as "it worked"
/// would show a slider that does nothing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct Applied(pub bool);

impl HapticDevice {
    /// Opens a device by identifier.
    pub fn open(game: &GameContext<'_>, id: u32) -> Result<Self> {
        let (native, handle) = game.native_game();
        let mut device = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local receiving a new owned handle.
        native.check(unsafe { (native.runtime.haptics_open)(handle, id, &mut device) })?;
        Ok(Self {
            native: Arc::clone(native),
            handle: device,
        })
    }

    /// Opens the haptic side of a joystick.
    pub fn open_from_joystick(game: &GameContext<'_>, joystick_id: u32) -> Result<Self> {
        let (native, handle) = game.native_game();
        let mut device = sys::CNA_INVALID_HANDLE;
        // SAFETY: as above.
        native.check(unsafe {
            (native.runtime.haptics_open_from_joystick)(handle, joystick_id, &mut device)
        })?;
        Ok(Self {
            native: Arc::clone(native),
            handle: device,
        })
    }

    /// Opens the haptic side of the mouse.
    pub fn open_from_mouse(game: &GameContext<'_>) -> Result<Self> {
        let (native, handle) = game.native_game();
        let mut device = sys::CNA_INVALID_HANDLE;
        // SAFETY: as above.
        native.check(unsafe { (native.runtime.haptics_open_from_mouse)(handle, &mut device) })?;
        Ok(Self {
            native: Arc::clone(native),
            handle: device,
        })
    }

    /// Whether the device is still open.
    pub fn is_open(&self) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.runtime.haptic_device_get_is_open)(self.handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// The device's name.
    pub fn name(&self) -> Result<String> {
        let native = &self.native;
        let api = &native.runtime;
        read_string(
            |value| native.check(value),
            // SAFETY: CNA's canonical size-then-copy pair for one string.
            |bytes| unsafe { (api.haptic_device_get_name_size)(self.handle, bytes) },
            |destination, capacity, written| unsafe {
                (api.haptic_device_copy_name)(self.handle, destination, capacity, written)
            },
        )
    }

    /// What the device can do.
    pub fn capabilities(&self) -> Result<HapticCapabilities> {
        let mut value = sys::CNA_HapticCapabilities {
            struct_size: core::mem::size_of::<sys::CNA_HapticCapabilities>() as u32,
            struct_version: 1,
            ..sys::CNA_HapticCapabilities::default()
        };
        // SAFETY: the structure is a caller-owned versioned output.
        self.native.check(unsafe {
            (self.native.runtime.haptic_device_get_capabilities)(self.handle, &mut value)
        })?;
        Ok(HapticCapabilities::from_native(value))
    }

    /// Prepares the simple rumble path -- the one XNA's `SetVibration` uses.
    pub fn init_rumble(&self) -> Result<Applied> {
        self.applied(|api, handle, out| unsafe { (api.haptic_device_init_rumble)(handle, out) })
    }

    /// Plays a rumble at `strength` for `length_ms`.
    pub fn play_rumble(&self, strength: f32, length_ms: u32) -> Result<Applied> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned, the scalars are by value, and the
        // output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.haptic_device_play_rumble)(
                self.handle,
                strength,
                length_ms,
                &mut value,
            )
        })?;
        Ok(Applied(value != sys::CNA_FALSE))
    }

    /// Stops a rumble.
    pub fn stop_rumble(&self) -> Result<Applied> {
        self.applied(|api, handle, out| unsafe { (api.haptic_device_stop_rumble)(handle, out) })
    }

    /// Sets the device's global output gain, where the device has one.
    pub fn set_gain(&self, gain: i32) -> Result<Applied> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.haptic_device_set_gain)(self.handle, gain, &mut value)
        })?;
        Ok(Applied(value != sys::CNA_FALSE))
    }

    /// Sets the device's self-centring force, where the device has one.
    pub fn set_autocenter(&self, autocenter: i32) -> Result<Applied> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.haptic_device_set_autocenter)(self.handle, autocenter, &mut value)
        })?;
        Ok(Applied(value != sys::CNA_FALSE))
    }

    /// Pauses playback.
    pub fn pause(&self) -> Result<Applied> {
        self.applied(|api, handle, out| unsafe { (api.haptic_device_pause)(handle, out) })
    }

    /// Resumes playback.
    pub fn resume(&self) -> Result<Applied> {
        self.applied(|api, handle, out| unsafe { (api.haptic_device_resume)(handle, out) })
    }

    /// Stops every effect the device is playing.
    pub fn stop_all_effects(&self) -> Result<Applied> {
        self.applied(|api, handle, out| unsafe {
            (api.haptic_device_stop_all_effects)(handle, out)
        })
    }

    /// Closes the device without dropping this value.
    pub fn dispose(&self) -> Result<()> {
        // SAFETY: the handle is owned by this value.
        self.native
            .check(unsafe { (self.native.runtime.haptic_device_dispose)(self.handle) })
    }

    fn applied(
        &self,
        call: impl FnOnce(
            &crate::native::runtime::RuntimeApi,
            sys::CNA_HapticDeviceHandle,
            *mut sys::CNA_Bool,
        ) -> sys::CNA_Result,
    ) -> Result<Applied> {
        let mut value = sys::CNA_FALSE;
        self.native
            .check(call(&self.native.runtime, self.handle, &mut value))?;
        Ok(Applied(value != sys::CNA_FALSE))
    }
}

impl Drop for HapticDevice {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        let _ = unsafe { (self.native.runtime.haptic_device_destroy)(self.handle) };
    }
}
