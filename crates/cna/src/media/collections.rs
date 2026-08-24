use std::sync::{Arc, Mutex};
use std::vec::IntoIter;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::native::Native;

use super::runtime::MediaRuntime;
use super::{native_function, Album, Artist, Genre, Picture, PictureAlbum, Playlist, ResourceCore, Song};

type CountFn = unsafe extern "C" fn(sys::CNA_Handle, *mut i32) -> sys::CNA_Result;
type ItemFn =
    unsafe extern "C" fn(sys::CNA_Handle, i32, *mut sys::CNA_Handle) -> sys::CNA_Result;
type BoolFn =
    unsafe extern "C" fn(sys::CNA_Handle, *mut sys::CNA_Bool) -> sys::CNA_Result;
type UnaryFn = unsafe extern "C" fn(sys::CNA_Handle) -> sys::CNA_Result;

pub(crate) trait CollectionElement: Send + Sync + Sized + 'static {
    fn from_collection_handle(
        native: Arc<Native>,
        runtime: Arc<MediaRuntime>,
        generation: u64,
        handle: sys::CNA_Handle,
    ) -> Arc<Self>;
    fn invalidate_collection_view(&self);
}

struct CollectionCore<T: CollectionElement> {
    state: Arc<ResourceCore>,
    count: CountFn,
    item: ItemFn,
    cache: Mutex<Vec<Option<Arc<T>>>>,
}

impl<T: CollectionElement> CollectionCore<T> {
    fn new(
        native: Arc<Native>,
        runtime: Arc<MediaRuntime>,
        generation: u64,
        handle: sys::CNA_Handle,
        count: CountFn,
        item: ItemFn,
        is_disposed: BoolFn,
        dispose: UnaryFn,
        destroy: UnaryFn,
    ) -> Self {
        Self {
            state: ResourceCore::new(
                native,
                runtime,
                generation,
                handle,
                Some(dispose),
                destroy,
                Some(is_disposed),
            ),
            count,
            item,
            cache: Mutex::new(Vec::new()),
        }
    }

    fn Count(&self) -> Result<i32> {
        let mut value = 0;
        // SAFETY: the handle and output pointer are valid for the call.
        self.state.native().check(unsafe {
            (self.count)(self.state.handle()?, &mut value)
        })?;
        Ok(value)
    }

    fn Item(&self, index: i32) -> Result<Arc<T>> {
        let count = self.Count()?;
        if index < 0 || index >= count {
            return Err(CnaError::InvalidInput("media collection index is out of range"));
        }
        let position = usize::try_from(index)
            .map_err(|_| CnaError::InvalidInput("media collection index is out of range"))?;
        let mut cache = self
            .cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if cache.len() < usize::try_from(count).unwrap_or(0) {
            cache.resize_with(usize::try_from(count).unwrap_or(0), || None);
        }
        if let Some(value) = cache[position].as_ref() {
            return Ok(Arc::clone(value));
        }
        let mut handle = 0;
        // SAFETY: the index is range-checked and output pointer is valid.
        self.state.native().check(unsafe {
            (self.item)(self.state.handle()?, index, &mut handle)
        })?;
        let value = T::from_collection_handle(
            Arc::clone(self.state.native()),
            Arc::clone(self.state.runtime()),
            self.state.generation(),
            handle,
        );
        cache[position] = Some(Arc::clone(&value));
        Ok(value)
    }

    fn GetEnumerator(&self) -> Result<IntoIter<Arc<T>>> {
        let count = self.Count()?;
        let mut values = Vec::with_capacity(usize::try_from(count).unwrap_or(0));
        for index in 0..count {
            values.push(self.Item(index)?);
        }
        Ok(values.into_iter())
    }

    fn IsDisposed(&self) -> Result<bool> {
        self.state.IsDisposed()
    }

    fn Dispose(&self) -> Result<()> {
        let cached = self
            .cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain(..)
            .flatten()
            .collect::<Vec<_>>();
        for value in cached.iter().rev() {
            value.invalidate_collection_view();
        }
        self.state.Dispose()
    }

    fn invalidate(&self) {
        let cached = self
            .cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain(..)
            .flatten()
            .collect::<Vec<_>>();
        for value in cached.iter().rev() {
            value.invalidate_collection_view();
        }
        self.state.invalidate();
    }
}

macro_rules! collection {
    (
        $name:ident, $element:ty,
        $count_alias:ty, $count_field:ident,
        $item_alias:ty, $item_field:ident,
        $disposed_alias:ty, $disposed_field:ident,
        $dispose_alias:ty, $dispose_field:ident,
        $destroy_alias:ty, $destroy_field:ident
    ) => {
        pub struct $name {
            core: CollectionCore<$element>,
        }

        impl $name {
            pub(crate) fn from_handle(
                native: Arc<Native>,
                runtime: Arc<MediaRuntime>,
                generation: u64,
                handle: sys::CNA_Handle,
            ) -> Arc<Self> {
                let count: $count_alias = unsafe { native_function(native.media.$count_field) };
                let item: $item_alias = unsafe { native_function(native.media.$item_field) };
                let is_disposed: $disposed_alias = unsafe { native_function(native.media.$disposed_field) };
                let dispose: $dispose_alias = unsafe { native_function(native.media.$dispose_field) };
                let destroy: $destroy_alias = unsafe { native_function(native.media.$destroy_field) };
                Arc::new(Self {
                    core: CollectionCore::new(
                        native, runtime, generation, handle, count, item, is_disposed, dispose, destroy,
                    ),
                })
            }

            pub fn Count(&self) -> Result<i32> { self.core.Count() }
            pub fn Item(&self, index: i32) -> Result<Arc<$element>> { self.core.Item(index) }
            pub fn GetEnumerator(&self) -> Result<IntoIter<Arc<$element>>> { self.core.GetEnumerator() }
            pub fn IsDisposed(&self) -> Result<bool> { self.core.IsDisposed() }
            pub fn Dispose(&self) -> Result<()> { self.core.Dispose() }
            pub fn Finalize(&self) -> Result<()> { Ok(()) }

            #[allow(dead_code)]
            pub(crate) fn native_handle(&self) -> Result<sys::CNA_Handle> { self.core.state.handle() }
            pub(crate) fn invalidate(&self) { self.core.invalidate(); }
        }

        impl Drop for $name {
            fn drop(&mut self) { self.core.invalidate(); }
        }

        impl IntoIterator for &$name {
            type Item = Arc<$element>;
            type IntoIter = IntoIter<Arc<$element>>;

            fn into_iter(self) -> Self::IntoIter {
                self.GetEnumerator().unwrap_or_else(|_| Vec::new().into_iter())
            }
        }
    };
}

collection!(
    AlbumCollection, Album,
    sys::cna_album_collection_get_count_fn, album_collection_get_count,
    sys::cna_album_collection_get_at_fn, album_collection_get_at,
    sys::cna_album_collection_get_is_disposed_fn, album_collection_get_is_disposed,
    sys::cna_album_collection_dispose_fn, album_collection_dispose,
    sys::cna_album_collection_destroy_fn, album_collection_destroy
);
collection!(
    ArtistCollection, Artist,
    sys::cna_artist_collection_get_count_fn, artist_collection_get_count,
    sys::cna_artist_collection_get_at_fn, artist_collection_get_at,
    sys::cna_artist_collection_get_is_disposed_fn, artist_collection_get_is_disposed,
    sys::cna_artist_collection_dispose_fn, artist_collection_dispose,
    sys::cna_artist_collection_destroy_fn, artist_collection_destroy
);
collection!(
    GenreCollection, Genre,
    sys::cna_genre_collection_get_count_fn, genre_collection_get_count,
    sys::cna_genre_collection_get_at_fn, genre_collection_get_at,
    sys::cna_genre_collection_get_is_disposed_fn, genre_collection_get_is_disposed,
    sys::cna_genre_collection_dispose_fn, genre_collection_dispose,
    sys::cna_genre_collection_destroy_fn, genre_collection_destroy
);
collection!(
    SongCollection, Song,
    sys::cna_song_collection_get_count_fn, song_collection_get_count,
    sys::cna_song_collection_get_at_fn, song_collection_get_at,
    sys::cna_song_collection_get_is_disposed_fn, song_collection_get_is_disposed,
    sys::cna_song_collection_dispose_fn, song_collection_dispose,
    sys::cna_song_collection_destroy_fn, song_collection_destroy
);
collection!(
    PlaylistCollection, Playlist,
    sys::cna_playlist_collection_get_count_fn, playlist_collection_get_count,
    sys::cna_playlist_collection_get_at_fn, playlist_collection_get_at,
    sys::cna_playlist_collection_get_is_disposed_fn, playlist_collection_get_is_disposed,
    sys::cna_playlist_collection_dispose_fn, playlist_collection_dispose,
    sys::cna_playlist_collection_destroy_fn, playlist_collection_destroy
);
collection!(
    PictureCollection, Picture,
    sys::cna_picture_collection_get_count_fn, picture_collection_get_count,
    sys::cna_picture_collection_get_at_fn, picture_collection_get_at,
    sys::cna_picture_collection_get_is_disposed_fn, picture_collection_get_is_disposed,
    sys::cna_picture_collection_dispose_fn, picture_collection_dispose,
    sys::cna_picture_collection_destroy_fn, picture_collection_destroy
);
collection!(
    PictureAlbumCollection, PictureAlbum,
    sys::cna_picture_album_collection_get_count_fn, picture_album_collection_get_count,
    sys::cna_picture_album_collection_get_at_fn, picture_album_collection_get_at,
    sys::cna_picture_album_collection_get_is_disposed_fn, picture_album_collection_get_is_disposed,
    sys::cna_picture_album_collection_dispose_fn, picture_album_collection_dispose,
    sys::cna_picture_album_collection_destroy_fn, picture_album_collection_destroy
);
