use super::{
    command_spec, parse_login_hints, process_environment, profile_dir, ProcessKind, ProviderId,
};
use std::collections::HashMap;
use std::sync::LazyLock;
use tauri::Emitter;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const MAX_ACTIVE_LOGINS: usize = 2;
const MAX_LOGIN_OUTPUT: usize = 16 * 1024;
const LOGIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10 * 60);
const STATUS_EVENT: &str = "oauth-provider-status-changed";

static ACTIVE: LazyLock<Mutex<HashMap<ProviderId, CancellationToken>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn run(app: tauri::AppHandle, provider: ProviderId) -> Result<(), String> {
    if !matches!(provider, ProviderId::Moonshot | ProviderId::Xai) {
        return Err("Provider OAuth invalide".to_string());
    }
    let cancel = register(provider).await?;
    let result = run_registered(&app, provider, cancel.clone()).await;
    ACTIVE.lock().await.remove(&provider);
    if result.is_ok() {
        super::status::mark_connected(provider)?;
        super::login_progress::emit(&app, provider, "success");
        let _ = app.emit(STATUS_EVENT, ());
    } else if !cancel.is_cancelled() {
        super::login_progress::emit(&app, provider, "error");
    }
    result
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
) -> Result<(), String> {
    let spec = command_spec(provider, ProcessKind::Login);
    let binary = super::status::binary_path(provider)
        .ok_or_else(|| "Client officiel non installé".to_string())?;
    let home = profile_dir(provider);
    tokio::fs::create_dir_all(&home)
        .await
        .map_err(|_| "Connexion impossible".to_string())?;
    tokio::fs::create_dir_all(home.join("agent-home"))
        .await
        .map_err(|_| "Connexion impossible".to_string())?;
    super::logout::remove_credentials_in(&home, provider).await?;
    super::status::mark_connected(provider)?;
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
        .map_err(|_| "Connexion impossible".to_string())?;
    super::login_progress::emit(app, provider, "waiting");
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Connexion impossible".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Connexion impossible".to_string())?;
    let stdout_task = tokio::spawn(read_hints(stdout, app.clone(), provider));
    let stderr_task = tokio::spawn(read_hints(stderr, app.clone(), provider));
    let status = tokio::select! {
        result = child.wait() => result.map_err(|_| "Connexion impossible".to_string())?,
        _ = cancel.cancelled() => {
            let _ = child.kill().await;
            stdout_task.abort();
            stderr_task.abort();
            super::login_progress::emit(app, provider, "cancelled");
            return Err("Connexion annulée".to_string());
        },
        _ = tokio::time::sleep(LOGIN_TIMEOUT) => {
            let _ = child.kill().await;
            stdout_task.abort();
            stderr_task.abort();
            return Err("Connexion impossible".to_string());
        }
    };
    let _ = tokio::join!(stdout_task, stderr_task);
    if status.success() && super::status::credentials_present_in(&home, provider) {
        Ok(())
    } else {
        Err("Connexion impossible".to_string())
    }
}

async fn read_hints<R: AsyncRead + Unpin>(
    mut output: R,
    app: tauri::AppHandle,
    provider: ProviderId,
) {
    let mut collected = Vec::new();
    let mut chunk = [0u8; 1024];
    let mut last_hints = None;
    while collected.len() < MAX_LOGIN_OUTPUT {
        let remaining = MAX_LOGIN_OUTPUT - collected.len();
        let read_len = remaining.min(chunk.len());
        let Ok(count) = output.read(&mut chunk[..read_len]).await else {
            break;
        };
        if count == 0 {
            break;
        }
        collected.extend_from_slice(&chunk[..count]);
        let raw = String::from_utf8_lossy(&collected);
        let hints = parse_login_hints(&raw);
        if hints != Default::default() && Some(&hints) != last_hints.as_ref() {
            super::login_progress::emit_verification(&app, provider, &hints);
            last_hints = Some(hints);
        }
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
