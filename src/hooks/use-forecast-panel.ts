import { useState, useCallback, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

export type ForecastSection = "view" | "scenarios" | "analysis" | "notes" | "history";
export type PanelMode = "preview" | "forecast";

interface ForecastPanelState {
  activeSection: ForecastSection;
  navOpen: boolean;
  currentAnalysisId: string | null;
  panelMode: PanelMode;
}

interface ForecastAnalysisCreatedEvent {
  analysis_id: string;
  session_id: string;
}

const DEFAULT_PANEL_STATE = {
  activeSection: "view" as ForecastSection,
  navOpen: false,
  currentAnalysisId: null,
  panelMode: "preview" as PanelMode,
};

function loadFromStorage(storageKey: string): ForecastPanelState {
  try {
    const saved = localStorage.getItem(storageKey);
    if (saved) {
      const parsed: unknown = JSON.parse(saved);
      if (parsed && typeof parsed === "object") {
        return parsed as ForecastPanelState;
      }
    }
  } catch { /* ignore */ }
  return DEFAULT_PANEL_STATE;
}

export function useForecastPanel(sessionId: string | null) {
  const stateKey = sessionId ?? "__no_session__";
  const storageKey = sessionId ? `fc-panel-${sessionId}` : null;

  const [states, setStates] = useState<Record<string, ForecastPanelState>>(() => {
    const saved = storageKey ? loadFromStorage(storageKey) : DEFAULT_PANEL_STATE;
    return { [stateKey]: saved };
  });
  const state = states[stateKey] ?? (storageKey ? loadFromStorage(storageKey) : DEFAULT_PANEL_STATE);

  const persist = useCallback((next: ForecastPanelState) => {
    setStates((prev) => ({ ...prev, [stateKey]: next }));
    if (storageKey) {
      try { localStorage.setItem(storageKey, JSON.stringify(next)); } catch { /* ignore */ }
    }
  }, [stateKey, storageKey]);

  const setSection = useCallback((section: ForecastSection) => {
    persist({ ...state, activeSection: section, navOpen: false });
  }, [state, persist]);

  const toggleNav = useCallback(() => {
    persist({ ...state, navOpen: !state.navOpen });
  }, [state, persist]);

  const loadAnalysis = useCallback((id: string) => {
    persist({ ...state, currentAnalysisId: id, activeSection: "view", panelMode: "forecast" });
  }, [state, persist]);

  const closeAnalysis = useCallback(() => {
    persist({ ...state, currentAnalysisId: null });
  }, [state, persist]);

  const setPanelMode = useCallback((mode: PanelMode) => {
    persist({ ...state, panelMode: mode });
  }, [state, persist]);

  useEffect(() => {
    if (!sessionId) return;
    let cancelled = false;
    const unlisten = listen<ForecastAnalysisCreatedEvent>(
      "forecast-analysis-created",
      (event) => {
        if (cancelled || event.payload.session_id !== sessionId) return;
        loadAnalysis(event.payload.analysis_id);
      },
    );
    return () => {
      cancelled = true;
      void unlisten.then((cleanup) => cleanup());
    };
  }, [sessionId, loadAnalysis]);

  return {
    ...state,
    setSection,
    toggleNav,
    loadAnalysis,
    closeAnalysis,
    setPanelMode,
  };
}
