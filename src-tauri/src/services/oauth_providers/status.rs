use super::{profile_dir, OAuthClientState, OAuthProviderStatus, ProviderId};
use std::path::PathBuf;

const OPENAI_INSTALL: &str = "https://chatgpt.com/";
const KIMI_INSTALL: &str = "https://www.kimi.com/code/docs/en/";
const GROK_INSTALL: &str = "https://docs.x.ai/build/overview";

pub fn list_statuses() -> Vec<OAuthProviderStatus> {
    vec![
        openai_status(),
        external_status(ProviderId::Moonshot),
        external_status(ProviderId::Xai),
    ]
}

fn openai_status() -> OAuthProviderStatus {
    let connected = crate::services::codex_oauth::store::is_logged_in();
    let account = connected
        .then(|| {
            crate::commands::codex_status()
                .ok()
                .and_then(|status| status.email)
        })
        .flatten();
    OAuthProviderStatus {
        id: ProviderId::OpenAi,
        display_name: "OpenAI",
        connected,
        account,
        client_state: OAuthClientState::Ready,
        install_url: OPENAI_INSTALL,
    }
}

fn external_status(provider: ProviderId) -> OAuthProviderStatus {
    let (display_name, install_url) = match provider {
        ProviderId::Moonshot => ("Moonshot AI", KIMI_INSTALL),
        ProviderId::Xai => ("xAI", GROK_INSTALL),
        ProviderId::OpenAi => unreachable!(),
    };
    OAuthProviderStatus {
        id: provider,
        display_name,
        connected: is_connected(provider),
        account: None,
        client_state: if binary_path(provider).is_some() {
            OAuthClientState::Ready
        } else {
            OAuthClientState::Missing
        },
        install_url,
    }
}

pub fn binary_path(provider: ProviderId) -> Option<PathBuf> {
    let program = super::command_spec(provider, super::ProcessKind::Acp).program;
    if program.is_empty() {
        return None;
    }
    which::which(program)
        .ok()?
        .canonicalize()
        .ok()
        .filter(|path| path.is_file())
}

fn marker_path(provider: ProviderId) -> PathBuf {
    profile_dir(provider).join(".cl-go-invalid")
}

pub fn is_connected(provider: ProviderId) -> bool {
    credentials_present_in(&profile_dir(provider), provider) && !marker_path(provider).is_file()
}

pub fn mark_connected(provider: ProviderId) -> Result<(), String> {
    remove_marker(marker_path(provider), "Connexion impossible")
}

pub fn mark_disconnected(provider: ProviderId) -> Result<(), String> {
    remove_marker(marker_path(provider), "Déconnexion impossible")?;
    remove_marker(
        profile_dir(provider).join(".cl-go-connected"),
        "Déconnexion impossible",
    )
}

pub fn mark_invalid(provider: ProviderId) -> Result<(), String> {
    let root = profile_dir(provider);
    std::fs::create_dir_all(&root).map_err(|_| "Connexion impossible".to_string())?;
    std::fs::write(marker_path(provider), b"invalid")
        .map_err(|_| "Connexion impossible".to_string())
}

fn remove_marker(marker: PathBuf, message: &str) -> Result<(), String> {
    match std::fs::remove_file(marker) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(message.to_string()),
    }
}

pub fn credentials_present_in(root: &std::path::Path, provider: ProviderId) -> bool {
    match provider {
        ProviderId::OpenAi => false,
        ProviderId::Xai => root.join("auth.json").is_file(),
        ProviderId::Moonshot => kimi_credentials_present(root),
    }
}

fn kimi_credentials_present(root: &std::path::Path) -> bool {
    let Ok(entries) = std::fs::read_dir(root.join("credentials")) else {
        return false;
    };
    for entry in entries.take(32).flatten() {
        let is_file = entry
            .file_type()
            .map(|kind| kind.is_file())
            .unwrap_or(false);
        let is_json = entry.path().extension().and_then(|ext| ext.to_str()) == Some("json");
        if is_file && is_json {
            return true;
        }
    }
    false
}
