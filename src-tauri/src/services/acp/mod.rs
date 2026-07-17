mod connection;
mod policy;
mod process;
mod protocol;
pub(crate) mod session_store;

pub use connection::AcpConnection;
pub use policy::{auth_method, native_tool_allowed};
pub use process::AcpProcess;
pub use protocol::{AcpUpdate, JsonLineReader};

pub fn provider_from_chat(value: &str) -> Option<crate::services::oauth_providers::ProviderId> {
    match value {
        "moonshot-oauth" => Some(crate::services::oauth_providers::ProviderId::Moonshot),
        "xai-oauth" => Some(crate::services::oauth_providers::ProviderId::Xai),
        _ => None,
    }
}

pub async fn probe(provider: crate::services::oauth_providers::ProviderId) -> bool {
    let Ok(working_dir) = std::env::current_dir().and_then(|path| path.canonicalize()) else {
        return false;
    };
    let Ok(mut process) = AcpProcess::spawn(provider, &working_dir).await else {
        return false;
    };
    let request = process.connection.request(
        "initialize",
        serde_json::json!({
            "protocolVersion": 1,
            "clientCapabilities": {},
            "clientInfo": {"name":"CL-GO","version":env!("CARGO_PKG_VERSION")}
        }),
    );
    matches!(
        tokio::time::timeout(std::time::Duration::from_secs(5), request).await,
        Ok(Ok(response)) if response["protocolVersion"].as_u64() == Some(1)
            && response["agentCapabilities"]["mcpCapabilities"]["http"].as_bool() == Some(true)
    )
}

#[cfg(test)]
mod tests;
