use crate::services::agent_local::types_session::{AgentSession, AgentSessionMeta};
use chrono::Utc;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use uuid::Uuid;

static SESSION_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-f0-9\-]+$").unwrap());

pub fn validate_session_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 {
        return Err("Identifiant de session invalide".into());
    }
    if !SESSION_ID_REGEX.is_match(id) {
        return Err("Identifiant de session invalide".into());
    }
    Ok(())
}

static SESSION_LOCKS: LazyLock<Mutex<HashMap<String, std::sync::Arc<Mutex<()>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) async fn lock_session(id: &str) -> std::sync::Arc<Mutex<()>> {
    let mut map = SESSION_LOCKS.lock().await;
    map.entry(id.to_string())
        .or_insert_with(|| std::sync::Arc::new(Mutex::new(())))
        .clone()
}

pub async fn remove_session_lock(id: &str) {
    SESSION_LOCKS.lock().await.remove(id);
}

fn sessions_dir() -> PathBuf {
    crate::services::paths::data_dir().join("agent-sessions")
}

pub async fn create_with_flags(
    name: &str,
    model: &str,
    provider: &str,
    is_heartbeat: bool,
) -> Result<AgentSession, String> {
    create_full(name, model, provider, is_heartbeat, None).await
}

pub async fn create_full(
    name: &str,
    model: &str,
    provider: &str,
    is_heartbeat: bool,
    project_id: Option<String>,
) -> Result<AgentSession, String> {
    let session = AgentSession {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        created_at: Utc::now(),
        model: model.to_string(),
        provider: provider.to_string(),
        thinking_enabled: false,
        accumulated_tokens: 0,
        messages: Vec::new(),
        is_heartbeat,
        project_id,
    };
    save(&session).await?;
    Ok(session)
}

/// Cherche la conversation heartbeat existante pour un couple (provider, model).
/// Retourne la plus récente si plusieurs existent, `None` sinon.
pub async fn find_heartbeat_session(
    provider: &str,
    model: &str,
) -> Result<Option<String>, String> {
    let dir = sessions_dir();
    if !dir.exists() {
        return Ok(None);
    }
    let mut read_dir = tokio::fs::read_dir(&dir)
        .await
        .map_err(|e| e.to_string())?;
    let mut best: Option<(chrono::DateTime<Utc>, String)> = None;
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(data) = tokio::fs::read_to_string(&path).await {
            if let Ok(session) = serde_json::from_str::<AgentSession>(&data) {
                let matches_provider = session.provider == provider;
                if session.is_heartbeat && matches_provider && session.model == model {
                    let candidate = (session.created_at, session.id);
                    if best.as_ref().map(|b| candidate.0 > b.0).unwrap_or(true) {
                        best = Some(candidate);
                    }
                }
            }
        }
    }
    Ok(best.map(|(_, id)| id))
}

pub async fn get(id: &str) -> Result<AgentSession, String> {
    validate_session_id(id)?;
    let path = sessions_dir().join(format!("{id}.json"));
    let data = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Session introuvable: {e}"))?;
    serde_json::from_str(&data).map_err(|e| format!("JSON invalide: {e}"))
}

pub async fn list() -> Result<Vec<AgentSessionMeta>, String> {
    let dir = sessions_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut read_dir = tokio::fs::read_dir(&dir)
        .await
        .map_err(|e| e.to_string())?;
    let mut metas = Vec::new();
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(data) = tokio::fs::read_to_string(&path).await {
            if let Ok(session) = serde_json::from_str::<AgentSession>(&data) {
                metas.push(AgentSessionMeta {
                    id: session.id,
                    name: session.name,
                    created_at: session.created_at,
                    model: session.model,
                    provider: session.provider,
                    message_count: session.messages.len(),
                    is_heartbeat: session.is_heartbeat,
                    project_id: session.project_id,
                });
            }
        }
    }
    metas.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(metas)
}

pub async fn save(session: &AgentSession) -> Result<(), String> {
    validate_session_id(&session.id)?;
    let dir = sessions_dir();
    tokio::fs::create_dir_all(&dir).await.map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.json", session.id));
    let tmp = dir.join(format!(".{}.{}.tmp", session.id, Uuid::new_v4()));
    let data = serde_json::to_string_pretty(session).map_err(|e| e.to_string())?;
    tokio::fs::write(&tmp, &data).await.map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, &path).await.map_err(|e| e.to_string())?;
    Ok(())
}

const MAX_MESSAGES_PER_SESSION: usize = 2000;

pub async fn add_messages(
    id: &str,
    mut new_messages: Vec<crate::services::agent_local::types_session::AgentMessage>,
    tokens: u32,
) -> Result<(), String> {
    validate_session_id(id)?;
    let lock = lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = get(id).await?;
    if tokens > 0 {
        if let Some(last) = new_messages.last_mut() {
            last.tokens = tokens;
        }
    }
    session.messages.extend(new_messages);
    if session.messages.len() > MAX_MESSAGES_PER_SESSION {
        let excess = session.messages.len() - MAX_MESSAGES_PER_SESSION;
        session.messages.drain(..excess);
    }
    session.accumulated_tokens = session.messages.iter().map(|m| m.tokens).sum();
    save(&session).await
}

pub async fn rename(id: &str, name: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let mut session = get(id).await?;
    session.name = name.to_string();
    save(&session).await
}

pub async fn update_model(id: &str, model: &str, provider: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let mut session = get(id).await?;
    session.model = model.to_string();
    session.provider = provider.to_string();
    save(&session).await
}

pub async fn delete(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let path = sessions_dir().join(format!("{id}.json"));
    tokio::fs::remove_file(&path).await.map_err(|e| format!("Erreur suppression: {e}"))
}

pub use super::session_ops::{
    clear_project_id, export_markdown, truncate_and_replace, truncate_at,
};

#[path = "session_store_tests.rs"]
#[cfg(test)]
mod tests;
