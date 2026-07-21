use crate::services::forecast::limits::{
    MAX_ANALYSIS_INDEX_BYTES, MAX_STORED_ANALYSES, MAX_STORED_ANALYSIS_BYTES,
};
use crate::services::forecast::types::{ForecastAnalysisMeta, ForecastResult};
use crate::services::paths::data_dir;
use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::sync::Mutex;

static INDEX_LOCK: Mutex<()> = Mutex::const_new(());

static ANALYSIS_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-f0-9\-]+$").unwrap());

fn validate_analysis_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 {
        return Err("Identifiant d'analyse invalide".into());
    }
    if !ANALYSIS_ID_REGEX.is_match(id) {
        return Err("Identifiant d'analyse invalide".into());
    }
    Ok(())
}

fn validate_analysis_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    let len = trimmed.chars().count();
    if len == 0 || len > 120 || trimmed.chars().any(|c| c.is_control()) {
        return Err("Nom d'analyse invalide".into());
    }
    Ok(trimmed.to_string())
}

fn analyses_dir() -> PathBuf {
    data_dir().join("forecast-analyses")
}

fn index_path() -> PathBuf {
    analyses_dir().join("index.json")
}

fn analysis_path(id: &str) -> PathBuf {
    analyses_dir().join(format!("{id}.json"))
}

pub async fn save(result: &ForecastResult) -> Result<(), String> {
    validate_analysis_id(&result.id)?;
    let json =
        serde_json::to_vec_pretty(result).map_err(|_| "Erreur de sérialisation".to_string())?;
    if json.len() > MAX_STORED_ANALYSIS_BYTES {
        return Err("Analyse Forecast trop volumineuse".into());
    }
    let target = analysis_path(&result.id);
    let already_exists = tokio::fs::try_exists(&target)
        .await
        .map_err(|_| "Erreur de sauvegarde".to_string())?;
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
    let path = analysis_path(id);
    let data = super::storage_io::read_bounded(&path, MAX_STORED_ANALYSIS_BYTES)
        .await
        .map_err(|_| "Analyse introuvable".to_string())?;
    serde_json::from_slice(&data).map_err(|_| "Données d'analyse corrompues".to_string())
}

pub async fn delete(id: &str) -> Result<(), String> {
    validate_analysis_id(id)?;
    let path = analysis_path(id);
    if path.exists() {
        tokio::fs::remove_file(&path)
            .await
            .map_err(|_| "Suppression échouée".to_string())?;
    }
    remove_from_index(id).await
}

pub async fn rename(id: &str, name: &str) -> Result<ForecastAnalysisMeta, String> {
    validate_analysis_id(id)?;
    let next_name = validate_analysis_name(name)?;
    let mut analysis = load(id).await?;
    analysis.name = next_name;
    save(&analysis).await?;
    Ok(analysis.to_meta())
}

pub async fn list() -> Result<Vec<ForecastAnalysisMeta>, String> {
    let entries = read_index().await?;
    hydrate_index(entries).await
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
        || entries
            .iter()
            .any(|entry| validate_analysis_id(&entry.id).is_err())
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
        tokio::fs::remove_file(analysis_path(&id))
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
