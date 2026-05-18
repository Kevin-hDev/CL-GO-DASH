import type { FilePreviewActiveTab } from "@/types/file-preview";
import type { ForecastSection, PanelMode } from "@/hooks/use-forecast-panel";

export type MainTabId = "heartbeat" | "personality" | "agent-local" | "settings";
export type SettingsSubTab =
  | "general" | "ollama" | "connectors" | "channels" | "api-keys"
  | "forecast" | "llm" | "advanced" | "shortcuts" | "about";

export type OllamaSettingsSubTab = "modelfile" | "models";
export type ForecastSettingsSubTab = "config" | "models";

export interface AgentLocalNavState {
  sessionId: string | null;
  previewOpen: boolean;
  previewActiveTab: FilePreviewActiveTab;
  previewFullscreen: boolean;
  panelMode: PanelMode;
  forecastSection: ForecastSection;
  forecastAnalysisId: string | null;
  fileTreeOpen: boolean;
  terminalOpen: boolean;
  terminalActiveTabId: string | null;
}

export interface SettingsNavState {
  subTab: SettingsSubTab;
  apiKeyProviderId: string | null;
  connectorId: string | null;
  channelKey: string | null;
  ollamaSubTab: OllamaSettingsSubTab;
  ollamaInstalledModel: string | null;
  ollamaFamily: string | null;
  ollamaVariant: string | null;
  forecastSubTab: ForecastSettingsSubTab;
  forecastConfigModelId: string | null;
  forecastFamilyId: string | null;
  forecastModelId: string | null;
  llmView: LlmNavState;
}

export type LlmNavState =
  | { kind: "idle"; showFamilies: boolean }
  | { kind: "search"; query: string }
  | { kind: "family"; family: string }
  | { kind: "detail"; modelKey: string; parent: Exclude<LlmNavState, { kind: "detail" }> };

export interface AppNavState {
  tab: MainTabId;
  agentLocal: AgentLocalNavState;
  heartbeat: { wakeupId: string | null };
  personality: { path: string | null };
  settings: SettingsNavState;
}

export type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends object ? DeepPartial<T[K]> : T[K];
};

export type AppNavPatch = DeepPartial<AppNavState>;

export const DEFAULT_APP_NAV: AppNavState = {
  tab: "agent-local",
  agentLocal: {
    sessionId: null,
    previewOpen: false,
    previewActiveTab: "summary",
    previewFullscreen: false,
    panelMode: "preview",
    forecastSection: "view",
    forecastAnalysisId: null,
    fileTreeOpen: false,
    terminalOpen: false,
    terminalActiveTabId: null,
  },
  heartbeat: { wakeupId: null },
  personality: { path: null },
  settings: {
    subTab: "general",
    apiKeyProviderId: null,
    connectorId: null,
    channelKey: null,
    ollamaSubTab: "modelfile",
    ollamaInstalledModel: null,
    ollamaFamily: null,
    ollamaVariant: null,
    forecastSubTab: "config",
    forecastConfigModelId: null,
    forecastFamilyId: null,
    forecastModelId: null,
    llmView: { kind: "idle", showFamilies: false },
  },
};
