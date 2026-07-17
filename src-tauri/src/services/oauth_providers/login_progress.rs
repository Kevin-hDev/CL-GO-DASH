use super::{login_diagnostics::LoginDiagnostic, LoginHints, OAuthLoginProgress, ProviderId};
use tauri::Emitter;

const PROGRESS_EVENT: &str = "oauth-login-progress";

pub fn emit(
    app: &tauri::AppHandle,
    provider: ProviderId,
    stage: &'static str,
    diagnostic: &LoginDiagnostic,
) {
    let emitted = app
        .emit(
            PROGRESS_EVENT,
            OAuthLoginProgress {
                provider_id: provider,
                stage,
                hint: None,
                verification_url: None,
                user_code: None,
            },
        )
        .is_ok();
    diagnostic.progress(stage, emitted);
}

pub fn emit_verification(
    app: &tauri::AppHandle,
    provider: ProviderId,
    hints: &LoginHints,
    diagnostic: &LoginDiagnostic,
) {
    let hint = match (&hints.verification_url, &hints.user_code) {
        (Some(url), Some(code)) => Some(format!("{url}\n{code}")),
        (Some(url), None) => Some(url.clone()),
        (None, Some(code)) => Some(code.clone()),
        (None, None) => None,
    };
    let emitted = app
        .emit(
            PROGRESS_EVENT,
            OAuthLoginProgress {
                provider_id: provider,
                stage: "verification",
                hint,
                verification_url: hints.verification_url.clone(),
                user_code: hints.user_code.clone(),
            },
        )
        .is_ok();
    diagnostic.progress("verification", emitted);
}
