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
             Example: {\"path\": \"output.xlsx\", \"operations\": [\
             {\"type\": \"set_row\", \"row\": 0, \"values\": [\"Name\", \"Age\", \"City\"]}, \
             {\"type\": \"set_cell\", \"cell\": \"A2\", \"value\": \"Alice\"}, \
             {\"type\": \"set_cell\", \"cell\": \"B2\", \"value\": 30}]}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to .xlsx file"},
                    "operations": {
                        "type": "array",
                        "description": "Array of operations. Types: set_cell ({cell:'A1',value:'x'}), set_formula ({cell:'B5',formula:'=SUM(B1:B4)'}), set_row ({row:0,values:['a','b']}), add_sheet ({name:'Sheet2'}), set_column_width ({col:0,width:20})",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {"type": "string", "enum": ["set_cell", "set_formula", "set_row", "add_sheet", "set_column_width"]},
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
                        "description": "Content blocks: heading ({text,level:1-6}), paragraph ({text,bold,italic}), table ({headers,rows}), list ({items,ordered})",
                        "items": {"type": "object"}
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
