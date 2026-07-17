use std::sync::LazyLock;

use tokio::sync::Mutex;

use super::{kimi, store, xai, AccessToken, LlmOAuthProvider, OAuthFailure};

static REFRESH_LOCKS: LazyLock<[Mutex<()>; 2]> = LazyLock::new(|| [Mutex::new(()), Mutex::new(())]);

pub async fn access_token(provider: LlmOAuthProvider) -> Result<AccessToken, String> {
    let tokens = store::load(provider)?.ok_or_else(not_connected)?;
    let generation = store::generation(provider);
    if tokens.is_fresh() {
        return Ok(AccessToken {
            value: tokens.access,
            generation,
        });
    }
    refresh_locked(provider, generation).await
}

pub async fn force_refresh(
    provider: LlmOAuthProvider,
    used_generation: u64,
) -> Result<AccessToken, String> {
    refresh_locked(provider, used_generation).await
}

async fn refresh_locked(
    provider: LlmOAuthProvider,
    expected_generation: u64,
) -> Result<AccessToken, String> {
    let _guard = REFRESH_LOCKS[provider.index()].lock().await;
    let current = store::load(provider)?.ok_or_else(not_connected)?;
    let current_generation = store::generation(provider);
    if current_generation != expected_generation && current.is_fresh() {
        return Ok(AccessToken {
            value: current.access,
            generation: current_generation,
        });
    }
    let refreshed = match provider {
        LlmOAuthProvider::Xai => xai::refresh(&current.refresh).await,
        LlmOAuthProvider::Kimi => kimi::refresh(&current.refresh).await,
    };
    match refreshed {
        Ok(tokens) => {
            let generation = store::save(provider, &tokens)?;
            Ok(AccessToken {
                value: tokens.access,
                generation,
            })
        }
        Err(OAuthFailure::Unauthorized) => {
            let _ = store::clear(provider);
            Err(not_connected())
        }
        Err(_) => Err("Renouvellement impossible".to_string()),
    }
}

fn not_connected() -> String {
    "Connexion requise".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[tokio::test]
    async fn same_provider_refreshes_are_serialized() {
        let first = REFRESH_LOCKS[LlmOAuthProvider::Xai.index()].lock().await;
        let entered = std::sync::Arc::new(AtomicBool::new(false));
        let observed = entered.clone();
        let waiting = tokio::spawn(async move {
            let _second = REFRESH_LOCKS[LlmOAuthProvider::Xai.index()].lock().await;
            observed.store(true, Ordering::SeqCst);
        });
        tokio::task::yield_now().await;
        assert!(!entered.load(Ordering::SeqCst));
        drop(first);
        waiting.await.unwrap();
        assert!(entered.load(Ordering::SeqCst));
    }
}
