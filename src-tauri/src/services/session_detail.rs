use crate::services::session_parser;
use crate::services::session_types::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn sessions_dir() -> PathBuf {
    let home = dirs::home_dir().expect("cannot resolve home");
    home.join(".claude/projects/-Users-kevinh")
}

pub fn get_detail(session_id: &str) -> Result<SessionDetail, String> {
    let dir = sessions_dir();
    let path = find_file(&dir, session_id)?;
    let names: HashMap<String, String> = HashMap::new();
    let metas = session_parser::list_sessions(60, 0)?;
    let meta = metas
        .into_iter()
        .find(|m| m.id.as_str() == session_id)
        .ok_or_else(|| {
            // Fallback: build minimal meta
            drop(&names);
            format!("Session {} not found in list", session_id)
        })?;

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut messages: Vec<SessionMessage> = Vec::new();
    let mut tools: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();

    for line in content.lines() {
        let entry: RawEntry = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let etype = entry.entry_type.as_deref().unwrap_or("");

        if matches!(etype, "user" | "assistant") {
            if let Some(ref msg) = entry.message {
                let text = msg.content.as_ref().map(content_to_string).unwrap_or_default();
                if text.contains("<system-reminder>") || text.is_empty() {
                    continue;
                }
                let truncated = if text.len() > 2000 {
                    let end = text.char_indices()
                        .take_while(|(i, _)| *i < 2000)
                        .last()
                        .map(|(i, c)| i + c.len_utf8())
                        .unwrap_or(2000.min(text.len()));
                    format!("{}…", &text[..end])
                } else {
                    text
                };
                messages.push(SessionMessage {
                    role: msg.role.as_deref().unwrap_or(etype).to_string(),
                    content: truncated,
                    timestamp: entry.timestamp.unwrap_or_default(),
                });
            }
        }

        if etype.contains("file-history-snapshot") {
            if let Ok(raw) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(backups) = raw
                    .get("snapshot")
                    .and_then(|s| s.get("trackedFileBackups"))
                    .and_then(|b| b.as_object())
                {
                    for key in backups.keys() {
                        if !files.contains(key) {
                            files.push(key.clone());
                        }
                    }
                }
            }
        }

        if etype.contains("assistant") {
            if let Some(ref msg) = entry.message {
                if let Some(serde_json::Value::Array(arr)) = &msg.content {
                    for block in arr {
                        if let Some(name) = block.get("name").and_then(|n| n.as_str()) {
                            let s = name.to_string();
                            if !tools.contains(&s) {
                                tools.push(s);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(SessionDetail {
        meta,
        messages,
        files_modified: files,
        tools_used: tools,
    })
}

fn find_file(dir: &PathBuf, session_id: &str) -> Result<PathBuf, String> {
    let direct = dir.join(format!("{}.jsonl", session_id));
    if direct.exists() {
        return Ok(direct);
    }

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
            if let Ok(line) = read_first_line(&path) {
                if line.contains(session_id) {
                    return Ok(path);
                }
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
