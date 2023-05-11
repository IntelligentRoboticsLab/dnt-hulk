use std::time::Duration;

use color_eyre::{Report, Result};
use nalgebra::{point, vector, Isometry2};
use serde::{Deserialize, Serialize};

use crate::{BallPosition, PlayerNumber, DNT_TEAM_NUMBER};

use bifrost::{
    communication::RoboCupGameControlReturnData,
    serialization::{Decode, Encode},
};

// Internal representation of the game controller return message,
// with compacted data from the GameControllerReturnMessage.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct GameControllerReturnMessage {
    pub player_number: PlayerNumber,
    pub fallen: bool,
    pub robot_to_field: Isometry2<f32>,
    pub ball_position: Option<BallPosition>,
}

impl TryFrom<GameControllerReturnMessage> for Vec<u8> {
    type Error = Report;

    fn try_from(message: GameControllerReturnMessage) -> Result<Self> {
        let message: RoboCupGameControlReturnData = message.into();
        let mut buffer = Vec::new();

        message.encode(&mut buffer)?;
        Ok(buffer)
    }
}

impl TryFrom<&[u8]> for GameControllerReturnMessage {
    type Error = Report;

    fn try_from(mut buffer: &[u8]) -> Result<Self> {
        let message = RoboCupGameControlReturnData::decode(&mut buffer)?;
        message.try_into()
    }
}

impl TryFrom<RoboCupGameControlReturnData> for GameControllerReturnMessage {
    type Error = Report;

    fn try_from(message: RoboCupGameControlReturnData) -> Result<Self> {
        Ok(Self {
            player_number: match message.player_num {
                1 => PlayerNumber::One,
                2 => PlayerNumber::Two,
                3 => PlayerNumber::Three,
                4 => PlayerNumber::Four,
                5 => PlayerNumber::Five,
                6 => PlayerNumber::Six,
                7 => PlayerNumber::Seven,
                player_num => panic!("Invalid player number {player_num}"),
            },
            fallen: match message.fallen {
                0 => false,
                1 => true,
                fallen => panic!("Invalid fallen value {fallen}"),
            },
            robot_to_field: Isometry2::new(
                vector![message.pose[0] / 1000.0, message.pose[1] / 1000.0],
                message.pose[2],
            ),
            ball_position: if message.ball_age == -1.0 {
                None
            } else {
                Some(BallPosition {
                    relative_position: point![message.ball[0] / 1000.0, message.ball[1] / 1000.0],
                    age: Duration::from_secs_f32(message.ball_age),
                })
            },
        })
    }
}

impl From<GameControllerReturnMessage> for RoboCupGameControlReturnData {
    fn from(message: GameControllerReturnMessage) -> Self {
        let (ball_position, ball_age) = match &message.ball_position {
            Some(ball_position) => (
                [
                    ball_position.relative_position.x * 1000.0,
                    ball_position.relative_position.y * 1000.0,
                ],
                ball_position.age.as_secs_f32(),
            ),
            None => ([0.0; 2], -1.0),
        };
        RoboCupGameControlReturnData::new(
            match message.player_number {
                PlayerNumber::One => 1,
                PlayerNumber::Two => 2,
                PlayerNumber::Three => 3,
                PlayerNumber::Four => 4,
                PlayerNumber::Five => 5,
                PlayerNumber::Six => 6,
                PlayerNumber::Seven => 7,
            },
            DNT_TEAM_NUMBER,
            u8::from(message.fallen),
            [
                message.robot_to_field.translation.vector.x * 1000.0,
                message.robot_to_field.translation.vector.y * 1000.0,
                message.robot_to_field.rotation.angle(),
            ],
            ball_age,
            ball_position,
        )
    }
}

#[cfg(test)]
mod test {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, SQRT_2};

    use approx::assert_relative_eq;
    use nalgebra::{point, vector};

    use super::*;

    #[test]
    fn zero_isometry() {
        let input_message = GameControllerReturnMessage {
            player_number: PlayerNumber::One,
            fallen: false,
            robot_to_field: Isometry2::default(),
            ball_position: None,
        };
        let output_message: RoboCupGameControlReturnData = input_message.into();

        assert_relative_eq!(output_message.pose[0], 0.0);
        assert_relative_eq!(output_message.pose[1], 0.0);
        assert_relative_eq!(output_message.pose[2], 0.0);

        let input_message_again: GameControllerReturnMessage = output_message.try_into().unwrap();

        assert_relative_eq!(input_message_again.robot_to_field, Isometry2::default());
    }

    #[test]
    fn one_to_the_left_isometry() {
        let input_message = GameControllerReturnMessage {
            player_number: PlayerNumber::One,
            fallen: false,
            robot_to_field: Isometry2::new(vector![0.0, 1.0], FRAC_PI_2),
            ball_position: None,
        };
        let output_message: RoboCupGameControlReturnData = input_message.into();

        assert_relative_eq!(output_message.pose[0], 0.0, epsilon = 0.001);
        assert_relative_eq!(output_message.pose[1], 1000.0, epsilon = 0.001);
        assert_relative_eq!(output_message.pose[2], FRAC_PI_2, epsilon = 0.001);

        let input_message_again: GameControllerReturnMessage = output_message.try_into().unwrap();

        assert_relative_eq!(
            input_message_again.robot_to_field,
            Isometry2::new(vector![0.0, 1.0], FRAC_PI_2),
            epsilon = 0.001
        );
    }

    #[test]
    fn one_schräg_to_the_top_right_isometry() {
        let input_message = GameControllerReturnMessage {
            player_number: PlayerNumber::One,
            fallen: false,
            robot_to_field: Isometry2::new(vector![1.0, 1.0], FRAC_PI_4),
            ball_position: None,
        };
        let output_message: RoboCupGameControlReturnData = input_message.into();

        assert_relative_eq!(
            input_message.robot_to_field * point![1.0 / SQRT_2, -1.0 / SQRT_2],
            point![2.0, 1.0],
            epsilon = 0.001
        );
        assert_relative_eq!(
            input_message.robot_to_field * point![0.0, 0.0],
            point![1.0, 1.0],
            epsilon = 0.001
        );

        assert_relative_eq!(output_message.pose[0], 1000.0, epsilon = 0.001);
        assert_relative_eq!(output_message.pose[1], 1000.0, epsilon = 0.001);
        assert_relative_eq!(output_message.pose[2], FRAC_PI_4, epsilon = 0.001);

        let input_message_again: GameControllerReturnMessage = output_message.try_into().unwrap();

        assert_relative_eq!(
            input_message_again.robot_to_field,
            Isometry2::new(vector![1.0, 1.0], FRAC_PI_4),
            epsilon = 0.001
        );
    }
}
