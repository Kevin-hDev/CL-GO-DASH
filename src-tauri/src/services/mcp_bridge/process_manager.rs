use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use std::process::Stdio;
use std::sync::Arc;

use super::stdio_cmd;

const MAX_PROCESSES: usize = 8;
const TTL_SECS: u64 = 600;

pub struct ProcessHandle {
    pub stdin: Arc<tokio::sync::Mutex<ChildStdin>>,
    pub reader: Arc<tokio::sync::Mutex<BufReader<ChildStdout>>>,
    pub request_lock: Arc<tokio::sync::Mutex<()>>,
}

struct PoolEntry {
    child: Child,
    last_used: Instant,
}

static POOL: std::sync::LazyLock<Mutex<HashMap<String, PoolEntry>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

static HANDLES: std::sync::LazyLock<Mutex<HashMap<String, ProcessHandle>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

fn safe_env() -> Vec<(String, String)> {
    [
        "PATH", "HOME", "TMPDIR", "LANG", "LC_ALL",
        "USER", "SHELL",
        "XDG_DATA_HOME", "XDG_CACHE_HOME", "XDG_CONFIG_HOME",
        "NODE_PATH", "NPM_CONFIG_CACHE", "NPM_CONFIG_PREFIX",
        "DENO_DIR",
    ]
    .iter()
    .filter_map(|k| std::env::var(k).ok().map(|v| (k.to_string(), v)))
    .collect()
}

pub fn get_alive_handle(connector_id: &str) -> Option<ProcessHandle> {
    let mut pool = POOL.lock().ok()?;
    let entry = pool.get_mut(connector_id)?;
    if entry.child.id().is_none() {
        return None;
    }
    entry.last_used = Instant::now();
    let handles = HANDLES.lock().ok()?;
    handles.get(connector_id).map(|h| ProcessHandle {
        stdin: Arc::clone(&h.stdin),
        reader: Arc::clone(&h.reader),
        request_lock: Arc::clone(&h.request_lock),
    })
}

pub fn spawn(
    connector_id: &str,
    install_command: &str,
    env_tokens: &[(String, String)],
) -> Result<ProcessHandle, String> {
    let parsed = stdio_cmd::parse_install_command(install_command)?;

    let program_path = which::which(&parsed.program)
        .map_err(|_| "runtime requis non trouvé dans le PATH".to_string())?;

    let mut env = safe_env();
    for (k, v) in env_tokens {
        env.push((k.clone(), v.clone()));
    }

    let mut child = Command::new(&program_path)
        .args(&parsed.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear()
        .envs(env)
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| "impossible de démarrer le connecteur MCP".to_string())?;

    let stdin = child.stdin.take().ok_or("stdin indisponible")?;
    let stdout = child.stdout.take().ok_or("stdout indisponible")?;

    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut buf = String::new();
            loop {
                buf.clear();
                match reader.read_line(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    _ => {}
                }
            }
        });
    }

    let handle = ProcessHandle {
        stdin: Arc::new(tokio::sync::Mutex::new(stdin)),
        reader: Arc::new(tokio::sync::Mutex::new(BufReader::new(stdout))),
        request_lock: Arc::new(tokio::sync::Mutex::new(())),
    };

    let evicted = {
        let mut pool = POOL.lock().map_err(|_| "erreur interne")?;
        let evicted = evict_expired_inner(&mut pool);
        let mut lru_evicted: Option<String> = None;
        if pool.len() >= MAX_PROCESSES && !pool.contains_key(connector_id) {
            if let Some(oldest_key) = pool
                .iter()
                .min_by_key(|(_, v)| v.last_used)
                .map(|(k, _)| k.clone())
            {
                if let Some(mut old) = pool.remove(&oldest_key) {
                    let _ = old.child.start_kill();
                }
                lru_evicted = Some(oldest_key);
            }
        }
        pool.insert(
            connector_id.to_string(),
            PoolEntry {
                child,
                last_used: Instant::now(),
            },
        );
        let mut all_evicted = evicted;
        if let Some(key) = lru_evicted {
            all_evicted.push(key);
        }
        all_evicted
    };

    if !evicted.is_empty() {
        if let Ok(mut handles) = HANDLES.lock() {
            for key in &evicted {
                handles.remove(key);
            }
        }
    }

    {
        let mut handles = HANDLES.lock().map_err(|_| "erreur interne")?;
        handles.insert(
            connector_id.to_string(),
            ProcessHandle {
                stdin: Arc::clone(&handle.stdin),
                reader: Arc::clone(&handle.reader),
                request_lock: Arc::clone(&handle.request_lock),
            },
        );
    }

    Ok(handle)
}

pub fn shutdown_one(connector_id: &str) {
    if let Ok(mut pool) = POOL.lock() {
        if let Some(mut entry) = pool.remove(connector_id) {
            let _ = entry.child.start_kill();
        }
    }
    if let Ok(mut handles) = HANDLES.lock() {
        handles.remove(connector_id);
    }
}

pub fn shutdown_all() {
    if let Ok(mut pool) = POOL.lock() {
        for (_, mut entry) in pool.drain() {
            let _ = entry.child.start_kill();
        }
    }
    if let Ok(mut handles) = HANDLES.lock() {
        handles.clear();
    }
}

fn evict_expired_inner(pool: &mut HashMap<String, PoolEntry>) -> Vec<String> {
    let expired: Vec<String> = pool
        .iter()
        .filter(|(_, e)| e.last_used.elapsed().as_secs() > TTL_SECS)
        .map(|(k, _)| k.clone())
        .collect();
    for key in &expired {
        if let Some(mut entry) = pool.remove(key) {
            let _ = entry.child.start_kill();
        }
    }
    expired
}
