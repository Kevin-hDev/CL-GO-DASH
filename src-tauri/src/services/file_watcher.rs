use notify::{Event, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use tauri::{AppHandle, Emitter};

const EVENT_CONFIG: &str = "fs:config-changed";
const EVENT_PERSONALITY: &str = "fs:personality-changed";
const EVENT_LOGS: &str = "fs:logs-changed";

pub fn start(app: &AppHandle) {
    let base = crate::services::paths::data_dir();

    let paths: Vec<(PathBuf, &'static str)> = vec![
        (base.clone(), EVENT_CONFIG),
        (base.join("memory/core"), EVENT_PERSONALITY),
        (base.join("logs/heartbeat"), EVENT_LOGS),
    ];

    let handle = app.clone();

    thread::spawn(move || {
        let (tx, rx) = mpsc::channel::<Event>();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })
        .expect("Failed to create file watcher");

        for (path, _) in &paths {
            if path.exists() {
                let mode = if path.is_dir() {
                    RecursiveMode::NonRecursive
                } else {
                    RecursiveMode::NonRecursive
                };
                if let Err(e) = watcher.watch(path, mode) {
                    eprintln!("Watch error for {}: {}", path.display(), e);
                }
            }
        }

        // Debounce: collect events then emit
        loop {
            if let Ok(event) = rx.recv() {
                // Drain pending events (debounce 200ms)
                thread::sleep(std::time::Duration::from_millis(200));
                while rx.try_recv().is_ok() {}

                // Determine which event to emit
                for changed_path in &event.paths {
                    let normalized: String = changed_path
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy().into_owned())
                        .collect::<Vec<_>>()
                        .join("/");
                    let event_name = if normalized.contains("config.json") {
                        EVENT_CONFIG
                    } else if normalized.contains("memory/core") {
                        EVENT_PERSONALITY
                    } else if normalized.contains("logs/heartbeat") {
                        EVENT_LOGS
                    } else {
                        continue;
                    };

                    let _ = handle.emit(event_name, ());
                }
            }
        }
    });
}
