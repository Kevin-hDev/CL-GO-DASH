import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ForecastWorkbenchSnapshot } from "./forecast-workbench-types";
import { useForecastWorkbenchContext } from "./use-forecast-workbench-context";

const FIRST: ForecastWorkbenchSnapshot = {
  context: {
    session_id: "550e8400-e29b-41d4-a716-446655440000",
    analysis_id: null,
    revision: 1,
  },
  session_name: "Session ventes",
  analysis_name: null,
};

describe("useForecastWorkbenchContext", () => {
  let contextChanged: (() => void) | null;

  beforeEach(() => {
    contextChanged = null;
    vi.mocked(invoke).mockReset();
    vi.mocked(listen).mockImplementation((_event, handler) => {
      contextChanged = () => handler({ payload: FIRST.context } as never);
      return Promise.resolve(() => {});
    });
  });

  it("reloads the backend snapshot after a compact context event", async () => {
    const second: ForecastWorkbenchSnapshot = {
      ...FIRST,
      context: { ...FIRST.context, revision: 2 },
      analysis_name: "Prévision juillet",
    };
    vi.mocked(invoke)
      .mockResolvedValueOnce(FIRST)
      .mockResolvedValueOnce(second);
    const { result } = renderHook(() => useForecastWorkbenchContext());

    await waitFor(() => expect(result.current.snapshot).toEqual(FIRST));
    act(() => contextChanged?.());
    await waitFor(() => expect(result.current.snapshot).toEqual(second));

    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it("fails closed when the snapshot is malformed", async () => {
    vi.mocked(invoke).mockResolvedValue({ context: { session_id: "../bad" } });
    const { result } = renderHook(() => useForecastWorkbenchContext());

    await waitFor(() => expect(result.current.failed).toBe(true));
    expect(result.current.snapshot).toBeNull();
  });
});
