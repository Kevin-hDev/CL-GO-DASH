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

pub(crate) fn install_forecast_sidecar(app_handle: &tauri::AppHandle, base: &std::path::Path) {
    use std::fs;

    let target = base.join("forecast-sidecar");
    let resource_base = match app_handle.path().resource_dir() {
        Ok(p) => p.join("resources").join("forecast-sidecar"),
        Err(_) => return,
    };
    if !resource_base.exists() {
        return;
    }
    if let Err(e) = copy_recursive(&resource_base, &target) {
        let _ = fs::remove_dir_all(&target);
        eprintln!("[forecast] sidecar install: {e}");
    }
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
    if src.is_dir() {
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
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
    }
    Ok(())
}
