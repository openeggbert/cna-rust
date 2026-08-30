//! Native qualification for the wider profile's GamerServices object model.
//!
//! Everything here runs against a live CNA library. The host has no gamer
//! service, so the test does what a platform layer would: it publishes a
//! roster through `cna::extensions::gamer_services` and then reads it back
//! through the strict XNA surface. That is the only honest way to exercise
//! `Gamer.SignedInGamers` on a headless host -- the strict projection itself
//! never invents a gamer.
//!
//! Assertions are semantic. A test that only proved a call returned `Ok` would
//! pass against a projection that answered the wrong gamer, released a
//! borrowed handle, or handed out a fresh object for each read of one roster
//! position; each of those has its own negative case below.

#![allow(non_snake_case)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use cna::extensions::gamer_services::{
    PendingGuideRequest, SignedInGamerPublisher, SignedInGamerRegistration,
};
use cna::Microsoft::Xna::Framework::GamerServices::{
    Achievement, Gamer, GamerServicesDispatcher, Guide, LeaderboardIdentity, LeaderboardKey,
    NotificationPosition, PropertyDictionary, SignedInGamer,
};
use cna::Microsoft::Xna::Framework::PlayerIndex;
use cna::{CnaError, GamerBase, GamerCollectionBase, PropertyValueKind, Result};

fn native_enabled() -> bool {
    std::env::var_os("CNA_NATIVE_LIBRARY").is_some()
}

/// CNA's gamer services are process-global, exactly as XNA's statics are, so
/// two tests that publish a roster or open a Guide screen at the same time are
/// fighting over one object. This is the same guard the Framework suite uses
/// for the one live `Game`.
fn gamer_services_guard() -> MutexGuard<'static, ()> {
    static SERVICES: OnceLock<Mutex<()>> = OnceLock::new();
    SERVICES
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn roster(tags: &[(&str, PlayerIndex)]) -> Vec<SignedInGamerRegistration> {
    tags.iter()
        .map(|(tag, slot)| SignedInGamerRegistration {
            gamertag: (*tag).to_owned(),
            is_signed_in_to_live: false,
            is_guest: false,
            player_index: *slot,
        })
        .collect()
}

#[test]
fn an_unpublished_roster_is_empty_rather_than_invented() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    let gamers = Gamer::SignedInGamers()?;
    // A headless host has no gamer service. Empty is the true answer, and the
    // projection must not manufacture a player to fill it.
    assert_eq!(gamers.Count()?, 0);
    assert!(gamers.Item(PlayerIndex::One)?.is_none());
    assert!(matches!(
        gamers.ItemAt(0),
        Err(CnaError::InvalidInput(_))
    ));
    Ok(())
}

#[test]
fn a_published_roster_reaches_the_strict_surface() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    let publisher = SignedInGamerPublisher::publish(&roster(&[
        ("alpha", PlayerIndex::One),
        ("beta", PlayerIndex::Two),
    ]))?;

    let gamers = Gamer::SignedInGamers()?;
    assert_eq!(gamers.Count()?, 2);
    assert_eq!(gamers.ItemAt(0)?.Gamertag()?, "alpha");
    assert_eq!(gamers.ItemAt(1)?.Gamertag()?, "beta");

    // The player-index indexer is a different question from the position one
    // and must not be answered by position.
    let second = gamers
        .Item(PlayerIndex::Two)?
        .expect("slot two is occupied");
    assert_eq!(second.Gamertag()?, "beta");
    assert_eq!(second.PlayerIndex()?, PlayerIndex::Two);
    assert!(gamers.Item(PlayerIndex::Four)?.is_none());

    // Out of range stays out of range on a populated roster.
    assert!(matches!(gamers.ItemAt(2), Err(CnaError::InvalidInput(_))));
    assert!(matches!(gamers.ItemAt(-1), Err(CnaError::InvalidInput(_))));

    drop(publisher);
    // Retiring the roster must actually clear it: a projection that cached a
    // snapshot instead of reading CNA would still answer two here.
    assert_eq!(Gamer::SignedInGamers()?.Count()?, 0);
    Ok(())
}

#[test]
fn one_roster_position_keeps_one_logical_identity() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    let _publisher = SignedInGamerPublisher::publish(&roster(&[("stable", PlayerIndex::One)]))?;
    let gamers = Gamer::SignedInGamers()?;

    // CNA answers a *new* view handle for every read of a roster position, so
    // a projection that skipped identity caching would hand out unrelated
    // objects here. The tag a caller sets through one read must be visible
    // through the next.
    let first = gamers.ItemAt(0)?;
    first.SetTag(0x5AFE)?;
    let second = gamers.ItemAt(0)?;
    assert_eq!(second.Tag()?, 0x5AFE);

    // And the shared identity must not have been produced by cloning state:
    // writing through the second read is visible through the first.
    second.SetTag(0x1234)?;
    assert_eq!(first.Tag()?, 0x1234);
    Ok(())
}

#[test]
fn a_gamer_survives_the_collection_that_answered_it() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    let _publisher = SignedInGamerPublisher::publish(&roster(&[("outlives", PlayerIndex::One)]))?;
    let gamer = {
        let gamers = Gamer::SignedInGamers()?;
        gamers.ItemAt(0)?
    };
    // The roster view is an owned handle over a borrowed gamer: dropping the
    // collection facade must not have released the gamer CNA still publishes.
    assert_eq!(gamer.Gamertag()?, "outlives");
    assert!(!gamer.IsDisposed()?);
    Ok(())
}

#[test]
fn a_signed_in_gamers_own_objects_answer_from_cna() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    let _publisher = SignedInGamerPublisher::publish(&roster(&[("owner", PlayerIndex::One)]))?;
    let gamer = Gamer::SignedInGamers()?.ItemAt(0)?;

    assert!(!gamer.IsSignedInToLive()?);
    assert!(!gamer.IsGuest()?);

    // Presence is the gamer's own object: a write through it must be readable
    // back through a second read of the property.
    let presence = gamer.Presence()?;
    presence.SetPresenceValue(42)?;
    assert_eq!(gamer.Presence()?.PresenceValue()?, 42);

    let defaults = gamer.GameDefaults()?;
    // Colour preferences the gamer never expressed stay absent rather than
    // becoming black.
    assert!(defaults.PrimaryColor()?.is_none());
    assert!(defaults.SecondaryColor()?.is_none());

    let privileges = gamer.Privileges()?;
    // Whatever CNA reports, reading it twice must agree.
    assert_eq!(
        privileges.AllowOnlineSessions()?,
        gamer.Privileges()?.AllowOnlineSessions()?
    );

    // No friend service exists here. An empty collection is a success.
    let friends = gamer.GetFriends()?;
    assert_eq!(friends.Count()?, 0);
    assert!(!friends.IsDisposed()?);
    Ok(())
}

#[test]
fn awarded_achievements_come_back_from_cna() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    let _publisher = SignedInGamerPublisher::publish(&roster(&[("earner", PlayerIndex::One)]))?;
    let gamer = Gamer::SignedInGamers()?.ItemAt(0)?;

    let key = format!("cna-rust-test-{}", std::process::id());
    gamer.AwardAchievement(&key)?;

    let achievements = gamer.GetAchievements()?;
    assert!(achievements.Count()? >= 1);
    // The award must be findable by its own key, not merely by position.
    let earned: Achievement = achievements.Item(&key)?;
    assert_eq!(earned.Key()?, key);
    assert!(earned.IsEarned()?);
    // CNA has no achievement catalog, so the name is empty. A projection that
    // filled it in with the key would fail here.
    assert_eq!(earned.Name()?, "");
    assert_eq!(earned.GamerScore()?, 0);

    // A key nobody awarded is an error, not an empty achievement.
    assert!(achievements.Item("cna-rust-never-awarded").is_err());
    Ok(())
}

#[test]
fn the_leaderboard_writer_keeps_one_entry_per_identity() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    let _publisher = SignedInGamerPublisher::publish(&roster(&[("writer", PlayerIndex::One)]))?;
    let gamer = Gamer::SignedInGamers()?.ItemAt(0)?;
    let identity = LeaderboardIdentity::Create(LeaderboardKey::BestScoreLifeTime, 7);
    // Inherited from `Gamer`, so it reaches a `SignedInGamer` through the base
    // contract exactly as XNA's inheritance does.
    let writer = gamer.LeaderboardWriter()?;

    let entry = writer.GetLeaderboard(identity.clone())?;
    entry.SetRating(99)?;
    // XNA hands out the same entry for one identity, so a caller's edits
    // accumulate rather than being lost by a second lookup.
    assert_eq!(writer.GetLeaderboard(identity.clone())?.Rating()?, 99);
    assert!(Arc::ptr_eq(&entry, &writer.GetLeaderboard(identity)?));

    // A different game mode is a different leaderboard.
    let other = LeaderboardIdentity::Create(LeaderboardKey::BestScoreLifeTime, 8);
    assert_eq!(writer.GetLeaderboard(other)?.Rating()?, 0);
    Ok(())
}

#[test]
fn a_leaderboard_entrys_columns_are_the_entrys_own() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    let _publisher = SignedInGamerPublisher::publish(&roster(&[("columns", PlayerIndex::One)]))?;
    let gamer = Gamer::SignedInGamers()?.ItemAt(0)?;
    let writer = gamer.LeaderboardWriter()?;
    let entry = writer.GetLeaderboard(LeaderboardIdentity::CreateWithKey(
        LeaderboardKey::BestTimeLifeTime,
    ))?;

    let columns: PropertyDictionary = entry.Columns()?;
    columns.SetValue("laps", 12)?;
    columns.SetValueWithKeyAndValueAsStringAndString("track", "ring")?;
    assert_eq!(columns.Count()?, 2);
    assert_eq!(columns.GetValueInt32("laps")?, 12);
    assert_eq!(columns.GetValueString("track")?, "ring");

    // The dictionary is the entry's own, not a copy: a second read of the
    // property must see the same values.
    assert_eq!(entry.Columns()?.GetValueInt32("laps")?, 12);

    // The kind query names which typed accessor reads a value, and answers
    // nothing at all for a key the dictionary does not hold.
    let mut kind = None;
    assert!(columns.TryGetValue("laps", &mut kind)?);
    assert_eq!(kind, Some(PropertyValueKind::Int32));
    let mut absent = Some(PropertyValueKind::String);
    assert!(!columns.TryGetValue("missing", &mut absent)?);
    assert_eq!(absent, None);

    // Reading a value with the wrong accessor is CNA's error, never a
    // silently coerced number.
    assert!(columns.GetValueInt32("track").is_err());

    let keys: Vec<String> = columns.GetEnumerator()?.collect();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"laps".to_owned()));
    Ok(())
}

#[test]
fn a_disposed_object_reports_disposed_rather_than_answering() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    let _publisher = SignedInGamerPublisher::publish(&roster(&[("disposal", PlayerIndex::One)]))?;
    let gamer = Gamer::SignedInGamers()?.ItemAt(0)?;
    let profile = gamer.GetProfile()?;
    assert!(!profile.IsDisposed()?);

    profile.Dispose()?;
    assert!(profile.IsDisposed()?);
    // Disposing twice is a no-op, not a second native release.
    profile.Dispose()?;
    profile.Dispose()?;
    // And a disposed object refuses rather than reading a released handle.
    assert!(matches!(profile.Motto(), Err(CnaError::InvalidInput(_))));

    let friends = gamer.GetFriends()?;
    friends.Dispose()?;
    assert!(friends.IsDisposed()?);
    friends.Dispose()?;
    Ok(())
}

#[test]
fn the_guide_reports_what_cna_has_rather_than_a_screen() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    // The Guide's own state round-trips through CNA rather than through a
    // cached Rust value.
    let original = Guide::NotificationPosition()?;
    Guide::SetNotificationPosition(NotificationPosition::TopLeft)?;
    assert_eq!(Guide::NotificationPosition()?, NotificationPosition::TopLeft);
    Guide::SetNotificationPosition(NotificationPosition::BottomRight)?;
    assert_eq!(
        Guide::NotificationPosition()?,
        NotificationPosition::BottomRight
    );
    Guide::SetNotificationPosition(original)?;

    // The screen-saver flag belongs to the platform's display layer, and a
    // headless host has none. CNA answers `true` and its setter changes
    // nothing; the projection reports that rather than caching the value a
    // caller passed and reading it back as if it had taken effect.
    assert!(Guide::IsScreenSaverEnabled()?);
    Guide::SetIsScreenSaverEnabled(false)?;
    assert!(Guide::IsScreenSaverEnabled()?);

    // Trial-mode simulation is CNA's own state and does round-trip.
    Guide::SetSimulateTrialMode(true)?;
    assert!(Guide::SimulateTrialMode()?);
    Guide::SetSimulateTrialMode(false)?;
    assert!(!Guide::SimulateTrialMode()?);

    // `IsVisible` is derived from whether a Guide request is pending, not from
    // a stored flag: CNA accepts its own setter and ignores it, and the
    // projection does not pretend otherwise.
    PendingGuideRequest::reset_message_box()?;
    PendingGuideRequest::reset_keyboard_input()?;
    assert!(!Guide::IsVisible()?);
    PendingGuideRequest::set_visible(true)?;
    assert!(!Guide::IsVisible()?);
    let pending = Guide::BeginShowKeyboardInput(
        PlayerIndex::One,
        "Name",
        "Enter a name",
        "player",
        None,
        None,
    )?;
    // Now something *is* pending, so the derived property says so.
    assert!(Guide::IsVisible()?);
    assert!(PendingGuideRequest::has_keyboard_input()?);
    assert_eq!(PendingGuideRequest::keyboard_input_title()?, "Name");
    assert_eq!(
        PendingGuideRequest::keyboard_input_description()?,
        "Enter a name"
    );
    assert_eq!(PendingGuideRequest::keyboard_input_display_text()?, "player");

    // Cancelling is an answer, and a cancelled request is not the default
    // text: a projection that returned `defaultText` here would be inventing
    // what somebody typed.
    PendingGuideRequest::cancel_keyboard_input()?;
    assert!(PendingGuideRequest::keyboard_input_was_canceled()?);
    assert_eq!(Guide::EndShowKeyboardInput(&pending)?, "");
    PendingGuideRequest::reset_keyboard_input()?;
    assert!(!Guide::IsVisible()?);
    Ok(())
}

#[test]
fn a_message_box_stays_unanswered_until_something_answers_it() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    let observed = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&observed);
    let result = Guide::BeginShowMessageBox(
        PlayerIndex::One,
        "Quit?",
        "Leave the game?",
        &["Yes", "No"],
        1,
        cna::Microsoft::Xna::Framework::GamerServices::MessageBoxIcon::Warning,
        Some(Box::new(move |_| {
            counted.fetch_add(1, Ordering::SeqCst);
        })),
        None,
    )?;
    // The completion callback runs exactly once, and it runs before `End`.
    assert_eq!(observed.load(Ordering::SeqCst), 1);
    assert!(result.IsCompleted());
    assert!(result.CompletedSynchronously());

    // Nobody has chosen a button. CNA refuses rather than answering, and the
    // projection reports that refusal: inventing the focus button here would
    // be inventing a person's decision.
    assert!(PendingGuideRequest::has_message_box()?);
    assert_eq!(PendingGuideRequest::message_box_focus_button()?, 1);
    let unanswered = Guide::EndShowMessageBox(&result);
    assert!(matches!(unanswered, Err(CnaError::Native { .. })));

    // Answer it the way CNA lets a host answer it, and a fresh request reports
    // that exact choice.
    PendingGuideRequest::reset_message_box()?;
    let answered = Guide::BeginShowMessageBox(
        PlayerIndex::One,
        "Quit?",
        "Leave the game?",
        &["Yes", "No"],
        1,
        cna::Microsoft::Xna::Framework::GamerServices::MessageBoxIcon::Warning,
        None,
        None,
    )?;
    PendingGuideRequest::click_message_box(0)?;
    assert!(!PendingGuideRequest::has_message_box()?);
    assert_eq!(Guide::EndShowMessageBox(&answered)?, Some(0));
    // `End` is one-shot.
    assert!(matches!(
        Guide::EndShowMessageBox(&answered),
        Err(CnaError::InvalidInput(_))
    ));
    Ok(())
}

#[test]
fn a_panicking_completion_callback_never_crosses_c() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    let _publisher = SignedInGamerPublisher::publish(&roster(&[("panic", PlayerIndex::One)]))?;
    let gamer = Gamer::SignedInGamers()?.ItemAt(0)?;
    let outcome = gamer.BeginGetAchievements(
        Some(Box::new(|_| panic!("intentional completion panic"))),
        None,
    );
    assert!(matches!(outcome, Err(CnaError::Callback(_))));
    // The library is still usable afterwards.
    assert_eq!(gamer.Gamertag()?, "panic");
    Ok(())
}

#[test]
fn the_async_pattern_enforces_the_clr_end_rules() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    let _publisher = SignedInGamerPublisher::publish(&roster(&[("async", PlayerIndex::One)]))?;
    let gamer = Gamer::SignedInGamers()?.ItemAt(0)?;

    let state: Arc<dyn std::any::Any + Send + Sync> = Arc::new(7_i32);
    let result = gamer.BeginGetAchievements(None, Some(Arc::clone(&state)))?;
    // XNA passes the caller's state through untouched.
    let returned = result.AsyncState().expect("state was supplied");
    assert_eq!(returned.downcast_ref::<i32>(), Some(&7));

    let achievements = gamer.EndGetAchievements(&result)?;
    assert!(achievements.Count()? >= 0);
    // One End per result.
    assert!(matches!(
        gamer.EndGetAchievements(&result),
        Err(CnaError::InvalidInput(_))
    ));

    // A result belongs to the operation that made it: handing an achievements
    // result to a different End must fail rather than reinterpret the value.
    let other = gamer.BeginGetAchievements(None, None)?;
    assert!(matches!(
        SignedInGamer::EndAwardAchievement(&gamer, &other),
        Err(CnaError::InvalidInput(_))
    ));
    Ok(())
}

#[test]
fn sign_in_handlers_stop_receiving_once_removed() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _services = gamer_services_guard();
    let seen = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&seen);
    let registration =
        SignedInGamer::AddSignedInHandler(Box::new(move |_: &dyn std::any::Any, _| {
            counted.fetch_add(1, Ordering::SeqCst);
        }));
    assert_ne!(registration, 0);

    // The dispatcher is where a refused subscription would surface. On this
    // host CNA accepts it, so Update must be clean.
    GamerServicesDispatcher::Update()?;

    assert!(SignedInGamer::RemoveSignedInHandler(registration));
    // Removing twice reports that there was nothing left to remove.
    assert!(!SignedInGamer::RemoveSignedInHandler(registration));

    // Publishing a roster after the handler is gone must not reach it. CNA
    // raises no sign-in event for a published roster on this host either, so
    // the count stays zero for both reasons -- the assertion catches a
    // projection that invented one.
    let _publisher = SignedInGamerPublisher::publish(&roster(&[("late", PlayerIndex::One)]))?;
    GamerServicesDispatcher::Update()?;
    assert_eq!(seen.load(Ordering::SeqCst), 0);
    Ok(())
}
