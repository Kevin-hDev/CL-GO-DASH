use crate::services::forecast::types::ModelDownloadProgress;
use crate::services::forecast::{catalog, validation};
use crate::services::paths::data_dir;
use std::path::PathBuf;
use tauri::ipc::Channel;

pub mod download;
pub mod download_github;

fn models_dir() -> PathBuf {
    data_dir().join("forecast-models")
}

pub fn model_path(model_id: &str) -> PathBuf {
    models_dir().join(model_id)
}

pub fn is_installed(model_id: &str) -> bool {
    if validation::validate_model_id(model_id).is_err() {
        return false;
    }
    model_path(model_id).join(".complete").exists()
}

pub async fn install(
    model_id: &str,
    on_progress: &Channel<ModelDownloadProgress>,
) -> Result<(), String> {
    validation::validate_model_id(model_id)?;
    let spec =
        catalog::find_model(model_id).ok_or_else(|| format!("Modèle inconnu: {model_id}"))?;

    let hf_repo = spec.hf_repo;
    let github_repo = spec.github_repo;

    let target_dir = model_path(model_id);
    let staging_dir = models_dir().join(format!(".{model_id}.staging"));
    let _ = tokio::fs::remove_dir_all(&staging_dir).await;
    tokio::fs::create_dir_all(&staging_dir)
        .await
        .map_err(|_| "Impossible de préparer l'installation".to_string())?;

    let download_result = if let Some(repo) = hf_repo {
        download::download_model(repo, spec.hf_revision, &staging_dir, model_id, on_progress).await
    } else if let Some(repo) = github_repo {
        download_github::download_repo_snapshot(
            repo,
            spec.github_revision,
            &staging_dir,
            model_id,
            on_progress,
        )
        .await
    } else {
        Err("Ce modèle n'a pas de source téléchargeable".to_string())
    };

    if let Err(e) = download_result {
        let _ = tokio::fs::remove_dir_all(&staging_dir).await;
        return Err(e);
    }
    tokio::fs::write(staging_dir.join(".complete"), b"ok")
        .await
        .map_err(|_| "Validation installation échouée".to_string())?;
    let _ = tokio::fs::remove_dir_all(&target_dir).await;
    tokio::fs::rename(&staging_dir, &target_dir)
        .await
        .map_err(|_| "Finalisation installation échouée".to_string())
}

pub async fn uninstall(model_id: &str) -> Result<(), String> {
    validation::validate_model_id(model_id)?;
    let path = model_path(model_id);
    if path.exists() {
        tokio::fs::remove_dir_all(&path)
            .await
            .map_err(|_| "Suppression échouée".to_string())?;
    }
    Ok(())
}

pub fn get_model_size(model_id: &str) -> u64 {
    if validation::validate_model_id(model_id).is_err() {
        return 0;
    }
    let path = model_path(model_id);
    if !path.exists() {
        return 0;
    }
    walkdir_size(&path)
}

fn walkdir_size(path: &PathBuf) -> u64 {
    std::fs::read_dir(path)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| {
                    if e.path().is_dir() {
                        walkdir_size(&e.path())
                    } else {
                        e.metadata().map(|m| m.len()).unwrap_or(0)
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}
