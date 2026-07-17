mod login;
mod login_output;
mod login_progress;
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
pub(crate) use status::binary_path;
#[cfg(test)]
pub(crate) use status::credentials_present_in;
pub(crate) use status::is_connected;
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
    let _ = status::mark_invalid(provider);
}

#[cfg(test)]
mod tests;
