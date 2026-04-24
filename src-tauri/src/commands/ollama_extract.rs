use std::path::Path;

pub fn extract_archive(
    archive: &Path,
    dest: &Path,
    name: &str,
    expected_binary: &str,
) -> Result<(), String> {
    let tmp = tempfile::tempdir().map_err(|e| format!("tmpdir: {e}"))?;

    if name.ends_with(".tgz") || name.ends_with(".tar.gz") {
        run_cmd(
            "tar",
            &[
                "-xzf",
                &archive.display().to_string(),
                "-C",
                &tmp.path().display().to_string(),
            ],
        )?;
    } else if name.ends_with(".tar.zst") {
        run_cmd(
            "tar",
            &[
                "--zstd",
                "-xf",
                &archive.display().to_string(),
                "-C",
                &tmp.path().display().to_string(),
            ],
        )?;
    } else if name.ends_with(".zip") {
        extract_zip(archive, tmp.path())?;
    } else {
        return Err(format!("format inconnu: {name}"));
    }

    move_inner_to_dest(tmp.path(), dest)?;

    if !dest.join(expected_binary).is_file() {
        return Err(format!(
            "installation incomplète: {} absent après extraction",
            expected_binary
        ));
    }

    Ok(())
}

fn run_cmd(program: &str, args: &[&str]) -> Result<(), String> {
    let status = std::process::Command::new(program)
        .args(args)
        .status()
        .map_err(|e| format!("{program}: {e}"))?;
    if !status.success() {
        return Err(format!("{program} failed"));
    }
    Ok(())
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<(), String> {
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
            .map_err(|e| format!("powershell: {e}"))?;
        if !status.success() {
            return Err("Expand-Archive failed".into());
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        run_cmd(
            "unzip",
            &[
                "-o",
                "-q",
                &archive.display().to_string(),
                "-d",
                &dest.display().to_string(),
            ],
        )?;
    }
    Ok(())
}

fn move_inner_to_dest(tmp: &Path, dest: &Path) -> Result<(), String> {
    let entries: Vec<_> = std::fs::read_dir(tmp)
        .map_err(|e| format!("readdir: {e}"))?
        .filter_map(|e| e.ok())
        .collect();
    if entries.is_empty() {
        return Err("archive vide après extraction".into());
    }

    let source = if entries.len() == 1 && entries[0].path().is_dir() {
        entries[0].path()
    } else {
        tmp.to_path_buf()
    };

    for entry in std::fs::read_dir(&source).map_err(|e| format!("readdir: {e}"))? {
        let entry = entry.map_err(|e| format!("entry: {e}"))?;
        let target = dest.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target).map_err(|e| format!("copy: {e}"))?;
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn ps_single_quote(path: &Path) -> String {
    path.display().to_string().replace('\'', "''")
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("mkdir: {e}"))?;
    for entry in std::fs::read_dir(src).map_err(|e| format!("readdir: {e}"))? {
        let entry = entry.map_err(|e| format!("entry: {e}"))?;
        let target = dst.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target).map_err(|e| format!("copy: {e}"))?;
        }
    }
    Ok(())
}
