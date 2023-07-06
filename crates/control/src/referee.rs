use color_eyre::{eyre::WrapErr, Result};
use context_attribute::context;
use nalgebra::Isometry2;
use rand::Rng;
use spl_network_messages::{GameControllerReturnMessage, PlayerNumber};
use std::time::SystemTime;
use types::{hardware::Interface, messages::OutgoingMessage, CycleTime, FilteredWhistle};

pub struct Referee {
    last_heard_timestamp: Option<SystemTime>,
}

#[context]
pub struct CreationContext {
    pub player_number: Parameter<PlayerNumber, "player_number">,
}

#[context]
pub struct CycleContext {
    pub filtered_whistle: Input<FilteredWhistle, "filtered_whistle">,
    pub hardware: HardwareInterface,
    pub cycle_time: Input<CycleTime, "cycle_time">,
    pub player_number: Parameter<PlayerNumber, "player_number">,
    pub robot_to_field: Input<Option<Isometry2<f32>>, "robot_to_field?">,
}

impl Referee {
    pub fn new(_context: CreationContext) -> Result<Self> {
        Ok(Self {
            last_heard_timestamp: None,
        })
    }

    pub fn cycle(&mut self, context: CycleContext<impl Interface>) -> Result<()> {
        if context.filtered_whistle.started_this_cycle {
            if let Some(cycle_time) = self.last_heard_timestamp {
                match cycle_time.duration_since(cycle_time) {
                    Ok(duration) => {
                        if duration.as_secs() < 20 {
                            let mut rng_gen = rand::thread_rng();
                            let handsignal: u8 = rng_gen.gen_range(1..=16);
                            self.send_referee_message(&context, handsignal)?;
                        }
                    }
                    Err(_err) => {}
                }
            }
        }

        Ok(())
    }

    fn send_referee_message(
        &mut self,
        context: &CycleContext<impl Interface>,
        handsignal: u8,
    ) -> Result<()> {
        // self.last_transmitted_game_controller_return_message = Some(cycle_start_time);
        context
            .hardware
            .write_to_network(OutgoingMessage::GameController(
                GameControllerReturnMessage {
                    player_number: *context.player_number,
                    fallen: unsafe { std::mem::transmute(handsignal) },
                    robot_to_field: context.robot_to_field.copied().unwrap_or_default(),
                    ball_position: None,
                },
            ))
            .wrap_err("failed to write GameControllerReturnMessage to hardware")?;

        Ok(())
    }
}
