import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "@/hooks/use-settings";
import type { Theme } from "@/hooks/use-theme";
import { GearSix, Key, Sliders, Info, ChatCircleDots } from "@/components/ui/icons";
import { ThemedIcon } from "@/components/ui/themed-icon";
import { GeneralSettings } from "./general-settings";
import { AdvancedSettings } from "./advanced-settings";
import { LlmExplorer } from "./llm-explorer";
import { OllamaTab } from "@/components/ollama/ollama-tab";
import { ApiKeysTab } from "@/components/api-keys/api-keys-tab";
import ollamaDark from "@/assets/ollama.png";
import ollamaLight from "@/assets/ollama-light.png";
import type { Icon } from "@phosphor-icons/react";

type SettingsSubTab = "general" | "ollama" | "api-keys" | "llm" | "advanced" | "about";

interface SubTabDef {
  id: SettingsSubTab;
  i18n: string;
  icon?: Icon;
  imgDark?: string;
  imgLight?: string;
}

const SUB_TABS: SubTabDef[] = [
  { id: "general", i18n: "settings.tabs.general", icon: GearSix },
  { id: "ollama", i18n: "settings.tabs.ollama", imgDark: ollamaDark, imgLight: ollamaLight },
  { id: "api-keys", i18n: "settings.tabs.apiKeys", icon: Key },
  { id: "llm", i18n: "settings.tabs.llm", icon: ChatCircleDots },
  { id: "advanced", i18n: "settings.tabs.advanced", icon: Sliders },
  { id: "about", i18n: "settings.tabs.about", icon: Info },
];

interface SettingsTabProps {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
  activeSubTab?: string;
  onSubTabChange?: (subTab: string) => void;
}

export function SettingsTab({ theme, onThemeChange, activeSubTab, onSubTabChange }: SettingsTabProps): {
  list: React.ReactNode;
  detail: React.ReactNode;
} {
  const [subTab, setSubTabState] = useState<SettingsSubTab>("general");

  useEffect(() => {
    if (activeSubTab && SUB_TABS.some((t) => t.id.startsWith(activeSubTab))) {
      setSubTabState(activeSubTab as SettingsSubTab);
    }
  }, [activeSubTab]);

  const setSubTab = (id: SettingsSubTab) => {
    setSubTabState(id);
    onSubTabChange?.(id);
  };
  const settings = useSettings();
  const { t } = useTranslation();

  const ollamaTab = OllamaTab();
  const apiKeysTab = ApiKeysTab();

  const list = (
    <div style={{ display: "flex", flexDirection: "column", flex: 1, minHeight: 0, overflow: "hidden" }}>
      <div style={{ padding: "var(--space-sm)", flexShrink: 0 }}>
        {SUB_TABS.map((tab) => (
          <div
            key={tab.id}
            onClick={() => setSubTab(tab.id)}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 8,
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
            {tab.icon ? (
              <tab.icon
                size={16}
                weight={subTab === tab.id ? "fill" : "regular"}
                style={{ flexShrink: 0 }}
              />
            ) : tab.imgDark && tab.imgLight ? (
              <ThemedIcon
                darkSrc={tab.imgDark}
                lightSrc={tab.imgLight}
                size="16px"
                style={{
                  flexShrink: 0,
                  opacity: subTab === tab.id ? 1 : 0.6,
                }}
              />
            ) : null}
            {t(tab.i18n)}
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
    if (subTab === "api-keys") return apiKeysTab.detail;
    if (subTab === "llm") return <LlmExplorer />;
    if (subTab === "advanced") return <AdvancedSettings />;
    return null;
  })();

  return { list, detail };
}
