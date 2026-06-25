use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStreamFailure {
    pub code: String,
    pub occurred_at: DateTime<Utc>,
    pub is_connection: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_todo_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_todo_title: Option<String>,
}
