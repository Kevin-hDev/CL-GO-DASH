use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types_diagnostics::{AgentDiagnosticRun, AgentStreamFailure};
#[allow(unused_imports)]
pub use super::types_message::{
    AgentMessage, FileAttachment, SavedSegment, ToolActivityRecord, ToolCallRequest,
    ToolCallRequestFunction,
};
use super::types_plan::{AgentPlanApprovalDecision, AgentPlanRun, AgentPlanWorkflowStatus};
use super::types_todo::{AgentTodoItem, AgentTodoRun};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloneMode {
    Cut,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<DateTime<Utc>>,
    pub model: String,
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub thinking_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_mode: Option<String>,
    pub accumulated_tokens: u32,
    pub messages: Vec<AgentMessage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub todos: Vec<AgentTodoItem>,
    #[serde(default)]
    pub todo_neglect_count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub todo_runs: Vec<AgentTodoRun>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_todo_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stream_failures: Vec<AgentStreamFailure>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostic_runs: Vec<AgentDiagnosticRun>,
    #[serde(default)]
    pub plan_mode_enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plan_runs: Vec<AgentPlanRun>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_plan_id: Option<String>,
    #[serde(default)]
    pub plan_workflow_status: AgentPlanWorkflowStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_approval_decision: Option<AgentPlanApprovalDecision>,
    #[serde(default)]
    pub is_heartbeat: bool,
    #[serde(default)]
    pub is_gateway: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway_channel_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default)]
    pub working_dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_worktree: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_parent_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_parent_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_mode: Option<CloneMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clone_read_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clone_modified_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionMeta {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<DateTime<Utc>>,
    pub model: String,
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub thinking_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_mode: Option<String>,
    pub message_count: usize,
    #[serde(default)]
    pub is_heartbeat: bool,
    #[serde(default)]
    pub is_gateway: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway_channel_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_parent_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_parent_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_mode: Option<CloneMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
}

fn default_provider() -> String {
    "ollama".to_string()
}
