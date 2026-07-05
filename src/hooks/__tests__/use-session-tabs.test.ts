import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useSessionTabs } from "../use-session-tabs";
import type { CloneSessionResult, SessionTabs } from "@/types/agent";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../use-session-activity-indicators", () => ({
  clearSessionRunning: vi.fn(),
  markSessionComplete: vi.fn(),
  markSessionRunning: vi.fn(),
}));

const rootTabs: SessionTabs = {
  active_tab_id: "main",
  tabs: [{ tab_id: "main", session_id: "root", label: "Main", is_main: true }],
};

const cloneTabs: SessionTabs = {
  active_tab_id: "branch-1",
  tabs: [
    ...rootTabs.tabs,
    { tab_id: "branch-1", session_id: "clone", label: "Branche 1", is_main: false },
  ],
};

const cloneResult: CloneSessionResult = {
  root_session_id: "root",
  clone_session_id: "clone",
  operation_id: "op-1",
  tabs: cloneTabs,
};

describe("useSessionTabs", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation((command: string, args?: unknown) => {
      if (command === "list_session_tabs") return Promise.resolve(rootTabs);
      if (command === "clone_agent_session") return Promise.resolve(cloneResult);
      if (command === "save_session_tabs") {
        return Promise.resolve((args as { tabs: SessionTabs }).tabs);
      }
      return Promise.resolve(rootTabs);
    });
  });

  it("garde l'onglet actif précédent quand le résumé finit en arrière-plan", async () => {
    const { result } = renderHook(() => useSessionTabs("root"));
    await waitFor(() => expect(result.current.tabs).toEqual(rootTabs));

    await act(async () => {
      await result.current.cloneMessage({
        messageId: "m1",
        mode: "summary",
        operationId: "op-frontend",
        shouldActivateOnComplete: () => false,
      });
    });

    expect(invoke).toHaveBeenCalledWith("clone_agent_session", {
      sessionId: "root",
      messageId: "m1",
      mode: "summary",
      customFocus: null,
      operationId: "op-frontend",
    });
    expect(invoke).toHaveBeenCalledWith("save_session_tabs", {
      sessionId: "root",
      tabs: { ...cloneTabs, active_tab_id: "main" },
    });
    expect(result.current.tabs?.active_tab_id).toBe("main");
    expect(result.current.attentionTabIds.has("branch-1")).toBe(true);
  });

  it("propage l'erreur backend quand le maximum de 3 onglets est atteint", async () => {
    vi.mocked(invoke).mockImplementation((command: string) => {
      if (command === "list_session_tabs") return Promise.resolve(rootTabs);
      if (command === "clone_agent_session") return Promise.reject(new Error("max tabs"));
      return Promise.resolve(rootTabs);
    });
    const { result } = renderHook(() => useSessionTabs("root"));
    await waitFor(() => expect(result.current.tabs).toEqual(rootTabs));

    await expect(result.current.cloneMessage({
      messageId: "m1",
      mode: "summary",
      operationId: "op-frontend",
    })).rejects.toThrow("max tabs");
  });
});
