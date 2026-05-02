import type { AgentMessage, ToolActivityRecord } from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";
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

const MAX_FILE_OPERATIONS = 500;

export function collectFileOperations(messages: AgentMessage[]): FileOperation[] {
  const operations: FileOperation[] = [];
  for (const message of messages) {
    const tools = message.segments?.flatMap((segment) => inferSavedToolPaths(segment.tools))
      ?? message.tool_activities
      ?? [];
    inferSavedToolPaths(tools).forEach((tool, index) => {
      const operation = toolToOperation(tool, message.id, index, message.timestamp);
      if (operation) operations.push(operation);
    });
    if (operations.length >= MAX_FILE_OPERATIONS) break;
  }
  return operations.slice(0, MAX_FILE_OPERATIONS);
}

function toolToOperation(
  tool: ToolActivityRecord,
  messageId: string,
  index: number,
  timestamp: string,
): FileOperation | null {
  if (tool.is_error || !tool.summary) return null;
  if (tool.name === "write_file" && tool.content != null) {
    return {
      id: `${messageId}-${index}`,
      path: tool.summary,
      name: fileNameFromPath(tool.summary),
      type: "write",
      timestamp,
      content: tool.content,
      additions: countLines(tool.content),
      deletions: 0,
    };
  }
  if (tool.name === "edit_file" && tool.old_text != null && tool.new_text != null) {
    return {
      id: `${messageId}-${index}`,
      path: tool.summary,
      name: fileNameFromPath(tool.summary),
      type: "edit",
      timestamp,
      oldText: tool.old_text,
      newText: tool.new_text,
      startLine: tool.start_line,
      additions: countLines(tool.new_text),
      deletions: countLines(tool.old_text),
    };
  }
  const OFFICE_WRITE = ["write_spreadsheet", "write_document"];
  if (OFFICE_WRITE.includes(tool.name)) {
    return {
      id: `${messageId}-${index}`,
      path: tool.summary,
      name: fileNameFromPath(tool.summary),
      type: "write",
      timestamp,
      content: tool.content,
      additions: 0,
      deletions: 0,
    };
  }
  return null;
}
