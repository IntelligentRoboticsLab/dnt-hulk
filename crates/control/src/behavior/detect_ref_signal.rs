use crate::behavior::node::VisRefFieldSide;
use nalgebra::point;
use types::{FieldDimensions, HeadMotion, MotionCommand, PrimaryState, WorldState};

pub fn execute(
    world_state: &WorldState,
    field_dimensions: &FieldDimensions,
    visref_fieldside: VisRefFieldSide,
) -> Option<MotionCommand> {
    let robot_to_field = world_state
        .robot
        .robot_to_field
        .expect("Failed to get robot_to_field.");

    // The refere will stand at the T junction of the field, so use field_dimensions to get the location of the T junction
    let visref_location = match visref_fieldside {
        VisRefFieldSide::Left => point![field_dimensions.length / 2.0, 0.0],
        VisRefFieldSide::Right => point![field_dimensions.length / 2.0, field_dimensions.width]
    };

    let head_motion = HeadMotion::LookAt {
        target: robot_to_field.inverse() * visref_location,
    };
    match world_state.robot.primary_state {
        PrimaryState::Ready | PrimaryState::Set | PrimaryState::Playing => {
            println!("Ref signal");
            Some(MotionCommand::Stand {
                head: head_motion,
                is_energy_saving: false,
            })
        }
        _ => None,
    }
}
