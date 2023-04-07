use std::f32::consts::FRAC_PI_4;

use framework::AdditionalOutput;
use nalgebra::{point, Isometry2, UnitComplex, Vector2};
use types::{
    rotate_towards, BallState, FieldDimensions, FilteredGameState, MotionCommand, PathObstacle,
    Side, WorldState,
};

use super::{head::LookAction, walk_to_pose::WalkAndStand};

#[allow(clippy::too_many_arguments)]
pub fn execute(
    world_state: &WorldState,
    field_dimensions: &FieldDimensions,
    field_side: Option<Side>,
    distance_to_ball: f32,
    maximum_x_in_ready_and_when_ball_is_not_free: f32,
    minimum_x: f32,
    walk_and_stand: &WalkAndStand,
    look_action: &LookAction,
    path_obstacles_output: &mut AdditionalOutput<Vec<PathObstacle>>,
) -> Option<MotionCommand> {
    let pose = support_pose(
        world_state,
        field_dimensions,
        field_side,
        distance_to_ball,
        maximum_x_in_ready_and_when_ball_is_not_free,
        minimum_x,
    )?;
    walk_and_stand.execute(pose, look_action.execute(), path_obstacles_output)
}

fn support_pose(
    world_state: &WorldState,
    field_dimensions: &FieldDimensions,
    field_side: Option<Side>,
    distance_to_ball: f32,
    maximum_x_in_ready_and_when_ball_is_not_free: f32,
    minimum_x: f32,
) -> Option<Isometry2<f32>> {
    let robot_to_field = world_state.robot.robot_to_field?;
    let ball = world_state
        .ball
        .map(|ball| BallState {
            position: robot_to_field * ball.position,
            field_side: ball.field_side,
            penalty_shot_direction: Default::default(),
        })
        .unwrap_or_default();
    let side = field_side.unwrap_or_else(|| ball.field_side.opposite());
    let offset_vector = UnitComplex::new(match side {
        Side::Left => -FRAC_PI_4,
        Side::Right => FRAC_PI_4,
    }) * -(Vector2::x() * distance_to_ball);
    let supporting_position = ball.position + offset_vector;
    let clamped_x = match world_state.filtered_game_state {
        Some(FilteredGameState::Ready { .. })
        | Some(FilteredGameState::Playing {
            ball_is_free: false,
        }) => supporting_position.x.clamp(
            minimum_x.min(maximum_x_in_ready_and_when_ball_is_not_free),
            minimum_x.max(maximum_x_in_ready_and_when_ball_is_not_free),
        ),
        _ => supporting_position
            .x
            .clamp(minimum_x, field_dimensions.length / 2.0),
    };
    let clamped_position = point![clamped_x, supporting_position.y];
    let support_pose = Isometry2::new(
        clamped_position.coords,
        rotate_towards(clamped_position, ball.position).angle(),
    );
    Some(robot_to_field.inverse() * support_pose)
}