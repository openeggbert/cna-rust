//! `runtime.h`'s per-game routes: the frame budget, the launch parameters CNA
//! actually holds, and the title container.
//!
//! Separate from [`super::runtime`], which is process-global and takes no game.
//! Everything here is scoped to one.
//!
//! # The launch parameters are CNA's, not a Rust mirror
//!
//! `crates/cna/src/game/services.rs` gives every game a
//! [`LaunchParameters`](crate::Microsoft::Xna::Framework::LaunchParameters)
//! dictionary, and it is a Rust `HashMap` that CNA never sees. CNA keeps its
//! own per-game dictionary, and `cna_game_launch_parameters_parse_ext` is what
//! fills it from a process's arguments -- which is where a real XNA game's
//! command line lands.
//!
//! So the two are different dictionaries, and this module publishes CNA's
//! rather than silently merging them. [`NativeLaunchParameters::import_into`]
//! is the bridge for a caller who wants the XNA-shaped one to carry what CNA
//! parsed.
//!
//! # The frame budget
//!
//! `TargetElapsedTime` is XNA's way of saying how long a frame should take.
//! CNA reports the same budget two other ways -- as frames per second and as
//! milliseconds -- and converts between them. The conversions are CNA's, so
//! they are called rather than restated.

use crate::error::Result;
use crate::game::LaunchParametersExt;
use crate::value::Color;
use crate::Microsoft::Xna::Framework::{GameContext, LaunchParameters};

/// The frame budget, as CNA reports and converts it.
#[derive(Clone, Copy, Debug)]
pub struct FrameBudget;

impl FrameBudget {
    /// The frame rate the current target step implies.
    pub fn target_fps(game: &GameContext<'_>) -> Result<f64> {
        game.native.game_target_fps(game.handle)
    }

    /// The current target step, in milliseconds.
    pub fn target_frame_milliseconds(game: &GameContext<'_>) -> Result<f64> {
        game.native.game_target_frame_milliseconds(game.handle)
    }

    /// Converts a frame rate to a frame time, by CNA's own arithmetic.
    ///
    /// Free of any game: it is a conversion, not a reading.
    pub fn fps_to_frame_milliseconds(frames_per_second: i32) -> Result<f64> {
        crate::native::Native::process()?.fps_to_frame_milliseconds(frames_per_second)
    }
}

/// The canonical run loop's own keep-going flag.
///
/// Distinct from `Game::Exit`, which asks the *game* to stop at the end of the
/// current frame. This is the flag CNA's own loop reads, and setting it false
/// stops the loop wherever it is.
#[derive(Clone, Copy, Debug)]
pub struct RunLoop;

impl RunLoop {
    /// Whether the canonical run loop should keep going.
    pub fn is_running(game: &GameContext<'_>) -> Result<bool> {
        game.native.game_run_application(game.handle)
    }

    /// Sets whether the canonical run loop should keep going.
    pub fn set_running(game: &GameContext<'_>, running: bool) -> Result<()> {
        game.native.set_game_run_application(game.handle, running)
    }
}

/// Clears the game's current graphics target to a colour.
///
/// The one route in `runtime.h` that draws. It is here rather than on
/// `GraphicsDevice` because it takes the *game* handle and clears whatever
/// target that game currently has, which is not the same as clearing a device a
/// caller built.
pub fn clear_game_target(game: &GameContext<'_>, color: Color) -> Result<()> {
    game.native.game_clear_to_color(
        game.handle,
        cna_sys::CNA_Color {
            r: color.R(),
            g: color.G(),
            b: color.B(),
            a: color.A(),
        },
    )
}

/// CNA's own per-game launch parameters.
#[derive(Clone, Copy, Debug)]
pub struct NativeLaunchParameters;

impl NativeLaunchParameters {
    /// How many parameters CNA holds for this game.
    pub fn count(game: &GameContext<'_>) -> Result<u64> {
        game.native.launch_parameter_count(game.handle)
    }

    /// Whether CNA holds a parameter under this key.
    pub fn contains_key(game: &GameContext<'_>, key: &str) -> Result<bool> {
        game.native.launch_parameters_contain(game.handle, key)
    }

    /// The value CNA holds under a key.
    ///
    /// Answers `None` for a key CNA does not hold, rather than an empty string:
    /// "absent" and "present and empty" are different, and a command line can
    /// produce either.
    pub fn value(game: &GameContext<'_>, key: &str) -> Result<Option<String>> {
        if !Self::contains_key(game, key)? {
            return Ok(None);
        }
        game.native.launch_parameter_value(game.handle, key).map(Some)
    }

    /// The key at an index, in whatever order CNA enumerates them.
    pub fn key_at(game: &GameContext<'_>, index: u64) -> Result<String> {
        game.native.launch_parameter_key(game.handle, index)
    }

    /// Every key and value CNA holds.
    pub fn entries(game: &GameContext<'_>) -> Result<Vec<(String, String)>> {
        let count = Self::count(game)?;
        let mut entries = Vec::with_capacity(usize::try_from(count).unwrap_or(0));
        for index in 0..count {
            let key = Self::key_at(game, index)?;
            let value = game.native.launch_parameter_value(game.handle, &key)?;
            entries.push((key, value));
        }
        Ok(entries)
    }

    /// Adds one parameter. A key already present is **kept**, not replaced.
    ///
    /// `RUST-UPSTREAM-026`: the header calls this "adds or replaces" and it
    /// does neither. Upstream's `emplace` keeps the value already there, the
    /// call still answers success, and XNA's own dictionary would have thrown
    /// on the duplicate. So a caller cannot overwrite a parameter through this
    /// route, and gets no signal that the write was dropped -- check
    /// [`Self::contains_key`] first if that matters.
    ///
    /// The XNA-shaped [`LaunchParametersExt::Add`] keeps XNA's behaviour and
    /// refuses a duplicate; it is a different dictionary, as the module
    /// documentation explains.
    ///
    /// [`LaunchParametersExt::Add`]: crate::LaunchParametersExt::Add
    pub fn add(game: &GameContext<'_>, key: &str, value: &str) -> Result<()> {
        game.native.add_launch_parameter(game.handle, key, value)
    }

    /// Replaces every parameter by parsing an argument list.
    ///
    /// This is what a platform does with `argv` at startup.
    pub fn parse(game: &GameContext<'_>, arguments: &[&str]) -> Result<()> {
        game.native.parse_launch_parameters(game.handle, arguments)
    }

    /// Copies everything CNA holds into an XNA-shaped dictionary.
    ///
    /// The bridge between the two: CNA parses the command line, and a game that
    /// reads `Game::LaunchParameters` gets what was parsed instead of an empty
    /// map. Keys already present are left alone, because the XNA dictionary
    /// refuses a duplicate and overwriting silently would lose whatever the
    /// game had put there itself.
    ///
    /// Answers how many were added.
    pub fn import_into(
        game: &GameContext<'_>,
        parameters: &LaunchParameters,
    ) -> Result<usize> {
        let mut added = 0;
        for (key, value) in Self::entries(game)? {
            if !parameters.ContainsKey(&key) {
                parameters.Add(&key, &value)?;
                added += 1;
            }
        }
        Ok(added)
    }
}

/// The title's base path, and reading files relative to it.
#[derive(Clone, Copy, Debug)]
pub struct TitleContainer;

impl TitleContainer {
    /// The title's base path.
    pub fn path(game: &GameContext<'_>) -> Result<String> {
        game.native.title_path(game.handle)
    }

    /// Overrides the title's base path.
    pub fn set_path(game: &GameContext<'_>, path: &str) -> Result<()> {
        game.native.set_title_path(game.handle, path)
    }

    /// Reads a file relative to the title's base path.
    ///
    /// Sized then read, in two calls, because the route reports the file's byte
    /// count whether or not the buffer could hold it and performs no partial
    /// write.
    pub fn read(game: &GameContext<'_>, path: &str) -> Result<Vec<u8>> {
        game.native.read_title_file(game.handle, path)
    }
}
