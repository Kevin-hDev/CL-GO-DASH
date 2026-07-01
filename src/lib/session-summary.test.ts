import { describe, expect, it } from "vitest";
import {
  activeTodoRuns,
  childSubagents,
  summarizeLastRequestChanges,
} from "./session-summary";
import type { AgentMessage, AgentSession, AgentSessionMeta } from "@/types/agent";

function message(id: string, tools: AgentMessage["tool_activities"]): AgentMessage {
  return {
    id,
    role: "assistant",
    content: "",
    files: [],
    timestamp: "2026-07-01T12:00:00Z",
    tool_activities: tools,
  };
}

describe("session-summary", () => {
  it("compte uniquement les modifications du dernier message assistant", () => {
    const summary = summarizeLastRequestChanges([
      message("old", [{ name: "write_file", summary: "old.ts", content: "a\nb" }]),
      message("new", [{ name: "edit_file", summary: "new.ts", old_text: "a\nb\nc", new_text: "x" }]),
    ]);

    expect(summary).toEqual({ additions: 1, deletions: 3, files: 1 });
  });

  it("compte write_file comme additions sans suppressions", () => {
    const summary = summarizeLastRequestChanges([
      message("write", [{ name: "write_file", summary: "new.ts", content: "a\nb\nc" }]),
    ]);

    expect(summary).toEqual({ additions: 3, deletions: 0, files: 1 });
  });

  it("ignore les outils en erreur", () => {
    const summary = summarizeLastRequestChanges([
      message("failed", [{
        name: "edit_file",
        summary: "bad.ts",
        old_text: "a",
        new_text: "b",
        is_error: true,
      }]),
    ]);

    expect(summary).toEqual({ additions: 0, deletions: 0, files: 0 });
  });

  it("ne garde que les todo runs actifs", () => {
    const session = {
      todo_runs: [
        { id: "a", title: "Active", status: "active", todos: [], created_at: "", updated_at: "" },
        { id: "p", title: "Paused", status: "paused", todos: [], created_at: "", updated_at: "" },
      ],
    } satisfies Pick<AgentSession, "todo_runs">;

    expect(activeTodoRuns(session).map((run) => run.title)).toEqual(["Active"]);
  });

  it("filtre les sous-agents par session parent", () => {
    const sessions = [
      meta("child-a", "parent", "explorer"),
      meta("other-child", "other", "coder"),
      meta("child-b", "parent", "coder"),
    ];

    expect(childSubagents("parent", sessions).map((agent) => agent.sessionId))
      .toEqual(["child-a", "child-b"]);
  });
});

function meta(id: string, parent: string, type: "explorer" | "coder"): AgentSessionMeta {
  return {
    id,
    name: id,
    created_at: "2026-07-01T12:00:00Z",
    model: "gpt",
    provider: "openai",
    message_count: 0,
    parent_session_id: parent,
    subagent_type: type,
  };
}
