import type { AgentMessage, ToolActivityRecord } from "@/types/agent";
import type { FileOperation, FileOperationGroups } from "@/types/file-preview";
import { inferSavedToolPaths } from "./tool-file-path";

const FILE_SEPARATOR = /[/\\]/;

export function fileNameFromPath(path: string): string {
  return path.split(FILE_SEPARATOR).filter(Boolean).pop() ?? path;
}

export function shortPath(path: string, baseDir?: string): string {
  if (!baseDir) return path;
  const normalizedPath = path.replaceAll("\\", "/");
  const normalizedBase = baseDir.replaceAll("\\", "/").replace(/\/$/, "");
  return normalizedPath.startsWith(`${normalizedBase}/`)
    ? normalizedPath.slice(normalizedBase.length + 1)
    : path;
}

export function countLines(text?: string): number {
  if (!text) return 0;
  return text.split(/\r?\n/).length;
}

export const MAX_FILE_OPERATIONS = 500;

interface CollectFileOperationsOptions {
  liveTools?: ToolActivityRecord[];
  baseDir?: string;
}

export function normalizeFileOperationPath(path: string): string {
  return path.replaceAll("\\", "/").replace(/\/+$/, "");
}

export function fileOperationKey(path: string, baseDir?: string): string {
  const normalizedPath = normalizeFileOperationPath(path);
  if (!baseDir || isAbsolutePath(normalizedPath)) return normalizedPath;
  const normalizedBase = normalizeFileOperationPath(baseDir);
  return `${normalizedBase}/${normalizedPath}`.replace(/\/+/g, "/");
}

export function collectFileOperations(
  messages: AgentMessage[],
  options: CollectFileOperationsOptions = {},
): FileOperation[] {
  return collectFileOperationGroups(messages, options).all;
}

export function collectFileOperationGroups(
  messages: AgentMessage[],
  options: CollectFileOperationsOptions = {},
): FileOperationGroups {
  const byPath = new Map<string, FileOperation>();
  if (options.liveTools?.length) {
    appendLatestToolOperations(
      byPath,
      options.liveTools,
      "live",
      new Date().toISOString(),
      options.baseDir,
    );
  }

  const latestLive = collectLatestToolOperations(
    options.liveTools ?? [],
    "live",
    new Date().toISOString(),
    options.baseDir,
  );

  let latest = latestLive;
  for (let i = messages.length - 1; i >= 0 && byPath.size < MAX_FILE_OPERATIONS; i--) {
    if (latest.length === 0) {
      latest = collectLatestToolOperations(
        toolsFromMessage(messages[i]),
        messages[i].id,
        messages[i].timestamp,
        options.baseDir,
      );
    }
    appendLatestToolOperations(
      byPath,
      toolsFromMessage(messages[i]),
      messages[i].id,
      messages[i].timestamp,
      options.baseDir,
    );
  }

  return { all: Array.from(byPath.values()), latest };
}

function toolsFromMessage(message: AgentMessage): ToolActivityRecord[] {
  const segmentTools = message.segments
    ?.flatMap((segment) => inferSavedToolPaths(segment.tools))
    ?? [];
  return segmentTools.length > 0 ? segmentTools : message.tool_activities ?? [];
}

function appendLatestToolOperations(
  byPath: Map<string, FileOperation>,
  tools: ToolActivityRecord[],
  messageId: string,
  timestamp: string,
  baseDir: string | undefined,
) {
  const inferred = inferSavedToolPaths(tools);
  for (let index = inferred.length - 1; index >= 0; index--) {
    if (byPath.size >= MAX_FILE_OPERATIONS) return;
    const tool = inferred[index];
    const operations = toolToOperations(tool, messageId, index, timestamp);
    for (const operation of operations) {
      if (byPath.size >= MAX_FILE_OPERATIONS) return;
      const key = fileOperationKey(operation.path, baseDir);
      if (!key || byPath.has(key)) continue;
      byPath.set(key, { ...operation, id: `file:${key}` });
    }
  }
}

function collectLatestToolOperations(
  tools: ToolActivityRecord[],
  messageId: string,
  timestamp: string,
  baseDir: string | undefined,
): FileOperation[] {
  const byPath = new Map<string, FileOperation>();
  appendLatestToolOperations(byPath, tools, messageId, timestamp, baseDir);
  return Array.from(byPath.values());
}

function isAbsolutePath(path: string): boolean {
  return path.startsWith("/") || /^[A-Za-z]:\//.test(path);
}

function toolToOperations(
  tool: ToolActivityRecord,
  messageId: string,
  index: number,
  timestamp: string,
): FileOperation[] {
  if (tool.is_error) return [];
  if (tool.name === "bash" && tool.affected_paths?.length) {
    return tool.affected_paths
      .filter((path) => path.trim())
      .map((path, pathIndex) => ({
        id: `${messageId}-${index}-${pathIndex}`,
        path,
        name: fileNameFromPath(path),
        type: "write",
        timestamp,
        additions: 0,
        deletions: 0,
      }));
  }

  const path = tool.resolved_path?.trim() || tool.summary.trim();
  if (!path) return [];
  if (tool.name === "write_file" && tool.content != null) {
    return [{
      id: `${messageId}-${index}`,
      path,
      name: fileNameFromPath(path),
      type: "write",
      timestamp,
      content: tool.content,
      additions: countLines(tool.content),
      deletions: 0,
    }];
  }
  if (tool.name === "edit_file" && tool.old_text != null && tool.new_text != null) {
    return [{
      id: `${messageId}-${index}`,
      path,
      name: fileNameFromPath(path),
      type: "edit",
      timestamp,
      oldText: tool.old_text,
      newText: tool.new_text,
      startLine: tool.start_line,
      additions: countLines(tool.new_text),
      deletions: countLines(tool.old_text),
    }];
  }
  const OFFICE_WRITE = ["write_spreadsheet", "write_document"];
  if (OFFICE_WRITE.includes(tool.name)) {
    return [{
      id: `${messageId}-${index}`,
      path,
      name: fileNameFromPath(path),
      type: "write",
      timestamp,
      content: tool.content,
      additions: 0,
      deletions: 0,
    }];
  }
  return [];
}
