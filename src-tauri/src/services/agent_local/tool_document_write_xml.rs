use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::io::Cursor;

/// Génère le XML OOXML pour word/document.xml à partir d'une liste de blocs JSON.
pub fn build_document_xml(blocks: &[serde_json::Value]) -> Result<String, String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(|e| format!("XML error: {e}"))?;

    let mut doc_start = BytesStart::new("w:document");
    doc_start.push_attribute((
        "xmlns:w",
        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
    ));
    writer
        .write_event(Event::Start(doc_start))
        .map_err(|e| format!("XML error: {e}"))?;

    writer
        .write_event(Event::Start(BytesStart::new("w:body")))
        .map_err(|e| format!("XML error: {e}"))?;

    for block in blocks {
        let block_type = block["type"].as_str().unwrap_or("");
        match block_type {
            "heading" => write_heading(&mut writer, block)?,
            "paragraph" => write_paragraph(&mut writer, block)?,
            "table" => write_table(&mut writer, block)?,
            "list" => super::tool_document_write_list::write_list(&mut writer, block)?,
            _ => return Err(format!("Bloc document inconnu: {block_type}")),
        }
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:body")))
        .map_err(|e| format!("XML error: {e}"))?;
    writer
        .write_event(Event::End(BytesEnd::new("w:document")))
        .map_err(|e| format!("XML error: {e}"))?;

    let bytes = writer.into_inner().into_inner();
    String::from_utf8(bytes).map_err(|e| format!("UTF-8 error: {e}"))
}

fn write_heading(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    block: &serde_json::Value,
) -> Result<(), String> {
    let text = block["text"].as_str().unwrap_or("");
    let level = block["level"].as_u64().unwrap_or(1).clamp(1, 6);
    let style = format!("Heading{level}");

    writer
        .write_event(Event::Start(BytesStart::new("w:p")))
        .map_err(|e| format!("XML error: {e}"))?;

    writer
        .write_event(Event::Start(BytesStart::new("w:pPr")))
        .map_err(|e| format!("XML error: {e}"))?;
    let mut style_elem = BytesStart::new("w:pStyle");
    style_elem.push_attribute(("w:val", style.as_str()));
    writer
        .write_event(Event::Empty(style_elem))
        .map_err(|e| format!("XML error: {e}"))?;
    writer
        .write_event(Event::End(BytesEnd::new("w:pPr")))
        .map_err(|e| format!("XML error: {e}"))?;

    write_run(writer, text, false, false)?;

    writer
        .write_event(Event::End(BytesEnd::new("w:p")))
        .map_err(|e| format!("XML error: {e}"))?;
    Ok(())
}

fn write_paragraph(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    block: &serde_json::Value,
) -> Result<(), String> {
    let text = block["text"].as_str().unwrap_or("");
    let bold = block["bold"].as_bool().unwrap_or(false);
    let italic = block["italic"].as_bool().unwrap_or(false);

    writer
        .write_event(Event::Start(BytesStart::new("w:p")))
        .map_err(|e| format!("XML error: {e}"))?;
    write_run(writer, text, bold, italic)?;
    writer
        .write_event(Event::End(BytesEnd::new("w:p")))
        .map_err(|e| format!("XML error: {e}"))?;
    Ok(())
}

pub fn write_run(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    text: &str,
    bold: bool,
    italic: bool,
) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new("w:r")))
        .map_err(|e| format!("XML error: {e}"))?;

    if bold || italic {
        writer
            .write_event(Event::Start(BytesStart::new("w:rPr")))
            .map_err(|e| format!("XML error: {e}"))?;
        if bold {
            writer
                .write_event(Event::Empty(BytesStart::new("w:b")))
                .map_err(|e| format!("XML error: {e}"))?;
        }
        if italic {
            writer
                .write_event(Event::Empty(BytesStart::new("w:i")))
                .map_err(|e| format!("XML error: {e}"))?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("w:rPr")))
            .map_err(|e| format!("XML error: {e}"))?;
    }

    writer
        .write_event(Event::Start(BytesStart::new("w:t")))
        .map_err(|e| format!("XML error: {e}"))?;
    writer
        .write_event(Event::Text(BytesText::new(text)))
        .map_err(|e| format!("XML error: {e}"))?;
    writer
        .write_event(Event::End(BytesEnd::new("w:t")))
        .map_err(|e| format!("XML error: {e}"))?;

    writer
        .write_event(Event::End(BytesEnd::new("w:r")))
        .map_err(|e| format!("XML error: {e}"))?;
    Ok(())
}

fn write_table(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    block: &serde_json::Value,
) -> Result<(), String> {
    let headers = block["headers"]
        .as_array()
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    let rows = block["rows"]
        .as_array()
        .map(|a| a.as_slice())
        .unwrap_or(&[]);

    if headers.is_empty() && rows.is_empty() {
        return Ok(());
    }

    writer
        .write_event(Event::Start(BytesStart::new("w:tbl")))
        .map_err(|e| format!("XML error: {e}"))?;

    if !headers.is_empty() {
        write_table_row(writer, headers, true)?;
    }
    for row in rows {
        if let Some(cells) = row.as_array() {
            write_table_row(writer, cells, false)?;
        }
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:tbl")))
        .map_err(|e| format!("XML error: {e}"))?;
    Ok(())
}

fn write_table_row(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    cells: &[serde_json::Value],
    is_header: bool,
) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new("w:tr")))
        .map_err(|e| format!("XML error: {e}"))?;
    for cell in cells {
        let text = cell.as_str().unwrap_or("");
        writer
            .write_event(Event::Start(BytesStart::new("w:tc")))
            .map_err(|e| format!("XML error: {e}"))?;
        writer
            .write_event(Event::Start(BytesStart::new("w:p")))
            .map_err(|e| format!("XML error: {e}"))?;
        write_run(writer, text, is_header, false)?;
        writer
            .write_event(Event::End(BytesEnd::new("w:p")))
            .map_err(|e| format!("XML error: {e}"))?;
        writer
            .write_event(Event::End(BytesEnd::new("w:tc")))
            .map_err(|e| format!("XML error: {e}"))?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("w:tr")))
        .map_err(|e| format!("XML error: {e}"))?;
    Ok(())
}
