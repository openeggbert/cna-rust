use std::any::Any;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::game::{GameContext, TimeSpan};
use crate::native::Native;

use super::collections::CollectionElement;
use super::runtime::MediaRuntime;
use super::{
    native_function, read_blob, read_bool, read_i32, read_i64, read_string, string_view,
    AlbumCollection, PictureAlbumCollection,
    PictureCollection, ResourceCore, SongCollection,
};

type OptionalHandleFn = unsafe extern "C" fn(
    sys::CNA_Handle,
    *mut sys::CNA_Handle,
    *mut sys::CNA_Bool,
) -> sys::CNA_Result;
type HandleFn =
    unsafe extern "C" fn(sys::CNA_Handle, *mut sys::CNA_Handle) -> sys::CNA_Result;

fn optional_child<T>(
    core: &ResourceCore,
    cache: &Mutex<Option<Arc<T>>>,
    getter: OptionalHandleFn,
    create: impl FnOnce(Arc<Native>, Arc<MediaRuntime>, u64, sys::CNA_Handle) -> Arc<T>,
) -> Result<Option<Arc<T>>> {
    if let Some(value) = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
    {
        return Ok(Some(Arc::clone(value)));
    }
    let mut handle = 0;
    let mut available = sys::CNA_FALSE;
    // SAFETY: the parent handle and both output pointers are valid.
    core.native().check(unsafe {
        getter(core.handle()?, &mut handle, &mut available)
    })?;
    if available == sys::CNA_FALSE {
        return Ok(None);
    }
    let value = create(
        Arc::clone(core.native()),
        Arc::clone(core.runtime()),
        core.generation(),
        handle,
    );
    *cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::clone(&value));
    Ok(Some(value))
}

fn required_child<T>(
    core: &ResourceCore,
    cache: &Mutex<Option<Arc<T>>>,
    getter: HandleFn,
    create: impl FnOnce(Arc<Native>, Arc<MediaRuntime>, u64, sys::CNA_Handle) -> Arc<T>,
) -> Result<Arc<T>> {
    if let Some(value) = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
    {
        return Ok(Arc::clone(value));
    }
    let mut handle = 0;
    // SAFETY: the parent handle and output pointer are valid.
    core.native()
        .check(unsafe { getter(core.handle()?, &mut handle) })?;
    let value = create(
        Arc::clone(core.native()),
        Arc::clone(core.runtime()),
        core.generation(),
        handle,
    );
    *cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::clone(&value));
    Ok(value)
}

macro_rules! common_object {
    ($type:ident, $name_alias:ty, $name_field:ident, $copy_alias:ty, $copy_field:ident,
     $equals_alias:ty, $equals_field:ident, $hash_alias:ty, $hash_field:ident) => {
        impl $type {
            pub fn Name(&self) -> Result<String> {
                let size: $name_alias = unsafe { native_function(self.core.native().media.$name_field) };
                let copy: $copy_alias = unsafe { native_function(self.core.native().media.$copy_field) };
                read_string(&self.core, size, copy)
            }

            pub fn IsDisposed(&self) -> Result<bool> { self.core.IsDisposed() }
            pub fn Dispose(&self) -> Result<()> { self.invalidate_children(); self.core.Dispose() }
            pub fn Finalize(&self) -> Result<()> { Ok(()) }
            pub fn ToString(&self) -> Result<String> { self.Name() }
            pub fn GetHashCode(&self) -> Result<i32> {
                let getter: $hash_alias = unsafe { native_function(self.core.native().media.$hash_field) };
                read_i32(&self.core, getter)
            }
            pub fn EqualsWithOther(&self, other: &$type) -> Result<bool> {
                let equals: $equals_alias = unsafe { native_function(self.core.native().media.$equals_field) };
                let mut value = sys::CNA_FALSE;
                self.core.native().check(unsafe {
                    equals(self.core.handle()?, other.core.handle()?, &mut value)
                })?;
                Ok(value != sys::CNA_FALSE)
            }
            pub fn Equals(&self, obj: &dyn Any) -> Result<bool> {
                obj.downcast_ref::<$type>().map_or(Ok(false), |other| self.EqualsWithOther(other))
            }
        }

        impl PartialEq for $type {
            fn eq(&self, other: &Self) -> bool { self.EqualsWithOther(other).unwrap_or(false) }
        }
        impl Eq for $type {}
        impl Drop for $type {
            fn drop(&mut self) { self.invalidate_children(); self.core.invalidate(); }
        }
    };
}

pub struct Album {
    pub(crate) core: Arc<ResourceCore>,
    artist: Mutex<Option<Arc<Artist>>>,
    genre: Mutex<Option<Arc<Genre>>>,
    songs: Mutex<Option<Arc<SongCollection>>>,
}

impl Album {
    pub(crate) fn from_handle(native: Arc<Native>, runtime: Arc<MediaRuntime>, generation: u64, handle: sys::CNA_Handle) -> Arc<Self> {
        let dispose: sys::cna_album_dispose_fn = unsafe { native_function(native.media.album_dispose) };
        let destroy: sys::cna_album_destroy_fn = unsafe { native_function(native.media.album_destroy) };
        let is_disposed: sys::cna_album_get_is_disposed_fn = unsafe { native_function(native.media.album_get_is_disposed) };
        Arc::new(Self { core: ResourceCore::new(native, runtime, generation, handle, Some(dispose), destroy, Some(is_disposed)), artist: Mutex::new(None), genre: Mutex::new(None), songs: Mutex::new(None) })
    }
    pub fn Artist(&self) -> Result<Option<Arc<Artist>>> { let f: sys::cna_album_get_artist_fn = unsafe { native_function(self.core.native().media.album_get_artist) }; optional_child(&self.core, &self.artist, f, Artist::from_handle) }
    pub fn Genre(&self) -> Result<Option<Arc<Genre>>> { let f: sys::cna_album_get_genre_fn = unsafe { native_function(self.core.native().media.album_get_genre) }; optional_child(&self.core, &self.genre, f, Genre::from_handle) }
    pub fn Songs(&self) -> Result<Arc<SongCollection>> { let f: sys::cna_album_get_songs_fn = unsafe { native_function(self.core.native().media.album_get_songs) }; required_child(&self.core, &self.songs, f, SongCollection::from_handle) }
    pub fn Duration(&self) -> Result<TimeSpan> { let f: sys::cna_album_get_duration_fn = unsafe { native_function(self.core.native().media.album_get_duration) }; Ok(TimeSpan::from_ticks(read_i64(&self.core, f)?)) }
    pub fn HasArt(&self) -> Result<bool> { let f: sys::cna_album_get_has_art_fn = unsafe { native_function(self.core.native().media.album_get_has_art) }; read_bool(&self.core, f) }
    pub fn GetAlbumArt(&self) -> Result<Box<dyn Read + Send>> { let s: sys::cna_album_get_art_size_fn = unsafe { native_function(self.core.native().media.album_get_art_size) }; let c: sys::cna_album_copy_art_fn = unsafe { native_function(self.core.native().media.album_copy_art) }; read_blob(&self.core, s, c) }
    pub fn GetThumbnail(&self) -> Result<Box<dyn Read + Send>> { let s: sys::cna_album_get_thumbnail_size_fn = unsafe { native_function(self.core.native().media.album_get_thumbnail_size) }; let c: sys::cna_album_copy_thumbnail_fn = unsafe { native_function(self.core.native().media.album_copy_thumbnail) }; read_blob(&self.core, s, c) }
    fn invalidate_children(&self) { if let Some(v)=self.songs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();} if let Some(v)=self.artist.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.core.invalidate();} if let Some(v)=self.genre.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.core.invalidate();} }
}
common_object!(Album, sys::cna_album_get_name_size_fn, album_get_name_size, sys::cna_album_copy_name_fn, album_copy_name, sys::cna_album_equals_fn, album_equals, sys::cna_album_get_hash_code_fn, album_get_hash_code);

pub struct Artist { pub(crate) core: Arc<ResourceCore>, albums: Mutex<Option<Arc<AlbumCollection>>>, songs: Mutex<Option<Arc<SongCollection>>> }
impl Artist {
    pub(crate) fn from_handle(native: Arc<Native>, runtime: Arc<MediaRuntime>, generation: u64, handle: sys::CNA_Handle) -> Arc<Self> { let d:sys::cna_artist_dispose_fn=unsafe{native_function(native.media.artist_dispose)}; let x:sys::cna_artist_destroy_fn=unsafe{native_function(native.media.artist_destroy)}; let i:sys::cna_artist_get_is_disposed_fn=unsafe{native_function(native.media.artist_get_is_disposed)}; Arc::new(Self{core:ResourceCore::new(native,runtime,generation,handle,Some(d),x,Some(i)),albums:Mutex::new(None),songs:Mutex::new(None)}) }
    pub fn Albums(&self)->Result<Arc<AlbumCollection>>{let f:sys::cna_artist_get_albums_fn=unsafe{native_function(self.core.native().media.artist_get_albums)};required_child(&self.core,&self.albums,f,AlbumCollection::from_handle)}
    pub fn Songs(&self)->Result<Arc<SongCollection>>{let f:sys::cna_artist_get_songs_fn=unsafe{native_function(self.core.native().media.artist_get_songs)};required_child(&self.core,&self.songs,f,SongCollection::from_handle)}
    fn invalidate_children(&self){if let Some(v)=self.albums.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}if let Some(v)=self.songs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}}
}
common_object!(Artist,sys::cna_artist_get_name_size_fn,artist_get_name_size,sys::cna_artist_copy_name_fn,artist_copy_name,sys::cna_artist_equals_fn,artist_equals,sys::cna_artist_get_hash_code_fn,artist_get_hash_code);

pub struct Genre { pub(crate) core: Arc<ResourceCore>, albums: Mutex<Option<Arc<AlbumCollection>>>, songs: Mutex<Option<Arc<SongCollection>>> }
impl Genre {
    pub(crate) fn from_handle(native: Arc<Native>, runtime: Arc<MediaRuntime>, generation: u64, handle: sys::CNA_Handle) -> Arc<Self> { let d:sys::cna_genre_dispose_fn=unsafe{native_function(native.media.genre_dispose)};let x:sys::cna_genre_destroy_fn=unsafe{native_function(native.media.genre_destroy)};let i:sys::cna_genre_get_is_disposed_fn=unsafe{native_function(native.media.genre_get_is_disposed)};Arc::new(Self{core:ResourceCore::new(native,runtime,generation,handle,Some(d),x,Some(i)),albums:Mutex::new(None),songs:Mutex::new(None)}) }
    pub fn Albums(&self)->Result<Arc<AlbumCollection>>{let f:sys::cna_genre_get_albums_fn=unsafe{native_function(self.core.native().media.genre_get_albums)};required_child(&self.core,&self.albums,f,AlbumCollection::from_handle)}
    pub fn Songs(&self)->Result<Arc<SongCollection>>{let f:sys::cna_genre_get_songs_fn=unsafe{native_function(self.core.native().media.genre_get_songs)};required_child(&self.core,&self.songs,f,SongCollection::from_handle)}
    fn invalidate_children(&self){if let Some(v)=self.albums.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}if let Some(v)=self.songs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}}
}
common_object!(Genre,sys::cna_genre_get_name_size_fn,genre_get_name_size,sys::cna_genre_copy_name_fn,genre_copy_name,sys::cna_genre_equals_fn,genre_equals,sys::cna_genre_get_hash_code_fn,genre_get_hash_code);

pub struct Playlist { pub(crate) core: Arc<ResourceCore>, songs: Mutex<Option<Arc<SongCollection>>> }
impl Playlist {
    pub(crate) fn from_handle(native: Arc<Native>, runtime: Arc<MediaRuntime>, generation: u64, handle: sys::CNA_Handle)->Arc<Self>{let d:sys::cna_playlist_dispose_fn=unsafe{native_function(native.media.playlist_dispose)};let x:sys::cna_playlist_destroy_fn=unsafe{native_function(native.media.playlist_destroy)};let i:sys::cna_playlist_get_is_disposed_fn=unsafe{native_function(native.media.playlist_get_is_disposed)};Arc::new(Self{core:ResourceCore::new(native,runtime,generation,handle,Some(d),x,Some(i)),songs:Mutex::new(None)})}
    pub fn Songs(&self)->Result<Arc<SongCollection>>{let f:sys::cna_playlist_get_songs_fn=unsafe{native_function(self.core.native().media.playlist_get_songs)};required_child(&self.core,&self.songs,f,SongCollection::from_handle)}
    pub fn Duration(&self)->Result<TimeSpan>{let f:sys::cna_playlist_get_duration_fn=unsafe{native_function(self.core.native().media.playlist_get_duration)};Ok(TimeSpan::from_ticks(read_i64(&self.core,f)?))}
    fn invalidate_children(&self){if let Some(v)=self.songs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}}
}
common_object!(Playlist,sys::cna_playlist_get_name_size_fn,playlist_get_name_size,sys::cna_playlist_copy_name_fn,playlist_copy_name,sys::cna_playlist_equals_fn,playlist_equals,sys::cna_playlist_get_hash_code_fn,playlist_get_hash_code);

pub struct Song { pub(crate) core: Arc<ResourceCore>, album: Mutex<Option<Arc<Album>>>, artist: Mutex<Option<Arc<Artist>>>, genre: Mutex<Option<Arc<Genre>>> }
impl Song {
    pub(crate) fn from_handle(native: Arc<Native>,runtime:Arc<MediaRuntime>,generation:u64,handle:sys::CNA_Handle)->Arc<Self>{let d:sys::cna_song_dispose_fn=unsafe{native_function(native.media.song_dispose)};let x:sys::cna_song_destroy_fn=unsafe{native_function(native.media.song_destroy)};let i:sys::cna_song_get_is_disposed_fn=unsafe{native_function(native.media.song_get_is_disposed)};Arc::new(Self{core:ResourceCore::new(native,runtime,generation,handle,Some(d),x,Some(i)),album:Mutex::new(None),artist:Mutex::new(None),genre:Mutex::new(None)})}
    pub fn FromUri(game:&GameContext<'_>,name:&str,uri:&str)->Result<Self>{let (native,game_handle)=game.native_game();let runtime=Arc::clone(game.media_runtime());let generation=game.media_generation();let f:sys::cna_song_create_from_uri_fn=unsafe{native_function(native.media.song_create_from_uri)};let mut handle=0;native.check(unsafe{f(game_handle,string_view(name),string_view(uri),&mut handle)})?;Arc::try_unwrap(Self::from_handle(Arc::clone(native),runtime,generation,handle)).map_err(|_|CnaError::InvalidInput("new Song identity was unexpectedly shared"))}
    pub fn Album(&self)->Result<Option<Arc<Album>>>{let f:sys::cna_song_get_album_fn=unsafe{native_function(self.core.native().media.song_get_album)};optional_child(&self.core,&self.album,f,Album::from_handle)}
    pub fn Artist(&self)->Result<Option<Arc<Artist>>>{let f:sys::cna_song_get_artist_fn=unsafe{native_function(self.core.native().media.song_get_artist)};optional_child(&self.core,&self.artist,f,Artist::from_handle)}
    pub fn Genre(&self)->Result<Option<Arc<Genre>>>{let f:sys::cna_song_get_genre_fn=unsafe{native_function(self.core.native().media.song_get_genre)};optional_child(&self.core,&self.genre,f,Genre::from_handle)}
    pub fn Duration(&self)->Result<TimeSpan>{let f:sys::cna_song_get_duration_fn=unsafe{native_function(self.core.native().media.song_get_duration)};Ok(TimeSpan::from_ticks(read_i64(&self.core,f)?))}
    pub fn IsProtected(&self)->Result<bool>{let f:sys::cna_song_get_is_protected_fn=unsafe{native_function(self.core.native().media.song_get_is_protected)};read_bool(&self.core,f)}
    pub fn IsRated(&self)->Result<bool>{let f:sys::cna_song_get_is_rated_fn=unsafe{native_function(self.core.native().media.song_get_is_rated)};read_bool(&self.core,f)}
    pub fn PlayCount(&self)->Result<i32>{let f:sys::cna_song_get_play_count_fn=unsafe{native_function(self.core.native().media.song_get_play_count)};read_i32(&self.core,f)}
    pub fn Rating(&self)->Result<i32>{let f:sys::cna_song_get_rating_fn=unsafe{native_function(self.core.native().media.song_get_rating)};read_i32(&self.core,f)}
    pub fn TrackNumber(&self)->Result<i32>{let f:sys::cna_song_get_track_number_fn=unsafe{native_function(self.core.native().media.song_get_track_number)};read_i32(&self.core,f)}
    pub(crate) fn native_handle(&self)->Result<sys::CNA_Handle>{self.core.handle()}
    fn invalidate_children(&self){if let Some(v)=self.album.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.core.invalidate();}if let Some(v)=self.artist.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.core.invalidate();}if let Some(v)=self.genre.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.core.invalidate();}}
}
common_object!(Song,sys::cna_song_get_name_size_fn,song_get_name_size,sys::cna_song_copy_name_fn,song_copy_name,sys::cna_song_equals_fn,song_equals,sys::cna_song_get_hash_code_fn,song_get_hash_code);

pub struct Picture { pub(crate) core: Arc<ResourceCore>, album: Mutex<Option<Arc<PictureAlbum>>> }
impl Picture {
    pub(crate) fn from_handle(native:Arc<Native>,runtime:Arc<MediaRuntime>,generation:u64,handle:sys::CNA_Handle)->Arc<Self>{let d:sys::cna_picture_dispose_fn=unsafe{native_function(native.media.picture_dispose)};let x:sys::cna_picture_destroy_fn=unsafe{native_function(native.media.picture_destroy)};let i:sys::cna_picture_get_is_disposed_fn=unsafe{native_function(native.media.picture_get_is_disposed)};Arc::new(Self{core:ResourceCore::new(native,runtime,generation,handle,Some(d),x,Some(i)),album:Mutex::new(None)})}
    pub fn Album(&self)->Result<Option<Arc<PictureAlbum>>>{let f:sys::cna_picture_get_album_fn=unsafe{native_function(self.core.native().media.picture_get_album)};optional_child(&self.core,&self.album,f,PictureAlbum::from_handle)}
    pub fn Date(&self)->Result<SystemTime>{let f:sys::cna_picture_get_date_unix_ticks_fn=unsafe{native_function(self.core.native().media.picture_get_date_unix_ticks)};let ticks=read_i64(&self.core,f)?;let nanos=ticks.unsigned_abs().checked_mul(100).ok_or(CnaError::InvalidInput("picture date is out of range"))?;let duration=Duration::from_nanos(nanos);if ticks>=0{UNIX_EPOCH.checked_add(duration)}else{UNIX_EPOCH.checked_sub(duration)}.ok_or(CnaError::InvalidInput("picture date is out of range"))}
    pub fn Width(&self)->Result<i32>{let f:sys::cna_picture_get_width_fn=unsafe{native_function(self.core.native().media.picture_get_width)};read_i32(&self.core,f)}
    pub fn Height(&self)->Result<i32>{let f:sys::cna_picture_get_height_fn=unsafe{native_function(self.core.native().media.picture_get_height)};read_i32(&self.core,f)}
    pub fn GetImage(&self)->Result<Box<dyn Read+Send>>{let s:sys::cna_picture_get_image_size_fn=unsafe{native_function(self.core.native().media.picture_get_image_size)};let c:sys::cna_picture_copy_image_fn=unsafe{native_function(self.core.native().media.picture_copy_image)};read_blob(&self.core,s,c)}
    pub fn GetThumbnail(&self)->Result<Box<dyn Read+Send>>{let s:sys::cna_picture_get_thumbnail_size_fn=unsafe{native_function(self.core.native().media.picture_get_thumbnail_size)};let c:sys::cna_picture_copy_thumbnail_fn=unsafe{native_function(self.core.native().media.picture_copy_thumbnail)};read_blob(&self.core,s,c)}
    fn invalidate_children(&self){if let Some(v)=self.album.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.core.invalidate();}}
}
common_object!(Picture,sys::cna_picture_get_name_size_fn,picture_get_name_size,sys::cna_picture_copy_name_fn,picture_copy_name,sys::cna_picture_equals_fn,picture_equals,sys::cna_picture_get_hash_code_fn,picture_get_hash_code);

pub struct PictureAlbum { pub(crate) core: Arc<ResourceCore>, parent:Mutex<Option<Arc<PictureAlbum>>>, albums:Mutex<Option<Arc<PictureAlbumCollection>>>, pictures:Mutex<Option<Arc<PictureCollection>>> }
impl PictureAlbum {
    pub(crate) fn from_handle(native:Arc<Native>,runtime:Arc<MediaRuntime>,generation:u64,handle:sys::CNA_Handle)->Arc<Self>{let d:sys::cna_picture_album_dispose_fn=unsafe{native_function(native.media.picture_album_dispose)};let x:sys::cna_picture_album_destroy_fn=unsafe{native_function(native.media.picture_album_destroy)};let i:sys::cna_picture_album_get_is_disposed_fn=unsafe{native_function(native.media.picture_album_get_is_disposed)};Arc::new(Self{core:ResourceCore::new(native,runtime,generation,handle,Some(d),x,Some(i)),parent:Mutex::new(None),albums:Mutex::new(None),pictures:Mutex::new(None)})}
    pub fn Parent(&self)->Result<Option<Arc<super::PictureAlbum>>>{let f:sys::cna_picture_album_get_parent_fn=unsafe{native_function(self.core.native().media.picture_album_get_parent)};optional_child(&self.core,&self.parent,f,PictureAlbum::from_handle)}
    pub fn Albums(&self)->Result<Arc<PictureAlbumCollection>>{let f:sys::cna_picture_album_get_albums_fn=unsafe{native_function(self.core.native().media.picture_album_get_albums)};required_child(&self.core,&self.albums,f,PictureAlbumCollection::from_handle)}
    pub fn Pictures(&self)->Result<Arc<PictureCollection>>{let f:sys::cna_picture_album_get_pictures_fn=unsafe{native_function(self.core.native().media.picture_album_get_pictures)};required_child(&self.core,&self.pictures,f,PictureCollection::from_handle)}
    fn invalidate_children(&self){if let Some(v)=self.parent.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.core.invalidate();}if let Some(v)=self.albums.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}if let Some(v)=self.pictures.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}}
}
common_object!(PictureAlbum,sys::cna_picture_album_get_name_size_fn,picture_album_get_name_size,sys::cna_picture_album_copy_name_fn,picture_album_copy_name,sys::cna_picture_album_equals_fn,picture_album_equals,sys::cna_picture_album_get_hash_code_fn,picture_album_get_hash_code);

macro_rules! collection_element { ($type:ty,$ctor:path,$invalidate:expr)=>{impl CollectionElement for $type{fn from_collection_handle(native:Arc<Native>,runtime:Arc<MediaRuntime>,generation:u64,handle:sys::CNA_Handle)->Arc<Self>{$ctor(native,runtime,generation,handle)}fn invalidate_collection_view(&self){$invalidate(self)}}}; }
collection_element!(Album,Album::from_handle,|value:&Album|value.core.invalidate());
collection_element!(Artist,Artist::from_handle,|value:&Artist|value.core.invalidate());
collection_element!(Genre,Genre::from_handle,|value:&Genre|value.core.invalidate());
collection_element!(Playlist,Playlist::from_handle,|value:&Playlist|value.core.invalidate());
collection_element!(Song,Song::from_handle,|value:&Song|value.core.invalidate());
collection_element!(Picture,Picture::from_handle,|value:&Picture|value.core.invalidate());
collection_element!(PictureAlbum,PictureAlbum::from_handle,|value:&PictureAlbum|value.core.invalidate());
