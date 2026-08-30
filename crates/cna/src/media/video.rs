use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::content::{ContentDisposable, ContentLoadable};
use crate::error::{CnaError, Result};
use crate::game::{GameContext, TimeSpan};
use crate::graphics::{BorrowedHandle, GraphicsDevice, Texture2D};

use super::{native_function, MediaRuntime, MediaState, ResourceCore, VideoSoundtrackType};

pub struct Video {
    core: Arc<ResourceCore>,
    _graphics_device: GraphicsDevice,
}

impl Video {
    pub(crate) fn from_content(
        graphics_device: &GraphicsDevice,
        file_name: &str,
        duration_milliseconds: i32,
        width: i32,
        height: i32,
        frames_per_second: f32,
        soundtrack_type: VideoSoundtrackType,
    ) -> Result<Self> {
        let runtime = MediaRuntime::process();
        let (native, _, generation) = runtime.binding()?;
        let create: sys::cna_video_create_with_metadata_fn = unsafe {
            native_function(native.media.video_create_with_metadata)
        };
        let mut handle = 0;
        // SAFETY: the device is callback-live, input text is borrowed for the call, metadata uses
        // fixed-width ABI values, and output pointer is valid.
        native.check(unsafe {
            create(
                graphics_device.handle()?,
                super::string_view(file_name),
                duration_milliseconds,
                width,
                height,
                frames_per_second,
                soundtrack_type as u32,
                &mut handle,
            )
        })?;
        let destroy: sys::cna_video_destroy_fn = unsafe {
            native_function(native.media.video_destroy)
        };
        #[cfg(feature = "native-fault-injection")]
        if let Err(error) = crate::native::fault::check("video-create-after-native") {
            // SAFETY: CNA just returned this owned Video handle and no Rust owner exists yet.
            native.check(unsafe { destroy(handle) })?;
            return Err(error);
        }
        Ok(Self {
            core: ResourceCore::new(native, runtime, generation, handle, None, destroy, None),
            _graphics_device: graphics_device.clone(),
        })
    }

    pub fn Duration(&self) -> Result<TimeSpan> {
        let f:sys::cna_video_get_duration_fn=unsafe{native_function(self.core.native().media.video_get_duration)};let mut value=0;self.core.native().check(unsafe{f(self.core.handle()?,&mut value)})?;Ok(TimeSpan::from_ticks(value))
    }
    pub fn Width(&self)->Result<i32>{let f:sys::cna_video_get_width_fn=unsafe{native_function(self.core.native().media.video_get_width)};let mut value=0;self.core.native().check(unsafe{f(self.core.handle()?,&mut value)})?;Ok(value)}
    pub fn Height(&self)->Result<i32>{let f:sys::cna_video_get_height_fn=unsafe{native_function(self.core.native().media.video_get_height)};let mut value=0;self.core.native().check(unsafe{f(self.core.handle()?,&mut value)})?;Ok(value)}
    pub fn FramesPerSecond(&self)->Result<f32>{let f:sys::cna_video_get_frames_per_second_fn=unsafe{native_function(self.core.native().media.video_get_frames_per_second)};let mut value=0.0;self.core.native().check(unsafe{f(self.core.handle()?,&mut value)})?;Ok(value)}
    pub fn VideoSoundtrackType(&self)->Result<VideoSoundtrackType>{let f:sys::cna_video_get_soundtrack_type_fn=unsafe{native_function(self.core.native().media.video_get_soundtrack_type)};let mut value=0;self.core.native().check(unsafe{f(self.core.handle()?,&mut value)})?;VideoSoundtrackType::from_native(value)}
    pub(crate) fn native_handle(&self)->Result<sys::CNA_Handle>{self.core.handle()}
    pub(crate) fn graphics_device(&self)->&GraphicsDevice{&self._graphics_device}
}

impl Drop for Video { fn drop(&mut self){self.core.invalidate();} }
impl ContentDisposable for Video { fn DisposeContent(&self)->Result<()>{self.core.Dispose()} }
impl ContentLoadable for Video { fn ContentDisposable(value:&Arc<Self>)->Option<Arc<dyn ContentDisposable>>{Some(Arc::clone(value) as Arc<_>)} }

/// The borrow a `VideoPlayer` frame `Texture2D` holds.
///
/// CNA's frame texture is valid **only until the next call on its player** --
/// any later player call, `get_texture` included, replaces it, after which the
/// handle answers `CNA_RESULT_INVALID_HANDLE`. The Rust view therefore counts
/// player calls and refuses a stale texture one call earlier, without asking
/// the player again: asking would itself be the call that invalidates it.
struct VideoFrameBorrow {
    epoch: Arc<AtomicU64>,
    issued: u64,
}

impl BorrowedHandle for VideoFrameBorrow {
    fn validate(&self) -> Result<()> {
        if self.epoch.load(Ordering::Acquire) == self.issued {
            Ok(())
        } else {
            Err(CnaError::InvalidInput(
                "the VideoPlayer frame this Texture2D borrowed was replaced by a later player call",
            ))
        }
    }
}

pub struct VideoPlayer {
    core: Arc<ResourceCore>,
    video: Mutex<Option<Arc<Video>>>,
    is_looped: Mutex<bool>,
    is_muted: Mutex<bool>,
    volume: Mutex<f32>,
    /// Incremented by every native call this player makes, which is exactly
    /// what invalidates an outstanding frame borrow.
    frame_epoch: Arc<AtomicU64>,
}

impl VideoPlayer {
    pub fn new(game:&GameContext<'_>)->Result<Self>{
        let(native,game_handle)=game.native_game();let runtime=Arc::clone(game.media_runtime());let generation=game.media_generation();let create:sys::cna_video_player_create_fn=unsafe{native_function(native.media.video_player_create)};let mut handle=0;native.check(unsafe{create(game_handle,&mut handle)})?;let dispose:sys::cna_video_player_dispose_fn=unsafe{native_function(native.media.video_player_dispose)};let destroy:sys::cna_video_player_destroy_fn=unsafe{native_function(native.media.video_player_destroy)};let is_disposed:sys::cna_video_player_get_is_disposed_fn=unsafe{native_function(native.media.video_player_get_is_disposed)};let core=ResourceCore::new(Arc::clone(native),runtime,generation,handle,Some(dispose),destroy,Some(is_disposed));let loop_fn:sys::cna_video_player_get_is_looped_fn=unsafe{native_function(native.media.video_player_get_is_looped)};let mute_fn:sys::cna_video_player_get_is_muted_fn=unsafe{native_function(native.media.video_player_get_is_muted)};let volume_fn:sys::cna_video_player_get_volume_fn=unsafe{native_function(native.media.video_player_get_volume)};let mut looped=0;let mut muted=0;let mut volume=1.0;native.check(unsafe{loop_fn(handle,&mut looped)})?;native.check(unsafe{mute_fn(handle,&mut muted)})?;native.check(unsafe{volume_fn(handle,&mut volume)})?;Ok(Self{core,video:Mutex::new(None),is_looped:Mutex::new(looped!=0),is_muted:Mutex::new(muted!=0),volume:Mutex::new(volume),frame_epoch:Arc::new(AtomicU64::new(0))})
    }
    pub fn IsDisposed(&self)->Result<bool>{self.core.IsDisposed()}
    pub fn Video(&self)->Result<Option<Arc<Video>>>{Ok(self.video.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone())}
    pub fn State(&self)->Result<MediaState>{let f:sys::cna_video_player_get_state_fn=unsafe{native_function(self.core.native().media.video_player_get_state)};let mut value=0;self.core.native().check(unsafe{f(self.player_handle()?,&mut value)})?;MediaState::from_native(value)}
    pub fn PlayPosition(&self)->Result<TimeSpan>{let f:sys::cna_video_player_get_play_position_ticks_fn=unsafe{native_function(self.core.native().media.video_player_get_play_position_ticks)};let mut value=0;self.core.native().check(unsafe{f(self.player_handle()?,&mut value)})?;Ok(TimeSpan::from_ticks(value))}
    pub fn IsLooped(&self)->Result<bool>{Ok(*self.is_looped.lock().unwrap_or_else(std::sync::PoisonError::into_inner))}
    pub fn SetIsLooped(&self,value:bool)->Result<()>{let f:sys::cna_video_player_set_is_looped_fn=unsafe{native_function(self.core.native().media.video_player_set_is_looped)};self.core.native().check(unsafe{f(self.player_handle()?,u8::from(value))})?;*self.is_looped.lock().unwrap_or_else(std::sync::PoisonError::into_inner)=value;Ok(())}
    pub fn IsMuted(&self)->Result<bool>{Ok(*self.is_muted.lock().unwrap_or_else(std::sync::PoisonError::into_inner))}
    pub fn SetIsMuted(&self,value:bool)->Result<()>{let f:sys::cna_video_player_set_is_muted_fn=unsafe{native_function(self.core.native().media.video_player_set_is_muted)};self.core.native().check(unsafe{f(self.player_handle()?,u8::from(value))})?;*self.is_muted.lock().unwrap_or_else(std::sync::PoisonError::into_inner)=value;Ok(())}
    pub fn Volume(&self)->Result<f32>{Ok(*self.volume.lock().unwrap_or_else(std::sync::PoisonError::into_inner))}
    pub fn SetVolume(&self,value:f32)->Result<()>{if value<0.0||value>1.0{return Err(CnaError::InvalidInput("VideoPlayer Volume must be in [0, 1] or NaN"));}let f:sys::cna_video_player_set_volume_fn=unsafe{native_function(self.core.native().media.video_player_set_volume)};self.core.native().check(unsafe{f(self.player_handle()?,value)})?;*self.volume.lock().unwrap_or_else(std::sync::PoisonError::into_inner)=value;Ok(())}
    pub fn Play(&self,video:Arc<Video>)->Result<()>{if video.core.generation()!=self.core.generation(){return Err(CnaError::InvalidInput("Video belongs to another Game generation"));}let f:sys::cna_video_player_play_fn=unsafe{native_function(self.core.native().media.video_player_play)};self.core.native().check(unsafe{f(self.player_handle()?,video.native_handle()?)})?;*self.video.lock().unwrap_or_else(std::sync::PoisonError::into_inner)=Some(video);Ok(())}
    pub fn Pause(&self)->Result<()>{self.operation(self.core.native().media.video_player_pause)}
    pub fn Resume(&self)->Result<()>{self.operation(self.core.native().media.video_player_resume)}
    pub fn Stop(&self)->Result<()>{self.operation(self.core.native().media.video_player_stop)}
    fn operation(&self,slot:usize)->Result<()>{let f:unsafe extern "C" fn(sys::CNA_VideoPlayerHandle)->sys::CNA_Result=unsafe{native_function(slot)};self.core.native().check(unsafe{f(self.player_handle()?)})}
    /// Returns the player's handle after invalidating any outstanding frame
    /// borrow, because that is exactly what the impending native call does.
    fn player_handle(&self) -> Result<sys::CNA_Handle> {
        let handle = self.core.handle()?;
        self.frame_epoch.fetch_add(1, Ordering::AcqRel);
        Ok(handle)
    }

    /// Reads the current frame together with the identity needed to track it.
    fn frame(&self) -> Result<sys::CNA_VideoFrameEXT> {
        let mut frame = sys::CNA_VideoFrameEXT {
            struct_size: core::mem::size_of::<sys::CNA_VideoFrameEXT>() as u32,
            struct_version: sys::CNA_VIDEO_FRAME_EXT_STRUCT_VERSION,
            ..sys::CNA_VideoFrameEXT::default()
        };
        let f: sys::cna_video_player_get_frame_ext_fn =
            unsafe { native_function(self.core.native().media.video_player_get_frame_ext) };
        // SAFETY: the handle is live, and the descriptor is a caller-owned
        // versioned output whose prefix this build declares exactly.
        self.core
            .native()
            .check(unsafe { f(self.player_handle()?, &mut frame) })?;
        Ok(frame)
    }

    /// Frames decoded since this player was created, zero before the first.
    ///
    /// XNA has no counterpart: it owns two frame textures and alternates
    /// between them, so its callers track change by object identity. CNA
    /// decodes into one texture in place and publishes this monotonic counter
    /// instead, which is never restarted by `Stop` or by playing another video.
    pub(crate) fn frame_generation(&self) -> Result<u64> {
        Ok(self.frame()?.generation)
    }

    /// Presentation timestamp of the held frame, or `None` when none exists.
    pub(crate) fn frame_presentation_time(&self) -> Result<Option<f64>> {
        let frame = self.frame()?;
        Ok((frame.available != sys::CNA_FALSE).then_some(frame.presentation_time))
    }

    pub fn GetTexture(&self) -> Result<Option<Texture2D>> {
        let video = self
            .video
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let Some(video) = video else {
            return Err(CnaError::InvalidInput("VideoPlayer has no current Video"));
        };
        let frame = self.frame()?;
        if frame.available == sys::CNA_FALSE || frame.texture == sys::CNA_INVALID_HANDLE {
            return Ok(None);
        }
        // The texture stays the player's. The Rust view never destroys it and
        // refuses every use once a later player call has replaced it.
        let borrow = Arc::new(VideoFrameBorrow {
            epoch: Arc::clone(&self.frame_epoch),
            issued: self.frame_epoch.load(Ordering::Acquire),
        });
        Texture2D::from_borrowed_handle(video.graphics_device(), frame.texture, borrow).map(Some)
    }
    pub fn Dispose(&self)->Result<()>{self.frame_epoch.fetch_add(1,Ordering::AcqRel);self.core.Dispose()}
    pub fn Finalize(&self)->Result<()> { Ok(()) }
}

impl Drop for VideoPlayer { fn drop(&mut self){self.frame_epoch.fetch_add(1,Ordering::AcqRel);self.core.invalidate();} }
