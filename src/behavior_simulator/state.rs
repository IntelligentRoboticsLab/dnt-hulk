    pub game_controller_state: GameControllerState,
    pub ball_is_free: bool,
    pub ball_position: Point2<f32>,
    pub ball_velocity: Vector2<f32>,
    pub broadcasted_spl_message_counter: usize,
    pub broadcasted_spl_messages: Vec<SplMessage>,
}

impl TryFrom<SimulationConfiguration> for State {
    type Error = anyhow::Error;

    fn try_from(configuration: SimulationConfiguration) -> anyhow::Result<Self> {
        Ok(Self {
            configuration,
            now: UNIX_EPOCH,
            filtered_game_state: FilteredGameState::Initial,
            game_controller_state: GameControllerState {
                game_state: GameState::Initial,
                game_phase: GamePhase::Normal,
