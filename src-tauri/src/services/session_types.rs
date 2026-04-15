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
    pub entries: Vec<SessionEntry>,
    pub messages: Vec<SessionMessage>,
    pub files_modified: Vec<String>,
    pub tools_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum SessionEntry {
    #[serde(rename = "message")]
    Message(SessionMessage),
    #[serde(rename = "tool")]
    Tool(ToolCall),
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolCall {
    pub name: String,
    pub summary: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_text: Option<String>,
}

#[derive(Deserialize)]
pub struct RawEntry {
    #[serde(rename = "type")]
    pub entry_type: Option<String>,
    #[allow(dead_code)]
    pub subtype: Option<String>,
    pub timestamp: Option<String>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    pub version: Option<String>,
    pub message: Option<RawMessage>,
    pub content: Option<String>,
    #[allow(dead_code)]
    pub entrypoint: Option<String>,
    #[allow(dead_code)]
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

pub fn tool_summary(name: &str, input: &serde_json::Value) -> String {
    let key = match name {
        "Bash" => "command",
        "Read" | "Write" | "Edit" => "file_path",
        "Grep" | "Glob" => "pattern",
        "Skill" => "skill",
        _ => return String::new(),
    };
    let raw = input.get(key).and_then(|v| v.as_str()).unwrap_or("");
    let line = raw.lines().next().unwrap_or("");
    if name == "Read" || name == "Write" || name == "Edit" {
        return shorten_path(line);
    }
    line.to_string()
}

fn shorten_path(s: &str) -> String {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() > 3 {
        format!(".../{}", parts[parts.len() - 3..].join("/"))
    } else {
        s.to_string()
    }
}
