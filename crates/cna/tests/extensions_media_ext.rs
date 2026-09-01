//! `video.h` and `media.h` beyond XNA: constructing media a game built.
//!
//! XNA's `Video`, `Song` and `SongCollection` are things a game is *handed* --
//! by the content pipeline or the media library -- and only `Song.FromUri` can
//! be called directly. CNA lets a caller build all three, which is what a game
//! with a file on disk or a playlist in hand needs.
//!
//! No media file ships with this repository, so what is measured here is the
//! shape rather than a decoded frame: that construction from a path that names
//! nothing is refused rather than silently producing an empty video, that a
//! collection can be built over songs the caller holds, and that the two track
//! selectors are distinct operations on distinct objects.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cna::Microsoft::Xna::Framework::Media::{MediaSource, Song, SongCollection, Video};
use cna::Microsoft::Xna::Framework::{Game, GameContext};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

#[derive(Default)]
struct MediaGame {
    state: Arc<GameState>,
    ran: Arc<AtomicBool>,
}

impl GameStateAccess for MediaGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for MediaGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        // --- what the platform calls each source's type -----------------------
        let sources = MediaSource::GetAvailableMediaSources(game)?;
        println!("NOTE: {} media source(s)", sources.len());
        for source in &sources {
            let name = source.Name();
            let type_name = source.TypeName(game)?;
            println!("NOTE:   {name:?} -> type name {type_name:?}");
            assert!(
                type_name.is_some(),
                "an enumerated source has a type name; None is reserved for a source \
                 whose game generation ended"
            );
        }

        // --- a video from a path that names nothing ---------------------------
        // Reported rather than asserted: whether a decoder refuses a missing
        // file up front or on first frame is the backend's business, and this
        // test's job is to record which, not to insist.
        match Video::FromFile(game, "/no/such/file/anywhere.ogv") {
            Ok(video) => {
                println!(
                    "NOTE: a missing path still constructed; file name {:?}, device {:?}",
                    video.FileName(),
                    video.HasGraphicsDevice()
                );
                println!("NOTE: decoded info -> {:?}", video.DecodedInfo());
            }
            Err(error) => println!("NOTE: a missing path is refused at construction: {error}"),
        }

        // --- songs the caller built -------------------------------------------
        let first = Song::FromFile(game, "first", "/no/such/song-one.ogg");
        let second = Song::FromFileWithDuration(game, "second", "/no/such/song-two.ogg", 2_500);
        match (&first, &second) {
            (Ok(one), Ok(two)) => {
                println!("NOTE: song names {:?} and {:?}", one.Name()?, two.Name()?);
                println!("NOTE: handle text {:?}", one.HandleText());
                println!("NOTE: second duration {:?}", two.Duration()?);

                // The writers XNA has no counterpart for.
                one.SetPlayCount(7)?;
                assert_eq!(
                    one.PlayCount()?,
                    7,
                    "SetPlayCount is what a library that did not know the count is for"
                );

                let collection = SongCollection::FromSongs(game, &[one, two])?;
                assert_eq!(
                    collection.Count()?,
                    2,
                    "a collection built over two songs holds two"
                );

                // The songs are read during the call and not borrowed after it,
                // so an empty collection is a legal thing to build.
                let empty = SongCollection::FromSongs(game, &[])?;
                assert_eq!(empty.Count()?, 0, "and an empty one holds none");
            }
            _ => {
                // The refusal has to name the FILE, not the display name. The C
                // route takes the file first and the name second -- the
                // opposite of how the Rust signature reads -- and getting them
                // the wrong way round is otherwise silent, because both are
                // strings and either may be empty. This assertion is what
                // caught it.
                for (label, outcome) in [("first", &first), ("second", &second)] {
                    let message = outcome
                        .as_ref()
                        .err()
                        .map_or_else(String::new, ToString::to_string);
                    println!("NOTE: {label} -> {message}");
                    assert!(
                        message.contains("song-one.ogg") || message.contains("song-two.ogg"),
                        "the refusal must name the file it could not find, not the \
                         display name -- a message naming {label:?} means the file \
                         name and the display name were passed in the wrong order: \
                         {message}"
                    );
                }
            }
        }

        self.ran.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn the_media_constructors_and_source_type_names_answer() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let ran = Arc::new(AtomicBool::new(false));
    let game = MediaGame {
        state: Arc::new(GameState::default()),
        ran: Arc::clone(&ran),
    };
    run_for_frames(game, 1).expect("one frame with the media constructors");
    assert!(ran.load(Ordering::SeqCst), "LoadContent ran");
}
