mod login;
mod logout;
mod specs;
mod status;
mod types;

pub use specs::{
    command_spec, profile_dir, profile_env_names, sanitize_login_output, ProcessKind, ProviderId,
};
pub(crate) use status::binary_path;
pub use status::list_statuses;
pub use types::{OAuthClientState, OAuthLoginProgress, OAuthProviderStatus};

pub async fn login_external(app: tauri::AppHandle, provider: ProviderId) -> Result<(), String> {
    login::run(app, provider).await
}

pub async fn cancel_login(provider: ProviderId) {
    login::cancel(provider).await;
}

pub async fn cancel_all() {
    login::cancel_all().await;
}

pub async fn logout_external(provider: ProviderId) -> Result<(), String> {
    logout::run(provider).await
}

pub fn invalidate_external_login(provider: ProviderId) {
    let _ = status::mark_disconnected(provider);
}

#[cfg(test)]
mod tests;
