use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ClgoConfig {
    pub scheduled_wakeups: Vec<ScheduledWakeup>,
    pub heartbeat: HeartbeatConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HeartbeatConfig {
    pub global_paused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledWakeup {
    pub id: String,
    pub name: String,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub schedule: WakeupSchedule,
    #[serde(default)]
    pub description: String,
    pub active: bool,
    #[serde(default)]
    pub paused_by_global: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum WakeupSchedule {
    Once { datetime: String },
    Daily { time: String },
    Weekly { weekday: u8, time: String },
}
