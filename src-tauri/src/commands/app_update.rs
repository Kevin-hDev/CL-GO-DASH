use serde::Serialize;

const GITHUB_REPO: &str = "Kevin-hDev/CL-GO-DASH";

#[derive(Serialize, Clone)]
pub struct AppUpdateInfo {
    pub version: String,
    pub download_url: String,
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

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !resp.status().is_success() {
        return Ok(None);
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("json: {}", e))?;

    let tag = json["tag_name"]
        .as_str()
        .unwrap_or_default()
        .trim_start_matches('v');
    let html_url = json["html_url"].as_str().unwrap_or_default();

    if tag.is_empty() || html_url.is_empty() {
        return Ok(None);
    }

    let current = env!("CARGO_PKG_VERSION");
    if version_gt(tag, current) {
        Ok(Some(AppUpdateInfo {
            version: tag.to_string(),
            download_url: html_url.to_string(),
        }))
    } else {
        Ok(None)
    }
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
