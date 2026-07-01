import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useSessionSummary } from "../use-session-summary";
import type { AgentMessage, AgentSession, StreamEvent } from "@/types/agent";

const invokeMock = vi.fn();
const listeners = new Map<string, ((event: { payload: unknown }) => void)[]>();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]): Promise<unknown> => invokeMock(...args) as Promise<unknown>,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((eventName: string, handler: (event: { payload: unknown }) => void) => {
    listeners.set(eventName, [...(listeners.get(eventName) ?? []), handler]);
    return Promise.resolve(() => {});
  }),
}));

beforeEach(() => {
  listeners.clear();
  invokeMock.mockReset();
});

describe("useSessionSummary", () => {
  it("garde la dernière diff si une requête suivante ne modifie aucun fichier", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "list_subagents") return Promise.resolve([]);
      return Promise.resolve(session([
        assistant("m1", [{ name: "write_file", summary: "a.ts", content: "a\nb" }]),
        assistant("m2", [{ name: "read_file", summary: "a.ts", result: "ok" }]),
      ]));
    });

    const { result } = renderHook(() => useSessionSummary("s1"));
    await waitFor(() => expect(result.current.changes).toEqual({ additions: 2, deletions: 0, files: 1 }));

    act(() => emit("s1", { event: "done", data: { evalCount: 0, evalDurationNs: 0, finalTps: 0, promptTokens: null, contextTokens: null } }));

    expect(result.current.changes).toEqual({ additions: 2, deletions: 0, files: 1 });
  });

  it("remplace la diff affichée en temps réel quand une nouvelle requête modifie un fichier", async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === "list_subagents") return Promise.resolve([]);
      return Promise.resolve(session([
        assistant("m1", [{ name: "write_file", summary: "a.ts", content: "a\nb" }]),
      ]));
    });

    const { result } = renderHook(() => useSessionSummary("s1"));
    await waitFor(() => expect(result.current.changes.additions).toBe(2));

    act(() => {
      emit("s1", { event: "toolCall", data: { name: "edit_file", arguments: { path: "a.ts", old_string: "x\ny", new_string: "z" } } });
      emit("s1", { event: "toolResult", data: { name: "edit_file", content: "ok", isError: false, toolCallIndex: 0 } });
    });

    expect(result.current.changes).toEqual({ additions: 1, deletions: 2, files: 1 });
  });
});

function emit(sessionId: string, event: StreamEvent) {
  for (const handler of listeners.get("agent-stream-event") ?? []) {
    handler({ payload: { sessionId, event } });
  }
}

function session(messages: AgentMessage[]): AgentSession {
  return {
    id: "s1",
    name: "Session",
    created_at: "2026-07-01T12:00:00Z",
    model: "gpt",
    provider: "openai",
    thinking_enabled: false,
    accumulated_tokens: 0,
    messages,
  };
}

function assistant(id: string, tools: AgentMessage["tool_activities"]): AgentMessage {
  return {
    id,
    role: "assistant",
    content: "",
    files: [],
    timestamp: "2026-07-01T12:00:00Z",
    tool_activities: tools,
  };
}
