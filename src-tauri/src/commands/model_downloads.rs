use crate::services::forecast::validation;
use crate::services::model_downloads::{
    emit_states, run_forecast_download, run_ollama_download, ModelDownloadKind,
    ModelDownloadManager, ModelDownloadState,
};
use tauri::AppHandle;

const MAX_OLLAMA_MODEL_ID_LEN: usize = 200;

#[tauri::command]
pub async fn start_model_download(
    app: AppHandle,
    kind: ModelDownloadKind,
    model_id: String,
    is_update: Option<bool>,
    downloads: tauri::State<'_, ModelDownloadManager>,
) -> Result<ModelDownloadState, String> {
    validate_download_request(kind, &model_id)?;
    let manager = downloads.inner_clone();
    let (state, cancel) = manager
        .start(kind, model_id, is_update.unwrap_or(false))
        .await?;
    emit_states(&app, manager.list().await);
    let app_for_task = app.clone();
    let manager_for_task = manager.clone();
    let state_for_task = state.clone();
    tauri::async_runtime::spawn(async move {
        match state_for_task.kind {
            ModelDownloadKind::Ollama => {
                run_ollama_download(app_for_task, manager_for_task, state_for_task, cancel).await;
            }
            ModelDownloadKind::Forecast => {
                run_forecast_download(app_for_task, manager_for_task, state_for_task, cancel).await;
            }
        }
    });
    Ok(state)
}

#[tauri::command]
pub async fn list_model_downloads(
    downloads: tauri::State<'_, ModelDownloadManager>,
) -> Result<Vec<ModelDownloadState>, String> {
    Ok(downloads.list().await)
}

#[tauri::command]
pub async fn cancel_model_download(
    id: String,
    downloads: tauri::State<'_, ModelDownloadManager>,
) -> Result<(), String> {
    if id.len() > 64 || id.contains("..") {
        return Err("model-download-not-found".into());
    }
    downloads.cancel(&id).await
}

fn validate_download_request(kind: ModelDownloadKind, model_id: &str) -> Result<(), String> {
    match kind {
        ModelDownloadKind::Forecast => validation::validate_model_id(model_id),
        ModelDownloadKind::Ollama => validate_ollama_model_id(model_id),
    }
}

fn validate_ollama_model_id(model_id: &str) -> Result<(), String> {
    if model_id.is_empty() || model_id.len() > MAX_OLLAMA_MODEL_ID_LEN {
        return Err("model-download-invalid-model".into());
    }
    if model_id.contains("..")
        || !model_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | ':' | '/'))
    {
        return Err("model-download-invalid-model".into());
    }
    Ok(())
}
