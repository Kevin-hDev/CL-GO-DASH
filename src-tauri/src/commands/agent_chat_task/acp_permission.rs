use super::acp_events::AcpTurnState;
use crate::services::acp::AcpConnection;
use crate::services::agent_local::permission_gate::{self, PermissionDecision};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::oauth_providers::ProviderId;
use serde_json::{json, Value};
use tokio::process::{ChildStdin, ChildStdout};
use tokio_util::sync::CancellationToken;

pub async fn respond(
    connection: &mut AcpConnection<ChildStdout, ChildStdin>,
    message: &Value,
    provider: ProviderId,
    mode: &str,
    state: &AcpTurnState,
    on_event: &AgentEventEmitter,
    cancel: CancellationToken,
) -> Result<(), String> {
    let id = message
        .get("id")
        .cloned()
        .ok_or_else(|| "Permission ACP invalide".to_string())?;
    let params = &message["params"];
    let tool_id = params["toolCallId"].as_str().unwrap_or("");
    let tool_name = state
        .tool_name(tool_id)
        .or_else(|| params["toolCall"]["title"].as_str())
        .unwrap_or("tool");
    let arguments = params["toolCall"]
        .get("rawInput")
        .cloned()
        .unwrap_or(Value::Null);
    let decision = if provider == ProviderId::Xai && tool_name.eq_ignore_ascii_case("bash") {
        PermissionDecision::Deny
    } else if mode == "auto" {
        PermissionDecision::AllowSession
    } else {
        permission_gate::request(on_event, tool_name, &arguments, cancel).await
    };
    let option = choose_option(&params["options"], decision);
    let result = match option {
        Some(option_id) => json!({"outcome":{"outcome":"selected","optionId":option_id}}),
        None => json!({"outcome":{"outcome":"cancelled"}}),
    };
    connection.respond(&id, result).await
}

fn choose_option(options: &Value, decision: PermissionDecision) -> Option<String> {
    let wanted = match decision {
        PermissionDecision::Allow => ["allow_once", "allow_always"],
        PermissionDecision::AllowSession => ["allow_always", "allow_once"],
        PermissionDecision::Deny => ["reject_once", "reject_always"],
    };
    let options = options.as_array()?;
    wanted.into_iter().find_map(|kind| {
        options
            .iter()
            .find(|option| option["kind"].as_str() == Some(kind))
            .and_then(|option| option["optionId"].as_str())
            .map(|id| id.chars().take(128).collect())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_deny_instead_of_allow_for_a_rejected_tool() {
        let options = json!([
            {"optionId":"yes","kind":"allow_once"},
            {"optionId":"no","kind":"reject_once"}
        ]);
        assert_eq!(
            choose_option(&options, PermissionDecision::Deny).as_deref(),
            Some("no")
        );
    }
}
