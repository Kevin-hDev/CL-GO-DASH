import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { MessageList } from "../message-list";
import type { AgentMessage, SubagentInfo } from "@/types/agent";

afterEach(() => cleanup());

vi.mock("@/hooks/use-compression", () => ({
  useCompression: () => ({ isCompressing: false }),
}));
vi.mock("../message-tool-timeline", () => ({
  SavedToolTimeline: () => null,
  StreamToolTimeline: () => null,
}));
vi.mock("../working-stats", () => ({ LoadingIndicator: () => null }));
vi.mock("../compression-indicator", () => ({ CompressionIndicator: () => null }));
vi.mock("../context-compression-marker", () => ({ ContextCompressionMarker: () => null }));
vi.mock("../user-message", () => ({ UserMessage: () => null }));
vi.mock("../assistant-message", () => ({ AssistantMessage: () => null }));
vi.mock("../subagent-bubble", () => ({
  SubagentBubble: ({ subagents }: { subagents: SubagentInfo[] }) => (
    <div data-testid="subagent-bubble">{subagents.map((agent) => agent.name).join(",")}</div>
  ),
}));
vi.mock("../plan-preview-bubble", () => ({ PlanPreviewBubble: () => null }));
vi.mock("../file-change-bubble", () => ({ FileChangeBubble: () => null }));
vi.mock("@/lib/file-preview-utils", () => ({ collectMessageFileOperations: () => [] }));
vi.mock("../chat.css", () => ({}));
vi.mock("../messages.css", () => ({}));

const messages: AgentMessage[] = [{
  id: "u1",
  role: "user",
  content: "Question",
  timestamp: "2026-07-07T10:00:00Z",
  files: [],
}];

describe("MessageList subagents", () => {
  it("affiche les sous-agents terminés sans message injecté", () => {
    render(
      <MessageList
        sessionId="parent"
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
        completedSubagents={[{
          sessionId: "child",
          name: "Audit",
          type: "explorer",
          status: "completed",
          promptPreview: "",
        }]}
      />,
    );

    expect(renderedBubble()).toHaveTextContent("Audit");
  });
});

function renderedBubble() {
  return document.querySelector("[data-testid='subagent-bubble']") as HTMLElement;
}
