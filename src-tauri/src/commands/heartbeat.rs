use crate::models::ScheduledWakeup;
use crate::services::{config as cfg, cron, log_reader, watcher};
use std::process::Command;
use uuid::Uuid;

#[tauri::command]
pub fn list_wakeups() -> Result<Vec<ScheduledWakeup>, String> {
    let mut config = cfg::read_config()?;

    // Auto-migrate: if no scheduled_wakeups but heartbeat exists, create one
    if config.scheduled_wakeups.is_empty() {
        let hb = &config.heartbeat;
        let time = cron::read_existing_cron_time().unwrap_or_else(|| "08:00".to_string());
        let wakeup = ScheduledWakeup {
            id: Uuid::new_v4().to_string(),
            time,
            mode: hb.mode.clone(),
            prompt: None,
            active: hb.active,
        };
        config.scheduled_wakeups.push(wakeup);
        cfg::write_config(&config)?;
    }

    // Migrate old "HH:MM" format to "YYYY-MM-DDTHH:MM"
    let mut needs_save = false;
    for w in &mut config.scheduled_wakeups {
        if !w.time.contains('T') && w.time.contains(':') {
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            w.time = format!("{}T{}", today, w.time);
            needs_save = true;
        }
    }

    // Import untracked cron entries (created by agents outside the dashboard)
    let known_ids: Vec<String> = config.scheduled_wakeups.iter().map(|w| w.id.clone()).collect();
    let untracked = cron::find_untracked_cron_entries(&known_ids);
    for entry in untracked {
        let year = chrono::Local::now().format("%Y").to_string();
        let day = if entry.day.as_str() == "*" { "01" } else { &entry.day };
        let month = if entry.month.as_str() == "*" { "01" } else { &entry.month };
        let time = format!("{}-{:0>2}-{:0>2}T{:02}:{:02}", year, month, day, entry.hour, entry.minute);

        let wakeup = ScheduledWakeup {
            id: Uuid::new_v4().to_string(),
            time,
            mode: config.heartbeat.mode.clone(),
            prompt: None,
            active: true,
        };
        config.scheduled_wakeups.push(wakeup);
        needs_save = true;
    }

    if needs_save {
        cfg::write_config(&config)?;
        // Resync crontab with new IDs
        sync_cron_from_config(&config.scheduled_wakeups)?;
    }

    Ok(config.scheduled_wakeups)
}

#[tauri::command]
pub fn create_wakeup(
    time: String,
    mode: String,
    prompt: Option<String>,
) -> Result<ScheduledWakeup, String> {
    validate_time(&time)?;
    validate_mode(&mode)?;

    let wakeup = ScheduledWakeup {
        id: Uuid::new_v4().to_string(),
        time,
        mode,
        prompt,
        active: true,
    };

    let mut config = cfg::read_config()?;
    config.scheduled_wakeups.push(wakeup.clone());
    cfg::write_config(&config)?;
    sync_cron_from_config(&config.scheduled_wakeups)?;

    Ok(wakeup)
}

#[tauri::command]
pub fn update_wakeup(wakeup: ScheduledWakeup) -> Result<(), String> {
    validate_time(&wakeup.time)?;
    validate_mode(&wakeup.mode)?;

    let mut config = cfg::read_config()?;
    let idx = config
        .scheduled_wakeups
        .iter()
        .position(|w| w.id.as_str() == wakeup.id.as_str())
        .ok_or_else(|| format!("Wakeup {} not found", wakeup.id))?;

    config.scheduled_wakeups[idx] = wakeup;
    cfg::write_config(&config)?;
    sync_cron_from_config(&config.scheduled_wakeups)?;

    Ok(())
}

#[tauri::command]
pub fn delete_wakeup(id: String) -> Result<(), String> {
    let mut config = cfg::read_config()?;
    config.scheduled_wakeups.retain(|w| w.id.as_str() != id.as_str());
    cfg::write_config(&config)?;
    sync_cron_from_config(&config.scheduled_wakeups)?;

    Ok(())
}

#[tauri::command]
pub fn get_heartbeat_config() -> Result<crate::models::HeartbeatConfig, String> {
    let config = cfg::read_config()?;
    Ok(config.heartbeat)
}

#[tauri::command]
pub fn set_heartbeat_active(active: bool) -> Result<(), String> {
    let mut config = cfg::read_config()?;
    config.heartbeat.active = active;
    cfg::write_config(&config)?;

    if !active {
        cron::clear_crontab()?;
    } else {
        sync_cron_from_config(&config.scheduled_wakeups)?;
    }

    Ok(())
}

#[tauri::command]
pub fn set_stop_at(stop_at: Option<String>) -> Result<(), String> {
    let mut config = cfg::read_config()?;
    config.heartbeat.stop_at = stop_at;
    cfg::write_config(&config)?;
    Ok(())
}

fn sync_cron_from_config(wakeups: &[ScheduledWakeup]) -> Result<(), String> {
    let active: Vec<(String, String)> = wakeups
        .iter()
        .filter(|w| w.active)
        .map(|w| (w.id.clone(), w.time.clone()))
        .collect();

    if active.is_empty() {
        cron::clear_crontab()
    } else {
        cron::sync_crontab(&active)
    }
}

#[tauri::command]
pub fn run_wakeup(id: String) -> Result<(), String> {
    let config = cfg::read_config()?;
    let wakeup = config
        .scheduled_wakeups
        .iter()
        .find(|w| w.id.as_str() == id.as_str())
        .ok_or_else(|| format!("Wakeup {} not found", id))?;

    let wrapper = dirs::home_dir()
        .expect("cannot resolve home")
        .join(".local/share/cl-go/scripts/heartbeat/go-heartbeat-wrapper.sh");

    let wrapper_str = wrapper.to_string_lossy().to_string();

    // Open Terminal.app with the wrapper script
    Command::new("open")
        .args(["-a", "Terminal.app", &wrapper_str])
        .spawn()
        .map_err(|e| format!("Cannot open Terminal: {}", e))?;

    let _ = wakeup; // wakeup context used for future prompt injection
    Ok(())
}

#[tauri::command]
pub fn get_session_status() -> Result<watcher::SessionStatus, String> {
    Ok(watcher::check_session_status())
}

#[tauri::command]
pub fn get_warnings() -> Result<Vec<log_reader::LogEntry>, String> {
    log_reader::read_warnings()
}

fn validate_time(time: &str) -> Result<(), String> {
    // Accept HH:MM or YYYY-MM-DDTHH:MM
    let re = regex::Regex::new(r"^(\d{4}-\d{2}-\d{2}T)?\d{2}:\d{2}$")
        .map_err(|e| format!("Regex error: {}", e))?;
    if !re.is_match(time) {
        return Err(format!("Invalid time: {}", time));
    }
    Ok(())
}

fn validate_mode(mode: &str) -> Result<(), String> {
    const VALID: &[&str] = &["auto", "explorer", "free", "evolve"];
    if VALID.contains(&mode) {
        Ok(())
    } else {
        Err(format!("Invalid mode: {}", mode))
    }
}
