use crate::services::agent_local::file_processor_docs;
use crate::services::agent_local::types_tools::{FileContent, ProcessedFile};
use base64::prelude::*;
use std::path::Path;

const MAX_FILE_SIZE: u64 = 20 * 1024 * 1024;
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp"];
const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "rs", "ts", "tsx", "js", "jsx", "py", "go", "toml",
    "yaml", "yml", "json", "html", "css", "sh", "bash", "zsh", "csv",
    "xml", "sql", "rb", "java", "c", "cpp", "h", "hpp", "swift",
];

pub async fn process_file(path: &Path) -> Result<ProcessedFile, String> {
    let meta = tokio::fs::metadata(path)
        .await
        .map_err(|e| format!("Fichier introuvable: {e}"))?;

    if meta.len() > MAX_FILE_SIZE {
        return Err("Fichier trop volumineux (max 20MB)".into());
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| e.to_str().unwrap_or("").to_lowercase())
        .unwrap_or_default();
    let mime = detect_mime(&ext);

    let content = match ext.as_str() {
        e if IMAGE_EXTENSIONS.contains(&e) => process_image(path).await?,
        e if TEXT_EXTENSIONS.contains(&e) => process_text(path).await?,
        "pdf" => file_processor_docs::extract_pdf(path).await?,
        "docx" => file_processor_docs::extract_docx(path).await?,
        "xlsx" | "xls" => file_processor_docs::extract_xlsx(path).await?,
        _ => process_text(path).await.unwrap_or_else(|_| {
            FileContent::Text("Format non supporté".into())
        }),
    };

    Ok(ProcessedFile {
        name,
        mime_type: mime,
        content,
        size: meta.len(),
    })
}

async fn process_image(path: &Path) -> Result<FileContent, String> {
    let bytes = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
    let encoded = BASE64_STANDARD.encode(&bytes);
    Ok(FileContent::Image(encoded))
}

async fn process_text(path: &Path) -> Result<FileContent, String> {
    let text = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("Erreur lecture: {e}"))?;
    Ok(FileContent::Text(text))
}

fn detect_mime(ext: &str) -> String {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "json" => "application/json",
        "html" => "text/html",
        "csv" => "text/csv",
        _ => "text/plain",
    }
    .to_string()
}
