import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ForecastSection } from "@/hooks/use-forecast-panel";
import { ForecastPanel } from "@/components/forecast/forecast-panel";
import { openForecastDocsWindow } from "@/components/forecast/open-forecast-docs";

interface ForecastNavigation {
  activeSection: ForecastSection;
  navOpen: boolean;
  currentAnalysisId: string | null;
  setSection: (section: ForecastSection) => void;
  toggleNav: () => void;
  loadAnalysis: (id: string) => void;
  focusAnalysis: (id: string) => void;
  closeAnalysis: () => void;
}

interface FilePreviewControls {
  fullscreen: boolean;
  setFullscreen: (value: boolean) => void;
  setExtraWidth: (width: number) => void;
}

interface Args {
  forecastNav: ForecastNavigation;
  filePreview: FilePreviewControls;
  docsWindowTitle: string;
}

export function useAgentLocalForecastContent({ forecastNav, filePreview, docsWindowTitle }: Args) {
  const [fullscreenSwitching, setFullscreenSwitching] = useState(false);
  const fullscreenTimerRef = useRef<number | null>(null);
  const handlePreviewFullscreenChange = useCallback((value: boolean) => {
    if (fullscreenTimerRef.current !== null) window.clearTimeout(fullscreenTimerRef.current);
    setFullscreenSwitching(true);
    filePreview.setFullscreen(value);
    fullscreenTimerRef.current = window.setTimeout(() => setFullscreenSwitching(false), 80);
  }, [filePreview]);

  useEffect(() => () => {
    if (fullscreenTimerRef.current !== null) window.clearTimeout(fullscreenTimerRef.current);
  }, []);

  const handleOpenForecastDocs = useCallback(() => {
    void openForecastDocsWindow(docsWindowTitle).catch(() => {});
  }, [docsWindowTitle]);

  const forecastContent = useMemo(() => (
    <ForecastPanel
      activeSection={forecastNav.activeSection}
      navOpen={forecastNav.navOpen}
      currentAnalysisId={forecastNav.currentAnalysisId}
      fullscreen={filePreview.fullscreen}
      onSectionChange={forecastNav.setSection}
      onToggleNav={forecastNav.toggleNav}
      onLoadAnalysis={forecastNav.loadAnalysis}
      onFocusAnalysis={forecastNav.focusAnalysis}
      onPanelExtraWidthChange={filePreview.setExtraWidth}
      onCloseAnalysis={forecastNav.closeAnalysis}
      onFullscreenChange={handlePreviewFullscreenChange}
    />
  ), [filePreview.fullscreen, filePreview.setExtraWidth, forecastNav, handlePreviewFullscreenChange]);

  return {
    forecastContent, fullscreenSwitching, handleOpenForecastDocs, handlePreviewFullscreenChange,
  };
}
