import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useAgentStream } from "../use-agent-stream";
import type { AgentMessage } from "@/types/agent";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  readFile: vi.fn(),
  startSession: vi.fn(),
  failSession: vi.fn(),
  stopSession: vi.fn(),
  subscribe: vi.fn(),
  getSnapshot: vi.fn(),
  isStreaming: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@tauri-apps/plugin-fs", () => ({
  readFile: mocks.readFile,
}));

vi.mock("../agent-stream-manager", () => ({
  agentStreamManager: {
    startSession: mocks.startSession,
    failSession: mocks.failSession,
    stopSession: mocks.stopSession,
    subscribe: mocks.subscribe,
    getSnapshot: mocks.getSnapshot,
    isStreaming: mocks.isStreaming,
  },
}));

function userMessage(files: AgentMessage["files"]): AgentMessage {
  return {
    id: "m1",
    role: "user",
    content: "Tu vois l'image ?",
    files,
    timestamp: "2026-06-24T10:00:00Z",
  };
}

describe("useAgentStream", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.invoke.mockResolvedValue(1);
    mocks.startSession.mockResolvedValue(undefined);
  });

  it("renvoie une image depuis son thumbnail quand le chemin n'est plus lisible", async () => {
    mocks.readFile.mockRejectedValue(new Error("missing file"));
    const message = userMessage([{
      name: "logo.png",
      path: "/tmp/logo.png",
      mime_type: "image/png",
      size: 123,
      thumbnail: "data:image/png;base64,abc123",
    }]);
    const { result } = renderHook(() => useAgentStream());

    await act(async () => {
      await result.current.startStream(
        "session-1",
        "model",
        "provider",
        [message],
        false,
        { displayMessages: [message], baseTokenCount: 0 },
      );
    });

    expect(mocks.invoke).toHaveBeenCalledWith("chat_stream", expect.objectContaining({
      messages: [expect.objectContaining({ images: ["abc123"] })],
    }));
  });

  it("envoie une image depuis son thumbnail quand elle n'a pas de chemin fichier", async () => {
    const message = userMessage([{
      name: "paste.png",
      path: "",
      mime_type: "image/png",
      size: 123,
      thumbnail: "data:image/png;base64,pasted123",
    }]);
    const { result } = renderHook(() => useAgentStream());

    await act(async () => {
      await result.current.startStream(
        "session-1",
        "model",
        "provider",
        [message],
        false,
        { displayMessages: [message], baseTokenCount: 0 },
      );
    });

    expect(mocks.readFile).not.toHaveBeenCalled();
    expect(mocks.invoke).toHaveBeenCalledWith("chat_stream", expect.objectContaining({
      messages: [expect.objectContaining({ images: ["pasted123"] })],
    }));
  });
});
