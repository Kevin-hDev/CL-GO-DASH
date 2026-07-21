import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ForecastSelectionPolicy } from "./forecast-selection-types";
import { useForecastSelectionPolicy } from "./use-forecast-selection-policy";

const MANUAL: ForecastSelectionPolicy = {
  mode: "manual",
  manual_model_id: "chronos-bolt-small",
  allow_cloud_in_auto: false,
};

describe("useForecastSelectionPolicy", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(listen).mockResolvedValue(() => {});
  });

  it("loads Manual and switches to Auto without losing the manual model", async () => {
    const auto = { ...MANUAL, mode: "auto" as const };
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "get_forecast_selection_policy") return Promise.resolve(MANUAL);
      if (command === "set_forecast_selection_mode") return Promise.resolve(auto);
      return Promise.reject(new Error("unexpected-command"));
    });
    const { result } = renderHook(() => useForecastSelectionPolicy());

    await waitFor(() => expect(result.current.ready).toBe(true));
    expect(result.current.policy).toEqual(MANUAL);
    act(() => result.current.setMode("auto"));
    await waitFor(() => expect(result.current.policy).toEqual(auto));

    expect(result.current.selectedModelId).toBe(MANUAL.manual_model_id);
    expect(invoke).toHaveBeenCalledWith("set_forecast_selection_mode", { mode: "auto" });
  });

  it("rejects malformed backend policy payloads", async () => {
    vi.mocked(invoke).mockResolvedValue({ mode: "auto", manual_model_id: "../bad" });
    const { result } = renderHook(() => useForecastSelectionPolicy());

    await waitFor(() => expect(result.current.ready).toBe(true));

    expect(result.current.policy.mode).toBe("manual");
    expect(result.current.selectedModelId).toBe("");
  });
});
