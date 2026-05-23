use crate::services::agent_local::{app_handle_global, types_tools::SearchResult};
use std::process::Child;
use tauri::Manager;
use tokio::sync::Mutex;

pub struct SearxngSidecar(pub Mutex<Option<SearxngHandle>>);

pub struct SearxngHandle {
    child: Child,
    port: u16,
}

impl SearxngSidecar {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    let app = app_handle_global::get().ok_or_else(|| "SearXNG: app non initialisée".to_string())?;
    let base_url = ensure_running(app).await?;
    super::client::search(&base_url, query).await
}

async fn ensure_running(app: &tauri::AppHandle) -> Result<String, String> {
    let state = app.state::<SearxngSidecar>();
    let mut guard = state.0.lock().await;
    if let Some(handle) = guard.as_mut() {
        match handle.child.try_wait() {
            Ok(None) => return Ok(base_url(handle.port)),
            Ok(Some(_)) => *guard = None,
            Err(_) => return Err("SearXNG: état processus illisible".to_string()),
        }
    }

    super::process::kill_orphan_sidecar();
    let source = super::paths::source_dir(app)?;
    let python = super::runtime::ensure_runtime(&source).await?;
    let port = super::settings::find_free_port()?;
    let settings = super::settings::write_settings(port)?;
    let child = super::process::spawn(&python, &source, &settings, port)?;
    let pid = child.id();
    super::process::save_pid(pid);
    let url = base_url(port);
    if let Err(e) = wait_until_ready(&url).await {
        super::process::kill_child_process(child);
        return Err(e);
    }
    eprintln!("[searxng] sidecar démarré pid={pid} port={port}");
    *guard = Some(SearxngHandle { child, port });
    Ok(url)
}

pub async fn stop(sidecar: &SearxngSidecar) {
    let mut guard = sidecar.0.lock().await;
    if let Some(handle) = guard.take() {
        super::process::kill_child_process(handle.child);
    }
    super::process::clear_pid_file();
}

async fn wait_until_ready(base_url: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{}/healthz", base_url);
    for _ in 0..40 {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                return Ok(());
            }
        }
    }
    Err("SearXNG: timeout au démarrage".to_string())
}

fn base_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}
