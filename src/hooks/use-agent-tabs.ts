import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { TabState, TabInfo } from "@/types/agent";

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

  const activeSessionId = tabs[activeIndex]?.session_id ?? null;

  return { tabs, activeIndex, activeSessionId, addTab, closeTab, selectTab, renameTab };
}
