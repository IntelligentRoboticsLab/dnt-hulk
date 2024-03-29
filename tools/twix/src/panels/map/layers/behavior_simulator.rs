use std::sync::Arc;

use color_eyre::Result;
use eframe::epaint::{Color32, Stroke};
use nalgebra::Isometry2;
use types::{FieldDimensions, MotionCommand};

use crate::{
    nao::Nao, panels::map::layer::Layer, players_value_buffer::PlayersValueBuffer,
    twix_painter::TwixPainter,
};

const TRANSPARENT_BLUE: Color32 = Color32::from_rgba_premultiplied(0, 0, 202, 150);
const TRANSPARENT_LIGHT_BLUE: Color32 = Color32::from_rgba_premultiplied(136, 170, 182, 150);

pub struct BehaviorSimulator {
    robot_to_field: PlayersValueBuffer,
    motion_command: PlayersValueBuffer,
}

impl Layer for BehaviorSimulator {
    const NAME: &'static str = "Behavior Simulator";

    fn new(nao: Arc<Nao>) -> Self {
        let robot_to_field = PlayersValueBuffer::try_new(
            nao.clone(),
            "BehaviorSimulator.main.databases",
            "main_outputs.robot_to_field",
        )
        .unwrap();
        let motion_command = PlayersValueBuffer::try_new(
            nao,
            "BehaviorSimulator.main.databases",
            "main_outputs.motion_command",
        )
        .unwrap();
        Self {
            robot_to_field,
            motion_command,
        }
    }

    fn paint(&self, painter: &TwixPainter, _field_dimensions: &FieldDimensions) -> Result<()> {
        for (player_number, value_buffer) in self.robot_to_field.0.iter() {
            let Ok(robot_to_field): Result<Isometry2<f32>> = value_buffer.parse_latest() else {
                continue
            };

            let pose_color = Color32::from_white_alpha(63);
            let pose_stroke = Stroke {
                width: 0.02,
                color: Color32::BLACK,
            };

            if let Ok(MotionCommand::Walk { path, .. }) =
                self.motion_command.0[player_number].parse_latest()
            {
                painter.path(
                    robot_to_field,
                    path,
                    TRANSPARENT_BLUE,
                    TRANSPARENT_LIGHT_BLUE,
                    0.025,
                );
            }

            painter.pose(robot_to_field, 0.15, 0.25, pose_color, pose_stroke);
        }

        Ok(())
    }
}
