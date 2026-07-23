use tauri::Manager;

const MAX_COPY_DEPTH: u32 = 10;

pub(crate) fn install_default_skills(app_handle: &tauri::AppHandle, base: &std::path::Path) {
    use std::fs;

    let skills_dir = base.join("skills");
    let resource_base = match app_handle.path().resource_dir() {
        Ok(p) => p.join("default-skills"),
        Err(_) => return,
    };
    if !resource_base.exists() {
        return;
    }

    let entries = match fs::read_dir(&resource_base) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name();
        let target = skills_dir.join(&name);
        if target.exists() {
            continue;
        }
        if let Err(e) = copy_recursive(&entry.path(), &target) {
            eprintln!("[skills] install {:?}: {}", name, e);
        }
    }
}

pub(crate) fn install_forecast_sidecar(
    app_handle: &tauri::AppHandle,
    base: &std::path::Path,
) -> Result<(), String> {
    let target = base.join("forecast-sidecar");
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|_| "Ressources Forecast indisponibles".to_string())?;
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let resource_base = resolve_forecast_resource_base(&resource_dir, manifest_dir)?;
    sync_forecast_sidecar_from(&resource_base, &target)
}

fn resolve_forecast_resource_base(
    resource_dir: &std::path::Path,
    manifest_dir: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    let candidates = [
        resource_dir.join("resources/forecast-sidecar"),
        resource_dir.join("forecast-sidecar"),
        manifest_dir.join("resources/forecast-sidecar"),
    ];
    candidates
        .into_iter()
        .find(|candidate| validate_forecast_assets(candidate).is_ok())
        .ok_or_else(|| "Ressources Forecast incomplètes".to_string())
}

fn sync_forecast_sidecar_from(
    resource_base: &std::path::Path,
    target: &std::path::Path,
) -> Result<(), String> {
    validate_forecast_assets(resource_base)?;
    copy_forecast_runtime(resource_base, target)
        .map_err(|_| "Synchronisation Forecast impossible".to_string())?;
    validate_forecast_assets(target)
}

fn copy_forecast_runtime(
    source: &std::path::Path,
    target: &std::path::Path,
) -> std::io::Result<()> {
    const FILES: [&str; 3] = ["server.py", "test_model_smoke.py", "requirements.txt"];
    for relative in FILES {
        copy_recursive(&source.join(relative), &target.join(relative))?;
    }
    copy_recursive(
        &source.join("forecast_runtime"),
        &target.join("forecast_runtime"),
    )
}

fn validate_forecast_assets(root: &std::path::Path) -> Result<(), String> {
    const REQUIRED: [&str; 4] = [
        "server.py",
        "test_model_smoke.py",
        "requirements.txt",
        "forecast_runtime/adapters.py",
    ];
    REQUIRED
        .iter()
        .all(|relative| root.join(relative).is_file())
        .then_some(())
        .ok_or_else(|| "Ressources Forecast incomplètes".to_string())
}

pub(crate) fn copy_items(src: &std::path::Path, dst: &std::path::Path) {
    let items: &[&str] = &[
        "agent-sessions",
        "agent-settings.json",
        "config.json",
        "memory",
        "inbox",
        "translations",
        "logs",
    ];
    for item in items {
        let s = src.join(item);
        let d = dst.join(item);
        if !s.exists() || d.exists() {
            continue;
        }
        if let Err(e) = copy_recursive(&s, &d) {
            eprintln!(
                "[storage migration] {} -> {}: {}",
                s.display(),
                d.display(),
                e
            );
        }
    }
}

fn copy_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    copy_recursive_inner(src, dst, dst, 0)
}

fn copy_recursive_inner(
    src: &std::path::Path,
    dst: &std::path::Path,
    root_dst: &std::path::Path,
    depth: u32,
) -> std::io::Result<()> {
    use std::fs;
    if depth > MAX_COPY_DEPTH {
        return Err(std::io::Error::other(
            "profondeur de copie maximale dépassée",
        ));
    }
    let metadata = fs::symlink_metadata(src)?;
    if metadata.file_type().is_symlink() {
        return Err(std::io::Error::other("lien symbolique refusé"));
    }
    if metadata.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let target = dst.join(entry.file_name());
            let canonical = fs::canonicalize(dst)?.join(entry.file_name());
            if !canonical.starts_with(fs::canonicalize(root_dst)?) {
                eprintln!("[migration] path traversal bloqué: {}", target.display());
                continue;
            }
            copy_recursive_inner(&entry.path(), &target, root_dst, depth + 1)?;
        }
    } else if metadata.is_file() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
    } else {
        return Err(std::io::Error::other("type de ressource refusé"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "storage_migration_files_tests.rs"]
mod tests;
