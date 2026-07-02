import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { collectFileOperations, normalizeFileOperationPath } from "@/lib/file-preview-utils";
import { checkPreviewFilesExist } from "@/services/file-preview";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { toolsToRecords, type StreamSegment, type ToolActivity } from "./agent-chat-utils";
import type { AgentMessage } from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";

export function useSessionFiles(
  messages: AgentMessage[],
  completedSegments: StreamSegment[] = [],
  currentTools: ToolActivity[] = [],
  baseDir?: string,
) {
  const liveTools = useMemo(
    () => toolsToRecords([
      ...completedSegments.flatMap((segment) => segment.tools),
      ...currentTools,
    ].filter(isCompletedTool)),
    [completedSegments, currentTools],
  );
  const operations = useMemo(
    () => collectFileOperations(messages, { liveTools, baseDir }),
    [messages, liveTools, baseDir],
  );
  return useExistingFileOperations(operations, baseDir, liveActivityKey(completedSegments, currentTools));
}

function useExistingFileOperations(
  operations: FileOperation[],
  baseDir: string | undefined,
  activityKey: string,
) {
  const [existence, setExistence] = useState<{ key: string; values: Map<string, boolean> }>({
    key: "",
    values: new Map(),
  });
  const requestRef = useRef(0);
  const paths = useMemo(() => operations.map((operation) => operation.path), [operations]);
  const pathListKey = useMemo(() => paths.join("\0"), [paths]);
  const existenceKey = useMemo(() => `${baseDir ?? ""}\0${pathListKey}`, [baseDir, pathListKey]);
  const pathsForCheck = useMemo(() => (
    pathListKey ? pathListKey.split("\0") : []
  ), [pathListKey]);

  const refreshExists = useCallback(() => {
    const requestId = ++requestRef.current;
    if (pathsForCheck.length === 0) {
      return;
    }
    const visibleKeys = new Set(pathsForCheck.map(normalizeFileOperationPath));
    checkPreviewFilesExist(pathsForCheck, baseDir)
      .then((results) => {
        if (requestId !== requestRef.current) return;
        setExistence(() => {
          const next = new Map<string, boolean>();
          for (const result of results) {
            next.set(normalizeFileOperationPath(result.path), result.exists);
          }
          for (const key of visibleKeys) {
            if (!next.has(key)) next.set(key, false);
          }
          return { key: existenceKey, values: next };
        });
      })
      .catch(() => {});
  }, [pathsForCheck, baseDir, existenceKey]);

  useEffect(() => {
    refreshExists();
    return () => { requestRef.current += 1; };
  }, [refreshExists, activityKey]);

  useEffect(() => {
    const unlisten = listen("file-tree-changed", () => refreshExists());
    return () => { cleanupTauriListener(unlisten); };
  }, [refreshExists]);

  return useMemo(
    () => operations.filter((operation) => (
      existence.key === existenceKey
        && existence.values.get(normalizeFileOperationPath(operation.path)) === true
    )),
    [operations, existence, existenceKey],
  );
}

function isCompletedTool(tool: ToolActivity): boolean {
  return tool.result !== undefined && !tool.isError;
}

function liveActivityKey(completedSegments: StreamSegment[], currentTools: ToolActivity[]): string {
  const completed = completedSegments
    .map((segment) => segment.tools.map(toolStateKey).join(","))
    .join("|");
  return `${completed}::${currentTools.map(toolStateKey).join(",")}`;
}

function toolStateKey(tool: ToolActivity): string {
  return `${tool.name}:${tool.result === undefined ? "pending" : "done"}:${tool.isError ? "err" : "ok"}`;
}
