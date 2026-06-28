use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::io::Cursor;

pub fn write_list(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    block: &serde_json::Value,
) -> Result<(), String> {
    let items = match block["items"].as_array() {
        Some(arr) => arr,
        None => return Ok(()),
    };
    let ordered = block["ordered"].as_bool().unwrap_or(false);

    for (idx, item) in items.iter().enumerate() {
        let text = item.as_str().unwrap_or("");
        let prefix = if ordered {
            format!("{}. {}", idx + 1, text)
        } else {
            format!("• {}", text)
        };

        writer
            .write_event(Event::Start(BytesStart::new("w:p")))
            .map_err(|e| format!("XML error: {e}"))?;
        super::tool_document_write_xml::write_run(writer, &prefix, false, false)?;
        writer
            .write_event(Event::End(BytesEnd::new("w:p")))
            .map_err(|e| format!("XML error: {e}"))?;
    }
    Ok(())
}
