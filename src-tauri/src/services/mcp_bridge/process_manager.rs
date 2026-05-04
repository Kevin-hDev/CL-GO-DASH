use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use std::process::Stdio;
use std::sync::Arc;

use super::stdio_cmd;

const MAX_PROCESSES: usize = 8;
const TTL_SECS: u64 = 600;

pub struct ProcessHandle {
    pub stdin: Arc<tokio::sync::Mutex<ChildStdin>>,
    pub reader: Arc<tokio::sync::Mutex<BufReader<ChildStdout>>>,
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
    ["PATH", "HOME", "TMPDIR", "LANG", "LC_ALL"]
        .iter()
        .filter_map(|k| std::env::var(k).ok().map(|v| (k.to_string(), v)))
        .collect()
}

pub fn is_alive(connector_id: &str) -> bool {
    let pool = match POOL.lock() {
        Ok(p) => p,
        Err(_) => return false,
    };
    if let Some(entry) = pool.get(connector_id) {
        entry.child.id().is_some()
    } else {
        false
    }
}

pub fn touch(connector_id: &str) {
    if let Ok(mut pool) = POOL.lock() {
        if let Some(entry) = pool.get_mut(connector_id) {
            entry.last_used = Instant::now();
        }
    }
}

pub fn spawn(
    connector_id: &str,
    install_command: &str,
    env_tokens: &[(String, String)],
) -> Result<ProcessHandle, String> {
    let parsed = stdio_cmd::parse_install_command(install_command)?;

    let program_path = which::which(&parsed.program)
        .map_err(|_| format!("'{}' non trouvé dans le PATH", parsed.program))?;

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
        .map_err(|e| format!("impossible de lancer le process MCP : {e}"))?;

    let stdin = child.stdin.take().ok_or("stdin indisponible")?;
    let stdout = child.stdout.take().ok_or("stdout indisponible")?;

    let handle = ProcessHandle {
        stdin: Arc::new(tokio::sync::Mutex::new(stdin)),
        reader: Arc::new(tokio::sync::Mutex::new(BufReader::new(stdout))),
    };

    {
        let mut pool = POOL.lock().map_err(|_| "erreur interne")?;
        evict_expired_inner(&mut pool);
        if pool.len() >= MAX_PROCESSES && !pool.contains_key(connector_id) {
            if let Some(oldest_key) = pool
                .iter()
                .min_by_key(|(_, v)| v.last_used)
                .map(|(k, _)| k.clone())
            {
                if let Some(mut old) = pool.remove(&oldest_key) {
                    let _ = old.child.start_kill();
                }
                if let Ok(mut h) = HANDLES.lock() {
                    h.remove(&oldest_key);
                }
            }
        }
        pool.insert(
            connector_id.to_string(),
            PoolEntry {
                child,
                last_used: Instant::now(),
            },
        );
    }

    {
        let mut handles = HANDLES.lock().map_err(|_| "erreur interne")?;
        handles.insert(
            connector_id.to_string(),
            ProcessHandle {
                stdin: Arc::clone(&handle.stdin),
                reader: Arc::clone(&handle.reader),
            },
        );
    }

    Ok(handle)
}

pub fn get_handle(connector_id: &str) -> Option<ProcessHandle> {
    let handles = HANDLES.lock().ok()?;
    handles.get(connector_id).map(|h| ProcessHandle {
        stdin: Arc::clone(&h.stdin),
        reader: Arc::clone(&h.reader),
    })
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

fn evict_expired_inner(pool: &mut HashMap<String, PoolEntry>) {
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
    if !expired.is_empty() {
        if let Ok(mut handles) = HANDLES.lock() {
            for key in &expired {
                handles.remove(key);
            }
        }
    }
}
