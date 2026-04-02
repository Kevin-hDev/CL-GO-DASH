use std::fs;
use std::path::PathBuf;

fn pid_file_path() -> PathBuf {
    let home = dirs::home_dir().expect("cannot resolve home");
    home.join(".local/share/cl-go/logs/heartbeat/session.pid")
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum SessionStatus {
    Idle,
    Running { pid: u32, since: String },
    Crashed,
}

pub fn check_session_status() -> SessionStatus {
    let path = pid_file_path();

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return SessionStatus::Idle,
    };

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return SessionStatus::Idle;
    }

    let pid: u32 = match lines[0].trim().parse() {
        Ok(p) => p,
        Err(_) => return SessionStatus::Crashed,
    };

    let since = lines.get(1).unwrap_or(&"").trim().to_string();

    // Check if process is alive (kill -0)
    if is_process_alive(pid) {
        SessionStatus::Running { pid, since }
    } else {
        // PID file exists but process is dead = crash
        SessionStatus::Crashed
    }
}

fn is_process_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}
