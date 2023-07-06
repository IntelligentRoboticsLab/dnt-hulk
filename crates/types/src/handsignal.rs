use serialize_hierarchy::SerializeHierarchy;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Deserialize, Serialize, SerializeHierarchy)]
pub struct HandSignal {
    pub handsignal: u8,
}