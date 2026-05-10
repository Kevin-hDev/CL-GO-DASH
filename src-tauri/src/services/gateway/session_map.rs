use std::collections::HashMap;
use std::sync::LazyLock;

use tokio::sync::Mutex;

use crate::services::paths::data_dir;

static MAP: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| {
    Mutex::new(load_from_disk().unwrap_or_default())
});

fn map_path() -> std::path::PathBuf {
    data_dir().join("agent-sessions").join("gateway-session-map.json")
}

fn load_from_disk() -> Option<HashMap<String, String>> {
    let data = std::fs::read_to_string(map_path()).ok()?;
    serde_json::from_str(&data).ok()
}

fn flush(map: &HashMap<String, String>) -> Result<(), String> {
    let path = map_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(map).map_err(|e| format!("json: {e}"))?;
    std::fs::write(&tmp, json).map_err(|e| format!("write: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("rename: {e}"))?;
    Ok(())
}

pub async fn find(channel_key: &str) -> Option<String> {
    let map = MAP.lock().await;
    map.get(channel_key).cloned()
}

pub async fn insert(channel_key: &str, session_id: &str) -> Result<(), String> {
    let mut map = MAP.lock().await;
    map.insert(channel_key.to_string(), session_id.to_string());
    flush(&map)
}

pub async fn remove(channel_key: &str) -> Result<(), String> {
    let mut map = MAP.lock().await;
    map.remove(channel_key);
    flush(&map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn insert_and_find() {
        let mut map = MAP.lock().await;
        map.insert("test/key".to_string(), "uuid-123".to_string());
        assert_eq!(map.get("test/key"), Some(&"uuid-123".to_string()));
        map.remove("test/key");
    }

    #[tokio::test]
    async fn find_missing_returns_none() {
        let map = MAP.lock().await;
        assert!(map.get("nonexistent/key").is_none());
    }
}
