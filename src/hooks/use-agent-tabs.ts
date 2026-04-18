import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { TabState, TabInfo } from "@/types/agent";

const MAX_TABS = 6;

export function useAgentTabs() {
  const [tabs, setTabs] = useState<TabInfo[]>([]);
  const [activeIndex, setActiveIndex] = useState(0);

  useEffect(() => {
    invoke<TabState>("get_tab_state").then((state) => {
      setTabs(state.tabs);
      setActiveIndex(state.active_index);
    }).catch(() => {});
  }, []);

  const persist = useCallback(async (next: TabInfo[], idx: number) => {
    setTabs(next);
    setActiveIndex(idx);
    await invoke("save_tab_state", {
      state: { tabs: next, active_index: idx },
    }).catch(() => {});
  }, []);

  const addTab = useCallback(async (sessionId: string, label: string) => {
    if (tabs.length >= MAX_TABS) return;
    const next = [...tabs, { session_id: sessionId, label }];
    await persist(next, next.length - 1);
  }, [tabs, persist]);

  const closeTab = useCallback(async (index: number) => {
    const next = tabs.filter((_, i) => i !== index);
    const newIdx = activeIndex >= next.length ? Math.max(0, next.length - 1) : activeIndex;
    await persist(next, newIdx);
  }, [tabs, activeIndex, persist]);

  const selectTab = useCallback(async (index: number) => {
    await persist(tabs, index);
  }, [tabs, persist]);

  const renameTab = useCallback(async (index: number, label: string) => {
    const next = tabs.map((t, i) => (i !== index ? t : { ...t, label }));
    await persist(next, activeIndex);
  }, [tabs, activeIndex, persist]);

  const reorderTabs = useCallback(async (fromIndex: number, toIndex: number) => {
    if (fromIndex === toIndex) return;
    const next = [...tabs];
    const [moved] = next.splice(fromIndex, 1);
    next.splice(toIndex, 0, moved);
    let newActive = activeIndex;
    if (activeIndex === fromIndex) {
      newActive = toIndex;
    } else if (fromIndex < activeIndex && toIndex >= activeIndex) {
      newActive = activeIndex - 1;
    } else if (fromIndex > activeIndex && toIndex <= activeIndex) {
      newActive = activeIndex + 1;
    }
    await persist(next, newActive);
  }, [tabs, activeIndex, persist]);

  const updateTab = useCallback(async (index: number, sessionId: string, label: string) => {
    const next = tabs.map((t, i) => (i !== index ? t : { session_id: sessionId, label }));
    await persist(next, index);
  }, [tabs, persist]);

  const deselectTab = useCallback(async () => {
    setActiveIndex(-1);
  }, []);

  const canAddTab = tabs.length < MAX_TABS;
  const activeSessionId = tabs[activeIndex]?.session_id ?? null;

  return {
    tabs, activeIndex, activeSessionId, canAddTab,
    addTab, closeTab, selectTab, renameTab, updateTab, reorderTabs, deselectTab,
  };
}
