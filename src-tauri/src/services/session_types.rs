use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct SessionMeta {
    pub id: String,
    pub file_path: String,
    pub start: String,
    pub end: String,
    pub duration_minutes: f64,
    pub mode: String,
    pub message_count: u32,
    pub version: String,
    pub custom_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionDetail {
    pub meta: SessionMeta,
    pub messages: Vec<SessionMessage>,
    pub files_modified: Vec<String>,
    pub tools_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Deserialize)]
pub struct RawEntry {
    #[serde(rename = "type")]
    pub entry_type: Option<String>,
    pub subtype: Option<String>,
    pub timestamp: Option<String>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    pub version: Option<String>,
    pub message: Option<RawMessage>,
    /// Content field for queue-operation entries (headless prompt)
    pub content: Option<String>,
    pub entrypoint: Option<String>,
    #[serde(rename = "durationMs")]
    pub duration_ms: Option<u64>,
    #[serde(rename = "messageCount")]
    pub message_count: Option<u32>,
}

#[derive(Deserialize)]
pub struct RawMessage {
    pub role: Option<String>,
    pub content: Option<serde_json::Value>,
}

pub fn content_to_string(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|b| b.get("text").and_then(|t| t.as_str()).map(String::from))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}
