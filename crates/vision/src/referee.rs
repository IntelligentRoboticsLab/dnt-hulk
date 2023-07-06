use color_eyre::{Result};
use context_attribute::context;
use framework::{MainOutput};
use compiled_nn::CompiledNN;
use types::{
    ycbcr422_image::YCbCr422Image, YCbCr422, YCbCr444, Rgb, handsignal::HandSignal,
};
use image::{RgbImage, imageops};

pub struct Referee {
    cnn: CompiledNN
}

#[context]
pub struct CreationContext {}

#[context]
#[derive(Default)]
pub struct MainOutputs {
    pub detected_handsignal: MainOutput<HandSignal>,
}

#[context]
pub struct CycleContext {
    pub image: Input<YCbCr422Image, "image">,
}

impl Referee {
    pub fn new(_context: CreationContext) -> Result<Self> {
        let mut network = CompiledNN::default();
        network.compile("tools/machine-learning/referee_challange/conv_orig_aug.h5");

        Ok(Self {
            cnn: network,
        })
    }

    pub fn cycle(&mut self, mut context: CycleContext) -> Result<MainOutputs> {
        // let input_img = self.resize_image(&context.image);
        // let input_tensor = self.cnn.input_mut(0);

        // // for y in 0..input_img.height() {
        // //     for x in 0..input_img.width() {
        // //         input_tensor.data[(x + y * 256) as usize] = types::Rgb::from(input_img.at(x, y));
        // //     }
        // // }

        // self.cnn.apply();
        // self.cnn.output(0).data[0];

        let temp = 8;
        Ok(MainOutputs {
            detected_handsignal: HandSignal { handsignal: temp }.into(),
        })
    }

    fn rgb_image_from_buffer_422(&self, width_422: u32, height: u32, buffer: &[YCbCr422]) -> RgbImage {
        let mut rgb_image = RgbImage::new(2 * width_422, height);

        for y in 0..height {
            for x in 0..width_422 {
                let pixel = buffer[(y * width_422 + x) as usize];
                let left_color: Rgb = YCbCr444 {
                    y: pixel.y1,
                    cb: pixel.cb,
                    cr: pixel.cr,
                }
                .into();
                let right_color: Rgb = YCbCr444 {
                    y: pixel.y2,
                    cb: pixel.cb,
                    cr: pixel.cr,
                }
                .into();
                rgb_image.put_pixel(
                    x * 2,
                    y,
                    image::Rgb([left_color.r, left_color.g, left_color.b]),
                );
                rgb_image.put_pixel(
                    x * 2 + 1,
                    y,
                    image::Rgb([right_color.r, right_color.g, right_color.b]),
                );
            }
        }

        rgb_image
    }

    fn resize_image(&self, src: &YCbCr422Image) -> YCbCr422Image {
        let mut rgb:RgbImage = self.rgb_image_from_buffer_422(src.width() / 2, src.height(), src.buffer());

        let resized_image = imageops::resize(&rgb, 256, 256, image::imageops::FilterType::Triangle);
        let result = YCbCr422Image::from_raw_buffer(256 / 2, 256,resized_image.to_vec());
        result
    }
}
