use super::{LoginHints, OAuthLoginProgress, ProviderId};
use tauri::Emitter;

const PROGRESS_EVENT: &str = "oauth-login-progress";

pub fn emit(app: &tauri::AppHandle, provider: ProviderId, stage: &'static str) {
    let _ = app.emit(
        PROGRESS_EVENT,
        OAuthLoginProgress {
            provider_id: provider,
            stage,
            hint: None,
            verification_url: None,
            user_code: None,
        },
    );
}

pub fn emit_verification(app: &tauri::AppHandle, provider: ProviderId, hints: &LoginHints) {
    let hint = match (&hints.verification_url, &hints.user_code) {
        (Some(url), Some(code)) => Some(format!("{url}\n{code}")),
        (Some(url), None) => Some(url.clone()),
        (None, Some(code)) => Some(code.clone()),
        (None, None) => None,
    };
    let _ = app.emit(
        PROGRESS_EVENT,
        OAuthLoginProgress {
            provider_id: provider,
            stage: "verification",
            hint,
            verification_url: hints.verification_url.clone(),
            user_code: hints.user_code.clone(),
        },
    );
}
