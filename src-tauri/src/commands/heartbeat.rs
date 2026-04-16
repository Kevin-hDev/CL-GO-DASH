use crate::models::{HeartbeatConfig, ScheduledWakeup, WakeupSchedule};
use crate::services::config as cfg;
use crate::services::scheduler::Scheduler;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateWakeupInput {
    pub name: String,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub schedule: WakeupSchedule,
    #[serde(default)]
    pub description: String,
}

#[tauri::command]
pub fn list_wakeups() -> Result<Vec<ScheduledWakeup>, String> {
    let config = cfg::read_config()?;
    Ok(config.scheduled_wakeups)
}

#[tauri::command]
pub fn create_wakeup(
    input: CreateWakeupInput,
    scheduler: State<'_, Scheduler>,
) -> Result<ScheduledWakeup, String> {
    validate_provider(&input.provider)?;
    validate_non_empty(&input.name, "name")?;
    validate_non_empty(&input.model, "model")?;
    validate_non_empty(&input.prompt, "prompt")?;
    validate_schedule(&input.schedule)?;

    let wakeup = ScheduledWakeup {
        id: Uuid::new_v4().to_string(),
        name: input.name,
        model: input.model,
        provider: input.provider,
        prompt: input.prompt,
        schedule: input.schedule,
        description: input.description,
        active: true,
        paused_by_global: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let mut config = cfg::read_config()?;
    config.scheduled_wakeups.push(wakeup.clone());
    cfg::write_config(&config)?;
    scheduler.notify_config_changed();

    Ok(wakeup)
}

#[tauri::command]
pub fn update_wakeup(
    wakeup: ScheduledWakeup,
    scheduler: State<'_, Scheduler>,
) -> Result<(), String> {
    validate_provider(&wakeup.provider)?;
    validate_non_empty(&wakeup.name, "name")?;
    validate_non_empty(&wakeup.model, "model")?;
    validate_non_empty(&wakeup.prompt, "prompt")?;
    validate_schedule(&wakeup.schedule)?;

    let mut config = cfg::read_config()?;
    let idx = config
        .scheduled_wakeups
        .iter()
        .position(|w| w.id == wakeup.id)
        .ok_or_else(|| format!("Wakeup {} not found", wakeup.id))?;

    config.scheduled_wakeups[idx] = wakeup;
    cfg::write_config(&config)?;
    scheduler.notify_config_changed();

    Ok(())
}

#[tauri::command]
pub fn delete_wakeup(id: String, scheduler: State<'_, Scheduler>) -> Result<(), String> {
    let mut config = cfg::read_config()?;
    config.scheduled_wakeups.retain(|w| w.id != id);
    cfg::write_config(&config)?;
    scheduler.notify_config_changed();
    Ok(())
}

#[tauri::command]
pub fn set_wakeup_active(
    id: String,
    active: bool,
    scheduler: State<'_, Scheduler>,
) -> Result<(), String> {
    let mut config = cfg::read_config()?;

    if config.heartbeat.global_paused {
        return Err("Réveils en veille — désactive d'abord le master switch.".into());
    }

    let w = config
        .scheduled_wakeups
        .iter_mut()
        .find(|w| w.id == id)
        .ok_or_else(|| format!("Wakeup {} not found", id))?;
    w.active = active;
    w.paused_by_global = false;

    cfg::write_config(&config)?;
    scheduler.notify_config_changed();
    Ok(())
}

#[tauri::command]
pub fn set_global_paused(paused: bool, scheduler: State<'_, Scheduler>) -> Result<(), String> {
    let mut config = cfg::read_config()?;

    if paused {
        for w in &mut config.scheduled_wakeups {
            if w.active {
                w.paused_by_global = true;
                w.active = false;
            }
        }
    } else {
        for w in &mut config.scheduled_wakeups {
            if w.paused_by_global {
                w.active = true;
                w.paused_by_global = false;
            }
        }
    }

    config.heartbeat.global_paused = paused;
    cfg::write_config(&config)?;
    scheduler.notify_config_changed();
    Ok(())
}

#[tauri::command]
pub fn get_heartbeat_config() -> Result<HeartbeatConfig, String> {
    let config = cfg::read_config()?;
    Ok(config.heartbeat)
}

const ALLOWED_PROVIDERS: &[&str] = &["ollama"];

fn validate_provider(provider: &str) -> Result<(), String> {
    if ALLOWED_PROVIDERS.contains(&provider) {
        Ok(())
    } else {
        Err(format!("Provider non supporté : {}", provider))
    }
}

fn validate_non_empty(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("Champ {} requis", field))
    } else {
        Ok(())
    }
}

fn validate_schedule(schedule: &WakeupSchedule) -> Result<(), String> {
    let time_re = regex::Regex::new(r"^\d{2}:\d{2}$").map_err(|e| e.to_string())?;
    let dt_re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$").map_err(|e| e.to_string())?;

    match schedule {
        WakeupSchedule::Once { datetime } => {
            if !dt_re.is_match(datetime) {
                return Err(format!("Datetime invalide : {} (attendu YYYY-MM-DDTHH:MM)", datetime));
            }
        }
        WakeupSchedule::Daily { time } => {
            if !time_re.is_match(time) {
                return Err(format!("Heure invalide : {} (attendu HH:MM)", time));
            }
        }
        WakeupSchedule::Weekly { weekday, time } => {
            if *weekday > 6 {
                return Err(format!("Jour invalide : {} (0..6 attendu)", weekday));
            }
            if !time_re.is_match(time) {
                return Err(format!("Heure invalide : {} (attendu HH:MM)", time));
            }
        }
    }
    Ok(())
}
