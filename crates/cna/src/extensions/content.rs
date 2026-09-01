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

use core::ffi::c_void;
use std::any::Any;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::game::GameContext;
use crate::graphics::{GraphicsDevice, SurfaceFormat, Texture2D, TextureCube};
use crate::value::{Rectangle, Vector3};
use crate::native::runtime::read_string;
use crate::native::Native;

/// Sanity bounds applied while parsing a document.
///
/// A `.cnb` file is untrusted input, so the parser is bounded rather than
/// trusting the file's own counts. `None` anywhere means "use CNA's default
/// for this bound", which is what passing no limits at all does.
///
/// Zero is a real bound to CNA, not an absent one: the canonical contract is
/// "initialize with `cna_cnb_read_limits_init`, then lower whatever a caller
/// wants tighter". So a `None` field is filled from CNA's own defaults rather
/// than sent as zero, which would otherwise mean that tightening one bound
/// silently set every other bound to zero and refused every document.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReadLimits {
    pub max_file_size: Option<u64>,
    pub max_chunk_size: Option<u64>,
    pub max_total_uncompressed_size: Option<u64>,
    pub max_chunk_count: Option<u32>,
    pub max_string_bytes: Option<u32>,
    pub max_array_element_count: Option<u32>,
    /// Largest chunk alignment a table-of-contents entry may declare.
    pub max_chunk_alignment: Option<u32>,
}

impl ReadLimits {
    /// CNA's process-wide default bounds, with this value's overrides applied.
    pub(crate) fn to_native(self, native: &Arc<Native>) -> Result<sys::CNA_CnbReadLimits> {
        let mut limits = sys::CNA_CnbReadLimits {
            struct_size: core::mem::size_of::<sys::CNA_CnbReadLimits>() as u32,
            struct_version: sys::CNA_CNB_READ_LIMITS_STRUCT_VERSION,
            ..sys::CNA_CnbReadLimits::default()
        };
        // SAFETY: the structure is a caller-owned versioned output whose
        // size and version this build sets before the call.
        native.check(unsafe { (native.runtime.cnb_read_limits_init)(&mut limits) })?;
        if let Some(value) = self.max_file_size {
            limits.max_file_size = value;
        }
        if let Some(value) = self.max_chunk_size {
            limits.max_chunk_size = value;
        }
        if let Some(value) = self.max_total_uncompressed_size {
            limits.max_total_uncompressed_size = value;
        }
        if let Some(value) = self.max_chunk_count {
            limits.max_chunk_count = value;
        }
        if let Some(value) = self.max_string_bytes {
            limits.max_string_bytes = value;
        }
        if let Some(value) = self.max_array_element_count {
            limits.max_array_element_count = value;
        }
        if let Some(value) = self.max_chunk_alignment {
            limits.max_chunk_alignment = value;
        }
        Ok(limits)
    }

    pub(crate) const fn is_default(self) -> bool {
        self.max_file_size.is_none()
            && self.max_chunk_size.is_none()
            && self.max_total_uncompressed_size.is_none()
            && self.max_chunk_count.is_none()
            && self.max_string_bytes.is_none()
            && self.max_array_element_count.is_none()
            && self.max_chunk_alignment.is_none()
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

/// How a `CnbDocument` relates to CNA's native document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DocumentOwner {
    /// This value parsed the document and destroys it.
    Owned,
    /// CNA lent the handle for the duration of one loader callback. It is
    /// invalidated before that callback returns and has no destroy operation,
    /// so this value must not release it.
    CallbackScoped,
}

/// One parsed `.cnb` document.
#[derive(Debug)]
pub struct CnbDocument {
    pub(crate) native: Arc<Native>,
    pub(crate) handle: sys::CNA_CnbDocumentHandle,
    owner: DocumentOwner,
}

/// Texture pixels decoded from, or destined for, a `.cnb` document.
#[derive(Debug)]
pub struct CnbTextureData {
    pub(crate) native: Arc<Native>,
    pub(crate) handle: sys::CNA_CnbTextureDataHandle,
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

    /// Wraps a raw identifier read out of a document.
    ///
    /// Not validated here: `CnbTextureFormat::is_known` is the question, and a
    /// caller reading a file wants to ask it explicitly rather than have a
    /// constructor decide.
    #[must_use]
    pub const fn from_value(value: u32) -> Self {
        Self(value)
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
pub(crate) fn accept_size_probe(native: &Arc<Native>, result: sys::CNA_Result) -> Result<()> {
    if result == sys::CNA_RESULT_BUFFER_TOO_SMALL {
        return Ok(());
    }
    native.check(result)
}

pub(crate) fn string_view(value: &str) -> sys::CNA_StringView {
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
        let native_limits = limits.to_native(&native)?;
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
        Ok(Self {
            native,
            handle,
            owner: DocumentOwner::Owned,
        })
    }

    /// Parses a `.cnb` file from the filesystem.
    pub fn parse_file(path: &Path, limits: ReadLimits) -> Result<Self> {
        let native = Native::process()?;
        let text = path.to_str().ok_or(CnaError::InvalidInput(
            "a .cnb path must be valid UTF-8 for the canonical route",
        ))?;
        let native_limits = limits.to_native(&native)?;
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
        Ok(Self {
            native,
            handle,
            owner: DocumentOwner::Owned,
        })
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
    /// Decodes the document's compiled model.
    ///
    /// The document must actually carry one: asking a texture document for a
    /// model is CNA's refusal, not a reinterpretation of its bytes.
    pub fn decode_model(&self) -> Result<CnbModel> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the document handle is owned and live; the output is a live
        // local that receives a newly owned model handle.
        self.native
            .check(unsafe { (self.native.runtime.cnb_decode_model)(self.handle, &mut handle) })?;
        Ok(CnbModel {
            native: Arc::clone(&self.native),
            handle,
        })
    }

    /// Decodes the document's compiled sprite font.
    pub fn decode_sprite_font(&self) -> Result<CnbSpriteFont> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the document handle is owned and live; the output is a live
        // local that receives a newly owned font handle.
        self.native.check(unsafe {
            (self.native.runtime.cnb_decode_sprite_font)(self.handle, &mut handle)
        })?;
        Ok(CnbSpriteFont {
            native: Arc::clone(&self.native),
            handle,
        })
    }

    /// Decodes the document's compiled sound effect.
    pub fn decode_sound_effect(&self) -> Result<CnbSoundEffect> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: as above, receiving a newly owned sound handle.
        self.native.check(unsafe {
            (self.native.runtime.cnb_decode_sound_effect)(self.handle, &mut handle)
        })?;
        Ok(CnbSoundEffect {
            native: Arc::clone(&self.native),
            handle,
        })
    }

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
        if self.owner == DocumentOwner::CallbackScoped {
            // CNA owns this one and invalidates it when the callback returns.
            return;
        }
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
        Ok(Self {
            native,
            handle,
        })
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

/// Which of a compiled effect's shading models a part draws with.
///
/// CNA's own identity, not XNA's: `Pbr`, `SkinnedPbr` and `External` have no
/// `Microsoft.Xna.Framework.Graphics` counterpart, which is exactly why the
/// compiled model lives here and is not projected onto XNA's `Model`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CnbEffectKind {
    Basic,
    Skinned,
    DualTexture,
    Pbr,
    SkinnedPbr,
    External,
}

impl CnbEffectKind {
    pub(crate) const fn from_native(value: sys::CNA_CnbEffectKind) -> Option<Self> {
        Some(match value {
            sys::CNA_CNB_EFFECT_KIND_BASIC => Self::Basic,
            sys::CNA_CNB_EFFECT_KIND_SKINNED => Self::Skinned,
            sys::CNA_CNB_EFFECT_KIND_DUAL_TEXTURE => Self::DualTexture,
            sys::CNA_CNB_EFFECT_KIND_PBR => Self::Pbr,
            sys::CNA_CNB_EFFECT_KIND_SKINNED_PBR => Self::SkinnedPbr,
            sys::CNA_CNB_EFFECT_KIND_EXTERNAL => Self::External,
            _ => return None,
        })
    }

    pub(crate) const fn to_native(self) -> sys::CNA_CnbEffectKind {
        match self {
            Self::Basic => sys::CNA_CNB_EFFECT_KIND_BASIC,
            Self::Skinned => sys::CNA_CNB_EFFECT_KIND_SKINNED,
            Self::DualTexture => sys::CNA_CNB_EFFECT_KIND_DUAL_TEXTURE,
            Self::Pbr => sys::CNA_CNB_EFFECT_KIND_PBR,
            Self::SkinnedPbr => sys::CNA_CNB_EFFECT_KIND_SKINNED_PBR,
            Self::External => sys::CNA_CNB_EFFECT_KIND_EXTERNAL,
        }
    }
}

/// Which of a material's eight texture **names** an operation addresses.
///
/// Upstream warns that these are not the same eight slots as the per-slot
/// state arrays, which are seven and in the importer's order. This type names
/// only the eight the copy/set routes take, so the two index spaces cannot be
/// confused by passing a bare integer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CnbMaterialTexture {
    BaseColor,
    /// `DualTextureEffect`'s second layer, which glTF has no counterpart for.
    Second,
    Normal,
    MetallicRoughness,
    Emissive,
    Occlusion,
    Specular,
    SpecularColor,
}

impl CnbMaterialTexture {
    pub(crate) const fn to_native(self) -> sys::CNA_CnbMaterialTextureSlot {
        match self {
            Self::BaseColor => sys::CNA_CNB_MATERIAL_TEXTURE_BASE_COLOR,
            Self::Second => sys::CNA_CNB_MATERIAL_TEXTURE_SECOND,
            Self::Normal => sys::CNA_CNB_MATERIAL_TEXTURE_NORMAL,
            Self::MetallicRoughness => sys::CNA_CNB_MATERIAL_TEXTURE_METALLIC_ROUGHNESS,
            Self::Emissive => sys::CNA_CNB_MATERIAL_TEXTURE_EMISSIVE,
            Self::Occlusion => sys::CNA_CNB_MATERIAL_TEXTURE_OCCLUSION,
            Self::Specular => sys::CNA_CNB_MATERIAL_TEXTURE_SPECULAR,
            Self::SpecularColor => sys::CNA_CNB_MATERIAL_TEXTURE_SPECULAR_COLOR,
        }
    }
}

/// A compiled model's counts and the two policy facts that travel with it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CnbModelInfo {
    pub bone_count: u64,
    pub part_count: u64,
    pub mesh_count: u64,
    pub animation_count: u64,
    pub light_count: u64,
    pub has_skeleton: bool,
    /// Whether the materials were authored under glTF's lighting conventions.
    ///
    /// Stated by the content rather than guessed from whether lights exist,
    /// because a glTF model expects the importer's default-light fallback and
    /// a hand-authored one expects XNA's unlit `BasicEffect` start.
    pub applies_gltf_lighting_policy: bool,
    /// Whether the source carried a real scene-node hierarchy.
    pub has_bone_hierarchy: bool,
}

/// One node of the compiled scene graph.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CnbModelBone {
    /// The parent's index, or `None` for the root.
    pub parent: Option<u64>,
    /// The bone-local transform in XNA's row-major `M11`..`M44` order.
    pub transform: [f32; 16],
}

/// One renderable part's numeric state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CnbModelPart {
    pub vertex_stride: u32,
    pub vertex_count: u32,
    pub index_count: u32,
    /// Bytes per index: 2 or 4. Stored rather than derived, so a truncated
    /// sidecar cannot silently decode as a shorter mesh.
    pub index_element_size: u32,
    pub primitive_topology: u32,
    pub primitive_count: u32,
    pub effect_kind: CnbEffectKind,
    pub vertex_color_enabled: bool,
    /// Whether the material is `KHR_materials_unlit`.
    pub unlit: bool,
}

/// A material's numeric state, without its texture names.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CnbMaterial {
    pub base_color_factor: [f32; 4],
    pub emissive_factor: [f32; 3],
    pub specular_color_factor: [f32; 3],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub ior: f32,
    pub specular_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub alpha_cutoff: f32,
    pub alpha_mode: u32,
    pub double_sided: bool,
}

impl Default for CnbMaterial {
    /// glTF's own defaults, which are what an unset material means.
    fn default() -> Self {
        Self {
            base_color_factor: [1.0; 4],
            emissive_factor: [0.0; 3],
            specular_color_factor: [1.0; 3],
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            ior: 1.5,
            specular_factor: 1.0,
            normal_scale: 1.0,
            occlusion_strength: 1.0,
            alpha_cutoff: 0.5,
            alpha_mode: 0,
            double_sided: false,
        }
    }
}

/// One mesh: a name, an optional parent bone, and its parts in draw order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CnbMeshInfo {
    /// The bone this mesh hangs from, or `None` when the model has no
    /// hierarchy.
    pub parent_bone: Option<u64>,
    pub part_index_count: u64,
}

/// A compiled `.cnb` model: bones, meshes, parts, materials and geometry.
///
/// This is CNA's compiled model, not XNA's `Model`, and the difference is
/// deliberate. The container carries PBR factors, `KHR_materials_*` extension
/// state, morph targets, punctual lights and a glTF lighting policy, none of
/// which `Microsoft.Xna.Framework.Graphics.Model` ever exposed. Projecting
/// them onto XNA's object model would mean inventing members Microsoft never
/// declared, so they live here instead.
#[derive(Debug)]
pub struct CnbModel {
    pub(crate) native: Arc<Native>,
    pub(crate) handle: sys::CNA_CnbModelDataHandle,
}

impl CnbModel {
    /// Starts an empty model to author.
    pub fn new() -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local receiving a newly owned handle.
        native.check(unsafe { (native.runtime.cnb_model_create)(&mut handle) })?;
        Ok(Self {
            native,
            handle,
        })
    }

    /// Sets the two policy facts that travel with the content.
    ///
    /// Neither is inferred from the model's shape, and upstream is explicit
    /// about why: a glTF-imported model expects the importer's default-light
    /// fallback while a hand-authored one expects XNA's unlit `BasicEffect`
    /// start, and "attach meshes to their named bone" versus "give every mesh
    /// its own child of the root" is a real semantic fork rather than a
    /// consequence of the bone count.
    pub fn set_flags(
        &self,
        applies_gltf_lighting_policy: bool,
        has_bone_hierarchy: bool,
    ) -> Result<()> {
        // SAFETY: both arguments are by value and the handle is owned.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_set_flags)(
                self.handle,
                u8::from(applies_gltf_lighting_policy),
                u8::from(has_bone_hierarchy),
            )
        })
    }

    /// Appends a scene-graph node and answers its index.
    ///
    /// `parent` is `None` for the root; the transform is XNA's row-major
    /// `M11`..`M44`.
    pub fn add_bone(&self, name: &str, parent: Option<u64>, transform: &[f32; 16]) -> Result<u64> {
        let parent = encode_parent(parent)?;
        let view = string_view(name);
        let mut index = 0_u64;
        // SAFETY: `name` and `transform` are borrowed for the duration of the
        // call and CNA copies both; the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_add_bone)(
                self.handle,
                view,
                parent,
                transform.as_ptr(),
                &mut index,
            )
        })?;
        Ok(index)
    }

    /// Appends a mesh naming the parts it draws, in draw order.
    pub fn add_mesh(&self, name: &str, parent_bone: Option<u64>, parts: &[u32]) -> Result<u64> {
        let parent = encode_parent(parent_bone)?;
        let view = string_view(name);
        let mut index = 0_u64;
        // SAFETY: `name` and `parts` are borrowed for the duration of the call
        // and the length is the slice's own; the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_add_mesh)(
                self.handle,
                view,
                parent,
                if parts.is_empty() {
                    core::ptr::null()
                } else {
                    parts.as_ptr()
                },
                parts.len() as u64,
                &mut index,
            )
        })?;
        Ok(index)
    }

    /// Appends a renderable part and answers its index.
    ///
    /// `external_effect` is used only by [`CnbEffectKind::External`].
    pub fn add_part(&self, part: CnbModelPart, name: &str, external_effect: &str) -> Result<u64> {
        let info = part.to_native();
        let name_view = string_view(name);
        let effect_view = string_view(external_effect);
        let mut index = 0_u64;
        // SAFETY: the descriptor and both string views are live for the call
        // and CNA copies what it keeps; the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_add_part)(
                self.handle,
                &info,
                name_view,
                effect_view,
                &mut index,
            )
        })?;
        Ok(index)
    }

    /// Stores a part's raw vertex bytes.
    ///
    /// The bytes are opaque here: their meaning is the part's stride and the
    /// vertex declaration the consumer chooses, and CNA validates only that
    /// stride times count matches this length.
    pub fn set_part_vertex_bytes(&self, part: u64, bytes: &[u8]) -> Result<()> {
        // SAFETY: `bytes` is borrowed for the duration of the call with its
        // own length.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_set_part_vertex_bytes)(
                self.handle,
                part,
                bytes.as_ptr(),
                bytes.len() as u64,
            )
        })
    }

    /// Stores a part's raw index bytes.
    pub fn set_part_index_bytes(&self, part: u64, bytes: &[u8]) -> Result<()> {
        // SAFETY: `bytes` is borrowed for the duration of the call with its
        // own length.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_set_part_index_bytes)(
                self.handle,
                part,
                bytes.as_ptr(),
                bytes.len() as u64,
            )
        })
    }

    /// Sets a part's material.
    pub fn set_material(&self, part: u64, material: CnbMaterial) -> Result<()> {
        let info = material.to_native();
        // SAFETY: the descriptor is a live local CNA copies during the call.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_set_material)(self.handle, part, &info)
        })
    }

    /// Names the asset a material's texture slot refers to.
    pub fn set_material_texture(
        &self,
        part: u64,
        slot: CnbMaterialTexture,
        asset_name: &str,
    ) -> Result<()> {
        let view = string_view(asset_name);
        // SAFETY: `asset_name` is borrowed for the duration of the call and
        // CNA copies it.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_set_material_texture)(
                self.handle,
                part,
                slot.to_native(),
                view,
            )
        })
    }

    /// Encodes the model as a complete `.cnb` document.
    pub fn encode(&self, content_name: &str) -> Result<Vec<u8>> {
        let api = &self.native.runtime;
        let view = string_view(content_name);
        let mut required = 0_u64;
        // SAFETY: `view` borrows `content_name` for the call; a null
        // destination with zero capacity asks for the required size.
        accept_size_probe(&self.native, unsafe {
            (api.cnb_encode_model)(self.handle, view, core::ptr::null_mut(), 0, &mut required)
        })?;
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("encoded document is too large"))?;
        let mut bytes = vec![0_u8; capacity];
        let mut written = 0_u64;
        // SAFETY: `bytes` holds exactly `required` writable bytes for the call.
        self.native.check(unsafe {
            (api.cnb_encode_model)(self.handle, view, bytes.as_mut_ptr(), required, &mut written)
        })?;
        let written = usize::try_from(written)
            .map_err(|_| CnaError::InvalidInput("encoded document is too large"))?;
        bytes.truncate(written.min(capacity));
        Ok(bytes)
    }

    /// The model's counts and policy facts.
    pub fn info(&self) -> Result<CnbModelInfo> {
        let mut info = sys::CNA_CnbModelInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbModelInfo>() as u32,
            struct_version: sys::CNA_CNB_MODEL_INFO_STRUCT_VERSION,
            ..sys::CNA_CnbModelInfo::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output whose
        // prefix this build declares exactly.
        self.native
            .check(unsafe { (self.native.runtime.cnb_model_info)(self.handle, &mut info) })?;
        Ok(CnbModelInfo {
            bone_count: info.bone_count,
            part_count: info.part_count,
            mesh_count: info.mesh_count,
            animation_count: info.animation_count,
            light_count: info.light_count,
            has_skeleton: info.has_skeleton != sys::CNA_FALSE,
            applies_gltf_lighting_policy: info.applies_gltf_lighting_policy != sys::CNA_FALSE,
            has_bone_hierarchy: info.has_bone_hierarchy != sys::CNA_FALSE,
        })
    }

    /// One bone's parent and local transform.
    pub fn bone(&self, index: u64) -> Result<CnbModelBone> {
        let mut bone = sys::CNA_CnbModelBone {
            struct_size: core::mem::size_of::<sys::CNA_CnbModelBone>() as u32,
            struct_version: sys::CNA_CNB_MODEL_BONE_STRUCT_VERSION,
            ..sys::CNA_CnbModelBone::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output.
        self.native
            .check(unsafe { (self.native.runtime.cnb_model_bone)(self.handle, index, &mut bone) })?;
        Ok(CnbModelBone {
            parent: decode_parent(bone.parent),
            transform: bone.transform,
        })
    }

    /// One bone's name.
    pub fn bone_name(&self, index: u64) -> Result<String> {
        let native = &self.native;
        let api = &native.runtime;
        read_string(
            |value| native.check(value),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.cnb_model_bone_name_size)(self.handle, index, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_model_copy_bone_name)(self.handle, index, destination, capacity, written)
            },
        )
    }

    /// One mesh's parent bone and part count.
    pub fn mesh(&self, index: u64) -> Result<CnbMeshInfo> {
        let mut info = sys::CNA_CnbMeshInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbMeshInfo>() as u32,
            struct_version: sys::CNA_CNB_MESH_INFO_STRUCT_VERSION,
            ..sys::CNA_CnbMeshInfo::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output.
        self.native
            .check(unsafe { (self.native.runtime.cnb_model_mesh)(self.handle, index, &mut info) })?;
        Ok(CnbMeshInfo {
            parent_bone: decode_parent(info.parent_bone),
            part_index_count: info.part_index_count,
        })
    }

    /// One mesh's name.
    pub fn mesh_name(&self, index: u64) -> Result<String> {
        let native = &self.native;
        let api = &native.runtime;
        read_string(
            |value| native.check(value),
            // SAFETY: as for `bone_name`.
            |bytes| unsafe { (api.cnb_model_mesh_name_size)(self.handle, index, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_model_copy_mesh_name)(self.handle, index, destination, capacity, written)
            },
        )
    }

    /// The parts one mesh draws, in draw order.
    pub fn mesh_part_indices(&self, index: u64) -> Result<Vec<u32>> {
        let count = self.mesh(index)?.part_index_count;
        let capacity = usize::try_from(count)
            .map_err(|_| CnaError::InvalidInput("mesh part list is too large"))?;
        let mut parts = vec![0_u32; capacity];
        let mut written = 0_u64;
        // SAFETY: `parts` holds exactly `count` writable elements for the call.
        self.native.check(unsafe {
            (self.native.runtime.cnb_model_copy_mesh_part_indices)(
                self.handle,
                index,
                if capacity == 0 {
                    core::ptr::null_mut()
                } else {
                    parts.as_mut_ptr()
                },
                count,
                &mut written,
            )
        })?;
        let written = usize::try_from(written)
            .map_err(|_| CnaError::InvalidInput("mesh part list is too large"))?;
        parts.truncate(written.min(capacity));
        Ok(parts)
    }

    /// One part's numeric state.
    pub fn part(&self, index: u64) -> Result<CnbModelPart> {
        let mut info = sys::CNA_CnbModelPartInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbModelPartInfo>() as u32,
            struct_version: sys::CNA_CNB_MODEL_PART_INFO_STRUCT_VERSION,
            ..sys::CNA_CnbModelPartInfo::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output.
        self.native
            .check(unsafe { (self.native.runtime.cnb_model_part)(self.handle, index, &mut info) })?;
        let effect_kind =
            CnbEffectKind::from_native(info.effect_kind).ok_or(CnaError::UnsupportedRuntime(
                "this .cnb model part names an effect kind this build does not know",
            ))?;
        Ok(CnbModelPart {
            vertex_stride: info.vertex_stride,
            vertex_count: info.vertex_count,
            index_count: info.index_count,
            index_element_size: info.index_element_size,
            primitive_topology: info.primitive_topology,
            primitive_count: info.primitive_count,
            effect_kind,
            vertex_color_enabled: info.vertex_color_enabled != sys::CNA_FALSE,
            unlit: info.unlit != sys::CNA_FALSE,
        })
    }

    /// One part's name.
    pub fn part_name(&self, index: u64) -> Result<String> {
        let native = &self.native;
        let api = &native.runtime;
        read_string(
            |value| native.check(value),
            // SAFETY: as for `bone_name`.
            |bytes| unsafe { (api.cnb_model_part_name_size)(self.handle, index, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_model_copy_part_name)(self.handle, index, destination, capacity, written)
            },
        )
    }

    /// One part's raw vertex bytes.
    pub fn part_vertex_bytes(&self, index: u64) -> Result<Vec<u8>> {
        let part = self.part(index)?;
        let expected = u64::from(part.vertex_stride) * u64::from(part.vertex_count);
        self.copy_part_bytes(index, expected, true)
    }

    /// One part's raw index bytes.
    pub fn part_index_bytes(&self, index: u64) -> Result<Vec<u8>> {
        let part = self.part(index)?;
        let expected = u64::from(part.index_element_size) * u64::from(part.index_count);
        self.copy_part_bytes(index, expected, false)
    }

    fn copy_part_bytes(&self, index: u64, expected: u64, vertices: bool) -> Result<Vec<u8>> {
        let capacity = usize::try_from(expected)
            .map_err(|_| CnaError::InvalidInput("part byte block is too large"))?;
        let mut bytes = vec![0_u8; capacity];
        let mut written = 0_u64;
        let destination = if capacity == 0 {
            core::ptr::null_mut()
        } else {
            bytes.as_mut_ptr()
        };
        let api = &self.native.runtime;
        // SAFETY: `bytes` holds exactly `expected` writable bytes for the call,
        // and the length comes from the part's own declared stride and count.
        self.native.check(unsafe {
            if vertices {
                (api.cnb_model_copy_part_vertex_bytes)(
                    self.handle,
                    index,
                    destination,
                    expected,
                    &mut written,
                )
            } else {
                (api.cnb_model_copy_part_index_bytes)(
                    self.handle,
                    index,
                    destination,
                    expected,
                    &mut written,
                )
            }
        })?;
        let written = usize::try_from(written)
            .map_err(|_| CnaError::InvalidInput("part byte block is too large"))?;
        bytes.truncate(written.min(capacity));
        Ok(bytes)
    }

    /// One part's material.
    pub fn material(&self, part: u64) -> Result<CnbMaterial> {
        let mut info = sys::CNA_CnbMaterialInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbMaterialInfo>() as u32,
            struct_version: sys::CNA_CNB_MATERIAL_INFO_STRUCT_VERSION,
            ..sys::CNA_CnbMaterialInfo::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output.
        self.native
            .check(unsafe { (self.native.runtime.cnb_model_material)(self.handle, part, &mut info) })?;
        Ok(CnbMaterial {
            base_color_factor: info.base_color_factor,
            emissive_factor: info.emissive_factor,
            specular_color_factor: info.specular_color_factor,
            metallic_factor: info.metallic_factor,
            roughness_factor: info.roughness_factor,
            ior: info.ior,
            specular_factor: info.specular_factor,
            normal_scale: info.normal_scale,
            occlusion_strength: info.occlusion_strength,
            alpha_cutoff: info.alpha_cutoff,
            alpha_mode: info.alpha_mode,
            double_sided: info.double_sided != sys::CNA_FALSE,
        })
    }

    /// The asset a material's texture slot names, empty when it names none.
    pub fn material_texture(&self, part: u64, slot: CnbMaterialTexture) -> Result<String> {
        let native = &self.native;
        let api = &native.runtime;
        let slot = slot.to_native();
        read_string(
            |value| native.check(value),
            // SAFETY: as for `bone_name`.
            |bytes| unsafe { (api.cnb_model_material_texture_size)(self.handle, part, slot, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_model_copy_material_texture)(
                    self.handle,
                    part,
                    slot,
                    destination,
                    capacity,
                    written,
                )
            },
        )
    }
}

impl Drop for CnbModel {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        let _ = unsafe { (self.native.runtime.cnb_model_destroy)(self.handle) };
    }
}

impl CnbModelPart {
    pub(crate) fn to_native(self) -> sys::CNA_CnbModelPartInfo {
        sys::CNA_CnbModelPartInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbModelPartInfo>() as u32,
            struct_version: sys::CNA_CNB_MODEL_PART_INFO_STRUCT_VERSION,
            vertex_stride: self.vertex_stride,
            vertex_count: self.vertex_count,
            index_count: self.index_count,
            index_element_size: self.index_element_size,
            primitive_topology: self.primitive_topology,
            primitive_count: self.primitive_count,
            effect_kind: self.effect_kind.to_native(),
            vertex_color_enabled: u8::from(self.vertex_color_enabled),
            unlit: u8::from(self.unlit),
            reserved: [0; 2],
        }
    }
}

impl CnbMaterial {
    fn to_native(self) -> sys::CNA_CnbMaterialInfo {
        sys::CNA_CnbMaterialInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbMaterialInfo>() as u32,
            struct_version: sys::CNA_CNB_MATERIAL_INFO_STRUCT_VERSION,
            base_color_factor: self.base_color_factor,
            emissive_factor: self.emissive_factor,
            specular_color_factor: self.specular_color_factor,
            metallic_factor: self.metallic_factor,
            roughness_factor: self.roughness_factor,
            ior: self.ior,
            specular_factor: self.specular_factor,
            normal_scale: self.normal_scale,
            occlusion_strength: self.occlusion_strength,
            alpha_cutoff: self.alpha_cutoff,
            alpha_mode: self.alpha_mode,
            double_sided: u8::from(self.double_sided),
            reserved: [0; 3],
        }
    }
}

/// Encodes an optional parent index as the format's signed sentinel.
fn encode_parent(value: Option<u64>) -> Result<i32> {
    match value {
        None => Ok(-1),
        Some(index) => i32::try_from(index)
            .map_err(|_| CnaError::InvalidInput("a .cnb parent index must fit in an i32")),
    }
}

/// Decodes the format's signed sentinel back to an optional index.
const fn decode_parent(value: i32) -> Option<u64> {
    if value < 0 {
        None
    } else {
        Some(value as u64)
    }
}

/// Builds one `.cnb` document.
///
/// This is how a game authors an asset of its **own** type: the built-in
/// `encode` routes cover CNA's types, and this covers everything else. The
/// asset type identifier must be the custom one
/// [`AssetTypeId::custom`] mints for the type's canonical name, and the
/// metadata must carry that same name -- CNA compares it against the
/// registered name before dispatching a loader.
#[derive(Debug)]
pub struct CnbWriter {
    pub(crate) native: Arc<Native>,
    pub(crate) handle: sys::CNA_CnbWriterHandle,
}

impl CnbWriter {
    /// Starts a document for one asset type and schema version.
    pub fn new(asset_type: AssetTypeId, asset_schema_version: u32) -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local receiving a newly owned handle.
        native.check(unsafe {
            (native.runtime.cnb_writer_create)(asset_type.value(), asset_schema_version, &mut handle)
        })?;
        Ok(Self { native, handle })
    }

    /// Records the document's canonical type name and content name.
    ///
    /// The type name is load-bearing for a custom type: CNA refuses to dispatch
    /// a custom-typed file that carries no name, and refuses one whose name
    /// disagrees with the registered loader's, because a 31-bit identifier can
    /// collide and decoding the wrong game's content is worse than failing.
    pub fn set_metadata(&self, asset_type_name: &str, content_name: &str) -> Result<()> {
        let type_view = string_view(asset_type_name);
        let content_view = string_view(content_name);
        // SAFETY: both names are borrowed for the call and CNA copies them.
        self.native.check(unsafe {
            (self.native.runtime.cnb_writer_set_metadata)(self.handle, type_view, content_view)
        })
    }

    /// Appends one chunk of the asset's own payload.
    pub fn add_chunk(&self, chunk: u32, data: &[u8], flags: u32, alignment: u32) -> Result<()> {
        // SAFETY: `data` is borrowed for the call with its own length.
        self.native.check(unsafe {
            (self.native.runtime.cnb_writer_add_chunk)(
                self.handle,
                chunk,
                if data.is_empty() {
                    core::ptr::null()
                } else {
                    data.as_ptr()
                },
                data.len() as u64,
                flags,
                alignment,
            )
        })
    }

    /// Produces the complete document bytes.
    pub fn build(&self) -> Result<Vec<u8>> {
        let api = &self.native.runtime;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the size.
        accept_size_probe(&self.native, unsafe {
            (api.cnb_writer_build)(self.handle, core::ptr::null_mut(), 0, &mut required)
        })?;
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("encoded document is too large"))?;
        let mut bytes = vec![0_u8; capacity];
        let mut written = 0_u64;
        // SAFETY: `bytes` holds exactly `required` writable bytes for the call.
        self.native.check(unsafe {
            (api.cnb_writer_build)(self.handle, bytes.as_mut_ptr(), required, &mut written)
        })?;
        let written = usize::try_from(written)
            .map_err(|_| CnaError::InvalidInput("encoded document is too large"))?;
        bytes.truncate(written.min(capacity));
        Ok(bytes)
    }
}

impl Drop for CnbWriter {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        let _ = unsafe { (self.native.runtime.cnb_writer_destroy)(self.handle) };
    }
}

/// A native CNA content manager.
///
/// This is not XNA's `ContentManager`, which this crate implements in Rust and
/// which has no native handle. It exists because CNA's loader registry takes
/// one by reference: `cna_cnb_loader_invoke` requires a manager so a loader can
/// resolve a document's external references through the normal cache, and
/// upstream refuses to manufacture a placeholder because doing so would install
/// the built-in loaders as a side effect nobody asked for.
#[derive(Debug)]
pub struct NativeContentManager {
    native: Arc<Native>,
    handle: sys::CNA_Handle,
}

impl NativeContentManager {
    /// Creates a content manager on a graphics device.
    ///
    /// An independently constructed [`GraphicsDevice`] works here, which is
    /// what makes the whole loader chain reachable without a running `Game`.
    pub fn new(graphics_device: &GraphicsDevice, root_directory: &str) -> Result<Self> {
        let native = Native::process()?;
        let info = sys::CNA_ContentManagerCreateInfo {
            struct_size: core::mem::size_of::<sys::CNA_ContentManagerCreateInfo>() as u32,
            struct_version: 1,
            root_directory: string_view(root_directory),
            reserved: 0,
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the descriptor borrows `root_directory` for the call and CNA
        // copies it; the output is a live local.
        native.check(unsafe {
            (native.runtime.content_manager_create)(
                graphics_device.handle()?,
                &info,
                &mut handle,
            )
        })?;
        Ok(Self { native, handle })
    }
}

impl NativeContentManager {
    /// The manager handle, for routes projected in another module.
    ///
    /// Borrowed for the duration of one call; the manager owns it.
    pub(crate) const fn handle(&self) -> sys::CNA_Handle {
        self.handle
    }
}

impl Drop for NativeContentManager {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        let _ = unsafe { (self.native.runtime.content_manager_destroy)(self.handle) };
    }
}

/// A game's own `.cnb` loader for one custom asset type.
///
/// CNA calls this on whatever thread performs the load, so it must be `Sync`.
/// A panic must not cross back into C, so one is caught at the boundary and
/// reported as a failed load rather than unwinding.
pub trait CnbLoader: Send + Sync + 'static {
    /// Turns one validated document into an object.
    ///
    /// `document` is borrowed for exactly this call: CNA invalidates it before
    /// the call returns and it has no destroy operation, so keeping a copy and
    /// reading it later fails rather than reading freed memory. Anything the
    /// loader needs afterwards must be copied out here.
    fn load(&self, document: &CnbDocument, asset_name: &str)
        -> Result<Arc<dyn Any + Send + Sync>>;
}

/// One live registration, which withdraws itself when dropped.
///
/// Registrations are process-wide in CNA and outlive any content manager, so
/// this is what bounds one: holding it keeps the loader installed, and dropping
/// it withdraws the loader and releases every object the loader produced.
///
/// That last part is a deliberate ownership choice. CNA never dereferences,
/// copies or frees a loader's object -- "its lifetime is the caller's own
/// business" -- so something on this side has to own it. The registration does,
/// which means a load whose object CNA hands to C++ code Rust never sees again
/// is still released, at the latest when the registration goes.
#[derive(Debug)]
pub struct CnbLoaderRegistration {
    asset_type: AssetTypeId,
}

impl CnbLoaderRegistration {
    /// The asset type this registration serves.
    #[must_use]
    pub const fn asset_type(&self) -> AssetTypeId {
        self.asset_type
    }
}

impl Drop for CnbLoaderRegistration {
    fn drop(&mut self) {
        let _ = CnbLoaderRegistry::remove(self.asset_type);
    }
}

/// One entry of this crate's side of the registry.
struct LoaderEntry {
    loader: Arc<dyn CnbLoader>,
    /// Every object this loader produced, keyed by the pointer handed to CNA.
    produced: Vec<(usize, Arc<dyn Any + Send + Sync>)>,
}

type LoaderTable = Mutex<HashMap<u32, LoaderEntry>>;

fn loader_table() -> &'static LoaderTable {
    static TABLE: OnceLock<LoaderTable> = OnceLock::new();
    TABLE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The trampoline CNA calls.
///
/// The context is the asset type identifier itself rather than a pointer, so
/// there is no context lifetime to get wrong: a stale registration simply finds
/// nothing in the table and fails the load.
unsafe extern "C" fn cnb_loader_trampoline(
    context: *mut c_void,
    document: sys::CNA_CnbDocumentHandle,
    _content_manager: sys::CNA_Handle,
    asset_name: sys::CNA_StringView,
    out_object: *mut *mut c_void,
) -> sys::CNA_Result {
    let asset_type = context as usize as u32;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let native = Native::process()?;
        let loader = {
            let table = loader_table()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match table.get(&asset_type) {
                Some(entry) => Arc::clone(&entry.loader),
                None => return Err(CnaError::InvalidInput("no Rust loader is registered")),
            }
        };
        // SAFETY: CNA documents `asset_name` as borrowed UTF-8 bytes valid for
        // the duration of this call, with no terminator.
        let name = unsafe { string_view_bytes(asset_name) };
        let name = core::str::from_utf8(name)
            .map_err(|_| CnaError::InvalidInput("CNA passed a non-UTF-8 asset name"))?;
        // The handle is CNA's, borrowed for exactly this call.
        let borrowed = CnbDocument {
            native,
            handle: document,
            owner: DocumentOwner::CallbackScoped,
        };
        loader.load(&borrowed, name)
    }));
    let object = match outcome {
        Ok(Ok(object)) => object,
        // A failed load is the loader's answer; a panic must not unwind into C,
        // so it becomes the same kind of failure rather than crossing back.
        Ok(Err(_)) | Err(_) => return sys::CNA_RESULT_IO,
    };
    let pointer = Arc::as_ptr(&object).cast::<c_void>().cast_mut();
    {
        let mut table = loader_table()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match table.get_mut(&asset_type) {
            Some(entry) => entry.produced.push((pointer as usize, object)),
            None => return sys::CNA_RESULT_IO,
        }
    }
    // SAFETY: `out_object` is CNA's live output for this call.
    unsafe { *out_object = pointer };
    sys::CNA_RESULT_SUCCESS
}

/// Reads a borrowed `CNA_StringView` as bytes.
///
/// # Safety
/// The view must describe live bytes for the duration of the borrow.
unsafe fn string_view_bytes<'a>(view: sys::CNA_StringView) -> &'a [u8] {
    if view.data.is_null() || view.byte_length == 0 {
        return &[];
    }
    let length = usize::try_from(view.byte_length).unwrap_or(0);
    // SAFETY: the caller guarantees `view` describes `length` live bytes.
    unsafe { core::slice::from_raw_parts(view.data.cast::<u8>(), length) }
}

/// CNA's process-wide `.cnb` loader registry.
///
/// Every operation here is process-wide, exactly as upstream's is: there is no
/// per-manager variant, and a registration outlives any content manager.
#[derive(Debug)]
pub struct CnbLoaderRegistry;

impl CnbLoaderRegistry {
    /// Installs a loader for one custom asset type.
    ///
    /// `canonical_type_name` is not a label. CNA compares it against the name
    /// the file itself carries before dispatching, so it must be exactly the
    /// string [`AssetTypeId::custom`] hashed; a custom identifier is a 31-bit
    /// hash and two unrelated game types can legitimately collide, so the name
    /// is what stops one game's file being decoded by another's loader.
    ///
    /// Only a custom identifier can be registered. CNA's built-in and reserved
    /// identifiers belong to CNA and there is deliberately no way to claim one.
    pub fn register(
        canonical_type_name: &str,
        loader: Arc<dyn CnbLoader>,
    ) -> Result<CnbLoaderRegistration> {
        let asset_type = AssetTypeId::custom(canonical_type_name)?;
        let native = Native::process()?;
        {
            let mut table = loader_table()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            table.insert(
                asset_type.value(),
                LoaderEntry {
                    loader,
                    produced: Vec::new(),
                },
            );
        }
        let view = string_view(canonical_type_name);
        // SAFETY: the name is borrowed for the call and CNA copies it; the
        // context is the identifier by value, not a pointer, so it cannot
        // dangle.
        let result = native.check(unsafe {
            (native.runtime.cnb_loader_register)(
                asset_type.value(),
                view,
                Some(cnb_loader_trampoline),
                asset_type.value() as usize as *mut c_void,
            )
        });
        if result.is_err() {
            // A refused registration must not leave this side claiming one.
            let _ = loader_table()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&asset_type.value());
        }
        result?;
        Ok(CnbLoaderRegistration { asset_type })
    }

    /// Withdraws a registration and releases the objects it produced.
    ///
    /// Answers whether one was installed; absence is an ordinary answer.
    pub fn remove(asset_type: AssetTypeId) -> Result<bool> {
        let native = Native::process()?;
        let mut removed = sys::CNA_FALSE;
        // SAFETY: the output is a live local of the declared type.
        let result = native.check(unsafe {
            (native.runtime.cnb_loader_remove)(asset_type.value(), &mut removed)
        });
        // Release this side's entry either way: leaving it behind after CNA has
        // dropped the registration would retain objects nothing can reach.
        let _ = loader_table()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&asset_type.value());
        result?;
        Ok(removed != sys::CNA_FALSE)
    }

    /// Withdraws every registration, CNA's and this crate's.
    ///
    /// Upstream describes this as primarily for test isolation, and notes that
    /// getting the whole built-in table back afterwards needs a content
    /// manager rather than [`CnbLoaderRegistry::register_builtins`] alone.
    pub fn clear() -> Result<()> {
        let native = Native::process()?;
        // SAFETY: the route takes no arguments.
        let result = native.check(unsafe { (native.runtime.cnb_loader_clear)() });
        loader_table()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        result
    }

    /// Whether a loader is installed for an identifier.
    pub fn is_registered(asset_type: AssetTypeId) -> Result<bool> {
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the output is a live local of the declared type.
        native.check(unsafe {
            (native.runtime.cnb_loader_is_registered)(asset_type.value(), &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// The canonical name an identifier was registered under.
    ///
    /// Empty when nothing is registered.
    pub fn registered_type_name(asset_type: AssetTypeId) -> Result<String> {
        let native = Native::process()?;
        let api = &native.runtime;
        let id = asset_type.value();
        read_string(
            |value| native.check(value),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.cnb_loader_registered_type_name_size)(id, bytes) },
            |destination, capacity, written| unsafe {
                (api.cnb_loader_copy_registered_type_name)(id, destination, capacity, written)
            },
        )
    }

    /// Installs the built-in loaders that need nothing but their own codec.
    ///
    /// `Curve` and `AnimationClip` only: upstream registers the other eight
    /// through a content manager, because they construct a runtime object.
    /// Idempotent, and every content manager calls it, so a game normally
    /// never needs to.
    pub fn register_builtins() -> Result<()> {
        let native = Native::process()?;
        // SAFETY: the route takes no arguments.
        native.check(unsafe { (native.runtime.cnb_loader_register_builtins)() })
    }

    /// Looks up a loader by identifier alone, without checking any type name.
    ///
    /// This is the wrong entry point for loading a file, and upstream says so:
    /// use [`CnbLoaderRegistry::resolve_for_document`], which also proves
    /// identity. This exists for tooling that has no document to hand.
    pub fn find(asset_type: AssetTypeId) -> Result<Option<CnbResolvedLoader>> {
        let native = Native::process()?;
        let mut found = sys::CNA_FALSE;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: both outputs are live locals; the handle is newly owned.
        native.check(unsafe {
            (native.runtime.cnb_loader_find)(asset_type.value(), &mut found, &mut handle)
        })?;
        if found == sys::CNA_FALSE || handle == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        Ok(Some(CnbResolvedLoader { native, handle }))
    }

    /// Resolves the loader that may decode a document, proving identity too.
    ///
    /// For a built-in type the number is authoritative, because CNA assigns
    /// those and they are frozen. For a custom one it is not: the document must
    /// also carry a canonical type name equal to the registered one, so a file
    /// whose 31-bit hash collides is refused instead of being decoded by
    /// somebody else's loader.
    pub fn resolve_for_document(document: &CnbDocument) -> Result<CnbResolvedLoader> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the document handle is live for the call; the output is a
        // live local receiving a newly owned loader handle.
        native.check(unsafe {
            (native.runtime.cnb_loader_resolve_for_document)(document.handle, &mut handle)
        })?;
        Ok(CnbResolvedLoader { native, handle })
    }
}

/// One loader, copied out of the registry and safe to invoke.
///
/// CNA returns loaders by value on purpose: a pointer into the table would be
/// invalidated by any later registration that rehashes it, with no way for the
/// holder to know.
#[derive(Debug)]
pub struct CnbResolvedLoader {
    native: Arc<Native>,
    handle: sys::CNA_CnbLoaderHandle,
}

impl CnbResolvedLoader {
    /// Runs the loader over a document.
    ///
    /// The document need not be the one this loader was resolved from, which is
    /// exactly why the loader is a value rather than a cursor into the table.
    ///
    /// The object comes back only for a loader registered from this crate.
    /// CNA's own built-in loaders construct C++ objects -- a `Curve`, a
    /// `Texture2D` -- and this reports [`CnaError::UnsupportedRuntime`] for one
    /// of those rather than handing back a pointer nothing here could name.
    pub fn invoke(
        &self,
        document: &CnbDocument,
        content_manager: &NativeContentManager,
        asset_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>> {
        let view = string_view(asset_name);
        let mut object: *mut c_void = core::ptr::null_mut();
        // SAFETY: both handles are live, the name is borrowed for the call, and
        // the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_loader_invoke)(
                self.handle,
                document.handle,
                content_manager.handle,
                view,
                &mut object,
            )
        })?;
        let key = object as usize;
        let table = loader_table()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for entry in table.values() {
            if let Some((_, value)) = entry.produced.iter().find(|(pointer, _)| *pointer == key) {
                return Ok(Arc::clone(value));
            }
        }
        Err(CnaError::UnsupportedRuntime(
            "this loader produced an object that did not come from a Rust loader",
        ))
    }
}

impl Drop for CnbResolvedLoader {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        let _ = unsafe { (self.native.runtime.cnb_loader_destroy)(self.handle) };
    }
}

/// One glyph of a compiled sprite font.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CnbGlyph {
    /// The character this glyph draws, as XNA's UTF-16 code unit.
    pub character: u16,
    /// The glyph's source rectangle inside the atlas.
    pub glyph_bounds: Rectangle,
    /// The per-glyph cropping and offset rectangle.
    pub cropping: Rectangle,
    /// Left bearing, glyph width and right bearing.
    pub kerning: Vector3,
}

/// A compiled sprite font's metrics, without its glyphs or its atlas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CnbSpriteFontInfo {
    pub glyph_count: u64,
    pub line_spacing: i32,
    pub spacing: f32,
    /// The fallback glyph, or `None` when a missing character is an error.
    ///
    /// XNA distinguishes these: a font without a default character throws on a
    /// character it has no glyph for, and one with a default substitutes it.
    /// The container stores the two separately for that reason, and this keeps
    /// them apart rather than encoding "no default" as some particular char.
    pub default_character: Option<u16>,
}

/// A compiled sprite font: metrics, glyphs and an atlas.
#[derive(Debug)]
pub struct CnbSpriteFont {
    pub(crate) native: Arc<Native>,
    pub(crate) handle: sys::CNA_CnbSpriteFontDataHandle,
}

impl CnbSpriteFont {
    /// Starts an empty font to author.
    pub fn new() -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local receiving a newly owned handle.
        native.check(unsafe { (native.runtime.cnb_sprite_font_create)(&mut handle) })?;
        Ok(Self { native, handle })
    }

    /// Sets the font's metrics. `glyph_count` is ignored here, as upstream says.
    pub fn set_info(&self, info: CnbSpriteFontInfo) -> Result<()> {
        let native = sys::CNA_CnbSpriteFontInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbSpriteFontInfo>() as u32,
            struct_version: sys::CNA_CNB_SPRITE_FONT_INFO_STRUCT_VERSION,
            glyph_count: 0,
            line_spacing: info.line_spacing,
            spacing: info.spacing,
            default_character: info.default_character.unwrap_or(0),
            has_default_character: u8::from(info.default_character.is_some()),
            reserved: [0; 5],
        };
        // SAFETY: the descriptor is a live local CNA copies during the call.
        self.native
            .check(unsafe { (self.native.runtime.cnb_sprite_font_set_info)(self.handle, &native) })
    }

    /// The font's metrics and glyph count.
    pub fn info(&self) -> Result<CnbSpriteFontInfo> {
        let mut info = sys::CNA_CnbSpriteFontInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbSpriteFontInfo>() as u32,
            struct_version: sys::CNA_CNB_SPRITE_FONT_INFO_STRUCT_VERSION,
            ..sys::CNA_CnbSpriteFontInfo::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output.
        self.native
            .check(unsafe { (self.native.runtime.cnb_sprite_font_get_info)(self.handle, &mut info) })?;
        Ok(CnbSpriteFontInfo {
            glyph_count: info.glyph_count,
            line_spacing: info.line_spacing,
            spacing: info.spacing,
            default_character: (info.has_default_character != sys::CNA_FALSE)
                .then_some(info.default_character),
        })
    }

    /// Appends a glyph and answers its index.
    pub fn add_glyph(&self, glyph: CnbGlyph) -> Result<u64> {
        let native = glyph.to_native();
        let mut index = 0_u64;
        // SAFETY: the descriptor is a live local CNA copies; the output is a
        // live local.
        self.native.check(unsafe {
            (self.native.runtime.cnb_sprite_font_add_glyph)(self.handle, &native, &mut index)
        })?;
        Ok(index)
    }

    /// One glyph by index.
    pub fn glyph(&self, index: u64) -> Result<CnbGlyph> {
        let mut native = sys::CNA_SpriteFontGlyph {
            struct_size: core::mem::size_of::<sys::CNA_SpriteFontGlyph>() as u32,
            struct_version: 1,
            glyph_bounds: sys::CNA_Rectangle::default(),
            cropping: sys::CNA_Rectangle::default(),
            character: 0,
            reserved: 0,
            kerning: sys::CNA_Vector3::default(),
        };
        // SAFETY: the descriptor is a caller-owned versioned output.
        self.native.check(unsafe {
            (self.native.runtime.cnb_sprite_font_get_glyph)(self.handle, index, &mut native)
        })?;
        Ok(CnbGlyph::from_native(native))
    }

    /// Sets the atlas the glyph rectangles index into.
    ///
    /// The texture data is copied into the font, so the caller keeps its own.
    pub fn set_atlas(&self, atlas: &CnbTextureData) -> Result<()> {
        // SAFETY: both handles are owned and live for the call.
        self.native.check(unsafe {
            (self.native.runtime.cnb_sprite_font_set_atlas)(self.handle, atlas.handle)
        })
    }

    /// The font's atlas, as a newly owned texture.
    pub fn atlas(&self) -> Result<CnbTextureData> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local receiving a newly owned handle.
        self.native.check(unsafe {
            (self.native.runtime.cnb_sprite_font_copy_atlas)(self.handle, &mut handle)
        })?;
        Ok(CnbTextureData {
            native: Arc::clone(&self.native),
            handle,
        })
    }

    /// Encodes the font as a complete `.cnb` document.
    pub fn encode(&self, content_name: &str) -> Result<Vec<u8>> {
        encode_document(&self.native, content_name, |view, destination, capacity, written| {
            // SAFETY: the handle is owned and the destination is either null
            // with zero capacity or a live buffer of exactly `capacity` bytes.
            unsafe {
                (self.native.runtime.cnb_encode_sprite_font)(
                    self.handle,
                    view,
                    destination,
                    capacity,
                    written,
                )
            }
        })
    }
}

impl Drop for CnbSpriteFont {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        let _ = unsafe { (self.native.runtime.cnb_sprite_font_destroy)(self.handle) };
    }
}

impl CnbGlyph {
    pub(crate) fn to_native(self) -> sys::CNA_SpriteFontGlyph {
        sys::CNA_SpriteFontGlyph {
            struct_size: core::mem::size_of::<sys::CNA_SpriteFontGlyph>() as u32,
            struct_version: 1,
            glyph_bounds: sys::CNA_Rectangle {
                x: self.glyph_bounds.X,
                y: self.glyph_bounds.Y,
                width: self.glyph_bounds.Width,
                height: self.glyph_bounds.Height,
            },
            cropping: sys::CNA_Rectangle {
                x: self.cropping.X,
                y: self.cropping.Y,
                width: self.cropping.Width,
                height: self.cropping.Height,
            },
            character: self.character,
            reserved: 0,
            kerning: sys::CNA_Vector3 {
                x: self.kerning.X,
                y: self.kerning.Y,
                z: self.kerning.Z,
            },
        }
    }

    const fn from_native(value: sys::CNA_SpriteFontGlyph) -> Self {
        Self {
            character: value.character,
            glyph_bounds: Rectangle {
                X: value.glyph_bounds.x,
                Y: value.glyph_bounds.y,
                Width: value.glyph_bounds.width,
                Height: value.glyph_bounds.height,
            },
            cropping: Rectangle {
                X: value.cropping.x,
                Y: value.cropping.y,
                Width: value.cropping.width,
                Height: value.cropping.height,
            },
            kerning: Vector3 {
                X: value.kerning.x,
                Y: value.kerning.y,
                Z: value.kerning.z,
            },
        }
    }
}

/// How a compiled sound's samples are encoded.
///
/// CNA's own identity: schema 1 writes `Pcm16`, and the rest exist for later
/// schemas. An unknown value is reported rather than mapped onto the nearest
/// format this build happens to know.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CnbAudioFormat {
    Unknown,
    Pcm16,
    Pcm8,
    PcmFloat32,
    Adpcm,
}

impl CnbAudioFormat {
    const fn from_native(value: sys::CNA_CnbAudioFormat) -> Option<Self> {
        Some(match value {
            sys::CNA_CNB_AUDIO_FORMAT_UNKNOWN => Self::Unknown,
            sys::CNA_CNB_AUDIO_FORMAT_PCM16 => Self::Pcm16,
            sys::CNA_CNB_AUDIO_FORMAT_PCM8 => Self::Pcm8,
            sys::CNA_CNB_AUDIO_FORMAT_PCM_FLOAT32 => Self::PcmFloat32,
            sys::CNA_CNB_AUDIO_FORMAT_ADPCM => Self::Adpcm,
            _ => return None,
        })
    }

    pub(crate) const fn to_native(self) -> sys::CNA_CnbAudioFormat {
        match self {
            Self::Unknown => sys::CNA_CNB_AUDIO_FORMAT_UNKNOWN,
            Self::Pcm16 => sys::CNA_CNB_AUDIO_FORMAT_PCM16,
            Self::Pcm8 => sys::CNA_CNB_AUDIO_FORMAT_PCM8,
            Self::PcmFloat32 => sys::CNA_CNB_AUDIO_FORMAT_PCM_FLOAT32,
            Self::Adpcm => sys::CNA_CNB_AUDIO_FORMAT_ADPCM,
        }
    }
}

/// A compiled sound's encoding, rate, shape and loop region.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CnbSoundEffectInfo {
    pub format: CnbAudioFormat,
    pub sample_rate: u32,
    pub channels: u32,
    /// Sample frames, that is, samples per channel.
    pub frame_count: u32,
    /// The loop region, or `None` when the sound does not loop.
    ///
    /// The container encodes "no loop" as a zero length rather than as an
    /// absent field, and this keeps the distinction rather than handing back a
    /// zero-length region a caller could loop on forever.
    pub loop_region: Option<(u32, u32)>,
}

/// A compiled sound effect: its shape and its samples.
#[derive(Debug)]
pub struct CnbSoundEffect {
    pub(crate) native: Arc<Native>,
    pub(crate) handle: sys::CNA_CnbSoundEffectDataHandle,
}

impl CnbSoundEffect {
    /// Builds a sound from its shape and its raw sample bytes.
    pub fn new(info: CnbSoundEffectInfo, samples: &[u8]) -> Result<Self> {
        let native = Native::process()?;
        let (loop_start, loop_length) = info.loop_region.unwrap_or((0, 0));
        let descriptor = sys::CNA_CnbSoundEffectInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbSoundEffectInfo>() as u32,
            struct_version: sys::CNA_CNB_SOUND_EFFECT_INFO_STRUCT_VERSION,
            format: info.format.to_native(),
            sample_rate: info.sample_rate,
            channels: info.channels,
            frame_count: info.frame_count,
            loop_start,
            loop_length,
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the descriptor and samples are borrowed for the call with the
        // slice's own length; the output is a live local.
        native.check(unsafe {
            (native.runtime.cnb_sound_effect_create)(
                &descriptor,
                if samples.is_empty() {
                    core::ptr::null()
                } else {
                    samples.as_ptr()
                },
                samples.len() as u64,
                &mut handle,
            )
        })?;
        Ok(Self { native, handle })
    }

    /// The sound's shape.
    pub fn info(&self) -> Result<CnbSoundEffectInfo> {
        let mut info = sys::CNA_CnbSoundEffectInfo {
            struct_size: core::mem::size_of::<sys::CNA_CnbSoundEffectInfo>() as u32,
            struct_version: sys::CNA_CNB_SOUND_EFFECT_INFO_STRUCT_VERSION,
            ..sys::CNA_CnbSoundEffectInfo::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output.
        self.native.check(unsafe {
            (self.native.runtime.cnb_sound_effect_get_info)(self.handle, &mut info)
        })?;
        let format = CnbAudioFormat::from_native(info.format).ok_or(
            CnaError::UnsupportedRuntime("this .cnb sound names an audio format this build does not know"),
        )?;
        Ok(CnbSoundEffectInfo {
            format,
            sample_rate: info.sample_rate,
            channels: info.channels,
            frame_count: info.frame_count,
            loop_region: (info.loop_length != 0).then_some((info.loop_start, info.loop_length)),
        })
    }

    /// The sound's raw sample bytes.
    pub fn samples(&self) -> Result<Vec<u8>> {
        let api = &self.native.runtime;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the size.
        accept_size_probe(&self.native, unsafe {
            (api.cnb_sound_effect_copy_samples)(self.handle, core::ptr::null_mut(), 0, &mut required)
        })?;
        let capacity = usize::try_from(required)
            .map_err(|_| CnaError::InvalidInput("sample block is too large"))?;
        let mut bytes = vec![0_u8; capacity];
        let mut written = 0_u64;
        // SAFETY: `bytes` holds exactly `required` writable bytes for the call.
        self.native.check(unsafe {
            (api.cnb_sound_effect_copy_samples)(
                self.handle,
                if capacity == 0 {
                    core::ptr::null_mut()
                } else {
                    bytes.as_mut_ptr()
                },
                required,
                &mut written,
            )
        })?;
        let written = usize::try_from(written)
            .map_err(|_| CnaError::InvalidInput("sample block is too large"))?;
        bytes.truncate(written.min(capacity));
        Ok(bytes)
    }

    /// Encodes the sound as a complete `.cnb` document.
    pub fn encode(&self, content_name: &str) -> Result<Vec<u8>> {
        encode_document(&self.native, content_name, |view, destination, capacity, written| {
            // SAFETY: as for the sprite font encode above.
            unsafe {
                (self.native.runtime.cnb_encode_sound_effect)(
                    self.handle,
                    view,
                    destination,
                    capacity,
                    written,
                )
            }
        })
    }
}

impl Drop for CnbSoundEffect {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this value and released exactly once.
        let _ = unsafe { (self.native.runtime.cnb_sound_effect_destroy)(self.handle) };
    }
}

/// The canonical size-probe-then-copy shape every `.cnb` encode route uses.
fn encode_document(
    native: &Arc<Native>,
    content_name: &str,
    mut encode: impl FnMut(sys::CNA_StringView, *mut u8, u64, *mut u64) -> sys::CNA_Result,
) -> Result<Vec<u8>> {
    let view = string_view(content_name);
    let mut required = 0_u64;
    accept_size_probe(native, encode(view, core::ptr::null_mut(), 0, &mut required))?;
    let capacity = usize::try_from(required)
        .map_err(|_| CnaError::InvalidInput("encoded document is too large"))?;
    let mut bytes = vec![0_u8; capacity];
    let mut written = 0_u64;
    native.check(encode(view, bytes.as_mut_ptr(), required, &mut written))?;
    let written = usize::try_from(written)
        .map_err(|_| CnaError::InvalidInput("encoded document is too large"))?;
    bytes.truncate(written.min(capacity));
    Ok(bytes)
}

/// What one entry of the content manifest carries.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct ManifestEntry {
    /// The path relative to the content root, as the manifest spells it.
    pub relative_path: String,
    /// Whether a compiled `.xnb` exists for it.
    pub has_xnb: bool,
    /// Whether a `.cnj` sidecar exists for it.
    pub has_cnj: bool,
    /// The native source extensions beside it, such as `.png` or `.gltf`.
    pub native_extensions: Vec<String>,
    /// The reader names the `.xnb` declares, when it has one.
    pub xnb_reader_names: Vec<String>,
}

/// How often one `.xnb` reader name is used across the content root.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct ReaderUsage {
    pub name: String,
    /// Whether this build has a reader registered under that name.
    pub is_registered: bool,
    /// How many files declare it.
    pub file_count: u64,
}

/// The manager's own surface: what it can see, and what it will load.
impl NativeContentManager {
    /// Creates a manager that loads through CNA's *resource* path.
    ///
    /// The difference from [`Self::new`] is where assets come from: this one
    /// reads the resources compiled into the application rather than a
    /// directory on disk, which is what a single-file build ships.
    pub fn new_resource(graphics_device: &GraphicsDevice, root_directory: &str) -> Result<Self> {
        let native = Native::process()?;
        let info = sys::CNA_ContentManagerCreateInfo {
            struct_size: core::mem::size_of::<sys::CNA_ContentManagerCreateInfo>() as u32,
            struct_version: 1,
            root_directory: string_view(root_directory),
            reserved: 0,
        };
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the descriptor borrows `root_directory` for the call and CNA
        // copies it; the output is a live local.
        native.check(unsafe {
            (native.runtime.content_manager_create_resource)(
                graphics_device.handle()?,
                &info,
                &mut handle,
            )
        })?;
        Ok(Self { native, handle })
    }

    /// The directory this manager resolves asset names against.
    pub fn root_directory(&self) -> Result<String> {
        let api = &self.native.runtime;
        crate::native::runtime::read_string(
            |value| self.native.check(value),
            // SAFETY: owned handle, live outputs; the size-then-copy pair.
            |bytes| unsafe { (api.content_manager_get_root_directory_size)(self.handle, bytes) },
            |destination, capacity, written| unsafe {
                (api.content_manager_copy_root_directory)(
                    self.handle,
                    destination,
                    capacity,
                    written,
                )
            },
        )
    }

    /// Changes the directory asset names resolve against.
    pub fn set_root_directory(&self, path: &str) -> Result<()> {
        // SAFETY: the handle is owned and the path is borrowed for the call.
        self.native.check(unsafe {
            (self.native.runtime.content_manager_set_root_directory)(
                self.handle,
                string_view(path),
            )
        })
    }

    /// The file one asset name resolves to.
    ///
    /// Which extension wins is the manager's rule, not the caller's, so this is
    /// the only honest way to find out what a name will actually open.
    pub fn asset_path(&self, asset_name: &str) -> Result<String> {
        let api = &self.native.runtime;
        crate::native::runtime::read_string(
            |value| self.native.check(value),
            // SAFETY: owned handle, borrowed name, live outputs.
            |bytes| unsafe {
                (api.content_manager_get_asset_path_size)(
                    self.handle,
                    string_view(asset_name),
                    bytes,
                )
            },
            |destination, capacity, written| unsafe {
                (api.content_manager_copy_asset_path)(
                    self.handle,
                    string_view(asset_name),
                    destination,
                    capacity,
                    written,
                )
            },
        )
    }

    /// The key an asset name is cached under.
    ///
    /// Two names that normalise to one key are the same asset, which is what
    /// makes a second load a cache hit rather than a second file read.
    ///
    /// The normalisation is exactly two rules -- backslashes become forward
    /// slashes, and the whole key is lowercased -- so `Textures\Hero` and
    /// `textures/hero` are one asset. It does **not** resolve a path:
    /// `./listed` and `listed` are two different keys and two separate cache
    /// entries, measured rather than assumed.
    pub fn normalized_key(&self, asset_name: &str) -> Result<String> {
        let api = &self.native.runtime;
        crate::native::runtime::read_string(
            |value| self.native.check(value),
            // SAFETY: owned handle, borrowed name, live outputs.
            |bytes| unsafe {
                (api.content_manager_get_normalized_key_size)(
                    self.handle,
                    string_view(asset_name),
                    bytes,
                )
            },
            |destination, capacity, written| unsafe {
                (api.content_manager_copy_normalized_key)(
                    self.handle,
                    string_view(asset_name),
                    destination,
                    capacity,
                    written,
                )
            },
        )
    }

    /// Whether a service provider is attached.
    pub fn has_service_provider(&self) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.content_manager_get_has_service_provider)(self.handle, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// The identity of the device this manager loads onto.
    pub fn graphics_device_identity(&self) -> Result<u64> {
        let mut value = sys::CNA_INVALID_HANDLE;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.content_manager_get_graphics_device)(self.handle, &mut value)
        })?;
        Ok(value)
    }

    /// Points the manager at a different device.
    pub fn set_graphics_device(&self, graphics_device: &GraphicsDevice) -> Result<()> {
        // SAFETY: both handles belong to live values.
        self.native.check(unsafe {
            (self.native.runtime.content_manager_set_graphics_device)(
                self.handle,
                graphics_device.handle()?,
            )
        })
    }

    /// Drops every cached asset.
    ///
    /// XNA's `ContentManager.Unload`. Anything still holding a loaded asset is
    /// holding it past this point, which is the caller's business.
    pub fn unload(&self) -> Result<()> {
        // SAFETY: the handle is owned.
        self.native
            .check(unsafe { (self.native.runtime.content_manager_unload)(self.handle) })
    }

    /// Registers CNA's own loaders for the built-in asset types.
    pub fn register_builtin_loaders(&self) -> Result<()> {
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self.native.runtime.content_manager_register_builtin_loaders)(self.handle)
        })
    }

    /// Re-reads the content root and rebuilds the manifest.
    pub fn refresh_content_manifest(&self) -> Result<()> {
        // SAFETY: the handle is owned.
        self.native.check(unsafe {
            (self.native.runtime.content_manager_refresh_content_manifest)(self.handle)
        })
    }

    /// How many entries the manifest holds.
    pub fn manifest_entry_count(&self) -> Result<u64> {
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.content_manager_get_manifest_entry_count)(self.handle, &mut value)
        })?;
        Ok(value)
    }

    /// One manifest entry, with its names.
    pub fn manifest_entry(&self, index: u64) -> Result<ManifestEntry> {
        let api = &self.native.runtime;
        let mut info = sys::CNA_ContentManifestEntryInfo {
            struct_size: core::mem::size_of::<sys::CNA_ContentManifestEntryInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_ContentManifestEntryInfo::default()
        };
        // SAFETY: the handle is owned and the output is a live local whose size
        // and version headers are set.
        self.native.check(unsafe {
            (api.content_manager_get_manifest_entry)(self.handle, index, &mut info)
        })?;
        // The relative path has no separate size route, so the copy route
        // asked with a zero capacity is the size probe -- which answers
        // `BUFFER_TOO_SMALL` rather than success, and that is the answer.
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the size.
        accept_size_probe(&self.native, unsafe {
            (api.content_manager_copy_manifest_relative_path)(
                self.handle,
                index,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        })?;
        let relative_path = crate::native::runtime::read_string_of_size(
            required,
            |value| self.native.check(value),
            // SAFETY: owned handle, `capacity` writable bytes.
            |destination, capacity, written| unsafe {
                (api.content_manager_copy_manifest_relative_path)(
                    self.handle,
                    index,
                    destination,
                    capacity,
                    written,
                )
            },
        )?;
        let native_extensions = (0..info.native_extension_count)
            .map(|slot| self.manifest_text(index, slot, true))
            .collect::<Result<Vec<String>>>()?;
        let xnb_reader_names = (0..info.xnb_reader_name_count)
            .map(|slot| self.manifest_text(index, slot, false))
            .collect::<Result<Vec<String>>>()?;
        Ok(ManifestEntry {
            relative_path,
            has_xnb: info.has_xnb != sys::CNA_FALSE,
            has_cnj: info.has_cnj != sys::CNA_FALSE,
            native_extensions,
            xnb_reader_names,
        })
    }

    fn manifest_text(&self, index: u64, slot: u64, extension: bool) -> Result<String> {
        let api = &self.native.runtime;
        let route = if extension {
            api.content_manager_copy_manifest_native_extension
        } else {
            api.content_manager_copy_manifest_xnb_reader_name
        };
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the size.
        accept_size_probe(&self.native, unsafe {
            route(self.handle, index, slot, core::ptr::null_mut(), 0, &mut required)
        })?;
        crate::native::runtime::read_string_of_size(
            required,
            |value| self.native.check(value),
            // SAFETY: owned handle, `capacity` writable bytes.
            |destination, capacity, written| unsafe {
                route(self.handle, index, slot, destination, capacity, written)
            },
        )
    }

    /// How many distinct `.xnb` reader names the content root declares.
    pub fn xnb_reader_usage_count(&self) -> Result<u64> {
        let mut value = 0_u64;
        // SAFETY: the handle is owned and the output is a live local.
        self.native.check(unsafe {
            (self.native.runtime.content_manager_get_xnb_reader_usage_count)(
                self.handle,
                &mut value,
            )
        })?;
        Ok(value)
    }

    /// One reader name, and whether this build can serve it.
    ///
    /// The answer to "will this content load on this build" before trying it,
    /// and the list a packaging step checks.
    pub fn xnb_reader_usage(&self, index: u64) -> Result<ReaderUsage> {
        let api = &self.native.runtime;
        let mut info = sys::CNA_ContentReaderUsageInfo {
            struct_size: core::mem::size_of::<sys::CNA_ContentReaderUsageInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_ContentReaderUsageInfo::default()
        };
        // SAFETY: the handle is owned and the output is a live local whose
        // headers are set.
        self.native
            .check(unsafe { (api.content_manager_get_xnb_reader_usage)(self.handle, index, &mut info) })?;
        let mut required = 0_u64;
        // SAFETY: a null destination with zero capacity asks for the size.
        accept_size_probe(&self.native, unsafe {
            (api.content_manager_copy_xnb_reader_usage_name)(
                self.handle,
                index,
                core::ptr::null_mut(),
                0,
                &mut required,
            )
        })?;
        let name = crate::native::runtime::read_string_of_size(
            required,
            |value| self.native.check(value),
            // SAFETY: owned handle, `capacity` writable bytes.
            |destination, capacity, written| unsafe {
                (api.content_manager_copy_xnb_reader_usage_name)(
                    self.handle,
                    index,
                    destination,
                    capacity,
                    written,
                )
            },
        )?;
        Ok(ReaderUsage {
            name,
            is_registered: info.is_registered != sys::CNA_FALSE,
            file_count: info.file_count,
        })
    }
}

/// Loading assets CNA's own manager knows how to read.
///
/// The Rust content pipeline reads `.xnb` and produces Rust values; these read
/// whatever the manager resolves a name to, which on a CNA content root
/// includes `.cnj` and the native source formats. Both paths are real and
/// neither replaces the other.
impl NativeContentManager {
    /// Loads a `SpriteFont` and the atlas it draws from.
    ///
    /// XNA's `ContentManager.Load::<SpriteFont>` reads an `.xnb` font
    /// container, and the Rust content pipeline does exactly that. This reads
    /// whatever CNA's manager resolves the name to, which on a CNA content
    /// root also includes its own `.cnj` font descriptor -- a format the Rust
    /// reader does not parse and does not intend to.
    ///
    /// # Ownership
    ///
    /// Two owned handles, one asset. The returned `SpriteFont` owns the font;
    /// the `Arc<Texture2D>` it holds owns the atlas, and the same one is
    /// returned so a caller that places glyphs itself can reach it. CNA
    /// refuses to destroy the atlas while the font uses it, so the font is
    /// released first -- which the `SpriteFont`'s own field order guarantees.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports. On any failure nothing is created
    /// and neither handle leaks.
    pub fn load_sprite_font(
        &self,
        graphics_device: &GraphicsDevice,
        asset_name: &str,
    ) -> Result<(crate::graphics::SpriteFont, Arc<Texture2D>)> {
        let mut font = sys::CNA_INVALID_HANDLE;
        let mut atlas = sys::CNA_INVALID_HANDLE;
        // SAFETY: the manager handle is owned, the name is borrowed and copied,
        // and both outputs are live locals receiving owned handles.
        self.native.check(unsafe {
            (self.native.runtime.content_manager_load_sprite_font)(
                self.handle,
                string_view(asset_name),
                &mut font,
                &mut atlas,
            )
        })?;
        let texture = match Texture2D::from_owned_handle(graphics_device, atlas) {
            Ok(texture) => Arc::new(texture),
            Err(error) => {
                // The font references the atlas, so it goes first. Neither
                // handle has an owner yet, which is why this is the one place
                // both can be released.
                let _ = self.native.destroy_sprite_font(font);
                let _ = self.native.destroy_texture(atlas);
                return Err(error);
            }
        };
        let adopted = crate::graphics::SpriteFont::adopt(Arc::clone(&texture), font)?;
        Ok((adopted, texture))
    }

    /// Loads a `SoundEffect` by asset name.
    ///
    /// CNA's loader deliberately does not cache: a second load of the same
    /// name answers a second, independently owned effect, which is what XNA's
    /// own `Load::<SoundEffect>` specialization does.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, including the refusal when no
    /// audio device is available.
    pub fn load_sound_effect(
        &self,
        game: &GameContext<'_>,
        asset_name: &str,
    ) -> Result<crate::audio::SoundEffect> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the manager handle is owned, the name is borrowed and copied,
        // and the output is a live local receiving an owned handle.
        self.native.check(unsafe {
            (self.native.runtime.content_manager_load_sound_effect)(
                self.handle,
                string_view(asset_name),
                &mut handle,
            )
        })?;
        crate::audio::SoundEffect::adopt(game, handle)
    }

    /// Loads a texture by asset name.
    pub fn load_texture2d(
        &self,
        graphics_device: &GraphicsDevice,
        asset_name: &str,
    ) -> Result<Texture2D> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the manager handle is owned, the name is borrowed and copied,
        // and the output is a live local receiving an owned handle.
        self.native.check(unsafe {
            (self.native.runtime.content_manager_load_texture2d)(
                self.handle,
                string_view(asset_name),
                &mut handle,
            )
        })?;
        Texture2D::from_owned_handle(graphics_device, handle)
    }

    /// Loads a cube map by asset name.
    pub fn load_texture_cube(
        &self,
        graphics_device: &GraphicsDevice,
        asset_name: &str,
    ) -> Result<TextureCube> {
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: as above.
        self.native.check(unsafe {
            (self.native.runtime.content_manager_load_texture_cube)(
                self.handle,
                string_view(asset_name),
                &mut handle,
            )
        })?;
        TextureCube::from_owned_handle(graphics_device, handle)
    }

    /// Loads an asset a caller's own reader produced.
    ///
    /// The pointer is the caller's own: CNA never dereferences, copies or frees
    /// it, and its lifetime is the caller's business. Raw for exactly that
    /// reason -- a safe Rust value here would claim an ownership nobody has.
    pub fn load_foreign(&self, asset_name: &str) -> Result<*mut core::ffi::c_void> {
        let mut object = core::ptr::null_mut();
        // SAFETY: the manager handle is owned, the name is borrowed for the
        // call, and the output is a live local. The pointer is not
        // dereferenced here.
        self.native.check(unsafe {
            (self.native.runtime.content_manager_load_foreign_ext)(
                self.handle,
                string_view(asset_name),
                &mut object,
            )
        })?;
        Ok(object)
    }
}

/// A content manager a `Game` owns, borrowed for as long as the game lives.
///
/// A game owns exactly one as a value member, so this borrows rather than owns:
/// it is the same manager every time, it cannot be destroyed on its own, and it
/// goes when the game does. Every content-manager route accepts it, so a caller
/// can set the root the game loads from and load through the game's own cache.
pub struct BorrowedContentManager<'game> {
    manager: NativeContentManager,
    owner: core::marker::PhantomData<&'game ()>,
}

impl BorrowedContentManager<'_> {
    /// The manager itself, for as long as this borrow lives.
    #[must_use]
    pub const fn manager(&self) -> &NativeContentManager {
        &self.manager
    }
}

impl Drop for BorrowedContentManager<'_> {
    fn drop(&mut self) {
        // The handle is the game's, not this value's: forget it rather than let
        // `NativeContentManager`'s own Drop destroy a manager the game owns.
        self.manager.handle = sys::CNA_INVALID_HANDLE;
    }
}

/// The content manager a game owns, and replacing it.
impl NativeContentManager {
    /// Borrows the manager a game owns.
    ///
    /// The borrow is bound to the [`GameContext`] it came from, which is what
    /// stops it outliving the game -- upstream allows that and answers an
    /// invalid handle, and a Rust caller should not have to find out that way.
    pub fn of_game<'game>(
        game: &'game GameContext<'_>,
    ) -> Result<BorrowedContentManager<'game>> {
        let (native, game_handle) = game.native_game();
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the game handle is callback-live and the output is a live
        // local receiving a borrowed handle.
        native.check(unsafe {
            (native.runtime.game_get_content_manager_ext)(game_handle, &mut handle)
        })?;
        Ok(BorrowedContentManager {
            manager: Self {
                native: Arc::clone(native),
                handle,
            },
            owner: core::marker::PhantomData,
        })
    }

    /// Makes this manager the one a game loads through.
    ///
    /// The game takes over the manager's lifetime, so this consumes it: keeping
    /// a second owner would mean two values racing to destroy one manager.
    pub fn install_into_game(self, game: &GameContext<'_>) -> Result<()> {
        let (native, game_handle) = game.native_game();
        let handle = self.handle;
        // SAFETY: both handles are live, and `self` is forgotten below so its
        // Drop cannot destroy what the game now owns.
        let result = native
            .check(unsafe { (native.runtime.game_set_content_manager_ext)(game_handle, handle) });
        if result.is_ok() {
            core::mem::forget(self);
        }
        result
    }
}

/// A game's own loader for one `"type"` in a `.cnj` descriptor.
///
/// The `.cnj` counterpart of a registered `.xnb` reader, and what makes
/// [`NativeContentManager::load_foreign`] reach more than compiled assets: a
/// reader answers for an `.xnb`, this answers for a `.cnj`, and the load route
/// does not care which produced the object.
///
/// CNA calls this on whatever thread performs the load, so it must be `Sync`. A
/// panic must not cross back into C, so one is caught at the boundary and
/// reported as a failed load rather than unwinding.
pub trait CnjLoader: Send + Sync + 'static {
    /// Turns one descriptor's JSON into an object.
    ///
    /// The pointer this answers is the caller's own: CNA never dereferences,
    /// copies or frees it, and whoever asked for the asset receives it. Its
    /// lifetime is the caller's business, which is why this returns a raw
    /// pointer rather than something with a `Drop`.
    fn load(&self, cnj_json: &str) -> Result<*mut core::ffi::c_void>;
}

/// One live `.cnj` loader registration.
///
/// The registration is owned by the content manager, and upstream says the
/// context "must outlive the content manager". There is no withdraw route, so
/// this value keeps the boxed loader alive for the whole process rather than
/// guessing when the manager is done with it -- one small allocation per
/// registered type, in a program that registers a handful.
#[derive(Debug)]
pub struct CnjLoaderRegistration {
    type_name: String,
}

impl CnjLoaderRegistration {
    /// The `"type"` value this registration answers for.
    #[must_use]
    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}

impl NativeContentManager {
    /// Registers a loader for one `"type"` value in a `.cnj` descriptor.
    ///
    /// Refused when that type name is already registered on this manager,
    /// which is deliberate on CNA's side: silently ignoring a repeat would
    /// hand back a live registration whose loader is never called.
    pub fn register_cnj_loader(
        &self,
        type_name: &str,
        loader: Box<dyn CnjLoader>,
    ) -> Result<CnjLoaderRegistration> {
        unsafe extern "C" fn trampoline(
            context: *mut core::ffi::c_void,
            cnj_json: sys::CNA_StringView,
            out_object: *mut *mut core::ffi::c_void,
        ) -> sys::CNA_Result {
            if context.is_null() || out_object.is_null() {
                return sys::CNA_RESULT_INVALID_ARGUMENT;
            }
            // SAFETY: the context is the leaked box registered below, which
            // outlives every manager that can reach it.
            let loader = unsafe { &*context.cast::<Box<dyn CnjLoader>>() };
            let length = usize::try_from(cnj_json.byte_length).unwrap_or(0);
            let text = if cnj_json.data.is_null() || length == 0 {
                String::new()
            } else {
                // SAFETY: CNA documents the bytes as counted UTF-8 borrowed for
                // this call; they are copied before it returns.
                let bytes =
                    unsafe { core::slice::from_raw_parts(cnj_json.data.cast::<u8>(), length) };
                String::from_utf8_lossy(bytes).into_owned()
            };
            let outcome = catch_unwind(AssertUnwindSafe(|| loader.load(&text)));
            match outcome {
                Ok(Ok(object)) => {
                    // SAFETY: the output is a live pointer CNA gave us.
                    unsafe { *out_object = object };
                    sys::CNA_RESULT_SUCCESS
                }
                // A failed load and a panicking loader are the same thing to
                // the caller who asked for the asset: the asset did not load.
                Ok(Err(_)) | Err(_) => sys::CNA_RESULT_IO,
            }
        }

        let boxed = Box::new(loader);
        let context = Box::into_raw(boxed).cast::<core::ffi::c_void>();
        // SAFETY: the handle is owned, the type name is borrowed and copied,
        // the trampoline has the audited signature, and the context is a box
        // deliberately never freed -- see `CnjLoaderRegistration`.
        let result = self.native.check(unsafe {
            (self.native.runtime.content_manager_register_cnj_loader_ext)(
                self.handle,
                string_view(type_name),
                Some(trampoline),
                context,
            )
        });
        if let Err(error) = result {
            // CNA never took the pointer, so this side still owns it.
            // SAFETY: the box was made two statements ago and handed to nobody.
            drop(unsafe { Box::from_raw(context.cast::<Box<dyn CnjLoader>>()) });
            return Err(error);
        }
        Ok(CnjLoaderRegistration {
            type_name: type_name.to_owned(),
        })
    }
}
