use crate::services::agent_local::types_ollama::{PullProgress, RegistryModel};
use futures_util::StreamExt;
use reqwest::Client;
use tauri::ipc::Channel;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::io::StreamReader;

const LOCAL_URL: &str = "http://localhost:11434";
const REGISTRY_URL: &str = "https://ollama.com";

pub async fn search_models(query: &str) -> Result<Vec<RegistryModel>, String> {
    let client = Client::new();
    let url = format!("{REGISTRY_URL}/api/search?q={}", urlencoded(query));
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Recherche impossible: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Erreur registre: {}", resp.status()));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let models = json["models"]
        .as_array()
        .or_else(|| json["items"].as_array())
        .unwrap_or(&Vec::new())
        .iter()
        .map(|m| RegistryModel {
            name: m["name"].as_str().unwrap_or_default().to_string(),
            description: m["description"].as_str().unwrap_or_default().to_string(),
            tags: m["tags"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            is_installed: false,
        })
        .collect();

    Ok(models)
}

pub async fn pull_model(
    name: &str,
    on_progress: &Channel<PullProgress>,
) -> Result<(), String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(3600))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .post(format!("{LOCAL_URL}/api/pull"))
        .json(&serde_json::json!({ "model": name, "stream": true }))
        .send()
        .await
        .map_err(|e| format!("Connexion Ollama impossible: {e}"))?;

    let byte_stream = resp
        .bytes_stream()
        .map(|r| r.map_err(|e| std::io::Error::other(e)));
    let mut lines = BufReader::new(StreamReader::new(byte_stream)).lines();

    while let Some(line) = lines.next_line().await.map_err(|e| e.to_string())? {
        if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&line) {
            let status = chunk["status"].as_str().unwrap_or("").to_string();
            let completed = chunk["completed"].as_u64();
            let total = chunk["total"].as_u64();
            let _ = on_progress.send(PullProgress {
                status: status.clone(),
                completed,
                total,
            });
            if status == "success" {
                return Ok(());
            }
            if let Some(err) = chunk["error"].as_str() {
                return Err(err.to_string());
            }
        }
    }
    Ok(())
}

pub async fn delete_model(name: &str) -> Result<(), String> {
    let client = Client::new();
    let resp = client
        .delete(format!("{LOCAL_URL}/api/delete"))
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
