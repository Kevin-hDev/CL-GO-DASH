use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;
use sysinfo::{Pid, System};

fn pid_file_path() -> PathBuf {
    crate::services::paths::data_dir().join("ollama-sidecar.pid")
}

pub fn save_pid(pid: u32) {
    let _ = std::fs::write(pid_file_path(), pid.to_string());
}

pub fn read_saved_pid() -> Option<u32> {
    let pid: u32 = std::fs::read_to_string(pid_file_path()).ok()?.trim().parse().ok()?;
    if pid < 2 { return None; }
    Some(pid)
}

pub fn clear_pid_file() {
    let _ = std::fs::remove_file(pid_file_path());
}

pub fn kill_orphan_sidecar() {
    let Some(pid) = read_saved_pid() else { return };
    clear_pid_file();

    if !is_ollama_process(pid) {
        eprintln!("[ollama] pid={pid} n'est plus ollama, ignoré");
        return;
    }

    eprintln!("[ollama] orphelin détecté pid={pid}, kill");
    tree_kill(pid);
}

fn is_ollama_process(pid: u32) -> bool {
    #[cfg(unix)]
    {
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output();
        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().contains("ollama"),
            Err(_) => false,
        }
    }
    #[cfg(windows)]
    {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
            .output();
        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_lowercase().contains("ollama"),
            Err(_) => false,
        }
    }
}

pub fn tree_kill(pid: u32) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F", "/T"])
            .output();
        eprintln!("[ollama] tree-kill Windows pid={pid}");
        return;
    }

    #[cfg(unix)]
    {
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let children = collect_children(&sys, Pid::from_u32(pid));
        for &child_pid in children.iter().rev() {
            let raw = child_pid.as_u32() as i32;
            eprintln!("[ollama] kill child pid={raw}");
            unsafe { libc::kill(raw, libc::SIGTERM); }
        }

        unsafe { libc::kill(pid as i32, libc::SIGTERM); }

        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(3) {
            if unsafe { libc::kill(pid as i32, 0) != 0 } {
                eprintln!("[ollama] arbre pid={pid} arrêté");
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        for &child_pid in children.iter().rev() {
            unsafe { libc::kill(child_pid.as_u32() as i32, libc::SIGKILL); }
        }
        unsafe { libc::kill(pid as i32, libc::SIGKILL); }
        eprintln!("[ollama] SIGKILL arbre pid={pid}");
    }
}

const MAX_CHILDREN: usize = 256;
const MAX_DEPTH: u32 = 10;

#[cfg(unix)]
fn collect_children(sys: &System, parent: Pid) -> Vec<Pid> {
    let mut result = Vec::new();
    collect_children_inner(sys, parent, &mut result, 0);
    result
}

#[cfg(unix)]
fn collect_children_inner(sys: &System, parent: Pid, result: &mut Vec<Pid>, depth: u32) {
    if depth >= MAX_DEPTH || result.len() >= MAX_CHILDREN { return; }
    for (pid, proc) in sys.processes() {
        if result.len() >= MAX_CHILDREN { return; }
        if proc.parent() == Some(parent) {
            result.push(*pid);
            collect_children_inner(sys, *pid, result, depth + 1);
        }
    }
}

pub fn kill_process(child: &mut Child) {
    let pid = child.id();
    eprintln!("[ollama] kill sidecar pid={pid}");
    tree_kill(pid);

    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(Some(_)) = child.try_wait() {
            eprintln!("[ollama] sidecar arrêté proprement");
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    let _ = child.kill();
    let _ = child.wait();
}

pub fn release_vram_blocking() {
    let base_url = crate::services::ollama_port::base_url();
    let url = format!("{base_url}/api/generate");
    let body = serde_json::json!({ "model": "", "keep_alive": "0" });
    let _ = std::thread::Builder::new()
        .name("vram-release".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            if let Ok(rt) = rt {
                let _ = rt.block_on(async {
                    let client = reqwest::Client::builder()
                        .timeout(Duration::from_secs(3))
                        .build()?;
                    client.post(&url).json(&body).send().await
                });
            }
            eprintln!("[ollama] VRAM release demandée");
        });
    std::thread::sleep(Duration::from_millis(500));
}
