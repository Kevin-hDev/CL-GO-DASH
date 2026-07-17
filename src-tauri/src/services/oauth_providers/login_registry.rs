use super::ProviderId;
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const MAX_ACTIVE_LOGINS: usize = 2;

static ACTIVE: LazyLock<Mutex<HashMap<ProviderId, CancellationToken>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn register(provider: ProviderId) -> Result<CancellationToken, String> {
    let mut active = ACTIVE.lock().await;
    if active.contains_key(&provider) || active.len() >= MAX_ACTIVE_LOGINS {
        return Err("Connexion déjà en cours".to_string());
    }
    let cancel = CancellationToken::new();
    active.insert(provider, cancel.clone());
    Ok(cancel)
}

pub async fn release(provider: ProviderId) {
    ACTIVE.lock().await.remove(&provider);
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
            release(provider).await;
        });

        let started = std::time::Instant::now();
        cancel(provider).await;

        assert!(!ACTIVE.lock().await.contains_key(&provider));
        assert!(started.elapsed() < std::time::Duration::from_millis(500));
        cleanup.await.expect("cleanup task");
    }
}
