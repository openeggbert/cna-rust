//! XNA's network session event payloads.
//!
//! Every one is a managed CLR `EventArgs`: the gamer it names is the session's
//! own object, and the projection retains the facade rather than copying its
//! state, so a handler reading `Gamer` sees the same object the session's
//! rosters answer.

#![allow(non_snake_case)]

use super::session::NetworkGamer;
use super::values::NetworkSessionEndReason;

/// XNA `Microsoft.Xna.Framework.Net.GameStartedEventArgs`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameStartedEventArgs;

impl GameStartedEventArgs {
    /// XNA `GameStartedEventArgs()`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// XNA `Microsoft.Xna.Framework.Net.GameEndedEventArgs`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameEndedEventArgs;

impl GameEndedEventArgs {
    /// XNA `GameEndedEventArgs()`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// XNA `Microsoft.Xna.Framework.Net.GamerJoinedEventArgs`.
#[derive(Clone, Debug)]
pub struct GamerJoinedEventArgs {
    gamer: NetworkGamer,
}

impl GamerJoinedEventArgs {
    /// XNA `GamerJoinedEventArgs(NetworkGamer)`.
    #[must_use]
    pub fn new(gamer: &NetworkGamer) -> Self {
        Self {
            gamer: gamer.clone(),
        }
    }

    /// XNA `GamerJoinedEventArgs.Gamer`.
    #[must_use]
    pub const fn Gamer(&self) -> &NetworkGamer {
        &self.gamer
    }
}

/// XNA `Microsoft.Xna.Framework.Net.GamerLeftEventArgs`.
#[derive(Clone, Debug)]
pub struct GamerLeftEventArgs {
    gamer: NetworkGamer,
}

impl GamerLeftEventArgs {
    /// XNA `GamerLeftEventArgs(NetworkGamer)`.
    #[must_use]
    pub fn new(gamer: &NetworkGamer) -> Self {
        Self {
            gamer: gamer.clone(),
        }
    }

    /// XNA `GamerLeftEventArgs.Gamer`.
    #[must_use]
    pub const fn Gamer(&self) -> &NetworkGamer {
        &self.gamer
    }
}

/// XNA `Microsoft.Xna.Framework.Net.HostChangedEventArgs`.
#[derive(Clone, Debug)]
pub struct HostChangedEventArgs {
    old_host: NetworkGamer,
    new_host: NetworkGamer,
}

impl HostChangedEventArgs {
    /// XNA `HostChangedEventArgs(NetworkGamer, NetworkGamer)`.
    #[must_use]
    pub fn new(oldHost: &NetworkGamer, newHost: &NetworkGamer) -> Self {
        Self {
            old_host: oldHost.clone(),
            new_host: newHost.clone(),
        }
    }

    /// XNA `HostChangedEventArgs.OldHost`.
    #[must_use]
    pub const fn OldHost(&self) -> &NetworkGamer {
        &self.old_host
    }

    /// XNA `HostChangedEventArgs.NewHost`.
    #[must_use]
    pub const fn NewHost(&self) -> &NetworkGamer {
        &self.new_host
    }
}

/// XNA `Microsoft.Xna.Framework.Net.NetworkSessionEndedEventArgs`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkSessionEndedEventArgs {
    end_reason: NetworkSessionEndReason,
}

impl NetworkSessionEndedEventArgs {
    /// XNA `NetworkSessionEndedEventArgs(NetworkSessionEndReason)`.
    #[must_use]
    pub const fn new(endReason: NetworkSessionEndReason) -> Self {
        Self {
            end_reason: endReason,
        }
    }

    /// XNA `NetworkSessionEndedEventArgs.EndReason`.
    #[must_use]
    pub const fn EndReason(&self) -> NetworkSessionEndReason {
        self.end_reason
    }
}

/// XNA `Microsoft.Xna.Framework.Net.WriteLeaderboardsEventArgs`.
///
/// XNA declares no public constructor: only a session raises this event.
#[derive(Clone, Debug)]
pub struct WriteLeaderboardsEventArgs {
    gamer: NetworkGamer,
    is_leaving: bool,
}

impl WriteLeaderboardsEventArgs {
    pub(crate) fn from_parts(gamer: NetworkGamer, isLeaving: bool) -> Self {
        Self {
            gamer,
            is_leaving: isLeaving,
        }
    }

    /// XNA `WriteLeaderboardsEventArgs.Gamer`.
    #[must_use]
    pub const fn Gamer(&self) -> &NetworkGamer {
        &self.gamer
    }

    /// XNA `WriteLeaderboardsEventArgs.IsLeaving`.
    #[must_use]
    pub const fn IsLeaving(&self) -> bool {
        self.is_leaving
    }
}
