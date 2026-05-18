const MAX_ENTRY_LEN: usize = 4000;

#[tauri::command]
pub fn frontend_diagnostic_log(entry: String) -> Result<(), String> {
    let mut sanitized = String::with_capacity(entry.len().min(MAX_ENTRY_LEN));
    for ch in entry.chars().take(MAX_ENTRY_LEN) {
        sanitized.push(if ch.is_control() { ' ' } else { ch });
    }
    eprintln!("[frontend-diagnostic] {sanitized}");
    Ok(())
}
