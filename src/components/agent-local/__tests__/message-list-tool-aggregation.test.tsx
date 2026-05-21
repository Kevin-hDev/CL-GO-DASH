import { describe, it, expect, vi, afterEach } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/react";
import { MessageList, SegmentedAssistantMessage } from "../message-list";
import type { AgentMessage } from "@/types/agent";

afterEach(cleanup);

vi.mock("@phosphor-icons/react", () => ({
  Spinner: () => <span data-testid="spinner" />,
}));
vi.mock("@/components/ui/icons", () => ({
  Copy: () => <span />,
  Check: () => <span data-testid="check-icon" />,
}));
vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn() }));
vi.mock("@/hooks/use-hover-class", () => ({ useHoverClass: () => ({ current: null }) }));
vi.mock("@/hooks/use-compression", () => ({ useCompression: () => ({ isCompressing: false }) }));
vi.mock("../working-stats", () => ({
  LoadingIndicator: () => <div data-testid="loading-indicator" />,
}));
vi.mock("../compression-indicator", () => ({
  CompressionIndicator: () => <div data-testid="compression-indicator" />,
}));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    i18n: { language: "en" },
    t: (key: string, opts?: Record<string, unknown>) => {
      const text = (value: unknown) => (
        typeof value === "string" || typeof value === "number" ? String(value) : ""
      );
      const count = text(opts?.count);
      if (key === "agentLocal.toolActivity.summary") {
        return `${text(opts?.group)}: ${text(opts?.details)}`;
      }
      if (key === "agentLocal.toolActivity.toggleDetails") return "Show tool details";
      if (key === "agentLocal.toolActivity.groups.command") return "Commands";
      if (key === "agentLocal.toolActivity.groups.modification") return "Changes";
      if (key === "agentLocal.toolActivity.groups.exploration") return "Exploration";
      if (key === "agentLocal.toolActivity.counts.files") return `${count} file read`;
      if (key === "agentLocal.toolActivity.counts.commands") return `${count} command executed`;
      if (key === "agentLocal.toolActivity.counts.writes") return `${count} file written`;
      if (key === "agentLocal.toolActivity.counts.edits") return `${count} file edited`;
      if (key === "agentLocal.toolActivity.actions.read") return "Read";
      if (key === "agentLocal.toolActivity.actions.create") return "Create";
      if (key === "agentLocal.toolActivity.actions.edit") return "Edit";
      if (key === "agentLocal.toolActivity.actions.list") return "List";
      if (key === "agentLocal.toolActivity.actions.search") return "Search";
      if (key === "agentLocal.toolActivity.actions.run") return "Run";
      if (key === "agentLocal.toolActivity.actions.createBranch") return "Create branch";
      if (key === "agentLocal.toolActivity.actions.switchBranch") return "Switch branch";
      if (key === "agentLocal.toolActivity.actions.tool") return "Tool";
      return key;
    },
  }),
}));
vi.mock("../tool-previews", () => ({
  ContentPreview: () => <div data-testid="content-preview" />,
  DiffPreview: () => <div data-testid="diff-preview" />,
  WebResultsPreview: () => <div data-testid="web-preview" />,
}));
vi.mock("../tool-office-previews", () => ({
  ReadSpreadsheetPreview: () => <div data-testid="read-spreadsheet-preview" />,
  WriteSpreadsheetPreview: () => <div data-testid="write-spreadsheet-preview" />,
  DocumentResultPreview: () => <div data-testid="document-preview" />,
  WriteDocumentPreview: () => <div data-testid="write-document-preview" />,
}));
vi.mock("@/lib/tool-file-path", () => ({
  isFileTool: (name: string) => ["write_file", "edit_file"].includes(name),
}));
vi.mock("../chat.css", () => ({}));
vi.mock("../messages.css", () => ({}));
vi.mock("../tool-bubble.css", () => ({}));

function assistantWithSegmentTools(): AgentMessage {
  return {
    id: "assistant-1",
    role: "assistant",
    content: "",
    files: [],
    timestamp: new Date(0).toISOString(),
    segments: [
      {
        thinking: "first reflection",
        content: "",
        tools: [
          { name: "bash", summary: "npm test", result: "ok" },
          { name: "write_file", summary: "a.ts", result: "ok", content: "x" },
        ],
      },
      {
        content: "",
        tools: [{ name: "edit_file", summary: "b.ts", result: "ok", old_text: "a", new_text: "b" }],
      },
      {
        thinking: "second reflection",
        content: "",
        tools: [{ name: "bash", summary: "npm run build", result: "ok" }],
      },
    ],
  };
}

describe("MessageList tool aggregation", () => {
  it("garde une bulle compacte par segment sauvegardé pour préserver l'ordre", () => {
    const { container } = render(
      <SegmentedAssistantMessage
        msg={assistantWithSegmentTools()}
        tps={0}
        totalElapsedMs={0}
      />,
    );

    const bubbles = container.querySelectorAll(".chat-bubble");
    expect(bubbles).toHaveLength(2);
    expect(bubbles[0].textContent).toContain("Commands: 1 command executed");
    expect(bubbles[0].textContent).toContain("Changes: 1 file written and 1 file edited");
    expect(bubbles[1].textContent).toContain("Commands: 1 command executed");
    expect(container.textContent).not.toContain("npm test");

    fireEvent.click(bubbles[0].querySelectorAll(".tb-group-toggle")[0]);
    expect(container.textContent).toContain("npm test");
    expect(container.textContent).not.toContain("npm run build");
  });

  it("garde les tools du stream sous le segment qui les a produits", () => {
    const { container } = render(
      <MessageList
        sessionId="session-1"
        messages={[]}
        completedSegments={[
          {
            thinking: "first reflection",
            content: "",
            tools: [
              { name: "bash", args: { command: "npm test" }, result: "ok" },
              { name: "write_file", args: { path: "a.ts", content: "x" }, result: "ok" },
            ],
          },
          {
            thinking: "",
            content: "",
            tools: [{ name: "read_file", args: { path: "a.ts" }, result: "contents" }],
          },
          { thinking: "second reflection", content: "", tools: [{ name: "bash", args: { command: "npm run build" }, result: "ok" }] },
        ]}
        currentContent=""
        currentThinking=""
        currentTools={[{ name: "edit_file", args: { path: "b.ts", old_string: "a", new_string: "b" }, result: "ok" }]}
        isStreaming
        tps={0}
        totalElapsedMs={0}
        segmentStartedAt={1}
        liveTokenCount={0}
      />,
    );

    const bubbles = container.querySelectorAll(".chat-bubble");
    expect(bubbles).toHaveLength(2);
    expect(bubbles[0].textContent).toContain("Commands: 1 command executed");
    expect(bubbles[0].textContent).toContain("Changes: 1 file written");
    expect(bubbles[0].textContent).toContain("Exploration: 1 file read");
    expect(bubbles[1].textContent).toContain("Commands: 1 command executed");
    expect(bubbles[1].textContent).toContain("Changes: 1 file edited");
  });
});
