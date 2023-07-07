mod bindings;
mod game_controller_return_message;
mod game_controller_state_conversion;
mod game_controller_state_message;
pub mod referee_return_message;

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
pub use referee_return_message::RefereeMessage;

pub type HulkMessage = GameControllerReturnMessage;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, SerializeHierarchy)]
pub struct BallPosition {
    pub relative_position: Point2<f32>,
    pub age: Duration,
}

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
    #[default]
    Five,
}

impl Display for PlayerNumber {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let number = match self {
            PlayerNumber::One => "1",
            PlayerNumber::Two => "2",
            PlayerNumber::Three => "3",
            PlayerNumber::Four => "4",
            PlayerNumber::Five => "5",
        };

        write!(formatter, "{number}")
    }
}
