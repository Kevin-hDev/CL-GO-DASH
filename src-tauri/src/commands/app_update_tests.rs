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
fn builds_update_info_without_release_notes() {
    let ext = asset_extension(current_platform());
    let release = json!({
        "tag_name": "v99.0.0",
        "name": "CL-GO v99.0.0",
        "published_at": "2026-06-30T12:00:00Z",
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
    assert!(info.notes_by_locale.is_none());
}

#[test]
fn ignores_release_notes_from_body() {
    let ext = asset_extension(current_platform());
    let release = json!({
        "tag_name": "v99.0.0",
        "body": "### App release notes\n- Body notes are not the app source.\n",
        "assets": [
            {
                "name": format!("CL-GO_99.0.0{}", ext),
                "browser_download_url": "https://example.invalid/app"
            }
        ]
    });

    let info = app_update_from_release(&release, "0.9.3").expect("update");

    assert_eq!(info.version, "99.0.0");
    assert!(info.notes_by_locale.is_none());
}

#[test]
fn rejects_unsafe_release_versions() {
    let ext = asset_extension(current_platform());
    let release = json!({
        "tag_name": "v99.0.0/notes",
        "assets": [
            {
                "name": format!("CL-GO_99.0.0{}", ext),
                "browser_download_url": "https://example.invalid/app"
            }
        ]
    });

    assert!(app_update_from_release(&release, "0.9.3").is_none());
}
