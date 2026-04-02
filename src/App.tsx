import { useState } from "react";
import { AppLayout } from "@/components/layout/app-layout";
import { useTheme } from "@/hooks/use-theme";
import { HeartbeatTab } from "@/components/heartbeat/heartbeat-tab";
import { HistoryTab } from "@/components/history/history-tab";
import { PersonalityTab } from "@/components/personality/personality-tab";
import type { TabId } from "@/components/layout/sidebar";

export default function App() {
  const [activeTab, setActiveTab] = useState<TabId>("heartbeat");
  const { theme, toggle: toggleTheme } = useTheme();

  const hbTab = HeartbeatTab();
  const histTab = HistoryTab();
  const persTab = PersonalityTab();

  const tabs: Record<TabId, { list: React.ReactNode; detail: React.ReactNode }> = {
    heartbeat: hbTab,
    history: histTab,
    personality: persTab,
  };

  const tab = tabs[activeTab];

  return (
    <AppLayout
      activeTab={activeTab}
      onTabChange={setActiveTab}
      theme={theme}
      onThemeToggle={toggleTheme}
      listContent={tab.list}
      detailContent={tab.detail}
    />
  );
}
