use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClgoConfig {
    pub version: String,
    pub projects_root: String,
    pub claude_projects: String,
    pub heartbeat: HeartbeatConfig,
    pub communication: CommunicationConfig,
    #[serde(default)]
    pub hooks: HooksConfig,
    #[serde(default)]
    pub scheduled_wakeups: Vec<ScheduledWakeup>,
    #[serde(default)]
    pub scheduled_tasks: Vec<serde_json::Value>,
    #[serde(default)]
    pub rube_usage: Option<RubeUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    pub active: bool,
    pub mode: String,
    pub stop_at: Option<String>,
    pub interval_minutes: u32,
    #[serde(default)]
    pub started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationConfig {
    pub provider: String,
    pub chat_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HooksConfig {
    #[serde(default)]
    pub post_explorer: Vec<HookEntry>,
    #[serde(default)]
    pub post_auto: Vec<HookEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEntry {
    pub name: String,
    pub command: Vec<String>,
    #[serde(default)]
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledWakeup {
    pub id: String,
    pub time: String,
    pub mode: String,
    #[serde(default)]
    pub prompt: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubeUsage {
    pub month: Option<String>,
    pub count: u32,
    pub limit: u32,
}
