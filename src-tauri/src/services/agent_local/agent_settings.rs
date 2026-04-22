use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSettings {
    pub permission_mode: String,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            permission_mode: "auto".to_string(),
        }
    }
}

fn settings_path() -> PathBuf {
    crate::services::paths::data_dir().join("agent-settings.json")
}

pub async fn load() -> AgentSettings {
    let path = settings_path();
    if !path.exists() {
        return AgentSettings::default();
    }
    match tokio::fs::read_to_string(&path).await {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => AgentSettings::default(),
    }
}

pub async fn save(settings: &AgentSettings) -> Result<(), String> {
    if !matches!(settings.permission_mode.as_str(), "auto" | "manual" | "chat") {
        return Err("permission_mode invalide".into());
    }
    let path = settings_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("tmp");
    let data = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    tokio::fs::write(&tmp, &data)
        .await
        .map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn get_permission_mode() -> String {
    load().await.permission_mode
}
