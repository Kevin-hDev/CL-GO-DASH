use crate::services::terminal::PtyManager;
use tauri::State;

#[tauri::command]
pub fn pty_spawn(
    cwd: Option<String>,
    cols: u16,
    rows: u16,
    state: State<'_, PtyManager>,
    app: tauri::AppHandle,
) -> Result<u32, String> {
    state.spawn(&app, cwd.as_deref(), cols, rows)
}

#[tauri::command]
pub fn pty_write(id: u32, data: String, state: State<'_, PtyManager>) -> Result<(), String> {
    state.write(id, data.as_bytes())
}

#[tauri::command]
pub fn pty_resize(
    id: u32,
    cols: u16,
    rows: u16,
    state: State<'_, PtyManager>,
) -> Result<(), String> {
    state.resize(id, cols, rows)
}

#[tauri::command]
pub fn pty_kill(id: u32, state: State<'_, PtyManager>) -> Result<(), String> {
    state.kill(id)
}
