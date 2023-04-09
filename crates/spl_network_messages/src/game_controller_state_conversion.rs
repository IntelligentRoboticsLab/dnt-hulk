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

impl From<BifrostHalf> for Half {
    fn from(half: BifrostHalf) -> Self {
        match half {
            BifrostHalf::First => Half::First,
            BifrostHalf::Second => Half::Second,
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

impl From<BifrostGameState> for GameState {
    fn from(state: BifrostGameState) -> Self {
        match state {
            BifrostGameState::Initial => GameState::Initial,
            BifrostGameState::Ready => GameState::Ready,
            BifrostGameState::Set => GameState::Set,
            BifrostGameState::Playing => GameState::Playing,
            BifrostGameState::Finished => GameState::Finished,
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

impl From<(BifrostGamePhase, u8)> for GamePhase {
    fn from((game_phase, kicking_team): (BifrostGamePhase, u8)) -> Self {
        let team = if kicking_team == HULKS_TEAM_NUMBER {
            Team::Hulks
        } else {
            Team::Opponent
        };
        match game_phase {
            BifrostGamePhase::Normal => GamePhase::Normal,
            BifrostGamePhase::PenaltyShoot => GamePhase::PenaltyShootout { kicking_team: team },
            BifrostGamePhase::Overtime => GamePhase::Overtime,
            BifrostGamePhase::Timeout => GamePhase::Timeout,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, SerializeHierarchy)]
pub enum SubState {
    None,
    GoalKick,
    PushingFreeKick,
    CornerKick,
    KickIn,
    PenaltyKick,
}

impl From<BifrostSubState> for SubState {
    fn from(set_play: BifrostSubState) -> Self {
        match set_play {
            BifrostSubState::None => SubState::None,
            BifrostSubState::GoalKick => SubState::GoalKick,
            BifrostSubState::PushingFreeKick => SubState::PushingFreeKick,
            BifrostSubState::CornerKick => SubState::CornerKick,
            BifrostSubState::KickIn => SubState::KickIn,
            BifrostSubState::PenaltyKick => SubState::PenaltyKick,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Team {
    Hulks,
    Opponent,
    #[default]
    Uncertain,
}

impl From<u8> for Team {
    fn from(team_number: u8) -> Self {
        if team_number == HULKS_TEAM_NUMBER {
            Team::Hulks
        } else {
            Team::Opponent
        }
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

impl From<BifrostTeamColor> for TeamColor {
    fn from(color: BifrostTeamColor) -> Self {
        match color {
            BifrostTeamColor::Blue => TeamColor::Blue,
            BifrostTeamColor::Red => TeamColor::Red,
            BifrostTeamColor::Yellow => TeamColor::Yellow,
            BifrostTeamColor::Black => TeamColor::Black,
            BifrostTeamColor::White => TeamColor::White,
            BifrostTeamColor::Green => TeamColor::Green,
            BifrostTeamColor::Orange => TeamColor::Orange,
            BifrostTeamColor::Purple => TeamColor::Purple,
            BifrostTeamColor::Brown => TeamColor::Brown,
            BifrostTeamColor::Gray => TeamColor::Gray,
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
            penalty: Penalty::from((remaining, player.penalty)),
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

impl From<(Duration, BifrostPenalty)> for Penalty {
    fn from((remaining, penalty): (Duration, BifrostPenalty)) -> Self {
        match penalty {
            BifrostPenalty::None => Penalty::None,
            BifrostPenalty::IllegalBallContact => Penalty::IllegalBallContact { remaining },
            BifrostPenalty::PlayerPushing => Penalty::PlayerPushing { remaining },
            BifrostPenalty::IllegalMotionInSet => Penalty::IllegalMotionInSet { remaining },
            BifrostPenalty::InactivePlayer => Penalty::InactivePlayer { remaining },
            BifrostPenalty::IllegalPosition => Penalty::IllegalPosition { remaining },
            BifrostPenalty::LeavingTheField => Penalty::LeavingTheField { remaining },
            BifrostPenalty::RequestForPickup => Penalty::RequestForPickup { remaining },
            BifrostPenalty::LocalGameStuck => Penalty::LocalGameStuck { remaining },
            BifrostPenalty::IllegalPositionInSet => Penalty::IllegalPositionInSet { remaining },
            BifrostPenalty::PlayerStance => Penalty::PlayerStance { remaining },
            BifrostPenalty::Substitute => Penalty::Substitute { remaining },
            BifrostPenalty::Manual => Penalty::Manual { remaining },
        }
    }
}

impl Penalty {
    pub fn is_some(&self) -> bool {
        !matches!(self, Penalty::None)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Penalty::None)
    }
}
