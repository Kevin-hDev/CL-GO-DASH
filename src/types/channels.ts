export type ChannelStatus = "off" | "starting" | "running" | "error" | "stopping";

export interface ChannelHealthEntry {
  channel_id: string;
  account_id: string;
  status: ChannelStatus;
  error?: string;
}

export interface GatewayHealth {
  running: boolean;
  channels: ChannelHealthEntry[];
}

export interface ChannelAccountConfig {
  account_id: string;
  enabled: boolean;
  allowlist: string[];
  require_mention: boolean;
  provider: string;
  model: string;
}

export interface GatewayConfig {
  enabled: boolean;
  start_with_app: boolean;
  run_when_window_closed: boolean;
  default_provider: string;
  default_model: string;
  max_sessions: number;
  max_messages_per_session: number;
  message_max_chars: number;
  rate_limits: RateLimitConfig;
  security: GatewaySecurityConfig;
  audit: AuditConfig;
  channels: ChannelsConfig;
}

export interface RateLimitConfig {
  per_user_per_minute: number;
  per_channel_per_minute: number;
  global_per_minute: number;
}

export interface GatewaySecurityConfig {
  default_dm_policy: "open" | "allowlist";
  allow_private_urls: boolean;
  tools_enabled_by_default: boolean;
  allow_wildcard_allowlist: boolean;
}

export interface AuditConfig {
  enabled: boolean;
  retention_days: number;
  redact_content: boolean;
}

export interface ChannelsConfig {
  telegram: ChannelAccountConfig[];
  slack: ChannelAccountConfig[];
  discord: ChannelAccountConfig[];
}

export type ChannelType = "telegram" | "slack" | "discord";
export type GatewayTokenKind = "default" | "bot" | "app";
