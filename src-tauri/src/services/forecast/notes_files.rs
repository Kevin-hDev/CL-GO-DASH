use crate::services::forecast::notes::ForecastNote;
use crate::services::forecast::types::{AnnotationSource, ForecastResult, MAX_ANNOTATIONS};
use crate::services::paths::data_dir;
use std::path::PathBuf;

const MAX_NOTE_BYTES: u64 = 64 * 1024;

pub(crate) async fn ensure_dir(analysis_id: &str) -> Result<(), String> {
    tokio::fs::create_dir_all(notes_dir(analysis_id))
        .await
        .map_err(|_| "Impossible de créer le dossier des notes".into())
}

pub(crate) fn note_path(analysis_id: &str, note_id: &str) -> PathBuf {
    notes_dir(analysis_id).join(format!("{note_id}.md"))
}

pub(crate) async fn read_notes(analysis_id: &str) -> Result<Vec<ForecastNote>, String> {
    ensure_dir(analysis_id).await?;
    let mut entries = tokio::fs::read_dir(notes_dir(analysis_id))
        .await
        .map_err(|_| "Impossible de lire les notes".to_string())?;
    let mut notes = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|_| "Impossible de lire les notes".to_string())?
    {
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "md") {
            if let Some(note) = read_note_file(path).await? {
                notes.push(note);
                if notes.len() >= MAX_ANNOTATIONS {
                    break;
                }
            }
        }
    }
    notes.sort_by(|a, b| a.date.cmp(&b.date).then(a.created_at.cmp(&b.created_at)));
    Ok(notes)
}

pub(crate) async fn load_note(analysis_id: &str, note_id: &str) -> Result<ForecastNote, String> {
    let path = note_path(analysis_id, note_id);
    read_note_file(path)
        .await?
        .ok_or_else(|| "Note introuvable".to_string())
}

pub(crate) async fn write_note(note: &ForecastNote) -> Result<(), String> {
    ensure_dir(&note.analysis_id).await?;
    let path = note_path(&note.analysis_id, &note.id);
    let tmp = path.with_extension("tmp");
    tokio::fs::write(&tmp, serialize_note(note))
        .await
        .map_err(|_| "Écriture de note échouée".to_string())?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|_| "Sauvegarde de note échouée".to_string())
}

pub(crate) async fn sync_annotation_files(analysis: &ForecastResult) -> Result<(), String> {
    ensure_dir(&analysis.id).await?;
    for annotation in &analysis.annotations {
        if !safe_note_file_id(&annotation.id) {
            continue;
        }
        let path = note_path(&analysis.id, &annotation.id);
        if path.exists() {
            continue;
        }
        let note = ForecastNote {
            id: annotation.id.clone(),
            analysis_id: analysis.id.clone(),
            date: annotation.date.clone(),
            title: title_from_content(&annotation.text),
            note_type: "context".into(),
            source: source_text(&annotation.source).into(),
            content: annotation.text.clone(),
            file_path: String::new(),
            created_at: analysis.created_at.clone(),
            updated_at: analysis.created_at.clone(),
        };
        write_note(&note).await?;
    }
    Ok(())
}

async fn read_note_file(path: PathBuf) -> Result<Option<ForecastNote>, String> {
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|_| "Lecture de note échouée".to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_NOTE_BYTES {
        return Ok(None);
    }
    let raw = tokio::fs::read_to_string(&path)
        .await
        .map_err(|_| "Lecture de note échouée".to_string())?;
    let Some(mut note) = parse_note(&raw) else {
        return Ok(None);
    };
    note.file_path = path.to_string_lossy().to_string();
    Ok(Some(note))
}

fn notes_dir(analysis_id: &str) -> PathBuf {
    data_dir().join("forecast-notes").join(analysis_id)
}

fn safe_note_file_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
}

fn serialize_note(note: &ForecastNote) -> String {
    format!(
        "---\nid: \"{}\"\nanalysis_id: \"{}\"\ndate: \"{}\"\ntype: \"{}\"\nsource: \"{}\"\ncreated_at: \"{}\"\nupdated_at: \"{}\"\n---\n\n# {}\n\n{}\n",
        note.id,
        note.analysis_id,
        note.date,
        note.note_type,
        note.source,
        note.created_at,
        note.updated_at,
        note.title,
        note.content
    )
}

fn parse_note(raw: &str) -> Option<ForecastNote> {
    let raw = raw.replace("\r\n", "\n");
    let body_start = raw.find("\n---\n")? + 5;
    let meta = &raw[4..body_start - 5];
    let body = raw[body_start..].trim();
    let title = body
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .unwrap_or("Note");
    Some(ForecastNote {
        id: meta_value(meta, "id")?,
        analysis_id: meta_value(meta, "analysis_id")?,
        date: meta_value(meta, "date")?,
        title: title.to_string(),
        note_type: meta_value(meta, "type").unwrap_or_else(|| "context".into()),
        source: meta_value(meta, "source").unwrap_or_else(|| "user".into()),
        content: body
            .trim_start_matches(&format!("# {title}"))
            .trim()
            .to_string(),
        file_path: String::new(),
        created_at: meta_value(meta, "created_at").unwrap_or_default(),
        updated_at: meta_value(meta, "updated_at").unwrap_or_default(),
    })
}

fn meta_value(meta: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    meta.lines()
        .find_map(|line| line.trim().strip_prefix(&prefix).map(clean_meta_value))
}

fn clean_meta_value(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn title_from_content(content: &str) -> String {
    content
        .lines()
        .next()
        .unwrap_or("Note")
        .chars()
        .take(80)
        .collect()
}

fn source_text(source: &AnnotationSource) -> &'static str {
    match source {
        AnnotationSource::User => "user",
        AnnotationSource::Llm => "llm",
    }
}
