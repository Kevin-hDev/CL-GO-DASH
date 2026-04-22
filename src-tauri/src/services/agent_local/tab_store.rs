use crate::services::agent_local::types_session::TabState;
use std::path::PathBuf;

fn tab_file() -> PathBuf {
    crate::services::paths::data_dir().join("agent-tabs.json")
}

pub async fn get_state() -> Result<TabState, String> {
    let path = tab_file();
    if !path.exists() {
        return Ok(TabState {
            tabs: Vec::new(),
            active_index: 0,
        });
    }
    let data = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

pub async fn save_state(state: &TabState) -> Result<(), String> {
    let path = tab_file();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("tmp");
    let data = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    tokio::fs::write(&tmp, &data).await.map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, &path).await.map_err(|e| e.to_string())?;
    Ok(())
}
