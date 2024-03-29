use std::{fmt::Debug, time::Duration};

use crate::Condition;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use types::ConditionInput;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilizedCondition {
    tolerance: f32,
    #[serde(
        serialize_with = "serialize_float_seconds",
        deserialize_with = "deserialize_float_seconds"
    )]
    timeout_duration: Duration,
}

impl Condition for StabilizedCondition {
    fn is_fulfilled(&self, condition_input: &ConditionInput, time_since_start: Duration) -> bool {
        condition_input.filtered_angular_velocity.norm() < self.tolerance
            || time_since_start > self.timeout_duration
    }
}

fn serialize_float_seconds<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f32(duration.as_secs_f32())
}

fn deserialize_float_seconds<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Duration::from_secs_f32(f32::deserialize(deserializer)?))
}
