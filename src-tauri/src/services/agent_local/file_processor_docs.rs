use crate::services::agent_local::types_tools::FileContent;
use std::io::Read;
use std::path::Path;

pub async fn extract_pdf(path: &Path) -> Result<FileContent, String> {
    let bytes = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
    let result = std::panic::catch_unwind(|| {
        pdf_extract::extract_text_from_mem(&bytes)
    });
    match result {
        Ok(Ok(text)) => Ok(FileContent::Text(text)),
        Ok(Err(e)) => Err(format!("Erreur extraction PDF: {e}")),
        Err(_) => Err("PDF malformé (extraction impossible)".into()),
    }
}

pub async fn extract_docx(path: &Path) -> Result<FileContent, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    let mut xml = String::new();
    archive
        .by_name("word/document.xml")
        .map_err(|e| format!("Fichier DOCX invalide: {e}"))?
        .read_to_string(&mut xml)
        .map_err(|e| e.to_string())?;

    let text = parse_docx_xml(&xml);
    Ok(FileContent::Text(text))
}

fn parse_docx_xml(xml: &str) -> String {
    use quick_xml::{events::Event, Reader};
    let mut reader = Reader::from_str(xml);
    let mut text = String::new();
    let mut in_t = false;
    let mut in_paragraph = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local = e.local_name();
                if local.as_ref() == b"t" {
                    in_t = true;
                } else if local.as_ref() == b"p" {
                    in_paragraph = true;
                }
            }
            Ok(Event::Text(e)) if in_t => {
                if let Ok(t) = e.unescape() {
                    text.push_str(&t);
                }
            }
            Ok(Event::End(e)) => {
                let local = e.local_name();
                if local.as_ref() == b"t" {
                    in_t = false;
                } else if local.as_ref() == b"p" && in_paragraph {
                    text.push('\n');
                    in_paragraph = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    text
}

pub async fn extract_xlsx(path: &Path) -> Result<FileContent, String> {
    use calamine::{open_workbook, Reader, Xlsx};

    let mut wb: Xlsx<_> =
        open_workbook(path).map_err(|e| format!("Erreur ouverture XLSX: {e}"))?;

    let sheet_names: Vec<String> = wb.sheet_names().to_vec();
    let mut text = String::new();

    for name in sheet_names {
        if let Ok(range) = wb.worksheet_range(&name) {
            text.push_str(&format!("## {name}\n\n"));
            for row in range.rows() {
                let cells: Vec<String> = row.iter().map(|c| format!("{c}")).collect();
                text.push_str(&cells.join("\t"));
                text.push('\n');
            }
            text.push('\n');
        }
    }
    Ok(FileContent::Text(text))
}
