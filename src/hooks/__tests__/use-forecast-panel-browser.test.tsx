import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useForecastPanel } from "../use-forecast-panel";

describe("useForecastPanel browser mode", () => {
  it("restaure le navigateur séparément pour chaque conversation", () => {
    localStorage.setItem("fc-panel-session-browser", JSON.stringify({
      activeSection: "view",
      navOpen: false,
      currentAnalysisId: null,
      panelMode: "browser",
    }));

    const { result } = renderHook(() => useForecastPanel("session-browser"));

    expect(result.current.panelMode).toBe("browser");
  });
});
