import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
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
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { TabSlots } from "@/components/agent-local/agent-local-tab-types";
import type { TabId } from "@/components/layout/sidebar";
import {
  DEFAULT_APP_NAV,
  type AgentLocalNavState,
  type DeepPartial,
  type SettingsNavState,
} from "@/types/navigation";

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
  const { current: nav, pushNav, replaceNav, goBack, goForward, canGoBack, canGoForward } =
    useTabHistory(DEFAULT_APP_NAV);

  const { choice, setTheme } = useTheme();
  const { t } = useTranslation();

  const [vaultError, setVaultError] = useState<string | null>(null);
  const [ollamaReady, setOllamaReady] = useState<boolean | null>(null);
  const { focusedPanel } = usePanelFocus();
  const [tabContent, setTabContent] = useState<{ list: ReactNode; detail: ReactNode }>({ list: null, detail: null });
  const tabContentRef = useRef(tabContent);

  useEffect(() => {
    invoke<boolean>("is_ollama_installed").then(setOllamaReady).catch(() => setOllamaReady(true));
    const unlisten = listen<string>("vault-init-failed", (e) => {
      setVaultError(e.payload);
    });
    return () => { cleanupTauriListener(unlisten); };
  }, []);

  const activeTab: TabId = nav.tab;
  const listActive = (tab: TabId) => focusedPanel === "list" && activeTab === tab;

  const reportContent = useCallback((slots: TabSlots) => {
    if (tabContentRef.current.list === slots.list && tabContentRef.current.detail === slots.detail) return;
    tabContentRef.current = slots;
    setTabContent(slots);
  }, []);

  const handleWakeupChange = useCallback((id: string | null) => pushNav({ heartbeat: { wakeupId: id } }), [pushNav]);
  const handlePathChange = useCallback((path: string | null) => pushNav({ personality: { path } }), [pushNav]);
  const handleSessionChange = useCallback((id: string | null) => pushNav({ agentLocal: { sessionId: id } }), [pushNav]);
  const handleAgentNavChange = useCallback((partial: DeepPartial<AgentLocalNavState>) => {
    pushNav({ agentLocal: partial });
  }, [pushNav]);
  const handleSettingsNavChange = useCallback((partial: DeepPartial<SettingsNavState>) => {
    pushNav({ settings: partial });
  }, [pushNav]);
  const handleSettingsNavReplace = useCallback((partial: DeepPartial<SettingsNavState>) => {
    replaceNav({ settings: partial });
  }, [replaceNav]);

  const ALL_TABS: TabId[] = ["agent-local", "heartbeat", "personality", "settings"];
  useArrowNavigation({
    items: ALL_TABS,
    selectedId: activeTab,
    onSelect: (t) => pushNav({ tab: t }),
    enabled: focusedPanel === "sidebar",
    focusActiveSelector: "[data-nav-zone='sidebar'] [data-nav-active='true']",
  });

  const handleShowWelcome = useCallback(() => {
    pushNav({ tab: "agent-local", agentLocal: { sessionId: null } });
  }, [pushNav]);

  const handleSearchSelect = useCallback((sessionId: string) => {
    pushNav({ tab: "agent-local", agentLocal: { sessionId } });
  }, [pushNav]);

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
        activeWakeupId={nav.heartbeat.wakeupId}
        onWakeupChange={handleWakeupChange}
        listFocused={listActive("heartbeat")}
        reportContent={reportContent}
      />
    )}
    {activeTab === "personality" && (
      <PersonalityTab
        activePath={nav.personality.path}
        onPathChange={handlePathChange}
        listFocused={listActive("personality")}
        reportContent={reportContent}
      />
    )}
    {activeTab === "agent-local" && (
      <AgentLocalTab
        navState={nav.agentLocal}
        onSessionChange={handleSessionChange}
        onNavChange={handleAgentNavChange}
        listFocused={listActive("agent-local")}
        reportContent={reportContent}
      />
    )}
    {activeTab === "settings" && (
      <SettingsTab
        themeChoice={choice}
        onThemeChange={setTheme}
        navState={nav.settings}
        onNavChange={handleSettingsNavChange}
        onNavReplace={handleSettingsNavReplace}
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
      onTabChange={(t) => pushNav({ tab: t })}
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
