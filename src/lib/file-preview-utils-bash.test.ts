import { describe, expect, it } from "vitest";
import { collectFileOperationGroups, collectFileOperations } from "./file-preview-utils";
import type { AgentMessage, ToolActivityRecord } from "@/types/agent";

describe("collectFileOperationGroups avec bash", () => {
  it("inclut les fichiers touchés par bash pendant le stream", () => {
    const operations = collectFileOperations([], {
      liveTools: [
        tool({
          name: "bash",
          summary: "touch a.md",
          result: "ok",
          affected_paths: ["/repo/a.md", "/repo/b.tsx"],
        }),
      ],
    });

    expect(operations.map((operation) => operation.path)).toEqual([
      "/repo/a.md",
      "/repo/b.tsx",
    ]);
  });

  it("utilise le dernier bash avec fichiers touchés au lieu d'un ancien write_file", () => {
    const groups = collectFileOperationGroups([
      message("old", [
        tool({ name: "write_file", summary: "/repo/old.ts", content: "old" }),
      ]),
      message("new", [
        tool({
          name: "bash",
          summary: "touch fresh.md",
          result: "ok",
          affected_paths: ["/repo/fresh.md"],
        }),
      ]),
    ]);

    expect(groups.latest.map((operation) => operation.path)).toEqual(["/repo/fresh.md"]);
    expect(groups.all.map((operation) => operation.path)).toEqual([
      "/repo/fresh.md",
      "/repo/old.ts",
    ]);
  });
});

function message(id: string, tools: ToolActivityRecord[]): AgentMessage {
  return {
    id,
    role: "assistant",
    content: "",
    files: [],
    timestamp: "2026-07-03T10:00:00Z",
    tool_activities: tools,
  };
}

function tool(overrides: Partial<ToolActivityRecord>): ToolActivityRecord {
  return {
    name: "bash",
    summary: "",
    ...overrides,
  };
}
