use serde::{Deserialize, Serialize};

use crate::model::{Phase, Settings, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    pub phase: Phase,
    pub remaining_secs: u32,
    pub running: bool,
    /// 1..=rounds_per_cycle, shown in the design as 第 {round}/4 轮.
    pub round: u8,
    pub active_task: Option<TaskId>,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            phase: Phase::Focus,
            remaining_secs: Settings::default().focus_secs,
            running: false,
            round: 1,
            active_task: None,
        }
    }
}
