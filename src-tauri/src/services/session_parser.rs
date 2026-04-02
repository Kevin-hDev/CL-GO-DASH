use crate::services::session_types::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn sessions_dir() -> PathBuf {
    let home = dirs::home_dir().expect("cannot resolve home");
    home.join(".claude/projects/-Users-kevinh-Projects")
}

fn names_file() -> PathBuf {
    let home = dirs::home_dir().expect("cannot resolve home");
    home.join(".local/share/cl-go/dash-session-names.json")
}

fn load_names() -> HashMap<String, String> {
    let path = names_file();
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

pub fn list_sessions(limit: usize, offset: usize) -> Result<Vec<SessionMeta>, String> {
    let dir = sessions_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let files: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| format!("Cannot read dir: {}", e))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "jsonl").unwrap_or(false))
        .collect();

    let names = load_names();
    let mut metas: Vec<SessionMeta> = files
        .into_iter()
        .filter_map(|p| parse_meta(&p, &names).ok())
        .collect();

    // Sort by actual session start timestamp, newest first
    metas.sort_by(|a, b| b.start.cmp(&a.start));

    Ok(metas.into_iter().skip(offset).take(limit).collect())
}

fn parse_meta(path: &PathBuf, names: &HashMap<String, String>) -> Result<SessionMeta, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Read: {}", e))?;
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Err("Empty".into());
    }

    let (mut first_ts, mut last_ts) = (String::new(), String::new());
    let (mut sid, mut ver, mut mode): (String, String, String) =
        (String::new(), String::new(), "unknown".into());
    let mut msg_count: u32 = 0;

    for line in lines.iter().take(20) {
        if let Ok(e) = serde_json::from_str::<RawEntry>(line) {
            if first_ts.is_empty() {
                if let Some(ref t) = e.timestamp { first_ts = t.clone(); }
            }
            if sid.is_empty() {
                if let Some(ref s) = e.session_id { sid = s.clone(); }
            }
            if ver.is_empty() {
                if let Some(ref v) = e.version { ver = v.clone(); }
            }
            if mode.contains("unknown") { detect_mode(&e, &mut mode); }
        }
    }

    let tail = if lines.len() > 10 { lines.len() - 10 } else { 0 };
    for line in &lines[tail..] {
        if let Ok(e) = serde_json::from_str::<RawEntry>(line) {
            if let Some(ref t) = e.timestamp { last_ts = t.clone(); }
            if let Some(mc) = e.message_count { msg_count = msg_count.max(mc); }
        }
    }

    // Duration = last_ts - first_ts (real elapsed time)
    let duration_minutes = compute_duration_minutes(&first_ts, &last_ts);

    // Fallback: use filename as id if sessionId is missing
    if sid.is_empty() {
        sid = path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
    }

    let custom_name = names.get(&sid).cloned();
    Ok(SessionMeta {
        id: sid,
        file_path: path.to_string_lossy().to_string(),
        start: first_ts,
        end: last_ts,
        duration_minutes,
        mode, message_count: msg_count, version: ver,
        custom_name,
    })
}

fn compute_duration_minutes(start: &str, end: &str) -> f64 {
    // ISO 8601: "2026-04-01T21:28:24.022Z"
    let parse = |s: &str| -> Option<i64> {
        // Extract just YYYY-MM-DDTHH:MM:SS, parse manually
        if s.len() < 19 { return None; }
        let parts: Vec<&str> = s[..19].split('T').collect();
        if parts.len() != 2 { return None; }
        let date: Vec<i64> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();
        let time: Vec<i64> = parts[1].split(':').filter_map(|p| p.parse().ok()).collect();
        if date.len() != 3 || time.len() != 3 { return None; }
        // Rough epoch: days since 2026-01-01 * 86400 + seconds
        let days = (date[1] - 1) * 30 + date[2]; // approximate
        Some(days * 86400 + time[0] * 3600 + time[1] * 60 + time[2])
    };

    match (parse(start), parse(end)) {
        (Some(s), Some(e)) if e > s => (e - s) as f64 / 60.0,
        _ => 0.0,
    }
}

fn detect_mode(entry: &RawEntry, mode: &mut String) {
    if let Some(ref msg) = entry.message {
        if let Some(ref c) = msg.content {
            let text = content_to_string(c);
            for m in ["auto", "explorer", "free", "evolve"] {
                if text.contains(&format!("--{}", m)) {
                    *mode = m.to_string();
                    return;
                }
            }
        }
    }
}

pub fn save_session_name(session_id: &str, name: &str) -> Result<(), String> {
    let mut names = load_names();
    names.insert(session_id.to_string(), name.to_string());
    let json = serde_json::to_string_pretty(&names).map_err(|e| e.to_string())?;
    fs::write(names_file(), json).map_err(|e| e.to_string())
}

pub fn delete_session(file_path: &str) -> Result<(), String> {
    let path = PathBuf::from(file_path);
    if !path.starts_with(sessions_dir()) {
        return Err("Invalid path".into());
    }
    fs::remove_file(&path).map_err(|e| format!("Delete: {}", e))
}
