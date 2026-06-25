use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum StreamEvent {
    #[serde(rename_all = "camelCase")]
    Token {
        content: String,
        token_count: u32,
        tps: f64,
    },
    Thinking {
        content: String,
    },
    ToolCall {
        name: String,
        arguments: serde_json::Value,
    },
    #[serde(rename_all = "camelCase")]
    ToolResult {
        name: String,
        content: String,
        is_error: bool,
        #[serde(default)]
        truncated: bool,
        tool_call_index: usize,
    },
    TurnEnd {},
    #[serde(rename_all = "camelCase")]
    PermissionRequest {
        id: String,
        tool_name: String,
        arguments: serde_json::Value,
    },
    #[serde(rename_all = "camelCase")]
    Done {
        eval_count: u32,
        eval_duration_ns: u64,
        final_tps: f64,
        prompt_tokens: u32,
        context_tokens: u32,
    },
    #[serde(rename_all = "camelCase")]
    Error {
        message: String,
        #[serde(default)]
        is_connection: bool,
    },
    #[serde(rename_all = "camelCase")]
    Notice {
        message_key: String,
    },
    Compressing {
        status: String,
    },
    CompressionComplete {},
    #[serde(rename_all = "camelCase")]
    SessionSnapshot {
        messages: Vec<crate::services::agent_local::types_session::AgentMessage>,
        token_count: u32,
    },
    #[serde(rename_all = "camelCase")]
    SubagentSpawned {
        subagent_session_id: String,
        subagent_name: String,
        subagent_type: String,
        prompt_preview: String,
        run_id: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    SubagentCompleted {
        subagent_session_id: String,
        success: bool,
        status: String,
        summary: String,
        all_done: bool,
        run_id: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    TodoUpdated {
        todos: Vec<crate::services::agent_local::types_todo::AgentTodoItem>,
    },
}

#[derive(Debug, Default)]
pub struct StreamResult {
    pub content: String,
    pub thinking: String,
    pub tool_calls: Vec<(String, serde_json::Value)>,
    /// IDs OpenAI-compat alignés avec `tool_calls` (vide pour Ollama).
    pub tool_call_ids: Vec<String>,
    pub tool_call_extra_content: Vec<Option<serde_json::Value>>,
    pub eval_count: u32,
    pub prompt_tokens: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullProgress {
    pub status: String,
    pub completed: Option<u64>,
    pub total: Option<u64>,
}
