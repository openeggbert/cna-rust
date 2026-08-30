//! CNA-only deterministic Media callback and video-frame hooks.

#![allow(non_snake_case)]

use crate::game::GameContext;
use crate::media::{MediaPlayer, Song, VideoPlayer};
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
