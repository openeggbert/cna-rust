//! Microsoft XNA 4.0 `Net`.
//!
//! Like `GamerServices`, this belongs to the wider Windows runtime profile
//! (`tools/api-compat/profiles/xna40-windows-full.json`) rather than the
//! selected seven-assembly one, and it divides the same way: [`values`] is
//! pure managed Rust because CLR metadata is the whole of an enum ordinal's or
//! an exception identity's contract, and the object graph in [`session`] is
//! backed by `net.h`, `net_gamers.h` and `net_sessions.h`.
//!
//! # Ownership
//!
//! CNA's session routes distinguish three things and the projection follows
//! them exactly:
//!
//! - a **session** is owned, and `cna_network_session_destroy` refuses while a
//!   borrowed gamer view is still open;
//! - a **roster gamer** is a borrowed view over a gamer the session owns, so
//!   the session must outlive it and the session's disposal releases every
//!   cached view first;
//! - a **discovered session** copied out of a search result owns an
//!   independent copy, which is why it stays valid after the collection it
//!   came from is disposed.
//!
//! # What a host without a peer can prove
//!
//! CNA's local session type needs no network, so a session really is created,
//! its rosters really are read, and its properties really do round-trip. What
//! a lone process cannot produce is a remote participant, so `RemoteGamers`
//! stays empty and nothing here invents one. `cna::extensions::net` publishes
//! CNA's own routes for injecting a remote gamer and delivering a packet,
//! which is how a single process drives the session lifecycle deterministically
//! -- and it is deliberately outside the strict hierarchy, because a game
//! calling it is standing in for the network.

mod events;
mod session;
mod values;

pub use events::{
    GameEndedEventArgs, GameStartedEventArgs, GamerJoinedEventArgs, GamerLeftEventArgs,
    HostChangedEventArgs, NetworkSessionEndedEventArgs, WriteLeaderboardsEventArgs,
};
pub use session::{
    AvailableNetworkSession, AvailableNetworkSessionCollection, LocalNetworkGamer, NetworkGamer,
    NetworkGamerBase, NetworkMachine, NetworkSession, QualityOfService,
    ReadOnlyCollectionBase,
};
pub use values::{
    NetworkSessionEndReason, NetworkSessionJoinError, NetworkSessionJoinException,
    NetworkSessionProperties, NetworkSessionState, NetworkSessionType, PacketReader, PacketWriter,
    SendDataOptions,
};

pub(crate) use session::{
    available_session_handle, gamer_handle, local_gamer_handle, net_runtime, session_handle,
    write_properties,
};

/// The canonical bits behind a `SendDataOptions`, for the extension surface.
pub(crate) fn send_data_bits(options: SendDataOptions) -> u32 {
    options.bits()
}
