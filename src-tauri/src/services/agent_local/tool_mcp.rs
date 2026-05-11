use serde_json::Value;

use crate::services::agent_local::types_tools::ToolResult;
use crate::services::mcp_bridge::registry;
use crate::services::mcp_bridge::transport::McpToolDef;

const MAX_TOOLS_PER_SERVICE: usize = 15;

pub async fn execute(args: &Value) -> ToolResult {
    let mode = args["mode"].as_str().unwrap_or("search");
    match mode {
        "search" => search(args).await,
        "call" => call(args).await,
        _ => ToolResult::err("mode invalide : utiliser 'search' ou 'call'".to_string()),
    }
}

async fn search(args: &Value) -> ToolResult {
    let raw_query = args["query"].as_str().unwrap_or("").to_lowercase();
    let keywords: Vec<&str> = raw_query
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .collect();
    let connectors = registry::get_enabled_connectors();

    if connectors.is_empty() {
        return ToolResult::ok("Aucun connecteur MCP activé.".to_string());
    }

    let mut sections: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for connector in &connectors {
        let tools = match registry::get_tools(connector).await {
            Ok(t) => t,
            Err(e) => {
                errors.push(format!("{}: {e}", connector.id));
                continue;
            }
        };

        let matched: Vec<String> = tools
            .iter()
            .filter(|t| matches_keywords(t, &keywords, &connector.id))
            .take(MAX_TOOLS_PER_SERVICE)
            .map(|t| {
                let tool_id = format!("{}.{}", connector.id, t.name);
                let desc = t.description.as_deref().unwrap_or("(pas de description)");
                format!("  - {tool_id} : {desc}")
            })
            .collect();

        if !matched.is_empty() {
            sections.push(format!(
                "**{}** ({} outils) :\n{}",
                connector.id,
                matched.len(),
                matched.join("\n")
            ));
        }
    }

    let mut output = String::new();

    if !sections.is_empty() {
        let total: usize = sections.iter().map(|s| s.matches("\n  - ").count()).sum();
        output.push_str(&format!(
            "{total} outils MCP trouvés :\n\n{}",
            sections.join("\n\n")
        ));
    }

    if !errors.is_empty() {
        if !output.is_empty() {
            output.push_str("\n\n");
        }
        output.push_str(&format!("Erreurs :\n{}", errors.join("\n")));
    }

    if output.is_empty() {
        return ToolResult::ok("Aucun outil MCP ne correspond à la recherche.".to_string());
    }

    ToolResult::ok(output)
}

fn matches_keywords(tool: &McpToolDef, keywords: &[&str], connector_id: &str) -> bool {
    if keywords.is_empty() {
        return true;
    }
    let name = tool.name.to_lowercase();
    let desc = tool.description.as_deref().unwrap_or("").to_lowercase();
    let cid = connector_id.to_lowercase();
    keywords
        .iter()
        .any(|kw| name.contains(kw) || desc.contains(kw) || cid.contains(kw))
}

async fn call(args: &Value) -> ToolResult {
    let tool_id = match args["tool_id"].as_str() {
        Some(id) if id.contains('.') => id,
        _ => return ToolResult::err("tool_id requis (format: connecteur.outil)".to_string()),
    };

    let (connector_id, tool_name) = match tool_id.split_once('.') {
        Some(pair) => pair,
        None => return ToolResult::err("format tool_id invalide".to_string()),
    };

    if !is_valid_id(connector_id) || !is_valid_tool_name(tool_name) {
        return ToolResult::err("identifiant invalide".to_string());
    }

    let connectors = registry::get_enabled_connectors();
    let connector = match connectors.iter().find(|c| c.id == connector_id) {
        Some(c) => c,
        None => return ToolResult::err(format!("connecteur '{connector_id}' non disponible")),
    };

    let arguments = args
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));

    let args_size = serde_json::to_string(&arguments)
        .map(|s| s.len())
        .unwrap_or(0);
    if args_size > 65_536 {
        return ToolResult::err("arguments MCP trop volumineux (max 64 Ko)".to_string());
    }

    match connector.transport.call_tool(tool_name, arguments).await {
        Ok(result) => ToolResult::ok(sanitize_mcp_output(&result)),
        Err(e) => ToolResult::err(sanitize_mcp_output(&e)),
    }
}

fn is_valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn is_valid_tool_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn is_bidi_override(c: char) -> bool {
    matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200F}' | '\u{200E}')
}

fn sanitize_mcp_output(s: &str) -> String {
    s.chars()
        .take(4096)
        .filter(|c| (!c.is_control() || *c == '\n' || *c == '\t') && !is_bidi_override(*c))
        .collect()
}
