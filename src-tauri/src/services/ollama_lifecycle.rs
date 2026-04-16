use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager};

/// État partagé du sidecar Ollama. `None` signifie :
/// - soit le sidecar n'a pas été démarré (Ollama externe déjà présent sur 11434)
/// - soit il a été arrêté
pub struct OllamaSidecar(pub Mutex<Option<Child>>);

impl OllamaSidecar {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

/// Teste si un daemon Ollama répond déjà sur localhost:11434.
/// Évite de lancer un sidecar si l'utilisateur a déjà Ollama.app installé.
fn port_open() -> bool {
    TcpStream::connect_timeout(
        &"127.0.0.1:11434".parse().unwrap(),
        Duration::from_millis(500),
    )
    .is_ok()
}

/// Chemin du binaire ollama dans le bundle de resources.
fn ollama_binary_path(app: &AppHandle) -> Result<PathBuf, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("resource_dir: {e}"))?;
    let path = resource_dir.join("resources").join("ollama-bundle").join("ollama");
    if !path.exists() {
        return Err(format!("Binaire Ollama introuvable : {}", path.display()));
    }
    Ok(path)
}

/// Démarre le sidecar si Ollama n'est pas déjà en cours d'exécution.
/// Retourne `Ok(true)` si un sidecar a été lancé, `Ok(false)` si Ollama
/// externe est utilisé.
pub fn start_sidecar(app: &AppHandle) -> Result<bool, String> {
    if port_open() {
        eprintln!("[ollama] daemon externe détecté sur 11434, sidecar ignoré");
        return Ok(false);
    }

    let binary = ollama_binary_path(app)?;
    let bundle_dir = binary.parent().ok_or("no parent dir")?.to_path_buf();

    let child = Command::new(&binary)
        .arg("serve")
        .current_dir(&bundle_dir)
        // Rend les .dylib trouvables depuis le dossier du binaire
        .env("DYLD_LIBRARY_PATH", &bundle_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("spawn ollama: {e}"))?;

    eprintln!("[ollama] sidecar démarré pid={}", child.id());

    let state = app.state::<OllamaSidecar>();
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    *guard = Some(child);

    Ok(true)
}

/// Arrêt propre : SIGTERM + attente courte, SIGKILL en dernier recours.
pub fn stop_sidecar(app: &AppHandle) {
    let state = app.state::<OllamaSidecar>();
    let mut guard = match state.0.lock() {
        Ok(g) => g,
        Err(_) => return,
    };

    let Some(mut child) = guard.take() else { return; };
    let pid = child.id();
    eprintln!("[ollama] arrêt sidecar pid={}", pid);

    // SIGTERM (demande propre)
    #[cfg(unix)]
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }

    // Grace period 3s
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(Some(_)) = child.try_wait() {
            eprintln!("[ollama] sidecar arrêté proprement");
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    // SIGKILL si toujours vivant
    eprintln!("[ollama] SIGKILL après timeout");
    let _ = child.kill();
    let _ = child.wait();
}
