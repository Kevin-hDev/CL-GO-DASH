use super::notes_files::{
    load_note, note_path, read_notes, remove_note, sync_annotation_files, write_note,
};
use super::{notes_transaction, notes_validation};
use crate::services::forecast::storage;
use crate::services::forecast::types::{
    Annotation, AnnotationSource, ForecastResult, MAX_ANNOTATIONS,
};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::process::Command;

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
    notes_validation::id(analysis_id, "Identifiant d'analyse invalide")?;
    let _guard = notes_transaction::lock().await;
    let analysis = storage::load(analysis_id).await?;
    sync_annotation_files(&analysis).await?;
    read_notes(analysis_id).await
}

pub async fn create(request: ForecastNoteCreateRequest) -> Result<ForecastNote, String> {
    notes_validation::id(&request.analysis_id, "Identifiant d'analyse invalide")?;
    let date = notes_validation::field(&request.date)?;
    let title = notes_validation::title(&request.title)?;
    let note_type = notes_validation::note_type(&request.note_type)?;
    let content = notes_validation::content(&request.content)?;
    let _guard = notes_transaction::lock().await;
    let mut analysis = storage::load(&request.analysis_id).await?;
    if analysis.annotations.len() >= MAX_ANNOTATIONS {
        return Err("Limite de notes atteinte".into());
    }
    let now = chrono::Utc::now().to_rfc3339();
    let note = ForecastNote {
        id: uuid::Uuid::new_v4().to_string(),
        analysis_id: request.analysis_id,
        date,
        title,
        note_type,
        source: "user".into(),
        content,
        file_path: String::new(),
        created_at: now.clone(),
        updated_at: now,
    };
    upsert_annotation(&mut analysis, &note)?;
    write_note(&note).await?;
    if let Err(error) = storage::save(&mut analysis).await {
        return Err(notes_transaction::rollback_created(&note, error).await);
    }
    load_note(&note.analysis_id, &note.id).await
}

pub async fn update(request: ForecastNoteUpdateRequest) -> Result<ForecastNote, String> {
    notes_validation::id(&request.analysis_id, "Identifiant d'analyse invalide")?;
    notes_validation::id(&request.note_id, "Identifiant de note invalide")?;
    let date = notes_validation::field(&request.date)?;
    let title = notes_validation::title(&request.title)?;
    let note_type = notes_validation::note_type(&request.note_type)?;
    let content = notes_validation::content(&request.content)?;
    let _guard = notes_transaction::lock().await;
    let mut current = load_note(&request.analysis_id, &request.note_id).await?;
    let previous = current.clone();
    current.date = date;
    current.title = title;
    current.note_type = note_type;
    current.content = content;
    current.updated_at = chrono::Utc::now().to_rfc3339();
    let mut analysis = storage::load(&request.analysis_id).await?;
    upsert_annotation(&mut analysis, &current)?;
    write_note(&current).await?;
    if let Err(error) = storage::save(&mut analysis).await {
        return Err(notes_transaction::rollback_updated(&previous, error).await);
    }
    load_note(&request.analysis_id, &request.note_id).await
}

pub async fn delete(analysis_id: &str, note_id: &str) -> Result<(), String> {
    notes_validation::id(analysis_id, "Identifiant d'analyse invalide")?;
    notes_validation::id(note_id, "Identifiant de note invalide")?;
    let _guard = notes_transaction::lock().await;
    let mut analysis = storage::load(analysis_id).await?;
    let path = note_path(analysis_id, note_id);
    let previous = if tokio::fs::try_exists(&path)
        .await
        .map_err(|_| "Suppression échouée".to_string())?
    {
        Some(load_note(analysis_id, note_id).await?)
    } else {
        None
    };
    remove_note(analysis_id, note_id).await?;
    analysis
        .annotations
        .retain(|annotation| annotation.id != note_id);
    if let Err(error) = storage::save(&mut analysis).await {
        return Err(notes_transaction::rollback_deleted(previous.as_ref(), error).await);
    }
    Ok(())
}

pub fn open(analysis_id: &str, note_id: &str) -> Result<(), String> {
    notes_validation::id(analysis_id, "Identifiant d'analyse invalide")?;
    notes_validation::id(note_id, "Identifiant de note invalide")?;
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
