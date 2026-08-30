//! CNA's `.cnb` content container.
//!
//! XNA has one content format, `.xnb`, and `ContentManager` reads it. CNA has
//! its own, and this module is where it lives: the strict
//! `Microsoft::Xna::Framework::Content::ContentManager` is never taught to
//! reinterpret a non-XNA format, because a game that asks XNA for an asset
//! must get XNA's answer.
//!
//! The slice implemented here is one complete vertical: build texture data,
//! encode it as a `.cnb` document, parse a document back, read its metadata,
//! and decode a texture out of it. Both handle kinds are owned and released
//! by `Drop`; nothing here borrows a native buffer past a call.

#![allow(clippy::missing_errors_doc)]

use std::path::Path;
use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::graphics::SurfaceFormat;
use crate::native::runtime::read_string;
use crate::native::Native;

/// Sanity bounds applied while parsing a document.
///
/// A `.cnb` file is untrusted input, so the parser is bounded rather than
/// trusting the file's own counts. `None` anywhere means "use CNA's default
/// for this bound", which is what passing no limits at all does.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReadLimits {
    pub max_file_size: Option<u64>,
    pub max_chunk_size: Option<u64>,
    pub max_total_uncompressed_size: Option<u64>,
    pub max_chunk_count: Option<u32>,
    pub max_string_bytes: Option<u32>,
    pub max_array_element_count: Option<u32>,
}

impl ReadLimits {
    fn to_native(self) -> sys::CNA_CnbReadLimits {
        sys::CNA_CnbReadLimits {
            struct_size: core::mem::size_of::<sys::CNA_CnbReadLimits>() as u32,
            struct_version: sys::CNA_CNB_READ_LIMITS_STRUCT_VERSION,
            max_file_size: self.max_file_size.unwrap_or(0),
            max_chunk_size: self.max_chunk_size.unwrap_or(0),
            max_total_uncompressed_size: self.max_total_uncompressed_size.unwrap_or(0),
            max_chunk_count: self.max_chunk_count.unwrap_or(0),
            max_string_bytes: self.max_string_bytes.unwrap_or(0),
            max_array_element_count: self.max_array_element_count.unwrap_or(0),
        }
    }

    const fn is_default(self) -> bool {
        self.max_file_size.is_none()
            && self.max_chunk_size.is_none()
            && self.max_total_uncompressed_size.is_none()
            && self.max_chunk_count.is_none()
            && self.max_string_bytes.is_none()
            && self.max_array_element_count.is_none()
    }
}

/// The optional metadata block a document may carry.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Metadata {
    /// Whether the document carries a metadata block at all.
    pub present: bool,
    pub flags: u32,
    /// The asset type's own name, empty when the block names none.
    pub asset_type_name: String,
    /// The content name the writer recorded, empty when it recorded none.
    pub content_name: String,
}

/// One parsed `.cnb` document.
#[derive(Debug)]
pub struct CnbDocument {
    native: Arc<Native>,
    handle: sys::CNA_CnbDocumentHandle,
}

/// Texture pixels decoded from, or destined for, a `.cnb` document.
#[derive(Debug)]
pub struct CnbTextureData {
    native: Arc<Native>,
    handle: sys::CNA_CnbTextureDataHandle,
}

/// What a texture's pixels are laid out as inside a document.
///
/// It is CNA's own identity, not XNA's `SurfaceFormat`: the container carries
/// formats XNA never had. [`CnbTextureFormat::surface_format`] maps the ones
/// that do have an XNA counterpart.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CnbTextureFormat(u32);

impl CnbTextureFormat {
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Bytes per pixel, or per block for a block-compressed format.
    pub fn unit_bytes(self) -> Result<u32> {
        let native = Native::process()?;
        let mut value = 0;
        // SAFETY: the output is a live local of the declared type.
        native.check(unsafe {
            (native.runtime.cnb_texture_format_unit_bytes)(self.0, &mut value)
        })?;
        Ok(value)
    }

    /// CNA's canonical name for the format.
    pub fn name(self) -> Result<String> {
        let native = Native::process()?;
        let api = &native.runtime;
        read_string(
            |value| native.check(value),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.cnb_texture_format_name_size)(self.0, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_copy_texture_format_name)(self.0, destination, capacity, written)
            },
        )
    }

    /// The XNA `SurfaceFormat` this maps to, when one exists.
    ///
    /// A `.cnb` format with no XNA counterpart reports
    /// [`CnaError::UnsupportedRuntime`] rather than being forced onto the
    /// nearest XNA format.
    pub fn surface_format(self) -> Result<SurfaceFormat> {
        let native = Native::process()?;
        let mut value = 0;
        // SAFETY: the output is a live local of the declared type.
        native.check(unsafe {
            (native.runtime.cnb_texture_format_to_surface_format)(self.0, &mut value)
        })?;
        SurfaceFormat::from_native(value).ok_or(CnaError::UnsupportedRuntime(
            "this .cnb texture format has no XNA SurfaceFormat counterpart",
        ))
    }
}

/// The numeric identity of an asset type inside a `.cnb` document.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssetTypeId(u32);

impl AssetTypeId {
    pub const INVALID: Self = Self(sys::CNA_CNB_ASSET_TYPE_INVALID);
    pub const TEXTURE2D: Self = Self(sys::CNA_CNB_ASSET_TYPE_TEXTURE2D);
    pub const TEXTURE3D: Self = Self(sys::CNA_CNB_ASSET_TYPE_TEXTURE3D);
    pub const TEXTURE_CUBE: Self = Self(sys::CNA_CNB_ASSET_TYPE_TEXTURE_CUBE);
    pub const SPRITE_FONT: Self = Self(sys::CNA_CNB_ASSET_TYPE_SPRITE_FONT);
    pub const MODEL: Self = Self(sys::CNA_CNB_ASSET_TYPE_MODEL);
    pub const ANIMATION_CLIP: Self = Self(sys::CNA_CNB_ASSET_TYPE_ANIMATION_CLIP);
    pub const CURVE: Self = Self(sys::CNA_CNB_ASSET_TYPE_CURVE);
    pub const SOUND_EFFECT: Self = Self(sys::CNA_CNB_ASSET_TYPE_SOUND_EFFECT);
    pub const SONG: Self = Self(sys::CNA_CNB_ASSET_TYPE_SONG);
    pub const VIDEO: Self = Self(sys::CNA_CNB_ASSET_TYPE_VIDEO);
    pub const EFFECT: Self = Self(sys::CNA_CNB_ASSET_TYPE_EFFECT);

    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Wraps a numeric identity, including one this build predates.
    #[must_use]
    pub const fn from_value(value: u32) -> Self {
        Self(value)
    }

    /// Mints the custom identity for a game-defined asset type name.
    ///
    /// This is **not** the inverse of [`AssetTypeId::name`]: it hashes the name
    /// into the custom range rather than looking up a built-in identity, so
    /// `custom("Texture2D")` is a custom identity and not
    /// [`AssetTypeId::TEXTURE2D`]. Collisions are possible in principle, which
    /// is why a document can also carry the type name for a loader to check.
    pub fn custom(name: &str) -> Result<Self> {
        let native = Native::process()?;
        let view = string_view(name);
        let mut value = 0;
        // SAFETY: `view` borrows `name` for the duration of the call.
        native.check(unsafe {
            (native.runtime.cnb_asset_type_id_from_name)(view, &mut value)
        })?;
        Ok(Self(value))
    }

    /// Whether this identity is a game-defined one rather than a built-in.
    pub fn is_custom(self) -> Result<bool> {
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the output is a live local of the declared type.
        native.check(unsafe {
            (native.runtime.cnb_is_custom_asset_type_id)(self.0, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// CNA's canonical name for this identity.
    pub fn name(self) -> Result<String> {
        let native = Native::process()?;
        let api = &native.runtime;
        read_string(
            |value| native.check(value),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.cnb_asset_type_name_size)(self.0, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_copy_asset_type_name)(self.0, destination, capacity, written)
            },
        )
    }
}

/// Accepts the answer a sizing probe gives.
///
/// A copy route asked with zero capacity reports the required byte count and
/// returns `BUFFER_TOO_SMALL`, which is the answer rather than a failure. Only
/// a zero-length result comes back as success, so both have to be admitted.
fn accept_size_probe(native: &Arc<Native>, result: sys::CNA_Result) -> Result<()> {
    if result == sys::CNA_RESULT_BUFFER_TOO_SMALL {
        return Ok(());
    }
    native.check(result)
}

fn string_view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast::<core::ffi::c_char>(),
        byte_length: value.len() as u64,
    }
}

impl CnbDocument {
    /// Parses a whole `.cnb` file already in memory.
    ///
    /// `origin` names the source in diagnostics; a path is the usual choice.
    pub fn parse(bytes: &[u8], origin: &str, limits: ReadLimits) -> Result<Self> {
        let native = Native::process()?;
        let native_limits = limits.to_native();
        let limits_pointer = if limits.is_default() {
            core::ptr::null()
        } else {
            &native_limits
        };
        let view = string_view(origin);
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: `bytes` and `origin` are borrowed for the duration of the
        // call, the limits pointer is either null or a live local, and the
        // output is a live local.
        native.check(unsafe {
            (native.runtime.cnb_document_parse)(
                bytes.as_ptr(),
                bytes.len() as u64,
                view,
                limits_pointer,
                &mut handle,
            )
        })?;
        Ok(Self { native, handle })
    }

    /// Parses a `.cnb` file from the filesystem.
    pub fn parse_file(path: &Path, limits: ReadLimits) -> Result<Self> {
        let native = Native::process()?;
        let text = path.to_str().ok_or(CnaError::InvalidInput(
            "a .cnb path must be valid UTF-8 for the canonical route",
        ))?;
        let native_limits = limits.to_native();
        let limits_pointer = if limits.is_default() {
            core::ptr::null()
        } else {
            &native_limits
        };
        let view = string_view(text);
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: `text` is borrowed for the call and both pointers are live.
        native.check(unsafe {
            (native.runtime.cnb_document_parse_file)(view, limits_pointer, &mut handle)
        })?;
        Ok(Self { native, handle })
    }

    /// The name the document was parsed under, for diagnostics.
    pub fn origin(&self) -> Result<String> {
        let api = &self.native.runtime;
        let handle = self.handle;
        read_string(
            |value| self.native.check(value),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.cnb_document_origin_size)(handle, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_document_copy_origin)(handle, destination, capacity, written)
            },
        )
    }

    /// The container format version the file declares.
    pub fn container_version(&self) -> Result<(u16, u16)> {
        let mut major = 0;
        let mut minor = 0;
        // SAFETY: both outputs are live locals of the declared types.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_container_major)(self.handle, &mut major)
        })?;
        // SAFETY: as above.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_container_minor)(self.handle, &mut minor)
        })?;
        Ok((major, minor))
    }

    /// Which kind of asset the document holds.
    pub fn asset_type(&self) -> Result<AssetTypeId> {
        let mut value = 0;
        // SAFETY: the output is a live local of the declared type.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_asset_type_id)(self.handle, &mut value)
        })?;
        Ok(AssetTypeId(value))
    }

    /// The asset schema version the document was written against.
    pub fn asset_schema_version(&self) -> Result<u32> {
        let mut value = 0;
        // SAFETY: the output is a live local of the declared type.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_asset_schema_version)(self.handle, &mut value)
        })?;
        Ok(value)
    }

    /// How many chunks the document contains.
    pub fn chunk_count(&self) -> Result<u64> {
        let mut value = 0;
        // SAFETY: the output is a live local of the declared type.
        self.native.check(unsafe {
            (self.native.runtime.cnb_document_chunk_count)(self.handle, &mut value)
        })?;
        Ok(value)
    }

    /// The document's metadata block.
    ///
    /// `present` is false for a document that carries none; the names are then
    /// empty rather than invented.
    pub fn metadata(&self) -> Result<Metadata> {
        let api = &self.native.runtime;
        let handle = self.handle;
        let mut block = sys::CNA_CnbMetadata {
            struct_size: core::mem::size_of::<sys::CNA_CnbMetadata>() as u32,
            struct_version: sys::CNA_CNB_METADATA_STRUCT_VERSION,
            ..sys::CNA_CnbMetadata::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output whose
        // prefix this build declares exactly.
        self.native
            .check(unsafe { (api.cnb_document_metadata)(handle, &mut block) })?;
        let asset_type_name = read_string(
            |value| self.native.check(value),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.cnb_document_metadata_asset_type_name_size)(handle, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_document_copy_metadata_asset_type_name)(
                    handle,
                    destination,
                    capacity,
                    written,
                )
            },
        )?;
        let content_name = read_string(
            |value| self.native.check(value),
            // SAFETY: as above.
            |bytes| unsafe { (api.cnb_document_metadata_content_name_size)(handle, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_document_copy_metadata_content_name)(handle, destination, capacity, written)
            },
        )?;
        Ok(Metadata {
            present: block.present != sys::CNA_FALSE,
            flags: block.flags,
            asset_type_name,
            content_name,
        })
    }

    /// Decodes the document's `Texture2D` payload.
    pub fn decode_texture2d(&self) -> Result<CnbTextureData> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the document handle is owned and live; the output is a live
        // local that receives a newly owned texture handle.
        self.native.check(unsafe {
            (self.native.runtime.cnb_decode_texture2d)(self.handle, &mut handle)
        })?;
        Ok(CnbTextureData {
            native: Arc::clone(&self.native),
            handle,
        })
    }
}

impl Drop for CnbDocument {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        let _ = unsafe { (self.native.runtime.cnb_document_destroy)(self.handle) };
    }
}

/// The shape of one texture inside a document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextureInfo {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub face_count: u32,
    pub mip_count: u32,
    pub representation_count: u32,
}

impl CnbTextureData {
    /// Builds texture data from tightly packed RGBA8 pixels.
    pub fn from_rgba8(width: u32, height: u32, rgba: &[u8]) -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: `rgba` is borrowed for the duration of the call and the
        // output is a live local.
        native.check(unsafe {
            (native.runtime.cnb_texture_create_rgba8)(
                width,
                height,
                rgba.as_ptr(),
                rgba.len() as u64,
                &mut handle,
            )
        })?;
        Ok(Self { native, handle })
    }

    /// The texture's dimensions, faces, mip levels and representations.
    pub fn info(&self) -> Result<TextureInfo> {
        let mut info = sys::CNA_CnbTextureInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbTextureInfo>() as u32,
            struct_version: sys::CNA_CNB_TEXTURE_INFO_STRUCT_VERSION,
            ..sys::CNA_CnbTextureInfo::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output whose
        // prefix this build declares exactly.
        self.native
            .check(unsafe { (self.native.runtime.cnb_texture_info)(self.handle, &mut info) })?;
        Ok(TextureInfo {
            width: info.width,
            height: info.height,
            depth: info.depth,
            face_count: info.face_count,
            mip_count: info.mip_count,
            representation_count: info.representation_count,
        })
    }

    /// The dimensions of one mip level.
    pub fn level_dimensions(&self, level: u32) -> Result<(u32, u32, u32)> {
        let mut width = 0;
        let mut height = 0;
        let mut depth = 0;
        // SAFETY: all three outputs are live locals of the declared types.
        self.native.check(unsafe {
            (self.native.runtime.cnb_texture_level_dimensions)(
                self.handle,
                level,
                &mut width,
                &mut height,
                &mut depth,
            )
        })?;
        Ok((width, height, depth))
    }

    /// How many pixel representations the texture carries.
    ///
    /// A `.cnb` texture may hold the same image in several formats so a game
    /// can pick one its renderer supports.
    pub fn representation_count(&self) -> Result<u64> {
        let mut value = 0;
        // SAFETY: the output is a live local of the declared type.
        self.native.check(unsafe {
            (self.native.runtime.cnb_texture_representation_count)(self.handle, &mut value)
        })?;
        Ok(value)
    }

    /// The format of one representation.
    pub fn representation_format(&self, representation: u64) -> Result<CnbTextureFormat> {
        let mut value = 0;
        // SAFETY: the output is a live local of the declared type.
        self.native.check(unsafe {
            (self.native.runtime.cnb_texture_representation_format)(
                self.handle,
                representation,
                &mut value,
            )
        })?;
        Ok(CnbTextureFormat(value))
    }

    /// How many mip levels one representation carries.
    pub fn level_count(&self, representation: u64) -> Result<u64> {
        let mut value = 0;
        // SAFETY: the output is a live local of the declared type.
        self.native.check(unsafe {
            (self.native.runtime.cnb_texture_level_count)(self.handle, representation, &mut value)
        })?;
        Ok(value)
    }

    /// Copies one mip level's bytes out of the texture.
    pub fn level_bytes(&self, representation: u64, level: u64) -> Result<Vec<u8>> {
        let api = &self.native.runtime;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity is the canonical way
        // to ask for the required size; `required` is a live local.
        accept_size_probe(&self.native, unsafe {
            (api.cnb_texture_copy_level)(
                self.handle,
                representation,
                level,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        })?;
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("texture level is too large"))?;
        let mut bytes = vec![0_u8; capacity];
        if capacity == 0 {
            return Ok(bytes);
        }
        let mut copied = 0_u64;
        // SAFETY: `bytes` holds exactly `required` writable bytes for the call.
        self.native.check(unsafe {
            (api.cnb_texture_copy_level)(
                self.handle,
                representation,
                level,
                bytes.as_mut_ptr(),
                required,
                &mut copied,
            )
        })?;
        let copied = usize::try_from(copied)
            .map_err(|_| CnaError::InvalidInput("texture level is too large"))?;
        bytes.truncate(copied.min(capacity));
        Ok(bytes)
    }

    /// Encodes the texture as a complete `.cnb` `Texture2D` document.
    pub fn encode_texture2d(&self, content_name: &str) -> Result<Vec<u8>> {
        let api = &self.native.runtime;
        let view = string_view(content_name);
        let mut required = 0_u64;
        // SAFETY: `view` borrows `content_name` for the call; a null
        // destination with zero capacity asks for the required size.
        accept_size_probe(&self.native, unsafe {
            (api.cnb_encode_texture2d)(
                self.handle,
                view,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        })?;
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("encoded document is too large"))?;
        let mut bytes = vec![0_u8; capacity];
        let mut written = 0_u64;
        // SAFETY: `bytes` holds exactly `required` writable bytes for the call.
        self.native.check(unsafe {
            (api.cnb_encode_texture2d)(
                self.handle,
                view,
                bytes.as_mut_ptr(),
                required,
                &mut written,
            )
        })?;
        let written = usize::try_from(written)
            .map_err(|_| CnaError::InvalidInput("encoded document is too large"))?;
        bytes.truncate(written.min(capacity));
        Ok(bytes)
    }
}

impl Drop for CnbTextureData {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        let _ = unsafe { (self.native.runtime.cnb_texture_destroy)(self.handle) };
    }
}
