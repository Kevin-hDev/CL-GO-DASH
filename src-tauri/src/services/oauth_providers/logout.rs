use super::{command_spec, profile_dir, profile_env_names, ProcessKind, ProviderId};
use tokio::process::Command;

const LOGOUT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

pub async fn run(provider: ProviderId) -> Result<(), String> {
    match provider {
        ProviderId::Moonshot => remove_kimi_credentials().await?,
        ProviderId::Xai => run_grok_logout().await?,
        ProviderId::OpenAi => return Err("Provider OAuth invalide".to_string()),
    }
    super::status::mark_disconnected(provider)
}

async fn run_grok_logout() -> Result<(), String> {
    let provider = ProviderId::Xai;
    let spec = command_spec(provider, ProcessKind::Logout);
    let binary = super::status::binary_path(provider)
        .ok_or_else(|| "Client officiel non installé".to_string())?;
    let mut command = Command::new(binary);
    for name in profile_env_names(provider) {
        command.env(name, profile_dir(provider));
    }
    let status = tokio::time::timeout(
        LOGOUT_TIMEOUT,
        command
            .args(spec.args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .status(),
    )
    .await
    .map_err(|_| "Déconnexion impossible".to_string())?
    .map_err(|_| "Déconnexion impossible".to_string())?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| "Déconnexion impossible".to_string())
}

async fn remove_kimi_credentials() -> Result<(), String> {
    let root = profile_dir(ProviderId::Moonshot);
    tokio::fs::create_dir_all(&root)
        .await
        .map_err(|_| "Déconnexion impossible".to_string())?;
    let root = root
        .canonicalize()
        .map_err(|_| "Déconnexion impossible".to_string())?;
    let credentials = root.join("credentials");
    if !credentials.exists() {
        return Ok(());
    }
    let resolved = credentials
        .canonicalize()
        .map_err(|_| "Déconnexion impossible".to_string())?;
    if !resolved.starts_with(&root) {
        return Err("Déconnexion impossible".to_string());
    }
    tokio::fs::remove_dir_all(resolved)
        .await
        .map_err(|_| "Déconnexion impossible".to_string())
}
