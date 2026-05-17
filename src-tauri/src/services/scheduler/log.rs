use crate::models::{WakeupRun, WakeupRunStatus};
use chrono::{DateTime, Local, Utc};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

const MAX_LINES: usize = 500;

fn log_path() -> PathBuf {
    crate::services::paths::data_dir()
        .join("logs")
        .join("wakeups.jsonl")
}

pub async fn log_ok(
    wakeup_id: &str,
    scheduled_for: DateTime<Local>,
    session_id: &str,
    tokens: u32,
) {
    let entry = WakeupRun {
        wakeup_id: wakeup_id.into(),
        scheduled_for: scheduled_for.to_rfc3339(),
        fired_at: Utc::now().to_rfc3339(),
        status: WakeupRunStatus::Ok,
        error: None,
        session_id: Some(session_id.into()),
        tokens: Some(tokens),
    };
    let _ = append(entry).await;
}

pub async fn log_err(wakeup_id: &str, scheduled_for: DateTime<Local>, err: &str) {
    let entry = WakeupRun {
        wakeup_id: wakeup_id.into(),
        scheduled_for: scheduled_for.to_rfc3339(),
        fired_at: Utc::now().to_rfc3339(),
        status: WakeupRunStatus::Error,
        error: Some(generic_error(err)),
        session_id: None,
        tokens: None,
    };
    let _ = append(entry).await;
}

pub async fn log_missed(wakeup_id: &str, scheduled_for: DateTime<Local>) {
    let entry = WakeupRun {
        wakeup_id: wakeup_id.into(),
        scheduled_for: scheduled_for.to_rfc3339(),
        fired_at: Utc::now().to_rfc3339(),
        status: WakeupRunStatus::Missed,
        error: Some("Réveil raté : l'application était indisponible".into()),
        session_id: None,
        tokens: None,
    };
    let _ = append(entry).await;
}

pub async fn list_runs(wakeup_id: Option<&str>) -> Result<Vec<WakeupRun>, String> {
    let content = match tokio::fs::read_to_string(log_path()).await {
        Ok(c) => c,
        Err(_) => return Ok(Vec::new()),
    };
    Ok(parse_runs(&content, wakeup_id))
}

fn generic_error(err: &str) -> String {
    let lower = err.to_lowercase();
    if lower.contains("rate limit") {
        "Limite de requêtes atteinte".into()
    } else if lower.contains("clé api") || lower.contains("unauthorized") || lower.contains("auth")
    {
        "Authentification échouée".into()
    } else if lower.contains("ollama") {
        "Ollama indisponible".into()
    } else {
        "Le réveil a échoué".into()
    }
}

async fn append(entry: WakeupRun) -> Result<(), String> {
    let path = log_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|_| "Erreur journal wakeup".to_string())?;
    }
    let line = format!(
        "{}\n",
        serde_json::to_string(&entry).map_err(|_| "Erreur journal wakeup".to_string())?
    );

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(|_| "Erreur journal wakeup".to_string())?;
    file.write_all(line.as_bytes())
        .await
        .map_err(|_| "Erreur journal wakeup".to_string())?;
    drop(file);

    trim_if_needed(&path).await?;
    Ok(())
}

fn parse_runs(content: &str, wakeup_id: Option<&str>) -> Vec<WakeupRun> {
    let mut runs: Vec<WakeupRun> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<WakeupRun>(line).ok())
        .filter(|run| wakeup_id.map(|id| run.wakeup_id == id).unwrap_or(true))
        .collect();
    runs.sort_by(|a, b| b.fired_at.cmp(&a.fired_at));
    runs.truncate(MAX_LINES);
    runs
}

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
        .map_err(|_| "Erreur journal wakeup".to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(id: &str, fired_at: &str, status: WakeupRunStatus) -> String {
        serde_json::to_string(&WakeupRun {
            wakeup_id: id.into(),
            scheduled_for: "2026-05-17T08:00:00+02:00".into(),
            fired_at: fired_at.into(),
            status,
            error: None,
            session_id: None,
            tokens: None,
        })
        .unwrap()
    }

    #[test]
    fn parse_runs_filters_and_sorts_newest_first() {
        let content = format!(
            "{}\n{}\n",
            line("a", "2026-05-17T08:00:00Z", WakeupRunStatus::Ok),
            line("b", "2026-05-17T09:00:00Z", WakeupRunStatus::Missed)
        );
        let runs = parse_runs(&content, None);
        assert_eq!(runs[0].wakeup_id, "b");
        assert_eq!(parse_runs(&content, Some("a")).len(), 1);
    }

    #[test]
    fn generic_error_does_not_return_raw_message() {
        assert_eq!(generic_error("token secret leaked"), "Le réveil a échoué");
        assert_eq!(generic_error("Ollama HTTP 500"), "Ollama indisponible");
    }
}
