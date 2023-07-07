use color_eyre::{eyre::WrapErr, Result};
use context_attribute::context;
use nalgebra::Isometry2;
use rand::Rng;
use spl_network_messages::{PlayerNumber, RefereeMessage};
use std::time::{SystemTime, Duration};
use types::{hardware::Interface, messages::OutgoingMessage, CycleTime, FilteredWhistle};

pub struct Referee {
    last_heard_timestamp: Option<SystemTime>,
    sent: bool,
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
            sent: false,
        })
    }

    pub fn cycle(&mut self, context: CycleContext<impl Interface>) -> Result<()> {
        if context.filtered_whistle.started_this_cycle {
            if let Some(cycle_time) = self.last_heard_timestamp {
                match cycle_time.duration_since(cycle_time) {
                    Ok(duration) => {
                        if duration.as_secs() < 20 && !self.sent{
                            let mut rng_gen = rand::thread_rng();
                            let handsignal: u8 = rng_gen.gen_range(1..=16);
                            self.send_referee_message(&context, handsignal, duration)?;
                            self.sent = true;
                        }
                        else if duration.as_secs() >= 20{
                            self.sent = false;
                            self.last_heard_timestamp = None;
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
        duration: Duration
    ) -> Result<()> {
        context
            .hardware
            .write_to_network(OutgoingMessage::RefereeReturnData(
                RefereeMessage {
                    header:  [82, 71, 114, 116],
                    version: 255,
                    player_num: *context.player_number as u8,
                    team_num: 8,
                    fallen: handsignal,
                    pose: [0.0, 0.0, 0.0],
                    ball_age: duration.as_secs_f32(),
                    ball: [0.0, 0.0],
                },
            ))
            .wrap_err("failed to write GameControllerReturnMessage to hardware")?;

        Ok(())
    }
}
