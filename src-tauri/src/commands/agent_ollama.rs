use crate::services::agent_local::ollama_client::OllamaClient;
use crate::services::agent_local::ollama_registry;
use crate::services::agent_local::ollama_registry_details;
use crate::services::agent_local::translation_cache;
use crate::services::agent_local::translator;
use crate::services::agent_local::types_ollama::{
    ModelInfo, OllamaModel, PullProgress, RegistryModel, RegistryModelDetails, RegistryTag,
};
use crate::PullCancel;
use tauri::ipc::Channel;
use tauri::Emitter;
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn list_ollama_models(
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<Vec<OllamaModel>, String> {
    ollama.list_models().await
}

#[tauri::command]
pub async fn show_ollama_model(
    name: String,
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<ModelInfo, String> {
    ollama.show_model(&name).await
}

#[tauri::command]
pub async fn is_ollama_running(
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<bool, String> {
    Ok(ollama.is_running().await)
}

#[tauri::command]
pub async fn search_ollama_models(query: String) -> Result<Vec<RegistryModel>, String> {
    ollama_registry::search_models(&query).await
}

#[tauri::command]
pub async fn get_registry_model_details(name: String) -> Result<RegistryModelDetails, String> {
    ollama_registry_details::fetch_model_details(&name).await
}

#[tauri::command]
pub async fn list_registry_tags(name: String) -> Result<Vec<RegistryTag>, String> {
    ollama_registry_details::fetch_model_tags(&name).await
}

#[tauri::command]
pub async fn translate_description(
    model_name: String,
    text: String,
    target_lang: String,
    translator_model: Option<String>,
) -> Result<String, String> {
    if let Some(cached) = translation_cache::get_cached(&model_name, &target_lang).await {
        return Ok(cached);
    }
    let translated = translator::translate_text(
        &text,
        &target_lang,
        translator_model.as_deref(),
    )
    .await?;
    translation_cache::set_cached(&model_name, &target_lang, &translated).await?;
    Ok(translated)
}

#[tauri::command]
pub async fn pull_ollama_model(
    app: tauri::AppHandle,
    name: String,
    is_update: bool,
    on_progress: Channel<PullProgress>,
    pull_cancel: tauri::State<'_, PullCancel>,
) -> Result<(), String> {
    let cancel = CancellationToken::new();
    { *pull_cancel.0.lock().await = Some(cancel.clone()); }

    let result = ollama_registry::pull_model(&name, &on_progress, &cancel).await;

    { *pull_cancel.0.lock().await = None; }

    match result {
        Ok(()) => { let _ = app.emit("ollama-models-changed", ()); Ok(()) }
        Err(ref e) if e == "cancelled" => {
            if !is_update {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let cleaned = ollama_registry::cleanup_partial_blobs();
                eprintln!("[pull] cancel {name} — {cleaned} fichiers partiels supprimés");
                let _ = ollama_registry::delete_model(&name).await;
            }
            Err("cancelled".to_string())
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn cancel_pull_ollama_model(
    pull_cancel: tauri::State<'_, PullCancel>,
) -> Result<(), String> {
    if let Some(cancel) = pull_cancel.0.lock().await.take() {
        cancel.cancel();
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_ollama_model(
    app: tauri::AppHandle,
    name: String,
) -> Result<(), String> {
    ollama_registry::delete_model(&name).await?;
    let _ = app.emit("ollama-models-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn get_modelfile(
    name: String,
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<String, String> {
    ollama.get_modelfile(&name).await
}

#[tauri::command]
pub async fn update_modelfile(
    app: tauri::AppHandle,
    name: String,
    content: String,
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<(), String> {
    ollama.update_modelfile(&name, &content).await?;
    let _ = app.emit("modelfile-updated", &name);
    Ok(())
}

#[tauri::command]
pub async fn update_system_prompt(
    app: tauri::AppHandle,
    name: String,
    system: String,
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<(), String> {
    ollama.update_system_prompt(&name, &system).await?;
    let _ = app.emit("modelfile-updated", &name);
    Ok(())
}

#[tauri::command]
pub async fn update_parameters(
    app: tauri::AppHandle,
    name: String,
    parameters: Vec<(String, String)>,
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<(), String> {
    ollama.update_parameters(&name, parameters).await?;
    let _ = app.emit("modelfile-updated", &name);
    Ok(())
}
