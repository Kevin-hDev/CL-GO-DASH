use serde_json::Value;

fn tool_def(name: &str, description: &str, parameters: Value) -> Value {
    serde_json::json!({
        "type": "function",
        "function": { "name": name, "description": description, "parameters": parameters }
    })
}

pub fn office_tool_definitions() -> Vec<Value> {
    vec![
        tool_def(
            "read_spreadsheet",
            "Read data from a spreadsheet (Excel .xlsx/.xls/.ods or CSV/TSV). Returns JSON with headers and rows. \
             Example: {\"path\": \"data.xlsx\", \"sheet\": \"Sheet1\", \"max_rows\": 100}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to the file"},
                    "sheet": {"type": "string", "description": "Sheet name (default: first). Ignored for CSV."},
                    "range": {"type": "string", "description": "Cell range (e.g. 'A1:D10'). Ignored for CSV."},
                    "max_rows": {"type": "integer", "description": "Max data rows (default: 500, max: 5000)"}
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "read_document",
            "Extract text from PDF or Word (.docx). Returns JSON with text content. \
             Example: {\"path\": \"report.pdf\"}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to PDF or .docx file"},
                    "pages": {"type": "string", "description": "Page range for PDF (e.g. '1-5'). Optional."}
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "read_image",
            "Read image metadata (dimensions, format, size). Supports JPEG, PNG, WebP, GIF, BMP. \
             Example: {\"path\": \"photo.jpg\"}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to the image file"}
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "write_spreadsheet",
            "Create or modify an Excel file (.xlsx). Use operations array to set cells, formulas, rows. \
             Each operation can target a specific sheet via 'sheet' (default: first sheet). \
             Example: {\"path\": \"output.xlsx\", \"operations\": [\
             {\"type\": \"add_sheet\", \"name\": \"Data\"}, \
             {\"type\": \"set_row\", \"sheet\": \"Data\", \"row\": 0, \"values\": [\"Name\", \"Age\"]}, \
             {\"type\": \"set_cell\", \"cell\": \"A2\", \"value\": \"Alice\"}]}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to .xlsx file"},
                    "operations": {
                        "type": "array",
                        "description": "Array of operations. Each can have 'sheet' to target a specific sheet (default: first).",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {"type": "string", "enum": ["set_cell", "set_formula", "set_row", "add_sheet", "set_column_width"]},
                                "sheet": {"type": "string", "description": "Target sheet name (default: first sheet)"},
                                "cell": {"type": "string"},
                                "row": {"type": "integer"},
                                "col": {"type": "integer"},
                                "value": {},
                                "formula": {"type": "string"},
                                "values": {"type": "array"},
                                "name": {"type": "string"},
                                "width": {"type": "number"}
                            },
                            "required": ["type"]
                        }
                    }
                },
                "required": ["path", "operations"]
            }),
        ),
        tool_def(
            "write_document",
            "Create a Word document (.docx) from content blocks. \
             Example: {\"path\": \"doc.docx\", \"content\": [\
             {\"type\": \"heading\", \"text\": \"Title\", \"level\": 1}, \
             {\"type\": \"paragraph\", \"text\": \"Hello world\", \"bold\": true}, \
             {\"type\": \"table\", \"headers\": [\"A\",\"B\"], \"rows\": [[\"1\",\"2\"]]}, \
             {\"type\": \"list\", \"items\": [\"a\",\"b\"], \"ordered\": true}]}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Output path for .docx file"},
                    "content": {
                        "type": "array",
                        "description": "Content blocks",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {"type": "string", "enum": ["heading", "paragraph", "table", "list"]},
                                "text": {"type": "string", "description": "Text content (heading, paragraph)"},
                                "level": {"type": "integer", "description": "Heading level 1-6"},
                                "bold": {"type": "boolean"},
                                "italic": {"type": "boolean"},
                                "headers": {"type": "array", "items": {"type": "string"}, "description": "Table headers"},
                                "rows": {"type": "array", "description": "Table rows (array of arrays)"},
                                "items": {"type": "array", "items": {"type": "string"}, "description": "List items"},
                                "ordered": {"type": "boolean", "description": "Ordered list (true) or bullet list (false)"}
                            },
                            "required": ["type"]
                        }
                    }
                },
                "required": ["path", "content"]
            }),
        ),
        tool_def(
            "process_image",
            "Resize, crop, or convert an image. \
             Example: {\"input_path\": \"photo.png\", \"output_path\": \"thumb.jpg\", \"operations\": [\
             {\"type\": \"resize\", \"width\": 200, \"height\": 200, \"mode\": \"fit\"}, \
             {\"type\": \"quality\", \"value\": 85}]}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "input_path": {"type": "string", "description": "Source image path"},
                    "output_path": {"type": "string", "description": "Output path (extension = format)"},
                    "operations": {
                        "type": "array",
                        "description": "Operations: resize ({width,height,mode:'fit'|'fill'|'exact'}), crop ({x,y,width,height}), quality ({value:1-100})",
                        "items": {"type": "object"}
                    }
                },
                "required": ["input_path", "output_path", "operations"]
            }),
        ),
    ]
}
