use std::{io::stdout, sync::Arc};

use chrono::Duration;
use color_eyre::{eyre::bail, install, Result};
use communication::server::Runtime;
use framework::{Reader, Writer};
use robot::Robot;
use serde::{Deserialize, Serialize};
use serialize_hierarchy::SerializeHierarchy;
use tokio::{select, sync::Notify};
use tokio_util::sync::CancellationToken;
use types::PrimaryState;

mod cycler;
mod interfake;
mod robot;

fn setup_logger(is_verbose: bool) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}  {:<18}  {:>5}  {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(if is_verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .chain(stdout())
        .apply()?;
    Ok(())
}

#[derive(Clone, Serialize, Deserialize, SerializeHierarchy)]
struct Configuration {
    time: usize,
}

#[derive(Clone, Default, Serialize, Deserialize, SerializeHierarchy)]
struct MainOutputs {
    x: f32,
    database: cycler::Database,
}
#[derive(Clone, Default, Serialize, Deserialize, SerializeHierarchy)]
struct BehaviorDatabase {
    main_outputs: MainOutputs,
}

async fn timeline_server(
    keep_running: CancellationToken,
    parameters_reader: Reader<Configuration>,
    parameters_changed: Arc<Notify>,
    outputs_writer: Writer<BehaviorDatabase>,
    outputs_changed: Arc<Notify>,
    frames: Vec<Vec<cycler::Database>>,
) {
    loop {
        select! {
            _ = parameters_changed.notified() => {
                {
                    let mut outputs = outputs_writer.next();
                    let parameters = parameters_reader.next();

                    outputs.main_outputs.x = (parameters.time as f32).sin();
                    outputs.main_outputs.database = frames[parameters.time][0].clone();
                }
                outputs_changed.notify_waiters();
            }
            _ = keep_running.cancelled() => {
                break
            }
        }
    }
}

struct State {
    time_elapsed: Duration,
    robots: Vec<Robot>,
}

impl State {
    fn new(keep_running: CancellationToken, robot_count: usize) -> Self {
        let robots: Vec<_> = (0..robot_count).map(|index| Robot::new(index)).collect();

        Self {
            time_elapsed: Duration::zero(),
            robots,
        }
    }

    pub fn stiffen_robots(&mut self) {
        for robot in &mut self.robots {
            robot.primary_state = PrimaryState::Playing;
        }
    }
}

fn run(keep_running: CancellationToken) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    let (outputs_writer, outputs_reader) = framework::multiple_buffer_with_slots([
        Default::default(),
        Default::default(),
        Default::default(),
    ]);

    let outputs_changed = std::sync::Arc::new(tokio::sync::Notify::new());
    let (subscribed_outputs_writer, _subscribed_outputs_reader) =
        framework::multiple_buffer_with_slots([
            Default::default(),
            Default::default(),
            Default::default(),
        ]);

    let communication_server = Runtime::<Configuration>::start(
        Some("[::]:1337"),
        "tools/behavior-simulator",
        "behavior_simulator".to_string(),
        "behavior_simulator".to_string(),
        2,
        keep_running.clone(),
    )?;

    communication_server.register_cycler_instance(
        "BehaviorSimulator",
        outputs_changed.clone(),
        outputs_reader,
        subscribed_outputs_writer,
    );

    let mut state = State::new(keep_running.clone(), 1);
    state.stiffen_robots();

    let mut frames = Vec::new();
    for _frame_index in 0..20 {
        let mut robot_frames = Vec::new();

        for robot in &mut state.robots {
            println!("cycling");
            robot.cycle().unwrap();
            let database = robot.database.clone();
            println!("{:?}", database.main_outputs.motion_command);
            // match database.main_outputs.motion_command {
            //     types::MotionCommand::Walk {
            //         head,
            //         path,
            //         left_arm,
            //         right_arm,
            //         orientation_mode,
            //     } => todo!(),
            //     types::MotionCommand::InWalkKick {
            //         head,
            //         kick,
            //         kicking_side,
            //     } => todo!(),
            //     _ => {}
            // }
            robot_frames.push(database);
        }
        frames.push(robot_frames);
    }

    keep_running.cancel();
    {
        let parameters_changed = communication_server.get_parameters_changed();
        let parameters_reader = communication_server.get_parameters_reader();
        runtime.spawn(async {
            timeline_server(
                keep_running,
                parameters_reader,
                parameters_changed,
                outputs_writer,
                outputs_changed,
                frames,
            )
            .await
        });
    }

    let mut encountered_error = false;
    match communication_server.join() {
        Ok(Err(error)) => {
            encountered_error = true;
            println!("{error:?}");
        }
        Err(error) => {
            encountered_error = true;
            println!("{error:?}");
        }
        _ => {}
    }

    if encountered_error {
        bail!("at least one cycler exited with error");
    }
    Ok(())
}

fn main() -> Result<()> {
    setup_logger(true)?;
    install()?;
    let keep_running = CancellationToken::new();
    {
        let keep_running = keep_running.clone();
        ctrlc::set_handler(move || {
            println!("Cancelling...");
            keep_running.cancel();
        })?;
    }
    run(keep_running)
}

fn cancel_on_error<T, E>(keep_running: &CancellationToken, result: Result<T, E>) -> Result<T, E> {
    if result.is_err() {
        keep_running.cancel();
    }
    result
}
