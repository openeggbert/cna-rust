//! `content_readers.h`: reading a compiled asset, and teaching CNA a new type.
//!
//! # The extension point this crate can project
//!
//! `content_readers.h` offers two ways to add a reader for a type CNA does not
//! know, and only one of them can be handed to safe Rust.
//!
//! The reflective builder declares a field as a *value kind* and a *byte
//! offset* -- "write this many bytes there" -- into an object the caller's
//! create callback returned. Nothing checks that the offset lies inside the
//! object or that the kind matches the field, so pairing a four-byte field's
//! offset with the `Matrix` kind writes sixty-four bytes over it, from safe
//! Rust, with no way for a wrapper to notice. That is why
//! `reflective-reader-writes-at-caller-offsets` rules it out.
//!
//! [`register_type_reader`] is the other way, and it inverts the dangerous
//! half: the callback is *handed* a borrowed [`ContentReaderView`] and reads
//! fields out of it, returning whatever object it likes. Nothing writes at an
//! offset, so nothing can write at the wrong one.
//!
//! # Two readers, two lifetimes
//!
//! [`ContentReader`] is **owned**. It borrows a [`StorageStream`] for its whole
//! life and closes it when destroyed; the stream value stays yours and its own
//! close is idempotent, so the two do not fight.
//!
//! [`ContentReaderView`] is the **callback-scoped borrow** handed to a read
//! callback. Upstream invalidates it before the callback returns and gives it
//! no destroy operation, so it carries a lifetime and cannot outlive the call.
//! Caching one past the callback would answer `CNA_RESULT_INVALID_HANDLE`
//! rather than fault, but the type makes it not compile instead.
//!
//! # The object a reader produces
//!
//! `read` answers a raw pointer, exactly as [`CnjLoader`] does and for the same
//! reason: this ABI never dereferences, copies or frees it, and it is handed to
//! whoever asked for the asset. Its lifetime is the caller's own business, so
//! the projection does not invent a `Drop` for it.
//!
//! [`StorageStream`]: crate::storage::StorageStream
//! [`CnjLoader`]: crate::extensions::content::CnjLoader

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::native::Native;
use crate::value::{Color, Matrix, Quaternion, Vector2, Vector3, Vector4};
use crate::Microsoft::Xna::Framework::BoundingSphere;
use crate::storage::StorageStream;

/// Every typed read `content_readers.h` publishes, over either reader.
///
/// Implemented by both the owned [`ContentReader`] and the callback-scoped
/// [`ContentReaderView`] so a reader callback written against the borrow works
/// unchanged against a reader the caller built.
pub trait ContentReads {
    /// The handle to read through.
    #[doc(hidden)]
    fn reader_handle(&self) -> Result<sys::CNA_ContentReaderHandle>;

    /// The `Native` table to read through.
    #[doc(hidden)]
    fn reader_native(&self) -> &Arc<Native>;

    /// The logical asset name, as the reader was told it.
    fn asset_name(&self) -> Result<String> {
        self.reader_native()
            .content_reader_asset_name(self.reader_handle()?)
    }

    /// The container version from the compiled asset header.
    fn version(&self) -> Result<i32> {
        self.reader_native()
            .content_reader_version(self.reader_handle()?)
    }

    /// The platform identifier byte from the compiled asset header.
    fn platform(&self) -> Result<u8> {
        self.reader_native()
            .content_reader_platform(self.reader_handle()?)
    }

    /// Reads a `Matrix`.
    fn read_matrix(&self) -> Result<Matrix> {
        let value = self
            .reader_native()
            .content_reader_read_matrix(self.reader_handle()?)?;
        Ok(Matrix::new(
            value.m11, value.m12, value.m13, value.m14, value.m21, value.m22, value.m23,
            value.m24, value.m31, value.m32, value.m33, value.m34, value.m41, value.m42,
            value.m43, value.m44,
        ))
    }

    /// Reads a `Quaternion`.
    fn read_quaternion(&self) -> Result<Quaternion> {
        let value = self
            .reader_native()
            .content_reader_read_quaternion(self.reader_handle()?)?;
        Ok(Quaternion::from_x_and_y_and_z_and_w(
            value.x, value.y, value.z, value.w,
        ))
    }

    /// Reads a `Vector2`.
    fn read_vector2(&self) -> Result<Vector2> {
        let value = self
            .reader_native()
            .content_reader_read_vector2(self.reader_handle()?)?;
        Ok(Vector2::from_x_and_y(value.x, value.y))
    }

    /// Reads a `Vector3`.
    fn read_vector3(&self) -> Result<Vector3> {
        let value = self
            .reader_native()
            .content_reader_read_vector3(self.reader_handle()?)?;
        Ok(Vector3::from_x_and_y_and_z(value.x, value.y, value.z))
    }

    /// Reads a `Vector4`.
    fn read_vector4(&self) -> Result<Vector4> {
        let value = self
            .reader_native()
            .content_reader_read_vector4(self.reader_handle()?)?;
        Ok(Vector4::from_x_and_y_and_z_and_w(
            value.x, value.y, value.z, value.w,
        ))
    }

    /// Reads a `Color`.
    fn read_color(&self) -> Result<Color> {
        let value = self
            .reader_native()
            .content_reader_read_color(self.reader_handle()?)?;
        Ok(Color::from_r_and_g_and_b_and_a_as_int32_and_int32_and_int32_and_int32(
            i32::from(value.r),
            i32::from(value.g),
            i32::from(value.b),
            i32::from(value.a),
        ))
    }

    /// Reads a `BoundingSphere`.
    fn read_bounding_sphere(&self) -> Result<BoundingSphere> {
        let value = self
            .reader_native()
            .content_reader_read_bounding_sphere(self.reader_handle()?)?;
        Ok(BoundingSphere::new(
            Vector3::from_x_and_y_and_z(value.center.x, value.center.y, value.center.z),
            value.radius,
        ))
    }

    /// Reads the next object through the dispatch protocol and discards it.
    ///
    /// Answers whether the reference was non-null. The object itself is not
    /// published: this route exists to *advance* the reader past an object a
    /// caller does not want, and CNA discards it before returning.
    fn skip_object(&self) -> Result<bool> {
        self.reader_native()
            .content_reader_read_object_tag(self.reader_handle()?)
    }

    /// Reads exactly `count` bytes, refusing a short read.
    ///
    /// `reader_name` appears in the diagnostic if the read fails, which is what
    /// makes a malformed asset traceable to the reader that choked on it.
    fn read_bytes_exact(&self, count: i32, reader_name: &str) -> Result<Vec<u8>> {
        self.reader_native()
            .content_reader_read_bytes_exact(self.reader_handle()?, count, reader_name)
    }

    /// Validates a declared element count against the reader's own limit.
    ///
    /// A compiled asset states how many elements a collection has *before* the
    /// elements themselves. Checking it first is what stops a corrupt or
    /// hostile file from asking a reader to allocate for four billion of them.
    fn check_element_count(&self, count: i64, reader_name: &str) -> Result<()> {
        self.reader_native()
            .content_reader_check_element_count(self.reader_handle()?, count, reader_name)
    }

    /// Validates a decoded buffer size against the reader's own limit.
    fn check_decoded_size(&self, byte_size: i64, reader_name: &str) -> Result<()> {
        self.reader_native()
            .content_reader_check_decoded_size(self.reader_handle()?, byte_size, reader_name)
    }
}

/// A reader over one compiled asset stream, owned by the caller.
pub struct ContentReader {
    handle: Mutex<sys::CNA_ContentReaderHandle>,
    native: Arc<Native>,
    /// Held so the stream cannot be dropped while the reader borrows it.
    _stream: StorageStream,
}

impl ContentReader {
    /// Opens a reader over a stream positioned after the container header.
    ///
    /// `version` and `platform` come from that header. The reader borrows the
    /// stream until it is dropped.
    pub fn new(
        stream: StorageStream,
        asset_name: &str,
        version: i32,
        platform: u8,
    ) -> Result<Self> {
        Self::create(None, stream, asset_name, version, platform)
    }

    /// Opens a reader that can resolve external references through a manager.
    ///
    /// A standalone reader is fine for an asset with none; only external
    /// references and the manager-backed disposal fallback need one.
    pub fn with_content_manager(
        content_manager: sys::CNA_Handle,
        stream: StorageStream,
        asset_name: &str,
        version: i32,
        platform: u8,
    ) -> Result<Self> {
        Self::create(
            Some(content_manager),
            stream,
            asset_name,
            version,
            platform,
        )
    }

    fn create(
        content_manager: Option<sys::CNA_Handle>,
        stream: StorageStream,
        asset_name: &str,
        version: i32,
        platform: u8,
    ) -> Result<Self> {
        let native = Native::process()?;
        let info = sys::CNA_ContentReaderCreateInfo {
            struct_size: core::mem::size_of::<sys::CNA_ContentReaderCreateInfo>() as u32,
            struct_version: 1,
            content_manager: content_manager.unwrap_or(sys::CNA_INVALID_HANDLE),
            stream: stream.native_handle()?,
            asset_name: sys::CNA_StringView {
                data: asset_name.as_ptr().cast(),
                byte_length: asset_name.len() as u64,
            },
            version,
            platform,
            reserved: [0; 3],
        };
        let handle = native.create_content_reader(&info)?;
        Ok(Self {
            handle: Mutex::new(handle),
            native,
            _stream: stream,
        })
    }

    /// The content manager this reader was created with, if any.
    ///
    /// Answers `None` for a standalone reader, which is a real state rather
    /// than a failure.
    pub fn content_manager_handle(&self) -> Result<Option<sys::CNA_Handle>> {
        self.native.content_reader_manager(self.get()?)
    }

    /// Reads and instantiates the compiled type-reader table, then the
    /// shared-resource count.
    ///
    /// Must run before the object graph is read. Fails with an IO result when
    /// an entry names a reader nothing is registered for -- which is what
    /// [`register_type_reader`] exists to prevent.
    pub fn initialize_type_readers(&self) -> Result<()> {
        self.native
            .content_reader_initialize_type_readers(self.get()?)
    }

    /// Reads every shared resource and runs the queued fixups.
    pub fn read_shared_resources(&self) -> Result<()> {
        self.native.content_reader_read_shared_resources(self.get()?)
    }

    /// Destroys the reader early, closing the stream it borrowed. Idempotent.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        self.native.destroy_content_reader(handle)
    }

    fn get(&self) -> Result<sys::CNA_ContentReaderHandle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput("the content reader is closed"));
        }
        Ok(handle)
    }
}

impl core::fmt::Debug for ContentReader {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ContentReader")
            .field(
                "handle",
                &*self
                    .handle
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            )
            .finish_non_exhaustive()
    }
}

impl Drop for ContentReader {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

impl ContentReads for ContentReader {
    fn reader_handle(&self) -> Result<sys::CNA_ContentReaderHandle> {
        self.get()
    }
    fn reader_native(&self) -> &Arc<Native> {
        &self.native
    }
}

/// The reader handed to a read callback, valid only for that call.
///
/// The lifetime is the whole point: upstream invalidates the handle before the
/// callback returns, so a view that escaped would be a handle to nothing.
#[derive(Debug)]
pub struct ContentReaderView<'callback> {
    handle: sys::CNA_ContentReaderHandle,
    native: Arc<Native>,
    marker: core::marker::PhantomData<&'callback ()>,
}

impl ContentReads for ContentReaderView<'_> {
    fn reader_handle(&self) -> Result<sys::CNA_ContentReaderHandle> {
        Ok(self.handle)
    }
    fn reader_native(&self) -> &Arc<Native> {
        &self.native
    }
}

/// One reader instance the registry produced, owned by the caller.
#[derive(Debug)]
pub struct ContentTypeReader {
    handle: Mutex<sys::CNA_ContentTypeReaderHandle>,
    native: Arc<Native>,
}

impl ContentTypeReader {
    /// Creates one fresh instance for a canonical reader name.
    ///
    /// Fails with `NOT_SUPPORTED` when nothing is registered under that name,
    /// which is a different answer from a reader that exists and refuses.
    pub fn for_name(canonical_name: &str) -> Result<Self> {
        let native = Native::process()?;
        let handle = native.create_content_type_reader(canonical_name)?;
        Ok(Self {
            handle: Mutex::new(handle),
            native,
        })
    }

    /// A placeholder that always refuses, for a type CNA recognises and cannot
    /// read.
    ///
    /// The point is a legible failure: an asset naming one of these fails with
    /// the reason rather than with "no such reader", which would send the
    /// reader of the diagnostic looking for a registration bug.
    pub fn known_unsupported(
        target_type_name: &str,
        reason: UnsupportedReason,
    ) -> Result<Self> {
        let native = Native::process()?;
        let handle =
            native.create_known_unsupported_reader(target_type_name, reason.to_native())?;
        Ok(Self {
            handle: Mutex::new(handle),
            native,
        })
    }

    /// Whether [`Self::read_into_existing_is_allowed`]'s callback accepts a
    /// non-null existing object.
    pub fn read_into_existing_is_allowed(&self) -> Result<bool> {
        self.native
            .type_reader_can_deserialize_into_existing(self.get()?)
    }

    /// The canonical target type name this reader produces.
    pub fn target_type_name(&self) -> Result<String> {
        self.native.type_reader_target_type_name(self.get()?)
    }

    /// The reader's own `TypeVersion`.
    pub fn type_version(&self) -> Result<i32> {
        self.native.type_reader_version(self.get()?)
    }

    /// Whether this reader accepts a file declaring `version`.
    pub fn supports_version(&self, version: i32) -> Result<bool> {
        self.native.type_reader_supports_version(self.get()?, version)
    }

    /// Runs the reader's one-time initialization.
    pub fn initialize(&self) -> Result<()> {
        self.native.type_reader_initialize(self.get()?)
    }

    /// Runs this reader against a content reader and discards the object.
    ///
    /// Answers whether the reader produced one. This measures that a reader
    /// *can* read the data in front of it, rather than producing something to
    /// keep.
    ///
    /// **Discarding leaks.** CNA never dereferences, copies or frees the
    /// pointer a reader produces -- that is the contract every reader here is
    /// written to -- so "discards" means it drops the pointer on the floor. A
    /// reader that allocates therefore leaks once per call through this route.
    /// It is the right shape for the route's purpose, which is checking that a
    /// reader accepts data; a reader whose objects must be reclaimed should be
    /// driven through a load that hands the object back instead.
    pub fn read_and_discard(&self, reader: &impl ContentReads) -> Result<bool> {
        self.native
            .type_reader_read_untyped(self.get()?, reader.reader_handle()?)
    }

    /// Destroys the reader early. Idempotent.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        self.native.destroy_content_type_reader(handle)
    }

    fn get(&self) -> Result<sys::CNA_ContentTypeReaderHandle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput("the content type reader is released"));
        }
        Ok(handle)
    }
}

impl Drop for ContentTypeReader {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// Why a placeholder reader refuses.
///
/// One reason exists at ABI 0.21. It is an enum rather than a bare integer so a
/// second one is a compile error at the match sites rather than a silently
/// unhandled value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum UnsupportedReason {
    /// The general effect reader, which would need compiled platform shader
    /// bytecode.
    CompiledPlatformShaderBytecode,
}

impl UnsupportedReason {
    const fn to_native(self) -> sys::CNA_UnsupportedContentReaderReason {
        match self {
            Self::CompiledPlatformShaderBytecode => {
                sys::CNA_UNSUPPORTED_CONTENT_READER_REASON_COMPILED_PLATFORM_SHADER_BYTECODE
            }
        }
    }
}

/// A caller's own reader for a type CNA does not know.
///
/// One instance is created per compiled asset file that names the reader, so a
/// value of the implementing type may hold per-file state without sharing it.
pub trait TypeReader: Send + Sync + 'static {
    /// The canonical target type name this reader produces.
    fn target_type_name(&self) -> &str;

    /// The reader's `TypeVersion`, matched against what each file declares.
    fn type_version(&self) -> i32 {
        0
    }

    /// Whether [`Self::read`] accepts a non-null `existing`.
    fn can_read_into_existing(&self) -> bool {
        false
    }

    /// Deserializes one object.
    ///
    /// `existing` is the object to read into, or null for a fresh one, and is
    /// non-null only when [`Self::can_read_into_existing`] answers true.
    ///
    /// The pointer this answers is the caller's own: CNA never dereferences,
    /// copies or frees it, and whoever asked for the asset receives it. Its
    /// lifetime is the caller's business, which is why this returns a raw
    /// pointer rather than something with a `Drop`.
    ///
    /// Returning an error **fails the load** and is reported to whoever asked
    /// for the asset. A half-read asset has no next frame to recover in, which
    /// is why this returns a result where a game component's callback does not.
    fn read(
        &self,
        reader: &ContentReaderView<'_>,
        existing: *mut core::ffi::c_void,
    ) -> Result<*mut core::ffi::c_void>;
}

/// One live type-reader registration.
///
/// The registry is process-wide, so this outlives any one game. Dropping it
/// withdraws the factory, in the one order that is safe: the registration is
/// cancelled before the boxed reader behind it is freed.
#[must_use = "dropping a TypeReaderRegistration immediately withdraws the factory"]
pub struct TypeReaderRegistration {
    native: Arc<Native>,
    registration: Mutex<sys::CNA_Handle>,
    reader: Mutex<*mut core::ffi::c_void>,
}

// SAFETY: the pointer is an owned box this value alone frees, and the reader
// behind it is required to be `Send + Sync`.
unsafe impl Send for TypeReaderRegistration {}
unsafe impl Sync for TypeReaderRegistration {}

type BoxedTypeReader = Box<dyn TypeReader>;

impl TypeReaderRegistration {
    /// Withdraws the factory early. Idempotent.
    pub fn unregister(&self) -> Result<()> {
        let mut guard = self
            .registration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let registration = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if registration == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        let result = self.native.unregister_content_type_reader(registration);
        let mut reader = self
            .reader
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pointer = core::mem::replace(&mut *reader, core::ptr::null_mut());
        if !pointer.is_null() {
            // SAFETY: the pointer came from `Box::into_raw` in
            // `register_type_reader`, and the registration naming it is
            // already withdrawn, so nothing can still call into it.
            drop(unsafe { Box::from_raw(pointer.cast::<BoxedTypeReader>()) });
        }
        result
    }
}

impl Drop for TypeReaderRegistration {
    fn drop(&mut self) {
        let _ = self.unregister();
    }
}

impl core::fmt::Debug for TypeReaderRegistration {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("TypeReaderRegistration")
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

/// Registers a reader for a canonical name compiled assets already spell.
///
/// The registry is process-wide, so the registration outlives any one game and
/// is withdrawn by dropping the returned value.
///
/// A name someone else already owns is **refused**, with `INVALID_STATE`. That
/// is upstream's deliberate deviation from the canonical `AddTypeCreator`,
/// which silently ignores a repeat: a caller who quietly lost the race would
/// hold a live handle whose factory is never called, and would find out from
/// assets deserializing into the wrong type.
pub fn register_type_reader(
    canonical_name: &str,
    reader: Box<dyn TypeReader>,
) -> Result<TypeReaderRegistration> {
    unsafe extern "C" fn create(
        context: *mut core::ffi::c_void,
        out_reader_context: *mut *mut core::ffi::c_void,
    ) -> sys::CNA_Result {
        if context.is_null() || out_reader_context.is_null() {
            return sys::CNA_RESULT_INVALID_ARGUMENT;
        }
        // One instance per file, and this reader is stateless from CNA's point
        // of view: the per-file state, if any, belongs to the caller's own
        // type. So the registration context is handed straight back, which the
        // header explicitly allows.
        // SAFETY: the output is a live pointer CNA supplied.
        unsafe { *out_reader_context = context };
        sys::CNA_RESULT_SUCCESS
    }

    unsafe extern "C" fn read(
        reader_context: *mut core::ffi::c_void,
        input: sys::CNA_ContentReaderHandle,
        existing_object: *mut core::ffi::c_void,
        out_object: *mut *mut core::ffi::c_void,
    ) -> sys::CNA_Result {
        if reader_context.is_null() || out_object.is_null() {
            return sys::CNA_RESULT_INVALID_ARGUMENT;
        }
        // SAFETY: the context is the box the registration owns and is freed
        // only after the registration naming it has been withdrawn.
        let reader = unsafe { &*reader_context.cast::<BoxedTypeReader>() };
        let native = match Native::process() {
            Ok(native) => native,
            Err(_) => return sys::CNA_RESULT_INTERNAL,
        };
        let view = ContentReaderView {
            handle: input,
            native,
            marker: core::marker::PhantomData,
        };
        // A panic must not cross back into C. Failing the load is the right
        // answer for one: a half-read asset cannot be recovered from.
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            reader.read(&view, existing_object)
        }));
        match outcome {
            Ok(Ok(object)) => {
                // SAFETY: the output is a live pointer CNA supplied.
                unsafe { *out_object = object };
                sys::CNA_RESULT_SUCCESS
            }
            Ok(Err(CnaError::Native { code, .. })) => code,
            Ok(Err(_)) => sys::CNA_RESULT_IO,
            Err(_) => sys::CNA_RESULT_CALLBACK,
        }
    }

    let native = Native::process()?;
    let target_type_name = reader.target_type_name().to_owned();
    let type_version = reader.type_version();
    let can_read_into_existing = reader.can_read_into_existing();
    let boxed: BoxedTypeReader = reader;
    let context = Box::into_raw(Box::new(boxed)).cast::<core::ffi::c_void>();

    let callbacks = sys::CNA_ContentTypeReaderCallbacks {
        struct_size: core::mem::size_of::<sys::CNA_ContentTypeReaderCallbacks>() as u32,
        struct_version: 1,
        target_type_name: sys::CNA_StringView {
            data: target_type_name.as_ptr().cast(),
            byte_length: target_type_name.len() as u64,
        },
        type_version,
        can_deserialize_into_existing_object: if can_read_into_existing {
            sys::CNA_TRUE
        } else {
            sys::CNA_FALSE
        },
        reserved: [0; 3],
        create: Some(create),
        read: Some(read),
        destroy: None,
        context,
    };

    match native.register_content_type_reader(canonical_name, &callbacks) {
        Ok(registration) => Ok(TypeReaderRegistration {
            native,
            registration: Mutex::new(registration),
            reader: Mutex::new(context),
        }),
        Err(error) => {
            // CNA never took the pointer, so this is the only owner left.
            // SAFETY: the box was created immediately above.
            drop(unsafe { Box::from_raw(context.cast::<BoxedTypeReader>()) });
            Err(error)
        }
    }
}

/// Whether a factory is registered under a canonical reader name.
pub fn type_reader_is_registered(canonical_name: &str) -> Result<bool> {
    Native::process()?.content_type_reader_is_registered(canonical_name)
}

/// Registers the placeholder readers for recognised but unsupported types.
///
/// Idempotent, per the header.
pub fn register_known_unsupported_readers() -> Result<()> {
    Native::process()?.register_known_unsupported_readers()
}

/// Removes **every** registered factory from the process-wide registry.
///
/// Process-wide and unconditional: it takes out the built-in readers too, so a
/// caller that runs this mid-process has a registry that can read nothing until
/// something re-registers. It is bound because a test that wants a known-empty
/// registry has no other way to get one.
pub fn clear_all_type_readers() -> Result<()> {
    Native::process()?.clear_content_type_readers()
}
