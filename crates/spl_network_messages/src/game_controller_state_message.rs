use std::{
    convert::{TryFrom, TryInto},
    time::Duration,
};

use color_eyre::{eyre::bail, Report, Result};
use serde::{Deserialize, Serialize};
use serialize_hierarchy::SerializeHierarchy;

use crate::{
    GamePhase, GameState, Half, PenaltyShoot, Player, SubState, Team, TeamColor, TeamState,
    HULKS_TEAM_NUMBER,
};
use bifrost::{communication::RoboCupGameControlData, serialization::Decode};

// Internal representation of the game controller state,
// with compacted data from the RoboCupGameControlData struct.
#[derive(Clone, Debug, Deserialize, Serialize, SerializeHierarchy)]
pub struct GameControllerStateMessage {
    pub game_phase: GamePhase,
    pub game_state: GameState,
    pub sub_state: SubState,
    pub half: Half,
    pub remaining_time_in_half: Duration,
    pub secondary_time: Duration,
    pub hulks_team: TeamState,
    pub opponent_team: TeamState,
    pub kicking_team: Team,
}

impl TryFrom<&[u8]> for GameControllerStateMessage {
    type Error = Report;

    fn try_from(mut buffer: &[u8]) -> Result<Self> {
        let message = RoboCupGameControlData::decode(&mut buffer)?;

        if !message.is_valid() {
            bail!("GameControllerStateMessage is not valid");
        }

        message.try_into()
    }
}

impl TryFrom<RoboCupGameControlData> for GameControllerStateMessage {
    type Error = Report;

    fn try_from(message: RoboCupGameControlData) -> Result<Self> {
        let (hulks_team_index, opponent_team_index) =
            match (message.teams[0].team_number, message.teams[1].team_number) {
                (HULKS_TEAM_NUMBER, _) => (0, 1),
                (_, HULKS_TEAM_NUMBER) => (1, 0),
                _ => bail!("failed to find HULKs team"),
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

        let hulks_players: Vec<Player> = (0..message.players_per_team)
            .map(|player_index| {
                message.teams[hulks_team_index].players[player_index as usize].try_into()
            })
            .collect::<Result<Vec<_>>>()?;

        let opponent_players: Vec<Player> = (0..message.players_per_team)
            .map(|player_index| {
                message.teams[opponent_team_index].players[player_index as usize].try_into()
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(GameControllerStateMessage {
            game_phase: GamePhase::from((message.game_phase, message.kicking_team)),
            game_state: GameState::from(message.state),
            sub_state: SubState::try_from(message.set_play)?,
            half: Half::from(message.first_half),
            remaining_time_in_half: Duration::from_secs(message.secs_remaining.max(0).try_into()?),
            secondary_time: Duration::from_secs(message.secondary_time.max(0).try_into()?),
            hulks_team: TeamState {
                team_number: message.teams[hulks_team_index].team_number,
                field_player_colour: TeamColor::from(
                    message.teams[hulks_team_index].field_player_colour,
                ),
                goalkeeper_colour: TeamColor::from(
                    message.teams[hulks_team_index].goalkeeper_colour,
                ),
                score: message.teams[hulks_team_index].score,
                penalty_shoot_index: message.teams[hulks_team_index].penalty_shot,
                penalty_shoots: hulks_penalty_shoots,
                remaining_amount_of_messages: message.teams[hulks_team_index].message_budget,
                players: hulks_players,
            },
            opponent_team: TeamState {
                team_number: message.teams[opponent_team_index].team_number,
                field_player_colour: TeamColor::from(
                    message.teams[opponent_team_index].field_player_colour,
                ),
                goalkeeper_colour: TeamColor::from(
                    message.teams[opponent_team_index].goalkeeper_colour,
                ),
                score: message.teams[opponent_team_index].score,
                penalty_shoot_index: message.teams[opponent_team_index].penalty_shot,
                penalty_shoots: opponent_penalty_shoots,
                remaining_amount_of_messages: message.teams[opponent_team_index].message_budget,
                players: opponent_players,
            },
            kicking_team: Team::from(message.kicking_team),
        })
    }
}
