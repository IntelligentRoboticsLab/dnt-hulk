use nalgebra::point;
use types::{FieldDimensions, HeadMotion, MotionCommand, PrimaryState, WorldState};

pub fn execute(
    world_state: &WorldState,
    field_dimensions: &FieldDimensions,
) -> Option<MotionCommand> {
    let robot_to_field = world_state
        .robot
        .robot_to_field
        .expect("Failed to get robot_to_field.");

    // The refere will stand at the T junction of the field, so use field_dimensions to get the location of the T junction
    let visref_location = point![0.0, field_dimensions.width / 2.0];
    let head_motion = HeadMotion::LookAt {
        target: robot_to_field.inverse() * visref_location,
    };
    match world_state.robot.primary_state {
        PrimaryState::Set | PrimaryState::Playing => {
            println!("Ref signal");
            Some(MotionCommand::Stand {
                head: head_motion,
                is_energy_saving: false,
            })
        }
        _ => None,
    }
}
