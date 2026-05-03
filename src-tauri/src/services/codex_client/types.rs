use serde::Serialize;

pub const CODEX_API_BASE: &str = "https://chatgpt.com/backend-api/codex";

pub const CODEX_MODELS: &[(&str, u32)] = &[
    ("gpt-5.5", 258_000),
    ("gpt-5.4", 258_000),
    ("gpt-5.4-mini", 258_000),
    ("gpt-5.4-pro", 258_000),
];

pub const CODEX_EFFORT_LEVELS: &[&str] = &["low", "medium", "high", "xhigh"];

#[derive(Serialize)]
pub struct CodexRequest {
    pub model: String,
    pub instructions: String,
    pub input: Vec<serde_json::Value>,
    pub stream: bool,
    pub store: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

#[derive(Serialize, Clone)]
pub struct ReasoningConfig {
    pub effort: String,
    pub summary: String,
}
