use crate::services::paths::data_dir;

pub fn run() -> Result<(), String> {
    use std::fs;

    let new = data_dir();

    fs::create_dir_all(new.join("logs"))
        .map_err(|e| format!("create logs dir: {}", e))?;

    #[cfg(not(target_os = "windows"))]
    {
        let home = dirs::home_dir().ok_or("cannot resolve home")?;

        let cl_go_legacy = home.join(".local/share/cl-go");
        let legacy_marker = new.join(".migrated-from-cl-go");
        if !legacy_marker.exists() && cl_go_legacy.exists() {
            copy_items(&cl_go_legacy, &new);
            let _ = fs::write(&legacy_marker, b"ok");
        }
    }

    #[cfg(target_os = "macos")]
    {
        let app_support_wrong = dirs::data_local_dir().and_then(|d| {
            let p = d.join("cl-go-dash");
            if p != new { Some(p) } else { None }
        });
        let appsupport_marker = new.join(".migrated-from-appsupport");
        if let Some(wrong) = app_support_wrong {
            if !appsupport_marker.exists() && wrong.exists() {
                copy_items(&wrong, &new);
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
                copy_items(&old, &new);
                let _ = fs::write(&win_marker, b"ok");
            }
        }
    }

    init_base_structure(&new)?;

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
        "logs/heartbeat",
    ];
    for d in &dirs {
        fs::create_dir_all(base.join(d))
            .map_err(|e| format!("create {d}: {e}"))?;
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
            fs::write(&path, content)
                .map_err(|e| format!("write {name}: {e}"))?;
        }
    }

    let empty_files = [
        "AGENT.md",
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
            fs::write(&path, b"")
                .map_err(|e| format!("write {name}: {e}"))?;
        }
    }

    Ok(())
}

fn copy_items(src: &std::path::Path, dst: &std::path::Path) {
    let items: &[&str] = &[
        "agent-sessions",
        "agent-settings.json",
        "agent-tabs.json",
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
            eprintln!("[storage migration] {} → {}: {}", s.display(), d.display(), e);
        }
    }
}

fn copy_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    use std::fs;
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            copy_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        }
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
    }
    Ok(())
}
