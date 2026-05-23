use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
pub(crate) fn current_macos_app_bundle() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| {
        eprintln!("[update] current exe: {e}");
        "update-install-path-error".to_string()
    })?;
    macos_app_bundle_from_exe(&exe).ok_or_else(|| {
        eprintln!("[update] cannot find .app bundle from {}", exe.display());
        "update-install-path-error".to_string()
    })
}

#[cfg(target_os = "linux")]
pub(crate) fn current_linux_appimage() -> Result<PathBuf, String> {
    if let Some(path) = non_empty_env_path("APPIMAGE") {
        return Ok(path);
    }

    let exe = std::env::current_exe().map_err(|e| {
        eprintln!("[update] current exe: {e}");
        "update-install-path-error".to_string()
    })?;
    if is_appimage(&exe) {
        return Ok(exe);
    }

    eprintln!(
        "[update] APPIMAGE env missing and current exe is not an AppImage: {}",
        exe.display()
    );
    Err("update-install-path-error".to_string())
}

#[cfg(target_os = "windows")]
pub(crate) fn current_windows_install_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| {
        eprintln!("[update] current exe: {e}");
        "update-install-path-error".to_string()
    })?;
    exe.parent().map(Path::to_path_buf).ok_or_else(|| {
        eprintln!("[update] current exe has no parent: {}", exe.display());
        "update-install-path-error".to_string()
    })
}

#[cfg(any(target_os = "macos", target_os = "linux", test))]
pub(crate) fn sh_quote_path(path: &Path) -> String {
    let raw = path.display().to_string();
    format!("'{}'", raw.replace('\'', "'\\''"))
}

#[cfg(any(target_os = "windows", test))]
pub(crate) fn batch_quote_path(path: &Path) -> Result<String, String> {
    let raw = path.display().to_string();
    if raw.contains('"') || raw.contains('\r') || raw.contains('\n') {
        eprintln!("[update] unsafe Windows path: {raw}");
        return Err("update-install-path-error".to_string());
    }
    Ok(format!("\"{}\"", raw.replace('%', "%%")))
}

#[cfg(any(target_os = "windows", test))]
pub(crate) fn batch_escape_text(value: &str) -> Result<String, String> {
    if value.contains('"') || value.contains('\r') || value.contains('\n') {
        eprintln!("[update] unsafe Windows value: {value}");
        return Err("update-install-path-error".to_string());
    }
    Ok(value.replace('%', "%%"))
}

#[cfg(target_os = "linux")]
fn non_empty_env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(any(target_os = "macos", test))]
fn macos_app_bundle_from_exe(exe: &Path) -> Option<PathBuf> {
    exe.ancestors()
        .find(|path| path.extension().is_some_and(|ext| ext == "app"))
        .map(Path::to_path_buf)
}

#[cfg(any(target_os = "linux", test))]
fn is_appimage(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "AppImage")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_macos_app_bundle_from_nested_exe() {
        let exe = Path::new("/Applications/CL-GO.app/Contents/MacOS/CL-GO");
        assert_eq!(
            macos_app_bundle_from_exe(exe).as_deref(),
            Some(Path::new("/Applications/CL-GO.app"))
        );
    }

    #[test]
    fn rejects_non_appimage_linux_exe() {
        assert!(!is_appimage(Path::new("/tmp/.mount_CL-GO/AppRun")));
        assert!(is_appimage(Path::new("/home/me/Apps/CL-GO.AppImage")));
    }

    #[test]
    fn shell_quote_handles_spaces_and_quotes() {
        assert_eq!(
            sh_quote_path(Path::new("/Users/me/My Apps/CL'GO.app")),
            "'/Users/me/My Apps/CL'\\''GO.app'"
        );
    }

    #[test]
    fn batch_quote_escapes_percent_values() {
        let quoted = batch_quote_path(Path::new(r"C:\Users\me\%APPDATA%\CL-GO")).unwrap();
        assert_eq!(quoted, r#""C:\Users\me\%%APPDATA%%\CL-GO""#);
    }

    #[test]
    fn batch_escape_rejects_quotes() {
        assert!(batch_escape_text(r#"/D=C:\Bad"Path"#).is_err());
    }
}
