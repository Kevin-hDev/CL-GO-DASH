import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import { MessageList } from "../message-list";
import type { ToolActivity } from "@/hooks/agent-chat-utils";

const compressionState = vi.hoisted(() => ({ isCompressing: false }));

afterEach(() => {
  compressionState.isCompressing = false;
  cleanup();
});

vi.mock("@/hooks/use-compression", () => ({
  useCompression: () => ({ isCompressing: compressionState.isCompressing }),
}));
vi.mock("../message-tool-timeline", () => ({
  SavedToolTimeline: () => null,
  StreamToolTimeline: ({
    currentContent,
    currentThinking,
    currentTools,
  }: {
    currentContent: string;
    currentThinking: string;
    currentTools: ToolActivity[];
  }) => (
    <div data-testid="stream-timeline">
      {currentContent}
      {currentThinking}
      {currentTools.map((tool) => tool.name).join(",")}
    </div>
  ),
}));
vi.mock("../working-stats", () => ({
  LoadingIndicator: ({ startedAt, liveTokenCount }: { startedAt: number; liveTokenCount: number }) => (
    <div data-testid="loading-indicator">{startedAt}:{liveTokenCount}</div>
  ),
}));
vi.mock("../compression-indicator", () => ({
  CompressionIndicator: () => <div data-testid="compression-indicator" />,
}));
vi.mock("../context-compression-marker", () => ({ ContextCompressionMarker: () => null }));
vi.mock("../user-message", () => ({
  UserMessage: ({ content }: { content: string }) => <div data-testid="user-message">{content}</div>,
}));
vi.mock("../assistant-message", () => ({ AssistantMessage: () => null }));
vi.mock("../subagent-bubble", () => ({ SubagentBubble: () => null }));
vi.mock("../plan-preview-bubble", () => ({ PlanPreviewBubble: () => null }));
vi.mock("../file-change-bubble", () => ({ FileChangeBubble: () => null }));
vi.mock("@/lib/file-preview-utils", () => ({
  collectMessageFileOperations: () => [],
}));
vi.mock("../chat.css", () => ({}));
vi.mock("../messages.css", () => ({}));

function renderStreaming(overrides: {
  currentContent?: string;
  currentThinking?: string;
  currentTools?: ToolActivity[];
  segmentStartedAt?: number | null;
} = {}) {
  const segmentStartedAt = "segmentStartedAt" in overrides
    ? overrides.segmentStartedAt ?? null
    : 123;

  return render(
    <MessageList
      sessionId="session-1"
      messages={[]}
      completedSegments={[]}
      currentContent={overrides.currentContent ?? ""}
      currentThinking={overrides.currentThinking ?? ""}
      currentTools={overrides.currentTools ?? []}
      isStreaming
      tps={0}
      totalElapsedMs={0}
      segmentStartedAt={segmentStartedAt}
      liveTokenCount={7}
    />,
  );
}

describe("MessageList loading indicator", () => {
  it("reste visible pendant le texte live", () => {
    const view = renderStreaming({ currentContent: "texte live" });

    expect(view.getByTestId("stream-timeline").textContent).toContain("texte live");
    expect(view.getByTestId("loading-indicator").textContent).toBe("123:7");
  });

  it("reste visible pendant le thinking live", () => {
    const view = renderStreaming({ currentThinking: "thinking live" });

    expect(view.getByTestId("stream-timeline").textContent).toContain("thinking live");
    expect(view.getByTestId("loading-indicator")).toBeTruthy();
  });

  it("reste visible pendant un outil actif", () => {
    const view = renderStreaming({ currentTools: [{ name: "bash", args: {} }] });

    expect(view.getByTestId("stream-timeline").textContent).toContain("bash");
    expect(view.getByTestId("loading-indicator")).toBeTruthy();
  });

  it("reste visible pendant la compression du stream", () => {
    compressionState.isCompressing = true;
    const view = renderStreaming({ currentContent: "texte live" });

    expect(view.getByTestId("loading-indicator")).toBeTruthy();
    expect(view.getByTestId("compression-indicator")).toBeTruthy();
  });

  it("reste masqué tant que le stream n'a pas d'heure de début", () => {
    const view = renderStreaming({ segmentStartedAt: null });

    expect(view.queryByTestId("loading-indicator")).toBeNull();
  });

  it("affiche le message en attente après le travail courant", () => {
    const view = renderStreaming({ currentContent: "travail visible" });
    view.rerender(
      <MessageList
        sessionId="session-1"
        messages={[]}
        queuedUserMessages={[{
          id: "u2", role: "user", content: "nouvelle précision", files: [], timestamp: "2026-07-12",
        }]}
        completedSegments={[]}
        currentContent="travail visible"
        currentThinking=""
        currentTools={[]}
        isStreaming
        tps={0}
        totalElapsedMs={0}
        segmentStartedAt={123}
        liveTokenCount={7}
      />,
    );

    const timeline = view.getByTestId("stream-timeline");
    const user = view.getByTestId("user-message");
    expect(timeline.compareDocumentPosition(user) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(user.textContent).toBe("nouvelle précision");
  });
});
