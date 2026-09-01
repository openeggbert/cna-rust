#![allow(non_snake_case, non_upper_case_globals, clippy::missing_errors_doc)]

use core::ffi::c_void;
use core::fmt;
use core::ops::{BitAnd, BitOr, BitOrAssign};
use std::any::Any;
use std::error::Error;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::thread::ThreadId;

use cna_sys as sys;

use crate::content::{SerializationInfo, StreamingContext};
use crate::error::{CnaError, ErrorCategory, Result};
use crate::extensions::events::{EventArgs, EventHandler};
use crate::graphics::resource::EventHandlers;
use crate::input::PlayerIndex;
use crate::native::Native;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FileMode {
    CreateNew = 1,
    Create = 2,
    Open = 3,
    OpenOrCreate = 4,
    Truncate = 5,
    Append = 6,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FileAccess {
    Read = 1,
    Write = 2,
    ReadWrite = 3,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct FileShare(u32);

impl FileShare {
    pub const None: Self = Self(0);
    pub const Read: Self = Self(1);
    pub const Write: Self = Self(2);
    pub const ReadWrite: Self = Self(3);
    pub const Delete: Self = Self(4);
    pub const Inheritable: Self = Self(16);

    const fn bits(self) -> u32 {
        self.0
    }
}

impl BitOr for FileShare {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for FileShare {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for FileShare {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

pub type StorageAsyncState = Option<Arc<dyn Any + Send + Sync>>;
pub type StorageAsyncCallback = Box<dyn FnOnce(&StorageAsyncResult) + Send>;

enum StorageAsyncValue {
    Device(StorageDevice),
    Container(StorageContainer),
}

struct StorageAsyncInner {
    value: Mutex<Option<StorageAsyncValue>>,
    state: StorageAsyncState,
    completed: AtomicBool,
    ended: AtomicBool,
    origin: u64,
}

#[derive(Clone)]
pub struct StorageAsyncResult {
    inner: Arc<StorageAsyncInner>,
}

impl StorageAsyncResult {
    fn completed(value: StorageAsyncValue, state: StorageAsyncState, origin: u64) -> Self {
        Self {
            inner: Arc::new(StorageAsyncInner {
                value: Mutex::new(Some(value)),
                state,
                completed: AtomicBool::new(true),
                ended: AtomicBool::new(false),
                origin,
            }),
        }
    }

    #[must_use]
    pub fn AsyncState(&self) -> StorageAsyncState {
        self.inner.state.clone()
    }

    #[must_use]
    pub fn CompletedSynchronously(&self) -> bool {
        true
    }

    #[must_use]
    pub fn IsCompleted(&self) -> bool {
        self.inner.completed.load(Ordering::Acquire)
    }

    fn take(&self) -> Result<StorageAsyncValue> {
        if self.inner.ended.swap(true, Ordering::AcqRel) {
            return Err(CnaError::InvalidInput(
                "a Storage End method cannot be called twice",
            ));
        }
        self.inner
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .ok_or(CnaError::InvalidInput(
                "storage asynchronous result has no value",
            ))
    }
}

fn complete_callback(
    result: StorageAsyncResult,
    callback: Option<StorageAsyncCallback>,
) -> Result<StorageAsyncResult> {
    if let Some(callback) = callback {
        catch_unwind(AssertUnwindSafe(|| callback(&result)))
            .map_err(|_| CnaError::Callback("storage completion callback panicked".to_owned()))?;
    }
    Ok(result)
}

static NEXT_DEVICE_ID: AtomicU64 = AtomicU64::new(0);

struct StorageDeviceState {
    native: Arc<Native>,
    handle: Mutex<sys::CNA_StorageDeviceHandle>,
    id: u64,
}

impl StorageDeviceState {
    fn handle(&self) -> Result<sys::CNA_StorageDeviceHandle> {
        take_device_changed_error()?;
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            Err(CnaError::InvalidInput("storage device is disconnected"))
        } else {
            Ok(handle)
        }
    }
}

impl Drop for StorageDeviceState {
    fn drop(&mut self) {
        let handle = *self
            .handle
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle != sys::CNA_INVALID_HANDLE && self.native.destroy_storage_device(handle).is_ok() {
            *self
                .handle
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = sys::CNA_INVALID_HANDLE;
        }
    }
}

#[derive(Clone)]
pub struct StorageDevice {
    state: Arc<StorageDeviceState>,
}

impl StorageDevice {
    fn from_handle(native: Arc<Native>, handle: sys::CNA_StorageDeviceHandle) -> Self {
        Self {
            state: Arc::new(StorageDeviceState {
                native,
                handle: Mutex::new(handle),
                id: NEXT_DEVICE_ID
                    .fetch_add(1, Ordering::AcqRel)
                    .wrapping_add(1),
            }),
        }
    }

    pub fn FreeSpace(&self) -> Result<i64> {
        self.state
            .native
            .storage_device_free_space(self.state.handle()?)
    }

    pub fn IsConnected(&self) -> Result<bool> {
        self.state
            .native
            .storage_device_is_connected(self.state.handle()?)
    }

    pub fn TotalSpace(&self) -> Result<i64> {
        self.state
            .native
            .storage_device_total_space(self.state.handle()?)
    }

    pub fn AddDeviceChangedHandler(handler: Box<dyn EventHandler>) -> u64 {
        let registration = device_changed_handlers().add(handler);
        ensure_device_changed_subscription();
        registration
    }

    pub fn RemoveDeviceChangedHandler(registration: u64) -> bool {
        let removed = device_changed_handlers().remove(registration);
        if removed && device_changed_handlers().is_empty() {
            let mut bridge = device_changed_bridge()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let released = bridge.as_mut().map_or(Ok(()), DeviceChangedBridge::release);
            match released {
                Ok(()) => {
                    let released = bridge.take();
                    drop(bridge);
                    drop(released);
                    *device_changed_owner()
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
                    DEVICE_CHANGED_PENDING.store(false, Ordering::Release);
                }
                Err(error) => record_device_changed_error(error),
            }
        }
        removed
    }

    pub fn BeginShowSelector(
        player: PlayerIndex,
        callback: Option<StorageAsyncCallback>,
        state: StorageAsyncState,
    ) -> Result<StorageAsyncResult> {
        Self::begin_selector(Some(player), None, callback, state)
    }

    pub fn BeginShowSelectorWithPlayerAndSizeInBytesAndDirectoryCountAndCallbackAndState(
        player: PlayerIndex,
        sizeInBytes: i32,
        directoryCount: i32,
        callback: Option<StorageAsyncCallback>,
        state: StorageAsyncState,
    ) -> Result<StorageAsyncResult> {
        Self::begin_selector(
            Some(player),
            Some((sizeInBytes, directoryCount)),
            callback,
            state,
        )
    }

    pub fn BeginShowSelectorWithCallbackAndState(
        callback: Option<StorageAsyncCallback>,
        state: StorageAsyncState,
    ) -> Result<StorageAsyncResult> {
        Self::begin_selector(None, None, callback, state)
    }

    pub fn BeginShowSelectorWithSizeInBytesAndDirectoryCountAndCallbackAndState(
        sizeInBytes: i32,
        directoryCount: i32,
        callback: Option<StorageAsyncCallback>,
        state: StorageAsyncState,
    ) -> Result<StorageAsyncResult> {
        Self::begin_selector(None, Some((sizeInBytes, directoryCount)), callback, state)
    }

    fn begin_selector(
        player: Option<PlayerIndex>,
        space: Option<(i32, i32)>,
        callback: Option<StorageAsyncCallback>,
        state: StorageAsyncState,
    ) -> Result<StorageAsyncResult> {
        take_device_changed_error()?;
        if space.is_some_and(|(size, _)| size < 0) {
            return Err(CnaError::InvalidInput(
                "storage selector size must not be negative",
            ));
        }
        // XNA Windows validates size but intentionally does not reject a negative
        // directoryCount. CNA rejects it, so preserve the XNA observable rule by
        // using its documented minimum requirement for the native selector.
        let space = space.map(|(size, directories)| (size, directories.max(0)));
        let native = Native::load()?;
        let handle = native.select_storage_device(player.map(player_index), space)?;
        let device = Self::from_handle(native, handle);
        complete_callback(
            StorageAsyncResult::completed(StorageAsyncValue::Device(device), state, 0),
            callback,
        )
    }

    pub fn EndShowSelector(result: &StorageAsyncResult) -> Result<Self> {
        match result.take()? {
            StorageAsyncValue::Device(device) => Ok(device),
            StorageAsyncValue::Container(_) => Err(CnaError::InvalidInput(
                "result did not originate from BeginShowSelector",
            )),
        }
    }

    pub fn BeginOpenContainer(
        &self,
        displayName: &str,
        callback: Option<StorageAsyncCallback>,
        state: StorageAsyncState,
    ) -> Result<StorageAsyncResult> {
        validate_container_name(displayName)?;
        let handle = self
            .state
            .native
            .open_storage_container(self.state.handle()?, displayName)?;
        let display_name = match self.state.native.storage_container_display_name(handle) {
            Ok(display_name) => display_name,
            Err(error) => {
                let _ = self.state.native.destroy_storage_container(handle);
                return Err(error);
            }
        };
        let container = StorageContainer::from_handle(handle, self.clone(), display_name)?;
        complete_callback(
            StorageAsyncResult::completed(
                StorageAsyncValue::Container(container),
                state,
                self.state.id,
            ),
            callback,
        )
    }

    pub fn EndOpenContainer(&self, result: &StorageAsyncResult) -> Result<StorageContainer> {
        if result.inner.origin != self.state.id {
            return Err(CnaError::InvalidInput(
                "result belongs to another storage device",
            ));
        }
        match result.take()? {
            StorageAsyncValue::Container(container) => Ok(container),
            StorageAsyncValue::Device(_) => Err(CnaError::InvalidInput(
                "result did not originate from BeginOpenContainer",
            )),
        }
    }

    pub fn DeleteContainer(&self, titleName: &str) -> Result<()> {
        validate_container_name(titleName)?;
        self.state
            .native
            .delete_storage_container(self.state.handle()?, titleName)
    }
}

fn player_index(value: PlayerIndex) -> sys::CNA_PlayerIndex {
    value as u32
}

static DEVICE_CHANGED_HANDLERS: OnceLock<EventHandlers<EventArgs>> = OnceLock::new();
static DEVICE_CHANGED_BRIDGE: OnceLock<Mutex<Option<DeviceChangedBridge>>> = OnceLock::new();
static DEVICE_CHANGED_ERROR: OnceLock<Mutex<Option<CnaError>>> = OnceLock::new();
static DEVICE_CHANGED_OWNER: OnceLock<Mutex<Option<ThreadId>>> = OnceLock::new();
static DEVICE_CHANGED_PENDING: AtomicBool = AtomicBool::new(false);

struct DeviceChangedBridge {
    native: Arc<Native>,
    registration: sys::CNA_Handle,
}

impl DeviceChangedBridge {
    fn release(&mut self) -> Result<()> {
        if self.registration != sys::CNA_INVALID_HANDLE {
            self.native
                .unsubscribe_storage_device_changed(self.registration)?;
            self.registration = sys::CNA_INVALID_HANDLE;
        }
        Ok(())
    }
}

impl Drop for DeviceChangedBridge {
    fn drop(&mut self) {
        if let Err(error) = self.release() {
            record_device_changed_error(error);
        }
    }
}

fn device_changed_handlers() -> &'static EventHandlers<EventArgs> {
    DEVICE_CHANGED_HANDLERS.get_or_init(EventHandlers::new)
}

fn device_changed_bridge() -> &'static Mutex<Option<DeviceChangedBridge>> {
    DEVICE_CHANGED_BRIDGE.get_or_init(|| Mutex::new(None))
}

fn device_changed_error() -> &'static Mutex<Option<CnaError>> {
    DEVICE_CHANGED_ERROR.get_or_init(|| Mutex::new(None))
}

fn device_changed_owner() -> &'static Mutex<Option<ThreadId>> {
    DEVICE_CHANGED_OWNER.get_or_init(|| Mutex::new(None))
}

fn record_device_changed_error(error: CnaError) {
    let mut pending = device_changed_error()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if pending.is_none() {
        *pending = Some(error);
    }
}

fn take_device_changed_error() -> Result<()> {
    pump_device_changed();
    device_changed_error()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
        .map_or(Ok(()), Err)
}

fn emit_device_changed() {
    if device_changed_handlers().emit(&(), EventArgs) {
        record_device_changed_error(CnaError::Callback(
            "StorageDevice.DeviceChanged handler panicked".to_owned(),
        ));
    }
}

fn pump_device_changed() {
    if !DEVICE_CHANGED_PENDING.swap(false, Ordering::AcqRel) {
        return;
    }
    let on_owner = device_changed_owner()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .is_some_and(|owner| *owner == std::thread::current().id());
    if on_owner {
        emit_device_changed();
    } else {
        DEVICE_CHANGED_PENDING.store(true, Ordering::Release);
    }
}

fn ensure_device_changed_subscription() {
    let mut bridge = device_changed_bridge()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if bridge.is_some() {
        return;
    }
    *device_changed_owner()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(std::thread::current().id());
    *bridge = (|| {
        let native = Native::load().ok()?;
        let context = (device_changed_handlers() as *const EventHandlers<EventArgs>)
            .cast_mut()
            .cast::<c_void>();
        let registration = native
            .subscribe_storage_device_changed(storage_device_changed, context)
            .ok()?;
        Some(DeviceChangedBridge {
            native,
            registration,
        })
    })();
    if bridge.is_none() {
        *device_changed_owner()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }
}

unsafe extern "C" fn storage_device_changed(context: *mut c_void) {
    if context.is_null() {
        return;
    }
    // SAFETY: context points at the process-lifetime OnceLock value. Its
    // identity is validated here; dispatch uses the same global registry.
    let handlers = unsafe { &*context.cast::<EventHandlers<EventArgs>>() };
    if !core::ptr::eq(handlers, device_changed_handlers()) {
        record_device_changed_error(CnaError::Callback(
            "StorageDevice.DeviceChanged received an invalid callback context".to_owned(),
        ));
        return;
    }
    let on_owner = device_changed_owner()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .is_some_and(|owner| *owner == std::thread::current().id());
    if on_owner {
        emit_device_changed();
    } else {
        DEVICE_CHANGED_PENDING.store(true, Ordering::Release);
    }
}

struct StorageContainerState {
    native: Arc<Native>,
    handle: Mutex<sys::CNA_StorageContainerHandle>,
    device: StorageDevice,
    display_name: String,
    disposed: AtomicBool,
    native_dispose_observed: AtomicBool,
    disposing_registration: Mutex<sys::CNA_Handle>,
    streams: Mutex<Vec<Weak<StorageStreamState>>>,
    disposing: EventHandlers<EventArgs>,
}

impl StorageContainerState {
    fn handle(&self) -> Result<sys::CNA_StorageContainerHandle> {
        take_device_changed_error()?;
        if self.disposed.load(Ordering::Acquire) {
            return Err(CnaError::InvalidInput("storage container is disposed"));
        }
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            Err(CnaError::InvalidInput("storage container is disposed"))
        } else {
            Ok(handle)
        }
    }

    fn close_streams(&self) -> Result<()> {
        let streams = self
            .streams
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>();
        for stream in streams {
            stream.close()?;
        }
        Ok(())
    }

    fn destroy_handle(&self) -> Result<()> {
        let mut handle = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *handle != sys::CNA_INVALID_HANDLE {
            self.native.destroy_storage_container(*handle)?;
            *handle = sys::CNA_INVALID_HANDLE;
        }
        Ok(())
    }

    fn release_disposing_registration(&self) -> Result<()> {
        let mut registration = self
            .disposing_registration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *registration != sys::CNA_INVALID_HANDLE {
            self.native
                .unsubscribe_storage_container_disposing(*registration)?;
            *registration = sys::CNA_INVALID_HANDLE;
        }
        Ok(())
    }
}

impl Drop for StorageContainerState {
    fn drop(&mut self) {
        let _ = self.close_streams();
        let handle = *self
            .handle
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle != sys::CNA_INVALID_HANDLE && !self.disposed.load(Ordering::Acquire) {
            let _ = self.native.dispose_storage_container(handle);
        }
        let registration = *self
            .disposing_registration
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if registration != sys::CNA_INVALID_HANDLE
            && self
                .native
                .unsubscribe_storage_container_disposing(registration)
                .is_ok()
        {
            *self
                .disposing_registration
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = sys::CNA_INVALID_HANDLE;
        }
        if handle != sys::CNA_INVALID_HANDLE
            && self.native.destroy_storage_container(handle).is_ok()
        {
            *self
                .handle
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = sys::CNA_INVALID_HANDLE;
        }
    }
}

#[derive(Clone)]
pub struct StorageContainer {
    state: Arc<StorageContainerState>,
}

impl StorageContainer {
    fn from_handle(
        handle: sys::CNA_StorageContainerHandle,
        device: StorageDevice,
        display_name: String,
    ) -> Result<Self> {
        let state = Arc::new(StorageContainerState {
            native: Arc::clone(&device.state.native),
            handle: Mutex::new(handle),
            device,
            display_name,
            disposed: AtomicBool::new(false),
            native_dispose_observed: AtomicBool::new(false),
            disposing_registration: Mutex::new(sys::CNA_INVALID_HANDLE),
            streams: Mutex::new(Vec::new()),
            disposing: EventHandlers::new(),
        });
        let context = Arc::as_ptr(&state).cast_mut().cast::<c_void>();
        let registration = state.native.subscribe_storage_container_disposing(
            handle,
            storage_container_disposing,
            context,
        )?;
        *state
            .disposing_registration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = registration;
        Ok(Self { state })
    }

    pub fn DisplayName(&self) -> Result<String> {
        let _ = self.state.handle()?;
        Ok(self.state.display_name.clone())
    }

    pub fn StorageDevice(&self) -> Result<&StorageDevice> {
        let _ = self.state.handle()?;
        Ok(&self.state.device)
    }

    #[must_use]
    pub fn IsDisposed(&self) -> bool {
        self.state.disposed.load(Ordering::Acquire)
    }

    pub fn AddDisposingHandler(&self, handler: Box<dyn EventHandler>) -> u64 {
        self.state.disposing.add(handler)
    }

    pub fn RemoveDisposingHandler(&self, registration: u64) -> bool {
        self.state.disposing.remove(registration)
    }

    pub fn DirectoryExists(&self, directory: &str) -> Result<bool> {
        validate_storage_path(directory)?;
        self.state
            .native
            .storage_directory_exists(self.state.handle()?, directory)
    }

    pub fn FileExists(&self, file: &str) -> Result<bool> {
        validate_storage_path(file)?;
        self.state
            .native
            .storage_file_exists(self.state.handle()?, file)
    }

    pub fn CreateDirectory(&self, directory: &str) -> Result<()> {
        validate_storage_path(directory)?;
        self.state
            .native
            .create_storage_directory(self.state.handle()?, directory)
    }

    pub fn CreateFile(&self, file: &str) -> Result<StorageStream> {
        validate_storage_path(file)?;
        let handle = self
            .state
            .native
            .create_storage_file(self.state.handle()?, file)?;
        Ok(StorageStream::from_handle(handle, &self.state))
    }

    pub fn DeleteDirectory(&self, directory: &str) -> Result<()> {
        validate_storage_path(directory)?;
        self.state
            .native
            .delete_storage_directory(self.state.handle()?, directory)
    }

    pub fn OpenFile(&self, file: &str, fileMode: FileMode) -> Result<StorageStream> {
        self.open_file(file, fileMode, None, None)
    }

    pub fn OpenFileWithFileAndFileModeAndFileAccess(
        &self,
        file: &str,
        fileMode: FileMode,
        fileAccess: FileAccess,
    ) -> Result<StorageStream> {
        self.open_file(file, fileMode, Some(fileAccess), None)
    }

    pub fn OpenFileWithFileAndFileModeAndFileAccessAndFileShare(
        &self,
        file: &str,
        fileMode: FileMode,
        fileAccess: FileAccess,
        fileShare: FileShare,
    ) -> Result<StorageStream> {
        self.open_file(file, fileMode, Some(fileAccess), Some(fileShare))
    }

    fn open_file(
        &self,
        file: &str,
        mode: FileMode,
        access: Option<FileAccess>,
        share: Option<FileShare>,
    ) -> Result<StorageStream> {
        validate_storage_path(file)?;
        let handle = self.state.native.open_storage_file(
            self.state.handle()?,
            file,
            mode as u32,
            access.map(|value| value as u32),
            share.map(FileShare::bits),
        )?;
        Ok(StorageStream::from_handle(handle, &self.state))
    }

    pub fn DeleteFile(&self, file: &str) -> Result<()> {
        validate_storage_path(file)?;
        self.state
            .native
            .delete_storage_file(self.state.handle()?, file)
    }

    pub fn GetDirectoryNames(&self) -> Result<Vec<String>> {
        self.state
            .native
            .storage_directory_names(self.state.handle()?, "")
    }

    pub fn GetDirectoryNamesWithSearchPattern(&self, searchPattern: &str) -> Result<Vec<String>> {
        validate_search_pattern(searchPattern)?;
        self.state
            .native
            .storage_directory_names(self.state.handle()?, searchPattern)
    }

    pub fn GetFileNames(&self) -> Result<Vec<String>> {
        self.state
            .native
            .storage_file_names(self.state.handle()?, "")
    }

    pub fn GetFileNamesWithSearchPattern(&self, searchPattern: &str) -> Result<Vec<String>> {
        validate_search_pattern(searchPattern)?;
        self.state
            .native
            .storage_file_names(self.state.handle()?, searchPattern)
    }

    pub fn Finalize(&self) {}

    pub fn Dispose(&mut self) -> Result<()> {
        if !self.state.disposed.load(Ordering::Acquire) {
            self.state.close_streams()?;
            let handle = self.state.handle()?;
            self.state.native.dispose_storage_container(handle)?;
            if !self.state.native_dispose_observed.load(Ordering::Acquire) {
                return Err(CnaError::Native {
                    code: sys::CNA_RESULT_INVALID_STATE,
                    category: ErrorCategory::None,
                    message: "CNA did not synchronously raise StorageContainer.Disposing"
                        .to_owned(),
                });
            }
            self.state.disposed.store(true, Ordering::Release);
            let panicked = self.state.disposing.emit(self, EventArgs);
            self.state.release_disposing_registration()?;
            if panicked {
                self.state.destroy_handle()?;
                return Err(CnaError::Callback(
                    "storage container Disposing handler panicked".to_owned(),
                ));
            }
            self.state.destroy_handle()
        } else {
            self.state.release_disposing_registration()?;
            self.state.destroy_handle()
        }
    }
}

unsafe extern "C" fn storage_container_disposing(context: *mut c_void) {
    if context.is_null() {
        return;
    }
    // SAFETY: the registration holds a context pointing at its owning Arc allocation.
    let state = unsafe { &*context.cast::<StorageContainerState>() };
    state.native_dispose_observed.store(true, Ordering::Release);
}

impl Drop for StorageContainer {
    fn drop(&mut self) {
        if Arc::strong_count(&self.state) == 1 {
            let _ = self.Dispose();
        }
    }
}

struct StorageStreamState {
    native: Arc<Native>,
    handle: Mutex<sys::CNA_StorageStreamHandle>,
    _container: Arc<StorageContainerState>,
}

impl StorageStreamState {
    fn handle(&self) -> Result<sys::CNA_StorageStreamHandle> {
        take_device_changed_error()?;
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            Err(CnaError::InvalidInput("storage stream is closed"))
        } else {
            Ok(handle)
        }
    }

    fn close(&self) -> Result<()> {
        let mut handle = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *handle != sys::CNA_INVALID_HANDLE {
            self.native.close_storage_stream(*handle)?;
            *handle = sys::CNA_INVALID_HANDLE;
        }
        Ok(())
    }
}

impl Drop for StorageStreamState {
    fn drop(&mut self) {
        let handle = *self
            .handle
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle != sys::CNA_INVALID_HANDLE && self.native.close_storage_stream(handle).is_ok() {
            *self
                .handle
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = sys::CNA_INVALID_HANDLE;
        }
    }
}

pub struct StorageStream {
    state: Arc<StorageStreamState>,
}

impl StorageStream {
    /// The native handle, for a route that reads this stream in place.
    ///
    /// `content_readers.h`'s `ContentReader` borrows a stream for its whole
    /// life and closes it when destroyed; the handle itself stays this value's
    /// to close, and `cna_storage_stream_close` is idempotent, so the two do
    /// not fight.
    pub(crate) fn native_handle(&self) -> Result<sys::CNA_StorageStreamHandle> {
        self.state.handle()
    }

    fn from_handle(
        handle: sys::CNA_StorageStreamHandle,
        container: &Arc<StorageContainerState>,
    ) -> Self {
        let state = Arc::new(StorageStreamState {
            native: Arc::clone(&container.native),
            handle: Mutex::new(handle),
            _container: Arc::clone(container),
        });
        container
            .streams
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(Arc::downgrade(&state));
        Self { state }
    }

    pub fn CanRead(&self) -> Result<bool> {
        self.state
            .native
            .storage_stream_can_read(self.state.handle()?)
    }

    pub fn CanWrite(&self) -> Result<bool> {
        self.state
            .native
            .storage_stream_can_write(self.state.handle()?)
    }

    pub fn CanSeek(&self) -> Result<bool> {
        self.state
            .native
            .storage_stream_can_seek(self.state.handle()?)
    }

    pub fn Length(&self) -> Result<i64> {
        self.state
            .native
            .storage_stream_length(self.state.handle()?)
    }

    pub fn Position(&self) -> Result<i64> {
        self.state
            .native
            .storage_stream_position(self.state.handle()?)
    }

    pub fn SetLength(&mut self, value: i64) -> Result<()> {
        self.state
            .native
            .set_storage_stream_length(self.state.handle()?, value)
    }

    pub fn Close(&mut self) -> Result<()> {
        self.state.close()
    }
}

impl Read for StorageStream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.state
            .native
            .storage_stream_read(self.state.handle().map_err(io_error)?, buffer)
            .map_err(io_error)
    }
}

impl Write for StorageStream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.state
            .native
            .storage_stream_write(self.state.handle().map_err(io_error)?, buffer)
            .map_err(io_error)?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.state
            .native
            .flush_storage_stream(self.state.handle().map_err(io_error)?)
            .map_err(io_error)
    }
}

impl Seek for StorageStream {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let (offset, origin) = match position {
            SeekFrom::Start(value) => (
                i64::try_from(value).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "seek offset exceeds i64")
                })?,
                sys::CNA_SEEK_ORIGIN_BEGIN,
            ),
            SeekFrom::Current(value) => (value, sys::CNA_SEEK_ORIGIN_CURRENT),
            SeekFrom::End(value) => (value, sys::CNA_SEEK_ORIGIN_END),
        };
        let value = self
            .state
            .native
            .storage_stream_seek(self.state.handle().map_err(io_error)?, offset, origin)
            .map_err(io_error)?;
        u64::try_from(value).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "CNA returned a negative position",
            )
        })
    }
}

impl Drop for StorageStream {
    fn drop(&mut self) {
        let _ = self.state.close();
    }
}

fn io_error(error: CnaError) -> io::Error {
    io::Error::new(io::ErrorKind::Other, error)
}

fn validate_container_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains(['/', '\\'])
        || name.contains(':')
    {
        return Err(CnaError::InvalidInput(
            "storage container name must be one non-empty relative component",
        ));
    }
    Ok(())
}

fn validate_storage_path(path: &str) -> Result<()> {
    if path.is_empty() || path.starts_with(['/', '\\']) || path.as_bytes().get(1) == Some(&b':') {
        return Err(CnaError::InvalidInput(
            "storage path must be non-empty and relative",
        ));
    }
    let mut depth = 0_usize;
    for component in path.split(['/', '\\']) {
        match component {
            "" | "." => {}
            ".." if depth == 0 => {
                return Err(CnaError::InvalidInput("storage path escapes its container"));
            }
            ".." => depth -= 1,
            value if value.contains(':') => {
                return Err(CnaError::InvalidInput(
                    "storage path contains an absolute-path prefix",
                ));
            }
            _ => depth += 1,
        }
    }
    if depth == 0 {
        return Err(CnaError::InvalidInput(
            "storage path must identify a child of the container",
        ));
    }
    Ok(())
}

fn validate_search_pattern(pattern: &str) -> Result<()> {
    if pattern.is_empty()
        || pattern.contains(['/', '\\'])
        || pattern == "."
        || pattern == ".."
        || pattern.contains(':')
    {
        return Err(CnaError::InvalidInput(
            "storage search pattern must be one non-empty relative component",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageDeviceNotConnectedException {
    message: String,
    inner_message: Option<String>,
    streaming_context: Option<i32>,
}

impl StorageDeviceNotConnectedException {
    #[must_use]
    pub fn new() -> Self {
        Self {
            message: "The storage device is not connected.".to_owned(),
            inner_message: None,
            streaming_context: None,
        }
    }

    #[must_use]
    pub fn from_message(message: &str) -> Self {
        Self {
            message: message.to_owned(),
            inner_message: None,
            streaming_context: None,
        }
    }

    #[must_use]
    pub fn from_info_and_context(info: SerializationInfo, context: StreamingContext) -> Self {
        Self {
            message: info.message,
            inner_message: None,
            streaming_context: Some(context.state),
        }
    }

    #[must_use]
    pub fn from_message_and_inner_exception(message: &str, innerException: &dyn Error) -> Self {
        Self {
            message: message.to_owned(),
            inner_message: Some(innerException.to_string()),
            streaming_context: None,
        }
    }
}

impl Default for StorageDeviceNotConnectedException {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for StorageDeviceNotConnectedException {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)?;
        if let Some(inner) = &self.inner_message {
            write!(formatter, ": {inner}")?;
        }
        Ok(())
    }
}

impl Error for StorageDeviceNotConnectedException {}

#[cfg(test)]
mod tests {
    use core::ffi::c_void;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use crate::error::CnaError;
    use crate::extensions::events::EventArgs;

    use super::{
        device_changed_handlers, device_changed_owner, storage_device_changed,
        take_device_changed_error, validate_search_pattern, validate_storage_path, EventHandlers,
    };

    #[test]
    fn containment_rejects_escapes_and_accepts_normalized_children() {
        for invalid in [
            "",
            "../escape",
            "a/../../escape",
            "/absolute",
            "C:\\escape",
            "\\\\server\\share",
        ] {
            assert!(validate_storage_path(invalid).is_err(), "{invalid}");
        }
        for valid in [
            "save.bin",
            "nested/child",
            "nested/./child",
            "nested//child",
            "a/../child",
            "mixed\\child",
        ] {
            assert!(validate_storage_path(valid).is_ok(), "{valid}");
        }
        assert!(validate_search_pattern("*.bin").is_ok());
        assert!(validate_search_pattern("nested/*.bin").is_err());
    }

    #[test]
    fn device_changed_panics_and_off_owner_dispatch_use_a_safe_boundary() {
        let handlers = device_changed_handlers();
        *device_changed_owner()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(std::thread::current().id());
        let registration = handlers.add(Box::new(|_: &dyn std::any::Any, _: EventArgs| {
            panic!("intentional DeviceChanged panic")
        }));
        let context = (handlers as *const EventHandlers<EventArgs>)
            .cast_mut()
            .cast::<c_void>();
        // SAFETY: this invokes the native callback shape synchronously with its
        // process-lifetime handler registry context.
        unsafe { storage_device_changed(context) };
        assert!(matches!(
            take_device_changed_error(),
            Err(CnaError::Callback(_))
        ));
        assert!(handlers.remove(registration));

        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let registration = handlers.add(Box::new(move |_: &dyn std::any::Any, _: EventArgs| {
            observed.fetch_add(1, Ordering::SeqCst);
        }));
        let context_address = context as usize;
        std::thread::spawn(move || {
            // SAFETY: the integer reconstructs the same process-lifetime
            // handler context solely for this synchronous callback exercise.
            unsafe { storage_device_changed(context_address as *mut c_void) };
        })
        .join()
        .expect("off-owner DeviceChanged callback");
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        take_device_changed_error().expect("owner-thread DeviceChanged pump");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(handlers.remove(registration));
        *device_changed_owner()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }
}
