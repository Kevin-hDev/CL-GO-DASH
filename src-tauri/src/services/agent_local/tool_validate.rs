use serde_json::Value;

#[derive(Clone, Copy)]
enum Ty {
    Str,
    Int,
    Float,
    Arr,
    Obj,
}

type Schema = &'static [(&'static str, Ty, bool)];

static BASH: Schema = &[("command", Ty::Str, true), ("timeout", Ty::Int, false)];
static READ_FILE: Schema = &[
    ("path", Ty::Str, true),
    ("offset", Ty::Int, false),
    ("limit", Ty::Int, false),
];
static WRITE_FILE: Schema = &[("path", Ty::Str, true), ("content", Ty::Str, true)];
static EDIT_FILE: Schema = &[
    ("path", Ty::Str, true),
    ("old_string", Ty::Str, true),
    ("new_string", Ty::Str, true),
];
static LIST_DIR: Schema = &[("path", Ty::Str, false)];
static GREP: Schema = &[
    ("pattern", Ty::Str, true),
    ("path", Ty::Str, false),
    ("glob", Ty::Str, false),
];
static GLOB: Schema = &[("pattern", Ty::Str, true), ("path", Ty::Str, false)];
static WEB_SEARCH: Schema = &[("query", Ty::Str, true)];
static WEB_FETCH: Schema = &[("url", Ty::Str, true)];
static TODO_WRITE: Schema = &[("todos", Ty::Arr, true)];
static LOAD_SKILL: Schema = &[("skill_name", Ty::Str, true)];
static CREATE_BRANCH: Schema = &[("branch_name", Ty::Str, true)];
static CHECKOUT_BRANCH: Schema = &[("branch_name", Ty::Str, true)];
static DELEGATE_TASK: Schema = &[
    ("prompt", Ty::Str, true),
    ("subagent_type", Ty::Str, true),
    ("name", Ty::Str, false),
];
static READ_SPREADSHEET: Schema = &[
    ("path", Ty::Str, true),
    ("sheet", Ty::Str, false),
    ("range", Ty::Str, false),
    ("max_rows", Ty::Int, false),
];
static READ_DOCUMENT: Schema = &[("path", Ty::Str, true), ("pages", Ty::Str, false)];
static READ_IMAGE: Schema = &[("path", Ty::Str, true)];
static WRITE_SPREADSHEET: Schema = &[("path", Ty::Str, true), ("operations", Ty::Arr, true)];
static WRITE_DOCUMENT: Schema = &[("path", Ty::Str, true), ("content", Ty::Arr, true)];
static PROCESS_IMAGE: Schema = &[
    ("input_path", Ty::Str, true),
    ("output_path", Ty::Str, true),
    ("operations", Ty::Arr, false),
];
static SEARCH_MCP: Schema = &[
    ("mode", Ty::Str, true),
    ("query", Ty::Str, false),
    ("tool_id", Ty::Str, false),
    ("arguments", Ty::Obj, false),
];
static FORECAST: Schema = &[
    ("data", Ty::Str, false),
    ("file_path", Ty::Str, false),
    ("target_column", Ty::Str, true),
    ("date_column", Ty::Str, true),
    ("series_column", Ty::Str, false),
    ("covariate_columns", Ty::Arr, false),
    ("horizon", Ty::Int, true),
    ("frequency", Ty::Str, true),
    ("confidence_level", Ty::Float, false),
];
static FORECAST_ANALYZE: Schema = &[
    ("analysis_id", Ty::Str, true),
    ("action", Ty::Str, true),
    ("params", Ty::Obj, true),
];
static FORECAST_READ: Schema = &[("analysis_id", Ty::Str, false)];
static FORECAST_MODELS: Schema = &[];

fn schema(tool: &str) -> Option<Schema> {
    Some(match tool {
        "bash" => BASH,
        "read_file" => READ_FILE,
        "write_file" => WRITE_FILE,
        "edit_file" => EDIT_FILE,
        "list_dir" => LIST_DIR,
        "grep" => GREP,
        "glob" => GLOB,
        "web_search" => WEB_SEARCH,
        "web_fetch" => WEB_FETCH,
        "todo_write" => TODO_WRITE,
        "load_skill" => LOAD_SKILL,
        "create_branch" => CREATE_BRANCH,
        "checkout_branch" => CHECKOUT_BRANCH,
        "delegate_task" => DELEGATE_TASK,
        "read_spreadsheet" => READ_SPREADSHEET,
        "read_document" => READ_DOCUMENT,
        "read_image" => READ_IMAGE,
        "write_spreadsheet" => WRITE_SPREADSHEET,
        "write_document" => WRITE_DOCUMENT,
        "process_image" => PROCESS_IMAGE,
        "search_mcp_tools" => SEARCH_MCP,
        "forecast" => FORECAST,
        "forecast_analyze" => FORECAST_ANALYZE,
        "forecast_read" => FORECAST_READ,
        "forecast_models" => FORECAST_MODELS,
        _ => return None,
    })
}

fn type_ok(val: &Value, ty: Ty) -> bool {
    match ty {
        Ty::Str => val.is_string(),
        Ty::Int => val.is_u64() || val.is_i64(),
        Ty::Float => val.is_f64() || val.is_u64() || val.is_i64(),
        Ty::Arr => val.is_array(),
        Ty::Obj => val.is_object(),
    }
}

fn ty_label(ty: Ty) -> &'static str {
    match ty {
        Ty::Str => "string",
        Ty::Int => "integer",
        Ty::Float => "number",
        Ty::Arr => "array",
        Ty::Obj => "object",
    }
}

pub fn validate(tool: &str, args: &Value) -> Result<Value, String> {
    let specs = match schema(tool) {
        Some(s) => s,
        None => return Ok(args.clone()),
    };

    let obj = match args.as_object() {
        Some(o) => o,
        None => return Err("les arguments doivent être un objet JSON".into()),
    };

    for &(name, ty, required) in specs {
        match obj.get(name) {
            None | Some(Value::Null) if required => {
                return Err(format!("paramètre '{name}' requis"));
            }
            Some(v) if !v.is_null() && !type_ok(v, ty) => {
                return Err(format!("'{name}' doit être de type {}", ty_label(ty)));
            }
            _ => {}
        }
    }

    let mut cleaned = serde_json::Map::with_capacity(specs.len());
    for (key, val) in obj {
        if specs.iter().any(|(n, _, _)| *n == key.as_str()) {
            cleaned.insert(key.clone(), val.clone());
        } else {
            eprintln!("[tool-validate] argument inconnu ignoré : {tool}.{key}");
        }
    }
    Ok(Value::Object(cleaned))
}

#[cfg(test)]
#[path = "tool_validate_tests.rs"]
mod tests;
