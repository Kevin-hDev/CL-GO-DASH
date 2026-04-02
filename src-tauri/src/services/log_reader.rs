use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub message: String,
    pub is_error: bool,
}

pub fn read_warnings() -> Result<Vec<LogEntry>, String> {
    let home = dirs::home_dir().expect("cannot resolve home");
    let path = home.join(".local/share/cl-go/logs/heartbeat/wrapper.log");

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read wrapper.log: {}", e))?;

    let mut entries: Vec<LogEntry> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Format: [YYYY-MM-DD HH:MM:SS] message
        let (ts, msg) = parse_log_line(trimmed);
        let is_err = msg.starts_with("ERROR:")
            || msg.starts_with("WARN:")
            || msg.contains("not found")
            || msg.contains("aborted")
            || msg.contains("failed");

        if is_err {
            entries.push(LogEntry {
                timestamp: ts.to_string(),
                message: msg.to_string(),
                is_error: true,
            });
        }
    }

    // Most recent first
    entries.reverse();

    // Cap at 100 entries
    entries.truncate(100);

    Ok(entries)
}

fn parse_log_line(line: &str) -> (&str, &str) {
    // [2026-04-02 04:06:00] message
    if line.starts_with('[') {
        if let Some(end) = line.find("] ") {
            let ts = &line[1..end];
            let msg = &line[end + 2..];
            return (ts, msg);
        }
    }
    ("", line)
}
