import { useState, useCallback, useRef, useEffect } from "react";
import { homeDir, join } from "@tauri-apps/api/path";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";

export interface TerminalTab {
  id: string;
  ptyId: number | null;
  label: string;
  cwd: string;
}

export interface TerminalState {
  tabs: TerminalTab[];
  activeTabId: string | null;
  isOpen: boolean;
  panelHeight: number;
}

const DEFAULT_HEIGHT = 120;
const MIN_HEIGHT = 80;

function generateId(): string {
  return crypto.randomUUID();
}

function folderName(cwd: string): string {
  const parts = cwd.replace(/[\\/]$/, "").split(/[\\/]/);
  return parts[parts.length - 1] || "Terminal";
}

interface SavedTab {
  label: string;
  cwd: string;
}

async function getTabsPath(): Promise<string> {
  const home = await homeDir();
  return join(home, ".local", "share", "cl-go-dash", "terminal-tabs.json");
}

async function loadSavedTabs(): Promise<SavedTab[]> {
  try {
    const path = await getTabsPath();
    const text = await readTextFile(path);
    return JSON.parse(text) as SavedTab[];
  } catch (err) {
    // File absent on first launch — not an error worth surfacing
    console.debug("[terminal-tabs] no saved tabs:", err);
    return [];
  }
}

async function saveTabs(tabs: { label: string; cwd: string }[]): Promise<void> {
  try {
    const path = await getTabsPath();
    const data = tabs.map(({ label, cwd }) => ({ label, cwd }));
    await writeTextFile(path, JSON.stringify(data));
  } catch (err) {
    console.warn("[terminal-tabs] failed to save tabs:", err);
  }
}

export function useTerminal(defaultCwd: string) {
  const [state, setState] = useState<TerminalState>({
    tabs: [],
    activeTabId: null,
    isOpen: false,
    panelHeight: DEFAULT_HEIGHT,
  });

  const [loaded, setLoaded] = useState(false);
  const maxHeightRef = useRef(0);

  useEffect(() => {
    loadSavedTabs().then((saved) => {
      if (saved.length > 0) {
        const tabs = saved.map((s) => ({
          id: generateId(),
          ptyId: null,
          label: s.label,
          cwd: s.cwd,
        }));
        setState((prev) => ({
          ...prev,
          tabs,
          activeTabId: tabs[0].id,
          isOpen: false,
        }));
      }
      setLoaded(true);
    });
  }, []);

  useEffect(() => {
    if (!loaded) return;
    saveTabs(state.tabs);
  }, [state.tabs, loaded]);

  const addTab = useCallback((cwd?: string) => {
    const dir = cwd || defaultCwd;
    const tab: TerminalTab = {
      id: generateId(),
      ptyId: null,
      label: folderName(dir),
      cwd: dir,
    };
    setState((prev) => ({
      ...prev,
      tabs: [...prev.tabs, tab],
      activeTabId: tab.id,
      isOpen: true,
    }));
    return tab.id;
  }, [defaultCwd]);

  const closeTab = useCallback((id: string) => {
    setState((prev) => {
      const filtered = prev.tabs.filter((t) => t.id !== id);
      let nextActive = prev.activeTabId;
      if (prev.activeTabId === id) {
        const closedIdx = prev.tabs.findIndex((t) => t.id === id);
        const next = filtered[Math.min(closedIdx, filtered.length - 1)];
        nextActive = next?.id ?? null;
      }
      return {
        ...prev,
        tabs: filtered,
        activeTabId: nextActive,
        isOpen: filtered.length > 0,
      };
    });
  }, []);

  const setActiveTab = useCallback((id: string) => {
    setState((prev) => ({ ...prev, activeTabId: id }));
  }, []);

  const renameTab = useCallback((id: string, label: string) => {
    setState((prev) => ({
      ...prev,
      tabs: prev.tabs.map((t) => (t.id === id ? { ...t, label } : t)),
    }));
  }, []);

  const reorderTabs = useCallback((fromIndex: number, toIndex: number) => {
    setState((prev) => {
      const tabs = [...prev.tabs];
      const [moved] = tabs.splice(fromIndex, 1);
      tabs.splice(toIndex, 0, moved);
      return { ...prev, tabs };
    });
  }, []);

  const togglePanel = useCallback(() => {
    setState((prev) => {
      if (prev.isOpen) {
        return { ...prev, isOpen: false };
      }
      if (prev.tabs.length === 0) {
        return prev;
      }
      return { ...prev, isOpen: true };
    });
  }, []);

  const setPtyId = useCallback((tabId: string, ptyId: number) => {
    setState((prev) => ({
      ...prev,
      tabs: prev.tabs.map((t) => (t.id === tabId ? { ...t, ptyId } : t)),
    }));
  }, []);

  const resizePanel = useCallback((height: number) => {
    const maxH = maxHeightRef.current;
    const clamped = Math.max(MIN_HEIGHT, Math.min(height, maxH));
    setState((prev) => ({ ...prev, panelHeight: clamped }));
  }, []);

  const setMaxHeight = useCallback((maxH: number) => {
    maxHeightRef.current = maxH;
  }, []);

  return {
    ...state,
    addTab,
    closeTab,
    setActiveTab,
    renameTab,
    reorderTabs,
    togglePanel,
    setPtyId,
    resizePanel,
    setMaxHeight,
  };
}
