use serde::Serialize;

use super::app_update_assets::{asset_extension, current_platform};
use super::app_update_notes::compact_release_notes;

const GITHUB_REPO: &str = "Kevin-hDev/CL-GO-DASH";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateInfo {
    pub version: String,
    pub asset_url: String,
    pub title: Option<String>,
    pub published_at: Option<String>,
    pub notes: Option<String>,
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
        .map_err(|e| {
            eprintln!("[update] check network: {e}");
            "update-check-error".to_string()
        })?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| {
        eprintln!("[update] parse json: {e}");
        "update-check-error".to_string()
    })?;

    Ok(app_update_from_release(&json, env!("CARGO_PKG_VERSION")))
}

fn app_update_from_release(json: &serde_json::Value, current: &str) -> Option<AppUpdateInfo> {
    let tag = json["tag_name"].as_str()?.trim_start_matches('v');
    if tag.is_empty() || !version_gt(tag, current) {
        return None;
    }

    let asset_url = find_platform_asset(json)?;
    Some(AppUpdateInfo {
        version: tag.to_string(),
        asset_url,
        title: optional_string(json["name"].as_str()),
        published_at: optional_string(json["published_at"].as_str()),
        notes: json["body"].as_str().and_then(compact_release_notes),
    })
}

fn optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn find_platform_asset(json: &serde_json::Value) -> Option<String> {
    find_asset_by_extension(json, asset_extension(current_platform()))
}

fn find_asset_by_extension(json: &serde_json::Value, ext: &str) -> Option<String> {
    let assets = json["assets"].as_array()?;
    assets.iter().find_map(|a| {
        let name = a["name"].as_str().unwrap_or_default();
        if name.ends_with(ext) {
            a["browser_download_url"].as_str().map(|s| s.to_string())
        } else {
            None
        }
    })
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn finds_linux_deb_when_appimage_is_also_present() {
        let release = json!({
            "assets": [
                {
                    "name": "CL-GO_0.9.1_amd64.AppImage",
                    "browser_download_url": "https://example.invalid/app.AppImage"
                },
                {
                    "name": "CL-GO_0.9.1_amd64.deb",
                    "browser_download_url": "https://example.invalid/app.deb"
                }
            ]
        });

        assert_eq!(
            find_asset_by_extension(&release, ".deb").as_deref(),
            Some("https://example.invalid/app.deb")
        );
    }

    #[test]
    fn ignores_assets_with_partial_extension_matches() {
        let release = json!({
            "assets": [
                {
                    "name": "CL-GO_0.9.1_amd64.deb.sha256",
                    "browser_download_url": "https://example.invalid/app.deb.sha256"
                }
            ]
        });

        assert!(find_asset_by_extension(&release, ".deb").is_none());
    }

    #[test]
    fn builds_update_info_with_release_notes() {
        let ext = asset_extension(current_platform());
        let release = json!({
            "tag_name": "v99.0.0",
            "name": "CL-GO v99.0.0",
            "published_at": "2026-06-30T12:00:00Z",
            "body": "### Features\n- **Context details** added\n",
            "assets": [
                {
                    "name": format!("CL-GO_99.0.0{}", ext),
                    "browser_download_url": "https://example.invalid/app"
                }
            ]
        });

        let info = app_update_from_release(&release, "0.9.3").expect("update");

        assert_eq!(info.version, "99.0.0");
        assert_eq!(info.title.as_deref(), Some("CL-GO v99.0.0"));
        assert_eq!(info.published_at.as_deref(), Some("2026-06-30T12:00:00Z"));
        assert!(info
            .notes
            .as_deref()
            .unwrap_or_default()
            .contains("Context details"));
    }

    #[test]
    fn builds_update_info_without_release_notes() {
        let ext = asset_extension(current_platform());
        let release = json!({
            "tag_name": "v99.0.0",
            "body": "",
            "assets": [
                {
                    "name": format!("CL-GO_99.0.0{}", ext),
                    "browser_download_url": "https://example.invalid/app"
                }
            ]
        });

        let info = app_update_from_release(&release, "0.9.3").expect("update");

        assert_eq!(info.version, "99.0.0");
        assert!(info.notes.is_none());
    }
}
