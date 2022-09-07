mod bindings;
mod game_controller_return_message;
mod game_controller_state_message;
mod spl_message;

use std::time::Duration;

use nalgebra::Point2;
use serde::{Deserialize, Serialize};

pub use game_controller_return_message::GameControllerReturnMessage;
pub use game_controller_state_message::{
    GameControllerStateMessage, GamePhase, GameState, Half, Penalty, PenaltyShoot, Player, SetPlay,
    Team, TeamColor, TeamState,
};
pub use spl_message::SplMessage;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct BallPosition {
    pub relative_position: Point2<f32>,
    pub age: Duration,
}

pub const HULKS_TEAM_NUMBER: u8 = 24;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlayerNumber {
    One,
    Two,
    Three,
    Four,
    Five,
}

impl Default for PlayerNumber {
    fn default() -> Self {
        PlayerNumber::Five
    }
}
