//! CNA-only deterministic Media callback and video-frame hooks.

#![allow(non_snake_case)]

use std::sync::Arc;

use crate::game::{GameContext, TimeSpan};
use crate::media::{MediaPlayer, Picture, Song, VideoPlayer};
use crate::Result;

pub fn RaiseActiveSongChanged(game: &GameContext<'_>) -> Result<()> {
    MediaPlayer::raise_active_song_changed(game)
}

pub fn RaiseMediaStateChanged(game: &GameContext<'_>) -> Result<()> {
    MediaPlayer::raise_media_state_changed(game)
}

/// Stops process-global playback from inside an owner-thread
/// `MediaPlayer` event handler. It fails outside that dispatch scope.
pub fn StopFromEvent() -> Result<()> {
    MediaPlayer::stop_from_event()
}

/// Starts a live same-generation Song from inside an owner-thread
/// `MediaPlayer` event handler. It fails outside that dispatch scope.
pub fn PlayFromEvent(song: &Song) -> Result<()> {
    MediaPlayer::play_from_event(song)
}

/// Frames this `VideoPlayer` has decoded, zero before the first.
///
/// XNA has no counterpart. It owns two frame textures and alternates
/// between them, so an XNA caller detects a new frame by object
/// identity; CNA decodes into one texture in place and publishes this
/// counter instead. It is monotonic for the player's lifetime and is
/// never restarted by `Stop` or by playing a different `Video`, so two
/// equal readings mean the same pixels.
///
/// Reading it is itself a call on the player, which invalidates any
/// `Texture2D` a previous `GetTexture` returned.
pub fn VideoFrameGeneration(player: &VideoPlayer) -> Result<u64> {
    player.frame_generation()
}

/// Presentation timestamp in seconds of the frame the player holds, or
/// `None` when it holds none. Reading it invalidates an outstanding
/// frame `Texture2D` for the same reason as [`VideoFrameGeneration`].
pub fn VideoFramePresentationTime(player: &VideoPlayer) -> Result<Option<f64>> {
    player.frame_presentation_time()
}

/// The `video.h` player routes with no XNA counterpart.
///
/// A CNA extension: import it to call these.
///
/// ```rust,ignore
/// use cna::extensions::media::VideoPlayerExt;
/// player.SetAudioTrack(1)?;
/// ```
pub trait VideoPlayerExt {
    /// Whether CNA reports this player bound to a video.
    ///
    /// The Rust side already remembers what it was handed; this asks CNA, which
    /// is the answer that survives a video being disposed underneath the
    /// player.
    fn HasNativeVideo(&self) -> Result<bool>;

    /// Selects the audio track for this player's current playback only.
    fn SetAudioTrack(&self, track: i32) -> Result<()>;

    /// Selects the video track for this player's current playback only.
    fn SetVideoTrack(&self, track: i32) -> Result<()>;
}

/// The `video.h` routes with no XNA counterpart.
///
/// XNA's `Video` is a content type with no public constructor: a game gets one
/// by loading it. CNA gives it two, from a file path and from a URI, and a Rust
/// caller holding either has nowhere else to go -- so they are bound.
///
/// The track selectors exist on both the video and the player and are not the
/// same operation. Setting a track on the *video* changes what any player will
/// use for it; setting one on the *player* changes only that player's current
/// playback, and is lost when it moves to another video.
///
/// A CNA extension: import it to call these, including the two producers --
/// an associated function on a trait resolves through `Video::` once the trait
/// is in scope.
///
/// ```rust,ignore
/// use cna::extensions::media::VideoExt;
/// let video = Video::FromFile(game, "intro.ogv")?;
/// ```
pub trait VideoExt: Sized {
    /// Opens a video from a file the platform can decode.
    fn FromFile(game: &GameContext<'_>, fileName: &str) -> Result<Self>;

    /// Opens a video from a URI.
    fn FromUri(game: &GameContext<'_>, uri: &str) -> Result<Self>;

    /// Frame width, height and rate, as the decoded stream reports them.
    ///
    /// Distinct from the `Width`, `Height` and `FramesPerSecond` properties a
    /// content-loaded video carries: those are the metadata the pipeline wrote
    /// down, and this is what the decoder found.
    fn DecodedInfo(&self) -> Result<(i32, i32, f64)>;

    /// Overrides the duration the video reports.
    fn SetDuration(&self, value: TimeSpan) -> Result<()>;

    /// Selects which audio track any player should use for this video.
    fn SetAudioTrack(&self, track: i32) -> Result<()>;

    /// Selects which video track any player should use for this video.
    fn SetVideoTrack(&self, track: i32) -> Result<()>;

    /// The file name this video was opened from.
    fn FileName(&self) -> Result<String>;

    /// Whether the video holds a graphics device for frame delivery.
    ///
    /// A video opened without one decodes but cannot hand back a texture, so
    /// this is what distinguishes "no frame yet" from "no frames ever".
    fn HasGraphicsDevice(&self) -> Result<bool>;
}

/// The `media_library.h` route that saves a picture from a `StorageStream`.
///
/// XNA's two `SavePicture` overloads take a byte array and a `System.IO.Stream`,
/// and the strict projection carries both. This is CNA's third: it takes the
/// storage stream a game that just wrote a screenshot already has open, which
/// is a CNA type and so cannot be an XNA overload.
///
/// A CNA extension: import it to call this.
pub trait MediaLibraryExt {
    /// Saves a picture the caller already has open as a stream.
    ///
    /// XNA's `SavePicture` takes a byte array or a stream and is one of the few
    /// media routes a game can *write* through. This is the stream form, which
    /// is what a game that just rendered a screenshot to a storage file has in
    /// hand.
    fn SavePictureFromStream(
        &self,
        name: &str,
        stream: &crate::storage::StorageStream,
    ) -> Result<Arc<Picture>>;
}

/// The `media_library.h` route with no XNA counterpart.
///
/// A CNA extension: import it to call this.
pub trait PictureExt {
    /// The platform token this picture carries.
    ///
    /// The backend's own identifier for the underlying media object, and the
    /// only way to tell two pictures with the same name apart.
    fn PlatformToken(&self) -> Result<String>;
}

/// The `media.h` route that names a source's *type*.
///
/// A CNA extension: import it to call this.
pub trait MediaSourceExt {
    /// The runtime type name CNA reports for this source.
    ///
    /// Measured, and narrower than the name suggests: it is the .NET *class*
    /// name -- `"Microsoft.Xna.Framework.Media.MediaSource"` -- and not a
    /// spelling of which kind of source this is. `MediaSourceType` is what
    /// answers that, and `Name` is what the source itself is called.
    ///
    /// It is bound because it is the only route that reports the type name at
    /// all, and a caller writing a diagnostic wants what CNA would print; it is
    /// documented this way so nobody reaches for it expecting the kind.
    ///
    /// Answers `None` for a source this process did not enumerate, which is
    /// the state a source has after its game generation ended.
    fn TypeName(&self, game: &GameContext<'_>) -> Result<Option<String>>;
}

/// The `media.h` route that builds a `SongCollection`.
///
/// XNA has none: a game gets a collection from the media library. CNA lets a
/// caller build one over songs already in hand.
///
/// A CNA extension: import it to call this.
pub trait SongCollectionExt: Sized {
    /// Builds a collection over songs the caller already holds.
    ///
    /// The songs are read during the call; the collection does not borrow them
    /// afterwards, so they may be dropped independently of it.
    fn FromSongs(game: &GameContext<'_>, songs: &[&Song]) -> Result<Arc<Self>>;
}

/// A `Song`'s CNA-only surface.
///
/// XNA's `Song` has one public producer -- `Song.FromUri` -- and no way to name
/// a file directly. CNA gives two, and a Rust caller with a file on disk has
/// nowhere else to go.
///
/// `SetDuration` and `SetPlayCount` are writers for properties XNA exposes
/// read-only. They exist because the library a song came from may not know
/// either, and the values are the game's to correct.
///
/// A CNA extension: import it to call these, the producers included -- an
/// associated function on a trait resolves through `Song::` once the trait is
/// in scope.
///
/// ```rust,ignore
/// use cna::extensions::media::SongExt;
/// let song = Song::FromFile(game, "theme", "theme.ogg")?;
/// let handle = song.HandleText()?;
/// ```
pub trait SongExt: Sized {
    /// Builds a song from a name and a file the platform can decode.
    fn FromFile(game: &GameContext<'_>, name: &str, fileName: &str) -> Result<Self>;

    /// Builds a song that reports the duration it was given.
    fn FromFileWithDuration(
        game: &GameContext<'_>,
        name: &str,
        fileName: &str,
        durationMilliseconds: i32,
    ) -> Result<Self>;

    /// The platform handle text this song carries.
    ///
    /// A diagnostic string rather than a property: what the backend calls the
    /// underlying media object. It is the only way to tell two songs with the
    /// same name apart when a library has produced both.
    fn HandleText(&self) -> Result<String>;

    /// Overrides the duration this song reports.
    fn SetDuration(&self, value: TimeSpan) -> Result<()>;

    /// Overrides the play count this song reports.
    fn SetPlayCount(&self, value: i32) -> Result<()>;

    /// Whether CNA considers a song of this duration ended after `elapsed`.
    ///
    /// `media_player.h` rather than `media.h`, and here so that a song's whole
    /// CNA-only surface is one trait. Some backends never raise a song-ended
    /// notification, so a player that needs `MediaStateChanged` to fire has to
    /// watch the clock. *When* a song counts as ended is CNA's rule, and asking
    /// it is what keeps a hand-driven player and an event-driven one agreeing.
    fn EndedByElapsedTime(&self, elapsed: TimeSpan) -> Result<bool>;
}

/// The `media_player.h` routes with no XNA counterpart.
///
/// XNA's `MediaQueue` is read-only to a game -- it is filled by
/// `MediaPlayer.Play` with a collection. These are how a caller builds one
/// itself.
///
/// A CNA extension: import it to call these.
pub trait MediaQueueExt {
    /// Appends a song to the queue.
    ///
    /// XNA's `MediaQueue` is read-only to a game -- it is filled by
    /// `MediaPlayer.Play` with a collection. This and [`Clear`](Self::Clear)
    /// are how a caller builds one itself.
    fn Add(&self, song: &Song) -> Result<()>;

    /// Empties the queue.
    fn Clear(&self) -> Result<()>;
}
