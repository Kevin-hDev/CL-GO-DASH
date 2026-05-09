use notify::{Event, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const EVENT_CONFIG: &str = "fs:config-changed";
const EVENT_PERSONALITY: &str = "fs:personality-changed";
const EVENT_LOGS: &str = "fs:logs-changed";
const DEBOUNCE_MS: u64 = 200;

fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn classify_path(normalized: &str) -> Option<&'static str> {
    if normalized.contains("memory/core") || normalized.contains("memory\\core") {
        Some(EVENT_PERSONALITY)
    } else if normalized.ends_with("config.json") {
        Some(EVENT_CONFIG)
    } else if normalized.contains("logs/heartbeat") || normalized.contains("logs\\heartbeat") {
        Some(EVENT_LOGS)
    } else if normalized.contains("/inbox/") || normalized.contains("\\inbox\\") {
        Some(EVENT_PERSONALITY)
    } else {
        None
    }
}

pub fn start(app: &AppHandle) {
    let base = crate::services::paths::data_dir();

    let watch_paths: Vec<PathBuf> = vec![
        base.clone(),
        base.join("memory/core"),
        base.join("inbox"),
        base.join("logs/heartbeat"),
    ];

    let handle = app.clone();

    thread::spawn(move || {
        let (tx, rx) = mpsc::channel::<Event>();

        let mut watcher = match notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[file_watcher] failed to create watcher: {e}");
                return;
            }
        };

        for path in &watch_paths {
            if path.exists() {
                if let Err(e) = watcher.watch(path, RecursiveMode::NonRecursive) {
                    eprintln!("[file_watcher] watch error for {}: {e}", path.display());
                }
            }
        }

        loop {
            let Ok(first) = rx.recv() else { break };

            thread::sleep(Duration::from_millis(DEBOUNCE_MS));

            let mut all_paths: Vec<PathBuf> = first.paths;
            while let Ok(extra) = rx.try_recv() {
                all_paths.extend(extra.paths);
            }

            let mut emitted: HashSet<&str> = HashSet::new();
            for changed in &all_paths {
                let normalized = normalize_path(changed);
                if let Some(event_name) = classify_path(&normalized) {
                    if emitted.insert(event_name) {
                        let _ = handle.emit(event_name, ());
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_memory_core_file() {
        let p = "/Users/kevin/.local/share/cl-go-dash/memory/core/identity.md";
        assert_eq!(classify_path(p), Some(EVENT_PERSONALITY));
    }

    #[test]
    fn classify_memory_core_dir() {
        let p = "/Users/kevin/.local/share/cl-go-dash/memory/core";
        assert_eq!(classify_path(p), Some(EVENT_PERSONALITY));
    }

    #[test]
    fn classify_config_json() {
        let p = "/Users/kevin/.local/share/cl-go-dash/config.json";
        assert_eq!(classify_path(p), Some(EVENT_CONFIG));
    }

    #[test]
    fn classify_heartbeat_log() {
        let p = "/Users/kevin/.local/share/cl-go-dash/logs/heartbeat/2026-05-09.jsonl";
        assert_eq!(classify_path(p), Some(EVENT_LOGS));
    }

    #[test]
    fn classify_inbox_file() {
        let p = "/Users/kevin/.local/share/cl-go-dash/inbox/idea-discovery.md";
        assert_eq!(classify_path(p), Some(EVENT_PERSONALITY));
    }

    #[test]
    fn classify_unknown_returns_none() {
        let p = "/Users/kevin/.local/share/cl-go-dash/agent-sessions/abc.json";
        assert_eq!(classify_path(p), None);
    }

    #[test]
    fn classify_ds_store_returns_none() {
        let p = "/Users/kevin/.local/share/cl-go-dash/.DS_Store";
        assert_eq!(classify_path(p), None);
    }

    #[test]
    fn classify_temp_file_in_core() {
        let p = "/Users/kevin/.local/share/cl-go-dash/memory/core/.identity.md.tmp";
        assert_eq!(classify_path(p), Some(EVENT_PERSONALITY));
    }

    #[test]
    fn classify_windows_backslash_paths() {
        let core = "C:\\Users\\kevin\\AppData\\Local\\cl-go-dash\\memory\\core\\identity.md";
        assert_eq!(classify_path(core), Some(EVENT_PERSONALITY));

        let inbox = "C:\\Users\\kevin\\AppData\\Local\\cl-go-dash\\inbox\\idea.md";
        assert_eq!(classify_path(inbox), Some(EVENT_PERSONALITY));
    }

    #[test]
    fn dedup_emits_unique_events() {
        let paths = vec![
            "a/memory/core/identity.md",
            "a/memory/core/principles.md",
            "a/config.json",
        ];
        let mut emitted: HashSet<&str> = HashSet::new();
        for p in &paths {
            if let Some(event_name) = classify_path(p) {
                emitted.insert(event_name);
            }
        }
        assert!(emitted.contains(EVENT_PERSONALITY));
        assert!(emitted.contains(EVENT_CONFIG));
        assert_eq!(emitted.len(), 2);
    }

    #[test]
    fn normalize_path_joins_with_slash() {
        let p = PathBuf::from("/Users/kevin/test/file.md");
        let normalized = normalize_path(&p);
        assert!(normalized.contains("kevin"));
        assert!(normalized.contains("test"));
        assert!(normalized.contains("file.md"));
    }
}
