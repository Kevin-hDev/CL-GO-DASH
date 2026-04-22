use chrono::Utc;
use serde::Serialize;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

const MAX_LINES: usize = 500;

#[derive(Debug, Serialize)]
pub struct WakeupRun {
    pub wakeup_id: String,
    pub fired_at: String,
    pub status: String, // "ok" | "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u32>,
}

fn log_path() -> PathBuf {
    crate::services::paths::data_dir()
        .join("logs")
        .join("wakeups.jsonl")
}

pub async fn log_ok(wakeup_id: &str, session_id: &str, tokens: u32) {
    let entry = WakeupRun {
        wakeup_id: wakeup_id.into(),
        fired_at: Utc::now().to_rfc3339(),
        status: "ok".into(),
        error: None,
        session_id: Some(session_id.into()),
        tokens: Some(tokens),
    };
    let _ = append(entry).await;
}

pub async fn log_err(wakeup_id: &str, err: &str) {
    let entry = WakeupRun {
        wakeup_id: wakeup_id.into(),
        fired_at: Utc::now().to_rfc3339(),
        status: "error".into(),
        error: Some(err.into()),
        session_id: None,
        tokens: None,
    };
    let _ = append(entry).await;
}

async fn append(entry: WakeupRun) -> Result<(), String> {
    let path = log_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let line = format!(
        "{}\n",
        serde_json::to_string(&entry).map_err(|e| e.to_string())?
    );

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(|e| e.to_string())?;
    file.write_all(line.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    drop(file);

    trim_if_needed(&path).await?;
    Ok(())
}

/// Rolling 500 : si le fichier dépasse MAX_LINES, on tronque en gardant
/// seulement les MAX_LINES dernières lignes. Best-effort.
async fn trim_if_needed(path: &PathBuf) -> Result<(), String> {
    let content = match tokio::fs::read_to_string(path).await {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= MAX_LINES {
        return Ok(());
    }
    let start = lines.len() - MAX_LINES;
    let trimmed = lines[start..].join("\n") + "\n";
    tokio::fs::write(path, trimmed)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
