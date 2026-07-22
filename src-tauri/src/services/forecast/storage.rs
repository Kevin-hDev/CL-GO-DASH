use crate::services::forecast::limits::MAX_STORED_ANALYSIS_BYTES;
use crate::services::forecast::types::{ForecastAnalysisMeta, ForecastResult};
use tokio::sync::Mutex;

use super::storage_paths::{
    analysis_path_for_read, analysis_path_for_write, validate_analysis_id, validate_analysis_name,
};

static SAVE_LOCK: Mutex<()> = Mutex::const_new(());

pub async fn save(result: &mut ForecastResult) -> Result<(), String> {
    validate_analysis_id(&result.id)?;
    result.name = validate_analysis_name(&result.name)?;
    validate_session_id(result.session_id.as_deref())?;
    let _save_guard = SAVE_LOCK.lock().await;
    let target = analysis_path_for_write(&result.id).await?;
    let already_exists = tokio::fs::try_exists(&target)
        .await
        .map_err(|_| "Erreur de sauvegarde".to_string())?;
    let stored_revision = if already_exists {
        Some(load(&result.id).await?.revision)
    } else {
        None
    };
    result.revision = super::storage_revision::next(stored_revision, result.revision)?;
    let json =
        serde_json::to_vec_pretty(result).map_err(|_| "Erreur de sérialisation".to_string())?;
    if json.len() > MAX_STORED_ANALYSIS_BYTES {
        return Err("Analyse Forecast trop volumineuse".into());
    }
    crate::services::private_store::atomic_write_async(target.clone(), json)
        .await
        .map_err(|_| "Erreur de sauvegarde".to_string())?;

    if let Err(error) = super::storage_index::upsert(result.to_meta()).await {
        if !already_exists {
            let _ = tokio::fs::remove_file(target).await;
        }
        return Err(error);
    }
    Ok(())
}

pub async fn load(id: &str) -> Result<ForecastResult, String> {
    validate_analysis_id(id)?;
    let path = analysis_path_for_read(id)
        .await
        .map_err(|_| "Analyse introuvable".to_string())?;
    let data = super::storage_io::read_bounded(&path, MAX_STORED_ANALYSIS_BYTES)
        .await
        .map_err(|_| "Analyse introuvable".to_string())?;
    serde_json::from_slice(&data).map_err(|_| "Données d'analyse corrompues".to_string())
}

pub async fn delete(id: &str) -> Result<(), String> {
    validate_analysis_id(id)?;
    let _save_guard = SAVE_LOCK.lock().await;
    match analysis_path_for_read(id).await {
        Ok(path) => tokio::fs::remove_file(&path)
            .await
            .map_err(|_| "Suppression échouée".to_string())?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Err("Suppression échouée".to_string()),
    }
    super::storage_index::remove(id).await
}

pub async fn rename(id: &str, name: &str) -> Result<ForecastAnalysisMeta, String> {
    validate_analysis_id(id)?;
    let next_name = validate_analysis_name(name)?;
    let mut analysis = load(id).await?;
    analysis.name = next_name;
    save(&mut analysis).await?;
    Ok(analysis.to_meta())
}

pub async fn list() -> Result<Vec<ForecastAnalysisMeta>, String> {
    super::storage_index::list().await
}

pub async fn comparable_backtests(
    profile: &super::data_quality::DataProfile,
) -> Result<Vec<super::evaluation::types::BacktestIndexSummary>, String> {
    super::storage_backtests::comparable(super::storage_index::entries().await?, profile)
}

fn validate_session_id(session_id: Option<&str>) -> Result<(), String> {
    if let Some(id) = session_id {
        crate::services::agent_local::session_store::validate_session_id(id)
            .map_err(|_| "Identifiant de session invalide".to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_session_id_is_validated_before_saving() {
        assert!(validate_session_id(None).is_ok());
        assert!(validate_session_id(Some("550e8400-e29b-41d4-a716-446655440000")).is_ok());
        assert!(validate_session_id(Some("../session")).is_err());
    }
}
