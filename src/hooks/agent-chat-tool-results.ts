import type { ToolActivity } from "./agent-chat-utils";

export function applyToolResult(
  tools: ToolActivity[],
  index: number,
  content: string,
  isError: boolean,
  resolvedPath?: string,
  affectedPaths?: string[],
): ToolActivity[] {
  const next = [...tools];
  const apply = (i: number) => {
    next[i] = { ...next[i], result: content, isError };
    if (resolvedPath) next[i].resolvedPath = resolvedPath;
    if (affectedPaths?.length) next[i].affectedPaths = affectedPaths;
  };
  if (index >= 0 && index < next.length && !next[index].result) {
    apply(index);
  } else {
    const pendingIndex = next.findIndex((tool) => !tool.result);
    if (pendingIndex >= 0) apply(pendingIndex);
  }
  return next;
}
