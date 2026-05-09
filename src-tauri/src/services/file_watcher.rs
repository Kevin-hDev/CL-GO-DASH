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
const EVENT_CONNECTORS: &str = "fs:connectors-changed";
const EVENT_SKILLS: &str = "fs:skills-changed";
const EVENT_PROVIDERS: &str = "fs:providers-changed";
const DEBOUNCE_MS: u64 = 200;

fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn classify_path(n: &str) -> Option<&'static str> {
    if n.contains("memory/core") || n.contains("memory\\core") { return Some(EVENT_PERSONALITY); }
    if n.ends_with("mcp-connectors.json") { return Some(EVENT_CONNECTORS); }
    if n.ends_with("configured-providers.json") { return Some(EVENT_PROVIDERS); }
    if n.contains("/skills/") || n.contains("\\skills\\") || n.ends_with("/skills") || n.ends_with("\\skills") {
        return Some(EVENT_SKILLS);
    }
    if n.ends_with("config.json") || n.ends_with("favorite-models.json") || n.ends_with("agent-settings.json") {
        return Some(EVENT_CONFIG);
    }
    if n.contains("logs/heartbeat") || n.contains("logs\\heartbeat") { return Some(EVENT_LOGS); }
    if n.contains("/inbox/") || n.contains("\\inbox\\") { return Some(EVENT_PERSONALITY); }
    None
}

pub fn start(app: &AppHandle) {
    let base = crate::services::paths::data_dir();

    let watch_paths: Vec<(PathBuf, RecursiveMode)> = vec![
        (base.clone(), RecursiveMode::NonRecursive),
        (base.join("memory/core"), RecursiveMode::NonRecursive),
        (base.join("inbox"), RecursiveMode::NonRecursive),
        (base.join("logs/heartbeat"), RecursiveMode::NonRecursive),
        (base.join("skills"), RecursiveMode::Recursive),
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

        for (path, mode) in &watch_paths {
            if path.exists() {
                if let Err(e) = watcher.watch(path, *mode) {
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
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/memory/core/identity.md"), Some(EVENT_PERSONALITY));
    }

    #[test]
    fn classify_memory_core_dir() {
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/memory/core"), Some(EVENT_PERSONALITY));
    }

    #[test]
    fn classify_config_json() {
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/config.json"), Some(EVENT_CONFIG));
    }

    #[test]
    fn classify_heartbeat_log() {
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/logs/heartbeat/2026-05-09.jsonl"), Some(EVENT_LOGS));
    }

    #[test]
    fn classify_inbox_file() {
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/inbox/idea-discovery.md"), Some(EVENT_PERSONALITY));
    }

    #[test]
    fn classify_unknown_returns_none() {
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/agent-sessions/abc.json"), None);
    }

    #[test]
    fn classify_ds_store_returns_none() {
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/.DS_Store"), None);
    }

    #[test]
    fn classify_temp_file_in_core() {
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/memory/core/.identity.md.tmp"), Some(EVENT_PERSONALITY));
    }

    #[test]
    fn classify_windows_backslash_paths() {
        assert_eq!(classify_path("C:\\Users\\kevin\\AppData\\Local\\cl-go-dash\\memory\\core\\identity.md"), Some(EVENT_PERSONALITY));
        assert_eq!(classify_path("C:\\Users\\kevin\\AppData\\Local\\cl-go-dash\\inbox\\idea.md"), Some(EVENT_PERSONALITY));
    }

    #[test]
    fn dedup_emits_unique_events() {
        let paths = ["a/memory/core/identity.md", "a/memory/core/principles.md", "a/config.json"];
        let mut emitted: HashSet<&str> = HashSet::new();
        for p in &paths {
            if let Some(ev) = classify_path(p) { emitted.insert(ev); }
        }
        assert!(emitted.contains(EVENT_PERSONALITY));
        assert!(emitted.contains(EVENT_CONFIG));
        assert_eq!(emitted.len(), 2);
    }

    #[test]
    fn normalize_path_joins_with_slash() {
        let normalized = normalize_path(&PathBuf::from("/Users/kevin/test/file.md"));
        assert!(normalized.contains("kevin") && normalized.contains("test") && normalized.contains("file.md"));
    }

    #[test]
    fn classify_connectors_json() {
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/mcp-connectors.json"), Some(EVENT_CONNECTORS));
    }

    #[test]
    fn classify_favorite_models() {
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/favorite-models.json"), Some(EVENT_CONFIG));
    }

    #[test]
    fn classify_agent_settings() {
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/agent-settings.json"), Some(EVENT_CONFIG));
    }

    #[test]
    fn classify_skills_dir() {
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/skills"), Some(EVENT_SKILLS));
    }

    #[test]
    fn classify_skills_subdir() {
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/skills/web-search/skill.md"), Some(EVENT_SKILLS));
    }

    #[test]
    fn classify_configured_providers() {
        assert_eq!(classify_path("/Users/kevin/.local/share/cl-go-dash/configured-providers.json"), Some(EVENT_PROVIDERS));
    }
}
