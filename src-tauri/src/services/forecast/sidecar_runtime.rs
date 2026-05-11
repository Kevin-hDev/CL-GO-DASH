use std::path::{Path, PathBuf};
use std::process::Command;

const REQUIREMENTS_STAMP: &str = ".requirements.stamp";
const MAX_REQUIREMENTS_SIZE: usize = 16 * 1024;

pub fn ensure_runtime(sidecar_dir: &Path) -> Result<PathBuf, String> {
    let requirements = sidecar_dir.join("requirements.txt");
    let requirements_body =
        std::fs::read_to_string(&requirements).map_err(|_| "Runtime Forecast incomplet".to_string())?;
    if requirements_body.is_empty() || requirements_body.len() > MAX_REQUIREMENTS_SIZE {
        return Err("Configuration runtime Forecast invalide".to_string());
    }

    let venv_dir = sidecar_dir.join(".venv");
    let venv_python = venv_python_path(&venv_dir);
    let stamp = sidecar_dir.join(REQUIREMENTS_STAMP);

    if !venv_python.exists() {
        let python = find_python()?;
        run(
            Command::new(&python).args(["-m", "venv"]).arg(&venv_dir),
            "Création du runtime Forecast impossible",
        )?;
    }

    let installed_stamp = std::fs::read_to_string(&stamp).unwrap_or_default();
    if installed_stamp != requirements_body {
        run(
            Command::new(&venv_python).args(["-m", "pip", "install", "--upgrade", "pip"]),
            "Initialisation du runtime Forecast impossible",
        )?;
        run(
            Command::new(&venv_python)
                .args(["-m", "pip", "install", "-r"])
                .arg(&requirements),
            "Installation du moteur Forecast impossible",
        )?;
        std::fs::write(&stamp, requirements_body)
            .map_err(|_| "Validation du runtime Forecast impossible".to_string())?;
    }

    Ok(venv_python)
}

fn find_python() -> Result<PathBuf, String> {
    for candidate in ["python3.12", "python3.13", "python3.14", "python3", "python"] {
        if let Ok(path) = which::which(candidate) {
            return Ok(path);
        }
    }
    Err("Runtime Python introuvable".to_string())
}

fn venv_python_path(venv_dir: &Path) -> PathBuf {
    if cfg!(windows) {
        venv_dir.join("Scripts").join("python.exe")
    } else {
        venv_dir.join("bin").join("python")
    }
}

fn run(command: &mut Command, message: &str) -> Result<(), String> {
    command.env("PIP_DISABLE_PIP_VERSION_CHECK", "1");
    command.env("PYTHONUNBUFFERED", "1");
    let output = command
        .output()
        .map_err(|_| message.to_string())?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    eprintln!(
        "[forecast-runtime] {} | stdout={} | stderr={}",
        message,
        truncate(&stdout),
        truncate(&stderr)
    );
    Err(message.to_string())
}

fn truncate(text: &str) -> String {
    let trimmed = text.trim().replace('\n', " ");
    if trimmed.len() <= 400 {
        trimmed
    } else {
        format!("{}…", &trimmed[..400])
    }
}
