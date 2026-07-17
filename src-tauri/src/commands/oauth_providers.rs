use crate::services::oauth_providers::{self, ProviderId};
use serde::Serialize;
use tauri::Emitter;

const STATUS_EVENT: &str = "oauth-provider-status-changed";
const PROGRESS_EVENT: &str = "oauth-login-progress";

#[derive(Serialize)]
pub struct OAuthProviderModel {
    pub id: String,
    pub provider_id: ProviderId,
    pub display_name: String,
    pub context_length: Option<u32>,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub reasoning_modes: Vec<String>,
}

#[tauri::command]
pub async fn list_oauth_provider_statuses() -> Vec<oauth_providers::OAuthProviderStatus> {
    oauth_providers::list_statuses()
}

#[tauri::command]
pub async fn start_oauth_provider_login(
    app: tauri::AppHandle,
    provider_id: String,
) -> Result<(), String> {
    let provider = ProviderId::parse(&provider_id)?;
    match provider {
        ProviderId::OpenAi => {
            emit_openai_progress(&app, "waiting");
            if crate::commands::codex_login(app.clone()).await.is_err() {
                emit_openai_progress(&app, "error");
                return Err("Connexion impossible".to_string());
            }
            emit_openai_progress(&app, "success");
        }
        ProviderId::Moonshot | ProviderId::Xai => {
            oauth_providers::login_external(app.clone(), provider).await?;
        }
    }
    let _ = app.emit(STATUS_EVENT, ());
    Ok(())
}

#[tauri::command]
pub async fn cancel_oauth_provider_login(provider_id: String) -> Result<(), String> {
    let provider = ProviderId::parse(&provider_id)?;
    if provider == ProviderId::OpenAi {
        crate::services::codex_oauth::login::cancel_login().await;
    } else {
        oauth_providers::cancel_login(provider).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn disconnect_oauth_provider(
    app: tauri::AppHandle,
    provider_id: String,
) -> Result<(), String> {
    let provider = ProviderId::parse(&provider_id)?;
    match provider {
        ProviderId::OpenAi => crate::commands::codex_logout(app.clone())?,
        ProviderId::Moonshot | ProviderId::Xai => {
            oauth_providers::logout_external(provider).await?;
        }
    }
    let _ = app.emit(STATUS_EVENT, ());
    Ok(())
}

#[tauri::command]
pub async fn list_oauth_provider_models() -> Vec<OAuthProviderModel> {
    let statuses = oauth_providers::list_statuses();
    let mut models = Vec::new();
    if statuses
        .iter()
        .any(|status| status.id == ProviderId::OpenAi && status.connected)
    {
        models.extend(crate::commands::codex_models().into_iter().map(|model| {
            OAuthProviderModel {
                display_name: model.id.clone(),
                id: model.id,
                provider_id: ProviderId::OpenAi,
                context_length: model.context_length,
                supports_tools: model.supports_tools,
                supports_vision: model.supports_vision,
                supports_thinking: model.supports_thinking,
                reasoning_modes: model.reasoning_modes,
            }
        }));
    }
    for (provider_id, id, display_name) in [
        (ProviderId::Moonshot, "kimi-code", "Kimi Code"),
        (ProviderId::Xai, "grok-build", "Grok Build"),
    ] {
        if statuses
            .iter()
            .any(|status| status.id == provider_id && status.connected)
            && crate::services::acp::probe(provider_id).await
        {
            models.push(OAuthProviderModel {
                id: id.to_string(),
                provider_id,
                display_name: display_name.to_string(),
                context_length: None,
                supports_tools: true,
                supports_vision: false,
                supports_thinking: true,
                reasoning_modes: Vec::new(),
            });
        }
    }
    models
}

fn emit_openai_progress(app: &tauri::AppHandle, stage: &'static str) {
    let _ = app.emit(
        PROGRESS_EVENT,
        oauth_providers::OAuthLoginProgress {
            provider_id: ProviderId::OpenAi,
            stage,
            hint: None,
            verification_url: None,
            user_code: None,
        },
    );
}
