export function provider(id: string, displayName: string, category: string) {
  return {
    id,
    display_name: displayName,
    category,
    signup_url: "",
    free_tier_label: "",
    short_description: displayName,
    short_description_en: displayName,
  };
}

export function agentSettings() {
  return {
    permission_mode: "auto",
    enabled_optional_tools: ["load_skill", "planmode", "exitplanmode"],
  };
}

export function agentToolCatalog() {
  return [
    { id: "bash", locked: true, defaultEnabled: true, group: "core" },
    { id: "search_mcp_tools", locked: true, defaultEnabled: true, group: "mcp" },
    { id: "load_skill", locked: false, defaultEnabled: true, group: "workflow" },
    { id: "forecast", locked: false, defaultEnabled: false, group: "forecast" },
  ];
}

export function agentToolGroups() {
  return [
    { id: "web", locked: true, defaultEnabled: true, toolIds: ["web_search", "web_fetch"] },
    { id: "skills", locked: false, defaultEnabled: true, toolIds: ["load_skill"] },
    { id: "plan_mode", locked: false, defaultEnabled: true, toolIds: ["planmode", "exitplanmode"] },
    {
      id: "forecast",
      locked: false,
      defaultEnabled: false,
      toolIds: [
        "forecast_data_audit",
        "forecast",
        "forecast_models",
        "forecast_analyze",
        "forecast_read",
      ],
    },
  ];
}

export function ollamaModels() {
  return [{
    name: "llama3.2:latest",
    size: 2000,
    family: "llama",
    parameter_size: "3B",
    quantization: "Q4_K_M",
    architecture: "llama",
    is_moe: false,
    context_length: 8192,
    capabilities: ["completion"],
    digest_short: "abc123",
    aliases: [],
    is_customized: false,
  }];
}

export function mcpConnectors() {
  return [
    { id: "canva", status: "connected", enabled_in_chat: true },
    { id: "github", status: "disconnected", enabled_in_chat: false },
  ];
}

export function gatewayStatus() {
  return {
    running: true,
    channels: [{ channel_id: "telegram", account_id: "test-telegram", ok: true }],
  };
}

export function gatewayConfig() {
  return {
    enabled: false,
    start_with_app: true,
    run_when_window_closed: true,
    default_provider: "",
    default_model: "",
    max_sessions: 500,
    message_max_chars: 8000,
    rate_limits: { per_user_per_minute: 12, per_channel_per_minute: 120, global_per_minute: 300 },
    audit: { enabled: true, retention_days: 30 },
    channels: {
      telegram: [{ account_id: "test-telegram", enabled: true, allowlist: [], require_mention: true }],
      slack: [],
      discord: [{ account_id: "test-discord", enabled: true, allowlist: [], require_mention: true }],
    },
  };
}

export function forecastModels() {
  return {
    providers: [{ id: "nixtla", display_name: "Nixtla", configured: true }],
    configured_provider_ids: ["nixtla"],
    models: [{
      id: "chronos-bolt-small",
      provider_id: "local",
      family_id: "chronos-bolt",
      display_name: "Chronos Bolt Small",
      params: "small",
      size_mb: 120,
      ram_mb: 512,
      vram_mb: null,
      cpu_supported: true,
      gpu_supported: false,
      multivariate: false,
      covariates: false,
      horizon_max: 64,
      frequencies: "D,H",
      is_cloud: false,
      installed: true,
      runnable: true,
    }],
  };
}
