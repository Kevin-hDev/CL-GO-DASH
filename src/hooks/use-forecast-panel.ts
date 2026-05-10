import { useState, useCallback } from "react";

export type ForecastSection = "view" | "scenarios" | "analysis" | "notes" | "history";
export type PanelMode = "preview" | "forecast";

interface ForecastPanelState {
  activeSection: ForecastSection;
  navOpen: boolean;
  currentAnalysisId: string | null;
  panelMode: PanelMode;
}

export function useForecastPanel(sessionId: string | null) {
  const storageKey = sessionId ? `fc-panel-${sessionId}` : null;

  const [state, setState] = useState<ForecastPanelState>(() => {
    if (storageKey) {
      try {
        const saved = localStorage.getItem(storageKey);
        if (saved) {
          const parsed: unknown = JSON.parse(saved);
          if (parsed && typeof parsed === "object") return parsed as ForecastPanelState;
        }
      } catch { /* ignore */ }
    }
    return {
      activeSection: "view",
      navOpen: false,
      currentAnalysisId: null,
      panelMode: "preview",
    };
  });

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
