import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "@/hooks/use-settings";
import type { Theme } from "@/hooks/use-theme";
import { GeneralSettings } from "./general-settings";
import { OllamaTab } from "@/components/ollama/ollama-tab";
import { ApiKeysTab } from "@/components/api-keys/api-keys-tab";

type SettingsSubTab = "general" | "ollama" | "api-keys";

interface SettingsTabProps {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
}

export function SettingsTab({ theme, onThemeChange }: SettingsTabProps): {
  list: React.ReactNode;
  detail: React.ReactNode;
} {
  const [subTab, setSubTab] = useState<SettingsSubTab>("general");
  const settings = useSettings();
  const { t } = useTranslation();

  const ollamaTab = OllamaTab();
  const apiKeysTab = ApiKeysTab();

  const subTabs: { id: SettingsSubTab; label: string }[] = [
    { id: "general", label: t("settings.tabs.general") },
    { id: "ollama", label: t("settings.tabs.ollama") },
    { id: "api-keys", label: t("settings.tabs.apiKeys") },
  ];

  const list = (
    <div style={{ display: "flex", flexDirection: "column", flex: 1, minHeight: 0, overflow: "hidden" }}>
      <div style={{ padding: "var(--space-sm)", flexShrink: 0 }}>
        {subTabs.map((tab) => (
          <div
            key={tab.id}
            onClick={() => setSubTab(tab.id)}
            style={{
              padding: "5px var(--space-sm)",
              borderRadius: "var(--radius-sm)",
              cursor: "pointer",
              fontSize: "var(--text-sm)",
              color: subTab === tab.id ? "var(--ink)" : "var(--ink-muted)",
              background: subTab === tab.id ? "var(--surface-hover)" : "transparent",
              fontWeight: subTab === tab.id ? 500 : 400,
              marginBottom: 2,
              transition: "all 200ms ease",
            }}
          >
            {tab.label}
          </div>
        ))}
      </div>
      <div style={{ display: "flex", flexDirection: "column", flex: 1, minHeight: 0, overflow: "hidden" }}>
        {subTab === "ollama" && ollamaTab.list}
        {subTab === "api-keys" && apiKeysTab.list}
      </div>
    </div>
  );

  const detail = (() => {
    if (subTab === "general") {
      return (
        <GeneralSettings
          theme={theme}
          onThemeChange={onThemeChange}
          settings={settings}
        />
      );
    }
    if (subTab === "ollama") return ollamaTab.detail;
    return apiKeysTab.detail;
  })();

  return { list, detail };
}
