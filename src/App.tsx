import { useState, useCallback } from "react";
import { AppLayout } from "@/components/layout/app-layout";
import { useTheme } from "@/hooks/use-theme";
import { HeartbeatTab } from "@/components/heartbeat/heartbeat-tab";
import { PersonalityTab } from "@/components/personality/personality-tab";
import { AgentLocalTab } from "@/components/agent-local/agent-local-tab";
import { OllamaTab } from "@/components/ollama/ollama-tab";
import { ApiKeysTab } from "@/components/api-keys/api-keys-tab";
import { SettingsTab } from "@/components/settings/settings-tab";
import type { TabId } from "@/components/layout/sidebar";

export default function App() {
  const [activeTab, setActiveTab] = useState<TabId>("agent-local");
  const { theme, setTheme } = useTheme();

  const hbTab = HeartbeatTab();
  const persTab = PersonalityTab();
  const agentTab = AgentLocalTab();
  const ollamaTab = OllamaTab();
  const apiKeysTab = ApiKeysTab();
  const settTab = SettingsTab({ theme, onThemeChange: setTheme });

  const tabs: Record<TabId, { list: React.ReactNode; detail: React.ReactNode }> = {
    heartbeat: hbTab,
    personality: persTab,
    "agent-local": agentTab,
    ollama: ollamaTab,
    "api-keys": apiKeysTab,
    settings: settTab,
  };

  const tab = tabs[activeTab];

  const handleShowWelcome = useCallback(() => {
    setActiveTab("agent-local");
    agentTab.onShowWelcome?.();
  }, [agentTab]);

  return (
    <AppLayout
      activeTab={activeTab}
      onTabChange={setActiveTab}
      listContent={tab.list}
      detailContent={tab.detail}
      onShowWelcome={handleShowWelcome}
    />
  );
}
