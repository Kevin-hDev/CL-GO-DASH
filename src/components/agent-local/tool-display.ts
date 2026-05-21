import type { TFunction } from "i18next";
import type { RenderableTool } from "./tool-detail-row";

export interface ToolDisplayInfo {
  label: string;
  summary: string;
  additions?: number;
  deletions?: number;
}

const FILE_TOOLS = new Set([
  "read_file", "write_file", "edit_file", "read_spreadsheet", "read_document",
  "read_image", "write_spreadsheet", "write_document", "process_image",
]);

export function toolDisplayInfo(
  tool: RenderableTool,
  projectPath: string | undefined,
  t: TFunction,
): ToolDisplayInfo {
  if (tool.name === "bash") {
    return { label: tool.name, summary: compactCommand(tool.summary) };
  }
  if (tool.name === "web_search" || tool.name === "web_fetch") {
    return { label: tool.name, summary: tool.summary };
  }
  const labelKey = actionKey(tool.name);
  return {
    label: t(`agentLocal.toolActivity.actions.${labelKey}`),
    summary: displaySummary(tool, projectPath),
    ...changeStats(tool),
  };
}

function actionKey(name: string): string {
  if (name === "list_dir") return "list";
  if (name === "grep" || name === "glob") return "search";
  if (name.startsWith("read_")) return "read";
  if (name.startsWith("write_")) return "create";
  if (name === "edit_file" || name === "process_image") return "edit";
  if (name === "create_branch") return "createBranch";
  if (name === "checkout_branch") return "switchBranch";
  return "tool";
}

function displaySummary(tool: RenderableTool, projectPath?: string): string {
  if (!FILE_TOOLS.has(tool.name) && tool.name !== "list_dir") return tool.summary;
  return shortenPath(tool.summary, projectPath);
}

export function shortenPath(path: string, projectPath?: string): string {
  const normalized = path.replace(/\\/g, "/");
  const normalizedProject = projectPath?.replace(/\\/g, "/").replace(/\/+$/, "");
  if (normalizedProject && isInsideProject(normalized, normalizedProject)) {
    const rootName = basename(normalizedProject);
    const relative = normalized.slice(normalizedProject.length).replace(/^\/+/, "");
    return relative ? `${rootName}/${relative}` : rootName;
  }
  const projectsMarker = "/Projects/";
  const markerIndex = normalized.indexOf(projectsMarker);
  if (markerIndex >= 0) return normalized.slice(markerIndex + projectsMarker.length);
  return normalized.replace(/^\/Users\/[^/]+\//, "");
}

function isInsideProject(path: string, projectPath: string): boolean {
  return path === projectPath || path.startsWith(`${projectPath}/`);
}

function basename(path: string): string {
  const parts = path.split("/").filter(Boolean);
  return parts.length > 0 ? parts[parts.length - 1] : path;
}

function changeStats(tool: RenderableTool): Pick<ToolDisplayInfo, "additions" | "deletions"> {
  if (tool.name === "write_file" && tool.content != null) {
    return { additions: countLines(tool.content), deletions: 0 };
  }
  if (tool.name === "edit_file" && tool.old_text != null && tool.new_text != null) {
    return { additions: countLines(tool.new_text), deletions: countLines(tool.old_text) };
  }
  return {};
}

function countLines(text: string): number {
  if (text.length === 0) return 0;
  return text.replace(/\n$/, "").split("\n").length;
}

function compactCommand(command: string): string {
  const firstLine = command.split(/\r?\n/, 1)[0] ?? "";
  const maxLength = 96;
  if (command.includes("\n") || command.includes("\r")) return `${firstLine.slice(0, maxLength)}...`;
  if (firstLine.length > maxLength) return `${firstLine.slice(0, maxLength)}...`;
  return firstLine;
}
