//! Microsoft XNA 4.0 `GamerServices` and `Avatar` value identities.
//!
//! These belong to the wider Windows runtime profile
//! (`tools/api-compat/profiles/xna40-windows-full.json`), not the selected
//! seven-assembly profile. Everything here is a value type or an exception
//! identity: exact managed Rust with no native backing, because CLR metadata
//! is the whole of their contract.

#![allow(non_upper_case_globals, non_snake_case)]

use core::fmt;
use std::error::Error;

use crate::content::{SerializationInfo, StreamingContext};

/// XNA `Microsoft.Xna.Framework.GamerServices.AvatarAnimationPreset`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AvatarAnimationPreset {
    Stand0 = 0,
    Stand1 = 1,
    Stand2 = 2,
    Stand3 = 3,
    Stand4 = 4,
    Stand5 = 5,
    Stand6 = 6,
    Stand7 = 7,
    Clap = 8,
    Wave = 9,
    Celebrate = 10,
    FemaleIdleCheckNails = 11,
    FemaleIdleLookAround = 12,
    FemaleIdleShiftWeight = 13,
    FemaleIdleFixShoe = 14,
    FemaleAngry = 15,
    FemaleConfused = 16,
    FemaleLaugh = 17,
    FemaleCry = 18,
    FemaleShocked = 19,
    FemaleYawn = 20,
    MaleIdleLookAround = 21,
    MaleIdleStretch = 22,
    MaleIdleShiftWeight = 23,
    MaleIdleCheckHand = 24,
    MaleAngry = 25,
    MaleConfused = 26,
    MaleLaugh = 27,
    MaleCry = 28,
    MaleSurprised = 29,
    MaleYawn = 30,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.AvatarBodyType`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AvatarBodyType {
    Female = 0,
    Male = 1,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.AvatarBone`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AvatarBone {
    Root = 0,
    BackLower = 1,
    HipLeft = 2,
    HipRight = 3,
    BackUpper = 5,
    KneeLeft = 6,
    KneeRight = 8,
    AnkleLeft = 11,
    CollarLeft = 12,
    Neck = 14,
    AnkleRight = 15,
    CollarRight = 16,
    Head = 19,
    ShoulderLeft = 20,
    ToeLeft = 21,
    ShoulderRight = 22,
    ToeRight = 23,
    ElbowLeft = 25,
    ElbowRight = 28,
    WristLeft = 33,
    WristRight = 36,
    FingerIndexLeft = 37,
    FingerMiddleLeft = 38,
    FingerRingLeft = 39,
    FingerSmallLeft = 40,
    PropLeft = 41,
    SpecialLeft = 42,
    FingerThumbLeft = 43,
    FingerIndexRight = 44,
    FingerMiddleRight = 45,
    FingerRingRight = 46,
    FingerSmallRight = 47,
    PropRight = 48,
    SpecialRight = 49,
    FingerThumbRight = 50,
    FingerIndex2Left = 51,
    FingerMiddle2Left = 52,
    FingerRing2Left = 53,
    FingerSmall2Left = 54,
    FingerThumb2Left = 55,
    FingerIndex2Right = 56,
    FingerMiddle2Right = 57,
    FingerRing2Right = 58,
    FingerSmall2Right = 59,
    FingerThumb2Right = 60,
    FingerIndex3Left = 61,
    FingerMiddle3Left = 62,
    FingerRing3Left = 63,
    FingerSmall3Left = 64,
    FingerThumb3Left = 65,
    FingerIndex3Right = 66,
    FingerMiddle3Right = 67,
    FingerRing3Right = 68,
    FingerSmall3Right = 69,
    FingerThumb3Right = 70,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.AvatarEye`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AvatarEye {
    Neutral = 0,
    Sad = 1,
    Angry = 2,
    Confused = 3,
    Laughing = 4,
    Shocked = 5,
    Happy = 6,
    Yawning = 7,
    Sleeping = 8,
    LookUp = 9,
    LookDown = 10,
    LookLeft = 11,
    LookRight = 12,
    Blink = 13,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.AvatarEyebrow`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AvatarEyebrow {
    Neutral = 0,
    Sad = 1,
    Angry = 2,
    Confused = 3,
    Raised = 4,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.AvatarMouth`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AvatarMouth {
    Neutral = 0,
    Sad = 1,
    Angry = 2,
    Confused = 3,
    Laughing = 4,
    Shocked = 5,
    Happy = 6,
    PhoneticO = 7,
    PhoneticAi = 8,
    PhoneticEe = 9,
    PhoneticFv = 10,
    PhoneticW = 11,
    PhoneticL = 12,
    PhoneticDth = 13,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.AvatarRendererState`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AvatarRendererState {
    Loading = 0,
    Ready = 1,
    Unavailable = 2,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.ControllerSensitivity`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ControllerSensitivity {
    Low = 0,
    Medium = 1,
    High = 2,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.GameDifficulty`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GameDifficulty {
    Easy = 0,
    Normal = 1,
    Hard = 2,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.GamerPresenceMode`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GamerPresenceMode {
    None = 0,
    SinglePlayer = 1,
    Multiplayer = 2,
    LocalCoOp = 3,
    LocalVersus = 4,
    OnlineCoOp = 5,
    OnlineVersus = 6,
    VersusComputer = 7,
    Stage = 8,
    Level = 9,
    CoOpStage = 10,
    CoOpLevel = 11,
    ArcadeMode = 12,
    CampaignMode = 13,
    ChallengeMode = 14,
    ExplorationMode = 15,
    PracticeMode = 16,
    PuzzleMode = 17,
    ScenarioMode = 18,
    StoryMode = 19,
    SurvivalMode = 20,
    TutorialMode = 21,
    DifficultyEasy = 22,
    DifficultyMedium = 23,
    DifficultyHard = 24,
    DifficultyExtreme = 25,
    Score = 26,
    VersusScore = 27,
    Winning = 28,
    Losing = 29,
    ScoreIsTied = 30,
    Outnumbered = 31,
    OnARoll = 32,
    InCombat = 33,
    BattlingBoss = 34,
    TimeAttack = 35,
    TryingForRecord = 36,
    FreePlay = 37,
    WastingTime = 38,
    StuckOnAHardBit = 39,
    NearlyFinished = 40,
    LookingForGames = 41,
    WaitingForPlayers = 42,
    WaitingInLobby = 43,
    SettingUpMatch = 44,
    PlayingWithFriends = 45,
    AtMenu = 46,
    StartingGame = 47,
    Paused = 48,
    GameOver = 49,
    WonTheGame = 50,
    ConfiguringSettings = 51,
    CustomizingPlayer = 52,
    EditingLevel = 53,
    InGameStore = 54,
    WatchingCutscene = 55,
    WatchingCredits = 56,
    PlayingMinigame = 57,
    FoundSecret = 58,
    CornflowerBlue = 59,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.GamerPrivilegeSetting`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GamerPrivilegeSetting {
    Blocked = 0,
    FriendsOnly = 1,
    Everyone = 2,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.GamerZone`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GamerZone {
    Unknown = 0,
    Recreation = 1,
    Pro = 2,
    Family = 3,
    Underground = 4,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.LeaderboardKey`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LeaderboardKey {
    BestScoreLifeTime = 0,
    BestScoreRecent = 1,
    BestTimeLifeTime = 2,
    BestTimeRecent = 3,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.LeaderboardOutcome`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LeaderboardOutcome {
    None = 0,
    Win = 1,
    Loss = 2,
    Tie = 3,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.MessageBoxIcon`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MessageBoxIcon {
    None = 0,
    Error = 1,
    Warning = 2,
    Alert = 3,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.NotificationPosition`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NotificationPosition {
    TopLeft = 0,
    TopCenter = 1,
    TopRight = 2,
    CenterLeft = 3,
    Center = 4,
    CenterRight = 5,
    BottomLeft = 6,
    BottomCenter = 7,
    BottomRight = 8,
}

/// XNA `Microsoft.Xna.Framework.GamerServices.RacingCameraAngle`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RacingCameraAngle {
    Back = 0,
    Front = 1,
    Inside = 2,
}

/// Marker for the composed CLR base-class relationship of the two exceptions
/// that derive from `NetworkException`.
///
/// Rust has no class inheritance, so the projection composes the base's state
/// and states the relationship through this contract trait, exactly as the
/// graphics and component families already do.
pub trait NetworkExceptionBase {
    /// CLR `Exception.Message` of this exception.
    fn Message(&self) -> String;
}

macro_rules! xna_exception {
    ($name:ident, $default:literal $(, $base:ident)?) => {
        #[doc = concat!("XNA `", stringify!($name), "`.")]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name {
            message: String,
            inner_message: Option<String>,
            streaming_context: Option<i32>,
        }

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self {
                    message: $default.to_owned(),
                    inner_message: None,
                    streaming_context: None,
                }
            }

            #[must_use]
            pub fn from_message(message: &str) -> Self {
                Self {
                    message: message.to_owned(),
                    inner_message: None,
                    streaming_context: None,
                }
            }

            #[must_use]
            pub fn from_message_and_inner_exception(
                message: &str,
                innerException: &dyn Error,
            ) -> Self {
                Self {
                    message: message.to_owned(),
                    inner_message: Some(innerException.to_string()),
                    streaming_context: None,
                }
            }

            #[must_use]
            pub fn from_info_and_context(
                info: SerializationInfo,
                context: StreamingContext,
            ) -> Self {
                Self {
                    message: info.message().to_owned(),
                    inner_message: None,
                    streaming_context: Some(context.state()),
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.message)?;
                if let Some(inner) = &self.inner_message {
                    write!(formatter, ": {inner}")?;
                }
                Ok(())
            }
        }

        impl Error for $name {}

        $(
            impl $base for $name {
                fn Message(&self) -> String {
                    self.message.clone()
                }
            }
        )?
    };
}

pub(crate) use xna_exception;

xna_exception!(
    GameUpdateRequiredException,
    "A required title update is not installed."
);
xna_exception!(
    GamerPrivilegeException,
    "The gamer does not have the required privilege."
);
xna_exception!(
    GamerServicesNotAvailableException,
    "Gamer services are not available."
);
xna_exception!(GuideAlreadyVisibleException, "The Guide is already visible.");
xna_exception!(NetworkException, "A network error occurred.", NetworkExceptionBase);
xna_exception!(
    NetworkNotAvailableException,
    "The network is not available.",
    NetworkExceptionBase
);
