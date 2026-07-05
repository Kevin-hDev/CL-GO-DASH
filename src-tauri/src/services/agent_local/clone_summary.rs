use super::types_message::{AgentMessage, ToolActivityRecord};
use super::types_ollama::ChatMessage;

pub const CLONE_SUMMARY_PREFIX: &str = "This cloned session includes hidden branch context:";
const MAX_SUMMARY_INPUT_CHARS: usize = 120_000;
const MAX_TOOL_RESULT_CHARS: usize = 2_000;
const MAX_TRACKED_FILES: usize = 200;

pub fn hidden_context_content(summary: &str) -> String {
    format!("{CLONE_SUMMARY_PREFIX}\n\n{}", summary.trim())
}

pub fn build_summary_messages(serialized: &str, focus: Option<&str>) -> Vec<ChatMessage> {
    let focus_block = focus
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("\n\nFocus demandé:\n{value}"))
        .unwrap_or_default();
    vec![
        ChatMessage {
            role: "system".to_string(),
            content: "Summarize hidden context for a cloned coding session. Be concise, factual, and preserve decisions, files, commands, errors, and unresolved work. Do not invent file paths.".to_string(),
            ..Default::default()
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!(
                "The user cloned the conversation from an earlier message. Summarize only the conversation that happened after that point so the clone can continue with useful hidden context.{focus_block}\n\nConversation after the clone point:\n{serialized}"
            ),
            ..Default::default()
        },
    ]
}

pub fn serialize_messages(messages: &[AgentMessage]) -> String {
    let mut out = String::new();
    for msg in messages {
        push_bounded(&mut out, &format!("\n<message role=\"{}\">\n", msg.role));
        push_bounded(&mut out, msg.content.trim());
        if let Some(thinking) = msg.thinking.as_deref().filter(|value| !value.trim().is_empty()) {
            push_bounded(&mut out, "\n<thinking>\n");
            push_bounded(&mut out, thinking.trim());
            push_bounded(&mut out, "\n</thinking>");
        }
        for tool in message_tools(msg) {
            serialize_tool(&mut out, tool);
        }
        push_bounded(&mut out, "\n</message>\n");
        if out.chars().count() >= MAX_SUMMARY_INPUT_CHARS {
            break;
        }
    }
    out
}

pub fn extract_traced_files(messages: &[AgentMessage]) -> (Vec<String>, Vec<String>) {
    let mut read_files = Vec::new();
    let mut modified_files = Vec::new();
    for msg in messages {
        for tool in message_tools(msg) {
            collect_tool_files(tool, &mut read_files, &mut modified_files);
        }
    }
    (read_files, modified_files)
}

fn serialize_tool(out: &mut String, tool: &ToolActivityRecord) {
    push_bounded(out, &format!("\n<tool name=\"{}\">\n", tool.name));
    if let Some(args) = &tool.args {
        push_bounded(out, "args: ");
        push_bounded(out, &args.to_string());
        push_bounded(out, "\n");
    }
    if !tool.summary.trim().is_empty() {
        push_bounded(out, "summary: ");
        push_bounded(out, tool.summary.trim());
        push_bounded(out, "\n");
    }
    if let Some(result) = tool.result.as_deref().or(tool.content.as_deref()) {
        push_bounded(out, "result: ");
        push_bounded(out, &take_chars(result, MAX_TOOL_RESULT_CHARS));
        push_bounded(out, "\n");
    }
    push_bounded(out, "</tool>");
}

fn collect_tool_files(
    tool: &ToolActivityRecord,
    read_files: &mut Vec<String>,
    modified_files: &mut Vec<String>,
) {
    let is_read = matches!(
        tool.name.as_str(),
        "read_file" | "grep" | "glob" | "list_dir" | "read_document" | "read_image" | "read_spreadsheet"
    );
    let is_write = matches!(
        tool.name.as_str(),
        "write_file" | "edit_file" | "process_image" | "write_document" | "write_spreadsheet" | "bash"
    );
    if is_read {
        add_arg_path(tool, read_files);
    }
    if is_write || !tool.affected_paths.is_empty() {
        add_arg_path(tool, modified_files);
        for path in &tool.affected_paths {
            add_path(modified_files, path);
        }
    }
}

fn add_arg_path(tool: &ToolActivityRecord, files: &mut Vec<String>) {
    let Some(args) = &tool.args else { return; };
    for key in ["path", "file", "output_path"] {
        if let Some(path) = args.get(key).and_then(|value| value.as_str()) {
            add_path(files, path);
        }
    }
}

fn add_path(files: &mut Vec<String>, path: &str) {
    let trimmed = path.trim();
    if trimmed.is_empty()
        || trimmed.contains('\0')
        || trimmed.chars().count() > 4096
        || files.len() >= MAX_TRACKED_FILES
        || files.iter().any(|seen| seen == trimmed)
    {
        return;
    }
    files.push(trimmed.to_string());
}

fn message_tools(message: &AgentMessage) -> Vec<&ToolActivityRecord> {
    if let Some(segments) = &message.segments {
        let tools: Vec<_> = segments.iter().flat_map(|segment| segment.tools.iter()).collect();
        if !tools.is_empty() {
            return tools;
        }
    }
    message
        .tool_activities
        .as_deref()
        .unwrap_or_default()
        .iter()
        .collect()
}

fn push_bounded(out: &mut String, value: &str) {
    let remaining = MAX_SUMMARY_INPUT_CHARS.saturating_sub(out.chars().count());
    if remaining == 0 {
        return;
    }
    out.push_str(&take_chars(value, remaining));
}

fn take_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
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
}
