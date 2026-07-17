use std::path::Path;

const LEGACY_MARKERS: [&str; 2] = ["device_id", "kimi.json"];
const LEGACY_FILES: [&str; 3] = ["device_id", "kimi.json", "config.toml"];

pub(super) async fn reset_if_needed(root: &Path) -> Result<bool, String> {
    if !LEGACY_MARKERS.iter().any(|name| root.join(name).exists()) {
        return Ok(false);
    }
    for name in LEGACY_FILES {
        remove_file_if_present(&root.join(name)).await?;
    }
    super::logout::remove_credentials_in(root, super::ProviderId::Moonshot).await?;
    Ok(true)
}

async fn remove_file_if_present(path: &Path) -> Result<(), String> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Connexion impossible".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn removes_only_legacy_profile_state() {
        let root = tempfile::tempdir().expect("temporary Kimi profile");
        std::fs::write(root.path().join("device_id"), b"legacy").expect("legacy marker");
        std::fs::write(root.path().join("config.toml"), b"legacy").expect("legacy config");
        std::fs::write(root.path().join("keep.txt"), b"keep").expect("unrelated file");
        let credentials = root.path().join("credentials");
        std::fs::create_dir(&credentials).expect("credentials directory");
        std::fs::write(credentials.join("kimi-code.json"), b"secret").expect("credential");

        assert!(reset_if_needed(root.path()).await.expect("legacy reset"));
        assert!(!root.path().join("device_id").exists());
        assert!(!root.path().join("config.toml").exists());
        assert!(!credentials.exists());
        assert!(root.path().join("keep.txt").exists());
    }

    #[tokio::test]
    async fn leaves_current_profile_untouched() {
        let root = tempfile::tempdir().expect("temporary Kimi profile");
        std::fs::write(root.path().join("config.toml"), b"current").expect("current config");

        assert!(!reset_if_needed(root.path()).await.expect("current profile"));
        assert!(root.path().join("config.toml").exists());
    }
}
