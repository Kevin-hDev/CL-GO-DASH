use crate::services::forecast::limits::{
    MAX_ANALYSIS_INDEX_BYTES, MAX_STORED_ANALYSES, MAX_STORED_ANALYSIS_BYTES,
};
use crate::services::forecast::types::{ForecastAnalysisMeta, ForecastResult};
use tokio::sync::Mutex;

use super::storage_paths::{
    analysis_path_for_read, analysis_path_for_write, index_path, validate_analysis_id,
    validate_analysis_name,
};

static INDEX_LOCK: Mutex<()> = Mutex::const_new(());
static SAVE_LOCK: Mutex<()> = Mutex::const_new(());

pub async fn save(result: &mut ForecastResult) -> Result<(), String> {
    validate_analysis_id(&result.id)?;
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

    if let Err(error) = upsert_index(result.to_meta()).await {
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
    remove_from_index(id).await
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
    let entries = read_index().await?;
    hydrate_index(entries).await
}

pub async fn comparable_backtests(
    profile: &super::data_quality::DataProfile,
) -> Result<Vec<super::evaluation::types::BacktestIndexSummary>, String> {
    super::storage_backtests::comparable(read_index().await?, profile)
}

async fn read_index() -> Result<Vec<ForecastAnalysisMeta>, String> {
    let path = index_path();
    let data = match super::storage_io::read_bounded(&path, MAX_ANALYSIS_INDEX_BYTES).await {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err("Index forecast indisponible".into()),
    };
    let entries: Vec<ForecastAnalysisMeta> =
        serde_json::from_slice(&data).map_err(|_| "Index forecast corrompu".to_string())?;
    if entries.len() > MAX_STORED_ANALYSES
        || entries.iter().any(|entry| {
            validate_analysis_id(&entry.id).is_err()
                || entry.backtest.as_ref().is_some_and(|backtest| {
                    backtest.results.len() > crate::services::forecast::limits::MAX_BACKTEST_RESULTS
                })
        })
    {
        return Err("Index forecast corrompu".into());
    }
    Ok(entries)
}

async fn write_index(entries: &[ForecastAnalysisMeta]) -> Result<(), String> {
    if entries.len() > MAX_STORED_ANALYSES {
        return Err("Index forecast trop volumineux".into());
    }
    let json =
        serde_json::to_vec_pretty(entries).map_err(|_| "Index forecast invalide".to_string())?;
    if json.len() > MAX_ANALYSIS_INDEX_BYTES {
        return Err("Index forecast trop volumineux".into());
    }
    let target = index_path();
    crate::services::private_store::atomic_write_async(target, json)
        .await
        .map_err(|_| "Finalisation index forecast échouée".to_string())
}

async fn hydrate_index(
    entries: Vec<ForecastAnalysisMeta>,
) -> Result<Vec<ForecastAnalysisMeta>, String> {
    let mut changed = false;
    let mut hydrated = Vec::with_capacity(entries.len());

    for mut meta in entries {
        let scenarios_count = read_scenarios_count(&meta.id).await.unwrap_or(0);
        if meta.scenarios_count != scenarios_count {
            meta.scenarios_count = scenarios_count;
            changed = true;
        }
        hydrated.push(meta);
    }

    if changed {
        write_index(&hydrated).await?;
    }

    Ok(hydrated)
}

async fn read_scenarios_count(id: &str) -> Result<usize, String> {
    let analysis = load(id).await?;
    Ok(analysis.scenarios.len())
}

async fn upsert_index(meta: ForecastAnalysisMeta) -> Result<(), String> {
    let _guard = INDEX_LOCK.lock().await;
    let mut entries = read_index().await?;
    let mut removed_ids = Vec::new();
    if let Some(pos) = entries.iter().position(|e| e.id == meta.id) {
        entries[pos] = meta;
    } else {
        entries.push(meta);
        // Borner la collection : supprimer les plus anciennes si dépassement
        if entries.len() > MAX_STORED_ANALYSES {
            removed_ids.extend(
                entries
                    .drain(0..entries.len() - MAX_STORED_ANALYSES)
                    .map(|entry| entry.id),
            );
        }
    }
    write_index(&entries).await?;
    for id in removed_ids {
        validate_analysis_id(&id)?;
        let path = analysis_path_for_read(&id)
            .await
            .map_err(|_| "Nettoyage des analyses échoué".to_string())?;
        tokio::fs::remove_file(path)
            .await
            .map_err(|_| "Nettoyage des analyses échoué".to_string())?;
    }
    Ok(())
}

async fn remove_from_index(id: &str) -> Result<(), String> {
    let _guard = INDEX_LOCK.lock().await;
    let mut entries = read_index().await?;
    entries.retain(|e| e.id != id);
    write_index(&entries).await
}
