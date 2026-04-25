use crate::services::agent_local::types_ollama::{PullProgress, RegistryModel};
use crate::services::agent_local::OLLAMA_BASE_URL;
use futures_util::StreamExt;
use reqwest::Client;
use tauri::ipc::Channel;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::io::StreamReader;
use tokio_util::sync::CancellationToken;
const REGISTRY_URL: &str = "https://ollama.com";

pub async fn search_models(query: &str) -> Result<Vec<RegistryModel>, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    // Scrape la page de recherche Ollama (pas d'API JSON publique)
    let url = format!("{REGISTRY_URL}/search?q={}", urlencoded(query));
    let resp = client
        .get(&url)
        .header("User-Agent", "CL-GO-DASH/1.0")
        .send()
        .await
        .map_err(|e| format!("Recherche impossible: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Erreur registre: {}", resp.status()));
    }

    let html = resp.text().await.map_err(|e| e.to_string())?;
    Ok(parse_search_html(&html))
}

fn parse_search_html(html: &str) -> Vec<RegistryModel> {
    let mut models = Vec::new();
    // Les résultats sont dans des liens <a href="/library/MODEL_NAME">
    // avec le nom du modèle et une description
    for line in html.lines() {
        let trimmed = line.trim();
        if let Some(start) = trimmed.find("href=\"/library/") {
            let after = &trimmed[start + 15..];
            if let Some(end) = after.find('"') {
                let name = after[..end].to_string();
                if !name.is_empty() && !name.contains('/') && !models.iter().any(|m: &RegistryModel| m.name == name) {
                    models.push(RegistryModel {
                        name,
                        description: String::new(),
                        tags: vec![],
                        is_installed: false,
                    });
                }
            }
        }
    }
    models
}

pub async fn pull_model(
    name: &str,
    on_progress: &Channel<PullProgress>,
    cancel: &CancellationToken,
    pulled_digests: &mut Vec<String>,
) -> Result<(), String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(3600))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .post(format!("{OLLAMA_BASE_URL}/api/pull"))
        .json(&serde_json::json!({ "model": name, "stream": true }))
        .send()
        .await
        .map_err(|e| format!("Connexion Ollama impossible: {e}"))?;

    let byte_stream = resp
        .bytes_stream()
        .map(|r| r.map_err(|e| std::io::Error::other(e)));
    let mut lines = BufReader::new(StreamReader::new(byte_stream)).lines();
    let digests = pulled_digests;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                return Err("cancelled".to_string());
            }
            line = lines.next_line() => {
                let line = line.map_err(|e| e.to_string())?;
                let Some(line) = line else { break };
                if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&line) {
                    let status = chunk["status"].as_str().unwrap_or("").to_string();
                    if let Some(digest) = extract_digest(&status) {
                        if !digests.contains(&digest) { digests.push(digest); }
                    }
                    let completed = chunk["completed"].as_u64();
                    let total = chunk["total"].as_u64();
                    let _ = on_progress.send(PullProgress {
                        status: status.clone(), completed, total,
                    });
                    if status == "success" { return Ok(()); }
                    if let Some(err) = chunk["error"].as_str() {
                        return Err(err.to_string());
                    }
                }
            }
        }
    }
    Ok(())
}

fn extract_digest(status: &str) -> Option<String> {
    let trimmed = status.strip_prefix("pulling ")?;
    let digest = trimmed.trim();
    if digest.len() >= 12 { Some(digest.to_string()) } else { None }
}

pub fn cleanup_partial_blobs(digests: &[String]) -> usize {
    let blobs_dir = dirs::home_dir()
        .map(|h| h.join(".ollama/models/blobs"))
        .unwrap_or_default();
    if !blobs_dir.is_dir() { return 0; }

    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(&blobs_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if !name_str.contains("-partial") { continue; }
            let matches = digests.is_empty()
                || digests.iter().any(|d| name_str.contains(d));
            if matches && std::fs::remove_file(entry.path()).is_ok() {
                count += 1;
            }
        }
    }
    count
}

pub async fn delete_model(name: &str) -> Result<(), String> {
    let client = Client::new();
    let resp = client
        .delete(format!("{OLLAMA_BASE_URL}/api/delete"))
        .json(&serde_json::json!({ "model": name }))
        .send()
        .await
        .map_err(|e| format!("Erreur suppression: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Échec suppression: {body}"));
    }
    Ok(())
}

fn urlencoded(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            c if c.is_ascii_alphanumeric() || "-_.~".contains(c) => c.to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}
