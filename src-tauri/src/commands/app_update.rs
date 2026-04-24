use serde::Serialize;

const GITHUB_REPO: &str = "Kevin-hDev/CL-GO-DASH";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateInfo {
    pub version: String,
    pub asset_url: String,
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

    let asset_url = find_platform_asset(&json).unwrap_or_default();
    if asset_url.is_empty() {
        return Ok(None);
    }

    Ok(Some(AppUpdateInfo {
        version: tag.to_string(),
        asset_url,
    }))
}

fn find_platform_asset(json: &serde_json::Value) -> Option<String> {
    let assets = json["assets"].as_array()?;
    let ext = platform_extension();

    assets.iter().find_map(|a| {
        let name = a["name"].as_str().unwrap_or_default();
        if name.ends_with(ext) {
            a["browser_download_url"].as_str().map(|s| s.to_string())
        } else {
            None
        }
    })
}

fn platform_extension() -> &'static str {
    if cfg!(target_os = "macos") {
        ".dmg"
    } else if cfg!(target_os = "windows") {
        "-setup.exe"
    } else {
        ".AppImage"
    }
}

pub(crate) fn version_gt(remote: &str, local: &str) -> bool {
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
