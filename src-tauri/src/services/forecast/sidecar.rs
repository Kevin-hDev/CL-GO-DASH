use crate::services::forecast::{sidecar_process, sidecar_runtime};
use crate::services::paths::data_dir;
use rand::RngCore;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Child;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::sync::Mutex;
use zeroize::Zeroizing;

const PORT_RANGE_START: u16 = 12000;
const PORT_RANGE_END: u16 = 12099;
const DEFAULT_PORT: u16 = 12000;

static ACTIVE_PORT: AtomicU16 = AtomicU16::new(0);

struct SidecarHandle {
    child: Child,
    model_id: String,
    family_id: String,
    auth_token: Zeroizing<String>,
}

pub struct ChronosSidecar(Mutex<Option<SidecarHandle>>);

pub struct SidecarEndpoint {
    pub base_url: String,
    pub auth_token: Zeroizing<String>,
}

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

fn generate_auth_token() -> Zeroizing<String> {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let token = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    bytes.fill(0);
    Zeroizing::new(token)
}

pub fn find_free_port() -> u16 {
    for port in PORT_RANGE_START..=PORT_RANGE_END {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    DEFAULT_PORT
}

fn health_info(port: u16, auth_token: &str) -> Option<(u16, String, String)> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let addr = format!("127.0.0.1:{port}");
    let Ok(mut stream) = TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(2))
    else {
        return None;
    };
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let req = format!(
        "GET /health HTTP/1.0\r\nHost: 127.0.0.1:{port}\r\nX-CLGO-Forecast-Token: {auth_token}\r\n\r\n"
    );
    if stream.write_all(req.as_bytes()).is_err() {
        return None;
    }
    let mut buf = [0u8; 512];
    let n = stream.read(&mut buf).unwrap_or(0);
    let response = String::from_utf8_lossy(&buf[..n]);
    let body = response.split("\r\n\r\n").nth(1)?;
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    let model = json["model"].as_str()?.to_string();
    let family = json["family"].as_str().unwrap_or("").to_string();
    Some((port, model, family))
}

pub async fn start(
    sidecar: &ChronosSidecar,
    model_name: &str,
    family_id: &str,
) -> Result<SidecarEndpoint, String> {
    {
        let guard = sidecar.0.lock().await;
        if let Some(handle) = guard.as_ref() {
            if handle.model_id == model_name && handle.family_id == family_id {
                if let Some((_port, model, family)) =
                    health_info(get_port(), handle.auth_token.as_str())
                {
                    if model == model_name && family == family_id {
                        return Ok(SidecarEndpoint {
                            base_url: base_url(),
                            auth_token: handle.auth_token.clone(),
                        });
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
        let family = family_id.to_string();
        move || sidecar_runtime::ensure_runtime(&dir, &family)
    })
    .await
    .map_err(|_| "Initialisation du moteur Forecast impossible".to_string())?
    .map_err(|_| "Initialisation du moteur Forecast impossible".to_string())?;

    let models_dir = data_dir().join("forecast-models");
    let auth_token = generate_auth_token();
    let child = std::process::Command::new(runtime_python)
        .args([
            script.to_str().unwrap_or("server.py"),
            "--port",
            &port.to_string(),
            "--model",
            model_name,
            "--family",
            family_id,
            "--models-dir",
            models_dir.to_str().unwrap_or(""),
        ])
        .env("CLGO_FORECAST_TOKEN", auth_token.as_str())
        .env("TABPFN_DISABLE_TELEMETRY", "1")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|_| "Impossible de lancer le sidecar Forecast".to_string())?;

    sidecar_process::save_pid(child.id());
    ACTIVE_PORT.store(port, Ordering::Relaxed);

    *sidecar.0.lock().await = Some(SidecarHandle {
        child,
        model_id: model_name.to_string(),
        family_id: family_id.to_string(),
        auth_token: auth_token.clone(),
    });

    // Attendre que le sidecar soit prêt (max 30s)
    for _ in 0..60 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if let Some((ready_port, ready_model, ready_family)) =
            health_info(port, auth_token.as_str())
        {
            if ready_model == model_name && ready_family == family_id {
                return Ok(SidecarEndpoint {
                    base_url: format!("http://127.0.0.1:{ready_port}"),
                    auth_token,
                });
            }
        }
    }

    stop(sidecar).await;
    Err("Sidecar Forecast: timeout au démarrage".into())
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
