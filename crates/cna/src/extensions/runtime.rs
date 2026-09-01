//! CNA's process-level runtime identity: platform, renderer and backend.
//!
//! XNA 4.0 had one renderer and no notion of choosing, so this whole family is
//! a CNA concept and lives outside `cna::Microsoft::Xna::Framework`.
//!
//! Every route here is process-global upstream and deliberately so: the
//! renderer choice must be made before the first graphics device exists, which
//! is before a `Game` has anywhere natural to keep it. Nothing in this module
//! takes a `Game`, and nothing in it fabricates an answer CNA declined to give.

use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::native::runtime::read_string;
use crate::native::Native;

/// One graphics renderer identity CNA knows about.
///
/// This is a value rather than a Rust `enum` on purpose. CNA's identity set is
/// versioned -- ABI 0.20.0 retired eleven identities and moved the ceiling from
/// 50 to 49 -- and the retired numbers are never reused. A newtype keeps an
/// identity from a newer CNA representable and inspectable instead of turning
/// it into a panic or a lossy `Unknown`, and [`RendererType::name`] asks CNA
/// for the spelling rather than carrying a table that can drift.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RendererType(u32);

impl RendererType {
    pub const UNKNOWN: Self = Self(0);
    pub const SDL_RENDERER: Self = Self(1);
    pub const OPENGLES2: Self = Self(2);
    pub const OPENGLES3: Self = Self(3);
    pub const OPENGL33: Self = Self(4);
    pub const WEBGL1: Self = Self(5);
    pub const WEBGL2: Self = Self(6);
    pub const BGFX: Self = Self(7);
    pub const VULKAN: Self = Self(8);
    pub const WEBGPU: Self = Self(9);
    pub const HEADLESS: Self = Self(11);
    pub const SOFTWARE: Self = Self(12);
    pub const STUB: Self = Self(13);
    pub const DIRECTX11: Self = Self(14);
    pub const DIRECTX12: Self = Self(15);
    pub const DIRECT2D: Self = Self(16);
    pub const CANVAS: Self = Self(17);
    pub const HTML_DOM: Self = Self(18);
    pub const FREEDIRECT: Self = Self(21);
    pub const DIRECTX9: Self = Self(22);
    pub const DIRECTX1: Self = Self(23);
    pub const DIRECTX2: Self = Self(24);
    pub const DIRECTX3: Self = Self(25);
    pub const DIRECTX5: Self = Self(26);
    pub const DIRECTX6: Self = Self(27);
    pub const DIRECTX7: Self = Self(28);
    pub const DIRECTX8: Self = Self(29);
    pub const DIRECTX10: Self = Self(30);
    pub const SDL_GPU: Self = Self(31);
    pub const OPENGLES1: Self = Self(32);
    pub const OPENGL4: Self = Self(33);
    pub const OPENGL1: Self = Self(34);
    pub const OPENGL2: Self = Self(35);
    pub const GLIDE: Self = Self(39);
    pub const GDI: Self = Self(40);
    pub const METAL: Self = Self(42);
    pub const FNA3D: Self = Self(43);
    pub const SVG_DOM: Self = Self(44);
    pub const PORTABLEGL: Self = Self(46);
    pub const PIXIJS: Self = Self(49);

    /// The identity's canonical numeric value.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Wraps a canonical numeric identity, including one this build predates.
    #[must_use]
    pub const fn from_value(value: u32) -> Self {
        Self(value)
    }

    /// Resolves a `CNA_GRAPHICS_RENDERER` spelling, matched case-insensitively.
    ///
    /// Returns `None` when CNA does not recognize the name, which is an answer
    /// rather than a failure.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports, or a loader error when no CNA
    /// library is available.
    pub fn parse(name: &str) -> Result<Option<Self>> {
        let native = Native::process()?;
        let view = string_view(name);
        let mut value = 0;
        let mut recognized = sys::CNA_FALSE;
        // SAFETY: `view` borrows `name` for the duration of the call and the
        // outputs are live locals of the declared types.
        check(&native, unsafe {
            (native.runtime.renderer_try_parse_name)(view, &mut value, &mut recognized)
        })?;
        Ok((recognized != sys::CNA_FALSE).then_some(Self(value)))
    }
}

/// The host platform family CNA was built for.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Platform {
    Desktop,
    Android,
    Ios,
    Web,
    /// An identity a newer CNA introduced, carried rather than discarded.
    Other(u32),
}

/// The desktop operating system CNA reports.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DesktopOperatingSystem {
    Windows,
    Linux,
    MacOsX,
    Other,
    /// An identity a newer CNA introduced.
    Unrecognized(u32),
}

/// What kind of graphics stack a renderer sits on.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum BackendCategory {
    Native,
    TranslationLayer,
    Software,
    Web,
    Diagnostic,
    /// A category a newer CNA introduced.
    Unrecognized(u32),
}

/// How far a renderer has been taken.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum BackendMaturity {
    Production,
    Supported,
    Experimental,
    Historical,
    Deprecated,
    /// A maturity a newer CNA introduced.
    Unrecognized(u32),
}

/// Why CNA passed over a renderer it tried.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum RendererFallbackReason {
    NotCompiledIn,
    ProbeUnavailable,
    InitializationFailed,
    WindowKindConflict,
    /// A reason a newer CNA introduced.
    Unrecognized(u32),
}

/// One renderer CNA tried and passed over, with CNA's own explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RendererFallback {
    /// The identity that was tried.
    pub renderer: RendererType,
    /// Why it was passed over.
    pub reason: RendererFallbackReason,
    /// CNA's message for this record; empty when it published none.
    pub message: String,
}

fn check(native: &Arc<Native>, result: sys::CNA_Result) -> Result<()> {
    native.check(result)
}

fn string_view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast::<core::ffi::c_char>(),
        byte_length: value.len() as u64,
    }
}

impl Platform {
    const fn from_native(value: sys::CNA_Platform) -> Self {
        match value {
            sys::CNA_PLATFORM_DESKTOP => Self::Desktop,
            sys::CNA_PLATFORM_ANDROID => Self::Android,
            sys::CNA_PLATFORM_IOS => Self::Ios,
            sys::CNA_PLATFORM_WEB => Self::Web,
            other => Self::Other(other),
        }
    }
}

impl DesktopOperatingSystem {
    const fn from_native(value: sys::CNA_DesktopOS) -> Self {
        match value {
            sys::CNA_DESKTOP_OS_WINDOWS => Self::Windows,
            sys::CNA_DESKTOP_OS_LINUX => Self::Linux,
            sys::CNA_DESKTOP_OS_MACOSX => Self::MacOsX,
            sys::CNA_DESKTOP_OS_OTHER => Self::Other,
            other => Self::Unrecognized(other),
        }
    }
}

impl BackendCategory {
    const fn from_native(value: sys::CNA_GraphicsBackendCategory) -> Self {
        match value {
            sys::CNA_GRAPHICS_BACKEND_CATEGORY_NATIVE => Self::Native,
            sys::CNA_GRAPHICS_BACKEND_CATEGORY_TRANSLATION_LAYER => Self::TranslationLayer,
            sys::CNA_GRAPHICS_BACKEND_CATEGORY_SOFTWARE => Self::Software,
            sys::CNA_GRAPHICS_BACKEND_CATEGORY_WEB => Self::Web,
            sys::CNA_GRAPHICS_BACKEND_CATEGORY_DIAGNOSTIC => Self::Diagnostic,
            other => Self::Unrecognized(other),
        }
    }

    const fn as_native(self) -> sys::CNA_GraphicsBackendCategory {
        match self {
            Self::Native => sys::CNA_GRAPHICS_BACKEND_CATEGORY_NATIVE,
            Self::TranslationLayer => sys::CNA_GRAPHICS_BACKEND_CATEGORY_TRANSLATION_LAYER,
            Self::Software => sys::CNA_GRAPHICS_BACKEND_CATEGORY_SOFTWARE,
            Self::Web => sys::CNA_GRAPHICS_BACKEND_CATEGORY_WEB,
            Self::Diagnostic => sys::CNA_GRAPHICS_BACKEND_CATEGORY_DIAGNOSTIC,
            Self::Unrecognized(value) => value,
        }
    }
}

impl BackendMaturity {
    const fn from_native(value: sys::CNA_GraphicsBackendMaturity) -> Self {
        match value {
            sys::CNA_GRAPHICS_BACKEND_MATURITY_PRODUCTION => Self::Production,
            sys::CNA_GRAPHICS_BACKEND_MATURITY_SUPPORTED => Self::Supported,
            sys::CNA_GRAPHICS_BACKEND_MATURITY_EXPERIMENTAL => Self::Experimental,
            sys::CNA_GRAPHICS_BACKEND_MATURITY_HISTORICAL => Self::Historical,
            sys::CNA_GRAPHICS_BACKEND_MATURITY_DEPRECATED => Self::Deprecated,
            other => Self::Unrecognized(other),
        }
    }

    const fn as_native(self) -> sys::CNA_GraphicsBackendMaturity {
        match self {
            Self::Production => sys::CNA_GRAPHICS_BACKEND_MATURITY_PRODUCTION,
            Self::Supported => sys::CNA_GRAPHICS_BACKEND_MATURITY_SUPPORTED,
            Self::Experimental => sys::CNA_GRAPHICS_BACKEND_MATURITY_EXPERIMENTAL,
            Self::Historical => sys::CNA_GRAPHICS_BACKEND_MATURITY_HISTORICAL,
            Self::Deprecated => sys::CNA_GRAPHICS_BACKEND_MATURITY_DEPRECATED,
            Self::Unrecognized(value) => value,
        }
    }
}

impl RendererFallbackReason {
    const fn from_native(value: sys::CNA_GraphicsRendererFallbackReason) -> Self {
        match value {
            sys::CNA_GRAPHICS_RENDERER_FALLBACK_NOT_COMPILED_IN => Self::NotCompiledIn,
            sys::CNA_GRAPHICS_RENDERER_FALLBACK_PROBE_UNAVAILABLE => Self::ProbeUnavailable,
            sys::CNA_GRAPHICS_RENDERER_FALLBACK_INITIALIZATION_FAILED => Self::InitializationFailed,
            sys::CNA_GRAPHICS_RENDERER_FALLBACK_WINDOW_KIND_CONFLICT => Self::WindowKindConflict,
            other => Self::Unrecognized(other),
        }
    }

    const fn as_native(self) -> sys::CNA_GraphicsRendererFallbackReason {
        match self {
            Self::NotCompiledIn => sys::CNA_GRAPHICS_RENDERER_FALLBACK_NOT_COMPILED_IN,
            Self::ProbeUnavailable => sys::CNA_GRAPHICS_RENDERER_FALLBACK_PROBE_UNAVAILABLE,
            Self::InitializationFailed => sys::CNA_GRAPHICS_RENDERER_FALLBACK_INITIALIZATION_FAILED,
            Self::WindowKindConflict => sys::CNA_GRAPHICS_RENDERER_FALLBACK_WINDOW_KIND_CONFLICT,
            Self::Unrecognized(value) => value,
        }
    }
}

/// The platform family CNA was built for.
///
/// # Errors
///
/// Returns the exact error CNA reports, or a loader error when no CNA library
/// is available.
pub fn platform() -> Result<Platform> {
    let native = Native::process()?;
    let mut value = 0;
    // SAFETY: the output is a live local of the declared type.
    check(&native, unsafe {
        (native.runtime.platform_get_current)(&mut value)
    })?;
    Ok(Platform::from_native(value))
}

/// CNA's own name for the current platform.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn platform_name() -> Result<String> {
    let native = Native::process()?;
    let api = &native.runtime;
    read_string(
        |result| native.check(result),
        // SAFETY: both outputs are live locals; the two routes form CNA's
        // canonical size-then-copy pair for one UTF-8 string.
        |bytes| unsafe { (api.platform_get_name_size)(bytes) },
        |destination, capacity, written| unsafe {
            (api.platform_copy_name)(destination, capacity, written)
        },
    )
}

/// Whether CNA considers the current platform an Apple one.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn platform_is_apple() -> Result<bool> {
    let native = Native::process()?;
    let mut value = sys::CNA_FALSE;
    // SAFETY: the output is a live local of the declared type.
    check(&native, unsafe {
        (native.runtime.platform_get_is_apple)(&mut value)
    })?;
    Ok(value != sys::CNA_FALSE)
}

/// Whether CNA considers the current platform a mobile one.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn platform_is_mobile() -> Result<bool> {
    let native = Native::process()?;
    let mut value = sys::CNA_FALSE;
    // SAFETY: the output is a live local of the declared type.
    check(&native, unsafe {
        (native.runtime.platform_get_is_mobile)(&mut value)
    })?;
    Ok(value != sys::CNA_FALSE)
}

/// The desktop operating system CNA reports.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn desktop_operating_system() -> Result<DesktopOperatingSystem> {
    let native = Native::process()?;
    let mut value = 0;
    // SAFETY: the output is a live local of the declared type.
    check(&native, unsafe {
        (native.runtime.desktop_os_get_current)(&mut value)
    })?;
    Ok(DesktopOperatingSystem::from_native(value))
}

impl RendererType {
    /// CNA's canonical spelling of this identity.
    ///
    /// Only the running renderer's name has a canonical route, so this is
    /// available for the current renderer and reported as
    /// [`CnaError::UnsupportedRuntime`] for any other identity rather than
    /// answered from a table this crate would have to keep in step with CNA.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn name(self) -> Result<String> {
        if self != current_renderer()? {
            return Err(CnaError::UnsupportedRuntime(
                "CNA publishes a renderer name only for the running renderer",
            ));
        }
        let native = Native::process()?;
        let api = &native.runtime;
        read_string(
            |result| native.check(result),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.renderer_current_name_size)(bytes) },
            |destination, capacity, written| unsafe {
                (api.renderer_copy_current_name)(destination, capacity, written)
            },
        )
    }

    /// The kind of graphics stack this identity sits on.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn category(self) -> Result<BackendCategory> {
        let native = Native::process()?;
        let mut value = 0;
        // SAFETY: the output is a live local of the declared type.
        check(&native, unsafe {
            (native.runtime.backend_get_category)(self.0, &mut value)
        })?;
        Ok(BackendCategory::from_native(value))
    }

    /// How far this identity has been taken.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn maturity(self) -> Result<BackendMaturity> {
        let native = Native::process()?;
        let mut value = 0;
        // SAFETY: the output is a live local of the declared type.
        check(&native, unsafe {
            (native.runtime.backend_get_maturity)(self.0, &mut value)
        })?;
        Ok(BackendMaturity::from_native(value))
    }

    /// Whether this identity is compiled into the loaded CNA library.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn is_available(self) -> Result<bool> {
        let native = Native::process()?;
        let mut value = sys::CNA_FALSE;
        // SAFETY: the output is a live local of the declared type.
        check(&native, unsafe {
            (native.runtime.renderer_get_is_available)(self.0, &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }
}

impl BackendCategory {
    /// CNA's canonical name for this category.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn name(self) -> Result<String> {
        let native = Native::process()?;
        let api = &native.runtime;
        let value = self.as_native();
        read_string(
            |result| native.check(result),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.backend_category_name_size)(value, bytes) },
            |destination, capacity, written| unsafe {
                (api.backend_category_copy_name)(value, destination, capacity, written)
            },
        )
    }
}

impl BackendMaturity {
    /// CNA's canonical name for this maturity.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn name(self) -> Result<String> {
        let native = Native::process()?;
        let api = &native.runtime;
        let value = self.as_native();
        read_string(
            |result| native.check(result),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.backend_maturity_name_size)(value, bytes) },
            |destination, capacity, written| unsafe {
                (api.backend_maturity_copy_name)(value, destination, capacity, written)
            },
        )
    }
}

impl RendererFallbackReason {
    /// CNA's canonical name for this reason.
    ///
    /// # Errors
    ///
    /// Returns the exact error CNA reports.
    pub fn name(self) -> Result<String> {
        let native = Native::process()?;
        let api = &native.runtime;
        let value = self.as_native();
        read_string(
            |result| native.check(result),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.renderer_fallback_reason_name_size)(value, bytes) },
            |destination, capacity, written| unsafe {
                (api.renderer_fallback_reason_copy_name)(value, destination, capacity, written)
            },
        )
    }
}

/// The renderer identity that is actually running.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn current_renderer() -> Result<RendererType> {
    let native = Native::process()?;
    let mut value = 0;
    // SAFETY: the output is a live local of the declared type.
    check(&native, unsafe {
        (native.runtime.renderer_get_current_type)(&mut value)
    })?;
    Ok(RendererType(value))
}

/// The kind of graphics stack the running renderer sits on.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn current_backend_category() -> Result<BackendCategory> {
    let native = Native::process()?;
    let mut value = 0;
    // SAFETY: the output is a live local of the declared type.
    check(&native, unsafe {
        (native.runtime.backend_get_current_category)(&mut value)
    })?;
    Ok(BackendCategory::from_native(value))
}

/// How far the running renderer has been taken.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn current_backend_maturity() -> Result<BackendMaturity> {
    let native = Native::process()?;
    let mut value = 0;
    // SAFETY: the output is a live local of the declared type.
    check(&native, unsafe {
        (native.runtime.backend_get_current_maturity)(&mut value)
    })?;
    Ok(BackendMaturity::from_native(value))
}

/// Every renderer identity compiled into the loaded CNA library.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn available_renderers() -> Result<Vec<RendererType>> {
    let native = Native::process()?;
    let mut count = 0_u64;
    // SAFETY: the output is a live local of the declared type.
    check(&native, unsafe {
        (native.runtime.renderer_available_count)(&mut count)
    })?;
    let capacity = usize::try_from(count)
        .map_err(|_| CnaError::InvalidInput("CNA reported more renderers than fit in memory"))?;
    let mut values = vec![0_u32; capacity];
    let mut written = 0_u64;
    // SAFETY: `values` has exactly `count` elements of the declared type and
    // outlives the call; `written` is a live local.
    check(&native, unsafe {
        (native.runtime.renderer_copy_available)(values.as_mut_ptr(), count, &mut written)
    })?;
    let written = usize::try_from(written)
        .map_err(|_| CnaError::InvalidInput("CNA reported more renderers than fit in memory"))?;
    values.truncate(written.min(capacity));
    Ok(values.into_iter().map(RendererType).collect())
}

/// Requests the renderer CNA should attempt first.
///
/// The choice is process-wide and can only be made before CNA creates its
/// first renderer; see [`renderer_selection_is_latched`].
///
/// # Errors
///
/// Returns the exact error CNA reports, including `INVALID_STATE` once the
/// selection is latched.
pub fn set_preferred_renderer(renderer: RendererType) -> Result<()> {
    let native = Native::process()?;
    // SAFETY: the identity is a plain fixed-width value.
    check(&native, unsafe {
        (native.runtime.renderer_set_preferred)(renderer.0)
    })
}

/// Requests the renderer by its `CNA_GRAPHICS_RENDERER` spelling.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn set_preferred_renderer_by_name(name: &str) -> Result<()> {
    let native = Native::process()?;
    let view = string_view(name);
    // SAFETY: `view` borrows `name` for the duration of the call.
    check(&native, unsafe {
        (native.runtime.renderer_set_preferred_by_name)(view)
    })
}

/// The renderer CNA will attempt first.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn selected_renderer() -> Result<RendererType> {
    let native = Native::process()?;
    let mut value = 0;
    // SAFETY: the output is a live local of the declared type.
    check(&native, unsafe {
        (native.runtime.renderer_get_selected)(&mut value)
    })?;
    Ok(RendererType(value))
}

/// The renderer that was actually created.
///
/// It equals [`selected_renderer`] unless a fallback chain substituted another.
/// CNA refuses this before anything has been created rather than guessing, so
/// check [`renderer_selection_is_latched`] first.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn active_renderer() -> Result<RendererType> {
    let native = Native::process()?;
    let mut value = 0;
    // SAFETY: the output is a live local of the declared type.
    check(&native, unsafe {
        (native.runtime.renderer_get_active)(&mut value)
    })?;
    Ok(RendererType(value))
}

/// Whether CNA has created a renderer, after which the selection cannot change.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn renderer_selection_is_latched() -> Result<bool> {
    let native = Native::process()?;
    let mut value = sys::CNA_FALSE;
    // SAFETY: the output is a live local of the declared type.
    check(&native, unsafe {
        (native.runtime.renderer_get_is_latched)(&mut value)
    })?;
    Ok(value != sys::CNA_FALSE)
}

/// Sets the ordered list of renderers CNA may fall back to.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn set_renderer_fallback_chain(chain: &[RendererType]) -> Result<()> {
    let native = Native::process()?;
    let values = chain.iter().map(|value| value.0).collect::<Vec<_>>();
    // SAFETY: `values` holds exactly `len` elements of the declared type and
    // outlives the call; CNA copies what it needs.
    check(&native, unsafe {
        (native.runtime.renderer_set_fallback_chain)(values.as_ptr(), values.len() as u64)
    })
}

/// Enables or disables CNA's automatic fallback.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn set_automatic_renderer_fallback(enabled: bool) -> Result<()> {
    let native = Native::process()?;
    let value = if enabled { sys::CNA_TRUE } else { sys::CNA_FALSE };
    // SAFETY: the flag is one of the two canonical CNA_Bool values.
    check(&native, unsafe {
        (native.runtime.renderer_set_automatic_fallback)(value)
    })
}

/// Whether CNA's automatic fallback is enabled.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn automatic_renderer_fallback() -> Result<bool> {
    let native = Native::process()?;
    let mut value = sys::CNA_FALSE;
    // SAFETY: the output is a live local of the declared type.
    check(&native, unsafe {
        (native.runtime.renderer_get_automatic_fallback)(&mut value)
    })?;
    Ok(value != sys::CNA_FALSE)
}

/// Every renderer CNA tried and passed over, in the order it tried them.
///
/// The list is empty on a build where the first choice worked.
///
/// # Errors
///
/// Returns the exact error CNA reports.
pub fn renderer_fallbacks() -> Result<Vec<RendererFallback>> {
    let native = Native::process()?;
    let api = &native.runtime;
    let mut count = 0_u64;
    // SAFETY: the output is a live local of the declared type.
    check(&native, unsafe { (api.renderer_fallback_count)(&mut count) })?;
    let mut result = Vec::new();
    for index in 0..count {
        let mut record = sys::CNA_GraphicsRendererFallbackRecord {
            struct_size: core::mem::size_of::<sys::CNA_GraphicsRendererFallbackRecord>() as u32,
            struct_version: sys::CNA_GRAPHICS_RENDERER_FALLBACK_RECORD_STRUCT_VERSION,
            ..sys::CNA_GraphicsRendererFallbackRecord::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output whose
        // prefix this build declares exactly.
        check(&native, unsafe {
            (api.renderer_fallback_at)(index, &mut record)
        })?;
        let message = read_string(
            |value| native.check(value),
            // SAFETY: both outputs are live locals; the two routes form CNA's
            // canonical size-then-copy pair for one UTF-8 string.
            |bytes| unsafe { (api.renderer_fallback_message_size)(index, bytes) },
            |destination, capacity, written| unsafe {
                (api.renderer_fallback_copy_message)(index, destination, capacity, written)
            },
        )?;
        result.push(RendererFallback {
            renderer: RendererType(record.r#type),
            reason: RendererFallbackReason::from_native(record.reason),
            message,
        });
    }
    Ok(result)
}

/// The title CNA reports for the running assembly.
///
/// XNA reads it from the assembly's own metadata. A Rust binary has none, so
/// without [`set_assembly_title`] everything derived from it -- the default
/// window caption, the storage directory name -- falls back to whatever CNA
/// guesses from the executable's file name.
pub fn assembly_title() -> Result<String> {
    Native::process()?.assembly_title()
}

/// Sets the title CNA reports for the running assembly.
///
/// Process-wide, and best set before the first `Game` is created: anything
/// already derived from the old title keeps it.
pub fn set_assembly_title(title: &str) -> Result<()> {
    Native::process()?.set_assembly_title(title)
}

/// Clears the process-wide renderer choice so the next device re-selects.
///
/// The selection is made once and cached for the life of the process, which is
/// right for a game and wrong for a test that wants to exercise more than one
/// renderer in the same binary. This is the only way to undo it, and it is why
/// [`current_renderer`] can answer differently twice in one process.
pub fn reset_renderer_selection_for_tests() -> Result<()> {
    Native::process()?.reset_renderer_selection_for_tests()
}
