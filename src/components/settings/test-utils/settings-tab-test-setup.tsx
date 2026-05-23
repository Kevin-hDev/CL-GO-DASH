import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { vi } from "vitest";
import { SettingsTab } from "../settings-tab";
import { DEFAULT_APP_NAV, type DeepPartial, type SettingsNavState } from "@/types/navigation";
import { PanelSlotProvider, PanelSlotTarget } from "@/components/layout/panel-slots";

export const CHILD_COMMANDS = new Set([
  "list_ollama_models",
  "list_mcp_connectors",
  "gateway_get_config",
  "gateway_status",
  "list_llm_providers_catalog",
  "list_search_providers_catalog",
  "list_forecast_providers_catalog",
  "list_configured_providers",
  "list_forecast_models",
]);

const mocks = vi.hoisted(() => {
  function provider(id: string, displayName: string, category: string) {
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

  return {
    invoke: vi.fn((cmd: string) => {
      if (cmd === "get_advanced_settings") return Promise.resolve({});
      if (cmd === "is_ollama_installed") return Promise.resolve(true);
      if (cmd === "get_modelfile") return Promise.resolve("FROM llama3.2:latest\nPARAMETER temperature 0.7\n");
      if (cmd === "get_selected_forecast_model") return Promise.resolve("chronos-bolt-small");
      if (cmd === "list_configured_providers") return Promise.resolve(["groq", "brave", "nixtla"]);
      if (cmd === "list_llm_providers_catalog") {
        return Promise.resolve([provider("groq", "Groq", "llm"), provider("mistral", "Mistral", "llm")]);
      }
      if (cmd === "list_search_providers_catalog") return Promise.resolve([provider("brave", "Brave", "search")]);
      if (cmd === "list_forecast_providers_catalog") return Promise.resolve([provider("nixtla", "Nixtla", "forecast")]);
      if (cmd === "list_forecast_models") return Promise.resolve(forecastModels());
      if (cmd === "list_mcp_connectors") return Promise.resolve(mcpConnectors());
      if (cmd === "gateway_get_config") return Promise.resolve(gatewayConfig());
      if (cmd === "gateway_status") return Promise.resolve(gatewayStatus());
      if (cmd === "list_ollama_models") return Promise.resolve(ollamaModels());
      return Promise.resolve([]);
    }),
  };
});

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "en", changeLanguage: vi.fn() },
  }),
}));

vi.mock("@/i18n", () => ({
  default: { t: (key: string) => key },
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

const noop = vi.fn();

export function SettingsHarness() {
  const [navState, setNavState] = useState<SettingsNavState>(DEFAULT_APP_NAV.settings);
  const handleNavChange = useCallback((partial: DeepPartial<SettingsNavState>) => {
    setNavState((current) => ({ ...current, ...partial }) as SettingsNavState);
  }, []);

  return (
    <PanelSlotProvider>
      <SettingsTab
        themeChoice="dark"
        onThemeChange={noop}
        navState={navState}
        onNavChange={handleNavChange}
        onNavReplace={handleNavChange}
        listFocused={false}
      />
      <div data-testid="settings-list"><PanelSlotTarget name="list" /></div>
      <div data-testid="settings-detail"><PanelSlotTarget name="detail" /></div>
    </PanelSlotProvider>
  );
}

export function resetSettingsTestEnvironment() {
  vi.mocked(invoke).mockClear();
  const store = new Map<string, string>();
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => store.set(key, value),
      removeItem: (key: string) => store.delete(key),
      clear: () => store.clear(),
    },
  });
}

export function invokedCommands() {
  return vi.mocked(invoke).mock.calls.map(([cmd]) => cmd);
}

function ollamaModels() {
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

function mcpConnectors() {
  return [
    { id: "canva", status: "connected", enabled_in_chat: true },
    { id: "github", status: "disconnected", enabled_in_chat: false },
  ];
}

function gatewayStatus() {
  return {
    running: true,
    channels: [{ channel_id: "telegram", account_id: "test-telegram", ok: true }],
  };
}

function gatewayConfig() {
  return {
    enabled: false,
    start_with_app: true,
    run_when_window_closed: true,
    default_provider: "",
    default_model: "",
    max_sessions: 500,
    max_messages_per_session: 100,
    message_max_chars: 8000,
    rate_limits: { per_user_per_minute: 12, per_channel_per_minute: 120, global_per_minute: 300 },
    security: {
      default_dm_policy: "allowlist",
      allow_private_urls: false,
      tools_enabled_by_default: false,
      allow_wildcard_allowlist: false,
    },
    audit: { enabled: true, retention_days: 30, redact_content: true },
    channels: {
      telegram: [{ account_id: "test-telegram", enabled: true, allowlist: [], require_mention: true }],
      slack: [],
      discord: [{ account_id: "test-discord", enabled: true, allowlist: [], require_mention: true }],
    },
  };
}

function forecastModels() {
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
