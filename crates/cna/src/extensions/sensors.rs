//! CNA's motion sensors: accelerometer, compass, gyroscope and motion.
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
//!
//! ## Testing without hardware
//!
//! None of these sensors exists on the machines this crate is verified on, so
//! every one of them ships a deterministic backend and this module binds it.
//! The compass and the motion sensor take a whole substitute backend
//! ([`Compass::install_test_backend`]); the accelerometer and the gyroscope
//! have no such route and are steered field by field instead
//! ([`Accelerometer::force_supported_for_tests`] and its neighbours). The
//! names say `for_tests` because that is what upstream calls them and because
//! a game must not reach for them: they move CNA's own state, not the
//! device's.

#![allow(non_snake_case, clippy::missing_errors_doc)]

use core::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::game::GameContext;
use crate::native::runtime::read_string;
use crate::native::Native;
use crate::game::TimeSpan;
use crate::value::{Matrix, Quaternion, Vector3};

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

impl AccelerometerReading {
    /// XNA's `ToString()` for this reading.
    #[must_use]
    pub fn ToString(&self) -> String {
        format!("Acceleration:{}", self.acceleration.ToString())
    }
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

impl CompassReading {
    /// XNA's `ToString()` for this reading.
    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "MagneticHeading:{} TrueHeading:{} HeadingAccuracy:{} MagnetometerReading:{}",
            self.magnetic_heading,
            self.true_heading,
            self.heading_accuracy,
            self.magnetometer_reading.ToString()
        )
    }
}

/// One fused-orientation reading, in three equivalent forms.
///
/// Pitch, roll and yaw are in **radians**. The quaternion and the rotation
/// matrix describe the same orientation; upstream computes all three, so all
/// three are carried rather than one being recomputed here and disagreeing in
/// the last bits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttitudeReading {
    pub timestamp: SensorTimestamp,
    /// Rotation around the X axis, in radians.
    pub pitch: f32,
    /// Rotation around the Y axis, in radians.
    pub roll: f32,
    /// Rotation around the Z axis, in radians.
    pub yaw: f32,
    /// The same orientation as a quaternion.
    pub quaternion: Quaternion,
    /// The same orientation as a rotation matrix.
    pub rotation_matrix: Matrix,
}

impl AttitudeReading {
    /// XNA's `ToString()` for this reading.
    #[must_use]
    pub fn ToString(&self) -> String {
        format!("Pitch:{} Roll:{} Yaw:{}", self.pitch, self.roll, self.yaw)
    }
}

/// One motion reading: orientation, gravity, and acceleration without it.
///
/// The motion sensor is a *fusion* of the others rather than a fourth device.
/// That is why it reports acceleration twice over -- `device_acceleration`
/// excludes gravity and `gravity` is the part that was taken out -- and why it
/// can answer an orientation at all.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionReading {
    pub timestamp: SensorTimestamp,
    /// The fused device orientation.
    pub attitude: AttitudeReading,
    /// Acceleration excluding gravity, in **g**, per axis.
    pub device_acceleration: Vector3,
    /// Angular velocity in **radians per second**, per axis.
    pub device_rotation_rate: Vector3,
    /// The gravity vector, in **g**, per axis.
    pub gravity: Vector3,
}

impl MotionReading {
    /// XNA's `ToString()` for this reading.
    #[must_use]
    pub fn ToString(&self) -> String {
        format!(
            "DeviceAcceleration:{} Gravity:{}",
            self.device_acceleration.ToString(),
            self.gravity.ToString()
        )
    }
}

/// One gyroscope reading. Rotation is in **radians per second**, per axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GyroscopeReading {
    pub timestamp: SensorTimestamp,
    pub rotation_rate: Vector3,
}

impl GyroscopeReading {
    /// XNA's `ToString()` for this reading.
    #[must_use]
    pub fn ToString(&self) -> String {
        format!("RotationRate:{}", self.rotation_rate.ToString())
    }
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
    /// Injects an acceleration in **metres per second squared**, per axis.
    ///
    /// Not in `g`, which is what the *reading* is in. Upstream converts at this
    /// boundary -- "IPlatformSensors reports acceleration in SI metres per
    /// second squared. WP7 exposes the reading in fractions of standard
    /// gravity" -- so injecting `9.80665` reads back as `1.0`. The gyroscope
    /// and compass convert nothing; this is the only asymmetric one.
    pub fn inject(&self, x: f32, y: f32, z: f32) -> Result<()> {
        // SAFETY: the handle is owned and the axes are by value.
        self.native.check(unsafe {
            (self.native.runtime.accelerometer_inject_synthetic_update)(self.handle, x, y, z)
        })
    }
}

impl Gyroscope {
    /// Injects an angular velocity in **radians per second**, per axis.
    ///
    /// The same unit the reading comes back in: upstream defines both sides as
    /// radians per second and converts nothing.
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

sensor! {
    /// The host's motion sensor: orientation, gravity and linear acceleration.
    ///
    /// Not a fourth device but a *fusion* of the other three, which is why it
    /// is the only one that can answer an orientation.
    Motion, sys::CNA_MotionHandle, MotionReading, sys::CNA_MotionReading,
    create = motion_create, destroy = motion_destroy, dispose = motion_dispose,
    supported = motion_get_is_supported, state = motion_get_state,
    valid = motion_get_is_data_valid, value = motion_get_current_value,
    get_interval = motion_get_time_between_updates_ticks,
    set_interval = motion_set_time_between_updates_ticks,
    start = motion_start, stop = motion_stop,
    convert = |reading: sys::CNA_MotionReading| MotionReading {
        timestamp: SensorTimestamp::from_native(reading.timestamp),
        attitude: attitude(reading.attitude),
        device_acceleration: vector(reading.device_acceleration),
        device_rotation_rate: vector(reading.device_rotation_rate),
        gravity: vector(reading.gravity),
    },
}

const fn attitude(value: sys::CNA_AttitudeReading) -> AttitudeReading {
    AttitudeReading {
        timestamp: SensorTimestamp::from_native(value.timestamp),
        pitch: value.pitch,
        roll: value.roll,
        yaw: value.yaw,
        quaternion: Quaternion {
            X: value.quaternion.x,
            Y: value.quaternion.y,
            Z: value.quaternion.z,
            W: value.quaternion.w,
        },
        rotation_matrix: matrix(value.rotation_matrix),
    }
}

const fn matrix(value: sys::CNA_Matrix) -> Matrix {
    Matrix {
        M11: value.m11, M12: value.m12, M13: value.m13, M14: value.m14,
        M21: value.m21, M22: value.m22, M23: value.m23, M24: value.m24,
        M31: value.m31, M32: value.m32, M33: value.m33, M34: value.m34,
        M41: value.m41, M42: value.m42, M43: value.m43, M44: value.m44,
    }
}

fn native_attitude(value: AttitudeReading) -> sys::CNA_AttitudeReading {
    sys::CNA_AttitudeReading {
        struct_size: core::mem::size_of::<sys::CNA_AttitudeReading>() as u32,
        struct_version: 1,
        timestamp: sys::CNA_DateTimeOffset {
            ticks: value.timestamp.ticks,
            offset_ticks: value.timestamp.offset_ticks,
        },
        pitch: value.pitch,
        roll: value.roll,
        yaw: value.yaw,
        quaternion: sys::CNA_Quaternion {
            x: value.quaternion.X,
            y: value.quaternion.Y,
            z: value.quaternion.Z,
            w: value.quaternion.W,
        },
        rotation_matrix: sys::CNA_Matrix {
            m11: value.rotation_matrix.M11, m12: value.rotation_matrix.M12,
            m13: value.rotation_matrix.M13, m14: value.rotation_matrix.M14,
            m21: value.rotation_matrix.M21, m22: value.rotation_matrix.M22,
            m23: value.rotation_matrix.M23, m24: value.rotation_matrix.M24,
            m31: value.rotation_matrix.M31, m32: value.rotation_matrix.M32,
            m33: value.rotation_matrix.M33, m34: value.rotation_matrix.M34,
            m41: value.rotation_matrix.M41, m42: value.rotation_matrix.M42,
            m43: value.rotation_matrix.M43, m44: value.rotation_matrix.M44,
        },
    }
}

fn native_vector(value: Vector3) -> sys::CNA_Vector3 {
    sys::CNA_Vector3 {
        x: value.X,
        y: value.Y,
        z: value.Z,
    }
}

impl Motion {
    /// Whether the fused orientation is referenced to magnetic north.
    ///
    /// A device with no usable magnetometer still reports an attitude, but a
    /// relative one: the yaw is measured from wherever the device happened to
    /// be. This is how a caller tells the two apart before treating yaw as a
    /// compass heading.
    pub fn is_attitude_north_referenced(&self) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.motion_get_is_attitude_north_referenced_ext)(
                self.handle,
                &mut value,
            )
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Injects a whole motion reading, as the fusion would have produced it.
    pub fn inject(&self, reading: MotionReading) -> Result<()> {
        let native_reading = sys::CNA_MotionReading {
            struct_size: core::mem::size_of::<sys::CNA_MotionReading>() as u32,
            struct_version: 1,
            timestamp: sys::CNA_DateTimeOffset {
                ticks: reading.timestamp.ticks,
                offset_ticks: reading.timestamp.offset_ticks,
            },
            attitude: native_attitude(reading.attitude),
            device_acceleration: native_vector(reading.device_acceleration),
            device_rotation_rate: native_vector(reading.device_rotation_rate),
            gravity: native_vector(reading.gravity),
        };
        // SAFETY: the reading is a live local CNA copies during the call.
        self.native.check(unsafe {
            (self.native.runtime.motion_inject_synthetic_update_ext)(self.handle, &native_reading)
        })
    }

    /// Asks the sensor to raise its calibration event.
    ///
    /// The real trigger is the platform deciding the magnetometer has drifted.
    /// This is how a game's calibration prompt can be exercised without waiting
    /// for that.
    pub fn request_calibration(&self) -> Result<()> {
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self.native.runtime.motion_inject_calibration_request_ext)(self.handle)
        })
    }
}

impl Compass {
    /// Asks the sensor to raise its calibration event.
    pub fn request_calibration(&self) -> Result<()> {
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self.native.runtime.compass_inject_calibration_request_ext)(self.handle)
        })
    }
}

/// A live sensor event subscription.
///
/// # Why this owns the closure
///
/// CNA takes a function pointer and an opaque `void*`, keeps both, and calls
/// them until the registration is withdrawn. Rust closures are not function
/// pointers, so the closure is boxed and its address becomes the `void*`, with
/// a trampoline of the audited signature in front. That makes *this* value the
/// only thing that knows the box is still reachable, which is why it withdraws
/// the registration in `Drop` before the box is freed: the reverse order would
/// leave CNA holding a pointer to a dead closure, and dropping the
/// subscription is the only way a caller can end one.
///
/// Holding a subscription therefore keeps the callback alive; dropping it stops
/// the callbacks and frees the closure, in that order.
#[must_use = "dropping a Subscription immediately unsubscribes it"]
pub struct Subscription {
    native: Arc<Native>,
    registration: sys::CNA_SensorEventRegistrationHandle,
    /// The boxed closure CNA holds a pointer into. Never read here; it exists
    /// so the allocation outlives the registration.
    callback: *mut c_void,
    free: unsafe fn(*mut c_void),
}

// SAFETY: the pointer is an owned `Box` this value alone frees, and the
// closures behind it are required to be `Send`. Nothing here is shared.
unsafe impl Send for Subscription {}

impl Subscription {
    /// Withdraws the subscription early.
    ///
    /// Idempotent, and what `Drop` calls.
    pub fn unsubscribe(&mut self) -> Result<()> {
        if self.registration == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        let registration =
            core::mem::replace(&mut self.registration, sys::CNA_INVALID_HANDLE);
        // SAFETY: the registration is this value's own and withdrawn once. It
        // must come before the box is freed, or CNA is left holding a pointer
        // into freed memory.
        let result = self
            .native
            .check(unsafe { (self.native.runtime.sensor_unsubscribe_ext)(registration) });
        if !self.callback.is_null() {
            let callback = core::mem::replace(&mut self.callback, core::ptr::null_mut());
            // SAFETY: the pointer came from `Box::into_raw` in the matching
            // subscribe call, with the `free` recorded there for its type.
            unsafe { (self.free)(callback) };
        }
        result
    }
}

impl core::fmt::Debug for Subscription {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Subscription")
            .field("live", &(self.registration != sys::CNA_INVALID_HANDLE))
            .finish()
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        let _ = self.unsubscribe();
    }
}

/// Frees a boxed closure of one concrete type.
unsafe fn free_boxed<F>(pointer: *mut c_void) {
    // SAFETY: the caller passes back exactly the pointer `Box::into_raw`
    // produced for this `F`.
    drop(unsafe { Box::from_raw(pointer.cast::<F>()) });
}

/// Declares one sensor subscription over its native reading callback.
macro_rules! subscription {
    (
        $(#[$meta:meta])*
        $sensor:ty, $method:ident, $route:ident, $native:ty, $reading:ty,
        convert = $convert:expr,
    ) => {
        impl $sensor {
            $(#[$meta])*
            pub fn $method(
                &self,
                callback: impl FnMut($reading) + Send + 'static,
            ) -> Result<Subscription> {
                type Closure = Box<dyn FnMut($reading) + Send + 'static>;

                unsafe extern "C" fn trampoline(
                    reading: *const $native,
                    context: *mut c_void,
                ) {
                    if context.is_null() || reading.is_null() {
                        return;
                    }
                    // SAFETY: the context is the box this subscription made and
                    // keeps alive, and CNA borrows the reading for the call.
                    let closure = unsafe { &mut *context.cast::<Closure>() };
                    let value = unsafe { *reading };
                    // A panic must not cross back into C. There is nowhere to
                    // report it from inside a callback CNA is driving, so it is
                    // contained and the subscription simply misses that event.
                    let _ = catch_unwind(AssertUnwindSafe(|| {
                        let converted = $convert;
                        closure(converted(value));
                    }));
                }

                let boxed: Closure = Box::new(callback);
                let context = Box::into_raw(Box::new(boxed)).cast::<c_void>();
                let mut registration = sys::CNA_INVALID_HANDLE;
                // SAFETY: the trampoline has the audited signature, the context
                // is a live box the returned Subscription owns, and the output
                // is a live local.
                let result = self.native.check(unsafe {
                    (self.native.runtime.$route)(
                        self.handle,
                        Some(trampoline),
                        context,
                        &mut registration,
                    )
                });
                if let Err(error) = result {
                    // CNA never took the pointer, so this side still owns it.
                    // SAFETY: the box was made two statements ago and handed to
                    // nobody.
                    unsafe { free_boxed::<Closure>(context) };
                    return Err(error);
                }
                Ok(Subscription {
                    native: Arc::clone(&self.native),
                    registration,
                    callback: context,
                    free: free_boxed::<Closure>,
                })
            }
        }
    };
}

subscription! {
    /// Calls `callback` with every new reading, until the subscription drops.
    Accelerometer, on_current_value_changed,
    accelerometer_subscribe_current_value_changed,
    sys::CNA_AccelerometerReading, AccelerometerReading,
    convert = |reading: sys::CNA_AccelerometerReading| AccelerometerReading {
        timestamp: SensorTimestamp::from_native(reading.timestamp),
        acceleration: vector(reading.acceleration),
    },
}

subscription! {
    /// Calls `callback` with every new reading, until the subscription drops.
    Compass, on_current_value_changed, compass_subscribe_current_value_changed,
    sys::CNA_CompassReading, CompassReading,
    convert = |reading: sys::CNA_CompassReading| CompassReading {
        timestamp: SensorTimestamp::from_native(reading.timestamp),
        heading_accuracy: reading.heading_accuracy,
        magnetic_heading: reading.magnetic_heading,
        true_heading: reading.true_heading,
        magnetometer_reading: vector(reading.magnetometer_reading),
    },
}

subscription! {
    /// Calls `callback` with every new reading, until the subscription drops.
    Gyroscope, on_current_value_changed,
    gyroscope_subscribe_current_value_changed,
    sys::CNA_GyroscopeReading, GyroscopeReading,
    convert = |reading: sys::CNA_GyroscopeReading| GyroscopeReading {
        timestamp: SensorTimestamp::from_native(reading.timestamp),
        rotation_rate: vector(reading.rotation_rate),
    },
}

subscription! {
    /// Calls `callback` with every new reading, until the subscription drops.
    Motion, on_current_value_changed, motion_subscribe_current_value_changed,
    sys::CNA_MotionReading, MotionReading,
    convert = |reading: sys::CNA_MotionReading| MotionReading {
        timestamp: SensorTimestamp::from_native(reading.timestamp),
        attitude: attitude(reading.attitude),
        device_acceleration: vector(reading.device_acceleration),
        device_rotation_rate: vector(reading.device_rotation_rate),
        gravity: vector(reading.gravity),
    },
}

subscription! {
    /// Calls `callback` with the legacy `ReadingChanged` event's own shape.
    ///
    /// Windows Phone's `AccelerometerReadingEventArgs` carried three `double`
    /// axes rather than a `Vector3`, and a game written against it reads those.
    /// This is that event, not a second spelling of
    /// [`Accelerometer::on_current_value_changed`].
    Accelerometer, on_reading_changed, accelerometer_subscribe_reading_changed,
    sys::CNA_AccelerometerReadingEventInfo, AccelerometerReadingEvent,
    convert = |info: sys::CNA_AccelerometerReadingEventInfo| AccelerometerReadingEvent {
        timestamp: SensorTimestamp::from_native(info.timestamp),
        x: info.x,
        y: info.y,
        z: info.z,
    },
}

/// The legacy `AccelerometerReadingEventArgs`: three `double` axes, in **g**.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccelerometerReadingEvent {
    pub timestamp: SensorTimestamp,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl AccelerometerReadingEvent {
    /// XNA's `ToString()` for these event arguments.
    ///
    /// The braced vector shape rather than the `Acceleration:` prefix
    /// [`AccelerometerReading`] uses -- the two types format differently
    /// upstream, and copying one onto the other would be a silent infidelity.
    #[must_use]
    pub fn ToString(&self) -> String {
        format!("{{X:{} Y:{} Z:{}}}", self.x, self.y, self.z)
    }
}

/// Declares one sensor's calibration subscription.
macro_rules! calibration_subscription {
    ($sensor:ty, $route:ident) => {
        impl $sensor {
            /// Calls `callback` when the platform asks for a calibration.
            ///
            /// The event carries nothing: it is a request to show a prompt, and
            /// what to show is the game's business.
            pub fn on_calibrate(
                &self,
                callback: impl FnMut() + Send + 'static,
            ) -> Result<Subscription> {
                type Closure = Box<dyn FnMut() + Send + 'static>;

                unsafe extern "C" fn trampoline(context: *mut c_void) {
                    if context.is_null() {
                        return;
                    }
                    // SAFETY: the context is the box this subscription owns.
                    let closure = unsafe { &mut *context.cast::<Closure>() };
                    let _ = catch_unwind(AssertUnwindSafe(|| closure()));
                }

                let boxed: Closure = Box::new(callback);
                let context = Box::into_raw(Box::new(boxed)).cast::<c_void>();
                let mut registration = sys::CNA_INVALID_HANDLE;
                // SAFETY: as in `subscription!`.
                let result = self.native.check(unsafe {
                    (self.native.runtime.$route)(
                        self.handle,
                        Some(trampoline),
                        context,
                        &mut registration,
                    )
                });
                if let Err(error) = result {
                    // SAFETY: CNA never took the pointer.
                    unsafe { free_boxed::<Closure>(context) };
                    return Err(error);
                }
                Ok(Subscription {
                    native: Arc::clone(&self.native),
                    registration,
                    callback: context,
                    free: free_boxed::<Closure>,
                })
            }
        }
    };
}

calibration_subscription!(Compass, compass_subscribe_calibrate);
calibration_subscription!(Motion, motion_subscribe_calibrate);

/// The last sensor error identity CNA recorded, if it recorded one.
///
/// Process-wide and platform-specific: it is the host's own error number, kept
/// so a refusal that CNA can only report as "the platform said no" still leads
/// somewhere. `None` means no sensor error has been recorded, which is not the
/// same as the last operation having succeeded.
pub fn last_error_id() -> Result<Option<i32>> {
    let native = Native::process()?;
    let mut value = 0_i32;
    let mut has_value = sys::CNA_FALSE;
    // SAFETY: both outputs are live locals.
    native.check(unsafe {
        (native.runtime.sensors_get_last_error_id_ext)(&mut value, &mut has_value)
    })?;
    Ok((has_value != sys::CNA_FALSE).then_some(value))
}

/// A substitute backend, so a machine with no such device can still be tested.
///
/// Nothing here is for a game. These routes move **CNA's own** state, not the
/// device's: they make a sensor claim to be supported, force its started flag,
/// or drive its event dispatch directly. A game that reached for them would be
/// lying to itself about the hardware.
///
/// They are bound because without them this crate's sensor projection is
/// untestable. No verification machine here has an accelerometer, a gyroscope,
/// a compass or a fusion sensor, and upstream says so plainly of the compass
/// backend: "without it there is no compass on any verification machine and no
/// way to reach a single line past the unsupported refusal". A projection that
/// only ever exercises `NotSupported` is a projection nobody has checked.
///
/// The two families take different shapes because upstream gives them
/// different shapes. The compass and the motion sensor take a whole substitute
/// backend in one call. The accelerometer and the gyroscope have no such route
/// and are steered field by field instead.
impl Compass {
    /// Installs or removes CNA's own compass backend.
    ///
    /// `supported` is what the installed backend will answer for
    /// [`Compass::is_supported`], and is ignored when removing. Refused while
    /// acquisition is started, which is upstream's way of saying a backend may
    /// not be swapped underneath a reading in flight.
    pub fn set_test_backend(&self, installed: bool, supported: bool) -> Result<()> {
        // SAFETY: the handle is owned and both flags are by value.
        self.native.check(unsafe {
            (self.native.runtime.compass_set_test_backend_ext)(
                self.handle,
                u8::from(installed),
                u8::from(supported),
            )
        })
    }
}

impl Motion {
    /// Installs or removes CNA's own motion backend.
    ///
    /// `north_referenced` decides what
    /// [`Motion::is_attitude_north_referenced`] will answer, which is the one
    /// property of this sensor a substitute backend cannot infer.
    pub fn set_test_backend(
        &self,
        installed: bool,
        supported: bool,
        north_referenced: bool,
    ) -> Result<()> {
        // SAFETY: the handle is owned and the flags are by value.
        self.native.check(unsafe {
            (self.native.runtime.motion_set_test_backend_ext)(
                self.handle,
                u8::from(installed),
                u8::from(supported),
                u8::from(north_referenced),
            )
        })
    }
}

/// Declares the field-by-field test hooks the accelerometer and gyroscope have
/// in place of a substitute backend.
macro_rules! test_hooks {
    (
        $sensor:ty, $handle:ty,
        set_supported = $set_supported:ident,
        set_started = $set_started:ident,
        subsystem_held = $subsystem_held:ident,
        register = $register:ident, unregister = $unregister:ident,
        cleanup_hook = $cleanup_hook:ident,
        connected = $connected:ident,
        watch_failure = $watch_failure:ident,
        dispatch = $dispatch:ident,
        exceptions = $exceptions:ident,
        exception_size = $exception_size:ident,
        exception_copy = $exception_copy:ident,
    ) => {
        impl $sensor {
            /// Makes the sensor claim to be supported, or stop claiming it.
            pub fn set_supported_for_tests(&self, supported: bool) -> Result<()> {
                // SAFETY: the handle is owned and the flag is by value.
                self.native.check(unsafe {
                    (self.native.runtime.$set_supported)(self.handle, u8::from(supported))
                })
            }

            /// Forces the started flag without going through `start`/`stop`.
            ///
            /// Upstream's own hook for driving the acquisition state machine
            /// past the transitions a device would normally cause.
            pub fn set_started_for_tests(&self, started: bool) -> Result<()> {
                // SAFETY: the handle is owned and the flag is by value.
                self.native.check(unsafe {
                    (self.native.runtime.$set_started)(self.handle, u8::from(started))
                })
            }

            /// Whether this sensor currently holds the platform subsystem.
            ///
            /// The observable half of an ownership contract: a sensor that has
            /// stopped, or been disposed, must not still hold it. That is a
            /// question about *this* projection's teardown as much as about
            /// CNA's, which is why it is bound rather than left to CNA's own
            /// suite.
            pub fn holds_subsystem_for_tests(&self) -> Result<bool> {
                let mut value = sys::CNA_FALSE;
                // SAFETY: the handle is owned and the output is a live local.
                self.native.check(unsafe {
                    (self.native.runtime.$subsystem_held)(self.handle, &mut value)
                })?;
                Ok(value != sys::CNA_FALSE)
            }

            /// Adds this sensor to the set a dispatch reaches.
            pub fn register_started_instance_for_tests(&self) -> Result<()> {
                // SAFETY: the handle is owned.
                self.native
                    .check(unsafe { (self.native.runtime.$register)(self.handle) })
            }

            /// Removes this sensor from the set a dispatch reaches.
            pub fn unregister_started_instance_for_tests(&self) -> Result<()> {
                // SAFETY: the handle is owned.
                self.native
                    .check(unsafe { (self.native.runtime.$unregister)(self.handle) })
            }

            /// Runs `callback` when the sensor is disposed.
            ///
            /// Returns a [`Subscription`] rather than nothing, for the same
            /// reason every other callback here does: CNA keeps the pointer,
            /// so something on this side has to own the closure and outlive
            /// the registration.
            pub fn set_disposal_cleanup_hook_for_tests(
                &self,
                callback: impl FnMut() + Send + 'static,
            ) -> Result<Subscription> {
                type Closure = Box<dyn FnMut() + Send + 'static>;

                unsafe extern "C" fn trampoline(context: *mut c_void) {
                    if context.is_null() {
                        return;
                    }
                    // SAFETY: the context is the box the Subscription owns.
                    let closure = unsafe { &mut *context.cast::<Closure>() };
                    let _ = catch_unwind(AssertUnwindSafe(|| closure()));
                }

                let boxed: Closure = Box::new(callback);
                let context = Box::into_raw(Box::new(boxed)).cast::<c_void>();
                // SAFETY: the trampoline has the audited signature and the
                // context is a live box the returned value owns.
                let result = self.native.check(unsafe {
                    (self.native.runtime.$cleanup_hook)(self.handle, Some(trampoline), context)
                });
                if let Err(error) = result {
                    // SAFETY: CNA never took the pointer.
                    unsafe { free_boxed::<Closure>(context) };
                    return Err(error);
                }
                // This hook has no registration handle of its own; upstream
                // replaces it by setting another. The Subscription still owns
                // the closure, and clearing the hook is what its unsubscribe
                // would need -- there is no route for that, so the handle is
                // left invalid and `Drop` frees the box without a withdrawal.
                Ok(Subscription {
                    native: Arc::clone(&self.native),
                    registration: sys::CNA_INVALID_HANDLE,
                    callback: context,
                    free: free_boxed::<Closure>,
                })
            }

            /// Whether a platform sensor identifier is currently connected.
            pub fn is_sensor_connected_for_tests(
                game: &GameContext<'_>,
                sensor_id: i64,
            ) -> Result<bool> {
                let (native, handle) = game.native_game();
                let mut value = sys::CNA_FALSE;
                // SAFETY: the game handle is live and the output is a local.
                native.check(unsafe {
                    (native.runtime.$connected)(handle, sensor_id, &mut value)
                })?;
                Ok(value != sys::CNA_FALSE)
            }

            /// Makes the next event-watch registration fail, or stop failing.
            ///
            /// The error path a caller can otherwise only reach by breaking the
            /// host.
            pub fn set_event_watch_registration_failure_for_tests(
                game: &GameContext<'_>,
                should_fail: bool,
            ) -> Result<()> {
                let (native, handle) = game.native_game();
                // SAFETY: the game handle is live and the flag is by value.
                native.check(unsafe {
                    (native.runtime.$watch_failure)(handle, u8::from(should_fail))
                })
            }

            /// Delivers one synthetic reading to exactly these sensors.
            ///
            /// The dispatch path itself, driven directly, so a subscription's
            /// callback can be observed without a device and without waiting.
            ///
            /// The axes are in the same **platform** units the matching
            /// `inject` takes, so an accelerometer dispatch is in metres per
            /// second squared and arrives at the callback in `g`.
            pub fn dispatch_to_for_tests(
                game: &GameContext<'_>,
                sensors: &[&Self],
                x: f32,
                y: f32,
                z: f32,
            ) -> Result<()> {
                let (native, handle) = game.native_game();
                let handles: Vec<$handle> =
                    sensors.iter().map(|sensor| sensor.handle).collect();
                // SAFETY: the game handle is live, and the array is borrowed
                // for the call with the count it was sized against. A null
                // pointer is only valid for a zero count, which is what an
                // empty slice's `as_ptr` is paired with here.
                native.check(unsafe {
                    (native.runtime.$dispatch)(
                        handle,
                        handles.as_ptr(),
                        handles.len() as u64,
                        x,
                        y,
                        z,
                    )
                })
            }

            /// How many dispatches raised an exception a subscriber threw.
            ///
            /// A Rust subscription cannot make this count rise: a panic is
            /// caught at the trampoline and never crosses back into C. That is
            /// the point of reading it -- if it ever moves, a Rust callback has
            /// unwound into CNA, which is the bug this reports.
            pub fn dispatch_exception_count_for_tests(game: &GameContext<'_>) -> Result<i32> {
                let (native, handle) = game.native_game();
                let mut value = 0_i32;
                // SAFETY: the game handle is live and the output is a local.
                native.check(unsafe { (native.runtime.$exceptions)(handle, &mut value) })?;
                Ok(value)
            }

            /// What the last such exception said.
            pub fn last_dispatch_exception_message_for_tests(
                game: &GameContext<'_>,
            ) -> Result<String> {
                let (native, handle) = game.native_game();
                read_string(
                    |value| native.check(value),
                    // SAFETY: both outputs are live locals; the two routes are
                    // CNA's canonical size-then-copy pair for one UTF-8 string.
                    |bytes| unsafe { (native.runtime.$exception_size)(handle, bytes) },
                    |destination, capacity, written| unsafe {
                        (native.runtime.$exception_copy)(handle, destination, capacity, written)
                    },
                )
            }
        }
    };
}

test_hooks! {
    Accelerometer, sys::CNA_AccelerometerHandle,
    set_supported = accelerometer_set_supported_for_tests_ext,
    set_started = accelerometer_set_started_for_tests_ext,
    subsystem_held = accelerometer_get_subsystem_held_for_tests_ext,
    register = accelerometer_register_started_instance_for_tests_ext,
    unregister = accelerometer_unregister_started_instance_for_tests_ext,
    cleanup_hook = accelerometer_set_disposal_cleanup_hook_for_tests_ext,
    connected = accelerometer_is_sensor_connected_for_tests_ext,
    watch_failure = accelerometer_set_event_watch_registration_failure_for_tests_ext,
    dispatch = accelerometer_dispatch_to_instances_for_tests_ext,
    exceptions = accelerometer_get_dispatch_exception_count_for_tests_ext,
    exception_size = accelerometer_get_last_dispatch_exception_message_size_for_tests_ext,
    exception_copy = accelerometer_copy_last_dispatch_exception_message_for_tests_ext,
}

test_hooks! {
    Gyroscope, sys::CNA_GyroscopeHandle,
    set_supported = gyroscope_set_supported_for_tests_ext,
    set_started = gyroscope_set_started_for_tests_ext,
    subsystem_held = gyroscope_get_subsystem_held_for_tests_ext,
    register = gyroscope_register_started_instance_for_tests_ext,
    unregister = gyroscope_unregister_started_instance_for_tests_ext,
    cleanup_hook = gyroscope_set_disposal_cleanup_hook_for_tests_ext,
    connected = gyroscope_is_sensor_connected_for_tests_ext,
    watch_failure = gyroscope_set_event_watch_registration_failure_for_tests_ext,
    dispatch = gyroscope_dispatch_to_instances_for_tests_ext,
    exceptions = gyroscope_get_dispatch_exception_count_for_tests_ext,
    exception_size = gyroscope_get_last_dispatch_exception_message_size_for_tests_ext,
    exception_copy = gyroscope_copy_last_dispatch_exception_message_for_tests_ext,
}
