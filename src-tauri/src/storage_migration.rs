use crate::services::paths::data_dir;

pub fn run(app_handle: &tauri::AppHandle) -> Result<(), String> {
    use std::fs;

    let new = data_dir();

    fs::create_dir_all(new.join("logs")).map_err(|e| {
        eprintln!("[migration] logs dir: {e}");
        "Erreur d'initialisation des données".to_string()
    })?;

    #[cfg(not(target_os = "windows"))]
    {
        let home = dirs::home_dir().ok_or("cannot resolve home")?;

        let cl_go_legacy = home.join(".local/share/cl-go");
        let legacy_marker = new.join(".migrated-from-cl-go");
        if !legacy_marker.exists() && cl_go_legacy.exists() {
            crate::storage_migration_files::copy_items(&cl_go_legacy, &new);
            let _ = fs::write(&legacy_marker, b"ok");
        }
    }

    #[cfg(target_os = "macos")]
    {
        let app_support_wrong = dirs::data_local_dir().and_then(|d| {
            let p = d.join("cl-go-dash");
            if p != new {
                Some(p)
            } else {
                None
            }
        });
        let appsupport_marker = new.join(".migrated-from-appsupport");
        if let Some(wrong) = app_support_wrong {
            if !appsupport_marker.exists() && wrong.exists() {
                crate::storage_migration_files::copy_items(&wrong, &new);
                let _ = fs::write(&appsupport_marker, b"ok");
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = dirs::data_dir().map(|d| d.join("cl-go-dash"));
        let win_marker = new.join(".migrated-from-appdata");
        if let Some(old) = appdata {
            if !win_marker.exists() && old.exists() {
                crate::storage_migration_files::copy_items(&old, &new);
                let _ = fs::write(&win_marker, b"ok");
            }
        }
    }

    init_base_structure(&new)?;
    crate::storage_migration_files::install_default_skills(app_handle, &new);
    crate::storage_migration_files::install_forecast_sidecar(app_handle, &new);

    Ok(())
}

fn init_base_structure(base: &std::path::Path) -> Result<(), String> {
    use std::fs;

    let dirs = [
        "memory/core",
        "memory/archive",
        "memory/episodes",
        "memory/hypotheses",
        "memory/knowledge",
        "memory/procedures",
        "inbox",
        "skills",
        "agent-sessions",
        "tool-results",
        "translations",
        "logs",
    ];
    for d in &dirs {
        fs::create_dir_all(base.join(d)).map_err(|e| {
            eprintln!("[migration] create {d}: {e}");
            "Erreur d'initialisation des données".to_string()
        })?;
    }

    let json_defaults: &[(&str, &str)] = &[
        ("config.json", "{}"),
        ("agent-settings.json", "{\"permissionMode\":\"auto\"}"),
        ("agent-tabs.json", "[]"),
        ("configured-providers.json", "[]"),
        ("favorite-models.json", "[]"),
        ("projects.json", "[]"),
        ("terminal-tabs.json", "[]"),
        ("inbox/pending.json", "[]"),
        (
            "personality-injection.json",
            "{\
                \"identity.md\":false,\
                \"principles.md\":false,\
                \"user.md\":false,\
                \"idea-discovery.md\":false\
            }",
        ),
    ];
    for (name, content) in json_defaults {
        let path = base.join(name);
        if !path.exists() {
            fs::write(&path, content).map_err(|e| {
                eprintln!("[migration] write {name}: {e}");
                "Erreur d'initialisation des données".to_string()
            })?;
        }
    }

    let empty_files = [
        "AGENTS.md",
        "memory/core/identity.md",
        "memory/core/principles.md",
        "memory/core/user.md",
        "memory/archive/INDEX.md",
        "memory/episodes/INDEX.md",
        "memory/hypotheses/INDEX.md",
        "memory/knowledge/INDEX.md",
        "memory/procedures/INDEX.md",
        "memory/explorer-log.yaml",
        "inbox/idea-discovery.md",
    ];
    for name in &empty_files {
        let path = base.join(name);
        if !path.exists() {
            fs::write(&path, b"").map_err(|e| {
                eprintln!("[migration] write {name}: {e}");
                "Erreur d'initialisation des données".to_string()
            })?;
        }
    }

    Ok(())
}
