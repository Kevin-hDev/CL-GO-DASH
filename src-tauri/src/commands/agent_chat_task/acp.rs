use super::acp_events::AcpTurnState;
use super::common::{self, StreamMode};
use super::params::StreamTaskParams;
use crate::services::acp::{auth_method, AcpProcess, AcpUpdate};
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn run(params: StreamTaskParams, mode: StreamMode) -> Result<Vec<ChatMessage>, String> {
    let provider = crate::services::acp::provider_from_chat(&params.provider)
        .ok_or_else(|| "Provider ACP invalide".to_string())?;
    if !crate::services::oauth_providers::is_connected(provider) {
        crate::services::oauth_providers::invalidate_external_login(provider);
        return Err("Connexion OAuth requise".to_string());
    }
    let working_dir = common::resolve_working_dir(&params.working_dir)?;
    common::update_working_dir(&params.session_id, &working_dir).await;
    let prompt = params
        .messages
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .map(|message| message.content.clone())
        .ok_or_else(|| "Message ACP invalide".to_string())?;
    let mut process = AcpProcess::spawn(provider, &working_dir).await?;
    let connection = &mut process.connection;
    let init = timed(connection.request(
        "initialize",
        json!({
            "protocolVersion": 1,
            "clientCapabilities": {},
            "clientInfo": {"name":"CL-GO","version":env!("CARGO_PKG_VERSION")}
        }),
    ))
    .await?;
    if init["agentCapabilities"]["mcpCapabilities"]["http"].as_bool() != Some(true) {
        return Err("Client ACP incompatible".to_string());
    }
    let auth_methods = init["authMethods"].as_array().cloned().unwrap_or_default();
    if !auth_methods.is_empty() {
        let method = auth_method(provider, &Value::Array(auth_methods))
            .ok_or_else(|| "Connexion OAuth requise".to_string())?
            .to_string();
        if timed(connection.request(
            "authenticate",
            json!({"methodId":method,"_meta":{"headless":true}}),
        ))
        .await
        .is_err()
        {
            crate::services::oauth_providers::invalidate_external_login(provider);
            return Err("Connexion OAuth requise".to_string());
        }
    }
    let cwd = working_dir
        .to_str()
        .ok_or_else(|| "Répertoire ACP invalide".to_string())?;
    let mcp_server = crate::services::internal_mcp::InternalMcpServer::start(
        provider,
        &working_dir,
        &params.session_id,
        &mode.mode,
        params.on_event.clone(),
        params.cancel.clone(),
    )
    .await?;
    let acp_session_id =
        open_session(connection, &params.session_id, provider, cwd, &mcp_server).await?;
    let request_id = connection
        .start_request(
            "session/prompt",
            json!({
                "sessionId": acp_session_id,
                "prompt": [{"type":"text","text":prompt}]
            }),
        )
        .await?;
    let started = Instant::now();
    let mut state = AcpTurnState::new(&params.session_id);
    loop {
        let message = tokio::select! {
            result = connection.next_message() => result?,
            _ = params.cancel.cancelled() => {
                let _ = connection.notify("session/cancel", json!({"sessionId":acp_session_id})).await;
                return Err("Annulé".to_string());
            }
        };
        if message["id"].as_str() == Some(request_id.as_str()) && message.get("method").is_none() {
            if message.get("error").is_some() {
                return Err("Exécution ACP impossible".to_string());
            }
            break;
        }
        match message["method"].as_str() {
            Some("session/update") => {
                state.handle(
                    provider,
                    AcpUpdate::from_message(&message),
                    &params.on_event,
                )?;
            }
            Some("session/request_permission") if message.get("id").is_some() => {
                super::acp_permission::respond(
                    connection,
                    &message,
                    provider,
                    &mode.mode,
                    &state,
                    &params.on_event,
                    params.cancel.clone(),
                )
                .await?;
            }
            Some(_) if message.get("id").is_some() => {
                connection.respond_error(&message["id"]).await?;
            }
            _ => {}
        }
    }
    let elapsed = started.elapsed().as_nanos().min(u64::MAX as u128) as u64;
    let _ = params.on_event.send(StreamEvent::Done {
        eval_count: Some(state.token_count()),
        eval_duration_ns: elapsed,
        final_tps: 0.0,
        prompt_tokens: None,
        context_tokens: None,
    });
    Ok(params.messages)
}

async fn open_session<R, W>(
    connection: &mut crate::services::acp::AcpConnection<R, W>,
    cl_go_session_id: &str,
    provider: crate::services::oauth_providers::ProviderId,
    cwd: &str,
    mcp_server: &crate::services::internal_mcp::InternalMcpServer,
) -> Result<String, String>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    if let Some(id) = crate::services::acp::session_store::load(cl_go_session_id, provider).await? {
        if timed(connection.request(
            "session/resume",
            json!({
                "sessionId": id,
                "cwd": cwd,
                "mcpServers": [mcp_server.configuration()],
            }),
        ))
        .await
        .is_ok()
        {
            return Ok(id);
        }
    }
    let response = timed(connection.request(
        "session/new",
        json!({
            "cwd": cwd,
            "mcpServers": [mcp_server.configuration()],
        }),
    ))
    .await?;
    let id = response["sessionId"]
        .as_str()
        .ok_or_else(|| "Session ACP invalide".to_string())?
        .to_string();
    crate::services::acp::session_store::save(cl_go_session_id, provider, &id).await?;
    Ok(id)
}

async fn timed(
    future: impl std::future::Future<Output = Result<Value, String>>,
) -> Result<Value, String> {
    tokio::time::timeout(HANDSHAKE_TIMEOUT, future)
        .await
        .map_err(|_| "Délai ACP dépassé".to_string())?
}
