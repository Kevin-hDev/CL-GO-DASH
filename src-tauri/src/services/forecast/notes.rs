use super::notes_files::{load_note, note_path, read_notes, sync_annotation_files, write_note};
use crate::services::forecast::storage;
use crate::services::forecast::types::{
    Annotation, AnnotationSource, ForecastResult, MAX_ANNOTATIONS,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::process::Command;
use std::sync::LazyLock;

const MAX_NOTE_BYTES: usize = 64 * 1024;
const MAX_TITLE_CHARS: usize = 120;
const MAX_FIELD_CHARS: usize = 80;
static SAFE_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-f0-9\-]{1,64}$").unwrap());
static SAFE_TYPE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z_]{1,32}$").unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastNote {
    pub id: String,
    pub analysis_id: String,
    pub date: String,
    pub title: String,
    pub note_type: String,
    pub source: String,
    pub content: String,
    pub file_path: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ForecastNoteCreateRequest {
    pub analysis_id: String,
    pub date: String,
    pub title: String,
    pub note_type: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ForecastNoteUpdateRequest {
    pub analysis_id: String,
    pub note_id: String,
    pub date: String,
    pub title: String,
    pub note_type: String,
    pub content: String,
}

pub async fn list(analysis_id: &str) -> Result<Vec<ForecastNote>, String> {
    validate_id(analysis_id, "Identifiant d'analyse invalide")?;
    let analysis = storage::load(analysis_id).await?;
    sync_annotation_files(&analysis).await?;
    read_notes(analysis_id).await
}

pub async fn create(request: ForecastNoteCreateRequest) -> Result<ForecastNote, String> {
    let mut analysis = storage::load(&request.analysis_id).await?;
    if analysis.annotations.len() >= MAX_ANNOTATIONS {
        return Err("Limite de notes atteinte".into());
    }
    let now = chrono::Utc::now().to_rfc3339();
    let note = ForecastNote {
        id: uuid::Uuid::new_v4().to_string(),
        analysis_id: request.analysis_id,
        date: clean_field(&request.date)?,
        title: clean_title(&request.title)?,
        note_type: clean_note_type(&request.note_type)?,
        source: "user".into(),
        content: clean_content(&request.content)?,
        file_path: String::new(),
        created_at: now.clone(),
        updated_at: now,
    };
    write_note(&note).await?;
    upsert_annotation(&mut analysis, &note)?;
    storage::save(&mut analysis).await?;
    load_note(&note.analysis_id, &note.id).await
}

pub async fn update(request: ForecastNoteUpdateRequest) -> Result<ForecastNote, String> {
    validate_id(&request.note_id, "Identifiant de note invalide")?;
    let mut current = load_note(&request.analysis_id, &request.note_id).await?;
    current.date = clean_field(&request.date)?;
    current.title = clean_title(&request.title)?;
    current.note_type = clean_note_type(&request.note_type)?;
    current.content = clean_content(&request.content)?;
    current.updated_at = chrono::Utc::now().to_rfc3339();
    write_note(&current).await?;
    let mut analysis = storage::load(&request.analysis_id).await?;
    upsert_annotation(&mut analysis, &current)?;
    storage::save(&mut analysis).await?;
    load_note(&request.analysis_id, &request.note_id).await
}

pub async fn delete(analysis_id: &str, note_id: &str) -> Result<(), String> {
    let mut analysis = storage::load(analysis_id).await?;
    validate_id(note_id, "Identifiant de note invalide")?;
    let path = note_path(analysis_id, note_id);
    if path.exists() {
        tokio::fs::remove_file(&path)
            .await
            .map_err(|_| "Suppression échouée".to_string())?;
    }
    analysis
        .annotations
        .retain(|annotation| annotation.id != note_id);
    storage::save(&mut analysis).await
}

pub fn open(analysis_id: &str, note_id: &str) -> Result<(), String> {
    validate_id(analysis_id, "Identifiant d'analyse invalide")?;
    validate_id(note_id, "Identifiant de note invalide")?;
    let path = note_path(analysis_id, note_id);
    if !path.is_file() {
        return Err("Note introuvable".into());
    }
    #[cfg(target_os = "macos")]
    return spawn_cmd("open", &[path.as_os_str()]);
    #[cfg(target_os = "linux")]
    return spawn_cmd("xdg-open", &[path.as_os_str()]);
    #[cfg(target_os = "windows")]
    return spawn_cmd("explorer.exe", &[path.as_os_str()]);
}

fn validate_id(id: &str, message: &str) -> Result<(), String> {
    if !SAFE_ID.is_match(id) {
        return Err(message.into());
    }
    Ok(())
}

fn clean_field(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > MAX_FIELD_CHARS || trimmed.contains('\0') {
        return Err("Champ de note invalide".into());
    }
    Ok(trimmed.replace(['\n', '\r', '"'], ""))
}

fn clean_title(value: &str) -> Result<String, String> {
    let cleaned = clean_field(value)?;
    if cleaned.chars().count() > MAX_TITLE_CHARS {
        return Err("Titre de note invalide".into());
    }
    Ok(cleaned)
}

fn clean_note_type(value: &str) -> Result<String, String> {
    let cleaned = value.trim();
    if !SAFE_TYPE.is_match(cleaned) {
        return Err("Type de note invalide".into());
    }
    Ok(cleaned.into())
}

fn clean_content(value: &str) -> Result<String, String> {
    if value.len() > MAX_NOTE_BYTES || value.contains('\0') {
        return Err("Contenu de note invalide".into());
    }
    Ok(value.trim().to_string())
}

fn spawn_cmd(command: &str, args: &[&OsStr]) -> Result<(), String> {
    Command::new(command)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|_| "Impossible d'ouvrir la note".to_string())
}

fn upsert_annotation(analysis: &mut ForecastResult, note: &ForecastNote) -> Result<(), String> {
    let source = match note.source.as_str() {
        "llm" => AnnotationSource::Llm,
        _ => AnnotationSource::User,
    };
    if let Some(annotation) = analysis
        .annotations
        .iter_mut()
        .find(|item| item.id == note.id)
    {
        annotation.date = note.date.clone();
        annotation.text = note.title.clone();
        annotation.source = source;
        return Ok(());
    }
    if analysis.annotations.len() >= MAX_ANNOTATIONS {
        return Err("Limite de notes atteinte".into());
    }
    analysis.annotations.push(Annotation {
        id: note.id.clone(),
        date: note.date.clone(),
        text: note.title.clone(),
        source,
    });
    Ok(())
}
