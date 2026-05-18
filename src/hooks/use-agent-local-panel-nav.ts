import { useEffect, useRef } from "react";
import type { useFileTree } from "@/hooks/use-file-tree";
import type { useForecastPanel } from "@/hooks/use-forecast-panel";
import type { AgentLocalNavState, DeepPartial } from "@/types/navigation";

interface AgentLocalPanelNavArgs {
  navState: AgentLocalNavState;
  fileTree: ReturnType<typeof useFileTree>;
  forecast: ReturnType<typeof useForecastPanel>;
  onNavChange?: (partial: DeepPartial<AgentLocalNavState>) => void;
  onNavReplace?: (partial: DeepPartial<AgentLocalNavState>) => void;
}

export function useAgentLocalPanelNav({
  navState,
  fileTree,
  forecast,
  onNavChange,
  onNavReplace,
}: AgentLocalPanelNavArgs) {
  const reportedPanelState = useRef(false);
  const restoredNavKey = useRef<string | null>(null);
  const { open: fileTreeOpen, setOpen: setFileTreeOpen } = fileTree;
  const { panelMode, currentAnalysisId, activeSection, restorePanelState } = forecast;
  const navKey = JSON.stringify([
    navState.fileTreeOpen,
    navState.panelMode,
    navState.forecastAnalysisId,
    navState.forecastSection,
  ]);

  useEffect(() => {
    if (restoredNavKey.current === navKey) return;
    restoredNavKey.current = navKey;
    if (fileTreeOpen !== navState.fileTreeOpen) setFileTreeOpen(navState.fileTreeOpen);
    const forecastChanged =
      panelMode !== navState.panelMode ||
      currentAnalysisId !== navState.forecastAnalysisId ||
      activeSection !== navState.forecastSection;
    if (forecastChanged) {
      restorePanelState({
        activeSection: navState.forecastSection,
        navOpen: false,
        currentAnalysisId: navState.forecastAnalysisId,
        panelMode: navState.panelMode,
      });
    }
  }, [
    navKey, fileTreeOpen, setFileTreeOpen, panelMode, currentAnalysisId, activeSection,
    restorePanelState, navState.fileTreeOpen, navState.panelMode,
    navState.forecastAnalysisId, navState.forecastSection,
  ]);

  useEffect(() => {
    const report = reportedPanelState.current ? onNavChange : onNavReplace ?? onNavChange;
    reportedPanelState.current = true;
    report?.({
      fileTreeOpen,
      panelMode,
      forecastSection: activeSection,
      forecastAnalysisId: currentAnalysisId,
    });
  }, [
    fileTreeOpen, panelMode, activeSection, currentAnalysisId,
    onNavChange, onNavReplace,
  ]);
}
