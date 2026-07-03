import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { vi } from "vitest";
import { SettingsTab } from "../settings-tab";
import { PanelSlotProvider, PanelSlotTarget } from "@/components/layout/panel-slots";
import { DEFAULT_APP_NAV, type DeepPartial, type SettingsNavState } from "@/types/navigation";

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

vi.mock("@tauri-apps/api/core", async () => {
  const { vi: mockVi } = await import("vitest");
  const data = await import("./settings-tab-test-data");

  return {
    invoke: mockVi.fn((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "get_advanced_settings") return Promise.resolve({});
      if (cmd === "get_agent_settings") return Promise.resolve(data.agentSettings());
      if (cmd === "list_agent_tool_catalog") return Promise.resolve(data.agentToolCatalog());
      if (cmd === "set_agent_tool_enabled") {
        const enabled = args?.enabled === false ? [] : ["load_skill"];
        return Promise.resolve({ permission_mode: "auto", enabled_optional_tools: enabled });
      }
      if (cmd === "is_ollama_installed") return Promise.resolve(true);
      if (cmd === "get_modelfile") return Promise.resolve("FROM llama3.2:latest\nPARAMETER temperature 0.7\n");
      if (cmd === "get_selected_forecast_model") return Promise.resolve("chronos-bolt-small");
      if (cmd === "list_configured_providers") return Promise.resolve(["groq", "brave", "nixtla"]);
      if (cmd === "list_llm_providers_catalog") {
        return Promise.resolve([data.provider("groq", "Groq", "llm"), data.provider("mistral", "Mistral", "llm")]);
      }
      if (cmd === "list_search_providers_catalog") return Promise.resolve([data.provider("brave", "Brave", "search")]);
      if (cmd === "list_forecast_providers_catalog") return Promise.resolve([data.provider("nixtla", "Nixtla", "forecast")]);
      if (cmd === "list_forecast_models") return Promise.resolve(data.forecastModels());
      if (cmd === "list_mcp_connectors") return Promise.resolve(data.mcpConnectors());
      if (cmd === "gateway_get_config") return Promise.resolve(data.gatewayConfig());
      if (cmd === "gateway_status") return Promise.resolve(data.gatewayStatus());
      if (cmd === "list_ollama_models") return Promise.resolve(data.ollamaModels());
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

export function invokeCalls() {
  return vi.mocked(invoke).mock.calls;
}
