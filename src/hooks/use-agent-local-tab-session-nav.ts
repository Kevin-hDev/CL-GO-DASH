import { useCallback } from "react";
import type { useAgentTabs } from "@/hooks/use-agent-tabs";

interface Args {
  tabState: ReturnType<typeof useAgentTabs>;
  remove: (id: string) => Promise<void>;
  onSessionChange?: (id: string | null) => void;
}

function nextSessionAfterClose(tabState: ReturnType<typeof useAgentTabs>, index: number): string | null {
  const nextTabs = index < 0 ? tabState.tabs : tabState.tabs.filter((_, i) => i !== index);
  const nextIndex = tabState.activeIndex >= nextTabs.length
    ? Math.max(0, nextTabs.length - 1)
    : tabState.activeIndex;
  return nextTabs[nextIndex]?.session_id ?? null;
}

export function useAgentLocalTabSessionNav({ tabState, remove, onSessionChange }: Args) {
  const handleTabSelect = useCallback((index: number) => {
    const sessionId = tabState.tabs[index]?.session_id ?? null;
    void tabState.selectTab(index).then(() => onSessionChange?.(sessionId));
  }, [onSessionChange, tabState]);

  const handleTabClose = useCallback((index: number) => {
    const nextSessionId = nextSessionAfterClose(tabState, index);
    void tabState.closeTab(index).then(() => onSessionChange?.(nextSessionId));
  }, [onSessionChange, tabState]);

  const handleDeleteSession = useCallback((id: string) => {
    const index = tabState.tabs.findIndex((tab) => tab.session_id === id);
    const nextSessionId = nextSessionAfterClose(tabState, index);
    void tabState.closeBySessionId(id).then(() => {
      onSessionChange?.(nextSessionId);
      return remove(id);
    });
  }, [onSessionChange, remove, tabState]);

  return { handleTabSelect, handleTabClose, handleDeleteSession };
}
