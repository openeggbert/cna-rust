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

use crate::error::{CnaError, Result};
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
    /// The device identifier, which is what [`HapticDevice::open`] takes.
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

/// Which family a force-feedback effect belongs to.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum HapticEffectType {
    Constant,
    Sine,
    Square,
    Triangle,
    SawtoothUp,
    SawtoothDown,
    Ramp,
    Spring,
    Damper,
    Inertia,
    Friction,
    /// Two-motor rumble: the whole of what XNA could express.
    LeftRight,
    Custom,
}

/// How a force's direction is expressed.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum HapticDirectionType {
    Polar,
    Cartesian,
    Spherical,
    SteeringAxis,
}

macro_rules! haptic_identity {
    ($name:ident, $native:ty, $($variant:ident => $constant:ident),+ $(,)?) => {
        impl $name {
            const fn from_native(value: $native) -> Option<Self> {
                Some(match value {
                    $(sys::$constant => Self::$variant,)+
                    _ => return None,
                })
            }

            const fn to_native(self) -> $native {
                match self {
                    $(Self::$variant => sys::$constant,)+
                }
            }
        }
    };
}

haptic_identity!(
    HapticEffectType, sys::CNA_HapticEffectType,
    Constant => CNA_HAPTIC_EFFECT_TYPE_CONSTANT,
    Sine => CNA_HAPTIC_EFFECT_TYPE_SINE,
    Square => CNA_HAPTIC_EFFECT_TYPE_SQUARE,
    Triangle => CNA_HAPTIC_EFFECT_TYPE_TRIANGLE,
    SawtoothUp => CNA_HAPTIC_EFFECT_TYPE_SAWTOOTH_UP,
    SawtoothDown => CNA_HAPTIC_EFFECT_TYPE_SAWTOOTH_DOWN,
    Ramp => CNA_HAPTIC_EFFECT_TYPE_RAMP,
    Spring => CNA_HAPTIC_EFFECT_TYPE_SPRING,
    Damper => CNA_HAPTIC_EFFECT_TYPE_DAMPER,
    Inertia => CNA_HAPTIC_EFFECT_TYPE_INERTIA,
    Friction => CNA_HAPTIC_EFFECT_TYPE_FRICTION,
    LeftRight => CNA_HAPTIC_EFFECT_TYPE_LEFT_RIGHT,
    Custom => CNA_HAPTIC_EFFECT_TYPE_CUSTOM,
);

haptic_identity!(
    HapticDirectionType, sys::CNA_HapticDirectionType,
    Polar => CNA_HAPTIC_DIRECTION_TYPE_POLAR,
    Cartesian => CNA_HAPTIC_DIRECTION_TYPE_CARTESIAN,
    Spherical => CNA_HAPTIC_DIRECTION_TYPE_SPHERICAL,
    SteeringAxis => CNA_HAPTIC_DIRECTION_TYPE_STEERING_AXIS,
);

/// A force's direction, in whichever coordinate space it names.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HapticDirection {
    pub kind: HapticDirectionType,
    /// Up to three components; how many are meaningful depends on `kind`.
    pub values: [i32; 3],
}

impl HapticDirection {
    /// CNA's default direction.
    pub fn canonical_defaults() -> Result<Self> {
        let native = Native::process()?;
        let mut value = sys::CNA_HapticDirection::default();
        // SAFETY: the structure is a caller-owned output CNA fills.
        native.check(unsafe { (native.runtime.haptic_direction_init)(&mut value) })?;
        HapticDirectionType::from_native(value.r#type)
            .map(|kind| Self {
                kind,
                values: value.values,
            })
            .ok_or(CnaError::UnsupportedRuntime(
                "CNA named a haptic direction kind this build does not know",
            ))
    }

    /// Whether CNA considers this the same direction as `other`.
    ///
    /// Asks CNA rather than deriving `==`, for the same reason the device and
    /// capability comparisons do: CNA decides which of the three components
    /// matter for a given coordinate space, and a field-by-field Rust
    /// comparison would call two equivalent directions different.
    pub fn same_direction(&self, other: &Self) -> Result<bool> {
        let native = Native::process()?;
        let left = self.to_native();
        let right = other.to_native();
        let mut equal = sys::CNA_FALSE;
        // SAFETY: both descriptors are live locals and the output is one too.
        native.check(unsafe {
            (native.runtime.haptic_direction_equals)(&left, &right, &mut equal)
        })?;
        Ok(equal != sys::CNA_FALSE)
    }

    const fn to_native(self) -> sys::CNA_HapticDirection {
        sys::CNA_HapticDirection {
            r#type: self.kind.to_native(),
            values: self.values,
        }
    }
}

/// One force-feedback effect.
///
/// Thirty-one fields, most of which are meaningful only for some effect
/// families -- `ramp_start` for a ramp, the three-axis condition arrays for a
/// spring or damper, `large_magnitude` and `small_magnitude` for two-motor
/// rumble. That is why this is an owned value with accessors rather than a
/// public structure: a Rust type asserting which fields apply to which family
/// would be a taxonomy this crate invented, and there is no hardware here to
/// check it against. CNA's own `is_supported` is the authority instead.
#[derive(Clone, Debug)]
pub struct HapticEffect {
    inner: sys::CNA_HapticEffect,
    custom: Vec<u16>,
}

impl HapticEffect {
    /// CNA's defaults for an effect of one family.
    pub fn new(kind: HapticEffectType) -> Result<Self> {
        let native = Native::process()?;
        let mut inner = sys::CNA_HapticEffect {
            struct_size: core::mem::size_of::<sys::CNA_HapticEffect>() as u32,
            struct_version: 1,
            ..sys::CNA_HapticEffect::default()
        };
        // SAFETY: the structure is a caller-owned versioned output.
        native.check(unsafe { (native.runtime.haptic_effect_init)(&mut inner) })?;
        inner.r#type = kind.to_native();
        Ok(Self {
            inner,
            custom: Vec::new(),
        })
    }

    /// The effect's family.
    pub fn kind(&self) -> Result<HapticEffectType> {
        HapticEffectType::from_native(self.inner.r#type).ok_or(CnaError::UnsupportedRuntime(
            "CNA named a haptic effect kind this build does not know",
        ))
    }

    /// The direction the force is applied in.
    pub fn direction(&self) -> Result<HapticDirection> {
        HapticDirectionType::from_native(self.inner.direction.r#type)
            .map(|kind| HapticDirection {
                kind,
                values: self.inner.direction.values,
            })
            .ok_or(CnaError::UnsupportedRuntime(
                "CNA named a haptic direction kind this build does not know",
            ))
    }

    /// Sets the direction the force is applied in.
    pub fn set_direction(&mut self, value: HapticDirection) -> &mut Self {
        self.inner.direction = value.to_native();
        self
    }

    /// How long the effect runs, in milliseconds.
    #[must_use]
    pub const fn length(&self) -> u32 {
        self.inner.length
    }

    /// Sets how long the effect runs.
    pub fn set_length(&mut self, milliseconds: u32) -> &mut Self {
        self.inner.length = milliseconds;
        self
    }

    /// The effect's magnitude.
    #[must_use]
    pub const fn magnitude(&self) -> i16 {
        self.inner.magnitude
    }

    /// Sets the effect's magnitude.
    pub fn set_magnitude(&mut self, value: i16) -> &mut Self {
        self.inner.magnitude = value;
        self
    }

    /// The two motor amplitudes a `LeftRight` effect uses.
    ///
    /// This is the pair XNA's `GamePad.SetVibration` exposes, and it is one
    /// effect family out of thirteen here rather than the whole vocabulary.
    #[must_use]
    pub const fn rumble_magnitudes(&self) -> (u16, u16) {
        (self.inner.large_magnitude, self.inner.small_magnitude)
    }

    /// Sets the two motor amplitudes a `LeftRight` effect uses.
    pub fn set_rumble_magnitudes(&mut self, large: u16, small: u16) -> &mut Self {
        self.inner.large_magnitude = large;
        self.inner.small_magnitude = small;
        self
    }

    /// The samples a `Custom` effect plays.
    #[must_use]
    pub fn custom_samples(&self) -> &[u16] {
        &self.custom
    }

    /// Sets the samples a `Custom` effect plays.
    ///
    /// The samples are copied into this value, so nothing borrows the caller's
    /// buffer past the call that stores it.
    pub fn set_custom_samples(&mut self, samples: &[u16]) -> &mut Self {
        self.custom = samples.to_vec();
        self
    }

    /// Whether CNA considers this the same effect as `other`.
    ///
    /// Asks CNA, which compares the custom sample data alongside the fields.
    pub fn same_effect(&self, other: &Self) -> Result<bool> {
        let native = Native::process()?;
        let mut equal = sys::CNA_FALSE;
        // SAFETY: both descriptors and both sample buffers are live for the
        // duration of the call, with their own lengths.
        native.check(unsafe {
            (native.runtime.haptic_effect_equals)(
                &self.inner,
                self.samples_pointer(),
                self.custom.len() as u64,
                &other.inner,
                other.samples_pointer(),
                other.custom.len() as u64,
                &mut equal,
            )
        })?;
        Ok(equal != sys::CNA_FALSE)
    }

    fn samples_pointer(&self) -> *const u16 {
        if self.custom.is_empty() {
            core::ptr::null()
        } else {
            self.custom.as_ptr()
        }
    }
}

/// A created effect, identified on the device that holds it.
///
/// Not an owning handle: the device owns the effect, and destroying the device
/// takes its effects with it. Releasing one early is [`HapticDevice::destroy_effect`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HapticEffectId(i32);

impl HapticEffectId {
    /// The identifier CNA assigned.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl HapticDevice {
    /// Whether the device can play an effect as described.
    ///
    /// CNA is the authority on which fields a family uses, so this is how a
    /// caller finds out rather than a Rust taxonomy asserting it.
    pub fn is_effect_supported(&self, effect: &HapticEffect) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the descriptor and samples are live for the call.
        self.native.check(unsafe {
            (self.native.runtime.haptic_device_get_is_effect_supported)(
                self.handle,
                &effect.inner,
                effect.samples_pointer(),
                effect.custom.len() as u64,
                &mut value,
            )
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Uploads an effect to the device.
    pub fn create_effect(&self, effect: &HapticEffect) -> Result<HapticEffectId> {
        let mut id = 0_i32;
        // SAFETY: the descriptor and samples are live for the call, and the
        // output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.haptic_device_create_effect)(
                self.handle,
                &effect.inner,
                effect.samples_pointer(),
                effect.custom.len() as u64,
                &mut id,
            )
        })?;
        Ok(HapticEffectId(id))
    }

    /// Replaces an uploaded effect's description.
    pub fn update_effect(&self, id: HapticEffectId, effect: &HapticEffect) -> Result<Applied> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: as for `create_effect`.
        self.native.check(unsafe {
            (self.native.runtime.haptic_device_update_effect)(
                self.handle,
                id.0,
                &effect.inner,
                effect.samples_pointer(),
                effect.custom.len() as u64,
                &mut value,
            )
        })?;
        Ok(Applied(value != sys::CNA_FALSE))
    }

    /// Plays an uploaded effect `iterations` times.
    pub fn run_effect(&self, id: HapticEffectId, iterations: u32) -> Result<Applied> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.haptic_device_run_effect)(self.handle, id.0, iterations, &mut value)
        })?;
        Ok(Applied(value != sys::CNA_FALSE))
    }

    /// Stops an uploaded effect.
    pub fn stop_effect(&self, id: HapticEffectId) -> Result<Applied> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.haptic_device_stop_effect)(self.handle, id.0, &mut value)
        })?;
        Ok(Applied(value != sys::CNA_FALSE))
    }

    /// Whether an uploaded effect is currently playing.
    pub fn effect_is_playing(&self, id: HapticEffectId) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.haptic_device_get_effect_status)(self.handle, id.0, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Releases an uploaded effect.
    pub fn destroy_effect(&self, id: HapticEffectId) -> Result<()> {
        // SAFETY: the handle is owned and the identifier is by value.
        self.native
            .check(unsafe { (self.native.runtime.haptic_device_destroy_effect)(self.handle, id.0) })
    }
}
