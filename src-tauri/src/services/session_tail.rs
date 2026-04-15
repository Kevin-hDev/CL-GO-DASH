use crate::services::session_types::*;
use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const EVENT_SESSION_MSG: &str = "fs:session-message";
const POLL_PID_MS: u64 = 2000;
const POLL_TAIL_MS: u64 = 500;

pub fn start(app: &AppHandle) {
    let handle = app.clone();

    thread::spawn(move || {
        let mut current_file: Option<PathBuf> = None;
        let mut file_pos: u64 = 0;

        loop {
            let active = active_session_jsonl();

            match (&current_file, &active) {
                (None, Some(path)) => {
                    file_pos = file_len(path);
                    current_file = Some(path.clone());
                }
                (Some(cur), Some(path)) if cur != path => {
                    file_pos = file_len(path);
                    current_file = Some(path.clone());
                }
                (Some(_), None) => {
                    current_file = None;
                    file_pos = 0;
                }
                _ => {}
            }

            if let Some(ref path) = current_file {
                let entries = read_new_entries(path, &mut file_pos);
                if !entries.is_empty() {
                    let _ = handle.emit(EVENT_SESSION_MSG, &entries);
                }
            }

            let sleep = if current_file.is_some() { POLL_TAIL_MS } else { POLL_PID_MS };
            thread::sleep(Duration::from_millis(sleep));
        }
    });
}

fn active_session_jsonl() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let pid_path = home.join(".local/share/cl-go/logs/heartbeat/session.pid");
    let content = fs::read_to_string(&pid_path).ok()?;
    let pid: u32 = content.lines().next()?.trim().parse().ok()?;
    if !is_alive(pid) { return None; }

    let sessions_dir = home.join(".claude/projects/-Users-kevinh");
    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;
    for entry in fs::read_dir(&sessions_dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "jsonl") {
            if let Ok(meta) = path.metadata() {
                if let Ok(modified) = meta.modified() {
                    if newest.as_ref().is_none_or(|(_, t)| modified > *t) {
                        newest = Some((path, modified));
                    }
                }
            }
        }
    }
    newest.map(|(p, _)| p)
}

fn read_new_entries(path: &PathBuf, pos: &mut u64) -> Vec<SessionEntry> {
    let mut entries = Vec::new();
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return entries,
    };
    let flen = file.metadata().map(|m| m.len()).unwrap_or(0);
    if flen <= *pos { return entries; }

    let mut reader = BufReader::new(file);
    if reader.seek(SeekFrom::Start(*pos)).is_err() { return entries; }

    let mut line = String::new();
    while reader.read_line(&mut line).unwrap_or(0) > 0 {
        if let Ok(raw) = serde_json::from_str::<RawEntry>(&line) {
            let etype = raw.entry_type.as_deref().unwrap_or("");
            let ts = raw.timestamp.clone().unwrap_or_default();

            if let Some(ref msg) = raw.message {
                if let Some(serde_json::Value::Array(arr)) = &msg.content {
                    parse_blocks(&arr, etype, &ts, &mut entries);
                } else if let Some(ref val) = msg.content {
                    let text = content_to_string(val);
                    if !text.is_empty() && !text.contains("<system-reminder>") && matches!(etype, "user" | "assistant") {
                        entries.push(SessionEntry::Message(SessionMessage {
                            role: msg.role.as_deref().unwrap_or(etype).to_string(),
                            content: text,
                            timestamp: ts,
                        }));
                    }
                }
            }
        }
        line.clear();
    }
    *pos = flen;
    entries
}

fn parse_blocks(arr: &[serde_json::Value], etype: &str, ts: &str, entries: &mut Vec<SessionEntry>) {
    let text: String = arr.iter()
        .filter_map(|b| b.get("text").and_then(|t| t.as_str()).map(String::from))
        .collect::<Vec<_>>()
        .join("\n");

    if !text.is_empty() && !text.contains("<system-reminder>") {
        entries.push(SessionEntry::Message(SessionMessage {
            role: etype.to_string(), content: text, timestamp: ts.to_string(),
        }));
    }

    for block in arr {
        if block.get("type").and_then(|t| t.as_str()) != Some("tool_use") { continue; }
        let name = block.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
        let input = block.get("input").cloned().unwrap_or(serde_json::Value::Null);
        let (old_text, new_text) = if name == "Edit" {
            (
                input.get("old_string").and_then(|v| v.as_str()).map(String::from),
                input.get("new_string").and_then(|v| v.as_str()).map(String::from),
            )
        } else { (None, None) };
        entries.push(SessionEntry::Tool(ToolCall {
            name: name.to_string(),
            summary: tool_summary(name, &input),
            timestamp: ts.to_string(),
            old_text, new_text,
        }));
    }
}

fn file_len(path: &PathBuf) -> u64 { fs::metadata(path).map(|m| m.len()).unwrap_or(0) }
fn is_alive(pid: u32) -> bool { unsafe { libc::kill(pid as i32, 0) == 0 } }
