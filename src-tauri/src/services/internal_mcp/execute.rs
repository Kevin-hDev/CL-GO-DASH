use crate::services::agent_local::permission_gate::{self, PermissionDecision};
use crate::services::agent_local::{permission_policy, sensitive_data, tool_dispatcher};
use serde_json::Value;

use super::server::ServerContext;

const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

pub async fn call(
    context: &ServerContext,
    name: &str,
    arguments: &Value,
) -> Result<(String, bool), String> {
    if !super::catalog::contains(context.provider, name) || !arguments.is_object() {
        return Err("Outil MCP non autorisé".to_string());
    }
    if context.cancel.is_cancelled() {
        return Err("Action annulée".to_string());
    }
    if !check_permission(context, name, arguments).await {
        return Ok(("Action refusée".to_string(), true));
    }
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(180),
        tool_dispatcher::dispatch(
            name,
            arguments,
            &context.working_dir,
            &context.session_id,
            context.cancel.clone(),
        ),
    )
    .await
    .map_err(|_| "Délai MCP dépassé".to_string())?;
    Ok((truncate(result.content), result.is_error))
}

async fn check_permission(context: &ServerContext, name: &str, arguments: &Value) -> bool {
    let sensitive_bash = name == "bash"
        && arguments["command"]
            .as_str()
            .is_some_and(sensitive_data::bash_touches_sensitive_data);
    if sensitive_bash {
        return request(context, name, arguments).await;
    }
    if permission_policy::uses_auto_bypass(&context.mode) {
        return permission_policy::check_data_dir_write(
            &context.emitter,
            name,
            arguments,
            &context.working_dir,
            context.cancel.clone(),
        )
        .await;
    }
    if !permission_gate::requires_permission(name, arguments)
        || permission_gate::is_allowed(&context.session_id, name).await
    {
        return true;
    }
    request(context, name, arguments).await
}

async fn request(context: &ServerContext, name: &str, arguments: &Value) -> bool {
    match permission_gate::request(&context.emitter, name, arguments, context.cancel.clone()).await
    {
        PermissionDecision::Allow => true,
        PermissionDecision::AllowSession => {
            permission_gate::mark_allowed(&context.session_id, name).await;
            true
        }
        PermissionDecision::Deny => false,
    }
}

pub(crate) fn truncate(mut value: String) -> String {
    if value.len() <= MAX_OUTPUT_BYTES {
        return value;
    }
    let end = value
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= MAX_OUTPUT_BYTES)
        .last()
        .unwrap_or(0);
    value.truncate(end);
    value
}
