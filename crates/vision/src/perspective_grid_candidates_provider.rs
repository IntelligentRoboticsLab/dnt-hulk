use context_attribute::context;
use framework::{MainOutput, OptionalInput, Parameter};

pub struct PerspectiveGridCandidatesProvider {}

#[context]
pub struct NewContext {
    pub ball_radius: Parameter<f32, "field_dimensions/ball_radius">,
    pub fallback_radius:
        Parameter<f32, "$cycler_instance/perspective_grid_candidates_provider/fallback_radius">,
    pub minimum_radius:
        Parameter<f32, "$cycler_instance/perspective_grid_candidates_provider/minimum_radius">,
}

#[context]
pub struct CycleContext {
    pub camera_matrix: OptionalInput<CameraMatrix, "camera_matrix?">,
    pub filtered_segments: OptionalInput<FilteredSegments, "filtered_segments?">,
    pub line_data: OptionalInput<LineData, "line_data?">,

    pub ball_radius: Parameter<f32, "field_dimensions/ball_radius">,
    pub fallback_radius:
        Parameter<f32, "$cycler_instance/perspective_grid_candidates_provider/fallback_radius">,
    pub minimum_radius:
        Parameter<f32, "$cycler_instance/perspective_grid_candidates_provider/minimum_radius">,
}

#[context]
#[derive(Default)]
pub struct MainOutputs {
    pub perspective_grid_candidates: MainOutput<PerspectiveGridCandidates>,
}

impl PerspectiveGridCandidatesProvider {
    pub fn new(context: NewContext) -> anyhow::Result<Self> {
        Ok(Self {})
    }

    pub fn cycle(&mut self, context: CycleContext) -> anyhow::Result<MainOutputs> {
        Ok(MainOutputs::default())
    }
}