use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub family: String,
    pub parameter_size: String,
    pub quantization: String,
    pub architecture: String,
    pub is_moe: bool,
    pub context_length: u64,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub modelfile: String,
    pub parameters: String,
    pub template: String,
    pub family: String,
    pub parameter_size: String,
    pub quantization: String,
    pub architecture: String,
    pub is_moe: bool,
    pub context_length: u64,
    pub capabilities: Vec<String>,
    pub has_audio: bool,
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub name: String,
    pub parameter_size: String,
    pub file_size: u64,
    pub architecture: String,
    pub context_length: u64,
    pub family: String,
    pub quantization: String,
    pub capabilities: Vec<String>,
    pub is_moe: bool,
    pub has_audio: bool,
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryModel {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub is_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ChatOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallOllama>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallOllama {
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum StreamEvent {
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
    ToolResult {
        name: String,
        content: String,
        is_error: bool,
    },
    TurnEnd {},
    Done {
        eval_count: u32,
        eval_duration_ns: u64,
        final_tps: f64,
        prompt_tokens: u32,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Default)]
pub struct StreamResult {
    pub content: String,
    pub thinking: String,
    pub tool_calls: Vec<(String, serde_json::Value)>,
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
