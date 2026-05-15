use std::path::{Path, PathBuf};
use std::process::Command;

const REQUIREMENTS_STAMP: &str = ".requirements.stamp";
const MAX_REQUIREMENTS_SIZE: usize = 16 * 1024;
const MAX_FAMILY_REQUIREMENTS_SIZE: usize = 32 * 1024;

pub fn ensure_runtime(sidecar_dir: &Path, family_id: &str) -> Result<PathBuf, String> {
    validate_family_id(family_id)?;
    let requirements = sidecar_dir.join("requirements.txt");
    let requirements_body = std::fs::read_to_string(&requirements)
        .map_err(|_| "Runtime Forecast incomplet".to_string())?;
    if requirements_body.len() > MAX_REQUIREMENTS_SIZE {
        return Err("Configuration runtime Forecast invalide".to_string());
    }

    let family_requirements =
        family_requirements(family_id).ok_or("Adapter Forecast indisponible".to_string())?;
    if family_requirements.is_empty() || family_requirements.len() > MAX_FAMILY_REQUIREMENTS_SIZE {
        return Err("Configuration runtime Forecast invalide".to_string());
    }

    let venv_dir = family_venv_dir(sidecar_dir, family_id);
    let venv_python = venv_python_path(&venv_dir);
    let stamp = venv_dir.join(REQUIREMENTS_STAMP);
    let combined_requirements = format!("{requirements_body}\n{family_requirements}");

    if !venv_python.exists() {
        let python = find_python()?;
        run(
            Command::new(&python).args(["-m", "venv"]).arg(&venv_dir),
            "Création du runtime Forecast impossible",
        )?;
    }

    let installed_stamp = std::fs::read_to_string(&stamp).unwrap_or_default();
    if installed_stamp != combined_requirements {
        run(
            Command::new(&venv_python).args(["-m", "pip", "install", "--upgrade", "pip"]),
            "Initialisation du runtime Forecast impossible",
        )?;
        let manifest = venv_dir.join("requirements.txt");
        std::fs::write(&manifest, &combined_requirements)
            .map_err(|_| "Préparation du runtime Forecast impossible".to_string())?;
        run(
            Command::new(&venv_python)
                .args(["-m", "pip", "install", "-r"])
                .arg(&manifest),
            "Installation du moteur Forecast impossible",
        )?;
        std::fs::write(&stamp, combined_requirements)
            .map_err(|_| "Validation du runtime Forecast impossible".to_string())?;
    }

    Ok(venv_python)
}

pub fn family_runtime_ready(sidecar_dir: &Path, family_id: &str) -> bool {
    if validate_family_id(family_id).is_err() {
        return false;
    }
    let venv_dir = family_venv_dir(sidecar_dir, family_id);
    venv_python_path(&venv_dir).exists() && venv_dir.join(REQUIREMENTS_STAMP).exists()
}

fn family_requirements(family_id: &str) -> Option<&'static str> {
    match family_id {
        "chronos-bolt" | "chronos-2" => Some("pandas<3\nchronos-forecasting==2.2.2\n"),
        "timesfm-2-5" => Some("timesfm\ntransformers\naccelerate\n"),
        "toto-2" => Some(
            "torch>=2.6,<3\ntoto-2 @ git+https://github.com/DataDog/toto.git#subdirectory=toto2\n",
        ),
        "moirai-2" => Some("git+https://github.com/SalesforceAIResearch/uni2ts.git\ngluonts\n"),
        "flowstate" => Some("granite-tsfm\n"),
        "tabpfn-ts" => Some("pandas\ntabpfn-time-series\ntabpfn\n"),
        "tirex" => Some("tirex-ts\n"),
        "kairos" => Some("git+https://github.com/foundation-model-research/Kairos.git\n"),
        "sundial" => Some("transformers>=4.40.1\naccelerate\n"),
        _ => None,
    }
}

fn family_venv_dir(sidecar_dir: &Path, family_id: &str) -> PathBuf {
    sidecar_dir.join(".venvs").join(family_id)
}

fn validate_family_id(family_id: &str) -> Result<(), String> {
    if family_id.is_empty()
        || family_id.len() > 80
        || !family_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
    {
        return Err("Famille Forecast invalide".to_string());
    }
    Ok(())
}

fn find_python() -> Result<PathBuf, String> {
    for candidate in [
        "python3.12",
        "python3.13",
        "python3.14",
        "python3",
        "python",
    ] {
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
    let output = command.output().map_err(|_| message.to_string())?;
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
    let mut indices = trimmed.char_indices();
    if indices.nth(400).is_none() {
        trimmed
    } else {
        let cutoff = trimmed
            .char_indices()
            .nth(400)
            .map(|(idx, _)| idx)
            .unwrap_or(trimmed.len());
        format!("{}...", &trimmed[..cutoff])
    }
}
