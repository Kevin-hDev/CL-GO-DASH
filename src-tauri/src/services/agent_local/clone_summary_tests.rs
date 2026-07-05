use super::*;
use chrono::Utc;
use serde_json::json;

fn message_with_tool(tool: ToolActivityRecord) -> AgentMessage {
    AgentMessage {
        id: "m1".into(),
        role: "assistant".into(),
        content: "done".into(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: Some(vec![tool]),
        segments: None,
        files: vec![],
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
    }
}

#[test]
fn serialize_limits_tool_result() {
    let tool = ToolActivityRecord {
        name: "read_file".into(),
        summary: "read".into(),
        args: Some(json!({"path": "src/main.rs"})),
        result: Some("a".repeat(MAX_TOOL_RESULT_CHARS + 50)),
        is_error: None,
        content: None,
        old_text: None,
        new_text: None,
        start_line: None,
        affected_paths: vec![],
    };
    let serialized = serialize_messages(&[message_with_tool(tool)]);
    assert!(serialized.len() < MAX_TOOL_RESULT_CHARS + 500);
}

#[test]
fn extract_files_uses_tool_traces() {
    let tool = ToolActivityRecord {
        name: "edit_file".into(),
        summary: "edit".into(),
        args: Some(json!({"path": "src/lib.rs"})),
        result: None,
        is_error: None,
        content: None,
        old_text: None,
        new_text: None,
        start_line: None,
        affected_paths: vec!["src/lib.rs".into()],
    };
    let (read, modified) = extract_traced_files(&[message_with_tool(tool)]);
    assert!(read.is_empty());
    assert_eq!(modified, vec!["src/lib.rs"]);
}

#[test]
fn serialize_adds_truncated_marker_at_limit() {
    let mut msg = message_with_tool(ToolActivityRecord {
        name: "read_file".into(),
        summary: "read".into(),
        args: Some(json!({"path": "src/main.rs"})),
        result: Some("ok".into()),
        is_error: None,
        content: None,
        old_text: None,
        new_text: None,
        start_line: None,
        affected_paths: vec![],
    });
    msg.content = "a".repeat(MAX_SUMMARY_INPUT_CHARS);
    let serialized = serialize_messages(&[msg]);
    assert!(serialized.ends_with("[Truncated]"));
    assert!(serialized.chars().count() <= MAX_SUMMARY_INPUT_CHARS);
}
