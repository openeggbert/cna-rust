//! Audited Media/Video calls over the canonical CNA ABI table.

use cna_sys as sys;

use crate::error::Result;

use super::loader::NativeSource;
use super::Native;

/// Every reviewed Media/Video route, resolved once when the tables are filled.
///
/// A slot is kept even where the safe API prefers a richer sibling --
/// `cna_video_player_get_texture` next to `cna_video_player_get_frame_ext` is
/// the current case. Resolving the whole reviewed table up front is what makes
/// a library missing any of it fail at load rather than at the first call that
/// happens to need the missing route.
///
/// These fields used to be `usize` slots that call sites transmuted back into
/// a function type they chose themselves. That put the whole family outside the
/// `SYMBOL_TYPE_MISMATCH` gate -- it checks a field's declared alias against
/// the symbol it acquires, and there was no declared alias to check -- and it
/// made the family the one table a directly linked build could not fill, since
/// a linked declaration is a typed function and not an address.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct MediaApi {
    pub(crate) media_source_get_available_count: sys::cna_media_source_get_available_count_fn,
    pub(crate) media_source_get_type_at: sys::cna_media_source_get_type_at_fn,
    pub(crate) media_source_get_name_size_at: sys::cna_media_source_get_name_size_at_fn,
    pub(crate) media_source_copy_name_at: sys::cna_media_source_copy_name_at_fn,
    pub(crate) song_create_from_uri: sys::cna_song_create_from_uri_fn,
    pub(crate) song_get_name_size: sys::cna_song_get_name_size_fn,
    pub(crate) song_copy_name: sys::cna_song_copy_name_fn,
    pub(crate) song_get_duration: sys::cna_song_get_duration_fn,
    pub(crate) song_get_is_protected: sys::cna_song_get_is_protected_fn,
    pub(crate) song_get_is_rated: sys::cna_song_get_is_rated_fn,
    pub(crate) song_get_play_count: sys::cna_song_get_play_count_fn,
    pub(crate) song_get_rating: sys::cna_song_get_rating_fn,
    pub(crate) song_get_track_number: sys::cna_song_get_track_number_fn,
    pub(crate) song_get_is_disposed: sys::cna_song_get_is_disposed_fn,
    pub(crate) song_dispose: sys::cna_song_dispose_fn,
    pub(crate) song_destroy: sys::cna_song_destroy_fn,
    pub(crate) song_equals: sys::cna_song_equals_fn,
    pub(crate) song_get_hash_code: sys::cna_song_get_hash_code_fn,
    pub(crate) song_get_album: sys::cna_song_get_album_fn,
    pub(crate) song_get_artist: sys::cna_song_get_artist_fn,
    pub(crate) song_get_genre: sys::cna_song_get_genre_fn,
    pub(crate) song_collection_get_at: sys::cna_song_collection_get_at_fn,
    pub(crate) song_collection_get_count: sys::cna_song_collection_get_count_fn,
    pub(crate) song_collection_get_is_disposed: sys::cna_song_collection_get_is_disposed_fn,
    pub(crate) song_collection_dispose: sys::cna_song_collection_dispose_fn,
    pub(crate) song_collection_destroy: sys::cna_song_collection_destroy_fn,
    pub(crate) media_player_get_game_has_control: sys::cna_media_player_get_game_has_control_fn,
    pub(crate) media_player_get_is_muted: sys::cna_media_player_get_is_muted_fn,
    pub(crate) media_player_set_is_muted: sys::cna_media_player_set_is_muted_fn,
    pub(crate) media_player_get_is_repeating: sys::cna_media_player_get_is_repeating_fn,
    pub(crate) media_player_set_is_repeating: sys::cna_media_player_set_is_repeating_fn,
    pub(crate) media_player_get_is_shuffled: sys::cna_media_player_get_is_shuffled_fn,
    pub(crate) media_player_set_is_shuffled: sys::cna_media_player_set_is_shuffled_fn,
    pub(crate) media_player_get_play_position_ticks: sys::cna_media_player_get_play_position_ticks_fn,
    pub(crate) media_player_get_state: sys::cna_media_player_get_state_fn,
    pub(crate) media_player_get_volume: sys::cna_media_player_get_volume_fn,
    pub(crate) media_player_set_volume: sys::cna_media_player_set_volume_fn,
    pub(crate) media_player_get_is_visualization_enabled: sys::cna_media_player_get_is_visualization_enabled_fn,
    pub(crate) media_player_set_is_visualization_enabled: sys::cna_media_player_set_is_visualization_enabled_fn,
    pub(crate) media_player_get_visualization_data: sys::cna_media_player_get_visualization_data_fn,
    pub(crate) media_player_get_queue: sys::cna_media_player_get_queue_fn,
    pub(crate) media_player_play_song: sys::cna_media_player_play_song_fn,
    pub(crate) media_player_play_songs: sys::cna_media_player_play_songs_fn,
    pub(crate) media_player_play_songs_from: sys::cna_media_player_play_songs_from_fn,
    pub(crate) media_player_move_next: sys::cna_media_player_move_next_fn,
    pub(crate) media_player_move_previous: sys::cna_media_player_move_previous_fn,
    pub(crate) media_player_pause: sys::cna_media_player_pause_fn,
    pub(crate) media_player_resume: sys::cna_media_player_resume_fn,
    pub(crate) media_player_stop: sys::cna_media_player_stop_fn,
    pub(crate) media_player_update_ext: sys::cna_media_player_update_ext_fn,
    pub(crate) media_player_program_exit_ext: sys::cna_media_player_program_exit_ext_fn,
    pub(crate) media_player_subscribe_active_song_changed_ext: sys::cna_media_player_subscribe_active_song_changed_ext_fn,
    pub(crate) media_player_subscribe_media_state_changed_ext: sys::cna_media_player_subscribe_media_state_changed_ext_fn,
    pub(crate) media_player_unsubscribe_ext: sys::cna_media_player_unsubscribe_ext_fn,
    pub(crate) media_player_raise_active_song_changed_ext: sys::cna_media_player_raise_active_song_changed_ext_fn,
    pub(crate) media_player_raise_media_state_changed_ext: sys::cna_media_player_raise_media_state_changed_ext_fn,
    pub(crate) media_queue_get_count: sys::cna_media_queue_get_count_fn,
    pub(crate) media_queue_get_active_song_index: sys::cna_media_queue_get_active_song_index_fn,
    pub(crate) media_queue_set_active_song_index: sys::cna_media_queue_set_active_song_index_fn,
    pub(crate) media_queue_get_active_song: sys::cna_media_queue_get_active_song_fn,
    pub(crate) media_queue_get_at: sys::cna_media_queue_get_at_fn,
    pub(crate) media_queue_destroy: sys::cna_media_queue_destroy_fn,
    pub(crate) media_library_create: sys::cna_media_library_create_fn,
    pub(crate) media_library_create_from_source: sys::cna_media_library_create_from_source_fn,
    pub(crate) media_library_get_is_disposed: sys::cna_media_library_get_is_disposed_fn,
    pub(crate) media_library_dispose: sys::cna_media_library_dispose_fn,
    pub(crate) media_library_destroy: sys::cna_media_library_destroy_fn,
    pub(crate) media_library_get_media_source_type: sys::cna_media_library_get_media_source_type_fn,
    pub(crate) media_library_get_media_source_name_size: sys::cna_media_library_get_media_source_name_size_fn,
    pub(crate) media_library_copy_media_source_name: sys::cna_media_library_copy_media_source_name_fn,
    pub(crate) media_library_get_songs: sys::cna_media_library_get_songs_fn,
    pub(crate) media_library_get_albums: sys::cna_media_library_get_albums_fn,
    pub(crate) media_library_get_artists: sys::cna_media_library_get_artists_fn,
    pub(crate) media_library_get_genres: sys::cna_media_library_get_genres_fn,
    pub(crate) media_library_get_playlists: sys::cna_media_library_get_playlists_fn,
    pub(crate) media_library_get_pictures: sys::cna_media_library_get_pictures_fn,
    pub(crate) media_library_get_saved_pictures: sys::cna_media_library_get_saved_pictures_fn,
    pub(crate) media_library_get_root_picture_album: sys::cna_media_library_get_root_picture_album_fn,
    pub(crate) media_library_get_picture_from_token: sys::cna_media_library_get_picture_from_token_fn,
    pub(crate) media_library_save_picture: sys::cna_media_library_save_picture_fn,
    pub(crate) album_get_name_size: sys::cna_album_get_name_size_fn,
    pub(crate) album_copy_name: sys::cna_album_copy_name_fn,
    pub(crate) album_get_is_disposed: sys::cna_album_get_is_disposed_fn,
    pub(crate) album_dispose: sys::cna_album_dispose_fn,
    pub(crate) album_destroy: sys::cna_album_destroy_fn,
    pub(crate) album_get_hash_code: sys::cna_album_get_hash_code_fn,
    pub(crate) album_equals: sys::cna_album_equals_fn,
    pub(crate) album_get_songs: sys::cna_album_get_songs_fn,
    pub(crate) album_get_artist: sys::cna_album_get_artist_fn,
    pub(crate) album_get_genre: sys::cna_album_get_genre_fn,
    pub(crate) album_get_duration: sys::cna_album_get_duration_fn,
    pub(crate) album_get_has_art: sys::cna_album_get_has_art_fn,
    pub(crate) album_get_art_size: sys::cna_album_get_art_size_fn,
    pub(crate) album_copy_art: sys::cna_album_copy_art_fn,
    pub(crate) album_get_thumbnail_size: sys::cna_album_get_thumbnail_size_fn,
    pub(crate) album_copy_thumbnail: sys::cna_album_copy_thumbnail_fn,
    pub(crate) album_collection_get_count: sys::cna_album_collection_get_count_fn,
    pub(crate) album_collection_get_at: sys::cna_album_collection_get_at_fn,
    pub(crate) album_collection_get_is_disposed: sys::cna_album_collection_get_is_disposed_fn,
    pub(crate) album_collection_dispose: sys::cna_album_collection_dispose_fn,
    pub(crate) album_collection_destroy: sys::cna_album_collection_destroy_fn,
    pub(crate) artist_get_name_size: sys::cna_artist_get_name_size_fn,
    pub(crate) artist_copy_name: sys::cna_artist_copy_name_fn,
    pub(crate) artist_get_is_disposed: sys::cna_artist_get_is_disposed_fn,
    pub(crate) artist_dispose: sys::cna_artist_dispose_fn,
    pub(crate) artist_destroy: sys::cna_artist_destroy_fn,
    pub(crate) artist_get_hash_code: sys::cna_artist_get_hash_code_fn,
    pub(crate) artist_equals: sys::cna_artist_equals_fn,
    pub(crate) artist_get_songs: sys::cna_artist_get_songs_fn,
    pub(crate) artist_get_albums: sys::cna_artist_get_albums_fn,
    pub(crate) artist_collection_get_count: sys::cna_artist_collection_get_count_fn,
    pub(crate) artist_collection_get_at: sys::cna_artist_collection_get_at_fn,
    pub(crate) artist_collection_get_is_disposed: sys::cna_artist_collection_get_is_disposed_fn,
    pub(crate) artist_collection_dispose: sys::cna_artist_collection_dispose_fn,
    pub(crate) artist_collection_destroy: sys::cna_artist_collection_destroy_fn,
    pub(crate) genre_get_name_size: sys::cna_genre_get_name_size_fn,
    pub(crate) genre_copy_name: sys::cna_genre_copy_name_fn,
    pub(crate) genre_get_is_disposed: sys::cna_genre_get_is_disposed_fn,
    pub(crate) genre_dispose: sys::cna_genre_dispose_fn,
    pub(crate) genre_destroy: sys::cna_genre_destroy_fn,
    pub(crate) genre_get_hash_code: sys::cna_genre_get_hash_code_fn,
    pub(crate) genre_equals: sys::cna_genre_equals_fn,
    pub(crate) genre_get_songs: sys::cna_genre_get_songs_fn,
    pub(crate) genre_get_albums: sys::cna_genre_get_albums_fn,
    pub(crate) genre_collection_get_count: sys::cna_genre_collection_get_count_fn,
    pub(crate) genre_collection_get_at: sys::cna_genre_collection_get_at_fn,
    pub(crate) genre_collection_get_is_disposed: sys::cna_genre_collection_get_is_disposed_fn,
    pub(crate) genre_collection_dispose: sys::cna_genre_collection_dispose_fn,
    pub(crate) genre_collection_destroy: sys::cna_genre_collection_destroy_fn,
    pub(crate) playlist_get_name_size: sys::cna_playlist_get_name_size_fn,
    pub(crate) playlist_copy_name: sys::cna_playlist_copy_name_fn,
    pub(crate) playlist_get_is_disposed: sys::cna_playlist_get_is_disposed_fn,
    pub(crate) playlist_dispose: sys::cna_playlist_dispose_fn,
    pub(crate) playlist_destroy: sys::cna_playlist_destroy_fn,
    pub(crate) playlist_get_hash_code: sys::cna_playlist_get_hash_code_fn,
    pub(crate) playlist_equals: sys::cna_playlist_equals_fn,
    pub(crate) playlist_get_songs: sys::cna_playlist_get_songs_fn,
    pub(crate) playlist_get_duration: sys::cna_playlist_get_duration_fn,
    pub(crate) playlist_collection_get_count: sys::cna_playlist_collection_get_count_fn,
    pub(crate) playlist_collection_get_at: sys::cna_playlist_collection_get_at_fn,
    pub(crate) playlist_collection_get_is_disposed: sys::cna_playlist_collection_get_is_disposed_fn,
    pub(crate) playlist_collection_dispose: sys::cna_playlist_collection_dispose_fn,
    pub(crate) playlist_collection_destroy: sys::cna_playlist_collection_destroy_fn,
    pub(crate) picture_get_name_size: sys::cna_picture_get_name_size_fn,
    pub(crate) picture_copy_name: sys::cna_picture_copy_name_fn,
    pub(crate) picture_get_album: sys::cna_picture_get_album_fn,
    pub(crate) picture_get_date_unix_ticks: sys::cna_picture_get_date_unix_ticks_fn,
    pub(crate) picture_get_width: sys::cna_picture_get_width_fn,
    pub(crate) picture_get_height: sys::cna_picture_get_height_fn,
    pub(crate) picture_get_image_size: sys::cna_picture_get_image_size_fn,
    pub(crate) picture_copy_image: sys::cna_picture_copy_image_fn,
    pub(crate) picture_get_thumbnail_size: sys::cna_picture_get_thumbnail_size_fn,
    pub(crate) picture_copy_thumbnail: sys::cna_picture_copy_thumbnail_fn,
    pub(crate) picture_get_is_disposed: sys::cna_picture_get_is_disposed_fn,
    pub(crate) picture_dispose: sys::cna_picture_dispose_fn,
    pub(crate) picture_destroy: sys::cna_picture_destroy_fn,
    pub(crate) picture_equals: sys::cna_picture_equals_fn,
    pub(crate) picture_get_hash_code: sys::cna_picture_get_hash_code_fn,
    pub(crate) picture_collection_get_count: sys::cna_picture_collection_get_count_fn,
    pub(crate) picture_collection_get_at: sys::cna_picture_collection_get_at_fn,
    pub(crate) picture_collection_get_is_disposed: sys::cna_picture_collection_get_is_disposed_fn,
    pub(crate) picture_collection_dispose: sys::cna_picture_collection_dispose_fn,
    pub(crate) picture_collection_destroy: sys::cna_picture_collection_destroy_fn,
    pub(crate) picture_album_get_name_size: sys::cna_picture_album_get_name_size_fn,
    pub(crate) picture_album_copy_name: sys::cna_picture_album_copy_name_fn,
    pub(crate) picture_album_get_parent: sys::cna_picture_album_get_parent_fn,
    pub(crate) picture_album_get_albums: sys::cna_picture_album_get_albums_fn,
    pub(crate) picture_album_get_pictures: sys::cna_picture_album_get_pictures_fn,
    pub(crate) picture_album_get_is_disposed: sys::cna_picture_album_get_is_disposed_fn,
    pub(crate) picture_album_dispose: sys::cna_picture_album_dispose_fn,
    pub(crate) picture_album_destroy: sys::cna_picture_album_destroy_fn,
    pub(crate) picture_album_equals: sys::cna_picture_album_equals_fn,
    pub(crate) picture_album_get_hash_code: sys::cna_picture_album_get_hash_code_fn,
    pub(crate) picture_album_collection_get_count: sys::cna_picture_album_collection_get_count_fn,
    pub(crate) picture_album_collection_get_at: sys::cna_picture_album_collection_get_at_fn,
    pub(crate) picture_album_collection_get_is_disposed: sys::cna_picture_album_collection_get_is_disposed_fn,
    pub(crate) picture_album_collection_dispose: sys::cna_picture_album_collection_dispose_fn,
    pub(crate) picture_album_collection_destroy: sys::cna_picture_album_collection_destroy_fn,
    pub(crate) video_create_with_metadata: sys::cna_video_create_with_metadata_fn,
    pub(crate) video_get_width: sys::cna_video_get_width_fn,
    pub(crate) video_get_height: sys::cna_video_get_height_fn,
    pub(crate) video_get_frames_per_second: sys::cna_video_get_frames_per_second_fn,
    pub(crate) video_get_duration: sys::cna_video_get_duration_fn,
    pub(crate) video_get_soundtrack_type: sys::cna_video_get_soundtrack_type_fn,
    pub(crate) video_destroy: sys::cna_video_destroy_fn,
    pub(crate) video_player_create: sys::cna_video_player_create_fn,
    pub(crate) video_player_get_is_disposed: sys::cna_video_player_get_is_disposed_fn,
    pub(crate) video_player_get_is_looped: sys::cna_video_player_get_is_looped_fn,
    pub(crate) video_player_set_is_looped: sys::cna_video_player_set_is_looped_fn,
    pub(crate) video_player_get_is_muted: sys::cna_video_player_get_is_muted_fn,
    pub(crate) video_player_set_is_muted: sys::cna_video_player_set_is_muted_fn,
    pub(crate) video_player_get_play_position_ticks: sys::cna_video_player_get_play_position_ticks_fn,
    pub(crate) video_player_get_state: sys::cna_video_player_get_state_fn,
    pub(crate) video_player_get_volume: sys::cna_video_player_get_volume_fn,
    pub(crate) video_player_set_volume: sys::cna_video_player_set_volume_fn,
    pub(crate) video_player_get_frame_ext: sys::cna_video_player_get_frame_ext_fn,
    pub(crate) video_player_get_texture: sys::cna_video_player_get_texture_fn,
    pub(crate) video_player_play: sys::cna_video_player_play_fn,
    pub(crate) video_player_stop: sys::cna_video_player_stop_fn,
    pub(crate) video_player_pause: sys::cna_video_player_pause_fn,
    pub(crate) video_player_resume: sys::cna_video_player_resume_fn,
    pub(crate) video_player_dispose: sys::cna_video_player_dispose_fn,
    pub(crate) video_player_destroy: sys::cna_video_player_destroy_fn,
}

impl MediaApi {
    pub(super) fn load(source: &NativeSource) -> Result<Self> {
        macro_rules! symbol {
            ($name:ident, $ty:ty) => {
                super::loader::acquire!(source, $name, $ty)
            };
        }
        Ok(Self {
            media_source_get_available_count: symbol!(cna_media_source_get_available_count, _),
            media_source_get_type_at: symbol!(cna_media_source_get_type_at, _),
            media_source_get_name_size_at: symbol!(cna_media_source_get_name_size_at, _),
            media_source_copy_name_at: symbol!(cna_media_source_copy_name_at, _),
            song_create_from_uri: symbol!(cna_song_create_from_uri, _),
            song_get_name_size: symbol!(cna_song_get_name_size, _),
            song_copy_name: symbol!(cna_song_copy_name, _),
            song_get_duration: symbol!(cna_song_get_duration, _),
            song_get_is_protected: symbol!(cna_song_get_is_protected, _),
            song_get_is_rated: symbol!(cna_song_get_is_rated, _),
            song_get_play_count: symbol!(cna_song_get_play_count, _),
            song_get_rating: symbol!(cna_song_get_rating, _),
            song_get_track_number: symbol!(cna_song_get_track_number, _),
            song_get_is_disposed: symbol!(cna_song_get_is_disposed, _),
            song_dispose: symbol!(cna_song_dispose, _),
            song_destroy: symbol!(cna_song_destroy, _),
            song_equals: symbol!(cna_song_equals, _),
            song_get_hash_code: symbol!(cna_song_get_hash_code, _),
            song_get_album: symbol!(cna_song_get_album, _),
            song_get_artist: symbol!(cna_song_get_artist, _),
            song_get_genre: symbol!(cna_song_get_genre, _),
            song_collection_get_at: symbol!(cna_song_collection_get_at, _),
            song_collection_get_count: symbol!(cna_song_collection_get_count, _),
            song_collection_get_is_disposed: symbol!(cna_song_collection_get_is_disposed, _),
            song_collection_dispose: symbol!(cna_song_collection_dispose, _),
            song_collection_destroy: symbol!(cna_song_collection_destroy, _),
            media_player_get_game_has_control: symbol!(cna_media_player_get_game_has_control, _),
            media_player_get_is_muted: symbol!(cna_media_player_get_is_muted, _),
            media_player_set_is_muted: symbol!(cna_media_player_set_is_muted, _),
            media_player_get_is_repeating: symbol!(cna_media_player_get_is_repeating, _),
            media_player_set_is_repeating: symbol!(cna_media_player_set_is_repeating, _),
            media_player_get_is_shuffled: symbol!(cna_media_player_get_is_shuffled, _),
            media_player_set_is_shuffled: symbol!(cna_media_player_set_is_shuffled, _),
            media_player_get_play_position_ticks: symbol!(cna_media_player_get_play_position_ticks, _),
            media_player_get_state: symbol!(cna_media_player_get_state, _),
            media_player_get_volume: symbol!(cna_media_player_get_volume, _),
            media_player_set_volume: symbol!(cna_media_player_set_volume, _),
            media_player_get_is_visualization_enabled: symbol!(cna_media_player_get_is_visualization_enabled, _),
            media_player_set_is_visualization_enabled: symbol!(cna_media_player_set_is_visualization_enabled, _),
            media_player_get_visualization_data: symbol!(cna_media_player_get_visualization_data, _),
            media_player_get_queue: symbol!(cna_media_player_get_queue, _),
            media_player_play_song: symbol!(cna_media_player_play_song, _),
            media_player_play_songs: symbol!(cna_media_player_play_songs, _),
            media_player_play_songs_from: symbol!(cna_media_player_play_songs_from, _),
            media_player_move_next: symbol!(cna_media_player_move_next, _),
            media_player_move_previous: symbol!(cna_media_player_move_previous, _),
            media_player_pause: symbol!(cna_media_player_pause, _),
            media_player_resume: symbol!(cna_media_player_resume, _),
            media_player_stop: symbol!(cna_media_player_stop, _),
            media_player_update_ext: symbol!(cna_media_player_update_ext, _),
            media_player_program_exit_ext: symbol!(cna_media_player_program_exit_ext, _),
            media_player_subscribe_active_song_changed_ext: symbol!(cna_media_player_subscribe_active_song_changed_ext, _),
            media_player_subscribe_media_state_changed_ext: symbol!(cna_media_player_subscribe_media_state_changed_ext, _),
            media_player_unsubscribe_ext: symbol!(cna_media_player_unsubscribe_ext, _),
            media_player_raise_active_song_changed_ext: symbol!(cna_media_player_raise_active_song_changed_ext, _),
            media_player_raise_media_state_changed_ext: symbol!(cna_media_player_raise_media_state_changed_ext, _),
            media_queue_get_count: symbol!(cna_media_queue_get_count, _),
            media_queue_get_active_song_index: symbol!(cna_media_queue_get_active_song_index, _),
            media_queue_set_active_song_index: symbol!(cna_media_queue_set_active_song_index, _),
            media_queue_get_active_song: symbol!(cna_media_queue_get_active_song, _),
            media_queue_get_at: symbol!(cna_media_queue_get_at, _),
            media_queue_destroy: symbol!(cna_media_queue_destroy, _),
            media_library_create: symbol!(cna_media_library_create, _),
            media_library_create_from_source: symbol!(cna_media_library_create_from_source, _),
            media_library_get_is_disposed: symbol!(cna_media_library_get_is_disposed, _),
            media_library_dispose: symbol!(cna_media_library_dispose, _),
            media_library_destroy: symbol!(cna_media_library_destroy, _),
            media_library_get_media_source_type: symbol!(cna_media_library_get_media_source_type, _),
            media_library_get_media_source_name_size: symbol!(cna_media_library_get_media_source_name_size, _),
            media_library_copy_media_source_name: symbol!(cna_media_library_copy_media_source_name, _),
            media_library_get_songs: symbol!(cna_media_library_get_songs, _),
            media_library_get_albums: symbol!(cna_media_library_get_albums, _),
            media_library_get_artists: symbol!(cna_media_library_get_artists, _),
            media_library_get_genres: symbol!(cna_media_library_get_genres, _),
            media_library_get_playlists: symbol!(cna_media_library_get_playlists, _),
            media_library_get_pictures: symbol!(cna_media_library_get_pictures, _),
            media_library_get_saved_pictures: symbol!(cna_media_library_get_saved_pictures, _),
            media_library_get_root_picture_album: symbol!(cna_media_library_get_root_picture_album, _),
            media_library_get_picture_from_token: symbol!(cna_media_library_get_picture_from_token, _),
            media_library_save_picture: symbol!(cna_media_library_save_picture, _),
            album_get_name_size: symbol!(cna_album_get_name_size, _),
            album_copy_name: symbol!(cna_album_copy_name, _),
            album_get_is_disposed: symbol!(cna_album_get_is_disposed, _),
            album_dispose: symbol!(cna_album_dispose, _),
            album_destroy: symbol!(cna_album_destroy, _),
            album_get_hash_code: symbol!(cna_album_get_hash_code, _),
            album_equals: symbol!(cna_album_equals, _),
            album_get_songs: symbol!(cna_album_get_songs, _),
            album_get_artist: symbol!(cna_album_get_artist, _),
            album_get_genre: symbol!(cna_album_get_genre, _),
            album_get_duration: symbol!(cna_album_get_duration, _),
            album_get_has_art: symbol!(cna_album_get_has_art, _),
            album_get_art_size: symbol!(cna_album_get_art_size, _),
            album_copy_art: symbol!(cna_album_copy_art, _),
            album_get_thumbnail_size: symbol!(cna_album_get_thumbnail_size, _),
            album_copy_thumbnail: symbol!(cna_album_copy_thumbnail, _),
            album_collection_get_count: symbol!(cna_album_collection_get_count, _),
            album_collection_get_at: symbol!(cna_album_collection_get_at, _),
            album_collection_get_is_disposed: symbol!(cna_album_collection_get_is_disposed, _),
            album_collection_dispose: symbol!(cna_album_collection_dispose, _),
            album_collection_destroy: symbol!(cna_album_collection_destroy, _),
            artist_get_name_size: symbol!(cna_artist_get_name_size, _),
            artist_copy_name: symbol!(cna_artist_copy_name, _),
            artist_get_is_disposed: symbol!(cna_artist_get_is_disposed, _),
            artist_dispose: symbol!(cna_artist_dispose, _),
            artist_destroy: symbol!(cna_artist_destroy, _),
            artist_get_hash_code: symbol!(cna_artist_get_hash_code, _),
            artist_equals: symbol!(cna_artist_equals, _),
            artist_get_songs: symbol!(cna_artist_get_songs, _),
            artist_get_albums: symbol!(cna_artist_get_albums, _),
            artist_collection_get_count: symbol!(cna_artist_collection_get_count, _),
            artist_collection_get_at: symbol!(cna_artist_collection_get_at, _),
            artist_collection_get_is_disposed: symbol!(cna_artist_collection_get_is_disposed, _),
            artist_collection_dispose: symbol!(cna_artist_collection_dispose, _),
            artist_collection_destroy: symbol!(cna_artist_collection_destroy, _),
            genre_get_name_size: symbol!(cna_genre_get_name_size, _),
            genre_copy_name: symbol!(cna_genre_copy_name, _),
            genre_get_is_disposed: symbol!(cna_genre_get_is_disposed, _),
            genre_dispose: symbol!(cna_genre_dispose, _),
            genre_destroy: symbol!(cna_genre_destroy, _),
            genre_get_hash_code: symbol!(cna_genre_get_hash_code, _),
            genre_equals: symbol!(cna_genre_equals, _),
            genre_get_songs: symbol!(cna_genre_get_songs, _),
            genre_get_albums: symbol!(cna_genre_get_albums, _),
            genre_collection_get_count: symbol!(cna_genre_collection_get_count, _),
            genre_collection_get_at: symbol!(cna_genre_collection_get_at, _),
            genre_collection_get_is_disposed: symbol!(cna_genre_collection_get_is_disposed, _),
            genre_collection_dispose: symbol!(cna_genre_collection_dispose, _),
            genre_collection_destroy: symbol!(cna_genre_collection_destroy, _),
            playlist_get_name_size: symbol!(cna_playlist_get_name_size, _),
            playlist_copy_name: symbol!(cna_playlist_copy_name, _),
            playlist_get_is_disposed: symbol!(cna_playlist_get_is_disposed, _),
            playlist_dispose: symbol!(cna_playlist_dispose, _),
            playlist_destroy: symbol!(cna_playlist_destroy, _),
            playlist_get_hash_code: symbol!(cna_playlist_get_hash_code, _),
            playlist_equals: symbol!(cna_playlist_equals, _),
            playlist_get_songs: symbol!(cna_playlist_get_songs, _),
            playlist_get_duration: symbol!(cna_playlist_get_duration, _),
            playlist_collection_get_count: symbol!(cna_playlist_collection_get_count, _),
            playlist_collection_get_at: symbol!(cna_playlist_collection_get_at, _),
            playlist_collection_get_is_disposed: symbol!(cna_playlist_collection_get_is_disposed, _),
            playlist_collection_dispose: symbol!(cna_playlist_collection_dispose, _),
            playlist_collection_destroy: symbol!(cna_playlist_collection_destroy, _),
            picture_get_name_size: symbol!(cna_picture_get_name_size, _),
            picture_copy_name: symbol!(cna_picture_copy_name, _),
            picture_get_album: symbol!(cna_picture_get_album, _),
            picture_get_date_unix_ticks: symbol!(cna_picture_get_date_unix_ticks, _),
            picture_get_width: symbol!(cna_picture_get_width, _),
            picture_get_height: symbol!(cna_picture_get_height, _),
            picture_get_image_size: symbol!(cna_picture_get_image_size, _),
            picture_copy_image: symbol!(cna_picture_copy_image, _),
            picture_get_thumbnail_size: symbol!(cna_picture_get_thumbnail_size, _),
            picture_copy_thumbnail: symbol!(cna_picture_copy_thumbnail, _),
            picture_get_is_disposed: symbol!(cna_picture_get_is_disposed, _),
            picture_dispose: symbol!(cna_picture_dispose, _),
            picture_destroy: symbol!(cna_picture_destroy, _),
            picture_equals: symbol!(cna_picture_equals, _),
            picture_get_hash_code: symbol!(cna_picture_get_hash_code, _),
            picture_collection_get_count: symbol!(cna_picture_collection_get_count, _),
            picture_collection_get_at: symbol!(cna_picture_collection_get_at, _),
            picture_collection_get_is_disposed: symbol!(cna_picture_collection_get_is_disposed, _),
            picture_collection_dispose: symbol!(cna_picture_collection_dispose, _),
            picture_collection_destroy: symbol!(cna_picture_collection_destroy, _),
            picture_album_get_name_size: symbol!(cna_picture_album_get_name_size, _),
            picture_album_copy_name: symbol!(cna_picture_album_copy_name, _),
            picture_album_get_parent: symbol!(cna_picture_album_get_parent, _),
            picture_album_get_albums: symbol!(cna_picture_album_get_albums, _),
            picture_album_get_pictures: symbol!(cna_picture_album_get_pictures, _),
            picture_album_get_is_disposed: symbol!(cna_picture_album_get_is_disposed, _),
            picture_album_dispose: symbol!(cna_picture_album_dispose, _),
            picture_album_destroy: symbol!(cna_picture_album_destroy, _),
            picture_album_equals: symbol!(cna_picture_album_equals, _),
            picture_album_get_hash_code: symbol!(cna_picture_album_get_hash_code, _),
            picture_album_collection_get_count: symbol!(cna_picture_album_collection_get_count, _),
            picture_album_collection_get_at: symbol!(cna_picture_album_collection_get_at, _),
            picture_album_collection_get_is_disposed: symbol!(cna_picture_album_collection_get_is_disposed, _),
            picture_album_collection_dispose: symbol!(cna_picture_album_collection_dispose, _),
            picture_album_collection_destroy: symbol!(cna_picture_album_collection_destroy, _),
            video_create_with_metadata: symbol!(cna_video_create_with_metadata, _),
            video_get_width: symbol!(cna_video_get_width, _),
            video_get_height: symbol!(cna_video_get_height, _),
            video_get_frames_per_second: symbol!(cna_video_get_frames_per_second, _),
            video_get_duration: symbol!(cna_video_get_duration, _),
            video_get_soundtrack_type: symbol!(cna_video_get_soundtrack_type, _),
            video_destroy: symbol!(cna_video_destroy, _),
            video_player_create: symbol!(cna_video_player_create, _),
            video_player_get_is_disposed: symbol!(cna_video_player_get_is_disposed, _),
            video_player_get_is_looped: symbol!(cna_video_player_get_is_looped, _),
            video_player_set_is_looped: symbol!(cna_video_player_set_is_looped, _),
            video_player_get_is_muted: symbol!(cna_video_player_get_is_muted, _),
            video_player_set_is_muted: symbol!(cna_video_player_set_is_muted, _),
            video_player_get_play_position_ticks: symbol!(cna_video_player_get_play_position_ticks, _),
            video_player_get_state: symbol!(cna_video_player_get_state, _),
            video_player_get_volume: symbol!(cna_video_player_get_volume, _),
            video_player_set_volume: symbol!(cna_video_player_set_volume, _),
            video_player_get_frame_ext: symbol!(cna_video_player_get_frame_ext, _),
            video_player_get_texture: symbol!(cna_video_player_get_texture, _),
            video_player_play: symbol!(cna_video_player_play, _),
            video_player_stop: symbol!(cna_video_player_stop, _),
            video_player_pause: symbol!(cna_video_player_pause, _),
            video_player_resume: symbol!(cna_video_player_resume, _),
            video_player_dispose: symbol!(cna_video_player_dispose, _),
            video_player_destroy: symbol!(cna_video_player_destroy, _),
        })
    }
}

/// `video.h` and `media.h`: constructing a `Video` or a `Song`, and the facts
/// they carry that no other route reports.
///
/// The construction routes are what let a game hold media it built rather than
/// media the library enumerated. XNA's `Video` and `Song` are content types
/// with no public constructor; CNA gives them one, and a Rust caller that has a
/// file path or a URI has nowhere else to go.
///
/// The track selectors exist twice on purpose -- once on the video and once on
/// the player -- and they are not the same operation: setting a track on the
/// *video* changes what any player will use for it, and setting one on the
/// *player* changes only that player's current playback.
impl Native {
    pub(crate) fn create_video(
        &self,
        game: sys::CNA_Handle,
        file_name: &str,
    ) -> Result<sys::CNA_VideoHandle> {
        let mut handle = 0;
        // SAFETY: the game handle is live and the name outlives the call.
        self.check(unsafe {
            (self.video_create)(game, media_view(file_name), &mut handle)
        })?;
        Ok(handle)
    }

    pub(crate) fn create_video_from_uri(
        &self,
        game: sys::CNA_Handle,
        uri: &str,
    ) -> Result<sys::CNA_VideoHandle> {
        let mut handle = 0;
        // SAFETY: the game handle is live and the URI outlives the call.
        self.check(unsafe {
            (self.video_create_from_uri_ext)(game, media_view(uri), &mut handle)
        })?;
        Ok(handle)
    }

    /// Width, height and frame rate, from the decoded stream.
    pub(crate) fn video_info(
        &self,
        handle: sys::CNA_VideoHandle,
    ) -> Result<(i32, i32, f64)> {
        let mut info = sys::CNA_VideoInfo {
            struct_size: core::mem::size_of::<sys::CNA_VideoInfo>() as u32,
            struct_version: 1,
            ..sys::CNA_VideoInfo::default()
        };
        // SAFETY: the output is a complete versioned local.
        self.check(unsafe { (self.video_get_info)(handle, &mut info) })?;
        Ok((info.width, info.height, info.fps))
    }

    pub(crate) fn set_video_duration(
        &self,
        handle: sys::CNA_VideoHandle,
        ticks: i64,
    ) -> Result<()> {
        // SAFETY: the handle is live and the duration is a scalar.
        self.check(unsafe { (self.video_set_duration)(handle, ticks) })
    }

    pub(crate) fn set_video_audio_track(
        &self,
        handle: sys::CNA_VideoHandle,
        track: i32,
    ) -> Result<()> {
        // SAFETY: the handle is live and the track is a scalar.
        self.check(unsafe { (self.video_set_audio_track_ext)(handle, track) })
    }

    pub(crate) fn set_video_video_track(
        &self,
        handle: sys::CNA_VideoHandle,
        track: i32,
    ) -> Result<()> {
        // SAFETY: the handle is live and the track is a scalar.
        self.check(unsafe { (self.video_set_video_track_ext)(handle, track) })
    }

    pub(crate) fn video_file_name(&self, handle: sys::CNA_VideoHandle) -> Result<String> {
        media_text(
            |result| self.check(result),
            // SAFETY: the handle is live and the output is the caller's local.
            |out| unsafe { (self.video_get_file_name_size)(handle, out) },
            // SAFETY: the destination has the capacity just measured.
            |destination, capacity, written| unsafe {
                (self.video_copy_file_name)(handle, destination, capacity, written)
            },
        )
    }

    pub(crate) fn video_has_graphics_device(
        &self,
        handle: sys::CNA_VideoHandle,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the handle is live and the output is a local.
        self.check(unsafe { (self.video_get_has_graphics_device)(handle, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    /// The video a player is currently bound to, if any.
    pub(crate) fn video_player_video(
        &self,
        handle: sys::CNA_VideoPlayerHandle,
    ) -> Result<Option<sys::CNA_VideoHandle>> {
        let mut video = 0;
        let mut present = sys::CNA_FALSE;
        // SAFETY: the handle is live and both outputs are locals.
        self.check(unsafe { (self.video_player_get_video)(handle, &mut video, &mut present) })?;
        Ok((present != sys::CNA_FALSE).then_some(video))
    }

    pub(crate) fn set_player_audio_track(
        &self,
        handle: sys::CNA_VideoPlayerHandle,
        track: i32,
    ) -> Result<()> {
        // SAFETY: the handle is live and the track is a scalar.
        self.check(unsafe { (self.video_player_set_audio_track_ext)(handle, track) })
    }

    pub(crate) fn set_player_video_track(
        &self,
        handle: sys::CNA_VideoPlayerHandle,
        track: i32,
    ) -> Result<()> {
        // SAFETY: the handle is live and the track is a scalar.
        self.check(unsafe { (self.video_player_set_video_track_ext)(handle, track) })
    }

    pub(crate) fn media_source_type_name(
        &self,
        game: sys::CNA_Handle,
        index: u32,
    ) -> Result<String> {
        media_text(
            |result| self.check(result),
            // SAFETY: the game handle is live and the output is a local.
            |out| unsafe { (self.media_source_get_type_name_size_at)(game, index, out) },
            // SAFETY: the destination has the capacity just measured.
            |destination, capacity, written| unsafe {
                (self.media_source_copy_type_name_at)(
                    game, index, destination, capacity, written,
                )
            },
        )
    }

    /// Builds a song from a file path and a display name.
    ///
    /// The C route takes the **file name first** and the display name second,
    /// which is the opposite of how the Rust signature reads. Getting it the
    /// wrong way round is silent -- both are strings and either can be empty --
    /// so the order is stated here rather than left to the argument names.
    pub(crate) fn create_song(
        &self,
        game: sys::CNA_Handle,
        name: &str,
        file_name: &str,
    ) -> Result<sys::CNA_SongHandle> {
        let mut handle = 0;
        // SAFETY: the game handle is live and both strings outlive the call.
        self.check(unsafe {
            (self.song_create)(game, media_view(file_name), media_view(name), &mut handle)
        })?;
        Ok(handle)
    }

    pub(crate) fn create_song_with_duration(
        &self,
        game: sys::CNA_Handle,
        name: &str,
        file_name: &str,
        duration_milliseconds: i32,
    ) -> Result<sys::CNA_SongHandle> {
        let mut handle = 0;
        // SAFETY: the game handle is live and both strings outlive the call.
        self.check(unsafe {
            (self.song_create_with_duration)(
                game,
                media_view(file_name),
                media_view(name),
                duration_milliseconds,
                &mut handle,
            )
        })?;
        Ok(handle)
    }

    /// The platform handle text a song carries, for diagnostics.
    pub(crate) fn song_handle_text(&self, handle: sys::CNA_SongHandle) -> Result<String> {
        media_text(
            |result| self.check(result),
            // SAFETY: the handle is live and the output is a local.
            |out| unsafe { (self.song_get_handle_text_size_ext)(handle, out) },
            // SAFETY: the destination has the capacity just measured.
            |destination, capacity, written| unsafe {
                (self.song_copy_handle_text_ext)(handle, destination, capacity, written)
            },
        )
    }

    pub(crate) fn set_song_duration(
        &self,
        handle: sys::CNA_SongHandle,
        ticks: i64,
    ) -> Result<()> {
        // SAFETY: the handle is live and the duration is a scalar.
        self.check(unsafe { (self.song_set_duration)(handle, ticks) })
    }

    pub(crate) fn set_song_play_count(
        &self,
        handle: sys::CNA_SongHandle,
        count: i32,
    ) -> Result<()> {
        // SAFETY: the handle is live and the count is a scalar.
        self.check(unsafe { (self.song_set_play_count)(handle, count) })
    }

    pub(crate) fn create_song_collection(
        &self,
        game: sys::CNA_Handle,
        songs: &[sys::CNA_SongHandle],
    ) -> Result<sys::CNA_SongCollectionHandle> {
        let mut handle = 0;
        // SAFETY: the slice outlives the call and its length is passed exactly;
        // a null pointer is only passed with a zero count.
        self.check(unsafe {
            (self.song_collection_create)(
                game,
                if songs.is_empty() {
                    core::ptr::null()
                } else {
                    songs.as_ptr()
                },
                songs.len() as u64,
                &mut handle,
            )
        })?;
        Ok(handle)
    }
}

fn media_view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast(),
        byte_length: value.len() as u64,
    }
}

fn media_text(
    check: impl Fn(sys::CNA_Result) -> Result<()>,
    size: impl Fn(*mut u64) -> sys::CNA_Result,
    copy: impl Fn(*mut core::ffi::c_char, u64, *mut u64) -> sys::CNA_Result,
) -> Result<String> {
    super::runtime::read_string(check, size, copy)
}
