import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import { AppLayout } from "@/components/layout/app-layout";
import { OllamaSetupScreen } from "@/components/ollama/ollama-setup-screen";
import { useTheme } from "@/hooks/use-theme";
import { useTabHistory } from "@/hooks/use-tab-history";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import { usePanelFocus } from "@/hooks/use-panel-focus";
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

  const { theme, choice, setTheme } = useTheme();
  const { t } = useTranslation();
  const [vaultError, setVaultError] = useState<string | null>(null);
  const [ollamaReady, setOllamaReady] = useState<boolean | null>(null);
  const { focusedPanel } = usePanelFocus();

  useEffect(() => {
    invoke<boolean>("is_ollama_installed").then(setOllamaReady).catch(() => setOllamaReady(true));
    const unlisten = listen<string>("vault-init-failed", (e) => {
      setVaultError(e.payload);
    });
    return () => { unlisten.then((fn) => fn()).catch(() => {}); };
  }, []);


  const activeTab = nav.tab as TabId;
  const listActive = (tab: TabId) => focusedPanel === "list" && activeTab === tab;

  const hbTab = HeartbeatTab({
    activeWakeupId: nav.wakeupId,
    onWakeupChange: (id) => push({ wakeupId: id }),
    listFocused: listActive("heartbeat"),
  });

  const persTab = PersonalityTab({
    activePath: nav.personalityPath,
    onPathChange: (path) => push({ personalityPath: path }),
    listFocused: listActive("personality"),
  });

  const agentTab = AgentLocalTab({
    requestedSessionId: nav.sessionId,
    onSessionChange: (id) => push({ sessionId: id }),
    listFocused: listActive("agent-local"),
  });

  const settTab = SettingsTab({
    theme,
    themeChoice: choice,
    onThemeChange: setTheme,
    activeSubTab: nav.settingsSubTab,
    onSubTabChange: (sub) => push({ settingsSubTab: sub }),
    listFocused: listActive("settings"),
  });

  const tabs: Record<TabId, { list: React.ReactNode; detail: React.ReactNode }> = {
    heartbeat: hbTab,
    personality: persTab,
    "agent-local": agentTab,
    settings: settTab,
  };

  const tab = tabs[activeTab];

  const ALL_TABS: TabId[] = ["agent-local", "heartbeat", "personality", "settings"];
  useArrowNavigation({
    items: ALL_TABS,
    selectedId: activeTab,
    onSelect: (t) => push({ tab: t }),
    enabled: focusedPanel === "sidebar",
  });

  const handleShowWelcome = useCallback(() => {
    push({ tab: "agent-local", sessionId: null });
    agentTab.onShowWelcome?.();
  }, [agentTab, push]);

  const handleSearchSelect = useCallback((sessionId: string) => {
    push({ tab: "agent-local", sessionId });
  }, [push]);

  useEffect(() => {
    if (ollamaReady !== null) {
      const splash = document.getElementById("splash");
      if (splash) {
        const timer = setTimeout(() => {
          requestAnimationFrame(() => splash.remove());
        }, 150);
        return () => clearTimeout(timer);
      }
    }
  }, [ollamaReady]);

  if (ollamaReady === false) {
    return (
      <div style={{
        width: "100vw", height: "100vh",
        background: "var(--void)",
        display: "flex", alignItems: "center", justifyContent: "center",
      }}>
        <OllamaSetupScreen onComplete={() => {
          invoke("start_ollama_sidecar").catch(() => {});
          setOllamaReady(true);
        }} />
      </div>
    );
  }

  return (
    <>
    {vaultError && (
      <div style={{
        position: "fixed", top: 0, left: 0, right: 0, zIndex: 9999,
        padding: "8px 16px", background: "var(--signal-error)", color: "white",
        fontSize: "var(--text-xs)", textAlign: "center", cursor: "pointer",
      }} role="button" tabIndex={0} onClick={() => setVaultError(null)} onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') setVaultError(null); }}>
        {t("errors.keyringFailed")}
      </div>
    )}
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
      onSearchSelect={handleSearchSelect}
      onNewSession={handleShowWelcome}
    />
    </>
  );
}
