//! `runtime.h`'s per-game routes against a live game.
//!
//! The measurement that made this slice worth doing: CNA keeps its own launch
//! parameters and the Rust `Game::LaunchParameters` dictionary is a different
//! object that CNA never sees. This asserts they are separate, that CNA's is
//! the one an argument list lands in, and that the bridge between them moves
//! what CNA parsed into the XNA-shaped one.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cna::extensions::game_runtime::{
    clear_game_target, FrameBudget, NativeLaunchParameters, RunLoop, TitleContainer,
};
use cna::LaunchParametersExt;
use cna::Microsoft::Xna::Framework::{Color, Game, GameContext};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

#[derive(Default)]
struct RuntimeGame {
    state: Arc<GameState>,
    ran: Arc<AtomicBool>,
}

impl GameStateAccess for RuntimeGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for RuntimeGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        // --- the frame budget, three ways ------------------------------------
        let fps = FrameBudget::target_fps(game)?;
        let milliseconds = FrameBudget::target_frame_milliseconds(game)?;
        println!("NOTE: target {fps} fps = {milliseconds} ms/frame");
        assert!(fps > 0.0, "a game has a positive target frame rate");
        assert!(milliseconds > 0.0, "and a positive frame budget");
        assert!(
            (fps * milliseconds - 1000.0).abs() < 1.0,
            "the two report the same budget: {fps} fps and {milliseconds} ms should \
             multiply to about a thousand"
        );

        // The conversion is CNA's, so it must agree with CNA's own reading.
        let converted = FrameBudget::fps_to_frame_milliseconds(60)?;
        println!("NOTE: 60 fps -> {converted} ms");
        assert!(
            (converted - 1000.0 / 60.0).abs() < 0.001,
            "60 fps is a sixtieth of a second: got {converted}"
        );

        // --- the run loop's own flag -----------------------------------------
        assert!(
            RunLoop::is_running(game)?,
            "the loop is running while a frame is being served"
        );

        // --- the launch parameters are two dictionaries ----------------------
        let count_before = NativeLaunchParameters::count(game)?;
        println!("NOTE: CNA holds {count_before} launch parameter(s) to start");

        // The separator is a COLON, not an equals sign, and leading `/` and `-`
        // are trimmed -- XNA's own rules, which CNA reproduces. Measured, not
        // assumed: an `=` form parses to nothing at all.
        NativeLaunchParameters::parse(
            game,
            &[
                "-windowed:1",     // a dash-flag with a value
                "/fullscreen:no",  // a slash-flag with a value
                "seed:17",         // bare
                "seed:99",         // a duplicate key: the first wins
                "verbose=1",       // an equals sign is not a separator
                "--just-a-flag",   // no value at all
                ":leading",        // a colon at index 0 is not a separator
                "trailing:",       // a colon at the end leaves no value
            ],
        )?;
        let parsed = NativeLaunchParameters::entries(game)?;
        let mut sorted = parsed.clone();
        sorted.sort();
        println!("NOTE: after parse -> {sorted:?}");
        assert_eq!(
            sorted,
            vec![
                ("fullscreen".to_owned(), "no".to_owned()),
                ("seed".to_owned(), "17".to_owned()),
                ("windowed".to_owned(), "1".to_owned()),
            ],
            "the parser takes `key:value` with leading slashes and dashes trimmed, \
             keeps the FIRST value for a repeated key, and ignores an argument with \
             an `=`, with no value, or with the colon at either end"
        );

        // The Rust dictionary knows nothing about any of it: that is the point
        // of publishing CNA's separately rather than pretending one exists.
        let rust_side = self.state.LaunchParameters();
        assert_eq!(
            rust_side.Count(),
            0,
            "CNA's launch parameters and the XNA-shaped Rust dictionary are two \
             different objects, and parsing into one does not populate the other"
        );

        // Add and read back, and check the two spellings agree.
        NativeLaunchParameters::add(game, "difficulty", "hard")?;
        assert!(NativeLaunchParameters::contains_key(game, "difficulty")?);
        assert_eq!(
            NativeLaunchParameters::value(game, "difficulty")?,
            Some("hard".to_owned())
        );
        assert_eq!(
            NativeLaunchParameters::value(game, "no-such-key")?,
            None,
            "an absent key answers None, not an empty string -- a command line can \
             produce a present-and-empty value and the two must stay distinct"
        );

        // RUST-UPSTREAM-026. The header says this route "adds or replaces".
        // It does neither: `emplace` keeps the value already there, the call
        // still answers success, and XNA's own dictionary would have thrown.
        // Asserted as measured, so a fix upstream fails here and says so.
        NativeLaunchParameters::add(game, "difficulty", "easy")?;
        assert_eq!(
            NativeLaunchParameters::value(game, "difficulty")?,
            Some("hard".to_owned()),
            "a second add is silently dropped and still reports success, which is \
             neither what the header promises nor what XNA does"
        );

        // Every key enumerates, and every enumerated key has a value.
        let count = NativeLaunchParameters::count(game)?;
        for index in 0..count {
            let key = NativeLaunchParameters::key_at(game, index)?;
            assert!(!key.is_empty(), "key {index} is not empty");
            assert!(
                NativeLaunchParameters::contains_key(game, &key)?,
                "an enumerated key is a key the dictionary contains: {key:?}"
            );
        }

        // --- the bridge -------------------------------------------------------
        let moved = NativeLaunchParameters::import_into(game, rust_side)?;
        println!("NOTE: imported {moved} parameter(s) into the XNA dictionary");
        assert_eq!(
            moved as u64, count,
            "every parameter CNA holds reaches the XNA dictionary"
        );
        assert_eq!(
            rust_side.Item("difficulty"),
            Some("hard".to_owned()),
            "and arrives with the value CNA actually holds -- \"hard\", because the \
             second add was dropped (RUST-UPSTREAM-026), not \"easy\""
        );
        // Importing twice adds nothing: the keys are already there, and the XNA
        // dictionary refuses a duplicate rather than overwriting.
        assert_eq!(
            NativeLaunchParameters::import_into(game, rust_side)?,
            0,
            "a second import is a no-op rather than a refusal"
        );

        // --- the title container ---------------------------------------------
        let path = TitleContainer::path(game)?;
        println!("NOTE: title path {path:?}");

        let directory = std::env::temp_dir().join(format!("cna-rust-title-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("a scratch title directory");
        std::fs::write(directory.join("greeting.txt"), b"hello from the title container")
            .expect("write a title file");
        TitleContainer::set_path(game, directory.to_str().expect("a UTF-8 path"))?;
        assert_eq!(
            TitleContainer::path(game)?,
            directory.to_str().expect("a UTF-8 path"),
            "the override is what the path now reports"
        );

        let bytes = TitleContainer::read(game, "greeting.txt")?;
        assert_eq!(
            String::from_utf8_lossy(&bytes),
            "hello from the title container",
            "a file read relative to the overridden title path comes back whole -- \
             which is what says the two-call sizing did not truncate it"
        );

        let missing = TitleContainer::read(game, "no-such-file.txt");
        assert!(missing.is_err(), "a file that is not there is a refusal");

        TitleContainer::set_path(game, &path)?;
        let _ = std::fs::remove_file(directory.join("greeting.txt"));
        let _ = std::fs::remove_dir(&directory);

        // --- clearing the game's own target ----------------------------------
        clear_game_target(game, Color::from_r_and_g_and_b_as_int32_and_int32_and_int32(20, 40, 60))?;

        self.ran.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn the_per_game_runtime_routes_answer_and_the_two_dictionaries_stay_separate() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let ran = Arc::new(AtomicBool::new(false));
    let game = RuntimeGame {
        state: Arc::new(GameState::default()),
        ran: Arc::clone(&ran),
    };
    run_for_frames(game, 1).expect("one frame with the per-game runtime routes");
    assert!(ran.load(Ordering::SeqCst), "LoadContent ran");
}
