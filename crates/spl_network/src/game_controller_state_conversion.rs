use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::HULKS_TEAM_NUMBER;

use bifrost::{
    communication::{
        GamePhase as BifrostGamePhase, GameState as BifrostGameState, Half as BifrostHalf,
        Penalty as BifrostPenalty, RobotInfo, SetPlay as BifrostSetPlay,
        TeamColor as BifrostTeamColor,
    },
    serialization::{Decode, Encode},
};

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Half {
    First,
    Second,
}

impl Half {
    pub fn try_from(half: BifrostHalf) -> anyhow::Result<Self> {
        match half {
            BifrostHalf::First => Ok(Half::First),
            BifrostHalf::Second => Ok(Half::Second),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Initial,
    Ready,
    Set,
    Playing,
    Finished,
}

impl GameState {
    pub fn try_from(state: BifrostGameState) -> anyhow::Result<Self> {
        match state {
            BifrostGameState::Initial => Ok(GameState::Initial),
            BifrostGameState::Ready => Ok(GameState::Ready),
            BifrostGameState::Set => Ok(GameState::Set),
            BifrostGameState::Playing => Ok(GameState::Playing),
            BifrostGameState::Finished => Ok(GameState::Finished),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub enum GamePhase {
    #[default]
    Normal,
    PenaltyShootout {
        kicking_team: Team,
    },
    Overtime,
    Timeout,
}

impl GamePhase {
    pub fn try_from(game_phase: BifrostGamePhase, kicking_team: u8) -> anyhow::Result<Self> {
        let team = if kicking_team == HULKS_TEAM_NUMBER {
            Team::Hulks
        } else {
            Team::Opponent
        };
        match game_phase {
            BifrostGamePhase::Normal => Ok(GamePhase::Normal),
            BifrostGamePhase::PenaltyShoot => Ok(GamePhase::PenaltyShootout { kicking_team: team }),
            BifrostGamePhase::Overtime => Ok(GamePhase::Overtime),
            BifrostGamePhase::Timeout => Ok(GamePhase::Timeout),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetPlay {
    None,
    GoalKick,
    PushingFreeKick,
    CornerKick,
    KickIn,
    PenaltyKick,
}

impl SetPlay {
    pub fn try_from(set_play: BifrostSetPlay) -> anyhow::Result<Self> {
        match set_play {
            BifrostSetPlay::None => Ok(SetPlay::None),
            BifrostSetPlay::GoalKick => Ok(SetPlay::GoalKick),
            BifrostSetPlay::PushingFreeKick => Ok(SetPlay::PushingFreeKick),
            BifrostSetPlay::CornerKick => Ok(SetPlay::CornerKick),
            BifrostSetPlay::KickIn => Ok(SetPlay::KickIn),
            BifrostSetPlay::PenaltyKick => Ok(SetPlay::PenaltyKick),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Team {
    Hulks,
    Opponent,
    Uncertain,
}

impl Default for Team {
    fn default() -> Self {
        Team::Uncertain
    }
}

impl Team {
    pub fn try_from(team_number: u8) -> anyhow::Result<Self> {
        let team = if team_number == HULKS_TEAM_NUMBER {
            Team::Hulks
        } else {
            Team::Opponent
        };
        Ok(team)
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeamColor {
    Blue,
    Red,
    Yellow,
    Black,
    White,
    Green,
    Orange,
    Purple,
    Brown,
    Gray,
}

impl TeamColor {
    pub fn try_from(color: BifrostTeamColor) -> anyhow::Result<Self> {
        match color {
            BifrostTeamColor::Blue => Ok(TeamColor::Blue),
            BifrostTeamColor::Red => Ok(TeamColor::Red),
            BifrostTeamColor::Yellow => Ok(TeamColor::Yellow),
            BifrostTeamColor::Black => Ok(TeamColor::Black),
            BifrostTeamColor::White => Ok(TeamColor::White),
            BifrostTeamColor::Green => Ok(TeamColor::Green),
            BifrostTeamColor::Orange => Ok(TeamColor::Orange),
            BifrostTeamColor::Purple => Ok(TeamColor::Purple),
            BifrostTeamColor::Brown => Ok(TeamColor::Brown),
            BifrostTeamColor::Gray => Ok(TeamColor::Gray),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamState {
    pub team_number: u8,
    pub field_player_colour: TeamColor,
    pub goalkeeper_colour: TeamColor,
    pub score: u8,
    pub penalty_shoot_index: u8,
    pub penalty_shoots: Vec<PenaltyShoot>,
    pub remaining_amount_of_messages: u16,
    pub players: Vec<Player>,
}

#[derive(Encode, Decode, Clone, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum PenaltyShoot {
    Successful = 1,
    Unsuccessful = 0,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Player {
    pub penalty: Penalty,
}

impl TryFrom<RobotInfo> for Player {
    type Error = anyhow::Error;

    fn try_from(player: RobotInfo) -> anyhow::Result<Self> {
        let remaining = Duration::from_secs(player.secs_till_unpenalised as u64);
        Ok(Self {
            penalty: Penalty::try_from(remaining, player.penalty)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Penalty {
    None,
    IllegalBallContact { remaining: Duration },
    PlayerPushing { remaining: Duration },
    IllegalMotionInSet { remaining: Duration },
    InactivePlayer { remaining: Duration },
    IllegalPosition { remaining: Duration },
    LeavingTheField { remaining: Duration },
    RequestForPickup { remaining: Duration },
    LocalGameStuck { remaining: Duration },
    IllegalPositionInSet { remaining: Duration },
    PlayerStance { remaining: Duration },
    Substitute { remaining: Duration },
    Manual { remaining: Duration },
}

impl Penalty {
    pub fn try_from(remaining: Duration, penalty: BifrostPenalty) -> anyhow::Result<Self> {
        match penalty {
            BifrostPenalty::None => Ok(Penalty::None),
            BifrostPenalty::IllegalBallContact => Ok(Penalty::IllegalBallContact { remaining }),
            BifrostPenalty::PlayerPushing => Ok(Penalty::PlayerPushing { remaining }),
            BifrostPenalty::IllegalMotionInSet => Ok(Penalty::IllegalMotionInSet { remaining }),
            BifrostPenalty::InactivePlayer => Ok(Penalty::InactivePlayer { remaining }),
            BifrostPenalty::IllegalPosition => Ok(Penalty::IllegalPosition { remaining }),
            BifrostPenalty::LeavingTheField => Ok(Penalty::LeavingTheField { remaining }),
            BifrostPenalty::RequestForPickup => Ok(Penalty::RequestForPickup { remaining }),
            BifrostPenalty::LocalGameStuck => Ok(Penalty::LocalGameStuck { remaining }),
            BifrostPenalty::IllegalPositionInSet => Ok(Penalty::IllegalPositionInSet { remaining }),
            BifrostPenalty::PlayerStance => Ok(Penalty::PlayerStance { remaining }),
            BifrostPenalty::Substitute => Ok(Penalty::Substitute { remaining }),
            BifrostPenalty::Manual => Ok(Penalty::Manual { remaining }),
        }
    }

    pub fn is_some(&self) -> bool {
        !matches!(self, Penalty::None)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Penalty::None)
    }
}
