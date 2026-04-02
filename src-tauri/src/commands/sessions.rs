use crate::services::{session_detail, session_parser, session_types::*};

#[tauri::command]
pub fn list_sessions(
    limit: usize,
    offset: usize,
) -> Result<Vec<SessionMeta>, String> {
    session_parser::list_sessions(limit, offset)
}

#[tauri::command]
pub fn get_session_detail(session_id: String) -> Result<SessionDetail, String> {
    session_detail::get_detail(&session_id)
}

#[tauri::command]
pub fn rename_session(session_id: String, name: String) -> Result<(), String> {
    session_parser::save_session_name(&session_id, &name)
}

#[tauri::command]
pub fn delete_session_file(file_path: String) -> Result<(), String> {
    session_parser::delete_session(&file_path)
}
