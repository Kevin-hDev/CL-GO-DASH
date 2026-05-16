use std::io::Read;
use std::path::Path;

const MAX_TAR_ENTRIES: usize = 50_000;
const MAX_UNPACKED_BYTES: u64 = 4 * 1024 * 1024 * 1024;

pub fn extract_overlay(archive: &Path, dest: &Path, name: &str) -> Result<(), String> {
    if name.ends_with(".tgz") || name.ends_with(".tar.gz") {
        extract_tar_gz(archive, dest)?;
    } else if name.ends_with(".tar.zst") {
        extract_tar_zst(archive, dest)?;
    } else if name.ends_with(".zip") {
        super::ollama_extract_zip::extract_zip(archive, dest)?;
    } else {
        return Err(format!("format inconnu: {name}"));
    }
    Ok(())
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|e| {
        eprintln!("[ollama-extract] open tar.gz: {e}");
        "ollama-extract-error".to_string()
    })?;
    let gz = flate2::read::GzDecoder::new(file);
    let tar = tar::Archive::new(gz);
    safe_unpack_tar(tar, dest)
}

fn extract_tar_zst(archive: &Path, dest: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|e| {
        eprintln!("[ollama-extract] open tar.zst: {e}");
        "ollama-extract-error".to_string()
    })?;
    let zst = zstd::Decoder::new(file).map_err(|e| {
        eprintln!("[ollama-extract] zstd decode: {e}");
        "ollama-extract-error".to_string()
    })?;
    let tar = tar::Archive::new(zst);
    safe_unpack_tar(tar, dest)
}

pub(crate) fn safe_unpack_tar<R: Read>(
    mut archive: tar::Archive<R>,
    dest: &Path,
) -> Result<(), String> {
    let canonical_dest = std::fs::canonicalize(dest).map_err(|e| {
        eprintln!("[ollama-extract] canonicalize dest: {e}");
        "ollama-extract-error".to_string()
    })?;

    let mut entry_count: usize = 0;
    let mut total_bytes: u64 = 0;

    for entry_result in archive.entries().map_err(|e| {
        eprintln!("[ollama-extract] tar entries: {e}");
        "ollama-extract-error".to_string()
    })? {
        entry_count += 1;
        if entry_count > MAX_TAR_ENTRIES {
            return Err("archive contient trop d'entrées — extraction refusée".into());
        }
        let mut entry = entry_result.map_err(|e| {
            eprintln!("[ollama-extract] tar entry: {e}");
            "ollama-extract-error".to_string()
        })?;
        let raw_path = entry
            .path()
            .map_err(|e| {
                eprintln!("[ollama-extract] tar path: {e}");
                "ollama-extract-error".to_string()
            })?
            .into_owned();

        if super::ollama_extract_tar_safe::has_parent_dir(&raw_path) {
            return Err("archive contient un chemin avec '..' — extraction refusée".into());
        }

        let entry_type = entry.header().entry_type();
        if matches!(entry_type, tar::EntryType::Symlink) {
            let target =
                super::ollama_extract_tar_safe::safe_target_path(&canonical_dest, dest, &raw_path)?;
            super::ollama_extract_tar_safe::unpack_safe_symlink(&entry, &canonical_dest, &target)?;
            continue;
        }
        if matches!(
            entry_type,
            tar::EntryType::Link
                | tar::EntryType::Block
                | tar::EntryType::Char
                | tar::EntryType::Fifo
        ) {
            return Err(format!(
                "type d'entrée non autorisé dans l'archive: {:?}",
                entry_type
            ));
        }

        let target_canonical =
            super::ollama_extract_tar_safe::safe_target_path(&canonical_dest, dest, &raw_path)?;

        total_bytes += entry.header().size().unwrap_or(0);
        if total_bytes > MAX_UNPACKED_BYTES {
            return Err("archive trop volumineuse — extraction refusée".into());
        }

        entry.unpack(&target_canonical).map_err(|e| {
            eprintln!("[ollama-extract] unpack: {e}");
            "ollama-extract-error".to_string()
        })?;
    }
    Ok(())
}
