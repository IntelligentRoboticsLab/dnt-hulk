use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use anyhow::Result;

use communication::ConnectionStatus;

use eframe::{
    egui::{
        CentralPanel, Context, Key, Modifiers, TopBottomPanel, Ui, Visuals, Widget, WidgetText,
    },
    epaint::Color32,
    run_native, App, CreationContext, Frame, NativeOptions, Storage,
};
use egui_dock::{DockArea, NodeIndex, TabAddAlign, TabIndex, Tree};
use fern::{colors::ColoredLevelConfig, Dispatch, InitError};

use serde_json::{from_str, to_string, Value};
use tokio::sync::mpsc;



fn main() {

}