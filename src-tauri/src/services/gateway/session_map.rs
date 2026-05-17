use std::collections::HashMap;
use std::sync::LazyLock;

use tokio::sync::Mutex;

use crate::services::paths::data_dir;

static MAP: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(load_from_disk().unwrap_or_default()));

fn map_path() -> std::path::PathBuf {
    data_dir()
        .join("agent-sessions")
        .join("gateway-session-map.json")
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

pub async fn insert_bounded(
    channel_key: &str,
    session_id: &str,
    max_mappings: usize,
) -> Result<(), String> {
    let mut map = MAP.lock().await;
    let limit = max_mappings.max(1);
    if !map.contains_key(channel_key) {
        while map.len() >= limit {
            let Some(oldest) = map.keys().next().cloned() else {
                break;
            };
            map.remove(&oldest);
        }
    }
    map.insert(channel_key.to_string(), session_id.to_string());
    flush(&map)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[tokio::test]
    async fn insert_and_find() {
        let _guard = TEST_LOCK.lock().await;
        let mut map = MAP.lock().await;
        map.insert("test/key".to_string(), "uuid-123".to_string());
        assert_eq!(map.get("test/key"), Some(&"uuid-123".to_string()));
        map.remove("test/key");
    }

    #[tokio::test]
    async fn find_missing_returns_none() {
        let _guard = TEST_LOCK.lock().await;
        let map = MAP.lock().await;
        assert!(map.get("nonexistent/key").is_none());
    }

    #[tokio::test]
    async fn bounded_insert_evicts() {
        let _guard = TEST_LOCK.lock().await;
        let mut map = MAP.lock().await;
        map.clear();
        let _ = flush(&map);
        drop(map);

        insert_bounded("a", "1", 2).await.unwrap();
        insert_bounded("b", "2", 2).await.unwrap();
        insert_bounded("c", "3", 2).await.unwrap();

        let map = MAP.lock().await;
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("c"), Some(&"3".to_string()));
        let mut map = map;
        map.clear();
        let _ = flush(&map);
    }

    #[tokio::test]
    async fn load_from_disk_reads_flushed_map() {
        let _guard = TEST_LOCK.lock().await;
        let mut map = MAP.lock().await;
        map.clear();
        map.insert("reload/key".to_string(), "session-1".to_string());
        flush(&map).unwrap();
        drop(map);

        let loaded = load_from_disk().unwrap();
        assert_eq!(loaded.get("reload/key"), Some(&"session-1".to_string()));

        let mut map = MAP.lock().await;
        map.clear();
        let _ = flush(&map);
    }
}
