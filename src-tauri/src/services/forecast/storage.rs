use crate::services::forecast::types::{ForecastAnalysisMeta, ForecastResult};
use crate::services::paths::data_dir;
use std::path::PathBuf;
use tokio::sync::Mutex;

static INDEX_LOCK: Mutex<()> = Mutex::const_new(());

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
        .map_err(|e| format!("Impossible de créer le dossier forecast: {e}"))
}

pub async fn save(result: &ForecastResult) -> Result<(), String> {
    ensure_dir().await?;
    let json = serde_json::to_string_pretty(result)
        .map_err(|e| format!("Sérialisation échouée: {e}"))?;

    let dir = analyses_dir();
    let tmp = dir.join(format!(".{}.tmp", result.id));
    let target = analysis_path(&result.id);

    tokio::fs::write(&tmp, &json)
        .await
        .map_err(|e| format!("Écriture tmp échouée: {e}"))?;
    tokio::fs::rename(&tmp, &target)
        .await
        .map_err(|e| format!("Rename échoué: {e}"))?;

    upsert_index(result.to_meta()).await
}

pub async fn load(id: &str) -> Result<ForecastResult, String> {
    let path = analysis_path(id);
    let data = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Lecture échouée: {e}"))?;
    serde_json::from_str(&data)
        .map_err(|e| format!("Parsing échoué: {e}"))
}

pub async fn delete(id: &str) -> Result<(), String> {
    let path = analysis_path(id);
    if path.exists() {
        tokio::fs::remove_file(&path)
            .await
            .map_err(|e| format!("Suppression échouée: {e}"))?;
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
    }
    write_index(&entries).await
}

async fn remove_from_index(id: &str) -> Result<(), String> {
    let _guard = INDEX_LOCK.lock().await;
    let mut entries = read_index().await.unwrap_or_default();
    entries.retain(|e| e.id != id);
    write_index(&entries).await
}
