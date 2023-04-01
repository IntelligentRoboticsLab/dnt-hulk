use std::time::SystemTime;

use module_derive::{module, require_some};
use spl_network::GameControllerStateMessage;
use types::{GameControllerState, SensorData};

pub struct GameControllerFilter {
    game_controller_state: Option<GameControllerState>,
    last_game_state_change: Option<SystemTime>,
}

#[module(control)]
#[input(path = sensor_data, data_type = SensorData)]
#[perception_input(path = game_controller_state_message, data_type = GameControllerStateMessage, cycler = spl_network)]
#[main_output(data_type = GameControllerState)]
impl GameControllerFilter {}

impl GameControllerFilter {
    fn new(_context: NewContext) -> anyhow::Result<Self> {
        Ok(Self {
            game_controller_state: None,
            last_game_state_change: None,
        })
    }

    fn cycle(&mut self, context: CycleContext) -> anyhow::Result<MainOutputs> {
        let cycle_start_time = require_some!(context.sensor_data).cycle_info.start_time;

        for game_controller_state_message in context
            .game_controller_state_message
            .persistent
            .values()
            .flatten()
            .copied()
            .flatten()
        {
            let game_state_changed = match &self.game_controller_state {
                Some(game_controller_state) => {
                    game_controller_state.game_state != game_controller_state_message.game_state
                }
                None => true,
            };
            if game_state_changed {
                self.last_game_state_change = Some(cycle_start_time);
            }
            self.game_controller_state = Some(GameControllerState {
                game_state: game_controller_state_message.game_state,
                game_phase: game_controller_state_message.game_phase,
                kicking_team: game_controller_state_message.kicking_team,
                last_game_state_change: self.last_game_state_change.unwrap(),
                penalties: game_controller_state_message.hulks_team.clone().into(),
                remaining_amount_of_messages: game_controller_state_message
                    .hulks_team
                    .remaining_amount_of_messages,
                set_play: match game_controller_state_message.set_play {
                    spl_network::SetPlay::None => None,
                    _ => Some(game_controller_state_message.set_play),
                },
            });
        }
        Ok(MainOutputs {
            game_controller_state: self.game_controller_state,
        })
    }
}
