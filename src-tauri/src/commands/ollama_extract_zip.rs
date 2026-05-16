use std::path::Path;

pub(crate) fn extract_zip(archive: &Path, dest: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        let status = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    ps_single_quote(archive),
                    ps_single_quote(dest)
                ),
            ])
            .creation_flags(0x08000000)
            .status()
            .map_err(|e| {
                eprintln!("[ollama-extract] powershell: {e}");
                "ollama-extract-error".to_string()
            })?;
        if !status.success() {
            return Err("Expand-Archive failed".into());
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let file = std::fs::File::open(archive).map_err(|e| {
            eprintln!("[ollama-extract] open zip: {e}");
            "ollama-extract-error".to_string()
        })?;
        let mut zip_archive = zip::ZipArchive::new(file).map_err(|e| {
            eprintln!("[ollama-extract] read zip: {e}");
            "ollama-extract-error".to_string()
        })?;
        safe_unpack_zip(&mut zip_archive, dest)?;
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn safe_unpack_zip(
    archive: &mut zip::ZipArchive<std::fs::File>,
    dest: &Path,
) -> Result<(), String> {
    let canonical_dest = std::fs::canonicalize(dest).map_err(|e| {
        eprintln!("[ollama-extract] canonicalize dest: {e}");
        "ollama-extract-error".to_string()
    })?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| {
            eprintln!("[ollama-extract] zip entry: {e}");
            "ollama-extract-error".to_string()
        })?;

        let raw_path = match entry.enclosed_name() {
            Some(p) => p.to_path_buf(),
            None => return Err("zip contient un chemin non sûr — extraction refusée".into()),
        };

        let target = canonical_dest.join(&raw_path);

        if entry.is_dir() {
            std::fs::create_dir_all(&target).map_err(|e| {
                eprintln!("[ollama-extract] mkdir zip: {e}");
                "ollama-extract-error".to_string()
            })?;
            continue;
        }

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                eprintln!("[ollama-extract] mkdir zip parent: {e}");
                "ollama-extract-error".to_string()
            })?;
        }

        let mut outfile = std::fs::File::create(&target).map_err(|e| {
            eprintln!("[ollama-extract] create zip file: {e}");
            "ollama-extract-error".to_string()
        })?;
        std::io::copy(&mut entry, &mut outfile).map_err(|e| {
            eprintln!("[ollama-extract] copy zip entry: {e}");
            "ollama-extract-error".to_string()
        })?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn ps_single_quote(path: &Path) -> String {
    path.display().to_string().replace('\'', "''")
}
