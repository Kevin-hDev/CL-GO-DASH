pub mod fire;
pub mod log;
pub mod next_fire;

use crate::services::config::read_config;
use chrono::{DateTime, Duration as ChronoDuration, Local};
use next_fire::next_fire_at;
use tauri::AppHandle;
use tokio::sync::watch;

/// Cap maximal de sleep entre deux ticks (résilience changement d'heure / DST).
const MAX_SLEEP_MIN: i64 = 60;

/// Scheduler interne — une unique task tokio qui dort jusqu'au prochain
/// fire en attente, ou jusqu'à un signal de reload.
pub struct Scheduler {
    reload_tx: watch::Sender<u64>,
}

impl Scheduler {
    pub fn spawn(app: AppHandle) -> Self {
        let (reload_tx, reload_rx) = watch::channel(0u64);
        tauri::async_runtime::spawn(run_loop(app, reload_rx));
        Scheduler { reload_tx }
    }

    /// À appeler après chaque mutation de config (création, update, delete,
    /// toggle, master switch). Force le scheduler à recalculer le prochain fire.
    pub fn notify_config_changed(&self) {
        let next = self.reload_tx.borrow().wrapping_add(1);
        let _ = self.reload_tx.send(next);
    }
}

async fn run_loop(app: AppHandle, mut reload_rx: watch::Receiver<u64>) {
    loop {
        let cfg = match read_config() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[scheduler] read_config error: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                continue;
            }
        };

        let now = Local::now();
        let cap = now + ChronoDuration::minutes(MAX_SLEEP_MIN);

        let next: Option<(DateTime<Local>, String)> = if cfg.heartbeat.global_paused {
            None
        } else {
            cfg.scheduled_wakeups
                .iter()
                .filter(|w| w.active && !w.paused_by_global)
                .filter_map(|w| next_fire_at(&w.schedule, now).map(|t| (t, w.id.clone())))
                .min_by_key(|(t, _)| *t)
        };

        let target_dt = match next.as_ref() {
            Some((t, _)) => (*t).min(cap),
            None => cap,
        };
        let sleep_dur = (target_dt - now)
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(60));

        tokio::select! {
            _ = tokio::time::sleep(sleep_dur) => {
                if let Some((target, id)) = next {
                    if Local::now() >= target {
                        let wakeup_opt = cfg.scheduled_wakeups
                            .iter()
                            .find(|w| w.id == id)
                            .cloned();
                        if let Some(w) = wakeup_opt {
                            let app_clone = app.clone();
                            tauri::async_runtime::spawn(async move {
                                fire::fire_wakeup(app_clone, w).await;
                            });
                        }
                    }
                }
            }
            _ = reload_rx.changed() => {
                // Config a muté → recalcul immédiat
            }
        }
    }
}
