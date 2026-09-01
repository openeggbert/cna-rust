//! Native qualification for the wider profile's `Net` object model.
//!
//! CNA's local session type needs no network, so everything below runs in one
//! process against the live library. What one process cannot produce is a
//! remote participant, and the strict projection never invents one:
//! `RemoteGamers` is empty until a host supplies a peer through
//! `cna::extensions::net`, which is CNA's own injection surface and is
//! deliberately outside `Microsoft.Xna.Framework`.
//!
//! Every classification the session graph can reach is exercised here except
//! one: a genuine second machine. That stays `NO_LIVE_PEER`, and the tests say
//! so rather than dressing an injected gamer up as a real one.

#![allow(non_snake_case)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use cna::extensions::gamer_services::{SignedInGamerPublisher, SignedInGamerRegistration};
use cna::extensions::net::{
    ApplySessionProperties, DiscoveredSession, DiscoveredSessionExt, DiscoveredSessionInjection,
    LastJoinError, LiveSessionCount, NetworkEventInjection, PendingSessionActionCount,
    RemoteGamerInjection,
};
use cna::Microsoft::Xna::Framework::Net::{
    NetworkSession, NetworkSessionEndReason, NetworkSessionJoinError, NetworkSessionProperties,
    NetworkSessionState, NetworkSessionType, PacketReader, PacketWriter, SendDataOptions,
};
use cna::Microsoft::Xna::Framework::{PlayerIndex, TimeSpan};
use cna::{
    CnaError, GamerBase, GamerCollectionBase, NetworkGamerBase, ReadOnlyCollectionBase, Result,
};

fn native_enabled() -> bool {
    std::env::var_os("CNA_NATIVE_LIBRARY").is_some()
}

/// CNA's sessions and its gamer roster are both process-global, so two tests
/// that touch either at once are fighting over one object.
fn net_guard() -> MutexGuard<'static, ()> {
    static NET: OnceLock<Mutex<()>> = OnceLock::new();
    NET.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Publishes one signed-in gamer, which every session needs.
///
/// XNA's `Create(sessionType, maxLocalGamers, maxGamers)` makes the first
/// signed-in gamer the host, so an empty roster is an argument failure rather
/// than an empty session. CNA preserves that, and one test asserts it.
fn publish_one(tag: &str) -> Result<SignedInGamerPublisher> {
    SignedInGamerPublisher::publish(&[SignedInGamerRegistration {
        gamertag: tag.to_owned(),
        is_signed_in_to_live: false,
        is_guest: false,
        player_index: PlayerIndex::One,
    }])
}

#[test]
fn a_local_session_reports_the_state_it_was_created_with() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();
    let _publisher = publish_one("localhost")?;

    let session = NetworkSession::Create(NetworkSessionType::Local, 1, 4)?;
    assert_eq!(session.SessionType()?, NetworkSessionType::Local);
    assert_eq!(session.SessionState()?, NetworkSessionState::Lobby);
    assert!(!session.IsDisposed()?);

    // The signed-in gamer became the host, so the local roster holds it and
    // the remote roster is empty -- one process has no peer, and nothing here
    // invents one.
    assert_eq!(session.AllGamers()?.Count()?, 1);
    assert_eq!(session.LocalGamers()?.Count()?, 1);
    assert_eq!(session.RemoteGamers()?.Count()?, 0);
    assert_eq!(session.PreviousGamers()?.Count()?, 0);
    assert!(session.IsHost()?);
    // The host exists, but CNA's C layer has no gamer-base access for a
    // network gamer, so its gamertag is a refusal rather than a guess. The
    // session-local identity it *does* publish is readable.
    let host = session.Host()?;
    assert_eq!(host.Id()?, 0);
    assert!(matches!(host.Gamertag(), Err(CnaError::Native { .. })));
    assert!(session.FindGamerById(200)?.is_none());

    // Settable session state round-trips through CNA.
    session.SetMaxGamers(8)?;
    assert_eq!(session.MaxGamers()?, 8);
    session.SetAllowHostMigration(true)?;
    assert!(session.AllowHostMigration()?);
    session.SetAllowJoinInProgress(true)?;
    assert!(session.AllowJoinInProgress()?);
    session.SetSimulatedPacketLoss(0.25)?;
    assert!((session.SimulatedPacketLoss()? - 0.25).abs() < f32::EPSILON);
    let latency = TimeSpan::FromMilliseconds(50.0);
    session.SetSimulatedLatency(latency)?;
    assert_eq!(session.SimulatedLatency()?.Ticks(), latency.Ticks());

    // XNA's own limits are constants, not something CNA reports.
    assert_eq!(NetworkSession::MaxSupportedGamers, 31);
    assert_eq!(NetworkSession::MaxPreviousGamers, 100);

    session.Dispose()?;
    assert!(session.IsDisposed()?);
    session.Dispose()?;
    Ok(())
}

#[test]
fn session_properties_round_trip_through_cna() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();
    let _publisher = publish_one("localhost")?;

    let mut properties = NetworkSessionProperties::new();
    properties.SetItem(0, Some(7));
    properties.SetItem(3, Some(-1));
    let session = NetworkSession::CreateWithSessionTypeAndMaxLocalGamersAndMaxGamersAndPrivateGamerSlotsAndSessionProperties(
        NetworkSessionType::Local,
        1,
        4,
        1,
        &properties,
    )?;

    // The session holds what it was given: a projection that dropped the
    // properties on the way through CNA would answer eight empty slots.
    let stored = session.SessionProperties()?;
    assert_eq!(stored.Item(0), Some(7));
    assert_eq!(stored.Item(3), Some(-1));
    assert_eq!(stored.Item(1), None);
    assert_eq!(stored.Count(), 8);
    assert_eq!(session.PrivateGamerSlots()?, 1);

    // XNA's getter is a reference: `session.SessionProperties[0] = 5` changes
    // the session. CNA hands back a copy, so writing to `stored` alone changes
    // nothing -- which is the state this projection was in, silently.
    let mut mutated = stored;
    mutated.SetItem(0, Some(5));
    mutated.SetItem(3, None);
    mutated.SetItem(7, Some(11));
    assert_eq!(
        session.SessionProperties()?.Item(0),
        Some(7),
        "a copy is a copy: mutating it must not reach the session"
    );

    ApplySessionProperties(&session, &mutated)?;
    let applied = session.SessionProperties()?;
    assert_eq!(applied.Item(0), Some(5));
    assert_eq!(applied.Item(3), None, "clearing a slot must clear it");
    assert_eq!(applied.Item(7), Some(11));
    assert_eq!(applied.Count(), 8);
    Ok(())
}

#[test]
fn a_search_with_no_peer_answers_an_empty_collection() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();
    let _publisher = publish_one("localhost")?;

    let properties = NetworkSessionProperties::new();
    // A `Local` session is not discoverable -- there is nothing to find on
    // this machine -- and XNA rejects the argument rather than answering an
    // empty list.
    assert!(matches!(
        NetworkSession::Find(NetworkSessionType::Local, 1, &properties),
        Err(CnaError::Native { .. })
    ));
    // A system-link search that finds nothing is a *success* with nothing in
    // it, not a failure and not a fabricated lobby list.
    //
    // The count is deliberately not asserted to be zero. A system-link search
    // is machine-wide, and this crate's test binaries run concurrently, so
    // another one advertising a session makes "nobody is advertising" false --
    // which is exactly how this test used to fail about one full-suite run in
    // ten. What is asserted instead holds however many sessions exist: the
    // search succeeds, every index below the count resolves, and the first
    // index past it does not. A fabricated lobby list fails that just as
    // surely as a non-empty one would have failed the old assertion.
    let found = NetworkSession::Find(NetworkSessionType::SystemLink, 1, &properties)?;
    let count = found.Count()?;
    if count != 0 {
        println!(
            "NOTE: {count} system-link session(s) are advertised on this machine; \
             another test binary is the usual reason"
        );
    }
    for index in 0..count {
        found
            .ItemAt(index)
            .unwrap_or_else(|error| panic!("index {index} is below the count but failed: {error}"));
    }
    assert!(
        matches!(found.ItemAt(count), Err(CnaError::Native { .. })),
        "the first index past the count must be refused, not answered"
    );
    assert!(!found.IsDisposed()?);
    found.Dispose()?;
    assert!(found.IsDisposed()?);
    Ok(())
}

#[test]
fn a_local_gamer_joins_the_session_and_carries_its_signed_in_gamer() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();

    let _publisher = publish_one("host")?;
    let session = NetworkSession::Create(NetworkSessionType::Local, 1, 4)?;

    let locals = session.LocalGamers()?;
    assert_eq!(locals.Count()?, 1);
    let local = locals.ItemAt(0)?;
    assert!(local.IsLocal()?);
    assert!(!local.HasLeftSession()?);
    // The session it belongs to is the one that admitted it.
    assert_eq!(local.Session()?.MaxGamers()?, session.MaxGamers()?);

    // CNA still has no gamer-base access for a network gamer, so the gamertag
    // a roster read could trivially have guessed from the published roster is
    // a refusal instead. Reporting the refusal is the point, and the message
    // is CNA's own rather than a summary of it. (RUST-BEHAVIOR-010,
    // re-measured on cnanext 599d14e5 and still blocked.)
    assert!(
        matches!(local.Gamertag(), Err(CnaError::Native { message, .. })
            if message.contains("does not name a gamer this call can use")),
        "a network gamer still cannot answer its inherited Gamer members"
    );
    assert!(matches!(local.DisplayName(), Err(CnaError::Native { .. })));

    // The signed-in gamer behind a local gamer *is* reachable now.
    // RUST-BEHAVIOR-011 recorded `NOT_SUPPORTED` -- "Signed-in gamers have no
    // C representation yet" -- and cnanext has since grown one. This asserts
    // the working path rather than tolerating either, so a regression fails
    // here instead of passing quietly.
    let signed_in = local
        .SignedInGamer()
        .expect("a local gamer answers the signed-in gamer behind it");
    assert_eq!(
        signed_in.Gamertag()?,
        "host",
        "and it is the gamer that was published, not some other one"
    );

    // `IsReady` is session state a caller sets, and `ResetReady` clears it.
    local.SetIsReady(true)?;
    assert!(local.IsReady()?);
    session.ResetReady()?;
    assert!(!local.IsReady()?);

    // The whole roster sees it too.
    assert_eq!(session.AllGamers()?.Count()?, 1);
    assert_eq!(session.RemoteGamers()?.Count()?, 0);
    Ok(())
}

#[test]
fn a_remote_gamer_only_exists_when_a_host_supplies_one() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();
    let _publisher = publish_one("localhost")?;

    let session = NetworkSession::Create(NetworkSessionType::Local, 1, 4)?;
    // Strict XNA alone can never produce a peer in one process: the host is
    // local and the remote roster is empty.
    assert_eq!(session.RemoteGamers()?.Count()?, 0);
    assert_eq!(session.LocalGamers()?.Count()?, 1);

    // CNA's injection surface is what a host uses to stand in for the network.
    let remote = RemoteGamerInjection::admit(&session, "peer")?;
    assert!(!remote.IsLocal()?);

    let remotes = session.RemoteGamers()?;
    assert_eq!(remotes.Count()?, 1);
    // The peer is in the remote roster and the host is not, so the combined
    // roster holds both. A projection that answered the same roster for every
    // identity would report one here.
    assert!(!remotes.ItemAt(0)?.IsLocal()?);
    assert_eq!(session.LocalGamers()?.Count()?, 1);
    assert_eq!(session.AllGamers()?.Count()?, 2);

    // The gamer is findable by the identifier the session gave it.
    let id = remote.Id()?;
    let found = session.FindGamerById(id)?.expect("the admitted gamer");
    assert_eq!(found.Id()?, id);

    // Removing it puts it in the previous-gamers roster rather than deleting
    // the history.
    RemoteGamerInjection::remove(&session, &remote, NetworkSessionEndReason::Disconnected)?;
    assert_eq!(session.RemoteGamers()?.Count()?, 0);
    assert!(session.PreviousGamers()?.Count()? >= 1);
    Ok(())
}

#[test]
fn session_events_fire_once_and_stop_when_removed() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();
    let _publisher = publish_one("localhost")?;

    let session = NetworkSession::Create(NetworkSessionType::Local, 1, 4)?;
    let started = Arc::new(AtomicUsize::new(0));
    let ended = Arc::new(AtomicUsize::new(0));

    let counted = Arc::clone(&started);
    let start_handler = session.AddGameStartedHandler(Box::new(
        move |_: &dyn std::any::Any, _| {
            counted.fetch_add(1, Ordering::SeqCst);
        },
    ));
    assert_ne!(start_handler, 0);

    let counted = Arc::clone(&ended);
    let end_handler = session.AddGameEndedHandler(Box::new(
        move |_: &dyn std::any::Any, _| {
            counted.fetch_add(1, Ordering::SeqCst);
        },
    ));

    // A subscription CNA refused would surface here, not from `+=`.
    session.Update()?;

    // The transitions are queued and land on `Update`, which is where XNA
    // delivers a session's events from.
    session.StartGame()?;
    session.Update()?;
    assert_eq!(session.SessionState()?, NetworkSessionState::Playing);
    // `EndGame` returns the session to the lobby -- `Ended` is a session that
    // has terminated, not one whose game finished.
    session.EndGame()?;
    session.Update()?;
    assert_eq!(session.SessionState()?, NetworkSessionState::Lobby);

    // Exactly once each: a projection that re-raised on every `Update` would
    // fail after these extra updates.
    session.Update()?;
    session.Update()?;
    assert_eq!(started.load(Ordering::SeqCst), 1);
    assert_eq!(ended.load(Ordering::SeqCst), 1);

    // A removed handler stops receiving, and removing twice reports that
    // nothing was left to remove.
    assert!(session.RemoveGameStartedHandler(start_handler));
    assert!(!session.RemoveGameStartedHandler(start_handler));
    assert!(session.RemoveGameEndedHandler(end_handler));

    session.Dispose()?;
    let replay = NetworkSession::Create(NetworkSessionType::Local, 1, 4)?;
    replay.StartGame()?;
    replay.Update()?;
    assert_eq!(started.load(Ordering::SeqCst), 1);
    Ok(())
}

#[test]
fn an_injected_state_change_reaches_a_session_handler() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();
    let _publisher = publish_one("localhost")?;

    let session = NetworkSession::Create(NetworkSessionType::Local, 1, 4)?;
    let reasons = Arc::new(Mutex::new(Vec::new()));
    let recorded = Arc::clone(&reasons);
    let registration = session.AddSessionEndedHandler(Box::new(
        move |_: &dyn std::any::Any, args: cna::Microsoft::Xna::Framework::Net::NetworkSessionEndedEventArgs| {
            recorded
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(args.EndReason());
        },
    ));
    assert_ne!(registration, 0);

    // A lone process has nothing to disconnect from, so the event only exists
    // because a host delivered it -- and the reason it carries must be the one
    // that was delivered, not a default.
    NetworkEventInjection::ended(&session, NetworkSessionEndReason::RemovedByHost)?;
    session.Update()?;
    assert_eq!(session.SessionState()?, NetworkSessionState::Ended);
    let seen = reasons
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(seen, vec![NetworkSessionEndReason::RemovedByHost]);
    Ok(())
}

#[test]
fn an_injected_packet_arrives_with_its_sender_and_bytes() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();

    let _publisher = publish_one("receiver")?;
    let session = NetworkSession::Create(NetworkSessionType::Local, 1, 4)?;
    let local = session.LocalGamers()?.ItemAt(0)?;
    let sender = RemoteGamerInjection::admit(&session, "sender")?;

    assert!(!local.IsDataAvailable()?);
    NetworkEventInjection::packet(&local, &sender, &[1, 2, 3, 4], SendDataOptions::Reliable)?;
    assert!(local.IsDataAvailable()?);

    let mut buffer = [0_u8; 16];
    let mut from = None;
    let received = local.ReceiveData(&mut buffer, &mut from)?;
    assert_eq!(received, 4);
    assert_eq!(&buffer[..4], &[1, 2, 3, 4]);
    // CNA delivers the payload; whether it also names a sender depends on the
    // session's own bookkeeping, so the assertion is that a reported sender is
    // the *right* gamer rather than that one is always reported. Comparison is
    // by session-local identifier, which is what CNA publishes for a network
    // gamer.
    if let Some(from) = from {
        assert_eq!(from.Id()?, sender.Id()?);
    }
    // The queue is drained, so a second read finds nothing.
    assert!(!local.IsDataAvailable()?);

    // The same packet read through a `PacketReader` decodes as XNA would.
    let mut writer = PacketWriter::new();
    writer.WriteWithValueAsSingle(1.5_f32)?;
    writer.WriteWithValueAsSingle(-2.5_f32)?;
    NetworkEventInjection::packet(
        &local,
        &sender,
        cna::extensions::net::PacketBytes(&writer),
        SendDataOptions::None,
    )?;
    let mut reader = PacketReader::new();
    let mut packet_sender = None;
    let length = local.ReceiveDataWithDataAndSender(&mut reader, &mut packet_sender)?;
    assert_eq!(length, 8);
    // The bytes decode as XNA's `PacketReader` would, which is what the packet
    // types exist for.
    assert_eq!(reader.ReadSingle()?, 1.5);
    assert_eq!(reader.ReadSingle()?, -2.5);
    if let Some(from) = packet_sender {
        assert_eq!(from.Id()?, sender.Id()?);
    }
    Ok(())
}

#[test]
fn a_packet_reader_receives_a_whole_large_packet_or_says_it_could_not() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();

    let _publisher = publish_one("receiver")?;
    let session = NetworkSession::Create(NetworkSessionType::Local, 1, 4)?;
    let local = session.LocalGamers()?.ItemAt(0)?;
    let sender = RemoteGamerInjection::admit(&session, "sender")?;

    // 4,096 bytes was the old buffer, and a packet past it came back short
    // with a success. XNA never returns a short packet: it sizes the reader to
    // the packet before reading it.
    let big: Vec<u8> = (0..20_000_u32).map(|index| (index % 251) as u8).collect();
    NetworkEventInjection::packet(&local, &sender, &big, SendDataOptions::Reliable)?;
    let mut reader = PacketReader::new();
    let mut from = None;
    let received = local.ReceiveDataWithDataAndSender(&mut reader, &mut from)?;
    assert_eq!(received, 20_000, "the whole packet, not the first 4,096 bytes");
    assert_eq!(reader.Length(), 20_000);
    reader.SetPosition(19_996);
    assert_eq!(
        reader.ReadSingle()?.to_bits(),
        u32::from_le_bytes([big[19_996], big[19_997], big[19_998], big[19_999]]),
        "the last four bytes must be the packet's own, read as XNA reads them"
    );

    // Above the stated ceiling the projection refuses rather than delivering a
    // packet with its tail removed. CNA truncates and reports the short count
    // as a success, so this is the one place the difference can be seen.
    let enormous = vec![7_u8; 64 * 1024 + 1];
    NetworkEventInjection::packet(&local, &sender, &enormous, SendDataOptions::Reliable)?;
    let mut reader = PacketReader::new();
    let mut from = None;
    assert!(
        matches!(
            local.ReceiveDataWithDataAndSender(&mut reader, &mut from),
            Err(CnaError::InvalidInput(_))
        ),
        "a packet past the ceiling is an error, not a silent short read"
    );
    Ok(())
}

fn discovered(host: &str, port: u16, sessionType: NetworkSessionType) -> DiscoveredSession {
    let mut properties = NetworkSessionProperties::new();
    properties.SetItem(2, Some(42));
    DiscoveredSession {
        host_gamertag: format!("{host}-host"),
        host_address: host.to_owned(),
        host_port: port,
        current_gamer_count: 3,
        open_public_gamer_slots: 5,
        open_private_gamer_slots: 1,
        session_type: sessionType,
        session_properties: properties,
        roundtrip: Some(TimeSpan::FromMilliseconds(25.0)),
    }
}

#[test]
fn an_injected_discovered_session_answers_every_xna_property_and_cnas_own() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();

    // `Find` on one machine finds nothing, so without an injection route the
    // whole of `AvailableNetworkSession` -- six XNA properties and three CNA
    // facts -- is unreachable rather than merely untested.
    let session = DiscoveredSessionInjection::session(&discovered("10.0.0.7", 27015, NetworkSessionType::PlayerMatch))?;

    assert_eq!(session.HostGamertag()?, "10.0.0.7-host");
    assert_eq!(session.CurrentGamerCount()?, 3);
    assert_eq!(session.OpenPublicGamerSlots()?, 5);
    assert_eq!(session.OpenPrivateGamerSlots()?, 1);
    assert_eq!(session.SessionProperties()?.Item(2), Some(42));
    let quality = session.QualityOfService()?;
    assert!(quality.IsAvailable());
    assert_eq!(quality.AverageRoundtripTime(), TimeSpan::FromMilliseconds(25.0));

    // XNA never says where the session is; CNA does, and a host layer that
    // makes its own connection decision needs all three.
    assert_eq!(session.connect_address()?, "10.0.0.7");
    assert_eq!(session.connect_port()?, 27015);
    assert_eq!(session.advertised_session_type()?, NetworkSessionType::PlayerMatch);

    // Identity, both halves. CNA publishes `equals` and `not_equals`
    // separately and the projection refuses an answer where they disagree.
    let other = DiscoveredSessionInjection::session(&discovered("10.0.0.8", 27016, NetworkSessionType::PlayerMatch))?;
    assert!(session.is_same_session(&session)?);
    assert!(!session.is_same_session(&other)?);

    let collection = DiscoveredSessionInjection::collection(&[
        discovered("10.0.0.7", 27015, NetworkSessionType::PlayerMatch),
        discovered("10.0.0.8", 27016, NetworkSessionType::PlayerMatch),
    ])?;
    assert_eq!(collection.Count()?, 2);
    assert_eq!(collection.ItemAt(1)?.connect_port()?, 27016);
    Ok(())
}

#[test]
fn a_refused_join_reports_the_error_xnas_exception_would_carry() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();
    let _publisher = publish_one("joiner")?;

    // Nothing is listening on this address, so the join fails. XNA puts the
    // reason on `NetworkSessionJoinException.JoinError`; an exception object
    // cannot cross the ABI, so CNA records it per thread and this is where a
    // caller reads it.
    //
    // `SystemLink` on purpose: CNA gates its real transport on that type, and
    // a `PlayerMatch` join never leaves the process, so it succeeds and there
    // is no join error to report.
    let unreachable =
        DiscoveredSessionInjection::session(&discovered("203.0.113.9", 27015, NetworkSessionType::SystemLink))?;
    let refused = NetworkSession::Join(&unreachable);
    assert!(refused.is_err(), "no session is listening on a TEST-NET-3 address");
    assert_eq!(
        LastJoinError()?,
        Some(NetworkSessionJoinError::SessionNotFound),
        "a refused join must report which of XNA's three reasons applied"
    );
    Ok(())
}

#[test]
fn a_disposed_session_stops_being_counted_and_its_packets_go_with_it() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();
    let _publisher = publish_one("counted")?;

    let before = LiveSessionCount()?;
    assert_eq!(PendingSessionActionCount()?, 0, "no creation is in flight");

    {
        let session = NetworkSession::Create(NetworkSessionType::Local, 1, 4)?;
        assert_eq!(
            LiveSessionCount()?,
            before + 1,
            "a created session is a session CNA holds"
        );

        // A queue that is cleared is a queue the next read finds empty.
        // Without the clear, a test that injected a packet and did not receive
        // it leaves the packet for whatever runs next.
        let local = session.LocalGamers()?.ItemAt(0)?;
        let sender = RemoteGamerInjection::admit(&session, "sender")?;
        NetworkEventInjection::packet(&local, &sender, &[9, 9, 9], SendDataOptions::Reliable)?;
        assert!(local.IsDataAvailable()?);
        NetworkEventInjection::clear_packets(&local)?;
        assert!(!local.IsDataAvailable()?, "a cleared queue is empty");

        session.Dispose()?;
    }
    assert_eq!(
        LiveSessionCount()?,
        before,
        "a released session is one CNA no longer holds"
    );
    Ok(())
}

#[test]
fn a_session_disposes_cleanly_after_handing_out_roster_views() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();
    let _publisher = publish_one("localhost")?;

    let session = NetworkSession::Create(NetworkSessionType::Local, 1, 4)?;
    let remote = RemoteGamerInjection::admit(&session, "viewed")?;
    // Reading the roster hands out borrowed views CNA counts. If disposal
    // released the session before them, CNA would refuse and the session would
    // still report itself live -- which is exactly what this asserts against.
    let all = session.AllGamers()?;
    assert_eq!(all.Count()?, 2);
    let _first = all.ItemAt(0)?;
    let _host_lookup = session.FindGamerById(remote.Id()?)?;

    session.Dispose()?;
    // Disposal has to succeed *and* be observable. If the views had been
    // released after the session rather than before it, CNA would have refused
    // the release and this would still report a live session.
    assert!(session.IsDisposed()?);
    // A disposed session refuses to act.
    assert!(session.StartGame().is_err());
    Ok(())
}

#[test]
fn an_async_create_completes_before_it_returns() -> Result<()> {
    if !native_enabled() {
        return Ok(());
    }
    let _net = net_guard();
    let _publisher = publish_one("localhost")?;

    let observed = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&observed);
    let state: Arc<dyn std::any::Any + Send + Sync> = Arc::new(11_u32);
    let result = NetworkSession::BeginCreate(
        NetworkSessionType::Local,
        1,
        4,
        Some(Box::new(move |_| {
            counted.fetch_add(1, Ordering::SeqCst);
        })),
        Some(Arc::clone(&state)),
    )?;
    assert_eq!(observed.load(Ordering::SeqCst), 1);
    assert!(result.IsCompleted());
    assert!(result.CompletedSynchronously());
    assert_eq!(
        result
            .AsyncState()
            .expect("state was supplied")
            .downcast_ref::<u32>(),
        Some(&11)
    );

    let session = NetworkSession::EndCreate(&result)?;
    assert_eq!(session.SessionType()?, NetworkSessionType::Local);
    // The asynchronous path used to substitute its own gamer limit and ignore
    // the requested one; upstream changed that during this milestone, so the
    // limit asked for above is the limit the session has. Asserting it means a
    // regression to the old substitution fails here.
    assert_eq!(
        session.MaxGamers()?,
        4,
        "BeginCreate preserves the requested gamer limit"
    );
    // One End per result.
    assert!(matches!(
        NetworkSession::EndCreate(&result),
        Err(CnaError::InvalidInput(_))
    ));

    // A result belongs to the operation that made it: handing a session result
    // to `EndFind` fails rather than reinterpreting the value.
    session.Dispose()?;
    let other = NetworkSession::BeginCreate(NetworkSessionType::Local, 1, 2, None, None)?;
    assert!(matches!(
        NetworkSession::EndFind(&other),
        Err(CnaError::InvalidInput(_))
    ));
    Ok(())
}
