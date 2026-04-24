use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager};

use crate::services::gpu_detect;

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

pub fn ollama_bundle_dir() -> PathBuf {
    crate::services::paths::data_dir().join("ollama-bundle")
}

pub fn ollama_binary_path() -> Result<PathBuf, String> {
    let binary_name = if cfg!(windows) { "ollama.exe" } else { "ollama" };
    let path = ollama_bundle_dir().join(binary_name);
    if !path.exists() {
        return Err("ollama-not-installed".into());
    }
    Ok(path)
}

pub fn is_ollama_ready() -> bool {
    port_open() || ollama_binary_path().is_ok()
}

pub fn start_sidecar(app: &AppHandle) -> Result<bool, String> {
    if port_open() {
        eprintln!("[ollama] daemon externe détecté sur 11434, sidecar ignoré");
        return Ok(false);
    }

    let binary = ollama_binary_path()?;
    let bundle_dir = binary.parent().ok_or("no parent dir")?.to_path_buf();

    let log_dir = crate::services::paths::data_dir().join("logs");
    let _ = std::fs::create_dir_all(&log_dir);
    let stderr_file = std::fs::File::create(log_dir.join("ollama-sidecar.log"))
        .map_err(|e| format!("log file: {e}"))?;

    let mut cmd = Command::new(&binary);
    cmd.arg("serve")
        .current_dir(&bundle_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file));

    let config = crate::services::config::read_config().unwrap_or_default();
    let accel = config.advanced.hardware_accel.as_str();
    let gpu = gpu_detect::detect();
    eprintln!("[ollama] GPU : {:?}, accel : {accel}", gpu);

    if accel == "cpu" {
        cmd.env("OLLAMA_LLM_LIBRARY", "cpu");
        eprintln!("[ollama] mode CPU forcé");
    } else {
        #[cfg(target_os = "windows")]
        if matches!(gpu, gpu_detect::GpuVendor::Amd) {
            cmd.env("OLLAMA_VULKAN", "1");
            eprintln!("[ollama] AMD Windows → OLLAMA_VULKAN=1");
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let child = cmd.spawn().map_err(|e| format!("spawn ollama: {e}"))?;

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
    #[cfg(not(unix))]
    {
        let _ = child.kill();
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
