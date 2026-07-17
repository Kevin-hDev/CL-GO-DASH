use crate::services::oauth_providers::ProviderId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MAX_FILE_BYTES: u64 = 2048;
const MAX_ACP_SESSION_ID_CHARS: usize = 256;

#[derive(Serialize, Deserialize)]
struct Metadata {
    provider: ProviderId,
    acp_session_id: String,
}

fn path(session_id: &str) -> Result<PathBuf, String> {
    crate::services::agent_local::session_store::validate_session_id(session_id)?;
    Ok(crate::services::paths::data_dir()
        .join("agent-sessions")
        .join(format!("{session_id}.acp.json")))
}

fn valid_acp_id(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= MAX_ACP_SESSION_ID_CHARS
        && !value.chars().any(char::is_control)
}

pub async fn save(
    session_id: &str,
    provider: ProviderId,
    acp_session_id: &str,
) -> Result<(), String> {
    if !valid_acp_id(acp_session_id) {
        return Err("Session ACP invalide".to_string());
    }
    let target = path(session_id)?;
    let directory = target
        .parent()
        .ok_or_else(|| "Session ACP invalide".to_string())?;
    tokio::fs::create_dir_all(directory)
        .await
        .map_err(|_| "Session ACP inaccessible".to_string())?;
    let data = serde_json::to_vec(&Metadata {
        provider,
        acp_session_id: acp_session_id.to_string(),
    })
    .map_err(|_| "Session ACP invalide".to_string())?;
    let temp = directory.join(format!(".{session_id}.{}.acp.tmp", uuid::Uuid::new_v4()));
    tokio::fs::write(&temp, data)
        .await
        .map_err(|_| "Session ACP inaccessible".to_string())?;
    tokio::fs::rename(temp, target)
        .await
        .map_err(|_| "Session ACP inaccessible".to_string())
}

pub async fn load(session_id: &str, provider: ProviderId) -> Result<Option<String>, String> {
    let target = path(session_id)?;
    let metadata = match tokio::fs::metadata(&target).await {
        Ok(metadata) if metadata.len() <= MAX_FILE_BYTES => metadata,
        Ok(_) => return Err("Session ACP invalide".to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("Session ACP inaccessible".to_string()),
    };
    if !metadata.is_file() {
        return Err("Session ACP invalide".to_string());
    }
    let data = tokio::fs::read(target)
        .await
        .map_err(|_| "Session ACP inaccessible".to_string())?;
    let stored: Metadata =
        serde_json::from_slice(&data).map_err(|_| "Session ACP invalide".to_string())?;
    if stored.provider != provider || !valid_acp_id(&stored.acp_session_id) {
        return Ok(None);
    }
    Ok(Some(stored.acp_session_id))
}

pub async fn remove(session_id: &str) -> Result<(), String> {
    match tokio::fs::remove_file(path(session_id)?).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Session ACP inaccessible".to_string()),
    }
}
