use super::{profile_dir, ProviderId};

pub async fn run(provider: ProviderId) -> Result<(), String> {
    match provider {
        ProviderId::Moonshot | ProviderId::Xai => {
            remove_credentials_in(&profile_dir(provider), provider).await?
        }
        ProviderId::OpenAi => return Err("Provider OAuth invalide".to_string()),
    }
    super::status::mark_disconnected(provider)
}

pub(crate) async fn remove_credentials_in(
    root: &std::path::Path,
    provider: ProviderId,
) -> Result<(), String> {
    tokio::fs::create_dir_all(&root)
        .await
        .map_err(|_| "Déconnexion impossible".to_string())?;
    let root = root
        .canonicalize()
        .map_err(|_| "Déconnexion impossible".to_string())?;
    let target = match provider {
        ProviderId::Moonshot => root.join("credentials"),
        ProviderId::Xai => root.join("auth.json"),
        ProviderId::OpenAi => return Err("Provider OAuth invalide".to_string()),
    };
    if !target.exists() {
        return Ok(());
    }
    let resolved = target
        .canonicalize()
        .map_err(|_| "Déconnexion impossible".to_string())?;
    if !resolved.starts_with(&root) {
        return Err("Déconnexion impossible".to_string());
    }
    if provider == ProviderId::Moonshot {
        tokio::fs::remove_dir_all(resolved).await
    } else {
        tokio::fs::remove_file(resolved).await
    }
    .map_err(|_| "Déconnexion impossible".to_string())
}
