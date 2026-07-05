use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types_stream::TokenPhase;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_activities: Option<Vec<ToolActivityRecord>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<SavedSegment>>,
    pub files: Vec<FileAttachment>,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub tokens: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolActivityRecord {
    pub name: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSegment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    pub tools: Vec<ToolActivityRecord>,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<TokenPhase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub extra_content: Option<serde_json::Value>,
    pub function: ToolCallRequestFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequestFunction {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    pub name: String,
    pub path: String,
    pub mime_type: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{AgentMessage, SavedSegment};
    use crate::services::agent_local::types_stream::TokenPhase;

    #[test]
    fn agent_message_accepts_missing_work_duration() {
        let msg: AgentMessage = serde_json::from_value(serde_json::json!({
            "id": "m1",
            "role": "assistant",
            "content": "ok",
            "files": [],
            "timestamp": "2026-07-01T12:00:00Z",
            "tokens": 0
        }))
        .unwrap();

        assert_eq!(msg.work_duration_ms, None);
    }

    #[test]
    fn agent_message_serializes_work_duration_when_present() {
        let msg: AgentMessage = serde_json::from_value(serde_json::json!({
            "id": "m1",
            "role": "assistant",
            "content": "ok",
            "files": [],
            "timestamp": "2026-07-01T12:00:00Z",
            "tokens": 0,
            "work_duration_ms": 266000
        }))
        .unwrap();
        let saved = serde_json::to_value(msg).unwrap();

        assert_eq!(saved["work_duration_ms"], 266000);
    }

    #[test]
    fn saved_segment_accepts_phase() {
        let segment: SavedSegment = serde_json::from_value(serde_json::json!({
            "tools": [],
            "content": "partial",
            "phase": "work"
        }))
        .unwrap();

        assert!(matches!(segment.phase, Some(TokenPhase::Work)));
    }
}
