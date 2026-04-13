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
    if (t.name === "shell") summary = String(a.command ?? "");
    else if (t.name === "read_file" || t.name === "write_file") summary = String(a.path ?? "");
    else if (t.name === "edit_file") summary = String(a.path ?? "");
    else if (t.name === "list_dir") summary = String(a.path ?? ".");
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
