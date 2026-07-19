use crate::services::git::{blob_preview, commit_files, history};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use std::io::Write;
use std::path::Path;

const MAX_TEXT_SIZE: usize = 2 * 1024 * 1024;
const BINARY_EXTENSIONS: &[&str] = &["docx", "pdf"];
const SPREADSHEET_EXTENSIONS: &[&str] = &["xlsx", "xls", "csv", "ods", "xlsm", "tsv"];

#[tauri::command]
pub async fn list_git_commits(
    path: String,
    expected_branch: String,
    cursor: Option<String>,
    limit: Option<usize>,
) -> Result<history::CommitPage, String> {
    let repo_path = super::git::registered_project_path(&path).await?;
    tokio::task::spawn_blocking(move || {
        history::list_commits(&repo_path, &expected_branch, cursor.as_deref(), limit)
    })
    .await
    .map_err(|_| unavailable())?
    .map_err(|_| unavailable())
}

#[tauri::command]
pub async fn list_git_commit_files(
    path: String,
    expected_branch: String,
    commit_id: String,
) -> Result<Vec<commit_files::CommitFile>, String> {
    let repo_path = super::git::registered_project_path(&path).await?;
    tokio::task::spawn_blocking(move || {
        commit_files::list_commit_files(&repo_path, &expected_branch, &commit_id)
    })
    .await
    .map_err(|_| unavailable())?
    .map_err(|_| unavailable())
}

#[tauri::command]
pub async fn list_git_uncommitted_files(
    path: String,
    expected_branch: String,
) -> Result<history::UncommittedSnapshot, String> {
    let repo_path = super::git::registered_project_path(&path).await?;
    tokio::task::spawn_blocking(move || {
        history::list_uncommitted(&repo_path, &expected_branch)
    })
    .await
    .map_err(|_| unavailable())?
    .map_err(|_| unavailable())
}

#[tauri::command]
pub async fn read_git_file_preview(
    path: String,
    expected_branch: String,
    commit_id: String,
    file_path: String,
    use_parent: bool,
) -> Result<String, String> {
    let bytes = load_blob(path, expected_branch, commit_id, file_path, use_parent).await?;
    if bytes.len() > MAX_TEXT_SIZE || bytes.contains(&0) {
        return Err(unavailable());
    }
    String::from_utf8(bytes).map_err(|_| unavailable())
}

#[tauri::command]
pub async fn read_git_binary_preview(
    path: String,
    expected_branch: String,
    commit_id: String,
    file_path: String,
    use_parent: bool,
) -> Result<String, String> {
    let _ = validate_extension(&file_path, BINARY_EXTENSIONS)?;
    let bytes = load_blob(path, expected_branch, commit_id, file_path, use_parent).await?;
    Ok(B64.encode(bytes))
}

#[tauri::command]
pub async fn read_git_spreadsheet_preview(
    path: String,
    expected_branch: String,
    commit_id: String,
    file_path: String,
    use_parent: bool,
    sheet: Option<String>,
    max_rows: Option<usize>,
) -> Result<String, String> {
    let extension = validate_extension(&file_path, SPREADSHEET_EXTENSIONS)?;
    let bytes = load_blob(path, expected_branch, commit_id, file_path, use_parent).await?;
    let mut temp = tempfile::Builder::new()
        .suffix(&format!(".{extension}"))
        .tempfile()
        .map_err(|_| unavailable())?;
    temp.write_all(&bytes).map_err(|_| unavailable())?;
    super::file_preview_office::read_spreadsheet_path(
        temp.path().to_path_buf(),
        sheet,
        max_rows,
    )
    .await
    .map_err(|_| unavailable())
}

async fn load_blob(
    path: String,
    expected_branch: String,
    commit_id: String,
    file_path: String,
    use_parent: bool,
) -> Result<Vec<u8>, String> {
    let repo_path = super::git::registered_project_path(&path).await?;
    tokio::task::spawn_blocking(move || {
        blob_preview::read_blob(
            &repo_path,
            &expected_branch,
            &commit_id,
            &file_path,
            use_parent,
        )
    })
    .await
    .map_err(|_| unavailable())?
    .map_err(|_| unavailable())
}

fn validate_extension(file_path: &str, allowed: &[&str]) -> Result<String, String> {
    let extension = Path::new(file_path)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(unavailable)?;
    allowed
        .contains(&extension.as_str())
        .then_some(())
        .ok_or_else(unavailable)?;
    Ok(extension)
}

fn unavailable() -> String {
    "Aperçu Git indisponible".to_string()
}
