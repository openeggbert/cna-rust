//! Crash-isolated Media/Video ownership, generation, callback, and backend qualification.

#![allow(non_snake_case, clippy::float_cmp, clippy::too_many_lines)]

use std::any::Any;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use cna::extensions::events::EventArgs;
use cna::extensions::media::{
    PlayFromEvent, RaiseActiveSongChanged, RaiseMediaStateChanged, StopFromEvent,
    VideoFrameGeneration, VideoFramePresentationTime,
};
use cna::Microsoft::Xna::Framework::Media::{
    MediaLibrary, MediaPlayer, MediaQueue, MediaSource, MediaState, Song, Video, VideoPlayer,
    VideoSoundtrackType, VisualizationData,
};
use cna::Microsoft::Xna::Framework::{FrameworkDispatcher, Game, GameContext, GameTime};
use cna::{run_for_frames, CnaError, GameState, GameStateAccess, Result};

const CHILD: &str = "CNA_RUST_MEDIA_STRESS_CHILD";

fn write_7bit(bytes: &mut Vec<u8>, mut value: usize) {
    loop {
        let mut next = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 { next |= 0x80; }
        bytes.push(next);
        if value == 0 { break; }
    }
}

fn write_string(bytes: &mut Vec<u8>, value: &str) {
    write_7bit(bytes, value.len());
    bytes.extend_from_slice(value.as_bytes());
}

fn video_xnb() -> Vec<u8> {
    let readers = [
        "Microsoft.Xna.Framework.Content.VideoReader, Microsoft.Xna.Framework.Video",
        "Microsoft.Xna.Framework.Content.StringReader",
        "Microsoft.Xna.Framework.Content.Int32Reader",
        "Microsoft.Xna.Framework.Content.SingleReader",
    ];
    let mut payload = Vec::new();
    write_7bit(&mut payload, readers.len());
    for reader in readers { write_string(&mut payload, reader); payload.extend_from_slice(&0_i32.to_le_bytes()); }
    write_7bit(&mut payload, 0);
    write_7bit(&mut payload, 1); // root VideoReader
    write_7bit(&mut payload, 2); write_string(&mut payload, "missing-project-authored-video.ogv");
    for value in [1_250_i32, 320, 180] { write_7bit(&mut payload, 3); payload.extend_from_slice(&value.to_le_bytes()); }
    write_7bit(&mut payload, 4); payload.extend_from_slice(&24.0_f32.to_le_bytes());
    write_7bit(&mut payload, 3); payload.extend_from_slice(&2_i32.to_le_bytes());
    let mut result = b"XNBw\x05\x00".to_vec();
    result.extend_from_slice(&(10_u32 + payload.len() as u32).to_le_bytes());
    result.extend_from_slice(&payload);
    result
}

fn wav() -> Vec<u8> {
    let samples = [0_i16; 160];
    let data_len = (samples.len() * 2) as u32;
    let mut value = Vec::new();
    value.extend_from_slice(b"RIFF"); value.extend_from_slice(&(36 + data_len).to_le_bytes());
    value.extend_from_slice(b"WAVEfmt "); value.extend_from_slice(&16_u32.to_le_bytes());
    value.extend_from_slice(&1_u16.to_le_bytes()); value.extend_from_slice(&1_u16.to_le_bytes());
    value.extend_from_slice(&8_000_u32.to_le_bytes()); value.extend_from_slice(&16_000_u32.to_le_bytes());
    value.extend_from_slice(&2_u16.to_le_bytes()); value.extend_from_slice(&16_u16.to_le_bytes());
    value.extend_from_slice(b"data"); value.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples { value.extend_from_slice(&sample.to_le_bytes()); }
    value
}

struct Fixtures { root: PathBuf, song: PathBuf }
impl Fixtures {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("cna-rust-media-{}", std::process::id()));
        fs::create_dir_all(&root).expect("create Media fixture directory");
        fs::write(root.join("video.xnb"), video_xnb()).expect("write Video XNB fixture");
        fs::write(root.join("video2.xnb"), video_xnb()).expect("write second Video XNB fixture");
        fs::write(root.join("fault-video.xnb"), video_xnb()).expect("write failed Video fixture");
        let song = root.join("project-authored.wav");
        fs::write(&song, wav()).expect("write Song fixture");
        Self { root, song }
    }
}
impl Drop for Fixtures { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); } }

struct MediaStressGame {
    state: Arc<GameState>,
    song_path: PathBuf,
    old_queue: Arc<Mutex<Option<Arc<MediaQueue>>>>,
    old_song: Arc<Mutex<Option<Arc<Song>>>>,
    old_video: Arc<Mutex<Option<Arc<Video>>>>,
    old_player: Arc<Mutex<Option<Arc<VideoPlayer>>>>,
    callbacks: Arc<AtomicUsize>,
    later_callbacks: Arc<AtomicUsize>,
    self_registration: Arc<AtomicU64>,
    remove_tokens: Vec<(bool, u64)>,
    dispatched: bool,
}

impl GameStateAccess for MediaStressGame { fn game_state(&self) -> &Arc<GameState> { &self.state } }

impl Game for MediaStressGame {
    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let sources = MediaSource::GetAvailableMediaSources(game)?;
        for source in &sources { let _ = (source.Name()?, source.MediaSourceType()?); }

        for cycle in 0..20 {
            let library = MediaLibrary::new(game)?;
            assert!(!library.IsDisposed()?);
            let songs = library.Songs()?;
            assert!(Arc::ptr_eq(&songs, &library.Songs()?));
            let song_count = songs.Count()?;
            assert!(song_count >= 0);
            assert_eq!(songs.GetEnumerator()?.count(), song_count as usize);
            assert!(songs.Item(song_count).is_err());
            // Exercise CNA's exact boundary outcome without imposing a Rust-only range rule:
            // canonical XNA stores the starting index unchecked, and ABI 0.7 preserves that.
            let invalid_start_outcome = MediaPlayer::PlayWithSongsAndIndex(game, &songs, song_count);
            MediaPlayer::Stop(game)?;
            if invalid_start_outcome.is_ok() {
                let qualification_queue = MediaPlayer::Queue(game)?;
                assert_eq!(qualification_queue.ActiveSongIndex()?, song_count);
                qualification_queue.SetActiveSongIndex(-1)?;
            }
            if song_count > 0 {
                assert!(Arc::ptr_eq(&songs.Item(0)?, &songs.Item(0)?));
            }
            assert!(library.Albums()?.Count()? >= 0);
            assert!(library.Artists()?.Count()? >= 0);
            assert!(library.Genres()?.Count()? >= 0);
            assert!(library.Playlists()?.Count()? >= 0);
            let pictures = library.Pictures()?;
            assert!(Arc::ptr_eq(&pictures, &library.Pictures()?));
            let picture_count = pictures.Count()?;
            assert!(picture_count >= 0);
            assert_eq!(pictures.GetEnumerator()?.count(), picture_count as usize);
            assert!(pictures.Item(picture_count).is_err());
            if picture_count > 0 {
                let picture = pictures.Item(0)?;
                assert!(Arc::ptr_eq(&picture, &pictures.Item(0)?));
                let _ = (picture.Name()?, picture.Width()?, picture.Height()?);
            }
            assert!(library.SavedPictures()?.Count()? >= 0);
            let source = library.MediaSource()?;
            assert!(Arc::ptr_eq(&source, &library.MediaSource()?));
            let _ = (source.Name()?, source.MediaSourceType()?);
            if cycle % 2 == 0 { library.Dispose()?; library.Dispose()?; } else { drop(library); }
            assert!(songs.IsDisposed()?);
            assert!(songs.Count().is_err());
        }

        let wrong_thread_library = MediaLibrary::new(game)?;
        let wrong_thread_library = std::thread::spawn(move || {
            assert!(wrong_thread_library.Dispose().is_err());
            wrong_thread_library
        }).join().expect("wrong-thread MediaLibrary remains recoverable");
        wrong_thread_library.Dispose()?;

        for cycle in 0..20 {
            let song = Song::FromUri(game, &format!("song-{cycle}"), path(&self.song_path))?;
            assert_eq!(song.Name()?, format!("song-{cycle}"));
            assert!(!song.IsProtected()?);
            assert_eq!(song.Duration()?.Ticks(), 0);
            assert!(!song.IsRated()?);
            assert_eq!(song.PlayCount()?, 0);
            assert_eq!(song.Rating()?, 0);
            assert_eq!(song.TrackNumber()?, 0);
            assert!(song.Artist()?.is_none() && song.Album()?.is_none() && song.Genre()?.is_none());
            song.Dispose()?; song.Dispose()?;
        }
        let missing_song = self.song_path.with_file_name("missing-project-authored.wav");
        assert!(Song::FromUri(game, "missing", path(&missing_song)).is_err());
        assert!(Song::FromUri(game, "remote", "https://example.invalid/song.wav").is_err());

        MediaPlayer::SetVolume(game, f32::NEG_INFINITY)?; assert_eq!(MediaPlayer::Volume(game)?, 0.0);
        MediaPlayer::SetVolume(game, f32::INFINITY)?; assert_eq!(MediaPlayer::Volume(game)?, 1.0);
        MediaPlayer::SetVolume(game, -0.5)?; assert_eq!(MediaPlayer::Volume(game)?, 0.0);
        MediaPlayer::SetVolume(game, 1.5)?; assert_eq!(MediaPlayer::Volume(game)?, 1.0);
        MediaPlayer::SetVolume(game, -0.0)?; assert_eq!(MediaPlayer::Volume(game)?.to_bits(), (-0.0_f32).to_bits());
        MediaPlayer::SetVolume(game, f32::NAN)?; assert!(MediaPlayer::Volume(game)?.is_nan());
        MediaPlayer::SetVolume(game, 0.5)?;
        MediaPlayer::SetIsMuted(game, true)?; assert!(MediaPlayer::IsMuted(game)?);
        MediaPlayer::SetIsMuted(game, false)?;
        MediaPlayer::SetIsRepeating(game, true)?; assert!(MediaPlayer::IsRepeating(game)?);
        MediaPlayer::SetIsShuffled(game, true)?; assert!(MediaPlayer::IsShuffled(game)?);
        MediaPlayer::SetIsRepeating(game, false)?; MediaPlayer::SetIsShuffled(game, false)?;
        let queue = MediaPlayer::Queue(game)?;
        assert!(Arc::ptr_eq(&queue, &MediaPlayer::Queue(game)?));
        assert_eq!(queue.Count()?, 0); assert!(queue.ActiveSong()?.is_none());
        assert_eq!(queue.ActiveSongIndex()?, -1);
        MediaPlayer::Pause(game)?; MediaPlayer::Resume(game)?; MediaPlayer::Stop(game)?;
        MediaPlayer::MoveNext(game)?; MediaPlayer::MovePrevious(game)?;
        let disposed_song = Song::FromUri(game, "disposed", path(&self.song_path))?;
        disposed_song.Dispose()?;
        assert!(MediaPlayer::Play(game, &disposed_song).is_err());
        let song = Arc::new(Song::FromUri(game, "playback", path(&self.song_path))?);
        MediaPlayer::Play(game, &song)?;
        assert_eq!(queue.Count()?, 1);
        let active = queue.ActiveSong()?.expect("active playback Song");
        assert!(Arc::ptr_eq(&active, &queue.Item(0)?));
        MediaPlayer::Pause(game)?; MediaPlayer::Resume(game)?;
        MediaPlayer::MoveNext(game)?; MediaPlayer::MovePrevious(game)?; MediaPlayer::Stop(game)?;
        let mut visualization = VisualizationData::new();
        MediaPlayer::GetVisualizationData(game, &mut visualization)?;
        MediaPlayer::SetIsVisualizationEnabled(game, true)?;
        MediaPlayer::GetVisualizationData(game, &mut visualization)?;
        assert_eq!(visualization.Frequencies().len(), 256);
        assert_eq!(visualization.Samples().len(), 256);
        assert_eq!(MediaPlayer::Volume(game)?, 0.5);
        assert!(!MediaPlayer::IsMuted(game)? && !MediaPlayer::IsRepeating(game)? && !MediaPlayer::IsShuffled(game)?);
        assert!(MediaPlayer::IsVisualizationEnabled(game)?);
        *self.old_queue.lock().unwrap() = Some(Arc::clone(&queue));
        *self.old_song.lock().unwrap() = Some(Arc::clone(&song));

        #[cfg(feature = "native-fault-injection")]
        {
            std::env::set_var("CNA_RUST_TEST_FAULT", "video-create-after-native");
            let failed_video = self.state.Content().Load::<Video>("fault-video");
            std::env::remove_var("CNA_RUST_TEST_FAULT");
            match failed_video {
                Err(CnaError::Content(error)) => {
                    assert!(error.to_string().contains("injected Rust bridge fault"));
                }
                Err(error) => panic!("unexpected failed Video error: {error}"),
                Ok(_) => panic!("injected post-create Video failure was ignored"),
            }
            let recovered_video = self.state.Content().Load::<Video>("fault-video")?;
            assert_eq!(recovered_video.Width()?, 320);
            self.state.Content().Unload()?;
            assert!(recovered_video.Width().is_err());
        }
        for _ in 0..20 {
            let video_cycle = self.state.Content().Load::<Video>("video")?;
            assert_eq!((video_cycle.Width()?, video_cycle.Height()?), (320, 180));
            self.state.Content().Unload()?;
            assert!(video_cycle.Duration().is_err());
        }
        let video = self.state.Content().Load::<Video>("video")?;
        let second_video = self.state.Content().Load::<Video>("video2")?;
        assert!(!Arc::ptr_eq(&video, &second_video));
        assert_eq!(video.Duration()?.TotalMilliseconds(), 1_250.0);
        assert_eq!((video.Width()?, video.Height()?), (320, 180));
        assert_eq!(video.FramesPerSecond()?, 24.0);
        assert_eq!(video.VideoSoundtrackType()?, VideoSoundtrackType::MusicAndDialog);
        *self.old_video.lock().unwrap() = Some(Arc::clone(&video));
        for _ in 0..20 {
            let player = VideoPlayer::new(game)?;
            assert_eq!(player.State()?, MediaState::Stopped);
            assert!(player.GetTexture().is_err());
            assert!(player.SetVolume(-0.25).is_err());
            assert!(player.SetVolume(2.0).is_err());
            player.SetVolume(f32::NAN)?; assert!(player.Volume()?.is_nan());
            player.SetIsLooped(true)?; player.SetIsMuted(true)?;
            player.Pause()?; player.Resume()?; player.Stop()?;
            player.Play(Arc::clone(&video))?;
            assert!(Arc::ptr_eq(&player.Video()?.expect("current Video"), &video));
            // CNA accepts the metadata/control object even when this legal fixture
            // has no decodable asset. HEADLESS decodes nothing, so the frame route
            // answers "no frame" rather than failing, and the generation counter
            // stays at zero. A backend that does decode hands back a borrowed
            // Texture2D whose validity ends at the next player call.
            assert!(matches!(player.GetTexture(), Ok(None)));
            assert_eq!(VideoFrameGeneration(&player)?, 0);
            assert!(VideoFramePresentationTime(&player)?.is_none());
            assert!(matches!(player.GetTexture(), Ok(None)));
            player.Pause()?;
            assert!(matches!(player.GetTexture(), Ok(None)));
            player.Resume()?;
            assert!(matches!(player.GetTexture(), Ok(None)));
            player.Play(Arc::clone(&second_video))?;
            assert!(Arc::ptr_eq(
                &player.Video()?.expect("replacement Video"),
                &second_video,
            ));
            assert!(matches!(player.GetTexture(), Ok(None)));
            player.Stop()?;
            // The counter is monotonic: neither Stop nor a different Video restarts it.
            assert_eq!(VideoFrameGeneration(&player)?, 0);
            assert!(matches!(player.GetTexture(), Ok(None)));
            player.Dispose()?; player.Dispose()?;
            assert!(player.IsDisposed()?);
            assert!(player.IsLooped()? && player.IsMuted()? && player.Volume()?.is_nan());
            assert!(player.State().is_err());
            assert!(player.GetTexture().is_err());
            assert!(VideoFrameGeneration(&player).is_err());
        }

        let wrong_thread_player = VideoPlayer::new(game)?;
        let wrong_thread_player = std::thread::spawn(move || {
            assert!(wrong_thread_player.Dispose().is_err());
            wrong_thread_player
        }).join().expect("wrong-thread VideoPlayer remains recoverable");
        wrong_thread_player.Dispose()?;
        let retained_player = Arc::new(VideoPlayer::new(game)?);
        retained_player.Play(Arc::clone(&video))?;
        assert!(matches!(retained_player.GetTexture(), Ok(None)));
        *self.old_player.lock().unwrap() = Some(retained_player);

        let self_registration = Arc::clone(&self.self_registration);
        let self_count = Arc::clone(&self.callbacks);
        let token = MediaPlayer::AddActiveSongChangedHandler(Box::new(move |_:&dyn Any,_:EventArgs| {
            self_count.fetch_add(1, Ordering::SeqCst);
            MediaPlayer::RemoveActiveSongChangedHandler(self_registration.load(Ordering::Acquire));
        }));
        self.self_registration.store(token, Ordering::Release);
        let later = Arc::clone(&self.later_callbacks);
        let later_token = MediaPlayer::AddActiveSongChangedHandler(Box::new(move |_:&dyn Any,_:EventArgs| { later.fetch_add(1, Ordering::SeqCst); }));
        let state_count = Arc::clone(&self.callbacks);
        let state_token = MediaPlayer::AddMediaStateChangedHandler(Box::new(move |_:&dyn Any,_:EventArgs| { state_count.fetch_add(1, Ordering::SeqCst); }));
        self.remove_tokens.extend([(true, later_token), (false, state_token)]);
        for _ in 0..25 { RaiseActiveSongChanged(game)?; RaiseMediaStateChanged(game)?; }
        assert_eq!(self.callbacks.load(Ordering::SeqCst), 0);
        Ok(())
    }

    fn Update(&mut self, game:&mut GameContext<'_>,_:&GameTime)->Result<()> {
        FrameworkDispatcher::Update(game)?;
        if !self.dispatched {
            self.dispatched = true;
            assert!(self.callbacks.load(Ordering::SeqCst) >= 26);
            // The 25 explicit raises are deterministic. CNA may additionally publish one
            // playback transition when its next native MediaPlayer update is pumped.
            assert!(self.later_callbacks.load(Ordering::SeqCst) >= 25);
            for (active, token) in self.remove_tokens.drain(..) {
                if active { MediaPlayer::RemoveActiveSongChangedHandler(token); }
                else { MediaPlayer::RemoveMediaStateChangedHandler(token); }
            }
        }
        Ok(())
    }
}

fn path(value:&Path)->&str { value.to_str().expect("UTF-8 fixture path") }

fn make_game(fixtures:&Fixtures, old_queue:Arc<Mutex<Option<Arc<MediaQueue>>>>, old_song:Arc<Mutex<Option<Arc<Song>>>>, old_video:Arc<Mutex<Option<Arc<Video>>>>, old_player:Arc<Mutex<Option<Arc<VideoPlayer>>>>, callbacks:Arc<AtomicUsize>, later:Arc<AtomicUsize>)->MediaStressGame {
    let state=Arc::new(GameState::new());
    state.Content().SetRootDirectory(path(&fixtures.root)).expect("set Media content root");
    MediaStressGame{state,song_path:fixtures.song.clone(),old_queue,old_song,old_video,old_player,callbacks,later_callbacks:later,self_registration:Arc::new(AtomicU64::new(0)),remove_tokens:Vec::new(),dispatched:false}
}

#[derive(Default)] struct QueueGenerationGame { state:Arc<GameState>, prior:Option<Arc<MediaQueue>>, prior_song:Option<Arc<Song>>, prior_video:Option<Arc<Video>> }
impl GameStateAccess for QueueGenerationGame { fn game_state(&self)->&Arc<GameState>{&self.state} }
impl Game for QueueGenerationGame { fn Initialize(&mut self,game:&mut GameContext<'_>)->Result<()>{let queue=MediaPlayer::Queue(game)?;if let Some(prior)=&self.prior{assert!(!Arc::ptr_eq(prior,&queue));assert!(prior.Count().is_err());}if let Some(song)=&self.prior_song{assert!(song.Name().is_err());assert!(MediaPlayer::Play(game,song).is_err());}if let Some(video)=&self.prior_video{assert!(video.Duration().is_err());let player=VideoPlayer::new(game)?;assert!(player.Play(Arc::clone(video)).is_err());}assert_eq!(MediaPlayer::Volume(game)?,0.5);assert!(!MediaPlayer::IsMuted(game)?&&!MediaPlayer::IsRepeating(game)?&&!MediaPlayer::IsShuffled(game)?);assert!(MediaPlayer::IsVisualizationEnabled(game)?);Ok(())} }

struct PanicEventGame {
    state: Arc<GameState>,
    later: Arc<AtomicUsize>,
    tokens: Arc<Mutex<Vec<u64>>>,
}
impl GameStateAccess for PanicEventGame { fn game_state(&self)->&Arc<GameState>{&self.state} }
impl Game for PanicEventGame {
    fn Initialize(&mut self,game:&mut GameContext<'_>)->Result<()>{
        let panic_token=MediaPlayer::AddActiveSongChangedHandler(Box::new(|_:&dyn Any,_:EventArgs|panic!("contained Media handler panic")));
        let later=Arc::clone(&self.later);
        let later_token=MediaPlayer::AddActiveSongChangedHandler(Box::new(move |_:&dyn Any,_:EventArgs|{later.fetch_add(1,Ordering::SeqCst);}));
        self.tokens.lock().unwrap().extend([panic_token,later_token]);
        RaiseActiveSongChanged(game)
    }
    fn Update(&mut self,game:&mut GameContext<'_>,_:&GameTime)->Result<()>{FrameworkDispatcher::Update(game)}
}

struct SkipDispatchGame {
    state: Arc<GameState>,
    calls: Arc<AtomicUsize>,
    token: Arc<AtomicU64>,
}

struct CutoffOrderGame {
    state: Arc<GameState>,
    order: Arc<Mutex<Vec<u8>>>,
    tokens: Vec<u64>,
}

impl GameStateAccess for CutoffOrderGame {
    fn game_state(&self) -> &Arc<GameState> { &self.state }
}

impl Game for CutoffOrderGame {
    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        for marker in [1_u8, 2] {
            let order = Arc::clone(&self.order);
            self.tokens.push(MediaPlayer::AddActiveSongChangedHandler(Box::new(
                move |_: &dyn Any, _: EventArgs| {
                    order.lock().expect("Media event order").push(marker);
                },
            )));
        }
        RaiseActiveSongChanged(game)?;
        let order = Arc::clone(&self.order);
        self.tokens.push(MediaPlayer::AddActiveSongChangedHandler(Box::new(
            move |_: &dyn Any, _: EventArgs| {
                order.lock().expect("late Media event handler").push(3);
            },
        )));
        Ok(())
    }

    fn Update(&mut self, game: &mut GameContext<'_>, _: &GameTime) -> Result<()> {
        FrameworkDispatcher::Update(game)?;
        assert_eq!(*self.order.lock().expect("Media event order"), [1, 2]);
        for token in self.tokens.drain(..) {
            MediaPlayer::RemoveActiveSongChangedHandler(token);
        }
        Ok(())
    }
}

struct ReentrantTransportGame {
    state: Arc<GameState>,
    song_path: PathBuf,
    calls: Arc<AtomicUsize>,
    token: Arc<AtomicU64>,
}

impl GameStateAccess for ReentrantTransportGame {
    fn game_state(&self) -> &Arc<GameState> { &self.state }
}

impl Game for ReentrantTransportGame {
    fn Initialize(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let song = Arc::new(Song::FromUri(game, "event-reentrant", path(&self.song_path))?);
        let calls = Arc::clone(&self.calls);
        let token = Arc::clone(&self.token);
        let registration = MediaPlayer::AddActiveSongChangedHandler(Box::new(
            move |_: &dyn Any, _: EventArgs| {
                MediaPlayer::RemoveActiveSongChangedHandler(token.load(Ordering::Acquire));
                StopFromEvent().expect("Stop is reentrant inside Media event dispatch");
                PlayFromEvent(&song).expect("Play is reentrant inside Media event dispatch");
                calls.fetch_add(1, Ordering::SeqCst);
            },
        ));
        self.token.store(registration, Ordering::Release);
        RaiseActiveSongChanged(game)
    }

    fn Update(&mut self, game: &mut GameContext<'_>, _: &GameTime) -> Result<()> {
        FrameworkDispatcher::Update(game)
    }
}
impl GameStateAccess for SkipDispatchGame { fn game_state(&self)->&Arc<GameState>{&self.state} }
impl Game for SkipDispatchGame {
    fn Initialize(&mut self,game:&mut GameContext<'_>)->Result<()>{
        let calls=Arc::clone(&self.calls);
        let token=MediaPlayer::AddActiveSongChangedHandler(Box::new(move |_:&dyn Any,_:EventArgs|{calls.fetch_add(1,Ordering::SeqCst);}));
        self.token.store(token,Ordering::Release);
        RaiseActiveSongChanged(game)
    }
    fn Update(&mut self,_:&mut GameContext<'_>,_:&GameTime)->Result<()>{
        Err(CnaError::InvalidInput("intentional Update failure before dispatcher"))
    }
}

#[test]
fn media_native_stress_isolated() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() { return; }
    if std::env::var_os(CHILD).is_none() {
        let data=std::env::temp_dir().join(format!("cna-rust-media-data-{}",std::process::id()));
        let status=Command::new(std::env::current_exe().unwrap()).args(["--exact","media_native_stress_isolated"]).env(CHILD,"1").env("XDG_DATA_HOME",data).env("SDL_AUDIODRIVER","dummy").status().expect("start Media stress child");
        assert!(status.success(),"Media stress child failed: {status}");
        return;
    }
    let fixtures=Fixtures::new();let old_queue=Arc::new(Mutex::new(None));let old_song=Arc::new(Mutex::new(None));let old_video=Arc::new(Mutex::new(None));let old_player=Arc::new(Mutex::new(None));let callbacks=Arc::new(AtomicUsize::new(0));let later=Arc::new(AtomicUsize::new(0));
    run_for_frames(make_game(&fixtures,Arc::clone(&old_queue),Arc::clone(&old_song),Arc::clone(&old_video),Arc::clone(&old_player),Arc::clone(&callbacks),Arc::clone(&later)),2).expect("Media graph/player/video stress");
    let prior=old_queue.lock().unwrap().clone().expect("retained Game #1 queue");
    let prior_song=old_song.lock().unwrap().clone().expect("retained Game #1 Song");
    let prior_video=old_video.lock().unwrap().clone().expect("retained Game #1 Video");
    let prior_player=old_player.lock().unwrap().clone().expect("retained Game #1 VideoPlayer");
    assert!(prior.Count().is_err());
    assert!(prior_song.Name().is_err());
    assert!(prior_video.Duration().is_err());
    assert!(prior_player.State().is_err());
    assert!(prior_player.GetTexture().is_err());
    for _ in 0..20 { run_for_frames(QueueGenerationGame{state:Arc::new(GameState::new()),prior:Some(Arc::clone(&prior)),prior_song:Some(Arc::clone(&prior_song)),prior_video:Some(Arc::clone(&prior_video))},1).expect("Media queue generation cycle"); }
    assert!(callbacks.load(Ordering::SeqCst)>=26);assert!(later.load(Ordering::SeqCst)>=25);
    let skipped_calls=Arc::new(AtomicUsize::new(0));let skipped_token=Arc::new(AtomicU64::new(0));
    let skipped=run_for_frames(SkipDispatchGame{state:Arc::new(GameState::new()),calls:Arc::clone(&skipped_calls),token:Arc::clone(&skipped_token)},1);
    assert!(matches!(skipped,Err(CnaError::InvalidInput(_))));
    assert_eq!(skipped_calls.load(Ordering::SeqCst),0);
    run_for_frames(QueueGenerationGame{state:Arc::new(GameState::new()),prior:Some(Arc::clone(&prior)),prior_song:Some(Arc::clone(&prior_song)),prior_video:Some(Arc::clone(&prior_video))},1).expect("fresh Game discards skipped stale Media event");
    assert_eq!(skipped_calls.load(Ordering::SeqCst),0);
    MediaPlayer::RemoveActiveSongChangedHandler(skipped_token.load(Ordering::Acquire));
    let panic_later=Arc::new(AtomicUsize::new(0));let panic_tokens=Arc::new(Mutex::new(Vec::new()));
    let panic_result=run_for_frames(PanicEventGame{state:Arc::new(GameState::new()),later:Arc::clone(&panic_later),tokens:Arc::clone(&panic_tokens)},1);
    assert!(matches!(panic_result,Err(CnaError::Callback(_))));
    assert_eq!(panic_later.load(Ordering::SeqCst),1);
    for token in panic_tokens.lock().unwrap().drain(..){MediaPlayer::RemoveActiveSongChangedHandler(token);}
    let reentrant_calls=Arc::new(AtomicUsize::new(0));
    let reentrant_token=Arc::new(AtomicU64::new(0));
    run_for_frames(ReentrantTransportGame{
        state:Arc::new(GameState::new()),
        song_path:fixtures.song.clone(),
        calls:Arc::clone(&reentrant_calls),
        token:Arc::clone(&reentrant_token),
    },1).expect("Media Stop/Play reentrancy inside handler");
    assert_eq!(reentrant_calls.load(Ordering::SeqCst),1);
    assert!(StopFromEvent().is_err());
    let order=Arc::new(Mutex::new(Vec::new()));
    run_for_frames(CutoffOrderGame{
        state:Arc::new(GameState::new()),
        order:Arc::clone(&order),
        tokens:Vec::new(),
    },1).expect("Media event order and registration cutoff");
    assert_eq!(*order.lock().unwrap(),[1,2]);
}
