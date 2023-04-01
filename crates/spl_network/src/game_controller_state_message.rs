use std::{
    convert::{TryFrom, TryInto},
    time::Duration,
};

use anyhow::bail;
use serde::{Deserialize, Serialize};

use crate::HULKS_TEAM_NUMBER;
use bifrost::{
    communication::game_controller_message::{
        GamePhase as BifrostGamePhase, GameState as BifrostGameState, Half as BifrostHalf,
        Penalty as BifrostPenalty, RoboCupGameControlData, RobotInfo, SetPlay as BifrostSetPlay,
        TeamColor, GAMECONTROLLER_STRUCT_HEADER, GAMECONTROLLER_STRUCT_VERSION, MAX_NUM_PLAYERS,
    },
    serialization::{Decode, Encode},
};

pub type GameState = BifrostGameState;
pub type Half = BifrostHalf;
pub type SetPlay = BifrostSetPlay;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameControllerStateMessage {
    pub game_phase: GamePhase,
    pub game_state: GameState,
    pub set_play: SetPlay,
    pub half: Half,
    pub remaining_time_in_half: Duration,
    pub secondary_time: Duration,
    pub hulks_team: TeamState,
    pub opponent_team: TeamState,
    pub kicking_team: Team,
}

impl TryFrom<&[u8]> for GameControllerStateMessage {
    type Error = anyhow::Error;

    fn try_from(mut buffer: &[u8]) -> anyhow::Result<Self> {
        RoboCupGameControlData::decode(&mut buffer)?.try_into()
    }
}

impl TryFrom<RoboCupGameControlData> for GameControllerStateMessage {
    type Error = anyhow::Error;

    fn try_from(message: RoboCupGameControlData) -> anyhow::Result<Self> {
        if message.header != GAMECONTROLLER_STRUCT_HEADER {
            bail!("Unexpected header");
        }

        if message.version != GAMECONTROLLER_STRUCT_VERSION {
            bail!("Unexpected version");
        }

        let (hulks_team_index, opponent_team_index) =
            match (message.teams[0].team_number, message.teams[1].team_number) {
                (HULKS_TEAM_NUMBER, _) => (0, 1),
                (_, HULKS_TEAM_NUMBER) => (1, 0),
                _ => bail!("Failed to find HULKs team"),
            };

        const MAXIMUM_NUMBER_OF_PENALTY_SHOOTS: u8 = 16;
        if message.teams[hulks_team_index].penalty_shot >= MAXIMUM_NUMBER_OF_PENALTY_SHOOTS {
            bail!("Unexpected penalty shoot index for team HULKs");
        }
        if message.teams[opponent_team_index].penalty_shot >= MAXIMUM_NUMBER_OF_PENALTY_SHOOTS {
            bail!("Unexpected penalty shoot index for opponent team");
        }

        let hulks_penalty_shoots: Vec<PenaltyShoot> = (0..message.teams[hulks_team_index]
            .penalty_shot)
            .map(|shoot_index| {
                // Get the bit corresponding to the shoot index, 1: successful, 0: unsuccessful
                let shoot = message.teams[hulks_team_index].single_shots & (1 << shoot_index);

                PenaltyShoot::decode(&mut &shoot.to_le_bytes()[..]).unwrap()
            })
            .collect();
        let opponent_penalty_shoots: Vec<PenaltyShoot> = (0..message.teams[opponent_team_index]
            .penalty_shot)
            .map(|shoot_index| {
                let shoot = message.teams[opponent_team_index].single_shots & (1 << shoot_index);

                PenaltyShoot::decode(&mut &shoot.to_le_bytes()[..]).unwrap()
            })
            .collect();

        if message.players_per_team >= MAX_NUM_PLAYERS {
            bail!("Unexpected number of players per team");
        }

        let hulks_players: Vec<Player> = (0..message.players_per_team)
            .map(|player_index| {
                message.teams[hulks_team_index].players[player_index as usize].try_into()
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        // let hps = Vec<Player> =

        let opponent_players: Vec<Player> = (0..message.players_per_team)
            .map(|player_index| {
                message.teams[opponent_team_index].players[player_index as usize].try_into()
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(GameControllerStateMessage {
            game_phase: GamePhase::try_from(message.game_phase, message.kicking_team)?,
            game_state: message.state,
            set_play: message.set_play,
            half: message.first_half,
            remaining_time_in_half: Duration::from_secs(message.secs_remaining.max(0).try_into()?),
            secondary_time: Duration::from_secs(message.secondary_time.max(0).try_into()?),
            hulks_team: TeamState {
                team_number: message.teams[hulks_team_index].team_number,
                field_player_colour: message.teams[hulks_team_index].field_player_colour,
                goalkeeper_colour: message.teams[hulks_team_index].goalkeeper_colour,
                score: message.teams[hulks_team_index].score,
                penalty_shoot_index: message.teams[hulks_team_index].penalty_shot,
                penalty_shoots: hulks_penalty_shoots,
                remaining_amount_of_messages: message.teams[hulks_team_index].message_budget,
                players: hulks_players,
            },
            // hulks_team: message.teams[hulks_team_index],
            // opponent_team: message.teams[opponent_team_index],
            opponent_team: TeamState {
                team_number: message.teams[opponent_team_index].team_number,
                field_player_colour: message.teams[opponent_team_index].field_player_colour,
                // .try_into()?,
                goalkeeper_colour: message.teams[opponent_team_index].goalkeeper_colour,
                // .try_into()?,
                score: message.teams[opponent_team_index].score,
                penalty_shoot_index: message.teams[opponent_team_index].penalty_shot,
                penalty_shoots: opponent_penalty_shoots,
                remaining_amount_of_messages: message.teams[opponent_team_index].message_budget,
                players: opponent_players,
            },
            kicking_team: Team::try_from(message.kicking_team)?,
        })
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
    fn try_from(game_phase: BifrostGamePhase, kicking_team: u8) -> anyhow::Result<Self> {
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

// #[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
// pub enum GameState {
//     Initial,
//     Ready,
//     Set,
//     Playing,
//     Finished,
// }

// impl GameState {
//     fn try_from(game_state: u8) -> anyhow::Result<Self> {
//         match game_state {
//             STATE_INITIAL => Ok(GameState::Initial),
//             STATE_READY => Ok(GameState::Ready),
//             STATE_SET => Ok(GameState::Set),
//             STATE_PLAYING => Ok(GameState::Playing),
//             STATE_FINISHED => Ok(GameState::Finished),
//             _ => bail!("Unexpected game state"),
//         }
//     }
// }

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
    fn try_from(team_number: u8) -> anyhow::Result<Self> {
        let team = if team_number == HULKS_TEAM_NUMBER {
            Team::Hulks
        } else {
            Team::Opponent
        };
        Ok(team)
    }
}

// impl SetPlay {
//     fn try_from(set_play: u8) -> anyhow::Result<Option<Self>> {
//         match set_play {
//             SET_PLAY_NONE => Ok(None),
//             SET_PLAY_GOAL_KICK => Ok(Some(SetPlay::GoalKick)),
//             SET_PLAY_PUSHING_FREE_KICK => Ok(Some(SetPlay::PushingFreeKick)),
//             SET_PLAY_CORNER_KICK => Ok(Some(SetPlay::CornerKick)),
//             SET_PLAY_KICK_IN => Ok(Some(SetPlay::KickIn)),
//             SET_PLAY_PENALTY_KICK => Ok(Some(SetPlay::PenaltyKick)),
//             _ => bail!("Unexpected set play"),
//         }
//     }
// }

// #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
// pub enum SetPlay {
//     GoalKick,
//     PushingFreeKick,
//     CornerKick,
//     KickIn,
//     PenaltyKick,
// }

// impl Default for SetPlay {
//     fn default() -> Self {
//         SetPlay::GoalKick
//     }
// }

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub enum Half {
//     First,
//     Second,
// }

// impl TryFrom<u8> for Half {
//     type Error = anyhow::Error;

//     fn try_from(half: u8) -> anyhow::Result<Self> {
//         match half {
//             1 => Ok(Half::First),
//             0 => Ok(Half::Second),
//             _ => bail!("Unexpected half"),
//         }
//     }
// }

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
    Substitute { remaining: Duration },
    Manual { remaining: Duration },
}

impl Penalty {
    fn try_from(remaining: Duration, penalty: BifrostPenalty) -> anyhow::Result<Self> {
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
            BifrostPenalty::Substitute => Ok(Penalty::Substitute { remaining }),
            BifrostPenalty::Manual => Ok(Penalty::Manual { remaining }),
            _ => bail!("Unexpected penalty type"),
        }
    }

    pub fn is_some(&self) -> bool {
        !matches!(self, Penalty::None)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Penalty::None)
    }
}
