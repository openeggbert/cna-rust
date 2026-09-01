use std::sync::{Arc, Mutex, OnceLock, Weak};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::extensions::events::EventHandler;
use crate::game::{GameContext, TimeSpan};
use crate::extensions::media::MediaQueueExt;

use super::{MediaRuntime, MediaState, ResourceCore, Song, SongCollection, VisualizationData};

static QUEUE_CACHE: OnceLock<Mutex<Option<(u64, Weak<MediaQueue>)>>> = OnceLock::new();

fn queue_cache() -> &'static Mutex<Option<(u64, Weak<MediaQueue>)>> {
    QUEUE_CACHE.get_or_init(|| Mutex::new(None))
}

/// Static/process-global XNA MediaPlayer facade. It has no public constructor.
pub struct MediaPlayer;

impl MediaPlayer {
    fn binding(game: &GameContext<'_>) -> Result<(Arc<crate::native::Native>, sys::CNA_Handle, u64)> {
        let (runtime_native, runtime_game, generation) = game.media_runtime().binding()?;
        let (native, game_handle) = game.native_game();
        if !Arc::ptr_eq(native, &runtime_native) || game_handle != runtime_game || generation != game.media_generation() {
            return Err(CnaError::InvalidInput("GameContext is not the active Media generation"));
        }
        Ok((Arc::clone(native), game_handle, generation))
    }

    fn read_bool(game:&GameContext<'_>, route:unsafe extern "C" fn(sys::CNA_Handle,*mut sys::CNA_Bool)->sys::CNA_Result)->Result<bool>{let (native,handle,_)=Self::binding(game)?;let f=route;let mut value=0;native.check(unsafe{f(handle,&mut value)})?;Ok(value!=0)}
    fn write_bool(game:&GameContext<'_>,route:unsafe extern "C" fn(sys::CNA_Handle,sys::CNA_Bool)->sys::CNA_Result,value:bool)->Result<()>{let(native,handle,_)=Self::binding(game)?;let f=route;native.check(unsafe{f(handle,u8::from(value))})}
    fn invoke(game:&GameContext<'_>,route:unsafe extern "C" fn(sys::CNA_Handle)->sys::CNA_Result)->Result<()>{let(native,handle,_)=Self::binding(game)?;let f=route;native.check(unsafe{f(handle)})}

    pub fn IsShuffled(game:&GameContext<'_>)->Result<bool>{Self::read_bool(game,game.native_game().0.media.media_player_get_is_shuffled)}
    pub fn SetIsShuffled(game:&GameContext<'_>,value:bool)->Result<()>{Self::write_bool(game,game.native_game().0.media.media_player_set_is_shuffled,value)}
    pub fn IsRepeating(game:&GameContext<'_>)->Result<bool>{Self::read_bool(game,game.native_game().0.media.media_player_get_is_repeating)}
    pub fn SetIsRepeating(game:&GameContext<'_>,value:bool)->Result<()>{Self::write_bool(game,game.native_game().0.media.media_player_set_is_repeating,value)}
    pub fn IsMuted(game:&GameContext<'_>)->Result<bool>{Self::read_bool(game,game.native_game().0.media.media_player_get_is_muted)}
    pub fn SetIsMuted(game:&GameContext<'_>,value:bool)->Result<()>{Self::write_bool(game,game.native_game().0.media.media_player_set_is_muted,value)}
    pub fn IsVisualizationEnabled(game:&GameContext<'_>)->Result<bool>{Self::read_bool(game,game.native_game().0.media.media_player_get_is_visualization_enabled)}
    pub fn SetIsVisualizationEnabled(game:&GameContext<'_>,value:bool)->Result<()>{Self::write_bool(game,game.native_game().0.media.media_player_set_is_visualization_enabled,value)}
    pub fn GameHasControl(game:&GameContext<'_>)->Result<bool>{Self::read_bool(game,game.native_game().0.media.media_player_get_game_has_control)}

    pub fn State(game:&GameContext<'_>)->Result<MediaState>{let(native,handle,_)=Self::binding(game)?;let f:sys::cna_media_player_get_state_fn=native.media.media_player_get_state;let mut value=0;native.check(unsafe{f(handle,&mut value)})?;MediaState::from_native(value)}
    pub fn PlayPosition(game:&GameContext<'_>)->Result<TimeSpan>{let(native,handle,_)=Self::binding(game)?;let f:sys::cna_media_player_get_play_position_ticks_fn=native.media.media_player_get_play_position_ticks;let mut value=0;native.check(unsafe{f(handle,&mut value)})?;Ok(TimeSpan::from_ticks(value))}
    pub fn Volume(game:&GameContext<'_>)->Result<f32>{let(native,handle,_)=Self::binding(game)?;let f:sys::cna_media_player_get_volume_fn=native.media.media_player_get_volume;let mut value=0.0;native.check(unsafe{f(handle,&mut value)})?;Ok(value)}
    pub fn SetVolume(game:&GameContext<'_>,value:f32)->Result<()>{let(native,handle,_)=Self::binding(game)?;let f:sys::cna_media_player_set_volume_fn=native.media.media_player_set_volume;let value=if value<0.0{0.0}else if value>1.0{1.0}else{value};native.check(unsafe{f(handle,value)})}

    pub fn Queue(game:&GameContext<'_>)->Result<Arc<MediaQueue>>{
        let(native,handle,generation)=Self::binding(game)?;
        let mut cache=queue_cache().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((cached_generation,weak))=cache.as_ref(){if *cached_generation==generation{if let Some(queue)=weak.upgrade(){return Ok(queue);}}}
        let f:sys::cna_media_player_get_queue_fn=native.media.media_player_get_queue;let mut queue_handle=0;native.check(unsafe{f(handle,&mut queue_handle)})?;let queue=MediaQueue::from_handle(Arc::clone(&native),Arc::clone(game.media_runtime()),generation,queue_handle);*cache=Some((generation,Arc::downgrade(&queue)));Ok(queue)
    }

    fn invalidate_queue_songs_for_generation(generation: u64) {
        let cache = queue_cache()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((cached_generation, queue)) = cache.as_ref() {
            if *cached_generation == generation {
                if let Some(queue) = queue.upgrade() {
                    queue.invalidate_songs();
                }
            }
        }
    }

    pub fn Play(game:&GameContext<'_>,song:&Song)->Result<()>{let(native,handle,generation)=Self::binding(game)?;if song.core.generation()!=generation{return Err(CnaError::InvalidInput("Song belongs to another Game generation"));}Self::invalidate_queue_songs_for_generation(generation);let f:sys::cna_media_player_play_song_fn=native.media.media_player_play_song;native.check(unsafe{f(handle,song.native_handle()?)})}
    pub fn PlayWithSongs(game:&GameContext<'_>,songs:&SongCollection)->Result<()>{let(native,handle,generation)=Self::binding(game)?;Self::invalidate_queue_songs_for_generation(generation);let f:sys::cna_media_player_play_songs_fn=native.media.media_player_play_songs;native.check(unsafe{f(handle,songs.native_handle()?)})}
    pub fn PlayWithSongsAndIndex(game:&GameContext<'_>,songs:&SongCollection,index:i32)->Result<()>{let(native,handle,generation)=Self::binding(game)?;Self::invalidate_queue_songs_for_generation(generation);let f:sys::cna_media_player_play_songs_from_fn=native.media.media_player_play_songs_from;native.check(unsafe{f(handle,songs.native_handle()?,index)})}
    pub fn Pause(game:&GameContext<'_>)->Result<()>{Self::invoke(game,game.native_game().0.media.media_player_pause)}
    pub fn Resume(game:&GameContext<'_>)->Result<()>{Self::invoke(game,game.native_game().0.media.media_player_resume)}
    pub fn Stop(game:&GameContext<'_>)->Result<()>{Self::invoke(game,game.native_game().0.media.media_player_stop)}
    pub fn MoveNext(game:&GameContext<'_>)->Result<()>{Self::invoke(game,game.native_game().0.media.media_player_move_next)}
    pub fn MovePrevious(game:&GameContext<'_>)->Result<()>{Self::invoke(game,game.native_game().0.media.media_player_move_previous)}
    pub fn GetVisualizationData(game:&GameContext<'_>,visualizationData:&mut VisualizationData)->Result<()>{let(native,handle,_)=Self::binding(game)?;let f:sys::cna_media_player_get_visualization_data_fn=native.media.media_player_get_visualization_data;let mut value=sys::CNA_VisualizationData{struct_size:core::mem::size_of::<sys::CNA_VisualizationData>() as u32,struct_version:1,frequencies:visualizationData.frequencies,samples:visualizationData.samples};native.check(unsafe{f(handle,&mut value)})?;visualizationData.update_from_native(&value);Ok(())}

    pub fn AddActiveSongChangedHandler(handler:Box<dyn EventHandler>)->u64{MediaRuntime::process().add_active_song_handler(handler)}
    pub fn RemoveActiveSongChangedHandler(registration:u64)->bool{MediaRuntime::process().remove_active_song_handler(registration)}
    pub fn AddMediaStateChangedHandler(handler:Box<dyn EventHandler>)->u64{MediaRuntime::process().add_media_state_handler(handler)}
    pub fn RemoveMediaStateChangedHandler(registration:u64)->bool{MediaRuntime::process().remove_media_state_handler(registration)}

    pub(crate) fn update(game:&GameContext<'_>)->Result<()>{Self::invoke(game,game.native_game().0.media.media_player_update_ext)}
    pub(crate) fn raise_active_song_changed(game:&GameContext<'_>)->Result<()>{Self::invoke(game,game.native_game().0.media.media_player_raise_active_song_changed_ext)}
    pub(crate) fn raise_media_state_changed(game:&GameContext<'_>)->Result<()>{Self::invoke(game,game.native_game().0.media.media_player_raise_media_state_changed_ext)}

    pub(crate) fn play_from_event(song: &Song) -> Result<()> {
        let runtime = MediaRuntime::process();
        let (native, handle, generation) = runtime.event_binding()?;
        if song.core.generation() != generation {
            return Err(CnaError::InvalidInput(
                "Song belongs to another Game generation",
            ));
        }
        Self::invalidate_queue_songs_for_generation(generation);
        let play: sys::cna_media_player_play_song_fn = native.media.media_player_play_song;
        native.check(unsafe { play(handle, song.native_handle()?) })
    }

    pub(crate) fn stop_from_event() -> Result<()> {
        let runtime = MediaRuntime::process();
        let (native, handle, _) = runtime.event_binding()?;
        let stop: unsafe extern "C" fn(sys::CNA_Handle) -> sys::CNA_Result = native.media.media_player_stop;
        native.check(unsafe { stop(handle) })
    }
}

pub struct MediaQueue {
    core: Arc<ResourceCore>,
    songs: Mutex<Vec<Option<Arc<Song>>>>,
}

impl MediaQueue {
    fn from_handle(native:Arc<crate::native::Native>,runtime:Arc<MediaRuntime>,generation:u64,handle:sys::CNA_Handle)->Arc<Self>{let destroy:sys::cna_media_queue_destroy_fn=native.media.media_queue_destroy;Arc::new(Self{core:ResourceCore::new(native,runtime,generation,handle,None,destroy,None),songs:Mutex::new(Vec::new())})}
    pub fn Count(&self)->Result<i32>{let f:sys::cna_media_queue_get_count_fn=self.core.native().media.media_queue_get_count;let mut value=0;self.core.native().check(unsafe{f(self.core.handle()?,&mut value)})?;Ok(value)}
    pub fn ActiveSongIndex(&self)->Result<i32>{let f:sys::cna_media_queue_get_active_song_index_fn=self.core.native().media.media_queue_get_active_song_index;let mut value=0;self.core.native().check(unsafe{f(self.core.handle()?,&mut value)})?;Ok(value)}
    pub fn SetActiveSongIndex(&self,value:i32)->Result<()>{let f:sys::cna_media_queue_set_active_song_index_fn=self.core.native().media.media_queue_set_active_song_index;self.core.native().check(unsafe{f(self.core.handle()?,value)})}
    pub fn Item(&self,index:i32)->Result<Arc<Song>>{let count=self.Count()?;if index<0||index>=count{return Err(CnaError::InvalidInput("media queue index is out of range"));}let position=index as usize;let mut cache=self.songs.lock().unwrap_or_else(std::sync::PoisonError::into_inner);if cache.len()<count as usize{cache.resize_with(count as usize,||None);}if let Some(song)=cache[position].as_ref(){return Ok(Arc::clone(song));}let f:sys::cna_media_queue_get_at_fn=self.core.native().media.media_queue_get_at;let mut handle=0;self.core.native().check(unsafe{f(self.core.handle()?,index,&mut handle)})?;let song=Song::from_handle(Arc::clone(self.core.native()),Arc::clone(self.core.runtime()),self.core.generation(),handle);cache[position]=Some(Arc::clone(&song));Ok(song)}
    pub fn ActiveSong(&self)->Result<Option<Arc<Song>>>{let index=self.ActiveSongIndex()?;if index<0{return Ok(None);}let f:sys::cna_media_queue_get_active_song_fn=self.core.native().media.media_queue_get_active_song;let mut handle=0;let mut available=0;self.core.native().check(unsafe{f(self.core.handle()?,&mut handle,&mut available)})?;if available==0{return Ok(None);}let position=index as usize;let count=self.Count()?;let mut cache=self.songs.lock().unwrap_or_else(std::sync::PoisonError::into_inner);if cache.len()<count.max(0) as usize{cache.resize_with(count.max(0) as usize,||None);}if position<cache.len(){if let Some(song)=cache[position].as_ref(){let destroy:sys::cna_song_destroy_fn=self.core.native().media.song_destroy;self.core.native().check(unsafe{destroy(handle)})?;return Ok(Some(Arc::clone(song)));}let song=Song::from_handle(Arc::clone(self.core.native()),Arc::clone(self.core.runtime()),self.core.generation(),handle);cache[position]=Some(Arc::clone(&song));return Ok(Some(song));}Ok(Some(Song::from_handle(Arc::clone(self.core.native()),Arc::clone(self.core.runtime()),self.core.generation(),handle)))}
    fn invalidate_songs(&self){let songs=self.songs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).drain(..).flatten().collect::<Vec<_>>();for song in songs.iter().rev(){song.core.invalidate();}}
}

impl Drop for MediaQueue { fn drop(&mut self){self.invalidate_songs();self.core.invalidate();} }

impl MediaQueueExt for MediaQueue {
    fn Add(&self, song: &Song) -> Result<()> {
        self.core
            .native()
            .media_queue_add(self.core.handle()?, song.core.handle()?)
    }

    fn Clear(&self) -> Result<()> {
        self.core.native().media_queue_clear(self.core.handle()?)
    }
}
