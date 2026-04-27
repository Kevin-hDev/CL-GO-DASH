use std::path::Path;

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
    let mut tar = tar::Archive::new(gz);
    tar.unpack(dest).map_err(|e| format!("extraction tar.gz: {e}"))?;
    Ok(())
}

fn extract_tar_zst(archive: &Path, dest: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive)
        .map_err(|e| format!("ouverture archive: {e}"))?;
    let zst = zstd::Decoder::new(file)
        .map_err(|e| format!("décompression zstd: {e}"))?;
    let mut tar = tar::Archive::new(zst);
    tar.unpack(dest).map_err(|e| format!("extraction tar.zst: {e}"))?;
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
        zip_archive.extract(dest)
            .map_err(|e| format!("extraction zip: {e}"))?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn ps_single_quote(path: &Path) -> String {
    path.display().to_string().replace('\'', "''")
}
