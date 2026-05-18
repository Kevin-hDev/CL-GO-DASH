"use no memo";
import { useEffect, useState, useMemo, useLayoutEffect, memo } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "@/hooks/use-settings";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import type { ThemeChoice } from "@/hooks/use-theme";
import { GearSix, Key, Sliders, Info, BookOpenText, Keyboard, Plugs, Broadcast, ChartLineUp } from "@/components/ui/icons";
import { ThemedIcon } from "@/components/ui/themed-icon";
import { GeneralSettings } from "./general-settings";
import { AdvancedSettings } from "./advanced-settings";
import { ShortcutsSettings } from "./shortcuts-settings";
import { AboutSettings } from "./about-settings";
import { LlmExplorer } from "./llm-explorer";
import { OllamaTab } from "@/components/ollama/ollama-tab";
import { ApiKeysTab } from "@/components/api-keys/api-keys-tab";
import { ConnectorsTab } from "@/components/connectors/connectors-tab";
import { ChannelsTab } from "@/components/channels/channels-tab";
import { ForecastTab } from "@/components/forecast/model-browser/forecast-tab";
import ollamaDark from "@/assets/ollama.png";
import ollamaLight from "@/assets/ollama-light.png";
import type { Icon } from "@phosphor-icons/react";
import type { TabSlots } from "@/components/agent-local/agent-local-tab-types";
import type { DeepPartial, SettingsNavState, SettingsSubTab } from "@/types/navigation";
import "./settings-tab.css";

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
  { id: "connectors", i18n: "settings.tabs.connectors", icon: Plugs },
  { id: "channels", i18n: "settings.tabs.channels", icon: Broadcast },
  { id: "api-keys", i18n: "settings.tabs.apiKeys", icon: Key },
  { id: "forecast", i18n: "forecast.title", icon: ChartLineUp },
  { id: "llm", i18n: "settings.tabs.llm", icon: BookOpenText },
  { id: "advanced", i18n: "settings.tabs.advanced", icon: Sliders },
  { id: "shortcuts", i18n: "settings.tabs.shortcuts", icon: Keyboard },
  { id: "about", i18n: "settings.tabs.about", icon: Info },
];

interface SettingsTabProps {
  themeChoice: ThemeChoice;
  onThemeChange: (theme: ThemeChoice) => void;
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
  listFocused?: boolean;
  reportContent: (slots: TabSlots) => void;
}

export const SettingsTab = memo(function SettingsTab({
  themeChoice,
  onThemeChange,
  navState,
  onNavChange,
  onNavReplace,
  listFocused = true,
  reportContent,
}: SettingsTabProps) {
  const [subTab, setSubTabState] = useState<SettingsSubTab>("general");

  useEffect(() => {
    setSubTabState(navState.subTab);
  }, [navState.subTab]);

  const setSubTab = (id: SettingsSubTab) => {
    setSubTabState(id);
    onNavChange({ subTab: id });
  };
  const subTabIds = useMemo(() => SUB_TABS.map((t) => t.id), []);
  useArrowNavigation({
    items: subTabIds,
    selectedId: subTab,
    onSelect: setSubTab,
    enabled: listFocused,
    focusActiveSelector: "[data-nav-zone='list'] [data-nav-active='true']",
  });

  const settings = useSettings();
  const { t } = useTranslation();

  const ollamaTab = OllamaTab({ navState, onNavChange, onNavReplace });
  const forecastTab = ForecastTab({ navState, onNavChange, onNavReplace });
  const connectorsTab = ConnectorsTab({ navState, onNavChange, onNavReplace });
  const channelsTab = ChannelsTab({ navState, onNavChange, onNavReplace });
  const apiKeysTab = ApiKeysTab({ navState, onNavChange, onNavReplace });

  const list = (
    <div style={{ display: "flex", flexDirection: "column", flex: 1, minHeight: 0, overflow: "hidden" }}>
      <div style={{ padding: "var(--space-sm)", flexShrink: 0 }}>
        {SUB_TABS.map((tab) => (
          <div
            key={tab.id}
            role="button"
            tabIndex={subTab === tab.id ? 0 : -1}
            data-nav-active={subTab === tab.id ? "true" : undefined}
            onClick={() => setSubTab(tab.id)}
            onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); setSubTab(tab.id); } }}
            className={`settings-subtab${subTab === tab.id ? " active" : ""}`}
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
        {subTab === "connectors" && connectorsTab.list}
        {subTab === "channels" && channelsTab.list}
        {subTab === "api-keys" && apiKeysTab.list}
        {subTab === "forecast" && forecastTab.list}
      </div>
    </div>
  );

  const detail = (() => {
    if (subTab === "general") {
      return (
        <GeneralSettings
          themeChoice={themeChoice}
          onThemeChange={onThemeChange}
          settings={settings}
        />
      );
    }
    if (subTab === "ollama") return ollamaTab.detail;
    if (subTab === "connectors") return connectorsTab.detail;
    if (subTab === "channels") return channelsTab.detail;
    if (subTab === "api-keys") return apiKeysTab.detail;
    if (subTab === "forecast") return forecastTab.detail;
    if (subTab === "llm") {
      return <LlmExplorer navState={navState.llmView} onNavChange={(llmView) => onNavChange({ llmView })} />;
    }
    if (subTab === "advanced") return <AdvancedSettings />;
    if (subTab === "shortcuts") return <ShortcutsSettings />;
    if (subTab === "about") return <AboutSettings />;
    return null;
  })();

  // eslint-disable-next-line react-hooks/exhaustive-deps -- reports the fresh slots from this render
  useLayoutEffect(() => { reportContent({ list, detail }); }, [
    reportContent, subTab, themeChoice, settings, navState, listFocused,
    ollamaTab, forecastTab, connectorsTab, channelsTab, apiKeysTab,
  ]);

  return null;
});
