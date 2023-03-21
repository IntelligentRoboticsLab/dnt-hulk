use communication::{
    Communication, ConnectionStatus, Cycler, CyclerOutput, HierarchyType, OutputHierarchy,
};

use serde_json::Value;
use tokio::runtime::{Builder, Runtime};

use crate::{image_buffer::ImageBuffer, value_buffer::ValueBuffer};

pub struct Nao {
    communication: Communication,
    runtime: Runtime,
}

