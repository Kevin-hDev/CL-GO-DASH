import type { ToolActivity } from "./agent-chat-utils";

export type ActiveStreamItem =
  | { kind: "thinking" }
  | { kind: "tools"; toolIndices: number[] }
  | null;

export function thinkingItem(): ActiveStreamItem {
  return { kind: "thinking" };
}

export function toolItems(toolIndices: number[]): ActiveStreamItem {
  return toolIndices.length > 0 ? { kind: "tools", toolIndices } : null;
}

export function pendingToolIndices(tools: ToolActivity[]): number[] {
  const indices: number[] = [];
  for (let i = 0; i < tools.length; i += 1) {
    if (tools[i].result === undefined && tools[i].isError === undefined) {
      indices.push(i);
    }
  }
  return indices;
}

export function lastPendingToolItem(tools: ToolActivity[]): ActiveStreamItem {
  for (let i = tools.length - 1; i >= 0; i -= 1) {
    if (tools[i].result === undefined && tools[i].isError === undefined) {
      return toolItems([i]);
    }
  }
  return null;
}

export function activeItemAfterToolResult(tools: ToolActivity[], finishedToolIndex: number): ActiveStreamItem {
  const pending = pendingToolIndices(tools);
  if (pending.length > 0) return toolItems(pending);
  if (finishedToolIndex >= 0 && finishedToolIndex < tools.length) return toolItems([finishedToolIndex]);
  return null;
}

export function isActiveTool(item: ActiveStreamItem, toolIndex?: number): boolean {
  return item?.kind === "tools" && toolIndex !== undefined && item.toolIndices.includes(toolIndex);
}
