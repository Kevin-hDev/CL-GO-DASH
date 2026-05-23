use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

const STAMP: &str = ".runtime.stamp";

pub async fn ensure_runtime(source: &Path) -> Result<PathBuf, String> {
    let source = source.to_path_buf();
    tokio::task::spawn_blocking(move || ensure_runtime_blocking(&source))
        .await
        .map_err(|_| "SearXNG: runtime indisponible".to_string())?
}

fn ensure_runtime_blocking(source: &Path) -> Result<PathBuf, String> {
    validate_source(source)?;
    let venv = super::paths::venv_dir();
    let python = venv_python(&venv);
    if !python.exists() {
        let base_python = find_python()?;
        run(Command::new(base_python).args(["-m", "venv"]).arg(&venv))?;
    }

    let stamp_path = venv.join(STAMP);
    let stamp = source_stamp(source)?;
    let installed = std::fs::read_to_string(&stamp_path).unwrap_or_default();
    if installed != stamp {
        run(Command::new(&python).args(["-m", "pip", "install", "--upgrade", "pip"]))?;
        install_source(&python, source)?;
        std::fs::write(stamp_path, stamp)
            .map_err(|_| "SearXNG: validation runtime impossible".to_string())?;
    }
    Ok(python)
}

fn validate_source(source: &Path) -> Result<(), String> {
    let required = ["setup.py", "requirements.txt", "LICENSE", "searx/webapp.py"];
    if required.iter().all(|file| source.join(file).exists()) {
        return Ok(());
    }
    Err("SearXNG: bundle incomplet".to_string())
}

fn source_stamp(source: &Path) -> Result<String, String> {
    let mut hasher = Sha256::new();
    for file in ["setup.py", "requirements.txt", "searx/version.py"] {
        let body = std::fs::read(source.join(file))
            .map_err(|_| "SearXNG: bundle incomplet".to_string())?;
        hasher.update(body);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn install_source(python: &Path, source: &Path) -> Result<(), String> {
    let wheels = source
        .parent()
        .map(|p| p.join("wheels"))
        .filter(|p| wheelhouse_usable(p));
    if let Some(wheels) = wheels {
        run(Command::new(python)
            .args(["-m", "pip", "install", "--no-index", "--find-links"])
            .arg(wheels)
            .args(["-e"])
            .arg(source))
    } else {
        run(Command::new(python)
            .args(["-m", "pip", "install", "-e"])
            .arg(source))
    }
}

fn wheelhouse_usable(path: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(path) else {
        return false;
    };
    entries.flatten().any(|entry| {
        entry
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("whl"))
    })
}

fn find_python() -> Result<PathBuf, String> {
    for candidate in [
        "python3.13",
        "python3.12",
        "python3.11",
        "python3.10",
        "python3",
        "python",
    ] {
        if let Ok(path) = which::which(candidate) {
            return Ok(path);
        }
    }
    Err("SearXNG: runtime Python introuvable".to_string())
}

fn venv_python(venv: &Path) -> PathBuf {
    if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    }
}

fn run(command: &mut Command) -> Result<(), String> {
    command.env("PIP_DISABLE_PIP_VERSION_CHECK", "1");
    command.env("PIP_NO_INPUT", "1");
    command.env("PYTHONUNBUFFERED", "1");
    let output = command
        .output()
        .map_err(|_| "SearXNG: runtime indisponible".to_string())?;
    if output.status.success() {
        return Ok(());
    }
    Err("SearXNG: installation runtime échouée".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheelhouse_requires_at_least_one_wheel() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!wheelhouse_usable(dir.path()));
        std::fs::write(dir.path().join("a.whl"), b"wheel").unwrap();
        assert!(wheelhouse_usable(dir.path()));
    }
}
