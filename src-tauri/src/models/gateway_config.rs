use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GatewayConfig {
    pub enabled: bool,
    pub start_with_app: bool,
    pub run_when_window_closed: bool,
    pub default_provider: String,
    pub default_model: String,
    pub max_sessions: u32,
    pub max_messages_per_session: u32,
    pub message_max_chars: u32,
    pub rate_limits: RateLimitConfig,
    pub security: GatewaySecurityConfig,
    pub audit: AuditConfig,
    pub channels: ChannelsConfig,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            start_with_app: true,
            run_when_window_closed: true,
            default_provider: String::new(),
            default_model: String::new(),
            max_sessions: 500,
            max_messages_per_session: 100,
            message_max_chars: 8000,
            rate_limits: RateLimitConfig::default(),
            security: GatewaySecurityConfig::default(),
            audit: AuditConfig::default(),
            channels: ChannelsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RateLimitConfig {
    pub per_user_per_minute: u32,
    pub per_channel_per_minute: u32,
    pub global_per_minute: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            per_user_per_minute: 12,
            per_channel_per_minute: 120,
            global_per_minute: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GatewaySecurityConfig {
    pub default_dm_policy: DmPolicy,
    pub allow_private_urls: bool,
    pub tools_enabled_by_default: bool,
    pub allow_wildcard_allowlist: bool,
}

impl Default for GatewaySecurityConfig {
    fn default() -> Self {
        Self {
            default_dm_policy: DmPolicy::Allowlist,
            allow_private_urls: false,
            tools_enabled_by_default: false,
            allow_wildcard_allowlist: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DmPolicy {
    Open,
    Allowlist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AuditConfig {
    pub enabled: bool,
    pub retention_days: u32,
    pub redact_content: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 30,
            redact_content: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ChannelsConfig {
    pub telegram: Vec<ChannelAccountConfig>,
    pub slack: Vec<ChannelAccountConfig>,
    pub discord: Vec<ChannelAccountConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ChannelAccountConfig {
    pub account_id: String,
    pub enabled: bool,
    pub allowlist: Vec<String>,
    pub require_mention: bool,
    pub provider: String,
    pub model: String,
}

impl Default for ChannelAccountConfig {
    fn default() -> Self {
        Self {
            account_id: "default".to_string(),
            enabled: false,
            allowlist: Vec::new(),
            require_mention: true,
            provider: String::new(),
            model: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_gateway_disabled() {
        let cfg = GatewayConfig::default();
        assert!(!cfg.enabled);
        assert!(cfg.start_with_app);
        assert!(cfg.run_when_window_closed);
    }

    #[test]
    fn default_security_is_restrictive() {
        let sec = GatewaySecurityConfig::default();
        assert!(matches!(sec.default_dm_policy, DmPolicy::Allowlist));
        assert!(!sec.allow_private_urls);
        assert!(!sec.tools_enabled_by_default);
        assert!(!sec.allow_wildcard_allowlist);
    }

    #[test]
    fn default_rate_limits_are_reasonable() {
        let rl = RateLimitConfig::default();
        assert_eq!(rl.per_user_per_minute, 12);
        assert_eq!(rl.per_channel_per_minute, 120);
        assert_eq!(rl.global_per_minute, 300);
    }

    #[test]
    fn channel_account_defaults_to_mention_required() {
        let acc = ChannelAccountConfig::default();
        assert!(acc.require_mention);
        assert!(!acc.enabled);
        assert_eq!(acc.account_id, "default");
    }

    #[test]
    fn deserialize_empty_json_uses_defaults() {
        let cfg: GatewayConfig = serde_json::from_str("{}").unwrap();
        assert!(!cfg.enabled);
        assert_eq!(cfg.max_sessions, 500);
        assert!(cfg.channels.telegram.is_empty());
    }
}
