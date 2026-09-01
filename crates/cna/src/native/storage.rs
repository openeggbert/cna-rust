//! Native XNA Storage calls over the reviewed CNA ABI 0.20 slice.

use core::ffi::c_void;

use cna_sys as sys;

use crate::error::{CnaError, ErrorCategory, Result};

use super::Native;

fn view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast(),
        byte_length: value.len() as u64,
    }
}

impl Native {
    pub(crate) fn select_storage_device(
        &self,
        player: Option<sys::CNA_PlayerIndex>,
        space: Option<(i32, i32)>,
    ) -> Result<sys::CNA_StorageDeviceHandle> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        let result = unsafe {
            // SAFETY: all callbacks are deliberately absent; the output lives for the call.
            match (player, space) {
                (None, None) => {
                    (self.storage_device_show_selector)(None, core::ptr::null_mut(), &mut handle)
                }
                (Some(player), None) => (self.storage_device_show_selector_for_player)(
                    player,
                    None,
                    core::ptr::null_mut(),
                    &mut handle,
                ),
                (None, Some((size, directories))) => {
                    (self.storage_device_show_selector_with_space)(
                        size,
                        directories,
                        None,
                        core::ptr::null_mut(),
                        &mut handle,
                    )
                }
                (Some(player), Some((size, directories))) => (self
                    .storage_device_show_selector_for_player_with_space)(
                    player,
                    size,
                    directories,
                    None,
                    core::ptr::null_mut(),
                    &mut handle,
                ),
            }
        };
        self.check(result)?;
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput(
                "CNA returned an invalid storage device",
            ));
        }
        Ok(handle)
    }

    pub(crate) fn storage_device_free_space(
        &self,
        device: sys::CNA_StorageDeviceHandle,
    ) -> Result<i64> {
        let mut value = 0;
        // SAFETY: the owned device and output are live.
        self.check(unsafe { (self.storage_device_get_free_space)(device, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn storage_device_is_connected(
        &self,
        device: sys::CNA_StorageDeviceHandle,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the owned device and output are live.
        self.check(unsafe { (self.storage_device_get_is_connected)(device, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn storage_device_total_space(
        &self,
        device: sys::CNA_StorageDeviceHandle,
    ) -> Result<i64> {
        let mut value = 0;
        // SAFETY: the owned device and output are live.
        self.check(unsafe { (self.storage_device_get_total_space)(device, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn delete_storage_container(
        &self,
        device: sys::CNA_StorageDeviceHandle,
        name: &str,
    ) -> Result<()> {
        // SAFETY: CNA copies the UTF-8 view during the call.
        self.check(unsafe { (self.storage_device_delete_container)(device, view(name)) })
    }

    pub(crate) fn subscribe_storage_device_changed(
        &self,
        callback: unsafe extern "C" fn(*mut c_void),
        context: *mut c_void,
    ) -> Result<sys::CNA_Handle> {
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: the callback has C ABI and the caller keeps context alive for registration life.
        self.check(unsafe {
            (self.storage_device_subscribe_device_changed)(
                Some(callback),
                context,
                &mut registration,
            )
        })?;
        Ok(registration)
    }

    pub(crate) fn unsubscribe_storage_device_changed(
        &self,
        registration: sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: registration ownership is transferred back exactly once.
        self.check(unsafe { (self.storage_device_unsubscribe_device_changed)(registration) })
    }

    pub(crate) fn destroy_storage_device(
        &self,
        device: sys::CNA_StorageDeviceHandle,
    ) -> Result<()> {
        // SAFETY: device is an owned handle released exactly once on success.
        self.check(unsafe { (self.storage_device_destroy)(device) })
    }

    pub(crate) fn open_storage_container(
        &self,
        device: sys::CNA_StorageDeviceHandle,
        name: &str,
    ) -> Result<sys::CNA_StorageContainerHandle> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: CNA copies the view, no callback is supplied, and output is live.
        self.check(unsafe {
            (self.storage_container_open)(
                device,
                view(name),
                None,
                core::ptr::null_mut(),
                &mut handle,
            )
        })?;
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput(
                "CNA returned an invalid storage container",
            ));
        }
        Ok(handle)
    }

    pub(crate) fn storage_container_display_name(
        &self,
        container: sys::CNA_StorageContainerHandle,
    ) -> Result<String> {
        let mut count = 0;
        // SAFETY: the owned container and output are live.
        self.check(unsafe {
            (self.storage_container_get_display_name_size)(container, &mut count)
        })?;
        self.copy_storage_string(container, count, self.storage_container_copy_display_name)
    }

    fn copy_storage_string(
        &self,
        container: sys::CNA_StorageContainerHandle,
        count: u64,
        copy: sys::cna_storage_container_copy_display_name_fn,
    ) -> Result<String> {
        let capacity = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("storage string is too large"))?;
        let mut bytes = vec![0_u8; capacity];
        let mut copied = 0;
        // SAFETY: destination has count writable bytes.
        self.check(unsafe { copy(container, bytes.as_mut_ptr().cast(), count, &mut copied) })?;
        String::from_utf8(bytes).map_err(|_| CnaError::Native {
            code: sys::CNA_RESULT_ENCODING,
            category: ErrorCategory::None,
            message: "CNA returned non-UTF-8 storage text".to_owned(),
        })
    }

    pub(crate) fn dispose_storage_container(
        &self,
        container: sys::CNA_StorageContainerHandle,
    ) -> Result<()> {
        // SAFETY: container is an owned live handle; CNA disposal is idempotent.
        self.check(unsafe { (self.storage_container_dispose)(container) })
    }

    pub(crate) fn subscribe_storage_container_disposing(
        &self,
        container: sys::CNA_StorageContainerHandle,
        callback: unsafe extern "C" fn(*mut c_void),
        context: *mut c_void,
    ) -> Result<sys::CNA_Handle> {
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: callback and context remain live until the returned registration is released.
        self.check(unsafe {
            (self.storage_container_subscribe_disposing)(
                container,
                Some(callback),
                context,
                &mut registration,
            )
        })?;
        Ok(registration)
    }

    pub(crate) fn unsubscribe_storage_container_disposing(
        &self,
        registration: sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: registration is owned and transferred back exactly once.
        self.check(unsafe { (self.storage_container_unsubscribe_disposing)(registration) })
    }

    pub(crate) fn destroy_storage_container(
        &self,
        container: sys::CNA_StorageContainerHandle,
    ) -> Result<()> {
        // SAFETY: container ownership is transferred exactly once on success.
        self.check(unsafe { (self.storage_container_destroy)(container) })
    }

    pub(crate) fn storage_path_call(
        &self,
        container: sys::CNA_StorageContainerHandle,
        path: &str,
        call: sys::cna_storage_container_create_directory_fn,
    ) -> Result<()> {
        // SAFETY: CNA copies the view during the call.
        self.check(unsafe { call(container, view(path)) })
    }

    pub(crate) fn storage_path_query(
        &self,
        container: sys::CNA_StorageContainerHandle,
        path: &str,
        call: sys::cna_storage_container_directory_exists_fn,
    ) -> Result<bool> {
        let mut exists = sys::CNA_FALSE;
        // SAFETY: CNA copies the view and output is live.
        self.check(unsafe { call(container, view(path), &mut exists) })?;
        Ok(exists != sys::CNA_FALSE)
    }

    pub(crate) fn create_storage_directory(
        &self,
        container: sys::CNA_StorageContainerHandle,
        path: &str,
    ) -> Result<()> {
        self.storage_path_call(container, path, self.storage_container_create_directory)
    }

    pub(crate) fn delete_storage_directory(
        &self,
        container: sys::CNA_StorageContainerHandle,
        path: &str,
    ) -> Result<()> {
        self.storage_path_call(container, path, self.storage_container_delete_directory)
    }

    pub(crate) fn storage_directory_exists(
        &self,
        container: sys::CNA_StorageContainerHandle,
        path: &str,
    ) -> Result<bool> {
        self.storage_path_query(container, path, self.storage_container_directory_exists)
    }

    pub(crate) fn delete_storage_file(
        &self,
        container: sys::CNA_StorageContainerHandle,
        path: &str,
    ) -> Result<()> {
        self.storage_path_call(container, path, self.storage_container_delete_file)
    }

    pub(crate) fn storage_file_exists(
        &self,
        container: sys::CNA_StorageContainerHandle,
        path: &str,
    ) -> Result<bool> {
        self.storage_path_query(container, path, self.storage_container_file_exists)
    }

    pub(crate) fn storage_names(
        &self,
        container: sys::CNA_StorageContainerHandle,
        pattern: &str,
        count_call: sys::cna_storage_container_get_directory_name_count_fn,
        copy_call: sys::cna_storage_container_copy_directory_name_fn,
    ) -> Result<Vec<String>> {
        let pattern = view(pattern);
        let mut count = 0;
        // SAFETY: pattern is borrowed for the call and output is live.
        self.check(unsafe { count_call(container, pattern, &mut count) })?;
        let capacity = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("storage name count is too large"))?;
        let mut names = Vec::with_capacity(capacity);
        for index in 0..count {
            let mut required = 0;
            // SAFETY: capacity zero permits null destination; output is live.
            let result = unsafe {
                copy_call(
                    container,
                    pattern,
                    index,
                    core::ptr::null_mut(),
                    0,
                    &mut required,
                )
            };
            if result != sys::CNA_RESULT_BUFFER_TOO_SMALL && result != sys::CNA_RESULT_SUCCESS {
                self.check(result)?;
            }
            let byte_count = usize::try_from(required)
                .map_err(|_| CnaError::InvalidInput("storage name is too large"))?;
            let mut bytes = vec![0_u8; byte_count];
            let mut copied = 0;
            // SAFETY: destination has required writable bytes.
            self.check(unsafe {
                copy_call(
                    container,
                    pattern,
                    index,
                    bytes.as_mut_ptr().cast(),
                    required,
                    &mut copied,
                )
            })?;
            names.push(String::from_utf8(bytes).map_err(|_| CnaError::Native {
                code: sys::CNA_RESULT_ENCODING,
                category: ErrorCategory::None,
                message: "CNA returned a non-UTF-8 storage name".to_owned(),
            })?);
        }
        Ok(names)
    }

    pub(crate) fn storage_directory_names(
        &self,
        container: sys::CNA_StorageContainerHandle,
        pattern: &str,
    ) -> Result<Vec<String>> {
        self.storage_names(
            container,
            pattern,
            self.storage_container_get_directory_name_count,
            self.storage_container_copy_directory_name,
        )
    }

    pub(crate) fn storage_file_names(
        &self,
        container: sys::CNA_StorageContainerHandle,
        pattern: &str,
    ) -> Result<Vec<String>> {
        self.storage_names(
            container,
            pattern,
            self.storage_container_get_file_name_count,
            self.storage_container_copy_file_name,
        )
    }

    pub(crate) fn create_storage_file(
        &self,
        container: sys::CNA_StorageContainerHandle,
        file: &str,
    ) -> Result<sys::CNA_StorageStreamHandle> {
        let mut stream = sys::CNA_INVALID_HANDLE;
        // SAFETY: CNA copies the view and output is live.
        self.check(unsafe {
            (self.storage_container_create_file)(container, view(file), &mut stream)
        })?;
        Ok(stream)
    }

    pub(crate) fn open_storage_file(
        &self,
        container: sys::CNA_StorageContainerHandle,
        file: &str,
        mode: sys::CNA_FileMode,
        access: Option<sys::CNA_FileAccess>,
        share: Option<sys::CNA_FileShare>,
    ) -> Result<sys::CNA_StorageStreamHandle> {
        let mut stream = sys::CNA_INVALID_HANDLE;
        let result = unsafe {
            // SAFETY: CNA copies the view, validates enum identities, and output is live.
            match (access, share) {
                (None, None) => {
                    (self.storage_container_open_file)(container, view(file), mode, &mut stream)
                }
                (Some(access), None) => (self.storage_container_open_file_access)(
                    container,
                    view(file),
                    mode,
                    access,
                    &mut stream,
                ),
                (Some(access), Some(share)) => (self.storage_container_open_file_share)(
                    container,
                    view(file),
                    mode,
                    access,
                    share,
                    &mut stream,
                ),
                (None, Some(_)) => unreachable!("share requires access"),
            }
        };
        self.check(result)?;
        Ok(stream)
    }

    pub(crate) fn storage_stream_read(
        &self,
        stream: sys::CNA_StorageStreamHandle,
        bytes: &mut [u8],
    ) -> Result<usize> {
        let mut read = 0;
        // SAFETY: destination is writable for its reported length.
        self.check(unsafe {
            (self.storage_stream_read)(stream, bytes.as_mut_ptr(), bytes.len() as u64, &mut read)
        })?;
        usize::try_from(read).map_err(|_| CnaError::InvalidInput("stream read count exceeds usize"))
    }

    pub(crate) fn storage_stream_write(
        &self,
        stream: sys::CNA_StorageStreamHandle,
        bytes: &[u8],
    ) -> Result<()> {
        // SAFETY: CNA copies exactly bytes.len() readable bytes.
        self.check(unsafe {
            (self.storage_stream_write)(stream, bytes.as_ptr(), bytes.len() as u64)
        })
    }

    pub(crate) fn storage_stream_seek(
        &self,
        stream: sys::CNA_StorageStreamHandle,
        offset: i64,
        origin: sys::CNA_SeekOrigin,
    ) -> Result<i64> {
        let mut position = 0;
        // SAFETY: stream is owned and output is live.
        self.check(unsafe { (self.storage_stream_seek)(stream, offset, origin, &mut position) })?;
        Ok(position)
    }

    pub(crate) fn storage_stream_length(
        &self,
        stream: sys::CNA_StorageStreamHandle,
    ) -> Result<i64> {
        let mut value = 0;
        // SAFETY: stream is owned and output is live.
        self.check(unsafe { (self.storage_stream_get_length)(stream, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn storage_stream_position(
        &self,
        stream: sys::CNA_StorageStreamHandle,
    ) -> Result<i64> {
        let mut value = 0;
        // SAFETY: stream is owned and output is live.
        self.check(unsafe { (self.storage_stream_get_position)(stream, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_storage_stream_length(
        &self,
        stream: sys::CNA_StorageStreamHandle,
        value: i64,
    ) -> Result<()> {
        // SAFETY: stream is owned and CNA validates length.
        self.check(unsafe { (self.storage_stream_set_length)(stream, value) })
    }

    pub(crate) fn storage_stream_capability(
        &self,
        stream: sys::CNA_StorageStreamHandle,
        call: sys::cna_storage_stream_get_can_read_fn,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: stream is owned and output is live.
        self.check(unsafe { call(stream, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn storage_stream_can_read(
        &self,
        stream: sys::CNA_StorageStreamHandle,
    ) -> Result<bool> {
        self.storage_stream_capability(stream, self.storage_stream_get_can_read)
    }

    pub(crate) fn storage_stream_can_write(
        &self,
        stream: sys::CNA_StorageStreamHandle,
    ) -> Result<bool> {
        self.storage_stream_capability(stream, self.storage_stream_get_can_write)
    }

    pub(crate) fn storage_stream_can_seek(
        &self,
        stream: sys::CNA_StorageStreamHandle,
    ) -> Result<bool> {
        self.storage_stream_capability(stream, self.storage_stream_get_can_seek)
    }

    pub(crate) fn flush_storage_stream(&self, stream: sys::CNA_StorageStreamHandle) -> Result<()> {
        // SAFETY: stream is owned and live.
        self.check(unsafe { (self.storage_stream_flush)(stream) })
    }

    pub(crate) fn close_storage_stream(&self, stream: sys::CNA_StorageStreamHandle) -> Result<()> {
        // SAFETY: stream ownership transfers exactly once on success.
        self.check(unsafe { (self.storage_stream_close)(stream) })
    }
}

/// The last of `storage.h`: where saves live, and what a container knows.
///
/// XNA's storage lands under a platform-decided per-title directory and gives a
/// game no say in either half. `set_app_name` and the root query are CNA's: the
/// name is what the directory is called, and the root is where it is. A tool
/// that wants to read a game's saves has no other way to find them.
impl Native {
    pub(crate) fn set_storage_app_name(&self, name: &str) -> Result<()> {
        let view = sys::CNA_StringView {
            data: name.as_ptr().cast(),
            byte_length: name.len() as u64,
        };
        // SAFETY: the name outlives the call, which is all the view borrows.
        self.check(unsafe { (self.storage_set_app_name_ext)(view) })
    }

    pub(crate) fn storage_root(&self) -> Result<String> {
        super::runtime::read_string(
            |result| self.check(result),
            // SAFETY: the output is the caller's live local.
            |out| unsafe { (self.storage_get_root_size_ext)(out) },
            // SAFETY: the destination has the capacity just measured.
            |destination, capacity, written| unsafe {
                (self.storage_copy_root_ext)(destination, capacity, written)
            },
        )
    }

    pub(crate) fn storage_container_is_disposed(
        &self,
        container: sys::CNA_StorageContainerHandle,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the container handle is live and the output is a local.
        self.check(unsafe { (self.storage_container_get_is_disposed)(container, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn storage_container_device(
        &self,
        container: sys::CNA_StorageContainerHandle,
    ) -> Result<Option<sys::CNA_StorageDeviceHandle>> {
        let mut value = sys::CNA_INVALID_HANDLE;
        // SAFETY: the container handle is live and the output is a local.
        self.check(unsafe {
            (self.storage_container_get_storage_device)(container, &mut value)
        })?;
        Ok((value != sys::CNA_INVALID_HANDLE).then_some(value))
    }
}
