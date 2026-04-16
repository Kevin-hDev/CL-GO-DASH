import { useState } from "react";
import { AppLayout } from "@/components/layout/app-layout";
import { useTheme } from "@/hooks/use-theme";
import { HeartbeatTab } from "@/components/heartbeat/heartbeat-tab";
import { PersonalityTab } from "@/components/personality/personality-tab";
import { AgentLocalTab } from "@/components/agent-local/agent-local-tab";
import { OllamaTab } from "@/components/ollama/ollama-tab";
import { SettingsTab } from "@/components/settings/settings-tab";
import type { TabId } from "@/components/layout/sidebar";

export default function App() {
  const [activeTab, setActiveTab] = useState<TabId>("agent-local");
  const { theme, setTheme } = useTheme();

  const hbTab = HeartbeatTab();
  const persTab = PersonalityTab();
  const agentTab = AgentLocalTab();
  const ollamaTab = OllamaTab();
  const settTab = SettingsTab({ theme, onThemeChange: setTheme });

  const tabs: Record<TabId, { list: React.ReactNode; detail: React.ReactNode }> = {
    heartbeat: hbTab,
    personality: persTab,
    "agent-local": agentTab,
    ollama: ollamaTab,
    settings: settTab,
  };

  const tab = tabs[activeTab];

  return (
    <AppLayout
      activeTab={activeTab}
      onTabChange={setActiveTab}
      listContent={tab.list}
      detailContent={tab.detail}
      hideDetailDrag={activeTab === "agent-local"}
    />
  );
}
