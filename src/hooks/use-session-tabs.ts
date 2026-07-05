import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { markSessionUnread } from "@/hooks/use-session-activity-indicators";
import type { CloneMode, CloneSessionResult, SessionTabs } from "@/types/agent";

interface CloneMessageOptions {
  messageId: string;
  mode: CloneMode;
  customFocus?: string;
}

export function useSessionTabs(
  rootSessionId: string | null | undefined,
  onSessionsRefresh?: () => Promise<void> | void,
) {
  const [tabs, setTabs] = useState<SessionTabs | null>(null);
  const [loading, setLoading] = useState(false);

  const refreshTabs = useCallback(async () => {
    if (!rootSessionId) {
      setTabs(null);
      return;
    }
    setLoading(true);
    try {
      const next = await invoke<SessionTabs>("list_session_tabs", { sessionId: rootSessionId });
      setTabs(next);
    } finally {
      setLoading(false);
    }
  }, [rootSessionId]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- chargement externe lié au changement de session
    void refreshTabs();
  }, [refreshTabs]);

  const activeTab = useMemo(() => {
    if (!tabs) return null;
    return tabs.tabs.find((tab) => tab.tab_id === tabs.active_tab_id) ?? tabs.tabs[0] ?? null;
  }, [tabs]);

  const activeSessionId = activeTab?.session_id ?? rootSessionId ?? null;

  const selectTab = useCallback(async (tabId: string) => {
    if (!rootSessionId || !tabs || tabId === tabs.active_tab_id) return;
    const next = { ...tabs, active_tab_id: tabId };
    setTabs(next);
    const saved = await invoke<SessionTabs>("save_session_tabs", {
      sessionId: rootSessionId,
      tabs: next,
    });
    setTabs(saved);
  }, [rootSessionId, tabs]);

  const cloneMessage = useCallback(async (options: CloneMessageOptions) => {
    if (!rootSessionId) throw new Error("missing_session");
    const operationId = crypto.randomUUID();
    const result = await invoke<CloneSessionResult>("clone_agent_session", {
      sessionId: rootSessionId,
      messageId: options.messageId,
      mode: options.mode,
      customFocus: options.customFocus?.trim() || null,
      operationId,
    });
    await onSessionsRefresh?.();
    setTabs(result.tabs);
    if (options.mode === "summary") {
      markSessionUnread(rootSessionId);
    }
    return result;
  }, [onSessionsRefresh, rootSessionId]);

  const cancelCloneSummary = useCallback(async (operationId: string) => {
    await invoke("cancel_clone_summary", { operationId });
  }, []);

  const closeTab = useCallback(async (tabId: string) => {
    if (!rootSessionId) return;
    const next = await invoke<SessionTabs>("close_session_tab", {
      sessionId: rootSessionId,
      tabId,
    });
    await onSessionsRefresh?.();
    setTabs(next);
  }, [onSessionsRefresh, rootSessionId]);

  const renameTab = useCallback(async (tabId: string, label: string) => {
    if (!rootSessionId) return;
    const next = await invoke<SessionTabs>("rename_session_tab", {
      sessionId: rootSessionId,
      tabId,
      label,
    });
    setTabs(next);
  }, [rootSessionId]);

  return {
    tabs,
    loading,
    activeTab,
    activeSessionId,
    refreshTabs,
    selectTab,
    cloneMessage,
    cancelCloneSummary,
    closeTab,
    renameTab,
  };
}
