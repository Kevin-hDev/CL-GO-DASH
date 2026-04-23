use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ClgoConfig {
    pub scheduled_wakeups: Vec<ScheduledWakeup>,
    pub heartbeat: HeartbeatConfig,
    pub advanced: AdvancedSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AdvancedSettings {
    pub autostart: bool,
    pub start_hidden: bool,
    pub show_tray: bool,
    pub default_model: String,
    pub keep_alive: String,
    pub allowed_paths: Vec<String>,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            autostart: false,
            start_hidden: false,
            show_tray: true,
            default_model: String::new(),
            keep_alive: "5m".to_string(),
            allowed_paths: default_allowed_paths(),
        }
    }
}

fn default_allowed_paths() -> Vec<String> {
    #[cfg(target_os = "windows")]
    { vec!["C:\\".to_string()] }
    #[cfg(not(target_os = "windows"))]
    { vec!["/".to_string()] }
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
