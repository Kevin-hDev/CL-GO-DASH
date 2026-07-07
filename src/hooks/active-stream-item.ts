import type { ToolActivity } from "./agent-chat-utils";

export type ActiveStreamItem =
  | { kind: "thinking" }
  | { kind: "tool"; toolIndex: number }
  | null;

export function thinkingItem(): ActiveStreamItem {
  return { kind: "thinking" };
}

export function toolItem(toolIndex: number): ActiveStreamItem {
  return { kind: "tool", toolIndex };
}

export function lastPendingToolItem(tools: ToolActivity[]): ActiveStreamItem {
  for (let i = tools.length - 1; i >= 0; i -= 1) {
    if (tools[i].result === undefined && tools[i].isError === undefined) {
      return toolItem(i);
    }
  }
  return null;
}

export function activeItemAfterToolResult(tools: ToolActivity[], finishedToolIndex: number): ActiveStreamItem {
  const pending = lastPendingToolItem(tools);
  if (pending) return pending;
  if (finishedToolIndex >= 0 && finishedToolIndex < tools.length) return toolItem(finishedToolIndex);
  return null;
}

export function isActiveTool(item: ActiveStreamItem, toolIndex?: number): boolean {
  return item?.kind === "tool" && toolIndex === item.toolIndex;
}
