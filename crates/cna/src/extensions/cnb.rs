//! CNB's own encoding: the primitive writer, the cursor, and chunk navigation.
//!
//! [`crate::extensions::content`] already opens a `.cnb` document and decodes
//! the asset types CNA knows. What it could not do is help a game that defines
//! an asset type of its own -- and that path is otherwise complete, with
//! `AssetTypeId::custom`, `CnbWriter` and `CnbLoader` all bound. Such a game had
//! to hand-roll the payload encoding on the writing side and the decoding on
//! the reading side, against a format specified by somebody else's source.
//!
//! # Why these are not `Vec<u8>` and `to_le_bytes`
//!
//! Nothing here is generic byte I/O. Each routine carries CNB's canonical
//! encoding *and* the checks that go with it:
//!
//! * a string is a length prefix and UTF-8 bytes, validated as well-formed and
//!   measured against the read limit before anything is allocated;
//! * a keyframe is a fixed 48-byte layout shared by standalone animation clips
//!   and by a model's embedded ones, so there is exactly one keyframe encoding;
//! * a seconds value is refused unless a `TimeSpan` can hold it, because a
//!   malformed file must surface as a content failure naming the file rather
//!   than as an exception from a `TimeSpan` factory;
//! * integers are decomposed byte by byte and floats go through an integer, so
//!   a built document does not depend on the host's byte order.
//!
//! Writing that again over `Vec<u8>` would be a second encoder of the same
//! format, and the two could disagree -- silently, in a file, long after the
//! run that produced it.
//!
//! # Reading is destructive; copying is not
//!
//! [`CnbReader::read_string`] is one call in Rust and two in C, and the C shape
//! is the reason: a single route taking a destination buffer could not report a
//! capacity that was too small without either losing the string it had already
//! consumed or consuming it twice. Upstream reads once, reports the size, and
//! holds the decoded bytes until the next read. The Rust method does both
//! halves, so a caller never sees the seam.

#![allow(clippy::missing_errors_doc)]

use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::content::{
    CnbSoundEffect, CnbSpriteFont,
    accept_size_probe, string_view, CnbDocument, CnbModel, CnbTextureData, CnbWriter, ReadLimits,
};
use crate::extensions::models::{AnimationClip, ClipTargetSpace, Keyframe, StagedClip};
use crate::graphics::SurfaceFormat;
use crate::value::{Curve, CurveContinuity, CurveKey, CurveLoopType};
use crate::native::Native;

/// A chunk's entry in a document's table of contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ChunkEntry {
    /// Where the chunk's stored bytes begin in the file.
    pub offset: u64,
    /// How many bytes the chunk occupies as stored, compressed or not.
    pub stored_size: u64,
    /// How many bytes it occupies once decompressed.
    pub uncompressed_size: u64,
    /// The four-character chunk identity.
    pub chunk: ChunkId,
    pub flags: u32,
    /// The CRC32C the writer recorded over the stored bytes.
    pub checksum: u32,
    /// Which compression the stored bytes use.
    pub compression: Compression,
    /// The alignment the writer asked for.
    pub alignment: u32,
}

/// A four-character chunk identity, as CNB packs it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ChunkId(pub u32);

/// How a chunk's bytes are stored.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Compression(pub u32);

/// What a schema declares it expects at one logical external name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ExternalReference {
    pub flags: u32,
    /// The asset type the referring schema expects, or `None` when it does not
    /// constrain one.
    pub expected_asset_type: Option<u32>,
}

/// CNB's primitive writer: the canonical little-endian encoding, in order.
///
/// Build a chunk payload with this and hand the bytes to
/// [`CnbWriter::add_chunk`]. Everything it emits is byte-deterministic --
/// nothing here reads the clock, a random source or a pointer value -- so the
/// same content produces the same file.
#[derive(Debug)]
pub struct CnbByteWriter {
    native: Arc<Native>,
    handle: Mutex<sys::CNA_CnbByteWriterHandle>,
}

impl CnbByteWriter {
    /// Starts an empty writer.
    pub fn new() -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local receiving a new owned handle.
        native.check(unsafe { (native.runtime.cnb_byte_writer_create)(&mut handle) })?;
        Ok(Self {
            native,
            handle: Mutex::new(handle),
        })
    }

    /// Starts a writer already holding these bytes.
    ///
    /// For appending to a payload that was assembled elsewhere.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: `bytes` is borrowed for the call with its own length, and a
        // null pointer is paired only with a zero length.
        native.check(unsafe {
            (native.runtime.cnb_byte_writer_create_from_bytes)(
                if bytes.is_empty() {
                    core::ptr::null()
                } else {
                    bytes.as_ptr()
                },
                bytes.len() as u64,
                &mut handle,
            )
        })?;
        Ok(Self {
            native,
            handle: Mutex::new(handle),
        })
    }

    fn get(&self) -> Result<sys::CNA_CnbByteWriterHandle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput("the byte writer has been released"));
        }
        Ok(handle)
    }

    /// How many bytes have been written so far.
    pub fn len(&self) -> Result<u64> {
        let handle = self.get()?;
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.runtime.cnb_byte_writer_get_size)(handle, &mut value) })?;
        Ok(value)
    }

    /// Whether nothing has been written yet.
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    /// A copy of what has been written, leaving the writer as it was.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        self.read_bytes(|destination, capacity, written| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable bytes; a null destination with zero capacity is the
            // documented size probe.
            unsafe { (api.cnb_byte_writer_copy_bytes)(handle, destination, capacity, written) }
        })
    }

    /// The bytes, leaving the writer empty.
    ///
    /// The difference from [`Self::to_bytes`] is upstream's: taking hands over
    /// the buffer rather than duplicating it, so a large payload is not held
    /// twice at once.
    pub fn take(&self) -> Result<Vec<u8>> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        // The size has to be read before the take, because taking empties the
        // writer and a probe afterwards would report nothing.
        let mut required = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (api.cnb_byte_writer_get_size)(handle, &mut required) })?;
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("the written payload is too large"))?;
        let mut buffer = vec![0_u8; capacity];
        let mut written = 0_u64;
        // SAFETY: the handle is owned and the destination holds exactly the
        // byte count just reported.
        self.native.check(unsafe {
            (api.cnb_byte_writer_take)(
                handle,
                if buffer.is_empty() {
                    core::ptr::null_mut()
                } else {
                    buffer.as_mut_ptr()
                },
                required,
                &mut written,
            )
        })?;
        buffer.truncate((written as usize).min(capacity));
        Ok(buffer)
    }

    fn read_bytes(
        &self,
        mut route: impl FnMut(*mut u8, u64, *mut u64) -> sys::CNA_Result,
    ) -> Result<Vec<u8>> {
        let mut required = 0_u64;
        accept_size_probe(
            &self.native,
            route(core::ptr::null_mut(), 0, &mut required),
        )?;
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("the written payload is too large"))?;
        let mut buffer = vec![0_u8; capacity];
        let mut written = 0_u64;
        self.native.check(route(
            if buffer.is_empty() {
                core::ptr::null_mut()
            } else {
                buffer.as_mut_ptr()
            },
            required,
            &mut written,
        ))?;
        buffer.truncate((written as usize).min(capacity));
        Ok(buffer)
    }

    /// Releases the writer early.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle is owned by this value and released exactly once.
        self.native
            .check(unsafe { (self.native.runtime.cnb_byte_writer_destroy)(handle) })
    }
}

/// Declares one primitive write over its native route.
macro_rules! write_scalar {
    ($(#[$meta:meta])* $method:ident, $route:ident, $value:ty) => {
        $(#[$meta])*
        pub fn $method(&self, value: $value) -> Result<()> {
            let handle = self.get()?;
            // SAFETY: the handle is owned and the value is passed by value.
            self.native
                .check(unsafe { (self.native.runtime.$route)(handle, value) })
        }
    };
}

impl CnbByteWriter {
    write_scalar!(/// Appends one byte.
        write_u8, cnb_byte_writer_write_u8, u8);
    write_scalar!(/// Appends a little-endian `u16`.
        write_u16, cnb_byte_writer_write_u16, u16);
    write_scalar!(/// Appends a little-endian `u32`.
        write_u32, cnb_byte_writer_write_u32, u32);
    write_scalar!(/// Appends a little-endian `u64`.
        write_u64, cnb_byte_writer_write_u64, u64);
    write_scalar!(/// Appends a little-endian `i32`.
        write_i32, cnb_byte_writer_write_i32, i32);
    write_scalar!(/// Appends an IEEE-754 `f32`, through an integer.
        write_f32, cnb_byte_writer_write_f32, f32);
    write_scalar!(/// Appends an IEEE-754 `f64`, through an integer.
        write_f64, cnb_byte_writer_write_f64, f64);
    write_scalar!(/// Appends `count` zero bytes, for padding to an alignment.
        write_zeros, cnb_byte_writer_write_zeros, u64);

    /// Appends raw bytes with no length prefix.
    pub fn write_bytes(&self, value: &[u8]) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: `value` is borrowed for the call with its own length, and a
        // null pointer is paired only with a zero length.
        self.native.check(unsafe {
            (self.native.runtime.cnb_byte_writer_write_bytes)(
                handle,
                if value.is_empty() {
                    core::ptr::null()
                } else {
                    value.as_ptr()
                },
                value.len() as u64,
            )
        })
    }

    /// Appends a byte length followed by the string's UTF-8 bytes.
    ///
    /// A Rust `&str` is already well-formed UTF-8, so the only refusal left is
    /// a string longer than `u32::MAX` bytes.
    pub fn write_string(&self, value: &str) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and the string is borrowed for the call.
        self.native.check(unsafe {
            (self.native.runtime.cnb_byte_writer_write_string)(handle, string_view(value))
        })
    }

    /// Appends one keyframe in CNB's fixed 48-byte layout.
    pub fn write_keyframe(&self, keyframe: Keyframe) -> Result<()> {
        let handle = self.get()?;
        let native = keyframe.to_native();
        // SAFETY: the handle is owned and the keyframe is a live local CNA
        // copies during the call.
        self.native.check(unsafe {
            (self.native.runtime.cnb_byte_writer_write_keyframe)(handle, &native)
        })
    }
}

impl Drop for CnbByteWriter {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// A cursor over one region of `.cnb` bytes.
///
/// Every read is bounded by the region *and* by the read limits, so a
/// malformed file fails the read rather than reaching for memory it should not.
/// [`CnbReader::fail`] is how a caller's own schema check produces a failure
/// that names the same context as CNA's, which is what makes a game's custom
/// asset type report problems the way a built-in one does.
#[derive(Debug)]
pub struct CnbReader {
    native: Arc<Native>,
    handle: Mutex<sys::CNA_CnbReaderHandle>,
}

impl CnbReader {
    /// Opens a cursor over bytes the caller already has.
    ///
    /// `context` names the source in failures -- an asset name is the usual
    /// choice -- and every message this cursor produces carries it.
    pub fn new(bytes: &[u8], context: &str, limits: ReadLimits) -> Result<Self> {
        let native = Native::process()?;
        let native_limits = limits.to_native(&native)?;
        let limits_pointer = if limits.is_default() {
            core::ptr::null()
        } else {
            &native_limits
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the bytes and the context are borrowed for the call, the
        // limits outlive it, and the output is a live local. CNA copies what it
        // needs; the cursor does not retain the caller's slice past the call
        // that made it, which is why the bytes are copied in below.
        native.check(unsafe {
            (native.runtime.cnb_reader_create)(
                if bytes.is_empty() {
                    core::ptr::null()
                } else {
                    bytes.as_ptr()
                },
                bytes.len() as u64,
                string_view(context),
                limits_pointer,
                &mut handle,
            )
        })?;
        Ok(Self {
            native,
            handle: Mutex::new(handle),
        })
    }

    fn get(&self) -> Result<sys::CNA_CnbReaderHandle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput("the reader has been released"));
        }
        Ok(handle)
    }

    /// How many bytes the region holds in total.
    pub fn len(&self) -> Result<u64> {
        self.count(|api, handle, out| {
            // SAFETY: owned handle, live output.
            unsafe { (api.cnb_reader_get_size)(handle, out) }
        })
    }

    /// Whether the region holds no bytes at all.
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    /// How far the cursor has advanced.
    pub fn position(&self) -> Result<u64> {
        self.count(|api, handle, out| {
            // SAFETY: owned handle, live output.
            unsafe { (api.cnb_reader_get_position)(handle, out) }
        })
    }

    /// How many bytes are left before the end of the region.
    pub fn remaining(&self) -> Result<u64> {
        self.count(|api, handle, out| {
            // SAFETY: owned handle, live output.
            unsafe { (api.cnb_reader_get_remaining)(handle, out) }
        })
    }

    fn count(
        &self,
        route: impl FnOnce(&crate::native::runtime::RuntimeApi, sys::CNA_CnbReaderHandle, *mut u64)
            -> sys::CNA_Result,
    ) -> Result<u64> {
        let handle = self.get()?;
        let mut value = 0_u64;
        self.native
            .check(route(&self.native.runtime, handle, &mut value))?;
        Ok(value)
    }

    /// The context this cursor names in its failures.
    pub fn context(&self) -> Result<String> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        crate::native::runtime::read_string(
            |value| self.native.check(value),
            // SAFETY: owned handle, live outputs; CNA's size-then-copy pair.
            |bytes| unsafe { (api.cnb_reader_get_context_size)(handle, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_reader_copy_context)(handle, destination, capacity, written)
            },
        )
    }

    /// Advances past `count` bytes without reading them.
    pub fn skip(&self, count: u64) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned; the count is checked against the region.
        self.native
            .check(unsafe { (self.native.runtime.cnb_reader_skip)(handle, count) })
    }

    /// Fails unless the cursor has consumed the whole region.
    ///
    /// A schema that has read everything it expects and finds bytes left over
    /// has misread something, and saying so is better than ignoring the tail.
    pub fn require_exhausted(&self) -> Result<()> {
        let handle = self.get()?;
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.runtime.cnb_reader_require_exhausted)(handle) })
    }

    /// Produces a content failure naming this cursor's context.
    ///
    /// Always an `Err`. For a caller's own schema check -- a field that is out
    /// of range, a combination the format forbids -- so the failure reads like
    /// one of CNA's rather than like a stray error from a game.
    pub fn fail<T>(&self, message: &str) -> Result<T> {
        let handle = self.get()?;
        // SAFETY: the handle is owned and the message is borrowed for the call.
        let result = self
            .native
            .check(unsafe { (self.native.runtime.cnb_reader_fail)(handle, string_view(message)) });
        match result {
            Err(error) => Err(error),
            // Upstream is documented to fail; a success would mean the route
            // stopped doing the one thing it exists for.
            Ok(()) => Err(CnaError::InvalidInput(
                "cna_cnb_reader_fail returned success, which it is documented never to do",
            )),
        }
    }

    /// Reads a length-prefixed, UTF-8-validated string.
    ///
    /// Two routes upstream and one method here: see the module documentation
    /// for why reading and copying are separate in C.
    pub fn read_string(&self) -> Result<String> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        let mut required = 0_u64;
        // SAFETY: the handle is owned and the output is a live local. This
        // consumes the string; the copy below is what retrieves it.
        self.native
            .check(unsafe { (api.cnb_reader_read_string)(handle, &mut required) })?;
        crate::native::runtime::read_string_of_size(
            required,
            |value| self.native.check(value),
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable bytes.
            |destination, capacity, written| unsafe {
                (api.cnb_reader_copy_string)(handle, destination, capacity, written)
            },
        )
    }

    /// Reads an element count and checks it against the array limit.
    ///
    /// `what` names the thing being counted in a failure message -- "the bone
    /// count", say -- which is why this is not just `read_u32`.
    pub fn read_count(&self, element_size: u64, what: &str) -> Result<u32> {
        let handle = self.get()?;
        let mut value = 0_u32;
        // SAFETY: the handle is owned, `what` is borrowed for the call, and the
        // output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_reader_read_count)(
                handle,
                element_size,
                string_view(what),
                &mut value,
            )
        })?;
        Ok(value)
    }

    /// Reads a seconds value a `TimeSpan` can hold.
    ///
    /// `what` names it in a failure -- "the clip duration", say. The range
    /// check happens before the value reaches a `TimeSpan` at all, so a
    /// malformed file is a content failure rather than an exception from a
    /// factory.
    pub fn read_seconds(&self, what: &str) -> Result<f64> {
        let handle = self.get()?;
        let mut value = 0_f64;
        // SAFETY: the handle is owned, `what` is borrowed for the call, and the
        // output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_reader_read_seconds)(handle, string_view(what), &mut value)
        })?;
        Ok(value)
    }

    /// Reads one keyframe in CNB's fixed 48-byte layout.
    pub fn read_keyframe(&self) -> Result<Keyframe> {
        let handle = self.get()?;
        let mut value = sys::CNA_KeyframeEXT::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.runtime.cnb_reader_read_keyframe)(handle, &mut value) })?;
        Ok(Keyframe::from_native(value))
    }

    /// Reads exactly `count` raw bytes.
    pub fn read_bytes(&self, count: u64) -> Result<Vec<u8>> {
        let handle = self.get()?;
        let capacity = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("more bytes than fit in memory"))?;
        let mut buffer = vec![0_u8; capacity];
        let mut written = 0_u64;
        // SAFETY: the handle is owned and the destination holds `count`
        // writable bytes.
        self.native.check(unsafe {
            (self.native.runtime.cnb_reader_read_bytes)(
                handle,
                count,
                if buffer.is_empty() {
                    core::ptr::null_mut()
                } else {
                    buffer.as_mut_ptr()
                },
                count,
                &mut written,
            )
        })?;
        buffer.truncate((written as usize).min(capacity));
        Ok(buffer)
    }

    /// Releases the cursor early.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle is owned by this value and released exactly once.
        self.native
            .check(unsafe { (self.native.runtime.cnb_reader_destroy)(handle) })
    }
}

/// Declares one primitive read over its native route.
macro_rules! read_scalar {
    ($(#[$meta:meta])* $method:ident, $route:ident, $value:ty) => {
        $(#[$meta])*
        pub fn $method(&self) -> Result<$value> {
            let handle = self.get()?;
            let mut value = <$value>::default();
            // SAFETY: the handle is owned and the output is a live local.
            self.native
                .check(unsafe { (self.native.runtime.$route)(handle, &mut value) })?;
            Ok(value)
        }
    };
}

impl CnbReader {
    read_scalar!(/// Reads one byte.
        read_u8, cnb_reader_read_u8, u8);
    read_scalar!(/// Reads a little-endian `u16`.
        read_u16, cnb_reader_read_u16, u16);
    read_scalar!(/// Reads a little-endian `u32`.
        read_u32, cnb_reader_read_u32, u32);
    read_scalar!(/// Reads a little-endian `u64`.
        read_u64, cnb_reader_read_u64, u64);
    read_scalar!(/// Reads a little-endian `i32`.
        read_i32, cnb_reader_read_i32, i32);
    read_scalar!(/// Reads an IEEE-754 `f32`.
        read_f32, cnb_reader_read_f32, f32);
    read_scalar!(/// Reads an IEEE-754 `f64`.
        read_f64, cnb_reader_read_f64, f64);
}

impl Drop for CnbReader {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// Navigating a parsed document's chunks and its external references.
impl CnbDocument {
    /// One chunk's table-of-contents entry.
    pub fn chunk_at(&self, index: u64) -> Result<ChunkEntry> {
        let mut entry = sys::CNA_CnbChunkEntry {
            struct_size: core::mem::size_of::<sys::CNA_CnbChunkEntry>() as u32,
            struct_version: 1,
            ..sys::CNA_CnbChunkEntry::default()
        };
        // SAFETY: the handle is owned and the output is a live local whose size
        // and version headers are set.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_get_chunk)(self.handle, index, &mut entry)
        })?;
        Ok(ChunkEntry {
            offset: entry.offset,
            stored_size: entry.stored_size,
            uncompressed_size: entry.uncompressed_size,
            chunk: ChunkId(entry.r#type),
            flags: entry.flags,
            checksum: entry.checksum,
            compression: Compression(entry.compression),
            alignment: entry.alignment,
        })
    }

    /// A chunk's bytes, decompressed if it was stored compressed.
    pub fn chunk_data(&self, index: u64) -> Result<Vec<u8>> {
        let api = &self.native.runtime;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the size.
        accept_size_probe(&self.native, unsafe {
            (api.cnb_document_copy_chunk_data)(
                self.handle,
                index,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        })?;
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("the chunk is too large to hold in memory"))?;
        let mut buffer = vec![0_u8; capacity];
        let mut written = 0_u64;
        // SAFETY: the handle is owned and the destination holds `capacity`
        // writable bytes.
        self.native.check(unsafe {
            (api.cnb_document_copy_chunk_data)(
                self.handle,
                index,
                if buffer.is_empty() {
                    core::ptr::null_mut()
                } else {
                    buffer.as_mut_ptr()
                },
                required,
                &mut written,
            )
        })?;
        buffer.truncate((written as usize).min(capacity));
        Ok(buffer)
    }

    /// A cursor over one chunk's decompressed bytes.
    ///
    /// The reading counterpart of building a payload with [`CnbByteWriter`],
    /// and what a [`crate::extensions::content::CnbLoader`] uses to decode its
    /// own asset type.
    pub fn open_chunk(&self, index: u64) -> Result<CnbReader> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local receiving
        // a new owned cursor.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_open_chunk)(self.handle, index, &mut handle)
        })?;
        Ok(CnbReader {
            native: Arc::clone(&self.native),
            handle: Mutex::new(handle),
        })
    }

    /// Every chunk of that identity, in file order.
    pub fn find_all(&self, chunk: ChunkId) -> Result<Vec<u64>> {
        let api = &self.native.runtime;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        accept_size_probe(&self.native, unsafe {
            (api.cnb_document_find_all)(
                self.handle,
                chunk.0,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        })?;
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("more chunks than fit in memory"))?;
        let mut buffer = vec![0_u64; capacity];
        let mut written = 0_u64;
        // SAFETY: the handle is owned and the destination holds `capacity`
        // writable indices.
        self.native.check(unsafe {
            (api.cnb_document_find_all)(
                self.handle,
                chunk.0,
                if buffer.is_empty() {
                    core::ptr::null_mut()
                } else {
                    buffer.as_mut_ptr()
                },
                required,
                &mut written,
            )
        })?;
        buffer.truncate((written as usize).min(capacity));
        Ok(buffer)
    }

    /// The one chunk of that identity, or `None` when there is none.
    ///
    /// Answers `None` for absent and fails for duplicated, which is the
    /// distinction [`Self::require_single`] collapses.
    pub fn find_single(&self, chunk: ChunkId) -> Result<Option<u64>> {
        let mut found = sys::CNA_FALSE;
        let mut index = 0_u64;
        // SAFETY: the handle is owned and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_find_single)(
                self.handle,
                chunk.0,
                &mut found,
                &mut index,
            )
        })?;
        Ok((found != sys::CNA_FALSE).then_some(index))
    }

    /// The one chunk of that identity, failing when it is absent.
    pub fn require_single(&self, chunk: ChunkId) -> Result<u64> {
        let mut index = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_require_single)(self.handle, chunk.0, &mut index)
        })?;
        Ok(index)
    }

    /// Fails unless the document is an asset this decoder can read.
    ///
    /// `max_schema_version` is the **highest** version the caller understands,
    /// not the version it expects: version 1 is always the lowest accepted, so
    /// a decoder that understands up to 3 reads a version-1 file happily. That
    /// is CNB's forward-compatibility rule, and getting it backwards would make
    /// a decoder refuse every file older than itself.
    ///
    /// The check a loader makes first, so a file of the wrong type fails on its
    /// identity rather than halfway through a decode.
    pub fn require_asset(&self, asset_type_id: u32, max_schema_version: u32) -> Result<()> {
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_require_asset)(
                self.handle,
                asset_type_id,
                max_schema_version,
            )
        })
    }

    /// Fails when the document carries a mandatory chunk this list omits.
    ///
    /// CNB's forward-compatibility rule: a reader may ignore a chunk it does
    /// not know only when the writer marked it optional. Passing the chunk
    /// identities a schema understands is how a loader honours that instead of
    /// silently dropping data a newer writer marked as required.
    pub fn require_mandatory_chunks_understood(&self, understood: &[ChunkId]) -> Result<()> {
        let ids: Vec<sys::CNA_CnbChunkId> = understood.iter().map(|id| id.0).collect();
        // SAFETY: the handle is owned and the array is borrowed for the call
        // with the count it was sized against.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_require_mandatory_chunks_understood)(
                self.handle,
                if ids.is_empty() {
                    core::ptr::null()
                } else {
                    ids.as_ptr()
                },
                ids.len() as u64,
            )
        })
    }

    /// How many external references the document declares.
    pub fn external_reference_count(&self) -> Result<u64> {
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_get_external_reference_count)(self.handle, &mut value)
        })?;
        Ok(value)
    }

    /// One external reference's logical name.
    pub fn external_reference_name(&self, index: u64) -> Result<String> {
        let api = &self.native.runtime;
        crate::native::runtime::read_string(
            |value| self.native.check(value),
            // SAFETY: owned handle, live outputs; CNA's size-then-copy pair.
            |bytes| unsafe {
                (api.cnb_document_get_external_reference_name_size)(self.handle, index, bytes)
            },
            |destination, capacity, written| unsafe {
                (api.cnb_document_copy_external_reference_name)(
                    self.handle,
                    index,
                    destination,
                    capacity,
                    written,
                )
            },
        )
    }

    /// What the schema expects at one logical name.
    pub fn external_reference(&self, index: u64, name: &str) -> Result<ExternalReference> {
        let mut value = sys::CNA_CnbExternalReference {
            struct_size: core::mem::size_of::<sys::CNA_CnbExternalReference>() as u32,
            struct_version: 1,
            ..sys::CNA_CnbExternalReference::default()
        };
        // SAFETY: the handle is owned, the name is borrowed for the call, and
        // the output is a live local whose headers are set.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_get_external_reference)(
                self.handle,
                index,
                string_view(name),
                &mut value,
            )
        })?;
        Ok(ExternalReference {
            flags: value.flags,
            // Upstream spells "unconstrained" as the invalid asset type.
            expected_asset_type: (value.expected_asset_type_id
                != sys::CNA_CNB_ASSET_TYPE_INVALID)
                .then_some(value.expected_asset_type_id),
        })
    }

    /// The read limits this document was parsed under.
    pub fn limits(&self) -> Result<ReadLimits> {
        let mut value = sys::CNA_CnbReadLimits::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_get_limits)(self.handle, &mut value)
        })?;
        Ok(limits_from_native(value))
    }

    /// A texture embedded in the document under that logical name.
    pub fn embedded_texture2d(&self, name: &str) -> Result<CnbTextureData> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned, the name is borrowed for the call, and
        // the output is a live local receiving a new owned handle.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_read_embedded_texture2d)(
                self.handle,
                string_view(name),
                &mut handle,
            )
        })?;
        Ok(CnbTextureData {
            native: Arc::clone(&self.native),
            handle,
        })
    }
}

fn limits_from_native(value: sys::CNA_CnbReadLimits) -> ReadLimits {
    ReadLimits {
        max_file_size: Some(value.max_file_size),
        max_chunk_size: Some(value.max_chunk_size),
        max_total_uncompressed_size: Some(value.max_total_uncompressed_size),
        max_chunk_count: Some(value.max_chunk_count),
        max_string_bytes: Some(value.max_string_bytes),
        max_array_element_count: Some(value.max_array_element_count),
        max_chunk_alignment: Some(value.max_chunk_alignment),
    }
}

impl ChunkEntry {
    /// Whether a reader that does not understand this chunk must refuse the
    /// file rather than skip it.
    pub fn is_mandatory(&self) -> Result<bool> {
        let native = Native::process()?;
        let entry = sys::CNA_CnbChunkEntry {
            struct_size: core::mem::size_of::<sys::CNA_CnbChunkEntry>() as u32,
            struct_version: 1,
            offset: self.offset,
            stored_size: self.stored_size,
            uncompressed_size: self.uncompressed_size,
            r#type: self.chunk.0,
            flags: self.flags,
            checksum: self.checksum,
            compression: self.compression.0,
            alignment: self.alignment,
            reserved: 0,
        };
        let mut value = sys::CNA_FALSE;
        // SAFETY: the entry is a live local and the output is a live local.
        native.check(unsafe {
            (native.runtime.cnb_chunk_entry_is_mandatory)(&entry, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }
}

/// The writer's side of external references, limits and compression.
impl CnbWriter {
    /// Declares that this asset refers to another by logical name.
    ///
    /// The name is what a loader resolves at load time; the expected asset type
    /// is the schema's constraint on what may answer to it, or `None` to leave
    /// it unconstrained.
    pub fn add_external_reference(
        &self,
        logical_name: &str,
        expected_asset_type: Option<u32>,
        flags: u32,
    ) -> Result<()> {
        let value = sys::CNA_CnbExternalReference {
            struct_size: core::mem::size_of::<sys::CNA_CnbExternalReference>() as u32,
            struct_version: 1,
            flags,
            expected_asset_type_id: expected_asset_type
                .unwrap_or(sys::CNA_CNB_ASSET_TYPE_INVALID),
        };
        // SAFETY: the handle is owned, and both the descriptor and the name
        // outlive the call.
        self.native.check(unsafe {
            (self.native.runtime.cnb_writer_add_external_reference)(
                self.handle,
                &value,
                string_view(logical_name),
            )
        })
    }

    /// Drops every external reference declared so far.
    pub fn clear_external_references(&self) -> Result<()> {
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self.native.runtime.cnb_writer_clear_external_references)(self.handle)
        })
    }

    /// Embeds a texture in the document under a logical name.
    pub fn append_embedded_texture2d(
        &self,
        texture: &CnbTextureData,
        logical_name: &str,
    ) -> Result<()> {
        // SAFETY: both handles are owned by live values and the name is
        // borrowed for the call.
        self.native.check(unsafe {
            (self.native.runtime.cnb_writer_append_embedded_texture2d)(
                self.handle,
                texture.handle,
                string_view(logical_name),
            )
        })
    }

    /// How many chunks the schema has contributed so far.
    pub fn schema_chunk_count(&self) -> Result<u64> {
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_writer_get_schema_chunk_count)(self.handle, &mut value)
        })?;
        Ok(value)
    }

    /// Chooses how chunk payloads are stored.
    ///
    /// `level` is the codec's own effort setting; what it means is the codec's
    /// business, and [`compression_is_supported`] says whether this build has
    /// the codec at all.
    pub fn set_compression(&self, compression: Compression, level: i32) -> Result<()> {
        // SAFETY: the handle is owned and both values are by value.
        self.native.check(unsafe {
            (self.native.runtime.cnb_writer_set_compression)(self.handle, compression.0, level)
        })
    }

    /// The limits the writer enforces on what it produces.
    pub fn limits(&self) -> Result<ReadLimits> {
        let mut value = sys::CNA_CnbReadLimits::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (self.native.runtime.cnb_writer_get_limits)(self.handle, &mut value) })?;
        Ok(limits_from_native(value))
    }

    /// Replaces the limits the writer enforces.
    ///
    /// A writer bounded the same way the reader will be, so a file that would
    /// be refused on load fails while it is being built instead.
    pub fn set_limits(&self, limits: ReadLimits) -> Result<()> {
        let native_limits = limits.to_native(&self.native)?;
        // SAFETY: the handle is owned and the limits outlive the call.
        self.native.check(unsafe {
            (self.native.runtime.cnb_writer_set_limits)(self.handle, &native_limits)
        })
    }

    /// Builds the document and writes it straight to a file.
    ///
    /// The difference from `build` is that nothing holds the whole document in
    /// memory on this side.
    pub fn write_to_file(&self, path: &str) -> Result<()> {
        // SAFETY: the handle is owned and the path is borrowed for the call.
        self.native.check(unsafe {
            (self.native.runtime.cnb_writer_write_to_file)(self.handle, string_view(path))
        })
    }
}

/// CRC32C over a byte range, as CNB records it for every chunk.
///
/// Hardware-accelerated where the CPU has the instruction;
/// [`crc32c_uses_hardware`] says whether it did, and [`crc32c_portable`] is the
/// software path, so a caller can check the two agree on this machine.
pub fn crc32c(bytes: &[u8]) -> Result<u32> {
    let native = Native::process()?;
    let mut value = 0_u32;
    // SAFETY: `bytes` is borrowed for the call with its own length, and a null
    // pointer is paired only with a zero length.
    native.check(unsafe {
        (native.runtime.cnb_crc32c)(
            if bytes.is_empty() {
                core::ptr::null()
            } else {
                bytes.as_ptr()
            },
            bytes.len() as u64,
            &mut value,
        )
    })?;
    Ok(value)
}

/// Continues a CRC32C over the next part of a stream.
pub fn crc32c_continue(seed: u32, bytes: &[u8]) -> Result<u32> {
    let native = Native::process()?;
    let mut value = 0_u32;
    // SAFETY: as in `crc32c`.
    native.check(unsafe {
        (native.runtime.cnb_crc32c_continue)(
            seed,
            if bytes.is_empty() {
                core::ptr::null()
            } else {
                bytes.as_ptr()
            },
            bytes.len() as u64,
            &mut value,
        )
    })?;
    Ok(value)
}

/// The software CRC32C, whatever the CPU supports.
pub fn crc32c_portable(bytes: &[u8]) -> Result<u32> {
    let native = Native::process()?;
    let mut value = 0_u32;
    // SAFETY: as in `crc32c`.
    native.check(unsafe {
        (native.runtime.cnb_crc32c_portable)(
            if bytes.is_empty() {
                core::ptr::null()
            } else {
                bytes.as_ptr()
            },
            bytes.len() as u64,
            &mut value,
        )
    })?;
    Ok(value)
}

/// Whether this machine's CRC32C uses a hardware instruction.
pub fn crc32c_uses_hardware() -> Result<bool> {
    let native = Native::process()?;
    let mut value = sys::CNA_FALSE;
    // SAFETY: the output is a live local.
    native.check(unsafe { (native.runtime.cnb_crc32c_uses_hardware)(&mut value) })?;
    Ok(value != sys::CNA_FALSE)
}

/// Packs four characters into a chunk identity.
pub fn make_chunk_id(a: u8, b: u8, c: u8, d: u8) -> Result<ChunkId> {
    let native = Native::process()?;
    let mut value = 0_u32;
    // SAFETY: the output is a live local.
    native.check(unsafe { (native.runtime.cnb_make_chunk_id)(a, b, c, d, &mut value) })?;
    Ok(ChunkId(value))
}

impl ChunkId {
    /// The four characters, as text.
    pub fn to_text(self) -> Result<String> {
        let native = Native::process()?;
        let api = &native.runtime;
        crate::native::runtime::read_string(
            |value| native.check(value),
            // SAFETY: live outputs; CNA's size-then-copy pair.
            |bytes| unsafe { (api.cnb_get_chunk_id_string_size)(self.0, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_copy_chunk_id_string)(self.0, destination, capacity, written)
            },
        )
    }

    /// Whether the four bytes are ones CNB allows in an identity.
    pub fn is_well_formed(self) -> Result<bool> {
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the output is a live local.
        native.check(unsafe {
            (native.runtime.cnb_is_well_formed_chunk_id)(self.0, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }
}

impl Compression {
    /// The codec's name.
    pub fn name(self) -> Result<String> {
        let native = Native::process()?;
        let api = &native.runtime;
        crate::native::runtime::read_string(
            |value| native.check(value),
            // SAFETY: live outputs; CNA's size-then-copy pair.
            |bytes| unsafe { (api.cnb_get_compression_name_size)(self.0, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_copy_compression_name)(self.0, destination, capacity, written)
            },
        )
    }
}

/// Whether this build can decompress chunks stored with that codec.
///
/// A codec CNA knows but was not built with is a real case, and the honest
/// answer is `false` rather than a failure at load.
pub fn compression_is_supported(compression: Compression) -> Result<bool> {
    let native = Native::process()?;
    let mut value = sys::CNA_FALSE;
    // SAFETY: the output is a live local.
    native.check(unsafe {
        (native.runtime.cnb_is_compression_supported)(compression.0, &mut value)
    })?;
    Ok(value != sys::CNA_FALSE)
}

/// The bytes CNB puts at the start of every document.
pub fn format_magic() -> Result<Vec<u8>> {
    let native = Native::process()?;
    let api = &native.runtime;
    let mut required = 0_u64;
    // SAFETY: a null destination with zero capacity asks for the size.
    accept_size_probe(&native, unsafe {
        (api.cnb_copy_format_magic)(core::ptr::null_mut(), 0, &mut required)
    })?;
    let capacity = usize::try_from(required)
        .map_err(|_| CnaError::InvalidInput("the format magic is implausibly large"))?;
    let mut buffer = vec![0_u8; capacity];
    let mut written = 0_u64;
    // SAFETY: the destination holds `capacity` writable bytes.
    native.check(unsafe {
        (api.cnb_copy_format_magic)(buffer.as_mut_ptr(), required, &mut written)
    })?;
    buffer.truncate((written as usize).min(capacity));
    Ok(buffer)
}

/// Whether these bytes begin with CNB's magic.
///
/// The cheap check before a parse, and the one that tells a `.cnb` from an
/// `.xnb` without reading either.
pub fn has_magic(bytes: &[u8]) -> Result<bool> {
    let native = Native::process()?;
    let mut value = sys::CNA_FALSE;
    // SAFETY: `bytes` is borrowed for the call with its own length.
    native.check(unsafe {
        (native.runtime.cnb_has_magic)(
            if bytes.is_empty() {
                core::ptr::null()
            } else {
                bytes.as_ptr()
            },
            bytes.len() as u64,
            &mut value,
        )
    })?;
    Ok(value != sys::CNA_FALSE)
}

/// Why a logical name is not usable, or `None` when it is.
///
/// A logical name ends up as a lookup key and sometimes as a path, so CNB
/// constrains it. This reports the reason rather than a bare `false`, because
/// "the name is bad" is not something a caller can act on.
pub fn logical_name_problem(name: &str) -> Result<Option<String>> {
    let native = Native::process()?;
    let api = &native.runtime;
    let text = crate::native::runtime::read_string(
        |value| native.check(value),
        // SAFETY: live outputs; CNA's size-then-copy pair.
        |bytes| unsafe { (api.cnb_get_logical_name_problem_size)(string_view(name), bytes) },
        |destination, capacity, written| unsafe {
            (api.cnb_copy_logical_name_problem)(
                string_view(name),
                destination,
                capacity,
                written,
            )
        },
    )?;
    Ok((!text.is_empty()).then_some(text))
}

/// Whether these bytes are well-formed UTF-8 by CNB's own check.
///
/// A Rust `&str` always is, so this exists for bytes that came from somewhere
/// else -- a file, a network, a C caller -- and are about to become one.
pub fn is_well_formed_utf8(bytes: &[u8]) -> Result<bool> {
    let native = Native::process()?;
    let view = sys::CNA_StringView {
        data: bytes.as_ptr().cast::<core::ffi::c_char>(),
        byte_length: bytes.len() as u64,
    };
    let mut value = sys::CNA_FALSE;
    // SAFETY: the view borrows `bytes` for the call and the output is a live
    // local. The route only reads; it never assumes the bytes are UTF-8, which
    // is the question being asked.
    native.check(unsafe { (native.runtime.cnb_is_well_formed_utf8)(view, &mut value) })?;
    Ok(value != sys::CNA_FALSE)
}

/// `a + b`, or `None` when it would wrap.
///
/// CNB's own overflow guard, exposed because a schema computing an offset or a
/// size should reach the same verdict CNA does.
pub fn checked_add(a: u64, b: u64) -> Result<Option<u64>> {
    let native = Native::process()?;
    let mut value = 0_u64;
    // SAFETY: the output is a live local.
    let result = unsafe { (native.runtime.cnb_checked_add)(a, b, &mut value) };
    if result == sys::CNA_RESULT_SUCCESS {
        return Ok(Some(value));
    }
    // An overflow is the answer, not a failure; anything else still is one.
    match native.check(result) {
        Err(CnaError::Native { .. }) => Ok(None),
        Err(error) => Err(error),
        Ok(()) => Ok(Some(value)),
    }
}

/// `a * b`, or `None` when it would wrap.
pub fn checked_multiply(a: u64, b: u64) -> Result<Option<u64>> {
    let native = Native::process()?;
    let mut value = 0_u64;
    // SAFETY: the output is a live local.
    let result = unsafe { (native.runtime.cnb_checked_multiply)(a, b, &mut value) };
    if result == sys::CNA_RESULT_SUCCESS {
        return Ok(Some(value));
    }
    match native.check(result) {
        Err(CnaError::Native { .. }) => Ok(None),
        Err(error) => Err(error),
        Ok(()) => Ok(Some(value)),
    }
}

/// What a texture format costs, and what shape it has.
impl crate::extensions::content::CnbTextureFormat {
    /// How many bytes one mip level of that size occupies.
    ///
    /// The arithmetic differs for a block-compressed format -- it rounds up to
    /// whole blocks rather than multiplying by a per-texel size -- which is why
    /// this is a route rather than a multiplication at the call site.
    pub fn level_byte_size(self, width: u32, height: u32, depth: u32) -> Result<u64> {
        let native = Native::process()?;
        let mut value = 0_u64;
        // SAFETY: the output is a live local.
        native.check(unsafe {
            (native.runtime.cnb_get_texture_level_byte_size)(
                self.value(),
                width,
                height,
                depth,
                &mut value,
            )
        })?;
        Ok(value)
    }

    /// Whether the format stores texels in compressed blocks.
    pub fn is_block_compressed(self) -> Result<bool> {
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the output is a live local.
        native.check(unsafe {
            (native.runtime.cnb_is_block_compressed_texture_format)(self.value(), &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// Whether this build knows that raw format identifier at all.
    ///
    /// Takes the raw number rather than a [`Self`] because the question is
    /// asked of a value read out of a file, before it is trusted enough to
    /// become one.
    pub fn is_known(value: u32) -> Result<bool> {
        let native = Native::process()?;
        let mut known = sys::CNA_FALSE;
        // SAFETY: the output is a live local.
        native.check(unsafe { (native.runtime.cnb_is_known_texture_format)(value, &mut known) })?;
        Ok(known != sys::CNA_FALSE)
    }

    /// The CNB format that stores an XNA surface format.
    ///
    /// The inverse of `surface_format`, and not total in the other direction:
    /// CNB carries formats XNA never had.
    pub fn from_surface_format(format: SurfaceFormat) -> Result<Self> {
        let native = Native::process()?;
        let mut value = 0_u32;
        // SAFETY: the output is a live local.
        native.check(unsafe {
            (native.runtime.cnb_texture_format_from_surface_format)(format as u32, &mut value)
        })?;
        Ok(Self::from_value(value))
    }
}

impl crate::extensions::content::CnbAudioFormat {
    /// The format's name.
    pub fn name(self) -> Result<String> {
        let native = Native::process()?;
        let api = &native.runtime;
        let format = self.to_native();
        crate::native::runtime::read_string(
            |value| native.check(value),
            // SAFETY: live outputs; CNA's size-then-copy pair.
            |bytes| unsafe { (api.cnb_get_audio_format_name_size)(format, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_copy_audio_format_name)(format, destination, capacity, written)
            },
        )
    }

    /// How many bytes one frame occupies for that channel count.
    ///
    /// A frame is one sample per channel, so this is what turns a byte count
    /// into a duration -- and for ADPCM it is not simply bytes times channels.
    pub fn frame_bytes(self, channels: u32) -> Result<u32> {
        let native = Native::process()?;
        let mut value = 0_u32;
        // SAFETY: the output is a live local.
        native.check(unsafe {
            (native.runtime.cnb_audio_frame_bytes)(self.to_native(), channels, &mut value)
        })?;
        Ok(value)
    }
}

/// Compresses bytes the way a chunk payload would be stored.
///
/// `level` is the codec's own effort setting. Answers the compressed bytes; the
/// caller keeps the original length, because [`decompress`] needs it.
pub fn compress(bytes: &[u8], compression: Compression, level: i32) -> Result<Vec<u8>> {
    let native = Native::process()?;
    let api = &native.runtime;
    let source = if bytes.is_empty() {
        core::ptr::null()
    } else {
        bytes.as_ptr()
    };
    let mut required = 0_u64;
    // SAFETY: `bytes` is borrowed for the call with its own length.
    native.check(unsafe {
        (api.cnb_get_compressed_byte_count)(
            source,
            bytes.len() as u64,
            compression.0,
            level,
            &mut required,
        )
    })?;
    let capacity = usize::try_from(required)
        .map_err(|_| CnaError::InvalidInput("the compressed payload is too large"))?;
    let mut buffer = vec![0_u8; capacity];
    let mut written = 0_u64;
    // SAFETY: the source is borrowed for the call and the destination holds
    // the byte count just reported.
    native.check(unsafe {
        (api.cnb_copy_compressed)(
            source,
            bytes.len() as u64,
            compression.0,
            level,
            if buffer.is_empty() {
                core::ptr::null_mut()
            } else {
                buffer.as_mut_ptr()
            },
            required,
            &mut written,
        )
    })?;
    buffer.truncate((written as usize).min(capacity));
    Ok(buffer)
}

/// Decompresses bytes stored with that codec.
///
/// `uncompressed_size` is what the chunk entry recorded, and `limit` is the
/// most this call may produce -- a decompression bomb is refused rather than
/// allocated, which is why the size is an argument instead of being discovered.
pub fn decompress(
    bytes: &[u8],
    compression: Compression,
    uncompressed_size: u64,
    limit: u64,
) -> Result<Vec<u8>> {
    let native = Native::process()?;
    let capacity = usize::try_from(uncompressed_size)
        .map_err(|_| CnaError::InvalidInput("the uncompressed payload is too large"))?;
    let mut buffer = vec![0_u8; capacity];
    let mut written = 0_u64;
    // SAFETY: the source is borrowed for the call and the destination holds
    // `uncompressed_size` writable bytes.
    native.check(unsafe {
        (native.runtime.cnb_copy_decompressed)(
            if bytes.is_empty() {
                core::ptr::null()
            } else {
                bytes.as_ptr()
            },
            bytes.len() as u64,
            compression.0,
            uncompressed_size,
            limit,
            if buffer.is_empty() {
                core::ptr::null_mut()
            } else {
                buffer.as_mut_ptr()
            },
            uncompressed_size,
            &mut written,
        )
    })?;
    buffer.truncate((written as usize).min(capacity));
    Ok(buffer)
}

/// Which of a morph target's three vertex streams a call means.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MorphDeltaStream {
    /// Per-vertex position deltas.
    #[default]
    Position,
    /// Per-vertex normal deltas.
    Normal,
    /// Per-vertex tangent deltas.
    Tangent,
}

impl MorphDeltaStream {
    const fn to_native(self) -> sys::CNA_CnbMorphDeltaStream {
        match self {
            Self::Position => sys::CNA_CNB_MORPH_DELTA_POSITION,
            Self::Normal => sys::CNA_CNB_MORPH_DELTA_NORMAL,
            Self::Tangent => sys::CNA_CNB_MORPH_DELTA_TANGENT,
        }
    }
}

/// Which of a morph weight key's three value streams a call means.
///
/// The tangents are only meaningful for a cubic-spline weight track; a step or
/// linear one carries weights alone.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MorphKeyStream {
    #[default]
    Weights,
    InTangent,
    OutTangent,
}

impl MorphKeyStream {
    const fn to_native(self) -> sys::CNA_CnbMorphKeyStream {
        match self {
            Self::Weights => sys::CNA_CNB_MORPH_KEY_WEIGHTS,
            Self::InTangent => sys::CNA_CNB_MORPH_KEY_IN_TANGENT,
            Self::OutTangent => sys::CNA_CNB_MORPH_KEY_OUT_TANGENT,
        }
    }
}

/// Which of a skeleton's three matrix arrays a call means.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum SkeletonMatrixSet {
    /// Each joint's local bind transform.
    #[default]
    BindPose,
    /// Each joint's inverse world bind transform, which is what skinning uses.
    InverseBindPose,
    /// The transform above the skeleton root, when the source declared one.
    RootPrefix,
}

impl SkeletonMatrixSet {
    const fn to_native(self) -> sys::CNA_CnbSkeletonMatrixSet {
        match self {
            Self::BindPose => sys::CNA_CNB_SKELETON_MATRIX_BIND_POSE,
            Self::InverseBindPose => sys::CNA_CNB_SKELETON_MATRIX_INVERSE_BIND_POSE,
            Self::RootPrefix => sys::CNA_CNB_SKELETON_MATRIX_ROOT_PREFIX,
        }
    }
}

/// What a model's skeleton carries.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct SkeletonInfo {
    pub joint_count: u64,
    /// Whether the source declared a transform above the skeleton root.
    pub has_root_prefix: bool,
}

/// What one mesh's morph targets carry.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct MorphInfo {
    pub vertex_count: u32,
    pub target_count: u64,
    pub weight_count: u64,
    pub weight_track_key_count: u64,
    /// Whether normals are recomputed from the blended positions rather than
    /// blended themselves.
    pub recompute_flat_normals: bool,
    pub weight_track_step_interpolation: bool,
    pub weight_track_cubic_spline: bool,
}

/// One key of a morph weight track.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct MorphWeightKeyInfo {
    pub time_seconds: f64,
    pub weight_count: u64,
    pub in_tangent_count: u64,
    pub out_tangent_count: u64,
}

/// A directional light the source scene declared.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct ModelLight {
    pub direction: [f32; 3],
    pub diffuse_color: [f32; 3],
}

/// The sampler a material asked for at one texture slot.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct CnbSamplerState {
    pub filter: u32,
    pub address_u: u32,
    pub address_v: u32,
    /// Whether the source declared a sampler here at all. A slot that declared
    /// none is not the same as one that declared the defaults, and only the
    /// second should override a caller's choice.
    pub declared: bool,
}

/// A `KHR_texture_transform` on one material slot.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct CnbTextureTransform {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    /// Rotation in radians, counter-clockwise about the origin.
    pub rotation: f32,
}

/// The rest of the CNB model schema: animations, lights, morph targets, the
/// skeleton, and the per-slot material state a glTF import records.
///
/// [`crate::extensions::content::CnbModel`] already carried bones, meshes,
/// parts and material colours. What is added here is everything a *glTF*
/// import produces that a hand-authored `.xnb` model never had -- and it is the
/// reading half as much as the writing half, because these are what a `.cnj`
/// asset carries.
impl CnbModel {
    /// The skeleton this model carries, or `None` when it has none.
    pub fn skeleton(&self) -> Result<Option<SkeletonInfo>> {
        let mut info = sys::CNA_CnbSkeletonInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbSkeletonInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_CnbSkeletonInfo::default()
        };
        // SAFETY: the handle is owned and the output is a live local whose size
        // and version headers are set.
        self.native
            .check(unsafe { (self.native.runtime.cnb_model_get_skeleton)(self.handle, &mut info) })?;
        if info.joint_count == 0 {
            return Ok(None);
        }
        Ok(Some(SkeletonInfo {
            joint_count: info.joint_count,
            has_root_prefix: info.has_root_prefix != sys::CNA_FALSE,
        }))
    }

    /// Each joint's parent index, `-1` for a root.
    pub fn skeleton_hierarchy(&self) -> Result<Vec<i32>> {
        let api = &self.native.runtime;
        self.read_array(|destination, capacity, written| {
            // SAFETY: the handle is owned and the destination holds `capacity`
            // writable entries; a null destination with zero capacity is the
            // documented size probe.
            unsafe {
                (api.cnb_model_copy_skeleton_hierarchy)(self.handle, destination, capacity, written)
            }
        })
    }

    /// One of the skeleton's three matrix arrays, as raw floats.
    ///
    /// Sixteen floats per joint, in the order the format stores them. Left as
    /// floats rather than turned into `Matrix` values because the root prefix
    /// set holds one matrix rather than one per joint, and a caller reading a
    /// file should see the shape the file has.
    pub fn skeleton_matrices(&self, set: SkeletonMatrixSet) -> Result<Vec<f32>> {
        let api = &self.native.runtime;
        let native_set = set.to_native();
        self.read_array(|destination, capacity, written| {
            // SAFETY: as above.
            unsafe {
                (api.cnb_model_copy_skeleton_matrices)(
                    self.handle,
                    native_set,
                    destination,
                    capacity,
                    written,
                )
            }
        })
    }

    /// Replaces the skeleton.
    ///
    /// `bind_pose` and `inverse_bind_pose` are sixteen floats per joint;
    /// `root_prefix` is either empty or one matrix. Upstream checks the lengths
    /// against the hierarchy, so a mismatched set is refused rather than
    /// half-applied.
    pub fn set_skeleton(
        &self,
        parents: &[i32],
        bind_pose: &[f32],
        inverse_bind_pose: &[f32],
        root_prefix: &[f32],
    ) -> Result<()> {
        // SAFETY: the handle is owned and all four arrays are borrowed for the
        // call with the joint count they were sized against; a null pointer is
        // paired only with an empty slice.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_set_skeleton)(
                self.handle,
                if parents.is_empty() {
                    core::ptr::null()
                } else {
                    parents.as_ptr()
                },
                parents.len() as u64,
                if bind_pose.is_empty() {
                    core::ptr::null()
                } else {
                    bind_pose.as_ptr()
                },
                if inverse_bind_pose.is_empty() {
                    core::ptr::null()
                } else {
                    inverse_bind_pose.as_ptr()
                },
                if root_prefix.is_empty() {
                    core::ptr::null()
                } else {
                    root_prefix.as_ptr()
                },
            )
        })
    }

    /// Removes the skeleton.
    pub fn clear_skeleton(&self) -> Result<()> {
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.runtime.cnb_model_clear_skeleton)(self.handle) })
    }

    /// One animation's duration, track count and target space.
    pub fn animation(&self, index: u64) -> Result<(f64, u64, ClipTargetSpace)> {
        let mut duration = 0_f64;
        let mut tracks = 0_u64;
        let mut space = 0_u32;
        // SAFETY: the handle is owned and all three outputs are live locals.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_get_animation)(
                self.handle,
                index,
                &mut duration,
                &mut tracks,
                &mut space,
            )
        })?;
        let space = ClipTargetSpace::from_native(space).ok_or(CnaError::InvalidInput(
            "CNA reported a clip target space this build does not know",
        ))?;
        Ok((duration, tracks, space))
    }

    /// One animation's name.
    pub fn animation_name(&self, index: u64) -> Result<String> {
        let api = &self.native.runtime;
        crate::native::runtime::read_string(
            |value| self.native.check(value),
            // SAFETY: owned handle, live outputs; CNA's size-then-copy pair.
            |bytes| unsafe { (api.cnb_model_get_animation_name_size)(self.handle, index, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_model_copy_animation_name)(
                    self.handle,
                    index,
                    destination,
                    capacity,
                    written,
                )
            },
        )
    }

    /// Which bone one track drives, and how many keyframes it has.
    pub fn animation_track(&self, animation: u64, track: u64) -> Result<(i32, u64)> {
        let mut bone = 0_i32;
        let mut keyframes = 0_u64;
        // SAFETY: the handle is owned and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_get_animation_track)(
                self.handle,
                animation,
                track,
                &mut bone,
                &mut keyframes,
            )
        })?;
        Ok((bone, keyframes))
    }

    /// One track's keyframes.
    pub fn animation_keyframes(&self, animation: u64, track: u64) -> Result<Vec<Keyframe>> {
        let api = &self.native.runtime;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        accept_size_probe(&self.native, unsafe {
            (api.cnb_model_copy_animation_keyframes)(
                self.handle,
                animation,
                track,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        })?;
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("more keyframes than fit in memory"))?;
        let mut buffer = vec![sys::CNA_KeyframeEXT::default(); capacity];
        let mut written = 0_u64;
        // SAFETY: the handle is owned and the destination holds `capacity`
        // writable keyframes.
        self.native.check(unsafe {
            (api.cnb_model_copy_animation_keyframes)(
                self.handle,
                animation,
                track,
                if buffer.is_empty() {
                    core::ptr::null_mut()
                } else {
                    buffer.as_mut_ptr()
                },
                required,
                &mut written,
            )
        })?;
        buffer.truncate((written as usize).min(capacity));
        Ok(buffer.into_iter().map(Keyframe::from_native).collect())
    }

    /// Appends one named animation, answering its index.
    pub fn add_animation(
        &self,
        name: &str,
        clip: &AnimationClip,
        target_space: ClipTargetSpace,
    ) -> Result<u64> {
        // The keyframe and track arrays have to outlive the call, so they are
        // staged here and borrowed by the descriptor rather than being
        // temporaries inside an expression.
        let staged = StagedClip::new(clip);
        let descriptor = staged.descriptor;
        let mut index = 0_u64;
        // SAFETY: the handle is owned, and every array the descriptor points at
        // is kept alive by `staged` for the duration of the call.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_add_animation)(
                self.handle,
                string_view(name),
                &descriptor,
                target_space.to_native(),
                &mut index,
            )
        })?;
        Ok(index)
    }

    /// One directional light the source scene declared.
    pub fn light(&self, index: u64) -> Result<ModelLight> {
        let mut value = sys::CNA_CnbModelLight::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_get_light)(self.handle, index, &mut value)
        })?;
        Ok(ModelLight {
            direction: value.direction,
            diffuse_color: value.diffuse_color,
        })
    }

    /// Appends one directional light, answering its index.
    pub fn add_light(&self, light: ModelLight) -> Result<u64> {
        let value = sys::CNA_CnbModelLight {
            direction: light.direction,
            diffuse_color: light.diffuse_color,
        };
        let mut index = 0_u64;
        // SAFETY: the handle is owned and the light is a live local CNA copies
        // during the call.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_add_light)(self.handle, &value, &mut index)
        })?;
        Ok(index)
    }

    fn read_array<T: Copy + Default>(
        &self,
        mut route: impl FnMut(*mut T, u64, *mut u64) -> sys::CNA_Result,
    ) -> Result<Vec<T>> {
        let mut required = 0_u64;
        accept_size_probe(
            &self.native,
            route(core::ptr::null_mut(), 0, &mut required),
        )?;
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("more elements than fit in memory"))?;
        let mut buffer = vec![T::default(); capacity];
        let mut written = 0_u64;
        self.native.check(route(
            if buffer.is_empty() {
                core::ptr::null_mut()
            } else {
                buffer.as_mut_ptr()
            },
            required,
            &mut written,
        ))?;
        buffer.truncate((written as usize).min(capacity));
        Ok(buffer)
    }
}

/// Morph targets, and the material state a glTF import records per slot.
impl CnbModel {
    /// Whether one mesh carries morph targets.
    pub fn has_morph(&self, mesh: u64) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_has_morph)(self.handle, mesh, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// What one mesh's morph targets carry.
    pub fn morph(&self, mesh: u64) -> Result<MorphInfo> {
        let mut info = sys::CNA_CnbMorphInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbMorphInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_CnbMorphInfo::default()
        };
        // SAFETY: the handle is owned and the output is a live local whose size
        // and version headers are set.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_get_morph)(self.handle, mesh, &mut info)
        })?;
        Ok(MorphInfo {
            vertex_count: info.vertex_count,
            target_count: info.target_count,
            weight_count: info.weight_count,
            weight_track_key_count: info.weight_track_key_count,
            recompute_flat_normals: info.recompute_flat_normals != sys::CNA_FALSE,
            weight_track_step_interpolation: info.weight_track_step_interpolation
                != sys::CNA_FALSE,
            weight_track_cubic_spline: info.weight_track_cubic_spline != sys::CNA_FALSE,
        })
    }

    /// Declares one mesh's morph shape, before its targets are filled in.
    pub fn set_morph(&self, mesh: u64, info: MorphInfo) -> Result<()> {
        let value = sys::CNA_CnbMorphInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbMorphInfo>() as u32,
            struct_version: 1,
            vertex_count: info.vertex_count,
            reserved: 0,
            target_count: info.target_count,
            weight_count: info.weight_count,
            weight_track_key_count: info.weight_track_key_count,
            recompute_flat_normals: u8::from(info.recompute_flat_normals),
            weight_track_step_interpolation: u8::from(info.weight_track_step_interpolation),
            weight_track_cubic_spline: u8::from(info.weight_track_cubic_spline),
            reserved2: [0; 5],
        };
        // SAFETY: the handle is owned and the info is a live local CNA copies.
        self.native
            .check(unsafe { (self.native.runtime.cnb_model_set_morph)(self.handle, mesh, &value) })
    }

    /// Removes one mesh's morph targets.
    pub fn clear_morph(&self, mesh: u64) -> Result<()> {
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.runtime.cnb_model_clear_morph)(self.handle, mesh) })
    }

    /// Appends one morph target to a mesh, answering its index.
    pub fn add_morph_target(&self, mesh: u64) -> Result<u64> {
        let mut index = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_add_morph_target)(self.handle, mesh, &mut index)
        })?;
        Ok(index)
    }

    /// One morph target's deltas for one vertex stream.
    ///
    /// Three floats per vertex, in vertex order.
    pub fn morph_target_deltas(
        &self,
        mesh: u64,
        target: u64,
        stream: MorphDeltaStream,
    ) -> Result<Vec<f32>> {
        let api = &self.native.runtime;
        let native_stream = stream.to_native();
        self.read_array(|destination, capacity, written| {
            // SAFETY: owned handle; the destination holds `capacity` floats,
            // and a null destination with zero capacity is the size probe.
            unsafe {
                (api.cnb_model_copy_morph_target_deltas)(
                    self.handle,
                    mesh,
                    target,
                    native_stream,
                    destination,
                    capacity,
                    written,
                )
            }
        })
    }

    /// Replaces one morph target's deltas for one vertex stream.
    pub fn set_morph_target_deltas(
        &self,
        mesh: u64,
        target: u64,
        stream: MorphDeltaStream,
        deltas: &[f32],
    ) -> Result<()> {
        // SAFETY: the handle is owned and the array is borrowed for the call
        // with its own length.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_set_morph_target_deltas)(
                self.handle,
                mesh,
                target,
                stream.to_native(),
                if deltas.is_empty() {
                    core::ptr::null()
                } else {
                    deltas.as_ptr()
                },
                deltas.len() as u64,
            )
        })
    }

    /// One mesh's resting morph weights, one per target.
    pub fn morph_weights(&self, mesh: u64) -> Result<Vec<f32>> {
        let api = &self.native.runtime;
        self.read_array(|destination, capacity, written| {
            // SAFETY: as above.
            unsafe {
                (api.cnb_model_copy_morph_weights)(self.handle, mesh, destination, capacity, written)
            }
        })
    }

    /// Replaces one mesh's resting morph weights.
    pub fn set_morph_weights(&self, mesh: u64, weights: &[f32]) -> Result<()> {
        // SAFETY: the handle is owned and the array is borrowed for the call.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_set_morph_weights)(
                self.handle,
                mesh,
                if weights.is_empty() {
                    core::ptr::null()
                } else {
                    weights.as_ptr()
                },
                weights.len() as u64,
            )
        })
    }

    /// One key of a mesh's morph weight track.
    pub fn morph_weight_key(&self, mesh: u64, key: u64) -> Result<MorphWeightKeyInfo> {
        let mut info = sys::CNA_CnbMorphWeightKeyInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbMorphWeightKeyInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_CnbMorphWeightKeyInfo::default()
        };
        // SAFETY: the handle is owned and the output is a live local whose
        // headers are set.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_get_morph_weight_key)(self.handle, mesh, key, &mut info)
        })?;
        Ok(MorphWeightKeyInfo {
            time_seconds: info.time_seconds,
            weight_count: info.weight_count,
            in_tangent_count: info.in_tangent_count,
            out_tangent_count: info.out_tangent_count,
        })
    }

    /// One weight key's values for one stream.
    pub fn morph_weight_key_values(
        &self,
        mesh: u64,
        key: u64,
        stream: MorphKeyStream,
    ) -> Result<Vec<f32>> {
        let api = &self.native.runtime;
        let native_stream = stream.to_native();
        self.read_array(|destination, capacity, written| {
            // SAFETY: as above.
            unsafe {
                (api.cnb_model_copy_morph_weight_key_values)(
                    self.handle,
                    mesh,
                    key,
                    native_stream,
                    destination,
                    capacity,
                    written,
                )
            }
        })
    }

    /// Appends one key to a mesh's morph weight track, answering its index.
    ///
    /// The tangents are only meaningful for a cubic-spline track; pass empty
    /// slices for a step or linear one.
    pub fn add_morph_weight_key(
        &self,
        mesh: u64,
        time_seconds: f64,
        weights: &[f32],
        in_tangents: &[f32],
        out_tangents: &[f32],
    ) -> Result<u64> {
        fn pointer(values: &[f32]) -> *const f32 {
            if values.is_empty() {
                core::ptr::null()
            } else {
                values.as_ptr()
            }
        }
        let mut index = 0_u64;
        // SAFETY: the handle is owned and all three arrays are borrowed for the
        // call with their own lengths.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_add_morph_weight_key)(
                self.handle,
                mesh,
                time_seconds,
                pointer(weights),
                weights.len() as u64,
                pointer(in_tangents),
                in_tangents.len() as u64,
                pointer(out_tangents),
                out_tangents.len() as u64,
                &mut index,
            )
        })?;
        Ok(index)
    }

    /// The sampler one material declared at one texture slot.
    pub fn material_sampler(&self, material: u64, slot: u64) -> Result<CnbSamplerState> {
        let mut value = sys::CNA_CnbSamplerState::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_get_material_sampler)(
                self.handle,
                material,
                slot,
                &mut value,
            )
        })?;
        Ok(CnbSamplerState {
            filter: value.filter,
            address_u: value.address_u,
            address_v: value.address_v,
            declared: value.declared != sys::CNA_FALSE,
        })
    }

    /// Records the sampler for one material's texture slot.
    pub fn set_material_sampler(
        &self,
        material: u64,
        slot: u64,
        sampler: CnbSamplerState,
    ) -> Result<()> {
        let value = sys::CNA_CnbSamplerState {
            filter: sampler.filter,
            address_u: sampler.address_u,
            address_v: sampler.address_v,
            declared: u8::from(sampler.declared),
            reserved: [0; 3],
        };
        // SAFETY: the handle is owned and the sampler is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_set_material_sampler)(
                self.handle,
                material,
                slot,
                &value,
            )
        })
    }

    /// Which UV set one material's texture slot reads.
    pub fn material_texture_coordinate_set(&self, material: u64, slot: u64) -> Result<u8> {
        let mut value = 0_u8;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_get_material_texture_coordinate_set)(
                self.handle,
                material,
                slot,
                &mut value,
            )
        })?;
        Ok(value)
    }

    /// Records which UV set one material's texture slot reads.
    pub fn set_material_texture_coordinate_set(
        &self,
        material: u64,
        slot: u64,
        coordinate_set: u8,
    ) -> Result<()> {
        // SAFETY: the handle is owned and the value is by value.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_set_material_texture_coordinate_set)(
                self.handle,
                material,
                slot,
                coordinate_set,
            )
        })
    }

    /// The `KHR_texture_transform` on one material's texture slot.
    pub fn material_texture_transform(
        &self,
        material: u64,
        slot: u64,
    ) -> Result<CnbTextureTransform> {
        let mut value = sys::CNA_CnbTextureTransform::default();
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_get_material_texture_transform)(
                self.handle,
                material,
                slot,
                &mut value,
            )
        })?;
        Ok(CnbTextureTransform {
            offset_x: value.offset_x,
            offset_y: value.offset_y,
            scale_x: value.scale_x,
            scale_y: value.scale_y,
            rotation: value.rotation,
        })
    }

    /// Records a `KHR_texture_transform` on one material's texture slot.
    pub fn set_material_texture_transform(
        &self,
        material: u64,
        slot: u64,
        transform: CnbTextureTransform,
    ) -> Result<()> {
        let value = sys::CNA_CnbTextureTransform {
            offset_x: transform.offset_x,
            offset_y: transform.offset_y,
            scale_x: transform.scale_x,
            scale_y: transform.scale_y,
            rotation: transform.rotation,
        };
        // SAFETY: the handle is owned and the transform is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_set_material_texture_transform)(
                self.handle,
                material,
                slot,
                &value,
            )
        })
    }

    /// The effect asset one part names, or an empty string for none.
    ///
    /// A part may point at an effect file rather than carrying one of CNA's
    /// stock kinds, and this is that name.
    pub fn part_external_effect(&self, part: u64) -> Result<String> {
        let api = &self.native.runtime;
        crate::native::runtime::read_string(
            |value| self.native.check(value),
            // SAFETY: owned handle, live outputs; CNA's size-then-copy pair.
            |bytes| unsafe { (api.cnb_model_get_part_external_effect_size)(self.handle, part, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_model_copy_part_external_effect)(
                    self.handle,
                    part,
                    destination,
                    capacity,
                    written,
                )
            },
        )
    }
}

/// Building a texture's pixel data, and choosing a representation to upload.
impl CnbTextureData {
    /// Starts empty texture data of that shape.
    ///
    /// `face_count` is 1 for a 2D or volume texture and 6 for a cube map;
    /// `depth` is 1 for anything but a volume texture.
    pub fn new(
        width: u32,
        height: u32,
        depth: u32,
        face_count: u32,
        mip_count: u32,
    ) -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local receiving a new owned handle.
        native.check(unsafe {
            (native.runtime.cnb_texture_data_create)(
                width,
                height,
                depth,
                face_count,
                mip_count,
                &mut handle,
            )
        })?;
        Ok(Self { native, handle })
    }

    /// Adds an alternative storage format for the same pixels.
    ///
    /// A `.cnb` texture may carry the same image several times over -- once
    /// uncompressed and once per block format -- so one file serves machines
    /// with different capabilities. Answers the new representation's index.
    pub fn add_representation(
        &self,
        format: crate::extensions::content::CnbTextureFormat,
    ) -> Result<u64> {
        let mut index = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_texture_data_add_representation)(
                self.handle,
                format.value(),
                &mut index,
            )
        })
        .map(|()| index)
    }

    /// Fills one mip level of one representation.
    pub fn set_level(&self, representation: u64, level: u64, bytes: &[u8]) -> Result<()> {
        // SAFETY: the handle is owned and `bytes` is borrowed for the call with
        // its own length.
        self.native.check(unsafe {
            (self.native.runtime.cnb_texture_data_set_level)(
                self.handle,
                representation,
                level,
                if bytes.is_empty() {
                    core::ptr::null()
                } else {
                    bytes.as_ptr()
                },
                bytes.len() as u64,
            )
        })
    }

    /// The first representation this machine can upload, in file order.
    ///
    /// `supported` is asked once per representation, in order, and the first
    /// `true` wins -- so the order a file was authored in is the preference
    /// order. `None` means the caller can upload none of them, which is a real
    /// answer for a file built for hardware this machine does not have.
    ///
    /// The predicate runs inside CNA's call and must not call back into this
    /// crate; a panic is caught at the boundary and reported as "unsupported"
    /// for that representation, because there is nowhere to unwind to.
    pub fn select_representation(
        &self,
        mut supported: impl FnMut(crate::extensions::content::CnbTextureFormat) -> bool,
    ) -> Result<Option<u64>> {
        type Predicate<'a> =
            &'a mut dyn FnMut(crate::extensions::content::CnbTextureFormat) -> bool;

        unsafe extern "C" fn trampoline(
            format: sys::CNA_CnbTextureFormat,
            context: *mut core::ffi::c_void,
        ) -> sys::CNA_Bool {
            if context.is_null() {
                return sys::CNA_FALSE;
            }
            // SAFETY: the context is the predicate this call put there, and
            // upstream documents the pointer as never retained past the call.
            let predicate = unsafe { &mut *context.cast::<Predicate<'_>>() };
            let answer = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                predicate(crate::extensions::content::CnbTextureFormat::from_value(format))
            }));
            match answer {
                Ok(true) => sys::CNA_TRUE,
                // A panic means the caller cannot answer, and "cannot upload"
                // is the safe reading of that.
                Ok(false) | Err(_) => sys::CNA_FALSE,
            }
        }

        let mut predicate: Predicate<'_> = &mut supported;
        let context = core::ptr::addr_of_mut!(predicate).cast::<core::ffi::c_void>();
        let mut found = sys::CNA_FALSE;
        let mut index = 0_u64;
        // SAFETY: the trampoline has the audited signature, the context points
        // at a local that outlives the call, and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.runtime.cnb_texture_data_select_representation)(
                self.handle,
                Some(trampoline),
                context,
                &mut found,
                &mut index,
            )
        })?;
        Ok((found != sys::CNA_FALSE).then_some(index))
    }
}

impl CnbSpriteFont {
    /// Replaces one glyph in the font being authored.
    pub fn set_glyph(
        &self,
        index: u64,
        glyph: crate::extensions::content::CnbGlyph,
    ) -> Result<()> {
        let native_glyph = glyph.to_native();
        // SAFETY: the handle is owned and the glyph is a live local CNA copies
        // during the call.
        self.native.check(unsafe {
            (self.native.runtime.cnb_sprite_font_data_set_glyph)(
                self.handle,
                index,
                &native_glyph,
            )
        })
    }
}

impl CnbModel {
    /// Replaces one part's shape.
    ///
    /// The counterpart of reading it back, for a caller editing a model it
    /// decoded rather than authoring one from nothing.
    pub fn set_part(
        &self,
        index: u64,
        part: crate::extensions::content::CnbModelPart,
    ) -> Result<()> {
        let info = part.to_native();
        // SAFETY: the handle is owned and the info is a live local CNA copies.
        self.native
            .check(unsafe { (self.native.runtime.cnb_model_set_part)(self.handle, index, &info) })
    }
}

/// An animation clip decoded from a standalone `.cnb` document.
///
/// Read-only: it is what came out of a file. Building one is
/// [`crate::extensions::models::AnimationClip`] and [`encode_animation_clip`].
#[derive(Debug)]
pub struct CnbAnimationClip {
    native: Arc<Native>,
    handle: Mutex<sys::CNA_CnbAnimationClipHandle>,
}

impl CnbAnimationClip {
    fn get(&self) -> Result<sys::CNA_CnbAnimationClipHandle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput("the animation clip has been released"));
        }
        Ok(handle)
    }

    /// The clip's duration, track count and target space.
    pub fn info(&self) -> Result<(f64, u64, ClipTargetSpace)> {
        let handle = self.get()?;
        let mut duration = 0_f64;
        let mut tracks = 0_u64;
        let mut space = 0_u32;
        // SAFETY: the handle is owned and all three outputs are live locals.
        self.native.check(unsafe {
            (self.native.runtime.cnb_animation_clip_get)(
                handle,
                &mut duration,
                &mut tracks,
                &mut space,
            )
        })?;
        let space = ClipTargetSpace::from_native(space).ok_or(CnaError::InvalidInput(
            "CNA reported a clip target space this build does not know",
        ))?;
        Ok((duration, tracks, space))
    }

    /// Which bone one track drives, and how many keyframes it has.
    pub fn track(&self, index: u64) -> Result<(i32, u64)> {
        let handle = self.get()?;
        let mut bone = 0_i32;
        let mut keyframes = 0_u64;
        // SAFETY: the handle is owned and both outputs are live locals.
        self.native.check(unsafe {
            (self.native.runtime.cnb_animation_clip_get_track)(
                handle,
                index,
                &mut bone,
                &mut keyframes,
            )
        })?;
        Ok((bone, keyframes))
    }

    /// One track's keyframes.
    pub fn keyframes(&self, track: u64) -> Result<Vec<Keyframe>> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the count.
        accept_size_probe(&self.native, unsafe {
            (api.cnb_animation_clip_copy_keyframes)(
                handle,
                track,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        })?;
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("more keyframes than fit in memory"))?;
        let mut buffer = vec![sys::CNA_KeyframeEXT::default(); capacity];
        let mut written = 0_u64;
        // SAFETY: the handle is owned and the destination holds `capacity`
        // writable keyframes.
        self.native.check(unsafe {
            (api.cnb_animation_clip_copy_keyframes)(
                handle,
                track,
                if buffer.is_empty() {
                    core::ptr::null_mut()
                } else {
                    buffer.as_mut_ptr()
                },
                required,
                &mut written,
            )
        })?;
        buffer.truncate((written as usize).min(capacity));
        Ok(buffer.into_iter().map(Keyframe::from_native).collect())
    }

    /// Releases the clip early.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle is owned by this value and released exactly once.
        self.native
            .check(unsafe { (self.native.runtime.cnb_animation_clip_destroy)(handle) })
    }
}

impl Drop for CnbAnimationClip {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// Decoding the asset types the document layer did not already cover.
impl CnbDocument {
    /// The animation clip this document carries.
    pub fn decode_animation_clip(&self) -> Result<CnbAnimationClip> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local receiving
        // a new owned handle.
        self.native.check(unsafe {
            (self.native.runtime.cnb_decode_animation_clip)(self.handle, &mut handle)
        })?;
        Ok(CnbAnimationClip {
            native: Arc::clone(&self.native),
            handle: Mutex::new(handle),
        })
    }

    /// The volume texture this document carries.
    pub fn decode_texture3d(&self) -> Result<CnbTextureData> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: as above.
        self.native.check(unsafe {
            (self.native.runtime.cnb_decode_texture3d)(self.handle, &mut handle)
        })?;
        Ok(CnbTextureData {
            native: Arc::clone(&self.native),
            handle,
        })
    }

    /// The cube map this document carries.
    pub fn decode_texture_cube(&self) -> Result<CnbTextureData> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: as above.
        self.native.check(unsafe {
            (self.native.runtime.cnb_decode_texture_cube)(self.handle, &mut handle)
        })?;
        Ok(CnbTextureData {
            native: Arc::clone(&self.native),
            handle,
        })
    }

    /// The song this document names: its display name, duration and stream.
    ///
    /// A song document carries a *reference* rather than audio: the stream is a
    /// path the media player opens, which is why nothing here decodes samples.
    pub fn decode_song(&self) -> Result<(String, u32, String)> {
        let api = &self.native.runtime;
        let mut milliseconds = 0_u32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (api.cnb_decode_song_duration_milliseconds)(self.handle, &mut milliseconds)
        })?;
        let name = crate::native::runtime::read_string(
            |value| self.native.check(value),
            // SAFETY: owned handle, live outputs; CNA's size-then-copy pair.
            |bytes| unsafe { (api.cnb_decode_song_name_size)(self.handle, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_decode_song_name)(self.handle, destination, capacity, written)
            },
        )?;
        let stream = crate::native::runtime::read_string(
            |value| self.native.check(value),
            // SAFETY: as above.
            |bytes| unsafe { (api.cnb_decode_song_stream_reference_size)(self.handle, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_decode_song_stream_reference)(
                    self.handle,
                    destination,
                    capacity,
                    written,
                )
            },
        )?;
        Ok((name, milliseconds, stream))
    }

    /// The video this document names: its shape and its stream reference.
    pub fn decode_video(&self) -> Result<(VideoInfo, String)> {
        let api = &self.native.runtime;
        let mut info = sys::CNA_CnbVideoInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbVideoInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_CnbVideoInfo::default()
        };
        // SAFETY: the handle is owned and the output is a live local whose
        // headers are set.
        self.native
            .check(unsafe { (api.cnb_decode_video)(self.handle, &mut info) })?;
        let stream = crate::native::runtime::read_string(
            |value| self.native.check(value),
            // SAFETY: owned handle, live outputs; CNA's size-then-copy pair.
            |bytes| unsafe { (api.cnb_decode_video_stream_reference_size)(self.handle, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_decode_video_stream_reference)(
                    self.handle,
                    destination,
                    capacity,
                    written,
                )
            },
        )?;
        Ok((
            VideoInfo {
                duration_milliseconds: info.duration_milliseconds,
                width: info.width,
                height: info.height,
                frames_per_second: info.frames_per_second,
                soundtrack_type: info.soundtrack_type,
            },
            stream,
        ))
    }
}

/// A video's shape, as a `.cnb` document records it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct VideoInfo {
    pub duration_milliseconds: u32,
    pub width: u32,
    pub height: u32,
    pub frames_per_second: f32,
    /// XNA's `VideoSoundtrackType`, as its raw identity.
    pub soundtrack_type: u32,
}

/// Producing a whole `.cnb` document for one asset, without a writer.
///
/// The counterpart of the decoders above. Each answers the complete file bytes
/// rather than a chunk, because these asset types have one canonical layout and
/// letting a caller assemble it chunk by chunk would be an invitation to
/// produce a document CNA's own reader refuses.
///
/// `content_name` is what the document records for diagnostics; an asset name
/// is the usual choice.
pub fn encode_animation_clip(
    clip: &AnimationClip,
    target_space: ClipTargetSpace,
    content_name: &str,
) -> Result<Vec<u8>> {
    let native = Native::process()?;
    // The keyframe and track arrays have to outlive the call.
    let staged = StagedClip::new(clip);
    let descriptor = staged.descriptor;
    let api = &native.runtime;
    read_encoded(&native, |destination, capacity, written| {
        // SAFETY: every array the descriptor points at is kept alive by
        // `staged`, the name is borrowed for the call, and a null destination
        // with zero capacity is the documented size probe.
        unsafe {
            (api.cnb_encode_animation_clip)(
                &descriptor,
                target_space.to_native(),
                string_view(content_name),
                destination,
                capacity,
                written,
            )
        }
    })
}

/// Produces a `.cnb` song document: a stream reference, a name and a duration.
///
/// A song document is **metadata plus a reference**, never embedded audio: a
/// song can be hundreds of megabytes and wants streaming, so the media stays
/// beside the file and is recorded as its one external reference. That is what
/// makes the dependency visible to a build tool.
///
/// The argument order mirrors upstream's, stream first, because the stream is
/// the required one -- an empty reference is refused, an empty display name is
/// not.
pub fn encode_song(
    stream_reference: &str,
    name: &str,
    duration_milliseconds: u32,
    content_name: &str,
) -> Result<Vec<u8>> {
    let native = Native::process()?;
    let api = &native.runtime;
    read_encoded(&native, |destination, capacity, written| {
        // SAFETY: all three names are borrowed for the call.
        unsafe {
            (api.cnb_encode_song)(
                string_view(stream_reference),
                string_view(name),
                duration_milliseconds,
                string_view(content_name),
                destination,
                capacity,
                written,
            )
        }
    })
}

/// Produces a `.cnb` video document: a stream reference and the video's shape.
pub fn encode_video(
    stream_reference: &str,
    info: VideoInfo,
    content_name: &str,
) -> Result<Vec<u8>> {
    let native = Native::process()?;
    let value = sys::CNA_CnbVideoInfo {
        struct_size: core::mem::size_of::<sys::CNA_CnbVideoInfo>() as u32,
        struct_version: 1,
        duration_milliseconds: info.duration_milliseconds,
        width: info.width,
        height: info.height,
        frames_per_second: info.frames_per_second,
        soundtrack_type: info.soundtrack_type,
        reserved: 0,
    };
    let api = &native.runtime;
    read_encoded(&native, |destination, capacity, written| {
        // SAFETY: the info outlives the call and both names are borrowed.
        unsafe {
            (api.cnb_encode_video)(
                string_view(stream_reference),
                &value,
                string_view(content_name),
                destination,
                capacity,
                written,
            )
        }
    })
}

/// Produces a `.cnb` volume-texture document.
pub fn encode_texture3d(texture: &CnbTextureData, content_name: &str) -> Result<Vec<u8>> {
    let native = Native::process()?;
    let api = &native.runtime;
    let handle = texture.handle;
    read_encoded(&native, |destination, capacity, written| {
        // SAFETY: the texture handle is owned by a live value and the name is
        // borrowed for the call.
        unsafe {
            (api.cnb_encode_texture3d)(
                handle,
                string_view(content_name),
                destination,
                capacity,
                written,
            )
        }
    })
}

/// Produces a `.cnb` cube-map document.
pub fn encode_texture_cube(texture: &CnbTextureData, content_name: &str) -> Result<Vec<u8>> {
    let native = Native::process()?;
    let api = &native.runtime;
    let handle = texture.handle;
    read_encoded(&native, |destination, capacity, written| {
        // SAFETY: as above.
        unsafe {
            (api.cnb_encode_texture_cube)(
                handle,
                string_view(content_name),
                destination,
                capacity,
                written,
            )
        }
    })
}

fn read_encoded(
    native: &Arc<Native>,
    mut route: impl FnMut(*mut u8, u64, *mut u64) -> sys::CNA_Result,
) -> Result<Vec<u8>> {
    let mut required = 0_u64;
    accept_size_probe(native, route(core::ptr::null_mut(), 0, &mut required))?;
    let capacity = usize::try_from(required)
        .map_err(|_| CnaError::InvalidInput("the encoded document is too large"))?;
    let mut buffer = vec![0_u8; capacity];
    let mut written = 0_u64;
    native.check(route(
        if buffer.is_empty() {
            core::ptr::null_mut()
        } else {
            buffer.as_mut_ptr()
        },
        required,
        &mut written,
    ))?;
    buffer.truncate((written as usize).min(capacity));
    Ok(buffer)
}

/// Importing a source asset a content pipeline would otherwise have to parse.
///
/// These read the *authoring* formats -- PNG, DDS, WAV -- rather than `.cnb`,
/// and hand back the same in-memory shapes the decoders do. Together with the
/// encoders above they are a content pipeline: read a source file, produce a
/// document.
pub fn import_image_as_texture2d(
    path: &str,
    color_key: Option<[u8; 3]>,
) -> Result<CnbTextureData> {
    let native = Native::process()?;
    let options = sys::CNA_CnbImageImportOptions {
        struct_size: core::mem::size_of::<sys::CNA_CnbImageImportOptions>() as u32,
        struct_version: 1,
        color_key: color_key.unwrap_or([0; 3]),
        has_color_key: u8::from(color_key.is_some()),
    };
    let mut handle = sys::CNA_INVALID_HANDLE;
    // SAFETY: the path and the options are borrowed for the call, and the
    // output is a live local receiving a new owned handle.
    native.check(unsafe {
        (native.runtime.cnb_import_image_as_texture2d)(
            string_view(path),
            &options,
            &mut handle,
        )
    })?;
    Ok(CnbTextureData { native, handle })
}

/// Reads a `.dds` file as cube-map pixel data.
pub fn import_dds_as_texture_cube(path: &str) -> Result<CnbTextureData> {
    let native = Native::process()?;
    let mut handle = sys::CNA_INVALID_HANDLE;
    // SAFETY: the path is borrowed for the call and the output is a live local.
    native.check(unsafe {
        (native.runtime.cnb_import_dds_as_texture_cube)(string_view(path), &mut handle)
    })?;
    Ok(CnbTextureData { native, handle })
}

/// Reads `.dds` bytes already in memory as cube-map pixel data.
///
/// `origin` names the source in diagnostics.
pub fn decode_dds_as_texture_cube(bytes: &[u8], origin: &str) -> Result<CnbTextureData> {
    let native = Native::process()?;
    let mut handle = sys::CNA_INVALID_HANDLE;
    // SAFETY: the bytes and the origin are borrowed for the call, and the
    // output is a live local.
    native.check(unsafe {
        (native.runtime.cnb_decode_dds_as_texture_cube)(
            if bytes.is_empty() {
                core::ptr::null()
            } else {
                bytes.as_ptr()
            },
            bytes.len() as u64,
            string_view(origin),
            &mut handle,
        )
    })?;
    Ok(CnbTextureData { native, handle })
}

/// Reads a `.wav` file as sound-effect data.
pub fn import_wav_as_sound_effect(path: &str) -> Result<CnbSoundEffect> {
    let native = Native::process()?;
    let mut handle = sys::CNA_INVALID_HANDLE;
    // SAFETY: the path is borrowed for the call and the output is a live local.
    native.check(unsafe {
        (native.runtime.cnb_import_wav_as_sound_effect)(string_view(path), &mut handle)
    })?;
    Ok(CnbSoundEffect { native, handle })
}

/// Reads `.wav` bytes already in memory as sound-effect data.
pub fn decode_wav_as_sound_effect(bytes: &[u8], origin: &str) -> Result<CnbSoundEffect> {
    let native = Native::process()?;
    let mut handle = sys::CNA_INVALID_HANDLE;
    // SAFETY: the bytes and the origin are borrowed for the call.
    native.check(unsafe {
        (native.runtime.cnb_decode_wav_as_sound_effect)(
            if bytes.is_empty() {
                core::ptr::null()
            } else {
                bytes.as_ptr()
            },
            bytes.len() as u64,
            string_view(origin),
            &mut handle,
        )
    })?;
    Ok(CnbSoundEffect { native, handle })
}

/// The result of compiling one `.cnj` document into a `.cnb` file image.
///
/// # Why this matters for a Rust game
///
/// `.cnj` is the shape CNA's own glTF import writes, and this crate has no
/// reader for it -- its content pipeline reads `.xnb`. So a game that wants to
/// ship a model imported from glTF has, until now, had no way to turn CNA's own
/// intermediate into anything it could load. This is that way: compile the
/// `.cnj`, get the bytes, and hand them to [`CnbDocument::parse`] or write them
/// to a file.
///
/// The result also says what the compile *consumed*, which is what a build
/// script needs: the absorbed files are now inside the `.cnb` and no longer
/// need to ship, and the external references are the ones that still do.
#[derive(Debug)]
pub struct CnjCompileResult {
    native: Arc<Native>,
    handle: Mutex<sys::CNA_CnjToCnbResultHandle>,
}

impl CnjCompileResult {
    fn get(&self) -> Result<sys::CNA_CnjToCnbResultHandle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput("the compile result has been released"));
        }
        Ok(handle)
    }

    /// The compiled `.cnb` bytes.
    pub fn bytes(&self) -> Result<Vec<u8>> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        read_encoded(&self.native, |destination, capacity, written| {
            // SAFETY: the handle is owned; a null destination with zero
            // capacity is the documented size probe.
            unsafe { (api.cnb_cnj_result_copy_bytes)(handle, destination, capacity, written) }
        })
    }

    /// The asset type the compiled document is.
    pub fn asset_type(&self) -> Result<u32> {
        let handle = self.get()?;
        let mut value = 0_u32;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_cnj_result_get_asset_type_id)(handle, &mut value)
        })?;
        Ok(value)
    }

    /// The asset type's canonical name.
    pub fn asset_type_name(&self) -> Result<String> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        crate::native::runtime::read_string(
            |value| self.native.check(value),
            // SAFETY: owned handle, live outputs; CNA's size-then-copy pair.
            |bytes| unsafe { (api.cnb_cnj_result_get_asset_type_name_size)(handle, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_cnj_result_copy_asset_type_name)(handle, destination, capacity, written)
            },
        )
    }

    /// The source files the compile absorbed, as the document wrote them.
    ///
    /// Their contents are now inside the `.cnb` and no longer need to ship.
    /// Paths are the authored ones rather than resolved filesystem paths, so a
    /// build script can match them against what it generated. The `.cnj` itself
    /// is always first.
    pub fn absorbed_files(&self) -> Result<Vec<String>> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (api.cnb_cnj_result_get_absorbed_file_count)(handle, &mut count) })?;
        (0..count)
            .map(|index| {
                crate::native::runtime::read_string(
                    |value| self.native.check(value),
                    // SAFETY: owned handle, live outputs.
                    |bytes| unsafe {
                        (api.cnb_cnj_result_get_absorbed_file_size)(handle, index, bytes)
                    },
                    |destination, capacity, written| unsafe {
                        (api.cnb_cnj_result_copy_absorbed_file)(
                            handle,
                            index,
                            destination,
                            capacity,
                            written,
                        )
                    },
                )
            })
            .collect()
    }

    /// The logical names the compiled asset still refers to.
    ///
    /// These are what must still be present at load time, and what a build
    /// script has to keep shipping.
    pub fn external_references(&self) -> Result<Vec<String>> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (api.cnb_cnj_result_get_external_reference_count)(handle, &mut count)
        })?;
        (0..count)
            .map(|index| {
                crate::native::runtime::read_string(
                    |value| self.native.check(value),
                    // SAFETY: owned handle, live outputs.
                    |bytes| unsafe {
                        (api.cnb_cnj_result_get_external_reference_size)(handle, index, bytes)
                    },
                    |destination, capacity, written| unsafe {
                        (api.cnb_cnj_result_copy_external_reference)(
                            handle,
                            index,
                            destination,
                            capacity,
                            written,
                        )
                    },
                )
            })
            .collect()
    }

    /// Releases the result early.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle is owned by this value and released exactly once.
        self.native
            .check(unsafe { (self.native.runtime.cnb_cnj_result_destroy)(handle) })
    }
}

impl Drop for CnjCompileResult {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// Compiles one `.cnj` document, and the binary sidecars it names, into a
/// `.cnb` file image.
///
/// All eight asset types are supported -- `Curve`, `AnimationClip`, `Model`,
/// `Texture2D`, `Texture3D`, `TextureCube`, `SpriteFont` and `SoundEffect`.
/// Any other type is refused by name rather than producing an empty file.
///
/// `content_root` is the directory sidecar references resolve against; pass an
/// empty string for the document's own parent directory, which is where CNA's
/// content tools write them. `content_name` is recorded for diagnostics; empty
/// means the document's stem.
pub fn compile_cnj(
    cnj_path: &str,
    content_root: &str,
    content_name: &str,
) -> Result<CnjCompileResult> {
    let native = Native::process()?;
    let mut handle = sys::CNA_INVALID_HANDLE;
    // SAFETY: all three paths are borrowed for the call and the output is a
    // live local receiving a new owned handle.
    native.check(unsafe {
        (native.runtime.cnb_compile_cnj)(
            string_view(cnj_path),
            string_view(content_root),
            string_view(content_name),
            &mut handle,
        )
    })?;
    Ok(CnjCompileResult {
        native,
        handle: Mutex::new(handle),
    })
}

/// A `.cnj` model compiled into a model description rather than into bytes.
///
/// The difference from [`compile_cnj`] is the shape of the answer: this hands
/// back a [`CnbModel`] to inspect or edit, where the other hands back the file.
#[derive(Debug)]
pub struct CnjModelBuild {
    native: Arc<Native>,
    handle: Mutex<sys::CNA_CnbModelFromCnjHandle>,
}

impl CnjModelBuild {
    fn get(&self) -> Result<sys::CNA_CnbModelFromCnjHandle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == sys::CNA_INVALID_HANDLE {
            return Err(CnaError::InvalidInput("the model build has been released"));
        }
        Ok(handle)
    }

    /// Takes the model out of the build.
    ///
    /// Consuming rather than borrowing, because upstream hands over the
    /// description: a second call has nothing left to give, and the build is
    /// gone either way.
    pub fn take_model(self) -> Result<CnbModel> {
        let handle = self.get()?;
        let mut model = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local receiving
        // the model handle this build was holding.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_from_cnj_take_model)(handle, &mut model)
        })?;
        Ok(CnbModel {
            native: Arc::clone(&self.native),
            handle: model,
        })
    }

    /// The source files the build absorbed, as the document wrote them.
    pub fn absorbed_files(&self) -> Result<Vec<String>> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (api.cnb_model_from_cnj_get_absorbed_file_count)(handle, &mut count)
        })?;
        (0..count)
            .map(|index| {
                crate::native::runtime::read_string(
                    |value| self.native.check(value),
                    // SAFETY: owned handle, live outputs.
                    |bytes| unsafe {
                        (api.cnb_model_from_cnj_get_absorbed_file_size)(handle, index, bytes)
                    },
                    |destination, capacity, written| unsafe {
                        (api.cnb_model_from_cnj_copy_absorbed_file)(
                            handle,
                            index,
                            destination,
                            capacity,
                            written,
                        )
                    },
                )
            })
            .collect()
    }

    /// The logical names the built model still refers to.
    pub fn external_references(&self) -> Result<Vec<String>> {
        let handle = self.get()?;
        let api = &self.native.runtime;
        let mut count = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (api.cnb_model_from_cnj_get_external_reference_count)(handle, &mut count)
        })?;
        (0..count)
            .map(|index| {
                crate::native::runtime::read_string(
                    |value| self.native.check(value),
                    // SAFETY: owned handle, live outputs.
                    |bytes| unsafe {
                        (api.cnb_model_from_cnj_get_external_reference_size)(handle, index, bytes)
                    },
                    |destination, capacity, written| unsafe {
                        (api.cnb_model_from_cnj_copy_external_reference)(
                            handle,
                            index,
                            destination,
                            capacity,
                            written,
                        )
                    },
                )
            })
            .collect()
    }

    /// Releases the build early.
    pub fn release(&self) -> Result<()> {
        let mut guard = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = core::mem::replace(&mut *guard, sys::CNA_INVALID_HANDLE);
        if handle == sys::CNA_INVALID_HANDLE {
            return Ok(());
        }
        // SAFETY: the handle is owned by this value and released exactly once.
        self.native
            .check(unsafe { (self.native.runtime.cnb_model_from_cnj_destroy)(handle) })
    }
}

impl Drop for CnjModelBuild {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// Compiles a `.cnj` model and its sidecars into a model description.
pub fn build_model_from_cnj(cnj_path: &str, content_root: &str) -> Result<CnjModelBuild> {
    let native = Native::process()?;
    let mut handle = sys::CNA_INVALID_HANDLE;
    // SAFETY: both paths are borrowed for the call and the output is a live
    // local receiving a new owned handle.
    native.check(unsafe {
        (native.runtime.cnb_build_model_from_cnj)(
            string_view(cnj_path),
            string_view(content_root),
            &mut handle,
        )
    })?;
    Ok(CnjModelBuild {
        native,
        handle: Mutex::new(handle),
    })
}

/// Moving a Rust [`Curve`] through CNB's codec.
///
/// A `Curve` is one of CNB's eight asset types, so a game may well have one in
/// its content. The arithmetic stays Rust's -- `cna::value::curve` evaluates,
/// loops and computes tangents itself, and none of that is bound -- but the
/// codec speaks in native curve handles, so these two functions build one,
/// hand it over, and take the answer apart again. No native curve escapes.
struct NativeCurve {
    native: Arc<Native>,
    handle: sys::CNA_CurveHandle,
}

impl NativeCurve {
    /// Builds a native curve carrying the same keys and loop modes.
    fn from_curve(native: &Arc<Native>, curve: &Curve) -> Result<Self> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local receiving a new owned handle.
        native.check(unsafe { (native.runtime.curve_create)(&mut handle) })?;
        let owned = Self {
            native: Arc::clone(native),
            handle,
        };
        // SAFETY: the handle is owned and the loop values are by value.
        native.check(unsafe {
            (native.runtime.curve_set_pre_loop)(handle, curve.PreLoop() as u32)
        })?;
        // SAFETY: as above.
        native.check(unsafe {
            (native.runtime.curve_set_post_loop)(handle, curve.PostLoop() as u32)
        })?;

        let mut keys = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        native.check(unsafe { (native.runtime.curve_get_keys)(handle, &mut keys) })?;
        let keys = NativeKeys {
            native: Arc::clone(native),
            handle: keys,
        };
        for index in 0..curve.Keys().Count() {
            let key = curve.Keys().Item(index);
            let value = sys::CNA_CurveKey {
                position: key.Position(),
                value: key.Value(),
                tangent_in: key.TangentIn(),
                tangent_out: key.TangentOut(),
                continuity: key.Continuity() as u32,
            };
            // SAFETY: the collection handle is owned and the key is by value.
            native.check(unsafe {
                (native.runtime.curve_key_collection_add)(keys.handle, value)
            })?;
        }
        Ok(owned)
    }

    /// Takes a native curve apart into a Rust one.
    fn to_curve(&self) -> Result<Curve> {
        let api = &self.native.runtime;
        let mut curve = Curve::new();
        let mut pre = 0_u32;
        let mut post = 0_u32;
        // SAFETY: the handle is owned and both outputs are live locals.
        self.native
            .check(unsafe { (api.curve_get_pre_loop)(self.handle, &mut pre) })?;
        // SAFETY: as above.
        self.native
            .check(unsafe { (api.curve_get_post_loop)(self.handle, &mut post) })?;
        curve.SetPreLoop(loop_type(pre)?);
        curve.SetPostLoop(loop_type(post)?);

        let mut keys = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native
            .check(unsafe { (api.curve_get_keys)(self.handle, &mut keys) })?;
        let keys = NativeKeys {
            native: Arc::clone(&self.native),
            handle: keys,
        };
        let mut count = 0_u64;
        // SAFETY: the collection handle is owned and the output is a local.
        self.native
            .check(unsafe { (api.curve_key_collection_get_count)(keys.handle, &mut count) })?;
        for index in 0..count {
            let index = i32::try_from(index)
                .map_err(|_| CnaError::InvalidInput("more curve keys than an index can hold"))?;
            let mut key = sys::CNA_CurveKey::default();
            // SAFETY: the collection handle is owned and the output is a local.
            self.native
                .check(unsafe { (api.curve_key_collection_get)(keys.handle, index, &mut key) })?;
            curve.Keys().Add(&CurveKey::
                from_position_and_value_and_tangent_in_and_tangent_out_and_continuity(
                    key.position,
                    key.value,
                    key.tangent_in,
                    key.tangent_out,
                    continuity(key.continuity)?,
                ));
        }
        Ok(curve)
    }
}

impl Drop for NativeCurve {
    fn drop(&mut self) {
        // SAFETY: the handle is this value's own, released exactly once.
        let _ = unsafe { (self.native.runtime.curve_destroy)(self.handle) };
    }
}

/// A key collection handle, released with the value that read it.
struct NativeKeys {
    native: Arc<Native>,
    handle: sys::CNA_CurveKeyCollectionHandle,
}

impl Drop for NativeKeys {
    fn drop(&mut self) {
        // SAFETY: the handle is this value's own, released exactly once.
        let _ = unsafe { (self.native.runtime.curve_key_collection_destroy)(self.handle) };
    }
}

fn loop_type(value: u32) -> Result<CurveLoopType> {
    Ok(match value {
        0 => CurveLoopType::Constant,
        1 => CurveLoopType::Cycle,
        2 => CurveLoopType::CycleOffset,
        3 => CurveLoopType::Oscillate,
        4 => CurveLoopType::Linear,
        _ => {
            return Err(CnaError::InvalidInput(
                "CNA reported a curve loop type this build does not know",
            ))
        }
    })
}

fn continuity(value: u32) -> Result<CurveContinuity> {
    Ok(match value {
        0 => CurveContinuity::Smooth,
        1 => CurveContinuity::Step,
        _ => {
            return Err(CnaError::InvalidInput(
                "CNA reported a curve continuity this build does not know",
            ))
        }
    })
}

impl CnbDocument {
    /// The curve this document carries.
    pub fn decode_curve(&self) -> Result<Curve> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local receiving
        // a new owned curve handle.
        self.native
            .check(unsafe { (self.native.runtime.cnb_decode_curve)(self.handle, &mut handle) })?;
        let native_curve = NativeCurve {
            native: Arc::clone(&self.native),
            handle,
        };
        native_curve.to_curve()
    }
}

/// Produces a `.cnb` curve document.
pub fn encode_curve(curve: &Curve, content_name: &str) -> Result<Vec<u8>> {
    let native = Native::process()?;
    let native_curve = NativeCurve::from_curve(&native, curve)?;
    let handle = native_curve.handle;
    let api = &native.runtime;
    read_encoded(&native, |destination, capacity, written| {
        // SAFETY: the curve handle is owned by `native_curve`, which outlives
        // this closure, and the name is borrowed for the call.
        unsafe {
            (api.cnb_encode_curve)(
                handle,
                string_view(content_name),
                destination,
                capacity,
                written,
            )
        }
    })
}
