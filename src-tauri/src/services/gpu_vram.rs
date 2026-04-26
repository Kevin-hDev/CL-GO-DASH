use std::process::Command;

pub fn detect_vram_mb() -> Option<u64> {
    #[cfg(target_os = "macos")]
    { return detect_macos(); }
    #[cfg(target_os = "linux")]
    { return detect_linux(); }
    #[cfg(target_os = "windows")]
    { return detect_windows(); }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    None
}

pub fn compute_default_num_ctx() -> u32 {
    match detect_vram_mb() {
        Some(mb) if mb >= 24_000 => 32768,
        Some(mb) if mb >= 12_000 => 16384,
        _ => 8192,
    }
}

#[cfg(target_os = "macos")]
fn detect_macos() -> Option<u64> {
    let output = Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .ok()?;
    let raw = String::from_utf8_lossy(&output.stdout);
    let bytes: u64 = raw.trim().parse().ok()?;
    Some(bytes / 1_048_576)
}

#[cfg(target_os = "macos")]
fn detect_macos_used() -> Option<u64> {
    let output = Command::new("vm_stat").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let page_size = parse_vm_stat_page_size(&text)?;
    let active = parse_vm_stat_field(&text, "Pages active")?;
    let wired = parse_vm_stat_field(&text, "Pages wired down")?;
    let compressed = parse_vm_stat_field(&text, "Pages occupied by compressor").unwrap_or(0);
    let used_bytes = (active + wired + compressed) * page_size;
    Some(used_bytes / 1_048_576)
}

#[cfg(target_os = "macos")]
fn parse_vm_stat_page_size(text: &str) -> Option<u64> {
    let line = text.lines().next()?;
    let num: String = line.chars().filter(|c| c.is_ascii_digit()).collect();
    num.parse().ok()
}

#[cfg(target_os = "macos")]
fn parse_vm_stat_field(text: &str, field: &str) -> Option<u64> {
    for line in text.lines() {
        if line.starts_with(field) {
            let val: String = line.chars().filter(|c| c.is_ascii_digit()).collect();
            return val.parse().ok();
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn detect_linux() -> Option<u64> {
    if let Some(v) = nvidia_smi_vram() { return Some(v); }
    if let Some(v) = drm_vram_total() { return Some(v); }
    None
}

#[cfg(target_os = "linux")]
fn drm_vram_total() -> Option<u64> {
    let drm = std::fs::read_dir("/sys/class/drm").ok()?;
    for entry in drm.flatten() {
        let path = entry.path().join("device/mem_info_vram_total");
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(bytes) = raw.trim().parse::<u64>() {
                if bytes > 0 {
                    return Some(bytes / 1_048_576);
                }
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn detect_windows() -> Option<u64> {
    if let Some(v) = nvidia_smi_vram() { return Some(v); }
    if let Some(v) = registry_vram() { return Some(v); }
    None
}

#[cfg(target_os = "windows")]
fn registry_vram() -> Option<u64> {
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile", "-Command",
        "Get-ItemProperty 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}\\0*' -Name HardwareInformation.qwMemorySize -ErrorAction SilentlyContinue | Select-Object -ExpandProperty 'HardwareInformation.qwMemorySize' | Measure-Object -Maximum | Select-Object -ExpandProperty Maximum",
    ]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let output = cmd.output().ok()?;
    if !output.status.success() { return None; }
    let raw = String::from_utf8_lossy(&output.stdout);
    let bytes: u64 = raw.trim().parse().ok()?;
    if bytes > 0 { Some(bytes / 1_048_576) } else { None }
}

pub fn detect_vram_used_mb() -> Option<u64> {
    #[cfg(target_os = "macos")]
    { return detect_macos_used(); }
    #[cfg(target_os = "linux")]
    { return detect_linux_used(); }
    #[cfg(target_os = "windows")]
    { return nvidia_smi_field("memory.used"); }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    None
}

#[cfg(target_os = "linux")]
fn detect_linux_used() -> Option<u64> {
    if let Some(v) = nvidia_smi_field("memory.used") { return Some(v); }
    if let Some(v) = drm_vram_used() { return Some(v); }
    None
}

#[cfg(target_os = "linux")]
fn drm_vram_used() -> Option<u64> {
    let drm = std::fs::read_dir("/sys/class/drm").ok()?;
    for entry in drm.flatten() {
        let path = entry.path().join("device/mem_info_vram_used");
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(bytes) = raw.trim().parse::<u64>() {
                if bytes > 0 {
                    return Some(bytes / 1_048_576);
                }
            }
        }
    }
    None
}

#[cfg(not(target_os = "macos"))]
fn nvidia_smi_field(field: &str) -> Option<u64> {
    let output = Command::new("nvidia-smi")
        .args([&format!("--query-gpu={field}"), "--format=csv,noheader,nounits"])
        .output()
        .ok()?;
    if !output.status.success() { return None; }
    let raw = String::from_utf8_lossy(&output.stdout);
    raw.lines().next()?.trim().parse::<u64>().ok()
}

#[cfg(not(target_os = "macos"))]
fn nvidia_smi_vram() -> Option<u64> {
    nvidia_smi_field("memory.total")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_vram_returns_something() {
        let vram = detect_vram_mb();
        if cfg!(target_os = "macos") {
            assert!(vram.is_some(), "macOS devrait retourner la RAM système");
        }
    }

    #[test]
    fn default_num_ctx_is_reasonable() {
        let ctx = compute_default_num_ctx();
        assert!(ctx >= 8192 && ctx <= 32768);
    }
}
