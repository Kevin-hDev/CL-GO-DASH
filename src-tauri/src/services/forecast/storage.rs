use crate::services::forecast::types::{ForecastAnalysisMeta, ForecastResult};
use crate::services::paths::data_dir;
use std::path::PathBuf;
use std::sync::LazyLock;
use regex::Regex;
use tokio::sync::Mutex;

const MAX_ANALYSES: usize = 500;

static INDEX_LOCK: Mutex<()> = Mutex::const_new(());

static ANALYSIS_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-f0-9\-]+$").unwrap());

fn validate_analysis_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 {
        return Err("Identifiant d'analyse invalide".into());
    }
    if !ANALYSIS_ID_REGEX.is_match(id) {
        return Err("Identifiant d'analyse invalide".into());
    }
    Ok(())
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

pub async fn ensure_dir() -> Result<(), String> {
    tokio::fs::create_dir_all(analyses_dir())
        .await
        .map_err(|_| "Impossible de créer le dossier forecast".into())
}

pub async fn save(result: &ForecastResult) -> Result<(), String> {
    validate_analysis_id(&result.id)?;
    ensure_dir().await?;
    let json = serde_json::to_string_pretty(result)
        .map_err(|_| "Erreur de sérialisation".to_string())?;

    let dir = analyses_dir();
    let tmp = dir.join(format!(".{}.tmp", result.id));
    let target = analysis_path(&result.id);

    tokio::fs::write(&tmp, &json)
        .await
        .map_err(|_| "Erreur d'écriture".to_string())?;
    tokio::fs::rename(&tmp, &target)
        .await
        .map_err(|_| "Erreur de sauvegarde".to_string())?;

    upsert_index(result.to_meta()).await
}

pub async fn load(id: &str) -> Result<ForecastResult, String> {
    validate_analysis_id(id)?;
    let path = analysis_path(id);
    let data = tokio::fs::read_to_string(&path)
        .await
        .map_err(|_| "Analyse introuvable".to_string())?;
    serde_json::from_str(&data)
        .map_err(|_| "Données d'analyse corrompues".to_string())
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

pub async fn list() -> Result<Vec<ForecastAnalysisMeta>, String> {
    read_index().await
}

async fn read_index() -> Result<Vec<ForecastAnalysisMeta>, String> {
    let path = index_path();
    match tokio::fs::read_to_string(&path).await {
        Ok(data) => serde_json::from_str(&data)
            .map_err(|_| "Index forecast corrompu".into()),
        Err(_) => Ok(Vec::new()),
    }
}

async fn write_index(entries: &[ForecastAnalysisMeta]) -> Result<(), String> {
    ensure_dir().await?;
    let json = serde_json::to_string_pretty(entries)
        .map_err(|e| format!("Sérialisation index: {e}"))?;

    let dir = analyses_dir();
    let tmp = dir.join(".index.tmp");
    let target = index_path();

    tokio::fs::write(&tmp, &json)
        .await
        .map_err(|e| format!("Écriture index tmp: {e}"))?;
    tokio::fs::rename(&tmp, &target)
        .await
        .map_err(|e| format!("Rename index: {e}"))
}

async fn upsert_index(meta: ForecastAnalysisMeta) -> Result<(), String> {
    let _guard = INDEX_LOCK.lock().await;
    let mut entries = read_index().await.unwrap_or_default();
    if let Some(pos) = entries.iter().position(|e| e.id == meta.id) {
        entries[pos] = meta;
    } else {
        entries.push(meta);
        // Borner la collection : supprimer les plus anciennes si dépassement
        if entries.len() > MAX_ANALYSES {
            entries.drain(0..entries.len() - MAX_ANALYSES);
        }
    }
    write_index(&entries).await
}

async fn remove_from_index(id: &str) -> Result<(), String> {
    let _guard = INDEX_LOCK.lock().await;
    let mut entries = read_index().await.unwrap_or_default();
    entries.retain(|e| e.id != id);
    write_index(&entries).await
}
