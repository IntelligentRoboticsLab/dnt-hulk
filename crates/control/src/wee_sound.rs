use context_attribute::context;
use std::time::{Duration, Instant};

use types::{FallState, PrimaryState};

use kira::{
    manager::{backend::cpal::CpalBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    tween::Tween,
};

pub struct PlaySound {
    sound_played: bool,
    manager: AudioManager<CpalBackend>,
    last_played: Option<Instant>,
}

#[context]
pub struct CreationContext {}

#[context]
pub struct CycleContext {
    pub fall_state: Input<FallState, "fall_state">,
    pub has_ground_contact: Input<bool, "has_ground_contact">,
    pub primary_state: Input<PrimaryState, "primary_state">,
    pub wee_sound_timeout: Parameter<Duration, "wee_sound.timeout">,
}

#[context]
#[derive(Default)]
pub struct MainOutputs {}

impl PlaySound {
    pub fn new(_context: CreationContext) -> color_eyre::Result<Self> {
        Ok(Self {
            sound_played: false,
            manager: AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap(),
            last_played: None,
        })
    }

    pub fn cycle(&mut self, context: CycleContext) -> color_eyre::Result<MainOutputs> {
        if *context.primary_state == PrimaryState::Unstiff {
            return Ok(MainOutputs {});
        }

        if let Some(last_played) = self.last_played {
            if last_played.elapsed() < *context.wee_sound_timeout {
                return Ok(MainOutputs {});
            }
        }

        if !context.has_ground_contact
            && *context.fall_state == FallState::Upright
            && !self.sound_played
        {
            self.sound_played = true;
            self.last_played = Some(Instant::now());
            let sound_data =
                StaticSoundData::from_file("/etc/sounds/weeeee.wav", StaticSoundSettings::new())
                    .unwrap();
            let mut sound = self.manager.play(sound_data).unwrap();
            sound
                .set_volume(
                    0.1,
                    Tween {
                        duration: Duration::from_secs(0),
                        ..Default::default()
                    },
                )
                .unwrap();
        } else if *context.has_ground_contact {
            self.sound_played = false;
        }

        Ok(MainOutputs {})
    }
}
