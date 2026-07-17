use super::{command_spec, process_environment, profile_dir, ProcessKind, ProviderId};
use std::collections::HashMap;
use std::sync::LazyLock;
use tauri::Emitter;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const MAX_ACTIVE_LOGINS: usize = 2;
const LOGIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10 * 60);
const STATUS_EVENT: &str = "oauth-provider-status-changed";

static ACTIVE: LazyLock<Mutex<HashMap<ProviderId, CancellationToken>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

enum LoginFailure {
    AccountAccessRequired,
    Generic,
}

pub async fn run(app: tauri::AppHandle, provider: ProviderId) -> Result<(), String> {
    if !matches!(provider, ProviderId::Moonshot | ProviderId::Xai) {
        return Err("Provider OAuth invalide".to_string());
    }
    let cancel = register(provider).await?;
    let result = run_registered(&app, provider, cancel.clone()).await;
    ACTIVE.lock().await.remove(&provider);
    match &result {
        Ok(()) => {
            super::status::mark_connected(provider)?;
            super::login_progress::emit(&app, provider, "success");
            let _ = app.emit(STATUS_EVENT, ());
        }
        Err(LoginFailure::AccountAccessRequired) if !cancel.is_cancelled() => {
            super::login_progress::emit(&app, provider, "account_access_required");
        }
        Err(LoginFailure::Generic) if !cancel.is_cancelled() => {
            super::login_progress::emit(&app, provider, "error");
        }
        Err(_) => {}
    }
    result.map_err(|_| "Connexion impossible".to_string())
}

async fn register(provider: ProviderId) -> Result<CancellationToken, String> {
    let mut active = ACTIVE.lock().await;
    if active.contains_key(&provider) || active.len() >= MAX_ACTIVE_LOGINS {
        return Err("Connexion déjà en cours".to_string());
    }
    let cancel = CancellationToken::new();
    active.insert(provider, cancel.clone());
    Ok(cancel)
}

async fn run_registered(
    app: &tauri::AppHandle,
    provider: ProviderId,
    cancel: CancellationToken,
) -> Result<(), LoginFailure> {
    let spec = command_spec(provider, ProcessKind::Login);
    let binary = super::status::binary_path(provider).ok_or(LoginFailure::Generic)?;
    let home = profile_dir(provider);
    tokio::fs::create_dir_all(&home)
        .await
        .map_err(|_| LoginFailure::Generic)?;
    tokio::fs::create_dir_all(home.join("agent-home"))
        .await
        .map_err(|_| LoginFailure::Generic)?;
    super::logout::remove_credentials_in(&home, provider)
        .await
        .map_err(|_| LoginFailure::Generic)?;
    super::status::mark_connected(provider).map_err(|_| LoginFailure::Generic)?;
    let mut command = Command::new(binary);
    for (name, value) in process_environment(provider) {
        command.env(name, value);
    }
    let mut child = command
        .args(spec.args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| LoginFailure::Generic)?;
    super::login_progress::emit(app, provider, "waiting");
    let stdout = child.stdout.take().ok_or(LoginFailure::Generic)?;
    let stderr = child.stderr.take().ok_or(LoginFailure::Generic)?;
    let stdout_task = tokio::spawn(super::login_output::read(stdout, app.clone(), provider));
    let stderr_task = tokio::spawn(super::login_output::read(stderr, app.clone(), provider));
    let status = tokio::select! {
        result = child.wait() => result.map_err(|_| LoginFailure::Generic)?,
        _ = cancel.cancelled() => {
            let _ = child.kill().await;
            stdout_task.abort();
            stderr_task.abort();
            super::login_progress::emit(app, provider, "cancelled");
            return Err(LoginFailure::Generic);
        },
        _ = tokio::time::sleep(LOGIN_TIMEOUT) => {
            let _ = child.kill().await;
            stdout_task.abort();
            stderr_task.abort();
            return Err(LoginFailure::Generic);
        }
    };
    let (stdout_summary, stderr_summary) = tokio::join!(stdout_task, stderr_task);
    let account_access_required = stdout_summary
        .map(|summary| summary.account_access_required)
        .unwrap_or(false)
        || stderr_summary
            .map(|summary| summary.account_access_required)
            .unwrap_or(false);
    if account_access_required {
        return Err(LoginFailure::AccountAccessRequired);
    }
    if status.success() && super::status::credentials_present_in(&home, provider) {
        Ok(())
    } else {
        Err(LoginFailure::Generic)
    }
}

pub async fn cancel(provider: ProviderId) {
    let token = { ACTIVE.lock().await.get(&provider).cloned() };
    if let Some(token) = token {
        token.cancel();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if !ACTIVE.lock().await.contains_key(&provider) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await;
    }
}

pub async fn cancel_all() {
    let tokens = ACTIVE.lock().await.values().cloned().collect::<Vec<_>>();
    for token in tokens {
        token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancel_waits_for_the_previous_login_to_release_its_slot() {
        let provider = ProviderId::OpenAi;
        let token = register(provider).await.expect("login slot");
        let cleanup = tokio::spawn(async move {
            token.cancelled().await;
            tokio::time::sleep(std::time::Duration::from_millis(30)).await;
            ACTIVE.lock().await.remove(&provider);
        });

        let started = std::time::Instant::now();
        cancel(provider).await;

        assert!(!ACTIVE.lock().await.contains_key(&provider));
        assert!(started.elapsed() < std::time::Duration::from_millis(500));
        cleanup.await.expect("cleanup task");
    }
}
