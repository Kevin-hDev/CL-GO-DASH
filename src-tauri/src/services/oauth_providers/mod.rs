mod client_compatibility;
mod legacy_kimi_profile;
mod login;
mod login_diagnostics;
#[cfg(test)]
mod login_diagnostics_tests;
mod login_output;
mod login_progress;
mod login_registry;
mod login_wait;
mod logout;
mod specs;
mod status;
mod types;

#[cfg(test)]
pub(crate) use logout::remove_credentials_in;
#[cfg(test)]
pub use specs::profile_env_names;
pub use specs::{
    command_spec, parse_login_hints, process_environment, profile_dir, LoginHints, ProcessKind,
    ProviderId,
};
pub(crate) use status::compatible_binary_path;
#[cfg(test)]
pub(crate) use status::credentials_present_in;
pub(crate) use status::is_connected;
pub use status::list_statuses;
pub use types::{OAuthClientState, OAuthLoginProgress, OAuthProviderStatus};

pub async fn login_external(
    app: tauri::AppHandle,
    provider: ProviderId,
    diagnostic_id: &str,
) -> Result<(), String> {
    let diagnostic = login_diagnostics::LoginDiagnostic::from_ui(provider, diagnostic_id)?;
    diagnostic.stage("command_received");
    login::run(app, provider, diagnostic).await
}

pub async fn cancel_login(provider: ProviderId) {
    login_registry::cancel(provider).await;
}

pub async fn cancel_all() {
    login_registry::cancel_all().await;
}

pub async fn logout_external(provider: ProviderId) -> Result<(), String> {
    logout::run(provider).await
}

pub fn invalidate_external_login(provider: ProviderId) {
    let _ = status::mark_invalid(provider);
}

#[cfg(test)]
mod tests;
