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
    if port == 0 { DEFAULT_PORT } else { port }
}

pub fn base_url() -> String {
    format!("http://127.0.0.1:{}", get_port())
}

fn pid_path() -> PathBuf {
    data_dir().join("chronos-sidecar.pid")
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
    let Ok(mut stream) = TcpStream::connect_timeout(
        &addr.parse().unwrap(),
        Duration::from_secs(2),
    ) else {
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

pub fn kill_orphan_sidecar() {
    let pid_file = pid_path();
    if !pid_file.exists() {
        return;
    }
    if let Ok(content) = std::fs::read_to_string(&pid_file) {
        if let Ok(pid) = content.trim().parse::<u32>() {
            #[cfg(unix)]
            {
                unsafe { libc::kill(pid as i32, libc::SIGTERM); }
            }
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F", "/T"])
                    .output();
            }
        }
    }
    let _ = std::fs::remove_file(&pid_file);
}

fn save_pid(pid: u32) {
    let tmp = pid_path().with_extension("tmp");
    let _ = std::fs::write(&tmp, pid.to_string());
    let _ = std::fs::rename(&tmp, pid_path());
}

pub async fn start(sidecar: &ChronosSidecar, model_name: &str) -> Result<u16, String> {
    kill_orphan_sidecar();

    let port = find_free_port();
    if detect_existing_instance(port) {
        ACTIVE_PORT.store(port, Ordering::Relaxed);
        return Ok(port);
    }

    let script = sidecar_dir().join("server.py");
    if !script.exists() {
        return Err("Sidecar Python non installé".into());
    }

    let models_dir = data_dir().join("forecast-models");
    let child = std::process::Command::new("uv")
        .args([
            "run", "python3",
            script.to_str().unwrap_or("server.py"),
            "--port", &port.to_string(),
            "--model", model_name,
            "--models-dir", models_dir.to_str().unwrap_or(""),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Impossible de lancer le sidecar Chronos: {e}"))?;

    save_pid(child.id());
    ACTIVE_PORT.store(port, Ordering::Relaxed);

    *sidecar.0.lock().await = Some(child);

    // Attendre que le sidecar soit prêt (max 30s)
    for _ in 0..60 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if detect_existing_instance(port) {
            return Ok(port);
        }
    }

    Err("Sidecar Chronos: timeout au démarrage".into())
}

pub async fn stop(sidecar: &ChronosSidecar) {
    if let Some(mut child) = sidecar.0.lock().await.take() {
        #[cfg(unix)]
        {
            unsafe { libc::kill(child.id() as i32, libc::SIGTERM); }
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(3),
                tokio::task::spawn_blocking(move || { let _ = child.wait(); }),
            )
            .await;
        }
        #[cfg(windows)]
        {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
    let _ = std::fs::remove_file(pid_path());
    ACTIVE_PORT.store(0, Ordering::Relaxed);
}
