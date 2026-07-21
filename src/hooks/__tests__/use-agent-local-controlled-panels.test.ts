import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { useFilePreview } from "@/hooks/use-file-preview";
import type { useFileTree } from "@/hooks/use-file-tree";
import type { useForecastPanel } from "@/hooks/use-forecast-panel";
import { DEFAULT_APP_NAV } from "@/types/navigation";
import { useAgentLocalControlledPanels } from "../use-agent-local-controlled-panels";

function dependencies() {
  return {
    filePreview: {
      setFullscreen: vi.fn(),
    } as unknown as ReturnType<typeof useFilePreview>,
    fileTree: {
      setOpen: vi.fn(),
      closeTree: vi.fn(),
    } as unknown as ReturnType<typeof useFileTree>,
    forecast: {
      setPanelMode: vi.fn(),
      setSection: vi.fn(),
      loadAnalysis: vi.fn(),
      focusAnalysis: vi.fn(),
      closeAnalysis: vi.fn(),
    } as unknown as ReturnType<typeof useForecastPanel>,
  };
}

describe("useAgentLocalControlledPanels", () => {
  it("sort du plein écran quand Forecast devient actif", () => {
    const deps = dependencies();
    const onNavChange = vi.fn();
    const { result } = renderHook(() => useAgentLocalControlledPanels({
      ...deps,
      navState: { ...DEFAULT_APP_NAV.agentLocal, previewFullscreen: true },
      onNavChange,
    }));

    act(() => result.current.forecastNav.setPanelMode("forecast"));

    expect(deps.filePreview.setFullscreen).toHaveBeenCalledWith(false);
    expect(onNavChange).toHaveBeenCalledWith({
      panelMode: "forecast",
      previewFullscreen: false,
    });
  });

  it("charge une analyse sans masquer la conversation", () => {
    const deps = dependencies();
    const onNavChange = vi.fn();
    const { result } = renderHook(() => useAgentLocalControlledPanels({
      ...deps,
      navState: { ...DEFAULT_APP_NAV.agentLocal, previewFullscreen: true },
      onNavChange,
    }));

    act(() => result.current.forecastNav.loadAnalysis("analysis-id"));

    expect(deps.forecast.loadAnalysis).toHaveBeenCalledWith("analysis-id");
    expect(onNavChange).toHaveBeenCalledWith({
      forecastAnalysisId: "analysis-id",
      forecastSection: "view",
      panelMode: "forecast",
      previewFullscreen: false,
    });
  });
});
