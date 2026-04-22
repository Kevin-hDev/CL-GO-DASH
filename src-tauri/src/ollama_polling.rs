use std::time::Duration;
use tauri::Emitter;

pub fn start(handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap_or_default();
        let mut last_running = false;

        loop {
            let running = client
                .get(format!(
                    "{}/api/tags",
                    crate::services::agent_local::OLLAMA_BASE_URL
                ))
                .send()
                .await
                .is_ok();

            if running != last_running {
                let _ = handle.emit("ollama-status", running);
                last_running = running;
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}
