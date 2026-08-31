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
    fn to_native(self, native: &Arc<Native>) -> Result<sys::CNA_CnbReadLimits> {
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

    const fn is_default(self) -> bool {
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
        Ok(Self { native, handle })
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
    const fn from_native(value: sys::CNA_CnbEffectKind) -> Option<Self> {
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

    const fn to_native(self) -> sys::CNA_CnbEffectKind {
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
    const fn to_native(self) -> sys::CNA_CnbMaterialTextureSlot {
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
    native: Arc<Native>,
    handle: sys::CNA_CnbModelDataHandle,
}

impl CnbModel {
    /// Starts an empty model to author.
    pub fn new() -> Result<Self> {
        let native = Native::process()?;
        let mut handle = sys::CNA_INVALID_HANDLE;
        // SAFETY: the output is a live local receiving a newly owned handle.
        native.check(unsafe { (native.runtime.cnb_model_create)(&mut handle) })?;
        Ok(Self { native, handle })
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
    fn to_native(self) -> sys::CNA_CnbModelPartInfo {
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
