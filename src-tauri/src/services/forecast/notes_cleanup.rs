use crate::services::paths::data_dir;
use std::path::PathBuf;
use uuid::Uuid;

pub async fn delete_analysis_notes(analysis_id: &str) -> Result<(), String> {
    validate_analysis_id(analysis_id)?;
    let dir = notes_dir(analysis_id);
    if !dir.exists() {
        return Ok(());
    }
    tokio::fs::remove_dir_all(&dir)
        .await
        .map_err(|_| "Suppression des notes échouée".to_string())
}

fn notes_dir(analysis_id: &str) -> PathBuf {
    data_dir().join("forecast-notes").join(analysis_id)
}

fn validate_analysis_id(analysis_id: &str) -> Result<(), String> {
    if Uuid::parse_str(analysis_id).is_ok() {
        Ok(())
    } else {
        Err("Identifiant d'analyse invalide".into())
    }
}
