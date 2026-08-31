//! CNA's motion sensors: accelerometer, compass and gyroscope.
//!
//! XNA had these only on Windows Phone, in `Microsoft.Devices.Sensors`, which
//! is not one of the ten runtime assemblies this binding projects. CNA exposes
//! them on every host that has them, so they live here.
//!
//! The point of this module is that **absence is an answer**. A desktop machine
//! has no accelerometer, and the honest report of that is
//! [`SensorState::NotSupported`] with no reading at all -- not a reading of
//! `Vector3::ZERO`, which is indistinguishable from a device lying perfectly
//! flat in free fall. Every accessor here returns a state or a `Result`, and
//! `current_value` returns `None` rather than a zero when the sensor has no
//! data.
//!
//! Units and frames are CNA's, and are stated rather than converted: the
//! accelerometer reports **g**, the gyroscope **radians per second**, and the
//! compass **degrees** for its headings and **micro-teslas** for the raw
//! magnetometer axes.

#![allow(clippy::missing_errors_doc)]

use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::game::GameContext;
use crate::native::runtime::read_string;
use crate::native::Native;
use crate::game::TimeSpan;
use crate::value::Vector3;

/// What kind of sensor a descriptor names.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum SensorKind {
    Unknown,
    Accelerometer,
    Gyroscope,
    /// A left-hand controller's accelerometer, on hardware that has a pair.
    AccelerometerLeft,
    GyroscopeLeft,
    AccelerometerRight,
    GyroscopeRight,
}

impl SensorKind {
    const fn from_native(value: sys::CNA_SensorType) -> Option<Self> {
        Some(match value {
            sys::CNA_SENSOR_TYPE_UNKNOWN => Self::Unknown,
            sys::CNA_SENSOR_TYPE_ACCELEROMETER => Self::Accelerometer,
            sys::CNA_SENSOR_TYPE_GYROSCOPE => Self::Gyroscope,
            sys::CNA_SENSOR_TYPE_ACCELEROMETER_LEFT => Self::AccelerometerLeft,
            sys::CNA_SENSOR_TYPE_GYROSCOPE_LEFT => Self::GyroscopeLeft,
            sys::CNA_SENSOR_TYPE_ACCELEROMETER_RIGHT => Self::AccelerometerRight,
            sys::CNA_SENSOR_TYPE_GYROSCOPE_RIGHT => Self::GyroscopeRight,
            _ => return None,
        })
    }
}

/// Why a sensor is or is not producing readings.
///
/// The distinctions matter: a game should tell a user to grant a permission,
/// wait for initialisation, or stop asking entirely, and one boolean cannot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum SensorState {
    /// The host has no such sensor. This is the ordinary desktop answer.
    NotSupported,
    Ready,
    Initializing,
    /// Present and started, but with nothing to report yet.
    NoData,
    /// Present, but the application may not read it.
    NoPermissions,
    Disabled,
}

impl SensorState {
    const fn from_native(value: sys::CNA_SensorState) -> Option<Self> {
        Some(match value {
            sys::CNA_SENSOR_STATE_NOT_SUPPORTED => Self::NotSupported,
            sys::CNA_SENSOR_STATE_READY => Self::Ready,
            sys::CNA_SENSOR_STATE_INITIALIZING => Self::Initializing,
            sys::CNA_SENSOR_STATE_NO_DATA => Self::NoData,
            sys::CNA_SENSOR_STATE_NO_PERMISSIONS => Self::NoPermissions,
            sys::CNA_SENSOR_STATE_DISABLED => Self::Disabled,
            _ => return None,
        })
    }
}

/// One enumerated sensor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SensorInfo {
    pub id: u32,
    pub kind: SensorKind,
    pub name: String,
}

/// When a reading was taken, in CNA's own calendar terms.
///
/// Kept as the two tick counts the container carries rather than converted to
/// a wall-clock type: the offset is part of the timestamp, and folding it away
/// would lose which zone the device recorded.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SensorTimestamp {
    /// Local time in 100-nanosecond ticks since 0001-01-01.
    pub ticks: i64,
    /// Offset from UTC in 100-nanosecond ticks.
    pub offset_ticks: i64,
}

impl SensorTimestamp {
    const fn from_native(value: sys::CNA_DateTimeOffset) -> Self {
        Self {
            ticks: value.ticks,
            offset_ticks: value.offset_ticks,
        }
    }
}

/// One accelerometer reading. Acceleration is in **g**, per axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccelerometerReading {
    pub timestamp: SensorTimestamp,
    pub acceleration: Vector3,
}

/// One compass reading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompassReading {
    pub timestamp: SensorTimestamp,
    /// Accuracy of the heading, in degrees.
    pub heading_accuracy: f64,
    /// Heading relative to magnetic north, in degrees.
    pub magnetic_heading: f64,
    /// Heading relative to true north, in degrees.
    pub true_heading: f64,
    /// Raw magnetometer axes, in micro-teslas.
    pub magnetometer_reading: Vector3,
}

/// One gyroscope reading. Rotation is in **radians per second**, per axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GyroscopeReading {
    pub timestamp: SensorTimestamp,
    pub rotation_rate: Vector3,
}

const fn vector(value: sys::CNA_Vector3) -> Vector3 {
    Vector3 {
        X: value.x,
        Y: value.y,
        Z: value.z,
    }
}

/// How many motion sensors the host currently reports.
///
/// Zero is an ordinary answer, and the usual one on a desktop machine.
pub fn count(game: &GameContext<'_>) -> Result<u32> {
    let (native, handle) = game.native_game();
    let mut value = 0_u32;
    // SAFETY: the game handle is live and the output is a live local.
    native.check(unsafe { (native.runtime.sensors_get_count)(handle, &mut value) })?;
    Ok(value)
}

/// Every enumerated sensor, as one snapshot.
///
/// Upstream states the enumeration is a point-in-time snapshot whose indices
/// are valid only until the sensor set changes, so no index escapes this call.
pub fn enumerate(game: &GameContext<'_>) -> Result<Vec<SensorInfo>> {
    let total = count(game)?;
    let (native, handle) = game.native_game();
    let api = &native.runtime;
    let mut sensors = Vec::with_capacity(total as usize);
    for index in 0..total {
        let mut info = sys::CNA_SensorInfo {
            struct_size: core::mem::size_of::<sys::CNA_SensorInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_SensorInfo::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output and the
        // index is below the count this call just read.
        native.check(unsafe { (api.sensors_get_info_at)(handle, index, &mut info) })?;
        let kind = SensorKind::from_native(info.r#type).ok_or(CnaError::UnsupportedRuntime(
            "CNA reported a sensor kind this build does not know",
        ))?;
        let name = read_string(
            |value| native.check(value),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.sensors_get_name_size_at)(handle, index, bytes) },
            |destination, capacity, written| unsafe {
                (api.sensors_copy_name_at)(handle, index, destination, capacity, written)
            },
        )?;
        sensors.push(SensorInfo {
            id: info.id,
            kind,
            name,
        });
    }
    Ok(sensors)
}

/// Declares one sensor family over its identical native shape.
macro_rules! sensor {
    (
        $(#[$meta:meta])*
        $name:ident, $handle:ty, $reading:ty, $native_reading:ty,
        create = $create:ident, destroy = $destroy:ident, dispose = $dispose:ident,
        supported = $supported:ident, state = $state:ident, valid = $valid:ident,
        value = $value:ident, get_interval = $get_interval:ident,
        set_interval = $set_interval:ident, start = $start:ident, stop = $stop:ident,
        convert = $convert:expr,
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name {
            native: Arc<Native>,
            handle: $handle,
        }

        impl $name {
            /// Whether this host has such a sensor at all.
            ///
            /// Answered without constructing one, because the answer on most
            /// desktops is "no" and constructing a sensor to find that out
            /// would be the wrong shape.
            pub fn is_supported(game: &GameContext<'_>) -> Result<bool> {
                let (native, handle) = game.native_game();
                let mut value = sys::CNA_FALSE;
                // SAFETY: the output is a live local of the declared type.
                native.check(unsafe { (native.runtime.$supported)(handle, &mut value) })?;
                Ok(value != sys::CNA_FALSE)
            }

            /// Constructs a sensor object.
            ///
            /// This succeeds on a host with no such sensor: the object exists
            /// and reports [`SensorState::NotSupported`], which is what lets a
            /// game ask *why* there are no readings rather than only that
            /// there are none.
            pub fn new(game: &GameContext<'_>) -> Result<Self> {
                let (native, handle) = game.native_game();
                let mut sensor = sys::CNA_INVALID_HANDLE;
                // SAFETY: the output is a live local receiving a new handle.
                native.check(unsafe { (native.runtime.$create)(handle, &mut sensor) })?;
                Ok(Self {
                    native: Arc::clone(native),
                    handle: sensor,
                })
            }

            /// Why the sensor is or is not producing readings.
            pub fn state(&self) -> Result<SensorState> {
                let mut value = 0;
                // SAFETY: the handle is owned and the output is a live local.
                self.native
                    .check(unsafe { (self.native.runtime.$state)(self.handle, &mut value) })?;
                SensorState::from_native(value).ok_or(CnaError::UnsupportedRuntime(
                    "CNA reported a sensor state this build does not know",
                ))
            }

            /// Whether the last reading is meaningful.
            pub fn is_data_valid(&self) -> Result<bool> {
                let mut value = sys::CNA_FALSE;
                // SAFETY: the handle is owned and the output is a live local.
                self.native
                    .check(unsafe { (self.native.runtime.$valid)(self.handle, &mut value) })?;
                Ok(value != sys::CNA_FALSE)
            }

            /// The current reading, or `None` when there is not one.
            ///
            /// `None` rather than a zeroed reading: an accelerometer reporting
            /// `(0, 0, 0)` is a device in free fall, which is a real and very
            /// different thing from a device that has no accelerometer.
            pub fn current_value(&self) -> Result<Option<$reading>> {
                if !self.is_data_valid()? {
                    return Ok(None);
                }
                let mut reading = <$native_reading>::default();
                reading.struct_size = core::mem::size_of::<$native_reading>() as u32;
                reading.struct_version = 1;
                // SAFETY: the reading is a caller-owned versioned output.
                self.native
                    .check(unsafe { (self.native.runtime.$value)(self.handle, &mut reading) })?;
                #[allow(clippy::redundant_closure_call)]
                Ok(Some($convert(reading)))
            }

            /// How often the sensor is asked for a new reading.
            pub fn time_between_updates(&self) -> Result<TimeSpan> {
                let mut ticks = 0_i64;
                // SAFETY: the handle is owned and the output is a live local.
                self.native.check(unsafe {
                    (self.native.runtime.$get_interval)(self.handle, &mut ticks)
                })?;
                Ok(TimeSpan::from_ticks(ticks))
            }

            /// Requests a sampling interval.
            pub fn set_time_between_updates(&self, value: TimeSpan) -> Result<()> {
                // SAFETY: the handle is owned and the interval is by value.
                self.native.check(unsafe {
                    (self.native.runtime.$set_interval)(self.handle, value.Ticks())
                })
            }

            /// Begins sampling.
            pub fn start(&self) -> Result<()> {
                // SAFETY: the handle is owned by this value.
                self.native
                    .check(unsafe { (self.native.runtime.$start)(self.handle) })
            }

            /// Stops sampling.
            pub fn stop(&self) -> Result<()> {
                // SAFETY: the handle is owned by this value.
                self.native
                    .check(unsafe { (self.native.runtime.$stop)(self.handle) })
            }

            /// Releases the sensor without dropping this value.
            pub fn dispose(&self) -> Result<()> {
                // SAFETY: the handle is owned by this value.
                self.native
                    .check(unsafe { (self.native.runtime.$dispose)(self.handle) })
            }

        }

        impl Drop for $name {
            fn drop(&mut self) {
                // SAFETY: the handle is owned by this value and released once.
                let _ = unsafe { (self.native.runtime.$destroy)(self.handle) };
            }
        }
    };
}

sensor! {
    /// The host's accelerometer. Readings are in **g**.
    Accelerometer, sys::CNA_AccelerometerHandle, AccelerometerReading,
    sys::CNA_AccelerometerReading,
    create = accelerometer_create, destroy = accelerometer_destroy,
    dispose = accelerometer_dispose, supported = accelerometer_get_is_supported,
    state = accelerometer_get_state, valid = accelerometer_get_is_data_valid,
    value = accelerometer_get_current_value,
    get_interval = accelerometer_get_time_between_updates_ticks,
    set_interval = accelerometer_set_time_between_updates_ticks,
    start = accelerometer_start, stop = accelerometer_stop,
    convert = |reading: sys::CNA_AccelerometerReading| AccelerometerReading {
        timestamp: SensorTimestamp::from_native(reading.timestamp),
        acceleration: vector(reading.acceleration),
    },
}

sensor! {
    /// The host's compass. Headings are in **degrees**, axes in micro-teslas.
    Compass, sys::CNA_CompassHandle, CompassReading, sys::CNA_CompassReading,
    create = compass_create, destroy = compass_destroy, dispose = compass_dispose,
    supported = compass_get_is_supported, state = compass_get_state,
    valid = compass_get_is_data_valid, value = compass_get_current_value,
    get_interval = compass_get_time_between_updates_ticks,
    set_interval = compass_set_time_between_updates_ticks,
    start = compass_start, stop = compass_stop,
    convert = |reading: sys::CNA_CompassReading| CompassReading {
        timestamp: SensorTimestamp::from_native(reading.timestamp),
        heading_accuracy: reading.heading_accuracy,
        magnetic_heading: reading.magnetic_heading,
        true_heading: reading.true_heading,
        magnetometer_reading: vector(reading.magnetometer_reading),
    },
}

sensor! {
    /// The host's gyroscope. Readings are in **radians per second**.
    Gyroscope, sys::CNA_GyroscopeHandle, GyroscopeReading, sys::CNA_GyroscopeReading,
    create = gyroscope_create, destroy = gyroscope_destroy, dispose = gyroscope_dispose,
    supported = gyroscope_get_is_supported, state = gyroscope_get_state,
    valid = gyroscope_get_is_data_valid, value = gyroscope_get_current_value,
    get_interval = gyroscope_get_time_between_updates_ticks,
    set_interval = gyroscope_set_time_between_updates_ticks,
    start = gyroscope_start, stop = gyroscope_stop,
    convert = |reading: sys::CNA_GyroscopeReading| GyroscopeReading {
        timestamp: SensorTimestamp::from_native(reading.timestamp),
        rotation_rate: vector(reading.rotation_rate),
    },
}

/// Injecting a reading, as the hardware would.
///
/// CNA provides these so a game's sensor handling can be tested on a machine
/// that has no such sensor. Each is a real delivery through the same path, not
/// a shortcut around it, and none makes `is_supported` start answering `true`:
/// injecting a reading does not conjure a device.
///
/// They are written out per family rather than folded into the shared macro
/// because they genuinely differ -- the accelerometer and gyroscope take three
/// axes, and the compass takes a whole reading, because a compass reading is
/// more than three numbers.
impl Accelerometer {
    /// Injects an acceleration in **g**, per axis.
    pub fn inject(&self, x: f32, y: f32, z: f32) -> Result<()> {
        // SAFETY: the handle is owned and the axes are by value.
        self.native.check(unsafe {
            (self.native.runtime.accelerometer_inject_synthetic_update)(self.handle, x, y, z)
        })
    }
}

impl Gyroscope {
    /// Injects an angular velocity in **radians per second**, per axis.
    pub fn inject(&self, x: f32, y: f32, z: f32) -> Result<()> {
        // SAFETY: the handle is owned and the axes are by value.
        self.native.check(unsafe {
            (self.native.runtime.gyroscope_inject_synthetic_update)(self.handle, x, y, z)
        })
    }
}

impl Compass {
    /// Injects a whole compass reading.
    pub fn inject(&self, reading: CompassReading) -> Result<()> {
        let native_reading = sys::CNA_CompassReading {
            struct_size: core::mem::size_of::<sys::CNA_CompassReading>() as u32,
            struct_version: 1,
            timestamp: sys::CNA_DateTimeOffset {
                ticks: reading.timestamp.ticks,
                offset_ticks: reading.timestamp.offset_ticks,
            },
            heading_accuracy: reading.heading_accuracy,
            magnetic_heading: reading.magnetic_heading,
            true_heading: reading.true_heading,
            magnetometer_reading: sys::CNA_Vector3 {
                x: reading.magnetometer_reading.X,
                y: reading.magnetometer_reading.Y,
                z: reading.magnetometer_reading.Z,
            },
        };
        // SAFETY: the reading is a live local CNA copies during the call.
        self.native.check(unsafe {
            (self.native.runtime.compass_inject_synthetic_update)(self.handle, &native_reading)
        })
    }
}
