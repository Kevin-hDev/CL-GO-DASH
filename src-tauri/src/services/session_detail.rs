use crate::services::session_parser;
use crate::services::session_types::*;
use std::fs;
use std::path::PathBuf;

fn sessions_dir() -> PathBuf {
    let home = dirs::home_dir().expect("cannot resolve home");
    home.join(".claude/projects/-Users-kevinh")
}

pub fn get_detail(session_id: &str) -> Result<SessionDetail, String> {
    let dir = sessions_dir();
    let path = find_file(&dir, session_id)?;
    let metas = session_parser::list_sessions(60, 0)?;
    let meta = metas
        .into_iter()
        .find(|m| m.id.as_str() == session_id)
        .ok_or_else(|| format!("Session {} not found in list", session_id))?;

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut entries: Vec<SessionEntry> = Vec::new();
    let mut messages: Vec<SessionMessage> = Vec::new();
    let mut tools: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();

    for line in content.lines() {
        let raw: RawEntry = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        let etype = raw.entry_type.as_deref().unwrap_or("");
        let ts = raw.timestamp.clone().unwrap_or_default();

        if let Some(ref msg) = raw.message {
            if let Some(serde_json::Value::Array(arr)) = &msg.content {
                parse_content_blocks(arr, etype, &ts, &mut entries, &mut messages, &mut tools);
            } else if let Some(ref val) = msg.content {
                parse_text_entry(val, etype, &ts, msg, &mut entries, &mut messages);
            }
        }

        if etype.contains("file-history-snapshot") {
            extract_files(line, &mut files);
        }
    }

    Ok(SessionDetail { meta, entries, messages, files_modified: files, tools_used: tools })
}

fn parse_content_blocks(
    arr: &[serde_json::Value], etype: &str, ts: &str,
    entries: &mut Vec<SessionEntry>, messages: &mut Vec<SessionMessage>,
    tools: &mut Vec<String>,
) {
    // Collect text blocks into a message
    let text_parts: Vec<String> = arr.iter()
        .filter_map(|b| b.get("text").and_then(|t| t.as_str()).map(String::from))
        .collect();
    let text = text_parts.join("\n");

    if !text.is_empty() && !text.contains("<system-reminder>") {
        let truncated = truncate_text(&text);
        let msg = SessionMessage {
            role: etype.to_string(),
            content: truncated,
            timestamp: ts.to_string(),
        };
        entries.push(SessionEntry::Message(msg.clone()));
        messages.push(msg);
    }

    // Extract tool_use blocks
    for block in arr {
        if block.get("type").and_then(|t| t.as_str()) != Some("tool_use") { continue; }
        let name = block.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
        let input = block.get("input").cloned().unwrap_or(serde_json::Value::Null);
        let summary = tool_summary(name, &input);

        let (old_text, new_text) = if name == "Edit" {
            (
                input.get("old_string").and_then(|v| v.as_str()).map(String::from),
                input.get("new_string").and_then(|v| v.as_str()).map(String::from),
            )
        } else {
            (None, None)
        };

        entries.push(SessionEntry::Tool(ToolCall {
            name: name.to_string(),
            summary,
            timestamp: ts.to_string(),
            old_text,
            new_text,
        }));

        let s = name.to_string();
        if !tools.contains(&s) { tools.push(s); }
    }
}

fn parse_text_entry(
    val: &serde_json::Value, etype: &str, ts: &str,
    msg: &RawMessage,
    entries: &mut Vec<SessionEntry>, messages: &mut Vec<SessionMessage>,
) {
    if !matches!(etype, "user" | "assistant") { return; }
    let text = content_to_string(val);
    if text.is_empty() || text.contains("<system-reminder>") { return; }

    let truncated = truncate_text(&text);
    let m = SessionMessage {
        role: msg.role.as_deref().unwrap_or(etype).to_string(),
        content: truncated,
        timestamp: ts.to_string(),
    };
    entries.push(SessionEntry::Message(m.clone()));
    messages.push(m);
}

fn truncate_text(text: &str) -> String {
    if text.len() > 2000 {
        let end = text.char_indices()
            .take_while(|(i, _)| *i < 2000)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(2000.min(text.len()));
        format!("{}…", &text[..end])
    } else {
        text.to_string()
    }
}

fn extract_files(line: &str, files: &mut Vec<String>) {
    if let Ok(raw) = serde_json::from_str::<serde_json::Value>(line) {
        if let Some(backups) = raw
            .get("snapshot")
            .and_then(|s| s.get("trackedFileBackups"))
            .and_then(|b| b.as_object())
        {
            for key in backups.keys() {
                if !files.contains(key) { files.push(key.clone()); }
            }
        }
    }
}

fn find_file(dir: &PathBuf, session_id: &str) -> Result<PathBuf, String> {
    let direct = dir.join(format!("{}.jsonl", session_id));
    if direct.exists() { return Ok(direct); }

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_some_and(|e| e == "jsonl") {
            if let Ok(line) = read_first_line(&path) {
                if line.contains(session_id) { return Ok(path); }
            }
        }
    }
    Err(format!("Session {} not found", session_id))
}

fn read_first_line(path: &PathBuf) -> Result<String, String> {
    use std::io::{BufRead, BufReader};
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    Ok(line)
}
