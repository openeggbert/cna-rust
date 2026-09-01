use std::io::Read;
use std::sync::{Arc, Mutex};

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::game::GameContext;
use crate::native::Native;
use crate::extensions::media::MediaLibraryExt;
use crate::extensions::media::MediaSourceExt;
use crate::extensions::media::PictureExt;

use super::runtime::MediaRuntime;
use super::{
    string_view, AlbumCollection, ArtistCollection, GenreCollection,
    MediaSourceType, Picture, PictureAlbum, PictureCollection, PlaylistCollection, ResourceCore,
    SongCollection,
};

pub struct MediaSource {
    runtime: Arc<MediaRuntime>,
    generation: u64,
    index: Option<u32>,
    source_type: MediaSourceType,
    name: String,
}

impl MediaSource {
    pub fn GetAvailableMediaSources(game: &GameContext<'_>) -> Result<Vec<Arc<MediaSource>>> {
        let (native, handle) = game.native_game();
        let runtime = Arc::clone(game.media_runtime());
        let generation = game.media_generation();
        let count_fn: sys::cna_media_source_get_available_count_fn = native.media.media_source_get_available_count;
        let type_fn: sys::cna_media_source_get_type_at_fn = native.media.media_source_get_type_at;
        let size_fn: sys::cna_media_source_get_name_size_at_fn = native.media.media_source_get_name_size_at;
        let copy_fn: sys::cna_media_source_copy_name_at_fn = native.media.media_source_copy_name_at;
        let mut count = 0;
        // SAFETY: game and output pointer are valid for this callback.
        native.check(unsafe { count_fn(handle, &mut count) })?;
        let mut result = Vec::with_capacity(count as usize);
        for index in 0..count {
            let mut source_type = 0;
            // SAFETY: the index is below the just-observed source count.
            native.check(unsafe { type_fn(handle, index, &mut source_type) })?;
            let mut required = 0;
            // SAFETY: output pointer is valid.
            native.check(unsafe { size_fn(handle, index, &mut required) })?;
            let capacity = usize::try_from(required)
                .map_err(|_| CnaError::InvalidInput("media source name is too large"))?;
            let mut bytes = vec![0_u8; capacity];
            let mut copied = 0;
            // SAFETY: destination has the queried capacity.
            native.check(unsafe {
                copy_fn(
                    handle,
                    index,
                    bytes.as_mut_ptr().cast(),
                    required,
                    &mut copied,
                )
            })?;
            let name = String::from_utf8(bytes)
                .map_err(|_| CnaError::InvalidInput("media source name is not UTF-8"))?;
            result.push(Arc::new(Self {
                runtime: Arc::clone(&runtime),
                generation,
                index: Some(index),
                source_type: MediaSourceType::from_native(source_type)?,
                name,
            }));
        }
        Ok(result)
    }

    pub fn MediaSourceType(&self) -> Result<MediaSourceType> {
        self.validate()?;
        Ok(self.source_type)
    }

    pub fn Name(&self) -> Result<String> {
        self.validate()?;
        Ok(self.name.clone())
    }

    pub fn ToString(&self) -> Result<String> {
        self.Name()
    }

    fn validate(&self) -> Result<()> {
        if self.runtime.is_generation_active(self.generation) {
            Ok(())
        } else {
            Err(CnaError::InvalidInput(
                "MediaSource belongs to a dead Game generation",
            ))
        }
    }
}

pub struct MediaLibrary {
    core: Arc<ResourceCore>,
    source: Mutex<Option<Arc<MediaSource>>>,
    songs: Mutex<Option<Arc<SongCollection>>>,
    albums: Mutex<Option<Arc<AlbumCollection>>>,
    artists: Mutex<Option<Arc<ArtistCollection>>>,
    genres: Mutex<Option<Arc<GenreCollection>>>,
    playlists: Mutex<Option<Arc<PlaylistCollection>>>,
    pictures: Mutex<Option<Arc<PictureCollection>>>,
    saved_pictures: Mutex<Option<Arc<PictureCollection>>>,
    root_picture_album: Mutex<Option<Arc<PictureAlbum>>>,
}

impl MediaLibrary {
    pub fn new(game: &GameContext<'_>) -> Result<Self> {
        Self::create(game, None)
    }

    pub fn from_media_source(game: &GameContext<'_>, mediaSource: &MediaSource) -> Result<Self> {
        mediaSource.validate()?;
        let index = mediaSource.index.ok_or(CnaError::InvalidInput(
            "this MediaSource is not an enumerated platform source",
        ))?;
        Self::create(game, Some(index))
    }

    fn create(game: &GameContext<'_>, source_index: Option<u32>) -> Result<Self> {
        let (native, game_handle) = game.native_game();
        let runtime = Arc::clone(game.media_runtime());
        let generation = game.media_generation();
        let mut handle = 0;
        if let Some(index) = source_index {
            let create: sys::cna_media_library_create_from_source_fn = native.media.media_library_create_from_source;
            // SAFETY: game is live and the output pointer is valid.
            native.check(unsafe { create(game_handle, index, &mut handle) })?;
        } else {
            let create: sys::cna_media_library_create_fn = native.media.media_library_create;
            // SAFETY: game is live and the output pointer is valid.
            native.check(unsafe { create(game_handle, &mut handle) })?;
        }
        let dispose: sys::cna_media_library_dispose_fn = native.media.media_library_dispose;
        let destroy: sys::cna_media_library_destroy_fn = native.media.media_library_destroy;
        let is_disposed: sys::cna_media_library_get_is_disposed_fn = native.media.media_library_get_is_disposed;
        Ok(Self {
            core: ResourceCore::new(
                Arc::clone(native),
                runtime,
                generation,
                handle,
                Some(dispose),
                destroy,
                Some(is_disposed),
            ),
            source: Mutex::new(None),
            songs: Mutex::new(None),
            albums: Mutex::new(None),
            artists: Mutex::new(None),
            genres: Mutex::new(None),
            playlists: Mutex::new(None),
            pictures: Mutex::new(None),
            saved_pictures: Mutex::new(None),
            root_picture_album: Mutex::new(None),
        })
    }

    fn collection<T>(
        &self,
        cache: &Mutex<Option<Arc<T>>>,
        getter: unsafe extern "C" fn(sys::CNA_Handle, *mut sys::CNA_Handle) -> sys::CNA_Result,
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
        // SAFETY: the library and output pointer are valid.
        self.core
            .native()
            .check(unsafe { getter(self.core.handle()?, &mut handle) })?;
        let value = create(
            Arc::clone(self.core.native()),
            Arc::clone(self.core.runtime()),
            self.core.generation(),
            handle,
        );
        *cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::clone(&value));
        Ok(value)
    }

    pub fn Songs(&self) -> Result<Arc<SongCollection>> { let f:sys::cna_media_library_get_songs_fn=self.core.native().media.media_library_get_songs;self.collection(&self.songs,f,SongCollection::from_handle) }
    pub fn Albums(&self) -> Result<Arc<AlbumCollection>> { let f:sys::cna_media_library_get_albums_fn=self.core.native().media.media_library_get_albums;self.collection(&self.albums,f,AlbumCollection::from_handle) }
    pub fn Artists(&self) -> Result<Arc<ArtistCollection>> { let f:sys::cna_media_library_get_artists_fn=self.core.native().media.media_library_get_artists;self.collection(&self.artists,f,ArtistCollection::from_handle) }
    pub fn Genres(&self) -> Result<Arc<GenreCollection>> { let f:sys::cna_media_library_get_genres_fn=self.core.native().media.media_library_get_genres;self.collection(&self.genres,f,GenreCollection::from_handle) }
    pub fn Playlists(&self) -> Result<Arc<PlaylistCollection>> { let f:sys::cna_media_library_get_playlists_fn=self.core.native().media.media_library_get_playlists;self.collection(&self.playlists,f,PlaylistCollection::from_handle) }
    pub fn Pictures(&self) -> Result<Arc<PictureCollection>> { let f:sys::cna_media_library_get_pictures_fn=self.core.native().media.media_library_get_pictures;self.collection(&self.pictures,f,PictureCollection::from_handle) }
    pub fn SavedPictures(&self) -> Result<Arc<PictureCollection>> { let f:sys::cna_media_library_get_saved_pictures_fn=self.core.native().media.media_library_get_saved_pictures;self.collection(&self.saved_pictures,f,PictureCollection::from_handle) }

    pub fn RootPictureAlbum(&self) -> Result<Option<Arc<PictureAlbum>>> {
        if let Some(value)=self.root_picture_album.lock().unwrap_or_else(std::sync::PoisonError::into_inner).as_ref(){return Ok(Some(Arc::clone(value)));}
        let f:sys::cna_media_library_get_root_picture_album_fn=self.core.native().media.media_library_get_root_picture_album;let mut handle=0;let mut available=0;self.core.native().check(unsafe{f(self.core.handle()?,&mut handle,&mut available)})?;if available==0{return Ok(None);}let value=PictureAlbum::from_handle(Arc::clone(self.core.native()),Arc::clone(self.core.runtime()),self.core.generation(),handle);*self.root_picture_album.lock().unwrap_or_else(std::sync::PoisonError::into_inner)=Some(Arc::clone(&value));Ok(Some(value))
    }

    pub fn MediaSource(&self) -> Result<Arc<MediaSource>> {
        if let Some(value)=self.source.lock().unwrap_or_else(std::sync::PoisonError::into_inner).as_ref(){return Ok(Arc::clone(value));}
        let type_fn:sys::cna_media_library_get_media_source_type_fn=self.core.native().media.media_library_get_media_source_type;let size_fn:sys::cna_media_library_get_media_source_name_size_fn=self.core.native().media.media_library_get_media_source_name_size;let copy_fn:sys::cna_media_library_copy_media_source_name_fn=self.core.native().media.media_library_copy_media_source_name;let handle=self.core.handle()?;let mut kind=0;self.core.native().check(unsafe{type_fn(handle,&mut kind)})?;let mut required=0;self.core.native().check(unsafe{size_fn(handle,&mut required)})?;let mut bytes=vec![0_u8;usize::try_from(required).map_err(|_|CnaError::InvalidInput("media source name is too large"))?];let mut copied=0;self.core.native().check(unsafe{copy_fn(handle,bytes.as_mut_ptr().cast(),required,&mut copied)})?;let value=Arc::new(MediaSource{runtime:Arc::clone(self.core.runtime()),generation:self.core.generation(),index:None,source_type:MediaSourceType::from_native(kind)?,name:String::from_utf8(bytes).map_err(|_|CnaError::InvalidInput("media source name is not UTF-8"))?});*self.source.lock().unwrap_or_else(std::sync::PoisonError::into_inner)=Some(Arc::clone(&value));Ok(value)
    }

    pub fn SavePicture(&self, name: &str, imageBuffer: &[u8]) -> Result<Arc<Picture>> {
        let f:sys::cna_media_library_save_picture_fn=self.core.native().media.media_library_save_picture;let mut handle=0;self.core.native().check(unsafe{f(self.core.handle()?,string_view(name),imageBuffer.as_ptr(),imageBuffer.len() as u64,&mut handle)})?;Ok(Picture::from_handle(Arc::clone(self.core.native()),Arc::clone(self.core.runtime()),self.core.generation(),handle))
    }

    pub fn SavePictureWithNameAndSource<R: Read>(&self, name:&str, source:&mut R)->Result<Arc<Picture>> { let mut bytes=Vec::new();source.read_to_end(&mut bytes).map_err(|error|CnaError::Io(error.to_string()))?;self.SavePicture(name,&bytes) }

    pub fn GetPictureFromToken(&self, token:&str)->Result<Option<Arc<Picture>>>{let f:sys::cna_media_library_get_picture_from_token_fn=self.core.native().media.media_library_get_picture_from_token;let mut handle=0;let mut available=0;self.core.native().check(unsafe{f(self.core.handle()?,string_view(token),&mut handle,&mut available)})?;if available==0{Ok(None)}else{Ok(Some(Picture::from_handle(Arc::clone(self.core.native()),Arc::clone(self.core.runtime()),self.core.generation(),handle)))} }
    pub fn IsDisposed(&self)->Result<bool>{self.core.IsDisposed()}
    pub fn Dispose(&self)->Result<()>{self.invalidate_children();self.core.Dispose()}
    pub fn Finalize(&self)->Result<()> { Ok(()) }

    fn invalidate_children(&self){if let Some(v)=self.songs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}if let Some(v)=self.albums.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}if let Some(v)=self.artists.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}if let Some(v)=self.genres.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}if let Some(v)=self.playlists.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}if let Some(v)=self.pictures.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}if let Some(v)=self.saved_pictures.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.invalidate();}if let Some(v)=self.root_picture_album.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(){v.core.invalidate();}self.source.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take();}
}

impl Drop for MediaLibrary { fn drop(&mut self){self.invalidate_children();self.core.invalidate();} }

impl MediaSourceExt for MediaSource {
    fn TypeName(&self, game: &GameContext<'_>) -> Result<Option<String>> {
        let Some(index) = self.index else {
            return Ok(None);
        };
        if !self.runtime.is_generation_active(self.generation) {
            return Err(CnaError::InvalidInput(
                "MediaSource belongs to a dead Game generation",
            ));
        }
        let (native, handle) = game.native_game();
        native.media_source_type_name(handle, index).map(Some)
    }
}

impl PictureExt for Picture {
    fn PlatformToken(&self) -> Result<String> {
        self.core.native().picture_token(self.core.handle()?)
    }
}

impl MediaLibraryExt for MediaLibrary {
    fn SavePictureFromStream(
        &self,
        name: &str,
        stream: &crate::storage::StorageStream,
    ) -> Result<Arc<Picture>> {
        let handle = self.core.native().save_picture_from_stream(
            self.core.handle()?,
            name,
            stream.native_handle()?,
        )?;
        Ok(Picture::from_handle(
            Arc::clone(self.core.native()),
            Arc::clone(self.core.runtime()),
            self.core.generation(),
            handle,
        ))
    }
}
