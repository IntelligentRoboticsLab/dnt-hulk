use color_eyre::eyre::Context;
use eframe::egui::{Response, Slider, Ui, Widget};
use log::{error, info};
use nalgebra::Vector3;
use serde_json::Value;
use std::{ops::RangeInclusive, sync::Arc, time::Duration};
use tokio::sync::mpsc;
use gilrs::{Axis, Button, Event, EventType, Gilrs};

use crate::{
    nao::Nao, panel::Panel, repository_parameters::RepositoryParameters, value_buffer::ValueBuffer,
};

use super::parameter::{add_save_button, subscribe};

type SubscribedType = Vector3<f64>;

struct ControllerStick {
    left_x: f32,
    left_y: f32,
    right_x: f32,
    right_y: f32,   
}

struct CameraParameterSubscriptions<DeserializedValueType> {
    human_friendly_label: String,
    path: String,
    value_buffer: Option<ValueBuffer>,
    value: DeserializedValueType,
    update_notify_receiver: mpsc::Receiver<()>,
}

pub struct ControllerPanel {
    // nao: Arc<Nao>,
    // repository_parameters: Option<RepositoryParameters>,
    extrinsic_rotation_subscriptions: [CameraParameterSubscriptions<Option<SubscribedType>>; 2],
}

const CAMERA_KEY_BASE: &str = "camera_matrix_parameters.vision_";
const ROTATIONS: &str = ".extrinsic_rotations";

impl Panel for ControllerPanel {
    const NAME: &'static str = "Controller";

    fn new(nao: Arc<Nao>, _value: Option<&Value>) -> Self {
        let extrinsic_rotation_subscriptions = ["Top", "Bottom"].map(|name| {
            let path = CAMERA_KEY_BASE.to_owned() + name.to_lowercase().as_str() + ROTATIONS;

            let (update_notify_sender, update_notify_receiver) = mpsc::channel(1);
            let value_buffer = subscribe(nao.clone(), &path, update_notify_sender);

            info!("Subscribing to path {}", path);

            CameraParameterSubscriptions {
                human_friendly_label: name.to_string(),
                path,
                value_buffer,
                value: None,
                update_notify_receiver,
            }
        });

        Self {
            // nao,
            // repository_parameters: RepositoryParameters::try_default().ok(),
            extrinsic_rotation_subscriptions,
        }
    }
}

fn add_extrinsic_calibration_ui_components(
    ui: &mut Ui,
    // nao: Arc<Nao>,
    // repository_parameters: &Option<RepositoryParameters>,
    extrinsic_rotations_subscription: &mut CameraParameterSubscriptions<Option<SubscribedType>>,
) {
    // let extrinsic_rotations_buffer_option = &extrinsic_rotations_subscription.value_buffer;
    let mut extrinsic_rotations_option = &mut extrinsic_rotations_subscription.value;
    // let label = &extrinsic_rotations_subscription.human_friendly_label;
    // let extrinsic_rotations_subscription_path = &extrinsic_rotations_subscription.path;
    // let extrinsic_rotations_update_notify_receiver =
    //     &mut extrinsic_rotations_subscription.update_notify_receiver;

    // let extrinsic_maximum_degrees = 15.0;
    let mut gilrs = Gilrs::new().unwrap();
    println!("UI run update");
    // ui.horizontal(|ui| {
    //     // if let Some(buffer) = &extrinsic_rotations_buffer_option {
    //     //     match buffer.get_latest() {
    //     //         Ok(value) => {
    //     //             if extrinsic_rotations_update_notify_receiver
    //     //                 .try_recv()
    //     //                 .is_ok()
    //     //             {
    //     //                 *extrinsic_rotations_option =
    //     //                     serde_json::from_value::<SubscribedType>(value).ok();
    //     //             }
    //     //         }
    //     //         Err(error) => {
    //     //             ui.label(format!("{error:#?}"));
    //     //         }
    //     //     }
    //     // }

    //     // ui.label(format!("{label:#} Camera"));

    //     add_save_button(
    //         ui,
    //         extrinsic_rotations_subscription_path,
    //         || {
    //             serde_json::to_value(&extrinsic_rotations_option)
    //                 .wrap_err("Conveting CameraMatrixParameters to serde_json::Value failed.")
    //         },
    //         nao.clone(),
    //         repository_parameters,
    //     );
    // });

    // ui.style_mut().spacing.slider_width = ui.available_size().x - 100.0;
    // let mut changed = false;
    // ui.label(format!(
    //     "Extrinsic Rotations [{}°, {}°]",
    //     -extrinsic_maximum_degrees, extrinsic_maximum_degrees
    // ));
    match &mut extrinsic_rotations_option {
        // Some(camera_parameter_value) => {
        //     // for (axis_value, axis_name) in camera_parameter_value
        //     //     .iter_mut()
        //     //     .zip(["Roll", "Pitch", "Yaw"])
        //     // {
        //     //     // let slider = Slider::new(
        //     //     //     axis_value,
        //     //     //     RangeInclusive::new(-extrinsic_maximum_degrees, extrinsic_maximum_degrees),
        //     //     // )
        //     //     // .text(axis_name)
        //     //     // .smart_aim(false);
        //     //     // if ui.add(slider).changed() {
        //     //     //     changed = true
        //     //     // };
        //     // }
        // }
        _ => {
            for (_id, gamepad) in gilrs.gamepads() {
                ui.label(format!("{} is {:?}", gamepad.name(), gamepad.power_info()));
            }
            let mut active_gamepad = None;

    // loop {
        // Examine new events
        println!("before while");

        if let Some(Event { id, event, .. }) = gilrs.next_event_blocking(Some(Duration::from_secs(5))) {
            active_gamepad = Some(id);
            // store the stick values the struct Stick
            println!("in while");
            let mut controller = ControllerStick {
                left_x: 0.0,
                left_y: 0.0,
                right_x: 0.0,
                right_y: 0.0,
            };

            if let EventType::AxisChanged(axis, value, _) = event {
                match axis {
                    Axis::LeftStickX => {
                        controller.left_x = value;
                    }
                    Axis::LeftStickY => {
                        controller.left_y = value;
                    }
                    Axis::RightStickX => {
                        controller.right_x = value;
                    }
                    Axis::RightStickY => {
                        controller.right_y = value;
                    }
                    _ => {}
                }
                ui.label(format!("Left X: {:?}", controller.left_x));
                ui.label(format!("Left Y: {:?}", controller.left_y));
                ui.label(format!("Right X: {:?}", controller.right_x));
                ui.label(format!("Right Y: {:?}", controller.right_y));
            }
        }

        // You can also use cached gamepad state
        if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            if gamepad.is_pressed(Button::West) {
                ui.label(format!("Button West is pressed (XBox - A, PS - X)"));
            } else if gamepad.is_pressed(Button::North) {
                ui.label(format!("Button North is pressed (XBox - Y, PS - Triangle)"));
            } else if gamepad.is_pressed(Button::East) {
                ui.label(format!("Button East is pressed (XBox - B, PS - Circle)"));
            } else if gamepad.is_pressed(Button::South) {
                ui.label(format!("Button South is pressed (XBox - X, PS - Square)"));
            }
        }
    // }
            ui.label("No robot connected.");
        }
    };
    // if changed {
    //     if let Some(camera_parameter_value) = extrinsic_rotations_option {
    //         match serde_json::value::to_value(camera_parameter_value) {
    //             Ok(value) => {
    //                 nao.update_parameter_value(extrinsic_rotations_subscription_path, value);
    //             }
    //             Err(error) => error!("Failed to serialize parameter value: {error:#?}"),
    //         }
    //     }
    // }
}

impl Widget for &mut ControllerPanel {
    fn ui(self, ui: &mut Ui) -> Response {
        println!("window run");
        ui.vertical(|ui| {
            for extrinsic_rotation_subscription in &mut self.extrinsic_rotation_subscriptions {
                add_extrinsic_calibration_ui_components(
                    ui,
                    // self.nao.clone(),
                    // &self.repository_parameters,
                    extrinsic_rotation_subscription,
                );

                ui.separator();
            }
        })
        .response
    }
}
