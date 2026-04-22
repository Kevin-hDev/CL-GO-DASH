/// One-shot migration: copie depuis l'ancien dossier ~/.local/share/cl-go/
/// (utilisé par CL-GO) vers ~/.local/share/cl-go-dash/ au premier démarrage
/// de cette nouvelle version. L'ancien dossier est laissé intact (CL-GO continue
/// d'y écrire). N'écrase rien si le nouveau dossier existe déjà.
pub fn run() -> Result<(), String> {
    use std::fs;

    let home = dirs::home_dir().ok_or("cannot resolve home")?;
    let new = home.join(".local/share/cl-go-dash");

    fs::create_dir_all(new.join("logs"))
        .map_err(|e| format!("create logs dir: {}", e))?;

    // Migration 1 : ~/.local/share/cl-go/ (legacy CL-GO)
    let cl_go_legacy = home.join(".local/share/cl-go");
    let legacy_marker = new.join(".migrated-from-cl-go");
    if !legacy_marker.exists() && cl_go_legacy.exists() {
        copy_items(&cl_go_legacy, &new);
        let _ = fs::write(&legacy_marker, b"ok");
    }

    // Migration 2 : ~/Library/Application Support/cl-go-dash/ (bug Phase 2 macOS)
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
