//! `content_readers.h` -- the XNB reader and the process-wide reader registry.
//!
//! This is the extension point the reflective builder is *not*. That builder
//! takes a byte offset into an object the caller allocated and writes a value
//! kind's worth of bytes at it, with nothing checking either half, which is why
//! `reflective-reader-writes-at-caller-offsets` rules it out of this binding.
//! `cna_content_type_reader_manager_register` reaches the same capability the
//! other way round: the caller's callback is handed a borrowed `ContentReader`
//! and reads fields *from* it, returning whatever object it likes. Nothing
//! writes at an offset, so nothing can write at the wrong one.
//!
//! Two lifetimes matter here and they are different.
//!
//! * The reader a caller creates is **owned** and closes its stream when
//!   destroyed. The stream handle stays valid and is still the caller's to
//!   close.
//! * The reader handed to a read callback is **callback-scoped and borrowed**.
//!   The header says it is invalidated before the callback returns and has no
//!   destroy operation, so caching it fails with `CNA_RESULT_INVALID_HANDLE`
//!   rather than faulting -- which is what lets the safe layer model it as a
//!   lifetime-bound view instead of an owned value.

use cna_sys as sys;

use crate::error::{CnaError, Result};

use super::runtime::read_string;
use super::Native;

impl Native {
    pub(crate) fn create_content_reader(
        &self,
        info: &sys::CNA_ContentReaderCreateInfo,
    ) -> Result<sys::CNA_ContentReaderHandle> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the descriptor is a complete versioned local whose string
        // view borrows a name that outlives the call, and the output is a live
        // local.
        self.check(unsafe { (self.content_reader_create)(info, &mut handle) })?;
        Ok(handle)
    }

    pub(crate) fn destroy_content_reader(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<()> {
        // SAFETY: the handle came from `create_content_reader` and is
        // destroyed exactly once.
        self.check(unsafe { (self.content_reader_destroy)(handle) })
    }

    pub(crate) fn content_reader_manager(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<Option<sys::CNA_Handle>> {
        let mut value = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { (self.content_reader_get_content_manager)(handle, &mut value) })?;
        Ok((value != sys::CNA_INVALID_HANDLE).then_some(value))
    }

    pub(crate) fn content_reader_asset_name(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<String> {
        read_string(
            |result| self.check(result),
            // SAFETY: the handle is live and the output is the caller's local.
            |out| unsafe { (self.content_reader_get_asset_name_size)(handle, out) },
            // SAFETY: the destination has the capacity just measured.
            |destination, capacity, written| unsafe {
                (self.content_reader_copy_asset_name)(handle, destination, capacity, written)
            },
        )
    }

    pub(crate) fn content_reader_version(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<i32> {
        let mut value = 0;
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { (self.content_reader_get_version)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn content_reader_platform(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<u8> {
        let mut value = 0;
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { (self.content_reader_get_platform)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn content_reader_read_matrix(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<sys::CNA_Matrix> {
        let mut value = sys::CNA_Matrix::default();
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { (self.content_reader_read_matrix)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn content_reader_read_quaternion(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<sys::CNA_Quaternion> {
        let mut value = sys::CNA_Quaternion::default();
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { (self.content_reader_read_quaternion)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn content_reader_read_vector2(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<sys::CNA_Vector2> {
        let mut value = sys::CNA_Vector2::default();
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { (self.content_reader_read_vector2)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn content_reader_read_vector3(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<sys::CNA_Vector3> {
        let mut value = sys::CNA_Vector3::default();
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { (self.content_reader_read_vector3)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn content_reader_read_vector4(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<sys::CNA_Vector4> {
        let mut value = sys::CNA_Vector4::default();
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { (self.content_reader_read_vector4)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn content_reader_read_color(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<sys::CNA_Color> {
        let mut value = sys::CNA_Color::default();
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { (self.content_reader_read_color)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn content_reader_read_bounding_sphere(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<sys::CNA_BoundingSphere> {
        let mut value = sys::CNA_BoundingSphere::default();
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { (self.content_reader_read_bounding_sphere)(handle, &mut value) })?;
        Ok(value)
    }

    /// Reads the next object through the dispatch protocol and discards it.
    ///
    /// Answers whether the reference was non-null.
    pub(crate) fn content_reader_read_object_tag(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { (self.content_reader_read_object_tag)(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn content_reader_initialize_type_readers(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<()> {
        // SAFETY: the handle is live.
        self.check(unsafe { (self.content_reader_initialize_type_readers)(handle) })
    }

    pub(crate) fn content_reader_read_shared_resources(
        &self,
        handle: sys::CNA_ContentReaderHandle,
    ) -> Result<()> {
        // SAFETY: the handle is live.
        self.check(unsafe { (self.content_reader_read_shared_resources)(handle) })
    }

    pub(crate) fn content_reader_check_element_count(
        &self,
        handle: sys::CNA_ContentReaderHandle,
        count: i64,
        reader_name: &str,
    ) -> Result<()> {
        // SAFETY: the name outlives the call, which is all the view borrows.
        self.check(unsafe {
            (self.content_reader_check_collection_element_count)(
                handle,
                count,
                view(reader_name),
            )
        })
    }

    pub(crate) fn content_reader_check_decoded_size(
        &self,
        handle: sys::CNA_ContentReaderHandle,
        byte_size: i64,
        reader_name: &str,
    ) -> Result<()> {
        // SAFETY: the name outlives the call, which is all the view borrows.
        self.check(unsafe {
            (self.content_reader_check_decoded_byte_size)(handle, byte_size, view(reader_name))
        })
    }

    pub(crate) fn content_reader_read_bytes_exact(
        &self,
        handle: sys::CNA_ContentReaderHandle,
        count: i32,
        reader_name: &str,
    ) -> Result<Vec<u8>> {
        let capacity = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("a byte count cannot be negative"))?;
        let mut bytes = vec![0_u8; capacity];
        let mut written = 0_u64;
        // SAFETY: the destination has exactly `capacity` writable bytes and
        // the name outlives the call.
        self.check(unsafe {
            (self.content_reader_read_bytes_exact)(
                handle,
                count,
                view(reader_name),
                if capacity == 0 {
                    core::ptr::null_mut()
                } else {
                    bytes.as_mut_ptr()
                },
                capacity as u64,
                &mut written,
            )
        })?;
        bytes.truncate(usize::try_from(written).unwrap_or(0).min(capacity));
        Ok(bytes)
    }

    // --- the process-wide registry -------------------------------------------

    pub(crate) fn clear_content_type_readers(&self) -> Result<()> {
        // SAFETY: the route takes nothing.
        self.check(unsafe { (self.content_type_reader_manager_clear_type_creators)() })
    }

    pub(crate) fn content_type_reader_is_registered(&self, name: &str) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the name outlives the call and the output is a local.
        self.check(unsafe {
            (self.content_type_reader_manager_get_is_registered)(view(name), &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn create_content_type_reader(
        &self,
        name: &str,
    ) -> Result<sys::CNA_ContentTypeReaderHandle> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the name outlives the call and the output is a local.
        self.check(unsafe {
            (self.content_type_reader_manager_create_reader)(view(name), &mut handle)
        })?;
        Ok(handle)
    }

    pub(crate) fn register_content_type_reader(
        &self,
        name: &str,
        callbacks: &sys::CNA_ContentTypeReaderCallbacks,
    ) -> Result<sys::CNA_Handle> {
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: the name and the table outlive the call -- every pointer in
        // the table is copied by CNA -- and the `context` it keeps is the
        // caller's to hold alive until the registration is withdrawn, which is
        // what the safe layer's registration value does.
        self.check(unsafe {
            (self.content_type_reader_manager_register)(
                view(name),
                callbacks,
                &mut registration,
            )
        })?;
        Ok(registration)
    }

    pub(crate) fn unregister_content_type_reader(
        &self,
        registration: sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the registration came from the register above and is
        // withdrawn exactly once.
        self.check(unsafe { (self.content_type_reader_manager_unregister)(registration) })
    }

    pub(crate) fn register_known_unsupported_readers(&self) -> Result<()> {
        // SAFETY: the route takes nothing and is documented idempotent.
        self.check(unsafe { (self.content_register_known_unsupported_xnb_readers)() })
    }

    pub(crate) fn create_known_unsupported_reader(
        &self,
        target_type_name: &str,
        reason: sys::CNA_UnsupportedContentReaderReason,
    ) -> Result<sys::CNA_ContentTypeReaderHandle> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the name outlives the call and the output is a local.
        self.check(unsafe {
            (self.known_unsupported_content_type_reader_create)(
                view(target_type_name),
                reason,
                &mut handle,
            )
        })?;
        Ok(handle)
    }

    // --- one type reader ------------------------------------------------------

    pub(crate) fn type_reader_can_deserialize_into_existing(
        &self,
        handle: sys::CNA_ContentTypeReaderHandle,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe {
            (self.content_type_reader_get_can_deserialize_into_existing_object)(
                handle, &mut value,
            )
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn type_reader_target_type_name(
        &self,
        handle: sys::CNA_ContentTypeReaderHandle,
    ) -> Result<String> {
        read_string(
            |result| self.check(result),
            // SAFETY: the handle is live and the output is the caller's local.
            |out| unsafe { (self.content_type_reader_get_target_type_name_size)(handle, out) },
            // SAFETY: the destination has the capacity just measured.
            |destination, capacity, written| unsafe {
                (self.content_type_reader_copy_target_type_name)(
                    handle,
                    destination,
                    capacity,
                    written,
                )
            },
        )
    }

    pub(crate) fn type_reader_version(
        &self,
        handle: sys::CNA_ContentTypeReaderHandle,
    ) -> Result<i32> {
        let mut value = 0;
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { (self.content_type_reader_get_type_version)(handle, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn type_reader_supports_version(
        &self,
        handle: sys::CNA_ContentTypeReaderHandle,
        version: i32,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe {
            (self.content_type_reader_supports_version)(handle, version, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn type_reader_initialize(
        &self,
        handle: sys::CNA_ContentTypeReaderHandle,
    ) -> Result<()> {
        // SAFETY: the handle is live.
        self.check(unsafe { (self.content_type_reader_initialize)(handle) })
    }

    pub(crate) fn type_reader_read_untyped(
        &self,
        handle: sys::CNA_ContentTypeReaderHandle,
        reader: sys::CNA_ContentReaderHandle,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: both handles are live and the output is a local.
        self.check(unsafe {
            (self.content_type_reader_read_untyped)(handle, reader, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn destroy_content_type_reader(
        &self,
        handle: sys::CNA_ContentTypeReaderHandle,
    ) -> Result<()> {
        // SAFETY: the handle is owned and destroyed exactly once.
        self.check(unsafe { (self.content_type_reader_destroy)(handle) })
    }
}

fn view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast(),
        byte_length: value.len() as u64,
    }
}
