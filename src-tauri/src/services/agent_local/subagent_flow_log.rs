use serde_json::{json, Value};

const MAX_LOG_BYTES: u64 = 4 * 1024 * 1024;

pub fn record(
    event: &str,
    parent_id: Option<&str>,
    child_id: Option<&str>,
    run_id: Option<&str>,
    detail: Value,
) {
    let entry = entry(event, parent_id, child_id, run_id, detail);
    eprintln!("[subagent-flow] {}", entry);

    let dir = crate::services::paths::data_dir().join("logs");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("subagent-flow.jsonl");
    if std::fs::metadata(&path)
        .map(|meta| meta.len() > MAX_LOG_BYTES)
        .unwrap_or(false)
    {
        let rotated = dir.join("subagent-flow.jsonl.1");
        let _ = std::fs::rename(&path, rotated);
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        use std::io::Write;
        let _ = writeln!(file, "{}", entry);
    }
}

pub fn entry(
    event: &str,
    parent_id: Option<&str>,
    child_id: Option<&str>,
    run_id: Option<&str>,
    detail: Value,
) -> Value {
    json!({
        "ts": chrono::Local::now().to_rfc3339(),
        "event": event,
        "parent_session_id": parent_id,
        "child_session_id": child_id,
        "run_id": run_id,
        "detail": detail,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_omits_prompt_and_report_content() {
        let entry = entry(
            "spawn_prepared",
            Some("parent"),
            Some("child"),
            Some("run"),
            json!({"status": "running"}),
        );
        assert_eq!(entry["event"], "spawn_prepared");
        assert_eq!(entry["parent_session_id"], "parent");
        assert!(entry.get("prompt").is_none());
        assert!(entry.get("summary").is_none());
        assert!(entry.get("report").is_none());
    }
}
