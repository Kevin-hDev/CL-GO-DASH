use std::process::Command;

const CRON_TAG: &str = "# CL-GO Heartbeat";

fn wrapper_path() -> String {
    let home = dirs::home_dir().expect("cannot resolve home");
    home.join(".local/share/cl-go/scripts/heartbeat/go-heartbeat-wrapper.sh")
        .to_string_lossy()
        .to_string()
}

fn read_crontab() -> Result<String, String> {
    let output = Command::new("crontab")
        .arg("-l")
        .output()
        .map_err(|e| format!("Cannot read crontab: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        // No crontab = empty
        Ok(String::new())
    }
}

fn write_crontab(content: &str) -> Result<(), String> {
    let mut child = Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Cannot start crontab: {}", e))?;

    use std::io::Write;
    if let Some(ref mut stdin) = child.stdin {
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| format!("Cannot write crontab: {}", e))?;
    }

    let status = child
        .wait()
        .map_err(|e| format!("crontab process error: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("crontab returned non-zero".to_string())
    }
}

fn remove_clgo_entries(crontab: &str) -> String {
    crontab
        .lines()
        .filter(|line| !line.contains(CRON_TAG))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Build a cron line: "MM HH * * * /path/to/wrapper.sh # CL-GO Heartbeat <id>"
fn build_cron_line(time_hhmm: &str, wakeup_id: &str) -> Result<String, String> {
    let parts: Vec<&str> = time_hhmm.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid time format: {}", time_hhmm));
    }

    let hour: u8 = parts[0]
        .parse()
        .map_err(|_| format!("Invalid hour: {}", parts[0]))?;
    let minute: u8 = parts[1]
        .parse()
        .map_err(|_| format!("Invalid minute: {}", parts[1]))?;

    if hour > 23 || minute > 59 {
        return Err(format!("Time out of range: {}", time_hhmm));
    }

    Ok(format!(
        "{} {} * * * {} {} {}",
        minute,
        hour,
        wrapper_path(),
        CRON_TAG,
        wakeup_id
    ))
}

/// Sync crontab with the list of active wakeups
pub fn sync_crontab(
    wakeups: &[(String, String)], // Vec of (id, time "HH:MM")
) -> Result<(), String> {
    let current = read_crontab()?;
    let mut cleaned = remove_clgo_entries(&current);

    for (id, time) in wakeups {
        let line = build_cron_line(time, id)?;
        if !cleaned.is_empty() && !cleaned.ends_with('\n') {
            cleaned.push('\n');
        }
        cleaned.push_str(&line);
    }

    // Ensure trailing newline
    if !cleaned.is_empty() && !cleaned.ends_with('\n') {
        cleaned.push('\n');
    }

    write_crontab(&cleaned)
}

/// Remove all CL-GO entries from crontab
pub fn clear_crontab() -> Result<(), String> {
    let current = read_crontab()?;
    let cleaned = remove_clgo_entries(&current);
    if cleaned.trim().is_empty() {
        // Remove crontab entirely
        Command::new("crontab")
            .arg("-r")
            .output()
            .ok();
        return Ok(());
    }
    write_crontab(&cleaned)
}
