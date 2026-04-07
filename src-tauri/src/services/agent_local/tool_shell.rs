use crate::services::agent_local::types_tools::ShellOutput;
use std::path::Path;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

const MAX_LINES: usize = 2000;
const MAX_BYTES: usize = 50 * 1024;
const DEFAULT_TIMEOUT: u64 = 30;

pub async fn execute_shell(
    command: &str,
    working_dir: &Path,
    timeout_secs: Option<u64>,
) -> Result<ShellOutput, String> {
    let secs = timeout_secs.unwrap_or(DEFAULT_TIMEOUT);
    let (shell, flag) = detect_shell();

    let mut child = Command::new(&shell)
        .args([&flag, command])
        .current_dir(working_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Erreur lancement shell: {e}"))?;

    let output = timeout(Duration::from_secs(secs), child.wait_with_output()).await;

    match output {
        Ok(Ok(out)) => {
            let stdout = truncate_output(&String::from_utf8_lossy(&out.stdout));
            let stderr = truncate_output(&String::from_utf8_lossy(&out.stderr));
            Ok(ShellOutput {
                stdout,
                stderr,
                exit_code: out.status.code().unwrap_or(-1),
                timed_out: false,
            })
        }
        Ok(Err(e)) => Err(format!("Erreur exécution: {e}")),
        Err(_) => {
            // kill_on_drop(true) s'en charge — child est dropped ici
            Ok(ShellOutput {
                stdout: String::new(),
                stderr: format!("Timeout après {secs}s"),
                exit_code: -1,
                timed_out: true,
            })
        }
    }
}

fn detect_shell() -> (String, String) {
    if cfg!(target_os = "windows") {
        ("powershell".to_string(), "-Command".to_string())
    } else {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        (shell, "-c".to_string())
    }
}

fn truncate_output(output: &str) -> String {
    let mut result = String::new();
    let mut line_count = 0;

    for line in output.lines() {
        if line_count >= MAX_LINES || result.len() + line.len() > MAX_BYTES {
            result.push_str("\n... [tronqué]");
            break;
        }
        if line_count > 0 {
            result.push('\n');
        }
        result.push_str(line);
        line_count += 1;
    }
    result
}
