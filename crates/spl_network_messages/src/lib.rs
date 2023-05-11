mod bindings;
mod game_controller_return_message;
mod game_controller_state_conversion;
mod game_controller_state_message;

use std::{
    fmt::{self, Display, Formatter},
    time::Duration,
};

use nalgebra::Point2;
use serde::{Deserialize, Serialize};

pub use game_controller_return_message::GameControllerReturnMessage;
pub use game_controller_state_conversion::{
    CompetitionPhase, CompetitionType, GamePhase, GameState, Half, Penalty, PenaltyShoot, Player,
    SubState, Team, TeamColor, TeamState,
};
pub use game_controller_state_message::GameControllerStateMessage;
use serialize_hierarchy::SerializeHierarchy;

pub type HulkMessage = GameControllerReturnMessage;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, SerializeHierarchy)]
pub struct BallPosition {
    pub relative_position: Point2<f32>,
    pub age: Duration,
}

pub const DNT_TEAM_NUMBER: u8 = 8;

use bifrost::serialization::{Decode, Encode};

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Encode,
    Decode,
    Eq,
    Hash,
    PartialEq,
    Serialize,
    SerializeHierarchy,
)]
pub enum PlayerNumber {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    #[default]
    Seven,
}

impl Display for PlayerNumber {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let number = match self {
            PlayerNumber::One => "1",
            PlayerNumber::Two => "2",
            PlayerNumber::Three => "3",
            PlayerNumber::Four => "4",
            PlayerNumber::Five => "5",
            PlayerNumber::Six => "6",
            PlayerNumber::Seven => "7",
        };

        write!(formatter, "{number}")
    }
}
