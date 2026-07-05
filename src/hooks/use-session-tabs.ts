import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  clearSessionRunning,
  markSessionComplete,
  markSessionRunning,
} from "@/hooks/use-session-activity-indicators";
import type { CloneMode, CloneSessionResult, SessionTabs } from "@/types/agent";

interface CloneMessageOptions {
  messageId: string;
  mode: CloneMode;
  customFocus?: string;
  operationId?: string;
  shouldActivateOnComplete?: () => boolean;
}

export function useSessionTabs(
  rootSessionId: string | null | undefined,
  onSessionsRefresh?: () => Promise<void> | void,
) {
  const [tabs, setTabs] = useState<SessionTabs | null>(null);
  const [attentionTabs, setAttentionTabs] = useState<Record<string, string[]>>({});
  const [loading, setLoading] = useState(false);
  const rootSessionIdRef = useRef(rootSessionId);

  useEffect(() => {
    rootSessionIdRef.current = rootSessionId;
  }, [rootSessionId]);

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
  const attentionTabIds = useMemo(
    () => new Set(rootSessionId ? attentionTabs[rootSessionId] ?? [] : []),
    [attentionTabs, rootSessionId],
  );

  const selectTab = useCallback(async (tabId: string) => {
    if (!rootSessionId || !tabs || tabId === tabs.active_tab_id) return;
    setAttentionTabs((current) => removeAttentionTab(current, rootSessionId, tabId));
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
    const previousActiveTabId = tabs?.active_tab_id ?? "main";
    const operationId = options.operationId ?? crypto.randomUUID();
    if (options.mode === "summary") {
      markSessionRunning(rootSessionId);
    }
    try {
      const result = await invoke<CloneSessionResult>("clone_agent_session", {
        sessionId: rootSessionId,
        messageId: options.messageId,
        mode: options.mode,
        customFocus: options.customFocus?.trim() || null,
        operationId,
      });
      await onSessionsRefresh?.();
      const cloneTabId = findCloneTabId(result);
      const shouldActivate = options.shouldActivateOnComplete?.() ?? true;
      const canActivate = shouldActivate && rootSessionIdRef.current === rootSessionId;
      const nextTabs = canActivate
        ? result.tabs
        : await savePreviousActiveTab(rootSessionId, result.tabs, previousActiveTabId);
      if (!canActivate && cloneTabId) {
        setAttentionTabs((current) => addAttentionTab(current, rootSessionId, cloneTabId));
      }
      if (rootSessionIdRef.current === rootSessionId) setTabs(nextTabs);
      if (options.mode === "summary") markSessionComplete(rootSessionId);
      return result;
    } catch (error) {
      if (options.mode === "summary") clearSessionRunning(rootSessionId);
      throw error;
    }
  }, [onSessionsRefresh, rootSessionId, tabs?.active_tab_id]);

  const cancelCloneSummary = useCallback(async (operationId: string) => {
    await invoke("cancel_clone_summary", { operationId });
  }, []);

  const closeTab = useCallback(async (tabId: string) => {
    if (!rootSessionId) return;
    setAttentionTabs((current) => removeAttentionTab(current, rootSessionId, tabId));
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
    attentionTabIds,
    refreshTabs,
    selectTab,
    cloneMessage,
    cancelCloneSummary,
    closeTab,
    renameTab,
  };
}

function findCloneTabId(result: CloneSessionResult): string | null {
  return result.tabs.tabs.find((tab) => tab.session_id === result.clone_session_id)?.tab_id ?? null;
}

async function savePreviousActiveTab(
  rootSessionId: string,
  tabs: SessionTabs,
  previousActiveTabId: string,
): Promise<SessionTabs> {
  const activeTabExists = tabs.tabs.some((tab) => tab.tab_id === previousActiveTabId);
  return invoke<SessionTabs>("save_session_tabs", {
    sessionId: rootSessionId,
    tabs: { ...tabs, active_tab_id: activeTabExists ? previousActiveTabId : "main" },
  });
}

function addAttentionTab(
  current: Record<string, string[]>,
  rootSessionId: string,
  tabId: string,
): Record<string, string[]> {
  const ids = current[rootSessionId] ?? [];
  if (ids.includes(tabId)) return current;
  return { ...current, [rootSessionId]: [...ids, tabId].slice(-3) };
}

function removeAttentionTab(
  current: Record<string, string[]>,
  rootSessionId: string,
  tabId: string,
): Record<string, string[]> {
  const ids = current[rootSessionId];
  if (!ids?.includes(tabId)) return current;
  const nextIds = ids.filter((id) => id !== tabId);
  const next = { ...current };
  if (nextIds.length > 0) next[rootSessionId] = nextIds;
  else delete next[rootSessionId];
  return next;
}
