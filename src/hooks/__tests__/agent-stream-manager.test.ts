import { beforeEach, describe, expect, it, vi } from "vitest";
import { agentStreamManager } from "../agent-stream-manager";
import { records } from "../agent-stream-records";
import type { AgentMessage, StreamEvent } from "@/types/agent";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));

let streamHandler: ((event: { payload: { sessionId: string; event: StreamEvent } }) => void) | null = null;

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: mocks.listen,
}));

function message(id: string, role: AgentMessage["role"], content: string): AgentMessage {
  return { id, role, content, timestamp: "2026-06-24T10:00:00Z", files: [] };
}

function emit(sessionId: string, event: StreamEvent) {
  streamHandler?.({ payload: { sessionId, event } });
}

describe("agentStreamManager", () => {
  beforeEach(() => {
    records.clear();
    streamHandler = null;
    vi.clearAllMocks();
    vi.stubGlobal("requestAnimationFrame", vi.fn());
    vi.stubGlobal("cancelAnimationFrame", vi.fn());
    mocks.listen.mockImplementation((_event: string, handler: typeof streamHandler) => {
      streamHandler = handler;
      return Promise.resolve(() => {});
    });
  });

  it("recharge la session et vide les buffers après compressionComplete", async () => {
    const reloadedMessages = [
      message("m1", "user", "Résumé de compression"),
      message("m2", "assistant", "réponse partielle"),
    ];
    mocks.invoke.mockResolvedValue({
      messages: reloadedMessages,
      accumulated_tokens: 42,
    });

    await agentStreamManager.startSession("s1", [message("u1", "user", "Question")], 10);
    emit("s1", { event: "token", data: { content: "réponse partielle", tokenCount: 3, tps: 1 } });
    emit("s1", { event: "thinking", data: { content: "raisonnement" } });
    emit("s1", { event: "toolCall", data: { name: "bash", arguments: { cmd: "pwd" } } });
    emit("s1", { event: "turnEnd", data: {} });
    emit("s1", { event: "token", data: { content: "suite", tokenCount: 4, tps: 1 } });

    const before = agentStreamManager.getSnapshot("s1");
    expect(before?.completedSegments).toHaveLength(1);
    expect(before?.currentContent).toBe("suite");

    emit("s1", { event: "compressionComplete", data: {} });

    await vi.waitFor(() => {
      expect(agentStreamManager.getSnapshot("s1")?.messages).toEqual(reloadedMessages);
    });

    const after = agentStreamManager.getSnapshot("s1");
    expect(after?.messages[1]?.content).toBe("réponse partielle");
    expect(after?.completedSegments).toEqual([]);
    expect(after?.currentContent).toBe("");
    expect(after?.currentThinking).toBe("");
    expect(after?.currentTools).toEqual([]);
    expect(after?.isStreaming).toBe(false);
    expect(mocks.invoke).toHaveBeenCalledWith("get_agent_session", { id: "s1" });
  });
});
