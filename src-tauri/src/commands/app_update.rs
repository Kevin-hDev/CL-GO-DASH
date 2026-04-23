use serde::Serialize;
use tauri::ipc::Channel;

const GITHUB_REPO: &str = "Kevin-hDev/CL-GO-DASH";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateInfo {
    pub version: String,
    pub dmg_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub completed: u64,
    pub total: u64,
}

#[tauri::command]
pub async fn check_app_update() -> Result<Option<AppUpdateInfo>, String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "CL-GO-DASH")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("network: {}", e))?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("json: {}", e))?;

    let tag = json["tag_name"]
        .as_str()
        .unwrap_or_default()
        .trim_start_matches('v');

    let current = env!("CARGO_PKG_VERSION");
    if tag.is_empty() || !version_gt(tag, current) {
        return Ok(None);
    }

    let dmg_url = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find_map(|a| {
                let name = a["name"].as_str().unwrap_or_default();
                if name.ends_with(".dmg") {
                    a["browser_download_url"].as_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_default();

    if dmg_url.is_empty() {
        return Ok(None);
    }

    Ok(Some(AppUpdateInfo {
        version: tag.to_string(),
        dmg_url,
    }))
}

#[tauri::command]
pub async fn download_app_update(
    dmg_url: String,
    on_progress: Channel<DownloadProgress>,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(&dmg_url)
        .header("User-Agent", "CL-GO-DASH")
        .send()
        .await
        .map_err(|e| format!("network: {}", e))?;

    if !resp.status().is_success() {
        return Err("download failed".into());
    }

    let total = resp.content_length().unwrap_or(0);
    let tmp = std::env::temp_dir().join("CL-GO-update.dmg");

    let mut file = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| format!("fs: {}", e))?;

    use tokio::io::AsyncWriteExt;
    use futures_util::StreamExt;

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("stream: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("write: {}", e))?;
        downloaded += chunk.len() as u64;
        let _ = on_progress.send(DownloadProgress {
            completed: downloaded,
            total,
        });
    }

    file.flush().await.map_err(|e| format!("flush: {}", e))?;
    drop(file);

    std::process::Command::new("open")
        .arg(&tmp)
        .spawn()
        .map_err(|e| format!("open: {}", e))?;

    Ok(())
}

fn version_gt(remote: &str, local: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.split('.')
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let r = parse(remote);
    let l = parse(local);
    for i in 0..r.len().max(l.len()) {
        let rv = r.get(i).copied().unwrap_or(0);
        let lv = l.get(i).copied().unwrap_or(0);
        if rv > lv {
            return true;
        }
        if rv < lv {
            return false;
        }
    }
    false
}
