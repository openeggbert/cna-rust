//! CNA's per-device input enumeration, hot-plug events and mouse cursors.
//!
//! XNA's input is positional: `Keyboard.GetState()` is *the* keyboard and
//! `Mouse.GetState()` is *the* mouse, with no way to ask how many there are,
//! which one moved, or when one is unplugged. CNA reports the devices
//! themselves, so that lives here rather than beside XNA's `Keyboard` and
//! `Mouse`.
//!
//! Two things about identity matter and are not guessable from the API's shape:
//!
//! - **An index is not an identity.** Upstream states that the enumeration is a
//!   point-in-time snapshot and that an index is valid only until the device
//!   set changes. So an index is never handed out as a durable reference;
//!   [`InputDevice::id`] is the stable one, and it is what hot-plug events
//!   carry.
//! - **A hot-plug event carries an id, not a device.** By the time a
//!   disconnection is delivered the device is gone, so there is nothing to
//!   enumerate; the id is the only thing that can still be meaningful.

#![allow(clippy::missing_errors_doc)]

use core::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, OnceLock};

use cna_sys as sys;

use crate::error::Result;
use crate::game::GameContext;
use crate::graphics::Texture2D;
use crate::native::runtime::read_string;
use crate::native::Native;

/// Which family of device a query or event concerns.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InputDeviceKind {
    Keyboard,
    Mouse,
    /// A touch digitiser. CNA enumerates these but reports no hot-plug events
    /// for them, so this kind cannot be subscribed to.
    TouchDevice,
}

/// One enumerated input device.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputDevice {
    /// The native instance identifier, which is the durable identity.
    ///
    /// Wider than the joystick and sensor identifiers because a touch device's
    /// identifier is 64-bit natively.
    pub id: u64,
    /// The device's reported name.
    pub name: String,
}

impl InputDevice {
    /// Whether CNA considers this the same device as `other`.
    ///
    /// This asks CNA rather than comparing the fields here, because CNA
    /// defines what equality means for a device -- it compares the identifier
    /// **and** the name -- and a Rust `==` that happened to agree today would
    /// be a guess that could quietly stop agreeing.
    pub fn same_device(&self, other: &Self) -> Result<bool> {
        let native = Native::process()?;
        let left = self.to_native()?;
        let right = other.to_native()?;
        let mut equal = sys::CNA_FALSE;
        // SAFETY: both descriptors and both names are live for the call, and
        // the output is a live local.
        native.check(unsafe {
            (native.runtime.input_device_info_equals)(
                &left,
                name_view(&self.name),
                &right,
                name_view(&other.name),
                &mut equal,
            )
        })?;
        Ok(equal != sys::CNA_FALSE)
    }

    /// Builds the canonical descriptor, starting from CNA's own initializer.
    ///
    /// Using `cna_input_device_info_init` rather than zeroing the structure
    /// means a field this build does not know about still gets whatever CNA
    /// considers its default.
    fn to_native(&self) -> Result<sys::CNA_InputDeviceInfo> {
        let native = Native::process()?;
        let mut info = sys::CNA_InputDeviceInfo {
            struct_size: core::mem::size_of::<sys::CNA_InputDeviceInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_InputDeviceInfo::default()
        };
        // SAFETY: the structure is a caller-owned versioned output whose size
        // and version this build sets before the call.
        native.check(unsafe { (native.runtime.input_device_info_init)(&mut info) })?;
        info.id = self.id;
        Ok(info)
    }
}

fn name_view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast::<core::ffi::c_char>(),
        byte_length: value.len() as u64,
    }
}

/// How many devices of one kind are currently enumerated.
pub fn count(game: &GameContext<'_>, kind: InputDeviceKind) -> Result<u32> {
    let (native, handle) = game.native_game();
    let mut value = 0_u32;
    let api = &native.runtime;
    // SAFETY: the game handle is live and the output is a live local.
    native.check(unsafe {
        match kind {
            InputDeviceKind::Keyboard => (api.input_devices_get_keyboard_count)(handle, &mut value),
            InputDeviceKind::Mouse => (api.input_devices_get_mouse_count)(handle, &mut value),
            InputDeviceKind::TouchDevice => {
                (api.input_devices_get_touch_device_count)(handle, &mut value)
            }
        }
    })?;
    Ok(value)
}

/// Every device of one kind, as one snapshot.
///
/// Taken in one pass because the enumeration is a snapshot: interleaving other
/// calls between the count and the reads would risk describing two different
/// device sets as one list.
pub fn enumerate(game: &GameContext<'_>, kind: InputDeviceKind) -> Result<Vec<InputDevice>> {
    let total = count(game, kind)?;
    let (native, handle) = game.native_game();
    let api = &native.runtime;
    let mut devices = Vec::with_capacity(total as usize);
    for index in 0..total {
        let mut info = sys::CNA_InputDeviceInfo {
            struct_size: core::mem::size_of::<sys::CNA_InputDeviceInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_InputDeviceInfo::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output and the
        // index is below the count this call just read.
        native.check(unsafe {
            match kind {
                InputDeviceKind::Keyboard => {
                    (api.input_devices_get_keyboard_info_at)(handle, index, &mut info)
                }
                InputDeviceKind::Mouse => {
                    (api.input_devices_get_mouse_info_at)(handle, index, &mut info)
                }
                InputDeviceKind::TouchDevice => {
                    (api.input_devices_get_touch_device_info_at)(handle, index, &mut info)
                }
            }
        })?;
        let name = read_string(
            |value| native.check(value),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe {
                match kind {
                    InputDeviceKind::Keyboard => {
                        (api.input_devices_get_keyboard_name_size_at)(handle, index, bytes)
                    }
                    InputDeviceKind::Mouse => {
                        (api.input_devices_get_mouse_name_size_at)(handle, index, bytes)
                    }
                    InputDeviceKind::TouchDevice => {
                        (api.input_devices_get_touch_device_name_size_at)(handle, index, bytes)
                    }
                }
            },
            |destination, capacity, written| unsafe {
                match kind {
                    InputDeviceKind::Keyboard => (api.input_devices_copy_keyboard_name_at)(
                        handle, index, destination, capacity, written,
                    ),
                    InputDeviceKind::Mouse => (api.input_devices_copy_mouse_name_at)(
                        handle, index, destination, capacity, written,
                    ),
                    InputDeviceKind::TouchDevice => (api.input_devices_copy_touch_device_name_at)(
                        handle, index, destination, capacity, written,
                    ),
                }
            },
        )?;
        devices.push(InputDevice { id: info.id, name });
    }
    Ok(devices)
}

/// Which transition a hot-plug subscription is interested in.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Hotplug {
    KeyboardConnected,
    KeyboardDisconnected,
    MouseConnected,
    MouseDisconnected,
}

/// A live hot-plug subscription. Dropping it stops delivery.
#[derive(Debug)]
pub struct HotplugSubscription {
    slot: u64,
    event: Hotplug,
}

impl Drop for HotplugSubscription {
    fn drop(&mut self) {
        release(self.event, self.slot);
    }
}

type HotplugHandler = Box<dyn FnMut(u64) + Send>;

/// One shared native registration per transition, and the handlers it feeds.
///
/// Shared for the same reason the text-input registrations are: CNA delivers
/// once per registration and this crate's trampoline delivers to every
/// handler, so one registration per subscriber would multiply deliveries.
#[derive(Default)]
struct EventState {
    registration: sys::CNA_InputDeviceEventRegistrationHandle,
    native: Option<Arc<Native>>,
    entries: Vec<(u64, HotplugHandler)>,
}

#[derive(Default)]
struct HotplugTable {
    next: Mutex<u64>,
    keyboard_connected: Mutex<EventState>,
    keyboard_disconnected: Mutex<EventState>,
    mouse_connected: Mutex<EventState>,
    mouse_disconnected: Mutex<EventState>,
}

fn table() -> &'static HotplugTable {
    static TABLE: OnceLock<HotplugTable> = OnceLock::new();
    TABLE.get_or_init(HotplugTable::default)
}

fn state(event: Hotplug) -> &'static Mutex<EventState> {
    match event {
        Hotplug::KeyboardConnected => &table().keyboard_connected,
        Hotplug::KeyboardDisconnected => &table().keyboard_disconnected,
        Hotplug::MouseConnected => &table().mouse_connected,
        Hotplug::MouseDisconnected => &table().mouse_disconnected,
    }
}

fn release(event: Hotplug, slot: u64) {
    let mut state = state(event)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state.entries.retain(|(id, _)| *id != slot);
    if !state.entries.is_empty() {
        return;
    }
    if let Some(native) = state.native.take() {
        if state.registration != sys::CNA_INVALID_HANDLE {
            // SAFETY: the registration was created here and is released once.
            let _ = unsafe { (native.runtime.input_devices_unsubscribe)(state.registration) };
        }
    }
    state.registration = sys::CNA_INVALID_HANDLE;
}

fn dispatch(event: Hotplug, device_id: u32) {
    // A panic must not unwind into C. Containing it costs one handler's
    // delivery; letting it out costs the process.
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let mut state = state(event)
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (_, handler) in state.entries.iter_mut() {
            let _ = catch_unwind(AssertUnwindSafe(|| handler(u64::from(device_id))));
        }
    }));
}

unsafe extern "C" fn keyboard_connected(id: u32, _context: *mut c_void) {
    dispatch(Hotplug::KeyboardConnected, id);
}
unsafe extern "C" fn keyboard_disconnected(id: u32, _context: *mut c_void) {
    dispatch(Hotplug::KeyboardDisconnected, id);
}
unsafe extern "C" fn mouse_connected(id: u32, _context: *mut c_void) {
    dispatch(Hotplug::MouseConnected, id);
}
unsafe extern "C" fn mouse_disconnected(id: u32, _context: *mut c_void) {
    dispatch(Hotplug::MouseDisconnected, id);
}

/// Delivers one hot-plug transition to `handler`, by device identifier.
///
/// The handler receives an identifier rather than a device, because by the
/// time a disconnection arrives the device is no longer enumerable.
pub fn subscribe(
    event: Hotplug,
    handler: impl FnMut(u64) + Send + 'static,
) -> Result<HotplugSubscription> {
    let native = Native::process()?;
    let slot = {
        let mut next = table()
            .next
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *next = next.wrapping_add(1).max(1);
        *next
    };
    let mut state = state(event)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if state.entries.is_empty() {
        let api = &native.runtime;
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: each trampoline has the canonical signature and needs no
        // context: it dispatches through this crate's own table, so there is
        // no pointer whose lifetime could be got wrong.
        native.check(unsafe {
            match event {
                Hotplug::KeyboardConnected => (api.input_devices_subscribe_keyboard_connected)(
                    Some(keyboard_connected),
                    core::ptr::null_mut(),
                    &mut registration,
                ),
                Hotplug::KeyboardDisconnected => {
                    (api.input_devices_subscribe_keyboard_disconnected)(
                        Some(keyboard_disconnected),
                        core::ptr::null_mut(),
                        &mut registration,
                    )
                }
                Hotplug::MouseConnected => (api.input_devices_subscribe_mouse_connected)(
                    Some(mouse_connected),
                    core::ptr::null_mut(),
                    &mut registration,
                ),
                Hotplug::MouseDisconnected => (api.input_devices_subscribe_mouse_disconnected)(
                    Some(mouse_disconnected),
                    core::ptr::null_mut(),
                    &mut registration,
                ),
            }
        })?;
        state.registration = registration;
        state.native = Some(Arc::clone(&native));
    }
    state.entries.push((slot, Box::new(handler)));
    Ok(HotplugSubscription { slot, event })
}

/// Raises a hot-plug transition, as the platform would.
///
/// CNA provides this so a game's device handling can be tested without
/// physically unplugging anything. It is a real delivery through the same
/// path, not a shortcut around it.
pub fn raise(game: &GameContext<'_>, event: Hotplug, device_id: u32) -> Result<()> {
    let (native, handle) = game.native_game();
    let api = &native.runtime;
    // SAFETY: both arguments are by value and the game handle is live.
    native.check(unsafe {
        match event {
            Hotplug::KeyboardConnected => {
                (api.input_devices_raise_keyboard_connected)(handle, device_id)
            }
            Hotplug::KeyboardDisconnected => {
                (api.input_devices_raise_keyboard_disconnected)(handle, device_id)
            }
            Hotplug::MouseConnected => (api.input_devices_raise_mouse_connected)(handle, device_id),
            Hotplug::MouseDisconnected => {
                (api.input_devices_raise_mouse_disconnected)(handle, device_id)
            }
        }
    })
}

/// One of the platform's standard cursor shapes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum StockCursor {
    Arrow,
    Crosshair,
    Hand,
    IBeam,
    No,
    SizeAll,
    SizeNesw,
    SizeNs,
    SizeNwse,
    SizeWe,
    Wait,
    WaitArrow,
}

impl StockCursor {
    const fn to_native(self) -> sys::CNA_MouseCursorStock {
        match self {
            Self::Arrow => sys::CNA_MOUSE_CURSOR_STOCK_ARROW,
            Self::Crosshair => sys::CNA_MOUSE_CURSOR_STOCK_CROSSHAIR,
            Self::Hand => sys::CNA_MOUSE_CURSOR_STOCK_HAND,
            Self::IBeam => sys::CNA_MOUSE_CURSOR_STOCK_IBEAM,
            Self::No => sys::CNA_MOUSE_CURSOR_STOCK_NO,
            Self::SizeAll => sys::CNA_MOUSE_CURSOR_STOCK_SIZE_ALL,
            Self::SizeNesw => sys::CNA_MOUSE_CURSOR_STOCK_SIZE_NESW,
            Self::SizeNs => sys::CNA_MOUSE_CURSOR_STOCK_SIZE_NS,
            Self::SizeNwse => sys::CNA_MOUSE_CURSOR_STOCK_SIZE_NWSE,
            Self::SizeWe => sys::CNA_MOUSE_CURSOR_STOCK_SIZE_WE,
            Self::Wait => sys::CNA_MOUSE_CURSOR_STOCK_WAIT,
            Self::WaitArrow => sys::CNA_MOUSE_CURSOR_STOCK_WAIT_ARROW,
        }
    }
}

/// A mouse cursor this value owns.
///
/// XNA had no cursor object at all -- `Game.IsMouseVisible` was the whole of
/// it -- so this is a CNA concept and stays out of the XNA `Mouse` projection.
#[derive(Debug)]
pub struct MouseCursor {
    native: Arc<Native>,
    handle: sys::CNA_MouseCursorHandle,
}

impl MouseCursor {
    /// The platform's default cursor.
    pub fn default_cursor() -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local receiving a newly owned handle.
        native.check(unsafe { (native.runtime.mouse_cursor_create)(&mut handle) })?;
        Ok(Self { native, handle })
    }

    /// One of the platform's standard shapes.
    pub fn stock(game: &GameContext<'_>, stock: StockCursor) -> Result<Self> {
        let (native, game_handle) = game.native_game();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the identity is checked and the output is a live local.
        native.check(unsafe {
            (native.runtime.mouse_cursor_get_stock)(game_handle, stock.to_native(), &mut handle)
        })?;
        Ok(Self {
            native: Arc::clone(native),
            handle,
        })
    }

    /// A cursor drawn from a texture, with a hotspot in texture pixels.
    pub fn from_texture(
        game: &GameContext<'_>,
        texture: &Texture2D,
        origin_x: i32,
        origin_y: i32,
    ) -> Result<Self> {
        let (native, game_handle) = game.native_game();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: both handles are live and the output is a live local. CNA
        // copies what it needs from the texture during the call, so the
        // cursor does not retain a borrow of it.
        native.check(unsafe {
            (native.runtime.mouse_cursor_create_from_texture2d)(
                game_handle,
                texture.handle()?,
                origin_x,
                origin_y,
                &mut handle,
            )
        })?;
        Ok(Self {
            native: Arc::clone(native),
            handle,
        })
    }

    /// Makes this the cursor the window shows.
    pub fn set_current(&self, game: &GameContext<'_>) -> Result<()> {
        let (native, game_handle) = game.native_game();
        // SAFETY: both handles are live for the call.
        native.check(unsafe { (native.runtime.mouse_set_cursor)(game_handle, self.handle) })
    }

    /// Releases the cursor's platform resources without dropping this value.
    pub fn dispose(&self) -> Result<()> {
        // SAFETY: the handle is owned by this value.
        self.native
            .check(unsafe { (self.native.runtime.mouse_cursor_dispose)(self.handle) })
    }
}

impl Drop for MouseCursor {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        let _ = unsafe { (self.native.runtime.mouse_cursor_destroy)(self.handle) };
    }
}

/// The last of `input_joystick.h`, `input_devices.h` and `input_text.h`.
///
/// Three things that reach past XNA, and one that reaches past the rest of this
/// module.
///
/// **Joystick hotplug is process-wide.** `cna_joysticks_subscribe_connected_ext`
/// takes no game handle, so a registration outlives any one game -- unlike
/// [`subscribe`], which is scoped to the game it was made on. Dropping the
/// returned value is what withdraws it.
///
/// **The device sensors are not the sensor objects.** `sensors.h` publishes an
/// `Accelerometer` with a lifecycle, events and a test backend;
/// [`device_accelerometer`] reads whatever the *device* reports right now and
/// answers `None` when it reports nothing. A desktop has no accelerometer and
/// says so rather than failing.
///
/// **The reset hooks are how a test starts from a known state.** Each clears
/// the process-wide state its family accumulated, which is what makes a second
/// test in the same binary see what the first one did not leave behind.

use crate::value::Vector3;

type HotplugClosure = Box<dyn FnMut(u32) + Send + 'static>;

/// Which hotplug event a joystick subscription is for.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum JoystickHotplug {
    /// A joystick was plugged in.
    Connected,
    /// A joystick was unplugged.
    Disconnected,
}

/// A live registration on a process-wide joystick hotplug event.
#[must_use = "dropping a JoystickSubscription immediately unsubscribes it"]
pub struct JoystickSubscription {
    native: Arc<Native>,
    registration: Mutex<sys::CNA_JoystickEventRegistrationHandle>,
    callback: Mutex<*mut core::ffi::c_void>,
}

// SAFETY: the pointer is an owned box this value alone frees, and the closure
// behind it is required to be `Send`.
unsafe impl Send for JoystickSubscription {}

impl JoystickSubscription {
    /// Withdraws the subscription early. Idempotent.
    pub fn unsubscribe(&self) -> Result<()> {
        let mut guard = self
            .registration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let registration = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if registration == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        let result = self.native.unsubscribe_joystick(registration);
        let mut callback = self
            .callback
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pointer = core::mem::replace(&mut *callback, core::ptr::null_mut());
        if !pointer.is_null() {
            // SAFETY: the pointer came from `Box::into_raw` below, and the
            // registration naming it is already withdrawn.
            drop(unsafe { Box::from_raw(pointer.cast::<HotplugClosure>()) });
        }
        result
    }
}

impl Drop for JoystickSubscription {
    fn drop(&mut self) {
        let _ = self.unsubscribe();
    }
}

impl core::fmt::Debug for JoystickSubscription {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("JoystickSubscription")
            .field(
                "registration",
                &*self
                    .registration
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            )
            .finish_non_exhaustive()
    }
}

/// Subscribes to a process-wide joystick hotplug event.
pub fn subscribe_joystick(
    event: JoystickHotplug,
    callback: impl FnMut(u32) + Send + 'static,
) -> Result<JoystickSubscription> {
    unsafe extern "C" fn trampoline(joystick_id: u32, context: *mut core::ffi::c_void) {
        if context.is_null() {
            return;
        }
        // SAFETY: the context is the box the subscription owns and is freed
        // only after the registration naming it has been withdrawn.
        let closure = unsafe { &mut *context.cast::<HotplugClosure>() };
        // A panic must not cross back into C, and a hotplug has nowhere to
        // report one.
        let _ =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| closure(joystick_id)));
    }

    let native = Native::process()?;
    let boxed: HotplugClosure = Box::new(callback);
    let context = Box::into_raw(Box::new(boxed)).cast::<core::ffi::c_void>();
    let outcome = match event {
        JoystickHotplug::Connected => {
            native.subscribe_joystick_connected(Some(trampoline), context)
        }
        JoystickHotplug::Disconnected => {
            native.subscribe_joystick_disconnected(Some(trampoline), context)
        }
    };
    match outcome {
        Ok(registration) => Ok(JoystickSubscription {
            native,
            registration: Mutex::new(registration),
            callback: Mutex::new(context),
        }),
        Err(error) => {
            // CNA never took the pointer, so this is the only owner left.
            // SAFETY: the box was created immediately above.
            drop(unsafe { Box::from_raw(context.cast::<HotplugClosure>()) });
            Err(error)
        }
    }
}

/// Raises a joystick hotplug event, for driving the path in a test.
pub fn raise_joystick(
    game: &GameContext<'_>,
    event: JoystickHotplug,
    joystick_id: u32,
) -> Result<()> {
    match event {
        JoystickHotplug::Connected => {
            game.native.raise_joystick_connected(game.handle, joystick_id)
        }
        JoystickHotplug::Disconnected => game
            .native
            .raise_joystick_disconnected(game.handle, joystick_id),
    }
}

/// Clears the process-wide joystick state.
pub fn reset_joysticks_for_tests(game: &GameContext<'_>) -> Result<()> {
    game.native.reset_joysticks_for_tests(game.handle)
}

/// Clears the process-wide input-device state.
pub fn reset_input_devices_for_tests(game: &GameContext<'_>) -> Result<()> {
    game.native.reset_input_devices_for_tests(game.handle)
}

/// Clears the process-wide text-input state.
pub fn reset_text_input_for_tests(game: &GameContext<'_>) -> Result<()> {
    game.native.reset_text_input_for_tests(game.handle)
}

/// Whether the clipboard currently holds text.
///
/// Cheaper than reading it, and a different answer: an empty clipboard and a
/// clipboard holding an empty string are distinguishable here and not through
/// the text route.
pub fn clipboard_has_text(game: &GameContext<'_>) -> Result<bool> {
    game.native.clipboard_has_text(game.handle)
}

/// The device's accelerometer reading, if it has one.
pub fn device_accelerometer(game: &GameContext<'_>) -> Result<Option<Vector3>> {
    Ok(game
        .native
        .device_accelerometer(game.handle)?
        .map(|value| Vector3::from_x_and_y_and_z(value.x, value.y, value.z)))
}

/// The device's gyroscope reading, if it has one.
pub fn device_gyroscope(game: &GameContext<'_>) -> Result<Option<Vector3>> {
    Ok(game
        .native
        .device_gyroscope(game.handle)?
        .map(|value| Vector3::from_x_and_y_and_z(value.x, value.y, value.z)))
}

/// What the battery is doing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatteryInfo {
    /// The power state.
    pub state: crate::extensions::devices::PowerState,
    /// Charge percentage, or `None` when the platform does not know.
    pub charge_percent: Option<i32>,
    /// Seconds of charge left, or `None` when the platform does not know.
    pub seconds_remaining: Option<i32>,
}

/// Reads the battery state, charge and remaining time in one call.
///
/// Wider than [`crate::extensions::devices::power_state`], which answers the
/// state alone. Both numbers are `None` when the platform reports them
/// unknown, which it does with `-1`; passing that through as a percentage
/// would be a number a caller could plot.
pub fn battery_info(game: &GameContext<'_>) -> Result<BatteryInfo> {
    let (state, charge_percent, seconds_remaining) = game.native.power_info(game.handle)?;
    Ok(BatteryInfo {
        state: crate::extensions::devices::PowerState::from_native(state),
        charge_percent,
        seconds_remaining,
    })
}
