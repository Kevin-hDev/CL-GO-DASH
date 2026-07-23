use super::fs_safety;
use sha2::{Digest, Sha256};
use std::path::Path;
use subtle::ConstantTimeEq;
use tokio::io::{AsyncWriteExt, BufWriter};

const MARKER: &str = ".smoke-v2";
const MAX_SOURCE_FILES: usize = 128;
const MAX_SOURCE_BYTES: u64 = 1024 * 1024;

pub(super) async fn write(model_dir: &Path, sidecar: &Path) -> Result<(), String> {
    if !fs_safety::is_real_directory(model_dir) {
        return Err(error());
    }
    let fingerprint = source_fingerprint(sidecar)?;
    let temporary = model_dir.join(".smoke-v2.tmp");
    let destination = model_dir.join(MARKER);
    fs_safety::remove_path(&temporary)
        .await
        .map_err(|_| error())?;
    let result = async {
        let file = tokio::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .await
            .map_err(|_| error())?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&fingerprint).await.map_err(|_| error())?;
        writer.flush().await.map_err(|_| error())?;
        writer.get_ref().sync_all().await.map_err(|_| error())?;
        drop(writer);
        fs_safety::remove_path(&destination)
            .await
            .map_err(|_| error())?;
        tokio::fs::rename(&temporary, &destination)
            .await
            .map_err(|_| error())
    }
    .await;
    if result.is_err() {
        let _ = fs_safety::remove_path(&temporary).await;
    }
    result
}

pub(super) fn is_current(model_dir: &Path, sidecar: &Path) -> bool {
    if !fs_safety::is_real_directory(model_dir) {
        return false;
    }
    let path = model_dir.join(MARKER);
    let Ok(metadata) = std::fs::symlink_metadata(&path) else {
        return false;
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() != 32 {
        return false;
    }
    let Ok(actual) = std::fs::read(path) else {
        return false;
    };
    let Ok(expected) = source_fingerprint(sidecar) else {
        return false;
    };
    actual.ct_eq(&expected).into()
}

fn source_fingerprint(sidecar: &Path) -> Result<[u8; 32], String> {
    let runtime = sidecar.join("forecast_runtime");
    let mut files = Vec::with_capacity(MAX_SOURCE_FILES);
    for entry in std::fs::read_dir(runtime).map_err(|_| error())? {
        let path = entry.map_err(|_| error())?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("py") {
            continue;
        }
        if files.len() >= MAX_SOURCE_FILES {
            return Err(error());
        }
        let metadata = std::fs::symlink_metadata(&path).map_err(|_| error())?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > MAX_SOURCE_BYTES
        {
            return Err(error());
        }
        files.push(path);
    }
    files.sort();
    if files.is_empty() {
        return Err(error());
    }
    let mut digest = Sha256::new();
    for path in files {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(error)?;
        let body = std::fs::read(&path).map_err(|_| error())?;
        digest.update((name.len() as u64).to_be_bytes());
        digest.update(name.as_bytes());
        digest.update((body.len() as u64).to_be_bytes());
        digest.update(body);
    }
    Ok(digest.finalize().into())
}

fn error() -> String {
    "Validation du modèle Forecast impossible".to_string()
}

#[cfg(test)]
mod tests {
    use super::{is_current, write};

    #[tokio::test]
    async fn proof_changes_with_adapter_source() {
        let root = tempfile::tempdir().unwrap();
        let runtime = root.path().join("forecast_runtime");
        let model = root.path().join("model");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::create_dir_all(&model).unwrap();
        std::fs::write(runtime.join("adapter.py"), "first").unwrap();
        write(&model, root.path()).await.unwrap();
        assert!(is_current(&model, root.path()));
        std::fs::write(runtime.join("adapter.py"), "second").unwrap();
        assert!(!is_current(&model, root.path()));
    }
}
