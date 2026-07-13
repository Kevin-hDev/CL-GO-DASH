use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;

const MAX_OUTPUT: usize = 65_536;

pub fn replace_file(source: &Path, destination: &Path) -> Result<(), String> {
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };
    let source = wide(source);
    let destination = wide(destination);
    let success = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    (success != 0)
        .then_some(())
        .ok_or_else(|| "stockage privé indisponible".to_string())
}

pub fn secure_acl(path: &Path) -> Result<(), String> {
    let sid = current_user_sid()?;
    let grant = if path.is_dir() {
        format!("*{sid}:(OI)(CI)F")
    } else {
        format!("*{sid}:(F)")
    };
    let status = Command::new("icacls")
        .arg(path.as_os_str())
        .args(["/inheritance:r", "/grant:r"])
        .arg(&grant)
        .args(["/remove:g", "*S-1-1-0", "*S-1-5-11", "*S-1-5-32-545"])
        .status()
        .map_err(|_| "stockage privé indisponible".to_string())?;
    if !status.success() {
        return Err("stockage privé indisponible".to_string());
    }
    verify_acl(path, &sid)
}

fn current_user_sid() -> Result<String, String> {
    let output = Command::new("whoami")
        .args(["/user", "/fo", "csv", "/nh"])
        .output()
        .map_err(|_| "stockage privé indisponible".to_string())?;
    if !output.status.success() || output.stdout.len() > MAX_OUTPUT {
        return Err("stockage privé indisponible".to_string());
    }
    let text =
        String::from_utf8(output.stdout).map_err(|_| "stockage privé indisponible".to_string())?;
    let sid = text
        .split(',')
        .nth(1)
        .map(|value| value.trim().trim_matches('"').to_string())
        .ok_or_else(|| "stockage privé indisponible".to_string())?;
    let valid = regex::Regex::new(r"^S-\d(?:-\d+)+$")
        .map_err(|_| "stockage privé indisponible".to_string())?;
    valid
        .is_match(&sid)
        .then_some(sid)
        .ok_or_else(|| "stockage privé indisponible".to_string())
}

fn verify_acl(path: &Path, sid: &str) -> Result<(), String> {
    let output = Command::new("icacls")
        .arg(path.as_os_str())
        .output()
        .map_err(|_| "stockage privé indisponible".to_string())?;
    if !output.status.success() || output.stdout.len() > MAX_OUTPUT {
        return Err("stockage privé indisponible".to_string());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let forbidden = ["S-1-1-0", "S-1-5-11", "S-1-5-32-545"];
    if !text.contains(sid) || forbidden.iter().any(|entry| text.contains(entry)) {
        return Err("stockage privé indisponible".to_string());
    }
    Ok(())
}

fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(Some(0)).collect()
}
