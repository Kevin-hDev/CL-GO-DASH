use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

use crate::services::gpu_detect;
use crate::services::ollama_kill;
use crate::services::ollama_port;

pub struct OllamaSidecar(pub Mutex<Option<Child>>);

impl OllamaSidecar {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
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
    ollama_port::is_port_open(ollama_port::get_port()) || ollama_binary_path().is_ok()
}

pub fn start_sidecar(app: &AppHandle) -> Result<bool, String> {
    ollama_kill::kill_orphan_sidecar();

    let port = ollama_port::find_free_port();

    if ollama_port::detect_existing_instance(port) {
        eprintln!("[ollama] daemon existant détecté sur {port}, sidecar ignoré");
        ollama_port::set_port(port);
        return Ok(false);
    }

    if ollama_port::detect_existing_instance(11434) {
        eprintln!("[ollama] daemon système détecté sur 11434, réutilisation");
        ollama_port::set_port(11434);
        return Ok(false);
    }

    ollama_port::set_port(port);
    eprintln!("[ollama] port sélectionné : {port}");

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
    let gpu = gpu_detect::detect();
    let env_vars = crate::services::ollama_env::build_env_vars(
        &config.advanced, &gpu, port,
    );
    for (key, val) in &env_vars {
        cmd.env(key, val);
    }
    eprintln!("[ollama] GPU : {:?}, accel : {}", gpu, config.advanced.hardware_accel);
    eprintln!("[ollama] env : {:?}", env_vars.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>());

    #[cfg(target_os = "linux")]
    {
        let lib_dir = bundle_dir.join("lib/ollama");
        if lib_dir.is_dir() {
            let existing = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
            let new_path = format!("{}:{existing}", lib_dir.display());
            cmd.env("LD_LIBRARY_PATH", new_path);
            eprintln!("[ollama] LD_LIBRARY_PATH prépend {}", lib_dir.display());
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let child = cmd.spawn().map_err(|e| format!("spawn ollama: {e}"))?;
    let pid = child.id();
    ollama_kill::save_pid(pid);
    eprintln!("[ollama] sidecar démarré pid={pid} port={port}");

    let state = app.state::<OllamaSidecar>();
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    *guard = Some(child);

    Ok(true)
}

pub fn stop_sidecar(app: &AppHandle) {
    let state = app.state::<OllamaSidecar>();
    let mut guard = match state.0.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };

    if let Some(mut child) = guard.take() {
        ollama_kill::kill_process(&mut child);
    }
    ollama_kill::clear_pid_file();
}
