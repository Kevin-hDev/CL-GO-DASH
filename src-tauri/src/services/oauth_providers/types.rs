use super::ProviderId;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthClientState {
    Ready,
    Missing,
    #[allow(dead_code)] // Kept in the serialized UI contract for negotiated ACP failures.
    Incompatible,
}

#[derive(Debug, Clone, Serialize)]
pub struct OAuthProviderStatus {
    pub id: ProviderId,
    pub display_name: &'static str,
    pub connected: bool,
    pub account: Option<String>,
    pub client_state: OAuthClientState,
    pub install_url: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct OAuthLoginProgress {
    pub provider_id: ProviderId,
    pub stage: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_code: Option<String>,
}
