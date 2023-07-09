use std::time::{Duration, SystemTime};

use color_eyre::Result;
use context_attribute::context;
use framework::{AdditionalOutput, MainOutput, PerceptionInput};
use nalgebra::{point, Point2};
use spl_network_messages::{GamePhase, GameState, SubState, Team, GameControllerReturnMessage, HulkMessage, Penalty, PlayerNumber};
use types::messages::IncomingMessage;
use types::{
    configuration::{Behavior as BehaviorConfiguration, InWalkKicks, LostBall},
    Action, CycleTime, FieldDimensions, FilteredGameState, FilteredWhistle, GameControllerState,
    MotionCommand, PathObstacle, PrimaryState, Role, Side, WorldState,
};

use super::{
    calibrate,
    defend::Defend,
    detect_ref_signal, dribble, fall_safely,
    head::LookAction,
    initial, jump, look_around, lost_ball, penalize, prepare_jump, search, sit_down, stand,
    stand_up, support, unstiff, walk_to_kick_off, walk_to_penalty_kick,
    walk_to_pose::{WalkAndStand, WalkPathPlanner},
};

pub struct Behavior {
    last_motion_command: MotionCommand,
    absolute_last_known_ball_position: Point2<f32>,
    active_since: Option<SystemTime>,
    active_since_visref: Option<SystemTime>,
    last_team_score: u8,
    last_opponent_score: u8,
}

#[context]
pub struct CreationContext {
    pub behavior: Parameter<BehaviorConfiguration, "behavior">,
    pub field_dimensions: Parameter<FieldDimensions, "field_dimensions">,
    pub lost_ball_parameters: Parameter<LostBall, "behavior.lost_ball">,
}

#[context]
pub struct CycleContext {
    pub path_obstacles: AdditionalOutput<Vec<PathObstacle>, "path_obstacles">,
    pub active_action: AdditionalOutput<Action, "active_action">,

    pub has_ground_contact: Input<bool, "has_ground_contact">,
    pub world_state: Input<WorldState, "world_state">,
    pub cycle_time: Input<CycleTime, "cycle_time">,
    pub filtered_whistle: Input<FilteredWhistle, "filtered_whistle">,

    pub configuration: Parameter<BehaviorConfiguration, "behavior">,
    pub in_walk_kicks: Parameter<InWalkKicks, "in_walk_kicks">,
    pub field_dimensions: Parameter<FieldDimensions, "field_dimensions">,
    pub lost_ball_parameters: Parameter<LostBall, "behavior.lost_ball">,
    pub network_message: PerceptionInput<IncomingMessage, "SplNetwork", "message">,

}

#[context]
#[derive(Default)]
pub struct MainOutputs {
    pub motion_command: MainOutput<MotionCommand>,
}

pub enum VisRefFieldSide {
    Left,
    Right,            
}

impl Behavior {
    pub fn new(_context: CreationContext) -> Result<Self> {
        Ok(Self {
            last_motion_command: MotionCommand::Unstiff,
            absolute_last_known_ball_position: point![0.0, 0.0],
            active_since: None,
            active_since_visref: None,
            last_team_score: 0,
            last_opponent_score: 0,
        })
    }

    pub fn cycle(&mut self, mut context: CycleContext) -> Result<MainOutputs> {       
        let world_state = context.world_state;

        if let Some(command) = &context.configuration.injected_motion_command {
            return Ok(MainOutputs {
                motion_command: command.clone().into(),
            });
        }

        if let Some(ball_state) = &world_state.ball {
            self.absolute_last_known_ball_position = ball_state.ball_in_field;
        }

        let now = context.cycle_time.start_time;
        match (self.active_since, world_state.robot.primary_state) {
            (None, PrimaryState::Ready { .. } | PrimaryState::Playing { .. }) => {
                self.active_since = Some(now)
            }
            (None, _) => {}
            (
                Some(_),
                PrimaryState::Ready { .. } | PrimaryState::Set | PrimaryState::Playing { .. },
            ) => {}
            (Some(_), _) => self.active_since = None,
        }

        let mut actions = vec![
            Action::Unstiff,
            Action::SitDown,
            Action::Penalize,
            Action::Initial,
            Action::FallSafely,
            Action::StandUp,
            Action::Stand,
            Action::Calibrate,
        ];


        let mut spl_messages = context
            .network_message
            .persistent
            .values()
            .flatten()
            .filter_map(|message| match message {
                IncomingMessage::GameController(message) => Some(message),
                IncomingMessage::Spl(_) => None,
            })
            .peekable();
        
        let current_spl_message  = spl_messages.peek();

        match self.active_since_visref {
            // Visref was not active
            None => {
                // Active on whistle or if a goal is scored (based on gamecontroller data)
                if context.filtered_whistle.is_detected
                || (current_spl_message.is_some() &&  ((self.last_team_score != current_spl_message.as_ref().unwrap().hulks_team.score) || (self.last_opponent_score != current_spl_message.as_ref().unwrap().opponent_team.score))){
                    self.active_since_visref = Some(now);
                    // Set visref on active and push the action in the same cycle
                    actions.push(Action::DetectRefSignal);
                }current_spl_message.unwrap
            },
            // Visref was already active
            Some(time) => {
                // If visref active after 15 seconds set to inactive
                if now.duration_since(self.active_since_visref.unwrap())? > Duration::from_secs(15) {
                    self.active_since_visref = None;
                }
                // Visref started less than 15 seconds ago push the action
                else {
                    actions.push(Action::DetectRefSignal);
                }

            },
            _ => {}
        }

        if current_spl_message.is_some() {
            self.last_team_score = current_spl_message.as_ref().unwrap().hulks_team.score;
            self.last_opponent_score = current_spl_message.as_ref().unwrap().opponent_team.score;
        }

        if let Some(active_since) = self.active_since {
            if now.duration_since(active_since)? < context.configuration.initial_lookaround_duration
            {
                actions.push(Action::LookAround);
            }
        }

        match world_state.robot.role {
            Role::DefenderLeft => actions.push(Action::DefendLeft),
            Role::DefenderRight => actions.push(Action::DefendRight),
            Role::Keeper => match world_state.game_controller_state {
                Some(GameControllerState {
                    game_phase: GamePhase::PenaltyShootout { .. },
                    ..
                }) => {
                    actions.push(Action::Jump);
                    actions.push(Action::PrepareJump);
                }
                _ => actions.push(Action::DefendGoal),
            },
            Role::Loser => actions.push(Action::SearchForLostBall),
            Role::MidfielderLeft => actions.push(Action::SupportLeft),
            Role::MidfielderRight => actions.push(Action::SupportRight),
            Role::ReplacementKeeper => actions.push(Action::DefendGoal),
            Role::Searcher => actions.push(Action::Search),
            Role::Striker => match world_state.filtered_game_state {
                None | Some(FilteredGameState::Playing { ball_is_free: true }) => {
                    actions.push(Action::Dribble);
                }
                Some(FilteredGameState::Ready {
                    kicking_team: Team::Hulks,
                }) => match world_state.game_controller_state {
                    Some(GameControllerState {
                        sub_state: Some(SubState::PenaltyKick),
                        ..
                    }) => actions.push(Action::WalkToPenaltyKick),
                    _ => actions.push(Action::WalkToKickOff),
                },
                _ => match world_state.game_controller_state {
                    Some(GameControllerState {
                        game_state: GameState::Ready,
                        sub_state: Some(SubState::PenaltyKick),
                        kicking_team: Team::Opponent,
                        ..
                    }) => actions.push(Action::DefendPenaltyKick),
                    _ => actions.push(Action::DefendKickOff),
                },
            },
            Role::StrikerSupporter => actions.push(Action::SupportStriker),
        };

        let walk_path_planner = WalkPathPlanner::new(
            context.field_dimensions,
            &world_state.obstacles,
            &context.configuration.path_planning,
        );
        let walk_and_stand = WalkAndStand::new(
            world_state,
            &context.configuration.walk_and_stand,
            &walk_path_planner,
            &self.last_motion_command,
        );
        let look_action = LookAction::new(world_state);
        let defend = Defend::new(
            world_state,
            context.field_dimensions,
            &context.configuration.role_positions,
            &walk_and_stand,
            &look_action,
        );

        let (action, motion_command) = actions
            .iter()
            .find_map(|action| {
                let motion_command = match action {
                    Action::Unstiff => unstiff::execute(world_state),
                    Action::SitDown => sit_down::execute(world_state),
                    Action::Penalize => penalize::execute(world_state),
                    Action::Initial => initial::execute(world_state),
                    Action::FallSafely => {
                        fall_safely::execute(world_state, *context.has_ground_contact)
                    }
                    Action::StandUp => stand_up::execute(world_state),
                    Action::DetectRefSignal => {
                        detect_ref_signal::execute(world_state, context.field_dimensions, VisRefFieldSide::Right)
                    },
                    Action::Stand => stand::execute(world_state, context.field_dimensions),
                    Action::LookAround => look_around::execute(world_state),
                    Action::Calibrate => calibrate::execute(world_state),
                    Action::DefendGoal => defend.goal(&mut context.path_obstacles),
                    Action::DefendKickOff => defend.kick_off(&mut context.path_obstacles),
                    Action::DefendLeft => defend.left(&mut context.path_obstacles),
                    Action::DefendRight => defend.right(&mut context.path_obstacles),
                    Action::DefendPenaltyKick => defend.penalty_kick(&mut context.path_obstacles),
                    Action::Dribble => dribble::execute(
                        world_state,
                        &walk_path_planner,
                        context.in_walk_kicks,
                        &context.configuration.dribbling,
                        &mut context.path_obstacles,
                    ),
                    Action::Jump => jump::execute(world_state),
                    Action::PrepareJump => prepare_jump::execute(world_state),
                    Action::Search => search::execute(
                        world_state,
                        &walk_path_planner,
                        &walk_and_stand,
                        context.field_dimensions,
                        &context.configuration.search,
                        &mut context.path_obstacles,
                    ),
                    Action::SearchForLostBall => lost_ball::execute(
                        world_state,
                        self.absolute_last_known_ball_position,
                        &walk_path_planner,
                        context.lost_ball_parameters,
                        &mut context.path_obstacles,
                    ),
                    Action::SupportLeft => support::execute(
                        world_state,
                        context.field_dimensions,
                        Some(Side::Left),
                        context
                            .configuration
                            .role_positions
                            .left_midfielder_distance_to_ball,
                        context
                            .configuration
                            .role_positions
                            .left_midfielder_maximum_x_in_ready_and_when_ball_is_not_free,
                        context
                            .configuration
                            .role_positions
                            .left_midfielder_minimum_x,
                        &walk_and_stand,
                        &look_action,
                        &mut context.path_obstacles,
                    ),
                    Action::SupportRight => support::execute(
                        world_state,
                        context.field_dimensions,
                        Some(Side::Right),
                        context
                            .configuration
                            .role_positions
                            .right_midfielder_distance_to_ball,
                        context
                            .configuration
                            .role_positions
                            .right_midfielder_maximum_x_in_ready_and_when_ball_is_not_free,
                        context
                            .configuration
                            .role_positions
                            .right_midfielder_minimum_x,
                        &walk_and_stand,
                        &look_action,
                        &mut context.path_obstacles,
                    ),
                    Action::SupportStriker => support::execute(
                        world_state,
                        context.field_dimensions,
                        None,
                        context
                            .configuration
                            .role_positions
                            .striker_supporter_distance_to_ball,
                        context
                            .configuration
                            .role_positions
                            .striker_supporter_maximum_x_in_ready_and_when_ball_is_not_free,
                        context
                            .configuration
                            .role_positions
                            .striker_supporter_minimum_x,
                        &walk_and_stand,
                        &look_action,
                        &mut context.path_obstacles,
                    ),
                    Action::WalkToKickOff => walk_to_kick_off::execute(
                        world_state,
                        &walk_and_stand,
                        &look_action,
                        &mut context.path_obstacles,
                    ),
                    Action::WalkToPenaltyKick => walk_to_penalty_kick::execute(
                        world_state,
                        &walk_and_stand,
                        &look_action,
                        &mut context.path_obstacles,
                        context.field_dimensions,
                    ),
                }?;
                Some((action, motion_command))
            })
            .unwrap_or_else(|| {
                panic!(
                    "there has to be at least one action available, world_state: {world_state:#?}",
                )
            });
        context.active_action.fill_if_subscribed(|| *action);

        self.last_motion_command = motion_command.clone();

        Ok(MainOutputs {
            motion_command: motion_command.into(),
        })
    }
}
