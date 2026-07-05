import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import { MessageList } from "../message-list";
import type { AgentMessage } from "@/types/agent";

const compressionState = vi.hoisted(() => ({ isCompressing: false }));

afterEach(() => {
  compressionState.isCompressing = false;
  cleanup();
});

vi.mock("@/hooks/use-compression", () => ({
  useCompression: () => ({ isCompressing: compressionState.isCompressing }),
}));
vi.mock("../compression-indicator", () => ({
  CompressionIndicator: () => <div data-testid="compression-indicator" />,
}));
vi.mock("../context-compression-marker", () => ({
  ContextCompressionMarker: () => <div data-testid="context-marker" />,
}));
vi.mock("../user-message", () => ({ UserMessage: () => null }));
vi.mock("../assistant-message", () => ({ AssistantMessage: () => null }));
vi.mock("../message-tool-timeline", () => ({
  SavedToolTimeline: () => null,
  StreamToolTimeline: () => null,
}));
vi.mock("../subagent-bubble", () => ({ SubagentBubble: () => null }));
vi.mock("../plan-preview-bubble", () => ({ PlanPreviewBubble: () => null }));
vi.mock("../file-change-bubble", () => ({ FileChangeBubble: () => null }));
vi.mock("../working-stats", () => ({ LoadingIndicator: () => null }));
vi.mock("@/lib/subagent-message-utils", () => ({
  extractSubagentsFromMessages: () => [],
  isSubagentInjectedMessage: () => false,
}));
vi.mock("@/lib/file-preview-utils", () => ({
  collectMessageFileOperations: () => [],
}));
vi.mock("../chat.css", () => ({}));
vi.mock("../messages.css", () => ({}));

function renderList(messages: AgentMessage[]) {
  return render(
    <MessageList
      sessionId="session-1"
      messages={messages}
      completedSegments={[]}
      currentContent=""
      currentThinking=""
      currentTools={[]}
      isStreaming={false}
      tps={0}
      totalElapsedMs={0}
      segmentStartedAt={null}
      liveTokenCount={0}
    />,
  );
}

describe("MessageList compression indicator", () => {
  it("masque l'animation quand le marqueur de compression existe déjà", () => {
    compressionState.isCompressing = true;
    const compressedMessage: AgentMessage = {
      id: "compressed-1",
      role: "assistant",
      content: "This session is being continued from a previous conversation.",
      files: [],
      timestamp: new Date(0).toISOString(),
    };

    const view = renderList([compressedMessage]);

    expect(view.queryByTestId("compression-indicator")).toBeNull();
    expect(view.getByTestId("context-marker")).toBeTruthy();
  });
});
