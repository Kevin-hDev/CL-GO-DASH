import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useAgentChat } from "../use-agent-chat";
import type { AgentSession } from "@/types/agent";

const startStream = vi.fn();
const stopStream = vi.fn();
const subscribeToStream = vi.fn(() => () => {});
const getStreamSnapshot = vi.fn(() => null);

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("../use-agent-stream", () => ({
  useAgentStream: () => ({ startStream, stopStream, subscribeToStream, getStreamSnapshot }),
}));
vi.mock("../use-gateway-session-updates", () => ({
  listenGatewaySessionUpdates: vi.fn(() => () => {}),
}));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

const session: AgentSession = {
  id: "session-1",
  name: "Test",
  model: "llama3",
  provider: "ollama",
  thinking_enabled: false,
  messages: [
    { id: "m1", role: "user", content: "Salut", files: [], timestamp: "2026-06-24T10:00:00Z" },
    { id: "m2", role: "assistant", content: "Bonjour", files: [], timestamp: "2026-06-24T10:00:01Z" },
  ],
  created_at: "2026-06-24T10:00:00Z",
  accumulated_tokens: 12,
};

describe("useAgentChat", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "get_agent_session") return Promise.resolve(session);
      return Promise.resolve(undefined);
    });
  });

  it("reload utilise truncate_and_replace_at sans ancienne commande truncate_session_at", async () => {
    const { result } = renderHook(() => useAgentChat("session-1", "llama3", "ollama"));
    await waitFor(() => expect(result.current.sessionLoading).toBe(false));

    await act(async () => {
      await result.current.reload("m2");
    });

    expect(invoke).toHaveBeenCalledWith("truncate_and_replace_at", {
      sessionId: "session-1",
      messageId: "m2",
      replacement: null,
    });
    expect(invoke).not.toHaveBeenCalledWith("truncate_session_at", expect.anything());
    expect(startStream).toHaveBeenCalled();
  });
});
