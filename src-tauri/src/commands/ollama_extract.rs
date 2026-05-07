use std::io::Read;
use std::path::Path;

const MAX_TAR_ENTRIES: usize = 50_000;
const MAX_UNPACKED_BYTES: u64 = 4 * 1024 * 1024 * 1024;

pub fn extract_overlay(
    archive: &Path,
    dest: &Path,
    name: &str,
) -> Result<(), String> {
    if name.ends_with(".tgz") || name.ends_with(".tar.gz") {
        extract_tar_gz(archive, dest)?;
    } else if name.ends_with(".tar.zst") {
        extract_tar_zst(archive, dest)?;
    } else if name.ends_with(".zip") {
        extract_zip(archive, dest)?;
    } else {
        return Err(format!("format inconnu: {name}"));
    }
    Ok(())
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive)
        .map_err(|e| format!("ouverture archive: {e}"))?;
    let gz = flate2::read::GzDecoder::new(file);
    let tar = tar::Archive::new(gz);
    safe_unpack_tar(tar, dest)
}

fn extract_tar_zst(archive: &Path, dest: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive)
        .map_err(|e| format!("ouverture archive: {e}"))?;
    let zst = zstd::Decoder::new(file)
        .map_err(|e| format!("décompression zstd: {e}"))?;
    let tar = tar::Archive::new(zst);
    safe_unpack_tar(tar, dest)
}

pub(crate) fn safe_unpack_tar<R: Read>(
    mut archive: tar::Archive<R>,
    dest: &Path,
) -> Result<(), String> {
    let canonical_dest = std::fs::canonicalize(dest)
        .map_err(|e| { eprintln!("[ollama-extract] canonicalize dest: {e}"); "ollama-extract-error".to_string() })?;

    let mut entry_count: usize = 0;
    let mut total_bytes: u64 = 0;

    for entry_result in archive.entries().map_err(|e| format!("tar entries: {e}"))? {
        entry_count += 1;
        if entry_count > MAX_TAR_ENTRIES {
            return Err("archive contient trop d'entrées — extraction refusée".into());
        }
        let mut entry = entry_result.map_err(|e| format!("tar entry: {e}"))?;
        let raw_path = entry.path().map_err(|e| format!("tar path: {e}"))?;

        if raw_path.components().any(|c| c == std::path::Component::ParentDir) {
            return Err("archive contient un chemin avec '..' — extraction refusée".into());
        }

        let entry_type = entry.header().entry_type();
        if matches!(
            entry_type,
            tar::EntryType::Symlink
                | tar::EntryType::Link
                | tar::EntryType::Block
                | tar::EntryType::Char
                | tar::EntryType::Fifo
        ) {
            return Err(format!(
                "type d'entrée non autorisé dans l'archive: {:?}",
                entry_type
            ));
        }

        let target = canonical_dest.join(&raw_path);
        let target_canonical = if target.exists() {
            std::fs::canonicalize(&target)
                .map_err(|e| { eprintln!("[ollama-extract] canonicalize entry: {e}"); "ollama-extract-error".to_string() })?
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("mkdir: {e}"))?;
            }
            let parent_canonical = std::fs::canonicalize(
                target.parent().unwrap_or(dest),
            )
            .map_err(|e| { eprintln!("[ollama-extract] canonicalize parent: {e}"); "ollama-extract-error".to_string() })?;
            parent_canonical.join(target.file_name().unwrap_or_default())
        };

        if !target_canonical.starts_with(&canonical_dest) {
            eprintln!(
                "[ollama-extract] path traversal: {} vs {}",
                raw_path.display(), canonical_dest.display()
            );
            return Err("ollama-extract-path-traversal".into());
        }

        total_bytes += entry.header().size().unwrap_or(0);
        if total_bytes > MAX_UNPACKED_BYTES {
            return Err("archive trop volumineuse — extraction refusée".into());
        }

        entry.unpack(&target_canonical)
            .map_err(|e| format!("unpack entry: {e}"))?;
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
        let file = std::fs::File::open(archive)
            .map_err(|e| format!("ouverture zip: {e}"))?;
        let mut zip_archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("lecture zip: {e}"))?;
        safe_unpack_zip(&mut zip_archive, dest)?;
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn safe_unpack_zip(
    archive: &mut zip::ZipArchive<std::fs::File>,
    dest: &Path,
) -> Result<(), String> {
    let canonical_dest = std::fs::canonicalize(dest)
        .map_err(|e| { eprintln!("[ollama-extract] canonicalize dest: {e}"); "ollama-extract-error".to_string() })?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("zip entry: {e}"))?;

        let raw_path = match entry.enclosed_name() {
            Some(p) => p.to_path_buf(),
            None => return Err("zip contient un chemin non sûr — extraction refusée".into()),
        };

        let target = canonical_dest.join(&raw_path);

        if entry.is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|e| format!("mkdir zip: {e}"))?;
            continue;
        }

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("mkdir zip parent: {e}"))?;
        }

        let mut outfile = std::fs::File::create(&target)
            .map_err(|e| format!("create zip file: {e}"))?;
        std::io::copy(&mut entry, &mut outfile)
            .map_err(|e| format!("copy zip entry: {e}"))?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn ps_single_quote(path: &Path) -> String {
    path.display().to_string().replace('\'', "''")
}

