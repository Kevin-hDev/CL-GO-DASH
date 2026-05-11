use crate::services::forecast::sidecar_process;
use crate::services::paths::data_dir;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Child;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::sync::Mutex;

const PORT_RANGE_START: u16 = 12000;
const PORT_RANGE_END: u16 = 12099;
const DEFAULT_PORT: u16 = 12000;

static ACTIVE_PORT: AtomicU16 = AtomicU16::new(0);

pub struct ChronosSidecar(pub Mutex<Option<Child>>);

impl ChronosSidecar {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

pub fn get_port() -> u16 {
    let port = ACTIVE_PORT.load(Ordering::Relaxed);
    if port == 0 {
        DEFAULT_PORT
    } else {
        port
    }
}

pub fn base_url() -> String {
    format!("http://127.0.0.1:{}", get_port())
}

fn sidecar_dir() -> PathBuf {
    data_dir().join("forecast-sidecar")
}

pub fn find_free_port() -> u16 {
    for port in PORT_RANGE_START..=PORT_RANGE_END {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    DEFAULT_PORT
}

pub fn detect_existing_instance(port: u16) -> bool {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let addr = format!("127.0.0.1:{port}");
    let Ok(mut stream) = TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(2))
    else {
        return false;
    };
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let req = format!("GET /health HTTP/1.0\r\nHost: 127.0.0.1:{port}\r\n\r\n");
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let mut buf = [0u8; 64];
    let n = stream.read(&mut buf).unwrap_or(0);
    let response = String::from_utf8_lossy(&buf[..n]);
    response.contains("200")
}

pub async fn start(sidecar: &ChronosSidecar, model_name: &str) -> Result<u16, String> {
    let existing_port = get_port();
    if detect_existing_instance(existing_port) {
        return Ok(existing_port);
    }

    sidecar_process::kill_orphan_sidecar();
    let port = find_free_port();

    let script = sidecar_dir().join("server.py");
    if !script.exists() {
        return Err("Sidecar Python non installé".into());
    }

    let models_dir = data_dir().join("forecast-models");
    let python = find_python()?;
    let child = std::process::Command::new(python)
        .args([
            script.to_str().unwrap_or("server.py"),
            "--port",
            &port.to_string(),
            "--model",
            model_name,
            "--models-dir",
            models_dir.to_str().unwrap_or(""),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Impossible de lancer le sidecar Chronos: {e}"))?;

    sidecar_process::save_pid(child.id());
    ACTIVE_PORT.store(port, Ordering::Relaxed);

    *sidecar.0.lock().await = Some(child);

    // Attendre que le sidecar soit prêt (max 30s)
    for _ in 0..60 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if detect_existing_instance(port) {
            return Ok(port);
        }
    }

    stop(sidecar).await;
    Err("Sidecar Chronos: timeout au démarrage".into())
}

fn find_python() -> Result<PathBuf, String> {
    which::which("python3")
        .or_else(|_| which::which("python"))
        .map_err(|_| "Runtime Python introuvable".to_string())
}

pub async fn stop(sidecar: &ChronosSidecar) {
    if let Some(child) = sidecar.0.lock().await.take() {
        let _ = tokio::task::spawn_blocking(move || {
            sidecar_process::kill_child_process(child);
        })
        .await;
    }
    sidecar_process::clear_pid_file();
    ACTIVE_PORT.store(0, Ordering::Relaxed);
}
