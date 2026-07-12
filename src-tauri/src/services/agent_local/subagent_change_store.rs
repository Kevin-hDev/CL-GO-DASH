use super::types_subagent_change::SubagentChangeMeta;
use std::path::PathBuf;

fn path(child_id: &str) -> Result<PathBuf, String> {
    super::types_subagent_change::validate_uuid(child_id)?;
    Ok(crate::services::paths::data_dir()
        .join("subagent-changes")
        .join(format!("{child_id}.json")))
}

pub async fn load(child_id: &str) -> Result<SubagentChangeMeta, String> {
    load_optional(child_id)
        .await?
        .ok_or_else(|| "Changement sous-agent indisponible".to_string())
}

pub async fn load_optional(child_id: &str) -> Result<Option<SubagentChangeMeta>, String> {
    let data = match tokio::fs::read_to_string(path(child_id)?).await {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("Changement sous-agent indisponible".to_string()),
    };
    let meta: SubagentChangeMeta = serde_json::from_str(&data)
        .map_err(|_| "Changement sous-agent indisponible".to_string())?;
    meta.validate()?;
    if meta.child_session_id != child_id {
        return Err("Changement sous-agent indisponible".into());
    }
    Ok(Some(meta))
}

pub async fn save(meta: &SubagentChangeMeta) -> Result<(), String> {
    meta.validate()?;
    let target = path(&meta.child_session_id)?;
    let dir = target
        .parent()
        .ok_or_else(|| "Persistance du changement impossible".to_string())?;
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|_| "Persistance du changement impossible".to_string())?;
    let tmp = dir.join(format!(".{}.{}.tmp", meta.child_session_id, uuid::Uuid::new_v4()));
    let data = serde_json::to_vec_pretty(meta)
        .map_err(|_| "Persistance du changement impossible".to_string())?;
    tokio::fs::write(&tmp, data)
        .await
        .map_err(|_| "Persistance du changement impossible".to_string())?;
    tokio::fs::rename(&tmp, target)
        .await
        .map_err(|_| "Persistance du changement impossible".to_string())
}

pub async fn remove(child_id: &str) -> Result<(), String> {
    match tokio::fs::remove_file(path(child_id)?).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Suppression du changement impossible".into()),
    }
}
