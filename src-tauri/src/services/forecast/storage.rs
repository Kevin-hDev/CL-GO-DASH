use crate::services::forecast::types::{ForecastAnalysisMeta, ForecastResult};
use crate::services::paths::data_dir;
use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::sync::Mutex;

const MAX_ANALYSES: usize = 500;

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

pub async fn ensure_dir() -> Result<(), String> {
    tokio::fs::create_dir_all(analyses_dir())
        .await
        .map_err(|_| "Impossible de créer le dossier forecast".into())
}

pub async fn save(result: &ForecastResult) -> Result<(), String> {
    validate_analysis_id(&result.id)?;
    ensure_dir().await?;
    let json =
        serde_json::to_string_pretty(result).map_err(|_| "Erreur de sérialisation".to_string())?;

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
    serde_json::from_str(&data).map_err(|_| "Données d'analyse corrompues".to_string())
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
    match tokio::fs::read_to_string(&path).await {
        Ok(data) => serde_json::from_str(&data).map_err(|_| "Index forecast corrompu".into()),
        Err(_) => Ok(Vec::new()),
    }
}

async fn write_index(entries: &[ForecastAnalysisMeta]) -> Result<(), String> {
    ensure_dir().await?;
    let json =
        serde_json::to_string_pretty(entries).map_err(|_| "Index forecast invalide".to_string())?;

    let dir = analyses_dir();
    let tmp = dir.join(".index.tmp");
    let target = index_path();

    tokio::fs::write(&tmp, &json)
        .await
        .map_err(|_| "Écriture index forecast échouée".to_string())?;
    tokio::fs::rename(&tmp, &target)
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
    let path = analysis_path(id);
    let data = tokio::fs::read_to_string(&path)
        .await
        .map_err(|_| "Analyse introuvable".to_string())?;
    let analysis: ForecastResult =
        serde_json::from_str(&data).map_err(|_| "Données d'analyse corrompues".to_string())?;
    Ok(analysis.scenarios.len())
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
