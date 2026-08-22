#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use core::ffi::c_void;
use core::mem::size_of;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use cna_sys as sys;

use crate::error::{CnaError, Result};
use crate::graphics::GraphicsDevice;
use crate::native::Native;

/// A signed .NET-compatible interval measured in 100-nanosecond ticks.
#[allow(non_snake_case, non_upper_case_globals)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TimeSpan {
    ticks: i64,
}

#[allow(non_snake_case, non_upper_case_globals)]
impl TimeSpan {
    pub const TicksPerMillisecond: i64 = 10_000;
    pub const TicksPerSecond: i64 = 10_000_000;
    pub const TicksPerMinute: i64 = 600_000_000;
    pub const TicksPerHour: i64 = 36_000_000_000;
    pub const TicksPerDay: i64 = 864_000_000_000;
    pub const Zero: Self = Self { ticks: 0 };
    pub const MaxValue: Self = Self { ticks: i64::MAX };
    pub const MinValue: Self = Self { ticks: i64::MIN };

    #[must_use]
    pub const fn from_ticks(ticks: i64) -> Self {
        Self { ticks }
    }

    #[must_use]
    pub fn FromMilliseconds(milliseconds: f64) -> Self {
        Self::from_interval(milliseconds, 1.0)
    }

    #[must_use]
    pub fn FromSeconds(seconds: f64) -> Self {
        Self::from_interval(seconds, 1_000.0)
    }

    #[must_use]
    pub const fn Ticks(&self) -> i64 {
        self.ticks
    }

    #[must_use]
    pub fn TotalMilliseconds(&self) -> f64 {
        self.ticks as f64 / Self::TicksPerMillisecond as f64
    }

    #[must_use]
    pub fn TotalSeconds(&self) -> f64 {
        self.ticks as f64 / Self::TicksPerSecond as f64
    }

    fn from_interval(value: f64, milliseconds_per_unit: f64) -> Self {
        assert!(value.is_finite(), "TimeSpan value must be finite");
        // .NET Framework 4 interval factories round to the nearest whole
        // millisecond before converting to ticks (halves away from zero).
        let milliseconds = (value * milliseconds_per_unit).round();
        let maximum_milliseconds = i64::MAX / Self::TicksPerMillisecond;
        assert!(
            milliseconds >= -maximum_milliseconds as f64
                && milliseconds <= maximum_milliseconds as f64,
            "TimeSpan overflow"
        );
        Self::from_ticks((milliseconds as i64) * Self::TicksPerMillisecond)
    }
}

impl core::ops::Add for TimeSpan {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_ticks(
            self.ticks
                .checked_add(rhs.ticks)
                .expect("TimeSpan overflow"),
        )
    }
}

impl core::ops::Sub for TimeSpan {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_ticks(
            self.ticks
                .checked_sub(rhs.ticks)
                .expect("TimeSpan overflow"),
        )
    }
}

impl core::ops::Neg for TimeSpan {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from_ticks(self.ticks.checked_neg().expect("TimeSpan overflow"))
    }
}

/// Timing information supplied to one update or draw callback.
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameTime {
    total_game_time: TimeSpan,
    elapsed_game_time: TimeSpan,
    is_running_slowly: bool,
}

#[allow(non_snake_case)]
impl GameTime {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_game_time: TimeSpan::Zero,
            elapsed_game_time: TimeSpan::Zero,
            is_running_slowly: false,
        }
    }

    #[must_use]
    pub const fn from_total_game_time_and_elapsed_game_time(
        total: TimeSpan,
        elapsed: TimeSpan,
    ) -> Self {
        Self::from_total_game_time_and_elapsed_game_time_and_is_running_slowly(
            total, elapsed, false,
        )
    }

    #[must_use]
    pub const fn from_total_game_time_and_elapsed_game_time_and_is_running_slowly(
        total: TimeSpan,
        elapsed: TimeSpan,
        running_slowly: bool,
    ) -> Self {
        Self {
            total_game_time: total,
            elapsed_game_time: elapsed,
            is_running_slowly: running_slowly,
        }
    }

    #[must_use]
    pub const fn TotalGameTime(&self) -> TimeSpan {
        self.total_game_time
    }

    #[must_use]
    pub const fn ElapsedGameTime(&self) -> TimeSpan {
        self.elapsed_game_time
    }

    #[must_use]
    pub const fn IsRunningSlowly(&self) -> bool {
        self.is_running_slowly
    }
}

impl From<&sys::CNA_GameTime> for GameTime {
    fn from(value: &sys::CNA_GameTime) -> Self {
        Self::from_total_game_time_and_elapsed_game_time_and_is_running_slowly(
            TimeSpan::from_ticks(value.total_game_time_ticks),
            TimeSpan::from_ticks(value.elapsed_game_time_ticks),
            value.is_running_slowly != sys::CNA_FALSE,
        )
    }
}

/// Callback-scoped access to the host-owned XNA game state.
///
/// The context cannot escape a lifecycle callback. In particular, a
/// [`GraphicsDevice`] borrowed from it cannot be retained across frames because
/// CNA invalidates the corresponding native handle at callback return.
pub struct GameContext<'callback> {
    pub(crate) native: &'callback Arc<Native>,
    pub(crate) handle: sys::CNA_Handle,
}

#[allow(non_snake_case)]
impl GameContext<'_> {
    pub fn GraphicsDevice(&self) -> Result<GraphicsDevice<'_>> {
        GraphicsDevice::borrow(self.native, self.handle)
    }

    pub fn Exit(&self) -> Result<()> {
        self.native.request_game_exit(self.handle)
    }
}

/// User lifecycle contract composed with CNA's host-owned XNA game state.
#[allow(non_snake_case)]
pub trait Game {
    fn Initialize(&mut self, _game: &mut GameContext<'_>) -> Result<()> {
        Ok(())
    }

    fn LoadContent(&mut self, _game: &mut GameContext<'_>) -> Result<()> {
        Ok(())
    }

    fn Update(&mut self, _game: &mut GameContext<'_>, _time: &GameTime) -> Result<()> {
        Ok(())
    }

    fn Draw(&mut self, _game: &mut GameContext<'_>, _time: &GameTime) -> Result<()> {
        Ok(())
    }

    fn UnloadContent(&mut self, _game: &mut GameContext<'_>) -> Result<()> {
        Ok(())
    }

    fn OnExiting(&mut self, _game: &mut GameContext<'_>) -> Result<()> {
        Ok(())
    }
}

struct CallbackState<G> {
    game: G,
    native: Arc<Native>,
    callback_error: Option<CnaError>,
    frame_limit: Option<u64>,
    drawn_frames: u64,
}

#[derive(Clone, Copy)]
enum Lifecycle {
    Initialize,
    LoadContent,
    Update,
    Draw,
    UnloadContent,
    Exiting,
}

/// Runs a game through CNA's native loop until it exits.
pub fn run<G: Game>(game: G) -> Result<()> {
    run_inner(game, None)
}

/// Runs a real CNA game and requests exit after exactly `frames` successful draws.
///
/// This is a binding-level deterministic test utility, not an XNA member.
pub fn run_for_frames<G: Game>(game: G, frames: u64) -> Result<()> {
    if frames == 0 {
        return Err(CnaError::InvalidInput(
            "frame limit must be greater than zero",
        ));
    }
    run_inner(game, Some(frames))
}

fn run_inner<G: Game>(game: G, frame_limit: Option<u64>) -> Result<()> {
    let native = Native::load()?;
    let mut state = Box::new(CallbackState {
        game,
        native: Arc::clone(&native),
        callback_error: None,
        frame_limit,
        drawn_frames: 0,
    });
    let context = core::ptr::addr_of_mut!(*state).cast::<c_void>();

    let callbacks = sys::CNA_GameCallbacks {
        struct_size: size_of::<sys::CNA_GameCallbacks>() as u32,
        struct_version: 1,
        load_content: Some(callback::<G, { Lifecycle::LoadContent as u8 }>),
        update: Some(callback::<G, { Lifecycle::Update as u8 }>),
        draw: Some(callback::<G, { Lifecycle::Draw as u8 }>),
        unload_content: Some(callback::<G, { Lifecycle::UnloadContent as u8 }>),
        exiting: Some(callback::<G, { Lifecycle::Exiting as u8 }>),
        context,
    };
    let title = b"CNA Rust\0";
    let create_info = sys::CNA_GameCreateInfo {
        struct_size: size_of::<sys::CNA_GameCreateInfo>() as u32,
        struct_version: 1,
        is_fixed_time_step: sys::CNA_TRUE,
        reserved: [0; 7],
        target_elapsed_time_ticks: 166_667,
        window_title: sys::CNA_StringView {
            data: title.as_ptr().cast(),
            byte_length: (title.len() - 1) as u64,
        },
        callbacks: &callbacks,
    };
    let mut handle = sys::CNA_INVALID_HANDLE;
    native.create_game(&create_info, &mut handle)?;

    let hooks = sys::CNA_GameFrameHooks {
        struct_size: size_of::<sys::CNA_GameFrameHooks>() as u32,
        struct_version: 1,
        initialize: Some(callback::<G, { Lifecycle::Initialize as u8 }>),
        begin_run: None,
        end_run: None,
        begin_draw: None,
        end_draw: None,
        context,
    };
    if let Err(error) = native.set_game_frame_hooks(handle, &hooks) {
        let _ = native.destroy_game(handle);
        return Err(error);
    }

    let run_result = native.run_game(handle);

    // ABI 0.7 requires every owned C child to be gone before game_destroy,
    // while its native Shutdown invokes unload_content only after enforcing
    // that precondition. Give Rust owners their deterministic release point
    // now. CNA will issue the lifecycle notification again during Shutdown;
    // Dispose/Drop and user UnloadContent implementations must be idempotent.
    let cleanup_result = catch_unwind(AssertUnwindSafe(|| {
        let mut game_context = GameContext {
            native: &state.native,
            handle,
        };
        state.game.UnloadContent(&mut game_context)
    }));
    if state.callback_error.is_none() {
        match cleanup_result {
            Ok(Err(error)) => state.callback_error = Some(error),
            Err(_) => {
                state.callback_error = Some(CnaError::Callback(
                    "Rust panic was contained during pre-destroy cleanup".to_owned(),
                ));
            }
            Ok(Ok(())) => {}
        }
    }

    let destroy_result = native.destroy_game(handle);

    if let Some(error) = state.callback_error.take() {
        return Err(error);
    }
    run_result?;
    destroy_result
}

unsafe extern "C" fn callback<G: Game, const LIFECYCLE: u8>(
    game_handle: sys::CNA_Handle,
    time: *const sys::CNA_GameTime,
    context: *mut c_void,
    _error: *mut sys::CNA_CallbackError,
) -> sys::CNA_Result {
    // SAFETY: `run_inner` passes a stable boxed `CallbackState<G>` pointer and
    // CNA invokes callbacks only before the enclosing run/destroy completes.
    let state = unsafe { &mut *context.cast::<CallbackState<G>>() };
    let mut game_context = GameContext {
        native: &state.native,
        handle: game_handle,
    };
    let lifecycle = match LIFECYCLE {
        value if value == Lifecycle::Initialize as u8 => Lifecycle::Initialize,
        value if value == Lifecycle::LoadContent as u8 => Lifecycle::LoadContent,
        value if value == Lifecycle::Update as u8 => Lifecycle::Update,
        value if value == Lifecycle::Draw as u8 => Lifecycle::Draw,
        value if value == Lifecycle::UnloadContent as u8 => Lifecycle::UnloadContent,
        _ => Lifecycle::Exiting,
    };
    let game_time = read_time(time);
    let result = catch_unwind(AssertUnwindSafe(|| match lifecycle {
        Lifecycle::Initialize => state.game.Initialize(&mut game_context),
        Lifecycle::LoadContent => state.game.LoadContent(&mut game_context),
        Lifecycle::Update => state.game.Update(&mut game_context, &game_time),
        Lifecycle::Draw => {
            state.game.Draw(&mut game_context, &game_time)?;
            state.drawn_frames += 1;
            if state.frame_limit == Some(state.drawn_frames) {
                game_context.Exit()?;
            }
            Ok(())
        }
        Lifecycle::UnloadContent => state.game.UnloadContent(&mut game_context),
        Lifecycle::Exiting => state.game.OnExiting(&mut game_context),
    }));

    match result {
        Ok(Ok(())) => sys::CNA_RESULT_SUCCESS,
        Ok(Err(error)) => {
            state.callback_error = Some(error);
            sys::CNA_RESULT_CALLBACK
        }
        Err(_) => {
            state.callback_error = Some(CnaError::Callback(
                "Rust panic was contained at the FFI boundary".to_owned(),
            ));
            sys::CNA_RESULT_CALLBACK
        }
    }
}

fn read_time(time: *const sys::CNA_GameTime) -> GameTime {
    if time.is_null() {
        GameTime::new()
    } else {
        // SAFETY: CNA documents the pointer as non-null and callback-scoped for
        // update/draw callbacks; it is read and copied synchronously.
        GameTime::from(unsafe { &*time })
    }
}

#[cfg(test)]
mod tests {
    use super::{GameTime, TimeSpan};

    #[test]
    fn timespan_is_signed_and_tick_exact() {
        let negative = TimeSpan::from_ticks(-1);
        assert_eq!(negative.Ticks(), -1);
        assert_eq!((negative + TimeSpan::from_ticks(2)).Ticks(), 1);
        assert_eq!(TimeSpan::FromSeconds(0.000_4).Ticks(), 0);
        assert_eq!(TimeSpan::FromSeconds(0.000_5).Ticks(), 10_000);
    }

    #[test]
    fn game_time_exposes_xna_properties() {
        let value = GameTime::from_total_game_time_and_elapsed_game_time_and_is_running_slowly(
            TimeSpan::from_ticks(20),
            TimeSpan::from_ticks(3),
            true,
        );
        assert_eq!(value.TotalGameTime().Ticks(), 20);
        assert_eq!(value.ElapsedGameTime().Ticks(), 3);
        assert!(value.IsRunningSlowly());
    }
}
