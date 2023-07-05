use std::time::{SystemTime};
use color_eyre::{eyre::WrapErr, Result};
use context_attribute::context;
use compiled_nn::CompiledNN;
use nalgebra::Isometry2;
use spl_network_messages::{GameControllerReturnMessage, PlayerNumber};
use types::{
    hardware::Interface,
    messages::{OutgoingMessage},
    CycleTime, FilteredWhistle, ycbcr422_image::YCbCr422Image
};
use image::{RgbImage, ImageBuffer, Rgb, imageops};

pub struct Referee {
    last_heard_timestamp: Option<SystemTime>,
    lstm: CompiledNN
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
    // pub image: Input<YCbCr422Image, "image">,
}

impl Referee {
    pub fn new(_context: CreationContext) -> Result<Self> {
        let mut network = CompiledNN::default();
        network.compile("tools/machine-learning/referee_challange/conv_orig_aug.h5");

        Ok(Self {
            last_heard_timestamp: None,
            lstm: network,
        })
    }

    pub fn cycle(&mut self, context: CycleContext<impl Interface>) -> Result<()> {
        if context.filtered_whistle.started_this_cycle {
            if let Some(cycle_time) = self.last_heard_timestamp {
                match cycle_time.duration_since(cycle_time) {
                    Ok(duration) => {
                        if duration.as_secs() < 20 {
                            self.send_referee_message(&context, 1)?;
                        }
                    }
                    Err(_err) => {}
                }
            }
        }

        // let input_img = resize_image(&context.image);
        // let input = self.lstm.classifier.input_mut(0);

        // self.lstm.classifier.apply();
        // self.lstm.classifier.output(0).data[0];

        Ok(())
    }

    // fn rgb_image_from_buffer_422(width_422: u32, height: u32, buffer: &[YCbCr422]) -> RgbImage {
    //     let mut rgb_image = RgbImage::new(2 * width_422, height);

    //     for y in 0..height {
    //         for x in 0..width_422 {
    //             let pixel = buffer[(y * width_422 + x) as usize];
    //             let left_color: Rgb = YCbCr444 {
    //                 y: pixel.y1,
    //                 cb: pixel.cb,
    //                 cr: pixel.cr,
    //             }
    //             .into();
    //             let right_color: Rgb = YCbCr444 {
    //                 y: pixel.y2,
    //                 cb: pixel.cb,
    //                 cr: pixel.cr,
    //             }
    //             .into();
    //             rgb_image.put_pixel(
    //                 x * 2,
    //                 y,
    //                 image::Rgb([left_color.r, left_color.g, left_color.b]),
    //             );
    //             rgb_image.put_pixel(
    //                 x * 2 + 1,
    //                 y,
    //                 image::Rgb([right_color.r, right_color.g, right_color.b]),
    //             );
    //         }
    //     }

    //     rgb_image
    // }

    // fn resize_image(src: &YCbCr422Image) -> Result<YCbCr422Image> {
    //     let mut rgb:RgbImage = rgb_image_from_buffer_422(src.width() / 2, src.height());

    //     let resized_image = imageops::resize(&rgb, 256, 256, image::imageops::FilterType::Triangle);
    //     resized_image.to_image();
    //     YCbCr422Image::from_raw_buffer(256 / 2, 256,resized_image.as_raw());
    // }

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
