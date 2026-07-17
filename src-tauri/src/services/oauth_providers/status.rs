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
    profile_dir(provider).join(".cl-go-connected")
}

pub fn is_connected(provider: ProviderId) -> bool {
    marker_path(provider).is_file()
}

pub fn mark_connected(provider: ProviderId) -> Result<(), String> {
    let root = profile_dir(provider);
    std::fs::create_dir_all(&root).map_err(|_| "Connexion impossible".to_string())?;
    let target = marker_path(provider);
    let temp = root.join(format!(".connected-{}.tmp", uuid::Uuid::new_v4()));
    std::fs::write(&temp, b"connected").map_err(|_| "Connexion impossible".to_string())?;
    std::fs::rename(temp, target).map_err(|_| "Connexion impossible".to_string())
}

pub fn mark_disconnected(provider: ProviderId) -> Result<(), String> {
    let marker = marker_path(provider);
    match std::fs::remove_file(marker) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Déconnexion impossible".to_string()),
    }
}
