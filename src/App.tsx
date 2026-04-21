import { useCallback } from "react";
import { AppLayout } from "@/components/layout/app-layout";
import { useTheme } from "@/hooks/use-theme";
import { useTabHistory } from "@/hooks/use-tab-history";
import { HeartbeatTab } from "@/components/heartbeat/heartbeat-tab";
import { PersonalityTab } from "@/components/personality/personality-tab";
import { AgentLocalTab } from "@/components/agent-local/agent-local-tab";
import { SettingsTab } from "@/components/settings/settings-tab";
import type { TabId } from "@/components/layout/sidebar";

export default function App() {
  const { current: nav, push, goBack, goForward, canGoBack, canGoForward } =
    useTabHistory({
      tab: "agent-local",
      settingsSubTab: "general",
      sessionId: null,
      wakeupId: null,
      personalityPath: null,
    });

  const { theme, setTheme } = useTheme();

  const hbTab = HeartbeatTab({
    activeWakeupId: nav.wakeupId,
    onWakeupChange: (id) => push({ wakeupId: id }),
  });

  const persTab = PersonalityTab({
    activePath: nav.personalityPath,
    onPathChange: (path) => push({ personalityPath: path }),
  });

  const agentTab = AgentLocalTab({
    requestedSessionId: nav.sessionId,
    onSessionChange: (id) => push({ sessionId: id }),
  });

  const settTab = SettingsTab({
    theme,
    onThemeChange: setTheme,
    activeSubTab: nav.settingsSubTab,
    onSubTabChange: (sub) => push({ settingsSubTab: sub }),
  });

  const tabs: Record<TabId, { list: React.ReactNode; detail: React.ReactNode }> = {
    heartbeat: hbTab,
    personality: persTab,
    "agent-local": agentTab,
    settings: settTab,
  };

  const activeTab = nav.tab as TabId;
  const tab = tabs[activeTab];

  const handleShowWelcome = useCallback(() => {
    push({ tab: "agent-local", sessionId: null });
    agentTab.onShowWelcome?.();
  }, [agentTab, push]);

  return (
    <AppLayout
      activeTab={activeTab}
      onTabChange={(t) => push({ tab: t })}
      listContent={tab.list}
      detailContent={tab.detail}
      onShowWelcome={handleShowWelcome}
      onBack={goBack}
      onForward={goForward}
      canGoBack={canGoBack}
      canGoForward={canGoForward}
    />
  );
}
