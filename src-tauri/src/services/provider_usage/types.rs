use serde::{Deserialize, Serialize};

pub const CONNECTION_LIMIT: usize = 32;
pub const DAY_LIMIT: usize = 400;
pub const WINDOW_LIMIT: usize = 8;
pub const BALANCE_LIMIT: usize = 4;

const CONNECTIONS: &[&str] = &[
    "groq",
    "google",
    "mistral",
    "cerebras",
    "openrouter",
    "openai",
    "deepseek",
    "xai",
    "moonshot",
    "zai",
    "codex-oauth",
    "xai-oauth",
    "moonshot-oauth",
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageOrigin {
    #[default]
    ManualChat,
    ExternalChannel,
    Automation,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageWorkload {
    #[default]
    Primary,
    Subagent,
    Compression,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TokenTotals {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_input_tokens: u64,
    pub reasoning_output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UsageAggregate {
    pub tokens: TokenTotals,
    pub request_count: u64,
    pub usage_request_count: u64,
    pub cost_usd_micros: u64,
    pub priced_request_count: u64,
    pub exact_cost_request_count: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OriginBreakdown {
    pub manual_chat: UsageAggregate,
    pub external_channel: UsageAggregate,
    pub automation: UsageAggregate,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkloadBreakdown {
    pub primary: UsageAggregate,
    pub subagent: UsageAggregate,
    pub compression: UsageAggregate,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageBreakdown {
    pub totals: UsageAggregate,
    pub origins: OriginBreakdown,
    pub workloads: WorkloadBreakdown,
}

#[derive(Debug, Clone, Default)]
pub struct LocalSnapshot {
    pub today: UsageBreakdown,
    pub seven_days: UsageBreakdown,
    pub thirty_days: UsageBreakdown,
    pub all_time: UsageBreakdown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderWindow {
    pub label_code: String,
    pub used: Option<f64>,
    pub limit: Option<f64>,
    pub remaining: Option<f64>,
    pub used_percent: Option<f64>,
    pub resets_at: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderBalance {
    pub label_code: String,
    pub amount: String,
    pub currency: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemoteData {
    pub windows: Vec<ProviderWindow>,
    pub balances: Vec<ProviderBalance>,
    pub notice_code: Option<String>,
    pub fetched_at: i64,
    pub stale: bool,
}

pub fn validate_connection_id(value: &str) -> Result<(), String> {
    CONNECTIONS
        .contains(&value)
        .then_some(())
        .ok_or_else(|| "Fournisseur invalide".to_string())
}

pub async fn context_for_session(
    session_id: &str,
    requested_workload: UsageWorkload,
) -> (UsageOrigin, UsageWorkload) {
    let Ok(session) = crate::services::agent_local::session_store::get(session_id).await else {
        return (UsageOrigin::ManualChat, requested_workload);
    };
    let origin = if session.is_heartbeat {
        UsageOrigin::Automation
    } else if session.is_gateway {
        UsageOrigin::ExternalChannel
    } else {
        UsageOrigin::ManualChat
    };
    let workload = if requested_workload == UsageWorkload::Primary
        && (session.parent_session_id.is_some() || session.subagent_type.is_some())
    {
        UsageWorkload::Subagent
    } else {
        requested_workload
    };
    (origin, workload)
}
