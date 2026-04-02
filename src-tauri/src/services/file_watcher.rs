use notify::{Event, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use tauri::{AppHandle, Emitter};

const EVENT_CONFIG: &str = "fs:config-changed";
const EVENT_SESSIONS: &str = "fs:sessions-changed";
const EVENT_PERSONALITY: &str = "fs:personality-changed";
const EVENT_LOGS: &str = "fs:logs-changed";

pub fn start(app: &AppHandle) {
    let home = dirs::home_dir().expect("cannot resolve home");

    let paths: Vec<(PathBuf, &'static str)> = vec![
        (home.join(".local/share/cl-go/config.json"), EVENT_CONFIG),
        (home.join(".claude/projects/-Users-kevinh-Projects"), EVENT_SESSIONS),
        (home.join(".local/share/cl-go/memory/core"), EVENT_PERSONALITY),
        (home.join(".local/share/cl-go/logs/heartbeat"), EVENT_LOGS),
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
                    let path_str = changed_path.to_string_lossy();
                    let event_name = if path_str.contains("config.json") {
                        EVENT_CONFIG
                    } else if path_str.contains("-Users-kevinh-Projects") {
                        EVENT_SESSIONS
                    } else if path_str.contains("memory/core") {
                        EVENT_PERSONALITY
                    } else if path_str.contains("logs/heartbeat") {
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
