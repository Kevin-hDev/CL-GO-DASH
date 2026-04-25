import type { ToolActivityRecord } from "@/types/agent";

export interface ToolActivity {
  name: string;
  args: Record<string, unknown>;
  result?: string;
  isError?: boolean;
}

export interface StreamSegment {
  thinking: string;
  tools: ToolActivity[];
  content: string;
}

export function toolsToRecords(tools: ToolActivity[]): ToolActivityRecord[] {
  return tools.map((t) => {
    const a = t.args;
    let summary = "";
    if (t.name === "bash") summary = String(a.command ?? "");
    else if (t.name === "read_file" || t.name === "write_file") summary = String(a.path ?? "");
    else if (t.name === "edit_file") summary = String(a.path ?? "");
    else if (t.name === "list_dir") summary = String(a.path ?? ".");
    else if (t.name === "grep") summary = String(a.pattern ?? "");
    else if (t.name === "glob") summary = String(a.pattern ?? "");
    else if (t.name === "web_search") summary = String(a.query ?? "");
    else if (t.name === "web_fetch") summary = String(a.url ?? "");
    else summary = JSON.stringify(a).slice(0, 80);

    return {
      name: t.name,
      summary,
      result: t.result,
      is_error: t.isError,
      content: t.name === "write_file" ? String(a.content ?? "") : undefined,
      old_text: t.name === "edit_file" ? String(a.old_string ?? "") : undefined,
      new_text: t.name === "edit_file" ? String(a.new_string ?? "") : undefined,
    };
  });
}

export function segmentsToRecords(segments: StreamSegment[]): ToolActivityRecord[] {
  return segments.flatMap((seg) => toolsToRecords(seg.tools));
}

export function buildSegmentedMessage(
  allSegments: StreamSegment[],
): { content: string; thinking?: string; toolRecords?: ToolActivityRecord[]; segments?: SavedSegment[] } {
  const fullContent = allSegments.map((seg) => seg.content).filter(Boolean).join("\n\n");
  const fullThinking = allSegments.map((seg) => seg.thinking).filter(Boolean).join("\n\n");
  const allToolRecords = segmentsToRecords(allSegments);

  return {
    content: fullContent,
    thinking: fullThinking || undefined,
    toolRecords: allToolRecords.length > 0 ? allToolRecords : undefined,
    segments: allSegments.map((seg) => ({
      thinking: seg.thinking || undefined,
      tools: toolsToRecords(seg.tools),
      content: seg.content,
    })),
  };
}

export interface SavedSegment {
  thinking?: string;
  tools: ToolActivityRecord[];
  content: string;
}

interface ChatMsg {
  role: string;
  content: string;
  images?: string[] | null;
  tool_calls?: unknown[] | null;
  tool_name?: string | null;
  tool_call_id?: string | null;
}

function rebuildArgs(name: string, summary: string): Record<string, string> {
  if (name === "web_search") return { query: summary };
  if (name === "web_fetch") return { url: summary };
  if (name === "bash") return { command: summary };
  if (name === "grep" || name === "glob") return { pattern: summary };
  if (["read_file", "write_file", "edit_file", "list_dir"].includes(name)) return { path: summary };
  return { input: summary };
}

export function expandToolActivities(
  activities: ToolActivityRecord[], content: string,
): ChatMsg[] {
  const msgs: ChatMsg[] = [];
  const toolCalls = activities.map((t, i) => ({
    id: `restored-${i}`,
    function: { name: t.name, arguments: rebuildArgs(t.name, t.summary) },
  }));
  msgs.push({ role: "assistant", content: "", tool_calls: toolCalls });
  for (let i = 0; i < activities.length; i++) {
    const t = activities[i];
    msgs.push({
      role: "tool", content: t.result ?? "",
      tool_name: t.name, tool_call_id: `restored-${i}`,
    });
  }
  if (content) {
    msgs.push({ role: "assistant", content });
  }
  return msgs;
}
