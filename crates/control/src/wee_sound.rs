use context_attribute::context;
use std::time::Duration;

use types::FallState;

use kira::{
    manager::{backend::cpal::CpalBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    tween::Tween,
};

pub struct PlaySound {
    sound_played: bool,

    manager: AudioManager<CpalBackend>,
}

#[context]
pub struct CreationContext {}

#[context]
pub struct CycleContext {
    pub fall_state: Input<FallState, "fall_state">,
    pub has_ground_contact: Input<bool, "has_ground_contact">,
}

#[context]
#[derive(Default)]
pub struct MainOutputs {}

impl PlaySound {
    pub fn new(_context: CreationContext) -> color_eyre::Result<Self> {
        Ok(Self {
            sound_played: false,

            manager: AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap(),
        })
    }

    pub fn cycle(&mut self, context: CycleContext) -> color_eyre::Result<MainOutputs> {
        if !context.has_ground_contact
            && *context.fall_state == FallState::Upright
            && !self.sound_played
        {
            self.sound_played = true;
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
