#![allow(
    non_snake_case,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

mod catalog;
mod collections;
mod library;
mod player;
mod runtime;
mod video;

use std::io::Cursor;
use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::native::Native;

pub use catalog::{Album, Artist, Genre, Picture, PictureAlbum, Playlist, Song};
pub use collections::{
    AlbumCollection, ArtistCollection, GenreCollection, PictureAlbumCollection,
    PictureCollection, PlaylistCollection, SongCollection,
};
pub use library::{MediaLibrary, MediaSource};
pub use player::{MediaPlayer, MediaQueue};
pub(crate) use runtime::MediaRuntime;
pub use video::{Video, VideoPlayer};

/// CNA media source kind. The gap between 0 and 4 is part of XNA.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum MediaSourceType {
    LocalDevice = 0,
    WindowsMediaConnect = 4,
}

impl MediaSourceType {
    fn from_native(value: u32) -> Result<Self> {
        match value {
            sys::CNA_MEDIA_SOURCE_TYPE_LOCAL_DEVICE => Ok(Self::LocalDevice),
            sys::CNA_MEDIA_SOURCE_TYPE_WINDOWS_MEDIA_CONNECT => Ok(Self::WindowsMediaConnect),
            _ => Err(CnaError::InvalidInput("native media source type is undefined")),
        }
    }
}

/// Media or video playback state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum MediaState {
    Stopped = 0,
    Playing = 1,
    Paused = 2,
}

impl MediaState {
    fn from_native(value: u32) -> Result<Self> {
        match value {
            sys::CNA_MEDIA_STATE_STOPPED => Ok(Self::Stopped),
            sys::CNA_MEDIA_STATE_PLAYING => Ok(Self::Playing),
            sys::CNA_MEDIA_STATE_PAUSED => Ok(Self::Paused),
            _ => Err(CnaError::InvalidInput("native media state is undefined")),
        }
    }
}

/// Declared audio content in a video.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum VideoSoundtrackType {
    Music = 0,
    Dialog = 1,
    MusicAndDialog = 2,
}

impl VideoSoundtrackType {
    fn from_native(value: u32) -> Result<Self> {
        match value {
            sys::CNA_VIDEO_SOUNDTRACK_TYPE_MUSIC => Ok(Self::Music),
            sys::CNA_VIDEO_SOUNDTRACK_TYPE_DIALOG => Ok(Self::Dialog),
            sys::CNA_VIDEO_SOUNDTRACK_TYPE_MUSIC_AND_DIALOG => Ok(Self::MusicAndDialog),
            _ => Err(CnaError::InvalidInput(
                "native video soundtrack type is undefined",
            )),
        }
    }
}

/// Fixed XNA visualization buffers. Both read-only views always contain 256 values.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualizationData {
    frequencies: [f32; sys::CNA_VISUALIZATION_DATA_SIZE as usize],
    samples: [f32; sys::CNA_VISUALIZATION_DATA_SIZE as usize],
}

impl Default for VisualizationData {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualizationData {
    #[must_use]
    pub fn new() -> Self {
        Self {
            frequencies: [0.0; sys::CNA_VISUALIZATION_DATA_SIZE as usize],
            samples: [0.0; sys::CNA_VISUALIZATION_DATA_SIZE as usize],
        }
    }

    #[must_use]
    pub fn Frequencies(&self) -> &[f32] {
        &self.frequencies
    }

    #[must_use]
    pub fn Samples(&self) -> &[f32] {
        &self.samples
    }

    fn update_from_native(&mut self, value: &sys::CNA_VisualizationData) {
        self.frequencies.copy_from_slice(&value.frequencies);
        self.samples.copy_from_slice(&value.samples);
    }
}

type DestroyFn = unsafe extern "C" fn(sys::CNA_Handle) -> sys::CNA_Result;
type DisposeFn = unsafe extern "C" fn(sys::CNA_Handle) -> sys::CNA_Result;
type IsDisposedFn =
    unsafe extern "C" fn(sys::CNA_Handle, *mut sys::CNA_Bool) -> sys::CNA_Result;

pub(crate) struct ResourceCore {
    native: Arc<Native>,
    runtime: Arc<MediaRuntime>,
    generation: u64,
    handle: Mutex<sys::CNA_Handle>,
    dispose: Option<DisposeFn>,
    destroy: DestroyFn,
    is_disposed: Option<IsDisposedFn>,
}

impl ResourceCore {
    pub(crate) fn new(
        native: Arc<Native>,
        runtime: Arc<MediaRuntime>,
        generation: u64,
        handle: sys::CNA_Handle,
        dispose: Option<DisposeFn>,
        destroy: DestroyFn,
        is_disposed: Option<IsDisposedFn>,
    ) -> Arc<Self> {
        let state = Arc::new(Self {
            native,
            runtime,
            generation,
            handle: Mutex::new(handle),
            dispose,
            destroy,
            is_disposed,
        });
        let resource: Arc<dyn runtime::MediaInvalidatable> = Arc::clone(&state) as Arc<_>;
        state.runtime.register_resource(&resource);
        state
    }

    pub(crate) fn handle(&self) -> Result<sys::CNA_Handle> {
        if !self.runtime.is_generation_active(self.generation) {
            return Err(CnaError::InvalidInput(
                "Media object belongs to a dead Game generation",
            ));
        }
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (handle != 0)
            .then_some(handle)
            .ok_or(CnaError::InvalidInput("Media object is disposed"))
    }

    pub(crate) fn raw_handle(&self) -> Option<sys::CNA_Handle> {
        let handle = *self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (handle != 0).then_some(handle)
    }

    pub(crate) fn native(&self) -> &Arc<Native> {
        &self.native
    }

    pub(crate) fn runtime(&self) -> &Arc<MediaRuntime> {
        &self.runtime
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn IsDisposed(&self) -> Result<bool> {
        let Some(handle) = self.raw_handle() else {
            return Ok(true);
        };
        if !self.runtime.is_generation_active(self.generation) {
            return Ok(true);
        }
        if let Some(getter) = self.is_disposed {
            let mut value = sys::CNA_FALSE;
            // SAFETY: handle is live and `value` is a valid output pointer.
            self.native.check(unsafe { getter(handle, &mut value) })?;
            Ok(value != sys::CNA_FALSE)
        } else {
            Ok(false)
        }
    }

    pub(crate) fn Dispose(&self) -> Result<()> {
        let mut slot = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = *slot;
        if handle == 0 {
            return Ok(());
        }
        if !self.runtime.is_generation_active(self.generation) {
            *slot = 0;
            return Ok(());
        }
        if let Some(dispose) = self.dispose {
            // SAFETY: the stored function is the matching dispose route for this owned handle.
            self.native.check(unsafe { dispose(handle) })?;
        }
        // SAFETY: the stored function is the matching destroy route and this call consumes the
        // binding's one native handle identity.
        self.native.check(unsafe { (self.destroy)(handle) })?;
        *slot = 0;
        Ok(())
    }

    pub(crate) fn invalidate(&self) {
        let mut slot = self
            .handle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = *slot;
        *slot = 0;
        drop(slot);
        if handle != 0 {
            self.runtime
                .release_or_defer(&self.native, self.destroy, handle);
        }
    }
}

impl runtime::MediaInvalidatable for ResourceCore {
    fn invalidate_for_game_shutdown(&self) {
        self.invalidate();
    }
}

impl Drop for ResourceCore {
    fn drop(&mut self) {
        self.invalidate();
    }
}

pub(crate) fn read_i32(
    core: &ResourceCore,
    getter: unsafe extern "C" fn(sys::CNA_Handle, *mut i32) -> sys::CNA_Result,
) -> Result<i32> {
    let mut value = 0;
    // SAFETY: the handle is validated and output pointer lives through the call.
    core.native.check(unsafe { getter(core.handle()?, &mut value) })?;
    Ok(value)
}

pub(crate) fn read_i64(
    core: &ResourceCore,
    getter: unsafe extern "C" fn(sys::CNA_Handle, *mut i64) -> sys::CNA_Result,
) -> Result<i64> {
    let mut value = 0;
    // SAFETY: the handle is validated and output pointer lives through the call.
    core.native.check(unsafe { getter(core.handle()?, &mut value) })?;
    Ok(value)
}

pub(crate) fn read_bool(
    core: &ResourceCore,
    getter: unsafe extern "C" fn(sys::CNA_Handle, *mut sys::CNA_Bool) -> sys::CNA_Result,
) -> Result<bool> {
    let mut value = sys::CNA_FALSE;
    // SAFETY: the handle is validated and output pointer lives through the call.
    core.native.check(unsafe { getter(core.handle()?, &mut value) })?;
    Ok(value != sys::CNA_FALSE)
}

pub(crate) fn read_string(
    core: &ResourceCore,
    size: unsafe extern "C" fn(sys::CNA_Handle, *mut u64) -> sys::CNA_Result,
    copy: unsafe extern "C" fn(
        sys::CNA_Handle,
        *mut core::ffi::c_char,
        u64,
        *mut u64,
    ) -> sys::CNA_Result,
) -> Result<String> {
    let handle = core.handle()?;
    let mut required = 0;
    // SAFETY: output pointer is valid.
    core.native.check(unsafe { size(handle, &mut required) })?;
    let capacity = usize::try_from(required)
        .map_err(|_| CnaError::InvalidInput("native media string is too large"))?;
    let mut bytes = vec![0_u8; capacity];
    let mut copied = 0;
    // SAFETY: destination has `required` writable bytes and remains live for the call.
    core.native.check(unsafe {
        copy(handle, bytes.as_mut_ptr().cast(), required, &mut copied)
    })?;
    String::from_utf8(bytes).map_err(|_| CnaError::InvalidInput("native media string is not UTF-8"))
}

pub(crate) fn read_blob(
    core: &ResourceCore,
    size: unsafe extern "C" fn(sys::CNA_Handle, *mut u64) -> sys::CNA_Result,
    copy: unsafe extern "C" fn(sys::CNA_Handle, *mut u8, u64, *mut u64) -> sys::CNA_Result,
) -> Result<Box<dyn std::io::Read + Send>> {
    let handle = core.handle()?;
    let mut required = 0;
    // SAFETY: output pointer is valid.
    core.native.check(unsafe { size(handle, &mut required) })?;
    let capacity = usize::try_from(required)
        .map_err(|_| CnaError::InvalidInput("native media blob is too large"))?;
    let mut bytes = vec![0_u8; capacity];
    let mut copied = 0;
    // SAFETY: destination has `required` writable bytes and remains live for the call.
    core.native.check(unsafe {
        copy(handle, bytes.as_mut_ptr(), required, &mut copied)
    })?;
    bytes.truncate(usize::try_from(copied).unwrap_or(bytes.len()).min(bytes.len()));
    Ok(Box::new(Cursor::new(bytes)))
}

pub(crate) fn string_view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast(),
        byte_length: value.len() as u64,
    }
}
