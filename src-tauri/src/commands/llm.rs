//! Commandes Tauri pour le module LLM multi-provider.

use crate::services::llm::{
    catalog::{ProviderSpec, LLM_PROVIDERS},
    openai_compat::OpenAiCompatProvider,
    tool_capable,
    types::ModelInfo,
};

#[tauri::command]
pub fn list_llm_providers_catalog() -> Vec<ProviderSpec> {
    LLM_PROVIDERS.to_vec()
}

#[tauri::command]
pub async fn list_llm_models(provider_id: String) -> Result<Vec<ModelInfo>, String> {
    let provider = OpenAiCompatProvider::new(&provider_id).map_err(String::from)?;
    let mut models = provider.list_models().await.map_err(String::from)?;
    // Enrichit le flag supports_tools via les patterns hardcodés
    // (l'API ne le renvoie pas pour la plupart des providers)
    for m in &mut models {
        if !m.supports_tools {
            m.supports_tools = tool_capable::supports_tools(&provider_id, &m.id);
        }
    }
    Ok(models)
}

#[tauri::command]
pub async fn test_llm_connection(provider_id: String) -> Result<(), String> {
    let provider = OpenAiCompatProvider::new(&provider_id).map_err(String::from)?;
    provider.test_connection().await.map_err(String::from)
}

#[tauri::command]
pub fn supports_tool_use(provider_id: String, model_id: String) -> bool {
    tool_capable::supports_tools(&provider_id, &model_id)
}
