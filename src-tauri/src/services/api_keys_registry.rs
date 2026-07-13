fn registry_path() -> std::path::PathBuf {
    crate::services::paths::data_dir().join("configured-providers.json")
}

fn read_registry() -> Vec<String> {
    let path = registry_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn write_registry(ids: &[String]) -> Result<(), String> {
    let path = registry_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| "erreur de stockage".to_string())?;
    }
    let json = serde_json::to_string_pretty(ids).map_err(|_| "erreur de stockage".to_string())?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &json).map_err(|_| "erreur de stockage".to_string())?;
    std::fs::rename(&tmp, &path).map_err(|_| "erreur de stockage".to_string())?;
    Ok(())
}

fn add_to_registry(provider_id: &str) -> Result<(), String> {
    let mut ids = read_registry();
    if !ids.iter().any(|id| id == provider_id) {
        ids.push(provider_id.to_string());
        write_registry(&ids)?;
    }
    Ok(())
}

fn remove_from_registry(provider_id: &str) -> Result<(), String> {
    let mut ids = read_registry();
    let before = ids.len();
    ids.retain(|id| id != provider_id);
    if ids.len() != before {
        write_registry(&ids)?;
    }
    Ok(())
}
