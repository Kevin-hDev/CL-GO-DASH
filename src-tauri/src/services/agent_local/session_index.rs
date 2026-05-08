use crate::services::agent_local::types_session::{AgentSession, AgentSessionMeta};
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

static INDEX_LOCK: Mutex<()> = Mutex::const_new(());

fn index_path() -> PathBuf {
    crate::services::paths::data_dir()
        .join("agent-sessions")
        .join("index.json")
}

pub async fn read_index() -> Result<Vec<AgentSessionMeta>, String> {
    let path = index_path();
    match tokio::fs::read_to_string(&path).await {
        Ok(data) => match serde_json::from_str::<Vec<AgentSessionMeta>>(&data) {
            Ok(entries) => Ok(entries),
            Err(_) => rebuild_index().await,
        },
        Err(_) => rebuild_index().await,
    }
}

pub async fn rebuild_index() -> Result<Vec<AgentSessionMeta>, String> {
    let dir = crate::services::paths::data_dir().join("agent-sessions");
    rebuild_index_from(&dir).await
}

pub async fn rebuild_index_from(dir: &Path) -> Result<Vec<AgentSessionMeta>, String> {
    let mut entries = Vec::new();
    if !dir.exists() {
        return Ok(entries);
    }
    let mut read_dir = tokio::fs::read_dir(dir)
        .await
        .map_err(|e| e.to_string())?;
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("index.json") {
            continue;
        }
        if let Ok(data) = tokio::fs::read_to_string(&path).await {
            if let Ok(session) = serde_json::from_str::<AgentSession>(&data) {
                entries.push(meta_from_session(&session));
            }
        }
    }
    write_index_to(dir, &entries).await?;
    Ok(entries)
}

pub async fn upsert_entry(meta: AgentSessionMeta) -> Result<(), String> {
    let _guard = INDEX_LOCK.lock().await;
    let mut entries = read_index_raw().await;
    if let Some(pos) = entries.iter().position(|e| e.id == meta.id) {
        entries[pos] = meta;
    } else {
        entries.push(meta);
    }
    write_index(&entries).await
}

pub async fn remove_entry(id: &str) -> Result<(), String> {
    let _guard = INDEX_LOCK.lock().await;
    let mut entries = read_index_raw().await;
    entries.retain(|e| e.id != id);
    write_index(&entries).await
}

pub fn meta_from_session(session: &AgentSession) -> AgentSessionMeta {
    AgentSessionMeta {
        id: session.id.clone(),
        name: session.name.clone(),
        created_at: session.created_at,
        model: session.model.clone(),
        provider: session.provider.clone(),
        message_count: session.messages.len(),
        is_heartbeat: session.is_heartbeat,
        project_id: session.project_id.clone(),
        parent_session_id: session.parent_session_id.clone(),
        subagent_type: session.subagent_type.clone(),
        subagent_status: session.subagent_status.clone(),
        subagent_run_id: session.subagent_run_id.clone(),
    }
}

async fn read_index_raw() -> Vec<AgentSessionMeta> {
    let path = index_path();
    if let Ok(data) = tokio::fs::read_to_string(&path).await {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

async fn write_index(entries: &[AgentSessionMeta]) -> Result<(), String> {
    let dir = crate::services::paths::data_dir().join("agent-sessions");
    write_index_to(&dir, entries).await
}

pub(crate) async fn write_index_to(
    dir: &Path,
    entries: &[AgentSessionMeta],
) -> Result<(), String> {
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|e| e.to_string())?;
    let path = dir.join("index.json");
    let tmp = dir.join(format!(".index.{}.tmp", uuid::Uuid::new_v4()));
    let data = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?;
    tokio::fs::write(&tmp, &data)
        .await
        .map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[path = "session_index_tests.rs"]
#[cfg(test)]
mod tests;
