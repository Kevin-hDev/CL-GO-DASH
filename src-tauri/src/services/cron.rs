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

/// Build a cron line from "HH:MM" or "YYYY-MM-DDTHH:MM"
fn build_cron_line(time_str: &str, wakeup_id: &str) -> Result<String, String> {
    // Parse: "2026-04-03T22:00" or "22:00"
    let (day, month, hour, minute) = if time_str.contains('T') {
        let parts: Vec<&str> = time_str.split('T').collect();
        let date_parts: Vec<&str> = parts[0].split('-').collect();
        let time_parts: Vec<&str> = parts[1].split(':').collect();
        let d: u8 = date_parts.get(2).unwrap_or(&"*").parse().unwrap_or(0);
        let m: u8 = date_parts.get(1).unwrap_or(&"*").parse().unwrap_or(0);
        let h: u8 = time_parts[0].parse().map_err(|_| "Invalid hour")?;
        let min: u8 = time_parts[1].parse().map_err(|_| "Invalid minute")?;
        (format!("{}", d), format!("{}", m), h, min)
    } else {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid time: {}", time_str));
        }
        let h: u8 = parts[0].parse().map_err(|_| "Invalid hour")?;
        let min: u8 = parts[1].parse().map_err(|_| "Invalid minute")?;
        ("*".to_string(), "*".to_string(), h, min)
    };

    // MM HH DD MM * /path <id> # tag id
    Ok(format!(
        "{} {} {} {} * {} {} {} {}",
        minute, hour, day, month,
        wrapper_path(), wakeup_id, CRON_TAG, wakeup_id
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

/// Read the first CL-GO cron time, or any cron entry time as fallback
pub fn read_existing_cron_time() -> Option<String> {
    let crontab = read_crontab().ok()?;
    for line in crontab.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Cron format: MIN HOUR ...
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            let min: u8 = parts[0].parse().ok()?;
            let hour: u8 = parts[1].parse().ok()?;
            return Some(format!("{:02}:{:02}", hour, min));
        }
    }
    None
}

/// Parse all cron entries that point to the heartbeat wrapper.
/// Returns Vec of (minute, hour, day, month) for untracked entries.
pub fn find_untracked_cron_entries(known_ids: &[String]) -> Vec<CronEntry> {
    let crontab = match read_crontab() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let wrapper = wrapper_path();
    let mut entries = Vec::new();

    for line in crontab.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Must contain our wrapper path
        if !trimmed.contains(&wrapper) && !trimmed.contains("go-heartbeat-wrapper") {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let min: u8 = match parts[0].parse() { Ok(v) => v, Err(_) => continue };
        let hour: u8 = match parts[1].parse() { Ok(v) => v, Err(_) => continue };
        let day = parts[2].to_string();
        let month = parts[3].to_string();

        // Check if this entry has a known ID
        let has_known_id = known_ids.iter().any(|id| trimmed.contains(id.as_str()));
        if !has_known_id {
            entries.push(CronEntry { minute: min, hour, day, month });
        }
    }

    entries
}

#[derive(Debug)]
pub struct CronEntry {
    pub minute: u8,
    pub hour: u8,
    pub day: String,
    pub month: String,
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
