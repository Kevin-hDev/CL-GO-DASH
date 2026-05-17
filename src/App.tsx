import { useCallback, useEffect, useState, type ReactNode } from "react";
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
import { ForecastDocsWindow } from "@/components/forecast-docs/forecast-docs-window";
import type { TabSlots } from "@/components/agent-local/agent-local-tab-types";
import type { TabId } from "@/components/layout/sidebar";

export default function App() {
  return window.location.hash === "#/forecast-docs" ? <ForecastDocsApp /> : <MainApp />;
}

function ForecastDocsApp() {
  useTheme();

  useEffect(() => {
    const splash = document.getElementById("splash");
    if (!splash) return;
    requestAnimationFrame(() => splash.remove());
  }, []);

  return <ForecastDocsWindow />;
}

function MainApp() {
  const { current: nav, push, goBack, goForward, canGoBack, canGoForward } =
    useTabHistory({
      tab: "agent-local",
      settingsSubTab: "general",
      sessionId: null,
      wakeupId: null,
      personalityPath: null,
    });

  const { choice, setTheme } = useTheme();
  const { t } = useTranslation();

  const [vaultError, setVaultError] = useState<string | null>(null);
  const [ollamaReady, setOllamaReady] = useState<boolean | null>(null);
  const { focusedPanel } = usePanelFocus();
  const [tabContent, setTabContent] = useState<{ list: ReactNode; detail: ReactNode }>({ list: null, detail: null });

  useEffect(() => {
    invoke<boolean>("is_ollama_installed").then(setOllamaReady).catch(() => setOllamaReady(true));
    const unlisten = listen<string>("vault-init-failed", (e) => {
      setVaultError(e.payload);
    });
    return () => { unlisten.then((fn) => fn()).catch(() => {}); };
  }, []);

  const activeTab = nav.tab as TabId;
  const listActive = (tab: TabId) => focusedPanel === "list" && activeTab === tab;

  const reportContent = useCallback((slots: TabSlots) => {
    setTabContent(slots);
  }, []);

  const handleWakeupChange = useCallback((id: string | null) => push({ wakeupId: id }), [push]);
  const handlePathChange = useCallback((path: string | null) => push({ personalityPath: path }), [push]);
  const handleSessionChange = useCallback((id: string | null) => push({ sessionId: id }), [push]);
  const handleSubTabChange = useCallback((sub: string) => push({ settingsSubTab: sub }), [push]);

  const ALL_TABS: TabId[] = ["agent-local", "heartbeat", "personality", "settings"];
  useArrowNavigation({
    items: ALL_TABS,
    selectedId: activeTab,
    onSelect: (t) => push({ tab: t }),
    enabled: focusedPanel === "sidebar",
  });

  const handleShowWelcome = useCallback(() => {
    push({ tab: "agent-local", sessionId: null });
  }, [push]);

  const handleSearchSelect = useCallback((sessionId: string) => {
    push({ tab: "agent-local", sessionId });
  }, [push]);

  useEffect(() => {
    const timer = setTimeout(() => {
      requestAnimationFrame(() => {
        document.getElementById("splash")?.remove();
      });
    }, 150);
    return () => clearTimeout(timer);
  }, []);

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
    {activeTab === "heartbeat" && (
      <HeartbeatTab
        activeWakeupId={nav.wakeupId}
        onWakeupChange={handleWakeupChange}
        listFocused={listActive("heartbeat")}
        reportContent={reportContent}
      />
    )}
    {activeTab === "personality" && (
      <PersonalityTab
        activePath={nav.personalityPath}
        onPathChange={handlePathChange}
        listFocused={listActive("personality")}
        reportContent={reportContent}
      />
    )}
    {activeTab === "agent-local" && (
      <AgentLocalTab
        requestedSessionId={nav.sessionId}
        onSessionChange={handleSessionChange}
        listFocused={listActive("agent-local")}
        reportContent={reportContent}
      />
    )}
    {activeTab === "settings" && (
      <SettingsTab
        themeChoice={choice}
        onThemeChange={setTheme}
        activeSubTab={nav.settingsSubTab}
        onSubTabChange={handleSubTabChange}
        listFocused={listActive("settings")}
        reportContent={reportContent}
      />
    )}
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
      listContent={tabContent.list}
      detailContent={tabContent.detail}
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
