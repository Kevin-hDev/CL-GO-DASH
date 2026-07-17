import type { ToolActivity } from "./agent-chat-utils";

export function applyToolResult(
  tools: ToolActivity[],
  index: number,
  content: string,
  isError: boolean,
  resolvedPath?: string,
  affectedPaths?: string[],
  metadata?: { toolCallId?: string; providerId?: string; source?: string; status?: string; kind?: string },
): ToolActivity[] {
  const next = [...tools];
  const apply = (i: number) => {
    const status = metadata?.status;
    const terminal = !status || ["completed", "failed", "cancelled"].includes(status);
    next[i] = {
      ...next[i],
      ...metadata,
      ...(terminal
        ? { result: content || next[i].partialResult || "", isError, partialResult: undefined }
        : { partialResult: content || next[i].partialResult }),
    };
    if (resolvedPath) next[i].resolvedPath = resolvedPath;
    if (affectedPaths?.length) next[i].affectedPaths = affectedPaths;
  };
  const stableIndex = metadata?.toolCallId
    ? next.findIndex((tool) => tool.toolCallId === metadata.toolCallId)
    : -1;
  if (stableIndex >= 0) {
    apply(stableIndex);
  } else if (index >= 0 && index < next.length && !next[index].result) {
    apply(index);
  } else {
    const pendingIndex = next.findIndex((tool) => !tool.result);
    if (pendingIndex >= 0) apply(pendingIndex);
  }
  return next;
}
