use crate::services::agent_local::types_session::{AgentSession, AgentSessionMeta};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use uuid::Uuid;

static SESSION_LOCKS: LazyLock<Mutex<HashMap<String, std::sync::Arc<Mutex<()>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

async fn lock_session(id: &str) -> std::sync::Arc<Mutex<()>> {
    let mut map = SESSION_LOCKS.lock().await;
    map.entry(id.to_string())
        .or_insert_with(|| std::sync::Arc::new(Mutex::new(())))
        .clone()
}

fn sessions_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("cl-go")
        .join("agent-sessions")
}

pub async fn create(name: &str, model: &str) -> Result<AgentSession, String> {
    let session = AgentSession {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        created_at: Utc::now(),
        model: model.to_string(),
        thinking_enabled: false,
        accumulated_tokens: 0,
        messages: Vec::new(),
    };
    save(&session).await?;
    Ok(session)
}

pub async fn get(id: &str) -> Result<AgentSession, String> {
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
                    message_count: session.messages.len(),
                });
            }
        }
    }
    metas.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(metas)
}

pub async fn save(session: &AgentSession) -> Result<(), String> {
    let dir = sessions_dir();
    tokio::fs::create_dir_all(&dir).await.map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.json", session.id));
    let tmp = dir.join(format!(".{}.{}.tmp", session.id, Uuid::new_v4()));
    let data = serde_json::to_string_pretty(session).map_err(|e| e.to_string())?;
    tokio::fs::write(&tmp, &data).await.map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, &path).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn add_messages(
    id: &str,
    mut new_messages: Vec<crate::services::agent_local::types_session::AgentMessage>,
    tokens: u32,
) -> Result<(), String> {
    let lock = lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = get(id).await?;
    // Affecter tous les tokens au dernier message ajouté (l'assistant typiquement)
    if tokens > 0 {
        if let Some(last) = new_messages.last_mut() {
            last.tokens = tokens;
        }
    }
    session.messages.extend(new_messages);
    session.accumulated_tokens = session.messages.iter().map(|m| m.tokens).sum();
    save(&session).await
}

pub async fn rename(id: &str, name: &str) -> Result<(), String> {
    let mut session = get(id).await?;
    session.name = name.to_string();
    save(&session).await
}

pub async fn delete(id: &str) -> Result<(), String> {
    let path = sessions_dir().join(format!("{id}.json"));
    tokio::fs::remove_file(&path).await.map_err(|e| format!("Erreur suppression: {e}"))
}

pub async fn export_markdown(id: &str) -> Result<String, String> {
    let session = get(id).await?;
    let mut md = format!("# {}\n\n", session.name);
    for msg in &session.messages {
        let role = match msg.role.as_str() {
            "user" => "**Utilisateur**",
            "assistant" => "**Assistant**",
            "tool" => "**Outil**",
            _ => &msg.role,
        };
        md.push_str(&format!("### {role}\n\n{}\n\n---\n\n", msg.content));
    }
    Ok(md)
}

pub async fn truncate_at(session_id: &str, message_id: &str) -> Result<(), String> {
    let lock = lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = get(session_id).await?;
    if let Some(idx) = session.messages.iter().position(|m| m.id == message_id) {
        session.messages.truncate(idx);
        session.accumulated_tokens = session.messages.iter().map(|m| m.tokens).sum();
        save(&session).await?;
    }
    Ok(())
}

pub async fn truncate_and_replace(
    session_id: &str,
    message_id: &str,
    replacement: Option<crate::services::agent_local::types_session::AgentMessage>,
) -> Result<(), String> {
    let lock = lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = get(session_id).await?;
    if let Some(idx) = session.messages.iter().position(|m| m.id == message_id) {
        match replacement {
            // Edit : supprime le message cible et tout ce qui suit, ajoute le nouveau
            Some(new_msg) => {
                session.messages.truncate(idx);
                session.messages.push(new_msg);
            }
            // Reload : garde le message cible, supprime tout ce qui suit
            None => {
                session.messages.truncate(idx + 1);
            }
        }
        session.accumulated_tokens = session.messages.iter().map(|m| m.tokens).sum();
        save(&session).await?;
    }
    Ok(())
}
