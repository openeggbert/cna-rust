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
