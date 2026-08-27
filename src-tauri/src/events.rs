use serde::Serialize;

use crate::core::desk::PetMood;
use crate::core::reminder::Intensity;
use crate::model::Phase;

pub const TICK: &str = "timer:tick";
pub const PHASE: &str = "timer:phase";
pub const CHANGED: &str = "model:changed";
pub const REMINDER_FIRE: &str = "reminder:fire";
pub const PET_STATE: &str = "pet:state";

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
    Reminders,
    Body,
}

impl Section {
    pub fn as_str(self) -> &'static str {
        match self {
            Section::Tasks => "tasks",
            Section::Settings => "settings",
            Section::Timer => "timer",
            Section::Reminders => "reminders",
            Section::Body => "body",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedPayload {
    pub section: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirePayload {
    pub id: u32,
    pub name: String,
    pub message: String,
    pub intensity: Intensity,
    pub color: String,
    /// Only meaningful for `Fullscreen`; the overlay hides its exits.
    pub must_complete: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetStatePayload {
    pub state: PetMood,
}
