/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { StrictMode, useCallback, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsTab } from "../settings-tab";
import { DEFAULT_APP_NAV, type DeepPartial, type SettingsNavState, type SettingsSubTab } from "@/types/navigation";
import type { TabSlots } from "@/components/agent-local/agent-local-tab-types";

const noop = vi.fn();
const CHILD_COMMANDS = new Set([
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
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_advanced_settings") return Promise.resolve({});
    if (cmd === "get_modelfile") return Promise.resolve("FROM llama3.2:latest\nPARAMETER temperature 0.7\n");
    if (cmd === "get_selected_forecast_model") return Promise.resolve("chronos-bolt-small");
    if (cmd === "list_forecast_models") {
      return Promise.resolve({
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
      });
    }
    if (cmd === "list_configured_providers") return Promise.resolve(["groq", "brave", "nixtla"]);
    if (cmd === "list_llm_providers_catalog") {
      return Promise.resolve([
        provider("groq", "Groq", "llm"),
        provider("mistral", "Mistral", "llm"),
      ]);
    }
    if (cmd === "list_search_providers_catalog") return Promise.resolve([provider("brave", "Brave", "search")]);
    if (cmd === "list_forecast_providers_catalog") return Promise.resolve([provider("nixtla", "Nixtla", "forecast")]);
    if (cmd === "list_mcp_connectors") {
      return Promise.resolve([
        { id: "canva", status: "connected", enabled_in_chat: true },
        { id: "github", status: "disconnected", enabled_in_chat: false },
      ]);
    }
    if (cmd === "gateway_get_config") {
      return Promise.resolve({
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
          telegram: [{
            account_id: "test-telegram",
            enabled: true,
            allowlist: [],
            require_mention: true,
            provider: "mistral",
            model: "ministral-14b-latest",
          }],
          slack: [],
          discord: [{
            account_id: "test-discord",
            enabled: true,
            allowlist: [],
            require_mention: true,
            provider: "mistral",
            model: "ministral-14b-latest",
          }],
        },
      });
    }
    if (cmd === "gateway_status") {
      return Promise.resolve({
        running: true,
        channels: [{ channel_id: "telegram", account_id: "test-telegram", ok: true }],
      });
    }
    if (cmd === "list_ollama_models") {
      return Promise.resolve([{
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
      }]);
    }
    return Promise.resolve([]);
  }),
}));

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

function applySettingsPatch(state: SettingsNavState, patch: DeepPartial<SettingsNavState>): SettingsNavState {
  return { ...state, ...patch } as SettingsNavState;
}

function SettingsHarness({ reportContent }: { reportContent: (slots: TabSlots) => void }) {
  const [navState, setNavState] = useState<SettingsNavState>(DEFAULT_APP_NAV.settings);
  const [slots, setSlots] = useState<{ list: ReactNode; detail: ReactNode }>({ list: null, detail: null });
  const handleReport = useCallback((slots: TabSlots) => {
    setSlots(slots);
    reportContent(slots);
  }, [reportContent]);
  const handleNavChange = useCallback((partial: DeepPartial<SettingsNavState>) => {
    setNavState((current) => applySettingsPatch(current, partial));
  }, []);

  return (
    <>
      <SettingsTab
        themeChoice="dark"
        onThemeChange={noop}
        navState={navState}
        onNavChange={handleNavChange}
        onNavReplace={handleNavChange}
        listFocused={false}
        reportContent={handleReport}
      />
      <div data-testid="settings-list">{slots.list}</div>
      <div data-testid="settings-detail">{slots.detail}</div>
    </>
  );
}

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe("SettingsTab reportContent", () => {
  afterEach(() => cleanup());

  beforeEach(() => {
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
  });

  it("ne republie pas les slots en boucle sur les pages principales", async () => {
    const reportContent = vi.fn();

    render(<StrictMode><SettingsHarness reportContent={reportContent} /></StrictMode>);

    await waitFor(() => expect(reportContent).toHaveBeenCalled());
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(reportContent.mock.calls.length).toBeLessThan(5);
    const commands = vi.mocked(invoke).mock.calls.map(([cmd]) => cmd);
    expect(commands.some((cmd) => CHILD_COMMANDS.has(cmd))).toBe(false);
  });

  it.each([
    ["ollama", "settings.tabs.ollama", "llama3.2:latest"],
    ["connectors", "settings.tabs.connectors", "Canva"],
    ["channels", "settings.tabs.channels", "test-telegram"],
    ["api-keys", "settings.tabs.apiKeys", "Groq"],
    ["forecast", "forecast.title", "Chronos Bolt Small"],
  ] as Array<[SettingsSubTab, string, string]>)("ouvre %s sans crash ni boucle", async (_subTab, label, expectedContent) => {
    const reportContent = vi.fn();

    render(<StrictMode><SettingsHarness reportContent={reportContent} /></StrictMode>);
    const [item] = await screen.findAllByText(label);
    fireEvent.click(item);

    await waitFor(() => {
      const active = screen.getAllByText(label)
        .some((element) => element.getAttribute("data-nav-active") === "true");
      expect(active).toBe(true);
    });
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(reportContent.mock.calls.length).toBeLessThan(20);
    await waitFor(() => expect(screen.getAllByText(expectedContent).length).toBeGreaterThan(0));
  });
});
