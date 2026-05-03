use crate::services::codex_client::request;
use crate::services::codex_client::types::{CODEX_EFFORT_LEVELS, CODEX_MODELS};
use crate::services::codex_oauth::{jwt, login, store};
use crate::services::llm::types::ModelInfo;

#[tauri::command]
pub async fn codex_login() -> Result<String, String> {
    login::login().await
}

#[tauri::command]
pub fn codex_logout() -> Result<(), String> {
    login::logout()
}

#[tauri::command]
pub fn codex_status() -> Result<CodexStatus, String> {
    let logged_in = store::is_logged_in();
    let email = if logged_in {
        store::load()?
            .and_then(|t| jwt::extract_claims(&t.access).ok())
            .and_then(|c| c.email)
    } else {
        None
    };
    Ok(CodexStatus { logged_in, email })
}

#[derive(serde::Serialize)]
pub struct CodexStatus {
    pub logged_in: bool,
    pub email: Option<String>,
}

#[tauri::command]
pub fn codex_models() -> Vec<ModelInfo> {
    CODEX_MODELS
        .iter()
        .map(|(id, ctx)| ModelInfo {
            id: id.to_string(),
            owned_by: Some("openai".to_string()),
            context_length: Some(*ctx),
            supports_tools: true,
            supports_vision: false,
            supports_thinking: true,
            is_free: true,
        })
        .collect()
}

#[tauri::command]
pub fn codex_set_effort(level: String) -> Result<(), String> {
    if !CODEX_EFFORT_LEVELS.contains(&level.as_str()) {
        return Err(format!("niveau invalide, valeurs possibles : {:?}", CODEX_EFFORT_LEVELS));
    }
    request::set_effort(&level);
    Ok(())
}

#[tauri::command]
pub fn codex_get_effort() -> String {
    request::get_effort()
}
