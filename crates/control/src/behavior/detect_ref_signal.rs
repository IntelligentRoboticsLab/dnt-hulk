use nalgebra::Point2;
use types::{MotionCommand, PrimaryState, WorldState, FilteredWhistle, HeadMotion};

pub fn execute(world_state: &WorldState, filtered_whistle: &FilteredWhistle) -> Option<MotionCommand> {
    let head_motion = HeadMotion::LookAt { target: Point2::new(0.0, 0.0) };
    match (world_state.robot.primary_state, filtered_whistle.is_detected) {
        (PrimaryState::Set | PrimaryState::Playing, true) => {
            println!("Ref signal");
            Some(MotionCommand::Stand {head: head_motion, is_energy_saving: false}
        )},
        (_, _) => None,
    }
}
