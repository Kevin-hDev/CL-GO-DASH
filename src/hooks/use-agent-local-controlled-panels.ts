import { useCallback, useMemo } from "react";
import type { useFileTree } from "@/hooks/use-file-tree";
import type { ForecastSection, PanelMode, useForecastPanel } from "@/hooks/use-forecast-panel";
import type { AgentLocalNavState, DeepPartial } from "@/types/navigation";

interface Args {
  navState: AgentLocalNavState;
  fileTree: ReturnType<typeof useFileTree>;
  forecast: ReturnType<typeof useForecastPanel>;
  onNavChange?: (partial: DeepPartial<AgentLocalNavState>) => void;
}

export function useAgentLocalControlledPanels({ navState, fileTree, forecast, onNavChange }: Args) {
  const toggleFileTree = useCallback(() => {
    const nextOpen = !navState.fileTreeOpen;
    fileTree.setOpen(nextOpen);
    onNavChange?.({ fileTreeOpen: nextOpen });
  }, [fileTree, navState.fileTreeOpen, onNavChange]);

  const fileTreeNav = useMemo(() => ({
    ...fileTree,
    open: navState.fileTreeOpen,
    toggleOpen: toggleFileTree,
    closeTree: () => {
      fileTree.closeTree();
      onNavChange?.({ fileTreeOpen: false });
    },
  }), [fileTree, navState.fileTreeOpen, onNavChange, toggleFileTree]);

  const setPanelMode = useCallback((mode: PanelMode) => {
    forecast.setPanelMode(mode);
    onNavChange?.({ panelMode: mode });
  }, [forecast, onNavChange]);

  const setSection = useCallback((section: ForecastSection) => {
    forecast.setSection(section);
    onNavChange?.({ forecastSection: section });
  }, [forecast, onNavChange]);

  const loadAnalysis = useCallback((id: string) => {
    forecast.loadAnalysis(id);
    onNavChange?.({ forecastAnalysisId: id, forecastSection: "view", panelMode: "forecast" });
  }, [forecast, onNavChange]);

  const focusAnalysis = useCallback((id: string) => {
    forecast.focusAnalysis(id);
    onNavChange?.({ forecastAnalysisId: id, panelMode: "forecast" });
  }, [forecast, onNavChange]);

  const closeAnalysis = useCallback(() => {
    forecast.closeAnalysis();
    onNavChange?.({ forecastAnalysisId: null });
  }, [forecast, onNavChange]);

  const forecastNav = useMemo(() => ({
    ...forecast,
    activeSection: navState.forecastSection,
    currentAnalysisId: navState.forecastAnalysisId,
    panelMode: navState.panelMode,
    setSection,
    loadAnalysis,
    focusAnalysis,
    closeAnalysis,
    setPanelMode,
  }), [
    closeAnalysis, focusAnalysis, forecast, loadAnalysis,
    navState.forecastAnalysisId, navState.forecastSection,
    navState.panelMode, setPanelMode, setSection,
  ]);

  return { fileTreeNav, forecastNav };
}
