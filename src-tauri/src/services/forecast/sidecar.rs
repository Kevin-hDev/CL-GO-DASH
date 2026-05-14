use crate::services::forecast::{sidecar_process, sidecar_runtime};
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

struct SidecarHandle {
    child: Child,
    model_id: String,
}

pub struct ChronosSidecar(Mutex<Option<SidecarHandle>>);

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

fn health_info(port: u16) -> Option<(u16, String)> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let addr = format!("127.0.0.1:{port}");
    let Ok(mut stream) = TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(2))
    else {
        return None;
    };
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let req = format!("GET /health HTTP/1.0\r\nHost: 127.0.0.1:{port}\r\n\r\n");
    if stream.write_all(req.as_bytes()).is_err() {
        return None;
    }
    let mut buf = [0u8; 512];
    let n = stream.read(&mut buf).unwrap_or(0);
    let response = String::from_utf8_lossy(&buf[..n]);
    let body = response.split("\r\n\r\n").nth(1)?;
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    let model = json["model"].as_str()?.to_string();
    Some((port, model))
}

pub async fn start(sidecar: &ChronosSidecar, model_name: &str) -> Result<u16, String> {
    {
        let guard = sidecar.0.lock().await;
        if let Some(handle) = guard.as_ref() {
            if handle.model_id == model_name {
                if let Some((port, model)) = health_info(get_port()) {
                    if model == model_name {
                        return Ok(port);
                    }
                }
            }
        }
    }

    stop(sidecar).await;
    sidecar_process::kill_orphan_sidecar();
    let port = find_free_port();

    let script = sidecar_dir().join("server.py");
    if !script.exists() {
        return Err("Sidecar Python non installé".into());
    }

    let runtime_python = tokio::task::spawn_blocking({
        let dir = sidecar_dir();
        move || sidecar_runtime::ensure_runtime(&dir)
    })
    .await
    .map_err(|_| "Initialisation du moteur Forecast impossible".to_string())?
    .map_err(|_| "Initialisation du moteur Forecast impossible".to_string())?;

    let models_dir = data_dir().join("forecast-models");
    let child = std::process::Command::new(runtime_python)
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
        .map_err(|_| "Impossible de lancer le sidecar Chronos".to_string())?;

    sidecar_process::save_pid(child.id());
    ACTIVE_PORT.store(port, Ordering::Relaxed);

    *sidecar.0.lock().await = Some(SidecarHandle {
        child,
        model_id: model_name.to_string(),
    });

    // Attendre que le sidecar soit prêt (max 30s)
    for _ in 0..60 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if let Some((ready_port, ready_model)) = health_info(port) {
            if ready_model == model_name {
                return Ok(ready_port);
            }
        }
    }

    stop(sidecar).await;
    Err("Sidecar Chronos: timeout au démarrage".into())
}

pub async fn stop(sidecar: &ChronosSidecar) {
    if let Some(handle) = sidecar.0.lock().await.take() {
        let _ = tokio::task::spawn_blocking(move || {
            sidecar_process::kill_child_process(handle.child);
        })
        .await;
    }
    sidecar_process::clear_pid_file();
    ACTIVE_PORT.store(0, Ordering::Relaxed);
}
