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
             The first row is always used as column headers. Formulas are returned as text (e.g. '=SUM(A1:A5)'), not computed values. \
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
             New files get a default sheet named 'Sheet1'. Each operation can target a specific sheet via 'sheet' (default: first sheet). \
             Do not use add_sheet for single-sheet files — Sheet1 is created automatically. \
             Example: {\"path\": \"output.xlsx\", \"operations\": [\
             {\"type\": \"set_row\", \"row\": 0, \"values\": [\"Name\", \"Age\"]}, \
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
            "Create a Word document (.docx) from content blocks. Only include fields relevant to each block type. \
             heading: {type, text, level}. paragraph: {type, text, bold?, italic?}. \
             table: {type, headers, rows}. list: {type, items, ordered}. \
             Example: {\"path\": \"doc.docx\", \"content\": [\
             {\"type\": \"heading\", \"text\": \"Title\", \"level\": 1}, \
             {\"type\": \"paragraph\", \"text\": \"Hello world\"}, \
             {\"type\": \"table\", \"headers\": [\"A\",\"B\"], \"rows\": [[\"1\",\"2\"]]}, \
             {\"type\": \"list\", \"items\": [\"a\",\"b\"], \"ordered\": true}]}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Output path for .docx file"},
                    "content": {
                        "type": "array",
                        "description": "Content blocks. Each block uses only the fields for its type.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {"type": "string", "enum": ["heading", "paragraph", "table", "list"]},
                                "text": {"type": "string", "description": "For heading/paragraph only"},
                                "level": {"type": "integer", "description": "For heading only (1-6)"},
                                "bold": {"type": "boolean", "description": "For paragraph only"},
                                "italic": {"type": "boolean", "description": "For paragraph only"},
                                "headers": {"type": "array", "items": {"type": "string"}, "description": "For table only — column headers"},
                                "rows": {"type": "array", "description": "For table only — array of arrays"},
                                "items": {"type": "array", "items": {"type": "string"}, "description": "For list only"},
                                "ordered": {"type": "boolean", "description": "For list only (true=numbered, false=bullets)"}
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
            "Resize, crop, or convert an image. To convert format, just change the extension. \
             Example convert: {\"input_path\": \"photo.png\", \"output_path\": \"photo.webp\"} \
             Example resize: {\"input_path\": \"photo.png\", \"output_path\": \"thumb.jpg\", \"operations\": [\
             {\"type\": \"resize\", \"width\": 200, \"height\": 200, \"mode\": \"fit\"}, \
             {\"type\": \"quality\", \"value\": 85}]}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "input_path": {"type": "string", "description": "Source image path"},
                    "output_path": {"type": "string", "description": "Output path (extension determines format: jpg, png, webp, gif, bmp)"},
                    "operations": {
                        "type": "array",
                        "description": "Optional. Operations: resize ({type,width,height,mode:'fit'|'fill'|'exact'}), crop ({type,x,y,width,height}), quality ({type,value:1-100}). Omit for simple format conversion.",
                        "items": {"type": "object"}
                    }
                },
                "required": ["input_path", "output_path"]
            }),
        ),
    ]
}
