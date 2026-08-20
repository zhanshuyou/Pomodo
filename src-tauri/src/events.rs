use serde::Serialize;

use crate::model::Phase;

pub const TICK: &str = "timer:tick";
pub const PHASE: &str = "timer:phase";
pub const CHANGED: &str = "model:changed";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TickPayload {
    pub remaining_secs: u32,
    pub phase: Phase,
    pub running: bool,
    pub round: u8,
    pub belly_cells: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum Section {
    Tasks,
    Settings,
    Timer,
}

impl Section {
    pub fn as_str(self) -> &'static str {
        match self {
            Section::Tasks => "tasks",
            Section::Settings => "settings",
            Section::Timer => "timer",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedPayload {
    pub section: &'static str,
}
