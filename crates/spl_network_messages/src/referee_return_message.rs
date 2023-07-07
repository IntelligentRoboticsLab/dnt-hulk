use bifrost::serialization::Encode;
use serde::{Deserialize, Serialize};


#[derive(Encode, Debug, Clone, Deserialize, Serialize)]
pub struct RefereeMessage {
   /// "RGrt"
   pub header: [u8; 4],

   /// Has to be set to GAMECONTROLLER_RETURN_STRUCT_VERSION
   pub version: u8,

   /// Player number starts with 1
   pub player_num: u8,

   /// Team number
   pub team_num: u8,

   /// 1 means that the robot is fallen, 0 means that the robot can play
   pub fallen: u8,

   /// Position and orientation of the robot
   ///
   /// coordinates in millimeters
   /// 0,0 is in center of field
   /// +ve x-axis points towards the goal we are attempting to score on
   /// +ve y-axis is 90 degrees counter clockwise from the +ve x-axis
   /// angle in radians, 0 along the +x axis, increasing counter clockwise
   pub pose: [f32; 3], // x,y,theta

   /// ball information
   pub ball_age: f32, // seconds since this robot last saw the ball. -1.f if we haven't seen it

   /// Position of ball relative to the robot
   ///
   /// coordinates in millimeters
   /// 0,0 is in center of the robot
   /// +ve x-axis points forward from the robot
   /// +ve y-axis is 90 degrees counter clockwise from the +ve x-axis
   pub ball: [f32; 2],
}


impl RefereeMessage {
    /// Construct a new [`RoboCupGameControlReturnData`] using the specified arguments.
    pub fn new(
        header: [u8; 4],
        version: u8,
        player_num: u8,
        team_num: u8,
        fallen: u8,
        pose: [f32; 3],
        ball_age: f32,
        ball: [f32; 2],
    ) -> Self {
        Self {
            header,
            version,
            player_num,
            team_num,
            fallen,
            pose,
            ball_age,
            ball,
        }
    }
}