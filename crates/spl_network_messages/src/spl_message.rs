use std::{
    convert::{TryFrom, TryInto},
    time::Duration,
};

use color_eyre::{eyre::bail, Report, Result};
use nalgebra::{point, vector, Isometry2};
use serde::{Deserialize, Serialize};
use serialize_hierarchy::SerializeHierarchy;

use crate::{
    // bindings::{
    //     SPLStandardMessage, SPL_STANDARD_MESSAGE_DATA_SIZE, SPL_STANDARD_MESSAGE_STRUCT_HEADER,
    //     SPL_STANDARD_MESSAGE_STRUCT_VERSION,
    // },
    BallPosition,
    PlayerNumber,
    HULKS_TEAM_NUMBER,
};

use bifrost::communication::SPLStandardMessage;

use bifrost::serialization::{Decode, Encode};

#[derive(
    Clone, Copy, Debug, Encode, Default, Decode, Serialize, Deserialize, SerializeHierarchy,
)]
pub enum SPLPacket {
    HeardWhistle {
        heard_whistle: bool,
    },
    #[default]
    LookingForBall,
    BallPosition([f32; 2]),
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, SerializeHierarchy)]
pub struct SplMessage {
    pub player_number: PlayerNumber,
    pub fallen: bool,
    pub robot_to_field: Isometry2<f32>,
    pub ball_position: Option<BallPosition>,
    pub packet: Option<SPLPacket>,
}

impl TryFrom<&mut &[u8]> for SplMessage {
    type Error = Report;

    fn try_from(buffer: &mut &[u8]) -> Result<Self> {
        let message = SPLStandardMessage::decode(buffer)?;
        message.try_into()
    }
}

impl TryFrom<SPLStandardMessage<SPLPacket>> for SplMessage {
    type Error = Report;

    fn try_from(message: SPLStandardMessage<SPLPacket>) -> Result<Self> {
        Ok(SplMessage {
            player_number: match message.player_num {
                1 => PlayerNumber::One,
                2 => PlayerNumber::Two,
                3 => PlayerNumber::Three,
                4 => PlayerNumber::Four,
                5 => PlayerNumber::Five,
                6 => PlayerNumber::Six,
                7 => PlayerNumber::Seven,
                _ => bail!("unexpected player number {}", message.player_num),
            },
            fallen: match message.fallen {
                1 => true,
                0 => false,
                _ => bail!("unexpected fallen state"),
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
            packet: message.payload.data,
        })
    }
}

impl TryFrom<SplMessage> for Vec<u8> {
    type Error = Report;
    fn try_from(message: SplMessage) -> Result<Self> {
        let mut buf = Vec::new();
        SPLStandardMessage::<SPLPacket>::from(message).encode(&mut buf)?;

        Ok(buf)
    }
}

impl From<SplMessage> for SPLStandardMessage<SPLPacket> {
    fn from(message: SplMessage) -> Self {
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
        SPLStandardMessage::<SPLPacket>::new(
            match message.player_number {
                PlayerNumber::One => 1,
                PlayerNumber::Two => 2,
                PlayerNumber::Three => 3,
                PlayerNumber::Four => 4,
                PlayerNumber::Five => 5,
                PlayerNumber::Six => 6,
                PlayerNumber::Seven => 7,
            },
            HULKS_TEAM_NUMBER,
            u8::from(message.fallen),
            [
                message.robot_to_field.translation.vector.x * 1000.0,
                message.robot_to_field.translation.vector.y * 1000.0,
                message.robot_to_field.rotation.angle(),
            ],
            ball_age,
            ball_position,
            message.packet.into(),
        )
    }
}

#[cfg(test)]
mod test {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, SQRT_2};

    use approx::assert_relative_eq;
    use nalgebra::vector;

    use super::*;

    #[test]
    fn zero_isometry() {
        let input_message = SplMessage {
            player_number: PlayerNumber::One,
            fallen: false,
            robot_to_field: Isometry2::default(),
            ball_position: None,
            packet: None,
        };
        let output_message: SPLStandardMessage<SPLPacket> = input_message.into();

        assert_relative_eq!(output_message.pose[0], 0.0);
        assert_relative_eq!(output_message.pose[1], 0.0);
        assert_relative_eq!(output_message.pose[2], 0.0);

        let input_message_again: SplMessage = output_message.try_into().unwrap();

        assert_relative_eq!(input_message_again.robot_to_field, Isometry2::default());
    }

    #[test]
    fn one_to_the_left_isometry() {
        let input_message = SplMessage {
            player_number: PlayerNumber::One,
            fallen: false,
            robot_to_field: Isometry2::new(vector![0.0, 1.0], FRAC_PI_2),
            ball_position: None,
            packet: None,
        };
        let output_message: SPLStandardMessage<SPLPacket> = input_message.into();

        assert_relative_eq!(output_message.pose[0], 0.0, epsilon = 0.001);
        assert_relative_eq!(output_message.pose[1], 1000.0, epsilon = 0.001);
        assert_relative_eq!(output_message.pose[2], FRAC_PI_2, epsilon = 0.001);

        let input_message_again: SplMessage = output_message.try_into().unwrap();

        assert_relative_eq!(
            input_message_again.robot_to_field,
            Isometry2::new(vector![0.0, 1.0], FRAC_PI_2),
            epsilon = 0.001
        );
    }

    #[test]
    fn one_schräg_to_the_top_right_isometry() {
        let input_message = SplMessage {
            player_number: PlayerNumber::One,
            fallen: false,
            robot_to_field: Isometry2::new(vector![1.0, 1.0], FRAC_PI_4),
            ball_position: None,
            packet: None,
        };
        let output_message: SPLStandardMessage<SPLPacket> = input_message.into();

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

        let input_message_again: SplMessage = output_message.try_into().unwrap();

        assert_relative_eq!(
            input_message_again.robot_to_field,
            Isometry2::new(vector![1.0, 1.0], FRAC_PI_4),
            epsilon = 0.001
        );
    }
}
