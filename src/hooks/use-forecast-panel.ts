import { useState, useCallback } from "react";

export type ForecastSection = "view" | "scenarios" | "analysis" | "notes" | "history";
export type PanelMode = "preview" | "forecast";

interface ForecastPanelState {
  activeSection: ForecastSection;
  navOpen: boolean;
  currentAnalysisId: string | null;
  panelMode: PanelMode;
  _sessionId: string | null;
}

const DEFAULT_PANEL_STATE = {
  activeSection: "view" as ForecastSection,
  navOpen: false,
  currentAnalysisId: null,
  panelMode: "preview" as PanelMode,
};

function loadFromStorage(storageKey: string): Omit<ForecastPanelState, "_sessionId"> {
  try {
    const saved = localStorage.getItem(storageKey);
    if (saved) {
      const parsed: unknown = JSON.parse(saved);
      if (parsed && typeof parsed === "object") {
        return parsed as Omit<ForecastPanelState, "_sessionId">;
      }
    }
  } catch { /* ignore */ }
  return DEFAULT_PANEL_STATE;
}

export function useForecastPanel(sessionId: string | null) {
  const storageKey = sessionId ? `fc-panel-${sessionId}` : null;

  const [state, setState] = useState<ForecastPanelState>(() => {
    const saved = storageKey ? loadFromStorage(storageKey) : DEFAULT_PANEL_STATE;
    return { ...saved, _sessionId: sessionId };
  });

  // Si sessionId a changé depuis le dernier render, recharger depuis localStorage
  // sans useEffect — on dérive dans le render lui-même (pattern "derived state reset")
  const activeStorageKey = state._sessionId ? `fc-panel-${state._sessionId}` : null;
  if (activeStorageKey !== storageKey) {
    const next = storageKey ? loadFromStorage(storageKey) : DEFAULT_PANEL_STATE;
    setState({ ...next, _sessionId: sessionId });
  }

  const persist = useCallback((next: ForecastPanelState) => {
    setState(next);
    if (storageKey) {
      try { localStorage.setItem(storageKey, JSON.stringify(next)); } catch { /* ignore */ }
    }
  }, [storageKey]);

  const setSection = useCallback((section: ForecastSection) => {
    persist({ ...state, activeSection: section, navOpen: false });
  }, [state, persist]);

  const toggleNav = useCallback(() => {
    persist({ ...state, navOpen: !state.navOpen });
  }, [state, persist]);

  const loadAnalysis = useCallback((id: string) => {
    persist({ ...state, currentAnalysisId: id, activeSection: "view" });
  }, [state, persist]);

  const closeAnalysis = useCallback(() => {
    persist({ ...state, currentAnalysisId: null });
  }, [state, persist]);

  const setPanelMode = useCallback((mode: PanelMode) => {
    persist({ ...state, panelMode: mode });
  }, [state, persist]);

  return {
    ...state,
    setSection,
    toggleNav,
    loadAnalysis,
    closeAnalysis,
    setPanelMode,
  };
}
