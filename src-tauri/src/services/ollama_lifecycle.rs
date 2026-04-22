use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager};

pub struct OllamaSidecar(pub Mutex<Option<Child>>);

impl OllamaSidecar {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

fn port_open() -> bool {
    TcpStream::connect_timeout(
        &"127.0.0.1:11434".parse().unwrap(),
        Duration::from_millis(500),
    )
    .is_ok()
}

fn ollama_binary_path(app: &AppHandle) -> Result<PathBuf, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("resource_dir: {e}"))?;
    let path = resource_dir
        .join("resources")
        .join("ollama-bundle")
        .join("ollama");
    if !path.exists() {
        return Err(format!("Binaire Ollama introuvable : {}", path.display()));
    }
    Ok(path)
}

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

pub fn stop_sidecar(app: &AppHandle) {
    let state = app.state::<OllamaSidecar>();
    let mut guard = match state.0.lock() {
        Ok(g) => g,
        Err(_) => return,
    };

    let Some(mut child) = guard.take() else { return };
    kill_process(&mut child);
}

fn kill_process(child: &mut Child) {
    let pid = child.id();
    eprintln!("[ollama] kill sidecar pid={pid}");

    #[cfg(unix)]
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }

    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(Some(_)) = child.try_wait() {
            eprintln!("[ollama] sidecar arrêté proprement");
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    eprintln!("[ollama] SIGKILL après timeout");
    let _ = child.kill();
    let _ = child.wait();
}
