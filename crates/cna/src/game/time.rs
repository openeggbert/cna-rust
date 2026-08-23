#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use cna_sys as sys;

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
        totalGameTime: TimeSpan,
        elapsedGameTime: TimeSpan,
    ) -> Self {
        Self::from_total_game_time_and_elapsed_game_time_and_is_running_slowly(
            totalGameTime,
            elapsedGameTime,
            false,
        )
    }

    #[must_use]
    pub const fn from_total_game_time_and_elapsed_game_time_and_is_running_slowly(
        totalGameTime: TimeSpan,
        elapsedGameTime: TimeSpan,
        isRunningSlowly: bool,
    ) -> Self {
        Self {
            total_game_time: totalGameTime,
            elapsed_game_time: elapsedGameTime,
            is_running_slowly: isRunningSlowly,
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

impl GameTime {
    pub(super) fn from_native(value: &sys::CNA_GameTime) -> Self {
        Self::from_total_game_time_and_elapsed_game_time_and_is_running_slowly(
            TimeSpan::from_ticks(value.total_game_time_ticks),
            TimeSpan::from_ticks(value.elapsed_game_time_ticks),
            value.is_running_slowly != sys::CNA_FALSE,
        )
    }
}
