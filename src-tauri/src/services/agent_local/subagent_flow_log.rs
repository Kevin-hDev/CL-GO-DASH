use serde_json::{json, Map, Value};

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
    let mut entry = Map::new();
    entry.insert("ts".into(), json!(chrono::Local::now().to_rfc3339()));
    entry.insert("event".into(), json!(event));
    if let Some(parent_id) = parent_id {
        entry.insert("parent_session_id".into(), json!(parent_id));
    }
    if let Some(child_id) = child_id {
        entry.insert("child_session_id".into(), json!(child_id));
    }
    if let Some(run_id) = run_id {
        entry.insert("run_id".into(), json!(run_id));
    }
    entry.insert("detail".into(), detail);
    Value::Object(entry)
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

    #[test]
    fn entry_omits_absent_ids() {
        let entry = entry("event", None, None, None, json!({}));

        assert!(entry.get("parent_session_id").is_none());
        assert!(entry.get("child_session_id").is_none());
        assert!(entry.get("run_id").is_none());
    }
}
