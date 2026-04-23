//! Types communs du module LLM multi-provider.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "system" | "user" | "assistant" | "tool"
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String, // toujours "function" pour OpenAI-compat
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String, // JSON stringifié (format OpenAI)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String, // "function"
    pub function: ToolFunctionDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON schema
}

#[derive(Debug, Clone, Default)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<ToolDefinition>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owned_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u32>,
    #[serde(default)]
    pub supports_tools: bool,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default)]
    pub supports_thinking: bool,
    #[serde(default)]
    pub is_free: bool,
}

/// Erreurs du module LLM. Volontairement basées sur `String` pour cohérence
/// avec le reste du projet (les commandes Tauri retournent `Result<_, String>`).
#[derive(Debug, Clone)]
pub enum LlmError {
    Unauthorized,
    RateLimit { retry_after_secs: Option<u64> },
    Http { status: u16, message: String },
    Network(String),
    Parse(String),
    Provider(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::Unauthorized => write!(f, "clé API invalide ou non autorisée"),
            LlmError::RateLimit { retry_after_secs } => match retry_after_secs {
                Some(s) => write!(f, "rate limit atteint, réessaie dans {}s", s),
                None => write!(f, "rate limit atteint, réessaie plus tard"),
            },
            LlmError::Http { status, message } => write!(f, "HTTP {}: {}", status, message),
            LlmError::Network(m) => write!(f, "erreur réseau : {}", m),
            LlmError::Parse(m) => write!(f, "erreur de parsing : {}", m),
            LlmError::Provider(m) => write!(f, "erreur provider : {}", m),
        }
    }
}

impl From<LlmError> for String {
    fn from(e: LlmError) -> Self {
        e.to_string()
    }
}

impl LlmError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            LlmError::RateLimit { .. } | LlmError::Http { status: 502..=504, .. }
        )
    }
}
