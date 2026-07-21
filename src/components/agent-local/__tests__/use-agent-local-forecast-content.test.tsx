import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useAgentLocalForecastContent } from "../use-agent-local-forecast-content";

describe("useAgentLocalForecastContent", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("applies fullscreen changes and clears the transition state", () => {
    const setFullscreen = vi.fn();
    const { result } = renderHook(() => useAgentLocalForecastContent({
      forecastNav: {
        activeSection: "view",
        navOpen: true,
        currentAnalysisId: null,
        setSection: vi.fn(),
        toggleNav: vi.fn(),
        loadAnalysis: vi.fn(),
        focusAnalysis: vi.fn(),
        closeAnalysis: vi.fn(),
      },
      filePreview: {
        fullscreen: false,
        setFullscreen,
        setExtraWidth: vi.fn(),
      },
      sessionId: "550e8400-e29b-41d4-a716-446655440000",
    }));

    act(() => result.current.handlePreviewFullscreenChange(true));
    expect(setFullscreen).toHaveBeenCalledWith(true);
    expect(result.current.fullscreenSwitching).toBe(true);

    act(() => {
      vi.advanceTimersByTime(80);
    });
    expect(result.current.fullscreenSwitching).toBe(false);
  });
});
