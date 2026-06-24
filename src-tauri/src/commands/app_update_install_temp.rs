use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use uuid::Uuid;

const MAX_TEMP_ATTEMPTS: usize = 8;

pub(crate) fn update_log_path() -> Result<PathBuf, String> {
    let dir = crate::services::paths::data_dir().join("logs");
    fs::create_dir_all(&dir).map_err(|e| {
        eprintln!("[update] create log dir: {e}");
        "update-install-error".to_string()
    })?;
    Ok(dir.join("update.log"))
}

pub(crate) fn create_unique_temp_file(
    prefix: &str,
    suffix: &str,
) -> Result<(PathBuf, File), String> {
    for _ in 0..MAX_TEMP_ATTEMPTS {
        let path = std::env::temp_dir().join(format!("{prefix}-{}{suffix}", Uuid::new_v4()));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => {
                eprintln!("[update] create temp file: {e}");
                return Err("update-write-error".to_string());
            }
        }
    }
    Err("update-write-error".to_string())
}

pub(crate) fn write_unique_temp_file(
    prefix: &str,
    suffix: &str,
    content: &str,
) -> Result<PathBuf, String> {
    let (path, mut file) = create_unique_temp_file(prefix, suffix)?;
    file.write_all(content.as_bytes()).map_err(|e| {
        eprintln!("[update] write temp file: {e}");
        "update-install-error".to_string()
    })?;
    Ok(path)
}
