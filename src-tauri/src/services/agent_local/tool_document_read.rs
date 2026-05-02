use crate::services::agent_local::security::validate_read_path;
use crate::services::agent_local::types_tools::ToolResult;
use std::path::Path;

pub async fn read_document(
    path: &str,
    _pages: Option<&str>,
    working_dir: &Path,
) -> ToolResult {
    if path.is_empty() {
        return ToolResult::err("Le paramètre 'path' est requis");
    }

    let resolved = super::tool_office_utils::resolve_path(path, working_dir);

    let validated = match validate_read_path(&resolved, working_dir) {
        Ok(p) => p,
        Err(e) => return ToolResult::err(e),
    };

    let ext = validated
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "pdf" => read_pdf(&validated),
        "docx" => read_docx(&validated),
        _ => ToolResult::err(
            "Format non supporté. Formats acceptés : pdf, docx",
        ),
    }
}

fn read_pdf(path: &Path) -> ToolResult {
    let path_str = match path.to_str() {
        Some(s) => s,
        None => return ToolResult::err("Chemin invalide"),
    };

    let text = match pdf_extract::extract_text(path_str) {
        Ok(t) => t,
        Err(_) => return ToolResult::err("Impossible de lire le fichier PDF"),
    };

    let char_count = text.chars().count();
    let json = serde_json::json!({
        "format": "pdf",
        "text": text,
        "char_count": char_count
    });
    ToolResult::ok(json.to_string())
}

fn read_docx(path: &Path) -> ToolResult {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return ToolResult::err("Impossible d'ouvrir le fichier"),
    };

    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return ToolResult::err("Fichier DOCX invalide ou corrompu"),
    };

    let xml_content = match archive.by_name("word/document.xml") {
        Ok(mut entry) => {
            use std::io::Read;
            let mut buf = String::new();
            if entry.read_to_string(&mut buf).is_err() {
                return ToolResult::err("Impossible de lire le contenu du document");
            }
            buf
        }
        Err(_) => return ToolResult::err("Structure DOCX invalide"),
    };

    let text = extract_text_from_ooxml(&xml_content);
    let char_count = text.chars().count();
    let json = serde_json::json!({
        "format": "docx",
        "text": text,
        "char_count": char_count
    });
    ToolResult::ok(json.to_string())
}

fn extract_text_from_ooxml(xml: &str) -> String {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut result = String::new();
    let mut in_paragraph = false;
    let mut para_text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                b"p" => {
                    in_paragraph = true;
                    para_text.clear();
                }
                _ => {}
            },
            Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                b"p" => {
                    if !para_text.is_empty() {
                        if !result.is_empty() {
                            result.push('\n');
                        }
                        result.push_str(&para_text);
                    }
                    in_paragraph = false;
                    para_text.clear();
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) if in_paragraph => {
                if let Ok(s) = e.unescape() {
                    para_text.push_str(&s);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    result
}
