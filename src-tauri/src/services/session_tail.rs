use crate::services::session_types::{content_to_string, RawEntry, SessionMessage};
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
                    // Session just started — seek to end
                    file_pos = file_len(path);
                    current_file = Some(path.clone());
                }
                (Some(cur), Some(path)) if cur != path => {
                    // Different session — reset
                    file_pos = file_len(path);
                    current_file = Some(path.clone());
                }
                (Some(_), None) => {
                    // Session ended
                    current_file = None;
                    file_pos = 0;
                }
                _ => {}
            }

            if let Some(ref path) = current_file {
                let messages = read_new_lines(path, &mut file_pos);
                if !messages.is_empty() {
                    let _ = handle.emit(EVENT_SESSION_MSG, &messages);
                }
            }

            let sleep = if current_file.is_some() {
                POLL_TAIL_MS
            } else {
                POLL_PID_MS
            };
            thread::sleep(Duration::from_millis(sleep));
        }
    });
}

fn active_session_jsonl() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let pid_path = home.join(".local/share/cl-go/logs/heartbeat/session.pid");
    let content = fs::read_to_string(&pid_path).ok()?;
    let pid: u32 = content.lines().next()?.trim().parse().ok()?;

    if !is_alive(pid) {
        return None;
    }

    // Find the most recently modified JSONL in the sessions dir
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

fn read_new_lines(path: &PathBuf, pos: &mut u64) -> Vec<SessionMessage> {
    let mut messages = Vec::new();

    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return messages,
    };

    let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
    if file_len <= *pos {
        return messages;
    }

    let mut reader = BufReader::new(file);
    if reader.seek(SeekFrom::Start(*pos)).is_err() {
        return messages;
    }

    let mut line = String::new();
    while reader.read_line(&mut line).unwrap_or(0) > 0 {
        if let Ok(entry) = serde_json::from_str::<RawEntry>(&line) {
            let etype = entry.entry_type.as_deref().unwrap_or("");
            if matches!(etype, "user" | "assistant") {
                if let Some(ref msg) = entry.message {
                    let text = msg
                        .content
                        .as_ref()
                        .map(content_to_string)
                        .unwrap_or_default();
                    if !text.is_empty() && !text.contains("<system-reminder>") {
                        messages.push(SessionMessage {
                            role: msg.role.as_deref().unwrap_or(etype).to_string(),
                            content: text,
                            timestamp: entry.timestamp.unwrap_or_default(),
                        });
                    }
                }
            }
        }
        line.clear();
    }

    *pos = file_len;
    messages
}

fn file_len(path: &PathBuf) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

fn is_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}
